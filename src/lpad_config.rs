use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexConfigInspection {
    pub config_path: PathBuf,
    pub exists: bool,
    pub model: Option<String>,
    pub model_provider: Option<String>,
    pub managed_by_launcher: bool,
    pub launcher_base_url: Option<String>,
    pub restore_state_available: bool,
}
use thiserror::Error;
use toml_edit::{value, DocumentMut, Item, Table};

use crate::config::{ApiKeyMode, LauncherConfig};

const ENV_KEY_NAME: &str = "OPENAI_API_KEY";
const BACKUP_DIR_NAME: &str = "launchpadx";

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
    #[error("restore state is missing; the revert could not restore your previous Codex model")]
    RestoreStateMissing,
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
            provider_id: config.lpad_provider_id(),
            provider_name: config.lpad_provider_name(),
            base_url,
            api_key,
            api_key_mode: config.lpad_api_key_mode(),
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
pub fn inspect(config: &LauncherConfig) -> Result<CodexConfigInspection, CodexConfigError> {
    let config_path = config
        .codex_config_path()
        .map(Ok)
        .unwrap_or_else(default_codex_config_path)?;
    let provider_id = config.lpad_provider_id();
    let exists = config_path.exists();
    let restore_state_available = restore_state_path(&config_path).exists();

    if !exists {
        return Ok(CodexConfigInspection {
            config_path,
            exists: false,
            model: None,
            model_provider: None,
            managed_by_launcher: false,
            launcher_base_url: None,
            restore_state_available,
        });
    }

    let document = read_document(&config_path)?;
    let model = root_string(&document, "model");
    let model_provider = root_string(&document, "model_provider");
    let managed_by_launcher = model_provider
        .as_deref()
        .map(|id| is_launcher_managed_provider(&document, id, provider_id.as_str()))
        .unwrap_or(false);
    let launcher_base_url = if managed_by_launcher {
        model_provider
            .as_deref()
            .and_then(|id| provider_string(&document, id, "base_url"))
    } else {
        None
    };

    Ok(CodexConfigInspection {
        config_path,
        exists: true,
        model,
        model_provider,
        managed_by_launcher,
        launcher_base_url,
        restore_state_available,
    })
}

