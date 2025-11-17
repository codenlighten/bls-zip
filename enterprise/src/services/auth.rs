// Auth/SSO Service - Single sign-on and session management

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Deserialize, Serialize};

use crate::error::{EnterpriseError, Result};
use crate::models::*;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,      // Subject (identity_id)
    exp: usize,       // Expiration time
    iat: usize,       // Issued at
    scopes: Vec<String>, // Permissions
}

pub struct AuthService {
    db: PgPool,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(db: PgPool) -> Self {
        // SECURITY: JWT secret MUST be set via environment variable
        // Generate a secure secret with: openssl rand -hex 32
        let jwt_secret = std::env::var("JWT_SECRET")
            .expect("CRITICAL SECURITY: JWT_SECRET environment variable must be set. Generate one with: openssl rand -hex 32");

        // Validate secret strength (minimum 32 characters)
        if jwt_secret.len() < 32 {
            panic!("CRITICAL SECURITY: JWT_SECRET must be at least 32 characters long. Generate one with: openssl rand -hex 32");
        }

        // FIX H-5: Validate entropy - reject weak secrets
        Self::validate_jwt_secret_entropy(&jwt_secret);

        Self { db, jwt_secret }
    }

    /// FIX H-5: Validate JWT secret has sufficient entropy
    /// Rejects secrets like "aaaaaaa...", "11111111...", etc.
    fn validate_jwt_secret_entropy(secret: &str) {
        // Check for all same character
        let first_char = secret.chars().next().unwrap();
        if secret.chars().all(|c| c == first_char) {
            panic!("CRITICAL SECURITY: JWT_SECRET has no entropy (all same character). Generate a secure secret with: openssl rand -hex 32");
        }

        // Check for excessive repetition (>75% same character)
        let mut char_counts = std::collections::HashMap::new();
        for c in secret.chars() {
            *char_counts.entry(c).or_insert(0) += 1;
        }

        let max_count = char_counts.values().max().unwrap_or(&0);
        let repetition_ratio = (*max_count as f32) / (secret.len() as f32);

        if repetition_ratio > 0.75 {
            panic!("CRITICAL SECURITY: JWT_SECRET has insufficient entropy (too many repeated characters). Generate a secure secret with: openssl rand -hex 32");
        }

        // Reject known weak patterns
        let weak_patterns = [
            "password", "secret", "admin", "test", "dev",
            "12345", "abcde", "qwerty", "changeme"
        ];

        let secret_lower = secret.to_lowercase();
        for pattern in &weak_patterns {
            if secret_lower.contains(pattern) {
                panic!("CRITICAL SECURITY: JWT_SECRET contains weak pattern '{}'. Generate a secure secret with: openssl rand -hex 32", pattern);
            }
        }
    }

