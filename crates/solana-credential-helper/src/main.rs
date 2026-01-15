//! Solana Credential Helper CLI

mod cli;
mod commands;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::SignTransaction(args) => commands::sign_transaction::run(args).await,
    }
}
