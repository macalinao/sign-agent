//! BIP-44 derivation path handling.

use crate::error::{LedgerError, Result};

/// Default Solana derivation path (BIP-44).
pub const DEFAULT_PATH: &str = "44'/501'/0'/0'";

/// Parse a derivation path string like "44'/501'/0'/0'" or "m/44'/501'/0'/0'".
///
/// Supports both `'` and `h` as hardened markers.
///
/// # Arguments
///
/// * `path` - The derivation path string.
///
/// # Returns
///
/// A vector of path components with hardened bit set where appropriate.
///
/// # Errors
///
/// Returns [`LedgerError::InvalidPath`] if the path cannot be parsed.
pub fn parse_path(path: &str) -> Result<Vec<u32>> {
    let mut components = vec![];

    for part in path.trim_start_matches("m/").split('/') {
        let (num_str, hardened) = if part.ends_with('\'') || part.ends_with('h') {
            (&part[..part.len() - 1], true)
        } else {
            (part, false)
        };

        let mut num: u32 = num_str
            .parse()
            .map_err(|_| LedgerError::InvalidPath(format!("Invalid component: {}", part)))?;

        if hardened {
            num |= 0x80000000;
        }
        components.push(num);
    }

    if components.is_empty() {
        return Err(LedgerError::InvalidPath("Empty derivation path".into()));
    }

    Ok(components)
}

/// Format a derivation path as a human-readable string.
///
/// # Arguments
///
/// * `path` - The path components.
///
/// # Returns
///
/// A string like "m/44'/501'/0'/0'".
pub fn format_path(path: &[u32]) -> String {
    let parts: Vec<String> = path
        .iter()
        .map(|&n| {
            if n >= 0x80000000 {
                format!("{}'", n - 0x80000000)
            } else {
                format!("{n}")
            }
        })
        .collect();

    format!("m/{}", parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_default_path() {
        let path = parse_path(DEFAULT_PATH).unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], 44 | 0x80000000);
        assert_eq!(path[1], 501 | 0x80000000);
        assert_eq!(path[2], 0 | 0x80000000);
        assert_eq!(path[3], 0 | 0x80000000);
    }

    #[test]
    fn test_parse_with_m_prefix() {
        let path = parse_path("m/44'/501'/0'/0'").unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], 44 | 0x80000000);
    }

    #[test]
    fn test_parse_with_h_marker() {
        let path = parse_path("44h/501h/0h/0h").unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0], 44 | 0x80000000);
    }

    #[test]
    fn test_parse_non_hardened() {
        let path = parse_path("44'/501'/0/1").unwrap();
        assert_eq!(path[2], 0);
        assert_eq!(path[3], 1);
    }

    #[test]
    fn test_format_path() {
        let path = vec![
            44 | 0x80000000,
            501 | 0x80000000,
            0 | 0x80000000,
            0 | 0x80000000,
        ];
        assert_eq!(format_path(&path), "m/44'/501'/0'/0'");
    }

    #[test]
    fn test_format_path_mixed() {
        let path = vec![44 | 0x80000000, 501 | 0x80000000, 0, 1];
        assert_eq!(format_path(&path), "m/44'/501'/0/1");
    }

    #[test]
    fn test_parse_invalid() {
        assert!(parse_path("invalid").is_err());
        assert!(parse_path("").is_err());
    }
}
