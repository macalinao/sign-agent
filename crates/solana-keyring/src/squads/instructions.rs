//! Squads v4 instruction builders
//!
//! Builds instructions for interacting with the Squads v4 program.

use borsh::BorshSerialize;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

/// System program ID
const SYSTEM_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("11111111111111111111111111111111");

/// Squads v4 instruction discriminators (Anchor-style 8-byte discriminators)
mod discriminator {
    /// vault_transaction_create
    pub const VAULT_TRANSACTION_CREATE: [u8; 8] = [228, 22, 32, 93, 162, 205, 116, 143];
    /// proposal_create
    pub const PROPOSAL_CREATE: [u8; 8] = [132, 116, 68, 174, 216, 160, 198, 22];
    /// proposal_approve
    pub const PROPOSAL_APPROVE: [u8; 8] = [227, 43, 144, 43, 163, 82, 190, 5];
    /// vault_transaction_execute
    pub const VAULT_TRANSACTION_EXECUTE: [u8; 8] = [142, 231, 170, 21, 232, 184, 207, 168];
}

/// Arguments for vault_transaction_create instruction
#[derive(BorshSerialize)]
pub struct VaultTransactionCreateArgs {
    /// Vault index
    pub vault_index: u8,
    /// Number of ephemeral signers required
    pub ephemeral_signers: u8,
    /// Serialized transaction message
    pub transaction_message: Vec<u8>,
    /// Optional memo
    pub memo: Option<String>,
}

/// Arguments for proposal_create instruction
#[derive(BorshSerialize)]
pub struct ProposalCreateArgs {
    /// Transaction index to create proposal for
    pub transaction_index: u64,
    /// Whether to create as draft (inactive)
    pub draft: bool,
}

/// Arguments for proposal_approve instruction
#[derive(BorshSerialize)]
pub struct ProposalVoteArgs {
    /// Optional memo
    pub memo: Option<String>,
}

/// Build a vault_transaction_create instruction
pub fn vault_transaction_create(
    multisig: Pubkey,
    transaction: Pubkey,
    creator: Pubkey,
    rent_payer: Pubkey,
    args: VaultTransactionCreateArgs,
    program_id: Pubkey,
) -> Instruction {
    let mut data = discriminator::VAULT_TRANSACTION_CREATE.to_vec();
    data.extend(borsh::to_vec(&args).unwrap());

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(multisig, false),
            AccountMeta::new(transaction, false),
            AccountMeta::new_readonly(creator, true),
            AccountMeta::new(rent_payer, true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

/// Build a proposal_create instruction
pub fn proposal_create(
    multisig: Pubkey,
    proposal: Pubkey,
    creator: Pubkey,
    rent_payer: Pubkey,
    args: ProposalCreateArgs,
    program_id: Pubkey,
) -> Instruction {
    let mut data = discriminator::PROPOSAL_CREATE.to_vec();
    data.extend(borsh::to_vec(&args).unwrap());

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(multisig, false),
            AccountMeta::new(proposal, false),
            AccountMeta::new_readonly(creator, true),
            AccountMeta::new(rent_payer, true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data,
    }
}

/// Build a proposal_approve instruction
pub fn proposal_approve(
    multisig: Pubkey,
    proposal: Pubkey,
    member: Pubkey,
    args: ProposalVoteArgs,
    program_id: Pubkey,
) -> Instruction {
    let mut data = discriminator::PROPOSAL_APPROVE.to_vec();
    data.extend(borsh::to_vec(&args).unwrap());

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(multisig, false),
            AccountMeta::new(proposal, false),
            AccountMeta::new_readonly(member, true),
        ],
        data,
    }
}

/// Build a vault_transaction_execute instruction
pub fn vault_transaction_execute(
    multisig: Pubkey,
    proposal: Pubkey,
    transaction: Pubkey,
    member: Pubkey,
    remaining_accounts: Vec<AccountMeta>,
    program_id: Pubkey,
) -> Instruction {
    let data = discriminator::VAULT_TRANSACTION_EXECUTE.to_vec();

    let mut accounts = vec![
        AccountMeta::new_readonly(multisig, false),
        AccountMeta::new(proposal, false),
        AccountMeta::new(transaction, false),
        AccountMeta::new_readonly(member, true),
    ];
    accounts.extend(remaining_accounts);

    Instruction {
        program_id,
        accounts,
        data,
    }
}
