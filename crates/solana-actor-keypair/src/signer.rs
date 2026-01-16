//! Keypair-based signer implementation.

use ed25519_dalek::SigningKey;
use solana_actor::{MessageSigner, SignerError, TransactionSigner};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::error::{KeypairError, Result};

/// A keypair-based signer with secure memory handling.
///
/// This signer holds an Ed25519 signing key in memory and automatically
/// zeroizes it when dropped. It implements both [`MessageSigner`] and
/// [`TransactionSigner`] traits for signing arbitrary messages and
/// transaction messages.
///
/// # Security
///
/// - Secret key is automatically zeroized when the signer is dropped
/// - Uses `ed25519-dalek` for cryptographic operations
/// - Supports loading from files, bytes, or base58 encoding
///
/// # Example
///
/// ```
/// use solana_actor_keypair::KeypairSigner;
/// use solana_actor::TransactionSigner;
///
/// // Generate a new random keypair
/// let signer = KeypairSigner::generate();
/// println!("Public key: {}", signer.pubkey_base58());
///
/// // Sign a message
/// let message = b"hello world";
/// let signature = signer.sign_transaction(message).unwrap();
/// ```
#[derive(ZeroizeOnDrop)]
pub struct KeypairSigner {
    #[zeroize(skip)]
    pubkey: Pubkey,
    secret: SigningKey,
}

impl KeypairSigner {
    /// Generate a new random keypair.
    ///
    /// # Example
    ///
    /// ```
    /// use solana_actor_keypair::KeypairSigner;
    ///
    /// let signer = KeypairSigner::generate();
    /// println!("Generated key: {}", signer.pubkey_base58());
    /// ```
    pub fn generate() -> Self {
        let secret = SigningKey::generate(&mut rand::thread_rng());
        let pubkey = Pubkey::new_from_array(secret.verifying_key().to_bytes());
        Self { pubkey, secret }
    }

    /// Create a signer from raw 32-byte secret key.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The 32-byte Ed25519 secret key.
    ///
    /// # Errors
    ///
    /// This function cannot fail for valid 32-byte arrays.
    ///
    /// # Example
    ///
    /// ```
    /// use solana_actor_keypair::KeypairSigner;
    ///
    /// let secret = [0u8; 32]; // Use real secret bytes
    /// let signer = KeypairSigner::from_bytes(&secret).unwrap();
    /// ```
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let secret = SigningKey::from_bytes(bytes);
        let pubkey = Pubkey::new_from_array(secret.verifying_key().to_bytes());
        Ok(Self { pubkey, secret })
    }

    /// Create a signer from a base58-encoded keypair.
    ///
    /// Accepts either:
    /// - 64 bytes (32-byte secret + 32-byte public key)
    /// - 32 bytes (secret key only)
    ///
    /// # Arguments
    ///
    /// * `encoded` - Base58-encoded keypair string.
    ///
    /// # Errors
    ///
    /// Returns [`KeypairError::InvalidFormat`] if the format is invalid.
    pub fn from_base58(encoded: &str) -> Result<Self> {
        let mut bytes = bs58::decode(encoded).into_vec()?;

        let result = if bytes.len() == 64 {
            // Full keypair format (secret + public)
            Self::from_bytes(
                bytes[..32]
                    .try_into()
                    .map_err(|_| KeypairError::InvalidFormat("Invalid key size".into()))?,
            )
        } else if bytes.len() == 32 {
            // Just the secret key
            Self::from_bytes(
                bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| KeypairError::InvalidFormat("Invalid key size".into()))?,
            )
        } else {
            bytes.zeroize();
            return Err(KeypairError::InvalidFormat(format!(
                "Expected 32 or 64 bytes, got {}",
                bytes.len()
            )));
        };

        bytes.zeroize();
        result
    }

    /// Get the raw 32-byte secret key.
    ///
    /// The returned [`Zeroizing`] wrapper will automatically zeroize
    /// the bytes when dropped.
    ///
    /// # Security
    ///
    /// Handle the returned bytes carefully and ensure they are zeroized
    /// when no longer needed.
    pub fn secret_bytes(&self) -> Zeroizing<[u8; 32]> {
        Zeroizing::new(self.secret.to_bytes())
    }

    /// Get the public key as raw bytes.
    pub fn pubkey_bytes(&self) -> [u8; 32] {
        self.pubkey.to_bytes()
    }

    /// Get the public key as a base58 string.
    pub fn pubkey_base58(&self) -> String {
        self.pubkey.to_string()
    }

    /// Convert to a Solana SDK Keypair.
    ///
    /// This creates a new Solana SDK [`Keypair`] instance from this signer.
    /// Note that the returned keypair is a separate copy and modifications
    /// to it will not affect this signer.
    ///
    /// [`Keypair`]: solana_sdk::signer::keypair::Keypair
    ///
    /// # Panics
    ///
    /// Panics if the internal keypair bytes are invalid (should never happen).
    pub fn to_solana_keypair(&self) -> solana_sdk::signer::keypair::Keypair {
        let secret = self.secret_bytes();
        let mut full_key = [0u8; 64];
        full_key[..32].copy_from_slice(&secret[..]);
        full_key[32..].copy_from_slice(&self.pubkey_bytes());

        let result = solana_sdk::signer::keypair::Keypair::try_from(full_key.as_slice())
            .expect("Valid keypair bytes");

        full_key.zeroize();
        result
    }

    /// Sign a message and return raw signature bytes.
    ///
    /// This is a lower-level method that returns the signature as a byte array.
    /// For most use cases, use the trait methods [`MessageSigner::sign_message`]
    /// or [`TransactionSigner::sign_transaction`] instead.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        use ed25519_dalek::Signer;
        self.secret.sign(message).to_bytes()
    }
}

