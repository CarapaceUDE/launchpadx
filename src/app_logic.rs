use crate::acp_client::{AcpClient, MessageResponse};
use crate::config::LauncherConfig;
use crate::launcher::{self, LaunchTarget};
use crate::lpad_config;
use crate::lpad_process::{CodexProcess, CodexProcessInfo, ProcessState};
use crate::ollama;
use crate::process_util;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(target_os = "windows"))]
use std::process::Command;
use std::time::Duration;

pub fn default_config_path(root: &Path) -> PathBuf {
    let packaged_config = root.join("config.json");
    if packaged_config.exists() {
        packaged_config
    } else {
        PathBuf::from("config.json")
    }
}

pub fn lpad_pid_file(config_path: &Path) -> PathBuf {
    // Place PID file next to config
    config_path.with_extension("pid")
}

pub fn codex_managed_by_launcher(config: &LauncherConfig) -> bool {
    lpad_config::inspect(config)
        .map(|inspection| inspection.managed_by_launcher)
        .unwrap_or(false)
}

/// Writes launcher-managed settings only when Local API is the active Codex provider.
pub fn write_config_for_launch(config: &LauncherConfig) -> Result<Option<String>, Box<dyn Error>> {
    if !codex_managed_by_launcher(config) {
        return Ok(None);
    }
    write_config(config).map(Some)
}

fn local_api_env(config: &LauncherConfig) -> Result<Option<(String, String)>, Box<dyn Error>> {
    if !codex_managed_by_launcher(config) {
        return Ok(None);
    }

    Ok(Some((config.openai_base_url()?, config.api_key()?)))
}

pub fn write_config(config: &LauncherConfig) -> Result<String, Box<dyn Error>> {
    let base_url = config.openai_base_url()?;
    let api_key = config.api_key()?;
    let resolved_model = if config.discover_ollama_models() {
        ollama::resolve_model(config, &base_url)?
    } else {
        config.lpad_model()
    };

    if config.lpad_persist_config() {
        let persistent_config = lpad_config::PersistentCodexConfig::from_launcher_config(
            config,
            resolved_model,
            base_url,
            api_key,
        )?;
        let path = persistent_config.config_path.display().to_string();
        lpad_config::apply(&persistent_config)?;
        return Ok(format!("Updated Codex config: {path}"));
    }

    Ok("Persistent Codex config is disabled; nothing was written.".to_string())
}

pub fn restore(config: &LauncherConfig) -> Result<String, Box<dyn Error>> {
    let (restored_path, _) = lpad_config::restore(config)?;
    Ok(format!(
        "Restored Codex config: {}",
        restored_path.display()
    ))
}

pub fn refresh_models(config: &LauncherConfig) -> Result<ollama::ModelCache, Box<dyn Error>> {
    let base_url = config.openai_base_url()?;
    let optional_api_key = config.api_key_if_configured();
    Ok(ollama::refresh_model_cache(
        &base_url,
        optional_api_key.as_deref(),
    )?)
}

pub fn list_models(config: &LauncherConfig) -> Result<ollama::ModelCache, Box<dyn Error>> {
    let base_url = config.openai_base_url()?;
    let optional_api_key = config.api_key_if_configured();
    match ollama::read_model_cache() {
        Ok(cache) => Ok(cache),
        Err(_) => Ok(ollama::refresh_model_cache(
            &base_url,
            optional_api_key.as_deref(),
        )?),
    }
}

