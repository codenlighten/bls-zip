# Boundless BLS - Phase 1 Completion Plan

**Status**: 65% Complete
**Target**: Functional blockchain with full node capability
**Timeline**: 6-8 weeks
**Last Updated**: November 14, 2025

---

## Executive Summary

Phase 1 has established strong foundations in cryptography, consensus, and smart contracts. However, **critical infrastructure components** are missing:

- ❌ No transaction signature verification
- ❌ No blockchain state management
- ❌ No full node implementation
- ❌ No P2P networking
- ❌ No RPC API
- ❌ No persistent storage

This plan outlines the steps to complete Phase 1 and achieve a **functional, testable blockchain**.

---

## Critical Path (Weeks 1-3)

### Week 1: Core Transaction & State Management

**Priority 1: Transaction Signature Verification** (3-5 days)
- [ ] Implement signature verification in `Transaction::validate()`
- [ ] Add public key recovery from addresses
- [ ] Support hybrid (Classical + PQC) signature validation
- [ ] Add comprehensive signature tests
- [ ] Test with all three PQC algorithms (ML-DSA, Falcon, Ed25519)

**Priority 2: Blockchain State Management** (4-6 days)
- [ ] Create `BlockchainState` struct with UTXO tracking
- [ ] Implement state transitions for block application
- [ ] Add double-spend prevention
- [ ] Account balance tracking
- [ ] Account nonce enforcement
- [ ] State rollback for reorgs

**Deliverable**: Transactions can be fully validated with signature verification

---

### Week 2: Node Binary & Block Production

**Priority 3: Full Node Implementation** (5-7 days)
- [ ] Create `node/` crate with main binary
- [ ] Implement block production loop
- [ ] Add block validation pipeline
- [ ] Integrate mining with consensus
- [ ] Add genesis block initialization
- [ ] CLI argument parsing (mining, RPC ports, data dir)

**Priority 4: Transaction Mempool** (3-4 days)
- [ ] Create mempool data structure
- [ ] Fee-based transaction ordering
- [ ] Double-spend detection in mempool
- [ ] Transaction expiration
- [ ] Mempool limits and eviction

**Deliverable**: Runnable node that can mine blocks and validate transactions

---

### Week 3: Persistence & RPC

**Priority 5: Storage Layer** (4-5 days)
- [ ] Integrate RocksDB
- [ ] Implement block storage
- [ ] Implement state storage (UTXO set)
- [ ] Add chain reorg handling
- [ ] Snapshot/checkpoint functionality
- [ ] Database migration system

**Priority 6: JSON-RPC API** (3-4 days)
- [ ] Create `rpc/` crate with jsonrpsee
- [ ] Implement core methods:
  - `chain_getBlockByHeight`
  - `chain_getBlockByHash`
  - `chain_getTransaction`
  - `chain_submitTransaction`
  - `chain_getBalance`
  - `chain_getBlockHeight`
- [ ] Add WebSocket support for subscriptions
- [ ] CORS configuration

**Deliverable**: Persistent blockchain with queryable RPC interface

---

## Phase 2 Foundation (Weeks 4-6)

### Week 4: P2P Networking Foundation

**Priority 7: Network Layer** (7-10 days)
- [ ] Create `p2p/` crate with libp2p
- [ ] Implement peer discovery (mDNS, Kademlia)
- [ ] Block propagation (gossipsub)
- [ ] Transaction propagation
- [ ] Sync protocol for new nodes
- [ ] Network message types
- [ ] Peer reputation system

**Deliverable**: Nodes can discover each other and sync blockchain

---

### Week 5: Smart Contract & Frontend Integration

**Priority 8: Smart Contract Fixes** (3-4 days)
- [ ] Add access control to token mint function
- [ ] Implement actual PHE operations in voting contract
- [ ] Add zero-knowledge proof verification
- [ ] Improve escrow contract (milestones, partial release)
- [ ] Add more comprehensive tests

**Priority 9: Contract Deployment System** (3-4 days)
- [ ] Contract upload transaction type
- [ ] WASM bytecode validation
- [ ] Contract address derivation
- [ ] Contract state storage
- [ ] Contract execution integration

