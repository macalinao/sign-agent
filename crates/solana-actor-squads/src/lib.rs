//! Squads multisig transport for Solana wallet operations.
//!
//! This crate provides [`SquadsTransport`], an implementation of the
//! [`WalletTransport`] trait that creates on-chain proposals for multisig
//! transactions using Squads Protocol v4.
//!
//! # Key Difference from Signers
//!
//! Unlike keypair or Ledger signers, Squads does NOT implement the signer
//! traits ([`MessageSigner`], [`TransactionSigner`]). Instead, it implements
//! [`WalletTransport`] because:
//!
//! - Multisig doesn't produce direct cryptographic signatures
//! - Transactions create on-chain proposals that need multiple approvals
//! - Execution happens on-chain when threshold is reached
//!
//! # Example
//!
//! ```ignore
//! use solana_actor_squads::SquadsTransport;
//! use solana_actor_keypair::KeypairSigner;
//! use solana_actor::{WalletTransport, SubmitResult};
//! use std::time::Duration;
//!
//! let member = KeypairSigner::from_file("member.json")?;
//! let transport = SquadsTransport::new(
//!     "MULTISIG_ADDRESS".parse()?,
//!     0, // vault_index
//!     "https://api.mainnet-beta.solana.com",
//!     member,
//! )?;
//!
//! // The vault PDA is the authority for transactions
//! let authority = transport.authority();
//!
//! // Submit creates a proposal and approves with the member key
//! let result = transport.submit(&tx_message).await?;
//!
//! match &result {
//!     SubmitResult::Executed { signature, .. } => {
//!         println!("Already executed: {}", signature);
//!     }
//!     SubmitResult::Pending { approvals, threshold, .. } => {
//!         println!("Pending: {}/{} approvals", approvals, threshold);
//!
//!         // Wait for other signers
//!         let final_result = transport
//!             .wait_for_completion(result, Duration::from_secs(300))
//!             .await?;
//!     }
//!     _ => {}
//! }
//! ```
//!
//! # Architecture
//!
//! The transport wraps any [`TransactionSigner`] as the member signer:
//!
//! ```ignore
//! // With a keypair
//! let transport = SquadsTransport::new(multisig, 0, url, KeypairSigner::generate())?;
//!
//! // With a Ledger
//! let transport = SquadsTransport::new(multisig, 0, url, LedgerSigner::connect()?)?;
//! ```

mod error;
mod instructions;
mod pda;
mod transport;

pub use error::{Result, SquadsError};
pub use pda::{get_proposal_pda, get_transaction_pda, get_vault_pda};
pub use transport::SquadsTransport;

// Re-export traits for convenience
pub use solana_actor::{SubmitResult, TransactionSigner, TransportError, WalletTransport};

/// Squads V4 program ID (mainnet).
pub const SQUADS_PROGRAM_ID: &str = "SQDS4nPHovALA9Sm5LCgJqkKhkYshJwKhN9kD3h8Zzg";
