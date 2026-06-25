use crate::app_logic;
use crate::codex_process;
use crate::config::LauncherConfig;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tao::dpi::LogicalSize;
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use wry::WebView;
use wry::WebViewAttributes;

#[derive(Clone)]
pub struct RpcState {
    pub config_path: PathBuf,
    pub root: PathBuf,
}

const ICON_FILENAME: &str = "icon.ico";
const ICON_PNG: &str = "icon.png";
const DIST_DIR: &str = "web/dist";
const DIST_INDEX: &str = "web/dist/index.html";
const MAX_RPC_BODY_BYTES: usize = 64 * 1024;
const MAX_LOG_ENTRIES: usize = 500;

macro_rules! gui_log {
    ($root:expr, $level:expr, $($arg:tt)*) => {{
        write_gui_log($root, $level, &format!($($arg)*));
    }};
}

/// Locate the directory that contains `web/dist/index.html` at runtime.
/// The GUI must not rely on compile-time `CARGO_MANIFEST_DIR` ΓÇö the binary may
/// be launched from `target/release/` or another checkout.
pub fn resolve_gui_root() -> PathBuf {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf());
        for _ in 0..6 {
            if let Some(ref d) = dir {
                candidates.push(d.clone());
                dir = d.parent().map(|p| p.to_path_buf());
            } else {
                break;
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd);
    }

    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    for root in candidates {
        if root.join(DIST_INDEX).is_file() {
            eprintln!("[GUI] Resolved app root: {}", root.display());
            return root;
        }
    }

    let fallback = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    eprintln!(
        "[GUI] Warning: {} not found; falling back to {}",
        DIST_INDEX,
        fallback.display()
    );
    fallback
}

fn resolve_icon_path(root: &Path) -> PathBuf {
    let candidates = [
        root.join("assets").join(ICON_FILENAME),
        root.join("assets").join(ICON_PNG),
        root.join(ICON_FILENAME),
        root.join(ICON_PNG),
        root.join("..").join("assets").join(ICON_FILENAME),
        root.join("..").join("assets").join(ICON_PNG),
    ];
    for candidate in &candidates {
        if candidate.exists() {
            return candidate.to_path_buf();
        }
    }
    root.join("assets").join(ICON_PNG)
}

fn load_icon(data: &[u8]) -> Option<tao::window::Icon> {
    if let Ok(img) = image::load_from_memory_with_format(data, image::ImageFormat::Ico) {
        let rgba = img.into_rgba8();
        let (w, h) = rgba.dimensions();
        return tao::window::Icon::from_rgba(rgba.into_raw(), w, h).ok();
    }
    if let Ok(img) = image::load_from_memory(data) {
        let rgba = img.into_rgba8();
        let (w, h) = rgba.dimensions();
        return tao::window::Icon::from_rgba(rgba.into_raw(), w, h).ok();
    }
    None
}

fn write_gui_log(root: Option<&Path>, level: &str, message: &str) {
    let sanitized = redact_sensitive_text(message);
    eprintln!("[{level}] {sanitized}");

    let Some(root) = root else {
        return;
    };

    let log_path = root.join("app.log");
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(file, "[{level}] {sanitized}");
    }
}

fn redact_sensitive_text(input: &str) -> String {
    let mut redacted = input.to_string();

    for key in ["apiKey", "api_key", "OPENAI_API_KEY"] {
        redacted = redact_quoted_field(&redacted, key);
    }

    for marker in [
        "Bearer ",
        "OPENAI_API_KEY=",
        "OPENAI_API_KEY: ",
        "apiKey=",
        "api_key=",
    ] {
        redacted = redact_token_after_marker(&redacted, marker);
    }

    redacted
}

fn redact_quoted_field(input: &str, field_name: &str) -> String {
    let patterns = [
        format!("\"{}\":", field_name),
        format!("\"{}\" =", field_name),
        format!("{field_name}:"),
        format!("{field_name} ="),
    ];

    let mut result = input.to_string();
    for pattern in patterns {
        result = redact_quoted_value_after_pattern(&result, &pattern);
    }
    result
}

