// Contract Deployment Transaction Building
//
// Provides utilities for building and submitting contract deployment transactions
// Uses boundless_core transaction types for blockchain compatibility

use crate::blockchain::BlockchainClient;
use crate::error::{EnterpriseError, Result};
use super::{Signature, Transaction, TxInput, TxOutput}; // Use enterprise transaction types
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use sha3::{Digest, Sha3_256};
use std::sync::Arc;

/// Re-export UTXO type from blockchain module
pub use crate::blockchain::UtxoInfo;

/// Deployer key for signing contract deployment transactions
pub struct DeployerKey {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    pub public_key: Vec<u8>,
    pub address: [u8; 32],
}

impl DeployerKey {
    /// Load deployer private key from DEPLOYER_PRIVATE_KEY environment variable
    pub fn from_env() -> Result<Self> {
        let priv_key_hex = std::env::var("DEPLOYER_PRIVATE_KEY").map_err(|_| {
            EnterpriseError::ValidationError(
                "DEPLOYER_PRIVATE_KEY environment variable not set. \
                Required for contract deployment."
                    .to_string(),
            )
        })?;

        Self::from_hex(&priv_key_hex)
    }

    /// Load deployer key from hex-encoded private key
    pub fn from_hex(priv_key_hex: &str) -> Result<Self> {
        let priv_key_bytes = hex::decode(priv_key_hex.trim()).map_err(|e| {
            EnterpriseError::CryptoError(format!("Invalid DEPLOYER_PRIVATE_KEY hex: {}", e))
        })?;

        if priv_key_bytes.len() != 32 {
            return Err(EnterpriseError::CryptoError(format!(
                "Invalid DEPLOYER_PRIVATE_KEY: expected 32 bytes, got {}",
                priv_key_bytes.len()
            )));
        }

        let key_array: [u8; 32] = priv_key_bytes.try_into().map_err(|_| {
            EnterpriseError::CryptoError("Failed to convert private key to array".to_string())
        })?;

        let signing_key = SigningKey::from_bytes(&key_array);
        let verifying_key = signing_key.verifying_key();
        let public_key = verifying_key.to_bytes().to_vec();

        // Calculate address (SHA3-256 hash of public key)
        let mut hasher = Sha3_256::new();
        hasher.update(&public_key);
        let address: [u8; 32] = hasher.finalize().into();

        Ok(Self {
            signing_key,
            verifying_key,
            public_key,
            address,
        })
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }
}

/// Contract deployment transaction builder
pub struct DeploymentBuilder {
    blockchain_client: Arc<BlockchainClient>,
}

impl DeploymentBuilder {
    /// Create a new deployment builder
    pub fn new(blockchain_client: Arc<BlockchainClient>) -> Self {
        Self { blockchain_client }
    }

    /// Query UTXOs for an address
    pub async fn query_utxos(&self, address: &[u8; 32]) -> Result<Vec<UtxoInfo>> {
        let address_hex = hex::encode(address);
        let utxos = self.blockchain_client.get_utxos(&address_hex).await?;

        if utxos.is_empty() {
            tracing::warn!(
                "No UTXOs found for deployer address {}. Balance may be zero.",
                address_hex
            );
        } else {
            tracing::info!("Found {} UTXOs for deployer", utxos.len());
        }

        Ok(utxos)
    }

    /// Select UTXOs using greedy algorithm (smallest first)
    ///
    /// Returns (selected_utxos, total_input_amount, estimated_fee)
    pub fn select_utxos(
        &self,
        utxos: &[UtxoInfo],
        required_amount: u64,
    ) -> Result<(Vec<UtxoInfo>, u64, u64)> {
        if utxos.is_empty() {
            return Err(EnterpriseError::BlockchainError(
                "No UTXOs available for transaction".to_string(),
            ));
        }

        // Fee estimation
        let base_fee = 500u64;
        let per_input_fee = 1000u64;

        let mut selected_utxos = Vec::new();
        let mut total_input = 0u64;
        let mut sorted_utxos = utxos.to_vec();
        sorted_utxos.sort_by_key(|u| u.amount); // Smallest first

        for utxo in sorted_utxos {
            selected_utxos.push(utxo.clone());
            total_input += utxo.amount;

            let estimated_fee = base_fee + (selected_utxos.len() as u64 * per_input_fee);
            let required_total = required_amount + estimated_fee;

            if total_input >= required_total {
                break;
            }
        }

        let final_fee = base_fee + (selected_utxos.len() as u64 * per_input_fee);
        let required_total = required_amount + final_fee;

        if total_input < required_total {
            return Err(EnterpriseError::BlockchainError(format!(
                "Insufficient funds: have {} satoshis, need {} (amount: {}, fee: {})",
                total_input, required_total, required_amount, final_fee
            )));
        }

        tracing::info!(
            "Selected {} UTXOs (total: {} satoshis, fee: {} satoshis)",
            selected_utxos.len(),
            total_input,
            final_fee
        );

        Ok((selected_utxos, total_input, final_fee))
    }

