//! Error types for Ledger operations.

use thiserror::Error;

/// Errors that can occur during Ledger operations.
#[derive(Error, Debug)]
pub enum LedgerError {
    /// Device not found or not connected.
    #[error("Ledger device not connected")]
    NotConnected,

    /// Communication error with the device.
    #[error("Ledger communication error: {0}")]
    Communication(String),

    /// Invalid response from device.
    #[error("Invalid response from Ledger: {0}")]
    InvalidResponse(String),

    /// User rejected the operation on the device.
    #[error("User rejected operation on Ledger")]
    UserRejected,

    /// App not opened on the device.
    #[error("Solana app not opened on Ledger")]
    AppNotOpened,

    /// Invalid derivation path.
    #[error("Invalid derivation path: {0}")]
    InvalidPath(String),

    /// HID API error.
    #[error("HID error: {0}")]
    Hid(String),
}

/// Result type for Ledger operations.
pub type Result<T> = std::result::Result<T, LedgerError>;

impl From<LedgerError> for solana_actor::SignerError {
    fn from(err: LedgerError) -> Self {
        match err {
            LedgerError::NotConnected => Self::DeviceNotFound,
            LedgerError::UserRejected => Self::UserCancelled,
            LedgerError::Communication(msg) => Self::DeviceError(msg),
            LedgerError::InvalidResponse(msg) => Self::DeviceError(msg),
            LedgerError::AppNotOpened => Self::DeviceError("Solana app not opened".into()),
            LedgerError::InvalidPath(msg) => Self::InvalidKey(msg),
            LedgerError::Hid(msg) => Self::DeviceError(msg),
        }
    }
}