fn redact_quoted_value_after_pattern(input: &str, pattern: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut cursor = 0;

    while let Some(relative_start) = input[cursor..].find(pattern) {
        let start = cursor + relative_start;
        let search_from = start + pattern.len();
        result.push_str(&input[cursor..search_from]);

        let remainder = &input[search_from..];
        let value_start = match remainder.find('"') {
            Some(index) => search_from + index,
            None => {
                cursor = search_from;
                continue;
            }
        };

        result.push_str(&input[search_from..=value_start]);
        let value_remainder = &input[value_start + 1..];
        let value_end = match value_remainder.find('"') {
            Some(index) => value_start + 1 + index,
            None => {
                cursor = value_start + 1;
                continue;
            }
        };

        result.push_str("<redacted>");
        cursor = value_end;
    }

    result.push_str(&input[cursor..]);
    result
}

fn redact_token_after_marker(input: &str, marker: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut cursor = 0;

    while let Some(relative_start) = input[cursor..].find(marker) {
        let start = cursor + relative_start;
        let value_start = start + marker.len();
        result.push_str(&input[cursor..value_start]);

        let remainder = &input[value_start..];
        let value_end = remainder
            .find(|c: char| {
                c.is_whitespace() || matches!(c, '"' | '\'' | ',' | ';' | ')' | ']' | '}')
            })
            .map(|index| value_start + index)
            .unwrap_or(input.len());

        result.push_str("<redacted>");
        cursor = value_end;
    }

    result.push_str(&input[cursor..]);
    result
}

fn ensure_dist_dir(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dist_dir = root.join(DIST_DIR);
    if !dist_dir.exists() {
        return Err("Web app not built. Run npm run build in the web/ directory.".into());
    }
    Ok(dist_dir)
}

fn spawn_server_thread(
    dist_dir: PathBuf,
    state: Arc<Mutex<RpcState>>,
    server: tiny_http::Server,
    thread_root: PathBuf,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        gui_log!(
            Some(thread_root.as_path()),
            "INFO",
            "HTTP server thread started"
        );
        for request in server.incoming_requests() {
            if let Err(_e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                handle_request(&dist_dir, &state, request);
            })) {
                gui_log!(
                    Some(thread_root.as_path()),
                    "ERROR",
                    "RPC handler panicked: {:?}",
                    _e
                );
            }
        }
    })
}

/// Headless HTTP server for Playwright and agent-driven E2E tests.
pub fn serve_web_ui(
    root: PathBuf,
    config_path: PathBuf,
    port: Option<u16>,
) -> Result<(), Box<dyn std::error::Error>> {
    gui_log!(Some(root.as_path()), "INFO", "Starting headless web backend");
    gui_log!(Some(root.as_path()), "INFO", "Root: {}", root.display());
    gui_log!(
        Some(root.as_path()),
        "INFO",
        "Config: {}",
        config_path.display()
    );

    let dist_dir = ensure_dist_dir(&root)?;
    let state = Arc::new(Mutex::new(RpcState {
        config_path,
        root: root.clone(),
    }));

    let (server, server_url) = start_server(port)?;
    println!("CODEX_LAUNCHER_READY={server_url}");
    gui_log!(
        Some(root.as_path()),
        "INFO",
        "HTTP server listening on {server_url}"
    );

    for request in server.incoming_requests() {
        if let Err(_e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            handle_request(&dist_dir, &state, request);
        })) {
            gui_log!(
                Some(root.as_path()),
                "ERROR",
                "RPC handler panicked: {:?}",
                _e
            );
        }
    }

    Ok(())
}

