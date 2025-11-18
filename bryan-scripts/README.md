# Boundless BLS - Bryan's Main Node Deployment

## 🎯 Overview

This deployment package sets up Bryan's server as the **main production node** for the Boundless BLS blockchain network. Your server will become the primary node due to:

- ✅ **Fiber optic connection** - High bandwidth, low latency
- ✅ **Massive TB hard drive** - Can store the entire blockchain + future growth
- ✅ **WSL Ubuntu** - Stable Linux environment on Windows
- ✅ **Always-on capability** - Reliable uptime for network stability

## 📦 What's Included

```
bryan-scripts/
├── deploy-bryan-main-node.sh    # ONE-CLICK complete deployment
├── start-node.sh                # Start the node
├── stop-node.sh                 # Stop the node
├── status.sh                    # Check node status
└── README.md                    # This file
```

## 🚀 Quick Start (One Command!)

### Step 1: Download and Run Deployment Script

```bash
# On your WSL Ubuntu terminal:
cd ~
curl -O https://raw.githubusercontent.com/Saifullah62/BLS/main/deploy-bryan-main-node.sh
bash deploy-bryan-main-node.sh
```

**That's it!** The script will:
1. ✅ Install all dependencies (Rust, liboqs, system packages)
2. ✅ Clone the repository
3. ✅ Build the release binary
4. ✅ Configure the node
5. ✅ Set up firewall rules
6. ✅ Prepare data directories

**Total time**: ~10-15 minutes (mostly compilation)

### Step 2: Start the Node

```bash
cd ~/boundless-bls-main
./bryan-scripts/start-node.sh
```

### Step 3: Verify It's Working

```bash
./bryan-scripts/status.sh
```

You should see:
- ✅ Node Status: RUNNING
- ✅ Block Height: syncing from Greg's node
- ✅ P2P Connections: 1+ (connected to bootstrap)

## 📊 What Happens During Deployment

### Automatic Installation:

1. **System Dependencies**
   - build-essential, cmake, ninja-build
   - libssl-dev, pkg-config
   - git, curl, wget, htop, net-tools

2. **Rust Toolchain**
   - Latest stable Rust compiler
   - Cargo package manager
   - Target optimization for your CPU

3. **liboqs (Post-Quantum Crypto)**
   - ML-KEM-768, ML-DSA-44, Falcon-512
   - Built from source for optimal performance

4. **Boundless BLS Node**
   - Compiled in release mode with optimizations
   - ~5-10 minute build time

5. **Configuration**
   - Auto-generated production config
   - Connected to Greg's bootstrap node
   - Optimized for your hardware

6. **Data Directories**
   ```
   ~/boundless-data/
   ├── db/              # Blockchain database (will grow to TB size)
   ├── logs/            # Node operation logs
   ├── backups/         # Database backups
   └── config.toml      # Node configuration
   ```

## 🔧 Configuration Details

Your node will be configured with:

**Network:**
- P2P Port: `30333` (listens on all interfaces)
- Bootstrap: Connected to Greg's node (`70.32.195.180:30333`)

**RPC:**
- HTTP: `9933`
- WebSocket: `9944`
- Accessible from all interfaces (can restrict later)

**Storage:**
- Database: `~/boundless-data/db`
- Cache: 4GB RAM (optimize for performance)
- Compression: LZ4 (saves ~30% disk space)

**Mining:**
- Enabled: Yes
- Threads: Auto-detect (uses all CPU cores)
- Your server becomes primary miner

**Security:**
- Firewall: UFW configured for P2P port
- TLS: Disabled (enable when exposing to internet)
- Authentication: Disabled (trusted network)

## 🎮 Node Management

### Start Node
```bash
cd ~/boundless-bls-main
./bryan-scripts/start-node.sh
```

### Stop Node
```bash
./bryan-scripts/stop-node.sh
```

### Check Status
```bash
./bryan-scripts/status.sh
```

### View Logs
```bash
tail -f ~/boundless-data/logs/node-*.log
```

### Query Blockchain
```bash
# Get current block height
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}'

# Get best block hash
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBestBlockHash","params":[],"id":1}'
```

## 📈 Expected Behavior

### Initial Sync (First 10 minutes):

1. **Node Starts**
   ```
   🚀 Starting Boundless BLS Node v0.1.0
   📁 Data directory: ~/boundless-data
   ⛓️  Blockchain initialized
   ```

2. **Connects to Bootstrap**
   ```
   🔗 Connected to peer 12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r
   📦 Received blocks from peer
   ```

3. **Syncs Blockchain**
   ```
   ✅ Added block #1 from peer
   ✅ Added block #2 from peer
   ...
   ✅ Added block #2000+ from peer
   ```

4. **Begins Mining**
   ```
   ⛏️  Mining enabled
   ✨ Mined block #2001 - Hash: 00051ca4...
   ```

