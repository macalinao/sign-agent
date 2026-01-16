# solana-actor

Core traits for Solana signing and transaction submission.

## What is an Actor?

An **actor** is any entity that can perform actions on the Solana blockchain. This includes:

- **Keypairs** - Software wallets that sign directly
- **Hardware wallets** - Ledger devices that sign with user confirmation
- **Multisigs** - Squads vaults that require multiple approvals

The "actor" abstraction unifies these different signing mechanisms behind common traits, allowing code to work with any actor type without knowing the underlying implementation.

## Overview

This crate provides a clean separation between:

- **Signers** - Synchronous, pure cryptographic operations
- **Transports** - Asynchronous transaction submission and status tracking
- **Connections** - Network operations (RPC calls)

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        SIGNERS (sync)                           │
│  ┌─────────────┐  ┌──────────────────┐                          │
│  │MessageSigner│  │TransactionSigner │  Just sign bytes         │
│  └─────────────┘  └──────────────────┘                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      TRANSPORTS (async)                         │
│  ┌──────────────────┐  ┌──────────────────┐                     │
│  │  DirectTransport │  │  SquadsTransport │  Submit, wait, etc. │
│  │  (wraps signer)  │  │  (wraps signer)  │                     │
│  └──────────────────┘  └──────────────────┘                     │
└─────────────────────────────────────────────────────────────────┘
```

## Traits

### Signer Traits (Sync)

- `MessageSigner` - Sign arbitrary messages (off-chain, SIWS)
- `TransactionSigner` - Sign transaction messages

### Transport Trait (Async)

- `WalletTransport` - Async submission with status tracking
- `SubmitResult` - Captures signed, pending, or executed states

### Connection Trait

- `Connection` - Network operations (send, confirm, query)
- `RpcConnection` - Standard Solana RPC implementation (with `rpc` feature)

## Usage

```rust
use solana_actor::{
    TransactionSigner, WalletTransport, DirectTransport, SubmitResult,
};

// Any signer implementing TransactionSigner
let signer = /* KeypairSigner, LedgerSigner, etc. */;

// Wrap in a transport for async operations
let transport = DirectTransport::new(signer);

// Submit and get result
let result = transport.submit(&tx_message).await?;
match result {
    SubmitResult::Signed(sig) => println!("Signed: {}", sig),
    SubmitResult::Pending { .. } => println!("Awaiting approvals"),
    SubmitResult::Executed { signature, .. } => println!("Executed: {}", signature),
}
```

## Features

- `rpc` (default) - Include `RpcConnection` implementation

## Related Crates

- `solana-actor-keypair` - File/memory keypair signer
- `solana-actor-ledger` - Ledger hardware wallet signer
- `solana-actor-squads` - Squads multisig transport

## License

Apache-2.0
