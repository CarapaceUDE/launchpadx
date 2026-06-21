use std::path::PathBuf;
use std::process::Command;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AutoStartError {
    #[error("could not locate home directory")]
    MissingHome,
    #[error("could not create directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not write autostart entry: {source}")]
    WriteEntry { source: std::io::Error },
    #[error("could not read autostart entry: {source}")]
    ReadEntry { source: std::io::Error },
    #[error("autostart registration failed: {0}")]
    Registration(String),
}

/// Get the binary path for autostart registration.
fn get_binary_path() -> Result<PathBuf, AutoStartError> {
    std::env::current_exe().map_err(|_| AutoStartError::MissingHome)
}

#[cfg(target_os = "windows")]
pub fn enable_auto_start() -> Result<(), AutoStartError> {
    let binary = get_binary_path()?;
    let binary_path = binary.to_string_lossy().replace('"', r#""""#);

    // Use PowerShell to set registry key
    let script = format!(
        r#"Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "CodexLocalLauncher" -Value "{}" -Force"#,
        binary_path
    );

    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map_err(|e| AutoStartError::Registration(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AutoStartError::Registration(stderr.trim().to_string()));
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn disable_auto_start() -> Result<(), AutoStartError> {
    // Use PowerShell to remove registry key
    let script = r#"Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "CodexLocalLauncher" -ErrorAction SilentlyContinue"#;

    let _output = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", script])
        .output()
        .map_err(|e| AutoStartError::Registration(e.to_string()))?;

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn enable_auto_start() -> Result<(), AutoStartError> {
    let home = dirs::home_dir().ok_or(AutoStartError::MissingHome)?;
    let launch_agents = home.join("Library/LaunchAgents");

    fs::create_dir_all(&launch_agents).map_err(|source| AutoStartError::CreateDir {
        path: launch_agents.clone(),
        source,
    })?;

    let binary = get_binary_path()?;
    let binary_path = binary.to_string_lossy();

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.codex-local-launcher.launcher</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>"#,
        binary_path
    );

    let plist_path = launch_agents.join("com.codex-local-launcher.launcher.plist");
    fs::write(&plist_path, plist.as_bytes())
        .map_err(|source| AutoStartError::WriteEntry { source })?;

    // Load the launch agent
    let _output = Command::new("launchctl")
        .args(["load", &plist_path.to_string_lossy()])
        .output()
        .map_err(|e| AutoStartError::Registration(e.to_string()))?;

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn disable_auto_start() -> Result<(), AutoStartError> {
    let home = dirs::home_dir().ok_or(AutoStartError::MissingHome)?;
    let launch_agents = home.join("Library/LaunchAgents");
    let plist_path = launch_agents.join("com.codex-local-launcher.launcher.plist");

    if plist_path.exists() {
        let _output = Command::new("launchctl")
            .args(["unload", &plist_path.to_string_lossy()])
            .output()
            .map_err(|e| AutoStartError::Registration(e.to_string()))?;
        fs::remove_file(&plist_path).map_err(|source| AutoStartError::WriteEntry { source })?;
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn enable_auto_start() -> Result<(), AutoStartError> {
    let home = dirs::home_dir().ok_or(AutoStartError::MissingHome)?;
    let autostart_dir = home.join(".config/autostart");

    fs::create_dir_all(&autostart_dir).map_err(|source| AutoStartError::CreateDir {
        path: autostart_dir.clone(),
        source,
    })?;

    let binary = get_binary_path()?;
    let binary_path = binary.to_string_lossy();

    let desktop = format!(
        r#"[Desktop Entry]
Type=Application
Name=Codex Local Launcher
Comment=Launch Codex with local Ollama endpoint
Exec={}
Terminal=false
Hidden=false
X-GNOME-Autostart-enabled=true
"#,
        binary_path
    );

    let desktop_path = autostart_dir.join("codex-local-launcher.desktop");
    fs::write(&desktop_path, desktop.as_bytes())
        .map_err(|source| AutoStartError::WriteEntry { source })?;

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn disable_auto_start() -> Result<(), AutoStartError> {
    let home = dirs::home_dir().ok_or(AutoStartError::MissingHome)?;
    let autostart_dir = home.join(".config/autostart");
    let desktop_path = autostart_dir.join("codex-local-launcher.desktop");

    if desktop_path.exists() {
        fs::remove_file(&desktop_path).map_err(|source| AutoStartError::WriteEntry { source })?;
    }

    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn enable_auto_start() -> Result<(), AutoStartError> {
    Err(AutoStartError::Registration(
        "Auto-start is not supported on this platform".to_string(),
    ))
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn disable_auto_start() -> Result<(), AutoStartError> {
    Err(AutoStartError::Registration(
        "Auto-start is not supported on this platform".to_string(),
    ))
}