pub fn launch_web_gui(
    root: PathBuf,
    config_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    gui_log!(Some(root.as_path()), "INFO", "Starting web backend");
    gui_log!(Some(root.as_path()), "INFO", "Root: {}", root.display());
    gui_log!(
        Some(root.as_path()),
        "INFO",
        "Config: {}",
        config_path.display()
    );

    let dist_dir = ensure_dist_dir(&root)?;
    gui_log!(
        Some(root.as_path()),
        "INFO",
        "Dist dir: {} (exists: {})",
        dist_dir.display(),
        dist_dir.exists()
    );

    let state = Arc::new(Mutex::new(RpcState {
        config_path,
        root: root.clone(),
    }));

    let (server, server_url) = start_server(None)?;
    gui_log!(
        Some(root.as_path()),
        "INFO",
        "HTTP server started on: {}",
        server_url
    );

    let _server_handle = spawn_server_thread(dist_dir, Arc::clone(&state), server, root.clone());

    gui_log!(Some(root.as_path()), "INFO", "Server thread spawned");
    gui_log!(Some(root.as_path()), "INFO", "Creating event loop");
    let event_loop = EventLoop::new();
    gui_log!(Some(root.as_path()), "INFO", "Event loop created");

    // Load icon - try ICO first, fallback to PNG
    let icon_path = resolve_icon_path(&root);
    gui_log!(
        Some(root.as_path()),
        "INFO",
        "Icon path: {}",
        icon_path.display()
    );
    let icon_data = std::fs::read(&icon_path).unwrap_or_default();
    let icon = if icon_data.is_empty() {
        gui_log!(Some(root.as_path()), "WARN", "No icon found");
        None
    } else {
        gui_log!(Some(root.as_path()), "INFO", "Loading icon");
        load_icon(&icon_data)
    };
    gui_log!(
        Some(root.as_path()),
        "INFO",
        "Icon loaded: {}",
        icon.is_some()
    );

    // Create a single visible window with WebView embedded
    gui_log!(Some(root.as_path()), "INFO", "Creating window");
    let window_builder = WindowBuilder::new()
        .with_title("Codex Local Launcher")
        .with_visible(true)
        .with_inner_size(LogicalSize::new(1280.0, 800.0));

    let window = if let Some(ref icon) = icon {
        window_builder
            .with_window_icon(Some(icon.clone()))
            .build(&event_loop)?
    } else {
        window_builder.build(&event_loop)?
    };
    gui_log!(Some(root.as_path()), "INFO", "Window created");

    // Create WebView - visible, embedded in parent window
    gui_log!(Some(root.as_path()), "INFO", "Creating WebView");
    let mut attrs = WebViewAttributes {
        url: Some(server_url.parse()?),
        devtools: true,
        ..WebViewAttributes::default()
    };
    attrs.initialization_scripts.push(
        r#"window.onerror = function(msg, url, line, col, err) {
            console.error("JS Error:", msg, "at", url, ":", line, ":", col);
          };
        window.addEventListener("unhandledrejection", function(e) {
            console.error("Unhandled rejection:", e.reason);
          });"#
            .to_string(),
    );

    let _webview = WebView::new(&window, attrs)?;
    gui_log!(Some(root.as_path()), "INFO", "WebView created");
    window.set_visible(true);
    gui_log!(Some(root.as_path()), "INFO", "Window shown");

    gui_log!(Some(root.as_path()), "INFO", "Running event loop");
    let _pid_file = codex_process::CodexProcess::spawn_pid_file_path(&root);
    event_loop.run(move |_, _, control_flow| {
        *control_flow = ControlFlow::Wait;
    });
}

fn start_server(
    port: Option<u16>,
) -> Result<(tiny_http::Server, String), Box<dyn std::error::Error>> {
    let listener = match port {
        Some(port) => TcpListener::bind(format!("127.0.0.1:{port}"))?,
        None => TcpListener::bind("127.0.0.1:0")?,
    };
    let port = listener.local_addr()?.port();
    let server = tiny_http::Server::from_listener(listener, None)
        .map_err(|e| Box::new(std::io::Error::other(e)))?;
    let url = format!("http://127.0.0.1:{}", port);
    Ok((server, url))
}

fn handle_rpc(state: &RpcState, method: &str, params: serde_json::Value) -> serde_json::Value {
    match method {
        "loadConfig" => rpc_load_config(state),
        "saveConfig" => rpc_save_config(state, params),
        "launch" => rpc_launch(state, params),
        "stop" => rpc_stop(state),
        "healthCheck" => rpc_health_check(state, params),
        "listModels" => rpc_list_models(state),
        "refreshModels" => rpc_refresh_models(state, params),
        "writeCodexConfig" => rpc_write_codex_config(state),
        "revertCodexConfig" => rpc_revert_codex_config(state),
        "detectCodex" => rpc_detect_codex(state),
        "killCodexByPid" => rpc_kill_codex_by_pid(params),
        "toggleAutoStart" => rpc_toggle_auto_start(state),
        "openDirectoryPicker" => rpc_open_directory_picker(),
        "getAppLogs" => rpc_get_app_logs(state),
        _ => serde_json::json!({"error": format!("Unknown method: {}", method)}),
    }
}

fn rpc_load_config(state: &RpcState) -> serde_json::Value {
    match LauncherConfig::read(&state.config_path) {
        Ok(c) => serde_json::to_value(c)
            .unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize config"})),
        Err(e) => serde_json::json!({"error": format!("Cannot read config: {}", e)}),
    }
}