### Steady State:

- **Block Height**: Matches Greg's node (same chain)
- **Connections**: 1+ P2P peers
- **Mining**: Active (~750K-1M H/s depending on CPU)
- **Database**: Growing (~100MB per 1000 blocks)
- **RAM Usage**: ~500MB-1GB
- **CPU Usage**: Variable (high during mining, low during idle)

## 🔒 Security & Firewall

The deployment script configures UFW firewall:

```bash
# Check firewall status
sudo ufw status

# You should see:
# 22/tcp    ALLOW    SSH
# 30333/tcp ALLOW    BLS P2P
# 9615/tcp  ALLOW    BLS Metrics
```

### For Public Internet Exposure:

If you want others to connect to YOUR node as bootstrap:

1. **Configure Windows Firewall** (if using WSL)
   - Allow inbound TCP 30333
   - Forward to WSL IP

2. **Configure Router Port Forwarding**
   - External port 30333 → Your PC's IP → Port 30333

3. **Get Your Public IP**
   ```bash
   curl ifconfig.me
   ```

4. **Share Your Bootnode Multiaddr**
   ```
   /ip4/<YOUR_PUBLIC_IP>/tcp/30333/p2p/<YOUR_PEER_ID>
   ```

## 📊 Monitoring & Maintenance

### Daily Checks:

```bash
# Quick status
./bryan-scripts/status.sh

# Expected output:
✅ Node Status: RUNNING
  Block Height: 2500+
  P2P Connections: 1+
  Database Size: 250MB+
```

### Weekly Maintenance:

```bash
# Check disk space (important with TB growth!)
df -h ~/boundless-data

# View recent logs
tail -100 ~/boundless-data/logs/node-*.log

# Check for errors
grep "ERROR" ~/boundless-data/logs/node-*.log
```

### Database Backups:

```bash
# Stop node first!
./bryan-scripts/stop-node.sh

# Backup database
tar -czf ~/boundless-data/backups/db-backup-$(date +%Y%m%d).tar.gz \
  -C ~/boundless-data db/

# Restart node
./bryan-scripts/start-node.sh
```

## 🆘 Troubleshooting

### Node Won't Start

```bash
# Check logs
cat ~/boundless-data/logs/node-*.log | tail -50

# Common issues:
# 1. Port conflict - kill other process on 30333
# 2. Config error - regenerate with deployment script
# 3. Database corruption - restore from backup
```

### Not Connecting to Greg's Node

```bash
# Test connectivity
nc -zv 70.32.195.180 30333

# Should show: "Connection succeeded"
# If fails:
#   - Check internet connection
#   - Verify firewall allows outbound 30333
#   - Check Greg's node is running
```

### Blockchain Not Syncing

```bash
# Check peer connection in logs
grep "Connected to peer" ~/boundless-data/logs/*.log

# Check block reception
grep "Added block" ~/boundless-data/logs/*.log | tail -20

# If no blocks received:
#   - Verify genesis hash matches
#   - Check network connectivity
#   - Restart both nodes
```

### Mining Not Working

```bash
# Check mining status in logs
grep "Mining" ~/boundless-data/logs/*.log

# Verify coinbase address in config
grep "coinbase_address" ~/boundless-data/config.toml

# Should be 64 character hex string
```

## 🌐 Network Topology

### Current Setup:
```
[Greg's Node]  ←→  [Your Node (Bryan)]
  (Bootstrap)        (Main Production)
   2000+ blocks       Syncing + Mining
```

### Future Multi-Node Network:
```
           [Your Node (Main)]
                 ↓  ↑
          ┌──────┴──────┐
          ↓              ↓
   [Greg's Node]   [Other Nodes]
    (Bootstrap)      (Network)
```

## 📞 Support

If you encounter issues:

1. **Check logs**: `~/boundless-data/logs/`
2. **Run status**: `./bryan-scripts/status.sh`
3. **Share error messages**: Post in project chat
4. **Verify Greg's node**: Is it still running?

## 🎯 Success Criteria

Your deployment is successful when:

- ✅ Node starts without errors
- ✅ Connects to Greg's bootstrap node
- ✅ Syncs all existing blocks (2000+)
- ✅ Begins mining new blocks
- ✅ RPC responds to queries
- ✅ Stays running for 24+ hours
- ✅ Database persists across restarts

## 🚦 Quick Reference

```bash
# Deploy (first time only)
bash deploy-bryan-main-node.sh

# Start node
cd ~/boundless-bls-main && ./bryan-scripts/start-node.sh

# Check status
./bryan-scripts/status.sh

# View logs
tail -f ~/boundless-data/logs/node-*.log

# Stop node
./bryan-scripts/stop-node.sh

# Query blockchain
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}'
```

---

**Your server is about to become the backbone of the Boundless BLS network!** 🚀
