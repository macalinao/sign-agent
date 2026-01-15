//! Unified signer interface

use serde::{Deserialize, Serialize};

use crate::db::Database;
use crate::error::{Error, Result};
use crate::keypair::SecureKeypair;

/// Type of signer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignerType {
    /// Local encrypted keypair
    Keypair,
    /// Ledger hardware wallet
    Ledger,
    /// Squads multisig
    Squads,
}

impl std::fmt::Display for SignerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignerType::Keypair => write!(f, "keypair"),
            SignerType::Ledger => write!(f, "ledger"),
            SignerType::Squads => write!(f, "squads"),
        }
    }
}

/// Information about a signer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerInfo {
    /// The signer's public key (base58 encoded).
    pub pubkey: String,
    /// Human-readable label for the signer.
    pub label: String,
    /// Type of signer (keypair, ledger, or squads).
    pub signer_type: SignerType,
    /// Tags associated with this signer.
    pub tags: Vec<String>,
}

/// Unified signer trait for all signing methods
pub trait Signer {
    /// Get the public key
    fn pubkey(&self) -> &str;

    /// Get the signer type
    fn signer_type(&self) -> SignerType;

    /// Sign a message
    fn sign(&self, message: &[u8]) -> Result<[u8; 64]>;
}

/// Local keypair signer
pub struct KeypairSigner {
    keypair: SecureKeypair,
    pubkey_str: String,
}

impl KeypairSigner {
    /// Create a new keypair signer
    pub fn new(keypair: SecureKeypair) -> Self {
        let pubkey_str = keypair.pubkey_base58();
        Self {
            keypair,
            pubkey_str,
        }
    }

    /// Load a keypair signer from the database
    pub fn load(db: &Database, identifier: &str, passphrase: &[u8]) -> Result<Self> {
        let keypair = db.load_keypair(identifier, passphrase)?;
        Ok(Self::new(keypair))
    }

    /// Get the underlying keypair
    pub fn keypair(&self) -> &SecureKeypair {
        &self.keypair
    }
}

impl Signer for KeypairSigner {
    fn pubkey(&self) -> &str {
        &self.pubkey_str
    }

    fn signer_type(&self) -> SignerType {
        SignerType::Keypair
    }

    fn sign(&self, message: &[u8]) -> Result<[u8; 64]> {
        Ok(self.keypair.sign(message))
    }
}

/// Ledger hardware wallet signer
pub struct LedgerSignerWrapper {
    inner: crate::ledger::LedgerSigner,
}

impl LedgerSignerWrapper {
    /// Create a new Ledger signer by connecting to the device
    pub fn connect(derivation_path: &str) -> Result<Self> {
        let inner = crate::ledger::LedgerSigner::connect(derivation_path)?;
        Ok(Self { inner })
    }

    /// Load from database and connect
    pub fn load(db: &Database, identifier: &str) -> Result<Self> {
        let wallets = db.list_ledger_wallets(None)?;
        let wallet = wallets
            .iter()
            .find(|w| w.pubkey == identifier || w.label == identifier)
            .ok_or_else(|| Error::KeypairNotFound(identifier.to_string()))?;

        Self::connect(&wallet.derivation_path)
    }
}

impl Signer for LedgerSignerWrapper {
    fn pubkey(&self) -> &str {
        self.inner.pubkey()
    }

    fn signer_type(&self) -> SignerType {
        SignerType::Ledger
    }

    fn sign(&self, message: &[u8]) -> Result<[u8; 64]> {
        self.inner.sign(message)
    }
}

/// List all available signers from the database
pub fn list_signers(db: &Database, tag_filter: Option<&str>) -> Result<Vec<SignerInfo>> {
    let mut signers = Vec::new();

    // Keypairs
    for row in db.list_keypairs(tag_filter)? {
        let tags = db.get_keypair_tags(&row.pubkey)?;
        signers.push(SignerInfo {
            pubkey: row.pubkey,
            label: row.label,
            signer_type: SignerType::Keypair,
            tags,
        });
    }

    // Ledger wallets
    for row in db.list_ledger_wallets(tag_filter)? {
        signers.push(SignerInfo {
            pubkey: row.pubkey,
            label: row.label,
            signer_type: SignerType::Ledger,
            tags: vec![], // TODO: add ledger tags
        });
    }

    // Squads multisigs
    for row in db.list_squads_multisigs(tag_filter)? {
        signers.push(SignerInfo {
            pubkey: row.multisig_pubkey,
            label: row.label,
            signer_type: SignerType::Squads,
            tags: vec![], // TODO: add squads tags
        });
    }

    Ok(signers)
}
