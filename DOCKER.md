# Boundless BLS Blockchain - Docker Deployment Guide

This guide explains how to deploy the Boundless BLS blockchain using Docker on Windows machines.

## Prerequisites

- **Docker Desktop for Windows** (with WSL 2 backend)
  - Download from: https://www.docker.com/products/docker-desktop
  - Minimum 4GB RAM allocated to Docker
  - 20GB free disk space

- **Docker Compose** (included with Docker Desktop)

## Quick Start

### 0. Optional: Configure Environment Variables

Before deploying, you can customize the configuration:

```powershell
# Copy the example environment file
cp .env.example .env

# Edit .env with your preferred settings:
# - RUST_LOG: Logging level (error, warn, info, debug, trace)
# - MINING_THREADS: Number of CPU threads for mining (1-16)
# - Ports for each node
```

**Example .env configuration:**
```env
RUST_LOG=info
MINING_THREADS=4
P2P_PORT_NODE1=30333
RPC_PORT_NODE1=9933
```

### 1. Build the Docker Image

Open PowerShell or Command Prompt in the project directory:

```powershell
# Build the image (this may take 10-15 minutes on first run)
docker build -t boundless-bls:latest .
```

### 2. Run a Single Node

#### Development Mode with Mining:
```powershell
docker run -d `
  --name boundless-node `
  -p 30333:30333 `
  -p 9933:9933 `
  -v boundless-data:/data `
  boundless-bls:latest `
  --dev --mining --rpc-external --rpc-cors all
```

#### Production Mode:
```powershell
docker run -d `
  --name boundless-node `
  -p 30333:30333 `
  -p 9933:9933 `
  -v boundless-data:/data `
  boundless-bls:latest `
  --base-path /data --rpc-external
```

### 3. Run Multi-Node Network with Docker Compose

Start a 3-node development network with **automatic peer discovery** using mDNS:

```powershell
# Start all nodes (they will automatically discover each other!)
docker-compose up -d

# View logs
docker-compose logs -f node1

# Stop all nodes
docker-compose down

# Stop and remove all data
docker-compose down -v
```

**Note:** In development mode (`--dev`), nodes use multicast DNS (mDNS) to automatically discover peers on the same Docker network. No manual bootnode configuration is required!

## Docker Commands Reference

### Building

```powershell
# Build the image
docker build -t boundless-bls:latest .

# Build with no cache (force rebuild)
docker build --no-cache -t boundless-bls:latest .

# Build for specific platform
docker build --platform linux/amd64 -t boundless-bls:latest .
```

### Running

```powershell
# Run node in foreground (see logs)
docker run --rm -p 30333:30333 -p 9933:9933 boundless-bls:latest --dev

# Run node in background
docker run -d --name my-node -p 30333:30333 -p 9933:9933 boundless-bls:latest --dev

# Run with custom data directory
docker run -d \
  --name my-node \
  -p 30333:30333 \
  -p 9933:9933 \
  -v C:\boundless-data:/data \
  boundless-bls:latest \
  --base-path /data

# Run with environment variables
docker run -d \
  --name my-node \
  -p 30333:30333 \
  -p 9933:9933 \
  -e RUST_LOG=info \
  boundless-bls:latest \
  --dev
```

### Managing Containers

```powershell
# List running containers
docker ps

# View container logs
docker logs boundless-node
docker logs -f boundless-node  # Follow logs

# Stop container
docker stop boundless-node

# Start stopped container
docker start boundless-node

# Restart container
docker restart boundless-node

# Remove container
docker rm boundless-node

# Remove container and volume
docker rm -v boundless-node
```

### Using the CLI

```powershell
# Run CLI commands
docker run --rm boundless-bls:latest boundless-cli --help

# Generate new keypair
docker run --rm boundless-bls:latest boundless-cli keygen

# Create transaction (interactive)
docker run --rm -it \
  --network boundless_boundless-network \
  boundless-bls:latest \
  boundless-cli create-tx \
  --rpc http://node1:9933
```