impl Clone for KeypairSigner {
    fn clone(&self) -> Self {
        let secret_bytes = self.secret_bytes();
        Self::from_bytes(&secret_bytes).expect("Valid keypair")
    }
}

impl MessageSigner for KeypairSigner {
    fn pubkey(&self) -> Pubkey {
        self.pubkey
    }

    fn sign_message(&self, message: &[u8]) -> std::result::Result<Signature, SignerError> {
        Ok(Signature::from(self.sign(message)))
    }
}

impl TransactionSigner for KeypairSigner {
    fn pubkey(&self) -> Pubkey {
        self.pubkey
    }

    fn sign_transaction(&self, message: &[u8]) -> std::result::Result<Signature, SignerError> {
        Ok(Signature::from(self.sign(message)))
    }

    fn is_interactive(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signer::Signer as SdkSigner;

    #[test]
    fn test_generate_unique_keys() {
        let k1 = KeypairSigner::generate();
        let k2 = KeypairSigner::generate();
        assert_ne!(k1.pubkey_bytes(), k2.pubkey_bytes());
    }

    #[test]
    fn test_from_bytes() {
        let signer = KeypairSigner::generate();
        let secret_bytes = signer.secret_bytes();

        let restored = KeypairSigner::from_bytes(&secret_bytes).unwrap();
        assert_eq!(signer.pubkey_bytes(), restored.pubkey_bytes());
    }

    #[test]
    fn test_sign_and_verify() {
        let signer = KeypairSigner::generate();
        let message = b"test message";

        let sig = signer.sign_message(message).unwrap();

        // Verify using ed25519-dalek
        use ed25519_dalek::{Signature as DalekSig, Verifier, VerifyingKey};
        let verifying_key = VerifyingKey::from_bytes(&signer.pubkey_bytes()).unwrap();
        let dalek_sig = DalekSig::from_bytes(&sig.into());
        assert!(verifying_key.verify(message, &dalek_sig).is_ok());
    }

    #[test]
    fn test_sign_transaction() {
        let signer = KeypairSigner::generate();
        let message = b"tx message";

        let sig = signer.sign_transaction(message).unwrap();
        assert!(!sig.to_string().is_empty());
    }

    #[test]
    fn test_clone() {
        let signer = KeypairSigner::generate();
        let cloned = signer.clone();

        assert_eq!(signer.pubkey_bytes(), cloned.pubkey_bytes());

        // Both should produce the same signature
        let message = b"test";
        assert_eq!(signer.sign(message), cloned.sign(message));
    }

    #[test]
    fn test_is_not_interactive() {
        let signer = KeypairSigner::generate();
        assert!(!signer.is_interactive());
    }

    #[test]
    fn test_to_solana_keypair() {
        let signer = KeypairSigner::generate();
        let sdk_keypair = signer.to_solana_keypair();

        assert_eq!(signer.pubkey_bytes(), sdk_keypair.pubkey().to_bytes());
    }
}
