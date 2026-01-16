# solana-actor-squads

Squads multisig transport for Solana wallet operations.

## Features

- **Multisig support** - Create and manage multi-signature transactions
- **WalletTransport trait** - Implements async submission with status tracking
- **Flexible member signer** - Works with any `TransactionSigner` (keypair, Ledger)
- **Squads v4** - Uses Squads Protocol v4

## Key Difference from Signers

Unlike keypair or Ledger signers, Squads does NOT implement signer traits.
Instead, it implements `WalletTransport` because:

- Multisig doesn't produce direct cryptographic signatures
- Transactions create on-chain proposals that need multiple approvals
- Execution happens on-chain when threshold is reached

## Usage

### Basic Setup

```rust
use solana_actor_squads::SquadsTransport;
use solana_actor_keypair::KeypairSigner;
use solana_actor::WalletTransport;

let member = KeypairSigner::from_file("member.json")?;
let transport = SquadsTransport::new(
    "MULTISIG_ADDRESS".parse()?,
    0, // vault_index
    "https://api.mainnet-beta.solana.com",
    member,
)?;

// The vault PDA is the authority for transactions
let authority = transport.authority();
```

### Submit a Transaction

```rust
use solana_actor::SubmitResult;

// Submit creates a proposal and approves with the member key
let result = transport.submit(&tx_message).await?;

match &result {
    SubmitResult::Executed { signature, .. } => {
        println!("Already executed: {}", signature);
    }
    SubmitResult::Pending { approvals, threshold, .. } => {
        println!("Pending: {}/{} approvals", approvals, threshold);
    }
    _ => {}
}
```

### Wait for Completion

```rust
use std::time::Duration;

// For pending proposals, wait for other signers
if result.is_pending() {
    let final_result = transport
        .wait_for_completion(result, Duration::from_secs(300))
        .await?;

    if let SubmitResult::Executed { signature, .. } = final_result {
        println!("Executed: {}", signature);
    }
}
```

### Use with Ledger

```rust
use solana_actor_squads::SquadsTransport;
use solana_actor_ledger::LedgerSigner;

let member = LedgerSigner::connect()?;
let transport = SquadsTransport::new(multisig, 0, url, member)?;
```

## Submit Result Types

- `SubmitResult::Signed` - Never returned by Squads (direct signing)
- `SubmitResult::Pending` - Proposal created, awaiting more approvals
- `SubmitResult::Executed` - Proposal reached threshold and was executed

## Related Crates

- `solana-actor` - Core traits
- `solana-actor-keypair` - Software keypair signer
- `solana-actor-ledger` - Hardware wallet signer

## License

Apache-2.0
