// Transaction Signer - Sign transactions with Post-Quantum Cryptography
//
// Uses Dilithium5 (ML-DSA) signatures for quantum-resistant transaction signing
// Compatible with Boundless blockchain's PQC signature verification

use crate::crypto::PqcKeyPair;
use crate::error::{EnterpriseError, Result};
use super::{Transaction, TxInput, Signature};
use zeroize::Zeroizing;

/// Transaction signer for creating signed transactions
pub struct TransactionSigner {
    keypair: PqcKeyPair,
}

impl TransactionSigner {
    /// Create a new transaction signer with a key pair
    pub fn new(keypair: PqcKeyPair) -> Self {
        Self { keypair }
    }

    /// Create a signer from raw key bytes
    pub fn from_keys(public_key: Vec<u8>, secret_key: Vec<u8>) -> Result<Self> {
        let keypair = PqcKeyPair::from_bytes(public_key, secret_key)?;
        Ok(Self { keypair })
    }

    /// Sign a transaction with PQC signature
    ///
    /// This method:
    /// 1. Calculates the transaction's signing hash (signature-free)
    /// 2. Signs the hash with Dilithium5 (ML-DSA)
    /// 3. Replaces all input signatures with the PQC signature
    ///
    /// **Security:** Uses signing_hash() to prevent signature malleability
    pub fn sign_transaction(&self, mut transaction: Transaction) -> Result<Transaction> {
        // Calculate signing hash (without signatures)
        let signing_hash = transaction.signing_hash();

        // Sign the hash with PQC
        let signature_bytes = self.keypair.sign_detached(&signing_hash)?;

        // Create PQC signature
        let pqc_signature = Signature::MlDsa(signature_bytes);

        // Get public key for verification
        let public_key = self.keypair.public_key_bytes().to_vec();

        // Replace all input signatures
        for input in &mut transaction.inputs {
            input.signature = pqc_signature.clone();
            input.public_key = public_key.clone();
        }

        Ok(transaction)
    }

    /// Sign a transaction with hybrid signature (Classical + PQC)
    ///
    /// For transition period when both classical and PQC signatures are needed
    /// Currently uses PQC-only since we don't have classical keys
    pub fn sign_transaction_hybrid(&self, transaction: Transaction) -> Result<Transaction> {
        // For now, just use PQC signature
        // In a full implementation, this would combine Ed25519 + Dilithium
        self.sign_transaction(transaction)
    }

    /// Sign multiple transactions in batch
    pub fn sign_batch(&self, transactions: Vec<Transaction>) -> Result<Vec<Transaction>> {
        transactions
            .into_iter()
            .map(|tx| self.sign_transaction(tx))
            .collect()
    }

    /// Get the public key bytes
    pub fn public_key(&self) -> &[u8] {
        self.keypair.public_key_bytes()
    }

    /// Get the Boundless address derived from this key pair
    pub fn address(&self) -> String {
        self.keypair.derive_address()
    }

    /// Verify that a transaction is properly signed
    ///
    /// Checks that all input signatures are valid for this signer's key
    pub fn verify_transaction(&self, transaction: &Transaction) -> Result<bool> {
        use crate::crypto::PqcSignature;

        let signing_hash = transaction.signing_hash();
        let public_key = self.keypair.public_key_bytes();

        // Check all inputs
        for input in &transaction.inputs {
            match &input.signature {
                Signature::MlDsa(sig) => {
                    let is_valid = PqcSignature::verify_detached(
                        public_key,
                        &signing_hash,
                        sig
                    )?;

                    if !is_valid {
                        return Ok(false);
                    }
                }
                Signature::Classical(_) => {
                    return Err(EnterpriseError::CryptoError(
                        "Classical signature verification not implemented".to_string()
                    ));
                }
                Signature::Falcon(_) => {
                    return Err(EnterpriseError::CryptoError(
                        "Falcon signature verification not implemented".to_string()
                    ));
                }
                Signature::Hybrid { .. } => {
                    return Err(EnterpriseError::CryptoError(
                        "Hybrid signature verification not implemented".to_string()
                    ));
                }
            }
        }

        Ok(true)
    }

    /// Sign a raw message (for testing or other use cases)
    pub fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>> {
        self.keypair.sign_detached(message)
    }
}

