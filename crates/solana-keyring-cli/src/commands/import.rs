//! Import a keypair

use std::path::PathBuf;

use anyhow::Result;
use solana_keyring::keypair::{import_base58, import_json};

use super::{get_verified_passphrase, open_db, prompt_passphrase};
use crate::cli::ImportArgs;

pub fn run(args: ImportArgs, db_path: &Option<PathBuf>) -> Result<()> {
    let db = open_db(db_path)?;
    let passphrase = get_verified_passphrase(&db)?;

    // Import keypair from file or base58
    let keypair = if let Some(file_path) = &args.file {
        import_json(file_path)?
    } else if let Some(base58) = &args.base58 {
        import_base58(base58)?
    } else {
        // Read from stdin
        let input = prompt_passphrase("Enter base58 encoded secret key: ")?;
        import_base58(&input)?
    };

    let pubkey = keypair.pubkey_base58();

    // Convert tags to &str slice
    let tags: Vec<&str> = args.tag.iter().map(|s| s.as_str()).collect();

    // Store in database
    db.store_keypair(&keypair, &args.label, passphrase.as_bytes(), &tags)?;

    println!("Imported keypair:");
    println!("  Public key: {}", pubkey);
    println!("  Label: {}", args.label);
    if !args.tag.is_empty() {
        println!("  Tags: {}", args.tag.join(", "));
    }

    Ok(())
}
