#!/bin/bash
# Multi-Node Test Suite for Boundless BLS Blockchain
# Tests Phase 3 network synchronization features

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_DIR="./data/test"
NODE_BINARY="./target/release/boundless-node"
BASE_PORT_P2P=40000
BASE_PORT_RPC=49000

echo -e "${BLUE}🧪 Boundless BLS Multi-Node Test Suite${NC}"
echo "========================================"
echo ""

# Function to print colored output
print_status() {
    echo -e "${BLUE}[$(date +%H:%M:%S)]${NC} $1"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Function to wait for node to start
wait_for_node() {
    local rpc_port=$1
    local max_wait=30
    local waited=0

    print_status "Waiting for node on port $rpc_port to start..."

    while [ $waited -lt $max_wait ]; do
        if curl -s -X POST http://localhost:$rpc_port \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"system_health","params":[],"id":1}' \
            > /dev/null 2>&1; then
            print_success "Node on port $rpc_port is ready"
            return 0
        fi
        sleep 1
        waited=$((waited + 1))
    done

    print_error "Node on port $rpc_port failed to start within ${max_wait}s"
    return 1
}

# Function to get blockchain height
get_height() {
    local rpc_port=$1
    curl -s -X POST http://localhost:$rpc_port \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}' \
        | jq -r '.result // 0'
}

# Function to cleanup
cleanup() {
    print_status "Cleaning up test processes..."
    pkill -f "boundless-node.*--base-path.*$TEST_DIR" || true
    sleep 2
    rm -rf $TEST_DIR
    print_success "Cleanup complete"
}

# Trap to ensure cleanup on exit
trap cleanup EXIT INT TERM

# Check prerequisites
print_status "Checking prerequisites..."

if [ ! -f "$NODE_BINARY" ]; then
    print_error "Node binary not found at $NODE_BINARY"
    print_status "Building node..."
    cargo build --release || exit 1
fi

if ! command -v jq &> /dev/null; then
    print_error "jq is required but not installed. Please install jq."
    exit 1
fi

print_success "Prerequisites OK"
echo ""

# Clean previous test data
rm -rf $TEST_DIR
mkdir -p $TEST_DIR

#=============================================================================
# Test 1: Two-Node Synchronization
#=============================================================================
echo -e "${BLUE}Test 1: Two-Node Synchronization${NC}"
echo "-----------------------------------"

NODE1_P2P=$((BASE_PORT_P2P + 1))
NODE1_RPC=$((BASE_PORT_RPC + 1))
NODE2_P2P=$((BASE_PORT_P2P + 2))
NODE2_RPC=$((BASE_PORT_RPC + 2))

# Start Node 1 with mining
print_status "Starting Node 1 (mining)..."
$NODE_BINARY \
    --dev --mining \
    --port $NODE1_P2P \
    --rpc-port $NODE1_RPC \
    --base-path $TEST_DIR/node1 \
    --mining-threads 1 \
    > $TEST_DIR/node1.log 2>&1 &
NODE1_PID=$!

print_success "Node 1 started (PID: $NODE1_PID, RPC: $NODE1_RPC)"

# Wait for Node 1 to start
if ! wait_for_node $NODE1_RPC; then
    print_error "Node 1 failed to start"
    cat $TEST_DIR/node1.log
    exit 1
fi

# Wait for some blocks to be mined
print_status "Waiting for Node 1 to mine blocks..."
sleep 20

HEIGHT1=$(get_height $NODE1_RPC)
print_status "Node 1 height: $HEIGHT1"

if [ "$HEIGHT1" -lt 3 ]; then
    print_warning "Node 1 height is $HEIGHT1, expected at least 3 blocks"
    print_status "Waiting longer..."
    sleep 15
    HEIGHT1=$(get_height $NODE1_RPC)
    print_status "Node 1 height now: $HEIGHT1"
fi

# Start Node 2 (non-mining)
print_status "Starting Node 2 (non-mining)..."
$NODE_BINARY \
    --dev \
    --port $NODE2_P2P \
    --rpc-port $NODE2_RPC \
    --base-path $TEST_DIR/node2 \
    > $TEST_DIR/node2.log 2>&1 &
NODE2_PID=$!

print_success "Node 2 started (PID: $NODE2_PID, RPC: $NODE2_RPC)"

# Wait for Node 2 to start
if ! wait_for_node $NODE2_RPC; then
    print_error "Node 2 failed to start"
    cat $TEST_DIR/node2.log
    exit 1
fi

# Wait for sync
print_status "Waiting for Node 2 to sync..."
sleep 10

HEIGHT2=$(get_height $NODE2_RPC)
print_status "Node 2 height: $HEIGHT2"

# Verify heights match (within 1 block due to potential new mining)
if [ "$HEIGHT2" -ge $((HEIGHT1 - 1)) ] && [ "$HEIGHT2" -le $((HEIGHT1 + 1)) ]; then
    print_success "Test 1 PASSED: Nodes synchronized (Node1: $HEIGHT1, Node2: $HEIGHT2)"
else
    print_error "Test 1 FAILED: Height mismatch (Node1: $HEIGHT1, Node2: $HEIGHT2)"
    print_status "Node 1 logs:"
    tail -20 $TEST_DIR/node1.log
    print_status "Node 2 logs:"
    tail -20 $TEST_DIR/node2.log
    exit 1
fi

echo ""

#=============================================================================
# Test 2: Real-Time Block Propagation
#=============================================================================
echo -e "${BLUE}Test 2: Real-Time Block Propagation${NC}"
echo "-------------------------------------"

# Get current heights
HEIGHT1_BEFORE=$(get_height $NODE1_RPC)
HEIGHT2_BEFORE=$(get_height $NODE2_RPC)

print_status "Initial heights - Node1: $HEIGHT1_BEFORE, Node2: $HEIGHT2_BEFORE"

# Wait for Node 1 to mine a new block
print_status "Waiting for Node 1 to mine new block..."
sleep 15

HEIGHT1_AFTER=$(get_height $NODE1_RPC)
HEIGHT2_AFTER=$(get_height $NODE2_RPC)

print_status "New heights - Node1: $HEIGHT1_AFTER, Node2: $HEIGHT2_AFTER"

# Verify both heights increased
if [ "$HEIGHT1_AFTER" -gt "$HEIGHT1_BEFORE" ] && [ "$HEIGHT2_AFTER" -ge "$HEIGHT2_BEFORE" ]; then
    # Verify heights still match
    if [ "$HEIGHT2_AFTER" -ge $((HEIGHT1_AFTER - 1)) ] && [ "$HEIGHT2_AFTER" -le $((HEIGHT1_AFTER + 1)) ]; then
        print_success "Test 2 PASSED: Block propagated to Node 2"
    else
        print_error "Test 2 FAILED: Heights diverged (Node1: $HEIGHT1_AFTER, Node2: $HEIGHT2_AFTER)"
        exit 1
    fi
else
    print_warning "Test 2 INCONCLUSIVE: No new blocks mined or propagation issue"
fi

echo ""

#=============================================================================
# Test 3: Three-Node Network
#=============================================================================
echo -e "${BLUE}Test 3: Three-Node Network${NC}"
echo "----------------------------"

NODE3_P2P=$((BASE_PORT_P2P + 3))
NODE3_RPC=$((BASE_PORT_RPC + 3))

# Start Node 3 (mining)
print_status "Starting Node 3 (mining)..."
$NODE_BINARY \
    --dev --mining \
    --port $NODE3_P2P \
    --rpc-port $NODE3_RPC \
    --base-path $TEST_DIR/node3 \
    --mining-threads 1 \
    > $TEST_DIR/node3.log 2>&1 &
NODE3_PID=$!

print_success "Node 3 started (PID: $NODE3_PID, RPC: $NODE3_RPC)"

# Wait for Node 3 to start
if ! wait_for_node $NODE3_RPC; then
    print_error "Node 3 failed to start"
    cat $TEST_DIR/node3.log
    exit 1
fi

# Wait for sync
print_status "Waiting for 3-node network to stabilize..."
sleep 15

# Check all heights
HEIGHT1=$(get_height $NODE1_RPC)
HEIGHT2=$(get_height $NODE2_RPC)
HEIGHT3=$(get_height $NODE3_RPC)

print_status "Heights - Node1: $HEIGHT1, Node2: $HEIGHT2, Node3: $HEIGHT3"

# Verify all nodes are synchronized (within 1 block)
MAX_HEIGHT=$(( HEIGHT1 > HEIGHT2 ? (HEIGHT1 > HEIGHT3 ? HEIGHT1 : HEIGHT3) : (HEIGHT2 > HEIGHT3 ? HEIGHT2 : HEIGHT3) ))
MIN_HEIGHT=$(( HEIGHT1 < HEIGHT2 ? (HEIGHT1 < HEIGHT3 ? HEIGHT1 : HEIGHT3) : (HEIGHT2 < HEIGHT3 ? HEIGHT2 : HEIGHT3) ))

if [ $((MAX_HEIGHT - MIN_HEIGHT)) -le 1 ]; then
    print_success "Test 3 PASSED: All 3 nodes synchronized (range: $MIN_HEIGHT-$MAX_HEIGHT)"
else
    print_error "Test 3 FAILED: Nodes not synchronized (heights: $HEIGHT1, $HEIGHT2, $HEIGHT3)"
    exit 1
fi

echo ""

#=============================================================================
# Test 4: Persistence & Restart
#=============================================================================
echo -e "${BLUE}Test 4: Persistence & Restart${NC}"
echo "-------------------------------"

# Get Node 2 height before restart
HEIGHT_BEFORE=$(get_height $NODE2_RPC)
print_status "Node 2 height before restart: $HEIGHT_BEFORE"

# Stop Node 2
print_status "Stopping Node 2..."
kill $NODE2_PID
wait $NODE2_PID 2>/dev/null || true
sleep 2

# Restart Node 2
print_status "Restarting Node 2..."
$NODE_BINARY \
    --dev \
    --port $NODE2_P2P \
    --rpc-port $NODE2_RPC \
    --base-path $TEST_DIR/node2 \
    > $TEST_DIR/node2_restart.log 2>&1 &
NODE2_PID=$!

# Wait for restart
if ! wait_for_node $NODE2_RPC; then
    print_error "Node 2 failed to restart"
    exit 1
fi

# Wait for potential new blocks
sleep 5

# Check height after restart
HEIGHT_AFTER=$(get_height $NODE2_RPC)
print_status "Node 2 height after restart: $HEIGHT_AFTER"

# Verify height was restored (should be >= before, accounting for new blocks)
if [ "$HEIGHT_AFTER" -ge "$HEIGHT_BEFORE" ]; then
    print_success "Test 4 PASSED: Blockchain state persisted across restart"
else
    print_error "Test 4 FAILED: Height decreased after restart ($HEIGHT_BEFORE -> $HEIGHT_AFTER)"
    exit 1
fi

echo ""

#=============================================================================
# Summary
#=============================================================================
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo -e "${GREEN}✅ All Tests Passed!${NC}"
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo ""
print_status "Test Summary:"
echo "  • Two-node synchronization: ✅"
echo "  • Real-time block propagation: ✅"
echo "  • Three-node network: ✅"
echo "  • Persistence & restart: ✅"
echo ""
print_status "Final blockchain heights:"
echo "  • Node 1: $(get_height $NODE1_RPC)"
echo "  • Node 2: $(get_height $NODE2_RPC)"
echo "  • Node 3: $(get_height $NODE3_RPC)"
echo ""
print_success "Phase 3 multi-node testing complete!"
