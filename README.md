# Boundless BLS Platform

> **Manage. Monetize. Innovate. Boundlessly.**
>
> A post-quantum secure blockchain platform with Enterprise E² Multipass operating system

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)
![PQC](https://img.shields.io/badge/PQC-NIST%20Standards-green.svg)
![Enterprise](https://img.shields.io/badge/Enterprise-E²%20Multipass-purple.svg)
![Boundless](https://img.shields.io/badge/Boundless-Trust-blue.svg)

## Overview

**Boundless BLS** is a next-generation blockchain platform designed for the post-quantum era, developed by **Boundless Trust** in partnership with **SmartLedger Solutions**. We are committed to pioneering the future of decentralized technology, making it accessible, compliant, and secure for everyone.

The platform combines NIST-standardized post-quantum cryptographic algorithms with privacy-preserving smart contracts to deliver quantum-resistant security and confidential computation, with a focus on **regulatory compliance**, **data sovereignty**, and **enterprise security**.

### Platform Components

1. **Boundless BLS Blockchain**: Core blockchain with post-quantum cryptography, SHA-3 PoW consensus, and WASM smart contracts
2. **Enterprise E² Multipass**: Enterprise operating system providing identity management, multi-asset wallets, and business application integration
3. **BLS Blockchain Explorer**: Modern, full-featured blockchain explorer with real-time data visualization, E² Multipass integration, and post-quantum cryptography support

### Core Values

- **Transparent Ecosystem**: Open and auditable blockchain infrastructure
- **Data Sovereignty**: Regional data compliance and user control
- **Trust & Integrity**: Built on cryptographic guarantees and verifiable proofs
- **Regulatory Compliance**: Designed for global standards and enterprise requirements
- **Innovation**: Bridging complex technology with practical applications

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

#### Enterprise E² Multipass (Complete Integration) ✅ PRODUCTION READY
**Not a Cryptocurrency Platform** - Enterprise blockchain for regulated business assets

- **CIVA 3-Layer Identity Model**: Identity Proof, Risk & Compliance, Attributes
- **KYC/AML Verification**: Multi-level verification with blockchain proof anchoring
- **Application-Aware Wallets**: Contextual wallets tied to business applications (not just addresses)
- **Multi-Asset Support**: 8+ asset types (Equity, Utility, Governance, Carbon Credits, Rewards, Stablecoins)
- **Post-Quantum Security**: ML-DSA (Dilithium5) and ML-KEM (Kyber1024) cryptography
- **Encrypted Keystore**: AES-256-GCM encrypted private key storage with master key management
- **Blockchain Anchoring**: Attestations and asset transfers anchored on-chain for immutability
- **Proof Verification**: RPC and HTTP REST endpoints to verify blockchain-anchored proofs
- **Secure Authentication**: JWT-based auth with Argon2id password hashing and middleware
- **Hardware Pass Integration**: NFC cards, secure elements, device attestation
- **Application Registry**: Pluggable business modules with permission management
- **Event System**: Real-time notifications, custom reports, analytics
- **Smart Contract Templates**: Pre-built templates for common business logic
- **PostgreSQL Backend**: 20+ tables with comprehensive schema (identities, wallets, assets, events)
- **Next.js Frontend**: Modern admin UI with TypeScript and TailwindCSS

#### BLS Blockchain Explorer (100% Complete) ✅ PRODUCTION READY
**Modern blockchain explorer with real-time data and E² Multipass integration**

- **Real-Time Blockchain Visualization**: Live block and transaction monitoring with auto-refresh
- **Block Explorer**: Detailed block information with previous/next navigation
- **Transaction Tracker**: Multi-signature support (Classical, ML-DSA, Falcon-512, Hybrid)
- **UTXO Visualization**: Input/output tracking with fee calculation
- **Network Statistics Dashboard**: Interactive charts and metrics with trend indicators
- **E² Multipass Authentication**: JWT-based login with identity management
- **Identity Explorer**: KYC level tracking and proof anchor verification
- **Post-Quantum Cryptography Support**: Full support for ML-DSA, Falcon-512, and hybrid signatures
- **Advanced Features**: Wallet integration, asset transfers, sustainability metrics
- **Next.js 14 + TypeScript**: Modern UI with shadcn/ui components and Tailwind CSS
- **Graceful Fallback**: Mock data mode for development without blockchain node
- **Real-Time Updates**: 30-second auto-refresh with connection health indicators
- See [BLS_Explorer/README.md](BLS_Explorer/README.md) for complete documentation

#### Compliance & Governance Modules (In Development by Boundless Trust)
- **GeoSovereign** (70% complete): Regional data compliance automation for multi-jurisdictional operations
- **RegBlock** (65% complete): Real-time regulatory mandate integration and compliance monitoring
- **ErasureGuard** (80% complete): GDPR "Right to Erasure" implementation with blockchain audit trails

#### Future Features
- **Multi-Chain Anchoring**: Multi-blockchain support (BTC, ETH, SOL, BSV, ADA, HBAR)
- **IoT Integration**: Internet of Things device connectivity and data verification
- **Cross-Chain Verification**: Interoperability between multiple blockchain networks
- **Energy-Efficient Mining**: ASIC-resistant mining with optimized energy consumption
- **ATMOS Integration**: Environmental entropy for enhanced randomness

## Quick Start

### Prerequisites

- **Rust 1.75+** (`rustup`) - Core blockchain development
- **liboqs** (Open Quantum Safe) - Post-quantum cryptography library
- **Node.js 18+** (for frontend) - Enterprise E² Multipass UI
- **PostgreSQL 14+** - Enterprise database (for E² Multipass)
- **Docker + Docker Compose** - Containerized deployment (optional)

See [DEPLOYMENT.md](DEPLOYMENT.md) and [WINDOWS_SETUP_GUIDE.md](WINDOWS_SETUP_GUIDE.md) for detailed installation instructions.

### Build and Run

```bash
# Clone repository
git clone https://github.com/Saifullah62/BLS.git
cd BLS

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

# In another terminal: Run frontend (basic demo - WIP)
cd frontend
npm install
npm run dev
# Open http://localhost:3000
```

### Enterprise E² Multipass Setup

For the complete enterprise platform with identity management and multi-asset wallets:

```bash
# 1. Setup PostgreSQL database
createdb enterprise_db

# 2. Configure environment
cd enterprise
cp .env.example .env
# Edit .env with your settings (DATABASE_URL, JWT_SECRET, etc.)

# 3. Run database migrations
sqlx migrate run

# 4. Start Enterprise backend
cargo run --bin enterprise-server
# Server runs on http://localhost:8080

# 5. In another terminal: Start Enterprise frontend
cd enterprise/frontend
npm install
npm run dev
# Open http://localhost:3001

# 6. Login with default admin
# Email: yourfriends@smartledger.solutions
# Password: BoundlessTrust
```

**See [enterprise/README.md](enterprise/README.md) for complete Enterprise E² Multipass documentation**

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

See the [Documentation Index](DOCUMENTATION_INDEX.md) for comprehensive testing guides and all documentation.

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
├── enterprise/       # ✅ Enterprise E² Multipass (Complete Integration)
│   ├── src/
│   │   ├── bin/server.rs    # Main server entry point
│   │   ├── models.rs        # Database models (20+ tables)
│   │   ├── api/             # REST API with JWT auth middleware
│   │   │   ├── identity.rs  # Identity & attestation endpoints
│   │   │   ├── wallet.rs    # Wallet management endpoints
│   │   │   ├── auth.rs      # Authentication & session endpoints
│   │   │   ├── asset.rs     # Asset & trading endpoints
│   │   │   ├── application.rs # Application registry endpoints
│   │   │   ├── events.rs    # Events & reporting endpoints
│   │   │   └── hardware.rs  # Hardware pass endpoints
│   │   ├── services/        # Business logic layer
│   │   │   ├── identity.rs  # KYC/AML, attestation anchoring
│   │   │   ├── wallet.rs    # Multi-asset wallet, key management
│   │   │   ├── auth.rs      # JWT auth, session management
│   │   │   ├── asset.rs     # Asset definitions, trading, anchoring
│   │   │   ├── application.rs # App registration, permissions
│   │   │   ├── events.rs    # Notifications, reports, analytics
│   │   │   └── hardware.rs  # NFC cards, device challenges
│   │   ├── blockchain/mod.rs # Boundless HTTP REST client
│   │   ├── crypto/mod.rs     # PQC (Dilithium5, Kyber1024)
│   │   ├── keystore/mod.rs   # AES-256-GCM encrypted key storage
│   │   ├── transaction/      # UTXO transaction builder & PQC signer
│   │   ├── middleware.rs     # JWT verification, rate limiting
│   │   └── validation.rs     # Input validation
│   ├── migrations/  # PostgreSQL database migrations (20+ tables)
│   └── frontend/    # Next.js 14 admin UI (TypeScript + TailwindCSS)
│       ├── src/
│       │   ├── app/         # Next.js app router pages
│       │   │   ├── dashboard/    # Dashboard page
│       │   │   ├── identity/     # Identity management
│       │   │   ├── wallet/       # Wallet UI
│       │   │   ├── contracts/    # Smart contracts
│       │   │   ├── trading/      # Asset trading
│       │   │   ├── admin/        # Admin panel
│       │   │   └── ...          # 12 total pages
│       │   ├── lib/api.ts   # API client
│       │   └── types/       # TypeScript definitions
│       └── package.json
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

📚 **[Documentation Index](DOCUMENTATION_INDEX.md)** - Complete documentation catalog organized by category

### Getting Started
- [Quick Start Guide](QUICKSTART.md) - Get started in 5 minutes
- [README](README.md) - This file
- [Project Status](STATUS.md) - Current status and progress

### Current Documentation
- [Technical Architecture](TECHNICAL-ARCHITECTURE.md) - Detailed architecture and cryptography layer
- [Post-Quantum Assurance](POST_QUANTUM_ASSURANCE.md) - **Comprehensive PQC security model and guarantees** 🔐
- [Smart Contract Progress](CONTRACT_IMPLEMENTATION_PROGRESS.md) - 98% complete smart contract implementation
- [UTXO Contract Architecture](UTXO_CONTRACT_ARCHITECTURE.md) - Contract design and architecture
- [Deployment Guide](DEPLOYMENT.md) - Production deployment
- [Docker Guide](DOCKER.md) - Docker deployment
- [Windows Setup](WINDOWS_SETUP_GUIDE.md) - Windows development setup

### Archived Documentation
Historical implementation reports and guides have been archived. See:
- [Documentation Index](DOCUMENTATION_INDEX.md) - Complete catalog including archived docs
- [Archived Reports](docs/archive/) - Historical progress reports and implementation guides

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

### Organizations
- **Boundless Trust**: Leading blockchain solutions provider
- **SmartLedger Solutions**: Enterprise blockchain integration partner

### Technology Partners
- **NIST**: Post-Quantum Cryptography Standardization (FIPS 203, FIPS 204)
- **Open Quantum Safe**: liboqs post-quantum cryptography library
- **Parity Technologies**: ink! smart contract framework
- **Bytecode Alliance**: Wasmtime WebAssembly runtime
- **libp2p**: Modern peer-to-peer networking stack
- **RocksDB**: High-performance embedded database

## Repository

- **GitHub**: https://github.com/Saifullah62/BLS
- **Issues**: https://github.com/Saifullah62/BLS/issues
- **Releases**: https://github.com/Saifullah62/BLS/releases

## Contact & Community

### Boundless Trust
- **Website**: https://boundlesstrust.org
- **Email**: contact@boundlesstrust.org
- **LinkedIn**: https://linkedin.com/company/boundless-trust
- **X/Twitter**: [@Boundless_Trust](https://x.com/Boundless_Trust)

### SmartLedger Solutions (Partner)
- **Enterprise Solutions**: yourfriends@smartledger.solutions

### Security
- **Security Contact**: security@boundlesstrust.org

---

## About Boundless Trust

**Boundless Trust** (also known as Boundless Blockchain or BLS) is a blockchain solutions provider focused on decentralized technology that emphasizes regulatory compliance, data sovereignty, and security for enterprises and governments. Our vision is a world where blockchain technology drives innovation across industries, bridging complex technology with practical applications.

In partnership with **SmartLedger Solutions**, we deliver enterprise-grade blockchain infrastructure with post-quantum security, making decentralized technology accessible, compliant, and secure for everyone.

---

**Built for the post-quantum era** 🔐🚀

*Manage. Monetize. Innovate. Boundlessly.*
