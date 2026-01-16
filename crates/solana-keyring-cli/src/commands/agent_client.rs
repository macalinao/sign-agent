//! Agent client for CLI commands

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Default agent socket path
pub fn default_socket_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".solana-keyring")
        .join("agent.sock")
}

/// Request message to agent
#[derive(Debug, Serialize)]
#[serde(tag = "method", content = "params")]
pub enum Request {
    #[serde(rename = "GenerateKeypair")]
    GenerateKeypair { label: String, tags: Vec<String> },

    #[serde(rename = "ImportKeypair")]
    ImportKeypair {
        label: String,
        secret_key: String,
        tags: Vec<String>,
    },

    #[serde(rename = "Status")]
    Status,
}

/// Response from agent
#[derive(Debug, Deserialize)]
#[serde(tag = "status")]
pub enum Response {
    #[serde(rename = "ok")]
    Ok { result: ResponseResult },

    #[serde(rename = "error")]
    Error { code: String, message: String },
}

/// Response result variants
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponseResult {
    GeneratedKeypair(GeneratedKeypairInfo),
    Status(AgentStatus),
    Unit,
}

#[derive(Debug, Deserialize)]
pub struct GeneratedKeypairInfo {
    pub pubkey: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct AgentStatus {
    pub unlocked: bool,
}

/// Send a request to the agent and get the response
pub async fn send_request(socket_path: &PathBuf, request: &Request) -> Result<Response> {
    let mut stream = UnixStream::connect(socket_path)
        .await
        .context("Failed to connect to agent socket")?;

    // Serialize request
    let request_bytes = serde_json::to_vec(request)?;

    // Send length prefix + message
    stream
        .write_all(&(request_bytes.len() as u32).to_be_bytes())
        .await?;
    stream.write_all(&request_bytes).await?;

    // Read response length
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    // Read response
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;

    // Parse response
    let response: Response = serde_json::from_slice(&buf)?;
    Ok(response)
}

/// Agent availability status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentAvailability {
    /// Agent is running and unlocked
    Available,
    /// Agent is running but locked
    Locked,
    /// Agent is not running
    NotRunning,
}

/// Check the agent's availability status
pub async fn check_agent_availability(socket_path: &PathBuf) -> AgentAvailability {
    match send_request(socket_path, &Request::Status).await {
        Ok(Response::Ok {
            result: ResponseResult::Status(status),
        }) => {
            if status.unlocked {
                AgentAvailability::Available
            } else {
                AgentAvailability::Locked
            }
        }
        _ => AgentAvailability::NotRunning,
    }
}

/// Generate a keypair via the agent
pub async fn generate_keypair(
    socket_path: &PathBuf,
    label: &str,
    tags: &[String],
) -> Result<GeneratedKeypairInfo> {
    let request = Request::GenerateKeypair {
        label: label.to_string(),
        tags: tags.to_vec(),
    };

    match send_request(socket_path, &request).await? {
        Response::Ok {
            result: ResponseResult::GeneratedKeypair(info),
        } => Ok(info),
        Response::Ok { result: _ } => anyhow::bail!("Unexpected response from agent"),
        Response::Error { code, message } => {
            anyhow::bail!("Agent error ({}): {}", code, message)
        }
    }
}

/// Import a keypair via the agent
pub async fn import_keypair(
    socket_path: &PathBuf,
    label: &str,
    secret_key: &str,
    tags: &[String],
) -> Result<GeneratedKeypairInfo> {
    let request = Request::ImportKeypair {
        label: label.to_string(),
        secret_key: secret_key.to_string(),
        tags: tags.to_vec(),
    };

    match send_request(socket_path, &request).await? {
        Response::Ok {
            result: ResponseResult::GeneratedKeypair(info),
        } => Ok(info),
        Response::Ok { result: _ } => anyhow::bail!("Unexpected response from agent"),
        Response::Error { code, message } => {
            anyhow::bail!("Agent error ({}): {}", code, message)
        }
    }
}