fn config_for_request(
    state: &RpcState,
    params: &serde_json::Value,
) -> Result<LauncherConfig, String> {
    let incoming = serde_json::from_value::<LauncherConfig>(params.clone()).ok();
    let has_overlay = incoming.as_ref().is_some_and(|cfg| {
        cfg.ollama_ip.is_some()
            || cfg.openai_base_url.is_some()
            || cfg.ollama_port.is_some()
            || cfg.ollama_scheme.is_some()
            || cfg.codex_model.is_some()
    });

    if has_overlay {
        let Some(incoming) = incoming else {
            return Err("Invalid config JSON".to_string());
        };
        match LauncherConfig::read(&state.config_path) {
            Ok(mut existing) => {
                existing.merge_from(&incoming);
                Ok(existing)
            }
            Err(_) => Ok(incoming),
        }
    } else {
        LauncherConfig::read(&state.config_path).map_err(|e| e.to_string())
    }
}

fn rpc_save_config(state: &RpcState, params: serde_json::Value) -> serde_json::Value {
    let incoming: LauncherConfig = match serde_json::from_value(params) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": format!("Invalid config JSON: {}", e)}),
    };
    let mut config = LauncherConfig::read(&state.config_path).unwrap_or_default();
    config.merge_from(&incoming);
    match config.write(&state.config_path) {
        Ok(_) => serde_json::json!({"ok": true}),
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

fn rpc_launch(state: &RpcState, params: serde_json::Value) -> serde_json::Value {
    let incoming = serde_json::from_value::<LauncherConfig>(params).ok();
    let has_overlay = incoming.as_ref().is_some_and(|cfg| {
        cfg.ollama_ip.is_some()
            || cfg.openai_base_url.is_some()
            || cfg.ollama_port.is_some()
            || cfg.ollama_scheme.is_some()
            || cfg.codex_model.is_some()
    });
    let config = if has_overlay {
        match LauncherConfig::read(&state.config_path) {
            Ok(mut existing) => {
                if let Some(ref incoming) = incoming {
                    existing.merge_from(incoming);
                }
                existing
            }
            Err(_) => incoming.unwrap_or_default(),
        }
    } else {
        match LauncherConfig::read(&state.config_path) {
            Ok(c) => c,
            Err(e) => return serde_json::json!({"error": format!("Cannot read config: {}", e)}),
        }
    };
    if let Err(e) = app_logic::write_config(&config) {
        return serde_json::json!({"error": e.to_string()});
    }
    let pid_file = codex_process::CodexProcess::spawn_pid_file_path(&state.root);
    match app_logic::launch(&config, &state.root, &pid_file) {
        Ok(message) => serde_json::json!({"ok": true, "message": message}),
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

fn rpc_stop(state: &RpcState) -> serde_json::Value {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": format!("Cannot read config: {}", e)}),
    };
    let pid_file = codex_process::CodexProcess::spawn_pid_file_path(&state.root);
    match app_logic::stop_codex(&config, &state.root, &pid_file) {
        Ok(msg) => serde_json::json!({"ok": true, "message": msg}),
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

fn rpc_health_check(state: &RpcState, params: serde_json::Value) -> serde_json::Value {
    let config = match config_for_request(state, &params) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": e}),
    };
    let codex = app_logic::detect_codex_process(&config, &state.root);
    let endpoint_ready = app_logic::endpoint_reachable(&config);
    let api_ready = if codex.running {
        app_logic::codex_api_ready(&config)
    } else {
        false
    };
    serde_json::json!({
        "running": codex.running,
        "apiReady": api_ready,
        "endpointReady": endpoint_ready,
        "pid": codex.pid,
        "method": codex.method,
    })
}

fn rpc_list_models(state: &RpcState) -> serde_json::Value {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": format!("Cannot read config: {}", e)}),
    };
    match app_logic::list_models(&config) {
        Ok(cache) => {
            let models: Vec<serde_json::Value> = cache
                .models
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "name": m.name.clone(),
                        "size": m.size.unwrap_or(0),
                        "digest": m.digest.clone().unwrap_or_default(),
                        "modified": m.modified_at.clone().unwrap_or_default(),
                    })
                })
                .collect();
            serde_json::json!({"models": models})
        }
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

