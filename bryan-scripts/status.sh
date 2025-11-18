#!/bin/bash
# Monitor Boundless BLS Node Status

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

DATA_DIR="$HOME/boundless-data"
PID_FILE="$DATA_DIR/node.pid"
RPC_PORT="9933"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Boundless BLS Node Status${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if running
if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if ps -p "$PID" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Node Status: RUNNING${NC}"
        echo "  PID: $PID"
        
        # Get CPU and memory usage
        CPU=$(ps -p "$PID" -o %cpu= | xargs)
        MEM=$(ps -p "$PID" -o %mem= | xargs)
        echo "  CPU: ${CPU}%"
        echo "  Memory: ${MEM}%"
        echo ""
    else
        echo -e "${RED}❌ Node Status: NOT RUNNING (stale PID)${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ Node Status: NOT RUNNING${NC}"
    exit 1
fi

# Query blockchain via RPC
echo -e "${BLUE}Blockchain Status:${NC}"

if command -v curl &> /dev/null; then
    # Get block height
    HEIGHT=$(curl -s -X POST http://localhost:$RPC_PORT \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}' \
        2>/dev/null | jq -r '.result // "N/A"' 2>/dev/null || echo "N/A")
    
    echo "  Block Height: $HEIGHT"
    
    # Get best block hash
    HASH=$(curl -s -X POST http://localhost:$RPC_PORT \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"chain_getBestBlockHash","params":[],"id":1}' \
        2>/dev/null | jq -r '.result // "N/A"' 2>/dev/null | cut -c1-16 || echo "N/A")
    
    if [ "$HASH" != "N/A" ]; then
        echo "  Best Block: ${HASH}..."
    fi
    
    # Get total supply
    SUPPLY=$(curl -s -X POST http://localhost:$RPC_PORT \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"chain_getTotalSupply","params":[],"id":1}' \
        2>/dev/null | jq -r '.result // "N/A"' 2>/dev/null || echo "N/A")
    
    echo "  Total Supply: $SUPPLY"
else
    echo "  (Install curl and jq for blockchain stats)"
fi

echo ""

# Network status
echo -e "${BLUE}Network Status:${NC}"
CONNECTIONS=$(ss -tn | grep ":30333" | grep ESTAB | wc -l)
echo "  P2P Connections: $CONNECTIONS"

# Disk usage
echo ""
echo -e "${BLUE}Disk Usage:${NC}"
if [ -d "$DATA_DIR/db" ]; then
    DB_SIZE=$(du -sh "$DATA_DIR/db" 2>/dev/null | cut -f1 || echo "N/A")
    echo "  Database Size: $DB_SIZE"
fi

AVAILABLE=$(df -h "$DATA_DIR" | awk 'NR==2 {print $4}')
echo "  Available Space: $AVAILABLE"

echo ""