    /// Register new credentials for an identity
    pub async fn register(
        &self,
        identity_id: Uuid,
        password: String,
    ) -> Result<MultipassCredential> {
        // SECURITY: Validate password strength
        crate::validation::validate_password(&password)?;

        // 1. Verify identity exists
        let identity = sqlx::query!(
            "SELECT identity_id FROM identity_profiles WHERE identity_id = $1",
            identity_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or_else(|| EnterpriseError::IdentityNotFound(identity_id.to_string()))?;

        // 2. Hash password with Argon2
        let password_hash = self.hash_password(&password)?;

        // 3. Insert into multipass_credentials table
        let credential_id = Uuid::new_v4();

        // Generate a username from email (simple approach)
        let username = sqlx::query!("SELECT email FROM identity_profiles WHERE identity_id = $1", identity_id)
            .fetch_one(&self.db)
            .await
            .map_err(|e| EnterpriseError::from_db_error(e))?
            .email;

        sqlx::query!(
            r#"
            INSERT INTO multipass_credentials
            (credential_id, identity_id, username, password_hash, totp_secret, backup_codes, require_2fa, locked, failed_attempts)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            credential_id,
            identity_id,
            username,
            password_hash,
            None::<String>,
            &[] as &[String],
            false,
            false,
            0
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        // 4. Return credential
        Ok(MultipassCredential {
            credential_id,
            identity_id,
            password_hash,
            username,
            totp_secret: None,
            backup_codes: vec![],
            require_2fa: false,
            locked: false,
            failed_attempts: 0,
            last_login_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Login with email/password
    pub async fn login(
        &self,
        email: String,
        password: String,
    ) -> Result<(MultipassSession, String)> {
        // FIX L-4: Add debug logging for troubleshooting
        tracing::debug!("Login attempt for email: {}", email);

        // SECURITY: Validate inputs
        crate::validation::validate_email(&email)?;

        // FIX M-2: Use case-insensitive email lookup
        // 1. Query identity by email
        let identity = sqlx::query!(
            "SELECT identity_id FROM identity_profiles WHERE LOWER(email) = LOWER($1)",
            email
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or(EnterpriseError::AuthenticationFailed)?;

        let identity_id = identity.identity_id;
        tracing::debug!("Found identity_id: {} for email: {}", identity_id, email);

        // 2. Get credentials
        let credential = sqlx::query!(
            "SELECT password_hash FROM multipass_credentials WHERE identity_id = $1",
            identity_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or(EnterpriseError::AuthenticationFailed)?;

        // FIX M-1: Validate password hash format after retrieval
        // This detects database corruption or tampering before verification attempt
        self.validate_password_hash(&credential.password_hash)?;

        // 3. Verify password hash
        self.verify_password(&password, &credential.password_hash)?;
        tracing::debug!("Password verified successfully for identity: {}", identity_id);

        // 4. Create session
        let session_id = Uuid::new_v4();
        let scopes = vec!["read".to_string(), "write".to_string()]; // Default scopes
        let expires_at = Utc::now() + Duration::hours(24); // 24 hour session
        tracing::debug!("Creating session {} for identity: {}", session_id, identity_id);

        // 5. Generate JWT token signed by Boundless key
        let token = self.generate_jwt(identity_id, &scopes, expires_at)?;

        // 6. Hash the token for storage (DB stores token_hash not token)
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(token.as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        sqlx::query!(
            r#"
            INSERT INTO multipass_sessions
            (session_id, identity_id, token_hash, scopes, device_info, expires_at, revoked)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            session_id,
            identity_id,
            token_hash,
            &scopes,
            serde_json::json!({}),
            expires_at,
            false
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        // 7. Return session + token (note: we return the actual token, not the hash)
        let session = MultipassSession {
            session_id,
            identity_id,
            token_hash,
            scopes: scopes.clone(),
            device_info: serde_json::json!({}),
            expires_at,
            revoked: false,
            created_at: Utc::now(),
        };

        Ok((session, token))
    }

    /// Verify JWT token
    pub async fn verify_token(&self, token: &str) -> Result<Uuid> {
        // 1. Decode and verify JWT signature
        let claims = self.decode_jwt(token)?;

        // 2. Check expiration (done automatically by jsonwebtoken)

        // 3. Parse identity_id from subject
        let identity_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| EnterpriseError::AuthenticationFailed)?;

        // 4. Hash the token to look it up (DB stores token_hash not token)
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(token.as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        // 5. Verify session is not revoked
        let session = sqlx::query!(
            "SELECT revoked FROM multipass_sessions WHERE token_hash = $1",
            token_hash
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if let Some(session) = session {
            if session.revoked {
                return Err(EnterpriseError::AuthenticationFailed);
            }
        }

        Ok(identity_id)
    }

    /// Refresh session and generate new token
    pub async fn refresh_session(&self, session_id: Uuid) -> Result<String> {
        // 1. Verify session exists and not expired
        let session = sqlx::query!(
            r#"
            SELECT identity_id, scopes, expires_at, revoked
            FROM multipass_sessions
            WHERE session_id = $1
            "#,
            session_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or(EnterpriseError::NotFound("Session not found".to_string()))?;

        if session.revoked {
            return Err(EnterpriseError::AuthenticationFailed);
        }

        if session.expires_at < Utc::now() {
            return Err(EnterpriseError::SessionExpired);
        }

        // 2. Generate new JWT token with extended expiry
        let new_expires_at = Utc::now() + Duration::hours(24);
        let scopes = session.scopes.unwrap_or_default();
        let token = self.generate_jwt(session.identity_id, &scopes, new_expires_at)?;

        // 3. Hash the new token
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(token.as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        // 4. Update session with new token hash and expiry
        sqlx::query!(
            "UPDATE multipass_sessions SET token_hash = $1, expires_at = $2 WHERE session_id = $3",
            token_hash,
            new_expires_at,
            session_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        Ok(token)
    }

    /// Logout (revoke session)
    pub async fn logout(&self, session_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE multipass_sessions SET revoked = true WHERE session_id = $1",
            session_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        Ok(())
    }

    /// Check if session has required scope
    pub async fn has_scope(&self, session_id: Uuid, scope: &str) -> Result<bool> {
        let session = sqlx::query!(
            "SELECT scopes FROM multipass_sessions WHERE session_id = $1",
            session_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or(EnterpriseError::NotFound("Session not found".to_string()))?;

        Ok(session.scopes.unwrap_or_default().contains(&scope.to_string()))
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: Uuid) -> Result<MultipassSession> {
        let row = sqlx::query!(
            r#"
            SELECT session_id, identity_id, token_hash, scopes, device_info, expires_at, revoked, created_at
            FROM multipass_sessions
            WHERE session_id = $1
            "#,
            session_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or(EnterpriseError::NotFound("Session not found".to_string()))?;

        // FIX M-7: Check session expiration
        if row.expires_at < Utc::now() {
            return Err(EnterpriseError::AuthenticationFailed);
        }

        // Check if session is revoked
        if row.revoked {
            return Err(EnterpriseError::AuthenticationFailed);
        }

        Ok(MultipassSession {
            session_id: row.session_id,
            identity_id: row.identity_id,
            token_hash: row.token_hash,
            scopes: row.scopes.unwrap_or_default(),
            device_info: row.device_info.unwrap_or(serde_json::json!({})),
            expires_at: row.expires_at,
            revoked: row.revoked,
            created_at: row.created_at,
        })
    }

    /// Get all sessions for an identity
    pub async fn get_identity_sessions(&self, identity_id: Uuid) -> Result<Vec<MultipassSession>> {
        let rows = sqlx::query!(
            r#"
            SELECT session_id, identity_id, token_hash, scopes, device_info, expires_at, revoked, created_at
            FROM multipass_sessions
            WHERE identity_id = $1
            ORDER BY created_at DESC
            "#,
            identity_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let sessions = rows
            .into_iter()
            .map(|row| MultipassSession {
                session_id: row.session_id,
                identity_id: row.identity_id,
                token_hash: row.token_hash,
                scopes: row.scopes.unwrap_or_default(),
                device_info: row.device_info.unwrap_or(serde_json::json!({})),
                expires_at: row.expires_at,
                revoked: row.revoked,
                created_at: row.created_at,
            })
            .collect();

        Ok(sessions)
    }

    // Private helper methods

    /// Hash password with Argon2
    fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| EnterpriseError::CryptoError(e.to_string()))?
            .to_string();

        Ok(password_hash)
    }

    /// Verify password against hash
    fn verify_password(&self, password: &str, hash: &str) -> Result<()> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| EnterpriseError::CryptoError(e.to_string()))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| EnterpriseError::AuthenticationFailed)?;

        Ok(())
    }

    /// FIX M-1: Validate password hash format
    /// Validates that a retrieved password hash is in valid Argon2 format
    /// This detects database corruption or tampering early
    fn validate_password_hash(&self, hash: &str) -> Result<()> {
        // Attempt to parse the hash - will fail if format is invalid
        PasswordHash::new(hash)
            .map_err(|e| EnterpriseError::CryptoError(format!(
                "Invalid password hash format in database: {}",
                e
            )))?;

        // Additional validation: Check hash starts with $argon2
        if !hash.starts_with("$argon2") {
            return Err(EnterpriseError::CryptoError(
                "Password hash is not Argon2 format".to_string()
            ));
        }

        // Validate minimum length (Argon2 hashes are typically > 80 chars)
        if hash.len() < 50 {
            return Err(EnterpriseError::CryptoError(
                "Password hash too short - possible corruption".to_string()
            ));
        }

        Ok(())
    }

    /// Generate JWT token
    fn generate_jwt(
        &self,
        identity_id: Uuid,
        scopes: &[String],
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<String> {
        let claims = Claims {
            sub: identity_id.to_string(),
            exp: expires_at.timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            scopes: scopes.to_vec(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| EnterpriseError::Internal(e.to_string()))?;

        Ok(token)
    }

    /// Decode and verify JWT token
    fn decode_jwt(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| EnterpriseError::AuthenticationFailed)?;

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // FIX L-1: Removed broken tests that would panic
    // TODO: Reimplement with proper mocking framework (e.g., mockall or sqlx::test)

    #[test]
    fn test_password_hashing_standalone() {
        // Test password hashing without database dependency
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password = "my_secure_password_123";
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        // Verify password
        let parsed_hash = PasswordHash::new(&password_hash).unwrap();
        assert!(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok());

        // Wrong password should fail
        assert!(Argon2::default()
            .verify_password(b"wrong_password", &parsed_hash)
            .is_err());
    }

    #[test]
    fn test_jwt_generation_standalone() {
        // Test JWT generation without database dependency
        use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};

        let jwt_secret = "test_secret_at_least_32_chars_long_12345";
        let identity_id = Uuid::new_v4();
        let scopes = vec!["read".to_string(), "write".to_string()];
        let expires_at = Utc::now() + Duration::hours(1);

        let claims = Claims {
            sub: identity_id.to_string(),
            exp: expires_at.timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
            scopes: scopes.clone(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        ).unwrap();

        // Decode and verify
        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        ).unwrap();

        assert_eq!(token_data.claims.sub, identity_id.to_string());
        assert_eq!(token_data.claims.scopes, scopes);
    }
}
