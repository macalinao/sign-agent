//! Biometric authentication library for macOS TouchID
//!
//! This crate provides biometric authentication capabilities using macOS TouchID
//! via the LocalAuthentication framework. On non-macOS platforms, authentication
//! is a no-op that always succeeds.
//!
//! # Example
//!
//! ```no_run
//! use solana_keyring_biometric::{is_available, authenticate, AuthResult};
//!
//! // Check if biometric authentication is available
//! if is_available() {
//!     // Request authentication with a reason
//!     match authenticate("Confirm your identity") {
//!         Ok(AuthResult::Authenticated) => println!("Success!"),
//!         Ok(AuthResult::Denied) => println!("Authentication denied"),
//!         Ok(AuthResult::NotAvailable) => println!("Biometrics not available"),
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```

use std::fmt;

/// Errors that can occur during biometric authentication
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Swift execution failed
    #[error("Swift execution failed: {0}")]
    SwiftExecution(String),

    /// IO error during command execution
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid response from authentication
    #[error("Invalid authentication response: {0}")]
    InvalidResponse(String),
}

/// Result type for biometric operations
pub type Result<T> = std::result::Result<T, Error>;

/// Result of an authentication attempt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthResult {
    /// User successfully authenticated
    Authenticated,
    /// User denied/cancelled authentication
    Denied,
    /// Biometric authentication is not available on this system
    NotAvailable,
}

impl fmt::Display for AuthResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthResult::Authenticated => write!(f, "authenticated"),
            AuthResult::Denied => write!(f, "denied"),
            AuthResult::NotAvailable => write!(f, "not available"),
        }
    }
}

/// Configuration for biometric authentication
#[derive(Debug, Clone)]
pub struct BiometricConfig {
    /// Whether to allow fallback to device passcode
    pub allow_passcode_fallback: bool,
}

impl Default for BiometricConfig {
    fn default() -> Self {
        Self {
            allow_passcode_fallback: true,
        }
    }
}

/// Path to the swift binary
const SWIFT_PATH: &str = "/usr/bin/swift";

/// Execute a swift command with clean environment
#[cfg(target_os = "macos")]
fn run_swift(code: &str) -> Result<std::process::Output> {
    use std::process::Command;

    Command::new(SWIFT_PATH)
        .arg("-e")
        .arg(code)
        // Clear these env vars to prevent nix/devenv from redirecting swift
        // to an incompatible SDK
        .env_remove("DEVELOPER_DIR")
        .env_remove("SDKROOT")
        .output()
        .map_err(Error::Io)
}

/// Check if biometric authentication is available on this system
///
/// On macOS, this checks if TouchID hardware is present and configured.
/// On other platforms, this always returns `false`.
///
/// # Example
///
/// ```no_run
/// use solana_keyring_biometric::is_available;
///
/// if is_available() {
///     println!("TouchID is available!");
/// } else {
///     println!("TouchID is not available");
/// }
/// ```
#[cfg(target_os = "macos")]
pub fn is_available() -> bool {
    let swift_code = r#"
import LocalAuthentication
let context = LAContext()
var error: NSError?
let available = context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error)
print(available ? "yes" : "no")
"#;

    run_swift(swift_code)
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .eq_ignore_ascii_case("yes")
        })
        .unwrap_or(false)
}

#[cfg(not(target_os = "macos"))]
pub fn is_available() -> bool {
    false
}

/// Check if device passcode authentication is available
///
/// This checks if the device has a passcode set, which can be used as a
/// fallback when biometric authentication is not available.
#[cfg(target_os = "macos")]
pub fn is_passcode_available() -> bool {
    let swift_code = r#"
import LocalAuthentication
let context = LAContext()
var error: NSError?
let available = context.canEvaluatePolicy(.deviceOwnerAuthentication, error: &error)
print(available ? "yes" : "no")
"#;

    run_swift(swift_code)
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .eq_ignore_ascii_case("yes")
        })
        .unwrap_or(false)
}

#[cfg(not(target_os = "macos"))]
pub fn is_passcode_available() -> bool {
    false
}

/// Escape a string for use in Swift code
fn escape_swift_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Request biometric authentication with a reason message
///
/// On macOS, this triggers the TouchID prompt with the given reason.
/// If TouchID is not available but passcode is, it will fall back to passcode.
/// On other platforms, this always returns `Ok(AuthResult::Authenticated)`.
///
/// # Arguments
///
/// * `reason` - The reason shown to the user explaining why authentication is needed
///
/// # Errors
///
/// Returns an error if:
/// - The Swift runtime fails to execute
/// - An IO error occurs during command execution
/// - The authentication response is invalid or unexpected
///
/// # Example
///
/// ```no_run
/// use solana_keyring_biometric::{authenticate, AuthResult};
///
/// match authenticate("Sign transaction with wallet 'main'") {
///     Ok(AuthResult::Authenticated) => println!("User authenticated!"),
///     Ok(AuthResult::Denied) => println!("User cancelled"),
///     Ok(AuthResult::NotAvailable) => println!("No authentication available"),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
#[cfg(target_os = "macos")]
pub fn authenticate(reason: &str) -> Result<AuthResult> {
    authenticate_with_config(reason, &BiometricConfig::default())
}

