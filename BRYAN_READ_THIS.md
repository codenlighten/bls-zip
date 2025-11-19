# 🚀 Bryan - Your Nodes Need an Update!

## The Issue

Your 3 Docker nodes rejected a block with an ML-DSA transaction. The networks have forked (you're on block 2031+, other node is on 2015).

## The Fix (2 Minutes Total!)

```bash
cd /path/to/boundless-git-collab
git pull origin main
./pull-docker-image.sh
```

**That's it!** The script:
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

```bash
# Check containers running
docker ps

# Check sync status
docker logs boundless-node1 | grep "Received new block"

# Check block height matches other node (~2015-2020)
curl http://localhost:3001/api/v1/chain/info | jq .height
```

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