fn rpc_refresh_models(state: &RpcState, params: serde_json::Value) -> serde_json::Value {
    let config = match config_for_request(state, &params) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": e}),
    };
    let endpoint = match config.openai_base_url() {
        Ok(url) => url,
        Err(e) => return serde_json::json!({"error": e.to_string()}),
    };
    match app_logic::refresh_models(&config) {
        Ok(cache) => {
            let models: Vec<serde_json::Value> = cache
                .models
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "name": m.name.clone(),
                        "size": m.size.unwrap_or(0),
                        "digest": m.digest.clone().unwrap_or_default(),
                        "modified": m.modified_at.clone().unwrap_or_default(),
                    })
                })
                .collect();
            serde_json::json!({
                "ok": true,
                "models": models,
                "endpoint": endpoint,
                "fetchedFrom": cache.fetched_from,
                "message": format!("Found {} model(s) from {}", models.len(), cache.fetched_from),
            })
        }
        Err(e) => serde_json::json!({
            "error": format!("{e} (endpoint: {endpoint})"),
        }),
    }
}

fn rpc_write_codex_config(state: &RpcState) -> serde_json::Value {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": format!("Cannot read config: {}", e)}),
    };
    match app_logic::write_config(&config) {
        Ok(msg) => serde_json::json!({"ok": true, "message": msg}),
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

fn rpc_revert_codex_config(state: &RpcState) -> serde_json::Value {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": format!("Cannot read config: {}", e)}),
    };
    match app_logic::revert_codex_config(&config) {
        Ok(msg) => serde_json::json!({"ok": true, "message": msg}),
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

fn rpc_detect_codex(state: &RpcState) -> serde_json::Value {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": format!("Cannot read config: {}", e)}),
    };
    let info = app_logic::detect_codex_process(&config, &state.root);
    serde_json::to_value(info)
        .unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize"}))
}

fn rpc_kill_codex_by_pid(params: serde_json::Value) -> serde_json::Value {
    let pid: u32 = match params.get("pid").and_then(|p| p.as_u64()) {
        Some(p) => p as u32,
        None => return serde_json::json!({"error": "Missing pid parameter"}),
    };
    match app_logic::kill_codex_by_pid_number(pid) {
        Ok(msg) => serde_json::json!({"ok": true, "message": msg}),
        Err(e) => serde_json::json!({"error": e.to_string()}),
    }
}

fn rpc_open_directory_picker() -> serde_json::Value {
    match rfd::FileDialog::new().pick_folder() {
        Some(path) => serde_json::json!({"path": path.to_string_lossy().to_string()}),
        None => serde_json::json!({"path": ""}),
    }
}

fn rpc_get_app_logs(state: &RpcState) -> serde_json::Value {
    let logs_path = state.root.join("app.log");
    let entries: Vec<serde_json::Value> = match std::fs::read_to_string(&logs_path) {
        Ok(content) => content
            .lines()
            .rev()
            .take(MAX_LOG_ENTRIES)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let (level, message) = if let Some(rest) = line.strip_prefix('[') {
                    if let Some((lvl, msg)) = rest.split_once(']') {
                        (lvl.trim().to_string(), redact_sensitive_text(msg.trim()))
                    } else {
                        ("INFO".to_string(), redact_sensitive_text(line))
                    }
                } else {
                    ("INFO".to_string(), redact_sensitive_text(line))
                };
                serde_json::json!({"level": level, "message": message})
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    serde_json::json!({"logs": entries})
}

fn rpc_toggle_auto_start(state: &RpcState) -> serde_json::Value {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": format!("Cannot read config: {}", e)}),
    };
    let enabled = config.auto_start.unwrap_or(false);
    if enabled {
        match app_logic::disable_auto_start(&config) {
            Ok(msg) => serde_json::json!({"ok": true, "enabled": false, "message": msg}),
            Err(e) => serde_json::json!({"error": e.to_string()}),
        }
    } else {
        match app_logic::enable_auto_start(&config) {
            Ok(msg) => serde_json::json!({"ok": true, "enabled": true, "message": msg}),
            Err(e) => serde_json::json!({"error": e.to_string()}),
        }
    }
}

fn make_json_response(
    status: u16,
    data: Option<serde_json::Value>,
    error: Option<String>,
) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    let body = serde_json::json!({"ok": data.is_some(), "data": data, "error": error});
    let body_bytes = serde_json::to_string(&body)
        .unwrap_or_default()
        .into_bytes();
    tiny_http::Response::new(
        tiny_http::StatusCode(status),
        vec![
            tiny_http::Header::from_bytes(b"Content-Type", b"application/json; charset=utf-8")
                .unwrap(),
            tiny_http::Header::from_bytes(b"Cache-Control", b"no-cache").unwrap(),
        ],
        std::io::Cursor::new(body_bytes),
        None,
        None,
    )
}

