# Security Policy

## Our Commitment

The Boundless BLS Platform team takes security seriously. We appreciate the efforts of security researchers and users who help us maintain the security of our platform.

## Supported Versions

We provide security updates for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

### How to Report

If you discover a security vulnerability, please report it by one of the following methods:

1. **Email**: security@boundlesstrust.org
2. **GitHub Security Advisory**: https://github.com/Saifullah62/BLS/security/advisories/new

### What to Include

Please include the following information in your report:

- **Description**: Brief description of the vulnerability
- **Component**: Which component is affected (blockchain, RPC, E² Multipass, Explorer, etc.)
- **Impact**: Potential impact of the vulnerability
- **Steps to Reproduce**: Detailed steps to reproduce the issue
- **Proof of Concept**: PoC code or exploit (if applicable)
- **Suggested Fix**: Potential solutions (if you have any)
- **Your Contact Information**: For follow-up questions

### Example Report Template

```
Subject: [SECURITY] Brief description

Component: Blockchain Core / RPC / E² Multipass / Explorer
Severity: Critical / High / Medium / Low

Description:
[Detailed description of the vulnerability]

Steps to Reproduce:
1. Step one
2. Step two
3. Step three

Impact:
[Describe the potential impact]

Proof of Concept:
[Code or detailed explanation]

Suggested Fix:
[If you have suggestions]
```

## Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 1 week
- **Status Update**: Weekly until resolved
- **Fix Timeline**: Varies by severity
  - **Critical**: 7-14 days
  - **High**: 14-30 days
  - **Medium**: 30-60 days
  - **Low**: 60-90 days

## Disclosure Policy

We follow a **coordinated disclosure** approach:

1. **You report** the vulnerability privately
2. **We confirm** and assess the vulnerability
3. **We develop** a fix in a private security fork
4. **We release** a security patch
5. **We publicly disclose** the vulnerability (with credit to you, if desired)

### Public Disclosure Timeline

- We will coordinate with you on the disclosure timeline
- Default disclosure: **90 days** after report or patch release (whichever comes first)
- Earlier disclosure may occur if:
  - A fix is available and deployed
  - The vulnerability is publicly known
  - We mutually agree to earlier disclosure

## Scope

### In Scope

The following components are in scope for security reports:

- **Blockchain Core** (`core/`)
  - Consensus mechanism
  - Transaction validation
  - Cryptographic implementations
  - State management

- **RPC Server** (`rpc/`)
  - API endpoints
  - Authentication/authorization
  - Input validation
  - Rate limiting

- **P2P Networking** (`node/`)
  - Peer discovery
  - Message propagation
  - Network security

- **E² Multipass** (`enterprise/`)
  - Identity management
  - KYC/AML verification
  - Wallet security
  - Database security
  - API security

- **BLS Explorer** (`BLS_Explorer/`)
  - XSS vulnerabilities
  - Data exposure
  - Authentication bypasses

### Out of Scope

The following are generally out of scope:

- Vulnerabilities in third-party dependencies (report to the maintainers)
- Social engineering attacks
- Denial of Service (DoS) attacks against public instances
- Issues that require physical access to a user's device
- Issues in unsupported versions

## Security Best Practices

### For Users

- **Keep software updated** to the latest version
- **Use strong passwords** for wallet encryption
- **Enable 2FA** where available
- **Verify signatures** of downloaded binaries
- **Backup private keys** securely
- **Don't share secrets** (private keys, mnemonics)

### For Developers

- **Review code** before committing
- **Use static analysis** tools (clippy, eslint)
- **Write tests** including security test cases
- **Follow secure coding** guidelines
- **Keep dependencies** up to date
- **Use secrets management** for sensitive data

## Security Features

### Post-Quantum Cryptography
- ML-DSA-44 (Dilithium)
- ML-KEM-768 (Kyber)
- Falcon-512
- Hybrid schemes (classical + PQC)

### Secure Storage
- AES-256-GCM encryption
- Argon2id password hashing
- Encrypted keystore
- Hardware security module support

### Network Security
- TLS 1.3 for all communications
- Peer authentication
- Message signing
- DDoS protection

### Application Security
- JWT authentication
- Input validation
- SQL injection protection
- XSS prevention
- CSRF protection

## Bug Bounty Program

We are currently evaluating options for a bug bounty program. Stay tuned for updates.

## Recognition

We appreciate responsible disclosure and will:

- **Credit researchers** in security advisories (with permission)
- **Maintain a hall of fame** for security researchers
- **Provide swag** for significant findings (when budget allows)

## Security Hall of Fame

*To be updated as security researchers contribute*

## Questions?

For general security questions (not vulnerability reports):
- Open a [Discussion](https://github.com/Saifullah62/BLS/discussions)
- Email: security@boundlesstrust.org

---

**Remember**: If you discover a security vulnerability, please report it privately. Public disclosure before a fix can put users at risk.

Thank you for helping keep Boundless BLS and our users safe!
