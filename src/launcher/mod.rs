use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;

use crate::config::LauncherConfig;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub mod autostart;

#[derive(Debug, Clone)]
pub enum LaunchTarget {
    Path(PathBuf),
    WindowsStartApp {
        app_id: String,
    },
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    MacAppBundle(PathBuf),
}

impl fmt::Display for LaunchTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LaunchTarget::Path(path) => write!(f, "Path:{}", path.display()),
            LaunchTarget::WindowsStartApp { app_id } => write!(f, "StartAppID:{app_id}"),
            LaunchTarget::MacAppBundle(path) => write!(f, "MacAppBundle:{}", path.display()),
        }
    }
}

#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("configured codexCommand was not found: {0}")]
    MissingConfiguredCommand(String),
    #[error("could not find Codex; set codexCommand in config.json")]
    CodexNotFound,
    #[error("failed to launch {program}: {source}")]
    Launch {
        program: String,
        source: std::io::Error,
    },
    #[error("{0}")]
    Platform(String),
}

pub fn resolve(config: &LauncherConfig) -> Result<LaunchTarget, LauncherError> {
    if let Some(command) = config.codex_command() {
        return resolve_configured_command(&command);
    }

    platform_resolve()
}

pub fn launch_path(
    path: &Path,
    working_directory: &Path,
    args: &[String],
    local_api: Option<(&str, &str)>,
) -> Result<(), LauncherError> {
    let mut command = Command::new(path);
    command.current_dir(working_directory).args(args);

    if let Some((base_url, api_key)) = local_api {
        command
            .env("OPENAI_BASE_URL", base_url)
            .env("OPENAI_API_KEY", api_key);
    }

    command.spawn().map_err(|source| LauncherError::Launch {
        program: path.display().to_string(),
        source,
    })?;

    Ok(())
}

pub fn launch_windows_start_app(
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))] app_id: &str,
) -> Result<(), LauncherError> {
    #[cfg(target_os = "windows")]
    {
        return windows::launch_start_app(app_id);
    }

    #[allow(unreachable_code)]
    Err(LauncherError::Platform(
        "Windows Start AppID launch is only available on Windows".to_string(),
    ))
}

pub fn launch_macos_bundle(
    #[cfg_attr(not(target_os = "macos"), allow(unused_variables))] bundle: &Path,
    #[cfg_attr(not(target_os = "macos"), allow(unused_variables))] working_directory: &Path,
    #[cfg_attr(not(target_os = "macos"), allow(unused_variables))] local_api: Option<(&str, &str)>,
) -> Result<(), LauncherError> {
    #[cfg(target_os = "macos")]
    {
        return macos::launch_bundle(bundle, working_directory, local_api);
    }

    #[allow(unreachable_code)]
    Err(LauncherError::Platform(
        "macOS app bundle launch is only available on macOS".to_string(),
    ))
}

fn resolve_configured_command(command: &str) -> Result<LaunchTarget, LauncherError> {
    let path = PathBuf::from(command);
    if path.exists() {
        return platform_target_for_path(path);
    }

    if let Some(path) = find_on_path(command) {
        return platform_target_for_path(path);
    }

    Err(LauncherError::MissingConfiguredCommand(command.to_string()))
}

fn platform_resolve() -> Result<LaunchTarget, LauncherError> {
    #[cfg(target_os = "windows")]
    {
        return windows::resolve();
    }

    #[cfg(target_os = "macos")]
    {
        return macos::resolve();
    }

    #[cfg(target_os = "linux")]
    {
        return linux::resolve();
    }

    #[allow(unreachable_code)]
    Err(LauncherError::CodexNotFound)
}

fn platform_target_for_path(path: PathBuf) -> Result<LaunchTarget, LauncherError> {
    #[cfg(target_os = "windows")]
    {
        if windows::is_packaged_codex_resource(&path) {
            if let Some(app_id) = windows::start_app_id() {
                return Ok(LaunchTarget::WindowsStartApp { app_id });
            }
        }
    }

    Ok(LaunchTarget::Path(path))
}

fn find_on_path(command: &str) -> Option<PathBuf> {
    if command.contains('/') || command.contains('\\') {
        return None;
    }

    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        for candidate in executable_candidates(&dir, command) {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn executable_candidates(dir: &Path, command: &str) -> Vec<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if Path::new(command).extension().is_some() {
            return vec![dir.join(command)];
        }
        ["exe", "cmd", "bat", "ps1"]
            .into_iter()
            .map(|ext| dir.join(format!("{command}.{ext}")))
            .collect()
    }

    #[cfg(not(target_os = "windows"))]
    {
        vec![dir.join(command)]
    }
}

fn first_existing(paths: impl IntoIterator<Item = PathBuf>) -> Option<PathBuf> {
    paths.into_iter().find(|path| path.exists())
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn common_path_commands() -> &'static [&'static str] {
    &["codex-app", "codex"]
}
