//! CLI definitions

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "solana-credential-helper",
    about = "Solana transaction signing helper",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Sign a transaction from stdin
    SignTransaction(SignTransactionArgs),
}

#[derive(clap::Args)]
pub struct SignTransactionArgs {
    /// Encoding for input/output (base64 or base58)
    #[arg(long, default_value = "base64")]
    pub encoding: Encoding,

    /// Signer public key or label
    #[arg(long)]
    pub signer: String,

    /// Sign with Ledger hardware wallet
    #[arg(long, conflicts_with = "squads")]
    pub ledger: bool,

    /// Sign via Squads multisig (creates/approves proposal)
    #[arg(long, conflicts_with = "ledger")]
    pub squads: Option<String>,

    /// RPC URL for Squads operations
    #[arg(long, default_value = "https://api.mainnet-beta.solana.com")]
    pub rpc_url: String,

    /// Connect to keyring agent socket instead of prompting
    #[arg(long)]
    pub use_agent: bool,

    /// Agent socket path
    #[arg(long)]
    pub agent_socket: Option<PathBuf>,

    /// Database path
    #[arg(long)]
    pub db_path: Option<PathBuf>,
}

#[derive(Clone, ValueEnum)]
pub enum Encoding {
    Base64,
    Base58,
}
