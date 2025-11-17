// Extended Transaction Types for Enterprise Features
//
// Supports standard value transfers, proof anchoring, and asset transfers

use crate::{proof::ProofType, Transaction, TxInput};
use serde::{Deserialize, Serialize};
use sha3::Digest;

/// Transaction type identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Standard value transfer (native BLS token)
    Standard,

    /// Proof anchoring transaction (for identity attestations)
    ProofAnchor,

    /// Multi-asset transfer
    AssetTransfer,

    /// Asset registration/creation
    AssetRegister,
}

impl TransactionType {
    /// Get type as u8 for encoding
    pub fn as_u8(&self) -> u8 {
        match self {
            TransactionType::Standard => 0,
            TransactionType::ProofAnchor => 1,
            TransactionType::AssetTransfer => 2,
            TransactionType::AssetRegister => 3,
        }
    }

    /// Parse from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(TransactionType::Standard),
            1 => Some(TransactionType::ProofAnchor),
            2 => Some(TransactionType::AssetTransfer),
            3 => Some(TransactionType::AssetRegister),
            _ => None,
        }
    }
}

/// Proof anchoring transaction data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofAnchorData {
    /// Identity that owns this proof
    pub identity_id: [u8; 32],

    /// Type of proof being anchored
    pub proof_type: ProofType,

    /// Hash of the proof data (stored off-chain)
    pub proof_hash: [u8; 32],

    /// Optional metadata (max 256 bytes)
    pub metadata: Vec<u8>,
}

impl ProofAnchorData {
    /// Create new proof anchor data
    pub fn new(
        identity_id: [u8; 32],
        proof_type: ProofType,
        proof_hash: [u8; 32],
        metadata: Vec<u8>,
    ) -> Self {
        Self {
            identity_id,
            proof_type,
            proof_hash,
            metadata,
        }
    }

    /// Encode to bytes for transaction data field
    pub fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    /// Decode from bytes
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data).map_err(|e| format!("Failed to decode proof data: {}", e))
    }

    /// Validate proof anchor data
    pub fn validate(&self) -> Result<(), String> {
        if self.metadata.len() > 256 {
            return Err("Metadata exceeds 256 bytes".to_string());
        }
        Ok(())
    }
}

/// Asset transfer transaction data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetTransferData {
    /// Asset identifier being transferred
    pub asset_id: [u8; 32],

    /// Amount to transfer
    pub amount: u64,

    /// Recipient address
    pub recipient: [u8; 32],

    /// Optional memo
    pub memo: Option<String>,
}

impl AssetTransferData {
    /// Create new asset transfer data
    pub fn new(asset_id: [u8; 32], amount: u64, recipient: [u8; 32], memo: Option<String>) -> Self {
        Self {
            asset_id,
            amount,
            recipient,
            memo,
        }
    }

    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    /// Decode from bytes
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data).map_err(|e| format!("Failed to decode asset transfer: {}", e))
    }

    /// Validate asset transfer data
    pub fn validate(&self) -> Result<(), String> {
        if self.amount == 0 {
            return Err("Transfer amount must be greater than 0".to_string());
        }

        if let Some(ref memo) = self.memo {
            if memo.len() > 256 {
                return Err("Memo exceeds 256 characters".to_string());
            }
        }

        Ok(())
    }
}

/// Asset registration transaction data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRegisterData {
    /// Asset type
    pub asset_type: String,

    /// Asset name
    pub name: String,

    /// Asset symbol (ticker)
    pub symbol: String,

    /// Number of decimal places
    pub decimals: u8,

    /// Initial supply
    pub total_supply: u64,

    /// Is transferable?
    pub transferable: bool,

    /// Is burnable?
    pub burnable: bool,

    /// Is mintable?
    pub mintable: bool,

    /// Metadata (JSON)
    pub metadata: Vec<u8>,
}

impl AssetRegisterData {
    /// Create new asset registration data
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        asset_type: String,
        name: String,
        symbol: String,
        decimals: u8,
        total_supply: u64,
        transferable: bool,
        burnable: bool,
        mintable: bool,
        metadata: Vec<u8>,
    ) -> Self {
        Self {
            asset_type,
            name,
            symbol,
            decimals,
            total_supply,
            transferable,
            burnable,
            mintable,
            metadata,
        }
    }

    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    /// Decode from bytes
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data)
            .map_err(|e| format!("Failed to decode asset registration: {}", e))
    }

    /// Validate asset registration data
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Asset name cannot be empty".to_string());
        }

        if self.symbol.is_empty() {
            return Err("Asset symbol cannot be empty".to_string());
        }

        if self.symbol.len() > 10 {
            return Err("Symbol exceeds 10 characters".to_string());
        }

        if self.decimals > 18 {
            return Err("Decimals cannot exceed 18".to_string());
        }

        if self.total_supply == 0 {
            return Err("Total supply must be greater than 0".to_string());
        }

        if self.metadata.len() > 1024 {
            return Err("Metadata exceeds 1024 bytes".to_string());
        }

        Ok(())
    }
}

