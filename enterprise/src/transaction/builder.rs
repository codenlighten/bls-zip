// Transaction Builder - Construct Boundless transactions
//
// Provides a fluent API for building UTXO-based transactions
// Compatible with Boundless blockchain's transaction format

use crate::error::{EnterpriseError, Result};
use super::{Transaction, TxInput, TxOutput, Signature};
use serde::{Deserialize, Serialize};
use sha3::Sha3_256;

/// Represents an unspent transaction output (UTXO)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnspentOutput {
    /// Transaction hash containing this output
    pub tx_hash: String,

    /// Output index in the transaction
    pub output_index: u32,

    /// Amount in this UTXO
    pub amount: u64,

    /// Script or locking condition (if any)
    pub script: Option<Vec<u8>>,

    /// Public key hash that owns this UTXO
    pub owner_pubkey_hash: [u8; 32],
}

impl UnspentOutput {
    /// Get the transaction hash as bytes
    pub fn tx_hash_bytes(&self) -> Result<[u8; 32]> {
        let hash_bytes = hex::decode(&self.tx_hash)
            .map_err(|e| EnterpriseError::CryptoError(format!("Invalid tx_hash: {}", e)))?;

        if hash_bytes.len() != 32 {
            return Err(EnterpriseError::CryptoError(
                format!("Invalid tx_hash length: expected 32, got {}", hash_bytes.len())
            ));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(&hash_bytes);
        Ok(array)
    }
}

/// Transaction builder for creating Boundless transactions
pub struct TransactionBuilder {
    version: u32,
    inputs: Vec<(UnspentOutput, Vec<u8>)>, // (UTXO, public_key)
    outputs: Vec<TxOutput>,
    data: Option<Vec<u8>>,
    fee_rate: u64, // Satoshis per byte
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new() -> Self {
        Self {
            version: 1,
            inputs: Vec::new(),
            outputs: Vec::new(),
            data: None,
            fee_rate: 100, // Default 100 satoshis per byte
        }
    }

    /// Set transaction version
    pub fn version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    /// Set fee rate (satoshis per byte)
    pub fn fee_rate(mut self, fee_rate: u64) -> Self {
        self.fee_rate = fee_rate;
        self
    }

    /// Add an input (UTXO to spend)
    pub fn add_input(mut self, utxo: UnspentOutput, public_key: Vec<u8>) -> Self {
        self.inputs.push((utxo, public_key));
        self
    }

    /// Add an output (recipient)
    pub fn add_output(mut self, recipient_address: &str, amount: u64) -> Result<Self> {
        let recipient_pubkey_hash = Self::address_to_pubkey_hash(recipient_address)?;

        self.outputs.push(TxOutput {
            amount,
            recipient_pubkey_hash,
            script: None,
        });

        Ok(self)
    }

    /// Add an output with a script
    pub fn add_output_with_script(
        mut self,
        recipient_address: &str,
        amount: u64,
        script: Vec<u8>,
    ) -> Result<Self> {
        let recipient_pubkey_hash = Self::address_to_pubkey_hash(recipient_address)?;

        self.outputs.push(TxOutput {
            amount,
            recipient_pubkey_hash,
            script: Some(script),
        });

        Ok(self)
    }

    /// Add a change output (returns change to sender)
    pub fn add_change_output(self, change_address: &str, amount: u64) -> Result<Self> {
        if amount > 0 {
            self.add_output(change_address, amount)
        } else {
            Ok(self)
        }
    }

    /// Set transaction data payload
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Calculate total input amount
    fn total_input(&self) -> u64 {
        self.inputs.iter().map(|(utxo, _)| utxo.amount).sum()
    }

    /// Calculate total output amount
    fn total_output(&self) -> u64 {
        self.outputs.iter().map(|out| out.amount).sum()
    }

    /// Estimate transaction size (without signatures)
    /// FIX L-7: Use named constants for component sizes to improve maintainability
    fn estimate_size(&self) -> usize {
        // Component size constants (in bytes)
        const VERSION_SIZE: usize = 4;
        const TIMESTAMP_SIZE: usize = 8;
        const TX_HASH_SIZE: usize = 32;
        const OUTPUT_INDEX_SIZE: usize = 4;
        const DILITHIUM5_SIGNATURE_SIZE: usize = 3500; // ML-DSA-87 typical signature
        const DILITHIUM5_PUBKEY_SIZE: usize = 2048;    // ML-DSA-87 public key
        const NONCE_SIZE: usize = 8;
        const AMOUNT_SIZE: usize = 8;
        const PUBKEY_HASH_SIZE: usize = 32;

        // Base transaction overhead
        let mut size = VERSION_SIZE + TIMESTAMP_SIZE;

        // Calculate input size
        const INPUT_SIZE: usize = TX_HASH_SIZE
            + OUTPUT_INDEX_SIZE
            + DILITHIUM5_SIGNATURE_SIZE
            + DILITHIUM5_PUBKEY_SIZE
            + NONCE_SIZE;
        size += self.inputs.len() * INPUT_SIZE;

        // Calculate output size
        const OUTPUT_SIZE: usize = AMOUNT_SIZE + PUBKEY_HASH_SIZE;
        size += self.outputs.len() * OUTPUT_SIZE;

        // Add data field size if present
        if let Some(ref data) = self.data {
            size += data.len();
        }

        size
    }

