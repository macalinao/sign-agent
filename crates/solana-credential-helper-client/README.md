# solana-credential-helper-client

[![Crates.io](https://img.shields.io/crates/v/solana-credential-helper-client.svg)](https://crates.io/crates/solana-credential-helper-client)
[![Downloads](https://img.shields.io/crates/d/solana-credential-helper-client.svg)](https://crates.io/crates/solana-credential-helper-client)
[![Documentation](https://docs.rs/solana-credential-helper-client/badge.svg)](https://docs.rs/solana-credential-helper-client)
[![License](https://img.shields.io/crates/l/solana-credential-helper-client.svg)](https://github.com/macalinao/sign-agent/blob/master/LICENSE)

Rust client library for Solana transaction signing via credential helper.

## Features

- **Agent Socket**: Sign via Unix socket for best performance
- **CLI Subprocess**: Fall back to CLI tool when needed
- **Multiple Signer Types**: Keypair, Ledger, Squads
- **Async/Await**: Built on Tokio for async operation

## Installation

```toml
[dependencies]
solana-credential-helper-client = "0.1"
```

## Usage

```rust
use solana_credential_helper_client::{
    CredentialHelperClient,
    CredentialHelperConfig,
    SignerType,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the client
    let config = CredentialHelperConfig::new("ABC123...")
        .signer_type(SignerType::Keypair)
        .use_agent(true);

    let client = CredentialHelperClient::new(config);

    // Sign a transaction message
    let message_bytes: Vec<u8> = vec![/* serialized message */];
    let signature = client.sign_transaction(&message_bytes).await?;

    println!("Signature: {:?}", signature);
    Ok(())
}
```

### Signing Methods

#### Via Agent Socket (Recommended)

```rust
let config = CredentialHelperConfig::new(pubkey)
    .use_agent(true)
    .agent_socket_path("/path/to/agent.sock");

let signature = client.sign_via_agent(&message).await?;
```

#### Via CLI Subprocess

```rust
let config = CredentialHelperConfig::new(pubkey)
    .use_agent(false)
    .binary_path("/usr/local/bin/solana-credential-helper");

let signature = client.sign_via_cli(&message).await?;
```

### Signer Types

```rust
// Local keypair
let config = CredentialHelperConfig::new(pubkey)
    .signer_type(SignerType::Keypair);

// Ledger hardware wallet
let config = CredentialHelperConfig::new(pubkey)
    .signer_type(SignerType::Ledger);

// Squads multisig
let config = CredentialHelperConfig::new(pubkey)
    .signer_type(SignerType::Squads)
    .squads_address("MULTISIG_ADDRESS")
    .rpc_url("https://api.mainnet-beta.solana.com");
```

## License

Apache-2.0
