# Contract Deployment Design

**Date**: November 17, 2025
**Status**: Design Complete

---

## Overview

This document outlines the design for real blockchain contract deployment in the Enterprise E2 Multipass system.

## Key Management Approach

### Option 1: Environment Variable (Chosen for Initial Implementation)

**Deployer Key Configuration:**
```bash
export DEPLOYER_PRIVATE_KEY="<hex-encoded-32-byte-ed25519-key>"
```

**Rationale:**
- Simple and straightforward
- Matches pattern used in CLI
- Easy to configure in deployment
- Can be rotated by updating environment

**Security:**
- Keep private key in secure secret management (e.g., HashiCorp Vault, AWS Secrets Manager)
- Never commit to version control
- Rotate regularly
- Use different keys for dev/staging/production

### Option 2: Encrypted Keystore (Future Enhancement)

Use the existing `enterprise/src/keystore/mod.rs` encrypted keystore:

**Configuration:**
```bash
export MASTER_ENCRYPTION_KEY="<64-hex-char-key>"
export DEPLOYER_KEY_ID="deployer-main"
```

**Benefits:**
- Encrypted at rest
- AES-256-GCM encryption
- Zeroizing for memory safety
- Can store multiple keys

**Implementation:** Phase 2 enhancement

---

## WASM Bytecode Loading

### Approach: Embedded Placeholder + External File Support

**Phase 1:** Embedded placeholder WASM
- Simple "hello world" contract bytecode
- Embedded in Rust binary as const
- Used for initial testing

**Phase 2:** External WASM files
- Load from `enterprise/contracts/wasm/` directory
- Support for template-based selection
- Future: On-demand compilation from ink! source

### WASM File Organization

```
enterprise/contracts/
├── wasm/                     # Compiled WASM binaries
│   ├── identity_access_control.wasm
│   ├── app_authorization.wasm
│   ├── asset_escrow.wasm
│   └── multisig_wallet.wasm
├── templates/                # Source templates
│   ├── identity_access_control.rs
│   ├── app_authorization.rs
│   ├── asset_escrow.rs
│   └── multisig_wallet.rs
└── abi/                      # Contract ABIs (future)
    └── ...
```

---

## Transaction Building Flow

### Step-by-Step Process

1. **Load Deployer Key**
   ```rust
   let deployer_key = load_deployer_key()?;
   let deployer_pubkey = derive_public_key(&deployer_key);
   let deployer_address = hash_public_key(&deployer_pubkey);
   ```

2. **Load WASM Bytecode**
   ```rust
   let wasm_bytes = load_wasm_for_template(&request.template_type)?;
   ```

3. **Query UTXOs**
   ```rust
   let utxos = client.get_utxos(&deployer_address).await?;
   ```

4. **Select UTXOs** (reuse CLI pattern)
   ```rust
   let (selected_utxos, total_input) = select_utxos_for_amount(
       &utxos,
       estimated_deployment_cost
   )?;
   ```

5. **Build Transaction**
   ```rust
   let tx = Transaction::new(
       1, // version
       build_inputs(&selected_utxos, &deployer_pubkey),
       vec![
           TxOutput {
               amount: 0, // Contract deployment (no value transfer)
               recipient_pubkey_hash: [0u8; 32], // Special contract address
               script: Some(wasm_bytes), // WASM in script field
           },
           // Change output if needed
       ],
       timestamp,
       None,
   );
   ```

6. **Sign Transaction**
   ```rust
   let signing_hash = tx.signing_hash();
   let signature = deployer_key.sign(&signing_hash);

   for input in &mut tx.inputs {
       input.signature = Signature::Classical(signature.to_bytes().to_vec());
   }
   ```

7. **Submit to Blockchain**
   ```rust
   let tx_hex = hex::encode(bincode::serialize(&tx)?);
   let tx_hash = client.send_transaction(&tx_hex).await?;
   ```

8. **Poll for Receipt**
   ```rust
   let receipt = poll_for_transaction_receipt(&tx_hash, client, timeout).await?;
   let contract_address = extract_contract_address(&receipt)?;
   ```

9. **Update Database**
   ```rust
   mark_deployed(contract_id, contract_address, tx_hash, gas_used).await?;
   ```

---

## Receipt Polling Strategy

### Polling Configuration

```rust
const POLL_INTERVAL: Duration = Duration::from_secs(2);
const MAX_POLL_ATTEMPTS: u32 = 30; // 60 seconds total
```

### Polling Logic

