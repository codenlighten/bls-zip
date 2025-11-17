# Boundless E² Multipass - Backend Integration Guide

This document explains how the E² Multipass frontend integrates with the Boundless blockchain backend services.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    E² Multipass Frontend                     │
│                      (Next.js 14)                            │
└──────────────────────┬──────────────────────────────────────┘
                       │ REST API / WebSocket
                       │
┌──────────────────────▼──────────────────────────────────────┐
│              Enterprise Backend Services                     │
│   (Identity, Wallet, Contracts, Markets, AI, etc.)         │
└──────────────────────┬──────────────────────────────────────┘
                       │ RPC / Native Calls
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                 Boundless Blockchain Node                    │
│         (Post-Quantum, SHA-3 PoW, UTXO Model)               │
└─────────────────────────────────────────────────────────────┘
```

## Configuration

### Environment Variables

Copy `.env.example` to `.env.local` and configure:

```bash
# Main backend API endpoint
NEXT_PUBLIC_API_URL=http://localhost:8080

# Direct blockchain node RPC (optional)
NEXT_PUBLIC_BLOCKCHAIN_RPC=http://localhost:9933

# WebSocket for real-time updates
NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws
```

### Backend Service Ports

Default ports for backend services:
- **Main API Gateway**: 8080
- **Blockchain Node RPC**: 9933
- **Blockchain Node P2P**: 30333
- **WebSocket Server**: 8080/ws

## API Integration

### Authentication Flow

**File**: `src/lib/api.ts:141-172`

```
1. User submits email/password
2. Frontend → POST /api/auth/login
3. Backend validates credentials
4. Backend creates session with blockchain-signed token
5. Frontend stores token + identity profile
6. All subsequent requests include Bearer token
```

**Key Functions**:
- `api.login()` - Authenticate with email/password
- `api.logout()` - Invalidate session
- `setToken()` - Store JWT token
- All requests automatically include Authorization header

### Identity & CIVA Integration

**Files**:
- `src/app/(authenticated)/identity/page.tsx`
- `src/lib/api.ts:226-255`

**Endpoints**:
- `GET /api/identity/:id` - Fetch identity profile
- `GET /api/identity/:id/attestations` - Get CIVA attestations (3 layers)
- `POST /api/identity/attestations` - Create new attestation
- `DELETE /api/identity/attestations/:id` - Revoke attestation

**3-Layer CIVA Model**:
1. **Layer 1 - Identity Proof**: KYC, biometrics, government IDs
2. **Layer 2 - Risk & Compliance**: AML, sanctions, PEP, fraud scoring
3. **Layer 3 - Attributes**: Credentials, licenses, memberships

All attestations are cryptographically signed and can be anchored on-chain via `chain_anchor_tx`.

### Wallet & Assets Integration

**Files**:
- `src/app/(authenticated)/wallet/page.tsx`
- `src/lib/api.ts:257-351`

**Key Features**:
- **Real-time balance sync** from blockchain node
- **Multi-asset support**: Native BLS, utility tokens, NFTs, equities, carbon credits
- **Application-aware wallet**: Per-app mini dashboards
- **Transaction history** with enterprise operation types

**Endpoints**:
- `GET /api/wallets/identity/:id` - Get all wallets for identity
- `GET /api/wallets/:id/balances` - Get current balances from blockchain
- `GET /api/wallets/:id/transactions` - Transaction history
- `POST /api/wallets/:id/sync` - Force sync with blockchain node

**Balance Syncing**:
The frontend calls `syncWalletBalance()` which triggers the backend to:
1. Query blockchain node for current UTXO set
2. Calculate available + locked balances
3. Update database cache
4. Return real-time balance data

### Smart Contracts Integration

**Files**:
- `src/app/(authenticated)/contracts/page.tsx`
- `src/lib/api.ts:353-433`

**Contract Lifecycle**:
1. **Template Selection** - Browse verified contract templates
2. **Parameter Configuration** - Fill contract parameters
3. **Party Management** - Add signatories with identity verification
4. **Deployment** - Deploy to Boundless blockchain with on-chain code hash
5. **Execution** - Execute contract functions with cryptographic signatures

**Endpoints**:
- `GET /api/contracts/templates` - List contract templates
- `POST /api/contracts/deploy` - Deploy contract to blockchain
- `POST /api/contracts/:id/sign` - Sign contract with identity
- `POST /api/contracts/:id/execute` - Execute contract function

**Natural Language + Code Hash**:
Each contract has both human-readable terms AND cryptographic code hash for verification.

### Document Storage Integration

**Files**:
- `src/app/(authenticated)/documents/page.tsx`
- `src/lib/api.ts:435-502`

**Features**:
- **Encrypted storage** with identity-bound keys
- **Immutability flags** for regulatory compliance
- **Permission management** (read/write/admin)
- **Secure messaging** with encrypted threads
- **On-chain anchoring** via SHA-3 hashes

**Endpoints**:
- `POST /api/documents/upload` - Upload encrypted document
- `GET /api/documents/:id` - Retrieve document (decrypts with user's key)
- `POST /api/documents/:id/permissions` - Grant access to another identity
- `POST /api/documents/threads` - Create encrypted message thread

### Internal Markets Integration

**Files**:
- `src/app/(authenticated)/trading/page.tsx`
- `src/lib/api.ts:726-769`

**NOT Cryptocurrency Trading**:
This is a regulated asset exchange for:
- Carbon credits
- Real estate tokens
- Private equity
- Utility tokens
- Healthcare subscriptions
- Event tickets

**Endpoints**:
- `POST /api/markets/listings` - Create asset listing with compliance requirements
- `GET /api/markets/listings` - Browse by sector/jurisdiction
- `POST /api/markets/orders` - Place order (triggers blockchain settlement)
- `GET /api/markets/assets/:id` - Get asset metadata

**Settlement Flow**:
1. Order placed via API
2. Backend validates compliance requirements
3. Blockchain transaction created (UTXO-based)
4. Both parties sign with post-quantum signatures
5. Settlement tx broadcast to blockchain
6. Ownership transferred atomically

### Advanced Features Integration

#### Identity-Bound Compute Sessions (IBC)

**File**: `src/app/(authenticated)/development/page.tsx`
**API**: `src/lib/api.ts:771-812`

Records terminal sessions, API calls, git operations with cryptographic signatures:
- `POST /api/ibc/sessions` - Start recording session
- `POST /api/ibc/sessions/:id/events` - Log event with signature
- `GET /api/ibc/sessions/:id/playback` - Retrieve full session timeline
- `PUT /api/ibc/sessions/:id/close` - Close and anchor on-chain

#### Portable Developer Environments (PDE)

**File**: `src/app/(authenticated)/development/page.tsx`
**API**: `src/lib/api.ts:875-921`

Identity-bound environment snapshots:
- `POST /api/pde/snapshots` - Capture environment config
- `POST /api/pde/snapshots/:id/deploy` - Deploy to infrastructure
- Backend manages Docker/K8s provisioning

#### Code Signing & Provenance

**File**: `src/app/(authenticated)/development/page.tsx`
**API**: `src/lib/api.ts:923-961`

Post-quantum code signing:
- `POST /api/provenance/sign` - Sign artifact with ML-DSA-44
- `GET /api/provenance/chain/:hash` - Get full provenance chain
- `POST /api/provenance/artifacts/:hash/build` - Attach build metadata

#### AI Agent Governance

**File**: `src/app/(authenticated)/ai-agents/page.tsx`
**API**: `src/lib/api.ts:814-873`

AI agents as first-class citizens:
- `POST /api/ai/agents` - Register AI agent with identity
- `POST /api/ai/agents/:id/tokens` - Issue capability token
- `PUT /api/ai/tokens/:id/revoke` - Revoke token
- `GET /api/ai/agents/:id/activity` - Audit log

All agent actions are cryptographically logged and tied to the owner's identity.

#### Enterprise Knowledge Vault

**File**: `src/app/(authenticated)/knowledge/page.tsx`
**API**: `src/lib/api.ts:963-1026`

Identity-aware knowledge graph:
- `POST /api/knowledge/nodes` - Create knowledge node
- `POST /api/knowledge/search` - Semantic search (AI-powered)
- `POST /api/knowledge/nodes/:id/relations` - Link nodes
- Backend uses embedding model for semantic search

#### Collaboration Capsules

**File**: `src/app/(authenticated)/collaboration/page.tsx`
**API**: `src/lib/api.ts:1028-1097`

Zero-trust ephemeral collaboration:
- `POST /api/capsules` - Create collaboration space
- `POST /api/capsules/:id/participants` - Add participant (after identity lookup)
- `POST /api/capsules/:id/resources` - Share resource
- `GET /api/capsules/:id/activity` - Audit trail

All actions enforced by capability-based policies.

### NFC/Hardware Pass Integration

**File**: `src/app/(authenticated)/identity/page.tsx:72-101`
**API**: `src/lib/api.ts:682-690`

**Key Exchange Flow**:
1. User initiates device registration in UI
2. Frontend sends request to `/api/hardware/register`
3. **Backend generates PQC keypair** (ML-KEM-768 or ML-DSA-44)
4. Backend provisions public key to hardware device via secure channel
5. Private key stored in hardware security module (HSM) or NFC chip
6. Frontend receives device info with public key

**Capabilities**:
- `login_only` - Basic authentication
- `sign_tx` - Transaction signing
- `unlock_doors` - Physical access control
- `access_control` - Full capabilities

**Offline Verification**:
NFC passes can verify identity offline using the on-device private key and cached blockchain state.

## Real-Time Updates (WebSocket)

**Endpoint**: `ws://localhost:8080/ws`

