# 🚀 Bryan - Your Nodes Need an Update!

## The Issue

Your 3 Docker nodes rejected a block with an ML-DSA transaction. The networks have forked (you're on block 2031+, other node is on 2015).

## The Fix (2 Minutes Total!)

### First: Find Your Project Folder

**In File Explorer:**
1. Open File Explorer
2. Navigate to where you cloned the repo (likely `C:\Users\YourName\Documents\` or `C:\Users\YourName\boundless-git-collab`)
3. Click on the address bar → Copy the full path
4. Use that path below

**Or check where your containers are running:**
```powershell
# See where the volumes are mounted
docker inspect boundless-node1 | Select-String "Source"
```
The folder above `docker-data` is your project folder.

### Then: Run the Update

**Option A - PowerShell:**
```powershell
cd C:\Users\YourName\boundless-git-collab  # Replace with your actual path
git pull origin main
bash pull-docker-image.sh
```

**Option B - WSL Ubuntu:**
```bash
# Windows paths in WSL start with /mnt/c/
cd /mnt/c/Users/YourName/boundless-git-collab  # Replace with your actual path
git pull origin main
./pull-docker-image.sh
```

**Option C - Manual (if scripts don't work):**
```powershell
# Stop containers
docker stop boundless-node1 boundless-node2 boundless-node3
docker rm boundless-node1 boundless-node2 boundless-node3

# Pull new image
docker pull ghcr.io/codenlighten/boundless-bls:latest

# Tag it locally
docker tag ghcr.io/codenlighten/boundless-bls:latest boundless-bls:latest

# Clear old data (PowerShell)
Remove-Item -Recurse -Force .\docker-data\node1\db\*
Remove-Item -Recurse -Force .\docker-data\node2\db\*
Remove-Item -Recurse -Force .\docker-data\node3\db\*

# Then restart containers with docker-compose or your usual method
```

**The script:
- Stops old containers
- Clears forked blockchain data
- Pulls pre-built Docker image (2 mins vs 15 mins building)
- Starts 3 updated nodes with ML-DSA support
- Connects to bootstrap peer automatically

## What to Expect

```
🐋 Boundless BLS - Quick Docker Pull & Update
==============================================

📦 Step 1/5: Stopping old containers...
✅ Containers stopped

🗑️  Step 2/5: Clearing old blockchain data (forked chain)...
✅ Old data cleared

⬇️  Step 3/5: Pulling pre-built Docker image from GitHub...
   (This takes 1-2 minutes - much faster than building!)
✅ Image pulled

🚀 Step 4/5: Starting updated containers...
✅ All containers started

⏳ Step 5/5: Waiting for nodes to initialize (30 seconds)...

✅ Update complete!
```

## After Update

Your 3 nodes will:
- Sync from genesis (fresh start, takes 1-5 minutes)
- Support ML-DSA post-quantum transactions
- Reconnect to the network
- All 4 nodes unified on same blockchain again

## Verify It Worked

**PowerShell:**
```powershell
# Check containers running
docker ps

# Check sync status
docker logs boundless-node1 --tail 20

# Check block height matches other node (~2015-2020)
docker exec boundless-node1 curl http://localhost:3001/api/v1/chain/info
```

**Or use Docker Desktop GUI:**
- Open Docker Desktop
- Go to Containers tab
- Click on boundless-node1
- View logs and stats

## Timeline

- Git pull: 5 seconds
- Run script: 2 minutes (downloads 400MB image)
- Sync blockchain: 1-5 minutes
- **Total: ~3-7 minutes**

---

**Then we can test the first ML-DSA transaction on a unified 4-node network!** 🎉

## Questions?

If anything fails:
- Check logs: `docker logs boundless-node1`
- Restart: `docker restart boundless-node1 boundless-node2 boundless-node3`
- Or build locally instead: `./update-docker-nodes.sh` (takes 15 mins)
