use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;

use crate::config::LauncherConfig;
use crate::connection_watch::{self, ConnectionAlert, ConnectionAlertKind, EndpointHealth};
use crate::failover::{self, APP_SERVER_RATE_LIMIT_SOURCE};
use crate::lpad_app_server;
use crate::lpad_process::{CodexProcess, CodexProcessInfo};
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
    pub active_connection_alert: Option<ConnectionAlert>,
    pub recent_connection_alerts: Vec<ConnectionAlert>,
    pub endpoint_health: Option<EndpointHealth>,
    pub codex_api_ready: bool,
    pub endpoint_reachable: bool,
    pub connection_log_hint: String,
}

#[derive(Debug)]
pub struct CodexMonitor {
    config_path: PathBuf,
    root: PathBuf,
    status: CodexMonitorStatus,
    poll_count: u64,
    last_session_fingerprints: HashMap<String, String>,
    last_codex_api_ready: Option<bool>,
    last_endpoint_reachable: Option<bool>,
    last_codex_running: Option<bool>,
    last_rate_limits_poll_signature: Option<String>,
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
                discovery_log_hint: rate_limit_discovery_hint(),
                active_connection_alert: None,
                recent_connection_alerts: Vec::new(),
                endpoint_health: None,
                codex_api_ready: false,
                endpoint_reachable: false,
                connection_log_hint: connection_discovery_hint(),
            },
            poll_count: 0,
            last_session_fingerprints: HashMap::new(),
            last_codex_api_ready: None,
            last_endpoint_reachable: None,
            last_codex_running: None,
            last_rate_limits_poll_signature: None,
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

    pub fn dismiss_connection_alert(&mut self) {
        if let Some(alert) = self.status.active_connection_alert.as_mut() {
            alert.dismissed = true;
        }
        self.status.active_connection_alert = None;
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
        let provider_mode = session_checkpoint::provider_mode_from_config(&config);
        self.status.auto_switch = settings.auto_switch;
        self.status.watching = true;
        self.status.last_poll_at = Some(Utc::now());
        self.status.discovery_log_hint = rate_limit_discovery_hint();
        self.status.connection_log_hint = connection_discovery_hint();

        let process = crate::app_logic::detect_codex_process(&config, &self.root);
        let codex_api_ready = process.running && crate::app_logic::codex_api_ready(&config);

        self.poll_connection_health(&config, &process, provider_mode, codex_api_ready);

        if provider_mode == ProviderModeKind::CodexAccount {
            self.poll_structured_rate_limits(&config);
        }

        if process.running && codex_api_ready {
            self.poll_session_watch(
                &config,
                &process,
                &settings,
                provider_mode == ProviderModeKind::CodexAccount,
            );
        }
    }

    fn poll_connection_health(
        &mut self,
        config: &LauncherConfig,
        process: &CodexProcessInfo,
        provider_mode: ProviderModeKind,
        codex_api_ready: bool,
    ) {
        self.status.codex_api_ready = codex_api_ready;

        let endpoint_health =
            if provider_mode == ProviderModeKind::LocalApi || self.poll_count.is_multiple_of(2) {
                Some(connection_watch::probe_endpoint_health(config))
            } else {
                None
            };

        if let Some(ref health) = endpoint_health {
            self.status.endpoint_health = Some(health.clone());
            self.status.endpoint_reachable = health.reachable;
        }

        if let Some(previous) = self.last_codex_running {
            if previous && !process.running {
                connection_watch::log_event(
                    Some(self.root.as_path()),
                    "WARN",
                    "codex_process_stopped",
                    json!({ "pid": process.pid, "method": process.method }),
                );
            } else if !previous && process.running {
                connection_watch::log_event(
                    Some(self.root.as_path()),
                    "INFO",
                    "codex_process_started",
                    json!({
                        "pid": process.pid,
                        "method": process.method,
                        "codexApiProbe": connection_watch::probe_codex_api_health(config),
                    }),
                );
            }
        }
        self.last_codex_running = Some(process.running);

        if process.running {
            if let Some(previous) = self.last_codex_api_ready {
                if previous && !codex_api_ready {
                    let alert = connection_watch::alert_for_kind(
                        ConnectionAlertKind::CodexApiDown,
                        endpoint_health.clone(),
                    );
                    self.push_connection_alert(alert);
                    connection_watch::log_event(
                        Some(self.root.as_path()),
                        "WARN",
                        "codex_api_disconnected",
                        json!({
                            "codexApiProbe": connection_watch::probe_codex_api_health(config),
                            "codexPid": process.pid,
                        }),
                    );
                } else if !previous && codex_api_ready {
                    let alert = connection_watch::alert_for_kind(
                        ConnectionAlertKind::CodexApiRestored,
                        endpoint_health.clone(),
                    );
                    self.push_connection_alert(alert);
                    connection_watch::log_event(
                        Some(self.root.as_path()),
                        "INFO",
                        "codex_api_reconnected",
                        json!({
                            "codexApiProbe": connection_watch::probe_codex_api_health(config),
                            "codexPid": process.pid,
                        }),
                    );
                }
            }
            self.last_codex_api_ready = Some(codex_api_ready);
        } else {
            self.last_codex_api_ready = None;
        }

        if provider_mode == ProviderModeKind::LocalApi {
            if let Some(health) = endpoint_health {
                if let Some(previous) = self.last_endpoint_reachable {
                    if previous && !health.reachable {
                        let alert = connection_watch::alert_for_kind(
                            ConnectionAlertKind::EndpointDown,
                            Some(health.clone()),
                        );
                        self.push_connection_alert(alert);
                        connection_watch::log_event(
                            Some(self.root.as_path()),
                            "WARN",
                            "endpoint_unreachable",
                            json!({ "endpointHealth": health }),
                        );
                    } else if !previous && health.reachable {
                        let alert = connection_watch::alert_for_kind(
                            ConnectionAlertKind::EndpointRestored,
                            Some(health.clone()),
                        );
                        self.push_connection_alert(alert);
                        connection_watch::log_event(
                            Some(self.root.as_path()),
                            "INFO",
                            "endpoint_reconnected",
                            json!({ "endpointHealth": health }),
                        );
                    }
                }
                self.last_endpoint_reachable = Some(health.reachable);
            }
        }
    }

    fn poll_structured_rate_limits(&mut self, config: &LauncherConfig) {
        if !self.poll_count.is_multiple_of(6) {
            return;
        }

        let status = lpad_app_server::read_rate_limits(config);
        let signature = rate_limits_poll_signature(&status);
        if self.last_rate_limits_poll_signature.as_deref() != Some(signature.as_str()) {
            self.last_rate_limits_poll_signature = Some(signature);
            rate_limit_watch::log_event(
                Some(self.root.as_path()),
                if status.ok { "INFO" } else { "WARN" },
                "app_server_rate_limits_poll",
                json!({
                    "ok": status.ok,
                    "error": status.error,
                    "planType": status.plan_type,
                    "rateLimitReachedType": lpad_app_server::rate_limit_reached_type(&status),
                    "snippet": lpad_app_server::rate_limit_status_snippet(&status),
                    "rateLimits": status.rate_limits,
                }),
            );
        }

        if failover::should_failover_for_rate_limits(&status) {
            let reached_type =
                failover::rate_limit_reached_from_status(&status).unwrap_or_default();
            let alert = RateLimitAlert {
                detected_at: Utc::now(),
                matched_pattern: reached_type,
                source: APP_SERVER_RATE_LIMIT_SOURCE.to_string(),
                session_id: None,
                snippet: lpad_app_server::rate_limit_status_snippet(&status),
                dismissed: false,
            };

            let checkpoint = failover::capture_checkpoint_from_running(
                config,
                &self.root,
                "auto_rate_limit_structured",
            )
            .ok()
            .flatten();

            if let Some(ref saved) = checkpoint {
                self.set_last_checkpoint(saved.clone());
            }

            rate_limit_watch::log_event(
                Some(self.root.as_path()),
                "WARN",
                "rate_limit_structured",
                json!({
                    "rateLimitReachedType": alert.matched_pattern,
                    "snippet": alert.snippet,
                    "checkpoint": checkpoint,
                    "rateLimits": status.rate_limits,
                }),
            );

            self.push_alert(alert);
            return;
        }

        if self
            .status
            .active_alert
            .as_ref()
            .is_some_and(|alert| alert.source == APP_SERVER_RATE_LIMIT_SOURCE && !alert.dismissed)
        {
            self.dismiss_alert();
            rate_limit_watch::log_event(
                Some(self.root.as_path()),
                "INFO",
                "rate_limit_cleared",
                json!({
                    "snippet": lpad_app_server::rate_limit_status_snippet(&status),
                }),
            );
        }
    }

    fn poll_session_watch(
        &mut self,
        config: &LauncherConfig,
        process: &CodexProcessInfo,
        settings: &crate::config::FailoverSettings,
        check_rate_limits: bool,
    ) {
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

        let client = match crate::acp_client::AcpClient::from_config(config) {
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
        let connection_patterns = connection_watch::default_connection_failure_patterns();

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

                if let Some(matched) = connection_watch::matches_connection_failure(
                    &response.content,
                    &connection_patterns,
                ) {
                    let endpoint_health = connection_watch::probe_endpoint_health(config);
                    let alert = connection_watch::alert_for_kind(
                        ConnectionAlertKind::SessionConnectionError,
                        Some(endpoint_health.clone()),
                    );
                    self.push_connection_alert(alert);
                    connection_watch::log_event(
                        Some(self.root.as_path()),
                        "WARN",
                        "session_connection_error",
                        json!({
                            "matchedPattern": matched,
                            "sessionId": session.session_id,
                            "contentPreview": truncate(&response.content, 4000),
                            "endpointHealth": endpoint_health,
                            "codexApiProbe": connection_watch::probe_codex_api_health(config),
                        }),
                    );
                }

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

            if check_rate_limits {
                if let Some(matched) =
                    failover::matches_rate_limit(&response.content, &settings.rate_limit_patterns)
                {
                    rate_limit_watch::log_event(
                        Some(self.root.as_path()),
                        "WARN",
                        "rate_limit_text_candidate",
                        json!({
                            "matchedPattern": matched,
                            "sessionId": session.session_id,
                            "createdAt": session.created_at,
                            "role": response.role,
                            "done": response.done,
                            "responseContent": truncate(&response.content, 8000),
                            "responseLength": response.content.len(),
                            "patternsChecked": settings.rate_limit_patterns,
                            "note": "Text-only match is logged for discovery; auto-failover requires app-server rateLimitReachedType.",
                            "sessionCount": sessions.sessions.len(),
                            "codexApiProbe": connection_watch::probe_codex_api_health(config),
                            "endpointHealth": connection_watch::probe_endpoint_health(config),
                            "codexProcess": {
                                "running": process.running,
                                "pid": process.pid,
                                "method": process.method,
                            },
                        }),
                    );
                }
            }
        }
    }

    fn push_alert(&mut self, alert: RateLimitAlert) {
        if self.status.active_alert.as_ref().is_some_and(|existing| {
            existing.source == alert.source
                && existing.matched_pattern == alert.matched_pattern
                && existing.snippet == alert.snippet
        }) {
            return;
        }

        self.status.active_alert = Some(alert.clone());
        self.status.recent_alerts.insert(0, alert);
        self.status.recent_alerts.truncate(20);
    }

    fn push_connection_alert(&mut self, alert: ConnectionAlert) {
        if self
            .status
            .active_connection_alert
            .as_ref()
            .is_some_and(|existing| {
                existing.kind == alert.kind && existing.message == alert.message
            })
        {
            return;
        }

        self.status.active_connection_alert = Some(alert.clone());
        self.status.recent_connection_alerts.insert(0, alert);
        self.status.recent_connection_alerts.truncate(20);
    }
}

