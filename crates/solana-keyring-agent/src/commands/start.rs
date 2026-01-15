//! Start the agent daemon

use std::time::Duration;

use anyhow::Result;
use solana_keyring::default_agent_socket_path;

use crate::agent::Agent;
use crate::cli::StartArgs;

pub async fn run(args: StartArgs) -> Result<()> {
    let socket_path = default_agent_socket_path();
    let lock_timeout = Duration::from_secs(args.lock_timeout);

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
        // TODO: Implement proper daemonization
        println!("Note: Background mode not yet implemented. Running in foreground.");
    }

    let agent = Agent::new(socket_path, args.db_path, lock_timeout);
    agent.run().await
}
