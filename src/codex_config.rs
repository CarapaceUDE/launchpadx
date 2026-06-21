use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml_edit::{value, DocumentMut, Item, Table};

use crate::config::{ApiKeyMode, LauncherConfig};

const ENV_KEY_NAME: &str = "OPENAI_API_KEY";
const BACKUP_DIR_NAME: &str = "codex-local-launcher";

#[derive(Debug, Error)]
pub enum CodexConfigError {
    #[error("could not locate home directory for ~/.codex/config.toml")]
    MissingHome,
    #[error("could not read Codex config {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not parse Codex config {path}: {source}")]
    Parse {
        path: PathBuf,
        source: Box<toml_edit::TomlError>,
    },
    #[error("could not create directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not write Codex config {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not back up Codex config from {source_path} to {backup_path}: {source}")]
    Backup {
        source_path: PathBuf,
        backup_path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not parse restore state {path}: {source}")]
    RestoreStateParse {
        path: PathBuf,
        source: serde_json::Error,
    },
}

#[derive(Debug, Clone)]
pub struct PersistentCodexConfig {
    pub config_path: PathBuf,
    pub model: Option<String>,
    pub provider_id: String,
    pub provider_name: String,
    pub base_url: String,
    pub api_key: String,
    pub api_key_mode: ApiKeyMode,
}

impl PersistentCodexConfig {
    pub fn from_launcher_config(
        config: &LauncherConfig,
        model: Option<String>,
        base_url: String,
        api_key: String,
    ) -> Result<Self, CodexConfigError> {
        Ok(Self {
            config_path: config
                .codex_config_path()
                .map(Ok)
                .unwrap_or_else(default_codex_config_path)?,
            model,
            provider_id: config.codex_provider_id(),
            provider_name: config.codex_provider_name(),
            base_url,
            api_key,
            api_key_mode: config.codex_api_key_mode(),
        })
    }
}

pub fn default_codex_config_path() -> Result<PathBuf, CodexConfigError> {
    let home = dirs::home_dir().ok_or(CodexConfigError::MissingHome)?;
    Ok(home.join(".codex").join("config.toml"))
}

pub fn apply(settings: &PersistentCodexConfig) -> Result<(), CodexConfigError> {
    let mut document = read_document(&settings.config_path)?;
    save_restore_state_if_needed(&document, settings)?;
    apply_to_document(&mut document, settings);
    write_document(&settings.config_path, &document)
}

pub fn restore(config: &LauncherConfig) -> Result<PathBuf, CodexConfigError> {
    let config_path = config
        .codex_config_path()
        .map(Ok)
        .unwrap_or_else(default_codex_config_path)?;
    let provider_id = config.codex_provider_id();
    let mut document = read_document(&config_path)?;
    let restore_state = read_restore_state(&config_path)?;

    restore_root_values(&mut document, &provider_id, restore_state.as_ref());
    remove_provider(&mut document, &provider_id);
    write_document(&config_path, &document)?;
    Ok(config_path)
}

fn read_document(path: &Path) -> Result<DocumentMut, CodexConfigError> {
    if !path.exists() {
        return Ok(DocumentMut::new());
    }

    let text = fs::read_to_string(path).map_err(|source| CodexConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    text.parse::<DocumentMut>()
        .map_err(|source| CodexConfigError::Parse {
            path: path.to_path_buf(),
            source: Box::new(source),
        })
}

fn apply_to_document(document: &mut DocumentMut, settings: &PersistentCodexConfig) {
    if let Some(model) = &settings.model {
        document["model"] = value(model);
    }
    document["model_provider"] = value(&settings.provider_id);

    let providers = document["model_providers"].or_insert(Item::Table(Table::new()));
    let provider = providers[&settings.provider_id].or_insert(Item::Table(Table::new()));

    provider["name"] = value(&settings.provider_name);
    provider["base_url"] = value(&settings.base_url);
    provider["wire_api"] = value("responses");
    provider["requires_openai_auth"] = value(false);

    if let Some(table) = provider.as_table_mut() {
        table.remove("env_key");
        table.remove("experimental_bearer_token");
        table.remove("auth");
    }

    match settings.api_key_mode {
        ApiKeyMode::EnvKey => provider["env_key"] = value(ENV_KEY_NAME),
        ApiKeyMode::ExperimentalBearerToken => {
            provider["experimental_bearer_token"] = value(&settings.api_key)
        }
        ApiKeyMode::None => {}
    }
}

fn save_restore_state_if_needed(
    document: &DocumentMut,
    settings: &PersistentCodexConfig,
) -> Result<(), CodexConfigError> {
    if root_string(document, "model_provider").as_deref() == Some(&settings.provider_id) {
        return Ok(());
    }

    let path = restore_state_path(&settings.config_path);
    if path.exists() {
        return Ok(());
    }

    let state = RestoreState::from_document(document);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| CodexConfigError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let data = serde_json::to_vec_pretty(&state).map_err(|source| {
        CodexConfigError::RestoreStateParse {
            path: path.clone(),
            source,
        }
    })?;
    fs::write(&path, data).map_err(|source| CodexConfigError::Write { path, source })
}

fn read_restore_state(path: &Path) -> Result<Option<RestoreState>, CodexConfigError> {
    let path = restore_state_path(path);
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&path).map_err(|source| CodexConfigError::Read {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&data)
        .map(Some)
        .map_err(|source| CodexConfigError::RestoreStateParse { path, source })
}

fn restore_root_values(
    document: &mut DocumentMut,
    provider_id: &str,
    restore_state: Option<&RestoreState>,
) {
    if root_string(document, "model_provider").as_deref() != Some(provider_id) {
        return;
    }

    if let Some(state) = restore_state {
        restore_root_string(document, "profile", state.had_profile, &state.profile);
        restore_root_string(document, "model", state.had_model, &state.model);
        restore_root_string(
            document,
            "model_provider",
            state.had_model_provider,
            &state.model_provider,
        );
        restore_root_string(
            document,
            "model_catalog_json",
            state.had_model_catalog_json,
            &state.model_catalog_json,
        );
    } else {
        document.as_table_mut().remove("model_provider");
    }
}

fn remove_provider(document: &mut DocumentMut, provider_id: &str) {
    if let Some(providers) = document["model_providers"].as_table_mut() {
        providers.remove(provider_id);
        if providers.is_empty() {
            document.as_table_mut().remove("model_providers");
        }
    }
}

fn restore_root_string(
    document: &mut DocumentMut,
    key: &str,
    had_value: bool,
    value: &Option<String>,
) {
    if had_value {
        if let Some(value) = value {
            document[key] = value.clone().into();
        }
    } else {
        document.as_table_mut().remove(key);
    }
}

fn root_string(document: &DocumentMut, key: &str) -> Option<String> {
    document
        .as_table()
        .get(key)
        .and_then(Item::as_str)
        .map(str::to_string)
}

fn write_document(path: &Path, document: &DocumentMut) -> Result<(), CodexConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| CodexConfigError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    if path.exists() {
        let backup_path = backup_path(path);
        if let Some(parent) = backup_path.parent() {
            fs::create_dir_all(parent).map_err(|source| CodexConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::copy(path, &backup_path).map_err(|source| CodexConfigError::Backup {
            source_path: path.to_path_buf(),
            backup_path,
            source,
        })?;
    }

    fs::write(path, document.to_string()).map_err(|source| CodexConfigError::Write {
        path: path.to_path_buf(),
        source,
    })
}

fn backup_path(path: &Path) -> PathBuf {
    let timestamp = timestamp_for_filename();
    let backup_root = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("backups")
        .join(BACKUP_DIR_NAME);
    backup_root.join(format!("config.toml.{timestamp}.bak"))
}

fn restore_state_path(path: &Path) -> PathBuf {
    path.parent()
        .unwrap_or_else(|| Path::new("."))
        .join("backups")
        .join(BACKUP_DIR_NAME)
        .join("restore-state.json")
}

fn timestamp_for_filename() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    seconds.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RestoreState {
    had_profile: bool,
    profile: Option<String>,
    had_model: bool,
    model: Option<String>,
    had_model_provider: bool,
    model_provider: Option<String>,
    had_model_catalog_json: bool,
    model_catalog_json: Option<String>,
}

impl RestoreState {
    fn from_document(document: &DocumentMut) -> Self {
        let profile = root_string(document, "profile");
        let model = root_string(document, "model");
        let model_provider = root_string(document, "model_provider");
        let model_catalog_json = root_string(document, "model_catalog_json");

        Self {
            had_profile: profile.is_some(),
            profile,
            had_model: model.is_some(),
            model,
            had_model_provider: model_provider.is_some(),
            model_provider,
            had_model_catalog_json: model_catalog_json.is_some(),
            model_catalog_json,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(path: PathBuf) -> PersistentCodexConfig {
        PersistentCodexConfig {
            config_path: path,
            model: Some("llama3.2".to_string()),
            provider_id: "codex-local-launcher".to_string(),
            provider_name: "Local Ollama".to_string(),
            base_url: "http://127.0.0.1:11434/v1".to_string(),
            api_key: "test-key".to_string(),
            api_key_mode: ApiKeyMode::ExperimentalBearerToken,
        }
    }

    #[test]
    fn applies_provider_without_removing_existing_sections() {
        let mut document = r#"
approval_policy = "never"

[desktop]
conversationDetailMode = "STEPS_COMMANDS"
"#
        .parse::<DocumentMut>()
        .expect("valid TOML");

        apply_to_document(&mut document, &settings(PathBuf::from("unused")));
        let text = document.to_string();

        assert!(text.contains(r#"approval_policy = "never""#));
        assert!(text.contains("[desktop]"));
        assert!(text.contains(r#"model = "llama3.2""#));
        assert!(text.contains(r#"model_provider = "codex-local-launcher""#));
        assert!(text.contains("[model_providers.codex-local-launcher]"));
        assert!(text.contains(r#"base_url = "http://127.0.0.1:11434/v1""#));
        assert!(text.contains(r#"experimental_bearer_token = "test-key""#));
    }

    #[test]
    fn env_key_mode_removes_persisted_bearer_token() {
        let mut document = r#"
[model_providers.codex-local-launcher]
experimental_bearer_token = "old"
"#
        .parse::<DocumentMut>()
        .expect("valid TOML");
        let mut settings = settings(PathBuf::from("unused"));
        settings.api_key_mode = ApiKeyMode::EnvKey;

        apply_to_document(&mut document, &settings);
        let text = document.to_string();

        assert!(text.contains(r#"env_key = "OPENAI_API_KEY""#));
        assert!(!text.contains("experimental_bearer_token"));
    }

    #[test]
    fn model_is_optional() {
        let mut document = r#"model = "existing""#.parse::<DocumentMut>().expect("valid TOML");
        let mut settings = settings(PathBuf::from("unused"));
        settings.model = None;

        apply_to_document(&mut document, &settings);

        assert!(document.to_string().contains(r#"model = "existing""#));
    }

    #[test]
    fn restore_returns_previous_root_values_and_removes_provider() {
        let mut document = r#"
model = "llama3.2"
model_provider = "codex-local-launcher"

[model_providers.codex-local-launcher]
name = "Local Ollama"
"#
        .parse::<DocumentMut>()
        .expect("valid TOML");
        let state = RestoreState {
            had_profile: false,
            profile: None,
            had_model: true,
            model: Some("gpt-5.5".to_string()),
            had_model_provider: false,
            model_provider: None,
            had_model_catalog_json: false,
            model_catalog_json: None,
        };

        restore_root_values(&mut document, "codex-local-launcher", Some(&state));
        remove_provider(&mut document, "codex-local-launcher");
        let text = document.to_string();

        assert!(text.contains(r#"model = "gpt-5.5""#));
        assert!(!text.contains("model_provider"));
        assert!(!text.contains("codex-local-launcher"));
    }

    #[test]
    fn restore_does_not_change_root_values_when_not_active_provider() {
        let mut document = r#"
model = "gpt-5.5"
model_provider = "openai"

[model_providers.codex-local-launcher]
name = "Local Ollama"
"#
        .parse::<DocumentMut>()
        .expect("valid TOML");

        restore_root_values(&mut document, "codex-local-launcher", None);
        remove_provider(&mut document, "codex-local-launcher");
        let text = document.to_string();

        assert!(text.contains(r#"model = "gpt-5.5""#));
        assert!(text.contains(r#"model_provider = "openai""#));
        assert!(!text.contains("Local Ollama"));
    }
}
