//! Squads multisig transport implementation.

use std::time::{Duration, Instant};

use async_trait::async_trait;
use solana_actor::{SubmitResult, TransactionSigner, TransportError, WalletTransport};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    instruction::AccountMeta, pubkey::Pubkey, signature::Signature, signer::Signer,
    transaction::Transaction,
};

use crate::SQUADS_PROGRAM_ID;
use crate::error::{Result, SquadsError};
use crate::instructions::{
    ProposalCreateArgs, ProposalVoteArgs, VaultTransactionCreateArgs, proposal_approve,
    proposal_create, vault_transaction_create, vault_transaction_execute,
};
use crate::pda::{get_proposal_pda, get_transaction_pda, get_vault_pda};

/// Squads multisig transport.
///
/// This transport creates on-chain proposals for transactions rather than
/// producing direct cryptographic signatures. It requires multiple approvals
/// to reach threshold before execution.
///
/// # Note
///
/// This type does NOT implement [`TransactionSigner`] or [`MessageSigner`]
/// because multisig operations don't produce direct signatures - they create
/// on-chain proposals that need to be approved by multiple parties.
///
/// # Example
///
/// ```ignore
/// use solana_actor_squads::SquadsTransport;
/// use solana_actor_keypair::KeypairSigner;
/// use solana_actor::{WalletTransport, SubmitResult};
///
/// let member = KeypairSigner::from_file("member.json")?;
/// let transport = SquadsTransport::new(
///     "MULTISIG_ADDRESS".parse()?,
///     0, // vault_index
///     "https://api.mainnet-beta.solana.com",
///     member,
/// )?;
///
/// // Submit creates a proposal and approves with the member key
/// let result = transport.submit(&tx_message).await?;
///
/// match result {
///     SubmitResult::Executed { signature, .. } => {
///         println!("Executed: {}", signature);
///     }
///     SubmitResult::Pending { approvals, threshold, .. } => {
///         println!("Pending: {}/{} approvals", approvals, threshold);
///     }
///     _ => {}
/// }
/// ```
pub struct SquadsTransport<S: TransactionSigner> {
    multisig: Pubkey,
    vault_index: u8,
    vault_pda: Pubkey,
    rpc_client: RpcClient,
    member: S,
    program_id: Pubkey,
}

impl<S: TransactionSigner> SquadsTransport<S> {
    /// Create a new Squads transport.
    ///
    /// # Arguments
    ///
    /// * `multisig` - The multisig account public key.
    /// * `vault_index` - The vault index (usually 0).
    /// * `rpc_url` - URL of the Solana RPC endpoint.
    /// * `member` - A signer for a member of the multisig.
    ///
    /// # Errors
    ///
    /// Returns an error if the program ID cannot be parsed.
    pub fn new(multisig: Pubkey, vault_index: u8, rpc_url: &str, member: S) -> Result<Self> {
        let program_id: Pubkey = SQUADS_PROGRAM_ID
            .parse()
            .map_err(|_| SquadsError::InvalidAddress("Invalid program ID".into()))?;

        let vault_pda = get_vault_pda(&multisig, vault_index, &program_id);
        let rpc_client = RpcClient::new(rpc_url.to_string());

        Ok(Self {
            multisig,
            vault_index,
            vault_pda,
            rpc_client,
            member,
            program_id,
        })
    }

    /// Get the multisig account address.
    pub fn multisig(&self) -> Pubkey {
        self.multisig
    }

    /// Get the vault PDA (the actual authority for transactions).
    pub fn vault_pda(&self) -> Pubkey {
        self.vault_pda
    }

    /// Get the vault index.
    pub fn vault_index(&self) -> u8 {
        self.vault_index
    }

    /// Get a reference to the member signer.
    pub fn member(&self) -> &S {
        &self.member
    }

    /// Get the program ID.
    pub fn program_id(&self) -> Pubkey {
        self.program_id
    }

