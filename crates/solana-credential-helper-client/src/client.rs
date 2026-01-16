//! Credential helper client implementation.

use std::path::PathBuf;
use std::process::Stdio;

use base64::Engine;
use solana_sdk::signature::Signature;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::process::Command;

use crate::error::{Error, Result};
use crate::types::{CredentialHelperConfig, SignerType};

const DEFAULT_BINARY: &str = "solana-credential-helper";

/// Client for interacting with the Solana credential helper.
///
/// This client can sign transactions via:
/// - The agent daemon (Unix socket)
/// - The CLI tool (subprocess)
///
/// # Example
///
/// ```no_run
/// use solana_credential_helper_client::{CredentialHelperClient, CredentialHelperConfig};
///
/// # async fn example() -> solana_credential_helper_client::Result<()> {
/// let config = CredentialHelperConfig::new("ABC123...")
///     .use_agent(true);
///
/// let client = CredentialHelperClient::new(config);
/// let message_bytes = b"transaction message";
/// let signature = client.sign_transaction(message_bytes).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct CredentialHelperClient {
    config: CredentialHelperConfig,
}

impl CredentialHelperClient {
    /// Create a new credential helper client with the given configuration.
    pub fn new(config: CredentialHelperConfig) -> Self {
        Self { config }
    }

    /// Get the public key from the configuration.
    pub fn public_key(&self) -> &str {
        &self.config.public_key
    }

    /// Get the signer type from the configuration.
    pub fn signer_type(&self) -> SignerType {
        self.config.signer_type
    }

    /// Sign a transaction message.
    ///
    /// If `use_agent` is enabled in the config, attempts to sign via the agent socket.
    /// Otherwise, spawns the CLI tool to perform signing.
    ///
    /// # Arguments
    ///
    /// * `message_bytes` - The serialized transaction message to sign.
    ///
    /// # Returns
    ///
    /// The signature as a `Signature`.
    ///
    /// # Errors
    ///
    /// Returns an error if signing fails.
    pub async fn sign_transaction(&self, message_bytes: &[u8]) -> Result<Signature> {
        if self.config.use_agent {
            self.sign_via_agent(message_bytes).await
        } else {
            self.sign_via_cli(message_bytes).await
        }
    }

    /// Sign a transaction via the agent daemon socket.
    ///
    /// # Arguments
    ///
    /// * `message_bytes` - The serialized transaction message to sign.
    ///
    /// # Errors
    ///
    /// Returns an error if the agent connection fails or signing fails.
    pub async fn sign_via_agent(&self, message_bytes: &[u8]) -> Result<Signature> {
        let socket_path = self
            .config
            .agent_socket_path
            .clone()
            .unwrap_or_else(default_agent_socket_path);

        let mut stream = UnixStream::connect(&socket_path).await.map_err(|e| {
            Error::Connection(format!(
                "Failed to connect to agent at {}: {}",
                socket_path.display(),
                e
            ))
        })?;

        // Build request
        let request = serde_json::json!({
            "method": "SignTransaction",
            "params": {
                "transaction": base64::engine::general_purpose::STANDARD.encode(message_bytes),
                "signer": self.config.public_key
            }
        });
        let request_bytes = serde_json::to_vec(&request)?;

        // Send request (length-prefixed)
        stream
            .write_all(&(request_bytes.len() as u32).to_be_bytes())
            .await?;
        stream.write_all(&request_bytes).await?;

        // Read response (length-prefixed)
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await?;

        let response: serde_json::Value = serde_json::from_slice(&buf)?;

        if response["status"] == "error" {
            return Err(Error::Agent(
                response["message"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .to_string(),
            ));
        }

        // Decode signature
        let sig_b64 = response["result"]
            .as_str()
            .ok_or_else(|| Error::InvalidSignature("Missing result in response".to_string()))?;

        let sig_bytes = base64::engine::general_purpose::STANDARD.decode(sig_b64)?;
        let sig_array: [u8; 64] = sig_bytes.try_into().map_err(|v: Vec<u8>| {
            Error::InvalidSignature(format!("Expected 64 bytes, got {}", v.len()))
        })?;

        Ok(Signature::from(sig_array))
    }

    /// Sign a transaction via the CLI tool (subprocess).
    ///
    /// This spawns the `solana-credential-helper sign-transaction` command
    /// and passes the transaction via stdin.
    ///
    /// # Arguments
    ///
    /// * `message_bytes` - The serialized transaction message to sign.
    ///
    /// # Errors
    ///
    /// Returns an error if the CLI process fails.
    pub async fn sign_via_cli(&self, message_bytes: &[u8]) -> Result<Signature> {
        let binary = self
            .config
            .binary_path
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or(DEFAULT_BINARY);

        let args = self.build_cli_args();
        let tx_base64 = base64::engine::general_purpose::STANDARD.encode(message_bytes);

        let mut child = Command::new(binary)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Write transaction to stdin
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(tx_base64.as_bytes()).await?;
        }

        let output = child.wait_with_output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Cli {
                code: output.status.code().unwrap_or(-1),
                message: stderr.to_string(),
            });
        }

        // Decode signature from stdout
        let sig_b64 = String::from_utf8_lossy(&output.stdout);
        let sig_bytes = base64::engine::general_purpose::STANDARD.decode(sig_b64.trim())?;
        let sig_array: [u8; 64] = sig_bytes.try_into().map_err(|v: Vec<u8>| {
            Error::InvalidSignature(format!("Expected 64 bytes, got {}", v.len()))
        })?;

        Ok(Signature::from(sig_array))
    }

    /// Build CLI arguments from the configuration.
    fn build_cli_args(&self) -> Vec<String> {
        let mut args = vec![
            "sign-transaction".to_string(),
            "--encoding".to_string(),
            "base64".to_string(),
            "--signer".to_string(),
            self.config.public_key.clone(),
        ];

        match self.config.signer_type {
            SignerType::Ledger => {
                args.push("--ledger".to_string());
            }
            SignerType::Squads => {
                if let Some(ref addr) = self.config.squads_address {
                    args.push("--squads".to_string());
                    args.push(addr.clone());
                }
                if let Some(ref url) = self.config.rpc_url {
                    args.push("--rpc-url".to_string());
                    args.push(url.clone());
                }
            }
            SignerType::Keypair => {}
        }

        if let Some(ref path) = self.config.db_path {
            args.push("--db-path".to_string());
            args.push(path.to_string_lossy().to_string());
        }

        args
    }
}

/// Get the default agent socket path.
fn default_agent_socket_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".solana-keyring")
        .join("agent.sock")
}

/// Get the default database path.
pub fn default_db_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".solana-keyring")
        .join("keyring.db")
}

/// Get the default agent socket path (public API).
pub fn default_socket_path() -> PathBuf {
    default_agent_socket_path()
}
