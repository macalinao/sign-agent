//! Types for the credential helper client.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Type of signer to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignerType {
    /// Local encrypted keypair
    #[default]
    Keypair,
    /// Ledger hardware wallet
    Ledger,
    /// Squads multisig
    Squads,
}

impl std::fmt::Display for SignerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignerType::Keypair => write!(f, "keypair"),
            SignerType::Ledger => write!(f, "ledger"),
            SignerType::Squads => write!(f, "squads"),
        }
    }
}

/// Configuration for the credential helper client.
#[derive(Debug, Clone, Default)]
pub struct CredentialHelperConfig {
    /// Public key of the signer (base58 encoded).
    pub public_key: String,

    /// Type of signer (default: Keypair).
    pub signer_type: SignerType,

    /// For Squads: the multisig address.
    pub squads_address: Option<String>,

    /// RPC URL for Squads operations.
    pub rpc_url: Option<String>,

    /// Path to credential helper binary (default: "solana-credential-helper").
    pub binary_path: Option<PathBuf>,

    /// Use agent socket if available.
    pub use_agent: bool,

    /// Agent socket path (default: ~/.solana-keyring/agent.sock).
    pub agent_socket_path: Option<PathBuf>,

    /// Database path (default: ~/.solana-keyring/keyring.db).
    pub db_path: Option<PathBuf>,
}

impl CredentialHelperConfig {
    /// Create a new configuration with the given public key.
    pub fn new(public_key: impl Into<String>) -> Self {
        Self {
            public_key: public_key.into(),
            ..Default::default()
        }
    }

    /// Set the signer type.
    pub fn signer_type(mut self, signer_type: SignerType) -> Self {
        self.signer_type = signer_type;
        self
    }

    /// Set the Squads multisig address.
    pub fn squads_address(mut self, address: impl Into<String>) -> Self {
        self.squads_address = Some(address.into());
        self
    }

    /// Set the RPC URL.
    pub fn rpc_url(mut self, url: impl Into<String>) -> Self {
        self.rpc_url = Some(url.into());
        self
    }

    /// Set the path to the credential helper binary.
    pub fn binary_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary_path = Some(path.into());
        self
    }

    /// Enable or disable agent usage.
    pub fn use_agent(mut self, use_agent: bool) -> Self {
        self.use_agent = use_agent;
        self
    }

    /// Set the agent socket path.
    pub fn agent_socket_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.agent_socket_path = Some(path.into());
        self
    }

    /// Set the database path.
    pub fn db_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.db_path = Some(path.into());
        self
    }
}
