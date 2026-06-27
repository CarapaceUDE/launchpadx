use std::path::PathBuf;

use super::{find_on_path, first_existing, LaunchTarget, LauncherError};

pub fn resolve() -> Result<LaunchTarget, LauncherError> {
    for name in common_path_commands_for_linux() {
        if let Some(path) = find_on_path(name) {
            return Ok(LaunchTarget::Path(path));
        }
    }

    if let Some(path) = first_existing(candidate_paths()) {
        return Ok(LaunchTarget::Path(path));
    }

    Err(LauncherError::CodexNotFound(
        "searched PATH and common install locations".to_string(),
    ))
}

fn common_path_commands_for_linux() -> &'static [&'static str] {
    &["codex"]
}

fn candidate_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("/usr/local/bin/codex"),
        PathBuf::from("/usr/bin/codex"),
        PathBuf::from("/opt/Codex/codex"),
        PathBuf::from("/opt/codex/codex"),
    ];

    if let Some(home) = dirs::home_dir() {
        paths.extend([
            home.join(".local/bin/codex"),
            home.join("Applications/Codex.AppImage"),
            home.join(".local/share/applications/Codex.AppImage"),
        ]);
    }

    paths
}