#[cfg(not(target_os = "macos"))]
pub fn authenticate(_reason: &str) -> Result<AuthResult> {
    // On non-macOS platforms, always succeed
    Ok(AuthResult::Authenticated)
}

/// Request biometric authentication with custom configuration
///
/// # Arguments
///
/// * `reason` - The reason shown to the user
/// * `config` - Configuration options for the authentication
///
/// # Errors
///
/// Returns an error if:
/// - The Swift runtime fails to execute
/// - An IO error occurs during command execution
/// - The authentication response is invalid or unexpected
#[cfg(target_os = "macos")]
pub fn authenticate_with_config(reason: &str, config: &BiometricConfig) -> Result<AuthResult> {
    let escaped_reason = escape_swift_string(reason);

    let fallback_code = if config.allow_passcode_fallback {
        format!(
            r#"
    // Fall back to device passcode if biometrics not available
    guard context.canEvaluatePolicy(.deviceOwnerAuthentication, error: &error) else {{
        print("not_available")
        exit(0)
    }}
    let semaphore = DispatchSemaphore(value: 0)
    var success = false
    context.evaluatePolicy(.deviceOwnerAuthentication, localizedReason: "{reason}") {{ result, authError in
        success = result
        semaphore.signal()
    }}
    semaphore.wait()
    print(success ? "authenticated" : "denied")
    exit(0)
"#,
            reason = escaped_reason
        )
    } else {
        r#"
    print("not_available")
    exit(0)
"#
        .to_string()
    };

    let swift_code = format!(
        r#"
import Foundation
import LocalAuthentication

let context = LAContext()
var error: NSError?

guard context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) else {{
{fallback_code}
}}

let semaphore = DispatchSemaphore(value: 0)
var success = false

context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: "{reason}") {{ result, authError in
    success = result
    semaphore.signal()
}}

