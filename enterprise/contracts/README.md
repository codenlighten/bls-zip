# Enterprise E2 Multipass Smart Contract Templates

Comprehensive smart contract templates for the Boundless BLS Enterprise E2 Multipass system, providing identity-based access control, multi-signature wallets, asset escrow, and application authorization.

## Overview

These templates demonstrate how to build enterprise-grade smart contracts that integrate with the E2 Multipass platform:

- **Identity Service**: CIVA-based identity verification
- **Wallet Service**: Post-quantum cryptographic wallets
- **Asset Service**: Multi-asset management (IRSC, CRSC, custom tokens)
- **Application Service**: App-aware permissions and OAuth-like scopes

## Templates

### 1. Identity-Based Access Control (`identity_access_control.rs`)

Role-based access control system integrated with E2 Multipass identities.

**Features:**
- Role hierarchy (Owner, Admin, User, Guest)
- Identity verification via PQC signatures
- KYC/attestation checking
- Multi-level permissions
- Integration with CIVA attestations

**Use Cases:**
- Enterprise DAO governance
- Regulated financial applications
- Compliance-required platforms
- Credential-gated services

**Example:**
```rust
// Register a new identity
contract.register_identity(
    identity_id,  // From E2 Multipass
    Role::User    // Initial role
)?;

// Grant admin role
contract.grant_role(account, Role::Admin)?;

// Check permissions
if contract.has_role(account, Role::Admin) {
    // Perform admin action
}
```

### 2. Multi-Signature Wallet (`multisig_wallet.rs`)

Secure multi-signature wallet requiring M-of-N approvals for transactions.

**Features:**
- Configurable threshold (M-of-N)
- Transaction proposal/approval workflow
- Daily spending limits per signer
- Time-locked transactions
- Support for multiple asset types
- E2 identity integration

**Use Cases:**
- Corporate treasury management
- Joint custody wallets
- Decentralized fund management
- Escrow services

**Example:**
```rust
// Create 2-of-3 multisig wallet
let wallet = MultisigWallet::new(
    vec![
        (alice, alice_identity, 10_000),
        (bob, bob_identity, 10_000),
        (charlie, charlie_identity, 10_000),
    ],
    2  // Threshold
);

// Propose transaction
let tx_id = wallet.propose_transaction(
    recipient,
    amount,
    None,  // Native token
    None   // No data
)?;

// Approve (need 2 signatures)
wallet.approve_transaction(tx_id)?;

// Execute after threshold reached
wallet.execute_transaction(tx_id)?;
```

### 3. Asset Escrow & Trading (`asset_escrow.rs`)

Peer-to-peer asset trading with secure escrow and dispute resolution.

**Features:**
- Atomic swaps between parties
- Time-locked escrow
- Multi-asset bundle trades
- Dispute resolution with arbitrators
- Integration with E2 locked_quantity system
- Support for IRSC, CRSC, custom assets

**Use Cases:**
- Decentralized asset exchanges
- OTC (over-the-counter) trading
- Cross-border settlements
- NFT marketplaces

**Example:**
```rust
// Propose a trade
let trade_id = escrow.propose_trade(
    proposer_identity,
    offer_assets,      // What you're offering
    request_assets,    // What you want
    Some(counterparty),// Optional: specific counterparty
    None,              // Counterparty identity (if public)
    None               // Default 7-day expiry
)?;

// Counterparty accepts and locks assets
escrow.accept_trade(trade_id, counterparty_identity)?;

// Both parties confirm
escrow.confirm_trade(trade_id)?;

// Trade automatically completes when both confirm
```

### 4. Application Authorization (`app_authorization.rs`)

OAuth-like authorization framework for app-aware permissions.

**Features:**
- OAuth-style scopes (read, write, execute)
- Application registration
- Time-limited authorization grants
- Delegation and sub-delegation
- Resource-based access control
- Integration with E2 application service

**Use Cases:**
- Third-party app integrations
- Delegated authority
- API access management
- Service-to-service authorization