## Docker Compose Usage

### Starting the Network

```powershell
# Start all services
docker-compose up -d

# Start specific service
docker-compose up -d node1

# Start with build
docker-compose up -d --build

# Scale to more nodes
docker-compose up -d --scale node2=3
```

### Viewing Logs

```powershell
# All services
docker-compose logs

# Specific service
docker-compose logs node1

# Follow logs
docker-compose logs -f node1

# Last 100 lines
docker-compose logs --tail=100 node1
```

### Managing Services

```powershell
# Stop services
docker-compose stop

# Start stopped services
docker-compose start

# Restart services
docker-compose restart

# Remove services
docker-compose down

# Remove services and volumes
docker-compose down -v

# Remove services, volumes, and images
docker-compose down -v --rmi all
```

## Configuration

### Environment Variables

The project includes a `.env.example` file with all configurable options. To customize your deployment:

```powershell
# 1. Copy the example file
cp .env.example .env

# 2. Edit .env with your preferred settings
notepad .env
```

**Available environment variables:**

```env
# Logging Configuration
RUST_LOG=info                  # Log level: error, warn, info, debug, trace
RUST_BACKTRACE=0               # Backtrace: 0 (off), 1 (on), full

# Mining Configuration
MINING_THREADS=4               # Number of CPU threads for mining (1-16)

# Network Ports - Node 1 (Bootstrap/Mining Node)
P2P_PORT_NODE1=30333          # P2P networking port
RPC_PORT_NODE1=9933           # RPC HTTP port

# Network Ports - Node 2
P2P_PORT_NODE2=30334
RPC_PORT_NODE2=9934

# Network Ports - Node 3
P2P_PORT_NODE3=30335
RPC_PORT_NODE3=9935
```

**Docker Compose automatically loads these variables** from `.env` when you run:
```powershell
docker-compose up -d
```

### Port Mapping

Default ports:
- **30333**: P2P networking
- **9933**: RPC endpoint

To use different ports on the host:
```powershell
docker run -p 8080:30333 -p 8081:9933 boundless-bls:latest
```

### Volume Mounting

Persistent data storage:

```powershell
# Windows path
docker run -v C:\blockchain-data:/data boundless-bls:latest

# Named volume
docker run -v my-blockchain-data:/data boundless-bls:latest

# Mount configuration file
docker run -v C:\config\node.toml:/config/node.toml boundless-bls:latest
```

## Networking

### Bridge Network (Default)

Containers can communicate using container names:
```
http://node1:9933
```

### Connect to Running Node

```powershell
# Execute command in running container
docker exec boundless-node boundless-cli --help

# Open shell in container
docker exec -it boundless-node /bin/bash

# Check node status
docker exec boundless-node curl http://localhost:9933/health
```

## Monitoring

### Health Checks

```powershell
# Check container health
docker inspect --format='{{.State.Health.Status}}' boundless-node

# View health check logs
docker inspect --format='{{json .State.Health}}' boundless-node | jq
```

### Resource Usage

```powershell
# Container stats
docker stats boundless-node

# All containers
docker stats
```

## Troubleshooting

### View Container Information

```powershell
# Detailed container info
docker inspect boundless-node

# View container IP
docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' boundless-node
```

### Debug Build Issues

```powershell
# Build with progress output
docker build --progress=plain -t boundless-bls:latest .

# Build specific stage
docker build --target builder -t boundless-bls:builder .
```

### Common Issues

#### 1. Port Already in Use
```powershell
# Find process using port
netstat -ano | findstr :9933

# Kill process
taskkill /PID <PID> /F
```

#### 2. Out of Disk Space
```powershell
# Clean up Docker
docker system prune -a

# Remove unused volumes
docker volume prune

# Remove specific volume
docker volume rm boundless-data
```

#### 3. Container Won't Start
```powershell
# Check logs
docker logs boundless-node

# Run in foreground to see errors
docker run --rm boundless-bls:latest --dev
```

## Production Deployment

