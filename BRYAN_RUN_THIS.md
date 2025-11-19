# 🚀 Bryan - One-Click Setup

## Run This Single Script

Open **PowerShell** and run:

```powershell
cd C:\Users\YourName\boundless-git-collab  # Your actual path
.\setup-bryan-nodes.ps1
```

**That's it!** The script will:

1. ✅ Stop old containers
2. ✅ Pull latest Docker image  
3. ✅ Clear all blockchain data
4. ✅ Start 3 nodes that sync from bootstrap
5. ✅ Generate your wallet
6. ✅ Show status

## What You'll See

The script takes **2-3 minutes** and shows progress:

```
🚀 Boundless BLS - Complete Node Setup
======================================

📦 Step 1/6: Stopping old containers...
✅ Old containers removed

⬇️  Step 2/6: Pulling latest Docker image...
✅ Image pulled

🗑️  Step 3/6: Clearing old blockchain data...
✅ Data cleared

📁 Step 4/6: Creating data directories...
✅ Directories created

🚀 Step 5/6: Starting containers...
✅ All containers started

⏳ Step 6/6: Waiting for nodes to initialize...

🔑 Generating ML-DSA wallet...
✅ Wallet files saved: bryan-wallet.priv, bryan-wallet.pub
```

## Verify It's Working

**Check logs:**
```powershell
docker logs -f boundless-node1
```

**Look for:**
```
✅ Connected to bootstrap peer
📩 Received NewBlock from peer
🆕 Received new block #1 from peer
🆕 Received new block #2 from peer
...
```

**Check sync status:**
```powershell
docker exec boundless-node1 curl http://localhost:3001/api/v1/chain/info
```

Should show block height increasing (syncing from bootstrap).

## Your Wallet

The script created:
- `bryan-wallet.priv` - Your private key (keep secret!)
- `bryan-wallet.pub` - Your public key

The script output shows your wallet address. Save it!

## Next Steps

Once synced (block height ~1400+):

**Check your balance:**
```powershell
docker exec boundless-node1 curl http://localhost:3001/api/v1/balance/YOUR_ADDRESS
```

**Send a transaction:**
```powershell
docker exec boundless-node1 boundless-cli send RECIPIENT_ADDRESS 100000000 --key /data/bryan-wallet.priv
```

---

**Questions?** Call/text Bryan Dev Team