**Example:**
```rust
// Register an application
auth.register_application(
    app_id,
    "My DApp",
    owner_identity,
    vec!["https://example.com/callback"],
    vec![Scope::ReadProfile, Scope::ReadWallet]
)?;

// User grants permissions to app
let grant_id = auth.issue_grant(
    app_id,
    user_identity,
    vec![Scope::ReadProfile],
    Some(30_days_ms),
    true  // Delegatable
)?;

// Check if app can access resource
if auth.can_access_resource(user, resource_id, app_id) {
    // Allow access
}
```

## Building Contracts

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-contract for ink! contracts
cargo install cargo-contract --force

# Add WASM target
rustup target add wasm32-unknown-unknown
```

### Build a Template

```bash
cd enterprise/contracts/templates

# Build identity access control
cargo contract build --manifest-path identity_access_control.toml --release

# Build multisig wallet
cargo contract build --manifest-path multisig_wallet.toml --release

# Build asset escrow
cargo contract build --manifest-path asset_escrow.toml --release

# Build app authorization
cargo contract build --manifest-path app_authorization.toml --release
```

This produces:
- `target/ink/{contract}.wasm` - WASM bytecode
- `target/ink/{contract}.json` - Contract metadata/ABI

## Testing Contracts

```bash
# Run unit tests
cargo test

# Run integration tests with E2 Multipass
cargo test --features e2-integration

# Test with fuel metering
cargo test --features fuel-metering
```

## Deploying Contracts

### Option 1: Using Deployment Utilities

```rust
use enterprise_contracts::deploy::{deploy_contract, DeploymentConfig};

let config = DeploymentConfig {
    wasm_path: "target/ink/identity_access_control.wasm".to_string(),
    constructor: "new".to_string(),
    constructor_args: vec![],
    deployer_account: "your_account".to_string(),
    deployer_identity: "your_e2_identity_id".to_string(),
    gas_limit: 50_000_000,
    rpc_url: "http://localhost:9933".to_string(),
};

let result = deploy_contract(config).await?;
println!("Deployed at: {}", result.contract_address);
```

### Option 2: Direct Transaction

```bash
# Using the enterprise CLI
./enterprise-cli contract deploy \
  --wasm target/ink/identity_access_control.wasm \
  --constructor new \
  --identity your_identity_id \
  --gas-limit 50000000
```

## WASM Runtime Integration

All contracts run on the Boundless WASM runtime with:

- **Fuel Metering**: Deterministic gas accounting
- **Memory Limits**: 10MB storage per contract, 16MB RAM
- **Timeout Enforcement**: 10-second execution limit
- **Host Functions**:
  - `storage_get/storage_set` - Persistent storage
  - `sha3_256` - Cryptographic hashing
  - `get_caller` - Caller address
  - `get_block_height/get_timestamp` - Blockchain context

### Host Function Usage

```rust
// In your contract (pseudo-code, actual implementation uses ink!)
let caller = self.env().caller();
let block_height = self.env().block_number();

// Storage (handled by ink! automatically)
self.storage.insert(key, value);
let value = self.storage.get(key);
```

## E2 Multipass Integration

### Identity Service Integration

```rust
// Verify identity exists in E2 system
let identity = e2_identity_service.get_profile(identity_id)?;

// Check attestations
let kyc_attestation = e2_identity_service
    .get_attestation(identity_id, AttestationType::KYC)?;

// Use in contract
contract.register_identity(identity_id, Role::User)?;
if identity.kyc_verified {
    contract.mark_kyc_verified(account)?;
}
```

### Asset Service Integration

```rust
// Lock assets for escrow
e2_asset_service.lock_quantity(
    account,
    asset_id,
    quantity
)?;

// On trade completion, transfer locked assets
e2_asset_service.transfer_locked(
    from_account,
    to_account,
    asset_id,
    quantity
)?;
```

### Application Service Integration

```rust
// Register contract as an application
e2_app_service.register_application(
    app_id,
    "Smart Contract App",
    owner_identity,
    required_scopes
)?;

