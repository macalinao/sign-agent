# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/macalinao/sign-agent/releases/tag/solana-keyring-cli-v0.1.0) - 2026-01-16

### Added

- add solana-actor crate family for unified signing traits

### Fixed

- *(cli)* provide specific error messages for agent status
- *(keyring-cli)* try agent by default, add --no-agent to disable

### Other

- Add --use-agent flag to solana-keyring CLI
- Use independent crate versions and add rust-toolchain.toml
- Add crate metadata and README files for crates.io
- Initial commit: Solana Credential Helper