/// Helper function to sign a transaction with a key pair
pub fn sign_transaction(transaction: Transaction, keypair: &PqcKeyPair) -> Result<Transaction> {
    let signer = TransactionSigner::new(keypair.clone());
    signer.sign_transaction(transaction)
}

/// Helper function to create a signed transaction from an unsigned one
pub fn create_signed_transaction(
    unsigned_tx: Transaction,
    secret_key: &[u8],
    public_key: &[u8],
) -> Result<Transaction> {
    // Reconstruct keypair
    let keypair = PqcKeyPair::from_bytes(public_key.to_vec(), secret_key.to_vec())?;

    // Sign transaction
    let signer = TransactionSigner::new(keypair);
    signer.sign_transaction(unsigned_tx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TxOutput, TransactionBuilder, UnspentOutput};

    #[test]
    fn test_transaction_signer_creation() {
        let keypair = PqcKeyPair::generate().unwrap();
        let signer = TransactionSigner::new(keypair);

        assert!(!signer.public_key().is_empty());
        // Address should be 64 hex characters (32 bytes)
        assert_eq!(signer.address().len(), 64);
        assert!(hex::decode(signer.address()).is_ok());
    }

    #[test]
    fn test_sign_transaction() {
        // Create a keypair
        let keypair = PqcKeyPair::generate().unwrap();
        let address = keypair.derive_address();

        // Create an unsigned transaction
        let utxo = UnspentOutput {
            tx_hash: "a".repeat(64),
            output_index: 0,
            amount: 10000,
            script: None,
            owner_pubkey_hash: [0u8; 32],
        };

        let unsigned_tx = TransactionBuilder::new()
            .add_input(utxo, keypair.public_key_bytes().to_vec())
            .add_output(&address, 5000)
            .unwrap()
            .build_unsigned()
            .unwrap();

        // Sign the transaction
        let signer = TransactionSigner::new(keypair);
        let signed_tx = signer.sign_transaction(unsigned_tx).unwrap();

        // Verify it's signed
        assert_eq!(signed_tx.inputs.len(), 1);
        match &signed_tx.inputs[0].signature {
            Signature::MlDsa(sig) => {
                assert!(!sig.is_empty());
            }
            _ => panic!("Expected ML-DSA signature"),
        }
    }

    #[test]
    fn test_verify_transaction() {
        // Create a keypair
        let keypair = PqcKeyPair::generate().unwrap();
        let address = keypair.derive_address();

        // Create and sign a transaction
        let utxo = UnspentOutput {
            tx_hash: "a".repeat(64),
            output_index: 0,
            amount: 10000,
            script: None,
            owner_pubkey_hash: [0u8; 32],
        };

        let unsigned_tx = TransactionBuilder::new()
            .add_input(utxo, keypair.public_key_bytes().to_vec())
            .add_output(&address, 5000)
            .unwrap()
            .build_unsigned()
            .unwrap();

        let signer = TransactionSigner::new(keypair);
        let signed_tx = signer.sign_transaction(unsigned_tx).unwrap();

        // Verify the signature
        let is_valid = signer.verify_transaction(&signed_tx).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_sign_batch() {
        let keypair = PqcKeyPair::generate().unwrap();
        let address = keypair.derive_address();

        // Create multiple transactions
        let mut transactions = Vec::new();
        for i in 0..3 {
            let utxo = UnspentOutput {
                tx_hash: format!("{:0>64}", i),
                output_index: 0,
                amount: 10000,
                script: None,
                owner_pubkey_hash: [0u8; 32],
            };

            let tx = TransactionBuilder::new()
                .add_input(utxo, keypair.public_key_bytes().to_vec())
                .add_output(&address, 5000)
                .unwrap()
                .build_unsigned()
                .unwrap();

            transactions.push(tx);
        }

        // Sign batch
        let signer = TransactionSigner::new(keypair);
        let signed_txs = signer.sign_batch(transactions).unwrap();

        assert_eq!(signed_txs.len(), 3);

        // Verify all are signed
        for tx in signed_txs {
            assert!(signer.verify_transaction(&tx).unwrap());
        }
    }

    #[test]
    fn test_sign_message() {
        let keypair = PqcKeyPair::generate().unwrap();
        let signer = TransactionSigner::new(keypair);

        let message = b"Test message for signing";
        let signature = signer.sign_message(message).unwrap();

        assert!(!signature.is_empty());
    }
}
