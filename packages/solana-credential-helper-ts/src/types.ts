/**
 * Type of signer to use
 */
export type SignerType = "keypair" | "ledger" | "squads";

/**
 * Configuration for the credential helper signer
 */
export interface CredentialHelperSignerConfig {
  /** Public key of the signer (base58 encoded) */
  publicKey: string;

  /** Type of signer (default: 'keypair') */
  signerType?: SignerType;

  /** For squads: the multisig address */
  squadsAddress?: string;

  /** RPC URL for squads operations */
  rpcUrl?: string;

  /** Path to credential helper binary (default: 'solana-credential-helper') */
  binaryPath?: string;

  /** Use agent socket if available */
  useAgent?: boolean;

  /** Agent socket path (default: ~/.solana-keyring/agent.sock) */
  agentSocketPath?: string;

  /** Database path (default: ~/.solana-keyring/keyring.db) */
  dbPath?: string;
}
