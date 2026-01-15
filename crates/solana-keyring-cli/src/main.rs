//! Solana Keyring CLI

mod cli;
mod commands;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New(args) => commands::new::run(args, &cli.db_path),
        Commands::Generate(args) => commands::generate::run(args, &cli.db_path),
        Commands::Import(args) => commands::import::run(args, &cli.db_path),
        Commands::Export(args) => commands::export::run(args, &cli.db_path),
        Commands::List(args) => commands::list::run(args, &cli.db_path),
        Commands::Label(args) => commands::label::run(args, &cli.db_path),
        Commands::Delete(args) => commands::delete::run(args, &cli.db_path),
        Commands::Tag(cmd) => commands::tag::run(cmd, &cli.db_path),
        Commands::Ledger(cmd) => commands::ledger::run(cmd, &cli.db_path),
        Commands::Squads(cmd) => commands::squads::run(cmd, &cli.db_path),
        Commands::AddressBook(cmd) => commands::address_book::run(cmd, &cli.db_path),
    }
}
