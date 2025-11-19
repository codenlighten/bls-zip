# 🚨 URGENT: Update Docker Nodes to Fix Network Fork

## Problem

Your 3 Docker nodes are running **old code** that doesn't support ML-DSA transactions. When the other node sent an ML-DSA transaction and mined it into a block, your nodes rejected it with:

```
❌ Failed to add block #2031: Transaction has no inputs
```

This caused a **network fork** - your nodes are at block #2031+ but the other node is stuck at #2015.

## Root Cause

The transaction was serialized with `Signature::MlDsa(...)` but your old code doesn't know how to deserialize this variant, so the inputs appeared empty during validation.

## Solution

You need to **pull latest code and rebuild Docker containers** to get the ML-DSA support.

---

## Update Instructions

### Step 1: Stop All Running Containers

```bash
docker stop boundless-node1 boundless-node2 boundless-node3
docker rm boundless-node1 boundless-node2 boundless-node3
```

### Step 2: Pull Latest Code

```bash
cd /path/to/boundless-git-collab
git pull origin main
```

**Expected changes:**
- `cli/src/tx.rs` - ML-DSA signature support (commit 08ec61a)
- `TRANSACTION_TESTING.md` - New documentation (commit a88cdc5)

### Step 3: Rebuild Docker Image

```bash
docker build -t boundless-bls:latest .
```

**Build time:** ~10-15 minutes (compiles with latest code)

### Step 4: Delete Old Blockchain Data

**IMPORTANT:** You must delete old blockchain data because it's on a forked chain:

```bash
rm -rf ./docker-data/node1/db/*
rm -rf ./docker-data/node2/db/*
rm -rf ./docker-data/node3/db/*
```

### Step 5: Restart Containers

```bash
# Node 1
docker run -d \
  --name boundless-node1 \
  -p 30333:30333 \
  -p 9933:9933 \
  -p 3001:3001 \
  -v "$(pwd)/docker-data/node1:/data" \
  boundless-bls:latest

# Node 2
docker run -d \
  --name boundless-node2 \
  -p 30334:30333 \
  -p 9934:9933 \
  -p 3002:3001 \
  -v "$(pwd)/docker-data/node2:/data" \
  boundless-bls:latest

# Node 3
docker run -d \
  --name boundless-node3 \
  -p 30335:30333 \
  -p 9935:9933 \
  -p 3003:3001 \
  -v "$(pwd)/docker-data/node3:/data" \
  boundless-bls:latest
```

### Step 6: Verify Sync

Wait 30 seconds, then check:

```bash
docker logs boundless-node1 | tail -20
```

**Look for:**
```
✅ Connected to bootstrap peer
📩 Received NewBlock from 12D3KooWDqN55HjCA5DBJ8DPvhrB3XPkgrRZH54DqcC5uQ94P74r
🆕 Received new block #XXX from peer
```

### Step 7: Confirm Block Height Matches

```bash
curl http://localhost:3001/api/v1/chain/info | jq .height
```

**Should match the other node's height** (currently around 2015-2020).

---

## What Changed in the Update

### 1. ML-DSA Signature Support (`cli/src/tx.rs`)

**Before:**
```rust
// Only supported Ed25519 (32-byte keys)
let signing_key = SigningKey::from_bytes(&key_array)?;
```

**After:**
```rust
// Supports ML-DSA (2528 bytes) and Falcon (~1281 bytes)
let signature_fn: Box<dyn Fn(&[u8]) -> Result<Signature>> = 
    if priv_key_bytes.len() == 2528 {
        // ML-DSA-44
        Box::new(move |message: &[u8]| {
            let signer = MlDsa44::new()?;
            let sig_bytes = signer.sign(message, &secret_key)?;
            Ok(Signature::MlDsa(sig_bytes))
        })
    } else if priv_key_bytes.len() >= 1200 && priv_key_bytes.len() <= 1300 {
        // Falcon-512
        ...
    }
```

### 2. Public Key Loading

**Before:** Tried to derive public key from private key (doesn't work with liboqs)

**After:** Loads separate `.pub` file created by keygen

```rust
let pub_key_file = key_file.with_extension("pub");
let pub_key_hex = fs::read_to_string(&pub_key_file)?;
let public_key = hex::decode(pub_key_hex.trim())?;
```

### 3. RPC Response Parsing

**Before:** Expected `{success: bool, message: string}`

**After:** Expects `{tx_hash: string}` (actual RPC format)

---

## Verification Checklist

After updating, verify:

- [ ] All 3 containers running: `docker ps`
- [ ] Connected to bootstrap peer: `docker logs boundless-node1 | grep "Connected to bootstrap"`
- [ ] Syncing blocks: `docker logs boundless-node1 | grep "Received new block"`
- [ ] Same block height as other node: `curl http://localhost:3001/api/v1/chain/info`
- [ ] No "Transaction has no inputs" errors: `docker logs boundless-node1 | grep "Failed to add block"`

---

## Expected Timeline

- **Stop containers:** 10 seconds
- **Pull latest code:** 5 seconds
- **Rebuild Docker image:** 10-15 minutes
- **Delete old data:** 5 seconds
- **Restart containers:** 30 seconds
- **Sync to network:** 1-5 minutes

**Total:** ~20 minutes

---

## What to Expect After Update

1. **Fresh blockchain:** Starts from genesis (block #0)
2. **Fast sync:** Will quickly catch up by downloading blocks from the other node
3. **ML-DSA support:** Can now validate and mine blocks with post-quantum transactions
4. **Network unity:** All 4 nodes on same chain again

---

## Testing After Update

Once synced, you can test ML-DSA transactions:

```bash
# Generate ML-DSA wallet
./target/release/boundless-cli keygen --algorithm ml-dsa --output ./test-wallet

# Check balance (if you've mined any blocks)
curl http://localhost:3001/api/v1/balance/YOUR_ADDRESS

# Send transaction
./target/release/boundless-cli send RECIPIENT_ADDRESS 100000000 --key ./test-wallet.priv
```

---

## Questions?

If you see any errors after updating:

1. Check Docker logs: `docker logs boundless-node1 --tail 100`
2. Verify network connectivity: `docker exec boundless-node1 ping -c 3 70.32.195.180`
3. Check if ports are exposed: `docker port boundless-node1`
4. Restart containers: `docker restart boundless-node1 boundless-node2 boundless-node3`

The other node is ready and waiting for you to sync!
