//! Program Derived Address utilities for Squads v4

use solana_sdk::pubkey::Pubkey;

/// Seed prefix for vault PDAs
pub const SEED_VAULT: &[u8] = b"squad";
/// Seed suffix for vault authority
pub const SEED_AUTHORITY: &[u8] = b"authority";
/// Seed prefix for transaction PDAs
pub const SEED_TRANSACTION: &[u8] = b"transaction";
/// Seed prefix for proposal PDAs
pub const SEED_PROPOSAL: &[u8] = b"proposal";

/// Get the vault PDA for a multisig
pub fn get_vault_pda(multisig: &Pubkey, vault_index: u8, program_id: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(
        &[
            SEED_VAULT,
            multisig.as_ref(),
            &[vault_index],
            SEED_AUTHORITY,
        ],
        program_id,
    );
    pda
}

/// Get the transaction PDA for a multisig transaction
pub fn get_transaction_pda(
    multisig: &Pubkey,
    transaction_index: u64,
    program_id: &Pubkey,
) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(
        &[
            SEED_TRANSACTION,
            multisig.as_ref(),
            &transaction_index.to_le_bytes(),
        ],
        program_id,
    );
    pda
}

/// Get the proposal PDA for a multisig transaction
pub fn get_proposal_pda(multisig: &Pubkey, transaction_index: u64, program_id: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(
        &[
            SEED_PROPOSAL,
            multisig.as_ref(),
            &transaction_index.to_le_bytes(),
        ],
        program_id,
    );
    pda
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_pda_derivation() {
        let multisig = Pubkey::new_unique();
        let program_id: Pubkey = super::super::SQUADS_PROGRAM_ID.parse().unwrap();
        let vault = get_vault_pda(&multisig, 0, &program_id);
        // Just verify it doesn't panic and returns a valid pubkey
        assert_ne!(vault, multisig);
    }

    #[test]
    fn test_transaction_pda_derivation() {
        let multisig = Pubkey::new_unique();
        let program_id: Pubkey = super::super::SQUADS_PROGRAM_ID.parse().unwrap();
        let tx_pda = get_transaction_pda(&multisig, 1, &program_id);
        assert_ne!(tx_pda, multisig);
    }

    #[test]
    fn test_proposal_pda_derivation() {
        let multisig = Pubkey::new_unique();
        let program_id: Pubkey = super::super::SQUADS_PROGRAM_ID.parse().unwrap();
        let proposal_pda = get_proposal_pda(&multisig, 1, &program_id);
        assert_ne!(proposal_pda, multisig);
    }
}