**Priority 10: Frontend RPC Integration** (3-4 days)
- [ ] Connect to real RPC endpoint
- [ ] Implement transaction signing
- [ ] Real-time balance updates
- [ ] Transaction history from chain
- [ ] Event monitoring (blocks, transactions)
- [ ] Contract deployment UI

**Deliverable**: Full contract deployment and interaction via dApp

---

### Week 6: Testing & Hardening

**Priority 11: Integration Testing** (5-7 days)
- [ ] Multi-node network tests
- [ ] Block propagation tests
- [ ] Fork and reorg tests
- [ ] Contract execution tests
- [ ] Load/stress tests
- [ ] Network partition recovery

**Priority 12: Security Hardening** (3-4 days)
- [ ] Input validation everywhere
- [ ] Cryptographic randomness for nonces
- [ ] Replay attack prevention tests
- [ ] DoS protection (rate limiting)
- [ ] Fuzzing infrastructure setup

**Deliverable**: Battle-tested blockchain ready for testnet

---

## Detailed Implementation Tasks

### 1. Transaction Signature Verification

**File**: `core/src/transaction.rs`

```rust
impl Transaction {
    /// Validate transaction signature with PQC support
    pub fn verify_signature(
        &self,
        public_key: &[u8],
        signature_type: &SignatureType,
    ) -> Result<bool, CoreError> {
        let tx_hash = self.hash();

        match self.signature.as_ref() {
            Some(Signature::Hybrid { classical, pqc }) => {
                // Verify both classical and PQC signatures
                let hybrid_verifier = HybridSignature::new()?;
                let hybrid_sig = HybridSignatureData {
                    classical: classical.clone(),
                    pqc: pqc.clone(),
                };
                let hybrid_pk = HybridSignaturePublicKey::from_bytes(public_key)?;

                hybrid_verifier.verify(&tx_hash, &hybrid_sig, &hybrid_pk)
            }
            Some(Signature::MlDsa(sig)) => {
                let verifier = MlDsa44::new()?;
                verifier.verify(&tx_hash, sig, public_key)
            }
            Some(Signature::Falcon(sig)) => {
                let verifier = Falcon512::new()?;
                verifier.verify(&tx_hash, sig, public_key)
            }
            Some(Signature::Classical(sig)) => {
                // Ed25519 verification
                let vk = VerifyingKey::from_bytes(
                    public_key.try_into()
                        .map_err(|_| CoreError::InvalidPublicKey)?
                )?;
                let ed_sig = Ed25519Signature::from_bytes(
                    sig.as_slice().try_into()
                        .map_err(|_| CoreError::InvalidSignature)?
                );
                Ok(vk.verify(&tx_hash, &ed_sig).is_ok())
            }
            None => Err(CoreError::MissingSignature),
        }
    }
}
```

**Tests Required**:
- Valid signatures for each type
- Invalid signatures
- Mismatched public keys
- Corrupted signatures
- Edge cases (empty, too large)

---

### 2. Blockchain State Management

**File**: `core/src/state.rs` (NEW)

