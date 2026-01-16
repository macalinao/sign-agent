//! Error types for keypair operations.

use thiserror::Error;

/// Errors that can occur during keypair operations.
#[derive(Error, Debug)]
pub enum KeypairError {
    /// Invalid key format or data.
    #[error("Invalid key format: {0}")]
    InvalidFormat(String),

    /// File not found.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Base58 decoding error.
    #[error("Base58 decode error: {0}")]
    Base58(#[from] bs58::decode::Error),
}

/// Result type for keypair operations.
pub type Result<T> = std::result::Result<T, KeypairError>;

impl From<KeypairError> for solana_actor::SignerError {
    fn from(err: KeypairError) -> Self {
        match err {
            KeypairError::InvalidFormat(msg) => Self::InvalidKey(msg),
            KeypairError::FileNotFound(path) => Self::FileNotFound(path),
            KeypairError::Io(e) => Self::Io(e),
            KeypairError::Json(e) => Self::InvalidFormat(e.to_string()),
            KeypairError::Base58(e) => Self::InvalidKey(e.to_string()),
        }
    }
}
