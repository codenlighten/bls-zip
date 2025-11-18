# Changelog

All notable changes to the Boundless BLS Platform will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub Actions CI/CD workflows
- Issue and PR templates
- Community files (CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, SUPPORT)
- CODEOWNERS file
- Dependabot configuration
- Production ready badges for all blockchain phases

### Changed
- Updated README with complete RPC endpoint documentation (13+ methods)

### Fixed
- N/A

### Security
- N/A

## [0.1.0] - 2025-01-XX

### Added

#### Blockchain Core
- Post-quantum cryptography (ML-KEM-768, ML-DSA-44, Falcon-512)
- Hybrid signature schemes (classical + PQC)
- SHA-3 based Proof-of-Work consensus
- WASM smart contract execution with fuel metering
- Paillier homomorphic encryption for privacy
- UTXO state management with nonce-based replay protection

#### Production Infrastructure
- JSON-RPC 2.0 API with 13+ methods
  - Core endpoints: `chain_getBlockHeight`, `chain_getInfo`, `chain_getBlockByHeight`, `chain_getBlockByHash`
  - Transaction endpoints: `chain_getTransaction`, `chain_submitTransaction`, `chain_getUtxos`
  - Proof verification: `chain_getProof`, `chain_verifyProof`, `chain_getProofsByIdentity`
  - System endpoints: `chain_getBalance`, `system_health`, `system_version`
- RocksDB persistent storage with 4 column families
- libp2p P2P networking with gossipsub and mDNS
- Transaction mempool with fee-based ordering
- Full node integration

#### Network Synchronization
- Automatic block broadcasting to all peers
- Network-wide transaction propagation
- Automatic block sync for missing blocks
- Peer status tracking and discovery
- Multi-node distributed operation

#### Enterprise E² Multipass
- CIVA 3-layer identity model
- KYC/AML verification with blockchain anchoring
- Application-aware multi-asset wallets
- 8+ asset types support
- Post-quantum cryptography (ML-DSA, ML-KEM)
- AES-256-GCM encrypted keystore
- Blockchain proof anchoring
- Proof verification endpoints (RPC + REST)
- JWT authentication with Argon2id hashing
- Hardware pass integration support
- Application registry with permissions
- Real-time event system
- Smart contract templates
- PostgreSQL backend (20+ tables)
- Next.js admin frontend

#### BLS Blockchain Explorer
- Real-time blockchain visualization
- Block explorer with navigation
- Multi-signature transaction tracking
- UTXO visualization
- Network statistics dashboard
- E² Multipass authentication
- Identity and proof anchor explorer
- Post-quantum cryptography support
- Next.js 14 + TypeScript frontend
- Mock data fallback mode
- 30-second auto-refresh

### Changed
- N/A (Initial release)

### Fixed
- N/A (Initial release)

### Security
- NIST-standardized post-quantum cryptography
- Hybrid cryptographic schemes
- Secure key storage and management
- Input validation and sanitization
- SQL injection protection
- XSS prevention
- CSRF protection
- TLS 1.3 for all communications

---

## Versioning Scheme

- **Major version (X.0.0)**: Breaking changes, major features
- **Minor version (0.X.0)**: New features, backward compatible
- **Patch version (0.0.X)**: Bug fixes, security patches

## Categories

- **Added**: New features
- **Changed**: Changes in existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Now removed features
- **Fixed**: Bug fixes
- **Security**: Security fixes and improvements

---

[Unreleased]: https://github.com/Saifullah62/BLS/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Saifullah62/BLS/releases/tag/v0.1.0
