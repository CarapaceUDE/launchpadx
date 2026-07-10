use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::lpad_app_server::CodexThreadSummary;

#[derive(Debug, Deserialize)]
struct SessionIndexRow {
    id: String,
    #[serde(rename = "thread_name")]
    thread_name: Option<String>,
    updated_at: Option<String>,
}

pub fn codex_home() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".codex"))
}

pub fn list_threads_from_store(limit: usize) -> Result<Vec<CodexThreadSummary>, String> {
    let home = codex_home().ok_or_else(|| "could not resolve Codex home directory".to_string())?;
    let index_path = home.join("session_index.jsonl");
    if !index_path.is_file() {
        return Err(format!("no session index at {}", index_path.display()));
    }

    let mut rows = read_session_index(&index_path)?;
    rows.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    rows.truncate(limit.clamp(1, 200));

    let rollout_paths = build_rollout_path_index(&home.join("sessions"));

    Ok(rows
        .into_iter()
        .map(|row| {
            let path = rollout_paths
                .get(&row.id)
                .map(|path| path.display().to_string());
            CodexThreadSummary {
                id: row.id,
                name: row.thread_name,
                status: Some("stored".to_string()),
                path,
                created_at: row.updated_at,
                model: None,
            }
        })
        .collect())
}

fn read_session_index(path: &Path) -> Result<Vec<SessionIndexRow>, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();

    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(row) = serde_json::from_str::<SessionIndexRow>(trimmed) {
            if !row.id.is_empty() {
                rows.push(row);
            }
        }
    }

    Ok(rows)
}

fn build_rollout_path_index(sessions_root: &Path) -> HashMap<String, PathBuf> {
    let mut index = HashMap::new();
    if !sessions_root.is_dir() {
        return index;
    }

    let mut stack = vec![sessions_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !file_name.starts_with("rollout-") || !file_name.ends_with(".jsonl") {
                continue;
            }

            let Some(id) = extract_rollout_id(file_name) else {
                continue;
            };
            index.entry(id).or_insert(path);
        }
    }

    index
}

fn extract_rollout_id(file_name: &str) -> Option<String> {
    let stem = file_name.strip_suffix(".jsonl")?;
    if stem.len() < 36 {
        return None;
    }
    let id = &stem[stem.len() - 36..];
    uuid_like(id).then(|| id.to_string())
}

fn uuid_like(value: &str) -> bool {
    value.len() == 36
        && value.as_bytes().get(8) == Some(&b'-')
        && value.as_bytes().get(13) == Some(&b'-')
        && value.as_bytes().get(18) == Some(&b'-')
        && value.as_bytes().get(23) == Some(&b'-')
}

pub fn list_session_ids_from_store(limit: usize) -> Result<Vec<(String, Option<String>)>, String> {
    let home = codex_home().ok_or_else(|| "could not resolve Codex home directory".to_string())?;
    let index_path = home.join("session_index.jsonl");
    let mut rows = read_session_index(&index_path)?;
    rows.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    rows.truncate(limit.clamp(1, 200));
    Ok(rows
        .into_iter()
        .map(|row| (row.id, row.updated_at))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_rollout_id_from_filename() {
        let id = extract_rollout_id(
            "rollout-2026-05-27T13-52-37-019e6ac7-fd7b-7f13-8b5f-830851de21db.jsonl",
        )
        .expect("id");
        assert_eq!(id, "019e6ac7-fd7b-7f13-8b5f-830851de21db");
    }

    #[test]
    fn parses_session_index_rows() {
        let dir = std::env::temp_dir().join(format!("codex-thread-store-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dir");
        let index = dir.join("session_index.jsonl");
        std::fs::write(
            &index,
            r#"{"id":"thr-1","thread_name":"Alpha","updated_at":"2026-06-02T10:00:00Z"}
{"id":"thr-2","thread_name":"Beta","updated_at":"2026-06-01T10:00:00Z"}"#,
        )
        .expect("write");

        let rows = read_session_index(&index).expect("rows");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].thread_name.as_deref(), Some("Alpha"));
    }
}
