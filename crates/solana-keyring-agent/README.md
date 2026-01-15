# solana-keyring-agent

[![Crates.io](https://img.shields.io/crates/v/solana-keyring-agent.svg)](https://crates.io/crates/solana-keyring-agent)
[![Downloads](https://img.shields.io/crates/d/solana-keyring-agent.svg)](https://crates.io/crates/solana-keyring-agent)
[![License](https://img.shields.io/crates/l/solana-keyring-agent.svg)](https://github.com/macalinao/sign-agent/blob/master/LICENSE)

Agent daemon for Solana keyring that keeps keys unlocked in memory for seamless signing.

## Features

- **Persistent Unlock**: Keep keyring unlocked for a configurable timeout
- **Unix Socket IPC**: JSON-RPC protocol over Unix socket
- **Biometric Confirmation**: TouchID prompts for each signing request
- **Auto-lock**: Automatic locking after timeout period
- **Secure Memory**: Keys zeroized on lock/shutdown

## Installation

```bash
cargo install solana-keyring-agent
```

## Usage

```bash
# Start the agent daemon
solana-keyring-agent start

# Start in foreground with custom timeout
solana-keyring-agent start --foreground --lock-timeout 3600

# Check agent status
solana-keyring-agent status

# Unlock the keyring
solana-keyring-agent unlock

# Lock the keyring
solana-keyring-agent lock

# Stop the agent
solana-keyring-agent stop
```

## Socket Protocol

The agent listens on `~/.solana-keyring/agent.sock` and accepts JSON-RPC messages:

```json
{"method": "SignTransaction", "params": {"transaction": "<base64>", "signer": "<pubkey>"}}
{"method": "Unlock", "params": {"passphrase": "<passphrase>"}}
{"method": "Lock", "params": {}}
{"method": "Status", "params": {}}
```

## License

Apache-2.0
