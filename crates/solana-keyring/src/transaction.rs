//! Transaction parsing and summarization

use solana_sdk::{message::Message, pubkey::Pubkey};

/// System program ID
const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";

use crate::error::Result;

/// Summary of a transaction for display to the user
#[derive(Debug, Clone)]
pub struct TransactionSummary {
    /// Human-readable description of the transaction
    pub description: String,
    /// List of programs being invoked
    pub programs: Vec<String>,
    /// List of accounts involved
    pub accounts: Vec<AccountInfo>,
    /// Estimated fee in lamports
    pub estimated_fee: Option<u64>,
}

/// Account information in a transaction.
#[derive(Debug, Clone)]
pub struct AccountInfo {
    /// The account's public key address.
    pub address: String,
    /// Optional human-readable label from address book.
    pub label: Option<String>,
    /// Whether this account is a signer.
    pub is_signer: bool,
    /// Whether this account is writable.
    pub is_writable: bool,
}

impl std::fmt::Display for TransactionSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.description)?;
        writeln!(f)?;
        writeln!(f, "Programs: {}", self.programs.join(", "))?;
        writeln!(f)?;
        writeln!(f, "Accounts:")?;
        for acc in &self.accounts {
            let flags = match (acc.is_signer, acc.is_writable) {
                (true, true) => "[signer, writable]",
                (true, false) => "[signer]",
                (false, true) => "[writable]",
                (false, false) => "",
            };
            if let Some(label) = &acc.label {
                writeln!(f, "  {} ({}) {}", acc.address, label, flags)?;
            } else {
                writeln!(f, "  {} {}", acc.address, flags)?;
            }
        }
        Ok(())
    }
}

/// Parse a transaction message and create a summary
pub fn summarize_transaction(message_bytes: &[u8]) -> Result<TransactionSummary> {
    // Try to deserialize as a Message
    let message: Message = bincode::deserialize(message_bytes)
        .map_err(|e| crate::error::Error::Solana(format!("Failed to parse message: {}", e)))?;

    let mut programs = Vec::new();
    let mut description_parts = Vec::new();

    // Analyze each instruction
    for ix in &message.instructions {
        let program_id = message
            .account_keys
            .get(ix.program_id_index as usize)
            .map(|p| p.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let program_name = identify_program(&program_id);
        programs.push(program_name.clone());

        // Try to decode known instruction types
        if let Some(desc) =
            decode_instruction(&program_id, &ix.data, &message.account_keys, &ix.accounts)
        {
            description_parts.push(desc);
        } else {
            description_parts.push(format!("Call to {}", program_name));
        }
    }

    // Build account info
    let accounts: Vec<AccountInfo> = message
        .account_keys
        .iter()
        .enumerate()
        .map(|(i, pubkey)| {
            let is_signer = i < message.header.num_required_signatures as usize;
            let is_writable = message.is_maybe_writable(i, None);
            AccountInfo {
                address: pubkey.to_string(),
                label: None, // Can be filled in by caller with address book
                is_signer,
                is_writable,
            }
        })
        .collect();

    // Dedupe programs
    programs.sort();
    programs.dedup();

    Ok(TransactionSummary {
        description: description_parts.join("\n"),
        programs,
        accounts,
        estimated_fee: Some(5000), // Default fee estimate
    })
}

/// Identify a program by its address
fn identify_program(program_id: &str) -> String {
    match program_id {
        "11111111111111111111111111111111" => "System Program".to_string(),
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => "Token Program".to_string(),
        "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb" => "Token-2022 Program".to_string(),
        "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL" => "Associated Token Program".to_string(),
        "ComputeBudget111111111111111111111111111111" => "Compute Budget".to_string(),
        "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" => "Jupiter".to_string(),
        "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc" => "Orca Whirlpool".to_string(),
        "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK" => "Raydium CLAMM".to_string(),
        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8" => "Raydium AMM".to_string(),
        "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr" => "Memo Program".to_string(),
        "SQDS4nPHovALA9Sm5LCgJqkKhkYshJwKhN9kD3h8Zzg" => "Squads V4".to_string(),
        _ => {
            // Truncate unknown program IDs
            if program_id.len() > 12 {
                format!("{}...", &program_id[..12])
            } else {
                program_id.to_string()
            }
        }
    }
}

/// Try to decode a known instruction type
fn decode_instruction(
    program_id: &str,
    data: &[u8],
    account_keys: &[Pubkey],
    account_indices: &[u8],
) -> Option<String> {
    // System Program
    if program_id == SYSTEM_PROGRAM_ID {
        return decode_system_instruction(data, account_keys, account_indices);
    }

    // Token Program
    if program_id == "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" {
        return decode_token_instruction(data);
    }

    None
}

fn decode_system_instruction(
    data: &[u8],
    account_keys: &[Pubkey],
    account_indices: &[u8],
) -> Option<String> {
    if data.is_empty() {
        return None;
    }

    // System instruction discriminator is first 4 bytes
    let discriminator = u32::from_le_bytes(data.get(0..4)?.try_into().ok()?);

    match discriminator {
        // Transfer
        2 => {
            let lamports = u64::from_le_bytes(data.get(4..12)?.try_into().ok()?);
            let sol = lamports as f64 / 1_000_000_000.0;

            let to = account_indices
                .get(1)
                .and_then(|&i| account_keys.get(i as usize))
                .map(|p| truncate_pubkey(&p.to_string()))
                .unwrap_or_else(|| "?".to_string());

            Some(format!("Transfer {:.6} SOL to {}", sol, to))
        }
        // CreateAccount
        0 => {
            let lamports = u64::from_le_bytes(data.get(4..12)?.try_into().ok()?);
            let sol = lamports as f64 / 1_000_000_000.0;
            Some(format!("Create account with {:.6} SOL", sol))
        }
        _ => None,
    }
}

fn decode_token_instruction(data: &[u8]) -> Option<String> {
    if data.is_empty() {
        return None;
    }

    match data[0] {
        3 => {
            // Transfer
            let amount = u64::from_le_bytes(data.get(1..9)?.try_into().ok()?);
            Some(format!("Token transfer: {} units", amount))
        }
        7 => Some("Mint tokens".to_string()),
        8 => Some("Burn tokens".to_string()),
        9 => Some("Close token account".to_string()),
        _ => None,
    }
}

fn truncate_pubkey(pubkey: &str) -> String {
    if pubkey.len() > 12 {
        format!("{}...{}", &pubkey[..6], &pubkey[pubkey.len() - 4..])
    } else {
        pubkey.to_string()
    }
}