**Subscriptions**:
```javascript
// Connect
const ws = new WebSocket(process.env.NEXT_PUBLIC_WS_URL)

// Subscribe to balance updates
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: `wallet:${wallet_id}:balances`
}))

// Subscribe to new blocks
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'blockchain:blocks'
}))
```

**Events Received**:
- `balance_updated` - Wallet balance changed
- `transaction_confirmed` - TX confirmed in block
- `new_attestation` - CIVA attestation issued
- `contract_executed` - Smart contract executed
- `market_order_filled` - Order settled

## Blockchain Integration Points

### Direct Node Queries (Optional)

If `NEXT_PUBLIC_BLOCKCHAIN_RPC` is set, frontend can query node directly:

```javascript
// Get current block height
fetch(`${BLOCKCHAIN_RPC}/chain/height`)

// Get transaction by hash
fetch(`${BLOCKCHAIN_RPC}/tx/${hash}`)

// Get UTXO set for address
fetch(`${BLOCKCHAIN_RPC}/utxos/${address}`)
```

**Note**: Most queries should go through backend API for caching and business logic.

### Post-Quantum Signatures

All signatures use NIST PQC algorithms:
- **Key Encapsulation**: ML-KEM-768
- **Digital Signatures**: ML-DSA-44 (primary), Falcon-512 (backup)
- **Hashing**: SHA3-256

