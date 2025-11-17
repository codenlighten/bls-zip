# Boundless Enterprise Multipass - API Documentation

## Overview

The Enterprise Multipass API is a RESTful API built on Axum (Rust) that provides comprehensive enterprise blockchain functionality. All endpoints return JSON responses and use JWT authentication (except login/register).

## Quick Start

### View Interactive API Documentation

The OpenAPI/Swagger specification is available at:
```
docs/openapi.yaml
```

To view interactively, use Swagger UI:
```bash
# Using Docker
docker run -p 8081:8080 -v $(pwd)/docs:/docs swaggerapi/swagger-ui

# Or use online editor
# Upload openapi.yaml to https://editor.swagger.io
```

### Base URL

```
http://localhost:8080  # Development
https://api.boundless.example.com  # Production
```

### Authentication

All endpoints (except `/api/auth/login` and `/api/auth/register`) require a JWT bearer token:

```http
Authorization: Bearer <your_jwt_token>
```

## API Modules

### 1. Authentication (`/api/auth`)

User authentication and session management.

**Endpoints:**
- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login and get JWT token (rate limited)
- `POST /api/auth/logout` - Invalidate session
- `POST /api/auth/refresh/:session_id` - Refresh JWT token
- `POST /api/auth/verify` - Verify token validity
- `GET /api/auth/session/:session_id` - Get session details
- `GET /api/auth/sessions/:identity_id` - List user sessions

**Example: Login**
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "secure_password_123"
  }'
```

Response:
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "identity_id": "7d7f8db0-1234-5678-9abc-def012345678",
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_at": "2025-11-17T12:00:00Z"
}
```

### 2. Identity (`/api/identity`)

KYC/AML identity profiles and attestations.

**Endpoints:**
- `POST /api/identity/create` - Create identity profile
- `GET /api/identity/:id` - Get identity by ID
- `GET /api/identity/email/:email` - Get identity by email
- `PUT /api/identity/:id/kyc-status` - Update KYC status
- `POST /api/identity/:id/attestations` - Create attestation
- `GET /api/identity/:id/attestations` - List attestations
- `DELETE /api/identity/attestations/:attestation_id` - Revoke attestation
- `GET /api/identity/list` - List all identities (admin)

**Example: Update KYC Status**
```bash
curl -X PUT http://localhost:8080/api/identity/7d7f8db0.../kyc-status \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"status": "Verified"}'
```

### 3. Wallet (`/api/wallet`)

Blockchain wallet management with PQC (Dilithium5 + Kyber1024).

**Endpoints:**
- `POST /api/wallet/create` - Create wallet (generates PQC keypair)
- `GET /api/wallet/:id` - Get wallet details
- `GET /api/wallet/:id/balances` - Get blockchain balances
- `GET /api/wallet/:id/transactions` - Get transaction history
- `POST /api/wallet/:id/transfer` - Transfer blockchain assets
- `POST /api/wallet/:id/sync` - Sync balances from blockchain
- `GET /api/wallet/identity/:identity_id` - List identity's wallets

**Key Features:**
- **Post-Quantum Cryptography**: Dilithium5 (ML-DSA) signatures + Kyber1024 (ML-KEM) encryption
- **32-byte Addresses**: Full SHA3-256 hash (64 hex characters)
- **Encrypted Keystore**: Private keys encrypted with master key (AES-256-GCM)

**Example: Create Wallet**
```bash
curl -X POST http://localhost:8080/api/wallet/create \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "identity_id": "7d7f8db0-1234-5678-9abc-def012345678",
    "wallet_type": "Personal"
  }'
```

Response:
```json
{
  "wallet_id": "a1b2c3d4-5678-90ab-cdef-1234567890ab",
  "identity_id": "7d7f8db0-1234-5678-9abc-def012345678",
  "wallet_type": "Personal",
  "boundless_address": "f4a8e2c9d...64_hex_chars...3b7c8f1a2e",
  "created_at": "2025-11-16T10:30:00Z"
}
```

**Example: Transfer Assets**
```bash
curl -X POST http://localhost:8080/api/wallet/a1b2c3d4.../transfer \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "to_address": "7f3e9a1b...destination_address...4c2d8e6f",
    "amount": 1000000
  }'
```

Response:
```json
{
  "tx_hash": "8a9b0c1d...transaction_hash...7e8f9a0b"
}
```

### 4. Assets (`/api/assets`)

Enterprise asset definitions (tokens, carbon credits, NFTs).

**Endpoints:**
- `POST /api/assets/define` - Define new asset
- `GET /api/assets/list` - List all assets
- `GET /api/assets/:asset_id` - Get asset details
- `POST /api/assets/:asset_id/issue` - Issue tokens to wallet
- `POST /api/assets/:asset_id/transfer` - Transfer with blockchain settlement
- `GET /api/assets/:asset_id/balance/:wallet_id` - Get balance

