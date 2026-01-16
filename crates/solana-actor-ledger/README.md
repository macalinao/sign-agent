# solana-actor-ledger

Ledger hardware wallet signer for Solana.

## Features

- **Hardware security** - Private keys never leave the Ledger device
- **User confirmation** - All signing requires physical button press
- **BIP-44 paths** - Standard derivation path support
- **Trait implementations** - Implements `MessageSigner` and `TransactionSigner`

## Requirements

- Ledger Nano S/X/S Plus with Solana app installed
- Solana app must be opened on the device
- USB connection to the device

## Usage

### Connect with Default Path

```rust
use solana_actor_ledger::LedgerSigner;
use solana_actor::TransactionSigner;

// Connect with default derivation path (44'/501'/0'/0')
let signer = LedgerSigner::connect()?;
println!("Ledger pubkey: {}", signer.pubkey_base58());

// Sign a transaction (user must confirm on device)
let signature = signer.sign_transaction(&tx_message)?;
```

### Custom Derivation Path

```rust
use solana_actor_ledger::LedgerSigner;

// Use a different account index
let signer = LedgerSigner::connect_with_path("44'/501'/1'/0'")?;
```

### With DirectTransport

```rust
use solana_actor_ledger::LedgerSigner;
use solana_actor::{DirectTransport, WalletTransport};

let signer = LedgerSigner::connect()?;
let transport = DirectTransport::new(signer);

// The transport will use spawn_blocking for the signing operation
let result = transport.submit(&tx_message).await?;
```

## Security

The `LedgerSigner` communicates with the Ledger device over USB HID. All
cryptographic operations happen on the device itself - private keys never
leave the secure element.

The `is_interactive()` method returns `true` to indicate that signing requires
user interaction (physical button press on the device).

## Derivation Paths

Default path: `44'/501'/0'/0'` (BIP-44 for Solana)

The path format supports:
- `'` or `h` for hardened derivation
- Optional `m/` prefix
- Multiple account indices (change the third component)

## Related Crates

- `solana-actor` - Core traits
- `solana-actor-keypair` - Software keypair signer
- `solana-actor-squads` - Multisig support

## License

Apache-2.0
