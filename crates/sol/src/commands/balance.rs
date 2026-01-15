//! Check account balance

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::cli::BalanceArgs;

pub async fn run(args: BalanceArgs) -> Result<()> {
    let rpc = RpcClient::new(&args.rpc);

    // Resolve address (could be a pubkey or label)
    let pubkey = resolve_address(&args.address, args.db_path.as_ref())?;

    // Get balance
    let balance = rpc.get_balance(&pubkey)?;
    let sol_balance = balance as f64 / 1_000_000_000.0;

    println!("Address: {}", pubkey);
    println!("Balance: {} SOL ({} lamports)", sol_balance, balance);

    Ok(())
}

fn resolve_address(address: &str, db_path: Option<&std::path::PathBuf>) -> Result<Pubkey> {
    // Try to parse as a pubkey first
    if let Ok(pubkey) = address.parse::<Pubkey>() {
        return Ok(pubkey);
    }

    // Try to look up in keyring database
    if let Some(db_path) = db_path {
        let db = solana_keyring::Database::open(db_path)?;

        // Try keypairs
        if let Ok(keypairs) = db.list_keypairs(None)
            && let Some(kp) = keypairs.iter().find(|k| k.label == address)
        {
            return kp
                .pubkey
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid pubkey in keyring: {}", kp.pubkey));
        }

        // Try ledger wallets
        if let Ok(wallets) = db.list_ledger_wallets(None)
            && let Some(w) = wallets.iter().find(|w| w.label == address)
        {
            return w
                .pubkey
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid pubkey in keyring: {}", w.pubkey));
        }
    }

    // Try default database path
    let default_db_path = solana_keyring::default_db_path();
    if default_db_path.exists() {
        let db = solana_keyring::Database::open(&default_db_path)?;

        // Try keypairs
        if let Ok(keypairs) = db.list_keypairs(None)
            && let Some(kp) = keypairs.iter().find(|k| k.label == address)
        {
            return kp
                .pubkey
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid pubkey in keyring: {}", kp.pubkey));
        }

        // Try ledger wallets
        if let Ok(wallets) = db.list_ledger_wallets(None)
            && let Some(w) = wallets.iter().find(|w| w.label == address)
        {
            return w
                .pubkey
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid pubkey in keyring: {}", w.pubkey));
        }
    }

    anyhow::bail!(
        "Could not resolve address '{}'. Provide a valid public key or label from keyring.",
        address
    )
}
