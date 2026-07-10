use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::LauncherConfig;
use crate::ollama;

const DISCOVERY_DIR: &str = ".launchpadx/discovery";
const DISCOVERY_FILE: &str = "connection.jsonl";
const APP_LOG_PREFIX: &str = "CONNECTION_WATCH";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionAlertKind {
    EndpointDown,
    EndpointRestored,
    CodexApiDown,
    CodexApiRestored,
    SessionConnectionError,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AlertSeverity {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointHealth {
    pub checked_at: DateTime<Utc>,
    pub endpoint_url: Option<String>,
    pub reachable: bool,
    pub status_code: Option<u16>,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
    pub model_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionAlert {
    pub detected_at: DateTime<Utc>,
    pub kind: ConnectionAlertKind,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub endpoint_health: Option<EndpointHealth>,
    pub dismissed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveryRecord {
    event: String,
    at: String,
    #[serde(flatten)]
    details: Value,
}

pub fn default_connection_failure_patterns() -> Vec<String> {
    vec![
        "connection refused".to_string(),
        "econnrefused".to_string(),
        "failed to connect".to_string(),
        "could not connect".to_string(),
        "couldn't connect".to_string(),
        "network error".to_string(),
        "connection error".to_string(),
        "connection reset".to_string(),
        "timed out".to_string(),
        "timeout".to_string(),
        "unreachable".to_string(),
        "no route to host".to_string(),
        "dial tcp".to_string(),
        "502 bad gateway".to_string(),
        "503 service unavailable".to_string(),
        "504 gateway timeout".to_string(),
        "ollama".to_string(),
    ]
}

pub fn matches_connection_failure(text: &str, patterns: &[String]) -> Option<String> {
    let lower = text.to_lowercase();
    patterns
        .iter()
        .find(|pattern| lower.contains(&pattern.to_lowercase()))
        .cloned()
}

pub fn probe_endpoint_health(config: &LauncherConfig) -> EndpointHealth {
    let checked_at = Utc::now();
    let base_url = match config.openai_base_url() {
        Ok(url) => url,
        Err(error) => {
            return EndpointHealth {
                checked_at,
                endpoint_url: None,
                reachable: false,
                status_code: None,
                latency_ms: None,
                error: Some(error.to_string()),
                model_count: None,
            };
        }
    };

    let tags_url = match ollama::tags_url_from_base(&base_url) {
        Ok(url) => url,
        Err(error) => {
            return EndpointHealth {
                checked_at,
                endpoint_url: Some(base_url),
                reachable: false,
                status_code: None,
                latency_ms: None,
                error: Some(error.to_string()),
                model_count: None,
            };
        }
    };

    let client = match Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return EndpointHealth {
                checked_at,
                endpoint_url: Some(tags_url),
                reachable: false,
                status_code: None,
                latency_ms: None,
                error: Some(error.to_string()),
                model_count: None,
            };
        }
    };

    let mut request = client.get(&tags_url);
    if let Some(api_key) = config.api_key_if_configured() {
        request = request.bearer_auth(api_key);
    }

    let started = Instant::now();
    match request.send() {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let latency_ms = started.elapsed().as_millis() as u64;
            let reachable = response.status().is_success();
            let model_count = if reachable {
                response
                    .json::<TagsResponse>()
                    .ok()
                    .map(|payload| payload.models.len())
            } else {
                None
            };
            let error = if reachable {
                None
            } else {
                Some(format!("HTTP {status_code}"))
            };

            EndpointHealth {
                checked_at,
                endpoint_url: Some(tags_url),
                reachable,
                status_code: Some(status_code),
                latency_ms: Some(latency_ms),
                error,
                model_count,
            }
        }
        Err(error) => EndpointHealth {
            checked_at,
            endpoint_url: Some(tags_url),
            reachable: false,
            status_code: None,
            latency_ms: Some(started.elapsed().as_millis() as u64),
            error: Some(error.to_string()),
            model_count: None,
        },
    }
}

pub fn probe_codex_api_health(config: &LauncherConfig) -> Value {
    let base_url = config.codex_api_base_url();
    let health_url = format!("{base_url}/health");
    let started = Instant::now();
    let client = match Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return json!({
                "codexApiBaseUrl": base_url,
                "healthOk": false,
                "error": error.to_string(),
            });
        }
    };

    match client.get(&health_url).send() {
        Ok(response) => json!({
            "codexApiBaseUrl": base_url,
            "healthOk": response.status().is_success(),
            "statusCode": response.status().as_u16(),
            "latencyMs": started.elapsed().as_millis() as u64,
        }),
        Err(error) => json!({
            "codexApiBaseUrl": base_url,
            "healthOk": false,
            "latencyMs": started.elapsed().as_millis() as u64,
            "error": error.to_string(),
        }),
    }
}

