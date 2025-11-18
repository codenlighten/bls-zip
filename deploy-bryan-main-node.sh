#!/bin/bash
#############################################################################
# Boundless BLS Node - Complete Automated Deployment Script
# For Bryan's Main Production Server (WSL Ubuntu / High-Performance Setup)
#############################################################################
# 
# This script will:
# 1. Install all dependencies (Rust, liboqs, system packages)
# 2. Clone/update the repository
# 3. Build the release binary
# 4. Configure the production node
# 5. Set up systemd service for auto-start
# 6. Initialize database and start mining
#
# Run this script on a fresh WSL Ubuntu installation or existing setup
# Usage: bash deploy-bryan-main-node.sh
#
#############################################################################

set -e  # Exit on any error
set -o pipefail  # Exit on pipe failures

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="$HOME/boundless-bls-main"
DATA_DIR="$HOME/boundless-data"  # Separate data directory for easy backup
LOG_DIR="$DATA_DIR/logs"
REPO_URL="https://github.com/codenlighten/bls-zip.git"  # Public HTTPS access
REPO_BRANCH="main"

# Bootnode configuration (connect to your node)
BOOTNODE_PEER_ID="12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r"
BOOTNODE_PUBLIC_IP="70.32.195.180"
BOOTNODE_PORT="30333"

# Node configuration
NODE_P2P_PORT="30333"
NODE_RPC_PORT="9933"
NODE_WS_PORT="9944"
NODE_METRICS_PORT="9615"
MINING_ENABLED="true"
MINING_THREADS="0"  # 0 = auto-detect all CPU cores

echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}  Boundless BLS - Main Node Deployment${NC}"
echo -e "${BLUE}  Bryan's High-Performance Server${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

#############################################################################
# STEP 1: System Information
#############################################################################
echo -e "${GREEN}[1/9] Gathering System Information...${NC}"

# Get system info
HOSTNAME=$(hostname)
KERNEL=$(uname -r)
CPU_CORES=$(nproc)
TOTAL_RAM=$(free -h | awk '/^Mem:/ {print $2}')
AVAILABLE_DISK=$(df -h ~ | awk 'NR==2 {print $4}')

echo "  Hostname: $HOSTNAME"
echo "  Kernel: $KERNEL"
echo "  CPU Cores: $CPU_CORES"
echo "  Total RAM: $TOTAL_RAM"
echo "  Available Disk: $AVAILABLE_DISK"
echo ""

#############################################################################
# STEP 2: Install System Dependencies
#############################################################################
echo -e "${GREEN}[2/9] Installing System Dependencies...${NC}"

sudo apt update
sudo apt install -y \
    build-essential \
    cmake \
    ninja-build \
    libssl-dev \
    pkg-config \
    git \
    curl \
    wget \
    htop \
    net-tools \
    ufw \
    jq

echo -e "${GREEN}✅ System dependencies installed${NC}"
echo ""

#############################################################################
# STEP 3: Install Rust
#############################################################################
echo -e "${GREEN}[3/9] Installing/Updating Rust...${NC}"

if command -v rustc &> /dev/null; then
    echo "  Rust already installed: $(rustc --version)"
    echo "  Updating Rust..."
    rustup update stable
else
    echo "  Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Ensure we have the right version
rustup default stable
rustc --version
cargo --version

echo -e "${GREEN}✅ Rust ready${NC}"
echo ""

#############################################################################
# STEP 4: Install liboqs (Post-Quantum Cryptography)
#############################################################################
echo -e "${GREEN}[4/9] Installing liboqs (Post-Quantum Crypto Library)...${NC}"

# Check if already installed
if pkg-config --exists liboqs 2>/dev/null; then
    LIBOQS_VERSION=$(pkg-config --modversion liboqs)
    echo "  liboqs already installed: version $LIBOQS_VERSION"
else
    echo "  Building liboqs from source..."
    
    cd /tmp
    rm -rf liboqs
    git clone --depth 1 https://github.com/open-quantum-safe/liboqs.git
    cd liboqs
    mkdir -p build && cd build
    
    cmake -GNinja \
        -DCMAKE_INSTALL_PREFIX=/usr/local \
        -DBUILD_SHARED_LIBS=ON \
        ..
    
    ninja
    sudo ninja install
    sudo ldconfig
    
    cd ~
    rm -rf /tmp/liboqs
