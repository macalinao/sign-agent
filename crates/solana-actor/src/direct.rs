//! Direct transport implementation for regular signers.
//!
//! This module provides [`DirectTransport`], which wraps any [`TransactionSigner`]
//! to provide the [`WalletTransport`] interface. This is the simplest transport
//! that performs synchronous signing in a blocking task.

use std::time::Duration;

use async_trait::async_trait;
use solana_sdk::{pubkey::Pubkey, signature::Signature};

use crate::error::{SignerError, TransportError};
use crate::signer::TransactionSigner;
use crate::transport::{SubmitResult, WalletTransport};

/// Direct transport that wraps any [`TransactionSigner`].
///
/// This transport performs synchronous signing using `spawn_blocking` to avoid
/// blocking the async runtime. The result is always [`SubmitResult::Signed`]
/// since direct signing produces immediate signatures.
///
/// # Example
///
/// ```ignore
/// use solana_actor::{DirectTransport, WalletTransport};
/// use solana_actor_keypair::KeypairSigner;
///
/// let signer = KeypairSigner::generate();
/// let transport = DirectTransport::new(signer);
///
/// let result = transport.submit(&message_bytes).await?;
/// assert!(result.is_complete());
/// ```
#[derive(Debug, Clone)]
pub struct DirectTransport<S> {
    signer: S,
}

impl<S> DirectTransport<S> {
    /// Create a new direct transport wrapping the given signer.
    pub fn new(signer: S) -> Self {
        Self { signer }
    }

    /// Get a reference to the underlying signer.
    pub fn signer(&self) -> &S {
        &self.signer
    }

    /// Get a mutable reference to the underlying signer.
    pub fn signer_mut(&mut self) -> &mut S {
        &mut self.signer
    }

    /// Consume the transport and return the underlying signer.
    pub fn into_inner(self) -> S {
        self.signer
    }
}

#[async_trait]
impl<S> WalletTransport for DirectTransport<S>
where
    S: TransactionSigner + Clone + 'static,
{
    fn authority(&self) -> Pubkey {
        self.signer.pubkey()
    }

    async fn submit(&self, message: &[u8]) -> Result<SubmitResult, TransportError> {
        let message = message.to_vec();
        let signer = self.signer.clone();

        let result: Result<Signature, SignerError> =
            tokio::task::spawn_blocking(move || signer.sign_transaction(&message))
                .await
                .map_err(|_| TransportError::TaskPanic)?;

        Ok(SubmitResult::Signed(result?))
    }

    async fn check_status(&self, result: &SubmitResult) -> Result<SubmitResult, TransportError> {
        // Direct signing is always complete
        Ok(result.clone())
    }

    async fn wait_for_completion(
        &self,
        result: SubmitResult,
        _timeout: Duration,
    ) -> Result<SubmitResult, TransportError> {
        // Direct signing is always complete
        Ok(result)
    }

    fn requires_network(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    /// Mock signer for testing
    #[derive(Clone)]
    struct MockSigner {
        pubkey: Pubkey,
    }

    impl TransactionSigner for MockSigner {
        fn pubkey(&self) -> Pubkey {
            self.pubkey
        }

        fn sign_transaction(&self, _message: &[u8]) -> Result<Signature, SignerError> {
            Ok(Signature::default())
        }
    }

    #[tokio::test]
    async fn test_direct_transport_submit() {
        let signer = MockSigner {
            pubkey: Pubkey::new_unique(),
        };
        let transport = DirectTransport::new(signer);

        let result = transport.submit(b"test message").await.unwrap();
        assert!(matches!(result, SubmitResult::Signed(_)));
        assert!(result.is_complete());
        assert!(!result.is_pending());
    }

    #[tokio::test]
    async fn test_direct_transport_authority() {
        let pubkey = Pubkey::new_unique();
        let signer = MockSigner { pubkey };
        let transport = DirectTransport::new(signer);

        assert_eq!(transport.authority(), pubkey);
    }

    #[tokio::test]
    async fn test_direct_transport_check_status() {
        let signer = MockSigner {
            pubkey: Pubkey::new_unique(),
        };
        let transport = DirectTransport::new(signer);

        let result = SubmitResult::Signed(Signature::default());
        let checked = transport.check_status(&result).await.unwrap();
        assert!(checked.is_complete());
    }

    #[test]
    fn test_direct_transport_requires_network() {
        let signer = MockSigner {
            pubkey: Pubkey::new_unique(),
        };
        let transport = DirectTransport::new(signer);
        assert!(!transport.requires_network());
    }
}
