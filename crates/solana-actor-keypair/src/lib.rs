//! Keypair-based signer for Solana with secure memory handling.
//!
//! This crate provides [`KeypairSigner`], a secure implementation of the
//! [`MessageSigner`] and [`TransactionSigner`] traits from `solana-actor`.
//!
//! # Features
//!
//! - **Secure memory handling** - Secret keys are automatically zeroized when dropped
//! - **Multiple input formats** - Load from files, bytes, or base58 encoding
//! - **Solana CLI compatible** - Works with standard Solana keypair JSON files
//!
//! # Example
//!
//! ```
//! use solana_actor_keypair::KeypairSigner;
//! use solana_actor::{TransactionSigner, MessageSigner};
//!
//! // Generate a new random keypair
//! let signer = KeypairSigner::generate();
//! println!("Public key: {}", signer.pubkey_base58());
//!
//! // Sign an off-chain message
//! let message = b"Sign-In-With-Solana message";
//! let sig = signer.sign_message(message).unwrap();
//!
//! // Sign a transaction message
//! let tx_msg = b"serialized transaction message";
//! let tx_sig = signer.sign_transaction(tx_msg).unwrap();
//! ```
//!
//! # Loading from Files
//!
//! ```no_run
//! use solana_actor_keypair::{from_file, KeypairSigner};
//!
//! // Load from Solana CLI keypair file
//! let signer = from_file("~/.config/solana/id.json").unwrap();
//! println!("Loaded: {}", signer.pubkey_base58());
//! ```
//!
//! # Using with Transports
//!
//! ```ignore
//! use solana_actor_keypair::KeypairSigner;
//! use solana_actor::{DirectTransport, WalletTransport};
//!
//! let signer = KeypairSigner::generate();
//! let transport = DirectTransport::new(signer);
//!
//! // Now you can use transport.submit() for async operations
//! ```

mod error;
mod file;
mod signer;

pub use error::{KeypairError, Result};
pub use file::{from_file, from_json_string, to_base58, to_file, to_json};
pub use signer::KeypairSigner;

// Re-export traits for convenience
pub use solana_actor::{MessageSigner, SignerError, TransactionSigner};