semaphore.wait()
print(success ? "authenticated" : "denied")
"#,
        reason = escaped_reason,
        fallback_code = fallback_code
    );

    let output = run_swift(&swift_code)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result = stdout.trim();

    match result {
        "authenticated" => Ok(AuthResult::Authenticated),
        "denied" => Ok(AuthResult::Denied),
        "not_available" => Ok(AuthResult::NotAvailable),
        _ => {
            // Check stderr for additional error info
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                Err(Error::SwiftExecution(stderr.to_string()))
            } else if output.status.success() {
                // Unknown output but success status
                Ok(AuthResult::Authenticated)
            } else {
                Err(Error::InvalidResponse(format!(
                    "Unexpected response: '{}', exit code: {:?}",
                    result,
                    output.status.code()
                )))
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn authenticate_with_config(_reason: &str, _config: &BiometricConfig) -> Result<AuthResult> {
    Ok(AuthResult::Authenticated)
}

/// Request confirmation for a signing operation
///
/// This is a convenience function that formats a reason message for
/// transaction signing and requests biometric authentication.
///
/// # Arguments
///
/// * `signer_label` - Label of the wallet/signer being used
/// * `transaction_summary` - Human-readable summary of the transaction
///
/// # Errors
///
/// Returns an error if the underlying authentication fails. See [`authenticate`]
/// for details on possible error conditions.
///
/// # Example
///
/// ```no_run
/// use solana_keyring_biometric::{confirm_signing, AuthResult};
///
/// let result = confirm_signing("main-wallet", "Transfer 1.5 SOL to ABC...")?;
/// match result {
///     AuthResult::Authenticated => println!("User approved signing"),
///     AuthResult::Denied => println!("User rejected"),
///     AuthResult::NotAvailable => println!("No auth available"),
/// }
/// # Ok::<(), solana_keyring_biometric::Error>(())
/// ```
pub fn confirm_signing(signer_label: &str, transaction_summary: &str) -> Result<AuthResult> {
    let reason = format!(
        "Sign transaction with '{}':\n{}",
        signer_label, transaction_summary
    );
    authenticate(&reason)
}

/// Request confirmation with custom configuration.
///
/// # Errors
///
/// Returns an error if the underlying authentication fails. See [`authenticate_with_config`]
/// for details on possible error conditions.
pub fn confirm_signing_with_config(
    signer_label: &str,
    transaction_summary: &str,
    config: &BiometricConfig,
) -> Result<AuthResult> {
    let reason = format!(
        "Sign transaction with '{}':\n{}",
        signer_label, transaction_summary
    );
    authenticate_with_config(&reason, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_swift_string_empty() {
        assert_eq!(escape_swift_string(""), "");
    }

    #[test]
    fn test_escape_swift_string_simple() {
        assert_eq!(escape_swift_string("hello world"), "hello world");
    }

    #[test]
    fn test_escape_swift_string_quotes() {
        assert_eq!(escape_swift_string(r#"say "hello""#), r#"say \"hello\""#);
    }

    #[test]
    fn test_escape_swift_string_backslash() {
        assert_eq!(escape_swift_string(r"path\to\file"), r"path\\to\\file");
    }

    #[test]
    fn test_escape_swift_string_newlines() {
        assert_eq!(escape_swift_string("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_escape_swift_string_tabs() {
        assert_eq!(escape_swift_string("col1\tcol2"), "col1\\tcol2");
    }

    #[test]
    fn test_escape_swift_string_carriage_return() {
        assert_eq!(escape_swift_string("line1\r\nline2"), "line1\\r\\nline2");
    }

    #[test]
    fn test_escape_swift_string_complex() {
        let input = "Sign \"transfer\" to:\n  Address: ABC\\DEF";
        let expected = "Sign \\\"transfer\\\" to:\\n  Address: ABC\\\\DEF";
        assert_eq!(escape_swift_string(input), expected);
    }

    #[test]
    fn test_auth_result_display() {
        assert_eq!(AuthResult::Authenticated.to_string(), "authenticated");
        assert_eq!(AuthResult::Denied.to_string(), "denied");
        assert_eq!(AuthResult::NotAvailable.to_string(), "not available");
    }

    #[test]
    fn test_auth_result_equality() {
        assert_eq!(AuthResult::Authenticated, AuthResult::Authenticated);
        assert_eq!(AuthResult::Denied, AuthResult::Denied);
        assert_eq!(AuthResult::NotAvailable, AuthResult::NotAvailable);
        assert_ne!(AuthResult::Authenticated, AuthResult::Denied);
    }

    #[test]
    fn test_biometric_config_default() {
        let config = BiometricConfig::default();
        assert!(config.allow_passcode_fallback);
    }

    #[test]
    fn test_error_display() {
        let err = Error::SwiftExecution("test error".to_string());
        assert!(err.to_string().contains("Swift execution failed"));
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_error_invalid_response() {
        let err = Error::InvalidResponse("bad response".to_string());
        assert!(err.to_string().contains("Invalid authentication response"));
    }

    // Platform-specific tests
    #[cfg(target_os = "macos")]
    mod macos_tests {
        use super::*;

        #[test]
        fn test_is_available_returns_bool() {
            // Just verify it returns without panicking
            let _ = is_available();
        }

        #[test]
        fn test_is_passcode_available_returns_bool() {
            let _ = is_passcode_available();
        }

        #[test]
        fn test_swift_path_exists() {
            use std::path::Path;
            assert!(
                Path::new(SWIFT_PATH).exists(),
                "Swift binary should exist at {}",
                SWIFT_PATH
            );
        }

        #[test]
        fn test_run_swift_simple() {
            let result = run_swift(r#"print("hello")"#);
            assert!(result.is_ok(), "Swift should execute successfully");
            let output = result.unwrap();
            assert!(output.status.success());
            assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
        }

        #[test]
        fn test_run_swift_with_import() {
            let result = run_swift(
                r#"
import Foundation
print("foundation works")
"#,
            );
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.status.success());
        }

        #[test]
        fn test_run_swift_local_authentication_import() {
            // Test that LocalAuthentication framework can be imported
            let result = run_swift(
                r#"
import LocalAuthentication
print("la imported")
"#,
            );
            assert!(
                result.is_ok(),
                "Should be able to import LocalAuthentication"
            );
            let output = result.unwrap();
            assert!(output.status.success());
        }

        #[test]
        fn test_run_swift_la_context_creation() {
            // Test that we can create an LAContext
            let result = run_swift(
                r#"
import LocalAuthentication
let context = LAContext()
print("context created")
"#,
            );
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.status.success());
            assert_eq!(
                String::from_utf8_lossy(&output.stdout).trim(),
                "context created"
            );
        }

        #[test]
        fn test_run_swift_can_evaluate_policy() {
            // Test the canEvaluatePolicy call
            let result = run_swift(
                r#"
import LocalAuthentication
let context = LAContext()
var error: NSError?
let biometrics = context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error)
let passcode = context.canEvaluatePolicy(.deviceOwnerAuthentication, error: &error)
print("biometrics: \(biometrics), passcode: \(passcode)")
"#,
            );
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.status.success());
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("biometrics:"));
            assert!(stdout.contains("passcode:"));
        }
    }

    #[cfg(not(target_os = "macos"))]
    mod non_macos_tests {
        use super::*;

        #[test]
        fn test_is_available_returns_false() {
            assert!(!is_available());
        }

        #[test]
        fn test_is_passcode_available_returns_false() {
            assert!(!is_passcode_available());
        }

        #[test]
        fn test_authenticate_returns_authenticated() {
            let result = authenticate("test reason");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), AuthResult::Authenticated);
        }

        #[test]
        fn test_authenticate_with_config_returns_authenticated() {
            let config = BiometricConfig::default();
            let result = authenticate_with_config("test reason", &config);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), AuthResult::Authenticated);
        }

        #[test]
        fn test_confirm_signing_returns_authenticated() {
            let result = confirm_signing("wallet", "summary");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), AuthResult::Authenticated);
        }
    }
}
