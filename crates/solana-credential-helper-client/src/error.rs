//! Error types for the credential helper client.

use std::io;

/// Error type for credential helper client operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO error (file, socket, etc.)
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Base64 decoding error
    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// Agent returned an error
    #[error("Agent error: {0}")]
    Agent(String),

    /// CLI process error
    #[error("CLI error (exit code {code}): {message}")]
    Cli {
        /// Exit code from the CLI process
        code: i32,
        /// Error message from stderr
        message: String,
    },

    /// Invalid signature received
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Connection failed
    #[error("Connection failed: {0}")]
    Connection(String),
}

/// Result type alias for credential helper operations.
pub type Result<T> = std::result::Result<T, Error>;
