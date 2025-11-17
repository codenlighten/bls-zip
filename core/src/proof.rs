// Proof Anchoring System for Enterprise Integration
//
// Enables anchoring identity attestations and credentials on-chain
// for verification and immutability guarantees.

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

/// Proof types supported by the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofType {
    /// KYC/AML verification proof
    KycVerification,
    /// Educational credential
    Credential,
    /// Employment verification
    Employment,
    /// Asset ownership
    AssetOwnership,
    /// Custom proof type
    Custom(String),
}

impl ProofType {
    pub fn as_str(&self) -> &str {
        match self {
            ProofType::KycVerification => "kyc_verification",
            ProofType::Credential => "credential",
            ProofType::Employment => "employment",
            ProofType::AssetOwnership => "asset_ownership",
            ProofType::Custom(s) => s.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "kyc_verification" => ProofType::KycVerification,
            "credential" => ProofType::Credential,
            "employment" => ProofType::Employment,
            "asset_ownership" => ProofType::AssetOwnership,
            _ => ProofType::Custom(s.to_string()),
        }
    }
}

/// On-chain proof anchor
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofAnchor {
    /// Unique proof identifier (hash of proof data)
    pub proof_id: [u8; 32],

    /// Identity that owns this proof
    pub identity_id: [u8; 32],

    /// Type of proof
    pub proof_type: ProofType,

    /// Hash of the actual proof data (stored off-chain)
    pub proof_hash: [u8; 32],

    /// Block height where proof was anchored
    pub block_height: u64,

    /// Timestamp of anchoring
    pub timestamp: u64,

    /// Optional metadata (max 256 bytes)
    pub metadata: Vec<u8>,
}

impl ProofAnchor {
    /// Create a new proof anchor
    pub fn new(
        identity_id: [u8; 32],
        proof_type: ProofType,
        proof_hash: [u8; 32],
        block_height: u64,
        timestamp: u64,
        metadata: Vec<u8>,
    ) -> Self {
        // Generate proof ID from identity + proof_hash + timestamp
        let mut hasher = Sha3_256::new();
        hasher.update(&identity_id);
        hasher.update(&proof_hash);
        hasher.update(&timestamp.to_le_bytes());
        let proof_id = hasher.finalize().into();

        Self {
            proof_id,
            identity_id,
            proof_type,
            proof_hash,
            block_height,
            timestamp,
            metadata,
        }
    }

    /// Verify proof anchor integrity
    pub fn verify(&self) -> bool {
        // Verify proof ID is correctly generated
        let mut hasher = Sha3_256::new();
        hasher.update(&self.identity_id);
        hasher.update(&self.proof_hash);
        hasher.update(&self.timestamp.to_le_bytes());
        let expected_id: [u8; 32] = hasher.finalize().into();

        expected_id == self.proof_id
    }

    /// Get proof ID as hex string
    pub fn proof_id_hex(&self) -> String {
        hex::encode(self.proof_id)
    }

    /// Get proof hash as hex string
    pub fn proof_hash_hex(&self) -> String {
        hex::encode(self.proof_hash)
    }
}

/// Proof storage for blockchain state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProofStorage {
    /// Map of proof_id -> ProofAnchor
    proofs: std::collections::HashMap<[u8; 32], ProofAnchor>,

    /// Map of identity_id -> list of proof_ids
    identity_proofs: std::collections::HashMap<[u8; 32], Vec<[u8; 32]>>,
}

impl ProofStorage {
    /// Create new proof storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Anchor a new proof
    pub fn anchor_proof(&mut self, proof: ProofAnchor) -> Result<(), String> {
        // Verify proof integrity
        if !proof.verify() {
            return Err("Invalid proof anchor".to_string());
        }

        // Check if proof already exists
        if self.proofs.contains_key(&proof.proof_id) {
            return Err("Proof already anchored".to_string());
        }

        // Store proof
        let proof_id = proof.proof_id;
        let identity_id = proof.identity_id;

        self.proofs.insert(proof_id, proof);

        // Add to identity index
        self.identity_proofs
            .entry(identity_id)
            .or_insert_with(Vec::new)
            .push(proof_id);

        Ok(())
    }

