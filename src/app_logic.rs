use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::acp_client::{AcpClient, MessageResponse};
use crate::codex_config;
use crate::codex_process::{CodexProcess, CodexProcessInfo, ProcessState};
use crate::config::LauncherConfig;
use crate::launcher;
use crate::ollama;

pub fn default_config_path(root: &Path) -> PathBuf {
    let packaged_config = root.join("config.json");
    if packaged_config.exists() {
        packaged_config
     } else {
        PathBuf::from("config.json")
     }
}

pub fn codex_pid_file(config_path: &Path) -> PathBuf {
     // Place PID file next to config
    config_path.with_extension("pid")
}

pub fn write_config(config: &LauncherConfig) -> Result<String, Box<dyn Error>> {
    let base_url = config.openai_base_url()?;
    let api_key = config.api_key()?;
    let resolved_model = if config.discover_ollama_models() {
        ollama::resolve_model(config, &base_url)?
     } else {
        config.codex_model()
     };

    if config.persist_codex_config() {
        let persistent_config = codex_config::PersistentCodexConfig::from_launcher_config(
            config,
            resolved_model,
            base_url,
            api_key,
         )?;
        let path = persistent_config.config_path.display().to_string();
        codex_config::apply(&persistent_config)?;
        return Ok(format!("Updated Codex config: {path}"));
     }

    Ok("Persistent Codex config is disabled; nothing was written.".to_string())
}

pub fn restore(config: &LauncherConfig) -> Result<String, Box<dyn Error>> {
    let restored_path = codex_config::restore(config)?;
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
    let base_url = config.openai_base_url()?;
    let api_key = config.api_key()?;
    let working_directory = config.working_directory(root)?;
    let codex_args = config.codex_args();
    let codex_command = config
         .codex_command()
         .ok_or("codexCommand not set in config")?;

    let target = launcher::resolve(config)?;
    let launch_target = target.to_string();

    CodexProcess::spawn(
         &codex_command,
         &working_directory,
         &codex_args,
         &base_url,
         &api_key,
        pid_file,
     )?;

     // Don't wait for API readiness in the basic launch - just return
    Ok(format!("Launching Codex via {launch_target}"))
}

pub async fn launch_and_wait(
    config: &LauncherConfig,
    root: &Path,
    pid_file: &std::path::Path,
) -> Result<CodexProcess, Box<dyn Error>> {
    let base_url = config.openai_base_url()?;
    let api_key = config.api_key()?;
    let working_directory = config.working_directory(root)?;
    let codex_args = config.codex_args();
    let codex_command = config
         .codex_command()
         .ok_or("codexCommand not set in config")?;

    let target = launcher::resolve(config)?;
    let _launch_target = target.to_string();

    let mut _process = CodexProcess::spawn(
         &codex_command,
         &working_directory,
         &codex_args,
         &base_url,
         &api_key,
        pid_file,
     )?;

     _process.wait_for_start(30)?;
    Ok(_process)
}

pub fn kill_codex(process: &mut CodexProcess) -> Result<String, Box<dyn Error>> {
    process.stop()?;
    Ok("Codex stopped.".to_string())
}

pub fn kill_codex_by_pid(pid_file: &Path) -> Result<String, Box<dyn Error>> {
    CodexProcess::kill_by_pid_file(pid_file)
}

