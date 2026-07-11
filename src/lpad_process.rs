use std::error::Error;
use std::fs;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use reqwest::Client;

const PID_FILE_NAME: &str = ".codex.pid";

#[derive(Debug)]
pub struct CodexProcess {
    pub handle: Option<std::process::Child>,
    pub base_url: String,
    pub pid_file: Option<PathBuf>,
}

/// Information about a running Codex process, used by the UI.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CodexProcessInfo {
    pub running: bool,
    pub pid: Option<u32>,
    pub method: Option<String>,
    pub restart_required: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct ProcessState {
    pub running: bool,
    pub api_ready: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("could not spawn codex: {0}")]
    Spawn(String),
    #[error("could not wait for codex: {0}")]
    Wait(String),
    #[error("could not kill codex process: {0}")]
    Kill(String),
    #[error("codex is not running")]
    NotRunning,
    #[error("codex API is not ready yet")]
    ApiNotReady,
    #[error("health check failed: {0}")]
    HealthCheck(String),
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
}

impl CodexProcess {
    pub fn new(base_url: String) -> Self {
        Self {
            handle: None,
            base_url,
            pid_file: None,
        }
    }

    pub fn spawn_pid_file_path(root: &Path) -> PathBuf {
        root.join(PID_FILE_NAME)
    }

    pub fn spawn(
        codex_command: &str,
        working_directory: &PathBuf,
        codex_args: &[String],
        local_api: Option<(&str, &str)>,
        pid_file: &std::path::Path,
    ) -> Result<Self, Box<dyn Error>> {
        let mut command = Command::new(codex_command);
        command
            .current_dir(working_directory)
            .args(codex_args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        if let Some((base_url, api_key)) = local_api {
            command
                .env("OPENAI_BASE_URL", base_url)
                .env("OPENAI_API_KEY", api_key);
        }

        // Detach from the launcher so Codex keeps running after the parent exits.
        #[cfg(target_os = "windows")]
        command.creation_flags(0x00000008); // DETACHED_PROCESS

        let child = command
            .spawn()
            .map_err(|e| ProcessError::Spawn(e.to_string()))?;
        let pid = child.id();

        // Write PID to file for later kill
        if let Some(parent) = pid_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(pid_file, pid.to_string())
            .map_err(|e| ProcessError::Kill(format!("Could not write PID file: {e}")))?;

        Ok(Self {
            handle: Some(child),
            base_url: local_api
                .map(|(base_url, _)| base_url.to_string())
                .unwrap_or_default(),
            pid_file: Some(pid_file.to_path_buf()),
        })
    }

    pub fn is_running(&mut self) -> bool {
        match &mut self.handle {
            Some(child) => match child.try_wait() {
                Ok(Some(_)) => {
                    self.handle = None;
                    self.cleanup_pid_file();
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            },
            None => false,
        }
    }

    pub async fn health_check(&self, timeout_secs: u64) -> Result<ProcessState, Box<dyn Error>> {
        let client = Client::new();
        let start = std::time::Instant::now();
        let interval = Duration::from_millis(500);

        loop {
            if start.elapsed() > Duration::from_secs(timeout_secs) {
                return Ok(ProcessState {
                    running: true,
                    api_ready: false,
                });
            }

            // Try to reach the API - if the process is running, it will eventually respond
            match client
                .get(format!("{}/health", self.base_url))
                .timeout(Duration::from_secs((timeout_secs / 2).max(1)))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    return Ok(ProcessState {
                        running: true,
                        api_ready: true,
                    });
                }
                Ok(_) => {
                    // Got a response (even non-2xx) - process is running
                    return Ok(ProcessState {
                        running: true,
                        api_ready: false,
                    });
                }
                Err(_) => {
                    // Connection refused - wait and retry
                    tokio::time::sleep(interval).await;
                }
            }
        }
    }

