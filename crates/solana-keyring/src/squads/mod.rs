//! Squads multisig integration
//!
//! Implements Squads Protocol v4 for multi-signature transaction management.

mod execute;
mod instructions;
mod pda;
mod proposal;

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::error::{Error, Result};
use crate::keypair::SecureKeypair;

pub use instructions::*;
pub use pda::*;

/// Squads V4 program ID (mainnet)
pub const SQUADS_PROGRAM_ID: &str = "SQDS4nPHovALA9Sm5LCgJqkKhkYshJwKhN9kD3h8Zzg";

/// Squads multisig signer
pub struct SquadsSigner {
    multisig_pda: Pubkey,
    vault_index: u8,
    rpc_client: RpcClient,
    member_keypair: SecureKeypair,
    pubkey_str: String,
    program_id: Pubkey,
}

impl SquadsSigner {
    /// Create a new Squads signer
    pub fn new(
        multisig_address: &str,
        vault_index: u8,
        rpc_url: &str,
        member_keypair: SecureKeypair,
    ) -> Result<Self> {
        let multisig_pda = multisig_address
            .parse()
            .map_err(|_| Error::Squads("Invalid multisig address".into()))?;

        let program_id = SQUADS_PROGRAM_ID
            .parse()
            .map_err(|_| Error::Squads("Invalid program ID".into()))?;

        let rpc_client = RpcClient::new(rpc_url.to_string());

        Ok(Self {
            pubkey_str: multisig_address.to_string(),
            multisig_pda,
            vault_index,
            rpc_client,
            member_keypair,
            program_id,
        })
    }

    /// Get the multisig public key
    pub fn pubkey(&self) -> &str {
        &self.pubkey_str
    }

    /// Get the vault address
    pub fn vault_address(&self) -> Pubkey {
        get_vault_pda(&self.multisig_pda, self.vault_index, &self.program_id)
    }

    /// Create a proposal for a transaction
    pub async fn create_proposal(&self, transaction_message: &[u8]) -> Result<(Pubkey, u64)> {
        proposal::create_proposal(self, transaction_message).await
    }

    /// Approve a proposal
    pub async fn approve_proposal(&self, transaction_index: u64) -> Result<()> {
        proposal::approve_proposal(self, transaction_index).await
    }

    /// Execute a proposal
    pub async fn execute_proposal(&self, transaction_index: u64) -> Result<String> {
        execute::execute_proposal(self, transaction_index).await
    }

    /// Get the RPC client
    pub(crate) fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }

    /// Get the member keypair
    pub(crate) fn member_keypair(&self) -> &SecureKeypair {
        &self.member_keypair
    }

    /// Get the multisig PDA
    pub(crate) fn multisig_pda(&self) -> &Pubkey {
        &self.multisig_pda
    }

    /// Get the vault index
    pub(crate) fn vault_index(&self) -> u8 {
        self.vault_index
    }

    /// Get the program ID
    pub(crate) fn program_id(&self) -> &Pubkey {
        &self.program_id
    }
}

/// Member permissions (bitmask)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Can initiate proposals
    Initiate = 1,
    /// Can vote on proposals
    Vote = 2,
    /// Can execute proposals
    Execute = 4,
}

impl Permission {
    /// Check if a bitmask contains this permission
    pub fn has(self, mask: u8) -> bool {
        mask & (self as u8) != 0
    }
}
