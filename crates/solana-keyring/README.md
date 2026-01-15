# solana-keyring

[![Crates.io](https://img.shields.io/crates/v/solana-keyring.svg)](https://crates.io/crates/solana-keyring)
[![Downloads](https://img.shields.io/crates/d/solana-keyring.svg)](https://crates.io/crates/solana-keyring)
[![Documentation](https://docs.rs/solana-keyring/badge.svg)](https://docs.rs/solana-keyring)
[![License](https://img.shields.io/crates/l/solana-keyring.svg)](https://github.com/macalinao/sign-agent/blob/master/LICENSE)

Secure keyring library for Solana with encrypted storage, Ledger hardware wallet integration, and Squads multisig support.

## Features

- **Encrypted Storage**: AES-256-GCM encryption with Argon2id key derivation
- **Row-level Encryption**: Each keypair encrypted with unique salt and nonce
- **Ledger Support**: Sign transactions with Ledger hardware wallets
- **Squads Multisig**: Create and approve Squads v4 proposals
- **Biometric Auth**: TouchID confirmation on macOS
- **Address Book**: Label and organize addresses
- **Transaction Parsing**: Human-readable transaction summaries

## Installation

```toml
[dependencies]
solana-keyring = "0.1"
```

## Usage

```rust
use solana_keyring::{Database, SecureKeypair};

// Open or create database
let db = Database::open(&solana_keyring::default_db_path())?;

// Initialize with master passphrase
db.initialize(b"your-secure-passphrase")?;

// Generate a new keypair
let keypair = SecureKeypair::generate();
db.store_keypair(&keypair, "my-wallet", b"passphrase", &["main"])?;

// Load and sign
let loaded = db.load_keypair("my-wallet", b"passphrase")?;
let signature = loaded.sign(message);
```

## License

Apache-2.0
