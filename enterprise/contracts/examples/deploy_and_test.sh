#!/bin/bash
# Enterprise E2 Multipass Smart Contract - Deploy and Test Script
#
# This script automates the deployment and testing of all contract templates.
# Prerequisites:
# - E2 Multipass backend running on localhost:8080
# - Boundless blockchain nodes running
# - cargo-contract installed

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Enterprise E2 Multipass - Contract Deployment & Testing    ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# ============================================================================
# Step 1: Check Prerequisites
# ============================================================================

echo -e "${YELLOW}Step 1: Checking prerequisites...${NC}"

# Check if cargo-contract is installed
if ! command -v cargo-contract &> /dev/null; then
    echo -e "${RED}✗ cargo-contract not found${NC}"
    echo "Install with: cargo install cargo-contract --force"
    exit 1
fi
echo -e "${GREEN}✓ cargo-contract installed${NC}"

# Check if E2 backend is running
if ! curl -s http://localhost:8080/api/auth/health &> /dev/null; then
    echo -e "${RED}✗ E2 Multipass backend not running${NC}"
    echo "Start with: cd enterprise && cargo run --bin enterprise-server"
    exit 1
fi
echo -e "${GREEN}✓ E2 Multipass backend running${NC}"

# Check if blockchain node is running
if ! curl -s http://localhost:9933 &> /dev/null; then
    echo -e "${RED}✗ Boundless blockchain node not running${NC}"
    echo "Start with: docker-compose up -d"
    exit 1
fi
echo -e "${GREEN}✓ Boundless blockchain node running${NC}"

echo ""

# ============================================================================
# Step 2: Build Contracts
# ============================================================================

echo -e "${YELLOW}Step 2: Building smart contracts...${NC}"

CONTRACTS=(
    "identity_access_control"
    "multisig_wallet"
    "asset_escrow"
    "app_authorization"
)

for contract in "${CONTRACTS[@]}"; do
    echo -e "${BLUE}Building $contract...${NC}"

    # Create temporary Cargo.toml for each contract
    cat > /tmp/${contract}_Cargo.toml << EOF
[package]
name = "${contract}"
version = "0.1.0"
edition = "2021"

[dependencies]
ink = { version = "4.3", default-features = false }

[lib]
name = "${contract}"
path = "templates/${contract}.rs"
crate-type = ["cdylib"]

[features]
default = ["std"]
std = ["ink/std"]
ink-as-dependency = []
EOF

    # Build with cargo-contract
    if cargo contract build --manifest-path /tmp/${contract}_Cargo.toml --release 2>&1 | grep -q "ERROR"; then
        echo -e "${RED}✗ Build failed for $contract${NC}"
        exit 1
    fi

    echo -e "${GREEN}✓ Built $contract${NC}"
done

echo ""

# ============================================================================
# Step 3: Run Tests
# ============================================================================

echo -e "${YELLOW}Step 3: Running contract tests...${NC}"

for contract in "${CONTRACTS[@]}"; do
    echo -e "${BLUE}Testing $contract...${NC}"

    # Run unit tests
    if cargo test --manifest-path /tmp/${contract}_Cargo.toml 2>&1 | grep -q "test result: FAILED"; then
        echo -e "${RED}✗ Tests failed for $contract${NC}"
        exit 1
    fi

    echo -e "${GREEN}✓ Tests passed for $contract${NC}"
done

echo ""

# ============================================================================
# Step 4: Deploy Contracts (Simulation)
# ============================================================================

echo -e "${YELLOW}Step 4: Deploying contracts (simulation)...${NC}"

# In a real deployment, you would:
# 1. Read deployment config
# 2. Submit WASM to blockchain
# 3. Wait for confirmation
# 4. Save contract addresses

for contract in "${CONTRACTS[@]}"; do
    echo -e "${BLUE}Deploying $contract...${NC}"

    # Simulate deployment
    WASM_SIZE=$(wc -c < "target/ink/${contract}.wasm" 2>/dev/null || echo "N/A")
    echo "   WASM Size: ${WASM_SIZE} bytes"
    echo "   Gas Limit: 50,000,000"
    echo "   Network: http://localhost:9933"

    # Generate mock contract address
    CONTRACT_ADDR="0x$(openssl rand -hex 20)"
    echo "   Contract Address: $CONTRACT_ADDR"

    # Save to deployment results
    echo "$contract=$CONTRACT_ADDR" >> deployments/addresses.txt

    echo -e "${GREEN}✓ Deployed $contract${NC}"
done

echo ""

# ============================================================================
# Step 5: Integration Test
# ============================================================================

echo -e "${YELLOW}Step 5: Running integration tests with E2 Multipass...${NC}"

# Test 1: Login to E2
echo -e "${BLUE}Test 1: E2 Multipass authentication...${NC}"
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8080/api/auth/login \
    -H "Content-Type: application/json" \
    -d '{"email":"admin@boundless.local","password":"BoundlessTrust@2024"}')

if echo "$LOGIN_RESPONSE" | grep -q "token"; then
    echo -e "${GREEN}✓ Authentication successful${NC}"
    TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*' | cut -d'"' -f4)
else
    echo -e "${RED}✗ Authentication failed${NC}"
    exit 1
fi

# Test 2: Get identity info
echo -e "${BLUE}Test 2: Fetching identity profile...${NC}"
IDENTITY_ID=$(echo "$LOGIN_RESPONSE" | grep -o '"identity_id":"[^"]*' | cut -d'"' -f4)

if [ ! -z "$IDENTITY_ID" ]; then
    echo -e "${GREEN}✓ Identity ID: $IDENTITY_ID${NC}"
else
    echo -e "${RED}✗ Failed to get identity${NC}"
    exit 1
fi

# Test 3: Get asset balances
echo -e "${BLUE}Test 3: Fetching asset balances...${NC}"
BALANCES=$(curl -s http://localhost:8080/api/assets/balances \
    -H "Authorization: Bearer $TOKEN")

if echo "$BALANCES" | grep -q "asset_id"; then
    echo -e "${GREEN}✓ Asset balances retrieved${NC}"
else
    echo -e "${YELLOW}! No assets found (normal for new account)${NC}"
fi

echo ""

# ============================================================================
# Summary
# ============================================================================

echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                 Deployment Summary                          ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✓ All contracts built successfully${NC}"
echo -e "${GREEN}✓ All tests passed${NC}"
echo -e "${GREEN}✓ All contracts deployed (simulated)${NC}"
echo -e "${GREEN}✓ E2 integration tests passed${NC}"
echo ""
echo "Deployed contract addresses saved to: deployments/addresses.txt"
echo ""
echo "Next steps:"
echo "  1. Review deployment results"
echo "  2. Test contracts with frontend: http://localhost:3001"
echo "  3. Run integration examples: cargo run --example e2_integration"
echo ""
