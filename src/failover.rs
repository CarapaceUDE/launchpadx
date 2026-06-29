use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use crate::codex_app_server::{self, CodexRateLimitsStatus};
use crate::config::{FailoverSettings, LauncherConfig, ProfileOverlay};
use crate::session_checkpoint::{self, SessionCheckpoint};

pub const APP_SERVER_RATE_LIMIT_SOURCE: &str = "app_server_rate_limits";

pub fn failover_settings(config: &LauncherConfig) -> FailoverSettings {
    config.failover.clone().unwrap_or_default()
}

pub fn profile_overlays(config: &LauncherConfig) -> HashMap<String, ProfileOverlay> {
    config.profiles.clone().unwrap_or_default()
}

pub fn matches_rate_limit(text: &str, patterns: &[String]) -> Option<String> {
    let lower = text.to_lowercase();
    patterns
        .iter()
        .find(|pattern| lower.contains(&pattern.to_lowercase()))
        .cloned()
}

pub fn rate_limit_reached_from_status(status: &CodexRateLimitsStatus) -> Option<String> {
    codex_app_server::rate_limit_reached_type(status)
}

pub fn should_failover_for_rate_limits(status: &CodexRateLimitsStatus) -> bool {
    status.ok && codex_app_server::is_rate_limit_reached(status)
}

pub fn local_overlay_from_config(config: &LauncherConfig) -> ProfileOverlay {
    ProfileOverlay {
        openai_base_url: config.openai_base_url.clone(),
        ollama_ip: config.ollama_ip.clone(),
        ollama_port: config.ollama_port,
        ollama_scheme: config.ollama_scheme.clone(),
        api_key: config.api_key.clone(),
        codex_model: config.codex_model.clone(),
        codex_provider_id: Some(config.codex_provider_id()),
        codex_provider_name: Some(config.codex_provider_name()),
        codex_api_key_mode: Some(config.codex_api_key_mode()),
        working_directory: config.working_directory.clone(),
    }
}

pub fn resolve_fallback_profile(
    config: &LauncherConfig,
    profile_name: Option<&str>,
) -> Result<(String, ProfileOverlay), Box<dyn Error>> {
    let settings = failover_settings(config);
    let profiles = profile_overlays(config);

    if let Some(explicit) = profile_name {
        if let Some(overlay) = profiles.get(explicit).cloned() {
            return Ok((explicit.to_string(), overlay));
        }
        return Err(format!("Failover profile '{explicit}' was not found").into());
    }

    if let Some(name) = settings.fallback_chain.first() {
        if let Some(overlay) = profiles.get(name).cloned() {
            return Ok((name.clone(), overlay));
        }
    }

    if config.ollama_ip.as_ref().is_some_and(|value| !value.trim().is_empty())
        || config
            .openai_base_url
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
    {
        return Ok(("launcher-local".to_string(), local_overlay_from_config(config)));
    }

    Err("No local endpoint is configured in this launcher's settings yet.".into())
}

pub fn apply_profile_overlay(
    base: &LauncherConfig,
    overlay: &ProfileOverlay,
) -> LauncherConfig {
    let mut next = base.clone();
    overlay.apply_to(&mut next);
    next
}

pub struct FailoverResult {
    pub message: String,
    pub profile_name: String,
    pub checkpoint: Option<SessionCheckpoint>,
    pub resume_prompt: Option<String>,
}

pub fn run_manual_failover(
    config: &LauncherConfig,
    root: &Path,
    pid_file: &Path,
    profile_name: Option<&str>,
    trigger: &str,
) -> Result<FailoverResult, Box<dyn Error>> {
    let (profile_name, overlay) = resolve_fallback_profile(config, profile_name)?;

    let checkpoint = capture_checkpoint_from_running(config, root, trigger)?;
    let resume_prompt = checkpoint.as_ref().map(|c| c.resume_prompt.clone());

    let mut next_config = apply_profile_overlay(config, &overlay);
    next_config.failover = config.failover.clone();
    next_config.profiles = config.profiles.clone();

    if crate::app_logic::detect_codex_process(config, root).running {
        crate::app_logic::stop_codex(config, root, pid_file)?;
    }

    crate::app_logic::write_config_for_launch(&next_config)?;
    let launch_message = crate::app_logic::launch(&next_config, root, pid_file)?;

    Ok(FailoverResult {
        message: format!(
            "Failover complete via profile '{profile_name}'. {launch_message} Paste the resume prompt into Codex to continue."
        ),
        profile_name,
        checkpoint,
        resume_prompt,
    })
}

