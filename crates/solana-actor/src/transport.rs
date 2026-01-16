//! Transport traits for async transaction submission.
//!
//! This module defines the [`WalletTransport`] trait for submitting transactions
//! and tracking their status. Unlike the synchronous signer traits, transports
//! handle async operations and may interact with the network.
//!
//! # Result Types
//!
//! The [`SubmitResult`] enum captures all possible outcomes:
//! - [`SubmitResult::Signed`] - Direct signature produced (from regular signers)
//! - [`SubmitResult::Pending`] - Multisig proposal awaiting additional approvals
//! - [`SubmitResult::Executed`] - Multisig proposal was executed on-chain

use std::time::Duration;

use async_trait::async_trait;
use solana_sdk::{pubkey::Pubkey, signature::Signature};

use crate::error::TransportError;

/// Result of submitting a transaction via a transport.
#[derive(Debug, Clone)]
pub enum SubmitResult {
    /// Direct cryptographic signature produced.
    ///
    /// This is returned by transports wrapping regular signers (keypair, Ledger).
    /// The transaction can be submitted to the network using this signature.
    Signed(Signature),

    /// Submitted to multisig, awaiting additional approvals.
    ///
    /// This is returned by multisig transports (e.g., Squads) when the
    /// threshold hasn't been met yet.
    Pending {
        /// The proposal account public key.
        proposal: Pubkey,
        /// The transaction index within the multisig.
        transaction_index: u64,
        /// Current number of approvals.
        approvals: u32,
        /// Required number of approvals (threshold).
        threshold: u32,
    },

    /// Multisig proposal was executed on-chain.
    ///
    /// This is returned when a multisig transaction reaches threshold
    /// and is executed, producing an on-chain signature.
    Executed {
        /// The transaction signature from execution.
        signature: Signature,
        /// The proposal account that was executed.
        proposal: Pubkey,
    },
}

impl SubmitResult {
    /// Get the signature if available.
    ///
    /// Returns `Some(&Signature)` for `Signed` and `Executed` variants,
    /// `None` for `Pending`.
    pub fn signature(&self) -> Option<&Signature> {
        match self {
            Self::Signed(sig) => Some(sig),
            Self::Executed { signature, .. } => Some(signature),
            Self::Pending { .. } => None,
        }
    }

    /// Whether this result represents a completed transaction.
    ///
    /// Returns `true` for `Signed` and `Executed`, `false` for `Pending`.
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Signed(_) | Self::Executed { .. })
    }

    /// Whether this result is pending additional approvals.
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    /// Get the proposal pubkey if this is a multisig result.
    pub fn proposal(&self) -> Option<&Pubkey> {
        match self {
            Self::Pending { proposal, .. } | Self::Executed { proposal, .. } => Some(proposal),
            Self::Signed(_) => None,
        }
    }
}

/// Async transport for submitting transactions.
///
/// This trait abstracts over different transaction submission methods:
/// - Direct signing with immediate signature return
/// - Multisig proposal creation with deferred execution
///
/// # Example
///
/// ```ignore
/// use solana_actor::{WalletTransport, SubmitResult};
/// use std::time::Duration;
///
/// async fn submit_tx<T: WalletTransport>(
///     transport: &T,
///     message: &[u8],
/// ) -> Result<Signature, TransportError> {
///     let result = transport.submit(message).await?;
///
///     match result {
///         SubmitResult::Signed(sig) => Ok(sig),
///         SubmitResult::Executed { signature, .. } => Ok(signature),
///         SubmitResult::Pending { .. } => {
///             // Wait for other signers
///             let final_result = transport
///                 .wait_for_completion(result, Duration::from_secs(300))
///                 .await?;
///             Ok(final_result.signature().unwrap().clone())
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait WalletTransport: Send + Sync {
    /// The authority pubkey (signer key or vault PDA).
    ///
    /// For direct signers, this is the signer's public key.
    /// For multisig transports, this is typically the vault PDA.
    fn authority(&self) -> Pubkey;

    /// Submit a transaction for signing/execution.
    ///
    /// # Arguments
    ///
    /// * `message` - The serialized transaction message bytes.
    ///
    /// # Returns
    ///
    /// - `SubmitResult::Signed` for direct signers
    /// - `SubmitResult::Pending` or `SubmitResult::Executed` for multisig
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if submission fails.
    async fn submit(&self, message: &[u8]) -> Result<SubmitResult, TransportError>;

    /// Check the current status of a previous submission.
    ///
    /// For direct signers, this simply returns the same result (always complete).
    /// For multisig, this queries the on-chain state to get updated approval counts.
    ///
    /// # Arguments
    ///
    /// * `result` - A previous [`SubmitResult`] to check.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] if the status check fails.
    async fn check_status(&self, result: &SubmitResult) -> Result<SubmitResult, TransportError>;

    /// Wait for a pending submission to complete.
    ///
    /// Polls the status until the transaction is complete or the timeout is reached.
    /// For direct signers, this returns immediately since signatures are always complete.
    ///
    /// # Arguments
    ///
    /// * `result` - The [`SubmitResult`] to wait on.
    /// * `timeout` - Maximum time to wait.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Timeout`] if the timeout is exceeded.
    async fn wait_for_completion(
        &self,
        result: SubmitResult,
        timeout: Duration,
    ) -> Result<SubmitResult, TransportError>;

    /// Whether this transport requires network access.
    ///
    /// Returns `true` for multisig transports that need to create on-chain proposals.
    /// Returns `false` for direct signers that only perform local cryptographic operations.
    fn requires_network(&self) -> bool;
}
