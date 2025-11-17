# UTXO-Native Smart Contract Architecture

**Date**: November 17, 2025
**Status**: Design Proposal
**Author**: Development Team

---

## Executive Summary

This document proposes a UTXO-native smart contract architecture for the Boundless BLS blockchain. Unlike Ethereum's account model, this design embraces the UTXO (Unspent Transaction Output) model while still enabling stateful smart contract execution.

**Key Design Principles**:
1. Contracts are UTXOs, not accounts
2. Contract state is stored separately from UTXO set (hybrid model)
3. WASM runtime executes contracts using function names, not selectors
4. Contract calls create transactions that consume UTXOs and update state

---

## Architecture Overview

### 1. Contract as UTXO

**Contract Deployment Transaction**:
```
Transaction {
  inputs: [UTXO with deployer funds for fees],
  outputs: [
    TxOutput {
      amount: 0,
      recipient_pubkey_hash: CONTRACT_DEPLOYMENT_MARKER, // Special marker: [0xFF; 32]
      script: Some(wasm_bytecode),
    },
    TxOutput {
      amount: change,
      recipient_pubkey_hash: deployer_address,
      script: None,
    }
  ],
  data: Some(ContractDeploymentData {
    deployer: [u8; 32],
    initial_state: Vec<u8>,
    metadata: Vec<u8>,
  }),
}
```

**Contract Address**: SHA3-256 hash of the deployment transaction hash

**Contract Registry** (stored in blockchain state):
```rust
pub struct ContractInfo {
    pub contract_address: [u8; 32],     // Derived from tx hash
    pub wasm_bytecode: Vec<u8>,         // Stored once at deployment
    pub deployer: [u8; 32],             // Original deployer
    pub deployed_at_height: u64,        // Block height
    pub deployed_at_tx: [u8; 32],       // Transaction hash
}
```

---

### 2. Contract State Storage (Hybrid Model)

Unlike pure UTXO systems, we maintain a **separate contract state tree** that coexists with the UTXO set:

```rust
pub struct ContractState {
    /// Contract address
    pub address: [u8; 32],

    /// Key-value storage (like Ethereum storage)
    /// Key: SHA3-256 hash of storage slot
    /// Value: arbitrary bytes (max 1024 bytes per slot)
    pub storage: HashMap<[u8; 32], Vec<u8>>,

    /// Storage quota (prevents bloat)
    pub storage_quota: u64,  // Max number of storage slots
    pub storage_used: u64,   // Current slots used

    /// Last modified block height
    pub last_modified: u64,
}
```

**State Transitions**:
- Contract state is updated when blocks are applied
- State changes are deterministic based on contract execution
- Rollback support: state changes are tracked per block height

---

### 3. Contract Call Transaction

**Write Call** (modifies state, requires transaction):
```
Transaction {
  inputs: [UTXO with caller funds for fees],
  outputs: [
    TxOutput {
      amount: change,
      recipient_pubkey_hash: caller_address,
      script: None,
    }
  ],
  data: Some(ContractCallData {
    contract_address: [u8; 32],
    function_name: String,
    args: Vec<u8>,          // Raw bytes, NOT ABI-encoded with selector
    caller: [u8; 32],
  }),
}
```

**Read Call** (view-only, no transaction):
- Executed via RPC without creating a transaction
- No state changes, no fees
- Returns result immediately

---

### 4. Transaction Types

Add to `core/src/tx_types.rs`:

```rust
pub enum TransactionType {
    Standard,
    ProofAnchor,
    AssetTransfer,
    AssetRegister,
    ContractDeployment,  // NEW
    ContractCall,        // NEW
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDeploymentData {
    /// Deployer address
    pub deployer: [u8; 32],

    /// Initial contract state (optional)
    pub initial_state: Vec<u8>,

    /// Contract metadata (name, version, etc.)
    pub metadata: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCallData {
    /// Target contract address
    pub contract_address: [u8; 32],

    /// Function name to call
    pub function_name: String,

    /// Raw arguments (serialized based on contract ABI)
    pub args: Vec<u8>,

    /// Caller address
    pub caller: [u8; 32],
}
```

---

### 5. Call Data Format (NOT Ethereum-style)

Since the WASM runtime expects `function_name: &str` and `args: &[u8]`, we use a simple format:

**Call Data Encoding**:
```
[function_name_len: u16][function_name: UTF-8][args: raw bytes]
```

