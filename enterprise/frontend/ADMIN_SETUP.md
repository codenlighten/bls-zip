# E² Multipass - Admin User Setup

## Default Admin Credentials

The Boundless E² Multipass platform includes a default admin user for initial setup and testing.

### Admin Login Credentials

```
Email:    yourfriends@smartledger.solutions
Password: BoundlessTrust
```

**⚠️ IMPORTANT SECURITY NOTES:**
- These credentials are for **development and initial setup only**
- **MUST** be changed immediately in production environments
- Never commit these credentials to version control
- Use environment variables for production credentials

---

## Admin User Profile

The default admin user has the following profile:

**Identity Information:**
- **Legal Name**: Smart Ledger Solutions Admin
- **Email**: yourfriends@smartledger.solutions
- **Organization**: Smart Ledger Solutions
- **KYC Status**: Verified (pre-seeded)
- **AML Risk Score**: 0 (Trusted)

**Permissions:**
- Full system administrator access
- All E² Multipass features enabled
- Can create and manage other users
- Can access all wallets and assets
- Can deploy and execute smart contracts
- Can manage applications and integrations
- Can access all collaboration capsules
- Can register AI agents
- Can manage knowledge vault

**CIVA Attestations:**
- **Layer 1 - Identity Proof**: KYC Verified, Biometric Verified
- **Layer 2 - Risk & Compliance**: AML Cleared, Sanctions Clear
- **Layer 3 - Attributes**: Administrator, Developer, Auditor

---

## Backend Setup

The backend needs to seed this admin user on first launch.

### Database Seed File

Create `enterprise-backend/seeds/admin_user.sql`:

```sql
-- Boundless E² Multipass - Admin User Seed Data
-- WARNING: For development only. Change credentials in production.

-- Generate admin identity
INSERT INTO identity_profiles (
    identity_id,
    root_pki_key_id,
    legal_name,
    email,
    org_name,
    kyc_status,
    aml_risk_score,
    created_at,
    updated_at
) VALUES (
    'admin_identity_001',
    'pki_root_admin_001',
    'Smart Ledger Solutions Admin',
    'yourfriends@smartledger.solutions',
    'Smart Ledger Solutions',
    'verified',
    0.0,
    NOW(),
    NOW()
);

-- Create admin credentials
INSERT INTO multipass_credentials (
    credential_id,
    identity_id,
    password_hash, -- Hash of 'BoundlessTrust' using Argon2id
    webauthn_credentials,
    pki_key_ids,
    status,
    created_at,
    updated_at
) VALUES (
    'cred_admin_001',
    'admin_identity_001',
    '$argon2id$v=19$m=65536,t=3,p=4$<HASH_HERE>', -- Backend should hash this
    '[]'::jsonb,
    '["pki_root_admin_001"]'::jsonb,
    'active',
    NOW(),
    NOW()
);

-- Create admin wallet
INSERT INTO wallet_accounts (
    wallet_id,
    identity_id,
    boundless_addresses,
    labels,
    created_at
) VALUES (
    'wallet_admin_001',
    'admin_identity_001',
    '[{"address": "bls1admin000000000000000000000000", "label": "Admin Main Wallet"}]'::jsonb,
    '["admin", "primary"]'::jsonb,
    NOW()
);

-- Seed admin CIVA attestations
INSERT INTO identity_attestations (
    attestation_id,
    identity_id,
    attestation_type,
    evidence_refs,
    issuer,
    status,
    valid_from,
    valid_to
) VALUES
-- Layer 1: Identity Proof
(
    'attest_admin_kyc',
    'admin_identity_001',
    'kyc',
    '["doc_admin_kyc_001"]'::jsonb,
    'Boundless System Authority',
    'valid',
    NOW(),
    NOW() + INTERVAL '10 years'
),
-- Layer 2: Risk & Compliance
(
    'attest_admin_aml',
    'admin_identity_001',
    'aml_cleared',
    '[]'::jsonb,
    'Boundless Compliance Authority',
    'valid',
    NOW(),
    NOW() + INTERVAL '10 years'
),
-- Layer 3: Attributes
(
    'attest_admin_role',
    'admin_identity_001',
    'administrator',
    '[]'::jsonb,
    'Boundless System Authority',
    'valid',
    NOW(),
    NOW() + INTERVAL '10 years'
);

COMMIT;
```

