//! Import a keypair

use std::path::PathBuf;

use anyhow::Result;
use solana_keyring::keypair::{import_base58, import_json};

use super::{AgentConfig, agent_client, get_verified_passphrase, open_db, prompt_passphrase};
use crate::cli::ImportArgs;

pub fn run(args: ImportArgs, db_path: &Option<PathBuf>, agent_config: &AgentConfig) -> Result<()> {
    // Try using agent first if requested and we have base58 input
    if agent_config.use_agent
        && let Some(base58) = &args.base58
    {
        let socket_path = agent_config.socket_path();
        let rt = tokio::runtime::Runtime::new()?;

        if rt.block_on(agent_client::is_agent_available(&socket_path)) {
            let result = rt.block_on(agent_client::import_keypair(
                &socket_path,
                &args.label,
                base58,
                &args.tag,
            ))?;

            println!("Imported keypair:");
            println!("  Public key: {}", result.pubkey);
            println!("  Label: {}", result.label);
            if !args.tag.is_empty() {
                println!("  Tags: {}", args.tag.join(", "));
            }
            return Ok(());
        } else {
            println!("Agent not available or not unlocked, falling back to passphrase prompt...");
        }
    }

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
