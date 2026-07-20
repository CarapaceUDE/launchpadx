//! Helpers for spawning child processes without flashing console windows on Windows.

use std::collections::HashSet;
use std::ffi::OsStr;
use std::process::Command;
use std::sync::{Mutex, OnceLock};

/// Create a `Command` that does not open a visible console window on Windows.
///
/// GUI builds use `windows_subsystem = "windows"`, so spawning `tasklist`, `netstat`,
/// `powershell`, and similar utilities without this flag causes focus-stealing flashes.
pub fn command(program: impl AsRef<OsStr>) -> Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut cmd = Command::new(program);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd
    }
    #[cfg(not(windows))]
    {
        Command::new(program)
    }
}

/// PIDs of short-lived helpers we spawn ourselves (e.g. `codex app-server` probes).
/// Health detection must ignore these so the UI does not flash "Stop Codex".
fn transient_helper_pids() -> &'static Mutex<HashSet<u32>> {
    static CELL: OnceLock<Mutex<HashSet<u32>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn register_transient_helper_pid(pid: u32) {
    if pid == 0 {
        return;
    }
    if let Ok(mut guard) = transient_helper_pids().lock() {
        guard.insert(pid);
    }
}

pub fn unregister_transient_helper_pid(pid: u32) {
    if pid == 0 {
        return;
    }
    if let Ok(mut guard) = transient_helper_pids().lock() {
        guard.remove(&pid);
    }
}

pub fn is_transient_helper_pid(pid: u32) -> bool {
    transient_helper_pids()
        .lock()
        .map(|guard| guard.contains(&pid))
        .unwrap_or(false)
}

/// True for LaunchPadX-owned `codex app-server --listen stdio://` probe processes.
pub fn is_app_server_probe_command(cmdline: &str) -> bool {
    let lower = cmdline.to_lowercase().replace('\\', "/");
    let has_app_server = lower.contains("app-server") || lower.contains("app_server");
    let stdio_listen = lower.contains("stdio://")
        || lower.contains("stdio:")
        || (lower.contains("--listen") && lower.contains("stdio"));
    has_app_server && stdio_listen
}

/// Best-effort process command line for filtering helper processes.
pub fn process_command_line(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }

    #[cfg(target_os = "windows")]
    {
        let script =
            format!("(Get-CimInstance Win32_Process -Filter \"ProcessId = {pid}\").CommandLine");
        let output = command("powershell.exe")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let cmdline = String::from_utf8_lossy(&output.stdout).trim().to_string();
        (!cmdline.is_empty()).then_some(cmdline)
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "args="])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let cmdline = String::from_utf8_lossy(&output.stdout).trim().to_string();
        (!cmdline.is_empty()).then_some(cmdline)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = pid;
        None
    }
}

/// True when this PID should not count as the user-facing Codex app.
pub fn should_ignore_codex_pid(pid: u32) -> bool {
    if is_transient_helper_pid(pid) {
        return true;
    }
    process_command_line(pid)
        .map(|cmdline| is_app_server_probe_command(&cmdline))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_stdio_app_server_probes() {
        assert!(is_app_server_probe_command(
            r#"C:\Users\me\codex.exe app-server --listen stdio://"#
        ));
        assert!(is_app_server_probe_command(
            "/usr/local/bin/codex app-server --listen stdio://"
        ));
        assert!(!is_app_server_probe_command(
            r#"C:\Program Files\Codex\Codex.exe"#
        ));
        assert!(!is_app_server_probe_command("codex"));
    }

    #[test]
    fn transient_pid_registry_round_trips() {
        register_transient_helper_pid(424242);
        assert!(is_transient_helper_pid(424242));
        unregister_transient_helper_pid(424242);
        assert!(!is_transient_helper_pid(424242));
    }
}
