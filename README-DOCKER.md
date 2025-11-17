# Boundless BLS Blockchain - Docker Quick Start

This guide provides the fastest way to get started with Boundless BLS blockchain using Docker on Windows.

## Prerequisites

1. **Install Docker Desktop for Windows**
   - Download: https://www.docker.com/products/docker-desktop
   - Ensure WSL 2 backend is enabled
   - Allocate at least 4GB RAM to Docker

2. **Verify Installation**
   ```powershell
   docker --version
   docker-compose --version
   ```

## Quick Start (3 Commands)

**Note:** Before starting, you can optionally configure environment variables:
```powershell
# Optional: Copy and customize environment variables
cp .env.example .env
# Edit .env to set RUST_LOG, MINING_THREADS, ports, etc.
```

### Option 1: Single Development Node

```powershell
# 1. Build the image
docker build -t boundless-bls:latest .

# 2. Run development node with mining
docker run -d --name boundless-node -p 30333:30333 -p 9933:9933 boundless-bls:latest --dev --mining --rpc-external

# 3. View logs
docker logs -f boundless-node
```

### Option 2: Using Windows Batch Script

```cmd
# 1. Build
docker-run.bat build

# 2. Start development node
docker-run.bat dev

# 3. View logs (automatically shown)
```

### Option 3: Multi-Node Network

**Note:** Nodes automatically discover each other using mDNS (multicast DNS) in development mode. No manual bootnode configuration needed!

```powershell
# 1. Optional: Configure environment (mining threads, logging, etc.)
cp .env.example .env
# Edit .env: set MINING_THREADS=4, RUST_LOG=info, etc.

# 2. Build
docker build -t boundless-bls:latest .

# 3. Start 3-node network
docker-compose up -d

# 4. View logs
docker-compose logs -f node1
```

## Available Commands

### Using docker-run.bat (Windows)

```cmd
docker-run.bat build     # Build Docker image
docker-run.bat dev       # Run single mining node
docker-run.bat network   # Start 3-node network
docker-run.bat logs      # View logs
docker-run.bat stop      # Stop all nodes
docker-run.bat clean     # Remove everything
docker-run.bat cli       # Run CLI commands
```

### Direct Docker Commands

```powershell
# Build
docker build -t boundless-bls:latest .

# Run single node
docker run -d --name node1 -p 30333:30333 -p 9933:9933 boundless-bls:latest --dev

# View logs
docker logs -f node1

# Stop node
docker stop node1

# Remove node
docker rm node1
```

### Docker Compose Commands

```powershell
# Start network
docker-compose up -d

# View logs
docker-compose logs -f node1

# Stop network
docker-compose down

# Remove all data
docker-compose down -v
```

## Accessing the Blockchain

### RPC Endpoints

- **Single Node**: http://localhost:9933
- **Node 1**: http://localhost:9933
- **Node 2**: http://localhost:9934
- **Node 3**: http://localhost:9935

### Using CLI

```powershell
# Generate keypair
docker run --rm boundless-bls:latest boundless-cli keygen

# Or with batch script
docker-run.bat cli keygen
```

### Example API Calls

```powershell
# Get blockchain info (PowerShell)
Invoke-RestMethod -Uri http://localhost:9933/chain_info -Method Get

# Using curl
curl http://localhost:9933/chain_info
```

## Common Tasks

### View Mining Activity

```powershell
docker logs -f boundless-node | Select-String "Block mined"
```

### Monitor Resource Usage

```powershell
docker stats boundless-node
```

### Backup Blockchain Data

```powershell
# Create backup
docker run --rm -v boundless-data:/data -v ${PWD}/backups:/backup alpine tar czf /backup/blockchain.tar.gz /data

# Restore backup
docker run --rm -v boundless-data:/data -v ${PWD}/backups:/backup alpine tar xzf /backup/blockchain.tar.gz -C /
```

### Connect to Container Shell

```powershell
docker exec -it boundless-node /bin/bash
```

## Troubleshooting

### Port Already in Use

```powershell
# Find process using port 9933
netstat -ano | findstr :9933

# Kill the process
taskkill /PID <PID> /F
```

### Container Won't Start

```powershell
# Check logs
docker logs boundless-node

# Run in foreground to see errors
docker run --rm -p 30333:30333 -p 9933:9933 boundless-bls:latest --dev
```

### Clean Everything

```cmd
docker-run.bat clean
```

Or manually:
```powershell
docker rm -f $(docker ps -aq)
docker volume prune -f
docker system prune -a
```

## Network Configuration

### Single Node Ports

- **30333**: P2P networking
- **9933**: RPC endpoint

### Multi-Node Ports

| Node   | P2P Port | RPC Port |
|--------|----------|----------|
| Node 1 | 30333    | 9933     |
| Node 2 | 30334    | 9934     |
| Node 3 | 30335    | 9935     |

## Performance Tips

1. **Allocate More Resources**
   - Docker Desktop → Settings → Resources
   - Increase Memory to 8GB
   - Increase CPUs to 4

2. **Use SSD for Docker Data**
   - Docker Desktop → Settings → Resources → Advanced
   - Change Docker data location to SSD

3. **Limit Container Resources**
   ```powershell
   docker run -d --memory 2g --cpus 2 boundless-bls:latest
   ```

## Next Steps

- Read full documentation: [DOCKER.md](DOCKER.md)
- Explore CLI commands: `docker-run.bat cli --help`
- Set up production deployment
- Configure custom network topology

## Support

For issues:
1. Check container logs: `docker logs boundless-node`
2. Review Docker Desktop logs
3. Open GitHub issue with logs attached

## Files Created

- `Dockerfile` - Multi-stage build configuration
- `docker-compose.yml` - Multi-node network setup
- `.dockerignore` - Build optimization
- `DOCKER.md` - Complete Docker documentation
- `docker-run.bat` - Windows management script (this is optional)
- `README-DOCKER.md` - This quick start guide
