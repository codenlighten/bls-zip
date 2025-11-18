# Production Node - First BLS Blockchain Seed

## 🎯 Status: **LIVE AND MINING** ✅

This is the **first production node** of the Boundless BLS blockchain network.

### Current Stats:
- **Node Status**: Running (PID in `node.pid`)
- **Block Height**: 800+ (and growing)
- **Mining Rate**: ~1M H/s
- **Peer ID**: `12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r`
- **Genesis Hash**: `80f5f229801e88f6438be3502582a363d2d1e4e2e0d2da503e80f168b91daa6e`

## 🚀 Quick Start

### Start the Node
```bash
./production-node/start-node.sh
```

### Stop the Node
```bash
./production-node/stop-node.sh
```

### View Logs
```bash
tail -f production-node/logs/node-*.log
```

### Check Status
```bash
# RPC query - current block height
curl -X POST http://localhost:9933 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}'

# Prometheus metrics
curl http://localhost:9615/metrics
```

## 📁 Files

- **`config.toml`** - Production node configuration
- **`start-node.sh`** - Automated startup script
- **`stop-node.sh`** - Graceful shutdown script
- **`BOOTNODE_INFO.txt`** - Connection details for Bryan's node
- **`BRYAN_DEPLOYMENT_GUIDE.md`** - Complete setup guide for Bryan
- **`data/`** - RocksDB blockchain database (not in git)
- **`logs/`** - Node operation logs (not in git)

## 🌐 Network Topology

### Current Setup (Single Node)
```
┌─────────────────────────┐
│   Production Node #1    │
│  (Our Server)           │
│                         │
│  Mining: ✅             │
│  RPC: :9933             │
│  P2P: :30333            │
│  Metrics: :9615         │
└─────────────────────────┘
```

### Target Setup (Bryan Connected)
```
┌─────────────────────────┐     P2P Network      ┌─────────────────────────┐
│   Production Node #1    │◄───────────────────►│   Production Node #2    │
│  (Our Server)           │                      │  (Bryan's Server)       │
│                         │                      │                         │
│  Mining: ✅             │                      │  Mining: ✅             │
│  RPC: :9933             │                      │  RPC: :9933             │
│  P2P: :30333            │                      │  P2P: :30333            │
│  Metrics: :9615         │                      │  Metrics: :9615         │
│                         │                      │                         │
│  Bootstrap Seed         │                      │  Connects via bootnode  │
└─────────────────────────┘                      └─────────────────────────┘
```

## 🔧 For Bryan

**See `BRYAN_DEPLOYMENT_GUIDE.md` for complete instructions.**

### Quick Setup:
1. Clone repository (or use fork with compilation fixes)
2. Install dependencies (Rust, liboqs)
3. Build release binary: `cargo build --release --bin boundless-node`
4. Copy `production-node/` directory setup
5. Add our node as bootnode in your `config.toml`:
   ```toml
   [network]
   bootnodes = [
       "/ip4/192.168.1.13/tcp/30333/p2p/12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r"
   ]
   ```
6. Run `./start-node.sh`
7. Verify connection and sync

## 🔐 Security Notes

**Current Configuration** (Development/Trusted Network):
- RPC on all interfaces (0.0.0.0)
- No TLS
- No authentication
- CORS allows all origins

**For Production Internet Deployment**:
- Enable TLS in config
- Restrict RPC to localhost or VPN
- Enable API authentication
- Configure firewall rules
- Use specific CORS origins

## 📊 Monitoring

### Prometheus Metrics
```bash
curl http://192.168.1.13:9615/metrics
```

Key metrics:
- `boundless_blockchain_height` - Current block height
- `boundless_mempool_size` - Transactions in mempool
- `boundless_peers_connected` - Number of connected peers
- `boundless_blocks_mined_total` - Total blocks mined
- `boundless_mining_hash_rate` - Current mining hash rate

### Log Files
Logs are stored in `production-node/logs/` with timestamped filenames:
- Node startup and shutdown
- Block mining events
- P2P peer connections
- RPC requests (if enabled)
- Error messages

## 🐛 Troubleshooting

### Node Won't Start
```bash
# Check logs
cat production-node/logs/node-*.log | tail -50

# Verify no port conflicts
sudo lsof -i :30333
sudo lsof -i :9933

# Check config syntax
cat production-node/config.toml
```

### Mining Not Working
```bash
# Check mining enabled in config
grep "enabled = true" production-node/config.toml

# Verify coinbase address (must be 32 bytes hex)
grep "coinbase_address" production-node/config.toml

# Check CPU threads allocation
grep "threads" production-node/config.toml
```

### Database Issues
```bash
# Stop node
./production-node/stop-node.sh

# Backup database
cp -r production-node/data/db production-node/data/db.backup

# Restart node
./production-node/start-node.sh
```

## 📈 Next Steps

1. ✅ First node deployed and mining
2. ⏳ Bryan deploys second node on his server
3. ⏳ Verify P2P connection between nodes
4. ⏳ Confirm blockchain sync across both nodes
5. ⏳ Test transaction propagation
6. ⏳ Benchmark network performance
7. ⏳ Deploy additional nodes for network resilience

---

**Node Started**: 2025-11-18 15:14:47 UTC  
**Last Updated**: 2025-11-18 15:20:00 UTC
