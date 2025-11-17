# Implementation Fixes - Boundless BLS Platform

**Date**: November 17, 2025
**Status**: In Progress

## Overview

This document tracks the fixes being applied to resolve the stubs, mocks, and incomplete implementations found in the codebase audit.

---

## Fix 1: Enterprise Contract Deployment Integration

### Issue
The Enterprise E2 Multipass contract service contains mock implementations for:
- Contract deployment (`enterprise/src/services/contract.rs:346`)
- Contract calls (`enterprise/src/services/contract.rs:440`)
- Transaction sending (`enterprise/src/services/contract.rs:483`)
- WASM bytecode loading (`enterprise/src/services/contract.rs:626`)

### Root Cause
The Enterprise layer was built before the blockchain integration layer was completed. It was designed to work with a running Boundless blockchain node via RPC, but:
1. No RPC client was implemented in the Enterprise service
2. WASM contracts have not been compiled from ink! source files
3. Transaction building logic with UTXOs was not implemented

### Solution Approach

**Phase 1: Add Blockchain RPC Client**
1. Create `enterprise/src/blockchain/mod.rs` - RPC client for Boundless node
2. Add methods for:
   - Transaction submission
   - UTXO queries
   - Balance/nonce queries
   - Transaction status polling

**Phase 2: Implement WASM Contract Compilation**
1. Add build script to compile ink! contracts to .wasm
2. Store compiled .wasm files in `enterprise/contracts/templates/build/`
3. Update `get_template_wasm()` to load actual compiled files

**Phase 3: Implement Transaction Building**
1. Query UTXOs for the deployer address
2. Build proper transaction with:
   - Valid inputs (UTXOs)
   - Outputs (change + contract deployment)
   - Data field (WASM bytecode + constructor args)
3. Sign transaction with PQC keys
4. Submit via RPC

**Phase 4: Graceful Degradation**
1. Detect if blockchain node is available
2. If available: Use actual blockchain integration
3. If unavailable: Use mock mode with clear warnings in logs
4. Add configuration flag `BLOCKCHAIN_NODE_URL`

### Prerequisites
- Boundless blockchain node must be running (e.g., `http://localhost:9933`)
- Node must have RPC enabled
- Deployer must have sufficient balance and UTXOs
- ink! contracts must be compiled to WASM

### Implementation Status
- [IN PROGRESS] Creating RPC client module
- [PENDING] Implementing transaction builder
- [PENDING] Adding WASM compilation
- [PENDING] Updating contract service to use actual blockchain

---

## Fix 2: CLI Transaction Creation (UTXO Selection)

### Issue
The CLI transaction creation tool uses placeholder UTXOs (`cli/src/tx.rs:60-73`), making it impossible to actually send transactions.

### Solution
1. Add `chain_getUtxos` RPC method to blockchain node
2. Implement UTXO selection algorithm in CLI
3. Build transactions with real UTXOs
4. Add proper error handling for insufficient funds

### Implementation Status
- [PENDING] Start after Fix 1 is complete

---

## Fix 3: RPC Proof Anchoring Placeholder Inputs

### Issue
The HTTP bridge proof anchoring endpoint uses placeholder transaction inputs (`rpc/src/http_bridge.rs:466-474`).

### Solution
1. Require UTXO inputs from client
2. Validate inputs before creating transaction
3. Return proper error if inputs are invalid
4. Add endpoint documentation

### Implementation Status
- [PENDING] Start after Fix 1 is complete

---

## Fix 4: Contract Deployment Utilities

### Issue
The `enterprise/contracts/templates/deploy.rs` helper utilities are stubs.

### Solution
1. Implement RPC calls to Boundless node
2. Add transaction building logic
3. Implement receipt polling
4. Add comprehensive error handling

### Implementation Status
- [PENDING] Start after Fix 1 is complete

---

## Fix 5: E2 Smart Contract Template Integration

### Issue
Smart contract templates have placeholder TODOs for E2 Multipass asset service integration:
- `asset_escrow.rs:319` - Asset locking via E2
- `multisig_wallet.rs:436` - Asset transfers

### Solution
1. Implement E2 asset service RPC client
2. Add asset locking/unlocking functions
3. Implement transfer logic
4. Add integration tests

### Implementation Status
- [PENDING] Start after Fix 1 is complete

---

## Fix 6: P2P Bootnode Configuration

### Issue
P2P network has empty bootnode list (`node/src/main_old.rs:133`).

### Solution
1. Add `bootnodes` field to config file
2. Load bootnodes from environment or config
3. Document bootnode format in README
4. Add example bootnodes for testnet

### Implementation Status
- [PENDING] Quick fix - 30 minutes

---

## Fix 7: Legacy Code Cleanup

### Issues
- `node/src/main_old.rs` should be removed or archived
- Some FIXME comments should be NOTE
- Misleading "simplified" comments on correct implementations

### Solution
1. Remove or archive `main_old.rs`
2. Update comment terminology
3. Remove misleading comments

### Implementation Status
- [PENDING] Quick fix - 1 hour

---

## Testing Plan

After all fixes are implemented:

1. **Unit Tests**
   - RPC client methods
   - Transaction building
   - UTXO selection
   - Contract deployment

2. **Integration Tests**
   - Deploy contract via Enterprise API
   - Call contract methods
   - Send transactions
   - Query contract state

3. **End-to-End Tests**
   - Full workflow: compile -> deploy -> interact
   - Multi-contract interactions
   - E2 Multipass integration

4. **Performance Tests**
   - Contract deployment time
   - Call latency
   - Transaction throughput

---

## Timeline Estimate

- **Fix 1**: 3-5 days (RPC client + transaction building + WASM loading)
- **Fix 2**: 2-3 days (CLI UTXO selection)
- **Fix 3**: 1-2 days (RPC proof anchoring)
- **Fix 4**: 2-3 days (Deployment utilities)
- **Fix 5**: 3-4 days (E2 integration)
- **Fix 6**: 0.5 days (Bootnode config)
- **Fix 7**: 1 day (Cleanup)
- **Testing**: 2-3 days

**Total**: 15-22 days for complete implementation

---

## Notes

- All fixes maintain backward compatibility
- Mock mode available for development/testing without blockchain node
- Comprehensive error messages guide users on setup requirements
- Documentation updated alongside code changes