pub fn capture_checkpoint_from_running(
    config: &LauncherConfig,
    root: &Path,
    trigger: &str,
) -> Result<Option<SessionCheckpoint>, Box<dyn Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let client = crate::acp_client::AcpClient::from_config(config)?;
    let sessions = runtime.block_on(client.list_sessions())?;
    let session = sessions
        .sessions
        .last()
        .or_else(|| sessions.sessions.first());

    let (session_id, last_user_message, last_assistant_summary) = if let Some(session) = session {
        let response = runtime
            .block_on(client.get_response(&session.session_id))
            .ok();
        let assistant = response.as_ref().map(|msg| msg.content.clone());
        (
            Some(session.session_id.clone()),
            None,
            assistant,
        )
    } else {
        (None, None, None)
    };

    let working_directory = config.working_directory(root).ok();
    let git_branch = working_directory
        .as_ref()
        .and_then(|dir| session_checkpoint::git_branch_for(dir));

    let combined_text = [
        last_user_message.as_deref().unwrap_or(""),
        last_assistant_summary.as_deref().unwrap_or(""),
    ]
    .join("\n");
    let active_goal = session_checkpoint::detect_goal_from_text(&combined_text);

    let id = format!("chk_{}", chrono::Utc::now().timestamp_millis());
    let mut checkpoint = SessionCheckpoint {
        id: id.clone(),
        captured_at: chrono::Utc::now(),
        thread_id: None,
        session_id,
        working_directory: working_directory.map(|path| path.display().to_string()),
        provider_mode: session_checkpoint::provider_mode_from_config(config),
        model: config.codex_model(),
        active_goal,
        last_user_message,
        last_assistant_summary,
        git_branch,
        trigger: trigger.to_string(),
        resume_prompt: String::new(),
    };
    checkpoint.resume_prompt = checkpoint.build_resume_prompt();
    session_checkpoint::save_checkpoint(&checkpoint)?;

    Ok(Some(checkpoint))
}

pub fn probe_codex_api(config: &LauncherConfig) -> serde_json::Value {
    let base_url = config.codex_api_base_url();
    let health_url = format!("{base_url}/health");
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build();

    let health_ok = client
        .as_ref()
        .ok()
        .and_then(|c| c.get(&health_url).send().ok())
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    serde_json::json!({
        "codexApiBaseUrl": base_url,
        "healthOk": health_ok,
        "restSessionsSupported": health_ok,
        "appServerWebSocketUrl": format!("ws://127.0.0.1:{}/", config.codex_api_port()),
        "notes": [
            "REST /sessions is wired for checkpoint capture and legacy text discovery logging.",
            "Auto-failover uses account/rateLimits/read rateLimitReachedType from codex app-server.",
            "Session text matches are logged for discovery but do not trigger auto-switch."
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_rate_limit_patterns;

    #[test]
    fn matches_configured_patterns() {
        let patterns = default_rate_limit_patterns();
        assert_eq!(
            matches_rate_limit("You are out of messages for today", &patterns),
            Some("out of messages".to_string())
        );
        assert!(matches_rate_limit("everything is fine", &patterns).is_none());
    }

    #[test]
    fn structured_rate_limit_requires_reached_type() {
        let limited = CodexRateLimitsStatus {
            ok: true,
            fetched_at: "now".to_string(),
            source: "test".to_string(),
            codex_cli: None,
            error: None,
            requires_auth: Some(false),
            plan_type: Some("plus".to_string()),
            rate_limits: Some(codex_app_server::CodexRateLimits {
                limit_id: Some("codex".to_string()),
                limit_name: None,
                primary: Some(codex_app_server::RateLimitWindow {
                    used_percent: Some(100),
                    window_duration_mins: Some(300),
                    resets_at: Some(1),
                }),
                secondary: None,
                credits: None,
                plan_type: Some("plus".to_string()),
                rate_limit_reached_type: Some("primary".to_string()),
            }),
            rate_limit_reset_credits: None,
        };

        assert!(should_failover_for_rate_limits(&limited));
        assert_eq!(
            rate_limit_reached_from_status(&limited).as_deref(),
            Some("primary")
        );

        let exhausted_without_signal = CodexRateLimitsStatus {
            rate_limits: Some(codex_app_server::CodexRateLimits {
                rate_limit_reached_type: None,
                primary: Some(codex_app_server::RateLimitWindow {
                    used_percent: Some(100),
                    window_duration_mins: Some(300),
                    resets_at: Some(1),
                }),
                ..Default::default()
            }),
            ..limited.clone()
        };
        assert!(!should_failover_for_rate_limits(&exhausted_without_signal));
    }
}