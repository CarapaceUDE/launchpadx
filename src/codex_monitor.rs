use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;

use crate::config::LauncherConfig;
use crate::failover;
use crate::rate_limit_watch;
use crate::session_checkpoint::{self, ProviderModeKind, SessionCheckpoint};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitAlert {
    pub detected_at: DateTime<Utc>,
    pub matched_pattern: String,
    pub source: String,
    pub session_id: Option<String>,
    pub snippet: String,
    pub dismissed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexMonitorStatus {
    pub watching: bool,
    pub auto_switch: bool,
    pub last_poll_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub active_alert: Option<RateLimitAlert>,
    pub recent_alerts: Vec<RateLimitAlert>,
    pub last_checkpoint: Option<SessionCheckpoint>,
    pub discovery_log_hint: String,
}

#[derive(Debug)]
pub struct CodexMonitor {
    config_path: PathBuf,
    root: PathBuf,
    status: CodexMonitorStatus,
    poll_count: u64,
    last_session_fingerprints: HashMap<String, String>,
}

impl CodexMonitor {
    pub fn new(config_path: PathBuf, root: PathBuf) -> Self {
        Self {
            config_path,
            root,
            status: CodexMonitorStatus {
                watching: true,
                auto_switch: false,
                last_poll_at: None,
                last_error: None,
                active_alert: None,
                recent_alerts: Vec::new(),
                last_checkpoint: None,
                discovery_log_hint: discovery_hint(),
            },
            poll_count: 0,
            last_session_fingerprints: HashMap::new(),
        }
    }

    pub fn status(&self) -> CodexMonitorStatus {
        self.status.clone()
    }

    pub fn dismiss_alert(&mut self) {
        if let Some(alert) = self.status.active_alert.as_mut() {
            alert.dismissed = true;
        }
        self.status.active_alert = None;
    }

    pub fn set_last_checkpoint(&mut self, checkpoint: SessionCheckpoint) {
        self.status.last_checkpoint = Some(checkpoint);
    }

    pub fn poll_once(&mut self) {
        self.poll_count += 1;

        let config = match LauncherConfig::read(&self.config_path) {
            Ok(config) => config,
            Err(error) => {
                self.status.last_error = Some(error.to_string());
                rate_limit_watch::log_event(
                    Some(self.root.as_path()),
                    "ERROR",
                    "config_read_failed",
                    json!({ "error": error.to_string() }),
                );
                return;
            }
        };

        let settings = failover::failover_settings(&config);
        self.status.auto_switch = settings.auto_switch;
        self.status.watching = true;
        self.status.last_poll_at = Some(Utc::now());
        self.status.discovery_log_hint = discovery_hint();

        if session_checkpoint::provider_mode_from_config(&config) == ProviderModeKind::LocalApi {
            self.status.watching = false;
            return;
        }

        let process = crate::app_logic::detect_codex_process(&config, &self.root);
        let api_ready = process.running && crate::app_logic::codex_api_ready(&config);

        if self.poll_count.is_multiple_of(6) {
            rate_limit_watch::log_event(
                Some(self.root.as_path()),
                "INFO",
                "watch_heartbeat",
                json!({
                    "providerMode": "codexAccount",
                    "codexRunning": process.running,
                    "codexApiReady": api_ready,
                    "codexPid": process.pid,
                    "codexDetectMethod": process.method,
                    "pollCount": self.poll_count,
                    "codexApiBaseUrl": config.codex_api_base_url(),
                    "codexApiProbe": failover::probe_codex_api(&config),
                }),
            );
        }

        if !process.running || !api_ready {
            return;
        }

        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                self.status.last_error = Some(error.to_string());
                rate_limit_watch::log_event(
                    Some(self.root.as_path()),
                    "ERROR",
                    "runtime_init_failed",
                    json!({ "error": error.to_string() }),
                );
                return;
            }
        };

        let client = match crate::acp_client::AcpClient::from_config(&config) {
            Ok(client) => client,
            Err(error) => {
                self.status.last_error = Some(error.to_string());
                rate_limit_watch::log_event(
                    Some(self.root.as_path()),
                    "WARN",
                    "acp_client_unavailable",
                    json!({
                        "error": error.to_string(),
                        "codexApiBaseUrl": config.codex_api_base_url(),
                    }),
                );
                return;
            }
        };

        let sessions = match runtime.block_on(client.list_sessions()) {
            Ok(sessions) => sessions,
            Err(error) => {
                self.status.last_error = Some(error.to_string());
                rate_limit_watch::log_event(
                    Some(self.root.as_path()),
                    "WARN",
                    "list_sessions_failed",
                    json!({
                        "error": error.to_string(),
                        "codexApiBaseUrl": config.codex_api_base_url(),
                    }),
                );
                return;
            }
        };

        self.status.last_error = None;

        if sessions.sessions.is_empty() && self.poll_count.is_multiple_of(12) {
            rate_limit_watch::log_event(
                Some(self.root.as_path()),
                "INFO",
                "no_active_sessions",
                json!({
                    "codexApiBaseUrl": config.codex_api_base_url(),
                    "note": "Codex is up but REST /sessions returned empty; desktop UI state may not be mirrored here yet."
                }),
            );
        }

        for session in &sessions.sessions {
            let response = match runtime.block_on(client.get_response(&session.session_id)) {
                Ok(response) => response,
                Err(error) => {
                    rate_limit_watch::log_event(
                        Some(self.root.as_path()),
                        "WARN",
                        "get_response_failed",
                        json!({
                            "sessionId": session.session_id,
                            "error": error.to_string(),
                        }),
                    );
                    continue;
                }
            };

            let fingerprint = format!(
                "{}:{}:{}",
                response.role,
                response.done,
                truncate(&response.content, 512)
            );
            let changed = self
                .last_session_fingerprints
                .get(&session.session_id)
                .is_none_or(|previous| previous != &fingerprint);
            if changed {
                self.last_session_fingerprints
                    .insert(session.session_id.clone(), fingerprint);
                if rate_limit_watch::looks_interesting(&response.content) {
                    rate_limit_watch::log_event(
                        Some(self.root.as_path()),
                        "WARN",
                        "interesting_session_response",
                        json!({
                            "sessionId": session.session_id,
                            "createdAt": session.created_at,
                            "role": response.role,
                            "done": response.done,
                            "contentPreview": truncate(&response.content, 4000),
                            "contentLength": response.content.len(),
                            "patternsChecked": settings.rate_limit_patterns,
                        }),
                    );
                }
            }

            if let Some(matched) =
                failover::matches_rate_limit(&response.content, &settings.rate_limit_patterns)
            {
                let snippet = truncate(&response.content, 500);
                let alert = RateLimitAlert {
                    detected_at: Utc::now(),
                    matched_pattern: matched.clone(),
                    source: "session_response".to_string(),
                    session_id: Some(session.session_id.clone()),
                    snippet: snippet.clone(),
                    dismissed: false,
                };

                let checkpoint = failover::capture_checkpoint_from_running(
                    &config,
                    &self.root,
                    "auto_rate_limit_candidate",
                )
                .ok()
                .flatten();

                if let Some(ref saved) = checkpoint {
                    self.set_last_checkpoint(saved.clone());
                }

                rate_limit_watch::log_event(
                    Some(self.root.as_path()),
                    "WARN",
                    "rate_limit_candidate",
                    json!({
                        "matchedPattern": matched,
                        "sessionId": session.session_id,
                        "createdAt": session.created_at,
                        "role": response.role,
                        "done": response.done,
                        "responseContent": truncate(&response.content, 8000),
                        "responseLength": response.content.len(),
                        "patternsChecked": settings.rate_limit_patterns,
                        "sessionCount": sessions.sessions.len(),
                        "sessions": sessions.sessions,
                        "codexApiProbe": failover::probe_codex_api(&config),
                        "codexProcess": {
                            "running": process.running,
                            "pid": process.pid,
                            "method": process.method,
                        },
                        "checkpoint": checkpoint,
                        "nextSteps": [
                            "Inspect rate-limit-discovery.jsonl beside app.log and ~/.codex-launchpad/discovery/rate-limit.jsonl",
                            "Run `codex app-server generate-json-schema --out ./codex-schemas` while reproducing",
                            "Compare REST /sessions output with Codex UI message"
                        ],
                    }),
                );

                self.push_alert(alert);
                return;
            }
        }
    }

    fn push_alert(&mut self, alert: RateLimitAlert) {
        if self
            .status
            .active_alert
            .as_ref()
            .is_some_and(|existing| existing.snippet == alert.snippet)
        {
            return;
        }

        self.status.active_alert = Some(alert.clone());
        self.status.recent_alerts.insert(0, alert);
        self.status.recent_alerts.truncate(20);
    }
}