pub async fn health_check(config: &LauncherConfig) -> Result<ProcessState, Box<dyn Error>> {
    let process = CodexProcess::new(config.codex_api_base_url());
    process.health_check(1).await
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

/// Detect if Codex is already running.
/// Tries multiple strategies in order:
/// 1. Check our own PID file (written when this launcher spawned it)
/// 2. Hit the configured codex_api_port for a /health endpoint
/// 3. Hit the configured ollamaIp:ollamaPort for a /api/tags endpoint
/// 4. Scan common Codex/Ollama ports for /health or /api/tags responses
/// Detect if Codex is already running by any means.
/// Strategies in priority order:
/// 1. Check PID file written when this launcher spawned Codex
/// 2. Scan for running Codex process by name (works when Codex was launched externally)
/// 3. Hit the configured codex_api_port for a /health endpoint
/// 4. Hit the configured ollamaIp:ollamaPort for a /api/tags endpoint
/// 5. Scan common ports for /health or /api/tags
pub fn detect_codex_process(config: &LauncherConfig) -> CodexProcessInfo {
         // Strategy 1: PID file from launcher spawn
        let config_dir = std::env::current_exe()
                 .ok()
                 .and_then(|path| path.parent().map(|p| p.to_path_buf()))
                 .unwrap_or_else(|| PathBuf::from("."));
        let launcher_pid_file = config_dir.join(".codex.pid");

        if launcher_pid_file.exists() {
            if let Ok(pid_str) = fs::read_to_string(&launcher_pid_file) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    if is_process_running(pid) {
                        let base_url = config.codex_api_base_url();
                        if endpoint_responds(&base_url, 2) {
                            return CodexProcessInfo {
                                running: true,
                                pid: Some(pid),
                                method: Some("pid_file".to_string()),
                                restart_required: true,
                             };
                         }
                        let _ = fs::remove_file(&launcher_pid_file);
                     } else {
                        let _ = fs::remove_file(&launcher_pid_file);
                     }
                 }
             }
         }

         // Strategy 2: detect Codex by process name (primary detection for externally-launched Codex)
        if let Some(info) = detect_codex_by_name() {
            return info;
         }

         // Strategy 3: configured codex_api_port
        let codex_url = config.codex_api_base_url();
        if endpoint_responds(&codex_url, 2) {
            if let Some((pid, _method)) = detect_process_on_port(config.codex_api_port()) {
                return CodexProcessInfo {
                    running: true,
                    pid: Some(pid),
                    method: Some("codex_api_port".to_string()),
                    restart_required: true,
                  };
               }
            return CodexProcessInfo {
                running: true,
                pid: None,
                method: Some("codex_api_port".to_string()),
                restart_required: true,
              };
           }

           // Strategy 4: ollamaIp:ollamaPort for /api/tags
        if let Some(ollama_ip) = config.ollama_ip.as_ref().map(|s| s.as_str()) {
            let port = config.ollama_port.unwrap_or(11434);
            let ollama_url = format!("http://{}:{}/api/tags", ollama_ip, port);
            if endpoint_responds(&ollama_url, 2) {
                if let Some((pid, _method)) = detect_process_on_port(port) {
                    return CodexProcessInfo {
                        running: true,
                        pid: Some(pid),
                        method: Some("ollama_port".to_string()),
                        restart_required: false,
                      };
                   }
                return CodexProcessInfo {
                    running: true,
                    pid: None,
                    method: Some("ollama_port".to_string()),
                    restart_required: false,
                  };
               }
           }

           // Strategy 5: scan common ports
        let common_ports = [11434u16, 8080u16, 8000u16, 3000u16];
        for port in &common_ports {
            if endpoint_responds(&format!("http://127.0.0.1:{}", port), 1) {
                if let Some((pid, _method)) = detect_process_on_port(*port) {
                    return CodexProcessInfo {
                        running: true,
                        pid: Some(pid),
                        method: Some("common_port".to_string()),
                        restart_required: true,
                      };
                   }
                return CodexProcessInfo {
                    running: true,
                    pid: None,
                    method: Some(format!("port_{}", port)),
                    restart_required: true,
                  };
               }
           }

        CodexProcessInfo {
            running: false,
            pid: None,
            method: None,
            restart_required: false,
          }
}

