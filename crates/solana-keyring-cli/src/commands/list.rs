//! List signers

use std::path::PathBuf;

use anyhow::Result;
use solana_keyring::{SignerType, list_signers};

use super::open_db;
use crate::cli::{ListArgs, OutputFormat, SignerTypeFilter};

pub fn run(args: ListArgs, db_path: &Option<PathBuf>) -> Result<()> {
    let db = open_db(db_path)?;

    let signers = list_signers(&db, args.tag.as_deref())?;

    // Filter by type if specified
    let filtered: Vec<_> = signers
        .into_iter()
        .filter(|s| match &args.signer_type {
            Some(SignerTypeFilter::Keypair) => s.signer_type == SignerType::Keypair,
            Some(SignerTypeFilter::Ledger) => s.signer_type == SignerType::Ledger,
            Some(SignerTypeFilter::Squads) => s.signer_type == SignerType::Squads,
            Some(SignerTypeFilter::All) | None => true,
        })
        .collect();

    match args.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&filtered)?);
        }
        OutputFormat::Table => {
            if filtered.is_empty() {
                println!("No signers found.");
                return Ok(());
            }

            println!("{:<8} {:<44} {:<20} TAGS", "TYPE", "PUBLIC KEY", "LABEL");
            println!("{}", "-".repeat(90));

            for signer in filtered {
                let tags = if signer.tags.is_empty() {
                    String::new()
                } else {
                    signer.tags.join(", ")
                };

                println!(
                    "{:<8} {:<44} {:<20} {}",
                    signer.signer_type.to_string(),
                    signer.pubkey,
                    truncate(&signer.label, 20),
                    tags
                );
            }
        }
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
