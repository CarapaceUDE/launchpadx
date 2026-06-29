use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::Serialize;
use serde_json::Value;

const DISCOVERY_DIR: &str = ".codex-launchpad/discovery";
const DISCOVERY_FILE: &str = "rate-limit.jsonl";
const APP_LOG_PREFIX: &str = "RATE_LIMIT_WATCH";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryRecord {
    pub event: String,
    pub at: String,
    #[serde(flatten)]
    pub details: Value,
}

pub fn discovery_paths(root: Option<&Path>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(root) = root {
        paths.push(root.join("rate-limit-discovery.jsonl"));
    }
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(DISCOVERY_DIR).join(DISCOVERY_FILE));
    }
    paths
}

pub fn log_event(root: Option<&Path>, level: &str, event: &str, details: Value) {
    let record = DiscoveryRecord {
        event: event.to_string(),
        at: Utc::now().to_rfc3339(),
        details,
    };

    let line = match serde_json::to_string(&record) {
        Ok(line) => line,
        Err(error) => {
            eprintln!("[{APP_LOG_PREFIX}] failed to serialize discovery record: {error}");
            return;
        }
    };

    for path in discovery_paths(root) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(file, "{line}");
        }
    }

    if let Some(root) = root {
        let app_log = root.join("app.log");
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(app_log) {
            let _ = writeln!(file, "[{level}] [{APP_LOG_PREFIX}] {event} {line}");
        }
    }

    eprintln!("[{level}] [{APP_LOG_PREFIX}] {event} {line}");
}

pub fn interesting_observation_keywords() -> &'static [&'static str] {
    &[
        "error",
        "limit",
        "quota",
        "exhaust",
        "429",
        "message",
        "upgrade",
        "subscription",
        "capacity",
        "blocked",
        "denied",
        "unavailable",
        "out of",
    ]
}

pub fn looks_interesting(text: &str) -> bool {
    let lower = text.to_lowercase();
    interesting_observation_keywords()
        .iter()
        .any(|keyword| lower.contains(keyword))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_quota_like_text() {
        assert!(looks_interesting("You are out of messages for this cycle"));
        assert!(!looks_interesting("Applied patch to src/main.rs"));
    }
}