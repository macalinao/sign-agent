//! Transfer SOL command

use std::io::{self, Write};

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_credential_helper_client::{CredentialHelperClient, CredentialHelperConfig, SignerType};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Signature, transaction::Transaction,
};
use solana_system_interface::instruction as system_instruction;

use crate::cli::TransferArgs;

pub async fn run(args: TransferArgs) -> Result<()> {
    let rpc = RpcClient::new(&args.rpc);

    // Resolve source address
    let from_pubkey = resolve_address(&args.from, args.db_path.as_ref())?;
    let to_pubkey: Pubkey = args
        .to
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid destination address: {}", args.to))?;

    let lamports = (args.amount * LAMPORTS_PER_SOL as f64) as u64;

    // Get balance to verify sufficient funds
    let balance = rpc.get_balance(&from_pubkey)?;

    println!("Transfer Details:");
    println!("  From: {} ({})", args.from, from_pubkey);
    println!("  To:   {}", to_pubkey);
    println!("  Amount: {} SOL ({} lamports)", args.amount, lamports);
    println!(
        "  Current balance: {} SOL",
        balance as f64 / LAMPORTS_PER_SOL as f64
    );
    println!();

    if balance < lamports {
        anyhow::bail!(
            "Insufficient balance: {} SOL < {} SOL",
            balance as f64 / LAMPORTS_PER_SOL as f64,
            args.amount
        );
    }

    // Confirm unless --yes flag
    if !args.yes {
        print!("Proceed with transfer? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Build transfer instruction
    let instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, lamports);

    // Get recent blockhash
    let blockhash = rpc.get_latest_blockhash()?;

    // Build transaction (unsigned) with blockhash
    let message = solana_sdk::message::Message::new_with_blockhash(
        &[instruction],
        Some(&from_pubkey),
        &blockhash,
    );
    let tx = Transaction::new_unsigned(message);

    // Serialize the transaction message for signing
    let tx_message_bytes = tx.message.serialize();

    println!("Signing transaction...");

    // Sign via credential helper client
    let signature = sign_transaction(&args, &from_pubkey, &tx_message_bytes).await?;

    // Add signature to transaction
    let mut signed_tx = tx;
    signed_tx.signatures = vec![signature];

    // Send and confirm
    println!("Sending transaction...");
    let tx_signature = rpc.send_and_confirm_transaction(&signed_tx)?;

    println!();
    println!("Success!");
    println!("Transaction signature: {}", tx_signature);
    println!("Explorer: https://solscan.io/tx/{}", tx_signature);

    Ok(())
}

fn resolve_address(address: &str, db_path: Option<&std::path::PathBuf>) -> Result<Pubkey> {
    // Try to parse as a pubkey first
    if let Ok(pubkey) = address.parse::<Pubkey>() {
        return Ok(pubkey);
    }

    // Try to look up in keyring database
    let db_path = db_path
        .cloned()
        .unwrap_or_else(solana_keyring::default_db_path);

    if db_path.exists() {
        let db = solana_keyring::Database::open(&db_path)?;

        // Try keypairs
        if let Ok(keypairs) = db.list_keypairs(None)
            && let Some(kp) = keypairs.iter().find(|k| k.label == address)
        {
            return kp
                .pubkey
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid pubkey in keyring: {}", kp.pubkey));
        }
    }

    anyhow::bail!(
        "Could not resolve address '{}'. Provide a valid public key or label from keyring.",
        address
    )
}

async fn sign_transaction(
    args: &TransferArgs,
    signer_pubkey: &Pubkey,
    message_bytes: &[u8],
) -> Result<Signature> {
    // Build config for the credential helper client
    let mut config = CredentialHelperConfig::new(signer_pubkey.to_string())
        .signer_type(SignerType::Keypair)
        .use_agent(args.use_agent);

    if let Some(ref socket_path) = args.agent_socket {
        config = config.agent_socket_path(socket_path.clone());
    }

    if let Some(ref db_path) = args.db_path {
        config = config.db_path(db_path.clone());
    }

    let client = CredentialHelperClient::new(config);

    // If not using agent, we need to handle direct signing with passphrase
    // The CLI subprocess will handle passphrase prompting
    if args.use_agent {
        // Sign via agent
        let signature = client.sign_transaction(message_bytes).await?;
        Ok(signature)
    } else {
        // For direct signing, we still use solana-keyring directly
        // since the credential helper client CLI mode requires the binary
        sign_directly(args, signer_pubkey, message_bytes)
    }
}

fn sign_directly(
    args: &TransferArgs,
    _signer_pubkey: &Pubkey,
    message_bytes: &[u8],
) -> Result<Signature> {
    let db_path = args
        .db_path
        .clone()
        .unwrap_or_else(solana_keyring::default_db_path);
    let db = solana_keyring::Database::open(&db_path)?;

    if !db.is_initialized()? {
        anyhow::bail!("Keyring not initialized. Run 'solana-keyring new' first.");
    }

    // Prompt for passphrase
    let passphrase = rpassword::prompt_password("Enter master passphrase: ")?;

    if !db.verify_passphrase(passphrase.as_bytes())? {
        anyhow::bail!("Invalid passphrase");
    }

    // Load keypair
    let keypair = db.load_keypair(&args.from, passphrase.as_bytes())?;

    // Sign
    let signature_bytes = keypair.sign(message_bytes);

    Ok(Signature::from(signature_bytes))
}
