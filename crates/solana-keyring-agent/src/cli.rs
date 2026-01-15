//! CLI definitions for agent

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "solana-keyring-agent",
    about = "Keyring agent daemon for Solana",
    version
)]
pub struct Cli {
    /// Socket path (default: ~/.solana-keyring/agent.sock)
    #[arg(long, global = true)]
    pub socket: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the agent daemon
    Start(StartArgs),
    /// Stop a running agent
    Stop,
    /// Unlock the agent with master passphrase
    Unlock,
    /// Lock the agent (clear passphrase from memory)
    Lock,
    /// Check agent status
    Status,
}

#[derive(clap::Args)]
pub struct StartArgs {
    /// Run in foreground (don't daemonize)
    #[arg(long)]
    pub foreground: bool,

    /// Lock timeout in seconds (default: 3600 = 1 hour)
    #[arg(long, default_value = "3600")]
    pub lock_timeout: u64,

    /// Database path
    #[arg(long)]
    pub db_path: Option<PathBuf>,
}
