//! CLI command implementations

pub mod address_book;
pub mod delete;
pub mod export;
pub mod generate;
pub mod import;
pub mod label;
pub mod ledger;
pub mod list;
pub mod new;
pub mod squads;
pub mod tag;

use std::path::PathBuf;

use anyhow::Result;
use solana_keyring::{Database, default_db_path};

/// Get the database path, using the provided path or the default
pub fn get_db_path(path: &Option<PathBuf>) -> PathBuf {
    path.clone().unwrap_or_else(default_db_path)
}

/// Open the database, ensuring it's initialized
pub fn open_db(path: &Option<PathBuf>) -> Result<Database> {
    let db_path = get_db_path(path);
    let db = Database::open(&db_path)?;

    if !db.is_initialized()? {
        anyhow::bail!("Keyring not initialized. Run 'solana-keyring new' first.");
    }

    Ok(db)
}

/// Prompt for the master passphrase
pub fn prompt_passphrase(prompt: &str) -> Result<String> {
    rpassword::prompt_password(prompt).map_err(Into::into)
}

/// Prompt for passphrase and verify it
pub fn get_verified_passphrase(db: &Database) -> Result<String> {
    let passphrase = prompt_passphrase("Enter master passphrase: ")?;

    if !db.verify_passphrase(passphrase.as_bytes())? {
        anyhow::bail!("Invalid passphrase");
    }

    Ok(passphrase)
}
