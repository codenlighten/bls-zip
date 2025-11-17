// Boundless Enterprise - Transaction Building and Signing
//
// This module provides functionality for:
// - Building Boundless-compatible UTXO transactions
// - Signing transactions with post-quantum cryptography
// - Managing UTXOs and transaction inputs/outputs
//
// Integration with Boundless core transaction format

pub mod builder;
pub mod signer;

pub use builder::{TransactionBuilder, UnspentOutput};
pub use signer::TransactionSigner;

// Re-export core transaction types for convenience
use serde::{Deserialize, Serialize};

/// Transaction signature type (compatible with Boundless core)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Transaction input (reference to previous output)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TxOutput {
    /// Amount of tokens
    pub amount: u64,

    /// Public key hash of the recipient
    pub recipient_pubkey_hash: [u8; 32],

    /// Optional script or data field
    pub script: Option<Vec<u8>>,
}

/// Complete transaction (compatible with Boundless core)
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
    /// Calculate the SHA3-256 hash of this transaction
    pub fn hash(&self) -> [u8; 32] {
        use sha3::{Digest, Sha3_256};
        // FIX L-6: Log serialization errors instead of silent defaults
        let serialized = bincode::serialize(self).unwrap_or_else(|e| {
            tracing::error!("Transaction serialization failed during hash: {}", e);
            vec![]
        });
        let mut hasher = Sha3_256::new();
        hasher.update(&serialized);
        hasher.finalize().into()
    }

    /// Calculate the signing hash (signature-free hash for verification)
    ///
    /// FIX H-2: Hash transaction fields individually to prevent signature malleability
    /// This creates a canonical hash regardless of signature type
    pub fn signing_hash(&self) -> [u8; 32] {
        use sha3::{Digest, Sha3_256};

        let mut hasher = Sha3_256::new();

        // Hash version
        hasher.update(&self.version.to_le_bytes());

        // Hash timestamp
        hasher.update(&self.timestamp.to_le_bytes());

        // Hash inputs (WITHOUT signatures)
        hasher.update(&(self.inputs.len() as u64).to_le_bytes());
        for input in &self.inputs {
            hasher.update(&input.previous_output_hash);
            hasher.update(&input.output_index.to_le_bytes());
            // Hash public key but NOT signature
            hasher.update(&(input.public_key.len() as u64).to_le_bytes());
            hasher.update(&input.public_key);
            // Hash nonce if present
            if let Some(nonce) = input.nonce {
                hasher.update(&[1u8]); // nonce present
                hasher.update(&nonce.to_le_bytes());
            } else {
                hasher.update(&[0u8]); // no nonce
            }
        }

        // Hash outputs
        hasher.update(&(self.outputs.len() as u64).to_le_bytes());
        for output in &self.outputs {
            hasher.update(&output.amount.to_le_bytes());
            hasher.update(&output.recipient_pubkey_hash);
            if let Some(script) = &output.script {
                hasher.update(&[1u8]); // script present
                hasher.update(&(script.len() as u64).to_le_bytes());
                hasher.update(script);
            } else {
                hasher.update(&[0u8]); // no script
            }
        }

        // Hash optional data payload
        if let Some(data) = &self.data {
            hasher.update(&[1u8]); // data present
            hasher.update(&(data.len() as u64).to_le_bytes());
            hasher.update(data);
        } else {
            hasher.update(&[0u8]); // no data
        }

        hasher.finalize().into()
    }

    /// Get the transaction ID (hex-encoded hash)
    pub fn txid(&self) -> String {
        hex::encode(self.hash())
    }

    /// Calculate the total output amount
    pub fn total_output(&self) -> u64 {
        self.outputs.iter().map(|out| out.amount).sum()
    }

    /// Get the size of this transaction in bytes
    pub fn size_bytes(&self) -> usize {
        // FIX L-6: Log serialization errors instead of silent defaults
        bincode::serialize(self).unwrap_or_else(|e| {
            tracing::error!("Transaction serialization failed during size calculation: {}", e);
            vec![]
        }).len()
    }
}
