# Boundless BLS - Quick Start Guide

## Prerequisites

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # Should be 1.75.0 or later
```

### 2. Install liboqs (Required for PQC)

**macOS:**
```bash
brew install liboqs
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y cmake ninja-build libssl-dev

git clone --depth 1 https://github.com/open-quantum-safe/liboqs.git
cd liboqs
mkdir build && cd build
cmake -GNinja -DCMAKE_INSTALL_PREFIX=/usr/local ..
ninja
sudo ninja install

# Set environment variables
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
```

Add to `~/.bashrc` for persistence:
```bash
echo 'export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH' >> ~/.bashrc
```

## Build the Node

```bash
# Clone repository
cd boundless-bls-platform

# Build all crates
cargo build --release

# This will compile:
# - core (blockchain data structures)
# - consensus (SHA-3 PoW)
# - crypto (PQC algorithms)
# - wasm-runtime (smart contract execution)
# - node (full node binary)
```

Build artifacts will be in `target/release/`.

## Run a Development Node

```bash
# Run node in development mode with mining enabled
./target/release/boundless-node --dev --mining

# Or with custom parameters
./target/release/boundless-node \
  --dev \
  --mining \
  --base-path ./my-node-data \
  --mining-threads 4
```

You should see output like:
```
🚀 Starting Boundless BLS Node v0.1.0
📁 Data directory: "./my-node-data"
🔧 Development mode enabled
⛓️  Blockchain initialized at height 1
🔗 Best block: 0000000000000000...
💾 Mempool initialized
⛏️  Mining enabled
💰 Coinbase: 0101010101010101...
🧵 Mining threads: 4
✅ Node is running
Press Ctrl+C to stop

📦 Building block with 0 transaction(s)
✨ Mined block #2 - Hash: 0000a3f59d2c... - 125487 hashes, 2456.32 H/s
```

## Run Tests

```bash
# Run all tests
cargo test --all

# Run with output
cargo test --all -- --nocapture

# Run specific crate tests
cargo test -p boundless-core
cargo test -p boundless-consensus
cargo test -p boundless-crypto
cargo test -p boundless-node

# Run ignored (slow) tests
cargo test --all -- --ignored
```

## Project Structure

```
boundless-bls-platform/
├── node/              # Full node binary ✅
│   ├── src/
│   │   ├── main.rs           # Entry point
│   │   ├── blockchain.rs      # Blockchain management
│   │   ├── mempool.rs         # Transaction pool
│   │   └── config.rs          # Configuration
│   └── Cargo.toml
│
├── core/              # Core data structures ✅
│   ├── src/
│   │   ├── block.rs           # Block & BlockHeader
│   │   ├── transaction.rs      # Transaction with PQC signatures
│   │   ├── state.rs           # UTXO state management
│   │   ├── merkle.rs          # Merkle tree
│   │   └── account.rs         # Account management
│   └── Cargo.toml
│
├── consensus/         # SHA-3 Proof-of-Work ✅
│   ├── src/
│   │   ├── pow.rs             # PoW validation
│   │   ├── difficulty.rs       # Difficulty adjustment
│   │   └── miner.rs           # Mining implementation
│   └── Cargo.toml
│
├── crypto/            # Post-Quantum Cryptography ✅
│   ├── src/
│   │   ├── pqc.rs             # ML-KEM, ML-DSA, Falcon
│   │   ├── hybrid.rs          # Hybrid schemes
│   │   └── phe.rs             # Paillier PHE
│   └── Cargo.toml
│
├── wasm-runtime/      # Smart contract execution ✅
│   ├── src/
│   │   ├── runtime.rs         # Wasmtime integration
│   │   ├── host_functions.rs  # Blockchain APIs
│   │   └── config.rs          # Gas metering config
│   └── Cargo.toml
│
└── contracts/         # Sample smart contracts ✅
    ├── token/                 # ERC-20 style token
    ├── voting/                # Private voting
    └── escrow/                # Multi-party escrow
```

## CLI Options

```bash
boundless-node --help

Options:
  --dev                      Run in development mode
  --mining                   Enable mining
  --coinbase <ADDRESS>       Mining coinbase address (hex)
  --base-path <PATH>         Data directory [default: ./data]
  --port <PORT>              P2P port [default: 30333]
  --rpc-port <PORT>          RPC HTTP port [default: 9933]
  --ws-port <PORT>           RPC WebSocket port [default: 9944]
  --config <FILE>            Config file path
  --mining-threads <NUM>     Mining threads [default: 1]
  -h, --help                 Print help
  -V, --version              Print version
```

## Configuration File

Create `config.toml`:

```toml
[network]
listen_addr = "/ip4/127.0.0.1/tcp/30333"
bootnodes = []

[consensus]
target_block_time_secs = 300      # 5 minutes
difficulty_adjustment_interval = 1008  # ~3.5 days
max_adjustment_factor = 4

[storage]
database_path = "./data/db"
cache_size_mb = 2048

[rpc]
http_addr = "127.0.0.1:9933"
ws_addr = "127.0.0.1:9944"
cors_allowed_origins = ["*"]

[mempool]
max_transactions = 10000
max_tx_size = 100000
min_fee_per_byte = 1
```

Run with config:
```bash
./target/release/boundless-node --config config.toml --mining
```

## Build Smart Contracts

```bash
# Install cargo-contract
cargo install cargo-contract --force

