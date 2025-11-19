# 🔄 Bryan - Sync From Bootstrap Node

## Important: Genesis Block Issue

Your Docker containers created their own genesis block, which means you're on a **separate blockchain** from the main node. You need to delete your data and sync from the bootstrap node's genesis.

## Quick Fix (2 minutes)

### Step 1: Stop Your Containers

**PowerShell:**
```powershell
docker stop boundless-node1 boundless-node2 boundless-node3
docker rm boundless-node1 boundless-node2 boundless-node3
```

**Or Docker Desktop:**
- Stop and delete all 3 containers

### Step 2: Clear ALL Blockchain Data

**IMPORTANT:** Delete everything, not just the `db` folder:

**PowerShell:**
```powershell
cd C:\Users\YourName\boundless-git-collab  # Your actual path

Remove-Item -Recurse -Force .\docker-data\node1\*
Remove-Item -Recurse -Force .\docker-data\node2\*
Remove-Item -Recurse -Force .\docker-data\node3\*
```

**Or File Explorer:**
- Delete ALL contents of `docker-data\node1\`
- Delete ALL contents of `docker-data\node2\`
- Delete ALL contents of `docker-data\node3\`

### Step 3: Restart Containers

The containers will now start with empty data and sync from the bootstrap peer.

**Using the script (if you have it):**
```bash
./pull-docker-image.sh
```

**Or manually start containers:**

**Node 1:**
```powershell
docker run -d `
  --name boundless-node1 `
  --restart unless-stopped `
  -p 30333:30333 `
  -p 9933:9933 `
  -p 3001:3001 `
  -v ${PWD}\docker-data\node1:/data `
  ghcr.io/codenlighten/boundless-bls:latest `
  --base-path /data `
  --chain testnet `
  --name "BryanNode1" `
  --bootnodes "/ip4/70.32.195.180/tcp/30333/p2p/12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r" `
  --mining `
  --mining-threads 0 `
  --http-port 3001 `
  --rpc-external `
  --rpc-cors all
```

**Node 2 & 3:** Same as before (see WINDOWS_UPDATE_GUIDE.md)

### Step 4: Verify Sync

Watch the logs - you should see:

```
✅ Connected to bootstrap peer
📩 Received NewBlock from 12D3KooWDqN...
🆕 Received new block #1 from peer
🆕 Received new block #2 from peer
...
```

**Check block height matches:**
```powershell
docker exec boundless-node1 curl http://localhost:3001/api/v1/chain/info
```

Should show block height matching the bootstrap node (currently ~300-400).

## What's Happening

- **Bootstrap node** (70.32.195.180) created genesis block `80f5f229...`
- Your nodes need to **download** this genesis and all subsequent blocks
- Once synced, all 4 nodes will be on the **same blockchain**
- Then mining and transactions will work across all nodes

## Timeline

- Stop containers: 30 seconds
- Clear data: 30 seconds  
- Start containers: 1 minute
- Sync blocks: 2-5 minutes (downloads ~300-400 blocks)
- **Total: ~5-7 minutes**

## How to Know It's Working

**Good signs:**
- Logs show "Received new block from peer"
- Block height increasing rapidly
- Eventually matches bootstrap node's height
- No "Genesis block mismatch" errors

**Bad signs:**
- Creating new genesis block (means data wasn't cleared)
- No "Received new block" messages
- Stuck at block 0 or low number

---

**After this, all 4 nodes will be unified on one blockchain with the same genesis!** 🎉
