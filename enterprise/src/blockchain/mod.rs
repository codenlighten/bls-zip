// Blockchain HTTP Client - Interface to Boundless Node via HTTP REST Bridge
//
// Connects to the Boundless HTTP REST bridge (NOT JSON-RPC)
// Compatible with the HTTP bridge endpoints at /api/v1/*

use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::time::Duration;

use crate::error::{EnterpriseError, Result};

/// Boundless blockchain HTTP client
/// Connects to the HTTP REST bridge (port 3001 by default)
pub struct BlockchainClient {
    http_url: String,
    client: Client,
}

impl BlockchainClient {
    /// Create new blockchain client
    pub fn new(http_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { http_url, client }
    }

    // ========================================================================
    // FIX H-3: Response Validation Helpers
    // ========================================================================

    /// Validate SHA-256 hash format (64 hex characters)
    fn validate_hash(hash: &str, field_name: &str) -> Result<()> {
        if hash.len() != 64 {
            return Err(EnterpriseError::BlockchainError(format!(
                "Invalid {}: expected 64 hex chars, got {}",
                field_name, hash.len()
            )));
        }

        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(EnterpriseError::BlockchainError(format!(
                "Invalid {}: contains non-hex characters",
                field_name
            )));
        }

        Ok(())
    }

    /// Validate address format (should be 64 hex characters for SHA-256 pubkey hash)
    fn validate_address(address: &str, field_name: &str) -> Result<()> {
        // Boundless addresses are 32-byte pubkey hashes encoded as 64 hex chars
        if address.len() != 64 {
            return Err(EnterpriseError::BlockchainError(format!(
                "Invalid {}: expected 64 hex chars, got {}",
                field_name, address.len()
            )));
        }

        if !address.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(EnterpriseError::BlockchainError(format!(
                "Invalid {}: contains non-hex characters",
                field_name
            )));
        }

        Ok(())
    }

    /// Validate response matches request (prevents malicious node from returning wrong data)
    fn validate_response_match(expected: &str, actual: &str, field_name: &str) -> Result<()> {
        if expected != actual {
            return Err(EnterpriseError::BlockchainError(format!(
                "Response validation failed: {} mismatch (expected: {}, got: {})",
                field_name, expected, actual
            )));
        }
        Ok(())
    }

    /// Create from environment variable or default
    pub fn from_env() -> Self {
        let http_url = std::env::var("BOUNDLESS_HTTP_URL")
            .unwrap_or_else(|_| "http://localhost:3001".to_string());
        Self::new(http_url)
    }

    /// Get chain information
    pub async fn get_chain_info(&self) -> Result<ChainInfo> {
        let response = self
            .client
            .get(format!("{}/api/v1/chain/info", self.http_url))
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(EnterpriseError::BlockchainError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        response
            .json::<ChainInfo>()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("Failed to parse response: {}", e)))
    }

    /// Get current block height
    pub async fn get_block_height(&self) -> Result<u64> {
        let response = self
            .client
            .get(format!("{}/api/v1/chain/height", self.http_url))
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(EnterpriseError::BlockchainError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let height_response: BlockHeightResponse = response
            .json()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("Failed to parse response: {}", e)))?;

        Ok(height_response.height)
    }

    /// Get balance for an address
    pub async fn get_balance(&self, address: &str) -> Result<BalanceInfo> {
        // FIX H-3: Validate request address format
        Self::validate_address(address, "request address")?;

        let response = self
            .client
            .get(format!("{}/api/v1/balance/{}", self.http_url, address))
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(EnterpriseError::BlockchainError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let balance_info = response
            .json::<BalanceInfo>()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("Failed to parse response: {}", e)))?;

        // FIX H-3: Validate response address matches request (prevents malicious node attacks)
        Self::validate_response_match(address, &balance_info.address, "balance address")?;

        Ok(balance_info)
    }

    /// Send a signed transaction to the blockchain
    ///
    /// The transaction must be hex-encoded
    pub async fn send_transaction(&self, transaction_hex: &str) -> Result<String> {
        let request = SendTransactionRequest {
            transaction_hex: transaction_hex.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/api/v1/transaction/send", self.http_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(EnterpriseError::BlockchainError(format!(
                "Transaction failed: {}",
                error_text
            )));
        }

        let tx_response: SendTransactionResponse = response
            .json()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("Failed to parse response: {}", e)))?;

        // FIX H-3: Validate transaction hash format
        Self::validate_hash(&tx_response.tx_hash, "transaction hash")?;

        Ok(tx_response.tx_hash)
    }

    /// Get transaction by hash
    ///
    /// NOTE: Requires transaction indexing on the Boundless node
    pub async fn get_transaction(&self, tx_hash: &str) -> Result<Option<TransactionInfo>> {
        // FIX H-3: Validate request hash format
        Self::validate_hash(tx_hash, "request tx_hash")?;

        let response = self
            .client
            .get(format!("{}/api/v1/transaction/{}", self.http_url, tx_hash))
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(EnterpriseError::BlockchainError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let tx_info = response
            .json::<TransactionInfo>()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("Failed to parse response: {}", e)))?;

        // FIX H-3: Validate response tx_hash matches request
        Self::validate_response_match(tx_hash, &tx_info.tx_hash, "transaction hash")?;

        Ok(Some(tx_info))
    }

    /// Get transaction history for an address
    ///
    /// NOTE: Requires transaction indexing on the Boundless node
    pub async fn get_transactions(
        &self,
        address: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<TransactionInfo>> {
        // FIX H-3: Validate request address format
        Self::validate_address(address, "request address")?;

        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let response = self
            .client
            .get(format!(
                "{}/api/v1/transactions/{}?limit={}&offset={}",
                self.http_url, address, limit, offset
            ))
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(EnterpriseError::BlockchainError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let tx_list: TransactionListResponse = response
            .json()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("Failed to parse response: {}", e)))?;

        // FIX H-3: Validate all transaction hashes in response
        for tx in &tx_list.transactions {
            Self::validate_hash(&tx.tx_hash, "transaction hash")?;
        }

        Ok(tx_list.transactions)
    }

    /// Get unspent transaction outputs (UTXOs) for an address
    ///
    /// NOTE: Requires UTXO indexing on the Boundless node
    pub async fn get_utxos(&self, address: &str) -> Result<Vec<UtxoInfo>> {
        // FIX H-3: Validate request address format
        Self::validate_address(address, "request address")?;

        let response = self
            .client
            .get(format!("{}/api/v1/utxos/{}", self.http_url, address))
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(EnterpriseError::BlockchainError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let utxo_response: UtxoListResponse = response
            .json()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("Failed to parse response: {}", e)))?;

        // FIX H-3: Validate all UTXO transaction hashes
        for utxo in &utxo_response.utxos {
            Self::validate_hash(&utxo.tx_hash, "UTXO tx_hash")?;
        }

        Ok(utxo_response.utxos)
    }

    /// Get block by height
    pub async fn get_block_by_height(&self, height: u64) -> Result<BlockInfo> {
        let response = self
            .client
            .get(format!("{}/api/v1/block/height/{}", self.http_url, height))
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(EnterpriseError::BlockchainError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let block_info = response
            .json::<BlockInfo>()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("Failed to parse response: {}", e)))?;

        // FIX H-3: Validate block hash and previous hash formats
        Self::validate_hash(&block_info.hash, "block hash")?;
        Self::validate_hash(&block_info.previous_hash, "previous block hash")?;

        Ok(block_info)
    }

    /// Get block by hash
    pub async fn get_block_by_hash(&self, block_hash: &str) -> Result<BlockInfo> {
        // FIX H-3: Validate request block hash format
        Self::validate_hash(block_hash, "request block hash")?;

        let response = self
            .client
            .get(format!("{}/api/v1/block/hash/{}", self.http_url, block_hash))
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(EnterpriseError::BlockchainError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let block_info = response
            .json::<BlockInfo>()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("Failed to parse response: {}", e)))?;

        // FIX H-3: Validate response block hash matches request
        Self::validate_response_match(block_hash, &block_info.hash, "block hash")?;
        Self::validate_hash(&block_info.previous_hash, "previous block hash")?;

        Ok(block_info)
    }

    /// Anchor a proof on-chain (for attestations, reports, etc.)
    pub async fn anchor_proof(
        &self,
        identity_id: &str,
        proof_type: &str,
        proof_hash: &str,
        metadata: serde_json::Value,
    ) -> Result<String> {
        let request = AnchorProofRequest {
            identity_id: identity_id.to_string(),
            proof_type: proof_type.to_string(),
            proof_hash: proof_hash.to_string(),
            metadata,
        };

        let response = self
            .client
            .post(format!("{}/api/v1/proof/anchor", self.http_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(EnterpriseError::ChainAnchorFailed(format!(
                "Anchor failed: {}",
                error_text
            )));
        }

        let proof_response: AnchorProofResponse = response
            .json()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("Failed to parse response: {}", e)))?;

        Ok(proof_response.proof_id)
    }

    /// Verify a proof exists on blockchain
    pub async fn verify_proof(&self, proof_hash: &str) -> Result<ProofVerification> {
        let request = VerifyProofRequest {
            proof_hash: proof_hash.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/api/v1/proof/verify", self.http_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(EnterpriseError::BlockchainError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        response
            .json::<ProofVerification>()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("Failed to parse response: {}", e)))
    }

    /// Get proof details by ID
    pub async fn get_proof(&self, proof_id: &str) -> Result<ProofInfo> {
        let response = self
            .client
            .get(format!("{}/api/v1/proof/{}", self.http_url, proof_id))
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(EnterpriseError::BlockchainError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        response
            .json::<ProofInfo>()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("Failed to parse response: {}", e)))
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/health", self.http_url))
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        Ok(response.status().is_success())
    }

    /// Query a deployed contract (read-only call)
    ///
    /// Sends ABI-encoded call data to a contract and returns the raw response bytes
    pub async fn query_contract(&self, contract_address: &str, call_data: &[u8]) -> Result<Vec<u8>> {
        // Validate contract address
        Self::validate_address(contract_address, "contract address")?;

        let request = ContractQueryRequest {
            contract_address: contract_address.to_string(),
            call_data: hex::encode(call_data),
        };

        let response = self
            .client
            .post(format!("{}/api/v1/contract/query", self.http_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(EnterpriseError::BlockchainError(format!(
                "Contract query failed: {}",
                error_text
            )));
        }

        let query_response: ContractQueryResponse = response
            .json()
            .await
            .map_err(|e| EnterpriseError::BlockchainError(format!("Failed to parse response: {}", e)))?;

        // Decode hex response to bytes
        hex::decode(&query_response.result)
            .map_err(|e| EnterpriseError::BlockchainError(format!("Invalid hex in response: {}", e)))
    }

    /// Send a transaction to a deployed contract (state-changing call)
    ///
    /// Builds, signs, and submits a contract interaction transaction
    pub async fn send_contract_transaction(
        &self,
        contract_address: &str,
        call_data: &[u8],
        deployer_key: &crate::transaction::deployment::DeployerKey,
    ) -> Result<String> {
        use crate::transaction::deployment::DeploymentBuilder;

        // Validate contract address
        Self::validate_address(contract_address, "contract address")?;

        // Use deployment builder to create contract transaction
        let deployment_builder = DeploymentBuilder::new(std::sync::Arc::new(self.clone()));

        // Build and send contract interaction transaction
        // NOTE: This is a simplified implementation that treats contract calls like deployments
        // A full implementation would have a dedicated contract transaction type
        deployment_builder
            .send_contract_call(contract_address, call_data, deployer_key)
            .await
    }
}

