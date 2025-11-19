# 🚀 QUICK FIX: Update Your Docker Nodes

## The Problem

Your Docker nodes are running old code → they rejected the ML-DSA transaction block → networks forked.

## The Solution (Choose One)

### Option A: Pull Pre-Built Image ⚡ **FASTEST** (2 minutes)

```bash
cd /path/to/boundless-git-collab
git pull origin main
./pull-docker-image.sh
```

### Option B: Build Locally 🔨 (15 minutes)

```bash
cd /path/to/boundless-git-collab
git pull origin main
./update-docker-nodes.sh
```

**Recommendation:** Use **Option A** if the Docker image is available on ghcr.io. Check with the team first!

---

## What It Does

1. ✅ Stops old containers
2. ✅ Clears forked blockchain data
3. ✅ Builds new image with ML-DSA support (~10-15 min)
4. ✅ Starts 3 updated nodes
5. ✅ Connects to bootstrap peer (70.32.195.180)
6. ✅ Syncs from genesis

---

## Expected Output

```
🐋 Boundless BLS - Docker Update Script
========================================

📦 Step 1/5: Stopping old containers...
✅ Containers stopped

🗑️  Step 2/5: Clearing old blockchain data (forked chain)...
✅ Old data cleared

🔨 Step 3/5: Building Docker image with ML-DSA support...
   (This takes 10-15 minutes - Docker is compiling Rust code)
✅ Image built

🚀 Step 4/5: Starting updated containers...
✅ All containers started

⏳ Step 5/5: Waiting for nodes to initialize (30 seconds)...

✅ Update complete!
```

---

## Verify It Worked

```bash
# Check all 3 containers are running
docker ps

# Check logs show connection to bootstrap
docker logs boundless-node1 | grep "Connected to bootstrap"

# Check blockchain height matches other node (~2015-2020)
curl http://localhost:3001/api/v1/chain/info | jq .height

# Watch real-time sync
docker logs -f boundless-node1
```

**Look for:**
- `✅ Connected to bootstrap peer`
- `📩 Received NewBlock from 12D3KooWDqN...` (other node's peer ID)
- **NO** `❌ Failed to add block: Transaction has no inputs`

---

## After Update

Once synced (1-5 minutes), all 4 nodes will be:
- ✅ On same blockchain (unified network)
- ✅ Supporting ML-DSA transactions
- ✅ Mining together
- ✅ Ready to test post-quantum transactions

---

## Troubleshooting

**"Docker not found"**
```bash
sudo systemctl start docker
```

**"Permission denied"**
```bash
sudo ./update-docker-nodes.sh
```

**"Can't pull origin"**
```bash
git remote -v  # Should show origin pointing to bls-zip repo
git pull  # Try without origin
```

**Containers won't start**
```bash
docker logs boundless-node1  # Check error messages
```

---

## Manual Alternative (If Script Fails)

If the automated script has issues, here's the manual process:

```bash
# 1. Stop old containers
docker stop boundless-node1 boundless-node2 boundless-node3
docker rm boundless-node1 boundless-node2 boundless-node3

# 2. Pull latest code
git pull origin main

# 3. Clear old data
rm -rf ./docker-data/node1/db/*
rm -rf ./docker-data/node2/db/*
rm -rf ./docker-data/node3/db/*

# 4. Rebuild image
docker build -t boundless-bls:latest .

# 5. Start containers (use commands from BRYAN_UPDATE_NODES.md)
```

---

## Timeline

- **Pull code:** 5 seconds
- **Run script:** 10-15 minutes (mostly Docker build)
- **Sync blockchain:** 1-5 minutes
- **Total:** ~20 minutes

---

**After this, we can test ML-DSA transactions on the unified 4-node network!** 🎉
