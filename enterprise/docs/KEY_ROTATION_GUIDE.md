# Key Rotation Guide

## FIX L-8: Comprehensive Key Rotation Documentation

This guide provides detailed procedures for rotating cryptographic keys used in the Boundless Enterprise E² Multipass platform.

---

## Overview

The platform uses multiple types of cryptographic keys that require periodic rotation for security:

1. **JWT Signing Secret** - Signs session tokens
2. **Keystore Master Key** - Encrypts wallet private keys
3. **Database Encryption Keys** - Encrypts sensitive data at rest
4. **API Keys** - Third-party integrations

---

## 1. JWT Secret Rotation

### When to Rotate
- **Scheduled**: Every 90 days (recommended)
- **Emergency**: Immediately if secret is compromised
- **Compliance**: As required by security policies

### Prerequisites
- Database backup completed
- Maintenance window scheduled
- New secret generated using: `openssl rand -hex 32`
- Secret validated for entropy (see auth service validation)

### Rotation Process

#### Step 1: Generate New Secret
```bash
# Generate a new 32-byte (256-bit) secret
NEW_SECRET=$(openssl rand -hex 32)
echo "New JWT Secret: $NEW_SECRET"

# Validate entropy (must pass service validation)
# - No repeated characters > 75%
# - No weak patterns (password, test, etc.)
```

#### Step 2: Dual-Key Operation (Zero-Downtime)
```rust
// Modify AuthService to accept TWO secrets
pub struct AuthService {
    db: PgPool,
    jwt_secret_primary: String,   // NEW secret (for signing)
    jwt_secret_secondary: String, // OLD secret (for verification only)
}

impl AuthService {
    // Sign new tokens with PRIMARY
    fn generate_jwt(&self, ...) -> Result<String> {
        encode(..., &EncodingKey::from_secret(self.jwt_secret_primary.as_bytes()))
    }

    // Verify with BOTH keys (try primary first, fallback to secondary)
    fn decode_jwt(&self, token: &str) -> Result<Claims> {
        // Try primary key first
        match decode::<Claims>(token, &DecodingKey::from_secret(self.jwt_secret_primary.as_bytes()), ...) {
            Ok(claims) => Ok(claims),
            Err(_) => {
                // Fallback to secondary (old) key
                decode::<Claims>(token, &DecodingKey::from_secret(self.jwt_secret_secondary.as_bytes()), ...)
                    .map_err(|_| EnterpriseError::AuthenticationFailed)
            }
        }
    }
}
```

#### Step 3: Configuration Update
```toml
# config.toml
[auth]
jwt_secret_primary = "NEW_SECRET_HERE"
jwt_secret_secondary = "OLD_SECRET_HERE"  # Keep for grace period
```

#### Step 4: Deploy and Monitor
```bash
# Deploy new configuration
systemctl restart boundless-enterprise

# Monitor logs for any authentication failures
journalctl -u boundless-enterprise -f | grep "AuthenticationFailed"

# Check metrics
curl http://localhost:9615/metrics | grep auth_failures
```

#### Step 5: Grace Period (24-48 hours)
- All new logins receive tokens signed with NEW secret
- Old tokens (signed with OLD secret) still valid
- Monitor for token verification errors

#### Step 6: Remove Old Secret
After grace period (when old tokens expired):
```toml
# config.toml
[auth]
jwt_secret_primary = "NEW_SECRET_HERE"
# jwt_secret_secondary removed
```

```rust
// Revert to single-key mode
pub struct AuthService {
    db: PgPool,
    jwt_secret: String,
}
```

---

## 2. Keystore Master Key Rotation

### When to Rotate
- **Scheduled**: Every 180 days (recommended)
- **Emergency**: If key exposure suspected
- **Before**: Personnel changes (employees leaving)

### Prerequisites
- All wallets backed up
- Database backup completed
- New master key generated using platform KDF

### Rotation Process

#### Step 1: Generate New Master Key
```rust
use chacha20poly1305::{aead::Aead, KeyInit, ChaCha20Poly1305};
use argon2::{Argon2, PasswordHasher};

// Generate new master key using Argon2
let salt = SaltString::generate(&mut OsRng);
let argon2 = Argon2::default();
let password = std::env::var("NEW_MASTER_PASSWORD")?;

let hash = argon2.hash_password(password.as_bytes(), &salt)?;
let master_key = hash.hash.unwrap();
```

#### Step 2: Re-encrypt All Wallet Keys
```rust
// Migration script
async fn rotate_keystore_master_key(
    db: &PgPool,
    old_keystore: &Keystore,
    new_keystore: &Keystore,
) -> Result<()> {
    // 1. Get all encrypted wallet keys
    let keys = sqlx::query!(
        "SELECT key_id, wallet_id, encrypted_private_key, encryption_nonce
         FROM wallet_keys
         WHERE is_active = true"
    )
    .fetch_all(db)
    .await?;

    tracing::info!("Re-encrypting {} wallet keys", keys.len());

    // 2. Re-encrypt each key
    for key_row in keys {
        // Decrypt with old master key
        let encrypted_old = EncryptedKey {
            ciphertext: key_row.encrypted_private_key,
            nonce: key_row.encryption_nonce,
        };
        let plaintext = old_keystore.decrypt_key(&encrypted_old)?;

        // Encrypt with new master key
        let encrypted_new = new_keystore.encrypt_key(&plaintext)?;

        // Update database
        sqlx::query!(
            "UPDATE wallet_keys
             SET encrypted_private_key = $1, encryption_nonce = $2
             WHERE key_id = $3",
            encrypted_new.ciphertext,
            encrypted_new.nonce,
            key_row.key_id
        )
        .execute(db)
        .await?;

        // Zeroize plaintext immediately
        drop(plaintext);
    }

    tracing::info!("Keystore master key rotation completed");
    Ok(())
}
```

