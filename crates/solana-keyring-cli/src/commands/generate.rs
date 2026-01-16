//! Generate a new keypair

use std::path::PathBuf;

use anyhow::Result;
use solana_keyring::keypair::generate_keypair;

use super::{AgentConfig, agent_client, get_verified_passphrase, open_db};
use crate::cli::GenerateArgs;

pub fn run(args: GenerateArgs, db_path: &Option<PathBuf>, agent_config: &AgentConfig) -> Result<()> {
    // Try using agent first if requested
    if agent_config.use_agent {
        let socket_path = agent_config.socket_path();

        // Use tokio runtime to run async code
        let rt = tokio::runtime::Runtime::new()?;

        // Check if agent is available
        if rt.block_on(agent_client::is_agent_available(&socket_path)) {
            // Generate via agent
            let result =
                rt.block_on(agent_client::generate_keypair(&socket_path, &args.label, &args.tag))?;

            println!("Generated keypair:");
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

    // Fall back to direct database access with passphrase prompt
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
