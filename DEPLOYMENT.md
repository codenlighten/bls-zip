# Boundless BLS Platform - Deployment Guide

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Building the Blockchain](#building-the-blockchain)
3. [Running Tests](#running-tests)
4. [Deploying Smart Contracts](#deploying-smart-contracts)
5. [Running the Frontend](#running-the-frontend)
6. [Enterprise Multipass Deployment](#enterprise-multipass-deployment)
7. [Production Deployment](#production-deployment)
8. [Monitoring and Maintenance](#monitoring-and-maintenance)

## Prerequisites

### System Requirements

- **Operating System**: Linux (Ubuntu 22.04+), macOS (12+), or Windows (WSL2)
- **CPU**: 4+ cores recommended
- **RAM**: 8GB minimum, 16GB+ recommended
- **Storage**: 50GB+ SSD for blockchain data
- **Network**: Stable internet connection for P2P networking

### Software Dependencies

#### Rust Toolchain

```bash
# Install Rust (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version  # Should be 1.75.0 or later
cargo --version
```

#### liboqs (Post-Quantum Cryptography)

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y cmake ninja-build libssl-dev

git clone --depth 1 https://github.com/open-quantum-safe/liboqs.git
cd liboqs
mkdir build && cd build
cmake -GNinja -DCMAKE_INSTALL_PREFIX=/usr/local ..
ninja
sudo ninja install

# macOS
brew install liboqs

# Verify
pkg-config --modversion liboqs
```

#### Node.js & npm (for frontend)

```bash
# Install Node.js 20+
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Verify
node --version  # Should be v20.0.0 or later
npm --version
```

#### ink! Smart Contract Tools

```bash
# Install cargo-contract
cargo install cargo-contract --force

# Verify
cargo contract --version
```

## Building the Blockchain

### 1. Clone the Repository

```bash
git clone https://github.com/your-org/boundless-bls-platform.git
cd boundless-bls-platform
```

### 2. Build the Workspace

```bash
# Build all crates in release mode
cargo build --release

# This will compile:
# - core (blockchain data structures)
# - consensus (SHA-3 PoW)
# - crypto (PQC algorithms)
# - wasm-runtime (smart contract execution)
# - p2p (networking)
# - rpc (JSON-RPC API)
# - node (full node binary)
```

Build artifacts will be in `target/release/`.

### 3. Run Tests

```bash
# Run all tests
cargo test --all

# Run tests with output
cargo test --all -- --nocapture

# Run specific crate tests
cargo test -p boundless-core
cargo test -p boundless-consensus
cargo test -p boundless-crypto

# Run ignored (slow) tests
cargo test --all -- --ignored
```

### 4. Start a Development Node

```bash
# Run a single-node development chain
./target/release/boundless-node --dev

# Custom data directory
./target/release/boundless-node --dev --base-path ./dev-chain

# Enable debug logging
RUST_LOG=debug ./target/release/boundless-node --dev
```

The node will:
- Start mining blocks with easy difficulty
- Expose JSON-RPC on `http://localhost:9933`
- Start P2P networking on port `30333`
- Create genesis block with initial accounts

## Deploying Smart Contracts

### 1. Build Contracts

```bash
cd contracts/token
cargo contract build --release

# Output:
# target/ink/boundless_token.wasm
# target/ink/boundless_token.json (metadata)
```

### 2. Deploy via CLI (Future)

```bash
# Deploy token contract
boundless-cli deploy \
  --wasm contracts/token/target/ink/boundless_token.wasm \
  --metadata contracts/token/target/ink/boundless_token.json \
  --constructor new \
  --args "Boundless Token,BLS,18,1000000000" \
  --gas-limit 100000000

# Call contract function
boundless-cli call \
  --contract 0x1234... \
  --function transfer \
  --args "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY,1000"
```

### 3. Deploy via Frontend

1. Build the contract WASM
2. Open the frontend dApp
3. Navigate to "Contracts" section
4. Upload `.wasm` and `.json` files
5. Set constructor parameters
6. Submit deployment transaction

## Running the Frontend

### 1. Install Dependencies

```bash
cd frontend
npm install
```

### 2. Development Mode

```bash
# Start Next.js dev server
npm run dev

# Frontend available at http://localhost:3000
```

### 3. Production Build

```bash
# Build for production
npm run build

# Start production server
npm start

# Or export static files
npm run build
npx next export
# Static files in ./out/
```

### 4. Configure RPC Endpoint

Edit `frontend/src/config.ts`:

```typescript
export const RPC_ENDPOINT =
  process.env.NEXT_PUBLIC_RPC_ENDPOINT || 'ws://localhost:9944'
```

## Enterprise Multipass Deployment

The Enterprise Multipass provides identity management, wallet services, and asset management on top of the Boundless blockchain.

### Prerequisites

- PostgreSQL 14+
- OpenSSL 3.0+
- Boundless node running with RPC enabled

### 1. Database Setup

```bash
# Install PostgreSQL (Ubuntu)
sudo apt install postgresql postgresql-contrib

# Create database and user
sudo -u postgres psql
CREATE DATABASE enterprise_db;
CREATE USER enterprise_user WITH ENCRYPTED PASSWORD 'strong_password';
GRANT ALL PRIVILEGES ON DATABASE enterprise_db TO enterprise_user;
\q

# Run migrations
cd enterprise
sqlx migrate run
```

### 2. Configure Environment Variables

Create `enterprise/.env`:

```bash
# Database
DATABASE_URL=postgresql://enterprise_user:strong_password@localhost:5432/enterprise_db
DATABASE_MAX_CONNECTIONS=20

# Security
JWT_SECRET=$(openssl rand -hex 32)
MASTER_ENCRYPTION_KEY=$(openssl rand -hex 32)

# Blockchain Integration
BLOCKCHAIN_RPC_URL=http://localhost:9933

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Logging
RUST_LOG=info
```

### 3. Build Enterprise Backend

```bash
cd enterprise
cargo build --release

# Binary at: target/release/enterprise-server
```

### 4. Run as Systemd Service

Create `/etc/systemd/system/enterprise-multipass.service`:

```ini
[Unit]
Description=Boundless Enterprise Multipass API
After=network.target postgresql.service boundless-node.service
Requires=postgresql.service

[Service]
Type=simple
User=enterprise
Group=enterprise
WorkingDirectory=/opt/boundless/enterprise
EnvironmentFile=/etc/boundless/enterprise.env
ExecStart=/opt/boundless/enterprise/enterprise-server
Restart=always
RestartSec=10

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/boundless/enterprise

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable enterprise-multipass
sudo systemctl start enterprise-multipass
sudo systemctl status enterprise-multipass
```

### 5. Deploy Enterprise Frontend

```bash
cd enterprise/frontend

# Install dependencies
npm ci --production

# Build for production
npm run build

# Start production server
npm start
```

### 6. Set Up Reverse Proxy for Enterprise API

Add to your Nginx configuration:

```nginx
# Enterprise API
server {
    listen 443 ssl http2;
    server_name api.boundless-bls.com;

    ssl_certificate /etc/letsencrypt/live/boundless-bls.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/boundless-bls.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# Enterprise Frontend
server {
    listen 443 ssl http2;
    server_name enterprise.boundless-bls.com;

    ssl_certificate /etc/letsencrypt/live/boundless-bls.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/boundless-bls.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### 7. Docker Deployment (Enterprise)

Create `enterprise/docker-compose.yml`:

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:14-alpine
    environment:
      POSTGRES_DB: enterprise_db
      POSTGRES_USER: enterprise_user
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  enterprise-api:
    build:
      context: .
      dockerfile: Dockerfile
    environment:
      DATABASE_URL: postgresql://enterprise_user:${DB_PASSWORD}@postgres:5432/enterprise_db
      JWT_SECRET: ${JWT_SECRET}
      MASTER_ENCRYPTION_KEY: ${MASTER_KEY}
      BLOCKCHAIN_RPC_URL: http://boundless-node:9933
    ports:
      - "8080:8080"
    depends_on:
      - postgres

  enterprise-frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    environment:
      NEXT_PUBLIC_API_URL: http://enterprise-api:8080
    ports:
      - "3001:3000"
    depends_on:
      - enterprise-api

volumes:
  postgres_data:
```

Start services:

```bash
# Generate secrets
export DB_PASSWORD=$(openssl rand -hex 32)
export JWT_SECRET=$(openssl rand -hex 32)
export MASTER_KEY=$(openssl rand -hex 32)

# Save to .env
cat > .env <<EOF
DB_PASSWORD=$DB_PASSWORD
JWT_SECRET=$JWT_SECRET
MASTER_KEY=$MASTER_KEY
EOF

# Start services
docker-compose up -d

# Check logs
docker-compose logs -f enterprise-api
```

### 8. Verify Enterprise Deployment

```bash
# Check API health
curl https://api.boundless-bls.com/health

# Test authentication
curl -X POST https://api.boundless-bls.com/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"yourfriends@smartledger.solutions","password":"BoundlessTrust"}'

# Access frontend
open https://enterprise.boundless-bls.com
```

## Production Deployment

### 1. Configure Node

Create `config.toml`:

```toml
[network]
listen_addresses = ["/ip4/0.0.0.0/tcp/30333"]
public_addresses = ["/ip4/YOUR_PUBLIC_IP/tcp/30333"]
boot_nodes = [
  "/ip4/BOOT_NODE_IP/tcp/30333/p2p/PEER_ID"
]

[rpc]
listen_address = "127.0.0.1:9933"
cors_allowed_origins = ["https://your-frontend.com"]

[consensus]
mining_enabled = true
mining_threads = 4

[storage]
database_path = "/var/lib/boundless/db"
cache_size_mb = 2048
```

### 2. Run as Systemd Service

Create `/etc/systemd/system/boundless-node.service`:

```ini
[Unit]
Description=Boundless BLS Blockchain Node
After=network.target

[Service]
Type=simple
User=boundless
Group=boundless
WorkingDirectory=/opt/boundless
ExecStart=/opt/boundless/boundless-node --config /etc/boundless/config.toml
Restart=always
RestartSec=10

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/boundless

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable boundless-node
sudo systemctl start boundless-node
sudo systemctl status boundless-node
```

### 3. Set Up Reverse Proxy (Nginx)

```nginx
server {
    listen 443 ssl http2;
    server_name rpc.boundless-bls.com;

    ssl_certificate /etc/letsencrypt/live/boundless-bls.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/boundless-bls.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:9933;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_read_timeout 3600s;
    }
}
```

### 4. Deploy Frontend

#### Option A: Vercel/Netlify

```bash
# Vercel
npm i -g vercel
cd frontend
vercel --prod

# Netlify
npm i -g netlify-cli
cd frontend
netlify deploy --prod
```

#### Option B: Docker

```dockerfile
# frontend/Dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:20-alpine
WORKDIR /app
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/public ./public
COPY --from=builder /app/package*.json ./
RUN npm ci --production
EXPOSE 3000
CMD ["npm", "start"]
```

```bash
docker build -t boundless-frontend frontend/
docker run -p 3000:3000 -e NEXT_PUBLIC_RPC_ENDPOINT=wss://rpc.boundless-bls.com boundless-frontend
```

## Monitoring and Maintenance

### 1. View Logs

```bash
# Systemd service logs
sudo journalctl -u boundless-node -f

# Direct node logs
tail -f /var/log/boundless/node.log
```

### 2. Metrics and Monitoring

The node exposes Prometheus metrics at `http://localhost:9615/metrics`:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'boundless-node'
    static_configs:
      - targets: ['localhost:9615']
```

Key metrics:
- `boundless_block_height` - Current blockchain height
- `boundless_peers_count` - Connected peer count
- `boundless_tx_pool_size` - Pending transaction count
- `boundless_mining_hash_rate` - Current mining hash rate

### 3. Database Backup

```bash
# Stop node
sudo systemctl stop boundless-node

# Backup database
tar -czf boundless-backup-$(date +%Y%m%d).tar.gz /var/lib/boundless/db

# Restart node
sudo systemctl start boundless-node
```

### 4. Upgrading

```bash
# Pull latest code
git pull origin main

# Rebuild
cargo build --release

# Stop service
sudo systemctl stop boundless-node

# Replace binary
sudo cp target/release/boundless-node /opt/boundless/

# Start service
sudo systemctl start boundless-node

# Verify version
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"system_version","params":[],"id":1}' http://localhost:9933
```

## Troubleshooting

### liboqs Not Found

```bash
# Set PKG_CONFIG_PATH
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH

# Add to ~/.bashrc for persistence
echo 'export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH' >> ~/.bashrc
```

### Compilation Errors

```bash
# Clean and rebuild
cargo clean
cargo build --release

# Update dependencies
cargo update
```

### Port Already in Use

```bash
# Find process using port
sudo lsof -i :30333
sudo lsof -i :9933

# Kill process
sudo kill -9 <PID>
```

### Frontend Won't Connect

1. Check RPC endpoint is accessible
2. Verify CORS settings in node config
3. Check firewall rules
4. Test WebSocket connection:

```bash
wscat -c ws://localhost:9944
> {"jsonrpc":"2.0","method":"system_health","params":[],"id":1}
```

## Security Considerations

1. **Never expose RPC publicly without authentication**
2. **Use TLS/SSL for all production endpoints**
3. **Keep liboqs and dependencies updated**
4. **Regularly backup private keys and database**
5. **Monitor for unusual network activity**
6. **Use hardware security modules (HSMs) for validator keys**
7. **Enable firewall rules (only 30333, 443 public)**

## Support

- Documentation: https://docs.boundless-bls.com
- GitHub Issues: https://github.com/your-org/boundless-bls-platform/issues
- Discord: https://discord.gg/boundless-bls
- Email: support@boundless-bls.com
