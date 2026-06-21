use std::path::{Path, PathBuf};
use std::process::Command;

use super::{common_path_commands, find_on_path, first_existing, LaunchTarget, LauncherError};

pub fn resolve() -> Result<LaunchTarget, LauncherError> {
    for name in common_path_commands() {
        if let Some(path) = find_on_path(name) {
            if is_packaged_codex_resource(&path) {
                continue;
            }
            return Ok(LaunchTarget::Path(path));
        }
    }

    if let Some(path) = first_existing(candidate_paths()) {
        return Ok(LaunchTarget::Path(path));
    }

    if let Some(app_id) = start_app_id() {
        return Ok(LaunchTarget::WindowsStartApp { app_id });
    }

    Err(LauncherError::CodexNotFound)
}

pub fn launch_start_app(app_id: &str) -> Result<(), LauncherError> {
    let target = format!("shell:AppsFolder\\{app_id}");
    let script = format!("Start-Process -FilePath '{}'", target.replace('\'', "''"));
    Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &script])
        .spawn()
        .map_err(|source| LauncherError::Launch {
            program: target,
            source,
        })?;
    Ok(())
}

pub fn is_packaged_codex_resource(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('/', "\\").to_lowercase();
    normalized.contains("\\windowsapps\\openai.codex_")
        && (normalized.ends_with("\\app\\resources\\codex.exe")
            || normalized.ends_with("\\app\\resources\\codex"))
}

pub fn start_app_id() -> Option<String> {
    let script = "(Get-StartApps Codex | Where-Object { $_.Name -eq 'Codex' -or $_.Name -like 'Codex*' } | Select-Object -First 1 -ExpandProperty AppID)";
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", script])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let app_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!app_id.is_empty()).then_some(app_id)
}

fn candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let program_files = std::env::var_os("ProgramFiles").map(PathBuf::from);
    let program_files_x86 = std::env::var_os("ProgramFiles(x86)").map(PathBuf::from);

    if let Some(local) = local {
        paths.extend([
            local.join("Programs/Codex/Codex.exe"),
            local.join("Programs/OpenAI Codex/Codex.exe"),
            local.join("Programs/codex-app/Codex.exe"),
            local.join("Programs/codex-app/codex-app.exe"),
            local.join("Codex/Codex.exe"),
            local.join("OpenAI Codex/Codex.exe"),
            local.join("OpenAI/Codex/Codex.exe"),
            local.join("openai-codex-electron/Codex.exe"),
        ]);

        for base in [
            local.join("Programs/Codex"),
            local.join("Programs/OpenAI Codex"),
            local.join("Codex"),
            local.join("OpenAI Codex"),
            local.join("OpenAI/Codex"),
            local.join("openai-codex-electron"),
        ] {
            if let Ok(entries) = std::fs::read_dir(base) {
                for entry in entries.flatten() {
                    let app_path = entry.path().join("Codex.exe");
                    if entry.file_name().to_string_lossy().starts_with("app-") {
                        paths.push(app_path);
                    }
                }
            }
        }
    }

    if let Some(program_files) = program_files {
        paths.extend([
            program_files.join("Codex/Codex.exe"),
            program_files.join("codex-app/Codex.exe"),
            program_files.join("codex-app/codex-app.exe"),
        ]);
    }

    if let Some(program_files_x86) = program_files_x86 {
        paths.extend([
            program_files_x86.join("Codex/Codex.exe"),
            program_files_x86.join("codex-app/Codex.exe"),
            program_files_x86.join("codex-app/codex-app.exe"),
        ]);
    }

    paths
}