/// Detect if a Codex process is running by name (cross-platform).
/// This handles the common case where Codex was launched externally
/// (user opened it manually, auto-started, etc.) rather than via this launcher.
fn detect_codex_by_name() -> Option<CodexProcessInfo> {
        #[cfg(target_os = "windows")]

         {
              // Use tasklist to find Codex/codex processes
              // Try all known Codex binary names, in order of likelihood
              let binary_names = ["Codex.exe", "codex.exe", "codex-app.exe", "codex-app", "codex.cmd", "codex.ps1"];
              for &name in &binary_names {
                  let output = Command::new("tasklist")
                               .args(["/FI", &format!("IMAGENAME eq {name}"), "/FO", "CSV", "/NH"])
                               .output()
                               .ok()?;
                  if output.status.success() {
                      let stdout = String::from_utf8_lossy(&output.stdout);
                      if stdout.contains(name) {
                              // Extract PID from CSV output (PID is second field)
                          for line in stdout.lines() {
                              if line.contains(name) {
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
                                 // Found at least one Codex process but couldn't parse PID
                                return Some(CodexProcessInfo {
                                    running: true,
                                    pid: None,
                                    method: Some("process_name".to_string()),
                                    restart_required: true,
                                   });
                               }
                          }
                   }
                   // Catch-all: search for any process whose name contains "codex" (case-insensitive)
                   // tasklist /FI doesn't support "contains", so we list all and filter
                  let output = Command::new("tasklist")
                               .args(["/FO", "CSV", "/NH"])
                               .output()
                               .ok()?;
                  if output.status.success() {
                      let stdout = String::from_utf8_lossy(&output.stdout);
                      for line in stdout.lines() {
                          let lower = line.to_lowercase();
                          if lower.contains("codex") {
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
                   }
              }
          #[cfg(target_os = "macos")]
          {
               // Use ps to find Codex processes
               // Try multiple possible binary names
              let names = ["Codex", "codex", "codex-app"];
              let output = Command::new("ps")
                           .args(["-ax"])
                           .output()
                           .ok()?;
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
                      }
                  // Fallback: case-insensitive check for any codex-related process
                  let lower = stdout.to_lowercase();
                  if lower.contains("codex") {
                      for line in stdout.lines() {
                          if line.to_lowercase().contains("codex") {
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
            let verify = Command::new("tasklist")
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
fn endpoint_responds(url: &str, timeout_secs: u64) -> bool {
    reqwest::blocking::Client::new()
             .get(url)
             .timeout(std::time::Duration::from_secs(timeout_secs))
             .send()
             .map(|r| r.status().is_success())
             .unwrap_or(false)
}


fn detect_process_on_port(port: u16) -> Option<(u32, String)> {
       #[cfg(target_os = "windows")]
       {
         let output = Command::new("netstat")
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
                         let verify = Command::new("tasklist")
                               .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
                               .output();

                         match verify {
                            Ok(v) if v.status.success() => {
                                 if String::from_utf8_lossy(&v.stdout).contains(&pid.to_string()) {
                                    return Some((pid, "netstat".to_string()));
                                 }
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

         if let Some(pid_str) = stdout.and_then(|s| s.trim().lines().next()) {
             if let Ok(pid) = pid_str.trim().parse::<u32>() {
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

            // Fallback to ss
         let ss_output = Command::new("sh")
               .args(["-c", &format!("ss -tlnp sport = :{} 2>/dev/null | grep -oP 'pid=\\\\K[0-9]+'", port)])
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
        let result = Command::new("taskkill")
               .args(["/F", "/T", "/PID", &pid.to_string()])
               .output();

        match result {
            Ok(output) if output.status.success() => {
                Ok(format!("Killed Codex process {pid}"))
               }
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
            Ok(output) if output.status.success() => {
                Ok(format!("Killed Codex process {pid}"))
               }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to kill process {pid}: {stderr}").into())
               }
            Err(e) => Err(format!("Could not run kill: {e}").into()),
           }
       }
}

pub fn revert_codex_config(config: &LauncherConfig) -> Result<String, Box<dyn Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not locate home directory; cannot determine Codex config path.")?;
    let config_path = match config.codex_config_path() {
        Some(p) => p,
        None => home_dir.join(".codex").join("config.toml"),
    };

    if !config_path.exists() {
        return Ok("Codex config file does not exist; nothing to revert.".to_string());
    }

    // Verify the path is within expected directory to prevent path traversal
    let canonical = match config_path.canonicalize() {
        Ok(c) => c,
        Err(e) => return Err(format!("Cannot resolve config path: {e}").into()),
    };

    if !canonical.starts_with(&home_dir) {
        return Err("Config path is outside home directory; refusing to revert.".into());
    }

    let restored_path = codex_config::restore(config)?;
    Ok(format!(
        "Reverted Codex config to first backup: {}",
        restored_path.display()
    ))
}

