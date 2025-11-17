// Transaction structures with Post-Quantum Cryptography support
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

use crate::error::CoreError;

/// Signature type supporting both classical and post-quantum algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Signature {
    /// Classical ECDSA signature (secp256k1 or P-256)
    Classical(Vec<u8>),

    /// Post-Quantum ML-DSA (Dilithium) signature
    MlDsa(Vec<u8>),

    /// Post-Quantum Falcon signature
    Falcon(Vec<u8>),

    /// Hybrid signature (Classical + PQC for transition period)
    Hybrid { classical: Vec<u8>, pqc: Vec<u8> },
}

impl Signature {
    /// Get the total size of the signature in bytes
    pub fn size_bytes(&self) -> usize {
        match self {
            Signature::Classical(sig) => sig.len(),
            Signature::MlDsa(sig) => sig.len(),
            Signature::Falcon(sig) => sig.len(),
            Signature::Hybrid { classical, pqc } => classical.len() + pqc.len(),
        }
    }
}

/// Transaction input (reference to previous output)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxInput {
    /// Hash of the transaction containing the output being spent
    pub previous_output_hash: [u8; 32],

    /// Index of the output in the previous transaction
    pub output_index: u32,

    /// Signature proving ownership
    pub signature: Signature,

    /// Public key (for verification)
    pub public_key: Vec<u8>,

    /// Optional nonce for replay protection
    pub nonce: Option<u64>,
}

/// Transaction output (new UTXO)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxOutput {
    /// Amount of tokens
    pub amount: u64,

    /// Public key hash of the recipient
    pub recipient_pubkey_hash: [u8; 32],

    /// Optional script or data field
    pub script: Option<Vec<u8>>,
}

