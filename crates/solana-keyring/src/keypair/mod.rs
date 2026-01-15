//! Keypair management

mod export;
mod generate;
mod import;

pub use export::{export_base58, export_json};
pub use generate::generate_keypair;
pub use import::{import_base58, import_json};

use ed25519_dalek::{SigningKey, VerifyingKey};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::Result;

/// A keypair that zeroizes its secret on drop
#[derive(ZeroizeOnDrop)]
pub struct SecureKeypair {
    #[zeroize(skip)]
    pubkey: VerifyingKey,
    secret: SigningKey,
}

impl SecureKeypair {
    /// Create from raw secret key bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let secret = SigningKey::from_bytes(bytes);
        let pubkey = secret.verifying_key();
        Ok(Self { pubkey, secret })
    }

    /// Generate a new random keypair
    pub fn generate() -> Self {
        let secret = SigningKey::generate(&mut rand::thread_rng());
        let pubkey = secret.verifying_key();
        Self { pubkey, secret }
    }

    /// Get the public key
    pub fn pubkey(&self) -> &VerifyingKey {
        &self.pubkey
    }

    /// Get the public key bytes
    pub fn pubkey_bytes(&self) -> [u8; 32] {
        self.pubkey.to_bytes()
    }

    /// Get the public key as base58 string
    pub fn pubkey_base58(&self) -> String {
        bs58::encode(self.pubkey.as_bytes()).into_string()
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        use ed25519_dalek::Signer;
        self.secret.sign(message).to_bytes()
    }

    /// Export secret key bytes (caller must zeroize when done)
    pub fn secret_bytes(&self) -> zeroize::Zeroizing<[u8; 32]> {
        zeroize::Zeroizing::new(self.secret.to_bytes())
    }

    /// Convert to solana-sdk Keypair
    pub fn to_solana_keypair(&self) -> solana_sdk::signer::keypair::Keypair {
        let secret = self.secret_bytes();
        let mut full_key = [0u8; 64];
        full_key[..32].copy_from_slice(&secret[..]);
        full_key[32..].copy_from_slice(&self.pubkey_bytes());

        let result = solana_sdk::signer::keypair::Keypair::try_from(full_key.as_slice())
            .expect("Valid keypair bytes");

        // Zeroize the temporary full key
        full_key.zeroize();

        result
    }
}

impl Clone for SecureKeypair {
    fn clone(&self) -> Self {
        let secret_bytes = self.secret_bytes();
        Self::from_bytes(&secret_bytes).expect("Valid keypair")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = SecureKeypair::generate();
        assert_eq!(keypair.pubkey_bytes().len(), 32);
    }

    #[test]
    fn test_sign_verify() {
        let keypair = SecureKeypair::generate();
        let message = b"test message";
        let signature = keypair.sign(message);

        use ed25519_dalek::Verifier;
        let sig = ed25519_dalek::Signature::from_bytes(&signature);
        assert!(keypair.pubkey().verify(message, &sig).is_ok());
    }

    #[test]
    fn test_from_bytes() {
        let keypair = SecureKeypair::generate();
        let secret_bytes = keypair.secret_bytes();

        let restored = SecureKeypair::from_bytes(&secret_bytes).unwrap();
        assert_eq!(keypair.pubkey_bytes(), restored.pubkey_bytes());
    }
}
