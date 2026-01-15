//! Generate a new keypair

use std::path::PathBuf;

use anyhow::Result;
use solana_keyring::keypair::generate_keypair;

use super::{get_verified_passphrase, open_db};
use crate::cli::GenerateArgs;

pub fn run(args: GenerateArgs, db_path: &Option<PathBuf>) -> Result<()> {
    let db = open_db(db_path)?;
    let passphrase = get_verified_passphrase(&db)?;

    // Generate keypair
    let keypair = generate_keypair();
    let pubkey = keypair.pubkey_base58();

    // Convert tags to &str slice
    let tags: Vec<&str> = args.tag.iter().map(|s| s.as_str()).collect();

    // Store in database
    db.store_keypair(&keypair, &args.label, passphrase.as_bytes(), &tags)?;

    println!("Generated keypair:");
    println!("  Public key: {}", pubkey);
    println!("  Label: {}", args.label);
    if !args.tag.is_empty() {
        println!("  Tags: {}", args.tag.join(", "));
    }

    Ok(())
}
