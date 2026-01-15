//! Database migrations

/// Current schema version
pub const SCHEMA: &str = r#"
-- Master configuration
CREATE TABLE IF NOT EXISTS config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    version INTEGER NOT NULL DEFAULT 1,
    password_salt BLOB NOT NULL,
    password_hash BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Tags for organizing credentials
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Keypairs table (row-level encryption)
CREATE TABLE IF NOT EXISTS keypairs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pubkey TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    encrypted_secret BLOB NOT NULL,
    encryption_nonce BLOB NOT NULL,
    encryption_salt BLOB NOT NULL,
    key_type TEXT NOT NULL DEFAULT 'ed25519',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Junction table for keypair tags
CREATE TABLE IF NOT EXISTS keypair_tags (
    keypair_id INTEGER NOT NULL REFERENCES keypairs(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (keypair_id, tag_id)
);

-- Ledger wallets
CREATE TABLE IF NOT EXISTS ledger_wallets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pubkey TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    derivation_path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Junction table for ledger tags
CREATE TABLE IF NOT EXISTS ledger_tags (
    ledger_id INTEGER NOT NULL REFERENCES ledger_wallets(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (ledger_id, tag_id)
);

-- Squads multisigs
CREATE TABLE IF NOT EXISTS squads_multisigs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    multisig_pubkey TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    vault_index INTEGER NOT NULL DEFAULT 0,
    threshold INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Junction table for squads tags
CREATE TABLE IF NOT EXISTS squads_tags (
    squads_id INTEGER NOT NULL REFERENCES squads_multisigs(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (squads_id, tag_id)
);

-- Squads multisig members
CREATE TABLE IF NOT EXISTS squads_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    multisig_id INTEGER NOT NULL REFERENCES squads_multisigs(id) ON DELETE CASCADE,
    member_pubkey TEXT NOT NULL,
    permissions INTEGER NOT NULL,
    label TEXT,
    UNIQUE(multisig_id, member_pubkey)
);

-- Address book
CREATE TABLE IF NOT EXISTS address_book (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pubkey TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for faster lookups
CREATE INDEX IF NOT EXISTS idx_keypairs_label ON keypairs(label);
CREATE INDEX IF NOT EXISTS idx_ledger_label ON ledger_wallets(label);
CREATE INDEX IF NOT EXISTS idx_squads_label ON squads_multisigs(label);
CREATE INDEX IF NOT EXISTS idx_address_book_label ON address_book(label);
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);
"#;
