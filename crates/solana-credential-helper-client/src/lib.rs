//! Client library for interacting with the Solana Credential Helper.
//!
//! This library provides a Rust API for signing Solana transactions using
//! the credential helper infrastructure, supporting:
//!
//! - Local encrypted keypairs
//! - Ledger hardware wallets
//! - Squads multisig
//!
//! # Example
//!
//! ```no_run
//! use solana_credential_helper_client::{
//!     CredentialHelperClient,
//!     CredentialHelperConfig,
//!     SignerType,
//! };
//!
//! # async fn example() -> solana_credential_helper_client::Result<()> {
//! // Create a client configured to use the agent daemon
//! let config = CredentialHelperConfig::new("ABC123...")
//!     .signer_type(SignerType::Keypair)
//!     .use_agent(true);
//!
//! let client = CredentialHelperClient::new(config);
//!
//! // Sign a transaction message
//! let message_bytes: Vec<u8> = vec![]; // Your serialized message
//! let signature = client.sign_transaction(&message_bytes).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Signing Methods
//!
//! The client supports two methods for signing:
//!
//! 1. **Agent Socket** - Connects to the `solana-keyring-agent` daemon via Unix socket.
//!    This is faster for multiple signatures as the keyring stays unlocked.
//!
//! 2. **CLI Subprocess** - Spawns `solana-credential-helper sign-transaction` for each
//!    signature. More portable but requires password entry each time (unless agent is running).
//!
//! Use [`CredentialHelperConfig::use_agent`] to choose which method to use.

mod client;
mod error;
mod types;

pub use client::{CredentialHelperClient, default_db_path, default_socket_path};
pub use error::{Error, Result};
pub use types::{CredentialHelperConfig, SignerType};