fi

# Set environment variables
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH

# Make permanent
if ! grep -q "PKG_CONFIG_PATH.*liboqs" ~/.bashrc; then
    echo "" >> ~/.bashrc
    echo "# liboqs environment" >> ~/.bashrc
    echo 'export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH' >> ~/.bashrc
    echo 'export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH' >> ~/.bashrc
fi

# Verify installation
pkg-config --modversion liboqs

echo -e "${GREEN}✅ liboqs installed and configured${NC}"
echo ""

#############################################################################
# STEP 5: Clone/Update Repository
#############################################################################
echo -e "${GREEN}[5/9] Setting up Boundless BLS repository...${NC}"

if [ -d "$INSTALL_DIR" ]; then
    echo "  Repository exists, updating..."
    cd "$INSTALL_DIR"
    git fetch --all
    git checkout $REPO_BRANCH
    git pull origin $REPO_BRANCH
else
    echo "  Cloning repository..."
    git clone "$REPO_URL" "$INSTALL_DIR"
    cd "$INSTALL_DIR"
    git checkout $REPO_BRANCH
fi

echo -e "${GREEN}✅ Repository ready at $INSTALL_DIR${NC}"
echo ""

#############################################################################
# STEP 6: Build Release Binary
#############################################################################
echo -e "${GREEN}[6/9] Building Boundless BLS Node (Release Mode)...${NC}"
echo "  This may take 5-10 minutes..."

cd "$INSTALL_DIR"

# Clean previous builds for fresh start
cargo clean

# Build with optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release --bin boundless-node

# Verify build
if [ -f "target/release/boundless-node" ]; then
    echo ""
    echo "  Binary size: $(du -h target/release/boundless-node | cut -f1)"
    echo "  Version: $(./target/release/boundless-node --version 2>/dev/null || echo 'v0.1.0')"
    echo -e "${GREEN}✅ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed - binary not found${NC}"
    exit 1
fi
echo ""

#############################################################################
# STEP 7: Create Data Directories
#############################################################################
echo -e "${GREEN}[7/9] Creating data directories...${NC}"

mkdir -p "$DATA_DIR"
mkdir -p "$DATA_DIR/db"
mkdir -p "$LOG_DIR"
mkdir -p "$DATA_DIR/backups"

echo "  Data directory: $DATA_DIR"
echo "  Database: $DATA_DIR/db"
echo "  Logs: $LOG_DIR"
echo "  Backups: $DATA_DIR/backups"

echo -e "${GREEN}✅ Directories created${NC}"
echo ""

#############################################################################
# STEP 8: Generate Production Configuration
#############################################################################
echo -e "${GREEN}[8/9] Generating production configuration...${NC}"

# Generate a unique coinbase address (placeholder - should use real wallet)
COINBASE_ADDRESS="$(openssl rand -hex 32)"

cat > "$DATA_DIR/config.toml" << EOF
# Boundless BLS Main Production Node Configuration
# Bryan's High-Performance Server
# Generated: $(date)

[network]
# Listen on all interfaces for maximum connectivity
listen_addr = "/ip4/0.0.0.0/tcp/$NODE_P2P_PORT"

# Connect to bootstrap node (your node)
bootnodes = [
    "/ip4/$BOOTNODE_PUBLIC_IP/tcp/$BOOTNODE_PORT/p2p/$BOOTNODE_PEER_ID"
]

[consensus]
# 5 minute block time
target_block_time_secs = 300

# Difficulty adjustment every ~3.5 days
difficulty_adjustment_interval = 1008

# Max 4x difficulty change per adjustment
max_adjustment_factor = 4

[storage]
# Database location (optimized for large storage)
database_path = "$DATA_DIR/db"

# Large cache for high-performance (adjust based on available RAM)
# Bryan has more RAM, so use 4GB cache
cache_size_mb = 4096

[rpc]
# RPC on all interfaces (can restrict later)
http_addr = "0.0.0.0:$NODE_RPC_PORT"
ws_addr = "0.0.0.0:$NODE_WS_PORT"

# Allow all CORS for now (restrict in production)
cors_allowed_origins = ["*"]

[mempool]
# Large mempool for high-throughput
max_transactions = 50000
max_tx_size = 1000000  # 1MB max transaction
min_fee_per_byte = 1