// Clone implementation for Arc usage
impl Clone for BlockchainClient {
    fn clone(&self) -> Self {
        Self {
            http_url: self.http_url.clone(),
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

// Request/Response types

#[derive(Serialize)]
struct SendTransactionRequest {
    transaction_hex: String,
}

#[derive(Deserialize)]
struct SendTransactionResponse {
    tx_hash: String,
}

#[derive(Deserialize)]
pub struct ChainInfo {
    pub chain_name: String,
    pub block_height: u64,
    pub difficulty: u64,
    pub total_transactions: u64,
}

#[derive(Deserialize)]
struct BlockHeightResponse {
    height: u64,
}

#[derive(Deserialize, Clone)]
pub struct BalanceInfo {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
}

#[derive(Deserialize, Clone)]
pub struct TransactionInfo {
    pub tx_hash: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub block_height: Option<u64>,
    pub timestamp: Option<i64>,
    pub status: String,
}

#[derive(Deserialize)]
struct TransactionListResponse {
    transactions: Vec<TransactionInfo>,
    total: u64,
}

/// FIX H-7: UTXO information from blockchain
#[derive(Deserialize, Clone, Debug)]
pub struct UtxoInfo {
    /// Transaction hash containing this UTXO
    pub tx_hash: String,

    /// Output index in the transaction
    pub output_index: u32,

    /// Amount in satoshis
    pub amount: u64,

    /// Block height where this UTXO was created
    pub block_height: u64,

    /// Optional locking script
    pub script: Option<String>,
}

/// FIX H-7: UTXO list response
#[derive(Deserialize)]
struct UtxoListResponse {
    utxos: Vec<UtxoInfo>,
}

#[derive(Deserialize, Clone)]
pub struct BlockInfo {
    pub height: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub nonce: u64,
    pub difficulty: u64,
    pub transactions_count: usize,
}

#[derive(Serialize)]
struct AnchorProofRequest {
    identity_id: String,
    proof_type: String,
    proof_hash: String,
    metadata: serde_json::Value,
}

#[derive(Deserialize)]
struct AnchorProofResponse {
    proof_id: String,
}

#[derive(Serialize)]
struct VerifyProofRequest {
    proof_hash: String,
}

#[derive(Deserialize, Clone)]
pub struct ProofVerification {
    pub verified: bool,
    pub proof_id: Option<String>,
    pub block_height: Option<u64>,
    pub timestamp: Option<u64>,
}

#[derive(Deserialize, Clone)]
pub struct ProofInfo {
    pub proof_id: String,
    pub identity_id: String,
    pub proof_type: String,
    pub proof_hash: String,
    pub block_height: u64,
    pub timestamp: u64,
    pub metadata: serde_json::Value,
}

#[derive(Serialize)]
struct ContractQueryRequest {
    contract_address: String,
    call_data: String,  // Hex-encoded
}

#[derive(Deserialize)]
struct ContractQueryResponse {
    result: String,  // Hex-encoded result bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = BlockchainClient::new("http://localhost:3001".to_string());
        assert_eq!(client.http_url, "http://localhost:3001");
    }

    #[test]
    fn test_client_from_env() {
        std::env::set_var("BOUNDLESS_HTTP_URL", "http://testnet:3001");
        let client = BlockchainClient::from_env();
        assert_eq!(client.http_url, "http://testnet:3001");
        std::env::remove_var("BOUNDLESS_HTTP_URL");
    }
}