### Rust Backend Seed Code

Create `enterprise-backend/src/seeds/mod.rs`:

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

pub async fn seed_admin_user(db: &Database) -> Result<(), Error> {
    println!("Seeding default admin user...");

    // Hash the default password
    let password = "BoundlessTrust";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    // Check if admin already exists
    let existing = db.get_identity_by_email("yourfriends@smartledger.solutions").await?;
    if existing.is_some() {
        println!("Admin user already exists, skipping seed.");
        return Ok(());
    }

    // Create admin identity
    let identity = IdentityProfile {
        identity_id: "admin_identity_001".to_string(),
        root_pki_key_id: "pki_root_admin_001".to_string(),
        legal_name: "Smart Ledger Solutions Admin".to_string(),
        email: "yourfriends@smartledger.solutions".to_string(),
        org_name: Some("Smart Ledger Solutions".to_string()),
        kyc_status: KycStatus::Verified,
        aml_risk_score: 0.0,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    db.create_identity(identity).await?;

    // Create credentials
    let credentials = MultipassCredential {
        credential_id: "cred_admin_001".to_string(),
        identity_id: "admin_identity_001".to_string(),
        password_hash,
        webauthn_credentials: vec![],
        nfc_card_id: None,
        pki_key_ids: vec!["pki_root_admin_001".to_string()],
        status: CredentialStatus::Active,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    db.create_credential(credentials).await?;

    println!("✓ Admin user seeded successfully");
    println!("  Email: yourfriends@smartledger.solutions");
    println!("  Password: BoundlessTrust");
    println!("  ⚠️  CHANGE PASSWORD IMMEDIATELY IN PRODUCTION");

    Ok(())
}
```

---

## Frontend Environment

Update `.env.local` with admin credentials for auto-fill during development:

```bash
# Default Admin Credentials (Development Only)
NEXT_PUBLIC_DEV_ADMIN_EMAIL=yourfriends@smartledger.solutions
NEXT_PUBLIC_DEV_ADMIN_PASSWORD=BoundlessTrust
NEXT_PUBLIC_AUTO_FILL_ADMIN=true
```

---

## Login Process

### Development Login (Auto-fill)

When `NEXT_PUBLIC_AUTO_FILL_ADMIN=true`, the login form will:
1. Pre-fill email: yourfriends@smartledger.solutions
2. Pre-fill password: BoundlessTrust
3. Show "Development Mode" indicator
4. Allow one-click login

### Production Login

In production (`NODE_ENV=production`):
1. Auto-fill is disabled
2. Admin must manually enter credentials
3. Password should be changed on first login
4. MFA/2FA should be enabled

---

## First Login Checklist

After logging in with admin credentials for the first time:

1. **Change Password**
   - Navigate to Identity → Security Settings
   - Change password from `BoundlessTrust` to secure password
   - Use password manager for storage

2. **Enable WebAuthn/Biometrics**
   - Navigate to Identity → Security Settings
   - Register fingerprint or face recognition
   - Test authentication

3. **Register NFC Hardware Pass** (Optional)
   - Navigate to Identity → NFC/Hardware Pass
   - Register physical multipass card
   - Test offline verification

4. **Create Additional Admin Users**
   - Navigate to Admin → User Management
   - Create additional admin accounts
   - Assign appropriate roles

5. **Configure System Settings**
   - Navigate to Admin → System Settings
   - Configure blockchain node connection
   - Set up email notifications
   - Configure backup schedules

6. **Review Audit Logs**
   - Navigate to Admin → Audit Logs
   - Verify seed process logged correctly
   - Set up log retention policies

---

## Security Best Practices

### Password Policy

The default password `BoundlessTrust` should be changed to meet these requirements:
- Minimum 12 characters
- Include uppercase, lowercase, numbers, symbols
- Not contain common words or patterns
- Unique (not reused from other systems)
- Stored in password manager

### Multi-Factor Authentication

Enable at least one additional authentication method:
- **WebAuthn/Biometrics**: Fingerprint, Face ID, Touch ID
- **NFC Hardware Pass**: Physical multipass card
- **TOTP**: Time-based one-time password (Google Authenticator)

### Session Management

Configure secure session settings:
- Session timeout: 1 hour (configurable)
- Idle timeout: 15 minutes (configurable)
- Concurrent sessions: Limited to 3 devices
- Geographic restrictions: Optional

### Blockchain Key Management

The admin user's blockchain keys should be:
- Generated using post-quantum algorithms (ML-DSA-44)
- Stored in hardware security module (HSM)
- Backed up with recovery phrase (BIP-39)
- Never exported or shared

---

## Admin Capabilities

### Identity Management
- ✓ Create new identities
- ✓ Issue CIVA attestations
- ✓ Revoke attestations
- ✓ Update KYC status
- ✓ Manage AML risk scores

### Wallet Management
- ✓ Create wallets for users
- ✓ View all wallet balances
- ✓ Monitor transactions
- ✓ Issue BLS tokens
- ✓ Mint custom assets

### Smart Contract Management
- ✓ Deploy contract templates
- ✓ Execute contracts as any party
- ✓ Approve contract deployments
- ✓ Audit contract executions

### Application Management
- ✓ Register new applications
- ✓ Approve app connections
- ✓ Revoke app access
- ✓ Monitor app activity

### Market Management
- ✓ List regulated assets
- ✓ Approve market listings
- ✓ Monitor trading activity
- ✓ Enforce compliance rules

### System Administration
- ✓ Manage users and roles
- ✓ Configure system settings
- ✓ View audit logs
- ✓ Generate compliance reports
- ✓ Manage blockchain nodes

---

## Troubleshooting

### Cannot Login

**Problem**: "Invalid credentials" error

**Solutions**:
1. Verify backend is running and seeded
2. Check database has admin user
3. Confirm password hash is correct
4. Review backend logs for errors
5. Try password reset flow

### Session Expires Immediately

**Problem**: Logged out right after login

**Solutions**:
1. Check JWT token expiration settings
2. Verify system time is synchronized
3. Review session configuration
4. Check for CORS issues
5. Inspect browser console errors

### Missing Permissions

**Problem**: Cannot access admin features

**Solutions**:
1. Verify identity has admin attestation
2. Check role assignments in database
3. Review permission policies
4. Ensure KYC status is verified
5. Check blockchain node connection

---

## Production Deployment

### Change Default Credentials

**CRITICAL**: Before deploying to production:

1. **Change Email**:
   ```sql
   UPDATE identity_profiles
   SET email = 'admin@your-company.com'
   WHERE identity_id = 'admin_identity_001';
   ```

2. **Change Password**:
   - Login with default credentials
   - Navigate to Security Settings
   - Change password to secure value
   - Logout and login with new password

3. **Remove Auto-fill**:
   ```bash
   # In .env.production
   NEXT_PUBLIC_AUTO_FILL_ADMIN=false
   ```

4. **Enable MFA**:
   - Register WebAuthn device
   - Enable TOTP
   - Test authentication flow

5. **Rotate Keys**:
   - Generate new PKI keys
   - Update blockchain identity
   - Archive old keys securely

### Security Audit

Before going live:
- [ ] Default password changed
- [ ] Auto-fill disabled
- [ ] MFA enabled
- [ ] Session settings configured
- [ ] Audit logging enabled
- [ ] Backup procedures tested
- [ ] Disaster recovery plan documented
- [ ] Security scan completed
- [ ] Penetration test passed
- [ ] Compliance review approved

---

## Support

For issues with admin setup:
1. Check backend logs: `tail -f enterprise-backend/logs/app.log`
2. Review database: `psql -U boundless -d enterprise_db`
3. Verify blockchain sync: `curl http://localhost:9933/health`
4. Contact Smart Ledger Solutions support

---

**Default Admin User**
Email: `yourfriends@smartledger.solutions`
Password: `BoundlessTrust`
⚠️ **Change immediately in production**
