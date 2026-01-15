//! Keypair generation

use super::SecureKeypair;

/// Generate a new random keypair
pub fn generate_keypair() -> SecureKeypair {
    SecureKeypair::generate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_produces_unique_keys() {
        let key1 = generate_keypair();
        let key2 = generate_keypair();
        assert_ne!(key1.pubkey_bytes(), key2.pubkey_bytes());
    }
}
