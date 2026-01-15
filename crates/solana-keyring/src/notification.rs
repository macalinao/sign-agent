//! Cross-platform notifications

use notify_rust::Notification;

use crate::error::Result;

/// Send a notification to the user
pub fn notify(title: &str, body: &str) -> Result<()> {
    Notification::new()
        .summary(title)
        .body(body)
        .appname("solana-keyring")
        .show()
        .map(|_| ())
        .map_err(|e| crate::error::Error::Io(std::io::Error::other(e.to_string())))
}

/// Send a notification for signing request
#[allow(dead_code)]
pub fn notify_sign_request(signer: &str, app: Option<&str>) -> Result<()> {
    let body = match app {
        Some(app) => format!("{} requested signature from {}", app, signer),
        None => format!("Signature requested from {}", signer),
    };
    notify("Signature Request", &body)
}

/// Send a notification for successful signing
#[allow(dead_code)]
pub fn notify_sign_success(signer: &str) -> Result<()> {
    notify(
        "Transaction Signed",
        &format!("Successfully signed with {}", signer),
    )
}

/// Send a notification for signing error
#[allow(dead_code)]
pub fn notify_sign_error(error: &str) -> Result<()> {
    notify("Signing Error", error)
}
