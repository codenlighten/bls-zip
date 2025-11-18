#!/bin/bash
# Stop Boundless BLS Main Node

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

DATA_DIR="$HOME/boundless-data"
PID_FILE="$DATA_DIR/node.pid"

if [ ! -f "$PID_FILE" ]; then
    echo -e "${YELLOW}⚠️  No PID file found${NC}"
    echo "Node may not be running"
    exit 1
fi

PID=$(cat "$PID_FILE")

if ps -p "$PID" > /dev/null 2>&1; then
    echo "🛑 Stopping node (PID: $PID)..."
    kill -TERM "$PID"
    
    # Wait for graceful shutdown
    for i in {1..30}; do
        if ! ps -p "$PID" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Node stopped successfully${NC}"
            rm "$PID_FILE"
            exit 0
        fi
        sleep 1
    done
    
    # Force kill if still running
    echo "⚠️  Forcing shutdown..."
    kill -9 "$PID" 2>/dev/null || true
    rm "$PID_FILE"
    echo -e "${GREEN}✅ Node stopped (forced)${NC}"
else
    echo -e "${YELLOW}⚠️  Node not running (stale PID file)${NC}"
    rm "$PID_FILE"
fi