fn make_file_response(
    status: u16,
    content_type: &str,
    data: Vec<u8>,
) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    tiny_http::Response::new(
        tiny_http::StatusCode(status),
        vec![
            tiny_http::Header::from_bytes(b"Content-Type", content_type.as_bytes()).unwrap(),
            tiny_http::Header::from_bytes(b"Cache-Control", b"no-cache").unwrap(),
        ],
        std::io::Cursor::new(data),
        None,
        None,
    )
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("css") => "text/css",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}

fn handle_request(dist_dir: &Path, state: &Arc<Mutex<RpcState>>, mut request: tiny_http::Request) {
    let uri = request.url();

    if uri == "/rpc" && *request.method() != tiny_http::Method::Post {
        let _ = request.respond(make_json_response(
            405,
            None,
            Some("Method Not Allowed".to_string()),
        ));
        return;
    }

    if uri == "/rpc" && *request.method() == tiny_http::Method::Post {
        if request
            .body_length()
            .is_some_and(|length| length > MAX_RPC_BODY_BYTES)
        {
            let _ = request.respond(make_json_response(
                413,
                None,
                Some("RPC request body too large".to_string()),
            ));
            return;
        }

        let mut limited_reader = request.as_reader().take((MAX_RPC_BODY_BYTES + 1) as u64);
        let body_str = std::io::read_to_string(&mut limited_reader).unwrap_or_default();
        if body_str.len() > MAX_RPC_BODY_BYTES {
            let _ = request.respond(make_json_response(
                413,
                None,
                Some("RPC request body too large".to_string()),
            ));
            return;
        }
        let request_data: serde_json::Value = match serde_json::from_str(&body_str) {
            Ok(d) => d,
            Err(e) => {
                let resp = make_json_response(400, None, Some(format!("Invalid JSON: {}", e)));
                let _ = request.respond(resp);
                return;
            }
        };
        let method = request_data
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("");
        let params = request_data
            .get("params")
            .unwrap_or(&serde_json::Value::Null);
        let result = match state.lock() {
            Ok(guard) => handle_rpc(&guard, method, params.clone()),
            Err(poisoned) => handle_rpc(&poisoned.into_inner(), method, params.clone()),
        };
        let resp = make_json_response(200, Some(result), None);
        let _ = request.respond(resp);
        return;
    }

    let clean_path = uri.trim_start_matches("/").trim_start_matches("index.html");
    let sanitized = clean_path
        .split("/")
        .filter(|s| *s != "." && *s != "..")
        .collect::<Vec<_>>()
        .join("/");
    let file_path = dist_dir.join(sanitized);

    if !file_path.starts_with(dist_dir) {
        let _ = request.respond(make_json_response(403, None, Some("Forbidden".to_string())));
        return;
    }

    let file_path = if clean_path.is_empty() || clean_path == "index.html" {
        dist_dir.join("index.html")
    } else {
        file_path
    };

    match std::fs::read(&file_path) {
        Ok(contents) => {
            let ct = content_type_for(&file_path);
            let resp = make_file_response(200, ct, contents);
            let _ = request.respond(resp);
        }
        Err(_) => {
            if let Ok(index_html) = std::fs::read(dist_dir.join("index.html")) {
                let resp = make_file_response(200, "text/html", index_html);
                let _ = request.respond(resp);
            } else {
                let resp = make_json_response(404, None, Some("Not Found".to_string()));
                let _ = request.respond(resp);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::redact_sensitive_text;

    #[test]
    fn redacts_json_api_keys() {
        let input = r#"{"apiKey":"secret-value","nested":{"api_key":"abc123"}}"#;
        let output = redact_sensitive_text(input);
        assert!(!output.contains("secret-value"));
        assert!(!output.contains("abc123"));
        assert!(output.contains(r#""apiKey":"<redacted>""#));
        assert!(output.contains(r#""api_key":"<redacted>""#));
    }

    #[test]
    fn redacts_bearer_and_env_tokens() {
        let input = "Authorization: Bearer secret-token OPENAI_API_KEY=super-secret";
        let output = redact_sensitive_text(input);
        assert!(!output.contains("secret-token"));
        assert!(!output.contains("super-secret"));
        assert!(output.contains("Bearer <redacted>"));
        assert!(output.contains("OPENAI_API_KEY=<redacted>"));
    }
}
