//! Error types for wallet operations.

use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

/// Errors from synchronous signing operations.
#[derive(Error, Debug)]
pub enum SignerError {
    /// Invalid key format or data.
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Signing operation failed.
    #[error("Signing failed: {0}")]
    SigningFailed(String),

    /// Hardware device communication error.
    #[error("Hardware device error: {0}")]
    DeviceError(String),

    /// Hardware device not found.
    #[error("Device not found")]
    DeviceNotFound,

    /// User cancelled the signing operation.
    #[error("User cancelled signing")]
    UserCancelled,

    /// File not found.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Invalid file format.
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors from async transport operations.
#[derive(Error, Debug)]
pub enum TransportError {
    /// Signing error.
    #[error("Signing error: {0}")]
    Signer(#[from] SignerError),

    /// Connection error.
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    /// Proposal creation failed.
    #[error("Proposal creation failed: {0}")]
    ProposalFailed(String),

    /// Approval failed.
    #[error("Approval failed: {0}")]
    ApprovalFailed(String),

    /// Execution failed.
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Timeout waiting for completion.
    #[error("Timeout waiting for completion")]
    Timeout,

    /// Task panicked during execution.
    #[error("Task panicked")]
    TaskPanic,

    /// Multisig account not found.
    #[error("Multisig not found: {0}")]
    MultisigNotFound(Pubkey),

    /// Insufficient approvals to execute.
    #[error("Insufficient approvals: {current}/{required}")]
    InsufficientApprovals {
        /// Current number of approvals.
        current: u32,
        /// Required number of approvals.
        required: u32,
    },
}

/// Errors from network connection operations.
#[derive(Error, Debug)]
pub enum ConnectionError {
    /// RPC error.
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Transaction failed.
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    /// Blockhash expired.
    #[error("Blockhash expired")]
    BlockhashExpired,

    /// Insufficient funds.
    #[error("Insufficient funds")]
    InsufficientFunds,

    /// Network unreachable.
    #[error("Network unreachable")]
    NetworkUnreachable,

    /// Rate limited.
    #[error("Rate limited")]
    RateLimited,

    /// Timeout.
    #[error("Timeout")]
    Timeout,
}
