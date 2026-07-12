use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::LauncherConfig;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProviderModeKind {
    CodexAccount,
    LocalApi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCheckpoint {
    pub id: String,
    pub captured_at: DateTime<Utc>,
    pub thread_id: Option<String>,
    pub session_id: Option<String>,
    pub working_directory: Option<String>,
    pub provider_mode: ProviderModeKind,
    pub model: Option<String>,
    pub active_goal: Option<String>,
    pub last_user_message: Option<String>,
    pub last_assistant_summary: Option<String>,
    pub git_branch: Option<String>,
    pub trigger: String,
    pub resume_prompt: String,
}

impl SessionCheckpoint {
    pub fn build_resume_prompt(&self) -> String {
        let mut parts = vec![
            "Continue the work from before the provider switch. Preserve context and pick up where we left off.".to_string(),
        ];

        if let Some(goal) = &self.active_goal {
            parts.push(format!("Active goal: {goal}"));
        }
        if let Some(msg) = &self.last_user_message {
            parts.push(format!("Last user message: {msg}"));
        }
        if let Some(summary) = &self.last_assistant_summary {
            parts.push(format!("Last assistant state: {summary}"));
        }
        if let Some(dir) = &self.working_directory {
            parts.push(format!("Working directory: {dir}"));
        }
        if let Some(branch) = &self.git_branch {
            parts.push(format!("Git branch: {branch}"));
        }
        if let Some(model) = &self.model {
            parts.push(format!("Previous model: {model}"));
        }

        parts.join("\n\n")
    }
}

pub fn checkpoints_dir() -> Result<PathBuf, std::io::Error> {
    let home = dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "home directory not found")
    })?;
    let dir = home.join(".launchpadx").join("checkpoints");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn save_checkpoint(checkpoint: &SessionCheckpoint) -> Result<PathBuf, std::io::Error> {
    let dir = checkpoints_dir()?;
    let path = dir.join(format!("{}.json", checkpoint.id));
    let data = serde_json::to_string_pretty(checkpoint)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    fs::write(&path, data)?;
    Ok(path)
}

pub fn list_checkpoints() -> Result<Vec<SessionCheckpoint>, std::io::Error> {
    let dir = checkpoints_dir()?;
    let mut checkpoints = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let data = fs::read_to_string(&path)?;
        if let Ok(mut checkpoint) = serde_json::from_str::<SessionCheckpoint>(&data) {
            if checkpoint.resume_prompt.is_empty() {
                checkpoint.resume_prompt = checkpoint.build_resume_prompt();
            }
            checkpoints.push(checkpoint);
        }
    }

    checkpoints.sort_by_key(|checkpoint| std::cmp::Reverse(checkpoint.captured_at));
    Ok(checkpoints)
}

pub fn load_checkpoint(id: &str) -> Result<Option<SessionCheckpoint>, std::io::Error> {
    let path = checkpoints_dir()?.join(format!("{id}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(path)?;
    let mut checkpoint: SessionCheckpoint = serde_json::from_str(&data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    if checkpoint.resume_prompt.is_empty() {
        checkpoint.resume_prompt = checkpoint.build_resume_prompt();
    }
    Ok(Some(checkpoint))
}

pub fn delete_checkpoint(id: &str) -> Result<(), std::io::Error> {
    let path = checkpoints_dir()?.join(format!("{id}.json"));
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn git_branch_for(working_directory: &Path) -> Option<String> {
    let output = crate::process_util::command("git")
        .args(["branch", "--show-current"])
        .current_dir(working_directory)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        None
    } else {
        Some(branch)
    }
}

pub fn detect_goal_from_text(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("/goal") {
            return Some(trimmed.to_string());
        }
        let lower = trimmed.to_lowercase();
        if lower.contains("goal mode") || lower.contains("running goal") {
            return Some(trimmed.to_string());
        }
    }
    None
}

pub fn provider_mode_from_config(config: &LauncherConfig) -> ProviderModeKind {
    if crate::lpad_config::inspect(config)
        .map(|inspection| inspection.managed_by_launcher)
        .unwrap_or(false)
    {
        ProviderModeKind::LocalApi
    } else {
        ProviderModeKind::CodexAccount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_resume_prompt_with_goal() {
        let checkpoint = SessionCheckpoint {
            id: "test".to_string(),
            captured_at: Utc::now(),
            thread_id: None,
            session_id: Some("sess_1".to_string()),
            working_directory: Some("/tmp/project".to_string()),
            provider_mode: ProviderModeKind::CodexAccount,
            model: Some("gpt-5".to_string()),
            active_goal: Some("/goal fix the auth bug".to_string()),
            last_user_message: Some("keep going".to_string()),
            last_assistant_summary: Some("updated login handler".to_string()),
            git_branch: Some("feature/auth".to_string()),
            trigger: "manual".to_string(),
            resume_prompt: String::new(),
        };

        let prompt = checkpoint.build_resume_prompt();
        assert!(prompt.contains("/goal fix the auth bug"));
        assert!(prompt.contains("feature/auth"));
    }

    #[test]
    fn detects_goal_prefix() {
        assert_eq!(
            detect_goal_from_text("  /goal ship failover\nnext line"),
            Some("/goal ship failover".to_string())
        );
    }
}
