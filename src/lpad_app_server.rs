use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::thread;
use std::time::Duration;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::LauncherConfig;
use crate::launcher;
use crate::lpad_thread_store;

const INIT_REQUEST_ID: i64 = 0;
const HANDSHAKE_DELAY_MS: u64 = 500;
const READ_TIMEOUT_SECS: u64 = 25;
const THREAD_LIST_LIMIT: u32 = 25;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitWindow {
    pub used_percent: Option<f64>,
    #[serde(alias = "windowMinutes")]
    pub window_duration_mins: Option<u32>,
    pub resets_at: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitCredits {
    pub has_credits: Option<bool>,
    pub unlimited: Option<bool>,
    pub balance: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpendControlLimitSnapshot {
    pub limit: Option<String>,
    pub used: Option<String>,
    pub remaining_percent: Option<f64>,
    pub resets_at: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexRateLimits {
    pub limit_id: Option<String>,
    pub limit_name: Option<String>,
    pub primary: Option<RateLimitWindow>,
    pub secondary: Option<RateLimitWindow>,
    pub credits: Option<RateLimitCredits>,
    pub individual_limit: Option<SpendControlLimitSnapshot>,
    pub spend_control_reached: Option<bool>,
    pub plan_type: Option<String>,
    pub rate_limit_reached_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitResetCredits {
    pub available_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexThreadSummary {
    pub id: String,
    pub name: Option<String>,
    pub status: Option<String>,
    pub path: Option<String>,
    pub created_at: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexThreadListStatus {
    pub ok: bool,
    pub fetched_at: String,
    pub source: String,
    pub codex_cli: Option<String>,
    pub error: Option<String>,
    pub threads: Vec<CodexThreadSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexRateLimitsStatus {
    pub ok: bool,
    pub fetched_at: String,
    pub source: String,
    pub codex_cli: Option<String>,
    pub error: Option<String>,
    pub requires_auth: Option<bool>,
    pub plan_type: Option<String>,
    pub rate_limits: Option<CodexRateLimits>,
    pub rate_limit_reset_credits: Option<RateLimitResetCredits>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppServerError {
    #[error("could not start codex app-server: {0}")]
    Spawn(String),
    #[error("codex CLI not found on PATH; install Codex CLI or set codexCommand")]
    CliNotFound,
    #[error("app-server request timed out")]
    Timeout,
    #[error("app-server returned an error: {0}")]
    Rpc(String),
    #[error("could not parse app-server response: {0}")]
    Parse(String),
}

pub fn list_threads(config: &LauncherConfig) -> CodexThreadListStatus {
    let fetched_at = Utc::now().to_rfc3339();
    if let Ok(status) = list_threads_via_app_server(config) {
        return CodexThreadListStatus {
            fetched_at,
            ..status
        };
    }

    if let Ok(threads) = lpad_thread_store::list_threads_from_store(THREAD_LIST_LIMIT as usize) {
        if !threads.is_empty() {
            return CodexThreadListStatus {
                ok: true,
                fetched_at,
                source: "codex ~/.codex/session_index.jsonl".to_string(),
                codex_cli: None,
                error: None,
                threads,
            };
        }
    }

    let cli_candidates = resolve_codex_cli_candidates(config);
    CodexThreadListStatus {
        ok: false,
        fetched_at,
        source: "codex app-server thread/list".to_string(),
        codex_cli: cli_candidates.first().cloned(),
        error: Some("Could not reach Codex app-server or read local session index.".to_string()),
        threads: Vec::new(),
    }
}

pub fn read_rate_limits(config: &LauncherConfig) -> CodexRateLimitsStatus {
    let fetched_at = Utc::now().to_rfc3339();
    if let Ok(status) = read_rate_limits_via_app_server(config) {
        return CodexRateLimitsStatus {
            fetched_at,
            ..status
        };
    }

    let cli_candidates = resolve_codex_cli_candidates(config);
    CodexRateLimitsStatus {
        ok: false,
        fetched_at,
        source: "codex app-server account/rateLimits/read".to_string(),
        codex_cli: cli_candidates.first().cloned(),
        error: Some(cli_discovery_error(
            "Codex app-server unavailable".to_string(),
            &cli_candidates,
        )),
        requires_auth: None,
        plan_type: None,
        rate_limits: None,
        rate_limit_reset_credits: None,
    }
}

struct AppServerClient {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    next_id: i64,
}

impl AppServerClient {
    fn spawn(cli: &str) -> Result<Self, AppServerError> {
        let mut child = crate::process_util::command(cli)
            .args(["app-server", "--listen", "stdio://"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    AppServerError::CliNotFound
                } else {
                    AppServerError::Spawn(error.to_string())
                }
            })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppServerError::Spawn("stdin unavailable".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppServerError::Spawn("stdout unavailable".to_string()))?;

        Ok(Self {
            child,
            stdin,
            reader: BufReader::new(stdout),
            next_id: 1,
        })
    }

    fn initialize(&mut self) -> Result<(), AppServerError> {
        let init = serde_json::json!({
            "method": "initialize",
            "id": INIT_REQUEST_ID,
            "params": {
                "clientInfo": {
                    "name": "launchpadx",
                    "title": "LaunchPadX",
                    "version": "0.1.0"
                }
            }
        });
        write_request(&mut self.stdin, &init)?;
        write_request(
            &mut self.stdin,
            &serde_json::json!({"method": "initialized", "params": {}}),
        )?;
        thread::sleep(Duration::from_millis(HANDSHAKE_DELAY_MS));

        let init_response =
            read_response_for_id(&mut self.reader, INIT_REQUEST_ID, READ_TIMEOUT_SECS)?;
        if init_response.get("error").is_some() {
            return Err(AppServerError::Rpc(
                init_response
                    .get("error")
                    .and_then(|value| value.get("message"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("initialize failed")
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn call(&mut self, method: &str, params: Value) -> Result<Value, AppServerError> {
        let request_id = self.next_id;
        self.next_id += 1;
        let payload = serde_json::json!({
            "method": method,
            "id": request_id,
            "params": params,
        });
        write_request(&mut self.stdin, &payload)?;
        let response = read_response_for_id(&mut self.reader, request_id, READ_TIMEOUT_SECS)?;
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|value| value.as_str())
                .unwrap_or(method);
            return Err(AppServerError::Rpc(message.to_string()));
        }
        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    }
}

impl Drop for AppServerClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn read_rate_limits_via_cli(cli: &str) -> Result<CodexRateLimitsStatus, AppServerError> {
    let mut client = AppServerClient::spawn(cli)?;
    client.initialize()?;
    let result = client.call("account/rateLimits/read", serde_json::json!({}))?;
    parse_rate_limits_result(result, cli)
}

fn list_threads_via_cli(cli: &str) -> Result<CodexThreadListStatus, AppServerError> {
    let mut client = AppServerClient::spawn(cli)?;
    client.initialize()?;
    let result = client.call(
        "thread/list",
        serde_json::json!({ "limit": THREAD_LIST_LIMIT }),
    )?;
    parse_thread_list_result(result, cli)
}

fn parse_thread_list_result(
    result: Value,
    cli: &str,
) -> Result<CodexThreadListStatus, AppServerError> {
    let threads = extract_thread_rows(&result);
    Ok(CodexThreadListStatus {
        ok: true,
        fetched_at: Utc::now().to_rfc3339(),
        source: "codex app-server thread/list".to_string(),
        codex_cli: Some(cli.to_string()),
        error: None,
        threads,
    })
}

fn extract_thread_rows(result: &Value) -> Vec<CodexThreadSummary> {
    let rows = result
        .get("data")
        .or_else(|| result.get("threads"))
        .and_then(|value| value.as_array())
        .cloned()
        .or_else(|| result.get("thread").map(|thread| vec![thread.clone()]))
        .unwrap_or_default();

    rows.into_iter().filter_map(parse_thread_summary).collect()
}

fn parse_thread_summary(value: Value) -> Option<CodexThreadSummary> {
    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .map(str::to_string)?;
    Some(CodexThreadSummary {
        id,
        name: value
            .get("name")
            .or_else(|| value.get("threadName"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        status: value
            .get("status")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        path: value
            .get("path")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        created_at: value
            .get("createdAt")
            .or_else(|| value.get("created_at"))
            .and_then(|v| {
                v.as_str()
                    .map(str::to_string)
                    .or_else(|| v.as_i64().map(|n| n.to_string()))
            }),
        model: value
            .get("model")
            .or_else(|| value.pointer("/settings/model"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
    })
}

fn parse_rate_limits_result(
    result: Value,
    cli: &str,
) -> Result<CodexRateLimitsStatus, AppServerError> {
    let rate_limits = extract_rate_limits(&result);
    let rate_limit_reset_credits: Option<RateLimitResetCredits> = result
        .get("rateLimitResetCredits")
        .and_then(|value| serde_json::from_value(value.clone()).ok());

    let plan_type = rate_limits
        .as_ref()
        .and_then(|limits| limits.plan_type.clone());

    Ok(CodexRateLimitsStatus {
        ok: true,
        fetched_at: Utc::now().to_rfc3339(),
        source: "codex app-server account/rateLimits/read".to_string(),
        codex_cli: Some(cli.to_string()),
        error: None,
        requires_auth: Some(false),
        plan_type,
        rate_limits,
        rate_limit_reset_credits,
    })
}

fn extract_rate_limits(result: &Value) -> Option<CodexRateLimits> {
    let main = result
        .get("rateLimits")
        .and_then(|value| serde_json::from_value::<CodexRateLimits>(value.clone()).ok());
    let by_limit_id = result
        .get("rateLimitsByLimitId")
        .and_then(|value| value.get("codex"))
        .and_then(|value| serde_json::from_value::<CodexRateLimits>(value.clone()).ok());

    let merged = match (main, by_limit_id) {
        (Some(main), Some(by_limit_id)) => Some(merge_rate_limits(main, by_limit_id)),
        (Some(main), None) => Some(main),
        (None, Some(by_limit_id)) => Some(by_limit_id),
        (None, None) => None,
    };

    merged.filter(|limits| rate_limits_has_usage(Some(limits)))
}

fn merge_rate_limits(mut base: CodexRateLimits, overlay: CodexRateLimits) -> CodexRateLimits {
    base.limit_id = base.limit_id.or(overlay.limit_id);
    base.limit_name = base.limit_name.or(overlay.limit_name);
    base.primary = merge_rate_limit_window(&base.primary, &overlay.primary);
    base.secondary = merge_rate_limit_window(&base.secondary, &overlay.secondary);
    base.credits = base.credits.or(overlay.credits);
    base.individual_limit = base.individual_limit.or(overlay.individual_limit);
    base.spend_control_reached = base.spend_control_reached.or(overlay.spend_control_reached);
    base.plan_type = base.plan_type.or(overlay.plan_type);
    base.rate_limit_reached_type = base
        .rate_limit_reached_type
        .or(overlay.rate_limit_reached_type);
    base
}

fn merge_rate_limit_window(
    current: &Option<RateLimitWindow>,
    incoming: &Option<RateLimitWindow>,
) -> Option<RateLimitWindow> {
    match (current.as_ref(), incoming.as_ref()) {
        (Some(current), Some(incoming)) => Some(RateLimitWindow {
            used_percent: current.used_percent.or(incoming.used_percent),
            window_duration_mins: current
                .window_duration_mins
                .or(incoming.window_duration_mins),
            resets_at: current.resets_at.or(incoming.resets_at),
        }),
        (Some(current), None) => Some(current.clone()),
        (None, Some(incoming)) => Some(incoming.clone()),
        (None, None) => None,
    }
}

fn rate_limits_has_usage(limits: Option<&CodexRateLimits>) -> bool {
    limits.is_some_and(|limits| {
        window_has_usage(limits.primary.as_ref()) || window_has_usage(limits.secondary.as_ref())
    })
}

fn window_has_usage(window: Option<&RateLimitWindow>) -> bool {
    window.is_some_and(|window| window.used_percent.is_some())
}

fn write_request(
    stdin: &mut std::process::ChildStdin,
    payload: &Value,
) -> Result<(), AppServerError> {
    let line =
        serde_json::to_string(payload).map_err(|error| AppServerError::Parse(error.to_string()))?;
    stdin
        .write_all(line.as_bytes())
        .and_then(|_| stdin.write_all(b"\n"))
        .map_err(|error| AppServerError::Spawn(error.to_string()))
}

fn read_response_for_id(
    reader: &mut BufReader<std::process::ChildStdout>,
    request_id: i64,
    timeout_secs: u64,
) -> Result<Value, AppServerError> {
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
    let mut line_buf = String::new();

    loop {
        if std::time::Instant::now() > deadline {
            return Err(AppServerError::Timeout);
        }

        line_buf.clear();
        let bytes = reader
            .read_line(&mut line_buf)
            .map_err(|error| AppServerError::Parse(error.to_string()))?;
        if bytes == 0 {
            return Err(AppServerError::Timeout);
        }

        let trimmed = line_buf.trim();
        if trimmed.is_empty() {
            continue;
        }

        let value: Value = serde_json::from_str(trimmed)
            .map_err(|error| AppServerError::Parse(error.to_string()))?;

        if value.get("id").and_then(|id| id.as_i64()) == Some(request_id) {
            return Ok(value);
        }
    }
}

pub fn rate_limit_reached_type(status: &CodexRateLimitsStatus) -> Option<String> {
    status
        .rate_limits
        .as_ref()
        .and_then(|limits| limits.rate_limit_reached_type.clone())
        .filter(|value| !value.trim().is_empty())
}

pub fn is_rate_limit_reached(status: &CodexRateLimitsStatus) -> bool {
    rate_limit_reached_type(status).is_some()
}

pub fn rate_limit_status_snippet(status: &CodexRateLimitsStatus) -> String {
    let reached = rate_limit_reached_type(status).unwrap_or_else(|| "unknown".to_string());
    let windows = status
        .rate_limits
        .as_ref()
        .map(collect_usage_windows)
        .unwrap_or_default();
    if windows.is_empty() {
        return format!("rateLimitReachedType={reached}; windows=unavailable");
    }

    let summary = windows
        .iter()
        .map(|window| format_window_usage(window))
        .collect::<Vec<_>>()
        .join("; ");
    format!("rateLimitReachedType={reached}; {summary}")
}

fn collect_usage_windows(limits: &CodexRateLimits) -> Vec<&RateLimitWindow> {
    let mut windows = Vec::new();
    if let Some(window) = limits.primary.as_ref() {
        if window_has_usage(Some(window)) {
            windows.push(window);
        }
    }
    if let Some(window) = limits.secondary.as_ref() {
        if window_has_usage(Some(window)) {
            windows.push(window);
        }
    }

    windows.sort_by_key(|window| window.window_duration_mins.unwrap_or(u32::MAX));
    windows.dedup_by_key(|window| window.window_duration_mins);
    windows
}

fn format_window_usage(window: &RateLimitWindow) -> String {
    let label = window_duration_label(window.window_duration_mins);
    match window.used_percent {
        Some(used) => format!("{label} {used:.0}% used"),
        None => format!("{label} unavailable"),
    }
}

fn window_duration_label(window_duration_mins: Option<u32>) -> &'static str {
    match window_duration_mins {
        Some(300) => "5h",
        Some(10_080) => "weekly",
        Some(43_200) => "monthly",
        Some(minutes) if minutes <= 360 => "short-term",
        _ => "window",
    }
}

fn resolve_codex_cli_candidates(config: &LauncherConfig) -> Vec<String> {
    launcher::cli_executable_candidates(config)
}

fn list_threads_via_app_server(
    config: &LauncherConfig,
) -> Result<CodexThreadListStatus, AppServerError> {
    let mut last_error = None;
    for cli in resolve_codex_cli_candidates(config) {
        match list_threads_via_cli(&cli) {
            Ok(mut status) => {
                status.codex_cli = Some(cli);
                return Ok(status);
            }
            Err(error) => last_error = Some(format!("{cli}: {error}")),
        }
    }
    Err(last_error
        .map(AppServerError::Rpc)
        .unwrap_or(AppServerError::CliNotFound))
}

fn read_rate_limits_via_app_server(
    config: &LauncherConfig,
) -> Result<CodexRateLimitsStatus, AppServerError> {
    let mut last_error = None;
    for cli in resolve_codex_cli_candidates(config) {
        match read_rate_limits_via_cli(&cli) {
            Ok(mut status) => {
                status.codex_cli = Some(cli);
                return Ok(status);
            }
            Err(error) => last_error = Some(format!("{cli}: {error}")),
        }
    }
    Err(last_error
        .map(AppServerError::Rpc)
        .unwrap_or(AppServerError::CliNotFound))
}

fn cli_discovery_error(last_error: String, candidates: &[String]) -> String {
    let checked = candidates.join(", ");
    format!("{last_error}. Checked: {checked}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rate_limit_reached_type() {
        let mut status = parse_rate_limits_result(
            serde_json::json!({
                "rateLimits": {
                    "limitId": "codex",
                    "primary": { "usedPercent": 100, "windowDurationMins": 300, "resetsAt": 1779459394 },
                    "secondary": { "usedPercent": 10, "windowDurationMins": 10080, "resetsAt": 1779826837 },
                    "planType": "plus",
                    "rateLimitReachedType": "primary"
                }
            }),
            "codex",
        )
        .expect("parse");
        assert!(is_rate_limit_reached(&status));
        assert_eq!(rate_limit_reached_type(&status).as_deref(), Some("primary"));

        status.rate_limits.as_mut().unwrap().rate_limit_reached_type = None;
        assert!(!is_rate_limit_reached(&status));
    }

    #[test]
    fn snippet_lists_windows_by_duration_not_slot_name() {
        let status = parse_rate_limits_result(
            serde_json::json!({
                "rateLimits": {
                    "limitId": "codex",
                    "primary": { "usedPercent": 25, "windowDurationMins": 300, "resetsAt": 1779459394 },
                    "secondary": { "usedPercent": 18, "windowDurationMins": 10080, "resetsAt": 1779826837 },
                    "planType": "plus"
                }
            }),
            "codex",
        )
        .expect("parse");

        let snippet = rate_limit_status_snippet(&status);
        assert!(snippet.contains("5h 25% used"));
        assert!(snippet.contains("weekly 18% used"));
        assert!(!snippet.contains("primary="));
        assert!(!snippet.contains("secondary="));
    }

    #[test]
    fn merges_sparse_rate_limits_with_rate_limits_by_limit_id() {
        let status = parse_rate_limits_result(
            serde_json::json!({
                "rateLimits": {
                    "limitId": "codex",
                    "primary": null,
                    "secondary": null,
                    "planType": "plus"
                },
                "rateLimitsByLimitId": {
                    "codex": {
                        "limitId": "codex",
                        "primary": {
                            "usedPercent": 12,
                            "windowDurationMins": 10080,
                            "resetsAt": 1784766572
                        },
                        "secondary": null,
                        "planType": "plus"
                    }
                }
            }),
            "codex",
        )
        .expect("parse");

        let limits = status.rate_limits.expect("limits");
        let primary = limits.primary.expect("primary");
        assert_eq!(primary.used_percent, Some(12.0));
        assert_eq!(primary.window_duration_mins, Some(10_080));
    }

    #[test]
    fn accepts_window_minutes_alias_from_session_logs() {
        let status = parse_rate_limits_result(
            serde_json::json!({
                "rateLimits": {
                    "limitId": "codex",
                    "primary": { "usedPercent": 2.0, "windowMinutes": 300, "resetsAt": 1783889970 },
                    "secondary": { "usedPercent": 47.0, "windowMinutes": 10080, "resetsAt": 1784358776 },
                    "planType": "plus"
                }
            }),
            "codex",
        )
        .expect("parse");

        let limits = status.rate_limits.expect("limits");
        assert_eq!(
            limits
                .primary
                .as_ref()
                .and_then(|window| window.window_duration_mins),
            Some(300)
        );
        assert_eq!(
            limits
                .secondary
                .as_ref()
                .and_then(|window| window.window_duration_mins),
            Some(10_080)
        );
    }

    #[test]
    fn parses_weekly_only_rate_limits_payload() {
        let payload = serde_json::json!({
            "rateLimits": {
                "limitId": "codex",
                "limitName": null,
                "primary": {
                    "usedPercent": 42.5,
                    "windowDurationMins": 10080,
                    "resetsAt": 1784766572
                },
                "secondary": null,
                "credits": {
                    "hasCredits": false,
                    "unlimited": false,
                    "balance": "0"
                },
                "individualLimit": null,
                "planType": "plus",
                "rateLimitReachedType": null
            },
            "rateLimitsByLimitId": {
                "codex": {
                    "limitId": "codex",
                    "primary": {
                        "usedPercent": 42.5,
                        "windowDurationMins": 10080,
                        "resetsAt": 1784766572
                    },
                    "secondary": null,
                    "planType": "plus"
                }
            },
            "rateLimitResetCredits": {
                "availableCount": 2
            }
        });

        let status = parse_rate_limits_result(payload, "codex").expect("parse");
        assert!(status.ok);
        assert!(rate_limit_status_snippet(&status).contains("weekly 42% used"));
        let limits = status.rate_limits.expect("limits");
        let primary = limits.primary.expect("primary");
        assert_eq!(primary.window_duration_mins, Some(10_080));
        assert_eq!(primary.used_percent, Some(42.5));
        assert!(limits.secondary.is_none());
    }

    #[test]
    fn detects_new_rate_limit_reached_type_values() {
        let status = parse_rate_limits_result(
            serde_json::json!({
                "rateLimits": {
                    "limitId": "codex",
                    "primary": { "usedPercent": 100, "windowDurationMins": 10080, "resetsAt": 1784766572 },
                    "secondary": null,
                    "planType": "plus",
                    "rateLimitReachedType": "rate_limit_reached"
                }
            }),
            "codex",
        )
        .expect("parse");

        assert!(is_rate_limit_reached(&status));
        assert_eq!(
            rate_limit_reached_type(&status).as_deref(),
            Some("rate_limit_reached")
        );
    }

    #[test]
    fn parses_thread_list_payload() {
        let threads = extract_thread_rows(&serde_json::json!({
            "data": [
                {
                    "id": "thr_abc",
                    "name": "Fix auth bug",
                    "status": "notLoaded",
                    "path": "/tmp/thread.json",
                    "createdAt": "2026-06-01T10:00:00Z",
                    "model": "gpt-5.4"
                }
            ]
        }));
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].id, "thr_abc");
        assert_eq!(threads[0].name.as_deref(), Some("Fix auth bug"));
    }

    #[test]
    fn auto_discovers_threads_or_local_store_on_this_machine() {
        let config = LauncherConfig::default();
        let status = list_threads(&config);
        if status.ok {
            assert!(
                !status.threads.is_empty(),
                "expected threads from {:?}",
                status.source
            );
            return;
        }
        assert!(
            status.error.is_some(),
            "expected either app-server or local session index data"
        );
    }

    #[test]
    fn auto_discovers_rate_limits_on_this_machine() {
        let config = LauncherConfig::default();
        let status = read_rate_limits(&config);
        if !status.ok {
            assert!(
                status.error.is_some(),
                "expected a discovery error when app-server is unavailable"
            );
            return;
        }

        let limits = status
            .rate_limits
            .as_ref()
            .expect("expected rate limits payload");
        assert!(
            !collect_usage_windows(limits).is_empty(),
            "expected at least one populated usage window"
        );
    }

    #[test]
    fn parses_rate_limits_payload() {
        let payload = serde_json::json!({
            "rateLimits": {
                "limitId": "codex",
                "primary": { "usedPercent": 25, "windowDurationMins": 300, "resetsAt": 1779459394 },
                "secondary": { "usedPercent": 18, "windowDurationMins": 10080, "resetsAt": 1779826837 },
                "credits": { "hasCredits": true, "unlimited": false, "balance": "12.50" },
                "planType": "plus",
                "rateLimitReachedType": null
            },
            "rateLimitResetCredits": { "availableCount": 2 }
        });

        let status = parse_rate_limits_result(payload, "codex").expect("parse");
        assert!(status.ok);
        let limits = status.rate_limits.expect("limits");
        assert_eq!(limits.plan_type.as_deref(), Some("plus"));
        assert_eq!(
            limits
                .primary
                .as_ref()
                .and_then(|window| window.used_percent),
            Some(25.0)
        );
        assert_eq!(
            status
                .rate_limit_reset_credits
                .and_then(|credits| credits.available_count),
            Some(2)
        );
    }
}
