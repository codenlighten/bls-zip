# Boundless Enterprise E² Multipass

> **Enterprise Operating System for the Boundless Blockchain**

**Version:** 1.0.0
**Status:** Production Ready
**Last Updated:** November 16, 2025

---

## 📖 Table of Contents

- [Overview](#overview)
- [What is E² Multipass?](#what-is-e-multipass)
- [Architecture](#architecture)
- [Features](#features)
- [Technology Stack](#technology-stack)
- [Quick Start](#quick-start)
- [Documentation](#documentation)
- [Project Structure](#project-structure)
- [Security](#security)
- [Integration Status](#integration-status)
- [Development](#development)
- [Support](#support)

---

## Overview

**Boundless Enterprise E² Multipass** is a comprehensive enterprise operating system built on the Boundless BLS blockchain. It provides a unified access and control layer for enterprise blockchain applications with **post-quantum cryptography**, **CIVA 3-layer identity model**, and **application-aware wallets**.

### Key Differentiators

- **Not a Cryptocurrency Platform** - Enterprise blockchain for regulated business assets (IRSC/CRSC tokens)
- **Post-Quantum Security** - ML-DSA (Dilithium5) and ML-KEM (Kyber1024) cryptography
- **CIVA Identity Model** - 3-layer identity framework (Identity Proof, Risk & Compliance, Attributes)
- **Application-Aware Wallets** - Contextual wallets tied to business applications
- **Smart Contract Templates** - Pre-built templates for common business logic

---

## What is E² Multipass?

E² Multipass is an **Enterprise Operating System** that sits on top of the Boundless blockchain, providing:

### Core Components

#### 1. Identity & Attestation Layer
- KYC/AML verification with on-chain proof anchoring
- Multi-factor authentication (MFA) with TOTP
- Role-based access control (RBAC)
- Device attestation with hardware passes
- CIVA 3-layer identity model

#### 2. Wallet Service
- **Application-aware wallets** (not just addresses)
- Multi-asset support (8+ asset types)
- Encrypted private key storage (AES-256-GCM)
- Transaction building and signing with PQC
- UTXO-based transaction model

#### 3. Auth/SSO Layer
- Single sign-on for enterprise applications
- JWT-based session management
- API key management
- Rate limiting and security controls

#### 4. Application Registry
- Pluggable business application modules
- Permission management
- Activity tracking
- Cross-application workflows

#### 5. Asset & Market Layer
- Token issuance and management
- Internal trading capabilities
- Position tracking
- Asset definitions with metadata
- Support for: Native BLS, Equity, Utility, Governance, Carbon Credits, Rewards, Stablecoins

#### 6. Events & Reporting
- Real-time notifications
- Custom report generation
- Analytics and insights
- Audit trail

#### 7. Hardware Pass Integration
- NFC card support
- Secure element integration
- Device challenges and verification
- Physical-digital asset bridging

---

## Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                     E² Multipass Frontend                       │
│              (Next.js 14 + TypeScript + TailwindCSS)            │
└────────────────────────┬────────────────────────────────────────┘
                         │ HTTP/REST
┌────────────────────────┴────────────────────────────────────────┐
│                   E² Multipass Backend                          │
│                    (Rust + Axum + PostgreSQL)                   │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Identity   │  │   Wallet     │  │     Auth     │         │
│  │   Service    │  │   Service    │  │   Service    │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  Application │  │     Asset    │  │    Events    │         │
│  │   Service    │  │    Service   │  │   Service    │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Hardware   │  │   Keystore   │  │    Crypto    │         │
│  │   Service    │  │ (AES-256-GCM)│  │  (Dilithium) │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└────────────────────────┬────────────────────────────────────────┘
                         │ HTTP/REST
┌────────────────────────┴────────────────────────────────────────┐
│              Boundless HTTP REST Bridge                         │
│                    (Port 3001)                                  │
└────────────────────────┬────────────────────────────────────────┘
                         │ JSON-RPC 2.0
┌────────────────────────┴────────────────────────────────────────┐
│                  Boundless BLS Blockchain                       │
│           (Post-Quantum + SHA-3 PoW + UTXO Model)               │
└─────────────────────────────────────────────────────────────────┘
```

### Database Schema

**PostgreSQL Tables (20+):**
- Identity: `identity_profiles`, `multipass_credentials`, `multipass_sessions`
- Wallet: `wallet_accounts`, `wallet_balances`, `wallet_transactions`, `wallet_keys`
- Blockchain: `blockchain_transactions`, `sync_state`
- Attestations: `attestations`
- Applications: `application_modules`
- Assets: `asset_definitions`, `positions`, `trades`
- Events: `notifications`, `report_definitions`, `generated_reports`
- Hardware: `hardware_passes`, `hardware_challenges`

---

## Features

### ✅ Identity & Attestation
- KYC/AML verification (3 levels)
- Multi-factor authentication (TOTP)
- Proof anchoring on blockchain
- Identity attestations
- Document upload

### ✅ Wallet Management
- Multi-asset wallet (8+ types)
- Encrypted private keys (AES-256-GCM)
- Post-quantum cryptography
- Transaction building (UTXO)
- Transaction signing (PQC)
- Balance tracking
- Transaction history

### ✅ Authentication & Security
- JWT-based authentication
- Session management
- API key management
- Rate limiting
- CORS and security headers
- Argon2 password hashing

### ✅ Application Management
- Application registration
- Permission management
- Activity tracking
- Module marketplace

### ✅ Asset Management
- Asset definitions
- Token issuance
- Multi-asset balances
- Trading capabilities
- Position tracking

### ✅ Events & Reporting
- Real-time notifications
- Custom report generation
- Analytics queries
- Event filtering

### ✅ Hardware Integration
- NFC card registration
- Device challenges
- Secure element support

### ✅ Blockchain Integration
- HTTP REST client
- Transaction submission
- Balance queries
- Proof anchoring
- Block queries
- Multi-asset support

---

## Technology Stack

### Backend
- **Language:** Rust (Edition 2021)
- **Framework:** Axum 0.7
- **Database:** PostgreSQL + SQLx
- **Cryptography:**
  - Post-Quantum: pqcrypto-dilithium, pqcrypto-kyber
  - Encryption: aes-gcm (AES-256-GCM)
  - Hashing: sha3, argon2
  - Security: zeroize
- **Auth:** jsonwebtoken
- **HTTP:** reqwest
- **Runtime:** tokio

### Frontend
- **Framework:** Next.js 14
- **Language:** TypeScript
- **Styling:** TailwindCSS
- **Components:** shadcn/ui
- **State:** React hooks

### Infrastructure
- **Database:** PostgreSQL 14+
- **Container:** Docker + Docker Compose
- **Ports:**
  - Frontend: 3001
  - Backend: 8080
  - Boundless HTTP: 3001
  - Boundless RPC: 9933
  - Database: 5432

---

## Quick Start

### Prerequisites

- Rust 1.70+ (MSVC toolchain on Windows)
- Node.js 18+
- PostgreSQL 14+
- Docker + Docker Compose
- OpenSSL

### 1. Environment Setup

```bash
# Backend (.env)
DATABASE_URL=postgresql://postgres:password@localhost:5432/enterprise_db
JWT_SECRET=<generate with: openssl rand -hex 32>
MASTER_ENCRYPTION_KEY=<generate with: openssl rand -hex 32>
BOUNDLESS_HTTP_URL=http://localhost:3001

# Frontend (.env.local)
NEXT_PUBLIC_API_URL=http://localhost:8080
```

### 2. Database Setup

```bash
createdb enterprise_db
cd enterprise
sqlx migrate run
```

### 3. Start Boundless Blockchain

```bash
docker-compose up -d
docker-compose ps  # Verify
```

### 4. Start E² Backend

```bash
cd enterprise
cargo run --bin enterprise-server  # Dev mode
# OR
cargo build --release && ./target/release/enterprise-server  # Production
```

### 5. Start E² Frontend

```bash
cd enterprise/frontend
npm install
npm run dev  # Dev mode
# OR
npm run build && npm start  # Production
```

### 6. Access E² Multipass

**URL:** http://localhost:3001

**Default Admin:**
- Email: `yourfriends@smartledger.solutions`
- Password: `BoundlessTrust`

---

## Documentation

### Core Docs
- **[E2_INTEGRATION_IMPLEMENTATION.md](./E2_INTEGRATION_IMPLEMENTATION.md)** - Integration details
- **[DEPLOYMENT.md](./DEPLOYMENT.md)** - Deployment guide
- **[Frontend README](./frontend/README.md)** - Frontend docs

### Archived Docs
- **[docs/archive/](./docs/archive/)** - Historical documentation

---

## Project Structure

```
enterprise/
├── src/
│   ├── bin/server.rs           # Main entry point
│   ├── models.rs               # Database models
│   ├── services/               # Business logic
│   │   ├── identity.rs
│   │   ├── wallet.rs
│   │   ├── auth.rs
│   │   ├── applications.rs
│   │   ├── assets.rs
│   │   ├── events.rs
│   │   └── hardware.rs
│   ├── api/                    # REST endpoints
│   ├── blockchain/mod.rs       # Boundless HTTP client
│   ├── crypto/mod.rs           # Post-quantum crypto
│   ├── keystore/mod.rs         # Encrypted key storage
│   ├── transaction/            # Transaction builder/signer
│   ├── validation.rs
│   ├── rate_limit.rs
│   └── middleware.rs
├── migrations/                 # Database migrations
├── frontend/                   # Next.js frontend
└── docs/                       # Documentation
```

---

## Security

### Cryptography
- **Signatures:** ML-DSA (Dilithium5) - NIST Level 5
- **Key Encapsulation:** ML-KEM (Kyber1024) - NIST Level 5
- **Hashing:** SHA-3
- **Password:** Argon2id
- **Encryption:** AES-256-GCM

### Key Management
- Encrypted storage (AES-256-GCM)
- Master key from environment
- Automatic memory wiping
- Key rotation support

### Authentication
- JWT tokens
- TOTP-based 2FA
- Session management
- Rate limiting

### Network
- HTTPS/TLS (production)
- CORS configuration
- Security headers
- Input validation

---

## Integration Status

### Boundless Integration ✅
- HTTP REST client
- Transaction submission
- Balance queries
- Block queries
- **Proof anchoring for attestations** (enterprise/src/services/identity.rs)
- **Proof anchoring for asset transfers** (enterprise/src/services/asset.rs)
- **RPC proof verification endpoints** (rpc/src/server.rs, node/src/rpc_impl.rs)
- **HTTP proof verification API** (rpc/src/http_bridge.rs)
- Multi-asset support

### Post-Quantum ✅
- Real PQC (no stubs)
- Dilithium5 signing
- Kyber1024 KEM
- Address derivation

### Authentication & Security ✅
- **JWT verification middleware** (enterprise/src/api/mod.rs)
- Argon2 password hashing
- Session management
- Rate limiting
- API key management

### WASM Runtime ✅
- **wasmtime v16 compatibility** (wasm-runtime/src/runtime.rs)
- Fuel metering for gas accounting
- **Resource limiter for memory and stack limits**
- Pooling allocator for performance optimization

---

## Development

### Building
```bash
# Backend
cargo build

# Frontend
npm run build
```

### Testing
```bash
# Backend
cargo test

# Frontend
npm test
```

### Code Quality
```bash
cargo fmt
cargo clippy
npm run type-check
```

---

## Support

**Email:** yourfriends@smartledger.solutions
**Documentation:** See [docs/](./docs/)

---

## License

**Proprietary** - Copyright © 2025 Smart Ledger Solutions

---

## Version History

**v1.0.0** (November 16, 2025)
- ✅ E² Multipass implementation
- ✅ Post-quantum cryptography
- ✅ Boundless blockchain integration
- ✅ Multi-asset wallet
- ✅ Encrypted keystore

---

**Built with ❤️ by Smart Ledger Solutions**
