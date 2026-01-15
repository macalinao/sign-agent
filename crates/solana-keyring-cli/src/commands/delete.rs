//! Delete a keypair

use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::Result;

use super::open_db;
use crate::cli::DeleteArgs;

pub fn run(args: DeleteArgs, db_path: &Option<PathBuf>) -> Result<()> {
    let db = open_db(db_path)?;

    // Confirm deletion
    if !args.force {
        print!(
            "Are you sure you want to delete '{}'? This cannot be undone. [y/N] ",
            args.identifier
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let deleted = db.delete_keypair(&args.identifier)?;

    if deleted {
        println!("Deleted keypair '{}'", args.identifier);
    } else {
        anyhow::bail!("Keypair not found: {}", args.identifier);
    }

    Ok(())
}