    pub async fn health_check_simple(
        &self,
        timeout_secs: u64,
    ) -> Result<ProcessState, Box<dyn Error>> {
        let client = Client::new();
        let start = std::time::Instant::now();
        let interval = Duration::from_millis(500);

        loop {
            if start.elapsed() > Duration::from_secs(timeout_secs) {
                return Ok(ProcessState {
                    running: true,
                    api_ready: false,
                });
            }

            match client
                .get(format!("{}/health", self.base_url))
                .timeout(Duration::from_secs(2))
                .send()
                .await
            {
                Ok(response) => {
                    return Ok(ProcessState {
                        running: true,
                        api_ready: response.status().is_success(),
                    });
                }
                Err(_) => {
                    tokio::time::sleep(interval).await;
                }
            }
        }
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        match &mut self.handle {
            Some(child) => {
                child
                    .kill()
                    .map_err(|e| ProcessError::Kill(e.to_string()))?;
                self.handle = None;
                self.cleanup_pid_file();
                Ok(())
            }
            None => Err(ProcessError::NotRunning.into()),
        }
    }

    fn cleanup_pid_file(&self) {
        if let Some(ref path) = self.pid_file {
            let _ = fs::remove_file(path);
        }
    }

    pub fn wait_for_start(&mut self, timeout_secs: u64) -> Result<(), Box<dyn Error>> {
        let start = std::time::Instant::now();
        let interval = Duration::from_millis(500);

        while start.elapsed() < Duration::from_secs(timeout_secs) {
            match &mut self.handle {
                Some(child) => match child.try_wait() {
                    Ok(Some(_)) => {
                        return Err(
                            ProcessError::Wait("codex exited unexpectedly".to_string()).into()
                        );
                    }
                    Ok(None) => {
                        if self.is_api_ready_blocking() {
                            return Ok(());
                        }
                        std::thread::sleep(interval);
                    }
                    Err(e) => {
                        return Err(ProcessError::Wait(e.to_string()).into());
                    }
                },
                None => {
                    return Err(ProcessError::NotRunning.into());
                }
            }
        }

        Err(ProcessError::ApiNotReady.into())
    }

    fn is_api_ready_blocking(&self) -> bool {
        use reqwest::blocking::Client;
        Client::new()
            .get(format!("{}/health", self.base_url))
            .timeout(Duration::from_secs(1))
            .send()
            .is_ok_and(|resp| resp.status().is_success())
    }

    /// Kill a codex process by reading PID from the PID file.
    pub fn kill_by_pid_file(pid_file: &std::path::Path) -> Result<String, Box<dyn Error>> {
        let content = fs::read_to_string(pid_file)
            .map_err(|e| ProcessError::Kill(format!("PID file not found: {e}")))?;

        let pid: u32 = content
            .trim()
            .parse()
            .map_err(|e| ProcessError::Kill(format!("Invalid PID in file: {e}")))?;

        #[cfg(target_os = "windows")]
        {
            let result = crate::process_util::command("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .output();

            match result {
                Ok(output) if output.status.success() => {
                    let _ = fs::remove_file(pid_file);
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
            // Try SIGTERM first for graceful shutdown
            let term_result = Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .output()?;

            if term_result.status.success() {
                // Wait briefly to see if process exits gracefully
                std::thread::sleep(Duration::from_secs(2));

                // Check if still running
                let check = Command::new("kill")
                    .args(["-0", &pid.to_string()])
                    .output()?;
                if check.status.success() {
                    // Still running, force kill
                    let kill_result = Command::new("kill")
                        .args(["-9", &pid.to_string()])
                        .output()?;
                    if !kill_result.status.success() {
                        let stderr = String::from_utf8_lossy(&kill_result.stderr);
                        return Err(ProcessError::Kill(format!(
                            "Force-kill failed for process {pid}: {stderr}"
                        ))
                        .into());
                    }
                    let _ = fs::remove_file(pid_file);
                    return Ok(format!(
                        "Killed process {pid} (SIGTERM timed out, used SIGKILL)"
                    ));
                }
                let _ = fs::remove_file(pid_file);
                return Ok(format!("Killed process {pid}"));
            }
            let stderr = String::from_utf8_lossy(&term_result.stderr);
            Err(
                ProcessError::Kill(format!("Failed to send SIGTERM to process {pid}: {stderr}"))
                    .into(),
            )
        }
    }
}