fn discovery_hint() -> String {
    let home = dirs::home_dir()
        .map(|path| path.join(".codex-launchpad/discovery/rate-limit.jsonl"))
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "~/.codex-launchpad/discovery/rate-limit.jsonl".to_string());
    format!("Also mirrored in app.log. Persistent copy: {home}")
}

fn truncate(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        return value.to_string();
    }
    let mut end = max_len;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &value[..end])
}

pub fn spawn_monitor(
    config_path: PathBuf,
    root: PathBuf,
    monitor: Arc<Mutex<CodexMonitor>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        rate_limit_watch::log_event(
            Some(root.as_path()),
            "INFO",
            "watch_started",
            json!({
                "message": "Automatic rate-limit discovery logging is active whenever Codex uses the account provider.",
                "discoveryHint": discovery_hint(),
            }),
        );

        let runtime = tokio::runtime::Runtime::new().expect("monitor tokio runtime");
        loop {
            let interval = {
                let settings = LauncherConfig::read(&config_path)
                    .map(|config| failover::failover_settings(&config))
                    .unwrap_or_default();
                Duration::from_secs(settings.monitor_interval_secs.max(5))
            };

            runtime.block_on(async {
                tokio::time::sleep(interval).await;
            });

            let mut guard = match monitor.lock() {
                Ok(guard) => guard,
                Err(_) => continue,
            };

            guard.poll_once();

            if guard.status.auto_switch {
                if let Some(alert) = guard.status.active_alert.clone() {
                    if !alert.dismissed {
                        let pid_file =
                            crate::codex_process::CodexProcess::spawn_pid_file_path(&root);
                        let config = match LauncherConfig::read(&config_path) {
                            Ok(config) => config,
                            Err(_) => continue,
                        };
                        match failover::run_manual_failover(
                            &config,
                            &root,
                            &pid_file,
                            None,
                            "auto_rate_limit",
                        ) {
                            Ok(result) => {
                                if let Some(checkpoint) = result.checkpoint {
                                    guard.set_last_checkpoint(checkpoint);
                                }
                                guard.dismiss_alert();
                            }
                            Err(error) => {
                                guard.status.last_error = Some(error.to_string());
                                rate_limit_watch::log_event(
                                    Some(root.as_path()),
                                    "ERROR",
                                    "auto_switch_failed",
                                    json!({ "error": error.to_string() }),
                                );
                            }
                        }
                    }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_multibyte_safe() {
        assert_eq!(truncate("hello", 10), "hello");
        assert!(truncate("abcdefghijklmnopqrstuvwxyz", 10).ends_with("..."));
    }
}