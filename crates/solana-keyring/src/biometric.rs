//! Biometric authentication (TouchID on macOS)
//!
//! This module re-exports functionality from the `solana-keyring-biometric` crate
//! and provides compatibility wrappers for the existing API.

pub use solana_keyring_biometric::{
    AuthResult, BiometricConfig, Error as BiometricError, authenticate, authenticate_with_config,
    confirm_signing, confirm_signing_with_config, is_available, is_passcode_available,
};

use crate::error::{Error, Result};

/// Compatibility wrapper that converts AuthResult to bool Result
///
/// This maintains backwards compatibility with code expecting `Result<bool>`.
pub fn authenticate_bool(reason: &str) -> Result<bool> {
    match authenticate(reason) {
        Ok(AuthResult::Authenticated) => Ok(true),
        Ok(AuthResult::Denied) => Ok(false),
        Ok(AuthResult::NotAvailable) => Ok(false),
        Err(e) => Err(Error::Biometric(e.to_string())),
    }
}

/// Compatibility wrapper for confirm_signing that returns bool Result
pub fn confirm_signing_bool(signer_label: &str, transaction_summary: &str) -> Result<bool> {
    match confirm_signing(signer_label, transaction_summary) {
        Ok(AuthResult::Authenticated) => Ok(true),
        Ok(AuthResult::Denied) => Ok(false),
        Ok(AuthResult::NotAvailable) => Ok(false),
        Err(e) => Err(Error::Biometric(e.to_string())),
    }
}