```rust
use std::collections::HashMap;
use crate::{Block, Transaction, TxInput, TxOutput};

/// Unique identifier for a transaction output
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct OutPoint {
    pub tx_hash: [u8; 32],
    pub index: u32,
}

/// Blockchain state with UTXO tracking
pub struct BlockchainState {
    /// UTXO set: OutPoint -> TxOutput
    utxo_set: HashMap<OutPoint, TxOutput>,

    /// Account nonces for replay protection
    account_nonces: HashMap<[u8; 32], u64>,

    /// Current block height
    block_height: u64,

    /// Total supply
    total_supply: u64,
}

impl BlockchainState {
    pub fn new() -> Self {
        Self {
            utxo_set: HashMap::new(),
            account_nonces: HashMap::new(),
            block_height: 0,
            total_supply: 0,
        }
    }

    /// Apply a block to the state
    pub fn apply_block(&mut self, block: &Block) -> Result<(), StateError> {
        // Validate block height
        if block.header.height != self.block_height + 1 {
            return Err(StateError::InvalidBlockHeight);
        }

        // Process each transaction
        for tx in &block.transactions {
            self.apply_transaction(tx)?;
        }

        self.block_height += 1;
        Ok(())
    }

    /// Apply a transaction to the state
    fn apply_transaction(&mut self, tx: &Transaction) -> Result<(), StateError> {
        let tx_hash = tx.hash();

        // Verify inputs exist and consume them
        let mut input_sum = 0u64;
        for input in &tx.inputs {
            let outpoint = OutPoint {
                tx_hash: input.previous_output_hash,
                index: input.output_index,
            };

            let output = self.utxo_set.remove(&outpoint)
                .ok_or(StateError::UTXONotFound)?;

            input_sum = input_sum.checked_add(output.amount)
                .ok_or(StateError::ArithmeticOverflow)?;

            // Verify nonce if applicable
            if let Some(nonce) = input.nonce {
                let current_nonce = self.account_nonces
                    .get(&output.recipient_pubkey_hash)
                    .copied()
                    .unwrap_or(0);

                if nonce != current_nonce {
                    return Err(StateError::InvalidNonce);
                }

                self.account_nonces.insert(
                    output.recipient_pubkey_hash,
                    nonce + 1
                );
            }
        }

        // Add new outputs
        let mut output_sum = 0u64;
        for (index, output) in tx.outputs.iter().enumerate() {
            let outpoint = OutPoint {
                tx_hash,
                index: index as u32,
            };

            self.utxo_set.insert(outpoint, output.clone());

            output_sum = output_sum.checked_add(output.amount)
                .ok_or(StateError::ArithmeticOverflow)?;
        }

        // Verify fees
        if input_sum < output_sum {
            return Err(StateError::InsufficientInputs);
        }

        Ok(())
    }

    /// Get account balance
    pub fn get_balance(&self, pubkey_hash: &[u8; 32]) -> u64 {
        self.utxo_set.values()
            .filter(|output| &output.recipient_pubkey_hash == pubkey_hash)
            .map(|output| output.amount)
            .sum()
    }

    /// Get account nonce
    pub fn get_nonce(&self, pubkey_hash: &[u8; 32]) -> u64 {
        self.account_nonces.get(pubkey_hash).copied().unwrap_or(0)
    }

    /// Check if UTXO exists
    pub fn has_utxo(&self, outpoint: &OutPoint) -> bool {
        self.utxo_set.contains_key(outpoint)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("UTXO not found")]
    UTXONotFound,

    #[error("Invalid block height")]
    InvalidBlockHeight,

    #[error("Invalid nonce")]
    InvalidNonce,

    #[error("Insufficient inputs")]
    InsufficientInputs,

    #[error("Arithmetic overflow")]
    ArithmeticOverflow,
}
```

---

### 3. Full Node Binary

**File**: `node/Cargo.toml` (NEW)

```toml
[package]
name = "boundless-node"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "boundless-node"
path = "src/main.rs"

[dependencies]
boundless-core = { path = "../core" }
boundless-consensus = { path = "../consensus" }
boundless-crypto = { path = "../crypto" }
boundless-wasm-runtime = { path = "../wasm-runtime" }

tokio = { version = "1.35", features = ["full"] }
clap = { version = "4.4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

**File**: `node/src/main.rs` (NEW)

```rust
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, error};

mod blockchain;
mod config;
mod mempool;

use blockchain::Blockchain;

#[derive(Parser, Debug)]
#[command(name = "boundless-node")]
#[command(about = "Boundless BLS Blockchain Node", long_about = None)]
struct Args {
    /// Run in development mode with easy mining
    #[arg(long)]
    dev: bool,

    /// Enable mining
    #[arg(long)]
    mining: bool,

    /// Mining coinbase address
    #[arg(long)]
    coinbase: Option<String>,

    /// Data directory
    #[arg(long, default_value = "./data")]
    base_path: PathBuf,

    /// P2P listen port
    #[arg(long, default_value = "30333")]
    port: u16,

    /// RPC HTTP port
    #[arg(long, default_value = "9933")]
    rpc_port: u16,

    /// RPC WebSocket port
    #[arg(long, default_value = "9944")]
    ws_port: u16,

    /// Config file
    #[arg(long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    let args = Args::parse();

    info!("Starting Boundless BLS Node");
    info!("Data directory: {:?}", args.base_path);

    // Load or create configuration
    let config = if let Some(config_path) = args.config {
        config::Config::from_file(&config_path)?
    } else if args.dev {
        config::Config::development()
    } else {
        config::Config::default()
    };