    /// Create a proposal for a transaction.
    async fn create_proposal(&self, transaction_message: &[u8]) -> Result<(Pubkey, u64)> {
        let member_pubkey = self.member.pubkey();

        // Get the current transaction index from the multisig account
        let multisig_data = self
            .rpc_client
            .get_account_data(&self.multisig)
            .map_err(|e| SquadsError::Rpc(format!("Failed to fetch multisig: {}", e)))?;

        // Parse transaction_index from multisig account data
        // Offset = 8 (discriminator) + 32 (create_key) + 32 (config_authority) + 2 (threshold) + 4 (time_lock) = 78
        const TX_INDEX_OFFSET: usize = 78;

        if multisig_data.len() < TX_INDEX_OFFSET + 8 {
            return Err(SquadsError::InvalidAccountData(
                "Multisig account too small".into(),
            ));
        }

        let transaction_index = u64::from_le_bytes(
            multisig_data[TX_INDEX_OFFSET..TX_INDEX_OFFSET + 8]
                .try_into()
                .map_err(|_| SquadsError::InvalidAccountData("Failed to parse tx index".into()))?,
        );
        let next_index = transaction_index + 1;

        // Derive PDAs for the new transaction and proposal
        let transaction_pda = get_transaction_pda(&self.multisig, next_index, &self.program_id);
        let proposal_pda = get_proposal_pda(&self.multisig, next_index, &self.program_id);

        // Build vault transaction create instruction
        let vault_tx_args = VaultTransactionCreateArgs {
            vault_index: self.vault_index,
            ephemeral_signers: 0,
            transaction_message: transaction_message.to_vec(),
            memo: None,
        };

        let vault_tx_ix = vault_transaction_create(
            self.multisig,
            transaction_pda,
            member_pubkey,
            member_pubkey,
            vault_tx_args,
            self.program_id,
        );

        // Build proposal create instruction
        let proposal_args = ProposalCreateArgs {
            transaction_index: next_index,
            draft: false,
        };

        let proposal_ix = proposal_create(
            self.multisig,
            proposal_pda,
            member_pubkey,
            member_pubkey,
            proposal_args,
            self.program_id,
        );

        // Get recent blockhash and sign
        let blockhash = self
            .rpc_client
            .get_latest_blockhash()
            .map_err(|e| SquadsError::Rpc(format!("Failed to get blockhash: {}", e)))?;

        let mut tx = Transaction::new_with_payer(&[vault_tx_ix, proposal_ix], Some(&member_pubkey));
        tx.partial_sign(&[&MemberSigner(&self.member)], blockhash);

        // Send transaction
        self.rpc_client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &tx,
                CommitmentConfig::confirmed(),
            )
            .map_err(|e| SquadsError::ProposalCreation(e.to_string()))?;

        Ok((proposal_pda, next_index))
    }

    /// Approve a proposal with the member key.
    async fn approve_proposal(&self, transaction_index: u64) -> Result<()> {
        let member_pubkey = self.member.pubkey();
        let proposal_pda = get_proposal_pda(&self.multisig, transaction_index, &self.program_id);

        let vote_args = ProposalVoteArgs { memo: None };

        let approve_ix = proposal_approve(
            self.multisig,
            proposal_pda,
            member_pubkey,
            vote_args,
            self.program_id,
        );

        let blockhash = self
            .rpc_client
            .get_latest_blockhash()
            .map_err(|e| SquadsError::Rpc(format!("Failed to get blockhash: {}", e)))?;

        let mut tx = Transaction::new_with_payer(&[approve_ix], Some(&member_pubkey));
        tx.partial_sign(&[&MemberSigner(&self.member)], blockhash);

        self.rpc_client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &tx,
                CommitmentConfig::confirmed(),
            )
            .map_err(|e| SquadsError::Approval(e.to_string()))?;

        Ok(())
    }

    /// Execute a proposal that has reached threshold.
    async fn execute_proposal(&self, transaction_index: u64) -> Result<Signature> {
        let member_pubkey = self.member.pubkey();
        let proposal_pda = get_proposal_pda(&self.multisig, transaction_index, &self.program_id);
        let transaction_pda =
            get_transaction_pda(&self.multisig, transaction_index, &self.program_id);

        // Fetch the vault transaction account to get the accounts list
        let tx_data = self
            .rpc_client
            .get_account_data(&transaction_pda)
            .map_err(|e| SquadsError::Rpc(format!("Failed to fetch transaction: {}", e)))?;

        let remaining_accounts = parse_vault_transaction_accounts(&tx_data, self.vault_pda)?;

        let execute_ix = vault_transaction_execute(
            self.multisig,
            proposal_pda,
            transaction_pda,
            member_pubkey,
            remaining_accounts,
            self.program_id,
        );

        let blockhash = self
            .rpc_client
            .get_latest_blockhash()
            .map_err(|e| SquadsError::Rpc(format!("Failed to get blockhash: {}", e)))?;

        let mut tx = Transaction::new_with_payer(&[execute_ix], Some(&member_pubkey));
        tx.partial_sign(&[&MemberSigner(&self.member)], blockhash);

        let signature = self
            .rpc_client
            .send_and_confirm_transaction_with_spinner_and_commitment(
                &tx,
                CommitmentConfig::confirmed(),
            )
            .map_err(|e| SquadsError::Execution(e.to_string()))?;

        Ok(signature)
    }

    /// Get the current proposal state.
    async fn get_proposal_state(&self, transaction_index: u64) -> Result<ProposalState> {
        let proposal_pda = get_proposal_pda(&self.multisig, transaction_index, &self.program_id);

        let proposal_data = self
            .rpc_client
            .get_account_data(&proposal_pda)
            .map_err(|e| SquadsError::Rpc(format!("Failed to fetch proposal: {}", e)))?;

        parse_proposal_state(&proposal_data)
    }

    /// Get the multisig threshold.
    fn get_threshold(&self) -> Result<u32> {
        let multisig_data = self
            .rpc_client
            .get_account_data(&self.multisig)
            .map_err(|e| SquadsError::Rpc(format!("Failed to fetch multisig: {}", e)))?;

        // Threshold offset = 8 (discriminator) + 32 (create_key) + 32 (config_authority) = 72
        const THRESHOLD_OFFSET: usize = 72;

        if multisig_data.len() < THRESHOLD_OFFSET + 2 {
            return Err(SquadsError::InvalidAccountData("Multisig too small".into()));
        }

        let threshold = u16::from_le_bytes(
            multisig_data[THRESHOLD_OFFSET..THRESHOLD_OFFSET + 2]
                .try_into()
                .map_err(|_| SquadsError::InvalidAccountData("Failed to parse threshold".into()))?,
        );

        Ok(threshold as u32)
    }
}

