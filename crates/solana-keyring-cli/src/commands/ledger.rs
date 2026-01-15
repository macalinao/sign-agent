//! Ledger wallet commands

use std::path::PathBuf;

use anyhow::Result;
use solana_keyring::ledger::LedgerSigner;

use super::open_db;
use crate::cli::LedgerCommands;

pub fn run(cmd: LedgerCommands, db_path: &Option<PathBuf>) -> Result<()> {
    let db = open_db(db_path)?;

    match cmd {
        LedgerCommands::Add(args) => {
            println!("Connecting to Ledger device...");

            // Connect and get public key
            let signer = LedgerSigner::connect(&args.derivation_path)?;
            let pubkey = signer.pubkey();

            // Convert tags to &str slice
            let tags: Vec<&str> = args.tag.iter().map(|s| s.as_str()).collect();

            // Store in database
            db.store_ledger_wallet(pubkey, &args.label, &args.derivation_path, &tags)?;

            println!("Added Ledger wallet:");
            println!("  Public key: {}", pubkey);
            println!("  Label: {}", args.label);
            println!("  Derivation path: {}", args.derivation_path);
            if !args.tag.is_empty() {
                println!("  Tags: {}", args.tag.join(", "));
            }
        }

        LedgerCommands::List => {
            let wallets = db.list_ledger_wallets(None)?;

            if wallets.is_empty() {
                println!("No Ledger wallets found.");
                return Ok(());
            }

            println!("{:<44} {:<20} DERIVATION PATH", "PUBLIC KEY", "LABEL");
            println!("{}", "-".repeat(80));

            for wallet in wallets {
                println!(
                    "{:<44} {:<20} {}",
                    wallet.pubkey, wallet.label, wallet.derivation_path
                );
            }
        }

        LedgerCommands::Remove(args) => {
            let deleted = db.delete_ledger_wallet(&args.identifier)?;

            if deleted {
                println!("Removed Ledger wallet '{}'", args.identifier);
            } else {
                anyhow::bail!("Ledger wallet not found: {}", args.identifier);
            }
        }
    }

    Ok(())
}
