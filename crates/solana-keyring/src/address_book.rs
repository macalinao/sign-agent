//! Address book for managing labeled addresses

use crate::db::{AddressBookRow, Database};
use crate::error::Result;

/// Address book operations
pub struct AddressBook<'a> {
    db: &'a Database,
}

impl<'a> AddressBook<'a> {
    /// Create a new address book handle
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Add an address to the address book
    pub fn add(&self, pubkey: &str, label: &str, notes: Option<&str>) -> Result<()> {
        self.db.add_address(pubkey, label, notes)
    }

    /// List all addresses
    pub fn list(&self) -> Result<Vec<AddressBookRow>> {
        self.db.list_addresses()
    }

    /// Remove an address
    pub fn remove(&self, identifier: &str) -> Result<bool> {
        self.db.delete_address(identifier)
    }

    /// Update an address label
    pub fn update_label(&self, identifier: &str, new_label: &str) -> Result<bool> {
        self.db.update_address_label(identifier, new_label)
    }

    /// Resolve a label or pubkey to a pubkey
    pub fn resolve(&self, identifier: &str) -> Result<Option<String>> {
        let addresses = self.list()?;
        for addr in addresses {
            if addr.pubkey == identifier || addr.label == identifier {
                return Ok(Some(addr.pubkey));
            }
        }
        Ok(None)
    }
}