### Recommended Settings

```powershell
docker run -d \
  --name boundless-node \
  --restart unless-stopped \
  --memory 2g \
  --cpus 2 \
  -p 30333:30333 \
  -p 9933:9933 \
  -v boundless-data:/data \
  -e RUST_LOG=warn \
  boundless-bls:latest \
  --base-path /data \
  --rpc-external \
  --rpc-methods Safe
```

### Security Best Practices

1. **Don't expose RPC publicly** without authentication
2. **Use `--rpc-methods Safe` in production** (the docker-compose.yml already uses safe defaults)
3. **Never use `--rpc-methods Unsafe`** in production environments
4. **Limit container resources** with --memory and --cpus flags
5. **Keep images updated** regularly with latest Rust and dependencies
6. **Use secrets management** for private keys (never commit .env files)
7. **Enable firewall rules** for P2P port (30333)
8. **Protect .env files** - they are excluded from Docker builds via .dockerignore

**Note:** The updated docker-compose.yml removes `--rpc-methods Unsafe` for security by default.

### Backup and Recovery

```powershell
# Backup blockchain data
docker run --rm \
  -v boundless-data:/data \
  -v C:\backups:/backup \
  alpine tar czf /backup/blockchain-backup.tar.gz /data

# Restore from backup
docker run --rm \
  -v boundless-data:/data \
  -v C:\backups:/backup \
  alpine tar xzf /backup/blockchain-backup.tar.gz -C /
```

## Advanced Usage

### Multi-Stage Builds

The Dockerfile uses multi-stage builds to create smaller images:
- **Builder stage**: Compiles the application (~3GB)
- **Runtime stage**: Final image (~200MB)

### Custom Configuration

Create a custom docker-compose.override.yml:

```yaml
version: '3.8'

services:
  node1:
    environment:
      - RUST_LOG=debug
    command: >
      --dev
      --mining
      --base-path /data
      --rpc-external
      --rpc-cors all
      --mining-threads 8
```

## Performance Tuning

### Docker Desktop Settings

1. **Resources > Memory**: Allocate 4-8GB
2. **Resources > CPUs**: Allocate 2-4 cores
3. **Resources > Disk**: 20GB+ free space

### Container Resource Limits

```yaml
services:
  node1:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G
```

## Support

For issues related to Docker deployment, please check:
- Docker logs: `docker logs <container>`
- Docker Desktop logs: `%APPDATA%\Docker\log`
- GitHub Issues: [repository]/issues

## What's New in This Version

### Recent Improvements (Latest Update)

1. **Fixed Docker Build Issues**
   - Updated to Rust 1.83 (latest stable)
   - Added `ninja-build` package for CMake compilation
   - Added `curl` to runtime image for health checks

2. **Improved Multi-Node Networking**
   - **Automatic peer discovery** via mDNS - no manual bootnode configuration needed!
   - Removed invalid bootnode placeholders from docker-compose.yml
   - Nodes automatically find each other on the same Docker network

3. **Enhanced Security**
   - Removed `--rpc-methods Unsafe` from default configuration
   - Secure defaults for all deployment modes
   - .env files excluded from Docker builds via .dockerignore

4. **Environment-Based Configuration**
   - New `.env.example` file with comprehensive configuration options
   - Support for configuring: logging, mining threads, ports
   - Easy customization without modifying docker-compose.yml

5. **PowerShell Compatibility**
   - All examples updated with proper PowerShell line continuation (backticks)
   - Windows-optimized deployment instructions

### Docker Files Overview

- **Dockerfile** - Multi-stage build (builder ~3GB, runtime ~200MB)
- **docker-compose.yml** - 3-node network with mDNS auto-discovery
- **docker-run.bat** - Windows batch script for easy management
- **.env.example** - Configuration template
- **.dockerignore** - Build optimization and security
- **README-DOCKER.md** - Quick start guide
- **DOCKER.md** - This comprehensive guide

## License

This Docker configuration is part of the Boundless BLS blockchain platform.