    /// Get proof by ID
    pub fn get_proof(&self, proof_id: &[u8; 32]) -> Option<&ProofAnchor> {
        self.proofs.get(proof_id)
    }

    /// Get all proofs for an identity
    pub fn get_identity_proofs(&self, identity_id: &[u8; 32]) -> Vec<&ProofAnchor> {
        self.identity_proofs
            .get(identity_id)
            .map(|proof_ids| {
                proof_ids
                    .iter()
                    .filter_map(|id| self.proofs.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Verify a proof exists and is valid
    pub fn verify_proof(&self, proof_hash: &[u8; 32]) -> Option<&ProofAnchor> {
        // Search for proof by hash
        self.proofs
            .values()
            .find(|p| p.proof_hash == *proof_hash && p.verify())
    }

    /// Get total number of anchored proofs
    pub fn total_proofs(&self) -> usize {
        self.proofs.len()
    }

    /// Get total number of identities with proofs
    pub fn total_identities(&self) -> usize {
        self.identity_proofs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_anchor_creation() {
        let identity_id = [1u8; 32];
        let proof_hash = [2u8; 32];
        let block_height = 100;
        let timestamp = 1234567890;
        let metadata = vec![1, 2, 3];

        let proof = ProofAnchor::new(
            identity_id,
            ProofType::KycVerification,
            proof_hash,
            block_height,
            timestamp,
            metadata.clone(),
        );

        assert_eq!(proof.identity_id, identity_id);
        assert_eq!(proof.proof_hash, proof_hash);
        assert_eq!(proof.block_height, block_height);
        assert_eq!(proof.timestamp, timestamp);
        assert_eq!(proof.metadata, metadata);
        assert!(proof.verify());
    }

    #[test]
    fn test_proof_storage() {
        let mut storage = ProofStorage::new();

        let identity_id = [1u8; 32];
        let proof_hash = [2u8; 32];

        let proof = ProofAnchor::new(
            identity_id,
            ProofType::Credential,
            proof_hash,
            100,
            1234567890,
            vec![],
        );

        let proof_id = proof.proof_id;

        // Anchor proof
        assert!(storage.anchor_proof(proof.clone()).is_ok());

        // Retrieve proof
        assert!(storage.get_proof(&proof_id).is_some());

        // Get identity proofs
        let identity_proofs = storage.get_identity_proofs(&identity_id);
        assert_eq!(identity_proofs.len(), 1);

        // Verify proof
        assert!(storage.verify_proof(&proof_hash).is_some());

        // Check totals
        assert_eq!(storage.total_proofs(), 1);
        assert_eq!(storage.total_identities(), 1);
    }

    #[test]
    fn test_duplicate_proof() {
        let mut storage = ProofStorage::new();

        let proof = ProofAnchor::new(
            [1u8; 32],
            ProofType::KycVerification,
            [2u8; 32],
            100,
            1234567890,
            vec![],
        );

        // First anchor succeeds
        assert!(storage.anchor_proof(proof.clone()).is_ok());

        // Second anchor fails
        assert!(storage.anchor_proof(proof).is_err());
    }

    #[test]
    fn test_proof_types() {
        assert_eq!(ProofType::KycVerification.as_str(), "kyc_verification");
        assert_eq!(ProofType::Credential.as_str(), "credential");

        assert_eq!(
            ProofType::from_str("kyc_verification"),
            ProofType::KycVerification
        );
        assert_eq!(ProofType::from_str("credential"), ProofType::Credential);

        match ProofType::from_str("custom_type") {
            ProofType::Custom(s) => assert_eq!(s, "custom_type"),
            _ => panic!("Expected custom proof type"),
        }
    }
}
