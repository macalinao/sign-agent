//! Key derivation using Argon2id

use argon2::{Algorithm, Argon2, Params, Version};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{Error, Result};

/// Memory cost for Argon2id (64 MB)
const ARGON2_M_COST: u32 = 65536;
/// Time cost for Argon2id (3 iterations)
const ARGON2_T_COST: u32 = 3;
/// Parallelism for Argon2id (4 lanes)
const ARGON2_P_COST: u32 = 4;

/// A derived encryption key that zeroizes on drop
#[derive(ZeroizeOnDrop)]
pub struct DerivedKey {
    key: [u8; 32],
}

impl DerivedKey {
    /// Derive a key from a password and salt using Argon2id
    pub fn derive(password: &[u8], salt: &[u8; 32]) -> Result<Self> {
        let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(32))
            .map_err(|e| Error::KeyDerivation(e.to_string()))?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut key = [0u8; 32];
        argon2
            .hash_password_into(password, salt, &mut key)
            .map_err(|e| Error::KeyDerivation(e.to_string()))?;

        Ok(Self { key })
    }

    /// Get the key bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

impl Zeroize for DerivedKey {
    fn zeroize(&mut self) {
        self.key.zeroize();
    }
}

/// Generate a password hash for verification
pub fn hash_password(password: &[u8], salt: &[u8; 32]) -> Result<[u8; 32]> {
    let key = DerivedKey::derive(password, salt)?;
    Ok(*key.as_bytes())
}

/// Verify a password against a stored hash
pub fn verify_password(password: &[u8], salt: &[u8; 32], expected_hash: &[u8; 32]) -> Result<bool> {
    let computed = hash_password(password, salt)?;
    // Constant-time comparison
    Ok(computed
        .iter()
        .zip(expected_hash.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key() {
        let password = b"test_password";
        let salt = [0u8; 32];

        let key = DerivedKey::derive(password, &salt).unwrap();
        assert_eq!(key.as_bytes().len(), 32);
    }

    #[test]
    fn test_verify_password() {
        let password = b"test_password";
        let mut salt = [0u8; 32];
        salt[0] = 1;

        let hash = hash_password(password, &salt).unwrap();
        assert!(verify_password(password, &salt, &hash).unwrap());
        assert!(!verify_password(b"wrong_password", &salt, &hash).unwrap());
    }
}
