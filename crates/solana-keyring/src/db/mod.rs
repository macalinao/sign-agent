//! Database module for keyring storage

mod migrations;
mod schema;

pub use schema::{AddressBookRow, KeypairRow, LedgerWalletRow, SquadsMultisigRow, TagRow};

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use zeroize::Zeroize;

use crate::crypto::{EncryptedData, decrypt_secret, encrypt_secret};
use crate::error::{Error, Result};
use crate::keypair::SecureKeypair;

/// Database handle for keyring operations
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create a database at the given path
    pub fn open(path: &Path) -> Result<Self> {
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        self.conn.execute_batch(migrations::SCHEMA)?;
        Ok(())
    }

    /// Check if the keyring has been initialized
    pub fn is_initialized(&self) -> Result<bool> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM config WHERE id = 1", [], |row| {
                    row.get(0)
                })?;
        Ok(count > 0)
    }

    /// Initialize the keyring with a master passphrase
    pub fn initialize(&self, passphrase: &[u8]) -> Result<()> {
        use crate::crypto::hash_password;
        use rand::RngCore;

        if self.is_initialized()? {
            return Err(Error::AlreadyExists("Keyring already initialized".into()));
        }

        let mut salt = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        let hash = hash_password(passphrase, &salt)?;

        self.conn.execute(
            "INSERT INTO config (id, version, password_salt, password_hash) VALUES (1, 1, ?1, ?2)",
            params![salt.as_slice(), hash.as_slice()],
        )?;

        Ok(())
    }

    /// Verify the master passphrase
    pub fn verify_passphrase(&self, passphrase: &[u8]) -> Result<bool> {
        use crate::crypto::verify_password;

        let (salt, hash): (Vec<u8>, Vec<u8>) = self
            .conn
            .query_row(
                "SELECT password_salt, password_hash FROM config WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| Error::NotInitialized)?;

        let salt: [u8; 32] = salt.try_into().map_err(|_| {
            Error::Database(rusqlite::Error::InvalidParameterName(
                "Invalid salt length".into(),
            ))
        })?;
        let hash: [u8; 32] = hash.try_into().map_err(|_| {
            Error::Database(rusqlite::Error::InvalidParameterName(
                "Invalid hash length".into(),
            ))
        })?;

        verify_password(passphrase, &salt, &hash)
    }

    // ==================== Keypair Operations ====================

    /// Store a keypair in the database
    pub fn store_keypair(
        &self,
        keypair: &SecureKeypair,
        label: &str,
        master_passphrase: &[u8],
        tags: &[&str],
    ) -> Result<()> {
        let pubkey_b58 = bs58::encode(keypair.pubkey_bytes()).into_string();
        let secret_bytes = keypair.secret_bytes();

        let encrypted = encrypt_secret(&secret_bytes[..], master_passphrase)?;

        self.conn.execute(
            "INSERT INTO keypairs (pubkey, label, encrypted_secret, encryption_nonce, encryption_salt)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                pubkey_b58,
                label,
                encrypted.ciphertext,
                encrypted.nonce.as_slice(),
                encrypted.salt.as_slice(),
            ],
        )?;

        // Add tags
        for tag in tags {
            self.add_tag_to_keypair(&pubkey_b58, tag)?;
        }

        Ok(())
    }

    /// Load a keypair from the database
    pub fn load_keypair(
        &self,
        identifier: &str,
        master_passphrase: &[u8],
    ) -> Result<SecureKeypair> {
        // Try to find by pubkey first, then by label
        let row: Option<(Vec<u8>, Vec<u8>, Vec<u8>)> = self
            .conn
            .query_row(
                "SELECT encrypted_secret, encryption_nonce, encryption_salt
             FROM keypairs WHERE pubkey = ?1 OR label = ?1",
                params![identifier],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;

        let (ciphertext, nonce, salt) =
            row.ok_or_else(|| Error::KeypairNotFound(identifier.into()))?;

        let nonce: [u8; 12] = nonce
            .try_into()
            .map_err(|_| Error::Encryption("Invalid nonce".into()))?;
        let salt: [u8; 32] = salt
            .try_into()
            .map_err(|_| Error::Encryption("Invalid salt".into()))?;

        let encrypted = EncryptedData {
            ciphertext,
            nonce,
            salt,
        };
        let mut secret_bytes = decrypt_secret(&encrypted, master_passphrase)?;

        let result = SecureKeypair::from_bytes(
            secret_bytes
                .as_slice()
                .try_into()
                .map_err(|_| Error::InvalidKeypairFormat("Wrong key size".into()))?,
        );

        secret_bytes.zeroize();
        result
    }

    /// List all keypairs
    pub fn list_keypairs(&self, tag_filter: Option<&str>) -> Result<Vec<KeypairRow>> {
        let query = if tag_filter.is_some() {
            "SELECT k.id, k.pubkey, k.label, k.key_type, k.created_at, k.updated_at
             FROM keypairs k
             INNER JOIN keypair_tags kt ON k.id = kt.keypair_id
             INNER JOIN tags t ON kt.tag_id = t.id
             WHERE t.name = ?1
             ORDER BY k.label"
        } else {
            "SELECT id, pubkey, label, key_type, created_at, updated_at
             FROM keypairs ORDER BY label"
        };

        let mut stmt = self.conn.prepare(query)?;

        fn map_row(row: &rusqlite::Row) -> rusqlite::Result<KeypairRow> {
            Ok(KeypairRow {
                id: row.get(0)?,
                pubkey: row.get(1)?,
                label: row.get(2)?,
                key_type: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        }

        let rows: Vec<KeypairRow> = if let Some(tag) = tag_filter {
            stmt.query_map(params![tag], map_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            stmt.query_map([], map_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };

        Ok(rows)
    }

    /// Get tags for a keypair
    pub fn get_keypair_tags(&self, pubkey: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.name FROM tags t
             INNER JOIN keypair_tags kt ON t.id = kt.tag_id
             INNER JOIN keypairs k ON kt.keypair_id = k.id
             WHERE k.pubkey = ?1",
        )?;

        let tags = stmt.query_map(params![pubkey], |row| row.get(0))?;
        tags.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Delete a keypair
    pub fn delete_keypair(&self, identifier: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM keypairs WHERE pubkey = ?1 OR label = ?1",
            params![identifier],
        )?;
        Ok(affected > 0)
    }

    /// Update keypair label
    pub fn update_keypair_label(&self, identifier: &str, new_label: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "UPDATE keypairs SET label = ?2, updated_at = CURRENT_TIMESTAMP
             WHERE pubkey = ?1 OR label = ?1",
            params![identifier, new_label],
        )?;
        Ok(affected > 0)
    }

    // ==================== Tag Operations ====================

    /// Create a tag if it doesn't exist, return its ID
    fn get_or_create_tag(&self, name: &str) -> Result<i64> {
        // Try to insert, ignore if exists
        self.conn.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            params![name],
        )?;

        let id: i64 = self.conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )?;

        Ok(id)
    }

    /// Add a tag to a keypair
    pub fn add_tag_to_keypair(&self, pubkey: &str, tag: &str) -> Result<()> {
        let tag_id = self.get_or_create_tag(tag)?;

        let keypair_id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM keypairs WHERE pubkey = ?1",
                params![pubkey],
                |row| row.get(0),
            )
            .map_err(|_| Error::KeypairNotFound(pubkey.into()))?;

        self.conn.execute(
            "INSERT OR IGNORE INTO keypair_tags (keypair_id, tag_id) VALUES (?1, ?2)",
            params![keypair_id, tag_id],
        )?;

        Ok(())
    }

    /// Remove a tag from a keypair
    pub fn remove_tag_from_keypair(&self, pubkey: &str, tag: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM keypair_tags
             WHERE keypair_id = (SELECT id FROM keypairs WHERE pubkey = ?1)
             AND tag_id = (SELECT id FROM tags WHERE name = ?2)",
            params![pubkey, tag],
        )?;
        Ok(affected > 0)
    }

    /// List all tags
    pub fn list_tags(&self) -> Result<Vec<TagRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, COUNT(kt.keypair_id) as count
             FROM tags t
             LEFT JOIN keypair_tags kt ON t.id = kt.tag_id
             GROUP BY t.id, t.name
             ORDER BY t.name",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(TagRow {
                id: row.get(0)?,
                name: row.get(1)?,
                count: row.get(2)?,
            })
        })?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Delete a tag
    pub fn delete_tag(&self, name: &str) -> Result<bool> {
        let affected = self
            .conn
            .execute("DELETE FROM tags WHERE name = ?1", params![name])?;
        Ok(affected > 0)
    }

    // ==================== Ledger Wallet Operations ====================

    /// Store a Ledger wallet
    pub fn store_ledger_wallet(
        &self,
        pubkey: &str,
        label: &str,
        derivation_path: &str,
        tags: &[&str],
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO ledger_wallets (pubkey, label, derivation_path)
             VALUES (?1, ?2, ?3)",
            params![pubkey, label, derivation_path],
        )?;

        for tag in tags {
            self.add_tag_to_ledger(pubkey, tag)?;
        }

        Ok(())
    }

    /// List all Ledger wallets
    pub fn list_ledger_wallets(&self, tag_filter: Option<&str>) -> Result<Vec<LedgerWalletRow>> {
        let query = if tag_filter.is_some() {
            "SELECT l.id, l.pubkey, l.label, l.derivation_path, l.created_at
             FROM ledger_wallets l
             INNER JOIN ledger_tags lt ON l.id = lt.ledger_id
             INNER JOIN tags t ON lt.tag_id = t.id
             WHERE t.name = ?1
             ORDER BY l.label"
        } else {
            "SELECT id, pubkey, label, derivation_path, created_at
             FROM ledger_wallets ORDER BY label"
        };

        let mut stmt = self.conn.prepare(query)?;

        fn map_row(row: &rusqlite::Row) -> rusqlite::Result<LedgerWalletRow> {
            Ok(LedgerWalletRow {
                id: row.get(0)?,
                pubkey: row.get(1)?,
                label: row.get(2)?,
                derivation_path: row.get(3)?,
                created_at: row.get(4)?,
            })
        }

        let rows: Vec<LedgerWalletRow> = if let Some(tag) = tag_filter {
            stmt.query_map(params![tag], map_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            stmt.query_map([], map_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };

        Ok(rows)
    }

    /// Add a tag to a Ledger wallet
    fn add_tag_to_ledger(&self, pubkey: &str, tag: &str) -> Result<()> {
        let tag_id = self.get_or_create_tag(tag)?;

        let ledger_id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM ledger_wallets WHERE pubkey = ?1",
                params![pubkey],
                |row| row.get(0),
            )
            .map_err(|_| Error::AddressNotFound(pubkey.into()))?;

        self.conn.execute(
            "INSERT OR IGNORE INTO ledger_tags (ledger_id, tag_id) VALUES (?1, ?2)",
            params![ledger_id, tag_id],
        )?;

        Ok(())
    }

    /// Delete a Ledger wallet
    pub fn delete_ledger_wallet(&self, identifier: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM ledger_wallets WHERE pubkey = ?1 OR label = ?1",
            params![identifier],
        )?;
        Ok(affected > 0)
    }

    // ==================== Squads Multisig Operations ====================

    /// Store a Squads multisig
    pub fn store_squads_multisig(
        &self,
        multisig_pubkey: &str,
        label: &str,
        vault_index: u32,
        threshold: u32,
        tags: &[&str],
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO squads_multisigs (multisig_pubkey, label, vault_index, threshold)
             VALUES (?1, ?2, ?3, ?4)",
            params![multisig_pubkey, label, vault_index, threshold],
        )?;

        for tag in tags {
            self.add_tag_to_squads(multisig_pubkey, tag)?;
        }

        Ok(())
    }

    /// List all Squads multisigs
    pub fn list_squads_multisigs(
        &self,
        tag_filter: Option<&str>,
    ) -> Result<Vec<SquadsMultisigRow>> {
        let query = if tag_filter.is_some() {
            "SELECT s.id, s.multisig_pubkey, s.label, s.vault_index, s.threshold, s.created_at, s.updated_at
             FROM squads_multisigs s
             INNER JOIN squads_tags st ON s.id = st.squads_id
             INNER JOIN tags t ON st.tag_id = t.id
             WHERE t.name = ?1
             ORDER BY s.label"
        } else {
            "SELECT id, multisig_pubkey, label, vault_index, threshold, created_at, updated_at
             FROM squads_multisigs ORDER BY label"
        };

        let mut stmt = self.conn.prepare(query)?;

        fn map_row(row: &rusqlite::Row) -> rusqlite::Result<SquadsMultisigRow> {
            Ok(SquadsMultisigRow {
                id: row.get(0)?,
                multisig_pubkey: row.get(1)?,
                label: row.get(2)?,
                vault_index: row.get(3)?,
                threshold: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        }

        let rows: Vec<SquadsMultisigRow> = if let Some(tag) = tag_filter {
            stmt.query_map(params![tag], map_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            stmt.query_map([], map_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };

        Ok(rows)
    }

    /// Add a tag to a Squads multisig
    fn add_tag_to_squads(&self, pubkey: &str, tag: &str) -> Result<()> {
        let tag_id = self.get_or_create_tag(tag)?;

        let squads_id: i64 = self
            .conn
            .query_row(
                "SELECT id FROM squads_multisigs WHERE multisig_pubkey = ?1",
                params![pubkey],
                |row| row.get(0),
            )
            .map_err(|_| Error::AddressNotFound(pubkey.into()))?;

        self.conn.execute(
            "INSERT OR IGNORE INTO squads_tags (squads_id, tag_id) VALUES (?1, ?2)",
            params![squads_id, tag_id],
        )?;

        Ok(())
    }

    /// Delete a Squads multisig
    pub fn delete_squads_multisig(&self, identifier: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM squads_multisigs WHERE multisig_pubkey = ?1 OR label = ?1",
            params![identifier],
        )?;
        Ok(affected > 0)
    }

    // ==================== Address Book Operations ====================

    /// Add an address to the address book
    pub fn add_address(&self, pubkey: &str, label: &str, notes: Option<&str>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO address_book (pubkey, label, notes)
             VALUES (?1, ?2, ?3)",
            params![pubkey, label, notes],
        )?;
        Ok(())
    }

    /// List all addresses in the address book
    pub fn list_addresses(&self) -> Result<Vec<AddressBookRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pubkey, label, notes, created_at, updated_at
             FROM address_book ORDER BY label",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(AddressBookRow {
                id: row.get(0)?,
                pubkey: row.get(1)?,
                label: row.get(2)?,
                notes: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Delete an address from the address book
    pub fn delete_address(&self, identifier: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM address_book WHERE pubkey = ?1 OR label = ?1",
            params![identifier],
        )?;
        Ok(affected > 0)
    }

    /// Update address label
    pub fn update_address_label(&self, identifier: &str, new_label: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "UPDATE address_book SET label = ?2, updated_at = CURRENT_TIMESTAMP
             WHERE pubkey = ?1 OR label = ?1",
            params![identifier, new_label],
        )?;
        Ok(affected > 0)
    }
}