#[async_trait]
impl<S: TransactionSigner + Clone + Send + Sync + 'static> WalletTransport for SquadsTransport<S> {
    fn authority(&self) -> Pubkey {
        self.vault_pda
    }

    async fn submit(&self, message: &[u8]) -> std::result::Result<SubmitResult, TransportError> {
        // 1. Create proposal
        let (proposal, tx_index) = self.create_proposal(message).await?;

        // 2. Approve with member signer
        self.approve_proposal(tx_index).await?;

        // 3. Check if we can execute
        let state = self.get_proposal_state(tx_index).await?;
        let threshold = self.get_threshold()?;

        if state.can_execute(threshold) {
            let sig = self.execute_proposal(tx_index).await?;
            Ok(SubmitResult::Executed {
                signature: sig,
                proposal,
            })
        } else {
            Ok(SubmitResult::Pending {
                proposal,
                transaction_index: tx_index,
                approvals: state.approval_count,
                threshold,
            })
        }
    }

    async fn check_status(
        &self,
        result: &SubmitResult,
    ) -> std::result::Result<SubmitResult, TransportError> {
        let SubmitResult::Pending {
            proposal,
            transaction_index,
            ..
        } = result
        else {
            return Ok(result.clone());
        };

        let state = self.get_proposal_state(*transaction_index).await?;
        let threshold = self.get_threshold()?;

        if state.is_executed {
            // If executed, we need to find the execution signature
            // For now, return executed with a default signature
            Ok(SubmitResult::Executed {
                signature: Signature::default(),
                proposal: *proposal,
            })
        } else {
            Ok(SubmitResult::Pending {
                proposal: *proposal,
                transaction_index: *transaction_index,
                approvals: state.approval_count,
                threshold,
            })
        }
    }

    async fn wait_for_completion(
        &self,
        result: SubmitResult,
        timeout: Duration,
    ) -> std::result::Result<SubmitResult, TransportError> {
        if result.is_complete() {
            return Ok(result);
        }

        let deadline = Instant::now() + timeout;
        let mut current = result;

        while Instant::now() < deadline {
            current = self.check_status(&current).await?;
            if current.is_complete() {
                return Ok(current);
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        Err(TransportError::Timeout)
    }

    fn requires_network(&self) -> bool {
        true
    }
}

/// Helper to wrap a TransactionSigner as a solana_sdk::signer::Signer.
struct MemberSigner<'a, S: TransactionSigner>(&'a S);

impl<S: TransactionSigner> Signer for MemberSigner<'_, S> {
    fn pubkey(&self) -> Pubkey {
        TransactionSigner::pubkey(self.0)
    }

    fn try_pubkey(&self) -> std::result::Result<Pubkey, solana_sdk::signer::SignerError> {
        Ok(TransactionSigner::pubkey(self.0))
    }

    fn sign_message(&self, message: &[u8]) -> Signature {
        self.0
            .sign_transaction(message)
            .expect("signing should succeed")
    }

    fn try_sign_message(
        &self,
        message: &[u8],
    ) -> std::result::Result<Signature, solana_sdk::signer::SignerError> {
        self.0
            .sign_transaction(message)
            .map_err(|e| solana_sdk::signer::SignerError::Custom(e.to_string()))
    }

    fn is_interactive(&self) -> bool {
        TransactionSigner::is_interactive(self.0)
    }
}