/// Helper functions for creating special transaction types
pub struct TransactionBuilder;

impl TransactionBuilder {
    /// Create a proof anchoring transaction
    pub fn create_proof_anchor(
        sender_input: TxInput,
        proof_data: ProofAnchorData,
        DIFFICULTY_ADJUSTMENT_INTERVAL: u64,
        timestamp: u64,
    ) -> Result<Transaction, String> {
        // Validate proof data
        proof_data.validate()?;

        // Create output for change (sender pays fee)
        // In a real implementation, this would calculate proper change
        let outputs = vec![]; // Proof anchoring doesn't transfer value

        // Encode proof data
        let mut data = vec![TransactionType::ProofAnchor.as_u8()];
        data.extend_from_slice(&proof_data.encode());

        Ok(Transaction::new(
            1, // version
            vec![sender_input],
            outputs,
            timestamp,
            Some(data),
        ))
    }

    /// Create an asset transfer transaction
    pub fn create_asset_transfer(
        sender_input: TxInput,
        transfer_data: AssetTransferData,
        DIFFICULTY_ADJUSTMENT_INTERVAL: u64,
        timestamp: u64,
    ) -> Result<Transaction, String> {
        // Validate transfer data
        transfer_data.validate()?;

        // Create output for the asset recipient
        // Note: Asset transfers don't use standard outputs
        // The asset registry handles the actual transfer
        let outputs = vec![];

        // Encode transfer data
        let mut data = vec![TransactionType::AssetTransfer.as_u8()];
        data.extend_from_slice(&transfer_data.encode());

        Ok(Transaction::new(
            1,
            vec![sender_input],
            outputs,
            timestamp,
            Some(data),
        ))
    }

    /// Create an asset registration transaction
    pub fn create_asset_register(
        issuer_input: TxInput,
        register_data: AssetRegisterData,
        DIFFICULTY_ADJUSTMENT_INTERVAL: u64,
        timestamp: u64,
    ) -> Result<Transaction, String> {
        // Validate registration data
        register_data.validate()?;

        let outputs = vec![];

        // Encode registration data
        let mut data = vec![TransactionType::AssetRegister.as_u8()];
        data.extend_from_slice(&register_data.encode());

        Ok(Transaction::new(
            1,
            vec![issuer_input],
            outputs,
            timestamp,
            Some(data),
        ))
    }

    /// Parse transaction type from transaction data
    pub fn get_transaction_type(tx: &Transaction) -> TransactionType {
        if let Some(ref data) = tx.data {
            if !data.is_empty() {
                if let Some(tx_type) = TransactionType::from_u8(data[0]) {
                    return tx_type;
                }
            }
        }
        TransactionType::Standard
    }

    /// Extract proof anchor data from transaction
    pub fn extract_proof_data(tx: &Transaction) -> Result<ProofAnchorData, String> {
        let tx_type = Self::get_transaction_type(tx);
        if tx_type != TransactionType::ProofAnchor {
            return Err("Not a proof anchor transaction".to_string());
        }

        if let Some(ref data) = tx.data {
            if data.len() > 1 {
                return ProofAnchorData::decode(&data[1..]);
            }
        }

        Err("No proof data in transaction".to_string())
    }

    /// Extract asset transfer data from transaction
    pub fn extract_asset_transfer(tx: &Transaction) -> Result<AssetTransferData, String> {
        let tx_type = Self::get_transaction_type(tx);
        if tx_type != TransactionType::AssetTransfer {
            return Err("Not an asset transfer transaction".to_string());
        }

        if let Some(ref data) = tx.data {
            if data.len() > 1 {
                return AssetTransferData::decode(&data[1..]);
            }
        }

        Err("No asset transfer data in transaction".to_string())
    }

