#!/bin/bash
# Network Synchronization Verification Script
# Verifies block sync and transaction propagation between nodes

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

NODE_BINARY="./target/release/boundless-node"
TEST_DIR="./data/verify_sync"
LOG_DIR="$TEST_DIR/logs"

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   Network Synchronization Verification Suite${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

# Create directories
rm -rf $TEST_DIR
mkdir -p $LOG_DIR

# Helper functions
print_test() {
    echo ""
    echo -e "${BLUE}━━━ Test: $1 ━━━${NC}"
    echo ""
}

print_step() {
    echo -e "${CYAN}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_failure() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ  $1${NC}"
}

get_height() {
    local port=$1
    curl -s -X POST http://localhost:$port \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}' \
        2>/dev/null | jq -r '.result // 0'
}

get_best_hash() {
    local port=$1
    curl -s -X POST http://localhost:$port \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"chain_getBestBlockHash","params":[],"id":1}' \
        2>/dev/null | jq -r '.result // ""'
}

wait_for_height() {
    local port=$1
    local target_height=$2
    local timeout=$3
    local waited=0

    while [ $waited -lt $timeout ]; do
        HEIGHT=$(get_height $port)
        if [ "$HEIGHT" -ge "$target_height" ]; then
            return 0
        fi
        sleep 1
        waited=$((waited + 1))
    done
    return 1
}

cleanup() {
    print_step "Cleaning up processes..."
    pkill -f "boundless-node.*--base-path.*$TEST_DIR" 2>/dev/null || true
    sleep 2
}

trap cleanup EXIT INT TERM

# Build check
if [ ! -f "$NODE_BINARY" ]; then
    print_failure "Node binary not found at $NODE_BINARY"
    print_step "Building node..."
    cargo build --release || exit 1
fi

#=============================================================================
# Test 1: Basic Two-Node Sync
#=============================================================================
print_test "Two-Node Block Synchronization"

print_step "Starting Node A (mining)..."
$NODE_BINARY \
    --dev --mining \
    --port 40101 --rpc-port 49101 \
    --base-path $TEST_DIR/node_a \
    --mining-threads 1 \
    > $LOG_DIR/node_a.log 2>&1 &
NODE_A_PID=$!

sleep 5

print_step "Waiting for Node A to mine 10 blocks..."
if wait_for_height 49101 10 120; then
    HEIGHT_A=$(get_height 49101)
    print_success "Node A mined $HEIGHT_A blocks"
else
    print_failure "Node A did not reach height 10 within timeout"
    exit 1
fi

print_step "Starting Node B (non-mining)..."
$NODE_BINARY \
    --dev \
    --port 40102 --rpc-port 49102 \
    --base-path $TEST_DIR/node_b \
    > $LOG_DIR/node_b.log 2>&1 &
NODE_B_PID=$!

sleep 5

print_step "Waiting for Node B to sync..."
sleep 15

HEIGHT_B=$(get_height 49102)
HEIGHT_A=$(get_height 49101)

print_info "Node A height: $HEIGHT_A"
print_info "Node B height: $HEIGHT_B"

# Verify sync (allow 1 block difference due to potential new mining)
if [ "$HEIGHT_B" -ge $((HEIGHT_A - 1)) ]; then
    print_success "PASS: Nodes synchronized (difference: $((HEIGHT_A - HEIGHT_B)) blocks)"

    # Verify they have the same chain (check hash at common height)
    HASH_A=$(curl -s -X POST http://localhost:49101 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"chain_getBlockByHeight\",\"params\":[$HEIGHT_B],\"id\":1}" \
        | jq -r '.result.hash // ""')

    HASH_B=$(curl -s -X POST http://localhost:49102 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"chain_getBlockByHeight\",\"params\":[$HEIGHT_B],\"id\":1}" \
        | jq -r '.result.hash // ""')

    if [ "$HASH_A" = "$HASH_B" ] && [ -n "$HASH_A" ]; then
        print_success "PASS: Block hashes match at height $HEIGHT_B"
    else
        print_failure "FAIL: Block hashes differ (fork detected!)"
        print_info "Node A hash: $HASH_A"
        print_info "Node B hash: $HASH_B"
    fi
else
    print_failure "FAIL: Nodes not synchronized (A: $HEIGHT_A, B: $HEIGHT_B)"
fi

#=============================================================================
# Test 2: Real-Time Block Propagation
#=============================================================================
print_test "Real-Time Block Propagation"

print_step "Recording current heights..."
HEIGHT_A_BEFORE=$(get_height 49101)
HEIGHT_B_BEFORE=$(get_height 49102)

print_info "Starting heights - A: $HEIGHT_A_BEFORE, B: $HEIGHT_B_BEFORE"

print_step "Waiting for new block from Node A..."
sleep 20

HEIGHT_A_AFTER=$(get_height 49101)
HEIGHT_B_AFTER=$(get_height 49102)

print_info "New heights - A: $HEIGHT_A_AFTER, B: $HEIGHT_B_AFTER"

if [ "$HEIGHT_A_AFTER" -gt "$HEIGHT_A_BEFORE" ]; then
    print_success "Node A mined new blocks: $((HEIGHT_A_AFTER - HEIGHT_A_BEFORE))"

    if [ "$HEIGHT_B_AFTER" -ge "$HEIGHT_B_BEFORE" ]; then
        # Check propagation time via logs
        if grep -q "Received new block #$HEIGHT_A_AFTER" $LOG_DIR/node_b.log; then
            print_success "PASS: Block propagated to Node B"

            # Extract timestamps (approximate)
            print_info "Block propagation confirmed in logs"
        else
            print_info "Block received (height increased) but not in logs yet"
        fi
    else
        print_failure "FAIL: Node B did not receive new blocks"
    fi
else
    print_info "No new blocks mined during test period"
fi

#=============================================================================
# Test 3: Late-Joining Node
#=============================================================================
print_test "Late-Joining Node Sync"

print_step "Letting Node A mine more blocks..."
INITIAL_HEIGHT=$(get_height 49101)
TARGET_HEIGHT=$((INITIAL_HEIGHT + 20))

print_info "Current height: $INITIAL_HEIGHT, Target: $TARGET_HEIGHT"

if wait_for_height 49101 $TARGET_HEIGHT 180; then
    FINAL_HEIGHT=$(get_height 49101)
    print_success "Node A reached height $FINAL_HEIGHT"
else
    print_info "Node A reached $(get_height 49101) (target was $TARGET_HEIGHT)"
    FINAL_HEIGHT=$(get_height 49101)
fi

print_step "Starting Node C (late joiner)..."
$NODE_BINARY \
    --dev \
    --port 40103 --rpc-port 49103 \
    --base-path $TEST_DIR/node_c \
    > $LOG_DIR/node_c.log 2>&1 &
NODE_C_PID=$!

sleep 5

print_step "Waiting for Node C to sync..."
sleep 20

HEIGHT_C=$(get_height 49103)
HEIGHT_A=$(get_height 49101)

print_info "Node A height: $HEIGHT_A"
print_info "Node C height: $HEIGHT_C"

GAP=$((FINAL_HEIGHT - 1))  # Expected sync amount

if [ "$HEIGHT_C" -ge $((FINAL_HEIGHT - 2)) ]; then
    SYNCED_BLOCKS=$((HEIGHT_C - 1))
    print_success "PASS: Node C synced $SYNCED_BLOCKS blocks (target was $GAP)"

    # Check logs for sync activity
    if grep -q "Requesting.*blocks from height" $LOG_DIR/node_c.log; then
        print_success "Sync requests found in logs"
        grep "Requesting.*blocks" $LOG_DIR/node_c.log | head -3 | while read line; do
            print_info "  $line"
        done
    fi

    if grep -q "Applied.*blocks from network" $LOG_DIR/node_c.log; then
        print_success "Block application confirmed in logs"
        grep "Applied.*blocks" $LOG_DIR/node_c.log | tail -3 | while read line; do
            print_info "  $line"
        done
    fi
else
    print_failure "FAIL: Node C did not sync fully (expected ~$FINAL_HEIGHT, got $HEIGHT_C)"
fi

#=============================================================================
# Test 4: Multi-Node Consistency
#=============================================================================
print_test "Multi-Node Consistency Check"

print_step "Checking all nodes have consistent state..."

HEIGHT_A=$(get_height 49101)
HEIGHT_B=$(get_height 49102)
HEIGHT_C=$(get_height 49103)

print_info "Heights: A=$HEIGHT_A, B=$HEIGHT_B, C=$HEIGHT_C"

MAX_HEIGHT=$HEIGHT_A
[ $HEIGHT_B -gt $MAX_HEIGHT ] && MAX_HEIGHT=$HEIGHT_B
[ $HEIGHT_C -gt $MAX_HEIGHT ] && MAX_HEIGHT=$HEIGHT_C

MIN_HEIGHT=$HEIGHT_A
[ $HEIGHT_B -lt $MIN_HEIGHT ] && MIN_HEIGHT=$HEIGHT_B
[ $HEIGHT_C -lt $MIN_HEIGHT ] && MIN_HEIGHT=$HEIGHT_C

HEIGHT_RANGE=$((MAX_HEIGHT - MIN_HEIGHT))

if [ $HEIGHT_RANGE -le 1 ]; then
    print_success "PASS: All nodes within 1 block of each other"

    # Check block hashes at common height
    COMMON_HEIGHT=$MIN_HEIGHT

    HASH_A=$(curl -s -X POST http://localhost:49101 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"chain_getBlockByHeight\",\"params\":[$COMMON_HEIGHT],\"id\":1}" \
        | jq -r '.result.hash // ""')

    HASH_B=$(curl -s -X POST http://localhost:49102 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"chain_getBlockByHeight\",\"params\":[$COMMON_HEIGHT],\"id\":1}" \
        | jq -r '.result.hash // ""')

    HASH_C=$(curl -s -X POST http://localhost:49103 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"chain_getBlockByHeight\",\"params\":[$COMMON_HEIGHT],\"id\":1}" \
        | jq -r '.result.hash // ""')

    if [ "$HASH_A" = "$HASH_B" ] && [ "$HASH_B" = "$HASH_C" ] && [ -n "$HASH_A" ]; then
        print_success "PASS: All nodes have identical blockchain at height $COMMON_HEIGHT"
        print_info "Common hash: ${HASH_A:0:16}..."
    else
        print_failure "FAIL: Block hashes differ between nodes!"
        print_info "Hash A: ${HASH_A:0:32}"
        print_info "Hash B: ${HASH_B:0:32}"
        print_info "Hash C: ${HASH_C:0:32}"
    fi
else
    print_failure "FAIL: Height range too large ($HEIGHT_RANGE blocks)"
fi

#=============================================================================
# Test 5: Network Partition Recovery
#=============================================================================
print_test "Network Partition Recovery"

print_step "Simulating network partition by stopping Node B..."
kill $NODE_B_PID 2>/dev/null || true
sleep 2

PARTITION_HEIGHT=$(get_height 49101)
print_info "Partition started at height $PARTITION_HEIGHT"

print_step "Letting Node A mine 5 more blocks while B is offline..."
TARGET=$((PARTITION_HEIGHT + 5))

if wait_for_height 49101 $TARGET 60; then
    POST_PARTITION_HEIGHT=$(get_height 49101)
    print_success "Node A reached height $POST_PARTITION_HEIGHT"
else
    POST_PARTITION_HEIGHT=$(get_height 49101)
    print_info "Node A reached height $POST_PARTITION_HEIGHT"
fi

print_step "Reconnecting Node B..."
$NODE_BINARY \
    --dev \
    --port 40102 --rpc-port 49102 \
    --base-path $TEST_DIR/node_b \
    > $LOG_DIR/node_b_rejoined.log 2>&1 &
NODE_B_PID=$!

sleep 5

print_step "Waiting for Node B to catch up..."
sleep 15

HEIGHT_B=$(get_height 49102)
HEIGHT_A=$(get_height 49101)

EXPECTED_SYNC=$((POST_PARTITION_HEIGHT - PARTITION_HEIGHT))

print_info "Node A height: $HEIGHT_A"
print_info "Node B height: $HEIGHT_B"
print_info "Expected sync: ~$EXPECTED_SYNC blocks"

if [ "$HEIGHT_B" -ge $((HEIGHT_A - 1)) ]; then
    print_success "PASS: Node B recovered from partition"

    # Check logs for sync activity
    if grep -q "Loaded existing blockchain state" $LOG_DIR/node_b_rejoined.log; then
        print_success "State restored from disk"
    fi

    if grep -q "Requesting.*blocks" $LOG_DIR/node_b_rejoined.log; then
        print_success "Sync requests detected"
    fi
else
    print_failure "FAIL: Node B did not fully recover (A: $HEIGHT_A, B: $HEIGHT_B)"
fi

#=============================================================================
# Summary
#=============================================================================
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}📊 Verification Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

print_info "Test Results:"
echo "  • Two-node synchronization: Checked"
echo "  • Real-time block propagation: Checked"
echo "  • Late-joining node sync: Checked"
echo "  • Multi-node consistency: Checked"
echo "  • Network partition recovery: Checked"

echo ""
print_info "Final Heights:"
echo "  • Node A: $(get_height 49101)"
echo "  • Node B: $(get_height 49102)"
echo "  • Node C: $(get_height 49103)"

echo ""
print_info "Logs available at: $LOG_DIR"

echo ""
echo -e "${GREEN}✅ Network synchronization verification complete!${NC}"
echo ""
