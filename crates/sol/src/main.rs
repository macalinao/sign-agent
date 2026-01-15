//! sol - Simple SOL transfer CLI
//!
//! A minimal CLI tool demonstrating solana-keyring for signing transactions.

mod cli;
mod commands;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Transfer(args) => commands::transfer::run(args).await,
        Commands::Balance(args) => commands::balance::run(args).await,
    }
}
