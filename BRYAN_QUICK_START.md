# 🚀 BRYAN - DEPLOY YOUR MAIN NODE (ONE COMMAND!)

## Quick Deploy (Copy-Paste This):

```bash
# On your WSL Ubuntu terminal - ONE COMMAND deploys everything:
cd ~ && curl -sL https://raw.githubusercontent.com/codenlighten/BLS-connect/main/deploy-bryan-main-node.sh | bash
```

That's it! The script handles everything automatically.

---

## What Will Happen:

### ⏱️ Timeline (Total: ~10-15 minutes)

**Minutes 0-2**: Installing system dependencies
- build-essential, cmake, git, etc.

**Minutes 2-5**: Installing Rust toolchain
- Latest stable Rust
- Cargo package manager

**Minutes 5-7**: Building liboqs (Post-Quantum Crypto)
- ML-KEM, ML-DSA, Falcon algorithms
- Compiled from source

**Minutes 7-15**: Building Boundless BLS Node
- Compiling in release mode
- Optimized for your CPU
- This is the longest step

**Minute 15**: Final setup
- Creating data directories
- Generating configuration
- Setting up firewall

### ✅ When Complete You'll See:

```
============================================
  ✅ DEPLOYMENT COMPLETE
============================================

Installation Summary:
  Install Directory: /home/bryan/boundless-bls-main
  Data Directory: /home/bryan/boundless-data
  Binary: boundless-node
  Configuration: config.toml

Node Configuration:
  P2P Port: 30333
  RPC Port: 9933 (HTTP)
  WebSocket Port: 9944
  Metrics Port: 9615
  Mining: true (auto threads)

Bootstrap Node:
  Peer ID: 12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r
  Address: 70.32.195.180:30333

Ready to launch! 🚀
```

---

## Then Start Your Node:

```bash
cd ~/boundless-bls-main
./bryan-scripts/start-node.sh
```

You'll see:
```
Starting Boundless BLS Main Node...
✅ Node started successfully!

Node Information:
  PID: 12345
  Peer ID: 12D3KooW... (your unique ID)
  Log: ~/boundless-data/logs/node-*.log

Connected to bootstrap: 70.32.195.180
Syncing blockchain...
✅ Added block #1 from peer
✅ Added block #2 from peer
...
```

---

## Check It's Working:

```bash
# Quick status check
./bryan-scripts/status.sh

# You should see:
✅ Node Status: RUNNING
  Block Height: 2000+ (syncing...)
  P2P Connections: 1+
  Database Size: 250MB+
```

---

## Where Everything Goes:

```
~/boundless-bls-main/          # Code and binary
~/boundless-data/
  ├── db/                      # Blockchain database (will grow to TB)
  ├── logs/                    # Node logs
  ├── backups/                 # Database backups
  └── config.toml              # Your configuration
```

---

## Your Server Becomes Main Node Because:

✅ **Fiber Optic** - High bandwidth, low latency  
✅ **TB Storage** - Can hold full blockchain + growth  
✅ **Always On** - Reliable uptime  
✅ **WSL Ubuntu** - Stable Linux environment  

---

## Troubleshooting

### If Deployment Fails:

**Check error message** - The script will show exactly what failed

**Common issues:**

1. **liboqs build fails** - Need cmake and ninja:
   ```bash
   sudo apt install -y cmake ninja-build
   ```

2. **Rust not found** - Source the environment:
   ```bash
   source $HOME/.cargo/env
   ```

3. **Out of disk space** - Free up ~10GB for build

### If Node Won't Start:

**Check logs:**
```bash
cat ~/boundless-data/logs/node-*.log | tail -50
```

**Test connectivity to Greg's node:**
```bash
nc -zv 70.32.195.180 30333
# Should show: "Connection succeeded"
```

**Restart with fresh config:**
```bash
cd ~/boundless-bls-main
bash deploy-bryan-main-node.sh  # Re-runs deployment
```

---

## What Your Node Will Do:

1. **Connect** to Greg's bootstrap node (70.32.195.180:30333)
2. **Sync** all existing blocks (2000+)
3. **Mine** new blocks using all your CPU cores
4. **Store** blockchain in ~/boundless-data/db (grows to TB)
5. **Serve** as main network node for future peers

---

## Management Commands:

```bash
# Start node
cd ~/boundless-bls-main && ./bryan-scripts/start-node.sh

# Stop node
./bryan-scripts/stop-node.sh

# Check status
./bryan-scripts/status.sh

# View live logs
tail -f ~/boundless-data/logs/node-*.log

# Query blockchain
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}'
```

---

## Success Criteria:

Your node is working when:

- ✅ Status shows "RUNNING"
- ✅ Block height matches Greg's node
- ✅ P2P connections: 1+
- ✅ Mining active (check logs for "Mined block" messages)
- ✅ Database growing in ~/boundless-data/db

---

## 📞 If You Need Help:

1. Share your logs: `cat ~/boundless-data/logs/node-*.log | tail -100`
2. Share status output: `./bryan-scripts/status.sh`
3. Check Greg's node is still running
4. Verify internet connectivity

---

## Ready? Run This:

```bash
cd ~ && curl -sL https://raw.githubusercontent.com/codenlighten/BLS-connect/main/deploy-bryan-main-node.sh | bash
```

**Your server will become the backbone of the Boundless BLS network!** 🌐

---

**Questions?** Check `bryan-scripts/README.md` for detailed documentation after deployment.
