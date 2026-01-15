import { spawn } from "node:child_process";
import type { SignatureDictionary, Transaction, TransactionPartialSigner } from "@solana/kit";
import { address } from "@solana/kit";
import type { CredentialHelperSignerConfig } from "./types.js";

/**
 * Creates a TransactionPartialSigner that delegates signing to the
 * solana-credential-helper CLI tool.
 *
 * @example
 * ```typescript
 * import { createCredentialHelperSigner } from '@macalinao/solana-credential-helper';
 *
 * const signer = createCredentialHelperSigner({
 *   publicKey: 'ABC123...',
 *   signerType: 'keypair',
 *   useAgent: true,
 * });
 *
 * // Use with @solana/kit
 * const signedTx = await signTransaction([signer], transaction);
 * ```
 */
export function createCredentialHelperSigner(
  config: CredentialHelperSignerConfig
): TransactionPartialSigner<string> {
  const publicKeyAddress = address(config.publicKey);
  const binaryPath = config.binaryPath ?? "solana-credential-helper";

  return {
    address: publicKeyAddress,

    async signTransactions(
      transactions: readonly Transaction[]
    ): Promise<readonly SignatureDictionary[]> {
      const results: SignatureDictionary[] = [];

      for (const tx of transactions) {
        // Encode the transaction message bytes as base64
        const txBase64 = Buffer.from(tx.messageBytes).toString("base64");

        const args = buildArgs(config);
        const signatureBase64 = await runCredentialHelper(
          binaryPath,
          args,
          txBase64
        );

        // Decode the signature
        const signatureBytes = new Uint8Array(
          Buffer.from(signatureBase64.trim(), "base64")
        );

        // Create signature dictionary with our address
        // The SignatureDictionary maps Address to SignatureBytes
        const sigDict = {
          [config.publicKey]: signatureBytes,
        } as SignatureDictionary;

        results.push(sigDict);
      }

      return results;
    },
  };
}

/**
 * Build CLI arguments from config
 */
function buildArgs(config: CredentialHelperSignerConfig): string[] {
  const args = [
    "sign-transaction",
    "--encoding",
    "base64",
    "--signer",
    config.publicKey,
  ];

  if (config.signerType === "ledger") {
    args.push("--ledger");
  } else if (config.signerType === "squads" && config.squadsAddress) {
    args.push("--squads", config.squadsAddress);
    if (config.rpcUrl) {
      args.push("--rpc-url", config.rpcUrl);
    }
  }

  if (config.useAgent) {
    args.push("--use-agent");
    if (config.agentSocketPath) {
      args.push("--agent-socket", config.agentSocketPath);
    }
  }

  if (config.dbPath) {
    args.push("--db-path", config.dbPath);
  }

  return args;
}

/**
 * Run the credential helper CLI and return the output
 */
function runCredentialHelper(
  binaryPath: string,
  args: string[],
  stdin: string
): Promise<string> {
  return new Promise((resolve, reject) => {
    const proc = spawn(binaryPath, args, {
      stdio: ["pipe", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";

    proc.stdout.on("data", (data: Buffer) => {
      stdout += data.toString();
    });

    proc.stderr.on("data", (data: Buffer) => {
      stderr += data.toString();
    });

    proc.on("close", (code) => {
      if (code === 0) {
        resolve(stdout);
      } else {
        reject(
          new Error(
            `solana-credential-helper exited with code ${code}: ${stderr}`
          )
        );
      }
    });

    proc.on("error", (err) => {
      reject(
        new Error(`Failed to spawn solana-credential-helper: ${err.message}`)
      );
    });

    proc.stdin.write(stdin);
    proc.stdin.end();
  });
}
