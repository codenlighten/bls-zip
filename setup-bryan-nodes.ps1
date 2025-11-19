# Boundless BLS - Complete Node Setup Script for Windows
# This script will:
# 1. Stop old containers
# 2. Pull latest Docker image
# 3. Clear all blockchain data
# 4. Start 3 nodes that sync from bootstrap
# 5. Generate a wallet
# 6. Show status

Write-Host "🚀 Boundless BLS - Complete Node Setup" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# Configuration
$BOOTSTRAP_PEER = "/ip4/70.32.195.180/tcp/30333/p2p/12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r"
$DOCKER_IMAGE = "ghcr.io/codenlighten/boundless-bls:latest"

# Step 1: Stop and remove old containers
Write-Host "📦 Step 1/6: Stopping old containers..." -ForegroundColor Yellow
docker stop boundless-node1 boundless-node2 boundless-node3 2>$null
docker rm boundless-node1 boundless-node2 boundless-node3 2>$null
Write-Host "✅ Old containers removed" -ForegroundColor Green
Write-Host ""

# Step 2: Pull latest Docker image
Write-Host "⬇️  Step 2/6: Pulling latest Docker image..." -ForegroundColor Yellow
Write-Host "   (This takes 1-2 minutes)" -ForegroundColor Gray
docker pull $DOCKER_IMAGE
Write-Host "✅ Image pulled" -ForegroundColor Green
Write-Host ""

# Step 3: Clear all blockchain data
Write-Host "🗑️  Step 3/6: Clearing old blockchain data..." -ForegroundColor Yellow
if (Test-Path ".\docker-data\node1") {
    Remove-Item -Recurse -Force .\docker-data\node1\* -ErrorAction SilentlyContinue
}
if (Test-Path ".\docker-data\node2") {
    Remove-Item -Recurse -Force .\docker-data\node2\* -ErrorAction SilentlyContinue
}
if (Test-Path ".\docker-data\node3") {
    Remove-Item -Recurse -Force .\docker-data\node3\* -ErrorAction SilentlyContinue
}
Write-Host "✅ Data cleared" -ForegroundColor Green
Write-Host ""

# Step 4: Create data directories
Write-Host "📁 Step 4/6: Creating data directories..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path .\docker-data\node1 | Out-Null
New-Item -ItemType Directory -Force -Path .\docker-data\node2 | Out-Null
New-Item -ItemType Directory -Force -Path .\docker-data\node3 | Out-Null
Write-Host "✅ Directories created" -ForegroundColor Green
Write-Host ""

# Step 5: Start containers
Write-Host "🚀 Step 5/6: Starting containers..." -ForegroundColor Yellow

# Node 1
Write-Host "   Starting Node 1..." -ForegroundColor Gray
docker run -d `
  --name boundless-node1 `
  --restart unless-stopped `
  -p 30333:30333 `
  -p 9933:9933 `
  -p 3001:3001 `
  -v ${PWD}\docker-data\node1:/data `
  $DOCKER_IMAGE `
  --base-path /data `
  --chain testnet `
  --name "BryanNode1" `
  --bootnodes $BOOTSTRAP_PEER `
  --mining `
  --mining-threads 0 `
  --http-port 3001 `
  --rpc-external `
  --rpc-cors all

Start-Sleep -Seconds 2

# Node 2
Write-Host "   Starting Node 2..." -ForegroundColor Gray
docker run -d `
  --name boundless-node2 `
  --restart unless-stopped `
  -p 30334:30333 `
  -p 9934:9933 `
  -p 3002:3001 `
  -v ${PWD}\docker-data\node2:/data `
  $DOCKER_IMAGE `
  --base-path /data `
  --chain testnet `
  --name "BryanNode2" `
  --bootnodes $BOOTSTRAP_PEER `
  --mining `
  --mining-threads 0 `
  --http-port 3001 `
  --rpc-external `
  --rpc-cors all

Start-Sleep -Seconds 2

# Node 3
Write-Host "   Starting Node 3..." -ForegroundColor Gray
docker run -d `
  --name boundless-node3 `
  --restart unless-stopped `
  -p 30335:30333 `
  -p 9935:9933 `
  -p 3003:3001 `
  -v ${PWD}\docker-data\node3:/data `
  $DOCKER_IMAGE `
  --base-path /data `
  --chain testnet `
  --name "BryanNode3" `
  --bootnodes $BOOTSTRAP_PEER `
  --mining `
  --mining-threads 0 `
  --http-port 3001 `
  --rpc-external `
  --rpc-cors all

Write-Host "✅ All containers started" -ForegroundColor Green
Write-Host ""

# Step 6: Wait for nodes to initialize
Write-Host "⏳ Step 6/6: Waiting for nodes to initialize (30 seconds)..." -ForegroundColor Yellow
Start-Sleep -Seconds 30
Write-Host ""

# Generate wallet
Write-Host "🔑 Generating ML-DSA wallet..." -ForegroundColor Cyan
docker exec boundless-node1 boundless-cli keygen --algorithm ml-dsa --output /data/bryan-wallet

Write-Host ""
Write-Host "📋 Copying wallet files to current directory..." -ForegroundColor Cyan
docker cp boundless-node1:/data/bryan-wallet.priv .\bryan-wallet.priv
docker cp boundless-node1:/data/bryan-wallet.pub .\bryan-wallet.pub
Write-Host "✅ Wallet files saved: bryan-wallet.priv, bryan-wallet.pub" -ForegroundColor Green
Write-Host ""

# Show status
Write-Host "📊 Node Status:" -ForegroundColor Cyan
Write-Host "===============" -ForegroundColor Cyan
docker ps --filter "name=boundless-node" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
Write-Host ""

Write-Host "📝 Quick Commands:" -ForegroundColor Cyan
Write-Host "==================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Check logs:" -ForegroundColor White
Write-Host "  docker logs boundless-node1 --tail 20" -ForegroundColor Gray
Write-Host ""
Write-Host "Check sync status:" -ForegroundColor White
Write-Host "  docker exec boundless-node1 curl http://localhost:3001/api/v1/chain/info" -ForegroundColor Gray
Write-Host ""
Write-Host "Check your wallet balance:" -ForegroundColor White
Write-Host "  docker exec boundless-node1 curl http://localhost:3001/api/v1/balance/YOUR_ADDRESS" -ForegroundColor Gray
Write-Host ""
Write-Host "Send transaction (1 BLS = 100000000):" -ForegroundColor White
Write-Host "  docker exec boundless-node1 boundless-cli send RECIPIENT_ADDR 100000000 --key /data/bryan-wallet.priv" -ForegroundColor Gray
Write-Host ""

Write-Host "✅ Setup Complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Your nodes are now syncing from the bootstrap peer." -ForegroundColor Cyan
Write-Host "Check logs to verify: docker logs -f boundless-node1" -ForegroundColor Cyan
Write-Host ""
Write-Host "Look for:" -ForegroundColor Yellow
Write-Host "  ✅ Connected to bootstrap peer" -ForegroundColor Gray
Write-Host "  📩 Received NewBlock from peer" -ForegroundColor Gray
Write-Host "  🆕 Received new block #XXX" -ForegroundColor Gray
Write-Host ""
Write-Host "Once synced, you'll see mining activity!" -ForegroundColor Green
