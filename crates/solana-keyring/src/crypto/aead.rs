//! AES-256-GCM encryption for keypair secrets

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use zeroize::Zeroize;

use super::kdf::DerivedKey;
use crate::error::Result;

/// Encrypted data with nonce and salt for key derivation
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// The encrypted ciphertext
    pub ciphertext: Vec<u8>,
    /// 12-byte nonce for AES-GCM
    pub nonce: [u8; 12],
    /// 32-byte salt for Argon2id key derivation
    pub salt: [u8; 32],
}

/// Encrypt a secret using AES-256-GCM with a password-derived key
///
/// Each encryption uses a unique random salt and nonce to ensure
/// that identical secrets produce different ciphertexts.
pub fn encrypt_secret(secret: &[u8], master_password: &[u8]) -> Result<EncryptedData> {
    // Generate random salt and nonce
    let mut salt = [0u8; 32];
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    // Derive encryption key from password + salt
    let derived_key = DerivedKey::derive(master_password, &salt)?;

    // Encrypt with AES-256-GCM
    let cipher = Aes256Gcm::new_from_slice(derived_key.as_bytes())
        .expect("AES-256-GCM key should be 32 bytes");
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, secret)?;

    Ok(EncryptedData {
        ciphertext,
        nonce: nonce_bytes,
        salt,
    })
}

/// Decrypt a secret using AES-256-GCM with a password-derived key
pub fn decrypt_secret(encrypted: &EncryptedData, master_password: &[u8]) -> Result<Vec<u8>> {
    // Derive the same key using stored salt
    let derived_key = DerivedKey::derive(master_password, &encrypted.salt)?;

    // Decrypt with AES-256-GCM
    let cipher = Aes256Gcm::new_from_slice(derived_key.as_bytes())
        .expect("AES-256-GCM key should be 32 bytes");
    let nonce = Nonce::from_slice(&encrypted.nonce);
    let mut plaintext = cipher.decrypt(nonce, encrypted.ciphertext.as_slice())?;

    // Return plaintext (caller should zeroize when done)
    let result = plaintext.clone();
    plaintext.zeroize();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let secret = b"my_secret_key_32_bytes_exactly!!";
        let password = b"test_password";

        let encrypted = encrypt_secret(secret, password).unwrap();
        let decrypted = decrypt_secret(&encrypted, password).unwrap();

        assert_eq!(decrypted, secret);
    }

    #[test]
    fn test_wrong_password() {
        let secret = b"my_secret_key_32_bytes_exactly!!";
        let password = b"test_password";
        let wrong_password = b"wrong_password";

        let encrypted = encrypt_secret(secret, password).unwrap();
        let result = decrypt_secret(&encrypted, wrong_password);

        assert!(result.is_err());
    }

    #[test]
    fn test_unique_ciphertexts() {
        let secret = b"same_secret";
        let password = b"test_password";

        let encrypted1 = encrypt_secret(secret, password).unwrap();
        let encrypted2 = encrypt_secret(secret, password).unwrap();

        // Different salt and nonce should produce different ciphertexts
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
        assert_ne!(encrypted1.salt, encrypted2.salt);
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
    }
}
