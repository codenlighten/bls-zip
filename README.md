# Boundless BLS Platform

> A post-quantum secure blockchain platform with privacy-preserving smart contracts

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)
![PQC](https://img.shields.io/badge/PQC-NIST%20Standards-green.svg)

## Overview

Boundless BLS is a next-generation blockchain platform designed for the post-quantum era. It combines NIST-standardized post-quantum cryptographic algorithms with privacy-preserving smart contracts to deliver quantum-resistant security and confidential computation.

### Key Features

#### Core Blockchain (Phase 1)
- **Post-Quantum Cryptography**: ML-KEM-768, ML-DSA-44, Falcon-512 (NIST standards)
- **Hybrid Schemes**: Gradual transition with classical+PQC algorithms
- **SHA-3 Proof-of-Work**: ASIC-resistant consensus using SHA-3/SHAKE256
- **WASM Smart Contracts**: Deterministic execution with fuel metering
- **Privacy-Preserving Computation**: Paillier homomorphic encryption
- **UTXO State Management**: Bitcoin-style UTXO tracking with nonce-based replay protection

#### Production Infrastructure (Phase 2)
- **JSON-RPC API**: 8 core methods for blockchain queries and transaction submission
- **Persistent Storage**: RocksDB with 4 column families and LZ4 compression
- **P2P Networking**: libp2p-based networking with gossipsub and mDNS peer discovery
- **Transaction Mempool**: Fee-based transaction ordering and management
- **Full Integration**: All components integrated into single node binary

#### Network Synchronization (Phase 3)
- **Block Broadcasting**: Automatic propagation of mined blocks to all peers
- **Transaction Broadcasting**: Network-wide transaction propagation via gossipsub
- **Automatic Block Sync**: Nodes automatically request and sync missing blocks
- **Peer Status Tracking**: Nodes discover chain height differences and auto-sync
- **Multi-Node Operation**: True distributed blockchain across multiple nodes

#### Enterprise Multipass (E2 Integration) ✅ NEW
- **Identity Management**: KYC/AML verified identity profiles with 7 attestation types
- **Blockchain Anchoring**: Attestations and asset transfers anchored on-chain for immutability
- **Secure Authentication**: JWT-based auth with Argon2 password hashing and middleware protection
- **Asset Management**: Multi-currency wallets with internal marketplace
- **Proof Verification**: RPC endpoints to verify and retrieve blockchain-anchored proofs
- **Hardware Security**: Hardware wallet and biometric authentication support
- **Application Registry**: OAuth2-style app registration with scope-based permissions
- **Event System**: Real-time notifications and compliance reporting

#### Future Features
- **Multi-Chain Anchoring**: BSV, Ethereum, Cardano, Solana, Hedera
- **ATMOS Integration**: Environmental entropy for enhanced randomness

## Quick Start

### Prerequisites

- Rust 1.75+ (`rustup`)
- liboqs (Open Quantum Safe)
- Node.js 20+ (for frontend)
- cargo-contract (for smart contracts)

See [DEPLOYMENT.md](DEPLOYMENT.md) for detailed installation instructions.

### Build and Run

```bash
# Clone repository
git clone https://github.com/your-org/boundless-bls-platform.git
cd boundless-bls-platform

# Build blockchain
cargo build --release

# Run development node with mining and RPC
./target/release/boundless-node --dev --mining

# Node will start:
# - RPC API on http://127.0.0.1:9933
# - P2P network on port 30333
# - Mining with auto-adjusting difficulty

# In another terminal: Query blockchain via RPC
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}'

# In another terminal: Run a second node (auto-discovers and syncs)
./target/release/boundless-node --dev --port 30334 --rpc-port 9934

# Nodes will:
# - Discover each other via mDNS
# - Automatically sync blocks
# - Broadcast new blocks and transactions
# - Maintain consensus

# In another terminal: Run frontend (coming soon)
cd frontend
npm install
npm run dev
# Open http://localhost:3000
```

### Docker Deployment (Easiest!)

For the simplest deployment, use Docker:

```bash
# Quick Start - Single Mining Node
docker build -t boundless-bls:latest .
docker run -d --name boundless-node -p 30333:30333 -p 9933:9933 boundless-bls:latest --dev --mining --rpc-external

# Or use Windows batch script
docker-run.bat build
docker-run.bat dev

# Or start 3-node network (nodes auto-discover via mDNS!)
docker-compose up -d
docker-compose logs -f node1
```

**See [README-DOCKER.md](README-DOCKER.md) for complete Docker guide**

### Run Tests

```bash
# Unit tests
cargo test --all

# Automated multi-node tests
chmod +x scripts/test_multi_node.sh
./scripts/test_multi_node.sh

# Network synchronization verification
chmod +x scripts/verify_network_sync.sh
./scripts/verify_network_sync.sh

# Performance benchmarking
chmod +x scripts/benchmark_performance.sh
./scripts/benchmark_performance.sh

# Smart contract tests
cd contracts/token
cargo test

# Frontend tests (when available)
cd frontend
npm test
```

See [TESTING.md](TESTING.md) and [MULTI_NODE_TESTING.md](MULTI_NODE_TESTING.md) for comprehensive testing guides.

## Architecture

```
boundless-bls-platform/
├── node/               # ✅ Full blockchain node binary
│   ├── main.rs        # Node startup and CLI
│   ├── blockchain.rs  # Blockchain state management
│   ├── mempool.rs     # Transaction mempool
│   └── config.rs      # Node configuration
│
├── core/              # ✅ Blockchain data structures
│   ├── block.rs       # Block and header types
│   ├── transaction.rs # Transaction types with PQC signatures
│   ├── merkle.rs      # Merkle tree for TX verification
│   ├── state.rs       # UTXO state management
│   └── account.rs     # Account state management
│
├── consensus/         # ✅ SHA-3 Proof-of-Work consensus
│   ├── pow.rs        # PoW validation
│   ├── difficulty.rs # Bitcoin-style epoch adjustment
│   └── miner.rs      # Multi-threaded mining
│
├── crypto/           # ✅ Post-quantum cryptography
│   ├── pqc.rs       # ML-KEM, ML-DSA, Falcon wrappers
│   ├── hybrid.rs    # Hybrid classical+PQC schemes
│   └── phe.rs       # Paillier homomorphic encryption
│
├── wasm-runtime/     # ✅ Smart contract execution
│   ├── runtime.rs   # Wasmtime with fuel metering + resource limiter
│   ├── host_functions.rs  # Blockchain host functions
│   └── config.rs    # Gas limits and execution config
│
├── rpc/              # ✅ JSON-RPC API + Proof Verification
│   ├── server.rs    # HTTP/WebSocket RPC server + proof endpoints
│   ├── http_bridge.rs # REST API with proof anchoring/verification
│   ├── types.rs     # RPC request/response types
│   └── error.rs     # RPC error handling
│
├── storage/          # ✅ Persistent storage (Phase 2)
│   ├── db.rs       # RocksDB wrapper with column families
│   └── error.rs    # Storage error types
│
├── p2p/              # ✅ P2P networking (Phase 2)
│   ├── network.rs  # libp2p network node
│   ├── protocol.rs # Boundless protocol messages
│   └── peer.rs     # Peer management
│
├── enterprise/       # ✅ Enterprise Multipass (E2 Integration)
│   ├── api/         # REST API with JWT auth middleware
│   ├── services/    # Business logic (identity, wallet, assets, auth)
│   ├── migrations/  # PostgreSQL database migrations
│   └── frontend/    # Next.js admin UI
│
├── contracts/        # ✅ Sample smart contracts (ink!)
│   ├── token/       # Fungible token (ERC-20 style)
│   ├── voting/      # Privacy-preserving voting
│   └── escrow/      # Multi-party escrow
│
└── frontend/         # 🚧 Next.js dApp interface (WIP)
    ├── components/  # React components
    └── pages/       # Application pages
```

## Cryptographic Specifications

### Post-Quantum Algorithms (NIST Standards)

| Algorithm | Type | Use Case | Security Level |
|-----------|------|----------|----------------|
| **ML-KEM-768** (FIPS 203) | Key Encapsulation | Hybrid key exchange | Level 3 (~AES-192) |
| **ML-DSA-44** (FIPS 204) | Digital Signature | Transaction signing | Level 2 (~AES-128) |
| **Falcon-512** | Digital Signature | Compact signatures | Level 1 (~AES-128) |

### Hybrid Schemes

- **Key Exchange**: X25519 + ML-KEM-768
- **Signatures**: Ed25519 + ML-DSA-44
- **Hashing**: SHA-3-256 for PoW, SHAKE256 for randomness

### Homomorphic Encryption

- **Paillier**: Partially homomorphic encryption for private voting and aggregation

## Smart Contracts

### Example: Token Transfer

```rust
#[ink::contract]
mod boundless_token {
    #[ink(message)]
    pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
        let from = self.env().caller();
        self.transfer_from_to(&from, &to, value)
    }
}
```

### Deploying Contracts

```bash
# Build contract
cd contracts/token
cargo contract build --release

# Deploy via frontend or CLI
boundless-cli deploy \
  --wasm target/ink/boundless_token.wasm \
  --constructor new \
  --args "Boundless Token,BLS,18,1000000000"
```

## Performance

### Benchmarks (AMD Ryzen 9 5900X)

| Operation | Throughput | Latency |
|-----------|-----------|---------|
| ML-KEM-768 Encapsulation | 4,970/sec | 201 µs |
| ML-DSA-44 Sign | 1,927/sec | 519 µs |
| ML-DSA-44 Verify | 3,412/sec | 293 µs |
| SHA-3-256 (PoW) | 2.4 MH/s | 416 ns |
| WASM Contract Call | 50,000/sec | 20 µs |

Block Time: **5 minutes** (configurable)
Difficulty Adjustment: **Every 1,008 blocks** (~3.5 days)

## Roadmap

### Phase 1: Foundation ✅ 95% Complete
- [x] Core blockchain implementation with UTXO state
- [x] SHA-3 PoW consensus with difficulty adjustment
- [x] PQC cryptography integration (ML-KEM, ML-DSA, Falcon)
- [x] Transaction signature verification
- [x] WASM runtime with fuel metering
- [x] Sample smart contracts with access control
- [x] Full node binary with mining

### Phase 2: Production Infrastructure ✅ 100% Complete
- [x] JSON-RPC API (8 core methods - HTTP/WebSocket)
- [x] RocksDB persistent storage (4 column families)
- [x] P2P networking foundation (libp2p, gossipsub, mDNS)
- [x] Transaction mempool with fee ordering
- [x] All components integrated into node binary

### Phase 3: Network Synchronization ✅ 90% Complete
- [x] Gossipsub topics for blocks and transactions
- [x] Block broadcasting (mined blocks propagate to peers)
- [x] Transaction broadcasting (mempool synchronization)
- [x] Automatic block sync protocol (nodes request missing blocks)
- [x] Peer status tracking and auto-sync
- [x] Network message handling (6 message types)
- [ ] Chain reorganization handling
- [ ] Request-response protocol (for efficient peer queries)
- [ ] Multi-node testnet deployment

### Phase 3: Advanced Features (Months 13-18)
- [ ] Multi-chain anchoring (BSV, ETH, ADA, SOL, HBAR)
- [ ] ATMOS environmental entropy integration
- [ ] HSM integration for validator keys
- [ ] Cross-chain bridges

### Phase 4: Production (Months 19-24)
- [ ] Mainnet launch
- [ ] Governance system
- [ ] Staking mechanisms
- [ ] Mobile wallet applications

## Security

### Post-Quantum Security

Boundless BLS is **architected as post-quantum aware by default**. See [Post-Quantum Assurance](POST_QUANTUM_ASSURANCE.md) for comprehensive details on:

- **Threat Model**: Protection against harvest-now, decrypt-later attacks
- **NIST Standards**: ML-KEM-768 (FIPS 203), ML-DSA-44 (FIPS 204), Falcon-512
- **Hybrid Constructions**: Ed25519+ML-DSA, X25519+ML-KEM for transition security
- **Algorithm Agility**: CryptoProfile system for seamless algorithm upgrades
- **Enterprise Assurance**: PQC-only profiles for regulated environments
- **Compliance**: FIPS alignment, audit trail durability, regulatory guidance

**Key Security Features:**
- ✅ NIST-standardized PQC algorithms
- ✅ SHA-3 hashing with domain separation
- ✅ Hybrid schemes for backward compatibility
- ✅ Long-lived identity protection
- ✅ Verifiable attestations with blockchain anchoring
- ✅ Formal verification roadmap

### Audit Status

- [ ] Internal security review
- [x] PQC algorithm integration (NIST standards)
- [ ] Third-party cryptographic audit (Q1 2026)
- [ ] Smart contract formal verification
- [ ] Penetration testing

### Responsible Disclosure

Please report security vulnerabilities to: [security@boundless-bls.com](mailto:security@boundless-bls.com)

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Standards

- **Rust**: Follow Rust API guidelines, use `cargo fmt` and `cargo clippy`
- **TypeScript**: Use ESLint and Prettier
- **Tests**: Maintain >80% code coverage
- **Documentation**: Update docs for all public APIs

## Documentation

### Getting Started
- [Quick Start Guide](QUICKSTART.md) - Get started in 5 minutes
- [README](README.md) - This file

### Implementation Guides
- [Phase 2 Completion Report](PHASE2_COMPLETE.md) - RPC, Storage, P2P implementation details
- [Phase 2 Summary](PHASE2_SUMMARY.md) - Comprehensive Phase 2 achievements
- [Phase 3 Network Sync](PHASE3_NETWORK_SYNC.md) - Block sync & broadcasting implementation
- [Implementation Status](IMPLEMENTATION_STATUS.md) - Complete feature status and API documentation

### Security & Cryptography
- [Post-Quantum Assurance](POST_QUANTUM_ASSURANCE.md) - **Comprehensive PQC security model and guarantees** 🔐
- [Technical Architecture](TECHNICAL-ARCHITECTURE.md) - Detailed architecture and cryptography layer

### Testing & Performance
- [Testing Guide](TESTING.md) - Comprehensive testing procedures
- [Multi-Node Testing](MULTI_NODE_TESTING.md) - Multi-node test scenarios and verification
- [Performance Optimization](PERFORMANCE_OPTIMIZATION.md) - Optimization analysis and recommendations

### Project Status
- [Project Status](PROJECT_STATUS.md) - Complete project status and metrics
- [Final Summary](FINAL_SUMMARY.md) - Comprehensive project summary
- [Development Plan](BOUNDLESS_COMPREHENSIVE_DEVELOPMENT_PLAN.md) - Long-term roadmap

### Deployment
- [Deployment Guide](DEPLOYMENT.md) - Production deployment
- [Docker Guide](README-DOCKER.md) - Docker deployment
- [Windows Setup](WINDOWS_SETUP_GUIDE.md) - Windows development setup
- [Technical Specification](Boundless_BLS_Platform_Technical_Specification_and_Implementation.pdf)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **NIST**: Post-Quantum Cryptography Standardization
- **Open Quantum Safe**: liboqs library
- **Parity Technologies**: ink! smart contract framework
- **Bytecode Alliance**: Wasmtime runtime

## Contact

- **Website**: https://boundless-bls.com
- **Documentation**: https://docs.boundless-bls.com
- **Discord**: https://discord.gg/boundless-bls
- **Twitter**: [@BoundlessBLS](https://twitter.com/BoundlessBLS)
- **Email**: info@boundless-bls.com

---

**Built for the post-quantum era** 🔐🚀
