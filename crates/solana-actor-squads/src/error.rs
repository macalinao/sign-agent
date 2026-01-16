//! Error types for Squads operations.

use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

/// Errors that can occur during Squads operations.
#[derive(Error, Debug)]
pub enum SquadsError {
    /// Invalid multisig address.
    #[error("Invalid multisig address: {0}")]
    InvalidAddress(String),

    /// Multisig not found on-chain.
    #[error("Multisig not found: {0}")]
    MultisigNotFound(Pubkey),

    /// Invalid account data.
    #[error("Invalid account data: {0}")]
    InvalidAccountData(String),

    /// RPC error.
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Proposal creation failed.
    #[error("Failed to create proposal: {0}")]
    ProposalCreation(String),

    /// Approval failed.
    #[error("Failed to approve proposal: {0}")]
    Approval(String),

    /// Execution failed.
    #[error("Failed to execute proposal: {0}")]
    Execution(String),

    /// Proposal not found.
    #[error("Proposal not found: {0}")]
    ProposalNotFound(Pubkey),

    /// Insufficient approvals.
    #[error("Insufficient approvals: {current}/{required}")]
    InsufficientApprovals {
        /// Current number of approvals.
        current: u32,
        /// Required number of approvals.
        required: u32,
    },

    /// Signer error.
    #[error("Signer error: {0}")]
    Signer(#[from] solana_actor::SignerError),
}

/// Result type for Squads operations.
pub type Result<T> = std::result::Result<T, SquadsError>;

impl From<SquadsError> for solana_actor::TransportError {
    fn from(err: SquadsError) -> Self {
        match err {
            SquadsError::MultisigNotFound(pk) => Self::MultisigNotFound(pk),
            SquadsError::InsufficientApprovals { current, required } => {
                Self::InsufficientApprovals { current, required }
            }
            SquadsError::ProposalCreation(msg) => Self::ProposalFailed(msg),
            SquadsError::Approval(msg) => Self::ApprovalFailed(msg),
            SquadsError::Execution(msg) => Self::ExecutionFailed(msg),
            SquadsError::Rpc(msg) => Self::Connection(solana_actor::ConnectionError::Rpc(msg)),
            SquadsError::Signer(e) => Self::Signer(e),
            SquadsError::InvalidAddress(msg) => Self::ProposalFailed(msg),
            SquadsError::InvalidAccountData(msg) => Self::ProposalFailed(msg),
            SquadsError::ProposalNotFound(pk) => Self::ProposalFailed(format!("Not found: {}", pk)),
        }
    }
}
