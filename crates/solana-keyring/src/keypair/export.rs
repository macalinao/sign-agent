//! Keypair export to various formats

use std::io::Write;
use std::path::Path;

use super::SecureKeypair;
use crate::error::Result;

/// Export a keypair to JSON format (Solana CLI compatible)
///
/// Returns a JSON array of the full 64-byte keypair (secret + public)
pub fn export_json(keypair: &SecureKeypair) -> String {
    let secret = keypair.secret_bytes();
    let pubkey = keypair.pubkey_bytes();

    let mut full = Vec::with_capacity(64);
    full.extend_from_slice(&secret[..]);
    full.extend_from_slice(&pubkey);

    serde_json::to_string(&full).expect("Valid JSON")
}

/// Export a keypair to a JSON file
#[cfg(unix)]
#[allow(dead_code)]
pub fn export_json_file(keypair: &SecureKeypair, path: &Path) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;

    let json = export_json(keypair);

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600) // Owner read/write only
        .open(path)?;

    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Export a keypair to a JSON file
#[cfg(not(unix))]
#[allow(dead_code)]
pub fn export_json_file(keypair: &SecureKeypair, path: &Path) -> Result<()> {
    let json = export_json(keypair);

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Export a keypair to base58 format (full 64-byte keypair)
pub fn export_base58(keypair: &SecureKeypair) -> String {
    let secret = keypair.secret_bytes();
    let pubkey = keypair.pubkey_bytes();

    let mut full = Vec::with_capacity(64);
    full.extend_from_slice(&secret[..]);
    full.extend_from_slice(&pubkey);

    bs58::encode(&full).into_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypair::import::{import_base58, import_json_string};

    #[test]
    fn test_export_json_roundtrip() {
        let keypair = SecureKeypair::generate();
        let json = export_json(&keypair);
        let imported = import_json_string(&json).unwrap();

        assert_eq!(keypair.pubkey_bytes(), imported.pubkey_bytes());
    }

    #[test]
    fn test_export_base58_roundtrip() {
        let keypair = SecureKeypair::generate();
        let encoded = export_base58(&keypair);
        let imported = import_base58(&encoded).unwrap();

        assert_eq!(keypair.pubkey_bytes(), imported.pubkey_bytes());
    }
}
