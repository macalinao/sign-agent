//! Check agent status

use std::path::PathBuf;

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use super::get_socket_path;
use crate::protocol::{Request, Response, ResponseResult};

pub async fn run(socket_path: &Option<PathBuf>) -> Result<()> {
    let path = get_socket_path(socket_path);

    let mut stream = match UnixStream::connect(&path).await {
        Ok(s) => s,
        Err(_) => {
            println!("Agent is not running.");
            return Ok(());
        }
    };

    // Send status request
    let request = Request::Status;
    let request_bytes = serde_json::to_vec(&request)?;

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
        Response::Ok {
            result: ResponseResult::Status(status),
        } => {
            println!("Agent Status:");
            println!("  Running: yes");
            println!("  Unlocked: {}", if status.unlocked { "yes" } else { "no" });
            println!("  Uptime: {} seconds", status.uptime_seconds);
            println!("  Lock timeout: {} seconds", status.lock_timeout_seconds);
        }
        Response::Ok { .. } => {
            println!("Unexpected response from agent");
        }
        Response::Error { message, .. } => {
            anyhow::bail!("Failed to get status: {}", message)
        }
    }

    Ok(())
}
