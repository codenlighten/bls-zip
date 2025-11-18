#!/bin/bash
# Stop the Boundless BLS production node

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PID_FILE="$SCRIPT_DIR/node.pid"

if [ ! -f "$PID_FILE" ]; then
    echo -e "${YELLOW}⚠️  No PID file found. Node may not be running.${NC}"
    exit 1
fi

PID=$(cat "$PID_FILE")

if ps -p "$PID" > /dev/null 2>&1; then
    echo "🛑 Stopping node (PID: $PID)..."
    kill -TERM "$PID"
    
    # Wait for graceful shutdown
    for i in {1..10}; do
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
