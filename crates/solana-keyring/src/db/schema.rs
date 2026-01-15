//! Database row types

use serde::{Deserialize, Serialize};

/// Keypair row from the database.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeypairRow {
    pub id: i64,
    pub pubkey: String,
    pub label: String,
    pub key_type: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Ledger wallet row from the database.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerWalletRow {
    pub id: i64,
    pub pubkey: String,
    pub label: String,
    pub derivation_path: String,
    pub created_at: String,
}

/// Squads multisig row from the database.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquadsMultisigRow {
    pub id: i64,
    pub multisig_pubkey: String,
    pub label: String,
    pub vault_index: u32,
    pub threshold: u32,
    pub created_at: String,
    pub updated_at: String,
}

/// Address book row from the database.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBookRow {
    pub id: i64,
    pub pubkey: String,
    pub label: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Tag row from the database.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagRow {
    pub id: i64,
    pub name: String,
    pub count: i64,
}
