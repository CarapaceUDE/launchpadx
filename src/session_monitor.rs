use serde::Serialize;

use crate::acp_client::AcpClient;
use crate::config::LauncherConfig;
use crate::lpad_thread_store;

const MAX_SESSION_PREVIEWS: usize = 12;
const PREVIEW_CHARS: usize = 480;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionPreviewContent {
    pub role: String,
    pub done: bool,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionDetail {
    pub session_id: String,
    pub created_at: Option<String>,
    pub preview: Option<SessionPreviewContent>,
    pub preview_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSessionListDetail {
    pub sessions: Vec<CodexSessionDetail>,
    pub error: Option<String>,
}

pub fn list_sessions_with_previews(config: &LauncherConfig) -> CodexSessionListDetail {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            return CodexSessionListDetail {
                sessions: Vec::new(),
                error: Some(error.to_string()),
            };
        }
    };

    let client = match AcpClient::from_config(config) {
        Ok(client) => client,
        Err(_) => return list_sessions_from_store(),
    };

    runtime.block_on(async {
        let list = match client.list_sessions().await {
            Ok(list) if !list.sessions.is_empty() => list,
            Ok(_) | Err(_) => {
                return list_sessions_from_store();
            }
        };

        let mut sessions = Vec::new();
        for session in list.sessions.iter().take(MAX_SESSION_PREVIEWS) {
            let preview_result = client.get_response(&session.session_id).await;
            let (preview, preview_error) = match preview_result {
                Ok(response) => (
                    Some(SessionPreviewContent {
                        role: response.role,
                        done: response.done,
                        content: truncate(&response.content, PREVIEW_CHARS),
                    }),
                    None,
                ),
                Err(error) => (None, Some(error.to_string())),
            };

            sessions.push(CodexSessionDetail {
                session_id: session.session_id.clone(),
                created_at: session.created_at.clone(),
                preview,
                preview_error,
            });
        }

        CodexSessionListDetail {
            sessions,
            error: None,
        }
    })
}

fn list_sessions_from_store() -> CodexSessionListDetail {
    match lpad_thread_store::list_session_ids_from_store(MAX_SESSION_PREVIEWS) {
        Ok(rows) => CodexSessionListDetail {
            sessions: rows
                .into_iter()
                .map(|(session_id, created_at)| CodexSessionDetail {
                    session_id,
                    created_at,
                    preview: None,
                    preview_error: None,
                })
                .collect(),
            error: None,
        },
        Err(error) => CodexSessionListDetail {
            sessions: Vec::new(),
            error: Some(error),
        },
    }
}

fn truncate(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        return value.to_string();
    }
    let mut end = max_len;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &value[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_multibyte_safe() {
        assert_eq!(truncate("hello", 10), "hello");
        assert!(truncate("abcdefghijklmnopqrstuvwxyz", 10).ends_with('…'));
    }
}
