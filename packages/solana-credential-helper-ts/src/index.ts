/**
 * Solana Credential Helper TypeScript Library
 *
 * Provides a TransactionSendingSigner implementation that delegates
 * signing to the solana-credential-helper CLI tool.
 */

export { createCredentialHelperSigner } from "./signer.js";

export { type SignerType, type CredentialHelperSignerConfig } from "./types.js";