fn rate_limits_poll_signature(status: &lpad_app_server::CodexRateLimitsStatus) -> String {
    format!(
        "{}|{}|{}|{}",
        status.ok,
        status.error.as_deref().unwrap_or(""),
        lpad_app_server::rate_limit_reached_type(status).unwrap_or_default(),
        lpad_app_server::rate_limit_status_snippet(status),
    )
}

fn rate_limit_discovery_hint() -> String {
    let home = dirs::home_dir()
        .map(|path| path.join(".launchpadx/discovery/rate-limit.jsonl"))
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "~/.launchpadx/discovery/rate-limit.jsonl".to_string());
    format!("Also mirrored in app.log. Persistent copy: {home}")
}

fn connection_discovery_hint() -> String {
    let home = dirs::home_dir()
        .map(|path| path.join(".launchpadx/discovery/connection.jsonl"))
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "~/.launchpadx/discovery/connection.jsonl".to_string());
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
                "message": "Automatic Codex interaction watch is active (rate limits, reconnects, endpoint health).",
                "rateLimitDiscoveryHint": rate_limit_discovery_hint(),
                "connectionDiscoveryHint": connection_discovery_hint(),
            }),
        );
        connection_watch::log_event(
            Some(root.as_path()),
            "INFO",
            "connection_watch_started",
            json!({
                "message": "Connection health monitoring is active for local API and Codex API transitions.",
                "discoveryHint": connection_discovery_hint(),
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
                        let pid_file = CodexProcess::spawn_pid_file_path(&root);
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
