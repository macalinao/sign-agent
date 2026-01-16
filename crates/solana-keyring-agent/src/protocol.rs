//! Agent protocol messages

use serde::{Deserialize, Serialize};

/// Request message from client to agent
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum Request {
    /// Ping to check if agent is alive
    Ping,

    /// Request list of available signers
    ListSigners {
        /// Optional tag filter
        tag: Option<String>,
    },

    /// Sign a transaction
    SignTransaction {
        /// Base64 encoded transaction message
        transaction: String,
        /// Public key of signer to use
        signer: String,
    },

    /// Generate a new keypair and store it
    GenerateKeypair {
        /// Label for the new keypair
        label: String,
        /// Tags to add to the keypair
        tags: Vec<String>,
    },

    /// Import a keypair from base58 secret key
    ImportKeypair {
        /// Label for the keypair
        label: String,
        /// Base58 encoded secret key
        secret_key: String,
        /// Tags to add to the keypair
        tags: Vec<String>,
    },

    /// Unlock the keyring (provide master passphrase)
    Unlock {
        /// Master passphrase
        passphrase: String,
    },

    /// Lock the keyring (clear passphrase from memory)
    Lock,

    /// Get agent status
    Status,

    /// Shutdown the agent
    Shutdown,
}

/// Response message from agent to client
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum Response {
    #[serde(rename = "ok")]
    Ok { result: ResponseResult },

    #[serde(rename = "error")]
    Error { code: ErrorCode, message: String },
}

impl Response {
    pub fn ok(result: ResponseResult) -> Self {
        Response::Ok { result }
    }

    pub fn error(code: ErrorCode, message: impl Into<String>) -> Self {
        Response::Error {
            code,
            message: message.into(),
        }
    }
}

/// Response result variants
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseResult {
    Pong,
    Signers(Vec<SignerInfo>),
    SignedTransaction(String), // Base64 encoded signed transaction
    GeneratedKeypair(GeneratedKeypairInfo),
    Status(AgentStatus),
    Unit,
}

/// Generated keypair information
#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedKeypairInfo {
    pub pubkey: String,
    pub label: String,
}

/// Signer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerInfo {
    pub pubkey: String,
    pub label: String,
    pub signer_type: String,
    pub tags: Vec<String>,
}

/// Agent status information
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentStatus {
    pub unlocked: bool,
    pub uptime_seconds: u64,
    pub signer_count: usize,
    pub lock_timeout_seconds: u64,
}

/// Error codes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ErrorCode {
    Locked,
    InvalidPassphrase,
    SignerNotFound,
    InvalidTransaction,
    HardwareError,
    InternalError,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::Locked => write!(f, "LOCKED"),
            ErrorCode::InvalidPassphrase => write!(f, "INVALID_PASSPHRASE"),
            ErrorCode::SignerNotFound => write!(f, "SIGNER_NOT_FOUND"),
            ErrorCode::InvalidTransaction => write!(f, "INVALID_TRANSACTION"),
            ErrorCode::HardwareError => write!(f, "HARDWARE_ERROR"),
            ErrorCode::InternalError => write!(f, "INTERNAL_ERROR"),
        }
    }
}
