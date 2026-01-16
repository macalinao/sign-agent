//! Signer traits for synchronous signing operations.
//!
//! This module defines two core signer traits:
//! - [`MessageSigner`] for off-chain message signing (SIWS, etc.)
//! - [`TransactionSigner`] for transaction message signing
//!
//! Both traits are synchronous and pure - they perform no network operations.
//! For async submission and network operations, see [`crate::transport::WalletTransport`].

use solana_sdk::{pubkey::Pubkey, signature::Signature};

use crate::error::SignerError;

/// Signs arbitrary messages (off-chain signing, SIWS, etc.).
///
/// This trait is for pure cryptographic signing of arbitrary byte sequences.
/// Use this for Sign-In-With-Solana, message authentication, and other
/// off-chain signing scenarios.
///
/// # Example
///
/// ```ignore
/// use solana_actor::MessageSigner;
///
/// fn sign_login_message<S: MessageSigner>(signer: &S, nonce: &str) -> Signature {
///     let message = format!("Sign in to MyApp: {}", nonce);
///     signer.sign_message(message.as_bytes()).expect("signing works")
/// }
/// ```
pub trait MessageSigner: Send + Sync {
    /// The public key of this signer.
    fn pubkey(&self) -> Pubkey;

    /// Sign an arbitrary message.
    ///
    /// # Arguments
    ///
    /// * `message` - The raw bytes to sign.
    ///
    /// # Errors
    ///
    /// Returns [`SignerError`] if signing fails (device error, user cancelled, etc.).
    fn sign_message(&self, message: &[u8]) -> Result<Signature, SignerError>;
}

/// Signs serialized transaction messages.
///
/// This trait is for signing Solana transaction messages. The input should be
/// the serialized transaction message bytes (from `Transaction::message().serialize()`).
///
/// # Example
///
/// ```ignore
/// use solana_actor::TransactionSigner;
/// use solana_sdk::transaction::Transaction;
///
/// fn sign_tx<S: TransactionSigner>(signer: &S, tx: &mut Transaction) {
///     let message_bytes = tx.message.serialize();
///     let sig = signer.sign_transaction(&message_bytes).expect("signing works");
///     tx.signatures[0] = sig;
/// }
/// ```
pub trait TransactionSigner: Send + Sync {
    /// The public key of this signer.
    fn pubkey(&self) -> Pubkey;

    /// Sign a serialized transaction message.
    ///
    /// # Arguments
    ///
    /// * `message` - The serialized transaction message bytes.
    ///
    /// # Errors
    ///
    /// Returns [`SignerError`] if signing fails.
    fn sign_transaction(&self, message: &[u8]) -> Result<Signature, SignerError>;

    /// Whether signing requires user interaction.
    ///
    /// Returns `true` for hardware wallets like Ledger that require
    /// physical button presses. Returns `false` for software signers.
    ///
    /// This can be used to show appropriate UI prompts before signing.
    fn is_interactive(&self) -> bool {
        false
    }
}