    /// Extract asset registration data from transaction
    pub fn extract_asset_register(tx: &Transaction) -> Result<AssetRegisterData, String> {
        let tx_type = Self::get_transaction_type(tx);
        if tx_type != TransactionType::AssetRegister {
            return Err("Not an asset registration transaction".to_string());
        }

        if let Some(ref data) = tx.data {
            if data.len() > 1 {
                return AssetRegisterData::decode(&data[1..]);
            }
        }

        Err("No asset registration data in transaction".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Signature, TxInput};

    #[test]
    fn test_transaction_type_encoding() {
        assert_eq!(TransactionType::Standard.as_u8(), 0);
        assert_eq!(TransactionType::ProofAnchor.as_u8(), 1);
        assert_eq!(TransactionType::AssetTransfer.as_u8(), 2);
        assert_eq!(TransactionType::AssetRegister.as_u8(), 3);

        assert_eq!(TransactionType::from_u8(0), Some(TransactionType::Standard));
        assert_eq!(
            TransactionType::from_u8(1),
            Some(TransactionType::ProofAnchor)
        );
        assert_eq!(TransactionType::from_u8(99), None);
    }

    #[test]
    fn test_proof_anchor_data() {
        let proof_data = ProofAnchorData::new(
            [1u8; 32],
            ProofType::KycVerification,
            [2u8; 32],
            vec![1, 2, 3],
        );

        assert!(proof_data.validate().is_ok());

        // Test encoding/decoding
        let encoded = proof_data.encode();
        let decoded = ProofAnchorData::decode(&encoded).unwrap();
        assert_eq!(proof_data, decoded);
    }

    #[test]
    fn test_asset_transfer_data() {
        let transfer_data =
            AssetTransferData::new([1u8; 32], 1000, [2u8; 32], Some("Test memo".to_string()));

        assert!(transfer_data.validate().is_ok());

        // Test encoding/decoding
        let encoded = transfer_data.encode();
        let decoded = AssetTransferData::decode(&encoded).unwrap();
        assert_eq!(transfer_data, decoded);
    }

    #[test]
    fn test_asset_register_data() {
        let register_data = AssetRegisterData::new(
            "equity".to_string(),
            "Company Stock".to_string(),
            "STCK".to_string(),
            2,
            1_000_000,
            true,
            false,
            false,
            vec![],
        );

        assert!(register_data.validate().is_ok());

        // Test encoding/decoding
        let encoded = register_data.encode();
        let decoded = AssetRegisterData::decode(&encoded).unwrap();
        assert_eq!(register_data, decoded);
    }

    #[test]
    fn test_create_proof_anchor_transaction() {
        let input = TxInput {
            previous_output_hash: [0u8; 32],
            output_index: 0,
            signature: Signature::Classical(vec![0u8; 64]),
            public_key: vec![0u8; 33],
            nonce: None,
        };

        let proof_data = ProofAnchorData::new(
            [1u8; 32],
            ProofType::KycVerification,
            [2u8; 32],
            vec![1, 2, 3],
        );

        let tx =
            TransactionBuilder::create_proof_anchor(input, proof_data.clone(), 100, 1234567890)
                .unwrap();

        // Verify transaction type
        assert_eq!(
            TransactionBuilder::get_transaction_type(&tx),
            TransactionType::ProofAnchor
        );

        // Extract and verify data
        let extracted = TransactionBuilder::extract_proof_data(&tx).unwrap();
        assert_eq!(extracted, proof_data);
    }

    #[test]
    fn test_create_asset_transfer_transaction() {
        let input = TxInput {
            previous_output_hash: [0u8; 32],
            output_index: 0,
            signature: Signature::Classical(vec![0u8; 64]),
            public_key: vec![0u8; 33],
            nonce: None,
        };

        let transfer_data =
            AssetTransferData::new([1u8; 32], 1000, [2u8; 32], Some("Payment".to_string()));

        let tx = TransactionBuilder::create_asset_transfer(
            input,
            transfer_data.clone(),
            100,
            1234567890,
        )
        .unwrap();

        // Verify transaction type
        assert_eq!(
            TransactionBuilder::get_transaction_type(&tx),
            TransactionType::AssetTransfer
        );

        // Extract and verify data
        let extracted = TransactionBuilder::extract_asset_transfer(&tx).unwrap();
        assert_eq!(extracted, transfer_data);
    }

    #[test]
    fn test_validation_errors() {
        // Test proof anchor with oversized metadata
        let mut proof_data = ProofAnchorData::new(
            [1u8; 32],
            ProofType::KycVerification,
            [2u8; 32],
            vec![0u8; 300],
        );
        assert!(proof_data.validate().is_err());

        // Test asset transfer with zero amount
        let transfer_data = AssetTransferData::new(
            [1u8; 32], 0, // Zero amount
            [2u8; 32], None,
        );
        assert!(transfer_data.validate().is_err());

        // Test asset register with empty symbol
        let register_data = AssetRegisterData::new(
            "equity".to_string(),
            "Test Asset".to_string(),
            "".to_string(), // Empty symbol
            2,
            1000,
            true,
            false,
            false,
            vec![],
        );
        assert!(register_data.validate().is_err());
    }
}
