# Starting Your BLS Node and Connecting to the Network

## Quick Start

Your node is already installed and configured. Follow these steps to start mining and sync with the network.

---

## Step 1: Navigate to the Node Directory

```bash
cd ~/boundless-bls-main
```

---

## Step 2: Start the Node

**Option A: Use the management script (recommended)**
```bash
./bryan-scripts/start-node.sh
```

**Option B: Manual start**
```bash
./target/release/boundless-node --config ~/boundless-data/config.toml --mining
```

---

## Step 3: Monitor Your Node

### Check Node Status
```bash
./bryan-scripts/status.sh
```

This shows:
- ✅ Current block height
- ✅ Mining status
- ✅ Peer connections
- ✅ System resources

### Watch Live Logs
```bash
tail -f ~/boundless-data/logs/node.log
```

**Look for these messages (indicates success):**
- `✅ Connected to peer: 12D3KooWDqN...` 
- `✅ Syncing blocks from height X`
- `✅ Successfully mined block at height X`

### Filter for Important Events
```bash
# Watch peer connections
tail -f ~/boundless-data/logs/node.log | grep -i "peer\|connected"

# Watch block sync
tail -f ~/boundless-data/logs/node.log | grep -i "sync\|block"

# Watch mining
tail -f ~/boundless-data/logs/node.log | grep -i "mining\|mined"
```

---

## Step 4: Verify Blockchain Sync

### Check Current Block Height
```bash
curl -X POST http://localhost:9933 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}'
```

**You should see your height increasing as you sync from the bootstrap node (currently at 2000+ blocks)**

### Check Network Peers
```bash
curl -X POST http://localhost:9933 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"network_getPeers","params":[],"id":1}'
```

**You should see the bootstrap node:**
- **Peer ID:** `12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r`
- **Address:** `70.32.195.180:30333`

---

## Bootstrap Node Configuration

Your node is already configured to connect to the main bootstrap node:

**Bootstrap Node Details:**
- **Peer ID:** `12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r`
- **Public IP:** `70.32.195.180`
- **Port:** `30333`

**Verify your configuration:**
```bash
cat ~/boundless-data/config.toml | grep -A 5 "bootstrap_peers"
```

Should show:
```toml
[network]
bootstrap_peers = ["/ip4/70.32.195.180/tcp/30333/p2p/12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r"]
```

---

## Your Node Configuration

**Ports:**
- **P2P (libp2p):** 30333 (must be open for incoming connections)
- **RPC (HTTP):** 9933 (localhost only)
- **WebSocket:** 9944 (localhost only)
- **Metrics:** 9615 (localhost only)

**Data Locations:**
- **Node Binary:** `~/boundless-bls-main/target/release/boundless-node`
- **Configuration:** `~/boundless-data/config.toml`
- **Blockchain Data:** `~/boundless-data/db/`
- **Logs:** `~/boundless-data/logs/node.log`

**Your Coinbase Address (where mining rewards go):**
```bash
cat ~/boundless-data/config.toml | grep coinbase_address
```

---

## Management Commands

### Start Node
```bash
cd ~/boundless-bls-main
./bryan-scripts/start-node.sh
```

### Stop Node
```bash
cd ~/boundless-bls-main
./bryan-scripts/stop-node.sh
```

### Check Status
```bash
cd ~/boundless-bls-main
./bryan-scripts/status.sh
```

### Restart Node
```bash
cd ~/boundless-bls-main
./bryan-scripts/stop-node.sh
./bryan-scripts/start-node.sh
```

---

## Troubleshooting

### Node Won't Start
```bash
# Check if already running
ps aux | grep boundless-node

# Kill existing process
pkill -9 boundless-node

# Try starting again
./bryan-scripts/start-node.sh
```

### Not Connecting to Bootstrap Node
```bash
# Test connectivity to bootstrap node
nc -zv 70.32.195.180 30333

# Should see: "Connection to 70.32.195.180 30333 port [tcp/*] succeeded!"
```

### Check Firewall (Ubuntu/WSL)
```bash
sudo ufw status
sudo ufw allow 30333/tcp
```

### Low or No Mining Hashrate
```bash
# Check CPU usage
htop

# Verify mining is enabled in config
cat ~/boundless-data/config.toml | grep -A 3 "mining"
```

Should show:
```toml
[mining]
enabled = true
threads = 0  # 0 = use all available CPU cores
```

---

## Expected Behavior

### Initial Startup (First 5 minutes)
1. ✅ Node starts and loads configuration
2. ✅ P2P network initializes on port 30333
3. ✅ Connects to bootstrap node (70.32.195.180:30333)
4. ✅ Begins syncing blocks (you'll see height increasing rapidly)
5. ✅ Mining starts once sync is complete

### Steady State (After sync)
- **Block Sync:** Every ~10 seconds (new blocks from network)
- **Mining:** Continuously attempting to mine new blocks
- **Peers:** At least 1 peer (the bootstrap node, more as network grows)
- **CPU Usage:** High (80-100% if mining with all cores)
- **Memory:** ~1-4 GB depending on cache settings

### Success Indicators
```bash
# Log messages showing success:
✅ "Blockchain initialized with genesis block"
✅ "Connected to bootstrap peer"
✅ "Synced to block height: 2000+"
✅ "Mining started with X threads"
✅ "Successfully mined block"
✅ "Received block from peer"
```

---

## Monitoring Dashboard (Future)

Once your node is fully synced, you can access metrics at:
```bash
curl http://localhost:9615/metrics
```

This shows:
- Block height
- Peer count
- Transaction pool size
- Mining hashrate
- Network bandwidth

---

## Next Steps

1. ✅ **Start your node** (see Step 2 above)
2. ✅ **Wait for sync** (may take 10-30 minutes to sync 2000+ blocks)
3. ✅ **Verify mining** (check logs for "Successfully mined block")
4. ✅ **Open P2P port** (port 30333 on your router for incoming connections)
5. ✅ **Monitor performance** (use `status.sh` regularly)

---

## Port Forwarding (Optional but Recommended)

To allow other nodes to connect to you (making you a full network participant):

1. **Find your local IP:**
   ```bash
   ip addr show eth0 | grep "inet " | awk '{print $2}' | cut -d/ -f1
   ```
   Example: `172.19.228.193`

2. **Configure router:**
   - External port: **30333** → Internal IP: **172.19.228.193** port **30333**

3. **Verify:**
   ```bash
   # From another machine:
   nc -zv YOUR_PUBLIC_IP 30333
   ```

---

## Support

**Configuration file:** `~/boundless-data/config.toml`  
**Logs:** `~/boundless-data/logs/node.log`  
**Scripts:** `~/boundless-bls-main/bryan-scripts/`

**Check blockchain explorer (coming soon):**  
The BLS Explorer will show real-time network stats, blocks, and transactions.

---

## Summary Commands (Copy & Paste)

```bash
# Navigate to node directory
cd ~/boundless-bls-main

# Start the node
./bryan-scripts/start-node.sh

# Watch logs
tail -f ~/boundless-data/logs/node.log

# Check status
./bryan-scripts/status.sh

# Test bootstrap connectivity
nc -zv 70.32.195.180 30333

# Check current block height
curl -X POST http://localhost:9933 -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}'
```

---

🚀 **Ready to mine! Your node will connect to the network and start contributing to the blockchain.**
