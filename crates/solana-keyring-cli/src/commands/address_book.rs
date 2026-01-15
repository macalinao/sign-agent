//! Address book commands

use std::path::PathBuf;

use anyhow::Result;
use solana_keyring::AddressBook;

use super::open_db;
use crate::cli::AddressBookCommands;

pub fn run(cmd: AddressBookCommands, db_path: &Option<PathBuf>) -> Result<()> {
    let db = open_db(db_path)?;
    let book = AddressBook::new(&db);

    match cmd {
        AddressBookCommands::Add(args) => {
            book.add(&args.pubkey, &args.label, args.notes.as_deref())?;

            println!("Added address:");
            println!("  Public key: {}", args.pubkey);
            println!("  Label: {}", args.label);
            if let Some(notes) = &args.notes {
                println!("  Notes: {}", notes);
            }
        }

        AddressBookCommands::List => {
            let addresses = book.list()?;

            if addresses.is_empty() {
                println!("No addresses in address book.");
                return Ok(());
            }

            println!("{:<44} {:<20} NOTES", "PUBLIC KEY", "LABEL");
            println!("{}", "-".repeat(80));

            for addr in addresses {
                println!(
                    "{:<44} {:<20} {}",
                    addr.pubkey,
                    addr.label,
                    addr.notes.unwrap_or_default()
                );
            }
        }

        AddressBookCommands::Remove(args) => {
            let removed = book.remove(&args.identifier)?;

            if removed {
                println!("Removed address '{}'", args.identifier);
            } else {
                anyhow::bail!("Address not found: {}", args.identifier);
            }
        }

        AddressBookCommands::Label(args) => {
            let updated = book.update_label(&args.identifier, &args.label)?;

            if updated {
                println!("Updated label to '{}'", args.label);
            } else {
                anyhow::bail!("Address not found: {}", args.identifier);
            }
        }
    }

    Ok(())
}
