//! Solana Keyring - Secure key management library for Solana
//!
//! This library provides encrypted storage for Solana keypairs with support for:
//! - Local keypairs with row-level AES-256-GCM encryption
//! - Ledger hardware wallet integration
//! - Squads multisig support
//! - Address book with labels
//! - Biometric authentication (TouchID on macOS)
//! - Transaction parsing and user confirmation

// Allow missing error/panic docs for internal library - errors are clear from context
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub mod biometric;
pub mod crypto;
pub mod db;
pub mod keypair;
pub mod ledger;
pub mod squads;
pub mod transaction;

mod address_book;
mod error;
mod notification;
mod signer;

pub use address_book::AddressBook;
pub use db::Database;
pub use error::{Error, Result};
pub use keypair::SecureKeypair;
pub use notification::notify;
pub use signer::{
    KeypairSigner, LedgerSignerWrapper, Signer, SignerInfo, SignerType, list_signers,
};

use std::path::PathBuf;

/// Default keyring directory
pub fn default_keyring_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".solana-keyring")
}

/// Default database path
pub fn default_db_path() -> PathBuf {
    default_keyring_dir().join("keyring.db")
}

/// Default agent socket path
pub fn default_agent_socket_path() -> PathBuf {
    default_keyring_dir().join("agent.sock")
}
