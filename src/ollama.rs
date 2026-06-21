use std::fs;
use std::path::PathBuf;

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::LauncherConfig;

const CACHE_DIR_NAME: &str = "codex-local-launcher";
const MODEL_CACHE_FILE_NAME: &str = "ollama-models.json";

#[derive(Debug, Error)]
pub enum OllamaError {
    #[error("could not derive Ollama API root from base URL: {0}")]
    InvalidBaseUrl(String),
    #[error("could not build HTTP client: {0}")]
    ClientBuild(#[source] reqwest::Error),
    #[error("could not fetch model list from {url}: {source}")]
    Fetch { url: String, source: reqwest::Error },
    #[error("could not parse model list from {url}: {source}")]
    Parse { url: String, source: reqwest::Error },
    #[error("could not locate a cache directory for model discovery")]
    MissingCacheDir,
    #[error("could not create model cache directory {path}: {source}")]
    CreateCacheDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not write model cache {path}: {source}")]
    WriteCache {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not read model cache {path}: {source}")]
    ReadCache {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not parse model cache {path}: {source}")]
    ParseCache {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("multiple Ollama models were discovered; set codexModel explicitly or use the model cache for UI selection")]
    AmbiguousModelSelection,
    #[error("no Ollama models were discovered from {0}")]
    NoModelsDiscovered(String),
     #[error("model cache expired")]
    CacheExpired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCache {
    pub api_root: String,
    pub fetched_from: String,
    pub fetched_at: Option<u64>,
    pub models: Vec<OllamaModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub model: String,
    pub modified_at: Option<String>,
    pub size: Option<u64>,
    pub digest: Option<String>,
    pub details: Option<OllamaModelDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelDetails {
    pub format: Option<String>,
    pub family: Option<String>,
    pub families: Option<Vec<String>>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<OllamaModel>,
}

pub fn resolve_model(
    config: &LauncherConfig,
    base_url: &str,
) -> Result<Option<String>, OllamaError> {
    if let Some(model) = config.codex_model() {
        return Ok(Some(model));
    }

    let cache = refresh_model_cache(base_url, config.api_key.as_deref())?;
    match cache.models.len() {
        0 => Err(OllamaError::NoModelsDiscovered(cache.fetched_from)),
        1 => Ok(cache.models.first().map(|model| model.name.clone())),
        _ => Err(OllamaError::AmbiguousModelSelection),
    }
}

pub fn refresh_model_cache(
    base_url: &str,
    api_key: Option<&str>,
) -> Result<ModelCache, OllamaError> {
    let api_root = derive_ollama_api_root(base_url)?;
    let url = format!("{api_root}/tags");

    let mut headers = HeaderMap::new();
    if let Some(api_key) = api_key.filter(|value| !value.trim().is_empty()) {
        let header = HeaderValue::from_str(&format!("Bearer {api_key}"))
            .map_err(|_| OllamaError::InvalidBaseUrl(base_url.to_string()))?;
        headers.insert(AUTHORIZATION, header);
    }

    let client = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(OllamaError::ClientBuild)?;
    let response = client
        .get(&url)
        .send()
        .map_err(|source| OllamaError::Fetch {
            url: url.clone(),
            source,
        })?;
    let response = response
        .error_for_status()
        .map_err(|source| OllamaError::Fetch {
            url: url.clone(),
            source,
        })?;
    let tags: TagsResponse = response.json().map_err(|source| OllamaError::Parse {
        url: url.clone(),
        source,
    })?;

    let now = std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .map(|d| d.as_secs())
          .unwrap_or(0);
    let cache = ModelCache {
        api_root,
        fetched_from: url,
        fetched_at: Some(now),
        models: tags.models,
    };
    write_model_cache(&cache)?;
    Ok(cache)
}

pub const fn cache_ttl_secs() -> u64 {
     300 // 5 minutes
}

pub fn read_model_cache() -> Result<ModelCache, OllamaError> {
    let path = model_cache_path()?;
    let data = fs::read_to_string(&path).map_err(|source| OllamaError::ReadCache {
        path: path.clone(),
        source,
    })?;
    let cache: ModelCache = serde_json::from_str(&data).map_err(|source| OllamaError::ParseCache { path, source })?;
     // Check cache TTL - expire after 5 minutes
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Some(fetched_at) = cache.fetched_at {
        if now.saturating_sub(fetched_at) > cache_ttl_secs() {
            return Err(OllamaError::CacheExpired);
        }
    }
    Ok(cache)
}

pub fn model_cache_path() -> Result<PathBuf, OllamaError> {
    let cache_dir = dirs::cache_dir().ok_or(OllamaError::MissingCacheDir)?;
    Ok(cache_dir.join(CACHE_DIR_NAME).join(MODEL_CACHE_FILE_NAME))
}

fn write_model_cache(cache: &ModelCache) -> Result<(), OllamaError> {
    let path = model_cache_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| OllamaError::CreateCacheDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let data = serde_json::to_vec_pretty(cache).map_err(|source| OllamaError::ParseCache {
        path: path.clone(),
        source,
    })?;
    fs::write(&path, data).map_err(|source| OllamaError::WriteCache { path, source })
}

fn derive_ollama_api_root(base_url: &str) -> Result<String, OllamaError> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(OllamaError::InvalidBaseUrl(base_url.to_string()));
    }

    if let Some(prefix) = trimmed.strip_suffix("/v1") {
        return Ok(format!("{prefix}/api"));
    }
    if trimmed.ends_with("/api") {
        return Ok(trimmed.to_string());
    }
    if trimmed.contains("/v1/") {
        let replaced = trimmed.replacen("/v1/", "/api/", 1);
        return Ok(replaced);
    }

    Err(OllamaError::InvalidBaseUrl(base_url.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_api_root_from_v1_base_url() {
        assert_eq!(
            derive_ollama_api_root("http://127.0.0.1:11434/v1").expect("api root"),
            "http://127.0.0.1:11434/api"
        );
        assert_eq!(
            derive_ollama_api_root("https://example.com/custom/v1/").expect("api root"),
            "https://example.com/custom/api"
        );
    }

    #[test]
    fn preserves_existing_api_root() {
        assert_eq!(
            derive_ollama_api_root("https://ollama.com/api").expect("api root"),
            "https://ollama.com/api"
        );
    }

    #[test]
    fn rejects_unrecognized_base_url() {
        assert!(derive_ollama_api_root("https://example.com").is_err());
    }
}
