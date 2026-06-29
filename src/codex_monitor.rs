use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::config::LauncherConfig;
use crate::config::FailoverSettings;
use crate::failover;
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
    pub enabled: bool,
    pub auto_switch: bool,
    pub monitoring: bool,
    pub last_poll_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub active_alert: Option<RateLimitAlert>,
    pub recent_alerts: Vec<RateLimitAlert>,
    pub last_checkpoint: Option<SessionCheckpoint>,
}

#[derive(Debug)]
pub struct CodexMonitor {
    config_path: PathBuf,
    root: PathBuf,
    status: CodexMonitorStatus,
}

impl CodexMonitor {
    pub fn new(config_path: PathBuf, root: PathBuf, settings: &FailoverSettings) -> Self {
        Self {
            config_path,
            root,
            status: CodexMonitorStatus {
                enabled: settings.enabled,
                auto_switch: settings.auto_switch,
                monitoring: settings.enabled,
                last_poll_at: None,
                last_error: None,
                active_alert: None,
                recent_alerts: Vec::new(),
                last_checkpoint: None,
            },
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
        let config = match LauncherConfig::read(&self.config_path) {
            Ok(config) => config,
            Err(error) => {
                self.status.last_error = Some(error.to_string());
                return;
            }
        };

        let settings = failover::failover_settings(&config);
        self.status.enabled = settings.enabled;
        self.status.auto_switch = settings.auto_switch;
        self.status.monitoring = settings.enabled;
        self.status.last_poll_at = Some(Utc::now());

        if !settings.enabled {
            return;
        }

        if session_checkpoint::provider_mode_from_config(&config) == ProviderModeKind::LocalApi
        {
            return;
        }

        let process = crate::app_logic::detect_codex_process(&config, &self.root);
        if !process.running || !crate::app_logic::codex_api_ready(&config) {
            return;
        }

        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                self.status.last_error = Some(error.to_string());
                return;
            }
        };

        let client = match crate::acp_client::AcpClient::from_config(&config) {
            Ok(client) => client,
            Err(error) => {
                self.status.last_error = Some(error.to_string());
                return;
            }
        };

        let sessions = match runtime.block_on(client.list_sessions()) {
            Ok(sessions) => sessions,
            Err(error) => {
                self.status.last_error = Some(error.to_string());
                return;
            }
        };

        self.status.last_error = None;

        for session in &sessions.sessions {
            let response = match runtime.block_on(client.get_response(&session.session_id)) {
                Ok(response) => response,
                Err(_) => continue,
            };

            if let Some(matched) =
                failover::matches_rate_limit(&response.content, &settings.rate_limit_patterns)
            {
                let snippet = truncate(&response.content, 240);
                let alert = RateLimitAlert {
                    detected_at: Utc::now(),
                    matched_pattern: matched,
                    source: "session_response".to_string(),
                    session_id: Some(session.session_id.clone()),
                    snippet,
                    dismissed: false,
                };
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