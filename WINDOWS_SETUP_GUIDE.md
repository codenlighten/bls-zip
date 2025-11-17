# Windows Setup Guide for Boundless BLS Testing

## Quick Start - Choose One Option

### Option 1: WSL2 (Recommended - Best for blockchain development)

```powershell
# Run in PowerShell as Administrator
wsl --install

# After restart, set up Ubuntu and then:
cd /mnt/c/Users/ripva/Desktop/boundless-bls-platform

# Install Rust in WSL
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install dependencies
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev

# Build and test
cargo build --release
./scripts/test_multi_node.sh
```

**Advantages:**
- Native Linux environment (blockchain's natural habitat)
- All bash scripts work perfectly
- Full development toolchain
- Better performance for P2P networking

---

### Option 2: Visual Studio Build Tools (Windows Native)

```powershell
# Download and install Visual Studio Build Tools
# https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022

# Select "Desktop development with C++" workload
# After installation, in Git Bash:

export PATH="$HOME/.cargo/bin:$PATH"
rustup default stable-x86_64-pc-windows-msvc

cd /c/Users/ripva/Desktop/boundless-bls-platform
cargo build --release

# Convert bash scripts to PowerShell or run via Git Bash
```

**Advantages:**
- Native Windows binaries
- Integrates with Visual Studio
- No virtualization overhead

---

### Option 3: MSYS2/MinGW-w64 (Lightweight)

```bash
# Download and install MSYS2 from https://www.msys2.org/

# In MSYS2 terminal:
pacman -S --needed base-devel mingw-w64-x86_64-toolchain

# Then in Git Bash:
export PATH="$HOME/.cargo/bin:$PATH"
rustup default stable-x86_64-pc-windows-gnu

cd /c/Users/ripva/Desktop/boundless-bls-platform
cargo build --release
```

**Advantages:**
- Lighter than Visual Studio
- Bash scripts work natively
- Good middle ground

---

## After Build Succeeds

### Run Automated Tests

```bash
# Multi-node synchronization test
chmod +x scripts/test_multi_node.sh
./scripts/test_multi_node.sh

# Network synchronization verification
chmod +x scripts/verify_network_sync.sh
./scripts/verify_network_sync.sh

# Performance benchmarks
chmod +x scripts/benchmark_performance.sh
./scripts/benchmark_performance.sh
```

### Manual Testing

```bash
# Terminal 1: Start first node (mining)
./target/release/boundless-node --dev --mining

# Terminal 2: Start second node (syncs automatically)
./target/release/boundless-node --dev --port 30334 --rpc-port 9934

# Terminal 3: Query blockchain
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}'
```

---

## Current Status

- ✅ Phase 1: Core blockchain (95% complete)
- ✅ Phase 2: RPC, Storage, P2P (100% complete)
- ✅ Phase 3: Network sync (90% complete)
- ✅ All test frameworks created
- ⏳ **Blocked on:** Windows C++ build tools installation

## Quick Decision Matrix

| Need | Use This |
|------|----------|
| Professional blockchain development | **WSL2** |
| Native Windows integration | **Visual Studio Build Tools** |
| Lightweight, bash-friendly | **MSYS2** |
| Already have Visual Studio | **VS Build Tools** |
| Docker experience | **Docker + WSL** |

---

## Troubleshooting

### If build fails with "linker error"
- MSVC: Install Visual Studio Build Tools
- GNU: Install MinGW-w64 binutils

### If scripts won't run
- WSL: Scripts work natively
- Windows: Use Git Bash or convert to PowerShell

### If ports are in use
- Change ports: `--port 30334 --rpc-port 9934`
- Kill processes: `pkill boundless-node`

---

## Next Steps After Setup

1. Build: `cargo build --release` (~10 minutes first time)
2. Unit tests: `cargo test --all` (~5 minutes)
3. Multi-node test: `./scripts/test_multi_node.sh` (~3 minutes)
4. Network verification: `./scripts/verify_network_sync.sh` (~5 minutes)
5. Benchmarks: `./scripts/benchmark_performance.sh` (~8 minutes)

**Total testing time: ~30 minutes**

---

## WSL2 One-Liner (After installing WSL)

```bash
cd /mnt/c/Users/ripva/Desktop/boundless-bls-platform && \
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
source $HOME/.cargo/env && \
sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev && \
cargo build --release && \
./scripts/test_multi_node.sh
```

This will build and test in one go after WSL is set up.
