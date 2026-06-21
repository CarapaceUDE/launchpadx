use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::LauncherConfig;

#[derive(Debug, thiserror::Error)]
pub enum AcpError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("decode failed: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("invalid response from server")]
    InvalidResponse,
    #[error("codex is not running")]
    CodexNotRunning,
    #[error("timeout waiting for response")]
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRequest {
    pub session_id: String,
    pub content: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub session_id: String,
    pub content: String,
    pub role: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionList {
    pub sessions: Vec<SessionInfo>,
}

#[derive(Debug)]
pub struct AcpClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl AcpClient {
    pub fn from_config(config: &LauncherConfig) -> Result<Self, AcpError> {
        Ok(Self {
            client: Client::new(),
            base_url: config.codex_api_base_url(),
            api_key: config.api_key().unwrap_or_default(),
        })
    }

    pub fn with_base_url(base_url: &str, api_key: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn create_session(&self) -> Result<SessionInfo, AcpError> {
        let response = self
            .client
            .post(format!("{}/sessions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .body("{}")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AcpError::SessionNotFound(format!(
                "Failed to create session: {}",
                response.status()
            )));
        }

        let body = response.text().await?;
        let session: SessionInfo = serde_json::from_str(&body)?;
        Ok(session)
    }

    pub async fn send_message(
        &self,
        session_id: &str,
        content: &str,
    ) -> Result<MessageResponse, AcpError> {
        let payload = serde_json::json!({
            "session_id": session_id,
            "content": content,
            "role": "user",
        });

        let response = self
            .client
            .post(format!(
                "{}/sessions/{}/messages",
                self.base_url, session_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AcpError::SessionNotFound(format!(
                "Failed to send message: {}",
                response.status()
            )));
        }

        let body = response.text().await?;
        let msg: MessageResponse = serde_json::from_str(&body)?;
        Ok(msg)
    }

    pub async fn get_response(&self, session_id: &str) -> Result<MessageResponse, AcpError> {
        let response = self
            .client
            .get(format!(
                "{}/sessions/{}/response",
                self.base_url, session_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AcpError::SessionNotFound(format!(
                "Failed to get response: {}",
                response.status()
            )));
        }

        let body = response.text().await?;
        let msg: MessageResponse = serde_json::from_str(&body)?;
        Ok(msg)
    }

    pub async fn close_session(&self, session_id: &str) -> Result<(), AcpError> {
        let response = self
            .client
            .delete(format!("{}/sessions/{}", self.base_url, session_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() && response.status() != reqwest::StatusCode::NOT_FOUND {
            return Err(AcpError::SessionNotFound(format!(
                "Failed to close session: {}",
                response.status()
            )));
        }

        Ok(())
    }

    pub async fn list_sessions(&self) -> Result<SessionList, AcpError> {
        let response = self
            .client
            .get(format!("{}/sessions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AcpError::SessionNotFound(format!(
                "Failed to list sessions: {}",
                response.status()
            )));
        }

        let body = response.text().await?;
        let list: SessionList = serde_json::from_str(&body)?;
        Ok(list)
    }

    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_client_from_config() {
        let config = LauncherConfig::default();
        let client = AcpClient::from_config(&config);
        assert!(client.is_ok());
    }
}
