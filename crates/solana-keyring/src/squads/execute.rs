//! Squads proposal execution

use solana_sdk::{
    commitment_config::CommitmentConfig, instruction::AccountMeta, pubkey::Pubkey,
    signature::Signer, transaction::Transaction,
};

use super::{
    SquadsSigner,
    instructions::vault_transaction_execute,
    pda::{get_proposal_pda, get_transaction_pda, get_vault_pda},
};
use crate::error::{Error, Result};

/// Execute a proposal that has reached threshold
pub async fn execute_proposal(signer: &SquadsSigner, transaction_index: u64) -> Result<String> {
    let rpc = signer.rpc_client();
    let multisig_pda = *signer.multisig_pda();
    let member = signer.member_keypair().to_solana_keypair();
    let member_pubkey = member.pubkey();
    let program_id = *signer.program_id();

    // Derive PDAs
    let proposal_pda = get_proposal_pda(&multisig_pda, transaction_index, &program_id);
    let transaction_pda = get_transaction_pda(&multisig_pda, transaction_index, &program_id);
    let vault_pda = get_vault_pda(&multisig_pda, signer.vault_index(), &program_id);

    // Fetch the vault transaction account to get the accounts list
    let tx_data = rpc
        .get_account_data(&transaction_pda)
        .map_err(|e| Error::Squads(format!("Failed to fetch transaction account: {}", e)))?;

    // Parse the remaining accounts from the vault transaction
    let remaining_accounts = parse_vault_transaction_accounts(&tx_data, vault_pda)?;

    // Build vault transaction execute instruction
    let execute_ix = vault_transaction_execute(
        multisig_pda,
        proposal_pda,
        transaction_pda,
        member_pubkey,
        remaining_accounts,
        program_id,
    );

    // Get recent blockhash
    let blockhash = rpc
        .get_latest_blockhash()
        .map_err(|e| Error::Squads(format!("Failed to get blockhash: {}", e)))?;

    // Build and sign transaction
    let tx = Transaction::new_signed_with_payer(
        &[execute_ix],
        Some(&member_pubkey),
        &[member],
        blockhash,
    );

    // Send transaction
    let signature = rpc
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &tx,
            CommitmentConfig::confirmed(),
        )
        .map_err(|e| Error::Squads(format!("Failed to execute proposal: {}", e)))?;

    println!("Executed proposal {}: {}", transaction_index, signature);

    Ok(signature.to_string())
}

/// Parse the remaining accounts needed for execution from the vault transaction data
fn parse_vault_transaction_accounts(tx_data: &[u8], vault_pda: Pubkey) -> Result<Vec<AccountMeta>> {
    // VaultTransaction struct layout (after 8-byte Anchor discriminator):
    // - multisig: Pubkey (32)
    // - creator: Pubkey (32)
    // - index: u64 (8)
    // - bump: u8 (1)
    // - vault_index: u8 (1)
    // - vault_bump: u8 (1)
    // - ephemeral_signer_bumps: Vec<u8> (4 + n)
    // - message: TransactionMessage (variable)

    const FIXED_SIZE: usize = 8 + 32 + 32 + 8 + 1 + 1 + 1;

    if tx_data.len() < FIXED_SIZE {
        return Err(Error::Squads("Invalid vault transaction data".into()));
    }

    // The vault is always needed as a signer for execution
    let mut accounts = vec![AccountMeta::new(vault_pda, false)];

    // Skip the fixed fields to get to ephemeral_signer_bumps
    let mut offset = FIXED_SIZE;

    if offset + 4 > tx_data.len() {
        return Err(Error::Squads("Invalid vault transaction data".into()));
    }

    // Read ephemeral_signer_bumps length (Borsh Vec encoding: 4-byte length prefix)
    let ephemeral_len = u32::from_le_bytes(
        tx_data[offset..offset + 4]
            .try_into()
            .map_err(|_| Error::Squads("Failed to parse ephemeral signers length".into()))?,
    ) as usize;
    offset += 4 + ephemeral_len;

    // Now we're at the TransactionMessage
    // TransactionMessage struct (Borsh encoding):
    // - num_signers: u8 (1)
    // - num_writable_signers: u8 (1)
    // - num_writable_non_signers: u8 (1)
    // - account_keys: Vec<Pubkey> (4 + 32*n)
    // - instructions: Vec<CompiledInstruction> (variable)
    // - address_table_lookups: Vec<MessageAddressTableLookup> (variable)

    if offset + 3 > tx_data.len() {
        return Ok(accounts); // Return just the vault if we can't parse further
    }

    let num_signers = tx_data[offset] as usize;
    let num_writable_signers = tx_data[offset + 1] as usize;
    let num_writable_non_signers = tx_data[offset + 2] as usize;
    offset += 3;

    if offset + 4 > tx_data.len() {
        return Ok(accounts);
    }

    // Read account_keys length
    let num_keys = u32::from_le_bytes(
        tx_data[offset..offset + 4]
            .try_into()
            .map_err(|_| Error::Squads("Failed to parse account keys length".into()))?,
    ) as usize;
    offset += 4;

    // Read each account key and add to remaining accounts
    for i in 0..num_keys {
        if offset + 32 > tx_data.len() {
            break;
        }

        let key_bytes: [u8; 32] = tx_data[offset..offset + 32]
            .try_into()
            .map_err(|_| Error::Squads("Failed to parse account key".into()))?;

        let pubkey = Pubkey::new_from_array(key_bytes);

        // Skip the vault itself, we already added it
        if pubkey != vault_pda {
            // Determine if this account is writable based on position
            let is_signer = i < num_signers;
            let is_writable = if is_signer {
                i < num_writable_signers
            } else {
                i < num_signers + num_writable_non_signers
            };

            if is_writable {
                accounts.push(AccountMeta::new(pubkey, false));
            } else {
                accounts.push(AccountMeta::new_readonly(pubkey, false));
            }
        }

        offset += 32;
    }

    Ok(accounts)
}
