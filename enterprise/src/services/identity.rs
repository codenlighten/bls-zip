// Identity & Attestation Service
// Manages verified identities with KYC/AML and attestations

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{EnterpriseError, Result};
use crate::models::*;
// Note: boundless_crypto and boundless_core are not available in the enterprise package
// use boundless_crypto::PqcKeyPair;
// use boundless_core::Transaction;

/// Identity service for managing identity profiles and attestations
pub struct IdentityService {
    db: PgPool,
    blockchain_rpc_url: String,
    http_client: reqwest::Client,
}

impl IdentityService {
    pub fn new(db: PgPool, blockchain_rpc_url: String) -> Self {
        Self {
            db,
            blockchain_rpc_url,
            http_client: reqwest::Client::new(),
        }
    }

    /// Create a new identity profile
    pub async fn create_identity(
        &self,
        full_name: String,
        email: String,
        phone: Option<String>,
        country_code: Option<String>,
    ) -> Result<IdentityProfile> {
        // SECURITY: Validate all inputs
        crate::validation::validate_name(&full_name, "Full name")?;
        crate::validation::validate_email(&email)?;

        if let Some(ref phone_num) = phone {
            crate::validation::validate_phone(phone_num)?;
        }

        // Validate country code if provided (should be 2-letter ISO code)
        if let Some(ref country) = country_code {
            if country.len() != 2 || !country.chars().all(|c| c.is_ascii_alphabetic()) {
                return Err(EnterpriseError::InvalidInput(
                    "Country code must be a 2-letter ISO code".to_string()
                ));
            }
        }

        let identity_id = Uuid::new_v4();

        let profile = IdentityProfile {
            identity_id,
            full_name: full_name.clone(),
            email: email.clone(),
            phone: phone.clone(),
            country_code: country_code.clone(),
            date_of_birth: None,
            verification_status: "pending".to_string(),
            kyc_level: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Insert into database
        sqlx::query!(
            r#"
            INSERT INTO identity_profiles
            (identity_id, full_name, email, phone, country_code, verification_status, kyc_level)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            identity_id,
            full_name,
            email,
            phone,
            country_code,
            "pending",
            0
        )
        .execute(&self.db)
        .await?;

        Ok(profile)
    }

    /// Get identity profile by ID
    pub async fn get_identity(&self, identity_id: Uuid) -> Result<IdentityProfile> {
        let row = sqlx::query!(
            r#"
            SELECT identity_id, full_name, email, phone, country_code, date_of_birth,
                   verification_status, kyc_level, created_at, updated_at
            FROM identity_profiles
            WHERE identity_id = $1
            "#,
            identity_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| EnterpriseError::IdentityNotFound(identity_id.to_string()))?;

        Ok(IdentityProfile {
            identity_id: row.identity_id,
            full_name: row.full_name,
            email: row.email,
            phone: row.phone,
            country_code: row.country_code,
            date_of_birth: row.date_of_birth,
            verification_status: row.verification_status,
            kyc_level: row.kyc_level,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// Get identity by email
    pub async fn get_identity_by_email(&self, email: &str) -> Result<IdentityProfile> {
        // SECURITY: Validate email format
        crate::validation::validate_email(email)?;

        // FIX M-2: Use case-insensitive email lookup
        let row = sqlx::query!(
            r#"
            SELECT identity_id, full_name, email, phone, country_code, date_of_birth,
                   verification_status, kyc_level, created_at, updated_at
            FROM identity_profiles
            WHERE LOWER(email) = LOWER($1)
            "#,
            email
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| EnterpriseError::IdentityNotFound(email.to_string()))?;

        Ok(IdentityProfile {
            identity_id: row.identity_id,
            full_name: row.full_name,
            email: row.email,
            phone: row.phone,
            country_code: row.country_code,
            date_of_birth: row.date_of_birth,
            verification_status: row.verification_status,
            kyc_level: row.kyc_level,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// Update KYC status
    pub async fn update_kyc_status(
        &self,
        identity_id: Uuid,
        verification_status: String,
        kyc_level: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE identity_profiles
            SET verification_status = $1, kyc_level = $2, updated_at = NOW()
            WHERE identity_id = $3
            "#,
            verification_status,
            kyc_level,
            identity_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Create an attestation for an identity
    /// Returns the attestation and optionally anchors it to the Boundless chain
    pub async fn create_attestation(
        &self,
        identity_id: Uuid,
        attestation_type: AttestationType,
        evidence_refs: Vec<String>,
        issuer: String,
        valid_to: Option<chrono::DateTime<Utc>>,
        anchor_to_chain: bool,
    ) -> Result<IdentityAttestation> {
        // Verify identity exists
        self.get_identity(identity_id).await?;

        let attestation_id = Uuid::new_v4();
        let valid_from = Utc::now();

        let mut chain_anchor_tx = None;

        // Anchor to Boundless chain if requested
        if anchor_to_chain {
            chain_anchor_tx = Some(self.anchor_attestation_to_chain(
                attestation_id,
                identity_id,
                &attestation_type,
                &evidence_refs,
            ).await?);
        }

        let attestation = IdentityAttestation {
            attestation_id,
            identity_id,
            attestation_type: attestation_type.clone(),
            evidence_refs: evidence_refs.clone(),
            issuer: issuer.clone(),
            status: AttestationStatus::Valid,
            valid_from,
            valid_to,
            chain_anchor_tx: chain_anchor_tx.clone(),
        };

        // Create proof hash from evidence
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        for evidence in &evidence_refs {
            hasher.update(evidence.as_bytes());
        }
        let proof_hash = hex::encode(hasher.finalize());

        // Insert into database (table is named 'attestations' not 'identity_attestations')
        sqlx::query!(
            r#"
            INSERT INTO attestations
            (attestation_id, identity_id, attestation_type, issuer, proof_hash, issued_at, expires_at, revoked, chain_anchor_tx, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            attestation_id,
            identity_id,
            format!("{:?}", attestation_type),
            issuer,
            proof_hash,
            valid_from,
            valid_to,
            false,
            chain_anchor_tx,
            serde_json::json!({"evidence_refs": evidence_refs})
        )
        .execute(&self.db)
        .await?;

        Ok(attestation)
    }

    /// Get all attestations for an identity
    pub async fn get_attestations(&self, identity_id: Uuid) -> Result<Vec<IdentityAttestation>> {
        let rows = sqlx::query!(
            r#"
            SELECT attestation_id, identity_id, attestation_type, issuer, proof_hash,
                   issued_at, expires_at, revoked, chain_anchor_tx, metadata
            FROM attestations
            WHERE identity_id = $1
            ORDER BY issued_at DESC
            "#,
            identity_id
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(|row| {
            let attestation_type: AttestationType = serde_json::from_value(
                serde_json::Value::String(row.attestation_type)
            ).unwrap_or(AttestationType::KycVerified);

            let evidence_refs: Vec<String> = row.metadata
                .as_ref()
                .and_then(|m| m.get("evidence_refs"))
                .and_then(|e| serde_json::from_value(e.clone()).ok())
                .unwrap_or_default();

            let status = if row.revoked {
                AttestationStatus::Revoked
            } else if let Some(expires_at) = row.expires_at {
                if Utc::now() > expires_at {
                    AttestationStatus::Expired
                } else {
                    AttestationStatus::Valid
                }
            } else {
                AttestationStatus::Valid
            };

            IdentityAttestation {
                attestation_id: row.attestation_id,
                identity_id: row.identity_id,
                attestation_type,
                evidence_refs,
                issuer: row.issuer,
                status,
                valid_from: row.issued_at,
                valid_to: row.expires_at,
                chain_anchor_tx: row.chain_anchor_tx,
            }
        }).collect())
    }

    /// Revoke an attestation
    pub async fn revoke_attestation(&self, attestation_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE attestations
            SET revoked = true
            WHERE attestation_id = $1
            "#,
            attestation_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Anchor an attestation to the Boundless blockchain
    /// Creates a transaction with the attestation hash as metadata
    async fn anchor_attestation_to_chain(
        &self,
        attestation_id: Uuid,
        identity_id: Uuid,
        attestation_type: &AttestationType,
        evidence_refs: &[String],
    ) -> Result<String> {
        // Create attestation proof data
        let proof_data = format!(
            "ATTESTATION:{}:{}:{:?}:{}",
            attestation_id,
            identity_id,
            attestation_type,
            evidence_refs.join(",")
        );

        // Hash the proof data using SHA3-256
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(proof_data.as_bytes());
        let proof_hash = hasher.finalize();
        let proof_hash_hex = hex::encode(proof_hash);

        // Convert UUID identity to 32-byte array for blockchain
        let identity_bytes = identity_id.as_bytes();
        let mut identity_hash = [0u8; 32];
        let mut id_hasher = Sha3_256::new();
        id_hasher.update(identity_bytes);
        identity_hash.copy_from_slice(&id_hasher.finalize());

        // Create metadata with attestation details
        let metadata = serde_json::json!({
            "attestation_id": attestation_id.to_string(),
            "attestation_type": format!("{:?}", attestation_type),
            "evidence_refs": evidence_refs,
        });

        // Create anchor proof request
        let request = serde_json::json!({
            "identity_id": hex::encode(identity_hash),
            "proof_type": match attestation_type {
                AttestationType::KycVerified => "kyc_verification",
                AttestationType::Kyc => "kyc_verification",
                AttestationType::AccreditedInvestor => "credential",
                AttestationType::Employment => "employment",
                AttestationType::Education => "credential",
                AttestationType::ProfessionalCredential => "credential",
                AttestationType::AddressProof => "address_proof",
                AttestationType::IncomeProof => "income_proof",
                AttestationType::AssetOwnership => "asset_ownership",
                AttestationType::SocialGraph => "social_graph",
            },
            "proof_hash": proof_hash_hex,
            "metadata": metadata,
        });

        // Submit to blockchain RPC
        let url = format!("{}/api/v1/proof/anchor", self.blockchain_rpc_url);
        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                EnterpriseError::Internal(format!("Failed to submit proof to blockchain: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(EnterpriseError::Internal(format!(
                "Blockchain RPC error {}: {}",
                status, error_text
            )));
        }

        // Parse response
        let response_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| EnterpriseError::Internal(format!("Failed to parse RPC response: {}", e)))?;

        let tx_hash = response_data
            .get("tx_hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                EnterpriseError::Internal("Missing tx_hash in RPC response".to_string())
            })?
            .to_string();

        // FIX M-5: Verify transaction was confirmed on blockchain
        // Wait briefly for transaction to be mined (adjust timeout based on block time)
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Query blockchain to verify transaction exists and is confirmed
        let verify_url = format!("{}/api/v1/transaction/{}", self.blockchain_rpc_url, tx_hash);
        let verify_response = self
            .http_client
            .get(&verify_url)
            .send()
            .await
            .map_err(|e| {
                EnterpriseError::Internal(format!("Failed to verify transaction on blockchain: {}", e))
            })?;

        if !verify_response.status().is_success() {
            return Err(EnterpriseError::Internal(format!(
                "Transaction {} not found on blockchain - anchor may have failed",
                tx_hash
            )));
        }

        // Parse transaction data to verify it exists
        let tx_data: serde_json::Value = verify_response
            .json()
            .await
            .map_err(|e| EnterpriseError::Internal(format!("Failed to parse transaction data: {}", e)))?;

        // Verify transaction is confirmed (has block_height)
        if tx_data.get("block_height").is_none() {
            return Err(EnterpriseError::Internal(format!(
                "Transaction {} is pending - not yet confirmed on blockchain",
                tx_hash
            )));
        }

        tracing::info!(
            "Anchored and CONFIRMED attestation {} to Boundless chain: {} (block: {})",
            attestation_id,
            tx_hash,
            tx_data.get("block_height").and_then(|v| v.as_u64()).unwrap_or(0)
        );

        Ok(tx_hash)
    }

    /// Verify an attestation is valid and not expired
    pub async fn verify_attestation(&self, attestation_id: Uuid) -> Result<bool> {
        let row = sqlx::query!(
            r#"
            SELECT revoked, expires_at
            FROM attestations
            WHERE attestation_id = $1
            "#,
            attestation_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| EnterpriseError::InvalidInput("Attestation not found".to_string()))?;

        // Check if revoked
        if row.revoked {
            return Ok(false);
        }

        // Check if expired
        if let Some(expires_at) = row.expires_at {
            if Utc::now() > expires_at {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// List all identities (admin function)
    pub async fn list_identities(&self, limit: i64, offset: i64) -> Result<Vec<IdentityProfile>> {
        let rows = sqlx::query!(
            r#"
            SELECT identity_id, full_name, email, phone, country_code, date_of_birth,
                   verification_status, kyc_level, created_at, updated_at
            FROM identity_profiles
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().map(|row| IdentityProfile {
            identity_id: row.identity_id,
            full_name: row.full_name,
            email: row.email,
            phone: row.phone,
            country_code: row.country_code,
            date_of_birth: row.date_of_birth,
            verification_status: row.verification_status,
            kyc_level: row.kyc_level,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Add unit tests with mock database
}
