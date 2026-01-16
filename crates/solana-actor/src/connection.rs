//! Connection traits and implementations for network operations.
//!
//! This module defines the [`Connection`] trait for interacting with Solana RPC nodes.
//! When the `rpc` feature is enabled, it also provides [`RpcConnection`] which wraps
//! the standard Solana RPC client.

use async_trait::async_trait;
use solana_sdk::{
    account::Account, hash::Hash, pubkey::Pubkey, signature::Signature, transaction::Transaction,
};

use crate::error::ConnectionError;

/// Configuration for transaction sending.
#[derive(Debug, Clone, Default)]
pub struct SendConfig {
    /// Skip preflight transaction checks.
    pub skip_preflight: bool,
    /// Maximum number of retries for sending.
    pub max_retries: Option<usize>,
}

/// Trait for network connection operations.
///
/// This trait abstracts over RPC client implementations, allowing for
/// testing with mock connections and supporting different RPC backends.
///
/// # Example
///
/// ```ignore
/// use solana_actor::Connection;
///
/// async fn get_sol_balance<C: Connection>(conn: &C, pubkey: &Pubkey) -> f64 {
///     let lamports = conn.get_balance(pubkey).await.unwrap();
///     lamports as f64 / 1_000_000_000.0
/// }
/// ```
#[async_trait]
pub trait Connection: Send + Sync {
    /// Send a transaction to the network.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The signed transaction to send.
    /// * `config` - Configuration options for sending.
    ///
    /// # Errors
    ///
    /// Returns [`ConnectionError`] if the transaction cannot be sent.
    async fn send_transaction(
        &self,
        transaction: &Transaction,
        config: SendConfig,
    ) -> Result<Signature, ConnectionError>;

    /// Send a transaction and wait for confirmation.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The signed transaction to send.
    /// * `config` - Configuration options for sending.
    ///
    /// # Errors
    ///
    /// Returns [`ConnectionError`] if the transaction fails or times out.
    async fn send_and_confirm(
        &self,
        transaction: &Transaction,
        config: SendConfig,
    ) -> Result<Signature, ConnectionError>;

    /// Get the latest blockhash.
    ///
    /// # Errors
    ///
    /// Returns [`ConnectionError`] if the RPC call fails.
    async fn get_latest_blockhash(&self) -> Result<Hash, ConnectionError>;

    /// Get the balance of an account in lamports.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - The account to query.
    ///
    /// # Errors
    ///
    /// Returns [`ConnectionError`] if the RPC call fails.
    async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64, ConnectionError>;

    /// Get account information.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - The account to query.
    ///
    /// # Returns
    ///
    /// `None` if the account doesn't exist, `Some(Account)` otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`ConnectionError`] if the RPC call fails.
    async fn get_account(&self, pubkey: &Pubkey) -> Result<Option<Account>, ConnectionError>;
}

#[cfg(feature = "rpc")]
mod rpc_impl {
    use super::*;
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_client::rpc_config::RpcSendTransactionConfig;
    use solana_commitment_config::CommitmentConfig;

    /// RPC-based connection implementation.
    ///
    /// This wraps the standard Solana [`RpcClient`] to implement the [`Connection`] trait.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solana_actor::RpcConnection;
    ///
    /// let conn = RpcConnection::new("https://api.mainnet-beta.solana.com");
    /// let balance = conn.get_balance(&my_pubkey).await?;
    /// ```
    pub struct RpcConnection {
        client: RpcClient,
    }

    impl RpcConnection {
        /// Create a new RPC connection with default commitment (confirmed).
        pub fn new(url: &str) -> Self {
            Self {
                client: RpcClient::new(url.to_string()),
            }
        }

        /// Create a new RPC connection with a specific commitment level.
        pub fn new_with_commitment(url: &str, commitment: CommitmentConfig) -> Self {
            Self {
                client: RpcClient::new_with_commitment(url.to_string(), commitment),
            }
        }

        /// Get a reference to the underlying RPC client.
        pub fn client(&self) -> &RpcClient {
            &self.client
        }
    }

    #[async_trait]
    impl Connection for RpcConnection {
        async fn send_transaction(
            &self,
            transaction: &Transaction,
            config: SendConfig,
        ) -> Result<Signature, ConnectionError> {
            let rpc_config = RpcSendTransactionConfig {
                skip_preflight: config.skip_preflight,
                max_retries: config.max_retries,
                ..Default::default()
            };
            self.client
                .send_transaction_with_config(transaction, rpc_config)
                .await
                .map_err(|e| ConnectionError::Rpc(e.to_string()))
        }

        async fn send_and_confirm(
            &self,
            transaction: &Transaction,
            _config: SendConfig,
        ) -> Result<Signature, ConnectionError> {
            self.client
                .send_and_confirm_transaction(transaction)
                .await
                .map_err(|e| ConnectionError::Rpc(e.to_string()))
        }

        async fn get_latest_blockhash(&self) -> Result<Hash, ConnectionError> {
            self.client
                .get_latest_blockhash()
                .await
                .map_err(|e| ConnectionError::Rpc(e.to_string()))
        }

        async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64, ConnectionError> {
            self.client
                .get_balance(pubkey)
                .await
                .map_err(|e| ConnectionError::Rpc(e.to_string()))
        }

        async fn get_account(&self, pubkey: &Pubkey) -> Result<Option<Account>, ConnectionError> {
            match self.client.get_account(pubkey).await {
                Ok(account) => Ok(Some(account)),
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("AccountNotFound")
                        || err_str.contains("could not find account")
                    {
                        Ok(None)
                    } else {
                        Err(ConnectionError::Rpc(err_str))
                    }
                }
            }
        }
    }
}

#[cfg(feature = "rpc")]
pub use rpc_impl::RpcConnection;