pub fn launch(
    config: &LauncherConfig,
    root: &Path,
    pid_file: &std::path::Path,
) -> Result<String, Box<dyn Error>> {
    let existing = detect_codex_process(config, root);
    if existing.running {
        let method = existing.method.unwrap_or_else(|| "unknown".to_string());
        return Ok(format!("Codex is already running (detected via {method})"));
    }

    let local_api = local_api_env(config)?;
    let working_directory = config.working_directory(root)?;
    let lpad_args = config.lpad_args();
    let target = launcher::resolve(config)?;
    let launch_target = target.to_string();

    match &target {
        LaunchTarget::Path(path) => {
            let command = path
                .to_str()
                .ok_or("Resolved Codex path is not valid UTF-8")?;
            CodexProcess::spawn(
                command,
                &working_directory,
                &lpad_args,
                local_api
                    .as_ref()
                    .map(|(base_url, api_key)| (base_url.as_str(), api_key.as_str())),
                pid_file,
            )?;
            #[cfg(target_os = "windows")]
            if !launcher::wait_for_codex_process(10) {
                return Err(format!(
                    "Codex launch was requested via {launch_target}, but no Codex process appeared. Set codexCommand in Settings to the full path of Codex.exe or run `launchpadx --diagnose`."
                )
               .into());
            }
        }
        LaunchTarget::WindowsStartApp { app_id } => {
            launcher::launch_windows_start_app(app_id)?;
        }
        LaunchTarget::MacAppBundle(bundle) => {
            launcher::launch_macos_bundle(
                bundle,
                &working_directory,
                local_api
                    .as_ref()
                    .map(|(base_url, api_key)| (base_url.as_str(), api_key.as_str())),
            )?;
        }
    }

    Ok(format!("Launching Codex via {launch_target}"))
}

pub async fn launch_and_wait(
    config: &LauncherConfig,
    root: &Path,
    pid_file: &std::path::Path,
) -> Result<CodexProcess, Box<dyn Error>> {
    let local_api = local_api_env(config)?;
    let working_directory = config.working_directory(root)?;
    let lpad_args = config.lpad_args();
    let target = launcher::resolve(config)?;

    let mut process = match &target {
        LaunchTarget::Path(path) => {
            let command = path
                .to_str()
                .ok_or("Resolved Codex path is not valid UTF-8")?;
            CodexProcess::spawn(
                command,
                &working_directory,
                &lpad_args,
                local_api
                    .as_ref()
                    .map(|(base_url, api_key)| (base_url.as_str(), api_key.as_str())),
                pid_file,
            )?
        }
        LaunchTarget::WindowsStartApp { app_id } => {
            launcher::launch_windows_start_app(app_id)?;
            return Err(
                "Codex launched via Windows Start App; cannot wait for API readiness".into(),
            );
        }
        LaunchTarget::MacAppBundle(bundle) => {
            launcher::launch_macos_bundle(
                bundle,
                &working_directory,
                local_api
                    .as_ref()
                    .map(|(base_url, api_key)| (base_url.as_str(), api_key.as_str())),
            )?;
            return Err("Codex launched via app bundle; cannot wait for API readiness".into());
        }
    };

    process.wait_for_start(30)?;
    Ok(process)
}

pub fn kill_codex(process: &mut CodexProcess) -> Result<String, Box<dyn Error>> {
    process.stop()?;
    Ok("Codex stopped.".to_string())
}

pub fn kill_codex_by_pid(pid_file: &Path) -> Result<String, Box<dyn Error>> {
    CodexProcess::kill_by_pid_file(pid_file)
}

fn codex_stop_pid_files(root: &Path, pid_file: &Path) -> Vec<PathBuf> {
    let mut paths = vec![
        pid_file.to_path_buf(),
        CodexProcess::spawn_pid_file_path(root),
    ];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let exe_pid = parent.join(".codex.pid");
            if !paths.contains(&exe_pid) {
                paths.push(exe_pid);
            }
        }
    }
    paths
}

fn cleanup_stop_pid_files(root: &Path, pid_file: &Path) {
    for path in codex_stop_pid_files(root, pid_file) {
        let _ = fs::remove_file(path);
    }
}

fn codex_appears_running(config: &LauncherConfig, root: &Path) -> bool {
    let info = detect_codex_process(config, root);
    info.running || codex_api_ready(config)
}

