use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use super::{common_path_commands, find_on_path, first_existing, LaunchTarget, LauncherError};

pub fn resolve() -> Result<LaunchTarget, LauncherError> {
    let mut saw_packaged_shim = false;

    for name in common_path_commands() {
        if let Some(path) = find_on_path(name) {
            if is_packaged_codex_resource(&path) {
                saw_packaged_shim = true;
                continue;
            }
            return Ok(LaunchTarget::Path(path));
        }
    }

    if saw_packaged_shim {
        if let Some(app_id) = start_app_id() {
            return Ok(LaunchTarget::WindowsStartApp { app_id });
        }
    }

    if let Some(path) = first_existing(candidate_paths()) {
        return Ok(LaunchTarget::Path(path));
    }

    if let Some(app_id) = start_app_id() {
        return Ok(LaunchTarget::WindowsStartApp { app_id });
    }

    Err(LauncherError::CodexNotFound(
        "searched PATH, common install folders, and the Windows Start menu".to_string(),
    ))
}

pub fn launch_start_app(app_id: &str) -> Result<(), LauncherError> {
    let target = format!("shell:AppsFolder\\{app_id}");

    let launched =
        Command::new("explorer.exe").arg(&target).spawn().is_ok() || launch_via_powershell(&target);

    if !launched {
        return Err(LauncherError::Launch {
            program: target.clone(),
            source: std::io::Error::other("explorer.exe and PowerShell Start-Process both failed"),
        });
    }

    if wait_for_codex_process(20) {
        return Ok(());
    }

    Ok(())
}

pub fn is_packaged_codex_resource(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('/', "\\").to_lowercase();
    normalized.contains("\\windowsapps\\openai.codex_")
        && (normalized.ends_with("\\app\\resources\\codex.exe")
            || normalized.ends_with("\\app\\resources\\codex"))
}

pub fn start_app_id() -> Option<String> {
    let script = r#"
$apps = Get-StartApps | Where-Object {
    $_.AppID -like '*OpenAI.Codex*' -or
    $_.Name -eq 'Codex' -or
    $_.Name -like 'Codex*'
}
$apps | Select-Object -First 1 -ExpandProperty AppID
"#;
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

fn launch_via_powershell(target: &str) -> bool {
    let script = format!("Start-Process -FilePath '{}'", target.replace('\'', "''"));
    Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &script])
        .spawn()
        .is_ok()
}

pub fn wait_for_codex_process(timeout_secs: u64) -> bool {
    let deadline = Duration::from_secs(timeout_secs);
    let start = std::time::Instant::now();

    while start.elapsed() < deadline {
        if codex_process_visible() {
            return true;
        }
        thread::sleep(Duration::from_millis(500));
    }

    false
}

pub fn codex_process_visible() -> bool {
    let Ok(output) = Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output()
    else {
        return false;
    };
    if !output.status.success() {
        return false;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
    stdout.contains("codex.exe")
        || stdout.contains("codex-app.exe")
        || stdout.contains("\\windowsapps\\openai.codex_")
}

pub fn cli_install_paths() -> Vec<PathBuf> {
    let mut paths = discover_spawnable_codex_bins();
    paths.extend(candidate_paths());
    paths.extend(discover_via_where("codex"));
    paths.extend(discover_via_where("codex-app"));
    paths.retain(|path| !is_blocked_windowsapps_cli(path));
    paths
}

/// Store installs block direct execution of `WindowsApps\\...\\codex.exe`, but Codex
/// mirrors a spawnable CLI under `%LOCALAPPDATA%\\OpenAI\\Codex\\bin\\...`.
pub fn discover_spawnable_codex_bins() -> Vec<PathBuf> {
    let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) else {
        return Vec::new();
    };

    let mut paths = Vec::new();
    paths.extend(collect_codex_bins_under(
        &local.join("OpenAI").join("Codex").join("bin"),
    ));

    let packages = local.join("Packages");
    if let Ok(entries) = std::fs::read_dir(&packages) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            if !name.starts_with("openai.codex_") {
                continue;
            }
            let cache_bin = entry
                .path()
                .join("LocalCache")
                .join("Local")
                .join("OpenAI")
                .join("Codex")
                .join("bin");
            paths.extend(collect_codex_bins_under(&cache_bin));
            let flat = cache_bin.join("codex.exe");
            if flat.is_file() {
                paths.push(flat);
            }
        }
    }

    paths.sort_by_key(|path| std::cmp::Reverse(modified_key(path)));
    paths
}

fn collect_codex_bins_under(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return paths,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let cli = path.join("codex.exe");
            if cli.is_file() {
                paths.push(cli);
            }
        } else if path
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case("codex.exe"))
        {
            paths.push(path);
        }
    }

    paths
}

fn modified_key(path: &Path) -> u64 {
    std::fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub fn is_blocked_windowsapps_cli(path: &Path) -> bool {
    path.to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase()
        .contains("\\windowsapps\\openai.codex_")
}

fn discover_via_where(command: &str) -> Vec<PathBuf> {
    let output = match Command::new("where.exe").arg(command).output() {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .collect()
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
            paths.extend(latest_app_bundle_paths(&base));
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

fn latest_app_bundle_paths(base: &Path) -> Vec<PathBuf> {
    let entries = match std::fs::read_dir(base) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut app_dirs: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("app-"))
        })
        .collect();

    app_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    app_dirs
        .into_iter()
        .map(|dir| dir.join("Codex.exe"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaged_codex_resource_matches_store_shim() {
        let path = Path::new(
            r"C:\Program Files\WindowsApps\OpenAI.Codex_1.2.3.0_x64__8wekyb3d8bbwe\app\resources\codex.exe",
        );
        assert!(is_packaged_codex_resource(path));
    }

    #[test]
    fn packaged_codex_resource_ignores_standalone_install() {
        let path = Path::new(r"C:\Users\alice\AppData\Local\Programs\Codex\Codex.exe");
        assert!(!is_packaged_codex_resource(path));
    }

    #[test]
    fn blocks_windowsapps_shim_paths() {
        let path = Path::new(
            r"C:\Program Files\WindowsApps\OpenAI.Codex_26.623.8305.0_x64__2p2nqsd0c76g0\app\resources\codex.exe",
        );
        assert!(is_blocked_windowsapps_cli(path));
    }
}
