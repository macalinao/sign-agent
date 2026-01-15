# solana-credential-helper

[![Crates.io](https://img.shields.io/crates/v/solana-credential-helper.svg)](https://crates.io/crates/solana-credential-helper)
[![Downloads](https://img.shields.io/crates/d/solana-credential-helper.svg)](https://crates.io/crates/solana-credential-helper)
[![License](https://img.shields.io/crates/l/solana-credential-helper.svg)](https://github.com/macalinao/sign-agent/blob/master/LICENSE)

Solana transaction signing helper CLI for secure offline signing.

## Features

- **Stdin/Stdout Interface**: Pipe transactions for signing
- **Multiple Signer Types**: Keypair, Ledger, Squads multisig
- **Agent Integration**: Use with keyring agent for seamless signing
- **Base64/Base58 Support**: Flexible encoding options

## Installation

```bash
cargo install solana-credential-helper
```

## Usage

### Sign with Local Keypair

```bash
# Via agent (no password prompt)
echo "<TX_BASE64>" | solana-credential-helper sign-transaction \
  --signer <PUBKEY> \
  --use-agent

# Direct (prompts for passphrase)
echo "<TX_BASE64>" | solana-credential-helper sign-transaction \
  --signer <PUBKEY>
```

### Sign with Ledger

```bash
echo "<TX_BASE64>" | solana-credential-helper sign-transaction \
  --signer <PUBKEY> \
  --ledger
```

### Sign via Squads Multisig

```bash
echo "<TX_BASE64>" | solana-credential-helper sign-transaction \
  --signer <PUBKEY> \
  --squads <MULTISIG_ADDRESS> \
  --rpc-url https://api.mainnet-beta.solana.com
```

### Options

```
--encoding <base64|base58>  Input/output encoding (default: base64)
--use-agent                 Use keyring agent socket
--agent-socket <PATH>       Custom agent socket path
--db-path <PATH>            Custom database path
```

## Integration

This tool is designed to be called by other programs. The TypeScript package `@macalinao/solana-credential-helper` provides a convenient wrapper:

```typescript
import { createCredentialHelperSigner } from '@macalinao/solana-credential-helper';

const signer = createCredentialHelperSigner({
  publicKey: 'ABC123...',
  useAgent: true,
});
```

## License

Apache-2.0