```rust
async fn poll_for_transaction_receipt(
    tx_hash: &str,
    client: &BlockchainClient,
    max_attempts: u32,
) -> Result<TransactionReceipt> {
    for attempt in 1..=max_attempts {
        match client.get_transaction(tx_hash).await? {
            Some(tx_info) if tx_info.block_height.is_some() => {
                // Transaction confirmed
                return Ok(TransactionReceipt {
                    tx_hash: tx_hash.to_string(),
                    block_height: tx_info.block_height.unwrap(),
                    gas_used: tx_info.gas_used.unwrap_or(0),
                    contract_address: extract_contract_address_from_tx(&tx_info)?,
                });
            }
            Some(_) => {
                // Still in mempool
                tracing::info!("Transaction {} pending (attempt {}/{})",
                    tx_hash, attempt, max_attempts);
            }
            None => {
                // Not found yet
                tracing::warn!("Transaction {} not found (attempt {}/{})",
                    tx_hash, attempt, max_attempts);
            }
        }

        if attempt < max_attempts {
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    Err(EnterpriseError::BlockchainError(
        format!("Transaction {} not confirmed after {} attempts",
            tx_hash, max_attempts)
    ))
}
```

---

## Contract Address Derivation

### Approach: Deterministic from Transaction Hash

**Method 1: Hash-based (Ethereum-style)**
```rust
fn derive_contract_address(deployer_address: &[u8; 32], nonce: u64) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(deployer_address);
    hasher.update(&nonce.to_le_bytes());
    hasher.finalize().into()
}
```

**Method 2: Transaction Hash-based (Simpler)**
```rust
fn extract_contract_address(tx_hash: &[u8; 32]) -> [u8; 32] {
    *tx_hash // Contract address = transaction hash
}
```

**Chosen:** Method 2 for simplicity in initial implementation

---

## Error Handling

### Failure Scenarios

1. **Deployer Key Not Found**
   - Error: `DEPLOYER_PRIVATE_KEY not set`
   - Action: Return error, keep contract in `Pending` status

2. **Insufficient Balance**
   - Error: `Insufficient funds for deployment`
   - Action: Return error with required amount

3. **WASM Load Failed**
   - Error: `Failed to load WASM for template X`
   - Action: Return error, suggest checking template availability

4. **Transaction Rejected**
   - Error: Blockchain rejection reason
   - Action: Mark contract as `Failed`, log rejection

5. **Confirmation Timeout**
   - Error: `Transaction not confirmed after N attempts`
   - Action: Keep contract in `Pending`, user can check manually

---

## Database Schema Updates

### Contract Status Enum

```sql
CREATE TYPE contract_status AS ENUM (
    'Pending',    -- Created but not deployed
    'Deploying',  -- Transaction submitted, waiting for confirmation
    'Deployed',   -- Successfully deployed and confirmed
    'Failed'      -- Deployment failed
);
```

### Additional Fields

- `deployment_tx_hash` - Transaction hash (populated when submitted)
- `deployment_attempts` - Number of deployment attempts
- `last_error` - Last error message if failed

---

## Configuration

### Environment Variables

```bash
# Required for real blockchain deployment
BOUNDLESS_HTTP_URL="http://localhost:3001"
DEPLOYER_PRIVATE_KEY="<hex-encoded-32-byte-key>"

# Optional
DEPLOYMENT_GAS_LIMIT="50000000"              # Default gas limit
DEPLOYMENT_POLL_TIMEOUT_SECONDS="60"         # Receipt polling timeout
DEPLOYMENT_POLL_INTERVAL_SECONDS="2"         # Poll interval
```

---

## Testing Strategy

### Unit Tests

1. **WASM Loading**
   - Test embedded bytecode loading
   - Test external file loading
   - Test invalid path handling

2. **Transaction Building**
   - Test with sufficient UTXOs
   - Test with insufficient UTXOs
   - Test signature generation

3. **Receipt Polling**
   - Test successful confirmation
   - Test timeout scenario
   - Test missing transaction

### Integration Tests

1. **End-to-End Deployment**
   - Deploy to local blockchain
   - Verify contract address
   - Check database updates

2. **Error Scenarios**
   - Missing key
   - Insufficient balance
   - Invalid WASM

---

## Timeline

| Task | Estimated Time | Priority |
|------|----------------|----------|
| WASM Loading | 1-2 hours | HIGH |
| Transaction Building | 2-3 hours | HIGH |
| Receipt Polling | 1-2 hours | HIGH |
| Update deploy_contract | 1-2 hours | HIGH |
| Testing | 2-3 hours | HIGH |
| Documentation | 1 hour | MEDIUM |

**Total**: 8-13 hours (1-2 days)

---

## Future Enhancements

1. **Encrypted Keystore Integration**
   - Use existing AES-256-GCM keystore
   - Support multiple deployer keys

2. **WASM Compilation**
   - On-demand compilation from ink! source
   - Caching of compiled bytecode

3. **Gas Estimation**
   - Pre-deployment gas estimation
   - Dynamic gas limit adjustment

4. **Multi-Sig Deployment**
   - Support for multi-signature deployments
   - Approval workflow integration

5. **Contract Upgrades**
   - Support for contract upgrade transactions
   - Version tracking

---

**Design Status**: ✅ Complete
**Implementation Status**: ⏳ In Progress
**Next Step**: Implement WASM loading module