    /// Build a contract deployment transaction
    pub fn build_deployment_transaction(
        &self,
        deployer: &DeployerKey,
        selected_utxos: &[UtxoInfo],
        total_input: u64,
        fee: u64,
        wasm_bytes: Vec<u8>,
    ) -> Result<Transaction> {
        // Create transaction inputs from selected UTXOs
        let mut tx_inputs = Vec::new();
        for utxo in selected_utxos {
            let tx_hash_bytes = hex::decode(&utxo.tx_hash).map_err(|e| {
                EnterpriseError::BlockchainError(format!("Invalid UTXO tx_hash hex: {}", e))
            })?;

            if tx_hash_bytes.len() != 32 {
                return Err(EnterpriseError::BlockchainError(format!(
                    "Invalid UTXO tx_hash length: {}",
                    tx_hash_bytes.len()
                )));
            }

            let mut tx_hash_array = [0u8; 32];
            tx_hash_array.copy_from_slice(&tx_hash_bytes);

            tx_inputs.push(TxInput {
                previous_output_hash: tx_hash_array,
                output_index: utxo.output_index,
                signature: Signature::Classical(vec![]), // Will be filled after signing
                public_key: deployer.public_key.clone(),
                nonce: None,
            });
        }

        // Contract deployment output (amount = 0, WASM in script field)
        let contract_output = TxOutput {
            amount: 0, // No value transfer for deployment
            recipient_pubkey_hash: [0u8; 32], // Special address for contract deployment
            script: Some(wasm_bytes),
        };

        let mut outputs = vec![contract_output];

        // Add change output if needed
        let change = total_input.saturating_sub(fee);
        if change > 0 {
            tracing::info!("Adding change output: {} satoshis", change);
            outputs.push(TxOutput {
                amount: change,
                recipient_pubkey_hash: deployer.address,
                script: None,
            });
        }

        // Create unsigned transaction
        let tx = Transaction {
            version: 1,
            inputs: tx_inputs,
            outputs,
            timestamp: chrono::Utc::now().timestamp() as u64,
            data: None,
        };

        Ok(tx)
    }

    /// Sign a transaction with the deployer key
    pub fn sign_transaction(
        &self,
        mut tx: Transaction,
        deployer: &DeployerKey,
    ) -> Result<Transaction> {
        let signing_hash = tx.signing_hash();
        let signature_bytes = deployer.sign(&signing_hash);

        tracing::info!(
            "Signing transaction {} with deployer key",
            hex::encode(tx.hash())
        );

        // Update all inputs with the same signature (single-sig)
        for input in &mut tx.inputs {
            input.signature = Signature::Classical(signature_bytes.clone());
        }

        Ok(tx)
    }

    /// Submit a signed transaction to the blockchain
    pub async fn submit_transaction(&self, tx: &Transaction) -> Result<String> {
        let tx_bytes = bincode::serialize(tx).map_err(|e| {
            EnterpriseError::BlockchainError(format!("Transaction serialization failed: {}", e))
        })?;
        let tx_hex = hex::encode(&tx_bytes);

        tracing::info!(
            "Submitting transaction {} ({} bytes)",
            hex::encode(tx.hash()),
            tx_bytes.len()
        );

        let tx_hash = self.blockchain_client.send_transaction(&tx_hex).await?;

        tracing::info!("Transaction submitted successfully: {}", tx_hash);

        Ok(tx_hash)
    }

