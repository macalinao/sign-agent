//! Squads multisig commands

use std::path::PathBuf;

use anyhow::Result;

use super::open_db;
use crate::cli::SquadsCommands;

pub fn run(cmd: SquadsCommands, db_path: &Option<PathBuf>) -> Result<()> {
    let db = open_db(db_path)?;

    match cmd {
        SquadsCommands::Add(args) => {
            // TODO: Fetch multisig info from chain to get threshold
            let threshold = 1; // Placeholder

            // Convert tags to &str slice
            let tags: Vec<&str> = args.tag.iter().map(|s| s.as_str()).collect();

            db.store_squads_multisig(
                &args.multisig_address,
                &args.label,
                0, // vault_index
                threshold,
                &tags,
            )?;

            println!("Added Squads multisig:");
            println!("  Address: {}", args.multisig_address);
            println!("  Label: {}", args.label);
            if !args.tag.is_empty() {
                println!("  Tags: {}", args.tag.join(", "));
            }
            println!(
                "\nNote: Run 'solana-keyring squads sync {}' to fetch members from chain.",
                args.label
            );
        }

        SquadsCommands::List => {
            let multisigs = db.list_squads_multisigs(None)?;

            if multisigs.is_empty() {
                println!("No Squads multisigs found.");
                return Ok(());
            }

            println!("{:<44} {:<20} THRESHOLD", "ADDRESS", "LABEL");
            println!("{}", "-".repeat(70));

            for ms in multisigs {
                println!(
                    "{:<44} {:<20} {}/{}",
                    ms.multisig_pubkey, ms.label, ms.threshold, ms.threshold
                );
            }
        }

        SquadsCommands::Remove(args) => {
            let deleted = db.delete_squads_multisig(&args.identifier)?;

            if deleted {
                println!("Removed Squads multisig '{}'", args.identifier);
            } else {
                anyhow::bail!("Squads multisig not found: {}", args.identifier);
            }
        }

        SquadsCommands::Sync(args) => {
            // TODO: Implement fetching members from chain
            println!(
                "Syncing multisig '{}' from {}...",
                args.identifier, args.rpc_url
            );
            println!("Note: Sync not yet implemented.");
        }
    }

    Ok(())
}
