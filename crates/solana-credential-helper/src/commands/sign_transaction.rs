//! Sign transaction command

use std::io::{self, Read, Write};

use anyhow::Result;
use base64::Engine;
use solana_keyring::{Database, default_agent_socket_path, default_db_path};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::cli::{Encoding, SignTransactionArgs};

pub async fn run(args: SignTransactionArgs) -> Result<()> {
    // Read transaction from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let input = input.trim();

    // Decode transaction
    let tx_bytes = match args.encoding {
        Encoding::Base64 => base64::engine::general_purpose::STANDARD.decode(input)?,
        Encoding::Base58 => bs58::decode(input).into_vec()?,
    };

    // Sign the transaction
    let signature = if args.use_agent {
        sign_via_agent(&args, &tx_bytes).await?
    } else if args.ledger {
        sign_with_ledger(&args, &tx_bytes)?
    } else if args.squads.is_some() {
        sign_with_squads(&args, &tx_bytes).await?
    } else {
        sign_with_keypair(&args, &tx_bytes)?
    };

    // Encode and output signature
    let output = match args.encoding {
        Encoding::Base64 => base64::engine::general_purpose::STANDARD.encode(signature),
        Encoding::Base58 => bs58::encode(signature).into_string(),
    };

    io::stdout().write_all(output.as_bytes())?;
    io::stdout().flush()?;

    Ok(())
}

async fn sign_via_agent(args: &SignTransactionArgs, tx_bytes: &[u8]) -> Result<[u8; 64]> {
    let socket_path = args
        .agent_socket
        .clone()
        .unwrap_or_else(default_agent_socket_path);

    let mut stream = UnixStream::connect(&socket_path).await?;

    // Build request
    let request = serde_json::json!({
        "method": "SignTransaction",
        "params": {
            "transaction": base64::engine::general_purpose::STANDARD.encode(tx_bytes),
            "signer": args.signer
        }
    });
    let request_bytes = serde_json::to_vec(&request)?;

    // Send request
    stream
        .write_all(&(request_bytes.len() as u32).to_be_bytes())
        .await?;
    stream.write_all(&request_bytes).await?;

    // Read response
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;

    let response: serde_json::Value = serde_json::from_slice(&buf)?;

    if response["status"] == "error" {
        anyhow::bail!(
            "Agent error: {}",
            response["message"].as_str().unwrap_or("Unknown error")
        );
    }

    // Decode signature
    let sig_b64 = response["result"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid response from agent"))?;

    let sig_bytes = base64::engine::general_purpose::STANDARD.decode(sig_b64)?;
    let sig: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid signature length"))?;

    Ok(sig)
}

fn sign_with_keypair(args: &SignTransactionArgs, tx_bytes: &[u8]) -> Result<[u8; 64]> {
    let db_path = args.db_path.clone().unwrap_or_else(default_db_path);
    let db = Database::open(&db_path)?;

    if !db.is_initialized()? {
        anyhow::bail!("Keyring not initialized. Run 'solana-keyring new' first.");
    }

    // Prompt for passphrase
    let passphrase = rpassword::prompt_password("Enter master passphrase: ")?;

    if !db.verify_passphrase(passphrase.as_bytes())? {
        anyhow::bail!("Invalid passphrase");
    }

    // Load keypair
    let keypair = db.load_keypair(&args.signer, passphrase.as_bytes())?;

    // Sign
    let signature = keypair.sign(tx_bytes);

    // Notify
    solana_keyring::notify(
        "Transaction Signed",
        &format!("Signed with {}", args.signer),
    )?;

    Ok(signature)
}

fn sign_with_ledger(args: &SignTransactionArgs, tx_bytes: &[u8]) -> Result<[u8; 64]> {
    use solana_keyring::ledger::LedgerSigner;

    eprintln!("Connecting to Ledger device...");
    eprintln!("Please confirm the transaction on your device.");

    let db_path = args.db_path.clone().unwrap_or_else(default_db_path);
    let db = Database::open(&db_path)?;

    // Find the Ledger wallet in database to get derivation path
    let wallets = db.list_ledger_wallets(None)?;
    let wallet = wallets
        .iter()
        .find(|w| w.pubkey == args.signer || w.label == args.signer)
        .ok_or_else(|| anyhow::anyhow!("Ledger wallet not found: {}", args.signer))?;

    let signer = LedgerSigner::connect(&wallet.derivation_path)?;
    let signature = signer.sign(tx_bytes)?;

    // Notify
    solana_keyring::notify(
        "Transaction Signed",
        &format!("Signed with Ledger: {}", args.signer),
    )?;

    Ok(signature)
}

async fn sign_with_squads(args: &SignTransactionArgs, tx_bytes: &[u8]) -> Result<[u8; 64]> {
    use solana_keyring::squads::SquadsSigner;

    let multisig_address = args
        .squads
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Squads address required"))?;

    let db_path = args.db_path.clone().unwrap_or_else(default_db_path);
    let db = Database::open(&db_path)?;

    if !db.is_initialized()? {
        anyhow::bail!("Keyring not initialized. Run 'solana-keyring new' first.");
    }

    // Prompt for passphrase
    let passphrase = rpassword::prompt_password("Enter master passphrase: ")?;

    if !db.verify_passphrase(passphrase.as_bytes())? {
        anyhow::bail!("Invalid passphrase");
    }

    // Load member keypair (the signer is the member who will sign the proposal)
    let member_keypair = db.load_keypair(&args.signer, passphrase.as_bytes())?;

    eprintln!("Creating Squads proposal for transaction...");
    eprintln!("Multisig: {}", multisig_address);
    eprintln!("Member: {}", member_keypair.pubkey_base58());

    // Create Squads signer
    let signer = SquadsSigner::new(multisig_address, 0, &args.rpc_url, member_keypair)?;

    // Create proposal
    let (proposal_pda, transaction_index) = signer.create_proposal(tx_bytes).await?;

    eprintln!(
        "Created proposal #{} at {}",
        transaction_index, proposal_pda
    );

    // Approve the proposal
    eprintln!("Approving proposal...");
    signer.approve_proposal(transaction_index).await?;

    eprintln!("Proposal approved!");
    eprintln!("Note: If threshold is met, use Squads UI or CLI to execute the transaction.");

    // Notify
    solana_keyring::notify(
        "Squads Proposal Created",
        &format!("Proposal #{} created and approved", transaction_index),
    )?;

    // Return zeros since we didn't directly sign (Squads executes the transaction)
    // The actual transaction will be signed by the vault PDA during execution
    Ok([0u8; 64])
}
