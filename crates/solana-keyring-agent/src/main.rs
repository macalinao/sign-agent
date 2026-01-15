//! Solana Keyring Agent daemon

mod agent;
mod cli;
mod commands;
mod protocol;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start(args) => commands::start::run(args).await,
        Commands::Stop => commands::stop::run(&cli.socket).await,
        Commands::Unlock => commands::unlock::run(&cli.socket).await,
        Commands::Lock => commands::lock::run(&cli.socket).await,
        Commands::Status => commands::status::run(&cli.socket).await,
    }
}