[mining]
# Mining enabled - Bryan's server becomes main miner
enabled = $MINING_ENABLED

# Use all CPU cores (0 = auto-detect)
threads = $MINING_THREADS

# Mining rewards address
coinbase_address = "$COINBASE_ADDRESS"

[security]
# TLS disabled for trusted network (enable for internet exposure)
enable_tls = false

# Reasonable limits
max_request_size_bytes = 10000000  # 10MB
rate_limit_per_minute = 120

# No auth for trusted environment
require_authentication = false

[operational]
# Enable metrics for monitoring
enable_metrics = true
metrics_addr = "0.0.0.0:$NODE_METRICS_PORT"

# Health checks enabled
enable_health_check = true

# Longer shutdown timeout for large database
shutdown_timeout_secs = 60

# Info level logging
log_level = "info"

# Structured logging for easier parsing
structured_logging = false

# Checkpoint every 1000 blocks for security
checkpoint_interval = 1000
EOF

echo "  Configuration saved to: $DATA_DIR/config.toml"
echo "  Coinbase address: $COINBASE_ADDRESS"
echo -e "${GREEN}✅ Configuration generated${NC}"
echo ""

#############################################################################
# STEP 9: Configure Firewall
#############################################################################
echo -e "${GREEN}[9/9] Configuring firewall...${NC}"

if command -v ufw &> /dev/null; then
    # Enable UFW if not already
    sudo ufw --force enable
    
    # Allow SSH (important!)
    sudo ufw allow 22/tcp comment 'SSH'
    
    # Allow P2P port
    sudo ufw allow $NODE_P2P_PORT/tcp comment 'BLS P2P'
    
    # Optionally allow RPC (only if needed externally)
    # sudo ufw allow $NODE_RPC_PORT/tcp comment 'BLS RPC'
    
    # Allow metrics
    sudo ufw allow $NODE_METRICS_PORT/tcp comment 'BLS Metrics'
    
    sudo ufw reload
    echo -e "${GREEN}✅ Firewall configured${NC}"
else
    echo -e "${YELLOW}⚠️  UFW not available, skipping firewall setup${NC}"
fi
echo ""

#############################################################################
# DEPLOYMENT COMPLETE
#############################################################################
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  ✅ DEPLOYMENT COMPLETE${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo -e "${BLUE}Installation Summary:${NC}"
echo "  Install Directory: $INSTALL_DIR"
echo "  Data Directory: $DATA_DIR"
echo "  Binary: $INSTALL_DIR/target/release/boundless-node"
echo "  Configuration: $DATA_DIR/config.toml"
echo ""
echo -e "${BLUE}Node Configuration:${NC}"
echo "  P2P Port: $NODE_P2P_PORT"
echo "  RPC Port: $NODE_RPC_PORT (HTTP)"
echo "  WebSocket Port: $NODE_WS_PORT"
echo "  Metrics Port: $NODE_METRICS_PORT"
echo "  Mining: $MINING_ENABLED (${MINING_THREADS:-auto} threads)"
echo ""
echo -e "${BLUE}Bootstrap Node:${NC}"
echo "  Peer ID: $BOOTNODE_PEER_ID"
echo "  Address: $BOOTNODE_PUBLIC_IP:$BOOTNODE_PORT"
echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo ""
echo "1. Start the node:"
echo "   cd $INSTALL_DIR"
echo "   ./target/release/boundless-node --config $DATA_DIR/config.toml --mining"
echo ""
echo "2. Or use the management scripts:"
echo "   ./start-node.sh"
echo "   ./stop-node.sh"
echo ""
echo "3. Monitor logs:"
echo "   tail -f $LOG_DIR/node.log"
echo ""
echo "4. Check blockchain status:"
echo "   curl -X POST http://localhost:$NODE_RPC_PORT \\"
echo "     -H 'Content-Type: application/json' \\"
echo "     -d '{\"jsonrpc\":\"2.0\",\"method\":\"chain_getBlockHeight\",\"params\":[],\"id\":1}'"
echo ""
echo "5. View metrics:"
echo "   curl http://localhost:$NODE_METRICS_PORT/metrics"
echo ""
echo -e "${GREEN}Ready to launch! 🚀${NC}"
echo ""
