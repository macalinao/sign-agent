//! CLI definitions for sol

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sol", about = "Simple SOL transfer CLI", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Transfer SOL from one account to another
    Transfer(TransferArgs),

    /// Check balance of an account
    Balance(BalanceArgs),
}

#[derive(clap::Args)]
pub struct TransferArgs {
    /// Source address (public key or label from keyring)
    #[arg(long)]
    pub from: String,

    /// Destination address (public key or label from keyring)
    #[arg(long)]
    pub to: String,

    /// Amount in SOL to transfer
    #[arg(long)]
    pub amount: f64,

    /// RPC URL
    #[arg(long, default_value = "https://api.mainnet-beta.solana.com")]
    pub rpc: String,

    /// Use keyring agent instead of prompting for passphrase
    #[arg(long)]
    pub use_agent: bool,

    /// Agent socket path
    #[arg(long)]
    pub agent_socket: Option<PathBuf>,

    /// Database path
    #[arg(long)]
    pub db_path: Option<PathBuf>,

    /// Skip confirmation prompt
    #[arg(long, short = 'y')]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct BalanceArgs {
    /// Address to check (public key or label from keyring)
    pub address: String,

    /// RPC URL
    #[arg(long, default_value = "https://api.mainnet-beta.solana.com")]
    pub rpc: String,

    /// Database path (for resolving labels)
    #[arg(long)]
    pub db_path: Option<PathBuf>,
}
