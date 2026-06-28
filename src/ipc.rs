use crate::app_logic;
#[allow(unused_imports)]
use crate::codex_process::CodexProcess;
use crate::config::LauncherConfig;
use std::path::PathBuf;

/// Shared state passed through IPC.
pub struct IpcState {
    pub config_path: PathBuf,
    pub root: PathBuf,
    pub codex_process: Option<CodexProcess>,
}

/// An IPC request from the frontend.
#[derive(serde::Deserialize)]
pub struct IpcRequest {
    pub r#type: String,
    pub payload: serde_json::Value,
}

/// An IPC response back to the frontend.
#[derive(serde::Serialize)]
pub struct IpcResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl IpcResponse {
    fn ok(data: impl Into<serde_json::Value>) -> Self {
        Self {
            success: true,
            data: Some(data.into()),
            error: None,
        }
    }

    fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

/// Handle an IPC message from the frontend.
pub fn handle(state: &IpcState, request: &IpcRequest) -> IpcResponse {
    match request.r#type.as_str() {
        "launch" => do_launch(state),
        "stop" => do_stop(state),
        "save_config" => do_save_config(state, &request.payload),
        "load_config" => do_load_config(state),
        "health_check" => do_health_check(state),
        "list_models" => do_list_models(state),
        "refresh_models" => do_refresh_models(state),
        "write_codex_config" => do_write_codex_config(state),
        "revert_codex_config" => do_revert_codex_config(state),
        "detect_codex" => do_detect_codex(state),
        "kill_codex_by_pid" => do_kill_codex_by_pid(state, request),
        _ => IpcResponse::err(format!("Unknown command: {}", request.r#type)),
    }
}

fn do_launch(state: &IpcState) -> IpcResponse {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return IpcResponse::err(format!("Cannot read config: {e}")),
    };

    if let Err(e) = app_logic::write_config_for_launch(&config) {
        return IpcResponse::err(format!("Failed to write Codex config: {e}"));
    }

    let pid_file = app_logic::codex_pid_file(&state.config_path);

    match app_logic::launch(&config, &state.root, &pid_file) {
        Ok(msg) => IpcResponse::ok(serde_json::json!({ "message": msg })),
        Err(e) => IpcResponse::err(format!("Failed to launch: {e}")),
    }
}

fn do_stop(state: &IpcState) -> IpcResponse {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return IpcResponse::err(format!("Cannot read config: {e}")),
    };
    let pid_file = app_logic::codex_pid_file(&state.config_path);
    match app_logic::stop_codex(&config, &state.root, &pid_file) {
        Ok(msg) => IpcResponse::ok(serde_json::json!({ "message": msg })),
        Err(e) => IpcResponse::err(format!("Failed to stop: {e}")),
    }
}

fn do_save_config(state: &IpcState, payload: &serde_json::Value) -> IpcResponse {
    let config: LauncherConfig = match serde_json::from_value(payload.clone()) {
        Ok(c) => c,
        Err(e) => return IpcResponse::err(format!("Invalid config: {e}")),
    };

    match LauncherConfig::write(&config, &state.config_path) {
        Ok(()) => IpcResponse::ok(serde_json::json!({ "message": "Config saved." })),
        Err(e) => IpcResponse::err(e.to_string()),
    }
}

fn do_load_config(state: &IpcState) -> IpcResponse {
    match LauncherConfig::read(&state.config_path) {
        Ok(config) => {
            IpcResponse::ok(serde_json::to_value(&config).unwrap_or_else(|_| serde_json::json!({})))
        }
        Err(e) => IpcResponse::err(format!("Cannot read config: {e}")),
    }
}

fn do_health_check(state: &IpcState) -> IpcResponse {
    // Check if Codex API is responding via blocking HTTP
    let base_url = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c.codex_api_base_url(),
        Err(_) => "http://127.0.0.1:4000".to_string(),
    };

    let health_url = format!("{}/health", base_url);
    match reqwest::blocking::Client::new()
        .get(&health_url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
    {
        Ok(response) => {
            let api_ready = response.status().is_success();
            IpcResponse::ok(serde_json::json!({
                "running": true,
                "api_ready": api_ready,
            }))
        }
        Err(_) => IpcResponse::ok(serde_json::json!({
            "running": false,
            "api_ready": false,
        })),
    }
}

fn do_list_models(state: &IpcState) -> IpcResponse {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return IpcResponse::err(format!("Failed to list models: {e}")),
    };

    match app_logic::list_models(&config) {
        Ok(cache) => {
            let models: Vec<serde_json::Value> = cache
                .models
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "name": m.name.clone(),
                        "size": m.size.unwrap_or(0) as i64,
                        "digest": m.digest.as_deref().unwrap_or(""),
                        "modified": m.modified_at.as_deref().unwrap_or(""),
                    })
                })
                .collect();
            IpcResponse::ok(
                serde_json::json!({ "models": models, "fetched_from": cache.fetched_from }),
            )
        }
        Err(e) => IpcResponse::err(format!("Failed to list models: {e}")),
    }
}

fn do_refresh_models(state: &IpcState) -> IpcResponse {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return IpcResponse::err(format!("Failed to refresh models: {e}")),
    };

    match app_logic::refresh_models(&config) {
        Ok(cache) => {
            let models: Vec<serde_json::Value> = cache
                .models
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "name": m.name.clone(),
                        "size": m.size.unwrap_or(0) as i64,
                        "digest": m.digest.as_deref().unwrap_or(""),
                        "modified": m.modified_at.as_deref().unwrap_or(""),
                    })
                })
                .collect();
            IpcResponse::ok(
                serde_json::json!({ "models": models, "fetched_from": cache.fetched_from }),
            )
        }
        Err(e) => IpcResponse::err(format!("Failed to refresh models: {e}")),
    }
}

fn do_write_codex_config(state: &IpcState) -> IpcResponse {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return IpcResponse::err(format!("Cannot read config: {e}")),
    };

    match app_logic::write_config(&config) {
        Ok(msg) => IpcResponse::ok(serde_json::json!({ "message": msg })),
        Err(e) => IpcResponse::err(format!("Failed to write Codex config: {e}")),
    }
}

fn do_revert_codex_config(state: &IpcState) -> IpcResponse {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return IpcResponse::err(format!("Cannot read config: {e}")),
    };

    match app_logic::revert_codex_config(&config) {
        Ok(msg) => IpcResponse::ok(serde_json::json!({ "message": msg })),
        Err(e) => IpcResponse::err(format!("Failed to revert Codex config: {e}")),
    }
}

fn do_detect_codex(state: &IpcState) -> IpcResponse {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return IpcResponse::err(format!("Cannot read config: {e}")),
    };

    let info = app_logic::detect_codex_process(&config, &state.root);
    IpcResponse::ok(serde_json::json!({
        "running": info.running,
        "pid": info.pid,
        "method": info.method,
        "restart_required": info.restart_required,
    }))
}

fn do_kill_codex_by_pid(_state: &IpcState, request: &IpcRequest) -> IpcResponse {
    let pid = match request.payload.get("pid").and_then(|p| p.as_u64()) {
        Some(p) => p as u32,
        None => return IpcResponse::err("No PID provided".to_string()),
    };

    match app_logic::kill_codex_by_pid_number(pid) {
        Ok(msg) => IpcResponse::ok(serde_json::json!({ "message": msg })),
        Err(e) => IpcResponse::err(format!("Failed to kill: {e}")),
    }
}