Signature verification happens in backend before blockchain submission.

### Transaction Broadcasting

**Flow**:
1. Frontend creates transaction object
2. Backend validates transaction
3. Backend signs with identity's PQC key
4. Backend broadcasts to blockchain node
5. Node validates and includes in block
6. Frontend receives confirmation via WebSocket

## Error Handling

All API responses follow this format:

```typescript
{
  data?: T,        // Success response
  error?: string,  // Error message
  message?: string // Additional info
}
```

**Frontend Error Handling**:
```typescript
const response = await api.someEndpoint()
if (response.error) {
  // Handle error - already logged to console
  alert(response.error)
} else if (response.data) {
  // Process data
}
```

## Security Considerations

### Authentication
- JWT tokens expire after 1 hour (configurable)
- Tokens stored in localStorage (consider httpOnly cookies for production)
- All API requests include CSRF protection

### Authorization
- Identity-based access control
- Capability tokens for AI agents
- Permission system for documents and capsules
- All actions audited and logged

### Cryptography
- All sensitive data encrypted with identity keys
- Hardware passes use HSM-backed keys
- Blockchain signatures use post-quantum algorithms
- Document hashes use SHA3-256

### Compliance
- KYC status checked before market operations
- AML risk scoring enforced
- Jurisdiction restrictions on asset trading
- All transactions permanently logged on blockchain

## Testing Backend Integration

### Start Backend Services

```bash
# Start Boundless blockchain node
cd boundless-node
./target/release/boundless-node --dev

# Start enterprise backend
cd enterprise-backend
cargo run --release

# Backend should connect to node at localhost:9933
# API available at localhost:8080
```

### Test API Endpoints

```bash
# Health check
curl http://localhost:8080/health

# Create test identity
curl -X POST http://localhost:8080/api/identity \
  -H "Content-Type: application/json" \
  -d '{
    "legal_name": "Test User",
    "email": "test@example.com",
    "password": "secure_password"
  }'

# Login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "secure_password"
  }'
```

### Test Frontend Integration

```bash
# Install dependencies
cd enterprise/frontend
npm install

# Copy environment file
cp .env.example .env.local

# Start development server
npm run dev

# Open http://localhost:3000
```

## Production Deployment

### Backend Requirements
- Boundless blockchain node (running and synced)
- Enterprise backend services (Rust)
- PostgreSQL database for caching
- Redis for session management (optional)

### Frontend Requirements
- Node.js 18+
- Next.js production build
- HTTPS with valid SSL certificate
- CDN for static assets (optional)

### Environment Variables (Production)
```bash
NEXT_PUBLIC_API_URL=https://api.boundless.example.com
NEXT_PUBLIC_BLOCKCHAIN_RPC=https://rpc.boundless.example.com
NEXT_PUBLIC_WS_URL=wss://api.boundless.example.com/ws
```

## Monitoring & Observability

### Frontend Metrics
- Page load times
- API response times
- WebSocket connection status
- User session duration

### Backend Metrics
- Blockchain sync status
- Transaction confirmation times
- API endpoint latency
- Database query performance

### Blockchain Metrics
- Block height
- Network hashrate
- Transaction pool size
- UTXO set size

## Troubleshooting

### "Failed to load data" errors
- Check `NEXT_PUBLIC_API_URL` is correct
- Verify backend is running on port 8080
- Check browser console for CORS errors

### "Session expired" errors
- Token timeout (default 1 hour)
- User needs to login again
- Check backend session management

### Balance sync issues
- Backend may be out of sync with blockchain
- Call `POST /api/wallets/:id/sync` to force refresh
- Check blockchain node is running and synced

### Transaction not confirming
- Check blockchain node logs
- Verify transaction was broadcast
- Check if UTXO was already spent
- Verify PQC signature is valid

## API Reference

Complete API documentation available at:
- Swagger UI: `http://localhost:8080/swagger`
- OpenAPI spec: `http://localhost:8080/api-docs`

## Support

For integration issues:
1. Check backend logs: `tail -f enterprise-backend/logs/app.log`
2. Check blockchain logs: `tail -f boundless-node/logs/node.log`
3. Enable debug mode: `NEXT_PUBLIC_ENABLE_DEBUG_LOGS=true`
4. Review this integration guide
5. Contact the Boundless development team
