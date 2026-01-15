# Solana Credential Helper

A secure credential management system for Solana with support for local keypairs, Ledger hardware wallets, and Squads multisig.

## Features

- **Encrypted Keyring**: Row-level AES-256-GCM encryption with Argon2id key derivation
- **Hardware Wallet Support**: Ledger device integration via USB/HID
- **Multisig Support**: Squads Protocol v4 integration for proposals and approvals
- **Agent Mode**: Unix socket daemon keeps keyring unlocked in memory
- **Biometric Auth**: TouchID confirmation on macOS before signing
- **Transaction Display**: Shows human-readable transaction details before signing
- **Tags & Labels**: Organize credentials with tags and labels
- **Address Book**: Store labeled addresses for easy reference
- **TypeScript SDK**: `TransactionSendingSigner` for `@solana/kit`

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/macalinao/sign-agent.git
cd sign-agent

# Build with Nix
devenv shell -- cargo build --release

# Or with cargo directly (requires OpenSSL)
cargo build --release

# Binaries are in target/release/
```

### Binaries

After building, you'll have four binaries:

- `solana-keyring` - Keyring management CLI
- `solana-keyring-agent` - Background agent daemon
- `solana-credential-helper` - Transaction signing CLI
- `sol` - Simple SOL transfer example CLI

## Quick Start

```bash
# Initialize a new keyring with master passphrase
solana-keyring new

# Generate a new keypair
solana-keyring generate --label "my-wallet"

# Import an existing keypair from JSON
solana-keyring import --label "imported" --file ~/my-keypair.json

# Import from base58
solana-keyring import --label "from-base58" --base58 "5abc..."

# List all signers
solana-keyring list

# Export a keypair
solana-keyring export my-wallet --format json
solana-keyring export my-wallet --format base58
```

## Keyring Commands

```bash
# Keypair management
solana-keyring new                              # Initialize keyring
solana-keyring generate --label NAME            # Generate keypair
solana-keyring import --label NAME [OPTIONS]    # Import keypair
solana-keyring export IDENTIFIER [OPTIONS]      # Export keypair
solana-keyring delete IDENTIFIER                # Delete keypair

# Organization
solana-keyring list [--type TYPE] [--tag TAG]   # List signers
solana-keyring label PUBKEY LABEL               # Update label
solana-keyring tag add PUBKEY TAG               # Add tag
solana-keyring tag remove PUBKEY TAG            # Remove tag

# Ledger hardware wallet
solana-keyring ledger add --label NAME          # Add Ledger wallet
solana-keyring ledger list                      # List Ledger wallets
solana-keyring ledger remove IDENTIFIER         # Remove Ledger

# Squads multisig
solana-keyring squads add ADDRESS --label NAME  # Add multisig
solana-keyring squads list                      # List multisigs
solana-keyring squads sync IDENTIFIER           # Sync from chain

# Address book
solana-keyring address-book add PUBKEY --label NAME
solana-keyring address-book list
solana-keyring address-book remove IDENTIFIER
```

## Agent Mode

The agent keeps your keyring unlocked in memory for seamless signing:

```bash
# Start the agent (runs in background)
solana-keyring-agent start

# Start in foreground with custom timeout
solana-keyring-agent start --foreground --lock-timeout 7200

# Check status
solana-keyring-agent status

# Stop the agent
solana-keyring-agent stop
```

The agent listens on `~/.solana-keyring/agent.sock`.

## Transaction Signing

Sign transactions via the credential helper CLI:

```bash
# Sign with a local keypair
echo "BASE64_TRANSACTION" | solana-credential-helper sign-transaction --signer PUBKEY

# Sign with Ledger
echo "BASE64_TRANSACTION" | solana-credential-helper sign-transaction --ledger --signer PUBKEY

# Sign via Squads (creates proposal)
echo "BASE64_TRANSACTION" | solana-credential-helper sign-transaction --squads MULTISIG_ADDR --signer PUBKEY

# Use the agent (no password prompt)
echo "BASE64_TRANSACTION" | solana-credential-helper sign-transaction --signer PUBKEY --use-agent
```

## SOL Transfer Example

The `sol` binary provides a simple way to transfer SOL using the keyring:

```bash
# Check balance
sol balance my-wallet --rpc https://api.devnet.solana.com

# Transfer SOL (prompts for passphrase)
sol transfer --from my-wallet --to RECIPIENT_PUBKEY --amount 0.1 --rpc https://api.devnet.solana.com

# Transfer using agent (no password prompt, triggers TouchID)
sol transfer --from my-wallet --to RECIPIENT_PUBKEY --amount 0.1 --use-agent --rpc https://api.devnet.solana.com

# Skip confirmation
sol transfer --from my-wallet --to RECIPIENT_PUBKEY --amount 0.1 --use-agent -y
```

## TypeScript SDK

Use with `@solana/kit` for programmatic signing:

```typescript
import { createCredentialHelperSigner } from '@macalinao/solana-credential-helper';

const signer = createCredentialHelperSigner({
  publicKey: 'ABC123...',
  signerType: 'keypair', // or 'ledger', 'squads'
  useAgent: true,
});

// Use with transactions
const signedTx = await signTransaction([signer], transaction);
```

## Security

### Encryption

- **Master passphrase**: Verified via Argon2id hash stored in database
- **Row-level encryption**: Each keypair encrypted with unique salt and nonce
- **Algorithm**: AES-256-GCM with 32-byte keys derived via Argon2id
- **Memory safety**: Sensitive data zeroized on drop via `zeroize` crate

### Agent Security

- Unix socket with `0600` permissions (owner-only access)
- Configurable auto-lock timeout (default: 1 hour)
- Passphrase stored in memory with `Zeroizing` wrapper

### Best Practices

1. Use a strong master passphrase
2. Enable agent timeout in production
3. Use Ledger for high-value accounts
4. Use Squads multisig for team/DAO funds

## Data Storage

```
~/.solana-keyring/
├── keyring.db      # Encrypted SQLite database
├── agent.sock      # Agent Unix socket (when running)
└── config.toml     # Optional configuration
```

## Development

```bash
# Enter dev environment
devenv shell

# Build
cargo build

# Test
cargo test

# Check
cargo clippy

# Format
cargo fmt
```

## Project Structure

```
sign-agent/
├── crates/
│   ├── solana-keyring/           # Core library
│   │   ├── crypto/               # Encryption (AES-GCM, Argon2)
│   │   ├── db/                   # SQLite storage
│   │   ├── keypair/              # Key management
│   │   ├── ledger/               # Hardware wallet (USB/HID)
│   │   ├── squads/               # Multisig (Squads v4)
│   │   ├── biometric.rs          # TouchID
│   │   └── transaction.rs        # TX parsing
│   ├── solana-keyring-cli/       # CLI tool
│   ├── solana-keyring-agent/     # Agent daemon
│   ├── solana-credential-helper/ # Signing CLI
│   └── sol/                      # SOL transfer example
└── packages/
    └── solana-credential-helper-ts/  # TypeScript SDK
```

## License

Apache-2.0