# Build token contract
cd contracts/token
cargo contract build --release

# Output:
# target/ink/boundless_token.wasm
# target/ink/boundless_token.json
```

## Enterprise Multipass (E2) Setup

The Enterprise Multipass provides identity management, wallet services, and asset management on top of the Boundless blockchain.

### Prerequisites

- PostgreSQL 14+
- Node.js 20+
- OpenSSL

### 1. Database Setup

```bash
# Create database
createdb enterprise_db

# Run migrations
cd enterprise
sqlx migrate run
```

### 2. Configure Environment

Create `enterprise/.env`:

```bash
DATABASE_URL=postgresql://postgres:password@localhost:5432/enterprise_db
JWT_SECRET=$(openssl rand -hex 32)
MASTER_ENCRYPTION_KEY=$(openssl rand -hex 32)
BLOCKCHAIN_RPC_URL=http://localhost:9933
```

### 3. Start Enterprise Backend

```bash
cd enterprise
cargo run --bin enterprise-server

# Or build for production
cargo build --release
./target/release/enterprise-server
```

The backend will start on `http://localhost:8080` with:
- JWT-based authentication
- Identity and attestation services
- Multi-asset wallet management
- Blockchain proof anchoring

### 4. Start Enterprise Frontend

```bash
cd enterprise/frontend
npm install
npm run dev
```

Access the admin dashboard at `http://localhost:3001`

**Default Login:**
- Email: `yourfriends@smartledger.solutions`
- Password: `BoundlessTrust`

### 5. Verify Integration

```bash
# Check backend health
curl http://localhost:8080/health

# Verify blockchain connection
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}'
```

## What's Working (Phase 1 - 90% Complete)

✅ **Core Blockchain**
- Block and transaction data structures
- Merkle tree verification
- UTXO state management
- Account nonce tracking

✅ **Consensus**
- SHA-3-256 Proof-of-Work
- Bitcoin-style difficulty adjustment
- Mining with nonce iteration

✅ **Cryptography**
- ML-KEM-768 (FIPS 203)
- ML-DSA-44 (FIPS 204)
- Falcon-512
- Hybrid schemes (X25519+ML-KEM, Ed25519+ML-DSA)
- Paillier PHE
- Transaction signature verification

✅ **Node Binary**
- Full blockchain node
- Block mining loop
- Transaction mempool
- Configuration system

✅ **Smart Contracts**
- Wasmtime runtime with fuel metering and resource limiter
- Token contract (with access control)
- Voting contract
- Escrow contract

✅ **Enterprise Multipass (E2)** 🆕
- JWT-based authentication with Argon2 password hashing
- Identity management with KYC/AML verification
- Multi-asset wallet with encrypted keystore (AES-256-GCM)
- Blockchain proof anchoring for attestations and asset transfers
- RPC proof verification endpoints
- Application registry and permission management
- Hardware pass integration

## What's Complete (Phase 2 & 3)

✅ **Networking**
- P2P communication (libp2p) with gossipsub
- Block and transaction propagation
- mDNS peer discovery
- Automatic block synchronization

✅ **RPC API**
- JSON-RPC server (8 core methods)
- HTTP/WebSocket support
- Proof verification endpoints
- REST HTTP bridge

✅ **Storage**
- RocksDB integration with 4 column families
- LZ4 compression
- State persistence
- Block and transaction storage

✅ **Enterprise Frontend**
- Next.js 14 admin dashboard
- Identity and wallet management
- Asset management interface
- Real-time notifications

## Common Issues

### liboqs not found

```bash
# Set PKG_CONFIG_PATH
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH

# Verify
pkg-config --modversion liboqs
```

### Compilation errors

```bash
# Clean and rebuild
cargo clean
cargo build --release
```

### Port already in use

```bash
# Find process
sudo lsof -i :30333
sudo kill -9 <PID>

# Or use different port
./target/release/boundless-node --dev --mining --port 30334
```

## Next Steps

1. **Test the node**: Run development node and watch it mine blocks
2. **Explore code**: Read through core/, consensus/, crypto/ modules
3. **Build contracts**: Compile and test smart contracts
4. **Contribute**: See PHASE1_COMPLETION_PLAN.md for remaining tasks

## Documentation

- [Post-Quantum Assurance](POST_QUANTUM_ASSURANCE.md) - **Comprehensive PQC security model** 🔐
- [Technical Architecture](TECHNICAL-ARCHITECTURE.md) - Detailed architecture and Enterprise layer
- [Deployment Guide](DEPLOYMENT.md) - Production deployment instructions
- [Testing Guide](TESTING.md) - Comprehensive testing procedures
- [Implementation Status](IMPLEMENTATION_STATUS.md) - Complete feature status and API docs
- [Development Plan](BOUNDLESS_COMPREHENSIVE_DEVELOPMENT_PLAN.md) - Long-term roadmap
- [Code Review](README.md) - Main project overview

## Support

- GitHub Issues: https://github.com/boundless-bls/platform/issues
- Documentation: https://docs.boundless-bls.com
- Discord: https://discord.gg/boundless-bls

---

**Welcome to Boundless BLS!** 🚀🔐