/// Complete transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// Transaction version
    pub version: u32,

    /// List of inputs
    pub inputs: Vec<TxInput>,

    /// List of outputs
    pub outputs: Vec<TxOutput>,

    /// Transaction timestamp
    pub timestamp: u64,

    /// Optional data payload (for smart contract calls, etc.)
    pub data: Option<Vec<u8>>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        version: u32,
        inputs: Vec<TxInput>,
        outputs: Vec<TxOutput>,
        timestamp: u64,
        data: Option<Vec<u8>>,
    ) -> Self {
        Self {
            version,
            inputs,
            outputs,
            timestamp,
            data,
        }
    }

    /// Calculate the SHA3-256 hash of this transaction
    pub fn hash(&self) -> [u8; 32] {
        let serialized = bincode::serialize(self).unwrap_or_default();
        let mut hasher = Sha3_256::new();
        hasher.update(&serialized);
        hasher.finalize().into()
    }

    /// Calculate the signing hash (signature-free hash for verification)
    ///
    /// This method creates a hash of the transaction WITHOUT including signatures,
    /// preventing signature malleability attacks. This is the hash that should be
    /// used for signature verification.
    ///
    /// **Security Critical**: This prevents transaction ID malleability by ensuring
    /// that the same transaction content always produces the same signing hash,
    /// regardless of signature variations.
    pub fn signing_hash(&self) -> [u8; 32] {
        // Create a copy of the transaction with empty signatures
        let mut tx_copy = self.clone();

        // Clear all signatures from inputs
        for input in &mut tx_copy.inputs {
            input.signature = Signature::Classical(vec![]);
        }

        // Serialize and hash the signature-free transaction
        let serialized = bincode::serialize(&tx_copy).unwrap_or_default();
        let mut hasher = Sha3_256::new();
        hasher.update(&serialized);
        hasher.finalize().into()
    }

    /// Get the transaction ID (hex-encoded hash)
    pub fn txid(&self) -> String {
        hex::encode(self.hash())
    }

    /// Calculate the total input amount
    pub fn total_input(&self) -> u64 {
        // Note: In a real implementation, this would look up the actual
        // values from the blockchain state. For now, we return 0.
        0
    }

    /// Calculate the total output amount
    pub fn total_output(&self) -> u64 {
        self.outputs.iter().map(|out| out.amount).sum()
    }

    /// Calculate the transaction fee (inputs - outputs)
    ///
    /// NOTE: This method cannot calculate fees accurately without blockchain state access.
    /// Use `BlockchainState::calculate_transaction_fee()` instead for accurate fee calculation.
    /// This method is kept for API compatibility and returns 0.
    ///
    /// See: `BlockchainState::calculate_transaction_fee()` in `core/src/state.rs`
    pub fn fee(&self) -> u64 {
        0 // Placeholder - use BlockchainState::calculate_transaction_fee() for real fees
    }

    /// Basic validation of transaction structure
    pub fn validate(&self) -> Result<(), CoreError> {
        // SECURITY FIX: Validate transaction size limits
        self.validate_size()?;

        // SECURITY FIX: Validate input/output counts
        self.validate_counts()?;

        // Check that we have at least one input and one output
        if self.inputs.is_empty() {
            return Err(CoreError::NoInputs);
        }

        if self.outputs.is_empty() {
            return Err(CoreError::NoOutputs);
        }

        // Check that output amounts are non-zero and don't overflow
        let mut total: u64 = 0;
        for output in &self.outputs {
            if output.amount == 0 {
                return Err(CoreError::ZeroAmount);
            }
            total = total
                .checked_add(output.amount)
                .ok_or(CoreError::AmountOverflow)?;
        }

        // Signature verification would happen here with blockchain state context
        // (requires access to the actual public keys and previous outputs)

        Ok(())
    }

    /// Validate transaction size limits
    ///
    /// SECURITY: Prevents DoS attacks via oversized transactions
    /// Maximum transaction size: 1MB
    pub fn validate_size(&self) -> Result<(), CoreError> {
        const MAX_TRANSACTION_SIZE: usize = 1_000_000; // 1MB

        let size = self.size_bytes();
        if size > MAX_TRANSACTION_SIZE {
            return Err(CoreError::TransactionSizeExceeded {
                size,
                max: MAX_TRANSACTION_SIZE,
            });
        }

        Ok(())
    }

    /// Validate input/output counts
    ///
    /// SECURITY: Prevents DoS attacks via excessive inputs/outputs
    /// Maximum inputs: 1000
    /// Maximum outputs: 1000
    pub fn validate_counts(&self) -> Result<(), CoreError> {
        const MAX_INPUTS: usize = 1000;
        const MAX_OUTPUTS: usize = 1000;

        if self.inputs.len() > MAX_INPUTS {
            return Err(CoreError::TooManyInputs {
                count: self.inputs.len(),
                max: MAX_INPUTS,
            });
        }

        if self.outputs.len() > MAX_OUTPUTS {
            return Err(CoreError::TooManyOutputs {
                count: self.outputs.len(),
                max: MAX_OUTPUTS,
            });
        }

        Ok(())
    }

    /// Calculate and validate transaction fee
    ///
    /// SECURITY: Prevents spam transactions by enforcing minimum fee per byte
    /// Minimum fee: 100 satoshis per byte
    ///
    /// NOTE: This requires total_input_amount from blockchain state.
    /// For validation without state, use `validate_fee_with_input_amount()`
    pub fn validate_fee(&self, total_input_amount: u64) -> Result<u64, CoreError> {
        const MIN_FEE_PER_BYTE: u64 = 100; // satoshis per byte

        let total_output = self.total_output();

        // Calculate fee with overflow protection
        let fee = total_input_amount
            .checked_sub(total_output)
            .ok_or(CoreError::InvalidTransaction(
                "Output amount exceeds input amount".to_string(),
            ))?;

        // Calculate minimum required fee
        let size = self.size_bytes();
        let minimum_fee = (size as u64)
            .checked_mul(MIN_FEE_PER_BYTE)
            .ok_or(CoreError::AmountOverflow)?;

        // Validate fee meets minimum
        if fee < minimum_fee {
            return Err(CoreError::FeeTooLow {
                actual: fee,
                minimum: minimum_fee,
                min_fee_per_byte: MIN_FEE_PER_BYTE,
                size_bytes: size,
            });
        }

        Ok(fee)
    }

    /// Calculate fee given total input amount
    ///
    /// This is a helper method that calculates the fee when you have the
    /// total input amount from blockchain state.
    pub fn calculate_fee(&self, total_input_amount: u64) -> Result<u64, CoreError> {
        let total_output = self.total_output();

        total_input_amount
            .checked_sub(total_output)
            .ok_or(CoreError::InvalidTransaction(
                "Output amount exceeds input amount".to_string(),
            ))
    }

    /// Verify input signature with PQC support
    ///
    /// Validates a transaction input signature using the provided public key.
    /// Supports classical, PQC, and hybrid signature schemes.
    pub fn verify_input_signature(
        &self,
        input_index: usize,
        public_key: &[u8],
    ) -> Result<bool, CoreError> {
        use boundless_crypto::{
            Falcon512, HybridSignature, HybridSignatureData, HybridSignaturePublicKey, MlDsa44,
        };
        use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};

        if input_index >= self.inputs.len() {
            return Err(CoreError::InvalidTransaction(format!(
                "Input index {} out of bounds",
                input_index
            )));
        }

        let input = &self.inputs[input_index];
        // SECURITY FIX: Use signing_hash() instead of hash() to prevent signature malleability
        let tx_hash = self.signing_hash();

        match &input.signature {
            Signature::Hybrid { classical, pqc } => {
                let hybrid_verifier = HybridSignature::new()
                    .map_err(|e| CoreError::CryptoError(format!("Hybrid signature init: {}", e)))?;

                let hybrid_sig = HybridSignatureData {
                    classical: classical.clone(),
                    pqc: pqc.clone(),
                };

                if public_key.len() < 32 {
                    return Err(CoreError::InvalidPublicKey);
                }

                let classical_vk = &public_key[0..32];
                let pqc_pk = &public_key[32..];

                let hybrid_pk = HybridSignaturePublicKey {
                    classical_verifying: classical_vk.to_vec(),
                    pqc_public: pqc_pk.to_vec(),
                };

                hybrid_verifier
                    .verify(&tx_hash, &hybrid_sig, &hybrid_pk)
                    .map_err(|e| CoreError::CryptoError(format!("Hybrid verification: {}", e)))
            }
            Signature::MlDsa(sig) => {
                let verifier = MlDsa44::new()
                    .map_err(|e| CoreError::CryptoError(format!("ML-DSA init: {}", e)))?;

                verifier
                    .verify(&tx_hash, sig, public_key)
                    .map_err(|e| CoreError::CryptoError(format!("ML-DSA verification: {}", e)))
            }
            Signature::Falcon(sig) => {
                let verifier = Falcon512::new()
                    .map_err(|e| CoreError::CryptoError(format!("Falcon init: {}", e)))?;

                verifier
                    .verify(&tx_hash, sig, public_key)
                    .map_err(|e| CoreError::CryptoError(format!("Falcon verification: {}", e)))
            }
            Signature::Classical(sig) => {
                if public_key.len() != 32 {
                    return Err(CoreError::InvalidPublicKey);
                }

                let vk_bytes: [u8; 32] = public_key
                    .try_into()
                    .map_err(|_| CoreError::InvalidPublicKey)?;

                let vk =
                    VerifyingKey::from_bytes(&vk_bytes).map_err(|_| CoreError::InvalidPublicKey)?;

                if sig.len() != 64 {
                    return Err(CoreError::InvalidSignature);
                }

                let sig_bytes: [u8; 64] = sig
                    .as_slice()
                    .try_into()
                    .map_err(|_| CoreError::InvalidSignature)?;

                let ed_sig = Ed25519Signature::from_bytes(&sig_bytes);

                vk.verify(&tx_hash, &ed_sig)
                    .map(|_| true)
                    .map_err(|_| CoreError::InvalidSignature)
            }
        }
    }

    /// Get the size of this transaction in bytes
    pub fn size_bytes(&self) -> usize {
        bincode::serialize(self).unwrap_or_default().len()
    }

    /// Check if this is a coinbase transaction (first tx in block, no inputs)
    pub fn is_coinbase(&self) -> bool {
        self.inputs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_hash() {
        let tx = Transaction::new(
            1,
            vec![],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        let hash = tx.hash();
        assert_eq!(hash.len(), 32);

        let txid = tx.txid();
        assert_eq!(txid.len(), 64); // hex encoding doubles length
    }

    #[test]
    fn test_signature_size() {
        let classical = Signature::Classical(vec![0u8; 64]);
        assert_eq!(classical.size_bytes(), 64);

        let ml_dsa = Signature::MlDsa(vec![0u8; 2420]);
        assert_eq!(ml_dsa.size_bytes(), 2420);

        let hybrid = Signature::Hybrid {
            classical: vec![0u8; 64],
            pqc: vec![0u8; 2420],
        };
        assert_eq!(hybrid.size_bytes(), 2484);
    }

    #[test]
    fn test_transaction_validation() {
        let valid_tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        assert!(valid_tx.validate().is_ok());

        // Test invalid tx (no inputs)
        let invalid_tx = Transaction::new(
            1,
            vec![],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        assert!(invalid_tx.validate().is_err());
    }

    // SECURITY TESTS: Transaction Fee Validation

    #[test]
    fn test_fee_too_low() {
        // Create a transaction with inputs and outputs
        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        // Calculate the minimum fee required
        let size = tx.size_bytes();
        let minimum_fee = (size as u64) * 100; // 100 satoshis per byte

        // Try with input amount that results in fee below minimum
        let total_input = 1000 + (minimum_fee / 2); // Fee will be only half the minimum

        let result = tx.validate_fee(total_input);
        assert!(matches!(result, Err(CoreError::FeeTooLow { .. })));
    }

    #[test]
    fn test_fee_minimum_acceptable() {
        // Create a simple transaction
        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        // Calculate exact minimum fee
        let size = tx.size_bytes();
        let minimum_fee = (size as u64) * 100;
        let total_input = 1000 + minimum_fee;

        // Should succeed with exact minimum fee
        let result = tx.validate_fee(total_input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), minimum_fee);
    }

    #[test]
    fn test_fee_above_minimum() {
        // Create a transaction
        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        // Use fee well above minimum
        let size = tx.size_bytes();
        let minimum_fee = (size as u64) * 100;
        let actual_fee = minimum_fee * 2;
        let total_input = 1000 + actual_fee;

        // Should succeed
        let result = tx.validate_fee(total_input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), actual_fee);
    }

    #[test]
    fn test_fee_output_exceeds_input() {
        // Create a transaction
        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        // Input amount less than output amount (invalid)
        let total_input = 500; // Less than 1000 output

        let result = tx.validate_fee(total_input);
        assert!(matches!(
            result,
            Err(CoreError::InvalidTransaction(_))
        ));
    }

    #[test]
    fn test_calculate_fee() {
        // Create a transaction
        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        // Calculate fee with valid input amount
        let total_input = 1500;
        let result = tx.calculate_fee(total_input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 500); // 1500 - 1000 = 500
    }

    // SECURITY TESTS: Transaction Size Limits

    #[test]
    fn test_transaction_size_within_limit() {
        // Create a normal-sized transaction
        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        // Should pass size validation
        assert!(tx.validate_size().is_ok());
    }

    #[test]
    fn test_transaction_size_exceeded() {
        // Create a transaction with a large data payload that exceeds 1MB
        let large_data = vec![0u8; 1_100_000]; // 1.1MB of data

        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            Some(large_data),
        );

        // Should fail size validation
        let result = tx.validate_size();
        assert!(matches!(
            result,
            Err(CoreError::TransactionSizeExceeded { .. })
        ));
    }

    #[test]
    fn test_transaction_at_max_size() {
        // Create a transaction close to but under 1MB limit
        // Account for serialization overhead (roughly 200 bytes)
        let data_size = 1_000_000 - 300;
        let data = vec![0u8; data_size];

        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            Some(data),
        );

        // Should pass (just under limit)
        assert!(tx.validate_size().is_ok());
    }

    // SECURITY TESTS: Input/Output Count Limits

    #[test]
    fn test_too_many_inputs() {
        // Create a transaction with more than 1000 inputs
        let mut inputs = Vec::new();
        for _ in 0..1001 {
            inputs.push(TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            });
        }

        let tx = Transaction::new(
            1,
            inputs,
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        // Should fail count validation
        let result = tx.validate_counts();
        assert!(matches!(result, Err(CoreError::TooManyInputs { .. })));
    }

    #[test]
    fn test_too_many_outputs() {
        // Create a transaction with more than 1000 outputs
        let mut outputs = Vec::new();
        for _ in 0..1001 {
            outputs.push(TxOutput {
                amount: 1,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            });
        }

        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            outputs,
            1234567890,
            None,
        );

        // Should fail count validation
        let result = tx.validate_counts();
        assert!(matches!(result, Err(CoreError::TooManyOutputs { .. })));
    }

    #[test]
    fn test_max_inputs_allowed() {
        // Create a transaction with exactly 1000 inputs (at limit)
        let mut inputs = Vec::new();
        for _ in 0..1000 {
            inputs.push(TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            });
        }

        let tx = Transaction::new(
            1,
            inputs,
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        // Should pass count validation
        assert!(tx.validate_counts().is_ok());
    }

    #[test]
    fn test_max_outputs_allowed() {
        // Create a transaction with exactly 1000 outputs (at limit)
        let mut outputs = Vec::new();
        for _ in 0..1000 {
            outputs.push(TxOutput {
                amount: 1,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            });
        }

        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            outputs,
            1234567890,
            None,
        );

        // Should pass count validation
        assert!(tx.validate_counts().is_ok());
    }

    #[test]
    fn test_validate_integrates_all_checks() {
        // Test that validate() calls all security checks

        // 1. Test that oversized transaction fails via validate()
        let large_data = vec![0u8; 1_100_000];
        let oversized_tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            Some(large_data),
        );
        assert!(oversized_tx.validate().is_err());

        // 2. Test that too many inputs fails via validate()
        let mut many_inputs = Vec::new();
        for _ in 0..1001 {
            many_inputs.push(TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            });
        }
        let too_many_inputs_tx = Transaction::new(
            1,
            many_inputs,
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );
        assert!(too_many_inputs_tx.validate().is_err());
    }
}
