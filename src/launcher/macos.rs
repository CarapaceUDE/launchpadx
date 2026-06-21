use std::path::{Path, PathBuf};
use std::process::Command;

use super::{common_path_commands, find_on_path, first_existing, LaunchTarget, LauncherError};

pub fn resolve() -> Result<LaunchTarget, LauncherError> {
    for name in common_path_commands() {
        if let Some(path) = find_on_path(name) {
            return platform_target_for_path(path);
        }
    }

    if let Some(bundle) = first_existing(candidate_bundles()) {
        return Ok(LaunchTarget::MacAppBundle(bundle));
    }

    Err(LauncherError::CodexNotFound)
}

fn platform_target_for_path(path: PathBuf) -> Result<LaunchTarget, LauncherError> {
    Ok(LaunchTarget::Path(path))
}

pub fn launch_bundle(
    bundle: &Path,
    working_directory: &Path,
    base_url: &str,
    api_key: &str,
) -> Result<(), LauncherError> {
    let executable = bundle.join("Contents/MacOS/Codex");
    if executable.exists() {
        return super::launch_path(&executable, working_directory, &[], base_url, api_key);
    }

    Command::new("open")
        .arg(bundle)
        .env("OPENAI_BASE_URL", base_url)
        .env("OPENAI_API_KEY", api_key)
        .spawn()
        .map_err(|source| LauncherError::Launch {
            program: bundle.display().to_string(),
            source,
        })?;
    Ok(())
}

fn candidate_bundles() -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from("/Applications/Codex.app")];
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join("Applications/Codex.app"));
    }
    paths
}