**Asset Types:**
- `Native` - Platform native token
- `UtilityToken` - Service access tokens
- `EquityToken` - Tokenized equity
- `CarbonCredit` - Carbon offset credits
- `NFT` - Non-fungible tokens
- `SubscriptionPass` - Time-based access passes

**Example: Define Asset**
```bash
curl -X POST http://localhost:8080/api/assets/define \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Carbon Credit Token",
    "symbol": "CCT",
    "asset_type": "CarbonCredit",
    "total_supply": 1000000
  }'
```

**Example: Transfer with Blockchain Settlement**
```bash
curl -X POST http://localhost:8080/api/assets/{asset_id}/transfer \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "from_wallet_id": "a1b2c3d4...",
    "to_wallet_id": "e5f6g7h8...",
    "quantity": 500
  }'
```

Response includes blockchain transaction hash:
```json
{
  "tx_hash": "9c8d7e6f...blockchain_tx...5a4b3c2d"
}
```

### 5. Market (`/api/market`)

Internal asset trading with automatic order matching.

**Endpoints:**
- `POST /api/market/orders` - Create buy/sell order
- `GET /api/market/orders/:order_id` - Get order details
- `PUT /api/market/orders/:order_id/cancel` - Cancel order
- `GET /api/market/wallet/:wallet_id/orders` - List wallet orders
- `GET /api/market/orderbook/:asset_id` - Get order book (bids/asks)
- `GET /api/market/positions/:wallet_id` - Get positions with locked quantities
- `GET /api/market/trades/:asset_id` - Get trade history

**Features:**
- **Automatic Order Matching**: Runs every 5 seconds (configurable)
- **Locked Quantities**: Tracks available vs locked tokens
- **Blockchain Settlement**: Trades settled on-chain with metadata

**Example: Create Buy Order**
```bash
curl -X POST http://localhost:8080/api/market/orders \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "wallet_id": "a1b2c3d4...",
    "asset_id": "b2c3d4e5...",
    "order_type": "Buy",
    "quantity": 100,
    "price": 1500
  }'
```

**Example: Get Order Book**
```bash
curl -X GET http://localhost:8080/api/market/orderbook/b2c3d4e5... \
  -H "Authorization: Bearer $TOKEN"
```

Response:
```json
{
  "bids": [
    {"price": 1500, "quantity": 100, "order_id": "..."},
    {"price": 1480, "quantity": 200, "order_id": "..."}
  ],
  "asks": [
    {"price": 1520, "quantity": 150, "order_id": "..."},
    {"price": 1550, "quantity": 300, "order_id": "..."}
  ]
}
```

**Example: Get Positions**
```bash
curl -X GET http://localhost:8080/api/market/positions/a1b2c3d4... \
  -H "Authorization: Bearer $TOKEN"
```

Response shows available and locked quantities:
```json
[
  {
    "asset_id": "b2c3d4e5...",
    "quantity": 1000,
    "locked_quantity": 100,
    "average_cost": 1450
  }
]
```

### 6. Applications (`/api/applications`)

Enterprise application registry for multi-tenant management.

**Endpoints:**
- `POST /api/applications/register` - Register application
- `GET /api/applications/list` - List applications
- `GET /api/applications/:app_id` - Get application
- `PUT /api/applications/:app_id/enable` - Enable/disable app
- `PUT /api/applications/:app_id/update` - Update app config
- `DELETE /api/applications/:app_id/delete` - Delete application
- `POST /api/applications/:app_id/events` - Log application event
- `GET /api/applications/:app_id/events/list` - Get app events
- `GET /api/applications/identity/:identity_id/events` - Get user events

### 7. Hardware Pass (`/api/hardware`)

NFC hardware device management for secure authentication.

**Endpoints:**
- `POST /api/hardware/register` - Register NFC device
- `GET /api/hardware/:device_id` - Get device details
- `DELETE /api/hardware/:device_id` - Delete device
- `GET /api/hardware/identity/:identity_id` - List user devices
- `POST /api/hardware/:device_id/authenticate` - Authenticate with device
- `POST /api/hardware/:device_id/challenge` - Generate challenge
- `PUT /api/hardware/:device_id/revoke` - Revoke device
- `PUT /api/hardware/:device_id/lost` - Mark device as lost
- `POST /api/hardware/:device_id/capability` - Check device capability
- `GET /api/hardware/:device_id/stats` - Get usage statistics

**Example: Challenge-Response Authentication**
```bash
# Step 1: Generate challenge
curl -X POST http://localhost:8080/api/hardware/{device_id}/challenge \
  -H "Authorization: Bearer $TOKEN"

# Response: {"challenge": "f3e9a2b..."}

# Step 2: Sign challenge with hardware device (offline)

# Step 3: Submit signed response
curl -X POST http://localhost:8080/api/hardware/{device_id}/authenticate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "challenge_response": "8c7d6e5f...signed_challenge..."
  }'
```

