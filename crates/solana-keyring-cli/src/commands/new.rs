//! Initialize a new keyring

use std::path::PathBuf;

use anyhow::Result;
use solana_keyring::{Database, default_db_path};

use super::prompt_passphrase;
use crate::cli::NewArgs;

pub fn run(args: NewArgs, db_path: &Option<PathBuf>) -> Result<()> {
    let path = db_path.clone().unwrap_or_else(default_db_path);

    // Check if database already exists
    if path.exists() && !args.force {
        anyhow::bail!(
            "Keyring already exists at {}. Use --force to overwrite.",
            path.display()
        );
    }

    // Remove existing database if force
    if path.exists() && args.force {
        std::fs::remove_file(&path)?;
    }

    // Create parent directory
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Prompt for passphrase
    let passphrase = prompt_passphrase("Enter new master passphrase: ")?;
    let confirm = prompt_passphrase("Confirm master passphrase: ")?;

    if passphrase != confirm {
        anyhow::bail!("Passphrases do not match");
    }

    if passphrase.len() < 8 {
        anyhow::bail!("Passphrase must be at least 8 characters");
    }

    // Create and initialize database
    let db = Database::open(&path)?;
    db.initialize(passphrase.as_bytes())?;

    println!("Keyring initialized at {}", path.display());
    Ok(())
}
