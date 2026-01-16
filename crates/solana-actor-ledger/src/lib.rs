//! Ledger hardware wallet signer for Solana.
//!
//! This crate provides [`LedgerSigner`], an implementation of the
//! [`MessageSigner`] and [`TransactionSigner`] traits that communicates
//! with Ledger hardware wallets.
//!
//! # Features
//!
//! - **Hardware security** - Private keys never leave the Ledger device
//! - **User confirmation** - All signing requires physical button press
//! - **BIP-44 paths** - Standard derivation path support
//! - **Trait implementations** - Implements `MessageSigner` and `TransactionSigner`
//!
//! # Requirements
//!
//! - Ledger Nano S/X/S Plus with Solana app installed
//! - Solana app must be opened on the device
//! - USB connection to the device
//!
//! # Example
//!
//! ```ignore
//! use solana_actor_ledger::LedgerSigner;
//! use solana_actor::TransactionSigner;
//!
//! // Connect with default derivation path (44'/501'/0'/0')
//! let signer = LedgerSigner::connect()?;
//! println!("Ledger pubkey: {}", signer.pubkey_base58());
//!
//! // Sign a transaction (user must confirm on device)
//! let signature = signer.sign_transaction(&tx_message)?;
//! ```
//!
//! # Custom Derivation Path
//!
//! ```ignore
//! use solana_actor_ledger::LedgerSigner;
//!
//! // Use a different account index
//! let signer = LedgerSigner::connect_with_path("44'/501'/1'/0'")?;
//! ```
//!
//! # Using with Transports
//!
//! ```ignore
//! use solana_actor_ledger::LedgerSigner;
//! use solana_actor::{DirectTransport, WalletTransport};
//!
//! let signer = LedgerSigner::connect()?;
//! let transport = DirectTransport::new(signer);
//!
//! // The transport will use spawn_blocking for the signing operation
//! let result = transport.submit(&tx_message).await?;
//! ```

mod derivation;
mod error;
mod signer;
mod transport;

pub use derivation::{DEFAULT_PATH, format_path, parse_path};
pub use error::{LedgerError, Result};
pub use signer::LedgerSigner;

// Re-export traits for convenience
pub use solana_actor::{MessageSigner, SignerError, TransactionSigner};
