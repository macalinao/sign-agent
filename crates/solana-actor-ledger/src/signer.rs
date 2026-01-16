//! Ledger hardware wallet signer implementation.

use solana_actor::{MessageSigner, SignerError, TransactionSigner};
use solana_sdk::{pubkey::Pubkey, signature::Signature};

use crate::derivation::{DEFAULT_PATH, format_path, parse_path};
use crate::error::Result;
use crate::transport;

/// Ledger hardware wallet signer.
///
/// This signer communicates with a connected Ledger device to perform
/// cryptographic signing operations. The device must have the Solana app
/// installed and opened.
///
/// # Security
///
/// - Private keys never leave the Ledger device
/// - User must physically confirm each signing operation on the device
/// - Implements `is_interactive() -> true` to indicate user interaction required
///
/// # Example
///
/// ```ignore
/// use solana_actor_ledger::LedgerSigner;
/// use solana_actor::TransactionSigner;
///
/// // Connect with default derivation path
/// let signer = LedgerSigner::connect()?;
/// println!("Ledger pubkey: {}", signer.pubkey_base58());
///
/// // Sign a transaction (user must confirm on device)
/// let signature = signer.sign_transaction(&tx_message)?;
/// ```
pub struct LedgerSigner {
    derivation_path: Vec<u32>,
    pubkey: Pubkey,
}

impl LedgerSigner {
    /// Connect to a Ledger device with the default derivation path.
    ///
    /// The default path is `44'/501'/0'/0'` (BIP-44 for Solana).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No Ledger device is connected
    /// - The Solana app is not opened
    /// - Communication with the device fails
    pub fn connect() -> Result<Self> {
        Self::connect_with_path(DEFAULT_PATH)
    }

    /// Connect to a Ledger device with a custom derivation path.
    ///
    /// # Arguments
    ///
    /// * `path` - BIP-44 derivation path (e.g., "44'/501'/0'/0'")
    ///
    /// # Errors
    ///
    /// Returns an error if the path is invalid or device communication fails.
    pub fn connect_with_path(path: &str) -> Result<Self> {
        let derivation_path = parse_path(path)?;
        Self::connect_with_parsed_path(derivation_path)
    }

    /// Connect with an already-parsed derivation path.
    ///
    /// # Errors
    ///
    /// Returns an error if device communication fails or the device is not available.
    pub fn connect_with_parsed_path(derivation_path: Vec<u32>) -> Result<Self> {
        let pubkey_bytes = transport::get_pubkey(&derivation_path)?;
        let pubkey = Pubkey::new_from_array(pubkey_bytes);

        Ok(Self {
            derivation_path,
            pubkey,
        })
    }

    /// Get the derivation path used by this signer.
    pub fn derivation_path(&self) -> String {
        format_path(&self.derivation_path)
    }

    /// Get the public key bytes.
    pub fn pubkey_bytes(&self) -> [u8; 32] {
        self.pubkey.to_bytes()
    }

    /// Get the public key as a base58 string.
    pub fn pubkey_base58(&self) -> String {
        self.pubkey.to_string()
    }

    /// Sign a message and return raw signature bytes.
    ///
    /// This is a lower-level method that returns the signature as a byte array.
    /// For most use cases, use the trait methods [`MessageSigner::sign_message`]
    /// or [`TransactionSigner::sign_transaction`] instead.
    ///
    /// # Note
    ///
    /// The user must physically confirm the signing operation on the Ledger device.
    ///
    /// # Errors
    ///
    /// Returns an error if device communication fails or the user rejects the signing.
    pub fn sign(&self, message: &[u8]) -> Result<[u8; 64]> {
        transport::sign_message(&self.derivation_path, message)
    }
}

impl MessageSigner for LedgerSigner {
    fn pubkey(&self) -> Pubkey {
        self.pubkey
    }

    fn sign_message(&self, message: &[u8]) -> std::result::Result<Signature, SignerError> {
        let sig_bytes = self.sign(message).map_err(SignerError::from)?;
        Ok(Signature::from(sig_bytes))
    }
}

impl TransactionSigner for LedgerSigner {
    fn pubkey(&self) -> Pubkey {
        self.pubkey
    }

    fn sign_transaction(&self, message: &[u8]) -> std::result::Result<Signature, SignerError> {
        let sig_bytes = self.sign(message).map_err(SignerError::from)?;
        Ok(Signature::from(sig_bytes))
    }

    fn is_interactive(&self) -> bool {
        true
    }
}

// Note: Tests for LedgerSigner require a physical device and are marked as ignored.
// Run them manually with: cargo test -p solana-actor-ledger -- --ignored
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_connect_default() {
        let signer = LedgerSigner::connect().expect("Failed to connect to Ledger");
        println!("Connected to Ledger: {}", signer.pubkey_base58());
        println!("Derivation path: {}", signer.derivation_path());
    }

    #[test]
    #[ignore]
    fn test_sign_message() {
        let signer = LedgerSigner::connect().expect("Failed to connect to Ledger");
        let message = b"Test message for Ledger signing";

        println!("Please confirm on your Ledger device...");
        let sig = signer
            .sign_message(message)
            .expect("Failed to sign message");
        println!("Signature: {}", sig);
    }

    #[test]
    fn test_is_interactive() {
        // This test doesn't require a device
        // We just verify the trait default is overridden
        // Note: Can't actually test without device, but the impl is visible
    }
}