pub fn alert_for_kind(
    kind: ConnectionAlertKind,
    endpoint_health: Option<EndpointHealth>,
) -> ConnectionAlert {
    let (severity, title, message) = match kind {
        ConnectionAlertKind::EndpointDown => {
            let detail = endpoint_health
                .as_ref()
                .and_then(|health| health.error.clone())
                .or_else(|| endpoint_health.as_ref().and_then(|h| h.endpoint_url.clone()))
                .unwrap_or_else(|| "configured endpoint".to_string());
            (
                AlertSeverity::Error,
                "Local API unreachable".to_string(),
                format!(
                    "Launchpad cannot reach your API endpoint ({detail}). Check that Ollama or your gateway is running and the IP/port in Settings are correct."
                ),
            )
        }
        ConnectionAlertKind::EndpointRestored => (
            AlertSeverity::Info,
            "Local API back online".to_string(),
            "Your OpenAI-compatible endpoint is reachable again.".to_string(),
        ),
        ConnectionAlertKind::CodexApiDown => (
            AlertSeverity::Warn,
            "Codex API disconnected".to_string(),
            "Codex is running but its local API stopped responding. Restart Codex if chat or automations stop working.".to_string(),
        ),
        ConnectionAlertKind::CodexApiRestored => (
            AlertSeverity::Info,
            "Codex API reconnected".to_string(),
            "Codex's local API is responding again.".to_string(),
        ),
        ConnectionAlertKind::SessionConnectionError => {
            let probe = endpoint_health
                .as_ref()
                .map(|health| {
                    if health.reachable {
                        format!(
                            "Launchpad probed {} and it responded ({} models, {} ms). The failure may be inside Codex or transient.",
                            health.endpoint_url.as_deref().unwrap_or("the endpoint"),
                            health.model_count.unwrap_or(0),
                            health.latency_ms.unwrap_or(0)
                        )
                    } else {
                        format!(
                            "Launchpad also cannot reach {} — {}",
                            health.endpoint_url.as_deref().unwrap_or("the endpoint"),
                            health.error.as_deref().unwrap_or("connection failed")
                        )
                    }
                })
                .unwrap_or_else(|| "Could not probe the configured endpoint.".to_string());
            (
                AlertSeverity::Error,
                "Codex reported a connection problem".to_string(),
                probe,
            )
        }
    };

    ConnectionAlert {
        detected_at: Utc::now(),
        kind,
        severity,
        title,
        message,
        endpoint_health,
        dismissed: false,
    }
}

pub fn discovery_paths(root: Option<&Path>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(root) = root {
        paths.push(root.join("connection-discovery.jsonl"));
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

// Private mirror of ollama tags response for probe parsing.
#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<ollama::OllamaModel>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_connection_errors() {
        let patterns = default_connection_failure_patterns();
        assert_eq!(
            matches_connection_failure(
                "dial tcp 127.0.0.1:11434: connect: connection refused",
                &patterns
            ),
            Some("connection refused".to_string())
        );
        assert!(matches_connection_failure("patch applied cleanly", &patterns).is_none());
    }
}
