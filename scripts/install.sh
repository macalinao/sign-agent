#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Installing solana-keyring tools..."

# Install all binary crates
cargo install --path "$ROOT_DIR/crates/solana-keyring-cli"
cargo install --path "$ROOT_DIR/crates/solana-keyring-agent"
cargo install --path "$ROOT_DIR/crates/solana-credential-helper"
cargo install --path "$ROOT_DIR/crates/solite"

echo ""
echo "Installed binaries:"
echo "  solana-keyring          - Keyring management CLI"
echo "  solana-keyring-agent    - Agent daemon for unlocked signing"
echo "  solana-credential-helper - Transaction signing helper"
echo "  solite                  - Lightweight Solana CLI"