    /// Poll for transaction confirmation
    ///
    /// Polls blockchain for transaction receipt with 2-second intervals
    /// Returns contract address (derived from transaction hash)
    pub async fn poll_for_confirmation(&self, tx_hash: &str, max_attempts: u32) -> Result<String> {
        use tokio::time::{sleep, Duration};

        const POLL_INTERVAL: Duration = Duration::from_secs(2);

        tracing::info!(
            "Polling for transaction {} confirmation (max {} attempts)",
            tx_hash,
            max_attempts
        );

        for attempt in 1..=max_attempts {
            match self.blockchain_client.get_transaction(tx_hash).await {
                Ok(Some(tx_info)) => {
                    if tx_info.block_height.is_some() {
                        let block_height = tx_info.block_height.unwrap();
                        tracing::info!(
                            "Transaction {} confirmed at block height {}",
                            tx_hash,
                            block_height
                        );

                        // Contract address is derived from transaction hash
                        // This matches the blockchain's contract address derivation
                        return Ok(tx_hash.to_string());
                    } else {
                        tracing::info!(
                            "Transaction {} still pending (attempt {}/{})",
                            tx_hash,
                            attempt,
                            max_attempts
                        );
                    }
                }
                Ok(None) => {
                    tracing::warn!(
                        "Transaction {} not found yet (attempt {}/{})",
                        tx_hash,
                        attempt,
                        max_attempts
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Error querying transaction {} (attempt {}/{}): {}",
                        tx_hash,
                        attempt,
                        max_attempts,
                        e
                    );
                }
            }

            if attempt < max_attempts {
                sleep(POLL_INTERVAL).await;
            }
        }

        Err(EnterpriseError::BlockchainError(format!(
            "Transaction {} not confirmed after {} attempts ({} seconds)",
            tx_hash,
            max_attempts,
            max_attempts * 2
        )))
    }

    /// Build, sign, submit, and confirm a contract deployment transaction (all-in-one)
    ///
    /// Returns the contract address (transaction hash) after confirmation
    pub async fn deploy_contract(
        &self,
        deployer: &DeployerKey,
        wasm_bytes: Vec<u8>,
    ) -> Result<String> {
        tracing::info!(
            "Starting contract deployment for deployer {} ({} bytes WASM)",
            hex::encode(deployer.address),
            wasm_bytes.len()
        );

        // Step 1: Query UTXOs
        let utxos = self.query_utxos(&deployer.address).await?;

        // Step 2: Select UTXOs (deployment itself costs 0, but we need to pay fees)
        let deployment_cost = 0u64; // Contract deployment has no value transfer
        let (selected_utxos, total_input, fee) = self.select_utxos(&utxos, deployment_cost)?;

        // Step 3: Build transaction
        let unsigned_tx = self.build_deployment_transaction(
            deployer,
            &selected_utxos,
            total_input,
            fee,
            wasm_bytes,
        )?;

        // Step 4: Sign transaction
        let signed_tx = self.sign_transaction(unsigned_tx, deployer)?;

        // Step 5: Submit to blockchain
        let tx_hash = self.submit_transaction(&signed_tx).await?;

        tracing::info!("Contract deployment transaction submitted: {}", tx_hash);

        // Step 6: Poll for confirmation (30 attempts = 60 seconds max)
        let contract_address = self.poll_for_confirmation(&tx_hash, 30).await?;

        tracing::info!(
            "Contract deployment confirmed! Contract address: {}",
            contract_address
        );

        Ok(contract_address)
    }

    /// Send a contract interaction transaction
    ///
    /// Similar to deploy_contract, but for calling existing contracts
    /// Returns the transaction hash after submission
    pub async fn send_contract_call(
        &self,
        _contract_address: &str,
        call_data: &[u8],
        deployer: &DeployerKey,
    ) -> Result<String> {
        tracing::info!(
            "Sending contract call ({} bytes call data)",
            call_data.len()
        );

        // For now, treat contract calls like deployments (putting call data in script)
        // A full implementation would have a dedicated contract call transaction type

        // Step 1: Query UTXOs
        let utxos = self.query_utxos(&deployer.address).await?;

        // Step 2: Select UTXOs (call itself costs 0, but we need to pay fees)
        let call_cost = 0u64; // Contract calls have no value transfer
        let (selected_utxos, total_input, fee) = self.select_utxos(&utxos, call_cost)?;

        // Step 3: Build transaction (using deployment builder with call data as WASM)
        let unsigned_tx = self.build_deployment_transaction(
            deployer,
            &selected_utxos,
            total_input,
            fee,
            call_data.to_vec(),
        )?;

        // Step 4: Sign transaction
        let signed_tx = self.sign_transaction(unsigned_tx, deployer)?;

        // Step 5: Submit to blockchain
        let tx_hash = self.submit_transaction(&signed_tx).await?;

        tracing::info!("Contract call transaction submitted: {}", tx_hash);

        Ok(tx_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployer_key_from_hex() {
        // Valid 32-byte hex key
        let key_hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let deployer = DeployerKey::from_hex(key_hex).unwrap();

        assert_eq!(deployer.public_key.len(), 32);
        assert_eq!(deployer.address.len(), 32);
    }

    #[test]
    fn test_deployer_key_invalid_length() {
        let key_hex = "0123456789abcdef"; // Only 8 bytes
        let result = DeployerKey::from_hex(key_hex);
        assert!(result.is_err());
    }
}
