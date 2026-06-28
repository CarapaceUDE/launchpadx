use std::path::{Path, PathBuf};
use std::time::Duration;

use reqwest::blocking::Client;

use crate::app_logic;
use crate::config::LauncherConfig;
use crate::launcher;
use crate::ollama;

#[derive(Debug)]
struct CheckResult {
    name: String,
    success: bool,
    status: Option<u16>,
    detail: String,
}

pub fn run(config_path: &Path, project_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config = LauncherConfig::read(config_path)?;
    let base_url = config.openai_base_url()?;
    let tags_url = ollama::tags_url_from_base(&base_url)?;
    let codex_api_base_url = config.codex_api_base_url();
    let working_directory = config.working_directory(project_root)?;
    let api_key = config.api_key_if_configured();

    println!("=== Codex Launchpad Diagnostic ===");
    print_section("Configuration");
    println!("Config path       : {}", config_path.display());
    println!("Endpoint          : {base_url}");
    println!("Tags endpoint     : {tags_url}");
    println!("Codex API         : {codex_api_base_url}");
    println!(
        "API key           : {}",
        format_configured_secret(api_key.as_deref())
    );
    println!("Working directory : {}", working_directory.display());
    println!(
        "Codex command     : {}",
        config
            .codex_command()
            .as_deref()
            .unwrap_or("(auto-detect)")
    );
    println!("Codex launch probe: {}", codex_launch_probe(&config));

    print_section("Local Checks");
    print_check(&CheckResult {
        name: "Config file parses".to_string(),
        success: true,
        status: None,
        detail: String::new(),
    });
    print_check(&CheckResult {
        name: "Working directory exists".to_string(),
        success: working_directory.exists(),
        status: None,
        detail: if working_directory.exists() {
            String::new()
        } else {
            format!("Missing path: {}", working_directory.display())
        },
    });
    print_check(&CheckResult {
        name: "Launcher binary present".to_string(),
        success: find_launcher_binary(project_root).is_some(),
        status: None,
        detail: find_launcher_binary(project_root)
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| {
                "Build the project first with `cargo build --release` or `scripts/build.sh`"
                    .to_string()
            }),
    });

    print_section("Network Checks");
    print_check(&probe_http(
        "Ollama-compatible tags endpoint",
        &tags_url,
        api_key.as_deref(),
    ));
    print_check(&probe_http(
        "Codex API health",
        &format!("{codex_api_base_url}/health"),
        None,
    ));

    let codex = app_logic::detect_codex_process(&config, project_root);
    print_section("Codex Process");
    print_check(&CheckResult {
        name: "Codex running".to_string(),
        success: codex.running,
        status: None,
        detail: if codex.running {
            format!(
                "Detected via {} (pid: {})",
                codex.method.unwrap_or_else(|| "unknown".to_string()),
                codex
                    .pid
                    .map(|pid| pid.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            )
        } else {
            String::new()
        },
    });
    print_check(&CheckResult {
        name: "Endpoint reachable".to_string(),
        success: app_logic::endpoint_reachable(&config),
        status: None,
        detail: String::new(),
    });
    print_check(&CheckResult {
        name: "Codex API ready".to_string(),
        success: if codex.running {
            app_logic::codex_api_ready(&config)
        } else {
            false
        },
        status: None,
        detail: String::new(),
    });

    println!();
    println!("Diagnostic complete.");
    Ok(())
}

fn codex_launch_probe(config: &LauncherConfig) -> String {
    match launcher::resolve(config) {
        Ok(target) => target.to_string(),
        Err(err) => err.to_string(),
    }
}

fn find_launcher_binary(project_root: &Path) -> Option<PathBuf> {
    let binary_name = if cfg!(windows) {
        "codex-launchpad.exe"
    } else {
        "codex-launchpad"
    };

    let candidates = [
        project_root.join("target/release").join(binary_name),
        project_root.join("target/debug").join(binary_name),
    ];

    candidates.into_iter().find(|path| path.is_file()).or_else(|| {
        std::env::current_exe()
            .ok()
            .filter(|path| path.file_name().is_some_and(|name| name == binary_name))
    })
}

fn format_configured_secret(value: Option<&str>) -> &'static str {
    match value {
        None => "(not set)",
        Some(value) if value.trim().is_empty() || value == "replace-with-your-api-key" => {
            "(not set)"
        }
        Some(_) => "Configured (redacted)",
    }
}

fn probe_http(name: &str, url: &str, api_key: Option<&str>) -> CheckResult {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build();

    let Ok(client) = client else {
        return CheckResult {
            name: name.to_string(),
            success: false,
            status: None,
            detail: "Could not create HTTP client".to_string(),
        };
    };

    let mut request = client.get(url);
    if let Some(api_key) = api_key.filter(|value| !value.trim().is_empty()) {
        request = request.bearer_auth(api_key);
    }

    match request.send() {
        Ok(response) => {
            let status = response.status().as_u16();
            CheckResult {
                name: name.to_string(),
                success: response.status().is_success(),
                status: Some(status),
                detail: if response.status().is_success() {
                    "OK".to_string()
                } else {
                    format!("HTTP {status}")
                },
            }
        }
        Err(err) => CheckResult {
            name: name.to_string(),
            success: false,
            status: None,
            detail: err.to_string(),
        },
    }
}

fn print_section(title: &str) {
    println!();
    println!("== {title} ==");
}

fn print_check(result: &CheckResult) {
    let status = if result.success { "PASS" } else { "FAIL" };
    let code = result
        .status
        .map(|value| format!(" [{value}]"))
        .unwrap_or_default();
    println!("{status:<5} {}{code}", result.name);
    if !result.success && !result.detail.is_empty() {
        println!("      {}", result.detail);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_secret_redacts_configured_values() {
        assert_eq!(format_configured_secret(None), "(not set)");
        assert_eq!(format_configured_secret(Some("")), "(not set)");
        assert_eq!(
            format_configured_secret(Some("replace-with-your-api-key")),
            "(not set)"
        );
        assert_eq!(format_configured_secret(Some("sk-test")), "Configured (redacted)");
    }
}