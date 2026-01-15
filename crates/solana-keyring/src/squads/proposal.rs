//! Squads proposal creation and approval

use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signer,
    transaction::Transaction,
};

use super::{
    SquadsSigner,
    instructions::{
        ProposalCreateArgs, ProposalVoteArgs, VaultTransactionCreateArgs, proposal_approve,
        proposal_create, vault_transaction_create,
    },
    pda::{get_proposal_pda, get_transaction_pda},
};
use crate::error::{Error, Result};

/// Create a vault transaction and proposal for a transaction
pub async fn create_proposal(
    signer: &SquadsSigner,
    transaction_message: &[u8],
) -> Result<(Pubkey, u64)> {
    let rpc = signer.rpc_client();
    let multisig_pda = *signer.multisig_pda();
    let member = signer.member_keypair().to_solana_keypair();
    let member_pubkey = member.pubkey();
    let program_id = *signer.program_id();

    // Get the current transaction index from the multisig account
    let multisig_data = rpc
        .get_account_data(&multisig_pda)
        .map_err(|e| Error::Squads(format!("Failed to fetch multisig account: {}", e)))?;

    // Parse transaction_index from multisig account data
    // The Squads v4 Multisig struct layout (after 8-byte Anchor discriminator):
    // - create_key: Pubkey (32)
    // - config_authority: Pubkey (32)
    // - threshold: u16 (2)
    // - time_lock: u32 (4)
    // - transaction_index: u64 (8)
    // Offset = 8 + 32 + 32 + 2 + 4 = 78
    const TX_INDEX_OFFSET: usize = 8 + 32 + 32 + 2 + 4;

    if multisig_data.len() < TX_INDEX_OFFSET + 8 {
        return Err(Error::Squads("Invalid multisig account data".into()));
    }

    let transaction_index = u64::from_le_bytes(
        multisig_data[TX_INDEX_OFFSET..TX_INDEX_OFFSET + 8]
            .try_into()
            .map_err(|_| Error::Squads("Failed to parse transaction index".into()))?,
    );
    let next_index = transaction_index + 1;

    // Derive PDAs for the new transaction and proposal
    let transaction_pda = get_transaction_pda(&multisig_pda, next_index, &program_id);
    let proposal_pda = get_proposal_pda(&multisig_pda, next_index, &program_id);

    // Build vault transaction create instruction
    let vault_tx_args = VaultTransactionCreateArgs {
        vault_index: signer.vault_index(),
        ephemeral_signers: 0,
        transaction_message: transaction_message.to_vec(),
        memo: None,
    };

    let vault_tx_ix = vault_transaction_create(
        multisig_pda,
        transaction_pda,
        member_pubkey,
        member_pubkey,
        vault_tx_args,
        program_id,
    );

    // Build proposal create instruction
    let proposal_args = ProposalCreateArgs {
        transaction_index: next_index,
        draft: false, // Create as active immediately
    };

    let proposal_ix = proposal_create(
        multisig_pda,
        proposal_pda,
        member_pubkey,
        member_pubkey,
        proposal_args,
        program_id,
    );

    // Get recent blockhash
    let blockhash = rpc
        .get_latest_blockhash()
        .map_err(|e| Error::Squads(format!("Failed to get blockhash: {}", e)))?;

    // Build and sign transaction
    let tx = Transaction::new_signed_with_payer(
        &[vault_tx_ix, proposal_ix],
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
        .map_err(|e| Error::Squads(format!("Failed to create proposal: {}", e)))?;

    println!("Created proposal at index {}: {}", next_index, signature);

    Ok((proposal_pda, next_index))
}

/// Approve a proposal
pub async fn approve_proposal(signer: &SquadsSigner, transaction_index: u64) -> Result<()> {
    let rpc = signer.rpc_client();
    let multisig_pda = *signer.multisig_pda();
    let member = signer.member_keypair().to_solana_keypair();
    let member_pubkey = member.pubkey();
    let program_id = *signer.program_id();

    // Derive proposal PDA
    let proposal_pda = get_proposal_pda(&multisig_pda, transaction_index, &program_id);

    // Build proposal approve instruction
    let vote_args = ProposalVoteArgs { memo: None };

    let approve_ix = proposal_approve(
        multisig_pda,
        proposal_pda,
        member_pubkey,
        vote_args,
        program_id,
    );

    // Get recent blockhash
    let blockhash = rpc
        .get_latest_blockhash()
        .map_err(|e| Error::Squads(format!("Failed to get blockhash: {}", e)))?;

    // Build and sign transaction
    let tx = Transaction::new_signed_with_payer(
        &[approve_ix],
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
        .map_err(|e| Error::Squads(format!("Failed to approve proposal: {}", e)))?;

    println!("Approved proposal {}: {}", transaction_index, signature);

    Ok(())
}
