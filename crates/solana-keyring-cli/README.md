# solana-keyring-cli

[![Crates.io](https://img.shields.io/crates/v/solana-keyring-cli.svg)](https://crates.io/crates/solana-keyring-cli)
[![Downloads](https://img.shields.io/crates/d/solana-keyring-cli.svg)](https://crates.io/crates/solana-keyring-cli)
[![License](https://img.shields.io/crates/l/solana-keyring-cli.svg)](https://github.com/macalinao/sign-agent/blob/master/LICENSE)

CLI tool for managing Solana keyring - generate, import, export, and organize keys.

## Installation

```bash
cargo install solana-keyring-cli
```

This installs the `solana-keyring` binary.

## Usage

### Initialize Keyring

```bash
solana-keyring new
```

### Keypair Management

```bash
# Generate a new keypair
solana-keyring generate --label my-wallet --tag main

# Import from JSON file
solana-keyring import --label imported --file keypair.json

# Import from base58 private key
solana-keyring import --label imported --base58 <PRIVATE_KEY>

# Export keypair
solana-keyring export my-wallet --format json
solana-keyring export my-wallet --format base58

# List all signers
solana-keyring list
solana-keyring list --type keypair --tag main --format json

# Delete a keypair
solana-keyring delete my-wallet
```

### Ledger Hardware Wallet

```bash
# Add Ledger wallet
solana-keyring ledger add --label my-ledger

# List Ledger wallets
solana-keyring ledger list

# Remove Ledger wallet
solana-keyring ledger remove my-ledger
```

### Squads Multisig

```bash
# Add Squads multisig
solana-keyring squads add <MULTISIG_ADDRESS> --label my-squad

# List multisigs
solana-keyring squads list

# Sync from chain
solana-keyring squads sync my-squad
```

### Tags

```bash
# Add tag to keypair
solana-keyring tag add my-wallet --tag production

# Remove tag
solana-keyring tag remove my-wallet --tag production

# List all tags
solana-keyring tag list
```

### Address Book

```bash
# Add address
solana-keyring address-book add <PUBKEY> --label "Exchange Hot Wallet"

# List addresses
solana-keyring address-book list

# Remove address
solana-keyring address-book remove "Exchange Hot Wallet"
```

## License

Apache-2.0
