//! Unlock command

use std::path::PathBuf;

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use super::get_socket_path;
use crate::protocol::{Request, Response};

pub async fn run(socket_path: &Option<PathBuf>) -> Result<()> {
    let path = get_socket_path(socket_path);

    // Prompt for passphrase
    let passphrase = rpassword::prompt_password("Enter master passphrase: ")?;

    // Connect to agent
    let mut stream = UnixStream::connect(&path).await?;

    // Build unlock request
    let request = Request::Unlock { passphrase };
    let request_bytes = serde_json::to_vec(&request)?;

    // Send request
    stream
        .write_all(&(request_bytes.len() as u32).to_be_bytes())
        .await?;
    stream.write_all(&request_bytes).await?;

    // Read response
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;

    let response: Response = serde_json::from_slice(&buf)?;

    match response {
        Response::Ok { .. } => {
            println!("Agent unlocked successfully.");
        }
        Response::Error { code, message } => {
            anyhow::bail!("Failed to unlock agent: {} - {}", code, message);
        }
    }

    Ok(())
}
