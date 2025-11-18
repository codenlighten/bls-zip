#!/bin/bash
#############################################################################
# Start Boundless BLS Main Node
# Optimized for Bryan's high-performance server
#############################################################################

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
INSTALL_DIR="$HOME/boundless-bls-main"
DATA_DIR="$HOME/boundless-data"
LOG_DIR="$DATA_DIR/logs"
NODE_BINARY="$INSTALL_DIR/target/release/boundless-node"
CONFIG_FILE="$DATA_DIR/config.toml"
PID_FILE="$DATA_DIR/node.pid"
LOG_FILE="$LOG_DIR/node-$(date +%Y%m%d-%H%M%S).log"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Starting Boundless BLS Main Node${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if already running
if [ -f "$PID_FILE" ]; then
    OLD_PID=$(cat "$PID_FILE")
    if ps -p "$OLD_PID" > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Node already running (PID: $OLD_PID)${NC}"
        echo "To restart, first run: ./stop-node.sh"
        exit 1
    else
        rm "$PID_FILE"
    fi
fi

# Verify binary exists
if [ ! -f "$NODE_BINARY" ]; then
    echo -e "${YELLOW}❌ Node binary not found at $NODE_BINARY${NC}"
    echo "Run deployment script first: bash deploy-bryan-main-node.sh"
    exit 1
fi

# Verify config exists
if [ ! -f "$CONFIG_FILE" ]; then
    echo -e "${YELLOW}❌ Configuration not found at $CONFIG_FILE${NC}"
    echo "Run deployment script first: bash deploy-bryan-main-node.sh"
    exit 1
fi

# Create log directory
mkdir -p "$LOG_DIR"

# Get system info
CPU_CORES=$(nproc)
TOTAL_RAM=$(free -h | awk '/^Mem:/ {print $2}')
IP_ADDR=$(hostname -I | awk '{print $1}')

echo -e "${GREEN}System Information:${NC}"
echo "  CPU Cores: $CPU_CORES"
echo "  Total RAM: $TOTAL_RAM"
echo "  Local IP: $IP_ADDR"
echo ""

echo -e "${GREEN}Starting node...${NC}"
echo "  Binary: $NODE_BINARY"
echo "  Config: $CONFIG_FILE"
echo "  Log: $LOG_FILE"
echo ""

# Start node in background
nohup "$NODE_BINARY" \
    --config "$CONFIG_FILE" \
    --mining \
    --mining-threads 0 \
    > "$LOG_FILE" 2>&1 &

NODE_PID=$!
echo $NODE_PID > "$PID_FILE"

# Wait for startup
sleep 3

# Check if still running
if ps -p "$NODE_PID" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Node started successfully!${NC}"
    echo ""
    echo -e "${BLUE}Node Information:${NC}"
    echo "  PID: $NODE_PID"
    echo "  Log: $LOG_FILE"
    echo ""
    
    # Try to get peer ID from logs
    sleep 2
    PEER_ID=$(grep -m 1 "Local PeerId" "$LOG_FILE" 2>/dev/null | awk '{print $NF}' || echo "")
    if [ -n "$PEER_ID" ]; then
        echo -e "${GREEN}🔑 Peer ID: $PEER_ID${NC}"
        echo ""
        echo -e "${BLUE}Share this with others to connect:${NC}"
        echo "  /ip4/$IP_ADDR/tcp/30333/p2p/$PEER_ID"
        echo ""
    fi
    
    echo -e "${BLUE}Quick Commands:${NC}"
    echo "  View logs:  tail -f $LOG_FILE"
    echo "  Stop node:  ./stop-node.sh"
    echo "  Node status: ps -p $NODE_PID"
    echo ""
    
else
    echo -e "${YELLOW}❌ Node failed to start${NC}"
    echo "Check log: cat $LOG_FILE"
    rm "$PID_FILE"
    exit 1
fi
