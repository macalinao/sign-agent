//! Ledger hardware wallet integration

mod transport;

use crate::error::{Error, Result};

/// Ledger signer for hardware wallet operations
pub struct LedgerSigner {
    derivation_path: Vec<u32>,
    pubkey: [u8; 32],
    pubkey_str: String,
}

impl LedgerSigner {
    /// Connect to a Ledger device and get the public key for the given derivation path
    pub fn connect(derivation_path: &str) -> Result<Self> {
        let path = parse_derivation_path(derivation_path)?;
        let pubkey = transport::get_pubkey(&path)?;
        let pubkey_str = bs58::encode(&pubkey).into_string();

        Ok(Self {
            derivation_path: path,
            pubkey,
            pubkey_str,
        })
    }

    /// Get the public key
    pub fn pubkey(&self) -> &str {
        &self.pubkey_str
    }

    /// Get the public key bytes
    pub fn pubkey_bytes(&self) -> &[u8; 32] {
        &self.pubkey
    }

    /// Sign a message using the Ledger device
    pub fn sign(&self, message: &[u8]) -> Result<[u8; 64]> {
        transport::sign_message(&self.derivation_path, message)
    }

    /// Get the derivation path
    pub fn derivation_path(&self) -> String {
        format_derivation_path(&self.derivation_path)
    }
}

/// Parse a derivation path string like "44'/501'/0'/0'"
fn parse_derivation_path(path: &str) -> Result<Vec<u32>> {
    let mut components = vec![];

    for part in path.trim_start_matches("m/").split('/') {
        let (num_str, hardened) = if part.ends_with('\'') || part.ends_with('h') {
            (&part[..part.len() - 1], true)
        } else {
            (part, false)
        };

        let mut num: u32 = num_str
            .parse()
            .map_err(|_| Error::Ledger(format!("Invalid derivation path component: {}", part)))?;

        if hardened {
            num |= 0x80000000;
        }
        components.push(num);
    }

    Ok(components)
}

/// Format a derivation path as a string
fn format_derivation_path(path: &[u32]) -> String {
    let parts: Vec<String> = path
        .iter()
        .map(|&n| {
            if n >= 0x80000000 {
                format!("{}'", n - 0x80000000)
            } else {
                format!("{}", n)
            }
        })
        .collect();

    format!("m/{}", parts.join("/"))
}

/// Default Solana derivation path
pub fn default_derivation_path() -> &'static str {
    "44'/501'/0'/0'"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_derivation_path() {
        let path = parse_derivation_path("44'/501'/0'/0'").unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], 44 | 0x80000000);
        assert_eq!(path[1], 501 | 0x80000000);
        assert_eq!(path[2], 0 | 0x80000000);
        assert_eq!(path[3], 0 | 0x80000000);
    }

    #[test]
    fn test_format_derivation_path() {
        let path = vec![
            44 | 0x80000000,
            501 | 0x80000000,
            0 | 0x80000000,
            0 | 0x80000000,
        ];
        assert_eq!(format_derivation_path(&path), "m/44'/501'/0'/0'");
    }
}