    /// Calculate recommended fee based on transaction size
    pub fn calculate_fee(&self) -> u64 {
        let estimated_size = self.estimate_size() as u64;
        estimated_size * self.fee_rate
    }

    /// Build an unsigned transaction
    ///
    /// Returns an unsigned transaction that needs to be signed using TransactionSigner
    pub fn build_unsigned(self) -> Result<Transaction> {
        // Validate inputs and outputs
        if self.inputs.is_empty() {
            return Err(EnterpriseError::ValidationError(
                "Transaction must have at least one input".to_string()
            ));
        }

        if self.outputs.is_empty() {
            return Err(EnterpriseError::ValidationError(
                "Transaction must have at least one output".to_string()
            ));
        }

        // FIX M-9: Generate nonce for replay protection
        // Use timestamp as base nonce (ensures uniqueness per transaction)
        let base_nonce = Self::current_timestamp();

        // Create unsigned inputs (empty signatures)
        let mut tx_inputs = Vec::new();
        for (idx, (utxo, public_key)) in self.inputs.into_iter().enumerate() {
            tx_inputs.push(TxInput {
                previous_output_hash: utxo.tx_hash_bytes()?,
                output_index: utxo.output_index,
                signature: Signature::Classical(vec![]), // Empty signature
                public_key,
                nonce: Some(base_nonce + idx as u64), // Unique nonce per input
            });
        }

        // Create transaction
        let tx = Transaction {
            version: self.version,
            inputs: tx_inputs,
            outputs: self.outputs,
            timestamp: Self::current_timestamp(),
            data: self.data,
        };

        Ok(tx)
    }

    /// Build a transaction with automatic fee calculation
    ///
    /// Automatically adds a change output if needed
    pub fn build_with_change(mut self, change_address: &str) -> Result<Transaction> {
        let total_in = self.total_input();
        let total_out = self.total_output();
        let fee = self.calculate_fee();

        // Validate sufficient funds
        if total_in < total_out + fee {
            return Err(EnterpriseError::ValidationError(
                format!("Insufficient funds: need {} (output) + {} (fee) = {}, have {}",
                    total_out, fee, total_out + fee, total_in)
            ));
        }

        // Calculate change
        let change = total_in - total_out - fee;

        // Add change output if significant (dust threshold: 1000 satoshis)
        if change > 1000 {
            self = self.add_change_output(change_address, change)?;
        }

        self.build_unsigned()
    }

    /// Convert a Boundless address (64 hex chars) to public key hash
    /// FIX: Updated to use 32-byte addresses (64 hex) to align with blockchain spec
    fn address_to_pubkey_hash(address: &str) -> Result<[u8; 32]> {
        // Validate address length (64 hex chars = 32 bytes)
        if address.len() != 64 {
            return Err(EnterpriseError::ValidationError(
                format!("Invalid Boundless address: must be 64 hex characters, got {}", address.len())
            ));
        }

        // Decode hex to bytes
        let hash_bytes = hex::decode(address)
            .map_err(|e| EnterpriseError::ValidationError(
                format!("Invalid address hex: {}", e)
            ))?;

        // Verify we have exactly 32 bytes
        if hash_bytes.len() != 32 {
            return Err(EnterpriseError::ValidationError(
                format!("Invalid address hash length: expected 32 bytes, got {}", hash_bytes.len())
            ));
        }

        // Convert to [u8; 32]
        let mut pubkey_hash = [0u8; 32];
        pubkey_hash.copy_from_slice(&hash_bytes);

        Ok(pubkey_hash)
    }

    /// Get current Unix timestamp in seconds
    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_builder_basic() {
        let utxo = UnspentOutput {
            tx_hash: "a".repeat(64), // 32 bytes in hex
            output_index: 0,
            amount: 10000,
            script: None,
            owner_pubkey_hash: [0u8; 32],
        };

        let result = TransactionBuilder::new()
            .add_input(utxo, vec![1u8; 32])
            .add_output(&"b".repeat(64), 5000)  // 64 hex chars = 32 bytes
            .unwrap()
            .build_unsigned();

        assert!(result.is_ok());
        let tx = result.unwrap();
        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 1);
    }

    #[test]
    fn test_fee_calculation() {
        let builder = TransactionBuilder::new()
            .fee_rate(100);

        let fee = builder.calculate_fee();
        assert!(fee > 0);
    }

    #[test]
    fn test_address_to_pubkey_hash() {
        let address = "a".repeat(64);  // 64 hex chars = 32 bytes
        let result = TransactionBuilder::address_to_pubkey_hash(&address);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_address() {
        let result = TransactionBuilder::address_to_pubkey_hash("invalid_address");
        assert!(result.is_err());
    }
}
