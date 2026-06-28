use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing config file: {0}; copy config.example.json to config.json and fill it in")]
    Missing(PathBuf),
    #[error("could not read config file {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not parse config file {path}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("could not serialize config file {path}: {source}")]
    Serialize {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("could not write config file {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("config.json must set either openaiBaseUrl or ollamaIp")]
    MissingBaseUrl,
    #[error("config.json still has the example ollamaIp; set your actual IP or openaiBaseUrl")]
    ExampleOllamaIp,
    #[error("config.json must set apiKey")]
    MissingApiKey,
    #[error("config.json still has the example apiKey")]
    ExampleApiKey,
    #[error("workingDirectory does not exist: {0}")]
    MissingWorkingDirectory(PathBuf),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherConfig {
    #[serde(default)]
    pub auto_start: Option<bool>,
    pub openai_base_url: Option<String>,
    pub ollama_ip: Option<String>,
    pub ollama_port: Option<u16>,
    pub ollama_scheme: Option<String>,
    pub api_key: Option<String>,
    pub persist_codex_config: Option<bool>,
    pub codex_model: Option<String>,
    pub codex_provider_id: Option<String>,
    pub codex_provider_name: Option<String>,
    pub codex_api_key_mode: Option<ApiKeyMode>,
    pub codex_config_path: Option<String>,
    pub codex_command: Option<String>,
    pub codex_api_port: Option<u16>,
    pub codex_api_scheme: Option<String>,
    pub discover_ollama_models: Option<bool>,
    pub codex_args: Option<Vec<String>>,
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ApiKeyMode {
    EnvKey,
    #[default]
    ExperimentalBearerToken,
    None,
}

impl LauncherConfig {
    pub fn read(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::Missing(path.to_path_buf()));
        }

        let data = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;

        serde_json::from_str(data.trim_start_matches('\u{feff}')).map_err(|source| {
            ConfigError::Parse {
                path: path.to_path_buf(),
                source,
            }
        })
    }

    pub fn write(&self, path: &Path) -> Result<(), ConfigError> {
        let data = serde_json::to_string_pretty(self).map_err(|source| ConfigError::Serialize {
            path: path.to_path_buf(),
            source,
        })?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::Write {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        // Atomic write: write to temp file then rename to avoid corruption on crash
        let tmp_path = path.with_extension("json.tmp");
        fs::write(&tmp_path, &data).map_err(|source| ConfigError::Write {
            path: tmp_path.clone(),
            source,
        })?;
        fs::rename(&tmp_path, path).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn openai_base_url(&self) -> Result<String, ConfigError> {
        if let Some(host) = non_empty(&self.ollama_ip) {
            if host == "100.64.0.10" {
                return Err(ConfigError::ExampleOllamaIp);
            }

            if host.starts_with("http://") || host.starts_with("https://") {
                return Ok(ensure_v1(host));
            }

            let scheme = non_empty(&self.ollama_scheme).unwrap_or_else(|| "http".to_string());
            let port = self.ollama_port.unwrap_or(11434);
            let host = if host.contains(':') {
                format!("[{host}]")
            } else {
                host
            };

            return Ok(format!("{scheme}://{host}:{port}/v1"));
        }

        if let Some(base_url) = non_empty(&self.openai_base_url) {
            return Ok(ensure_v1(base_url));
        }

        Err(ConfigError::MissingBaseUrl)
    }

    pub fn merge_from(&mut self, other: &LauncherConfig) {
        merge_option(&mut self.auto_start, &other.auto_start);
        merge_option(&mut self.openai_base_url, &other.openai_base_url);
        merge_option(&mut self.ollama_ip, &other.ollama_ip);
        merge_option(&mut self.ollama_port, &other.ollama_port);
        merge_option(&mut self.ollama_scheme, &other.ollama_scheme);
        merge_option(&mut self.api_key, &other.api_key);
        merge_option(&mut self.persist_codex_config, &other.persist_codex_config);
        merge_option(&mut self.codex_model, &other.codex_model);
        merge_option(&mut self.codex_provider_id, &other.codex_provider_id);
        merge_option(&mut self.codex_provider_name, &other.codex_provider_name);
        merge_option(&mut self.codex_api_key_mode, &other.codex_api_key_mode);
        merge_option(&mut self.codex_config_path, &other.codex_config_path);
        merge_option(&mut self.codex_command, &other.codex_command);
        merge_option(&mut self.codex_api_port, &other.codex_api_port);
        merge_option(&mut self.codex_api_scheme, &other.codex_api_scheme);
        merge_option(
            &mut self.discover_ollama_models,
            &other.discover_ollama_models,
        );
        merge_option(&mut self.codex_args, &other.codex_args);
        merge_option(&mut self.working_directory, &other.working_directory);
    }

    pub fn api_key(&self) -> Result<String, ConfigError> {
        let Some(api_key) = non_empty(&self.api_key) else {
            return Err(ConfigError::MissingApiKey);
        };
        if api_key == "replace-with-your-api-key" {
            return Err(ConfigError::ExampleApiKey);
        }
        Ok(api_key)
    }

    pub fn api_key_if_configured(&self) -> Option<String> {
        non_empty(&self.api_key).filter(|value| value != "replace-with-your-api-key")
    }

    pub fn codex_args(&self) -> Vec<String> {
        self.codex_args.clone().unwrap_or_default()
    }

    pub fn persist_codex_config(&self) -> bool {
        self.persist_codex_config.unwrap_or(true)
    }

    pub fn codex_model(&self) -> Option<String> {
        non_empty(&self.codex_model)
    }

    pub fn discover_ollama_models(&self) -> bool {
        self.discover_ollama_models.unwrap_or(true)
    }

    pub fn codex_provider_id(&self) -> String {
        non_empty(&self.codex_provider_id).unwrap_or_else(|| "codex-launchpad".to_string())
    }

    pub fn codex_provider_name(&self) -> String {
        non_empty(&self.codex_provider_name)
            .unwrap_or_else(|| crate::branding::DEFAULT_PROVIDER_NAME.to_string())
    }

    pub fn codex_api_key_mode(&self) -> ApiKeyMode {
        self.codex_api_key_mode
            .unwrap_or(ApiKeyMode::ExperimentalBearerToken)
    }

    pub fn codex_config_path(&self) -> Option<PathBuf> {
        non_empty(&self.codex_config_path).map(|value| PathBuf::from(expand_env_vars(&value)))
    }

    pub fn codex_command(&self) -> Option<String> {
        non_empty(&self.codex_command).map(|value| expand_env_vars(&value))
    }

    pub fn codex_api_port(&self) -> u16 {
        self.codex_api_port.unwrap_or(4000)
    }

    pub fn codex_api_scheme(&self) -> String {
        non_empty(&self.codex_api_scheme).unwrap_or_else(|| "http".to_string())
    }

    pub fn codex_api_base_url(&self) -> String {
        format!(
            "{}://127.0.0.1:{}",
            self.codex_api_scheme(),
            self.codex_api_port()
        )
    }

    pub fn working_directory(&self, default: &Path) -> Result<PathBuf, ConfigError> {
        let path = non_empty(&self.working_directory)
            .map(|value| PathBuf::from(expand_env_vars(&value)))
            .unwrap_or_else(|| default.to_path_buf());
        if !path.exists() {
            return Err(ConfigError::MissingWorkingDirectory(path));
        }
        Ok(path)
    }
}

fn merge_option<T: Clone>(target: &mut Option<T>, incoming: &Option<T>) {
    if incoming.is_some() {
        *target = incoming.clone();
    }
}

fn non_empty(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

fn ensure_v1(raw: String) -> String {
    let base = raw.trim_end_matches('/');
    if base.ends_with("/v1") {
        base.to_string()
    } else {
        format!("{base}/v1")
    }
}

fn expand_env_vars(value: &str) -> String {
    let mut expanded = value.to_string();

    // Collect env vars to avoid borrowing std::env::vars mutably
    let vars: Vec<(String, String)> = std::env::vars().collect();

    for (key, val) in &vars {
        // Replace %KEY% with boundary awareness to avoid partial matches
        let pattern = format!("%{}%", key);
        let mut result = String::new();
        let mut search_start = 0;
        let haystack = &expanded;
        while let Some(pos) = haystack[search_start..].find(&pattern) {
            let abs_pos = search_start + pos;
            let before_ok = abs_pos == 0
                || !haystack
                    .chars()
                    .nth(abs_pos - 1)
                    .unwrap_or(' ')
                    .is_alphanumeric();
            let after_pos = abs_pos + pattern.len();
            let after_ok = after_pos >= haystack.len()
                || !haystack
                    .chars()
                    .nth(after_pos)
                    .unwrap_or(' ')
                    .is_alphanumeric();
            if before_ok && after_ok {
                result.push_str(&haystack[search_start..abs_pos]);
                result.push_str(val);
                search_start = after_pos;
            } else {
                search_start = abs_pos + 1;
            }
        }
        if search_start == 0 {
            result = expanded.clone();
        } else {
            result.push_str(&haystack[search_start..]);
        }
        expanded = result;

        // For ${KEY} and ${KEY} patterns, use exact non-overlapping replace
        expanded = expanded.replacen(&format!("${{{key}}}"), val, 1);
        expanded = expanded.replacen(&format!("${key}"), val, 1);
    }

    expanded
}

#[cfg(test)]
mod tests {
    use super::LauncherConfig;
    use serde_json::Value;
    use std::collections::HashSet;

    #[test]
    fn config_example_matches_public_schema() {
        let example = include_str!("../config.example.json").trim_start_matches('\u{feff}');
        let parsed: LauncherConfig =
            serde_json::from_str(example).expect("config.example.json should parse");
        let json: Value =
            serde_json::from_str(example).expect("config.example.json should be valid JSON");
        let keys = json
            .as_object()
            .expect("config.example.json should be a JSON object")
            .keys()
            .cloned()
            .collect::<HashSet<_>>();

        assert!(
            parsed.codex_api_port.is_some(),
            "config.example.json should include codexApiPort"
        );
        assert!(
            parsed.codex_api_scheme.is_some(),
            "config.example.json should include codexApiScheme"
        );
        assert!(
            keys.contains("openaiBaseUrl"),
            "config.example.json should include openaiBaseUrl"
        );
    }
}
