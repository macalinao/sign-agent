//! File-based keypair loading and saving.

use std::io::Write;
use std::path::Path;

use zeroize::Zeroize;

use crate::error::{KeypairError, Result};
use crate::signer::KeypairSigner;

/// Load a keypair from a JSON file (Solana CLI format).
///
/// The JSON file should contain a byte array of either:
/// - 64 bytes (32-byte secret + 32-byte public key)
/// - 32 bytes (secret key only)
///
/// This is compatible with the Solana CLI keypair format.
///
/// # Arguments
///
/// * `path` - Path to the JSON keypair file.
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
///
/// # Example
///
/// ```no_run
/// use solana_actor_keypair::from_file;
///
/// let signer = from_file("~/.config/solana/id.json").unwrap();
/// println!("Loaded key: {}", signer.pubkey_base58());
/// ```
pub fn from_file(path: impl AsRef<Path>) -> Result<KeypairSigner> {
    let path = path.as_ref();
    let contents = std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            KeypairError::FileNotFound(path.display().to_string())
        } else {
            KeypairError::Io(e)
        }
    })?;
    from_json_string(&contents)
}

/// Load a keypair from a JSON string.
///
/// # Arguments
///
/// * `json` - JSON string containing a byte array.
///
/// # Errors
///
/// Returns an error if the JSON cannot be parsed or has invalid format.
pub fn from_json_string(json: &str) -> Result<KeypairSigner> {
    let mut bytes: Vec<u8> = serde_json::from_str(json)?;

    let result = if bytes.len() == 64 {
        // Full keypair: first 32 bytes are secret
        KeypairSigner::from_bytes(
            bytes[..32]
                .try_into()
                .map_err(|_| KeypairError::InvalidFormat("Invalid key size".into()))?,
        )
    } else if bytes.len() == 32 {
        // Just the secret key
        KeypairSigner::from_bytes(
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

/// Export a keypair to JSON format (Solana CLI compatible).
///
/// Returns a JSON array of the full 64-byte keypair (secret + public).
///
/// # Arguments
///
/// * `signer` - The keypair signer to export.
///
/// # Panics
///
/// Panics if JSON serialization fails (should never happen for valid keypairs).
pub fn to_json(signer: &KeypairSigner) -> String {
    let secret = signer.secret_bytes();
    let pubkey = signer.pubkey_bytes();

    let mut full = Vec::with_capacity(64);
    full.extend_from_slice(&secret[..]);
    full.extend_from_slice(&pubkey);

    serde_json::to_string(&full).expect("Valid JSON")
}

/// Export a keypair to a JSON file.
///
/// On Unix systems, the file is created with mode 0o600 (owner read/write only).
///
/// # Arguments
///
/// * `signer` - The keypair signer to export.
/// * `path` - Path to write the file.
///
/// # Errors
///
/// Returns an error if the file cannot be written.
#[cfg(unix)]
pub fn to_file(signer: &KeypairSigner, path: impl AsRef<Path>) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;

    let json = to_json(signer);

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600) // Owner read/write only
        .open(path)?;

    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Export a keypair to a JSON file.
///
/// # Arguments
///
/// * `signer` - The keypair signer to export.
/// * `path` - Path to write the file.
///
/// # Errors
///
/// Returns an error if the file cannot be written.
#[cfg(not(unix))]
pub fn to_file(signer: &KeypairSigner, path: impl AsRef<Path>) -> Result<()> {
    let json = to_json(signer);

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Export a keypair to base58 format (full 64-byte keypair).
///
/// # Arguments
///
/// * `signer` - The keypair signer to export.
pub fn to_base58(signer: &KeypairSigner) -> String {
    let secret = signer.secret_bytes();
    let pubkey = signer.pubkey_bytes();

    let mut full = Vec::with_capacity(64);
    full.extend_from_slice(&secret[..]);
    full.extend_from_slice(&pubkey);

    bs58::encode(&full).into_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_json_roundtrip_64_bytes() {
        let signer = KeypairSigner::generate();
        let json = to_json(&signer);
        let imported = from_json_string(&json).unwrap();

        assert_eq!(signer.pubkey_bytes(), imported.pubkey_bytes());
    }

    #[test]
    fn test_json_roundtrip_32_bytes() {
        let signer = KeypairSigner::generate();
        let secret = signer.secret_bytes();

        let json = serde_json::to_string(&secret[..].to_vec()).unwrap();
        let imported = from_json_string(&json).unwrap();

        assert_eq!(signer.pubkey_bytes(), imported.pubkey_bytes());
    }

    #[test]
    fn test_base58_roundtrip() {
        let signer = KeypairSigner::generate();
        let encoded = to_base58(&signer);
        let imported = KeypairSigner::from_base58(&encoded).unwrap();

        assert_eq!(signer.pubkey_bytes(), imported.pubkey_bytes());
    }

    #[test]
    fn test_file_roundtrip() {
        let signer = KeypairSigner::generate();
        let temp = NamedTempFile::new().unwrap();

        to_file(&signer, temp.path()).unwrap();
        let loaded = from_file(temp.path()).unwrap();

        assert_eq!(signer.pubkey_bytes(), loaded.pubkey_bytes());
    }

    #[test]
    fn test_file_not_found() {
        let result = from_file("/nonexistent/path/keypair.json");
        assert!(matches!(result, Err(KeypairError::FileNotFound(_))));
    }
}
