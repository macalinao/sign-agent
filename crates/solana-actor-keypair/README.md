# solana-actor-keypair

Keypair-based signer for Solana with secure memory handling.

## Features

- **Secure memory handling** - Secret keys are automatically zeroized when dropped
- **Multiple input formats** - Load from files, bytes, or base58 encoding
- **Solana CLI compatible** - Works with standard Solana keypair JSON files
- **Trait implementations** - Implements `MessageSigner` and `TransactionSigner`

## Usage

### Generate a New Keypair

```rust
use solana_actor_keypair::KeypairSigner;
use solana_actor::TransactionSigner;

let signer = KeypairSigner::generate();
println!("Public key: {}", signer.pubkey());

let signature = signer.sign_transaction(b"message").unwrap();
```

### Load from File

```rust
use solana_actor_keypair::from_file;

let signer = from_file("~/.config/solana/id.json").unwrap();
println!("Loaded: {}", signer.pubkey());
```

### Load from Base58

```rust
use solana_actor_keypair::KeypairSigner;

let encoded = "...base58 encoded keypair...";
let signer = KeypairSigner::from_base58(encoded).unwrap();
```

### Use with DirectTransport

```rust
use solana_actor_keypair::KeypairSigner;
use solana_actor::{DirectTransport, WalletTransport};

let signer = KeypairSigner::generate();
let transport = DirectTransport::new(signer);

// Use transport.submit() for async operations
let result = transport.submit(&tx_message).await?;
```

## Security

The `KeypairSigner` struct implements `ZeroizeOnDrop`, which means the secret key
material is automatically overwritten with zeros when the signer is dropped.
This helps prevent secret key leakage through memory inspection.

## Related Crates

- `solana-actor` - Core traits
- `solana-actor-ledger` - Hardware wallet support
- `solana-actor-squads` - Multisig support

## License

Apache-2.0