// Check user authorization
let authorized = e2_app_service.check_authorization(
    user_account,
    app_id,
    required_scopes
)?;
```

## Security Considerations

### 1. Gas Limits
- All operations are fuel-metered
- Default: 100M fuel units per call
- Production: 50M fuel units per call
- Adjust based on contract complexity

### 2. Storage Quotas
- Maximum 10MB total storage per contract
- Maximum 1MB per storage value
- Exceeding limits causes transaction failure

### 3. Timeout Protection
- 10-second execution timeout
- Prevents infinite loops
- Design contracts to complete quickly

### 4. Signature Verification
- Use E2 Multipass PQC signatures (Dilithium5)
- Verify caller identity before privileged operations
- Check attestations for sensitive functions

### 5. Reentrancy Protection
- ink! provides reentrancy guards by default
- Be cautious with cross-contract calls
- Use checks-effects-interactions pattern

## Best Practices

### 1. Identity Verification
```rust
// Always verify E2 identity before granting permissions
fn register_identity(&mut self, identity_id: [u8; 32]) -> Result<()> {
    // Verify identity exists in E2 system first
    self.verify_e2_identity(identity_id)?;

    // Then register locally
    self.identities.insert(identity_id, IdentityStatus::new());
    Ok(())
}
```

### 2. Attestation Checking
```rust
// Check attestations for compliance-critical operations
fn perform_regulated_action(&mut self) -> Result<()> {
    let caller = self.env().caller();

    // Require KYC attestation
    self.require_attestation(&caller, KYC_ATTESTATION_HASH)?;

    // Perform action
    Ok(())
}
```

### 3. Multi-Signature Approvals
```rust
// Use multisig for high-value operations
fn transfer_large_amount(&mut self, amount: Balance) -> Result<()> {
    if amount > MULTISIG_THRESHOLD {
        // Create proposal instead of direct execution
        return self.create_multisig_proposal(amount);
    }

    // Small amounts can be direct
    self.execute_transfer(amount)
}
```

### 4. Time-Lock Protection
```rust
// Add time delays for sensitive operations
fn execute_transaction(&mut self, tx_id: u64) -> Result<()> {
    let tx = self.transactions.get(tx_id)?;

    // Require minimum delay
    let now = self.env().block_timestamp();
    if now < tx.proposed_at + MIN_DELAY {
        return Err(Error::TooEarly);
    }

    // Execute
    Ok(())
}
```

## Example Deployment Config

```toml
# deployment_config.toml
wasm_path = "target/ink/identity_access_control.wasm"
constructor = "new"
constructor_args = []
deployer_account = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
deployer_identity = "c9ad4c39-c9f3-463d-ad66-c78649905b87"
gas_limit = 50_000_000
rpc_url = "http://localhost:9933"
```

## Troubleshooting

### Contract Deployment Fails
- Check WASM file size (should be < 1MB typically)
- Verify gas limit is sufficient
- Ensure deployer has sufficient balance
- Check deployer identity is registered

### Contract Execution Times Out
- Reduce computational complexity
- Break into smaller operations
- Use pagination for large data sets
- Optimize storage access patterns

### Out of Fuel Error
- Increase gas limit
- Optimize contract logic
- Remove unnecessary computations
- Use efficient data structures

### Storage Quota Exceeded
- Implement data pruning
- Use compact encoding
- Archive old data off-chain
- Optimize storage layout

## Additional Resources

- [Boundless BLS Documentation](../../README.md)
- [E2 Multipass Integration Guide](../../enterprise/E2_INTEGRATION_IMPLEMENTATION.md)
- [WASM Runtime Documentation](../../wasm-runtime/README.md)
- [ink! Documentation](https://use.ink/)

## License

Copyright © 2024 Boundless BLS Platform - Enterprise E2 Multipass

## Support

For questions or issues:
- GitHub Issues: https://github.com/boundless-bls/issues
- Documentation: https://docs.boundless.trust
- Community: https://discord.gg/boundless
