#!/bin/bash
# Boundless BLS Node - Production Deployment Script
# Run this script to start the first production node

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Boundless BLS Production Node${NC}"
echo -e "${BLUE}  First Network Seed Node${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Paths
NODE_BINARY="$PROJECT_ROOT/target/release/boundless-node"
CONFIG_FILE="$SCRIPT_DIR/config.toml"
DATA_DIR="$SCRIPT_DIR/data"
LOG_DIR="$SCRIPT_DIR/logs"
PID_FILE="$SCRIPT_DIR/node.pid"

# Create directories
mkdir -p "$DATA_DIR"
mkdir -p "$LOG_DIR"

# Check if node binary exists
if [ ! -f "$NODE_BINARY" ]; then
    echo -e "${YELLOW}⚠️  Node binary not found. Building...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --release --bin boundless-node
    echo -e "${GREEN}✅ Build complete${NC}"
fi

# Check if node is already running
if [ -f "$PID_FILE" ]; then
    OLD_PID=$(cat "$PID_FILE")
    if ps -p "$OLD_PID" > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Node is already running (PID: $OLD_PID)${NC}"
        echo "Stop it first with: ./stop-node.sh"
        exit 1
    else
        # Stale PID file
        rm "$PID_FILE"
    fi
fi

# Get network interface info
echo -e "${BLUE}📡 Network Information:${NC}"
IP_ADDR=$(hostname -I | awk '{print $1}')
echo "   Local IP: $IP_ADDR"
echo "   P2P Port: 30333"
echo "   RPC Port: 9933 (HTTP)"
echo "   WS Port:  9944 (WebSocket)"
echo "   Metrics:  9615 (Prometheus)"
echo ""

# Start the node
echo -e "${GREEN}🚀 Starting Boundless BLS Node...${NC}"
echo "   Config: $CONFIG_FILE"
echo "   Data:   $DATA_DIR"
echo "   Logs:   $LOG_DIR"
echo ""

# Run node in background with logging
LOG_FILE="$LOG_DIR/node-$(date +%Y%m%d-%H%M%S).log"

nohup "$NODE_BINARY" \
    --config "$CONFIG_FILE" \
    --mining \
    --mining-threads 0 \
    > "$LOG_FILE" 2>&1 &

NODE_PID=$!
echo $NODE_PID > "$PID_FILE"

# Wait a moment for startup
sleep 2

# Check if still running
if ps -p "$NODE_PID" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Node started successfully!${NC}"
    echo ""
    echo -e "${BLUE}Node Information:${NC}"
    echo "   PID: $NODE_PID"
    echo "   Log: $LOG_FILE"
    echo ""
    echo -e "${BLUE}Quick Commands:${NC}"
    echo "   View logs:  tail -f $LOG_FILE"
    echo "   Stop node:  ./production-node/stop-node.sh"
    echo "   Check RPC:  curl http://localhost:9933"
    echo ""
    echo -e "${YELLOW}📝 To get your peer ID for Bryan:${NC}"
    echo "   grep 'Local peer id' $LOG_FILE"
    echo ""
    
    # Try to extract peer ID (wait a few more seconds for it to appear)
    sleep 3
    PEER_ID=$(grep -m 1 "Local peer id" "$LOG_FILE" 2>/dev/null | awk '{print $NF}' || echo "")
    if [ -n "$PEER_ID" ]; then
        echo -e "${GREEN}🔑 Your Peer ID: $PEER_ID${NC}"
        echo ""
        echo -e "${BLUE}Bryan should add this bootnode to his config:${NC}"
        echo "   /ip4/$IP_ADDR/tcp/30333/p2p/$PEER_ID"
        echo ""
        
        # Save to file for Bryan
        cat > "$SCRIPT_DIR/BOOTNODE_INFO.txt" << EOF
Boundless BLS Bootnode Information
===================================

Primary Node (First Seed)
-------------------------
Peer ID: $PEER_ID
IP Address: $IP_ADDR
P2P Port: 30333

Multiaddr (add to Bryan's config.toml):
/ip4/$IP_ADDR/tcp/30333/p2p/$PEER_ID

RPC Endpoints:
- HTTP: http://$IP_ADDR:9933
- WebSocket: ws://$IP_ADDR:9944
- Metrics: http://$IP_ADDR:9615/metrics

Genesis Block:
Check logs for genesis hash - both nodes must use same genesis

Started: $(date)
EOF
        echo -e "${GREEN}✅ Bootnode info saved to: $SCRIPT_DIR/BOOTNODE_INFO.txt${NC}"
    fi
    
else
    echo -e "${YELLOW}⚠️  Node may have failed to start${NC}"
    echo "Check log file: $LOG_FILE"
    rm "$PID_FILE"
    exit 1
fi
