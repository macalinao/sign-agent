//! CLI definitions

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "solana-keyring",
    about = "Secure keyring management for Solana",
    version
)]
pub struct Cli {
    /// Database path (default: ~/.solana-keyring/keyring.db)
    #[arg(long, global = true)]
    pub db_path: Option<PathBuf>,

    /// Disable the keyring agent (will prompt for passphrase)
    #[arg(long, global = true)]
    pub no_agent: bool,

    /// Agent socket path (default: ~/.solana-keyring/agent.sock)
    #[arg(long, global = true)]
    pub agent_socket: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new keyring with master passphrase
    New(NewArgs),

    /// Generate a new keypair
    Generate(GenerateArgs),

    /// Import a keypair from JSON file or base58 string
    Import(ImportArgs),

    /// Export a keypair to JSON file or base58 string
    Export(ExportArgs),

    /// List all signers (keypairs, ledger wallets, squads)
    List(ListArgs),

    /// Add or update label for an address
    Label(LabelArgs),

    /// Delete a keypair
    Delete(DeleteArgs),

    /// Tag management
    #[command(subcommand)]
    Tag(TagCommands),

    /// Ledger hardware wallet commands
    #[command(subcommand)]
    Ledger(LedgerCommands),

    /// Squads multisig commands
    #[command(subcommand)]
    Squads(SquadsCommands),

    /// Address book management
    #[command(subcommand)]
    AddressBook(AddressBookCommands),
}

#[derive(clap::Args)]
pub struct NewArgs {
    /// Force overwrite if keyring already exists
    #[arg(long)]
    pub force: bool,
}

#[derive(clap::Args)]
pub struct GenerateArgs {
    /// Label for the new keypair (required)
    #[arg(short, long)]
    pub label: String,

    /// Tags to add to the keypair
    #[arg(short, long)]
    pub tag: Vec<String>,
}

#[derive(clap::Args)]
pub struct ImportArgs {
    /// Label for the imported keypair
    #[arg(short, long)]
    pub label: String,

    /// Path to JSON keypair file
    #[arg(short, long, conflicts_with = "base58")]
    pub file: Option<PathBuf>,

    /// Base58 encoded secret key
    #[arg(short, long, conflicts_with = "file")]
    pub base58: Option<String>,

    /// Tags to add to the keypair
    #[arg(short, long)]
    pub tag: Vec<String>,
}

#[derive(clap::Args)]
pub struct ExportArgs {
    /// Public key or label of keypair to export
    pub identifier: String,

    /// Output format
    #[arg(short, long, default_value = "json")]
    pub format: ExportFormat,

    /// Output file path (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Clone, ValueEnum)]
pub enum ExportFormat {
    Json,
    Base58,
}

#[derive(clap::Args)]
pub struct ListArgs {
    /// Filter by signer type
    #[arg(short = 't', long = "type")]
    pub signer_type: Option<SignerTypeFilter>,

    /// Filter by tag
    #[arg(long)]
    pub tag: Option<String>,

    /// Output format
    #[arg(short, long, default_value = "table")]
    pub format: OutputFormat,
}

#[derive(Clone, ValueEnum)]
pub enum SignerTypeFilter {
    Keypair,
    Ledger,
    Squads,
    All,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(clap::Args)]
pub struct LabelArgs {
    /// Public key or current label
    pub identifier: String,

    /// New label
    pub label: String,
}

#[derive(clap::Args)]
pub struct DeleteArgs {
    /// Public key or label to delete
    pub identifier: String,

    /// Skip confirmation
    #[arg(short, long)]
    pub force: bool,
}

// Tag commands
#[derive(Subcommand)]
pub enum TagCommands {
    /// List all tags
    List,
    /// Add a tag to a keypair
    Add(TagAddArgs),
    /// Remove a tag from a keypair
    Remove(TagRemoveArgs),
    /// Delete a tag entirely
    Delete(TagDeleteArgs),
}

#[derive(clap::Args)]
pub struct TagAddArgs {
    /// Public key or label of keypair
    pub identifier: String,
    /// Tag name
    pub tag: String,
}

#[derive(clap::Args)]
pub struct TagRemoveArgs {
    /// Public key or label of keypair
    pub identifier: String,
    /// Tag name
    pub tag: String,
}

#[derive(clap::Args)]
pub struct TagDeleteArgs {
    /// Tag name to delete
    pub tag: String,
}

// Ledger commands
#[derive(Subcommand)]
pub enum LedgerCommands {
    /// Add a Ledger-based wallet
    Add(LedgerAddArgs),
    /// List Ledger wallets
    List,
    /// Remove a Ledger wallet
    Remove(LedgerRemoveArgs),
}

#[derive(clap::Args)]
pub struct LedgerAddArgs {
    /// Label for the Ledger wallet
    #[arg(short, long)]
    pub label: String,

    /// Derivation path (default: 44'/501'/0'/0')
    #[arg(short, long, default_value = "44'/501'/0'/0'")]
    pub derivation_path: String,

    /// Tags to add
    #[arg(short, long)]
    pub tag: Vec<String>,
}

#[derive(clap::Args)]
pub struct LedgerRemoveArgs {
    /// Public key or label to remove
    pub identifier: String,
}

// Squads commands
#[derive(Subcommand)]
pub enum SquadsCommands {
    /// Add a Squads multisig
    Add(SquadsAddArgs),
    /// List Squads multisigs
    List,
    /// Remove a Squads multisig
    Remove(SquadsRemoveArgs),
    /// Sync multisig members from on-chain
    Sync(SquadsSyncArgs),
}

#[derive(clap::Args)]
pub struct SquadsAddArgs {
    /// Multisig address
    pub multisig_address: String,

    /// Label for the multisig
    #[arg(short, long)]
    pub label: String,

    /// RPC URL
    #[arg(long, default_value = "https://api.mainnet-beta.solana.com")]
    pub rpc_url: String,

    /// Tags to add
    #[arg(short, long)]
    pub tag: Vec<String>,
}

#[derive(clap::Args)]
pub struct SquadsRemoveArgs {
    /// Multisig address or label
    pub identifier: String,
}

#[derive(clap::Args)]
pub struct SquadsSyncArgs {
    /// Multisig address or label
    pub identifier: String,

    /// RPC URL
    #[arg(long, default_value = "https://api.mainnet-beta.solana.com")]
    pub rpc_url: String,
}

// Address book commands
#[derive(Subcommand)]
pub enum AddressBookCommands {
    /// Add an address to the book
    Add(AddressBookAddArgs),
    /// List all addresses
    List,
    /// Remove an address
    Remove(AddressBookRemoveArgs),
    /// Update an address label
    Label(AddressBookLabelArgs),
}

#[derive(clap::Args)]
pub struct AddressBookAddArgs {
    /// Public key
    pub pubkey: String,

    /// Label
    #[arg(short, long)]
    pub label: String,

    /// Optional notes
    #[arg(short, long)]
    pub notes: Option<String>,
}

#[derive(clap::Args)]
pub struct AddressBookRemoveArgs {
    /// Public key or label to remove
    pub identifier: String,
}

#[derive(clap::Args)]
pub struct AddressBookLabelArgs {
    /// Public key or current label
    pub identifier: String,

    /// New label
    pub label: String,
}
