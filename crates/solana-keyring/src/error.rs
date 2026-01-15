//! Error types for solana-keyring

use thiserror::Error;

/// Result type for solana-keyring operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in solana-keyring
#[derive(Debug, Error)]
pub enum Error {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Encryption/decryption error
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Key derivation error
    #[error("Key derivation error: {0}")]
    KeyDerivation(String),

    /// Invalid passphrase
    #[error("Invalid passphrase")]
    InvalidPassphrase,

    /// Keyring not initialized
    #[error("Keyring not initialized. Run 'solana-keyring new' first")]
    NotInitialized,

    /// Keyring already exists
    #[error("Keyring already exists at {0}")]
    AlreadyExists(String),

    /// Keypair not found
    #[error("Keypair not found: {0}")]
    KeypairNotFound(String),

    /// Address not found
    #[error("Address not found: {0}")]
    AddressNotFound(String),

    /// Invalid keypair format
    #[error("Invalid keypair format: {0}")]
    InvalidKeypairFormat(String),

    /// Ledger error
    #[error("Ledger error: {0}")]
    Ledger(String),

    /// Ledger not connected
    #[error("Ledger device not connected")]
    LedgerNotConnected,

    /// Squads error
    #[error("Squads error: {0}")]
    Squads(String),

    /// Biometric authentication error
    #[error("Biometric error: {0}")]
    Biometric(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Solana SDK error
    #[error("Solana error: {0}")]
    Solana(String),

    /// Base58 decode error
    #[error("Base58 decode error: {0}")]
    Base58(#[from] bs58::decode::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<aes_gcm::Error> for Error {
    fn from(e: aes_gcm::Error) -> Self {
        Error::Encryption(e.to_string())
    }
}

impl From<argon2::Error> for Error {
    fn from(e: argon2::Error) -> Self {
        Error::KeyDerivation(e.to_string())
    }
}

impl From<ed25519_dalek::SignatureError> for Error {
    fn from(e: ed25519_dalek::SignatureError) -> Self {
        Error::InvalidKeypairFormat(e.to_string())
    }
}
