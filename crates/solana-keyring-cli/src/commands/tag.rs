//! Tag management commands

use std::path::PathBuf;

use anyhow::Result;

use super::open_db;
use crate::cli::TagCommands;

pub fn run(cmd: TagCommands, db_path: &Option<PathBuf>) -> Result<()> {
    let db = open_db(db_path)?;

    match cmd {
        TagCommands::List => {
            let tags = db.list_tags()?;

            if tags.is_empty() {
                println!("No tags found.");
                return Ok(());
            }

            println!("{:<20} COUNT", "TAG");
            println!("{}", "-".repeat(30));

            for tag in tags {
                println!("{:<20} {}", tag.name, tag.count);
            }
        }

        TagCommands::Add(args) => {
            // First find the keypair to get its pubkey
            let keypairs = db.list_keypairs(None)?;
            let keypair = keypairs
                .iter()
                .find(|k| k.pubkey == args.identifier || k.label == args.identifier)
                .ok_or_else(|| anyhow::anyhow!("Keypair not found: {}", args.identifier))?;

            db.add_tag_to_keypair(&keypair.pubkey, &args.tag)?;
            println!("Added tag '{}' to '{}'", args.tag, keypair.label);
        }

        TagCommands::Remove(args) => {
            // First find the keypair to get its pubkey
            let keypairs = db.list_keypairs(None)?;
            let keypair = keypairs
                .iter()
                .find(|k| k.pubkey == args.identifier || k.label == args.identifier)
                .ok_or_else(|| anyhow::anyhow!("Keypair not found: {}", args.identifier))?;

            let removed = db.remove_tag_from_keypair(&keypair.pubkey, &args.tag)?;
            if removed {
                println!("Removed tag '{}' from '{}'", args.tag, keypair.label);
            } else {
                println!("Tag '{}' not found on '{}'", args.tag, keypair.label);
            }
        }

        TagCommands::Delete(args) => {
            let deleted = db.delete_tag(&args.tag)?;
            if deleted {
                println!("Deleted tag '{}'", args.tag);
            } else {
                println!("Tag '{}' not found", args.tag);
            }
        }
    }

    Ok(())
}
