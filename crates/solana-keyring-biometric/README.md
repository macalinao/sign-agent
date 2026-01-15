# solana-keyring-biometric

[![Crates.io](https://img.shields.io/crates/v/solana-keyring-biometric.svg)](https://crates.io/crates/solana-keyring-biometric)
[![Downloads](https://img.shields.io/crates/d/solana-keyring-biometric.svg)](https://crates.io/crates/solana-keyring-biometric)
[![Documentation](https://docs.rs/solana-keyring-biometric/badge.svg)](https://docs.rs/solana-keyring-biometric)
[![License](https://img.shields.io/crates/l/solana-keyring-biometric.svg)](https://github.com/macalinao/sign-agent/blob/master/LICENSE)

Biometric authentication (TouchID) for Solana keyring on macOS.

## Features

- **TouchID Integration**: Native macOS LocalAuthentication framework
- **Passcode Fallback**: Falls back to device passcode when biometrics unavailable
- **Cross-platform**: No-op on non-macOS platforms (always succeeds)
- **Transaction Confirmation**: Prompt users before signing

## Installation

```toml
[dependencies]
solana-keyring-biometric = "0.1"
```

## Usage

```rust
use solana_keyring_biometric::{authenticate, confirm_signing, AuthResult};

// Check availability
if solana_keyring_biometric::is_available() {
    // Request authentication
    match authenticate("Confirm your identity")? {
        AuthResult::Authenticated => println!("Success!"),
        AuthResult::Denied => println!("User cancelled"),
        AuthResult::NotAvailable => println!("Biometrics unavailable"),
    }
}

// Confirm transaction signing
match confirm_signing("my-wallet", "Transfer 1.5 SOL to ABC...")? {
    AuthResult::Authenticated => { /* proceed with signing */ }
    _ => { /* handle denial */ }
}
```

## Platform Support

| Platform | Support |
|----------|---------|
| macOS | Full TouchID/passcode support |
| Linux | No-op (always returns Authenticated) |
| Windows | No-op (always returns Authenticated) |

## License

Apache-2.0