#### Step 3: Verification
```rust
// Verify all keys can be decrypted with new master key
async fn verify_key_rotation(db: &PgPool, new_keystore: &Keystore) -> Result<()> {
    let keys = sqlx::query!(
        "SELECT key_id, encrypted_private_key, encryption_nonce FROM wallet_keys"
    )
    .fetch_all(db)
    .await?;

    for key_row in keys {
        let encrypted = EncryptedKey {
            ciphertext: key_row.encrypted_private_key,
            nonce: key_row.encryption_nonce,
        };

        // Attempt decryption
        new_keystore.decrypt_key(&encrypted)?;
    }

    tracing::info!("All {} keys verified successfully", keys.len());
    Ok(())
}
```

---

## 3. Emergency Key Rotation

### Scenario: Key Compromise Detected

#### Immediate Actions (< 1 hour)
1. **Revoke all active sessions**
   ```sql
   UPDATE multipass_sessions SET revoked = true WHERE revoked = false;
   ```

2. **Rotate JWT secret immediately** (no grace period)
   ```bash
   NEW_SECRET=$(openssl rand -hex 32)
   export JWT_SECRET="$NEW_SECRET"
   systemctl restart boundless-enterprise
   ```

3. **Force user re-authentication**
   ```
   All users must log in again with new tokens
   ```

#### Follow-up Actions (< 24 hours)
1. Rotate keystore master key (if wallet keys compromised)
2. Audit all transactions in past 30 days
3. Review access logs for suspicious activity
4. Update incident response documentation

---

## 4. Automation and Monitoring

### Automated Key Age Monitoring
```rust
// Add to operational metrics
pub struct KeyRotationMetrics {
    pub jwt_secret_age_days: u64,
    pub keystore_key_age_days: u64,
    pub last_rotation_timestamp: chrono::DateTime<Utc>,
}

impl KeyRotationMetrics {
    pub fn check_rotation_needed(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.jwt_secret_age_days > 90 {
            warnings.push(format!(
                "JWT secret is {} days old (recommended: 90 days)",
                self.jwt_secret_age_days
            ));
        }

        if self.keystore_key_age_days > 180 {
            warnings.push(format!(
                "Keystore master key is {} days old (recommended: 180 days)",
                self.keystore_key_age_days
            ));
        }

        warnings
    }
}
```

### Prometheus Alerts
```yaml
# prometheus-alerts.yml
groups:
  - name: key_rotation
    rules:
      - alert: JWTSecretRotationDue
        expr: jwt_secret_age_days > 90
        for: 1h
        annotations:
          summary: "JWT secret rotation overdue"
          description: "JWT secret is {{ $value }} days old (max: 90)"

      - alert: KeystoreRotationDue
        expr: keystore_master_key_age_days > 180
        for: 1h
        annotations:
          summary: "Keystore master key rotation overdue"
          description: "Master key is {{ $value }} days old (max: 180)"
```

---

## 5. Best Practices

### Key Generation
- ✅ Use cryptographically secure random generators (OsRng, /dev/urandom)
- ✅ Minimum 256 bits (32 bytes) for symmetric keys
- ✅ Validate entropy before deployment
- ❌ Never use hardcoded, predictable, or weak secrets

### Key Storage
- ✅ Store in environment variables or secure vaults (HashiCorp Vault, AWS Secrets Manager)
- ✅ Encrypt secrets at rest
- ✅ Restrict access via IAM/RBAC
- ❌ Never commit keys to version control
- ❌ Never log keys in plaintext

### Key Distribution
- ✅ Use secure channels (TLS, SSH, encrypted files)
- ✅ Verify integrity (checksums, signatures)
- ✅ Audit all key access events
- ❌ Never send keys via email or chat

### Rotation Schedule
- **JWT Secrets**: 90 days
- **Keystore Master Key**: 180 days
- **API Keys**: 365 days or on personnel change
- **Emergency**: Immediately on compromise

---

## 6. Compliance and Audit

### Documentation Requirements
- Record of all key rotations (timestamp, operator, reason)
- Verification test results
- Incident reports (if emergency rotation)

### Audit Trail
```sql
-- Create audit log table
CREATE TABLE key_rotation_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_type VARCHAR(50) NOT NULL,  -- 'jwt_secret', 'keystore_master', etc.
    rotation_date TIMESTAMP NOT NULL DEFAULT NOW(),
    operator VARCHAR(100),
    reason VARCHAR(200),            -- 'scheduled', 'compromise', 'compliance'
    verification_status BOOLEAN,
    notes TEXT
);

-- Insert audit entry
INSERT INTO key_rotation_audit (key_type, operator, reason, verification_status)
VALUES ('jwt_secret', 'admin@boundless.io', 'scheduled', true);
```

---

## 7. Testing and Validation

### Pre-Rotation Checklist
- [ ] New keys generated and validated
- [ ] Backup of current configuration
- [ ] Database backup completed
- [ ] Rollback plan documented
- [ ] Monitoring dashboards ready
- [ ] Communication sent to stakeholders

### Post-Rotation Validation
- [ ] All services started successfully
- [ ] User authentication working
- [ ] Wallet operations functional
- [ ] No error spikes in logs
- [ ] Metrics show normal operation
- [ ] Rollback plan tested (in staging)

---

## 8. Contact and Escalation

For key rotation support:
- **Normal Hours**: security-ops@boundless.io
- **After Hours**: on-call-security@boundless.io
- **Emergency**: +1-XXX-XXX-XXXX

---

**Last Updated**: 2025-01-16
**Version**: 1.0
**Owner**: Security Operations Team
