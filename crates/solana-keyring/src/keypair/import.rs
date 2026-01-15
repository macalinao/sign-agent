//! Keypair import from various formats

use std::path::Path;
use zeroize::Zeroize;

use super::SecureKeypair;
use crate::error::{Error, Result};

/// Import a keypair from a JSON file (Solana CLI format)
///
/// The JSON file should contain a byte array of the full 64-byte keypair
/// (32 bytes secret + 32 bytes public)
pub fn import_json(path: &Path) -> Result<SecureKeypair> {
    let contents = std::fs::read_to_string(path)?;
    import_json_string(&contents)
}

/// Import a keypair from a JSON string
pub fn import_json_string(json: &str) -> Result<SecureKeypair> {
    let mut bytes: Vec<u8> = serde_json::from_str(json)?;

    if bytes.len() == 64 {
        // Full keypair: first 32 bytes are secret
        let result = SecureKeypair::from_bytes(
            bytes[..32]
                .try_into()
                .map_err(|_| Error::InvalidKeypairFormat("Invalid key size".into()))?,
        );
        bytes.zeroize();
        result
    } else if bytes.len() == 32 {
        // Just the secret key
        let result = SecureKeypair::from_bytes(
            bytes
                .as_slice()
                .try_into()
                .map_err(|_| Error::InvalidKeypairFormat("Invalid key size".into()))?,
        );
        bytes.zeroize();
        result
    } else {
        bytes.zeroize();
        Err(Error::InvalidKeypairFormat(format!(
            "Expected 32 or 64 bytes, got {}",
            bytes.len()
        )))
    }
}

/// Import a keypair from a base58-encoded secret key
pub fn import_base58(encoded: &str) -> Result<SecureKeypair> {
    let mut bytes = bs58::decode(encoded).into_vec()?;

    let result = if bytes.len() == 64 {
        // Full keypair format
        SecureKeypair::from_bytes(
            bytes[..32]
                .try_into()
                .map_err(|_| Error::InvalidKeypairFormat("Invalid key size".into()))?,
        )
    } else if bytes.len() == 32 {
        // Just the secret key
        SecureKeypair::from_bytes(
            bytes
                .as_slice()
                .try_into()
                .map_err(|_| Error::InvalidKeypairFormat("Invalid key size".into()))?,
        )
    } else {
        bytes.zeroize();
        return Err(Error::InvalidKeypairFormat(format!(
            "Expected 32 or 64 bytes, got {}",
            bytes.len()
        )));
    };

    bytes.zeroize();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_json_64_bytes() {
        let keypair = SecureKeypair::generate();
        let secret = keypair.secret_bytes();
        let pubkey = keypair.pubkey_bytes();

        let mut full = Vec::with_capacity(64);
        full.extend_from_slice(&secret[..]);
        full.extend_from_slice(&pubkey);

        let json = serde_json::to_string(&full).unwrap();
        let imported = import_json_string(&json).unwrap();

        assert_eq!(keypair.pubkey_bytes(), imported.pubkey_bytes());
    }

    #[test]
    fn test_import_json_32_bytes() {
        let keypair = SecureKeypair::generate();
        let secret = keypair.secret_bytes();

        let json = serde_json::to_string(&secret[..].to_vec()).unwrap();
        let imported = import_json_string(&json).unwrap();

        assert_eq!(keypair.pubkey_bytes(), imported.pubkey_bytes());
    }

    #[test]
    fn test_import_base58() {
        let keypair = SecureKeypair::generate();
        let secret = keypair.secret_bytes();
        let pubkey = keypair.pubkey_bytes();

        let mut full = Vec::with_capacity(64);
        full.extend_from_slice(&secret[..]);
        full.extend_from_slice(&pubkey);

        let encoded = bs58::encode(&full).into_string();
        let imported = import_base58(&encoded).unwrap();

        assert_eq!(keypair.pubkey_bytes(), imported.pubkey_bytes());
    }
}