Example:
```
Function: "transfer"
Args: [recipient: [u8; 32], amount: u64]

Encoded:
[0x08, 0x00] "transfer" [recipient_bytes...] [amount_bytes...]
  ^          ^           ^                      ^
  |          |           |                      |
  8 bytes    function    32 bytes               8 bytes (little-endian)
```

**Decoding**:
```rust
pub fn decode_call_data(data: &[u8]) -> Result<(String, Vec<u8>), String> {
    if data.len() < 2 {
        return Err("Call data too short".to_string());
    }

    let name_len = u16::from_le_bytes([data[0], data[1]]) as usize;
    if data.len() < 2 + name_len {
        return Err("Invalid function name length".to_string());
    }

    let function_name = String::from_utf8(data[2..2+name_len].to_vec())
        .map_err(|e| format!("Invalid UTF-8 in function name: {}", e))?;

    let args = data[2+name_len..].to_vec();

    Ok((function_name, args))
}
```

---

### 6. Contract Execution Flow

**Deployment**:
1. User creates deployment transaction with WASM bytecode in output script
2. Transaction is mined into a block
3. When block is applied, `apply_contract_deployment()` is called:
   - Extract WASM bytecode from transaction output
   - Validate WASM bytecode (size limits, valid module)
   - Calculate contract address (SHA3-256 of tx hash)
   - Store contract in ContractRegistry
   - Initialize contract state with `initial_state` from transaction data
   - Execute constructor if present
4. Contract address is returned to deployer

**Write Call** (State-Modifying):
1. User creates contract call transaction
2. Transaction is mined into a block
3. When block is applied, `apply_contract_call()` is called:
   - Load contract bytecode from registry
   - Load current contract state
   - Decode call data (function name + args)
   - Execute WASM: `runtime.execute(module, function_name, args, caller, block_height, timestamp)`
   - Apply state changes returned by contract
   - Charge fees based on computation (fuel used)
4. State changes are permanent and indexed

**Read Call** (View-Only):
1. User sends RPC request to `/api/v1/contract/query`
2. RPC handler:
   - Load contract bytecode
   - Load current contract state
   - Execute WASM with read-only flag
   - Return result (no state changes)
3. No transaction created, no fees

---

### 7. WASM Runtime Integration

The existing WASM runtime (wasm-runtime/src/runtime.rs) already supports:
- Function execution: `execute(function_name, args, ...)`
- Fuel metering for gas accounting
- Host functions for storage access

**Required Updates**:

1. **Add Contract Context to Host Functions**:
```rust
pub struct ContractExecutionContext {
    pub contract_address: [u8; 32],
    pub caller: [u8; 32],
    pub block_height: u64,
    pub timestamp: u64,
    pub state: Arc<RwLock<ContractState>>,
}
```

2. **Expose Storage Host Functions**:
```rust
// In wasm-runtime/src/host_functions.rs
fn storage_get(key_ptr: u32, key_len: u32) -> u32;
fn storage_set(key_ptr: u32, key_len: u32, value_ptr: u32, value_len: u32);
fn storage_remove(key_ptr: u32, key_len: u32);
fn get_caller() -> u32;  // Returns pointer to caller address
fn get_block_height() -> u64;
```

3. **Return State Changes**:
```rust
pub struct ExecutionResult {
    pub return_value: Vec<u8>,
    pub fuel_used: u64,
    pub state_changes: Vec<StateChange>,  // NEW
    pub logs: Vec<LogEntry>,
}

pub struct StateChange {
    pub key: [u8; 32],
    pub value: Option<Vec<u8>>,  // None = deletion
}
```

---

### 8. State Processing (core/src/state.rs)

Add to `BlockchainState`:

```rust
pub struct BlockchainState {
    // Existing fields...
    pub utxo_set: HashMap<OutPoint, TxOutput>,
    pub account_nonces: HashMap<[u8; 32], u64>,

    // NEW: Contract state
    pub contract_registry: HashMap<[u8; 32], ContractInfo>,
    pub contract_states: HashMap<[u8; 32], ContractState>,

    // NEW: WASM runtime instance
    wasm_runtime: Arc<WasmRuntime>,
}

impl BlockchainState {
    fn apply_contract_deployment(
        &mut self,
        tx: &Transaction,
        block_height: u64,
    ) -> Result<u64, StateError> {
        // Extract deployment data
        let deploy_data = extract_contract_deployment_data(tx)?;

        // Find output with WASM bytecode
        let wasm_bytecode = tx.outputs.iter()
            .find_map(|out| out.script.clone())
            .ok_or(StateError::InvalidTransaction("No WASM bytecode in deployment".to_string()))?;

        // Validate WASM bytecode
        validate_wasm_bytecode(&wasm_bytecode)?;

        // Calculate contract address
        let contract_address = sha3_256(&tx.hash());

        // Check for duplicate
        if self.contract_registry.contains_key(&contract_address) {
            return Err(StateError::InvalidTransaction("Contract already deployed".to_string()));
        }

        // Store contract info
        self.contract_registry.insert(contract_address, ContractInfo {
            contract_address,
            wasm_bytecode: wasm_bytecode.clone(),
            deployer: deploy_data.deployer,
            deployed_at_height: block_height,
            deployed_at_tx: tx.hash(),
        });

        // Initialize contract state
        let mut contract_state = ContractState::new(contract_address);

        // Apply initial state if provided
        if !deploy_data.initial_state.is_empty() {
            // Parse and apply initial state
            apply_initial_state(&mut contract_state, &deploy_data.initial_state)?;
        }

        // Execute constructor if present
        if contract_has_constructor(&wasm_bytecode) {
            let result = self.wasm_runtime.execute(
                &wasm_bytecode,
                "constructor",
                &deploy_data.initial_state,
                deploy_data.deployer,
                block_height,
                tx.timestamp,
            ).await?;

            // Apply state changes from constructor
            apply_state_changes(&mut contract_state, result.state_changes)?;
        }

        self.contract_states.insert(contract_address, contract_state);

        // Fixed deployment fee
        Ok(1000)
    }

    fn apply_contract_call(
        &mut self,
        tx: &Transaction,
        block_height: u64,
    ) -> Result<u64, StateError> {
        // Extract call data
        let call_data = extract_contract_call_data(tx)?;

        // Get contract info
        let contract_info = self.contract_registry.get(&call_data.contract_address)
            .ok_or(StateError::InvalidTransaction("Contract not found".to_string()))?;

        // Get contract state
        let contract_state = self.contract_states.get_mut(&call_data.contract_address)
            .ok_or(StateError::InvalidTransaction("Contract state not found".to_string()))?;

        // Execute contract
        let result = self.wasm_runtime.execute(
            &contract_info.wasm_bytecode,
            &call_data.function_name,
            &call_data.args,
            call_data.caller,
            block_height,
            tx.timestamp,
        ).await?;

        // Apply state changes
        apply_state_changes(contract_state, result.state_changes)?;

        // Update last modified
        contract_state.last_modified = block_height;

        // Fee based on fuel used
        let fee = calculate_contract_fee(result.fuel_used);
        Ok(fee)
    }
}
```

---

### 9. RPC Endpoints (rpc/src/http_bridge.rs)

Add to router:

```rust
.route("/api/v1/contract/deploy", post(deploy_contract))
.route("/api/v1/contract/call", post(call_contract))
.route("/api/v1/contract/query", post(query_contract))
.route("/api/v1/contract/:address", get(get_contract_info))
.route("/api/v1/contract/:address/state", get(get_contract_state))
```

**Handlers**:

```rust
#[derive(Deserialize)]
struct DeployContractRequest {
    wasm_hex: String,           // Hex-encoded WASM bytecode
    initial_state: Option<String>,  // Hex-encoded initial state
    metadata: Option<String>,   // Hex-encoded metadata
}

#[derive(Serialize)]
struct DeployContractResponse {
    contract_address: String,
    tx_hash: String,
    success: bool,
}

async fn deploy_contract<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Json(req): Json<DeployContractRequest>,
) -> Result<Json<DeployContractResponse>, ApiError> {
    // Decode WASM bytecode
    let wasm_bytes = hex::decode(&req.wasm_hex)?;

    // Build deployment transaction
    // (Caller needs to provide signed transaction, not just WASM)
    // This is simplified - real implementation would require full transaction

    // Submit to blockchain
    // Return contract address
}

#[derive(Deserialize)]
struct ContractCallRequest {
    contract_address: String,
    function_name: String,
    args_hex: String,  // Hex-encoded arguments
}

#[derive(Serialize)]
struct ContractCallResponse {
    tx_hash: String,
    success: bool,
}

async fn call_contract<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Json(req): Json<ContractCallRequest>,
) -> Result<Json<ContractCallResponse>, ApiError> {
    // Build contract call transaction
    // Submit to blockchain
    // Return tx hash
}

#[derive(Deserialize)]
struct QueryContractRequest {
    contract_address: String,
    function_name: String,
    args_hex: String,
    caller: Option<String>,  // Optional caller for context
}

#[derive(Serialize)]
struct QueryContractResponse {
    result_hex: String,
    fuel_used: u64,
}

async fn query_contract<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Json(req): Json<QueryContractRequest>,
) -> Result<Json<QueryContractResponse>, ApiError> {
    let chain = state.blockchain.read().await;

    // Load contract
    let contract_address = parse_address(&req.contract_address)?;
    let contract_info = chain.get_contract(&contract_address)?;

    // Decode args
    let args = hex::decode(&req.args_hex)?;

    // Execute read-only
    let result = chain.query_contract_readonly(
        &contract_address,
        &req.function_name,
        &args,
        req.caller.as_ref().map(|c| parse_address(c)).transpose()?,
    ).await?;

    Ok(Json(QueryContractResponse {
        result_hex: hex::encode(&result.return_value),
        fuel_used: result.fuel_used,
    }))
}
```

---

### 10. ABI Encoding Adjustments

The current ABI implementation needs modification:

**Before** (Ethereum-style):
```
[4-byte selector][encoded params]
```

**After** (UTXO-native):
```
[2-byte function_name_len][function_name][raw params]
```

**Updated ABI Encoder**:
```rust
pub fn encode_function_call(function: &AbiFunction, params: &JsonValue) -> Result<Vec<u8>> {
    let mut encoded = Vec::new();

    // Add function name (NOT selector)
    let name_bytes = function.name.as_bytes();
    let name_len = name_bytes.len() as u16;
    encoded.extend_from_slice(&name_len.to_le_bytes());
    encoded.extend_from_slice(name_bytes);

    // Encode parameters (existing logic)
    for param in &function.inputs {
        let value = params.get(&param.name)?;
        let param_bytes = encode_value(&param.param_type, value)?;
        encoded.extend_from_slice(&param_bytes);
    }

    Ok(encoded)
}
```

---

## Implementation Roadmap

### Phase 1: Core Infrastructure (3-4 days)
1. Add `ContractDeployment` and `ContractCall` transaction types
2. Implement `ContractInfo` and `ContractState` storage
3. Add contract registry to `BlockchainState`
4. Implement `apply_contract_deployment()` and `apply_contract_call()`

### Phase 2: WASM Runtime Integration (2-3 days)
1. Add contract context to runtime
2. Implement storage host functions
3. Add state change tracking to execution results
4. Test contract execution end-to-end

### Phase 3: RPC Endpoints (2 days)
1. Implement `/api/v1/contract/query` endpoint
2. Implement `/api/v1/contract/call` endpoint
3. Implement contract info/state query endpoints
4. Add contract deployment support

### Phase 4: ABI Updates (1 day)
1. Modify ABI encoder to use function names instead of selectors
2. Update ABI decoder for new format
3. Update contract templates to match new encoding

### Phase 5: Testing & Documentation (2-3 days)
1. Write comprehensive contract tests
2. Test state persistence and rollback
3. Test contract interactions
4. Update documentation

**Total Estimated Time**: 10-13 days

---

## Security Considerations

1. **WASM Validation**: All bytecode must be validated before deployment
2. **Fuel Metering**: Prevent infinite loops via fuel limits
3. **Storage Quotas**: Prevent state bloat with per-contract storage limits
4. **Determinism**: Contract execution must be deterministic across all nodes
5. **State Rollback**: Support blockchain reorganizations with state rollback
6. **Access Control**: Contracts can only modify their own state

---

## Comparison with Ethereum

| Feature | Ethereum | Boundless UTXO |
|---------|----------|----------------|
| Transaction Model | Account-based | UTXO-based |
| Contract Storage | Account storage | Separate state tree |
| Call Format | Function selector (4 bytes) | Function name (variable) |
| State Location | In account | Hybrid (UTXO + state) |
| Gas Model | Gas per opcode | Fuel per WASM instruction |
| Deployment | Contract creation tx | Special UTXO output |

---

## Next Steps

1. Review and approve this design
2. Begin Phase 1 implementation
3. Create detailed implementation tasks
4. Assign development resources

---

**Questions?** Contact development team for clarification.
