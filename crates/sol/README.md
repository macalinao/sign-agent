# sol

[![Crates.io](https://img.shields.io/crates/v/sol.svg)](https://crates.io/crates/sol)
[![Downloads](https://img.shields.io/crates/d/sol.svg)](https://crates.io/crates/sol)
[![License](https://img.shields.io/crates/l/sol.svg)](https://github.com/macalinao/sign-agent/blob/master/LICENSE)

Simple and secure SOL transfer CLI using solana-keyring.

## Installation

```bash
cargo install sol
```

## Usage

### Check Balance

```bash
sol balance <ADDRESS_OR_LABEL>
sol balance my-wallet --rpc https://api.mainnet-beta.solana.com
```

### Transfer SOL

```bash
# Interactive (prompts for confirmation)
sol transfer --from my-wallet --to <DESTINATION> --amount 1.5

# Skip confirmation
sol transfer --from my-wallet --to <DESTINATION> --amount 1.5 -y

# Use keyring agent (no password prompt)
sol transfer --from my-wallet --to <DESTINATION> --amount 1.5 --use-agent
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
sol balance my-wallet

# Transfer with agent
sol transfer --from main --to 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU --amount 0.1 --use-agent

# Transfer on devnet
sol transfer --from dev-wallet --to <DEST> --amount 1 --rpc https://api.devnet.solana.com -y
```

## License

Apache-2.0
