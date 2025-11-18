# Quick Start: BLS Explorer with Boundless Blockchain

**Ready to see your BLS Explorer connected to a real blockchain?**

---

## Prerequisites

✅ **Node.js** 18+ installed
✅ **Rust** 1.75+ installed (for blockchain node)
✅ **PostgreSQL** 14+ installed (for E2 Multipass - optional)

---

## Option 1: Quick Test with Mock Data (No Blockchain Required)

```bash
cd C:\Users\ripva\Downloads\bls_explorer-main\bls_explorer-main
npm install
npm run dev
```

Open http://localhost:3000

✅ Explorer will use mock data automatically
✅ All UI features work without blockchain

---

## Option 2: Full Integration with Live Blockchain

### Step 1: Start the Blockchain Node

**Terminal 1:**
```bash
cd C:\Users\ripva\Desktop\boundless-bls-platform
cargo run --release --bin boundless-node -- --dev --mining --rpc-external
```

**Expected Output:**
```
🚀 Starting Boundless BLS Node...
📡 RPC server listening on http://127.0.0.1:9933
⛏️  Mining enabled
✅ Node is ready
```

**Wait until you see**: `✅ Node is ready` (takes 1-2 minutes on first run)

### Step 2: Start the BLS Explorer

**Terminal 2:**
```bash
cd C:\Users\ripva\Downloads\bls_explorer-main\bls_explorer-main
npm run dev
```

**Expected Output:**
```
✔ Ready in 3.2s
➜ Local: http://localhost:3000
```

### Step 3: Open the Explorer

**Browser**: Navigate to http://localhost:3000

**What to expect:**
- ✅ Green "Live Data" indicator in top right
- ✅ Real block heights from blockchain
- ✅ Latest blocks table with actual mined blocks
- ✅ Network stats updating every 30 seconds
- ✅ Real timestamps and hashes

---

## Option 3: Full Stack with E2 Multipass (Advanced)

### Step 1: Start PostgreSQL

```bash
# Windows (if not running as service)
pg_ctl start

# Create database
createdb enterprise_db
```

### Step 2: Start Blockchain Node

**Terminal 1:**
```bash
cd C:\Users\ripva\Desktop\boundless-bls-platform
cargo run --release --bin boundless-node -- --dev --mining --rpc-external
```

### Step 3: Start E2 Multipass Backend

**Terminal 2:**
```bash
cd C:\Users\ripva\Desktop\boundless-bls-platform\enterprise

# Set environment variables
$env:DATABASE_URL="postgresql://localhost:5432/enterprise_db"
$env:JWT_SECRET="your-secret-key-here"
$env:BLOCKCHAIN_RPC_URL="http://localhost:9933"

# Run migrations
sqlx migrate run

# Start server
cargo run --bin enterprise-server
```

**Expected Output:**
```
🚀 Enterprise E2 Multipass Server
📡 Listening on http://127.0.0.1:8080
✅ Connected to blockchain at http://localhost:9933
```

### Step 4: Start BLS Explorer

**Terminal 3:**
```bash
cd C:\Users\ripva\Downloads\bls_explorer-main\bls_explorer-main
npm run dev
```

**Open**: http://localhost:3000

---

## Verification Checklist

### ✅ Blockchain Connection

**Check 1: Connection Status**
- Top right corner shows green dot with "Live Data"

**Check 2: Block Height**
- Block height number is updating
- Number matches blockchain node logs

**Check 3: Latest Blocks**
- Table shows real blocks with actual hashes
- Timestamps are recent (within last few minutes)
- Block numbers are sequential

### ✅ Auto-Refresh

**Test**: Wait 30 seconds
- Block height should update
- New blocks appear in table
- Green indicator stays active

### ✅ Error Handling

**Test**: Stop the blockchain node (Ctrl+C in Terminal 1)
- Explorer shows red "Disconnected" indicator
- Error message appears at top
- UI still works with mock data
- **Restart node**: Explorer reconnects automatically within 30 seconds

---

## Troubleshooting

### Problem: "Cannot connect to blockchain node"

**Solutions:**
1. Check if blockchain node is running: `curl http://localhost:9933`
2. Verify RPC is enabled: Look for `--rpc-external` flag
3. Check firewall: Allow connections on port 9933

### Problem: CORS errors in browser console

**Solution**: Add to `boundless-bls-platform/rpc/src/http_bridge.rs`:
```rust
.layer(
    CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(vec![AUTHORIZATION, CONTENT_TYPE])
)
```

### Problem: Slow loading

**Cause**: Fetching many blocks
**Solution**: Reduce block count in `.env.local`:
```bash
NEXT_PUBLIC_DEFAULT_BLOCK_COUNT=5
```

### Problem: TypeScript errors

**Solution**:
```bash
npm install
npm run type-check
```

---

## Testing the Integration

### Test 1: View Live Blocks

1. Go to http://localhost:3000
2. Look at "Latest Blocks" table
3. Click on a block number
4. Verify block details display

### Test 2: Search Functionality

1. Use search bar in header
2. Enter a block number (e.g., "1")
3. Verify it navigates to block detail page
4. Try searching for a transaction hash

### Test 3: Network Stats

1. Check "Block Height" card
2. Verify it matches blockchain node height
3. Check "Network Hashrate"
4. Check "Avg Block Time"

### Test 4: Auto-Refresh

1. Note current block height
2. Wait 30 seconds
3. Verify height increases
4. New blocks appear in table

---

## Environment Configuration

### Required Variables (`.env.local`)

```bash
# Blockchain
NEXT_PUBLIC_BLOCKCHAIN_RPC_URL=http://localhost:9933

# Optional
NEXT_PUBLIC_E2_API_URL=http://localhost:8080
NEXT_PUBLIC_ENABLE_WEBSOCKET=false
NEXT_PUBLIC_ENABLE_AUTH=true
```

### Feature Flags

**Use mock data** (no blockchain required):
```bash
NEXT_PUBLIC_ENABLE_BLOCKCHAIN=false
```

**Enable authentication**:
```bash
NEXT_PUBLIC_ENABLE_AUTH=true
NEXT_PUBLIC_E2_API_URL=http://localhost:8080
```

---

## Performance Tips

### Speed Up Development

1. **Use Mock Data**: Set `NEXT_PUBLIC_ENABLE_BLOCKCHAIN=false`
2. **Reduce Blocks**: Fetch fewer blocks on dashboard
3. **Disable Auto-Refresh**: Comment out `setInterval` in `app/page.tsx`

### Optimize Production

1. **Enable Caching**: Add React Query
2. **Use WebSocket**: Replace polling with real-time updates
3. **CDN**: Deploy Next.js to Vercel for edge caching

---

## Next Steps

✅ **Phase 1 Complete**: Dashboard connected to blockchain
✅ **Phase 2 Complete**: Block and transaction detail pages
✅ **Phase 3 Complete**: E² authentication
✅ **Phase 4 Complete**: Wallet features and advanced functionality
✅ **Phase 5 Complete**: Real-time updates and auto-refresh

**All Integration Phases Complete! 🎉**

See `README.md` for complete feature list and documentation.

---

## Support

**Issues?** Check:
- `README.md` - Complete project documentation
- This file (`QUICKSTART_INTEGRATION.md`) - Quick start guide
- `boundless-bls-platform/README.md` - Blockchain documentation
- `boundless-bls-platform/enterprise/README.md` - E² Multipass documentation

**Questions?** Open an issue or contact the integration team.

---

**Last Updated**: November 18, 2025
**Status**: All Phases Complete ✅ (100% Integration)
