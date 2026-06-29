use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::connection_watch;
use crate::rate_limit_watch;

pub const DEFAULT_LIMIT: usize = 50;

const NOISE_EVENTS: &[&str] = &[
    "watch_heartbeat",
    "endpoint_heartbeat",
    "app_server_rate_limits_poll",
];

fn is_noise_event(event: &str) -> bool {
    NOISE_EVENTS.contains(&event)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryLogEntry {
    pub stream: String,
    pub source: String,
    pub at: String,
    pub event: String,
    pub details: Value,
}

pub fn tail_discovery_logs(root: &Path, limit: usize) -> Vec<DiscoveryLogEntry> {
    let limit = limit.clamp(1, 200);
    let mut entries = Vec::new();

    for path in rate_limit_watch::discovery_paths(Some(root)) {
        ingest_jsonl(&mut entries, "rateLimit", &path);
    }
    for path in connection_watch::discovery_paths(Some(root)) {
        ingest_jsonl(&mut entries, "connection", &path);
    }

    entries.sort_by_key(|entry| std::cmp::Reverse(parse_at(&entry.at)));
    entries.truncate(limit);
    entries
}

fn ingest_jsonl(entries: &mut Vec<DiscoveryLogEntry>, stream: &str, path: &PathBuf) {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return,
    };

    let reader = BufReader::new(file);
    let source = path.display().to_string();

    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: Value = match serde_json::from_str(trimmed) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let event = value
            .get("event")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        if is_noise_event(&event) {
            continue;
        }
        let at = value
            .get("at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let mut details = value.clone();
        if let Some(obj) = details.as_object_mut() {
            obj.remove("event");
            obj.remove("at");
        }

        entries.push(DiscoveryLogEntry {
            stream: stream.to_string(),
            source: source.clone(),
            at,
            event,
            details,
        });
    }
}

fn parse_at(at: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| DateTime::<Utc>::MIN_UTC)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn tails_and_sorts_discovery_entries() {
        let dir = std::env::temp_dir().join(format!("codex-launchpad-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tmpdir");

        let path = dir.join("rate-limit-discovery.jsonl");
        let mut file = std::fs::File::create(&path).expect("create");
        writeln!(
            file,
            r#"{{"event":"older","at":"2026-01-01T00:00:00Z","note":"a"}}"#
        )
        .expect("write");
        writeln!(
            file,
            r#"{{"event":"newer","at":"2026-06-01T12:00:00Z","note":"b"}}"#
        )
        .expect("write");
        file.sync_all().expect("sync");

        let mut entries = Vec::new();
        ingest_jsonl(&mut entries, "rateLimit", &path);
        entries.sort_by_key(|entry| std::cmp::Reverse(parse_at(&entry.at)));
        let newer_idx = entries.iter().position(|entry| entry.event == "newer");
        let older_idx = entries.iter().position(|entry| entry.event == "older");
        assert_eq!(entries.len(), 2);
        assert!(newer_idx.unwrap() < older_idx.unwrap());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn skips_noise_discovery_events() {
        assert!(is_noise_event("watch_heartbeat"));
        assert!(is_noise_event("endpoint_heartbeat"));
        assert!(!is_noise_event("rate_limit_structured"));
    }
}