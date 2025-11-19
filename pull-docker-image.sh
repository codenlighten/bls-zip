#!/bin/bash
# Quick Docker Pull & Update Script for Bryan
# Pulls pre-built ML-DSA Docker image from GitHub Container Registry

set -e  # Exit on error

echo "🐋 Boundless BLS - Quick Docker Pull & Update"
echo "=============================================="
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Error: Docker is not running. Please start Docker first."
    exit 1
fi

echo "📦 Step 1/5: Stopping old containers..."
docker stop boundless-node1 boundless-node2 boundless-node3 2>/dev/null || true
docker rm boundless-node1 boundless-node2 boundless-node3 2>/dev/null || true
echo "✅ Containers stopped"
echo ""

echo "🗑️  Step 2/5: Clearing old blockchain data (forked chain)..."
rm -rf ./docker-data/node1/db/* 2>/dev/null || true
rm -rf ./docker-data/node2/db/* 2>/dev/null || true
rm -rf ./docker-data/node3/db/* 2>/dev/null || true
echo "✅ Old data cleared"
echo ""

echo "⬇️  Step 3/5: Pulling pre-built Docker image from GitHub..."
echo "   (This takes 1-2 minutes - much faster than building!)"
docker pull ghcr.io/codenlighten/boundless-bls:latest

# Tag it locally for convenience
docker tag ghcr.io/codenlighten/boundless-bls:latest boundless-bls:latest

echo "✅ Image pulled"
echo ""

echo "🚀 Step 4/5: Starting updated containers..."

# Create data directories if they don't exist
mkdir -p ./docker-data/node1 ./docker-data/node2 ./docker-data/node3

# Node 1 - Main node with all ports exposed
docker run -d \
  --name boundless-node1 \
  --restart unless-stopped \
  -p 30333:30333 \
  -p 9933:9933 \
  -p 3001:3001 \
  -v "$(pwd)/docker-data/node1:/data" \
  boundless-bls:latest \
  --base-path /data \
  --chain testnet \
  --name "BryanNode1" \
  --bootnodes "/ip4/70.32.195.180/tcp/30333/p2p/12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r" \
  --mining \
  --mining-threads 0 \
  --http-port 3001 \
  --rpc-external \
  --rpc-cors all

sleep 2

# Node 2
docker run -d \
  --name boundless-node2 \
  --restart unless-stopped \
  -p 30334:30333 \
  -p 9934:9933 \
  -p 3002:3001 \
  -v "$(pwd)/docker-data/node2:/data" \
  boundless-bls:latest \
  --base-path /data \
  --chain testnet \
  --name "BryanNode2" \
  --bootnodes "/ip4/70.32.195.180/tcp/30333/p2p/12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r" \
  --mining \
  --mining-threads 0 \
  --http-port 3001 \
  --rpc-external \
  --rpc-cors all

sleep 2

# Node 3
docker run -d \
  --name boundless-node3 \
  --restart unless-stopped \
  -p 30335:30333 \
  -p 9935:9933 \
  -p 3003:3001 \
  -v "$(pwd)/docker-data/node3:/data" \
  boundless-bls:latest \
  --base-path /data \
  --chain testnet \
  --name "BryanNode3" \
  --bootnodes "/ip4/70.32.195.180/tcp/30333/p2p/12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r" \
  --mining \
  --mining-threads 0 \
  --http-port 3001 \
  --rpc-external \
  --rpc-cors all

echo "✅ All containers started"
echo ""

echo "⏳ Step 5/5: Waiting for nodes to initialize (30 seconds)..."
sleep 30
echo ""

echo "📊 Checking node status..."
echo ""
echo "Node 1 Status:"
docker logs boundless-node1 --tail 5
echo ""
echo "Node 2 Status:"
docker logs boundless-node2 --tail 5
echo ""
echo "Node 3 Status:"
docker logs boundless-node3 --tail 5
echo ""

echo "✅ Update complete!"
echo ""
echo "📝 Next steps:"
echo "   1. Check sync status: docker logs boundless-node1 --tail 20"
echo "   2. Monitor blockchain: curl http://localhost:3001/api/v1/chain/info | jq"
echo "   3. Watch all logs: docker logs -f boundless-node1"
echo ""
echo "🔍 Verify connection to bootstrap peer:"
echo "   docker logs boundless-node1 | grep 'Connected to bootstrap'"
echo ""
echo "✨ All 4 nodes should now be on the same chain with ML-DSA support!"
echo ""
echo "⏱️  Total time: ~2 minutes (vs 15 minutes building locally!)"
