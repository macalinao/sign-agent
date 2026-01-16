//! Start the agent daemon

use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::Result;
use solana_keyring::default_agent_socket_path;

use crate::agent::Agent;
use crate::cli::StartArgs;

pub async fn run(args: StartArgs) -> Result<()> {
    let socket_path = default_agent_socket_path();

    // Check if agent is already running
    if socket_path.exists() {
        // Try to connect
        if tokio::net::UnixStream::connect(&socket_path).await.is_ok() {
            anyhow::bail!("Agent is already running at {}", socket_path.display());
        }
        // Remove stale socket
        std::fs::remove_file(&socket_path)?;
    }

    if !args.foreground {
        // Spawn ourselves in the background with --foreground flag
        let exe = std::env::current_exe()?;
        let mut cmd = Command::new(exe);
        cmd.arg("start").arg("--foreground");
        cmd.arg("--lock-timeout").arg(args.lock_timeout.to_string());

        if let Some(ref db_path) = args.db_path {
            cmd.arg("--db-path").arg(db_path);
        }

        // Detach from terminal
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        let child = cmd.spawn()?;
        println!("Agent started in background (PID: {})", child.id());
        println!("Socket: {}", socket_path.display());
        println!();
        println!("Run 'solana-keyring-agent unlock' to unlock the agent.");
        return Ok(());
    }

    let lock_timeout = Duration::from_secs(args.lock_timeout);
    let agent = Agent::new(socket_path, args.db_path, lock_timeout);
    agent.run().await
}
