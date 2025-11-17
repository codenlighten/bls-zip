# Smart Contract Examples

This directory contains practical examples for deploying and using the Enterprise E2 Multipass smart contract templates.

## Files

### Configuration

- **deployment_config.toml** - Complete deployment configuration for all templates
  - Network settings (local, testnet, production)
  - Contract-specific settings
  - E2 Multipass integration config
  - Testing configuration

### Example Code

- **e2_integration_example.rs** - Comprehensive integration examples showing:
  - Identity registration with access control
  - Multi-signature wallet workflow
  - Asset trading with escrow
  - Application authorization flow
  - E2 Multipass API integration

### Scripts

- **deploy_and_test.sh** - Bash script for automated deployment (Linux/Mac)
- **deploy_and_test.ps1** - PowerShell script for automated deployment (Windows)

## Quick Start

### 1. Prerequisites

Ensure the following are running:

```bash
# E2 Multipass backend
cd enterprise
cargo run --bin enterprise-server
# Running on http://localhost:8080

# Boundless blockchain (3 nodes)
docker-compose up -d
# Node 1: http://localhost:9933

# E2 Frontend (optional)
cd enterprise/frontend
npm run dev
# Running on http://localhost:3001
```

### 2. Run Deployment Script

**On Windows:**
```powershell
cd enterprise/contracts/examples
.\deploy_and_test.ps1
```

**On Linux/Mac:**
```bash
cd enterprise/contracts/examples
chmod +x deploy_and_test.sh
./deploy_and_test.sh
```

This will:
- ✓ Check prerequisites
- ✓ Build all contract templates
- ✓ Run unit tests
- ✓ Deploy contracts (simulated)
- ✓ Run E2 integration tests

### 3. Run Integration Examples

```bash
cd enterprise/contracts
cargo run --example e2_integration
```

This demonstrates:
- Logging into E2 Multipass
- Registering identities in contracts
- Creating multi-sig wallets
- Trading assets with escrow
- Managing app permissions

## Example Workflows

### Identity & Access Control

```bash
# 1. Login to E2 Multipass
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@boundless.local","password":"BoundlessTrust@2024"}'

# 2. Get your identity ID from response
# identity_id: "c9ad4c39-c9f3-463d-ad66-c78649905b87"

# 3. Register identity in contract
# contract.register_identity(identity_id, Role::User)

# 4. Grant admin role
# contract.grant_role(account, Role::Admin)

# 5. Check permissions
# contract.has_role(account, Role::Admin) // true
```

### Multi-Signature Wallet

```bash
# 1. Create 2-of-3 wallet
signers = [
  (alice, alice_identity, 10_000),
  (bob, bob_identity, 10_000),
  (charlie, charlie_identity, 10_000)
]
wallet = MultisigWallet::new(signers, 2)

# 2. Propose transaction
tx_id = wallet.propose_transaction(recipient, 1000, None, None)

# 3. Approve (need 2 signatures)
wallet.approve_transaction(tx_id)  // Alice
wallet.approve_transaction(tx_id)  // Bob (threshold reached!)

# 4. Execute
wallet.execute_transaction(tx_id)
```

### Asset Trading

```bash
# 1. Get asset balances from E2
curl http://localhost:8080/api/assets/balances \
  -H "Authorization: Bearer $TOKEN"

# 2. Propose trade
trade_id = escrow.propose_trade(
  identity,
  offer_assets: [IRSC: 100],
  request_assets: [CRSC: 200],
  counterparty: Bob,
  None, None
)

# 3. Bob accepts and locks assets
escrow.accept_trade(trade_id, bob_identity)

# 4. Both confirm
escrow.confirm_trade(trade_id)  // Alice
escrow.confirm_trade(trade_id)  // Bob
# Trade completed!
```

### Application Authorization

```bash
# 1. Register app
auth.register_application(
  app_id,
  "My DeFi App",
  owner_identity,
  ["https://myapp.com/callback"],
  [ReadProfile, ReadWallet]
)

# 2. User grants permissions
grant_id = auth.issue_grant(
  app_id,
  user_identity,
  [ReadProfile, ReadWallet],
  30_days,
  delegatable: true
)

# 3. App checks access
can_access = auth.can_access_resource(user, resource_id, app_id)

# 4. User delegates to sub-service
delegation_id = auth.create_delegation(
  grant_id,
  sub_service_account,
  sub_service_identity,
  [ReadProfile],  // Subset of original scopes
  7_days
)
```

## Configuration Guide

### deployment_config.toml Structure

```toml
# Contract configuration
[contract_name]
wasm_path = "path/to/contract.wasm"
constructor = "new"
constructor_args = [...]
deployer_account = "substrate_address"
deployer_identity = "uuid"
gas_limit = 50_000_000
rpc_url = "http://localhost:9933"

# Network configuration
[network]
node1_rpc = "http://localhost:9933"
e2_api_url = "http://localhost:8080"

# Deployment settings
[deployment]
max_gas_price = 1000
timeout_ms = 300_000
confirmations = 3
```

### Customizing for Your Network

1. **Local Development**: Use default settings
2. **Testnet**: Update `rpc_url` and `e2_api_url`
3. **Production**: Use `[production]` section settings

## Testing

### Unit Tests

```bash
# Test individual contract
cd enterprise/contracts/templates
cargo test --lib identity_access_control

# Test all contracts
cargo test
```

### Integration Tests

```bash
# Run integration examples
cargo run --example e2_integration

# Expected output:
# ✓ Identity registration works
# ✓ Multi-sig wallet works
# ✓ Asset escrow works
# ✓ App authorization works
```

### Manual Testing

1. Deploy contracts using scripts
2. Open frontend: http://localhost:3001
3. Login with: admin@boundless.local / BoundlessTrust@2024
4. Interact with contracts through UI

## Troubleshooting

### Contract Build Fails

```bash
# Ensure cargo-contract is installed
cargo install cargo-contract --force

# Check Rust toolchain
rustup target add wasm32-unknown-unknown

# Build manually
cargo contract build --release
```

### Deployment Fails

```bash
# Check E2 backend is running
curl http://localhost:8080/api/auth/health

# Check blockchain is running
curl http://localhost:9933

# Verify you have gas/balance
# Check deployer account has funds
```

### Integration Test Fails

```bash
# Verify credentials
email: admin@boundless.local
password: BoundlessTrust@2024

# Check network connectivity
curl http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@boundless.local","password":"BoundlessTrust@2024"}'

# Expected: JWT token in response
```

## Next Steps

1. **Customize Templates**: Adapt templates for your use case
2. **Deploy to Testnet**: Update config for testnet deployment
3. **Build Frontend Integration**: Connect contracts to your UI
4. **Production Deployment**: Follow production checklist

## Resources

- [Contract Templates](../templates/)
- [Deployment Utilities](../templates/deploy.rs)
- [Main Documentation](../README.md)
- [E2 Integration Guide](../../E2_INTEGRATION_IMPLEMENTATION.md)
- [Frontend Integration](../../frontend/INTEGRATION.md)

## Support

For issues or questions:
- Check [Troubleshooting](#troubleshooting) section
- Review [Main README](../README.md)
- Open GitHub issue: https://github.com/boundless-bls/issues
