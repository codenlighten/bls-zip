# Enterprise Multipass Deployment Guide

This guide covers deploying the Boundless Enterprise Multipass system to production.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start (Development)](#quick-start-development)
3. [Production Deployment](#production-deployment)
4. [Environment Configuration](#environment-configuration)
5. [Database Setup](#database-setup)
6. [Service Architecture](#service-architecture)
7. [Monitoring & Logging](#monitoring--logging)
8. [Security Hardening](#security-hardening)
9. [Backup & Recovery](#backup--recovery)
10. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- **OS**: Linux (Ubuntu 20.04+ or CentOS 8+), macOS, or Windows with WSL2
- **RAM**: Minimum 4GB, recommended 8GB+
- **CPU**: 2+ cores recommended
- **Storage**: 50GB+ available disk space

### Software Dependencies

- **Rust**: 1.75+ ([install](https://rustup.rs/))
- **PostgreSQL**: 14+ ([install](https://www.postgresql.org/download/))
- **Boundless Node**: Running blockchain node
- **Docker** (optional): For containerized deployment

## Quick Start (Development)

### 1. Clone Repository

```bash
cd boundless-bls-platform/enterprise
```

### 2. Set Up Environment

```bash
# Copy example environment file
cp .env.example .env

# Edit .env with your configuration
nano .env
```

### 3. Set Up Database

```bash
# Create database
createdb boundless_enterprise

# Run migrations
sqlx database create
sqlx migrate run
```

### 4. Build and Run

```bash
# Build in release mode
cargo build --release

# Run the server
./target/release/enterprise-server
```

The server will start on `http://localhost:8080` (or the `BIND_ADDR` you configured).

## Production Deployment

### Option 1: Systemd Service (Linux)

#### 1. Build Release Binary

```bash
cargo build --release --bin enterprise-server
```

#### 2. Create Systemd Service File

Create `/etc/systemd/system/boundless-enterprise.service`:

```ini
[Unit]
Description=Boundless Enterprise Multipass Server
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=boundless
Group=boundless
WorkingDirectory=/opt/boundless/enterprise
Environment="DATABASE_URL=postgresql://user:password@localhost:5432/boundless_enterprise"
Environment="BOUNDLESS_RPC_URL=http://localhost:9933"
Environment="BIND_ADDR=0.0.0.0:8080"
Environment="JWT_SECRET=CHANGE_THIS_TO_SECURE_RANDOM_STRING"
ExecStart=/opt/boundless/enterprise/target/release/enterprise-server
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=boundless-enterprise

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/boundless/enterprise

[Install]
WantedBy=multi-user.target
```

#### 3. Enable and Start Service

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service to start on boot
sudo systemctl enable boundless-enterprise

# Start the service
sudo systemctl start boundless-enterprise

# Check status
sudo systemctl status boundless-enterprise

# View logs
sudo journalctl -u boundless-enterprise -f
```

### Option 2: Docker Deployment

#### 1. Create Dockerfile

Create `Dockerfile` in enterprise directory:

```dockerfile
# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release --bin enterprise-server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 -s /bin/bash boundless

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/enterprise-server /app/
COPY migrations ./migrations

# Change ownership
RUN chown -R boundless:boundless /app

USER boundless

EXPOSE 8080

CMD ["./enterprise-server"]
```

#### 2. Create docker-compose.yml

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: boundless_enterprise
      POSTGRES_USER: boundless
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U boundless"]
      interval: 10s
      timeout: 5s
      retries: 5

  enterprise:
    build: .
    depends_on:
      postgres:
        condition: service_healthy
    environment:
      DATABASE_URL: postgresql://boundless:${DB_PASSWORD}@postgres:5432/boundless_enterprise
      BOUNDLESS_RPC_URL: ${BOUNDLESS_RPC_URL:-http://host.docker.internal:9933}
      BIND_ADDR: 0.0.0.0:8080
      JWT_SECRET: ${JWT_SECRET}
      RUST_LOG: info
    ports:
      - "8080:8080"
    restart: unless-stopped

volumes:
  postgres_data:
```

#### 3. Deploy with Docker Compose

```bash
# Create .env file with secrets
cat > .env <<EOF
DB_PASSWORD=$(openssl rand -hex 32)
JWT_SECRET=$(openssl rand -hex 32)
BOUNDLESS_RPC_URL=http://your-node:9933
EOF

# Build and start services
docker-compose up -d

# View logs
docker-compose logs -f enterprise

# Stop services
docker-compose down
```

### Option 3: Kubernetes Deployment

See `k8s/` directory for Kubernetes manifests.

## Environment Configuration

### Required Variables

```bash
# Database connection (REQUIRED)
DATABASE_URL=postgresql://user:password@localhost:5432/boundless_enterprise

# JWT secret (REQUIRED - generate securely!)
JWT_SECRET=$(openssl rand -hex 32)

# Blockchain RPC endpoint (REQUIRED)
BOUNDLESS_RPC_URL=http://localhost:9933
```

### Optional Variables

See `.env.example` for all available configuration options.

### Generating Secure Secrets

```bash
# JWT secret (256-bit)
openssl rand -hex 32

# Database password (128-bit)
openssl rand -base64 32

# API key (192-bit)
openssl rand -hex 24
```

## Database Setup

### Creating Database

```bash
# Using psql
sudo -u postgres createdb boundless_enterprise
sudo -u postgres createuser boundless -P

# Grant privileges
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE boundless_enterprise TO boundless;"
```

### Running Migrations

Using sqlx-cli:

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
export DATABASE_URL=postgresql://boundless:password@localhost:5432/boundless_enterprise
sqlx migrate run
```

Using psql directly:

```bash
psql -U boundless -d boundless_enterprise -f migrations/001_create_enterprise_tables.sql
```

### Database Backup

```bash
# Create backup
pg_dump boundless_enterprise > backup_$(date +%Y%m%d).sql

# Restore backup
psql boundless_enterprise < backup_20240115.sql
```

## Service Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Enterprise Multipass                      │
├─────────────────────────────────────────────────────────────┤
│  REST API Server (Axum)                                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │
│  │ Identity │ │  Wallet  │ │   Auth   │ │   App    │      │
│  │ Service  │ │ Service  │ │ Service  │ │ Registry │      │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                   │
│  │  Asset & │ │  Events  │ │ Hardware │                   │
│  │  Market  │ │ Reporting│ │   Pass   │                   │
│  └──────────┘ └──────────┘ └──────────┘                   │
├─────────────────────────────────────────────────────────────┤
│  Database Layer (PostgreSQL)                                │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  19 Tables across 7 services                          │ │
│  └───────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  Blockchain Client (RPC)                                    │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  HTTP Client → Boundless Node (port 9933)             │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### API Endpoints

All endpoints are prefixed with `/api`:

- `/api/identity/*` - Identity management
- `/api/wallet/*` - Wallet operations
- `/api/auth/*` - Authentication & SSO
- `/api/applications/*` - Application registry
- `/api/assets/*` - Asset management
- `/api/market/*` - Trading & markets
- `/api/notifications/*` - Notifications
- `/api/reports/*` - Report generation
- `/api/hardware/*` - Hardware pass management

## Monitoring & Logging

### Structured Logging

Configure log level via environment:

```bash
RUST_LOG=info  # trace, debug, info, warn, error
```

### Log Aggregation

For production, use structured JSON logs:

```bash
LOG_JSON=true
```

Then aggregate with:
- ELK Stack (Elasticsearch, Logstash, Kibana)
- Grafana Loki
- Datadog
- CloudWatch (AWS)

### Metrics Endpoints

TODO: Add Prometheus metrics endpoint

### Health Checks

```bash
# Simple health check
curl http://localhost:8080/health

# Database connectivity check
curl http://localhost:8080/health/db

# Blockchain RPC check
curl http://localhost:8080/health/blockchain
```

## Security Hardening

### 1. Network Security

```bash
# Firewall rules (ufw example)
sudo ufw allow 8080/tcp  # API port
sudo ufw allow 5432/tcp  # PostgreSQL (only from app servers)
sudo ufw enable
```

### 2. TLS/SSL Termination

Use reverse proxy (nginx, Caddy, Traefik) for TLS:

```nginx
server {
    listen 443 ssl http2;
    server_name enterprise.yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### 3. Database Security

```sql
-- Create read-only user for reporting
CREATE USER enterprise_readonly WITH PASSWORD 'secure_password';
GRANT CONNECT ON DATABASE boundless_enterprise TO enterprise_readonly;
GRANT USAGE ON SCHEMA public TO enterprise_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO enterprise_readonly;

-- Enable SSL for PostgreSQL connections
-- Edit postgresql.conf:
-- ssl = on
-- ssl_cert_file = '/path/to/server.crt'
-- ssl_key_file = '/path/to/server.key'
```

### 4. Secrets Management

Use secret management tools:
- AWS Secrets Manager
- HashiCorp Vault
- Kubernetes Secrets
- Azure Key Vault

Example with Vault:

```bash
# Store secrets in Vault
vault kv put secret/enterprise jwt_secret="$(openssl rand -hex 32)"

# Retrieve in startup script
export JWT_SECRET=$(vault kv get -field=jwt_secret secret/enterprise)
```

## Backup & Recovery

### Automated Database Backups

```bash
#!/bin/bash
# /usr/local/bin/backup-enterprise-db.sh

BACKUP_DIR="/var/backups/boundless/enterprise"
DATE=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=30

# Create backup
pg_dump -U boundless boundless_enterprise | gzip > "$BACKUP_DIR/enterprise_$DATE.sql.gz"

# Remove old backups
find "$BACKUP_DIR" -name "enterprise_*.sql.gz" -mtime +$RETENTION_DAYS -delete

# Upload to S3 (optional)
aws s3 cp "$BACKUP_DIR/enterprise_$DATE.sql.gz" s3://your-bucket/backups/
```

Add to crontab:

```bash
# Daily backups at 2 AM
0 2 * * * /usr/local/bin/backup-enterprise-db.sh
```

### Disaster Recovery

```bash
# 1. Stop the service
sudo systemctl stop boundless-enterprise

# 2. Drop and recreate database
dropdb boundless_enterprise
createdb boundless_enterprise

# 3. Restore from backup
gunzip < /var/backups/boundless/enterprise_latest.sql.gz | psql boundless_enterprise

# 4. Restart service
sudo systemctl start boundless-enterprise
```

## Troubleshooting

### Common Issues

#### 1. Database Connection Failed

```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Check connection
psql -U boundless -d boundless_enterprise -c "SELECT 1;"

# Check DATABASE_URL format
echo $DATABASE_URL
```

#### 2. Blockchain RPC Unreachable

```bash
# Test RPC endpoint
curl http://localhost:9933/health

# Check firewall rules
sudo ufw status

# Verify BOUNDLESS_RPC_URL
echo $BOUNDLESS_RPC_URL
```

#### 3. High Memory Usage

```bash
# Check database connections
psql -U boundless -d boundless_enterprise -c "SELECT count(*) FROM pg_stat_activity;"

# Reduce DATABASE_MAX_CONNECTIONS in .env
DATABASE_MAX_CONNECTIONS=5
```

#### 4. Migration Failed

```bash
# Check migration status
sqlx migrate info

# Revert last migration
sqlx migrate revert

# Force migration (caution!)
psql -U boundless -d boundless_enterprise -f migrations/001_create_enterprise_tables.sql
```

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug ./enterprise-server
```

### Performance Profiling

```bash
# CPU profiling with perf
perf record -g ./enterprise-server
perf report

# Memory profiling with valgrind
valgrind --tool=massif ./enterprise-server
```

## Production Checklist

Before going live:

- [ ] Change all default passwords and secrets
- [ ] Configure TLS/SSL termination
- [ ] Set up automated database backups
- [ ] Configure log aggregation
- [ ] Set up monitoring and alerts
- [ ] Review firewall rules
- [ ] Enable rate limiting
- [ ] Configure CORS properly
- [ ] Set up disaster recovery plan
- [ ] Document runbook procedures
- [ ] Test backup restoration
- [ ] Load test the system
- [ ] Set up CI/CD pipeline
- [ ] Configure auto-scaling (if using cloud)

## Support

For issues and questions:
- GitHub Issues: https://github.com/boundless/boundless-bls-platform/issues
- Documentation: https://docs.boundless.com/enterprise
- Community: https://community.boundless.com
