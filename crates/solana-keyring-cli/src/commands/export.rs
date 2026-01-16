//! Export a keypair

use std::path::PathBuf;

use anyhow::Result;
use solana_keyring::keypair::{export_base58, export_json};

use super::{AgentConfig, get_verified_passphrase, open_db};
use crate::cli::{ExportArgs, ExportFormat};

pub fn run(args: ExportArgs, db_path: &Option<PathBuf>, _agent_config: &AgentConfig) -> Result<()> {
    // Note: Export doesn't use agent - we need direct passphrase access to decrypt
    let db = open_db(db_path)?;
    let passphrase = get_verified_passphrase(&db)?;

    // Load keypair
    let keypair = db.load_keypair(&args.identifier, passphrase.as_bytes())?;

    // Export in requested format
    let output = match args.format {
        ExportFormat::Json => export_json(&keypair),
        ExportFormat::Base58 => export_base58(&keypair),
    };

    // Write to file or stdout
    if let Some(output_path) = args.output {
        std::fs::write(&output_path, &output)?;
        println!("Exported to {}", output_path.display());
    } else {
        println!("{}", output);
    }

    Ok(())
}