### 8. Events & Notifications (`/api/events`)

User notifications and enterprise reporting.

**Endpoints:**
- `POST /api/events/` - Create notification
- `GET /api/events/:notification_id` - Get notification
- `PUT /api/events/:notification_id/read` - Mark as read
- `DELETE /api/events/:notification_id` - Delete notification
- `GET /api/events/identity/:identity_id` - Get user notifications
- `GET /api/events/identity/:identity_id/unread` - Get unread count
- `PUT /api/events/identity/:identity_id/mark_all_read` - Mark all read

**Reporting:**
- `POST /api/reporting/definitions` - Create report template
- `GET /api/reporting/definitions` - List report templates
- `GET /api/reporting/definitions/:report_id` - Get template
- `DELETE /api/reporting/definitions/:report_id` - Delete template
- `POST /api/reporting/generate` - Generate report (DISABLED - security)
- `GET /api/reporting/:generated_report_id` - Get generated report
- `GET /api/reporting/identity/:identity_id` - List user reports

**Note:** Custom SQL report generation is currently disabled due to SQL injection vulnerability. Use predefined report types only.

## Error Handling

All errors return standard JSON format:

```json
{
  "error": "Error message describing what went wrong"
}
```

Common HTTP Status Codes:
- `200` - Success
- `400` - Bad Request (invalid input)
- `401` - Unauthorized (missing/invalid token)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found
- `429` - Too Many Requests (rate limit exceeded)
- `500` - Internal Server Error

## Rate Limiting

Login endpoint is rate limited to prevent brute force attacks:
- **Limit:** 5 failed attempts per IP address
- **Lockout:** 30 minutes

## Security Features

### Post-Quantum Cryptography
- **Signatures:** Dilithium5 (ML-DSA) - NIST standardized
- **Encryption:** Kyber1024 (ML-KEM) - NIST standardized
- **Addresses:** 32-byte SHA3-256 hash (64 hex characters)

### Private Key Security
- **Master Encryption:** AES-256-GCM with `MASTER_ENCRYPTION_KEY`
- **Storage:** Encrypted in PostgreSQL database
- **Key Rotation:** Supported via migration (see KEY_ROTATION_GUIDE.md)

### Blockchain Integration
- **Address Format:** 32-byte (64 hex) aligned with Boundless spec
- **Transaction Signing:** Dilithium5 PQC signatures
- **UTXO Model:** Full compatibility with Boundless blockchain
- **Asset Transfers:** On-chain settlement with JSON metadata

## Development Tools

### Swagger UI

View and test API interactively:

```bash
# Serve OpenAPI spec with Swagger UI
docker run -p 8081:8080 \
  -v $(pwd)/docs/openapi.yaml:/openapi.yaml \
  -e SWAGGER_JSON=/openapi.yaml \
  swaggerapi/swagger-ui
```

Visit: http://localhost:8081

### Postman Collection

Import `docs/openapi.yaml` into Postman for testing.

### Example Request Flow

```bash
# 1. Register user
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Test123!","full_name":"Test User"}' \
  | jq -r '.session.token')

# 2. Create wallet
WALLET=$(curl -s -X POST http://localhost:8080/api/wallet/create \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"identity_id":"...", "wallet_type":"Personal"}' \
  | jq -r '.wallet_id')

# 3. Check balances
curl -X GET http://localhost:8080/api/wallet/$WALLET/balances \
  -H "Authorization: Bearer $TOKEN"

# 4. Transfer assets
curl -X POST http://localhost:8080/api/wallet/$WALLET/transfer \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"to_address":"7f3e9a1b...","amount":1000}'
```

## Configuration

Key environment variables:

```bash
# API Server
BIND_ADDR=0.0.0.0:8080

# Database
DATABASE_URL=postgresql://user:pass@localhost:5432/boundless_enterprise

# Security
JWT_SECRET=your_secret_key_here
MASTER_ENCRYPTION_KEY=generate_with_openssl_rand_hex_32

# Blockchain
BOUNDLESS_HTTP_URL=http://localhost:9933

# Market
ENABLE_ORDER_MATCHING=true
ORDER_MATCHING_INTERVAL_SECS=5
```

See `.env.example` for complete configuration options.

## Further Reading

- [OpenAPI Specification](openapi.yaml) - Complete API spec
- [Setup Guide](../SETUP_GUIDE.md) - Development setup
- [Deployment Guide](../DEPLOYMENT.md) - Production deployment
- [Security Audit](../SECURITY_AUDIT_REPORT.md) - Security review
- [Key Rotation Guide](KEY_ROTATION_GUIDE.md) - Rotate encryption keys

## Support

For API issues or questions:
- GitHub Issues: [boundless-platform/enterprise/issues]
- Documentation: `docs/` directory
- Example Code: `examples/` directory
