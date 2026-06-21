const fs = require("fs");
const content = `use crate::app_logic;

use crate::config::LauncherConfig;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use wry::WebView;
use wry::WebViewAttributes;

#[derive(Clone)]
pub struct RpcState {
    pub config_path: PathBuf,
    pub root: PathBuf,
}

const ICON_FILENAME: &str = "icon.png";
const DIST_DIR: &str = "web/dist";

fn resolve_icon_path(root: &Path) -> PathBuf {
    let candidates = [
        root.join("assets").join(ICON_FILENAME),
        root.join(ICON_FILENAME),
        root.join("..").join("assets").join(ICON_FILENAME),
       ];
    for candidate in &candidates {
        if candidate.exists() { return candidate.to_path_buf(); }
        }
    root.join("assets").join(ICON_FILENAME)
}

pub fn launch_web_gui(root: PathBuf, config_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[GUI] Starting web backend...");
    eprintln!("[GUI] Root: {}", root.display());
    eprintln!("[GUI] Config: {}", config_path.display());

    let dist_dir = root.join(DIST_DIR);
    if !dist_dir.exists() {
        return Err("Web app not built. Run npx vite build in the web/ directory.".into());
        }
    eprintln!("[GUI] Dist dir: {} (exists: {})", dist_dir.display(), dist_dir.exists());

    let state = Arc::new(Mutex::new(RpcState { config_path, root: root.clone() }));
    let (server, server_url) = start_server(&dist_dir, Arc::clone(&state))?;
    eprintln!("[GUI] HTTP server started on: {}", server_url);

    let _server_handle = std::thread::spawn(move || {
        eprintln!("[GUI] HTTP server thread started");
        for request in server.incoming_requests() {
            if let Err(_e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                handle_request(&dist_dir, &state, request);
                })) {
                eprintln!("[GUI] RPC handler panicked: {:?}", _e);
                }
             }
          });
    eprintln!("[GUI] Server thread spawned");

    eprintln!("[GUI] Creating event loop...");
    let event_loop = EventLoop::new();
    eprintln!("[GUI] Event loop created");

    let icon_path = resolve_icon_path(&root);
    eprintln!("[GUI] Icon path: {}", icon_path.display());
    let icon_data = std::fs::read(&icon_path).unwrap_or_default();
    let icon = if icon_data.is_empty() { None } else {
        eprintln!("[GUI] Loading icon...");
        image::load_from_memory(&icon_data[..]).ok()
                .map(|img| {
                let rgba = img.into_rgba8();
                let (w, h) = rgba.dimensions();
                tao::window::Icon::from_rgba(rgba.into_raw(), w, h).ok()
                }).flatten()
            };
    eprintln!("[GUI] Icon loaded: {}", icon.is_some());

    eprintln!("[GUI] Creating window...");
    let mut window_builder = WindowBuilder::new()
         .with_title("Codex Local Launcher");
    if let Some(icon) = icon { window_builder = window_builder.with_window_icon(Some(icon)); }
    let window = window_builder
         .with_inner_size(LogicalSize::new(1280.0, 800.0))
         .build(&event_loop)?;
    eprintln!("[GUI] Window created");

    eprintln!("[GUI] Creating WebView...");
    let mut attrs = WebViewAttributes::default();
    attrs.url = Some(server_url.parse()?);
    eprintln!("[GUI] WebView URL: {}", server_url);
    attrs.initialization_scripts.push(r#"
        window.onerror = function(msg, url, line, col, err) {
            console.error("JS Error:", msg, "at", url, ":", line, ":", col);
         };
        window.codexRPC = {
            call: function(method, params) {
                return fetch("/rpc", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ method: method, params: params || {} })
                    }).then(function(r) { return r.json(); })
                        .then(function(result) {
                        if (result.error) throw new Error(result.error);
                        return result;
                        });
                },
            launch: function() { return this.call("launch", {}); },
            stop: function() { return this.call("stop", {}); },
            saveConfig: function(cfg) { return this.call("saveConfig", cfg); },
            loadConfig: function() { return this.call("loadConfig", {}); },
            healthCheck: function() { return this.call("healthCheck", {}); },
            listModels: function() { return this.call("listModels", {}); },
            refreshModels: function() { return this.call("refreshModels", {}); },
            writeCodexConfig: function() { return this.call("writeCodexConfig", {}); },
            revertCodexConfig: function() { return this.call("revertCodexConfig", {}); },
            detectCodex: function() { return this.call("detectCodex", {}); },
            killCodexByPid: function(pid) { return this.call("killCodexByPid", { pid: pid }); },
            openDirectoryPicker: function() { return this.call("open_directory_picker", {}); },
            getAppLogs: function() { return this.call("get_app_logs", {}); },
            saveSettings: function(settings) { return this.call("save_settings", settings); },
            toggleAutoStart: function() { return this.call("toggleAutoStart", {}); },
             };
        "#.to_string());
    eprintln!("[GUI] WebView URL set");
    let _webview = WebView::new(&window, attrs)?;
    eprintln!("[GUI] WebView created successfully");

    eprintln!("[GUI] Starting event loop...");
    event_loop.run(move |event, _, control_flow| {
         *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                eprintln!("[GUI] Close requested");
                 *control_flow = ControlFlow::Exit;
              }
            Event::WindowEvent { event: WindowEvent::Destroyed, .. } => {
                eprintln!("[GUI] Window destroyed");
                 *control_flow = ControlFlow::Exit;
              }
                 _ => {}
             }
           });
    eprintln!("[GUI] Event loop ended");
          #[allow(unreachable_code)]
    drop(_webview);
    Ok(())
}

fn start_server(
    dist_dir: &Path,
    state: Arc<Mutex<RpcState>>,
) -> Result<(tiny_http::Server, String), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let server_url = format!(
         "http://127.0.0.1:{}",
        listener.local_addr()?.port()
      );
    eprintln!("[GUI] Listening on {}", server_url);
    Ok((tiny_http::Server::from_listener(listener), server_url))
}

fn handle_rpc(state: &RpcState, method: &str, params: serde_json::Value) -> serde_json::Value {
    match method {
         "launch" => rpc_launch(state),
         "stop" => rpc_stop(state),
         "saveConfig" => rpc_save_config(state, params),
         "loadConfig" => rpc_load_config(state),
         "healthCheck" => rpc_health_check(state),
         "listModels" => rpc_list_models(state),
         "refreshModels" => rpc_refresh_models(state),
         "writeCodexConfig" => rpc_write_codex_config(state),
         "revertCodexConfig" => rpc_revert_codex_config(state),
         "detectCodex" => rpc_detect_codex(state),
         "killCodexByPid" => rpc_kill_codex_by_pid(state, params),
         "open_directory_picker" => rpc_open_directory_picker(state),
         "get_app_logs" => rpc_get_app_logs(state),
         "save_settings" => rpc_save_settings(state, params),
         "toggleAutoStart" => rpc_toggle_auto_start(state),
         _ => serde_json::json!({"error": format!("Unknown method: {}", method)}),
      }
}

fn rpc_launch(state: &RpcState) -> serde_json::Value {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": format!("Cannot read config: {}", e)}),
      };
    let root = &state.root;
    let pid_file = app_logic::codex_pid_file(&state.config_path);
    match app_logic::launch(&config, root, &pid_file) {
        Ok(msg) => serde_json::json!({"ok": true, "message": msg}),
        Err(e) => serde_json::json!({"error": e.to_string()}),
      }
}

fn rpc_stop(state: &RpcState) -> serde_json::Value {
    match app_logic::stop() {
        Ok(msg) => serde_json::json!({"ok": true, "message": msg}),
        Err(e) => serde_json::json!({"error": e.to_string()}),
      }
}

fn rpc_save_config(state: &RpcState, params: serde_json::Value) -> serde_json::Value {
    let config = match serde_json::from_value::<LauncherConfig>(params) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": format!("Invalid config: {}", e)}),
      };
    match config.write(&state.config_path) {
        Ok(_) => serde_json::json!({"ok": true, "message": "Config saved"}),
        Err(e) => serde_json::json!({"error": e.to_string()}),
      }
}

fn rpc_load_config(state: &RpcState) -> serde_json::Value {
    match LauncherConfig::read(&state.config_path) {
        Ok(config) => serde_json::json!({"ok": true, "data": config}),
        Err(e) => serde_json::json!({"ok": false, "data": serde_json::Value::Null, "error": e.to_string()}),
      }
}

fn rpc_health_check(state: &RpcState) -> serde_json::Value {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": format!("Cannot read config: {}", e)}),
      };
    let state = app_logic::health_check(&config);
    // health_check is async, we need to block on it
    let rt = tokio::runtime::Runtime::new().ok();
    let result = rt.map(|r| r.block_on(state)).unwrap_or_else(|| {
        Err("Cannot create runtime".into())
    });
    match result {
        Ok(info) => serde_json::json!({"ok": true, "data": info, "error": null}),
        Err(e) => serde_json::json!({"ok": false, "data": serde_json::Value::Null, "error": e.to_string()}),
      }
}

fn rpc_list_models(state: &RpcState) -> serde_json::Value {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": format!("Cannot read config: {}", e)}),
      };
    match app_logic::list_models(&config) {
        Ok(cache) => serde_json::json!({"ok": true, "data": cache, "error": null}),
        Err(e) => serde_json::json!({"ok": false, "data": serde_json::Value::Null, "error": e.to_string()}),
      }
}

fn rpc_refresh_models(state: &RpcState) -> serde_json::Value {
    let config = match LauncherConfig::read(&state.config_path) {
        Ok(c) => c,
        Err(e) => return serde_json::json!({"error": format!("Cannot read config: {}", e)}),
      };
    match app_logic::refresh_models(&config) {
        Ok(cache) => serde_json::json!({"ok": true, "data": cache, "error": null}),
        Err(e) => serde_json::json!({"ok": false, "data": serde_json::Value::Null, "error": e.to_string()}),
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
    let info = app_logic::detect_codex_process(&config);
    serde_json::json!({"ok": true, "data": info, "error": null})
}

fn rpc_kill_codex_by_pid(state: &RpcState, params: serde_json::Value) -> serde_json::Value {
    let pid = match params.get("pid") {
        Some(serde_json::Value::Number(p)) => p.as_u64() as u32,
         _ => return serde_json::json!({"error": "Missing pid parameter"}),
      };
    match app_logic::kill_codex_by_pid_number(pid) {
        Ok(msg) => serde_json::json!({"ok": true, "message": msg}),
        Err(e) => serde_json::json!({"error": e.to_string()}),
      }
}

fn rpc_open_directory_picker(_state: &RpcState) -> serde_json::Value {
    match rfd::FileDialog::new().pick_folder() {
        Some(path) => serde_json::json!({"ok": true, "data": {"path": path.to_string_lossy().to_string()} }),
        None => serde_json::json!({"ok": true, "data": {"path": ""} }),
      }
}

fn rpc_get_app_logs(state: &RpcState) -> serde_json::Value {
    let logs_path = state.root.join("app.log");
    match std::fs::read_to_string(&logs_path) {
        Ok(logs) => serde_json::json!({"ok": true, "data": {"logs": logs} }),
        Err(_) => serde_json::json!({"ok": true, "data": {"logs": "No log file found"} }),
      }
}

fn rpc_save_settings(state: &RpcState, settings: serde_json::Value) -> serde_json::Value {
    match LauncherConfig::read(&state.config_path) {
        Ok(mut config) => {
            if let Some(obj) = settings.as_object() {
                for (key, value) in obj {
                    match key.as_str() {
                         "autoStart" => config.auto_start = Some(value.as_bool().unwrap_or(false)),
                         "openaiBaseUrl" => config.openai_base_url = Some(value.as_str().unwrap_or("").to_string()),
                         "ollamaIp" => config.ollama_ip = Some(value.as_str().unwrap_or("").to_string()),
                         "ollamaPort" => config.ollama_port = Some(value.as_u64().unwrap_or(0) as u16),
                         "apiKey" => config.api_key = Some(value.as_str().unwrap_or("").to_string()),
                         _ => {}
                      }
                  }
              }
            match config.write(&state.config_path) {
                Ok(_) => serde_json::json!({"ok": true, "message": "Settings saved"}),
                Err(e) => serde_json::json!({"error": e.to_string()}),
              }
          }
        Err(e) => serde_json::json!({"error": format!("Cannot read config: {}", e)}),
      }
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
}

fn make_json_response(
    status: u16,
    data: Option<serde_json::Value>,
    error: Option<String>,
) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    let body = serde_json::json!({"ok": data.is_some(), "data": data, "error": error});
    let body_bytes = serde_json::to_string(&body).unwrap_or_default().into_bytes();
    tiny_http::Response::new(
        tiny_http::StatusCode(status),
        vec![
            tiny_http::Header::from_bytes(b"Content-Type", b"application/json; charset=utf-8").unwrap(),
            tiny_http::Header::from_bytes(b"Cache-Control", b"no-cache").unwrap(),
           ],
        std::io::Cursor::new(body_bytes), None, None,
       )
}

fn make_file_response(status: u16, content_type: &str, data: Vec<u8>) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    tiny_http::Response::new(
        tiny_http::StatusCode(status),
        vec![
            tiny_http::Header::from_bytes(b"Content-Type", content_type.as_bytes()).unwrap(),
            tiny_http::Header::from_bytes(b"Cache-Control", b"no-cache").unwrap(),
           ],
        std::io::Cursor::new(data), None, None,
       )
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("css") => "text/css",
        Some("json") => "application/json; charset=utf-8",
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

fn handle_request(dist_dir: &PathBuf, state: &Arc<Mutex<RpcState>>, mut request: tiny_http::Request) {
    let uri = request.url();
    if uri == "/rpc" && *request.method() == tiny_http::Method::Post {
        let body_str = std::io::read_to_string(request.as_reader()).unwrap_or_default();
        let request_data: serde_json::Value = match serde_json::from_str(&body_str) {
            Ok(d) => d,
            Err(e) => {
                let resp = make_json_response(400, None, Some(format!("Invalid JSON: {e}")));
                let _ = request.respond(resp);
                return;
                  }
              };
        let method = request_data.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = request_data.get("params").unwrap_or(&serde_json::Value::Null);
        let result = match state.lock() {
            Ok(guard) => handle_rpc(&guard, method, params.clone()),
            Err(poisoned) => handle_rpc(&poisoned.into_inner(), method, params.clone()),
              };
        let resp = make_json_response(200, Some(result), None);
        let _ = request.respond(resp);
        return;
          }
       // Normalize the path by resolving .. components
    let clean_path = uri.trim_start_matches("/").trim_start_matches("index.html");
    let sanitized = clean_path.split("/").filter(|s| *s != "." && *s != "..").collect::<Vec<_>>().join("/");
    let file_path = dist_dir.join(sanitized);
       // Verify the resolved path stays within the dist directory
    if !file_path.starts_with(&dist_dir) {
        let _ = request.respond(make_json_response(403, None, Some("Forbidden".to_string())));
        return;
          }
    let file_path = if clean_path.is_empty() || clean_path == "index.html" {
        dist_dir.join("index.html")
          } else { file_path };
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
`;
fs.writeFileSync("C:/app/codex-local-launcher/src/web_backend.rs", content, "utf8");
console.log("Written " + content.length + " bytes");