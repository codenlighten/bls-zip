# Boundless BLS - Bryan's Server Deployment Guide

This guide will help Bryan replicate the exact production node setup on his server.

## 🎯 Goal
Set up a second production node that connects to our primary node and maintains a synchronized blockchain.

## 📋 Prerequisites on Bryan's Server

### 1. System Requirements
- **OS**: Ubuntu 22.04 LTS (or compatible Linux)
- **RAM**: Minimum 4GB, Recommended 8GB+
- **Storage**: Minimum 50GB free space (Bryan has more room, so this is fine)
- **Network**: Ports 30333, 9933, 9944, 9615 accessible

### 2. Install Dependencies

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install build essentials
sudo apt install -y build-essential cmake ninja-build libssl-dev pkg-config git curl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # Should be 1.75.0 or later

# Install liboqs (Post-Quantum Cryptography Library)
git clone --depth 1 https://github.com/open-quantum-safe/liboqs.git
cd liboqs
mkdir build && cd build
cmake -GNinja -DCMAKE_INSTALL_PREFIX=/usr/local ..
ninja
sudo ninja install

# Set environment variables
echo 'export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH' >> ~/.bashrc
source ~/.bashrc

# Verify installation
pkg-config --modversion liboqs  # Should show version 0.15.0 or similar
```

## 🚀 Setup Instructions

### Step 1: Clone Repository

```bash
# Clone your fork or the upstream repository
cd ~
git clone git@github.com:Saifullah62/BLS.git boundless-bls
cd boundless-bls

# Or if you want to use our exact code (with all fixes):
git clone git@github.com:codenlighten/BLS-connect.git boundless-bls
cd boundless-bls
```

### Step 2: Build the Node

```bash
# Build release version (takes ~5-10 minutes)
cargo build --release --bin boundless-node

# Verify build
./target/release/boundless-node --version
```

### Step 3: Configure the Node

Create production configuration:

```bash
# Create production directory
mkdir -p production-node/data

# Copy the configuration template
cp production-node/config.toml production-node/bryan-config.toml
```

Edit `production-node/bryan-config.toml` and update the `bootnodes` section:

```toml
[network]
listen_addr = "/ip4/0.0.0.0/tcp/30333"

# **IMPORTANT**: Add our node's multiaddr here
# Replace <PEER_ID> and <PRIMARY_IP> with values from BOOTNODE_INFO.txt
bootnodes = [
    "/ip4/<PRIMARY_IP>/tcp/30333/p2p/<PEER_ID>"
]
```

**Get the bootnode info from the file we'll provide: `BOOTNODE_INFO.txt`**

### Step 4: Start Bryan's Node

Use the deployment script:

```bash
cd production-node

# Start the node
./start-node.sh

# The node will:
# ✅ Connect to our primary node via bootnode
# ✅ Sync the blockchain (download genesis and all blocks)
# ✅ Begin mining blocks
# ✅ Propagate blocks to the network
```

### Step 5: Verify Connection

Check that Bryan's node connected:

```bash
# View logs
tail -f production-node/logs/node-*.log

# Look for these messages:
# - "🔗 Connected to peer <PEER_ID>"
# - "📦 Received X blocks from peer"
# - "✅ Added block #X from peer"
```

## 🔍 Verification Checklist

### On Bryan's Server:

```bash
# 1. Check node is running
ps aux | grep boundless-node

# 2. Check P2P connection
grep "Connected to peer" production-node/logs/*.log

# 3. Check blockchain sync
grep "Added block" production-node/logs/*.log | tail -10

# 4. Query RPC (should return current height)
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}'

# 5. Check Prometheus metrics
curl http://localhost:9615/metrics | grep boundless
```

### On Our Server (Primary Node):

```bash
# Check Bryan's connection
grep "peer connected" production-node/logs/*.log | grep <BRYAN_PEER_ID>

# Verify block propagation
grep "broadcasting block" production-node/logs/*.log
```

## 🔧 Common Issues & Solutions

### Issue 1: Connection Refused
**Symptom**: "Connection refused" in logs  
**Solution**: 
- Verify firewall allows port 30333
- Check our node's IP is correct in bootnodes
- Ensure our node is running: `./production-node/stop-node.sh && ./production-node/start-node.sh`

### Issue 2: Genesis Mismatch
**Symptom**: "Invalid genesis block" or hash mismatch  
**Solution**:
- Both nodes MUST start with the same genesis
- Delete Bryan's data: `rm -rf production-node/data/db`
- Restart Bryan's node to re-sync from our node

### Issue 3: liboqs Not Found
**Symptom**: Build fails with "liboqs" error  
**Solution**:
```bash
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
sudo ldconfig
```

### Issue 4: Port Already in Use
**Symptom**: "Address already in use"  
**Solution**:
```bash
sudo lsof -i :30333
sudo kill -9 <PID>
```

## 📊 Monitoring Both Nodes

### RPC API Calls

Get block height:
```bash
curl -X POST http://<NODE_IP>:9933 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}'
```

Get best block hash:
```bash
curl -X POST http://<NODE_IP>:9933 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBestBlockHash","params":[],"id":1}'
```

### Prometheus Metrics

```bash
# On either node
curl http://localhost:9615/metrics
```

## 📁 File Structure

After setup, Bryan's server will have:

```
boundless-bls/
├── target/release/
│   └── boundless-node          # Compiled binary
├── production-node/
│   ├── config.toml             # Bryan's configuration
│   ├── start-node.sh           # Start script
│   ├── stop-node.sh            # Stop script
│   ├── data/
│   │   └── db/                 # RocksDB blockchain data
│   ├── logs/
│   │   └── node-*.log          # Node logs
│   └── BOOTNODE_INFO.txt       # Our node's connection info
└── ... (source code)
```

## 🔐 Security Notes

**For Production Deployment:**

1. **Firewall Configuration**:
   ```bash
   sudo ufw allow 30333/tcp  # P2P
   sudo ufw allow 9933/tcp   # RPC (restrict to trusted IPs)
   sudo ufw allow 9944/tcp   # WebSocket (restrict to trusted IPs)
   sudo ufw enable
   ```

2. **Enable TLS** in config.toml:
   ```toml
   [security]
   enable_tls = true
   ```

3. **Restrict RPC Access**:
   ```toml
   [rpc]
   http_addr = "127.0.0.1:9933"  # Only localhost
   ```

4. **Enable Authentication**:
   ```toml
   [security]
   require_authentication = true
   ```

## 🎯 Success Criteria

Bryan's node is successfully deployed when:

- ✅ Node is running (check with `ps aux | grep boundless-node`)
- ✅ Connected to our node (logs show peer connection)
- ✅ Blockchain synced (both nodes show same block height)
- ✅ Mining enabled (logs show "Mined block" messages)
- ✅ Blocks propagating (logs show block exchange between nodes)
- ✅ RPC responding (curl commands return valid data)

## 📞 Support

If Bryan encounters issues:

1. Check logs: `tail -f production-node/logs/*.log`
2. Verify our node is accessible: `telnet <PRIMARY_IP> 30333`
3. Compare genesis hashes (both should match)
4. Share relevant log snippets for debugging

## 🚦 Quick Start Commands for Bryan

```bash
# Start node
cd ~/boundless-bls/production-node
./start-node.sh

# Check status
tail -f logs/*.log

# Stop node
./stop-node.sh

# View metrics
curl http://localhost:9615/metrics

# Query blockchain
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}'
```

---

**Note**: The `BOOTNODE_INFO.txt` file will be generated when you start the primary node and will contain the exact multiaddr Bryan needs to add to his configuration.
