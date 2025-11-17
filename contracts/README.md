# Boundless BLS Smart Contracts

This directory contains sample smart contracts demonstrating the capabilities of the Boundless BLS platform.

## Contracts

### 1. Token Contract (`token/`)
Standard fungible token implementation with:
- Transfer, mint, burn operations
- Balance tracking
- PQC-aware signature verification

### 2. Private Voting Contract (`voting/`)
Privacy-preserving voting using Paillier PHE:
- Encrypted vote tallying
- Public result verification
- Zero-knowledge voter anonymity

### 3. Escrow Contract (`escrow/`)
Multi-party escrow with:
- Time-locked funds
- Multi-signature release
- Dispute resolution

## Building Contracts

```bash
# Install cargo-contract for ink! contracts
cargo install cargo-contract --force

# Build a contract
cd token
cargo contract build --release

# This produces:
# - target/ink/token.wasm (WASM binary)
# - target/ink/token.json (metadata)
```

## Testing Contracts

```bash
cd token
cargo test
```

## Deploying Contracts

Contracts are deployed as WASM bytecode to the Boundless BLS blockchain. The runtime will:
1. Validate the WASM module
2. Compile it with Wasmtime
3. Instantiate with fuel limits
4. Execute with deterministic gas metering

## ink! vs Raw WASM

While ink! provides a convenient Rust DSL for contracts, the Boundless runtime also supports raw WASM modules compiled from any language (AssemblyScript, C, Rust without ink!, etc.) as long as they:
- Export required functions (`deploy`, `call`)
- Implement `allocate`/`deallocate` for memory management
- Stay within fuel and memory limits
