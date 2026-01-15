//! Update label for a keypair

use std::path::PathBuf;

use anyhow::Result;

use super::open_db;
use crate::cli::LabelArgs;

pub fn run(args: LabelArgs, db_path: &Option<PathBuf>) -> Result<()> {
    let db = open_db(db_path)?;

    let updated = db.update_keypair_label(&args.identifier, &args.label)?;

    if updated {
        println!("Updated label to '{}'", args.label);
    } else {
        anyhow::bail!("Keypair not found: {}", args.identifier);
    }

    Ok(())
}