/// Kill Codex processes by known binary names. Handles Electron multi-process and
/// Microsoft Store launches where a single PID kill is not enough.
fn kill_codex_processes_by_name() -> Result<Vec<String>, Box<dyn Error>> {
    let mut messages = Vec::new();

    #[cfg(target_os = "windows")]
    {
        for image in ["Codex.exe", "codex.exe", "codex-app.exe"] {
            match process_util::command("taskkill")
                .args(["/F", "/T", "/IM", image])
                .output()
            {
                Ok(output) if output.status.success() => {
                    messages.push(format!("Stopped processes matching {image}"));
                }
                Ok(_) => {}
                Err(error) => return Err(format!("Could not run taskkill: {error}").into()),
            }
        }

        let output = match process_util::command("tasklist")
            .args(["/FO", "CSV", "/NH"])
            .output()
        {
            Ok(output) => output,
            Err(_) => return Ok(messages),
        };
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let lower = line.to_lowercase();
                if !lower.contains("codex") || is_launcher_process_name(line) {
                    continue;
                }
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() < 2 {
                    continue;
                }
                let Ok(pid) = parts[1].trim().trim_matches('"').parse::<u32>() else {
                    continue;
                };
                if kill_codex_by_pid_number(pid).is_ok() {
                    messages.push(format!("Stopped Codex process {pid}"));
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("osascript")
            .args(["-e", r#"tell application "Codex" to quit"#])
            .output();
        for signal in ["-TERM", "-KILL"] {
            for name in ["Codex", "codex", "codex-app"] {
                if let Ok(output) = Command::new("pkill").args([signal, "-x", name]).output() {
                    if output.status.success() {
                        messages.push(format!("Stopped processes named {name}"));
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        for signal in ["-TERM", "-KILL"] {
            for name in ["codex", "Codex"] {
                if let Ok(output) = Command::new("pkill").args([signal, "-x", name]).output() {
                    if output.status.success() {
                        messages.push(format!("Stopped processes named {name}"));
                    }
                }
            }
        }
        if let Ok(output) = Command::new("pkill")
            .args(["-TERM", "-f", "Codex.AppImage"])
            .output()
        {
            if output.status.success() {
                messages.push("Stopped Codex AppImage processes".to_string());
            }
        }
    }

    Ok(messages)
}

/// Stop Codex whether it was launched by this app or externally.
pub fn stop_codex(
    config: &LauncherConfig,
    root: &Path,
    pid_file: &Path,
) -> Result<String, Box<dyn Error>> {
    if !codex_appears_running(config, root) {
        cleanup_stop_pid_files(root, pid_file);
        return Err("Codex is not running.".into());
    }

    let mut messages = Vec::new();

    for path in codex_stop_pid_files(root, pid_file) {
        if path.exists() {
            if let Ok(msg) = kill_codex_by_pid(&path) {
                messages.push(msg);
            }
        }
    }

    let info = detect_codex_process(config, root);
    if let Some(pid) = info.pid {
        if !info.restart_required {
            return Err(
                "Detected a running backend service, not the Codex app; refusing to stop it."
                    .into(),
            );
        }
        if let Ok(msg) = kill_codex_by_pid_number(pid) {
            messages.push(msg);
        }
    }

    if let Ok(name_kills) = kill_codex_processes_by_name() {
        messages.extend(name_kills);
    }

    std::thread::sleep(Duration::from_millis(750));

    if codex_appears_running(config, root) {
        let detail = detect_codex_process(config, root);
        let hint = match (detail.pid, detail.method) {
            (Some(pid), _) => format!(" (still detected as PID {pid})"),
            (None, Some(method)) => format!(" (still detected via {method})"),
            _ => String::new(),
        };
        return Err(format!(
            "Stop was attempted but Codex still appears to be running{hint}. Close it manually or run `launchpadx --kill`."
        )
        .into());
    }

    cleanup_stop_pid_files(root, pid_file);

    Ok(messages
        .last()
        .cloned()
        .unwrap_or_else(|| "Codex stopped.".to_string()))
}

pub async fn health_check(config: &LauncherConfig) -> Result<ProcessState, Box<dyn Error>> {
    let process = CodexProcess::new(config.codex_api_base_url());
    process.health_check(1).await
}

pub fn endpoint_reachable(config: &LauncherConfig) -> bool {
    let Ok(base_url) = config.openai_base_url() else {
        return false;
    };
    let Ok(url) = ollama::tags_url_from_base(&base_url) else {
        return false;
    };
    endpoint_responds(&url, config.api_key_if_configured().as_deref(), 2)
}

pub fn codex_api_ready(config: &LauncherConfig) -> bool {
    let url = format!("{}/health", config.codex_api_base_url());
    endpoint_responds(&url, None, 2)
}

pub async fn start_session(
    config: &LauncherConfig,
) -> Result<crate::acp_client::SessionInfo, Box<dyn Error>> {
    let client = AcpClient::from_config(config)?;
    Ok(client.create_session().await?)
}

pub async fn send_message(
    config: &LauncherConfig,
    session_id: &str,
    content: &str,
) -> Result<MessageResponse, Box<dyn Error>> {
    let client = AcpClient::from_config(config)?;
    Ok(client.send_message(session_id, content).await?)
}

pub async fn get_response(
    config: &LauncherConfig,
    session_id: &str,
) -> Result<MessageResponse, Box<dyn Error>> {
    let client = AcpClient::from_config(config)?;
    Ok(client.get_response(session_id).await?)
}

pub async fn close_session(
    config: &LauncherConfig,
    session_id: &str,
) -> Result<String, Box<dyn Error>> {
    let client = AcpClient::from_config(config)?;
    client.close_session(session_id).await?;
    Ok(format!("Session {session_id} closed."))
}

pub async fn list_sessions(config: &LauncherConfig) -> Result<String, Box<dyn Error>> {
    let client = AcpClient::from_config(config)?;
    let sessions = client.list_sessions().await?;

    let mut output = String::new();
    for s in &sessions.sessions {
        output.push_str(&format!(
            "   {} - created {}\n",
            s.session_id,
            s.created_at.as_deref().unwrap_or("unknown")
        ));
    }
    Ok(output)
}

pub fn enable_auto_start(config: &LauncherConfig) -> Result<String, Box<dyn std::error::Error>> {
    if config.auto_start.unwrap_or(false) {
        launcher::autostart::enable_auto_start()?;
        Ok("Auto-start enabled.".to_string())
    } else {
        Ok("Auto-start is already disabled.".to_string())
    }
}

pub fn disable_auto_start(config: &LauncherConfig) -> Result<String, Box<dyn std::error::Error>> {
    if !config.auto_start.unwrap_or(false) {
        Ok("Auto-start is already disabled.".to_string())
    } else {
        launcher::autostart::disable_auto_start()?;
        Ok("Auto-start disabled.".to_string())
    }
}

/// Detect whether the Codex desktop app is running.
/// Ollama/backend reachability is tracked separately via `endpoint_reachable`.
pub fn detect_codex_process(config: &LauncherConfig, root: &Path) -> CodexProcessInfo {
    let mut pid_files = vec![CodexProcess::spawn_pid_file_path(root)];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let exe_pid = parent.join(".codex.pid");
            if !pid_files.contains(&exe_pid) {
                pid_files.push(exe_pid);
            }
        }
    }

    for launcher_pid_file in pid_files {
        if !launcher_pid_file.exists() {
            continue;
        }
        if let Ok(pid_str) = fs::read_to_string(&launcher_pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if is_process_running(pid) {
                    return CodexProcessInfo {
                        running: true,
                        pid: Some(pid),
                        method: Some("pid_file".to_string()),
                        restart_required: true,
                    };
                }
                let _ = fs::remove_file(&launcher_pid_file);
            }
        }
    }

    if let Some(info) = detect_codex_by_name() {
        return info;
    }

    let codex_health_url = format!("{}/health", config.codex_api_base_url());
    if endpoint_responds(&codex_health_url, None, 2) {
        if let Some((pid, _method)) = detect_process_on_port(config.lpad_api_port()) {
            return CodexProcessInfo {
                running: true,
                pid: Some(pid),
                method: Some("lpad_api_port".to_string()),
                restart_required: true,
            };
        }
        return CodexProcessInfo {
            running: true,
            pid: None,
            method: Some("lpad_api_port".to_string()),
            restart_required: true,
        };
    }

    CodexProcessInfo {
        running: false,
        pid: None,
        method: None,
        restart_required: false,
    }
}

fn is_launcher_process_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("launchpadx")
}

/// Detect if a Codex process is running by name (cross-platform).
/// This handles the common case where Codex was launched externally
/// (user opened it manually, auto-started, etc.) rather than via this launcher.
fn detect_codex_by_name() -> Option<CodexProcessInfo> {
    #[cfg(target_os = "windows")]
    {
        let output = process_util::command("tasklist")
            .args(["/FO", "CSV", "/NH"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }

        let binary_names = [
            "Codex.exe",
            "codex.exe",
            "codex-app.exe",
            "codex-app",
            "codex.cmd",
            "codex.ps1",
        ];
        let stdout = String::from_utf8_lossy(&output.stdout);
        for &name in &binary_names {
            for line in stdout.lines() {
                if !line.contains(name) {
                    continue;
                }
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    if let Ok(pid) = parts[1].trim().trim_matches('"').parse::<u32>() {
                        return Some(CodexProcessInfo {
                            running: true,
                            pid: Some(pid),
                            method: Some("process_name".to_string()),
                            restart_required: true,
                        });
                    }
                }
                return Some(CodexProcessInfo {
                    running: true,
                    pid: None,
                    method: Some("process_name".to_string()),
                    restart_required: true,
                });
            }
        }

        for line in stdout.lines() {
            let lower = line.to_lowercase();
            if !lower.contains("codex") || is_launcher_process_name(line) {
                continue;
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                if let Ok(pid) = parts[1].trim().trim_matches('"').parse::<u32>() {
                    return Some(CodexProcessInfo {
                        running: true,
                        pid: Some(pid),
                        method: Some("process_name".to_string()),
                        restart_required: true,
                    });
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        // Use ps to find Codex processes
        // Try multiple possible binary names
        let names = ["Codex", "codex", "codex-app"];
        let output = Command::new("ps").args(["-ax"]).output().ok()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Check if any known Codex binary is running
            for name in &names {
                if stdout.contains(name) {
                    // Extract PID (first field in ps -ax output)
                    for line in stdout.lines() {
                        if line.contains(name) {
                            let pid_str = line.split_whitespace().next()?;
                            if let Ok(pid) = pid_str.parse::<u32>() {
                                return Some(CodexProcessInfo {
                                    running: true,
                                    pid: Some(pid),
                                    method: Some("process_name".to_string()),
                                    restart_required: true,
                                });
                            }
                        }
                    }
                    // Found at least one Codex process but couldn't parse PID
                    return Some(CodexProcessInfo {
                        running: true,
                        pid: None,
                        method: Some("process_name".to_string()),
                        restart_required: true,
                    });
                }
            }
            for line in stdout.lines() {
                if !line.to_lowercase().contains("codex") || is_launcher_process_name(line) {
                    continue;
                }
                let pid_str = line.split_whitespace().next()?;
                if let Ok(pid) = pid_str.parse::<u32>() {
                    return Some(CodexProcessInfo {
                        running: true,
                        pid: Some(pid),
                        method: Some("process_name".to_string()),
                        restart_required: true,
                    });
                }
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        // Use pgrep to find codex processes
        let output = Command::new("sh")
            .args(["-c", "pgrep -f codex"])
            .output()
            .ok()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for pid_str in stdout.lines() {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    let ps = Command::new("ps")
                        .args(["-p", &pid.to_string(), "-o", "args="])
                        .output()
                        .ok();
                    if let Some(output) = ps {
                        let cmdline = String::from_utf8_lossy(&output.stdout);
                        if is_launcher_process_name(&cmdline) {
                            continue;
                        }
                    }
                    return Some(CodexProcessInfo {
                        running: true,
                        pid: Some(pid),
                        method: Some("process_name".to_string()),
                        restart_required: true,
                    });
                }
            }
            if !stdout.trim().is_empty() {
                return Some(CodexProcessInfo {
                    running: true,
                    pid: None,
                    method: Some("process_name".to_string()),
                    restart_required: true,
                });
            }
        }
    }

    None
}

/// Check if a PID is actually running (cross-platform)
fn is_process_running(pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        let verify = process_util::command("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
            .output();
        match verify {
            Ok(v) if v.status.success() => {
                String::from_utf8_lossy(&v.stdout).contains(&pid.to_string())
            }
            _ => false,
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output()
            .ok()
            .is_some_and(|v| v.status.success())
    }
}

/// Check if an endpoint responds to an HTTP request within timeout_secs
fn endpoint_responds(url: &str, api_key: Option<&str>, timeout_secs: u64) -> bool {
    let mut request = reqwest::blocking::Client::new()
        .get(url)
        .timeout(std::time::Duration::from_secs(timeout_secs));
    if let Some(api_key) = api_key.filter(|value| !value.trim().is_empty()) {
        request = request.bearer_auth(api_key);
    }
    request
        .send()
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

fn detect_process_on_port(port: u16) -> Option<(u32, String)> {
    #[cfg(target_os = "windows")]
    {
        let output = process_util::command("netstat")
            .args(["-ano"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let port_str = format!(":{}", port);
        for line in stdout.lines() {
            if line.contains(&port_str) && line.contains("LISTENING") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pid_str) = parts.last() {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        // Verify PID is still alive
                        let verify = process_util::command("tasklist")
                            .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
                            .output();

                        match verify {
                            Ok(v)
                                if v.status.success()
                                    && String::from_utf8_lossy(&v.stdout)
                                        .contains(&pid.to_string()) =>
                            {
                                return Some((pid, "netstat".to_string()));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Try lsof first, fall back to ss if not available
        let lsof_output = Command::new("sh")
            .args(["-c", &format!("lsof -i :{} -t 2>/dev/null", port)])
            .output();

        let stdout = if let Ok(output) = &lsof_output {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(ref output) = stdout {
            if let Some(line) = output.lines().map(str::trim).find(|line| !line.is_empty()) {
                if let Ok(pid) = line.parse::<u32>() {
                    // Verify process still exists
                    let verify = Command::new("kill")
                        .args(["-0", &pid.to_string()])
                        .output()
                        .ok()?;
                    if verify.status.success() {
                        return Some((pid, "lsof".to_string()));
                    }
                }
            }
        }

        // Fallback to ss
        let ss_output = Command::new("sh")
            .args([
                "-c",
                &format!(
                    "ss -tlnp sport = :{} 2>/dev/null | grep -oP 'pid=\\\\K[0-9]+'",
                    port
                ),
            ])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&ss_output.stdout);
        if let Some(pid_str) = stdout.trim().lines().next() {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                let verify = Command::new("kill")
                    .args(["-0", &pid.to_string()])
                    .output()
                    .ok()?;
                if verify.status.success() {
                    return Some((pid, "ss".to_string()));
                }
            }
        }
    }

    None
}

pub fn kill_codex_by_pid_number(pid: u32) -> Result<String, Box<dyn Error>> {
    #[cfg(target_os = "windows")]
    {
        let result = process_util::command("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .output();

        match result {
            Ok(output) if output.status.success() => Ok(format!("Killed Codex process {pid}")),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to kill process {pid}: {stderr}").into())
            }
            Err(e) => Err(format!("Could not run taskkill: {e}").into()),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let result = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .output();

        match result {
            Ok(output) if output.status.success() => Ok(format!("Killed Codex process {pid}")),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to kill process {pid}: {stderr}").into())
            }
            Err(e) => Err(format!("Could not run kill: {e}").into()),
        }
    }
}

pub fn inspect_codex_config(
    config: &LauncherConfig,
) -> Result<lpad_config::CodexConfigInspection, Box<dyn Error>> {
    Ok(lpad_config::inspect(config)?)
}

/// Merge launcher settings with the live Codex profile on first launch.
pub fn bootstrap_launcher_from_codex(
    config: &mut LauncherConfig,
    inspection: &lpad_config::CodexConfigInspection,
) -> Vec<String> {
    let mut changes = Vec::new();

    if config.lpad_model().is_none() {
        if let Some(model) = inspection.model.as_ref().filter(|m| !m.is_empty()) {
            config.lpad_model = Some(model.clone());
            changes.push(format!("Adopted Codex model: {model}"));
        }
    }

    if inspection.managed_by_launcher {
        if let Some(base_url) = inspection.launcher_base_url.as_ref() {
            if config.openai_base_url.is_none() && config.ollama_ip.is_none() {
                config.openai_base_url = Some(base_url.clone());
                changes.push(format!("Adopted Codex endpoint: {base_url}"));
            }
        }
    }

    changes
}

pub fn sync_codex_config(config: &LauncherConfig) -> Result<String, Box<dyn Error>> {
    write_config(config)
}

pub fn revert_codex_config(config: &LauncherConfig) -> Result<String, Box<dyn Error>> {
    let config_path = config
        .codex_config_path()
        .map(Ok)
        .unwrap_or_else(lpad_config::default_codex_config_path)?;

    if !config_path.exists() {
        return Ok("Codex config file does not exist; nothing to revert.".to_string());
    }

    let (restored_path, warning) = lpad_config::restore(config)?;
    let message = if let Some(w) = warning {
        format!(
            "Switched Codex back to account provider at {} -- {w}",
            restored_path.display(),
            w = w
        )
    } else {
        format!(
            "Switched Codex back to account provider: {}",
            restored_path.display()
        )
    };
    Ok(message)
}
