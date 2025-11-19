# Windows Docker Update - Step by Step

## Quick Manual Update (5 minutes)

Since you're using Docker Desktop on Windows, here's the easiest way:

### 1. Stop Old Containers (Docker Desktop)

1. Open **Docker Desktop**
2. Go to **Containers** tab
3. Select all three nodes:
   - `boundless-node1`
   - `boundless-node2`
   - `boundless-node3`
4. Click **Stop** (trash icon)
5. Click **Delete** to remove them

### 2. Pull New Image

Open **PowerShell** and run:

```powershell
docker pull ghcr.io/codenlighten/boundless-bls:latest
docker tag ghcr.io/codenlighten/boundless-bls:latest boundless-bls:latest
```

**Or** in Docker Desktop:
- Go to **Images** tab
- Click **Pull an image**
- Enter: `ghcr.io/codenlighten/boundless-bls:latest`
- Click **Pull**

### 3. Clear Old Blockchain Data

**PowerShell:**
```powershell
cd C:\path\to\boundless-git-collab

Remove-Item -Recurse -Force .\docker-data\node1\db\*
Remove-Item -Recurse -Force .\docker-data\node2\db\*
Remove-Item -Recurse -Force .\docker-data\node3\db\*
```

**Or manually:**
- Navigate to `docker-data\node1\db` folder
- Delete all contents
- Repeat for `node2\db` and `node3\db`

### 4. Start New Containers

**Using docker-compose.yml** (if you have it):
```powershell
docker-compose up -d
```

**Or manually start each node:**

**Node 1:**
```powershell
docker run -d `
  --name boundless-node1 `
  --restart unless-stopped `
  -p 30333:30333 `
  -p 9933:9933 `
  -p 3001:3001 `
  -v ${PWD}\docker-data\node1:/data `
  boundless-bls:latest `
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

**Node 2:**
```powershell
docker run -d `
  --name boundless-node2 `
  --restart unless-stopped `
  -p 30334:30333 `
  -p 9934:9933 `
  -p 3002:3001 `
  -v ${PWD}\docker-data\node2:/data `
  boundless-bls:latest `
  --base-path /data `
  --chain testnet `
  --name "BryanNode2" `
  --bootnodes "/ip4/70.32.195.180/tcp/30333/p2p/12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r" `
  --mining `
  --mining-threads 0 `
  --http-port 3001 `
  --rpc-external `
  --rpc-cors all
```

**Node 3:**
```powershell
docker run -d `
  --name boundless-node3 `
  --restart unless-stopped `
  -p 30335:30333 `
  -p 9935:9933 `
  -p 3003:3001 `
  -v ${PWD}\docker-data\node3:/data `
  boundless-bls:latest `
  --base-path /data `
  --chain testnet `
  --name "BryanNode3" `
  --bootnodes "/ip4/70.32.195.180/tcp/30333/p2p/12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r" `
  --mining `
  --mining-threads 0 `
  --http-port 3001 `
  --rpc-external `
  --rpc-cors all
```

### 5. Verify in Docker Desktop

1. Go to **Containers** tab
2. You should see 3 green containers running
3. Click on **boundless-node1**
4. Go to **Logs** tab
5. Look for:
   - `✅ Connected to bootstrap peer`
   - `📩 Received NewBlock from 12D3KooWDqN...`
   - `Mining block...`

### 6. Check Sync Status

**In PowerShell:**
```powershell
# Check blockchain info
Invoke-WebRequest -Uri http://localhost:3001/api/v1/chain/info | Select-Object -ExpandProperty Content
```

**Or in browser:**
- Open: http://localhost:3001/api/v1/chain/info
- Should show block height ~2015-2020 (matching other node)

---

## Expected Timeline

- Stop containers: 30 seconds
- Pull image: 1-2 minutes (downloads 400MB)
- Clear data: 10 seconds
- Start containers: 1 minute
- Sync blockchain: 2-5 minutes
- **Total: ~5-10 minutes**

---

## What You'll See in Logs

```
2025-11-19T02:30:15.123Z  INFO boundless_node: 🚀 Starting Boundless BLS Node
2025-11-19T02:30:15.456Z  INFO boundless_p2p::network: 🌐 Listening on /ip4/0.0.0.0/tcp/30333
2025-11-19T02:30:16.789Z  INFO boundless_p2p::network: ✅ Connected to bootstrap peer
2025-11-19T02:30:20.123Z  INFO boundless_node: 📩 Received NewBlock from 12D3KooWDqN...
2025-11-19T02:30:20.124Z  INFO boundless_node: 🆕 Received new block #1 from peer
2025-11-19T02:30:25.456Z  INFO boundless_consensus::miner: ⛏️  Mining block with difficulty...
```

**Good signs:**
- ✅ No "Transaction has no inputs" errors
- ✅ "Received new block" messages
- ✅ Block height increasing

---

## Troubleshooting

**Containers won't start:**
- Make sure old containers are fully deleted
- Check Docker Desktop is running
- Try: `docker system prune -a` then pull image again

**Can't access localhost:3001:**
- Check Windows Firewall
- Verify port 3001 is exposed: `docker port boundless-node1`
- Try: http://127.0.0.1:3001/api/v1/chain/info

**Nodes not syncing:**
- Check internet connection
- Verify bootstrap peer IP: 70.32.195.180
- Check logs for connection errors

---

## After Update

All 4 nodes will be:
- ✅ On same blockchain (unified)
- ✅ Supporting ML-DSA transactions
- ✅ Mining together
- ✅ Ready for post-quantum transaction testing

Then we can send the first ML-DSA transaction and watch it propagate! 🚀