/// Parsed proposal state.
struct ProposalState {
    approval_count: u32,
    is_executed: bool,
}

impl ProposalState {
    fn can_execute(&self, threshold: u32) -> bool {
        self.approval_count >= threshold && !self.is_executed
    }
}

/// Parse proposal state from account data.
fn parse_proposal_state(data: &[u8]) -> Result<ProposalState> {
    // Proposal struct layout (after 8-byte Anchor discriminator):
    // - multisig: Pubkey (32)
    // - transaction_index: u64 (8)
    // - status: ProposalStatus (1 byte enum)
    // - bump: u8 (1)
    // - approved: Vec<Pubkey> (4 + 32*n)
    // - rejected: Vec<Pubkey> (4 + 32*n)
    // - cancelled: Vec<Pubkey> (4 + 32*n)

    const STATUS_OFFSET: usize = 8 + 32 + 8;
    const APPROVED_OFFSET: usize = STATUS_OFFSET + 1 + 1;

    if data.len() < APPROVED_OFFSET + 4 {
        return Err(SquadsError::InvalidAccountData("Proposal too small".into()));
    }

    let status = data[STATUS_OFFSET];
    let is_executed = status == 3; // Executed status

    let approval_count = u32::from_le_bytes(
        data[APPROVED_OFFSET..APPROVED_OFFSET + 4]
            .try_into()
            .map_err(|_| SquadsError::InvalidAccountData("Failed to parse approvals".into()))?,
    );

    Ok(ProposalState {
        approval_count,
        is_executed,
    })
}

/// Parse the remaining accounts needed for execution from the vault transaction data.
fn parse_vault_transaction_accounts(tx_data: &[u8], vault_pda: Pubkey) -> Result<Vec<AccountMeta>> {
    // VaultTransaction struct layout (after 8-byte Anchor discriminator):
    // - multisig: Pubkey (32)
    // - creator: Pubkey (32)
    // - index: u64 (8)
    // - bump: u8 (1)
    // - vault_index: u8 (1)
    // - vault_bump: u8 (1)
    // - ephemeral_signer_bumps: Vec<u8> (4 + n)
    // - message: TransactionMessage (variable)

    const FIXED_SIZE: usize = 8 + 32 + 32 + 8 + 1 + 1 + 1;

    if tx_data.len() < FIXED_SIZE {
        return Err(SquadsError::InvalidAccountData(
            "Vault transaction too small".into(),
        ));
    }

    // The vault is always needed as a signer for execution
    let mut accounts = vec![AccountMeta::new(vault_pda, false)];

    // Skip the fixed fields to get to ephemeral_signer_bumps
    let mut offset = FIXED_SIZE;

    if offset + 4 > tx_data.len() {
        return Ok(accounts);
    }

    // Read ephemeral_signer_bumps length
    let ephemeral_len = u32::from_le_bytes(
        tx_data[offset..offset + 4]
            .try_into()
            .map_err(|_| SquadsError::InvalidAccountData("Failed to parse ephemeral len".into()))?,
    ) as usize;
    offset += 4 + ephemeral_len;

    // Now we're at the TransactionMessage
    if offset + 3 > tx_data.len() {
        return Ok(accounts);
    }

    let num_signers = tx_data[offset] as usize;
    let num_writable_signers = tx_data[offset + 1] as usize;
    let num_writable_non_signers = tx_data[offset + 2] as usize;
    offset += 3;

    if offset + 4 > tx_data.len() {
        return Ok(accounts);
    }

    // Read account_keys length
    let num_keys = u32::from_le_bytes(
        tx_data[offset..offset + 4]
            .try_into()
            .map_err(|_| SquadsError::InvalidAccountData("Failed to parse keys len".into()))?,
    ) as usize;
    offset += 4;

    // Read each account key
    for i in 0..num_keys {
        if offset + 32 > tx_data.len() {
            break;
        }

        let key_bytes: [u8; 32] = tx_data[offset..offset + 32]
            .try_into()
            .map_err(|_| SquadsError::InvalidAccountData("Failed to parse key".into()))?;

        let pubkey = Pubkey::new_from_array(key_bytes);

        // Skip the vault itself
        if pubkey != vault_pda {
            let is_signer = i < num_signers;
            let is_writable = if is_signer {
                i < num_writable_signers
            } else {
                i < num_signers + num_writable_non_signers
            };

            if is_writable {
                accounts.push(AccountMeta::new(pubkey, false));
            } else {
                accounts.push(AccountMeta::new_readonly(pubkey, false));
            }
        }

        offset += 32;
    }

    Ok(accounts)
}