    // Initialize blockchain
    let mut blockchain = Blockchain::new(args.base_path, config)?;

    info!("Blockchain initialized at height {}", blockchain.height());

    // Start mining if enabled
    if args.mining || args.dev {
        let coinbase = if let Some(addr) = args.coinbase {
            parse_address(&addr)?
        } else if args.dev {
            // Use development coinbase
            [1u8; 32]
        } else {
            anyhow::bail!("Mining enabled but no coinbase address provided");
        };

        info!("Mining enabled with coinbase: {}", hex::encode(coinbase));

        // Start mining loop
        tokio::spawn(async move {
            loop {
                match blockchain.mine_block(coinbase).await {
                    Ok(block) => {
                        info!(
                            "Mined block #{} with hash {}",
                            block.header.height,
                            hex::encode(block.header.hash())
                        );
                    }
                    Err(e) => {
                        error!("Mining error: {}", e);
                    }
                }
            }
        });
    }

    info!("Node running. Press Ctrl+C to stop.");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;

    info!("Shutting down...");

    Ok(())
}

fn parse_address(addr: &str) -> anyhow::Result<[u8; 32]> {
    let bytes = hex::decode(addr)?;
    if bytes.len() != 32 {
        anyhow::bail!("Invalid address length");
    }
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    Ok(result)
}
```

---

## Success Metrics

### Week 1-3 (Critical Path)
- ✅ All transactions have verified signatures
- ✅ Blockchain can track UTXO set and balances
- ✅ Node binary compiles and runs
- ✅ Can mine blocks continuously
- ✅ State persists across restarts

### Week 4-6 (Phase 2 Foundation)
- ✅ Multiple nodes can discover each other
- ✅ Blocks propagate across network
- ✅ Frontend can query blockchain via RPC
- ✅ Smart contracts can be deployed and called
- ✅ Integration tests pass

---

## Risk Mitigation

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| liboqs compatibility issues | Medium | High | Test on multiple platforms early |
| RocksDB integration complexity | Low | Medium | Use well-tested patterns from Substrate |
| P2P network bugs | High | High | Extensive integration testing |
| Performance bottlenecks | Medium | Medium | Benchmark early and often |
| State corruption | Low | Critical | Implement state snapshots |

### Timeline Risks

| Risk | Mitigation |
|------|------------|
| Underestimated complexity | Focus on MVP features, defer nice-to-haves |
| Dependency on external libraries | Have backup options for critical dependencies |
| Testing takes longer than expected | Automate testing from day 1 |

---

## Post-Completion Checklist

Before declaring Phase 1 complete:

**Functionality**:
- [ ] Node starts and mines blocks
- [ ] Transactions are validated with signatures
- [ ] State persists across restarts
- [ ] RPC returns accurate data
- [ ] Frontend can interact with blockchain
- [ ] Smart contracts can be deployed

**Quality**:
- [ ] >80% test coverage
- [ ] No critical security issues
- [ ] Documentation up to date
- [ ] All TODOs addressed or documented
- [ ] Performance benchmarks recorded

**Deployment**:
- [ ] Can run multi-node testnet
- [ ] Deployment guide tested
- [ ] Docker images built
- [ ] CI/CD pipeline functional

---

## Resources & Dependencies

### Team Requirements
- 1-2 Rust developers (full-time)
- 1 DevOps engineer (part-time)
- 1 Security reviewer (part-time)

### External Dependencies
- liboqs (PQC library)
- RocksDB (storage)
- libp2p (networking)
- Wasmtime (smart contracts)

### Infrastructure
- Development testnet (3-5 nodes)
- CI/CD (GitHub Actions)
- Monitoring (Prometheus/Grafana)

---

## Next Steps

1. **Immediate** (Today):
   - Start implementing transaction signature verification
   - Create blockchain state management skeleton

2. **This Week**:
   - Complete signature verification with tests
   - Implement UTXO tracking
   - Begin node binary scaffolding

3. **Next Week**:
   - Complete node binary
   - Implement mempool
   - Begin storage integration

4. **Week 3**:
   - Complete RPC API
   - Integration testing
   - Documentation updates

---

**Document Version**: 1.0
**Status**: Active Development
**Next Review**: End of Week 1
