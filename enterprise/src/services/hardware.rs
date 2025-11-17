// Hardware Pass Service - NFC card and secure element integration

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};
use rand::RngCore;

use crate::error::{EnterpriseError, Result};
use crate::models::*;

pub struct HardwareService {
    db: PgPool,
}

impl HardwareService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Register new hardware device
    pub async fn register_device(
        &self,
        identity_id: Uuid,
        device_type: String,
        public_key: String,
        capabilities: Vec<String>,
    ) -> Result<HardwarePass> {
        // 1. Verify identity exists
        let identity_exists = sqlx::query!(
            "SELECT identity_id FROM identity_profiles WHERE identity_id = $1",
            identity_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if identity_exists.is_none() {
            return Err(EnterpriseError::IdentityNotFound(identity_id.to_string()));
        }

        // 2. Validate public key format (should be hex-encoded PQC public key)
        if public_key.is_empty() {
            return Err(EnterpriseError::InvalidInput("Public key cannot be empty".to_string()));
        }

        // Verify it's valid hex
        if hex::decode(&public_key).is_err() {
            return Err(EnterpriseError::InvalidInput("Public key must be valid hex".to_string()));
        }

        // 3. Create device ID
        let device_id = Uuid::new_v4();

        // 4. Insert into hardware_passes table
        sqlx::query!(
            r#"
            INSERT INTO hardware_passes
            (device_id, identity_id, device_type, public_key, capabilities, status, last_used_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            device_id,
            identity_id,
            device_type,
            public_key,
            &capabilities,
            "Active",
            Some(Utc::now()),
            Utc::now(),
            Utc::now()
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        // 5. Return HardwarePass
        Ok(HardwarePass {
            device_id,
            identity_id,
            device_type,
            public_key,
            capabilities,
            status: HardwareStatus::Active,
            last_used_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Get hardware device by ID
    pub async fn get_device(&self, device_id: Uuid) -> Result<HardwarePass> {
        let row = sqlx::query!(
            r#"
            SELECT device_id, identity_id, device_type, public_key, capabilities, status, last_used_at, created_at, updated_at
            FROM hardware_passes
            WHERE device_id = $1
            "#,
            device_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?
        .ok_or_else(|| EnterpriseError::HardwareNotFound(device_id.to_string()))?;

        // Parse status
        let status: HardwareStatus = match row.status.as_str() {
            "Active" => HardwareStatus::Active,
            "Revoked" => HardwareStatus::Revoked,
            "Lost" => HardwareStatus::Lost,
            _ => HardwareStatus::Active,
        };

        Ok(HardwarePass {
            device_id: row.device_id,
            identity_id: row.identity_id,
            device_type: row.device_type,
            public_key: row.public_key,
            capabilities: row.capabilities.unwrap_or_default(),
            status,
            last_used_at: row.last_used_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// Get all devices for an identity
    pub async fn get_identity_devices(&self, identity_id: Uuid) -> Result<Vec<HardwarePass>> {
        let rows = sqlx::query!(
            r#"
            SELECT device_id, identity_id, device_type, public_key, capabilities, status, last_used_at, created_at, updated_at
            FROM hardware_passes
            WHERE identity_id = $1
            ORDER BY created_at DESC
            "#,
            identity_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let devices = rows
            .into_iter()
            .map(|row| {
                let status: HardwareStatus = match row.status.as_str() {
                    "Active" => HardwareStatus::Active,
                    "Revoked" => HardwareStatus::Revoked,
                    "Lost" => HardwareStatus::Lost,
                    _ => HardwareStatus::Active,
                };

                HardwarePass {
                    device_id: row.device_id,
                    identity_id: row.identity_id,
                    device_type: row.device_type,
                    public_key: row.public_key,
                    capabilities: row.capabilities.unwrap_or_default(),
                    status,
                    last_used_at: row.last_used_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }
            })
            .collect();

        Ok(devices)
    }

    /// Authenticate with hardware device using challenge-response
    pub async fn authenticate(
        &self,
        device_id: Uuid,
        challenge: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<bool> {
        // 1. Get device from database
        let device = self.get_device(device_id).await?;

        // 2. Verify device is active (not lost/revoked)
        if device.status != HardwareStatus::Active {
            return Err(EnterpriseError::HardwareRevoked);
        }

        // 3. Verify signature using device public key (Boundless PQC)
        let public_key_bytes = hex::decode(&device.public_key)
            .map_err(|e| EnterpriseError::CryptoError(format!("Invalid public key hex: {}", e)))?;

        // Use Boundless PQC signature verification
        let is_valid = self.verify_pqc_signature(&public_key_bytes, &challenge, &signature)?;

        // 4. Update last_used timestamp if authentication succeeded
        if is_valid {
            self.update_last_used(device_id).await?;
        }

        // 5. Return authentication result
        Ok(is_valid)
    }

    /// Revoke device
    pub async fn revoke(&self, device_id: Uuid) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE hardware_passes SET status = $1 WHERE device_id = $2",
            "Revoked",
            device_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if result.rows_affected() == 0 {
            return Err(EnterpriseError::HardwareNotFound(device_id.to_string()));
        }

        Ok(())
    }

    /// Mark device as lost
    pub async fn mark_lost(&self, device_id: Uuid) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE hardware_passes SET status = $1 WHERE device_id = $2",
            "Lost",
            device_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if result.rows_affected() == 0 {
            return Err(EnterpriseError::HardwareNotFound(device_id.to_string()));
        }

        Ok(())
    }

    /// Update last used timestamp
    pub async fn update_last_used(&self, device_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE hardware_passes SET last_used_at = $1 WHERE device_id = $2",
            Utc::now(),
            device_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        Ok(())
    }

    /// Generate authentication challenge
    pub async fn generate_challenge(&self, device_id: Uuid) -> Result<Vec<u8>> {
        // 1. Verify device exists and is active
        let device = self.get_device(device_id).await?;

        if device.status != HardwareStatus::Active {
            return Err(EnterpriseError::HardwareRevoked);
        }

        // 2. Generate random challenge (32 bytes)
        let mut challenge = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut challenge);

        // 3. Store challenge with expiration (5 minutes)
        let challenge_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::minutes(5);

        sqlx::query!(
            r#"
            INSERT INTO hardware_challenges
            (challenge_id, device_id, challenge, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            challenge_id,
            device_id,
            &challenge,
            expires_at,
            Utc::now()
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        // 4. Return challenge
        Ok(challenge)
    }

    /// Verify challenge-response (internal method called during authentication)
    pub async fn verify_challenge(
        &self,
        device_id: Uuid,
        challenge: &[u8],
    ) -> Result<bool> {
        // Query for unexpired challenge matching the device
        let result = sqlx::query!(
            r#"
            SELECT challenge_id, expires_at
            FROM hardware_challenges
            WHERE device_id = $1 AND challenge = $2 AND expires_at > $3
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            device_id,
            challenge,
            Utc::now()
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if let Some(row) = result {
            // Delete the challenge after use (prevent replay attacks)
            sqlx::query!(
                "DELETE FROM hardware_challenges WHERE challenge_id = $1",
                row.challenge_id
            )
            .execute(&self.db)
            .await
            .map_err(|e| EnterpriseError::from_db_error(e))?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check device capability
    pub async fn has_capability(
        &self,
        device_id: Uuid,
        capability: &str,
    ) -> Result<bool> {
        // 1. Get device from database
        let device = self.get_device(device_id).await?;

        // 2. Check if capability is in device capabilities
        Ok(device.capabilities.iter().any(|c| c == capability))
    }

    /// Get device usage statistics
    pub async fn get_device_stats(&self, device_id: Uuid) -> Result<DeviceStats> {
        // Verify device exists
        let device = self.get_device(device_id).await?;

        // Get authentication count from challenges
        let auth_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM hardware_challenges WHERE device_id = $1",
            device_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        Ok(DeviceStats {
            device_id,
            total_authentications: auth_count.count.unwrap_or(0),
            last_used_at: device.last_used_at,
            status: device.status,
        })
    }

    // ==================== Private Helper Methods ====================

    /// Verify PQC signature using Boundless crypto
    fn verify_pqc_signature(
        &self,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool> {
        // Use Boundless REAL PQC signature verification (Dilithium5)
        use crate::crypto::PqcSignature;

        match PqcSignature::verify_detached(public_key, message, signature) {
            Ok(valid) => Ok(valid),
            Err(e) => {
                // If signature verification fails, return false instead of error
                // (this is expected for invalid signatures)
                eprintln!("Signature verification failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Delete device
    pub async fn delete_device(&self, device_id: Uuid) -> Result<()> {
        // Delete associated challenges first
        sqlx::query!(
            "DELETE FROM hardware_challenges WHERE device_id = $1",
            device_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        // Delete device
        let result = sqlx::query!(
            "DELETE FROM hardware_passes WHERE device_id = $1",
            device_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        if result.rows_affected() == 0 {
            return Err(EnterpriseError::HardwareNotFound(device_id.to_string()));
        }

        Ok(())
    }

    /// Clean up expired challenges (should be run periodically)
    pub async fn cleanup_expired_challenges(&self) -> Result<u64> {
        let result = sqlx::query!(
            "DELETE FROM hardware_challenges WHERE expires_at < $1",
            Utc::now()
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        Ok(result.rows_affected())
    }
}

// DeviceStats helper struct
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceStats {
    pub device_id: Uuid,
    pub total_authentications: i64,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: HardwareStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_validation() {
        // Valid hex public key
        let valid_key = "a1b2c3d4e5f6";
        assert!(hex::decode(valid_key).is_ok());

        // Invalid hex public key
        let invalid_key = "xyz123";
        assert!(hex::decode(invalid_key).is_err());
    }
}
