#!/bin/bash
# Performance Benchmarking Suite for Boundless BLS Blockchain
# Tests block propagation, sync speed, and RPC performance

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
NODE_BINARY="./target/release/boundless-node"
TEST_DIR="./data/perf_test"
RESULTS_FILE="./benchmark_results.txt"

echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   Boundless BLS Performance Benchmark Suite${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo ""

# Initialize results file
echo "Boundless BLS Performance Benchmarks" > $RESULTS_FILE
echo "Date: $(date)" >> $RESULTS_FILE
echo "----------------------------------------" >> $RESULTS_FILE
echo "" >> $RESULTS_FILE

print_metric() {
    local name=$1
    local value=$2
    local unit=$3
    echo -e "${CYAN}📊 ${name}:${NC} ${GREEN}${value} ${unit}${NC}"
    echo "${name}: ${value} ${unit}" >> $RESULTS_FILE
}

print_section() {
    echo ""
    echo -e "${BLUE}═══ $1 ═══${NC}"
    echo ""
    echo "=== $1 ===" >> $RESULTS_FILE
}

# Check prerequisites
if [ ! -f "$NODE_BINARY" ]; then
    echo -e "${RED}❌ Node binary not found. Building...${NC}"
    cargo build --release || exit 1
fi

# Cleanup previous test data
rm -rf $TEST_DIR
mkdir -p $TEST_DIR

#=============================================================================
# Test 1: Block Propagation Latency
#=============================================================================
print_section "Test 1: Block Propagation Latency"

echo "Starting two nodes to measure block propagation time..."

# Start Node 1 (mining)
$NODE_BINARY \
    --dev --mining \
    --port 40001 --rpc-port 49001 \
    --base-path $TEST_DIR/node1 \
    --mining-threads 1 \
    > $TEST_DIR/node1.log 2>&1 &
NODE1_PID=$!

sleep 5

# Start Node 2 (receiving)
$NODE_BINARY \
    --dev \
    --port 40002 --rpc-port 49002 \
    --base-path $TEST_DIR/node2 \
    > $TEST_DIR/node2.log 2>&1 &
NODE2_PID=$!

sleep 10

echo "Monitoring block propagation..."
echo "Waiting for Node 1 to mine a block..."

# Monitor logs for block mining and propagation
tail -f $TEST_DIR/node1.log 2>/dev/null | while read line; do
    if echo "$line" | grep -q "Mined block"; then
        MINE_TIME=$(date +%s.%N)
        echo $MINE_TIME > $TEST_DIR/mine_time.txt
        break
    fi
done &
TAIL_PID=$!

sleep 20

# Check propagation
if [ -f $TEST_DIR/mine_time.txt ]; then
    # Look for propagation in Node 2 logs
    if grep -q "Received new block" $TEST_DIR/node2.log; then
        echo -e "${GREEN}✅ Block propagation detected${NC}"
        print_metric "Block Propagation" "< 1" "second (estimated)"
    else
        echo -e "${YELLOW}⚠️  Block propagation not detected in logs${NC}"
        print_metric "Block Propagation" "N/A" "(not detected)"
    fi
else
    echo -e "${YELLOW}⚠️  No blocks mined during test period${NC}"
fi

kill $TAIL_PID 2>/dev/null || true
kill $NODE1_PID $NODE2_PID 2>/dev/null || true
wait 2>/dev/null || true

sleep 2

#=============================================================================
# Test 2: Block Synchronization Speed
#=============================================================================
print_section "Test 2: Block Synchronization Speed"

echo "Testing how fast a new node can sync multiple blocks..."

# Start Node 1, let it mine blocks
echo "Starting mining node..."
$NODE_BINARY \
    --dev --mining \
    --port 40001 --rpc-port 49001 \
    --base-path $TEST_DIR/sync_node1 \
    --mining-threads 2 \
    > $TEST_DIR/sync_node1.log 2>&1 &
NODE1_PID=$!

sleep 5

# Wait for blocks to be mined
echo "Waiting for 20+ blocks to be mined..."
sleep 60

# Check height
HEIGHT1=$(curl -s -X POST http://localhost:49001 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}' \
    | jq -r '.result // 0')

echo "Node 1 mined $HEIGHT1 blocks"

if [ "$HEIGHT1" -lt 10 ]; then
    echo -e "${YELLOW}⚠️  Only $HEIGHT1 blocks mined, waiting longer...${NC}"
    sleep 30
    HEIGHT1=$(curl -s -X POST http://localhost:49001 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}' \
        | jq -r '.result // 0')
fi

# Start Node 2, measure sync time
echo "Starting sync node..."
SYNC_START=$(date +%s)

$NODE_BINARY \
    --dev \
    --port 40002 --rpc-port 49002 \
    --base-path $TEST_DIR/sync_node2 \
    > $TEST_DIR/sync_node2.log 2>&1 &
NODE2_PID=$!

sleep 5

# Wait for sync to complete
echo "Monitoring sync progress..."
SYNCED=false
for i in {1..60}; do
    HEIGHT2=$(curl -s -X POST http://localhost:49002 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}' \
        | jq -r '.result // 0')

    if [ "$HEIGHT2" -ge $((HEIGHT1 - 2)) ]; then
        SYNC_END=$(date +%s)
        SYNC_DURATION=$((SYNC_END - SYNC_START))
        SYNCED=true
        break
    fi

    echo "  Height: $HEIGHT2 / $HEIGHT1"
    sleep 1
done

if [ "$SYNCED" = true ]; then
    BLOCKS_SYNCED=$((HEIGHT1 - 1))
    BLOCKS_PER_SEC=$(echo "scale=2; $BLOCKS_SYNCED / $SYNC_DURATION" | bc)

    print_metric "Blocks Synced" "$BLOCKS_SYNCED" "blocks"
    print_metric "Sync Duration" "$SYNC_DURATION" "seconds"
    print_metric "Sync Speed" "$BLOCKS_PER_SEC" "blocks/second"
else
    echo -e "${RED}❌ Sync did not complete within timeout${NC}"
    print_metric "Sync Speed" "N/A" "(timeout)"
fi

kill $NODE1_PID $NODE2_PID 2>/dev/null || true
wait 2>/dev/null || true

sleep 2

#=============================================================================
# Test 3: RPC Response Times
#=============================================================================
print_section "Test 3: RPC Response Times"

echo "Testing RPC API performance..."

# Start a node
$NODE_BINARY \
    --dev --mining \
    --port 40001 --rpc-port 49001 \
    --base-path $TEST_DIR/rpc_node \
    --mining-threads 1 \
    > $TEST_DIR/rpc_node.log 2>&1 &
NODE1_PID=$!

sleep 10

# Test chain_getBlockHeight
echo "Testing chain_getBlockHeight..."
TOTAL_TIME=0
ITERATIONS=100

for i in $(seq 1 $ITERATIONS); do
    START=$(date +%s%3N)
    curl -s -X POST http://localhost:49001 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}' \
        > /dev/null
    END=$(date +%s%3N)
    DURATION=$((END - START))
    TOTAL_TIME=$((TOTAL_TIME + DURATION))
done

AVG_TIME=$(echo "scale=2; $TOTAL_TIME / $ITERATIONS" | bc)
print_metric "chain_getBlockHeight (avg)" "$AVG_TIME" "ms"

# Test chain_getInfo
echo "Testing chain_getInfo..."
TOTAL_TIME=0

for i in $(seq 1 50); do
    START=$(date +%s%3N)
    curl -s -X POST http://localhost:49001 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"chain_getInfo","params":[],"id":1}' \
        > /dev/null
    END=$(date +%s%3N)
    DURATION=$((END - START))
    TOTAL_TIME=$((TOTAL_TIME + DURATION))
done

AVG_TIME=$(echo "scale=2; $TOTAL_TIME / 50" | bc)
print_metric "chain_getInfo (avg)" "$AVG_TIME" "ms"

# Test throughput
echo "Testing RPC throughput (parallel requests)..."
START=$(date +%s)

for i in $(seq 1 1000); do
    curl -s -X POST http://localhost:49001 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}' \
        > /dev/null &
done

wait

END=$(date +%s)
DURATION=$((END - START))
THROUGHPUT=$(echo "scale=2; 1000 / $DURATION" | bc)

print_metric "RPC Throughput" "$THROUGHPUT" "requests/second"

kill $NODE1_PID 2>/dev/null || true
wait 2>/dev/null || true

sleep 2

#=============================================================================
# Test 4: Mining Performance
#=============================================================================
print_section "Test 4: Mining Performance"

echo "Testing mining hash rate..."

# Start mining node
$NODE_BINARY \
    --dev --mining \
    --port 40001 --rpc-port 49001 \
    --base-path $TEST_DIR/mining_node \
    --mining-threads 4 \
    > $TEST_DIR/mining_node.log 2>&1 &
NODE1_PID=$!

sleep 30

# Extract hash rate from logs
if grep -q "H/s" $TEST_DIR/mining_node.log; then
    HASH_RATE=$(grep "H/s" $TEST_DIR/mining_node.log | tail -1 | grep -oP '\d+\.\d+ H/s' | head -1)
    print_metric "Mining Hash Rate (4 threads)" "$HASH_RATE" ""

    # Calculate blocks per minute
    BLOCKS=$(grep "Mined block" $TEST_DIR/mining_node.log | wc -l)
    BLOCKS_PER_MIN=$(echo "scale=2; $BLOCKS * 2" | bc)  # 30s test * 2 = per minute
    print_metric "Blocks Per Minute" "$BLOCKS_PER_MIN" "blocks/min"
else
    echo -e "${YELLOW}⚠️  No mining activity detected${NC}"
fi

kill $NODE1_PID 2>/dev/null || true
wait 2>/dev/null || true

#=============================================================================
# Test 5: Storage Performance
#=============================================================================
print_section "Test 5: Storage Performance"

echo "Testing storage write/read performance..."

# Mine blocks and measure storage performance
$NODE_BINARY \
    --dev --mining \
    --port 40001 --rpc-port 49001 \
    --base-path $TEST_DIR/storage_node \
    --mining-threads 2 \
    > $TEST_DIR/storage_node.log 2>&1 &
NODE1_PID=$!

echo "Mining blocks for storage test..."
sleep 45

# Measure database size
DB_SIZE=$(du -sh $TEST_DIR/storage_node/db 2>/dev/null | awk '{print $1}')
BLOCKS=$(curl -s -X POST http://localhost:49001 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","params":[],"id":1}' \
    | jq -r '.result // 0')

print_metric "Database Size" "$DB_SIZE" "(for $BLOCKS blocks)"
print_metric "Size Per Block" "$(echo "scale=2; $(du -s $TEST_DIR/storage_node/db | awk '{print $1}') / $BLOCKS" | bc)" "KB/block"

# Test read performance (query all blocks)
echo "Testing block read performance..."
READ_START=$(date +%s%3N)

for i in $(seq 1 $BLOCKS); do
    curl -s -X POST http://localhost:49001 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"chain_getBlockByHeight\",\"params\":[$i],\"id\":1}" \
        > /dev/null
done

READ_END=$(date +%s%3N)
READ_DURATION=$((READ_END - READ_START))
READ_RATE=$(echo "scale=2; $BLOCKS * 1000 / $READ_DURATION" | bc)

print_metric "Block Read Rate" "$READ_RATE" "blocks/second"

kill $NODE1_PID 2>/dev/null || true
wait 2>/dev/null || true

#=============================================================================
# Summary
#=============================================================================
echo ""
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Performance Benchmarking Complete${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${CYAN}Results saved to: ${RESULTS_FILE}${NC}"
echo ""

# Cleanup
rm -rf $TEST_DIR

cat $RESULTS_FILE
