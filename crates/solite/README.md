# solite

[![Crates.io](https://img.shields.io/crates/v/solite.svg)](https://crates.io/crates/solite)
[![Downloads](https://img.shields.io/crates/d/solite.svg)](https://crates.io/crates/solite)
[![License](https://img.shields.io/crates/l/solite.svg)](https://github.com/macalinao/sign-agent/blob/master/LICENSE)

A lightweight CLI for interacting with Solana.

## Installation

```bash
cargo install solite
```

## Usage

### Check Balance

```bash
solite balance <ADDRESS_OR_LABEL>
solite balance my-wallet --rpc https://api.mainnet-beta.solana.com
```

### Transfer SOL

```bash
# Interactive (prompts for confirmation)
solite transfer --from my-wallet --to <DESTINATION> --amount 1.5

# Skip confirmation
solite transfer --from my-wallet --to <DESTINATION> --amount 1.5 -y

# Use keyring agent (no password prompt)
solite transfer --from my-wallet --to <DESTINATION> --amount 1.5 --use-agent
```

### Options

```
--rpc <URL>           RPC endpoint (default: mainnet)
--use-agent           Use keyring agent for signing
--agent-socket <PATH> Custom agent socket path
--db-path <PATH>      Custom keyring database path
-y, --yes             Skip confirmation prompt
```

## Examples

```bash
# Check balance of labeled wallet
solite balance my-wallet

# Transfer with agent
solite transfer --from main --to 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU --amount 0.1 --use-agent

# Transfer on devnet
solite transfer --from dev-wallet --to <DEST> --amount 1 --rpc https://api.devnet.solana.com -y
```

## License

Apache-2.0