pub fn restore(config: &LauncherConfig) -> Result<(PathBuf, Option<String>), CodexConfigError> {
    let config_path = config
        .codex_config_path()
        .map(Ok)
        .unwrap_or_else(default_codex_config_path)?;
    let provider_id = config.lpad_provider_id();
    let mut document = read_document(&config_path)?;
    let restore_state = read_restore_state(&config_path)?;

    let launcher_provider_ids = collect_launcher_provider_ids(&document, provider_id.as_str());
    let on_launcher = root_string(&document, "model_provider")
        .as_deref()
        .is_some_and(|id| is_launcher_managed_provider(&document, id, provider_id.as_str()));

    for launcher_id in &launcher_provider_ids {
        remove_provider(&mut document, launcher_id);
    }
    if !launcher_provider_ids.iter().any(|id| id == &provider_id) {
        remove_provider(&mut document, &provider_id);
    }

    if on_launcher {
        revert_launcher_root_settings(&mut document, restore_state.as_ref());
    }

    if root_string(&document, "model_provider")
        .as_deref()
        .is_some_and(|id| is_launcher_managed_provider(&document, id, provider_id.as_str()))
    {
        revert_launcher_root_settings(&mut document, None);
    }

    write_document(&config_path, &document)?;
    clear_restore_state(&config_path);

    Ok((config_path, None))
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
    if let Some(current) = root_string(document, "model_provider") {
        if is_launcher_managed_provider(document, &current, settings.provider_id.as_str()) {
            return Ok(());
        }
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

fn revert_launcher_root_settings(document: &mut DocumentMut, restore_state: Option<&RestoreState>) {
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
        return;
    }

    document.as_table_mut().remove("model");
    document.as_table_mut().remove("model_catalog_json");

    if let Some(account_provider) = infer_account_provider_id(document) {
        document["model_provider"] = value(&account_provider);
    } else {
        document.as_table_mut().remove("model_provider");
    }
}

fn collect_launcher_provider_ids(
    document: &DocumentMut,
    configured_provider_id: &str,
) -> Vec<String> {
    let mut ids = Vec::new();
    if let Some(providers) = document.get("model_providers").and_then(Item::as_table) {
        for (key, _) in providers.iter() {
            let id = key.to_string();
            if is_launcher_managed_provider(document, &id, configured_provider_id) {
                ids.push(id);
            }
        }
    }
    if let Some(active) = root_string(document, "model_provider") {
        if is_launcher_managed_provider(document, &active, configured_provider_id)
            && !ids.iter().any(|id| id == &active)
        {
            ids.push(active);
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

fn infer_account_provider_id(document: &DocumentMut) -> Option<String> {
    let providers = document.get("model_providers").and_then(Item::as_table)?;
    let mut candidates: Vec<(i32, String)> = providers
        .iter()
        .filter_map(|(key, _)| {
            let id = key.to_string();
            if is_launcher_injected_provider(document, &id) {
                return None;
            }
            Some((score_account_provider(document, &id), id))
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    candidates.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    candidates.first().map(|(_, id)| id.clone())
}

fn score_account_provider(document: &DocumentMut, provider_id: &str) -> i32 {
    let mut score = 0;
    if provider_id == "openai" {
        score += 100;
    }

    let Some(provider) = document
        .get("model_providers")
        .and_then(Item::as_table)
        .and_then(|providers| providers.get(provider_id))
        .and_then(Item::as_table)
    else {
        return score;
    };

    if provider
        .get("requires_openai_auth")
        .and_then(Item::as_bool)
        .is_some_and(|required| required)
    {
        score += 50;
    }

    if let Some(base_url) = provider.get("base_url").and_then(Item::as_str) {
        if base_url.contains("api.openai.com") {
            score += 40;
        }
        if base_url.starts_with("https://") {
            score += 10;
        }
        if is_local_base_url(base_url) {
            score -= 100;
        }
    }

    score
}

fn is_local_base_url(base_url: &str) -> bool {
    let lower = base_url.to_ascii_lowercase();
    lower.contains("127.0.0.1")
        || lower.contains("localhost")
        || lower.contains("0.0.0.0")
        || lower.contains("[::1]")
}

fn is_launcher_injected_provider(document: &DocumentMut, provider_id: &str) -> bool {
    is_known_launcher_provider_id(provider_id)
        || provider_has_launcher_fingerprint(document, provider_id)
}

fn clear_restore_state(config_path: &Path) {
    let path = restore_state_path(config_path);
    let _ = fs::remove_file(path);
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

fn is_known_launcher_provider_id(id: &str) -> bool {
    matches!(
        id,
        "launchpadx" | "codex-local-launcher" | "codex_launchpad" | "codex-launchpad"
    )
}

fn provider_has_launcher_fingerprint(document: &DocumentMut, provider_id: &str) -> bool {
    let Some(provider) = document
        .get("model_providers")
        .and_then(Item::as_table)
        .and_then(|providers| providers.get(provider_id))
        .and_then(Item::as_table)
    else {
        return false;
    };

    let wire_api = provider.get("wire_api").and_then(Item::as_str);
    let requires_auth = provider.get("requires_openai_auth").and_then(Item::as_bool);
    let has_bearer = provider.contains_key("experimental_bearer_token");
    let has_env_key = provider
        .get("env_key")
        .and_then(Item::as_str)
        .is_some_and(|key| key == ENV_KEY_NAME);

    wire_api == Some("responses") && requires_auth == Some(false) && (has_bearer || has_env_key)
}

fn is_launcher_managed_provider(
    document: &DocumentMut,
    provider_id: &str,
    configured_provider_id: &str,
) -> bool {
    if provider_id == configured_provider_id {
        return true;
    }
    if is_known_launcher_provider_id(provider_id) {
        return true;
    }
    provider_has_launcher_fingerprint(document, provider_id)
}

fn provider_string(document: &DocumentMut, provider_id: &str, key: &str) -> Option<String> {
    document
        .get("model_providers")
        .and_then(Item::as_table)
        .and_then(|providers| providers.get(provider_id))
        .and_then(Item::as_table)
        .and_then(|provider| provider.get(key))
        .and_then(Item::as_str)
        .map(str::to_string)
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
            provider_id: "launchpadx".to_string(),
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
        assert!(text.contains(r#"model_provider = "launchpadx""#));
        assert!(text.contains("[model_providers.launchpadx]"));
        assert!(text.contains(r#"base_url = "http://127.0.0.1:11434/v1""#));
        assert!(text.contains(r#"experimental_bearer_token = "test-key""#));
    }

    #[test]
    fn env_key_mode_removes_persisted_bearer_token() {
        let mut document = r#"
[model_providers.launchpadx]
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
model_provider = "codex-launchpad"

[model_providers.codex-launchpad]
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

        revert_launcher_root_settings(&mut document, Some(&state));
        remove_provider(&mut document, "codex-launchpad");
        let text = document.to_string();

        assert!(text.contains(r#"model = "gpt-5.5""#));
        assert!(!text.contains("model_provider"));
        assert!(!text.contains("codex-launchpad"));
    }
    #[test]
    fn apply_and_restore_round_trip() {
        let dir = std::env::temp_dir().join(format!("launchpadx-roundtrip-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("config.toml");
        std::fs::write(
            &path,
            r#"
model = "gpt-test"
model_provider = "openai"
"#,
        )
        .expect("seed config");

        let launcher = LauncherConfig {
            codex_config_path: Some(path.to_string_lossy().to_string()),
            lpad_provider_id: Some("launchpadx".to_string()),
            lpad_provider_name: Some("LaunchPadX".to_string()),
            lpad_model: Some("llama3.2".to_string()),
            api_key: Some("test-key".to_string()),
            ..LauncherConfig::default()
        };

        let settings = PersistentCodexConfig {
            config_path: path.clone(),
            model: Some("llama3.2".to_string()),
            provider_id: "launchpadx".to_string(),
            provider_name: "LaunchPadX".to_string(),
            base_url: "http://127.0.0.1:11434/v1".to_string(),
            api_key: "test-key".to_string(),
            api_key_mode: ApiKeyMode::ExperimentalBearerToken,
        };

        apply(&settings).expect("apply");
        let managed = inspect(&launcher).expect("inspect after apply");
        assert!(managed.managed_by_launcher);
        assert_eq!(managed.model.as_deref(), Some("llama3.2"));

        let (restored_path, warning) = restore(&launcher).expect("restore");
        assert!(warning.is_none());
        assert_eq!(restored_path, path);

        let document = read_document(&path).expect("read restored");
        assert_eq!(root_string(&document, "model").as_deref(), Some("gpt-test"));
        assert_eq!(
            root_string(&document, "model_provider").as_deref(),
            Some("openai")
        );
        assert!(!document.to_string().contains("codex-launchpad"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn inspect_detects_managed_provider() {
        let path = std::env::temp_dir().join("launchpadx-inspect-test.toml");
        let text = r#"
model = "llama3.2"
model_provider = "launchpadx"

[model_providers.launchpadx]
base_url = "http://127.0.0.1:11434/v1"
"#;
        std::fs::write(&path, text).expect("write temp config");

        let launcher = LauncherConfig {
            codex_config_path: Some(path.to_string_lossy().to_string()),
            lpad_provider_id: Some("launchpadx".to_string()),
            ..LauncherConfig::default()
        };

        let inspection = inspect(&launcher).expect("inspect");
        assert!(inspection.exists);
        assert!(inspection.managed_by_launcher);
        assert_eq!(inspection.model.as_deref(), Some("llama3.2"));
        assert_eq!(
            inspection.launcher_base_url.as_deref(),
            Some("http://127.0.0.1:11434/v1")
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn restore_handles_legacy_provider_id_mismatch() {
        let dir =
            std::env::temp_dir().join(format!("launchpadx-legacy-restore-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("config.toml");
        std::fs::write(
            &path,
            r#"
model = "qwen3.635b-a3b-coding-mxfp8"
model_provider = "codex-local-launcher"

[model_providers.codex-local-launcher]
model_provider = "codex-local-launcher"
base_url = "http://127.0.0.1:11434/v1"
wire_api = "responses"
requires_openai_auth = false
experimental_bearer_token = "test-key"
"#,
        )
        .expect("seed config");

        let backup_dir = dir.join("backups").join(BACKUP_DIR_NAME);
        std::fs::create_dir_all(&backup_dir).expect("create backup dir");
        let restore_state = RestoreState {
            had_profile: false,
            profile: None,
            had_model: true,
            model: Some("gpt-5.5".to_string()),
            had_model_provider: true,
            model_provider: Some("openai".to_string()),
            had_model_catalog_json: false,
            model_catalog_json: None,
        };
        std::fs::write(
            backup_dir.join("restore-state.json"),
            serde_json::to_vec_pretty(&restore_state).expect("serialize restore state"),
        )
        .expect("write restore state");

        let launcher = LauncherConfig {
            codex_config_path: Some(path.to_string_lossy().to_string()),
            lpad_provider_id: Some("launchpadx".to_string()),
            ..LauncherConfig::default()
        };

        let inspection = inspect(&launcher).expect("inspect legacy config");
        assert!(inspection.managed_by_launcher);
        assert_eq!(
            inspection.model_provider.as_deref(),
            Some("codex-local-launcher")
        );

        let (restored_path, warning) = restore(&launcher).expect("restore legacy config");
        assert!(warning.is_none());
        assert_eq!(restored_path, path);

        let document = read_document(&path).expect("read restored");
        assert_eq!(root_string(&document, "model").as_deref(), Some("gpt-5.5"));
        assert_eq!(
            root_string(&document, "model_provider").as_deref(),
            Some("openai")
        );
        assert!(!document.to_string().contains("codex-local-launcher"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn restore_does_not_change_root_values_when_not_active_provider() {
        let dir = std::env::temp_dir().join(format!(
            "launchpadx-inactive-restore-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("config.toml");
        std::fs::write(
            &path,
            r#"
model = "gpt-5.5"
model_provider = "openai"

[model_providers.openai]
name = "OpenAI"
base_url = "https://api.openai.com/v1"

[model_providers.launchpadx]
name = "Local Ollama"
base_url = "http://127.0.0.1:11434/v1"
wire_api = "responses"
requires_openai_auth = false
experimental_bearer_token = "test-key"
"#,
        )
        .expect("seed config");

        let launcher = LauncherConfig {
            codex_config_path: Some(path.to_string_lossy().to_string()),
            lpad_provider_id: Some("launchpadx".to_string()),
            ..LauncherConfig::default()
        };

        restore(&launcher).expect("restore inactive launcher block");

        let document = read_document(&path).expect("read restored");
        assert_eq!(root_string(&document, "model").as_deref(), Some("gpt-5.5"));
        assert_eq!(
            root_string(&document, "model_provider").as_deref(),
            Some("openai")
        );
        assert!(!document.to_string().contains("codex-launchpad"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn restore_without_snapshot_infers_account_provider_from_config() {
        let dir =
            std::env::temp_dir().join(format!("launchpadx-infer-restore-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("config.toml");
        std::fs::write(
            &path,
            r#"
model = "qwen3.635b-a3b-coding-mxfp8"
model_provider = "launchpadx"

[model_providers.openai]
name = "OpenAI"
base_url = "https://api.openai.com/v1"
requires_openai_auth = true

[model_providers.codex-local-launcher]
name = "codex-local-launcher"
base_url = "http://127.0.0.1:11434/v1"
wire_api = "responses"
requires_openai_auth = false
experimental_bearer_token = "test-key"
"#,
        )
        .expect("seed config");

        let launcher = LauncherConfig {
            codex_config_path: Some(path.to_string_lossy().to_string()),
            lpad_provider_id: Some("launchpadx".to_string()),
            ..LauncherConfig::default()
        };

        let (restored_path, warning) = restore(&launcher).expect("restore without snapshot");
        assert!(warning.is_none());
        assert_eq!(restored_path, path);

        let document = read_document(&path).expect("read restored");
        assert!(root_string(&document, "model").is_none());
        assert_eq!(
            root_string(&document, "model_provider").as_deref(),
            Some("openai")
        );
        assert!(!document.to_string().contains("codex-local-launcher"));
        assert!(document.to_string().contains("[model_providers.openai]"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn restore_fixes_orphan_launcher_model_provider_without_provider_block() {
        let dir =
            std::env::temp_dir().join(format!("launchpadx-orphan-root-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("config.toml");
        std::fs::write(
            &path,
            r#"
model = "qwen3.635b-a3b-coding-mxfp8"
model_provider = "launchpadx"

[model_providers.openai]
name = "OpenAI"
base_url = "https://api.openai.com/v1"
requires_openai_auth = true
"#,
        )
        .expect("seed config");

        let launcher = LauncherConfig {
            codex_config_path: Some(path.to_string_lossy().to_string()),
            lpad_provider_id: Some("launchpadx".to_string()),
            ..LauncherConfig::default()
        };

        let inspection = inspect(&launcher).expect("inspect orphan root");
        assert!(inspection.managed_by_launcher);

        restore(&launcher).expect("restore orphan root");

        let document = read_document(&path).expect("read restored");
        assert!(root_string(&document, "model").is_none());
        assert_eq!(
            root_string(&document, "model_provider").as_deref(),
            Some("openai")
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn restore_without_snapshot_falls_back_to_account_sign_in() {
        let dir =
            std::env::temp_dir().join(format!("launchpadx-signin-restore-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("config.toml");
        std::fs::write(
            &path,
            r#"
model = "llama3.2"
model_provider = "launchpadx"

[model_providers.launchpadx]
name = "Local Ollama"
base_url = "http://127.0.0.1:11434/v1"
wire_api = "responses"
requires_openai_auth = false
experimental_bearer_token = "test-key"
"#,
        )
        .expect("seed config");

        let launcher = LauncherConfig {
            codex_config_path: Some(path.to_string_lossy().to_string()),
            lpad_provider_id: Some("launchpadx".to_string()),
            ..LauncherConfig::default()
        };

        restore(&launcher).expect("restore to account sign-in");

        let document = read_document(&path).expect("read restored");
        assert!(root_string(&document, "model").is_none());
        assert!(root_string(&document, "model_provider").is_none());
        assert!(!document.to_string().contains("codex-launchpad"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
