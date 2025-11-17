# Quick Start Guide - Enterprise E2 Multipass Smart Contracts

Get started with E2 Multipass smart contracts in 5 minutes.

## 1. Setup

```bash
# Install dependencies
cargo install cargo-contract --force
rustup target add wasm32-unknown-unknown

# Clone and navigate
cd enterprise/contracts/templates
```

## 2. Choose Your Template

### Identity Access Control
**When to use**: Need role-based permissions with E2 identity verification

```rust
// Create contract
let contract = IdentityAccessControl::new();

// Register user with E2 identity
contract.register_identity(identity_id, Role::User)?;

// Check permissions
if contract.has_role(account, Role::Admin) {
    // Authorized
}
```

### Multi-Signature Wallet
**When to use**: Corporate treasury, joint custody, escrow

```rust
// 2-of-3 multisig
let wallet = MultisigWallet::new(signers, 2);

// Propose transaction
let tx_id = wallet.propose_transaction(to, amount, None, None)?;

// Approve (need 2)
wallet.approve_transaction(tx_id)?;

// Execute
wallet.execute_transaction(tx_id)?;
```

### Asset Escrow
**When to use**: P2P trading, OTC deals, marketplaces

```rust
// Propose trade
let trade_id = escrow.propose_trade(
    identity_id,
    offer_assets,
    request_assets,
    counterparty,
    None,
    None
)?;

// Accept and lock
escrow.accept_trade(trade_id, counterparty_identity)?;

// Both confirm to complete
escrow.confirm_trade(trade_id)?;
```

### App Authorization
**When to use**: Third-party integrations, delegated permissions

```rust
// Register app
auth.register_application(
    app_id,
    "My App",
    owner_identity,
    redirect_uris,
    requested_scopes
)?;

// Grant permissions
let grant_id = auth.issue_grant(
    app_id,
    user_identity,
    scopes,
    validity,
    delegatable
)?;

// Check access
if auth.can_access_resource(user, resource_id, app_id) {
    // Allowed
}
```

## 3. Build Contract

```bash
# Create Cargo.toml for your contract
cat > Cargo.toml << EOF
[package]
name = "my-contract"
version = "0.1.0"
edition = "2021"

[dependencies]
ink = { version = "4.3", default-features = false }

[lib]
name = "my_contract"
path = "identity_access_control.rs"
crate-type = ["cdylib"]

[features]
default = ["std"]
std = ["ink/std"]
EOF

# Build
cargo contract build --release

# Output: target/ink/my_contract.wasm
```

## 4. Test Locally

```bash
# Run unit tests
cargo test

# Test with fuel metering
cargo test --features fuel-metering
```

## 5. Deploy to Boundless

### Method 1: Using CLI
```bash
# Ensure enterprise backend is running
cd enterprise
cargo run --bin enterprise-server

# Deploy contract (in another terminal)
curl -X POST http://localhost:8080/api/contracts/deploy \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "wasm": "base64_encoded_wasm_here",
    "constructor": "new",
    "args": [],
    "gas_limit": 50000000
  }'
```

### Method 2: Using Deployment Script
```rust
use enterprise_contracts::deploy::*;

#[tokio::main]
async fn main() -> Result<()> {
    let config = DeploymentConfig {
        wasm_path: "target/ink/my_contract.wasm".to_string(),
        constructor: "new".to_string(),
        constructor_args: vec![],
        deployer_account: "your_account".to_string(),
        deployer_identity: "your_e2_identity".to_string(),
        gas_limit: 50_000_000,
        rpc_url: "http://localhost:9933".to_string(),
    };

    let result = deploy_contract(config).await?;
    println!("Deployed: {}", result.contract_address);
    Ok(())
}
```

## 6. Interact with Contract

```rust
use enterprise_contracts::deploy::ContractClient;

#[tokio::main]
async fn main() -> Result<()> {
    let client = ContractClient::new(
        contract_address,
        "target/ink/my_contract.json",
        "http://localhost:9933".to_string()
    )?;

    // Call read-only method
    let result = client.call("get_identity", args).await?;

    // Send transaction
    let tx_result = client.send(
        "register_identity",
        args,
        50_000_000
    ).await?;

    Ok(())
}
```

## Common Patterns

### Pattern 1: Identity Verification
```rust
// Always verify E2 identity before granting permissions
fn grant_permission(&mut self, account: AccountId) -> Result<()> {
    // 1. Verify identity exists
    let identity = self.verify_e2_identity(account)?;

    // 2. Check attestations
    self.require_attestation(&account, KYC_HASH)?;

    // 3. Grant permission
    self.permissions.insert(account, Permission::Write);
    Ok(())
}
```

### Pattern 2: Multi-Step Approval
```rust
// Require multiple approvals for sensitive operations
fn execute_high_value_tx(&mut self, tx_id: u64) -> Result<()> {
    let tx = self.transactions.get(tx_id)?;

    // 1. Check threshold
    if tx.approvals.len() < self.threshold {
        return Err(Error::InsufficientApprovals);
    }

    // 2. Verify not expired
    if self.env().block_timestamp() > tx.expires_at {
        return Err(Error::Expired);
    }

    // 3. Execute
    self.perform_transfer(&tx)?;
    Ok(())
}
```

### Pattern 3: Time-Lock Safety
```rust
// Add delays for security-critical operations
fn finalize_proposal(&mut self, proposal_id: u64) -> Result<()> {
    let proposal = self.proposals.get(proposal_id)?;

    // Require minimum delay
    let elapsed = self.env().block_timestamp() - proposal.created_at;
    if elapsed < MIN_DELAY {
        return Err(Error::DelayNotMet);
    }

    // Execute
    self.execute_proposal(&proposal)?;
    Ok(())
}
```

### Pattern 4: Resource Access Control
```rust
// OAuth-like scope checking
fn access_resource(&self, user: AccountId, app_id: [u8; 32]) -> Result<()> {
    // 1. Get resource requirements
    let resource = self.resources.get(resource_id)?;

    // 2. Check if app has required scopes
    let has_scopes = self.verify_scopes(
        user,
        app_id,
        &resource.required_scopes
    )?;

    if !has_scopes {
        return Err(Error::InsufficientScopes);
    }

    Ok(())
}
```

## Troubleshooting

### "Out of Fuel" Error
```bash
# Increase gas limit
gas_limit: 100_000_000  # Double the default
```

### "Storage Quota Exceeded"
```rust
// Use compact encoding
use ink::storage::pack::PackedLayout;

// Or prune old data
if self.storage_size() > MAX_SIZE {
    self.prune_old_entries()?;
}
```

### "Timeout" Error
```rust
// Break into smaller operations
fn batch_operation(&mut self, items: Vec<Item>) -> Result<()> {
    // Process in chunks
    const CHUNK_SIZE: usize = 100;
    for chunk in items.chunks(CHUNK_SIZE) {
        self.process_chunk(chunk)?;
    }
    Ok(())
}
```

## Next Steps

1. **Read Full Documentation**: See [README.md](README.md) for comprehensive guide
2. **Explore Examples**: Check existing contracts in `contracts/` directory
3. **Integration Guide**: Read [E2_INTEGRATION_IMPLEMENTATION.md](../../E2_INTEGRATION_IMPLEMENTATION.md)
4. **WASM Runtime**: Understand [wasm-runtime/](../../../wasm-runtime/)

## Support

- **Issues**: https://github.com/boundless-bls/issues
- **Docs**: https://docs.boundless.trust
- **Community**: https://discord.gg/boundless

---

**Ready to build?** Start with the template that matches your use case and customize it for your needs!
