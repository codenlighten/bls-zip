// HTTP REST Bridge for Enterprise Integration
//
// Provides REST API endpoints for the Enterprise Multipass (E2) system
// Bridges between E2's REST expectations and Boundless JSON-RPC backend

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sha3::Digest;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::server::BlockchainRpc;

/// HTTP REST server state
#[derive(Clone)]
pub struct HttpBridgeState<B: BlockchainRpc + Clone> {
    pub blockchain: Arc<RwLock<B>>,
}

/// Start the HTTP REST bridge server
pub async fn start_http_bridge<B: BlockchainRpc + Clone + 'static>(
    addr: &str,
    blockchain: Arc<RwLock<B>>,
) -> anyhow::Result<()> {
    info!("🌐 Starting HTTP REST bridge on {}", addr);

    let state = HttpBridgeState { blockchain };

    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        .route("/api/v1/health", get(health_check))
        // Chain info
        .route("/api/v1/chain/info", get(get_chain_info))
        .route("/api/v1/chain/height", get(get_block_height))
        // Balance endpoints
        .route("/api/v1/balance/:address", get(get_balance))
        // UTXO endpoints
        .route("/api/v1/utxos/:address", get(get_utxos))
        // Transaction endpoints
        .route("/api/v1/transaction/send", post(send_transaction))
        .route("/api/v1/transaction/:tx_hash", get(get_transaction))
        .route(
            "/api/v1/transactions/:address",
            get(get_transaction_history),
        )
        // Block endpoints
        .route("/api/v1/block/height/:height", get(get_block_by_height))
        .route("/api/v1/block/hash/:hash", get(get_block_by_hash))
        // Proof anchoring endpoints
        .route("/api/v1/proof/anchor", post(anchor_proof))
        .route("/api/v1/proof/verify", post(verify_proof))
        .route("/api/v1/proof/:proof_id", get(get_proof))
        // Contract endpoints (Phase 4)
        .route("/api/v1/contract/query", post(query_contract))
        .route("/api/v1/contract/:address", get(get_contract_info))
        .route("/api/v1/contract/:address/state", get(get_contract_state))
        .with_state(state);

    let addr_parsed = addr.parse::<SocketAddr>()?;
    let listener = tokio::net::TcpListener::bind(addr_parsed).await?;

    info!("✅ HTTP REST bridge started on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// Health & Info Endpoints
// ============================================================================

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "boundless-http-bridge",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn get_chain_info<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
) -> Result<Json<ChainInfoResponse>, ApiError> {
    let chain = state.blockchain.read().await;

    Ok(Json(ChainInfoResponse {
        height: chain.height(),
        best_block_hash: hex::encode(chain.best_block_hash()),
        total_supply: chain.total_supply(),
        difficulty: chain.current_difficulty(),
    }))
}

async fn get_block_height<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
) -> Result<Json<BlockHeightResponse>, ApiError> {
    let chain = state.blockchain.read().await;

    Ok(Json(BlockHeightResponse {
        height: chain.height(),
    }))
}

// ============================================================================
// Balance Endpoints
// ============================================================================

async fn get_balance<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Path(address): Path<String>,
) -> Result<Json<BalanceResponse>, ApiError> {
    let address_bytes = hex::decode(&address)
        .map_err(|_| ApiError::InvalidAddress("Invalid address format".to_string()))?;

    if address_bytes.len() != 32 {
        return Err(ApiError::InvalidAddress(
            "Address must be 32 bytes".to_string(),
        ));
    }

    let mut address_array = [0u8; 32];
    address_array.copy_from_slice(&address_bytes);

    let chain = state.blockchain.read().await;

    Ok(Json(BalanceResponse {
        address: address.clone(),
        balance: chain.get_balance(&address_array),
        nonce: chain.get_nonce(&address_array),
    }))
}

// ============================================================================
// UTXO Endpoints
// ============================================================================

async fn get_utxos<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Path(address): Path<String>,
) -> Result<Json<UtxoListResponse>, ApiError> {
    let address_bytes = hex::decode(&address)
        .map_err(|_| ApiError::InvalidAddress("Invalid address format".to_string()))?;

    if address_bytes.len() != 32 {
        return Err(ApiError::InvalidAddress(
            "Address must be 32 bytes".to_string(),
        ));
    }

    let mut address_array = [0u8; 32];
    address_array.copy_from_slice(&address_bytes);

    let chain = state.blockchain.read().await;

    // Get UTXOs from blockchain
    let utxos = chain.get_utxos(&address_array);

    Ok(Json(UtxoListResponse { utxos }))
}

// ============================================================================
// Transaction Endpoints
// ============================================================================

#[derive(Debug, Deserialize)]
struct SendTransactionRequest {
    /// Hex-encoded serialized transaction
    transaction_hex: String,
}

async fn send_transaction<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Json(req): Json<SendTransactionRequest>,
) -> Result<Json<SendTransactionResponse>, ApiError> {
    let tx_bytes = hex::decode(&req.transaction_hex)
        .map_err(|_| ApiError::InvalidTransaction("Invalid transaction hex".to_string()))?;

    let tx: boundless_core::Transaction = bincode::deserialize(&tx_bytes).map_err(|e| {
        ApiError::Serialization(format!("Failed to deserialize transaction: {}", e))
    })?;

    let tx_hash = tx.hash();
    let chain = state.blockchain.read().await;

    match chain.submit_transaction(tx) {
        Ok(hash) => Ok(Json(SendTransactionResponse {
            tx_hash: hex::encode(hash),
            success: true,
            message: None,
        })),
        Err(e) => Ok(Json(SendTransactionResponse {
            tx_hash: hex::encode(tx_hash),
            success: false,
            message: Some(e),
        })),
    }
}

async fn get_transaction<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Path(tx_hash_str): Path<String>,
) -> Result<Json<TransactionResponse>, ApiError> {
    // Decode transaction hash
    let tx_hash_bytes = hex::decode(&tx_hash_str)
        .map_err(|_| ApiError::InvalidParams("Invalid transaction hash format".to_string()))?;

    if tx_hash_bytes.len() != 32 {
        return Err(ApiError::InvalidParams(
            "Transaction hash must be 32 bytes".to_string(),
        ));
    }

    let mut tx_hash = [0u8; 32];
    tx_hash.copy_from_slice(&tx_hash_bytes);

    let chain = state.blockchain.read().await;

    match chain.get_transaction(&tx_hash) {
        Some(record) => {
            // Extract from/to addresses from transaction
            let from = if !record.inputs.is_empty() {
                // Extract signature bytes from Signature enum
                let sig_bytes = match &record.inputs[0].signature {
                    boundless_core::Signature::Classical(bytes) => bytes,
                    boundless_core::Signature::MlDsa(bytes) => bytes,
                    boundless_core::Signature::Falcon(bytes) => bytes,
                    boundless_core::Signature::Hybrid { classical, .. } => classical,
                };
                hex::encode(sig_bytes.as_slice())
            } else {
                "coinbase".to_string()
            };

            let to = if !record.outputs.is_empty() {
                hex::encode(record.outputs[0].recipient_pubkey_hash)
            } else {
                "unknown".to_string()
            };

            let amount = record.outputs.iter().map(|o| o.amount).sum();

            Ok(Json(TransactionResponse {
                tx_hash: hex::encode(record.tx_hash),
                block_height: Some(record.block_height),
                block_hash: Some(hex::encode(record.block_hash)),
                timestamp: Some(record.timestamp),
                from,
                to,
                amount,
                fee: record.fee,
                status: record.status.as_str().to_string(),
            }))
        }
        None => Err(ApiError::BlockNotFound(format!(
            "Transaction {} not found",
            tx_hash_str
        ))),
    }
}

#[derive(Debug, Deserialize)]
struct TransactionHistoryQuery {
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

fn default_limit() -> u32 {
    50
}

async fn get_transaction_history<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Path(address_str): Path<String>,
    Query(params): Query<TransactionHistoryQuery>,
) -> Result<Json<TransactionHistoryResponse>, ApiError> {
    // Decode address
    let address_bytes = hex::decode(&address_str)
        .map_err(|_| ApiError::InvalidAddress("Invalid address format".to_string()))?;

    if address_bytes.len() != 32 {
        return Err(ApiError::InvalidAddress(
            "Address must be 32 bytes".to_string(),
        ));
    }

    let mut address = [0u8; 32];
    address.copy_from_slice(&address_bytes);

    let chain = state.blockchain.read().await;

    // HIGH PRIORITY FIX: Enforce pagination limit to prevent DoS
    use crate::types::enforce_pagination_limit;
    let safe_limit = enforce_pagination_limit(params.limit);

    // Get transaction history from index
    let records =
        chain.get_address_transactions(&address, safe_limit as usize, params.offset as usize);
    let total = chain.get_address_tx_count(&address);

    // Convert records to response format
    let transactions: Vec<TransactionResponse> = records
        .iter()
        .map(|record| {
            let from = if !record.inputs.is_empty() {
                // Extract signature bytes from Signature enum
                let sig_bytes = match &record.inputs[0].signature {
                    boundless_core::Signature::Classical(bytes) => bytes,
                    boundless_core::Signature::MlDsa(bytes) => bytes,
                    boundless_core::Signature::Falcon(bytes) => bytes,
                    boundless_core::Signature::Hybrid { classical, .. } => classical,
                };
                hex::encode(sig_bytes.as_slice())
            } else {
                "coinbase".to_string()
            };

            let to = if !record.outputs.is_empty() {
                hex::encode(record.outputs[0].recipient_pubkey_hash)
            } else {
                "unknown".to_string()
            };

            let amount = record.outputs.iter().map(|o| o.amount).sum();

            TransactionResponse {
                tx_hash: hex::encode(record.tx_hash),
                block_height: Some(record.block_height),
                block_hash: Some(hex::encode(record.block_hash)),
                timestamp: Some(record.timestamp),
                from,
                to,
                amount,
                fee: record.fee,
                status: record.status.as_str().to_string(),
            }
        })
        .collect();

    Ok(Json(TransactionHistoryResponse {
        address: address_str,
        transactions,
        total: total as u32,
        limit: params.limit,
        offset: params.offset,
    }))
}

// ============================================================================
// Block Endpoints
// ============================================================================

async fn get_block_by_height<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Path(height): Path<u64>,
) -> Result<Json<BlockResponse>, ApiError> {
    let chain = state.blockchain.read().await;

    match chain.get_block_by_height(height) {
        Some(block) => Ok(Json(BlockResponse::from_block(&block))),
        None => Err(ApiError::BlockNotFound(format!(
            "Block at height {} not found",
            height
        ))),
    }
}

async fn get_block_by_hash<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Path(hash): Path<String>,
) -> Result<Json<BlockResponse>, ApiError> {
    let hash_bytes = hex::decode(&hash)
        .map_err(|_| ApiError::InvalidParams("Invalid hash format".to_string()))?;

    if hash_bytes.len() != 32 {
        return Err(ApiError::InvalidParams("Hash must be 32 bytes".to_string()));
    }

    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash_bytes);

    let chain = state.blockchain.read().await;

    match chain.get_block_by_hash(&hash_array) {
        Some(block) => Ok(Json(BlockResponse::from_block(&block))),
        None => Err(ApiError::BlockNotFound(format!(
            "Block with hash {} not found",
            hash
        ))),
    }
}

// ============================================================================
// Proof Anchoring Endpoints
// ============================================================================

#[derive(Debug, Deserialize)]
struct AnchorProofRequest {
    /// Identity ID that owns this proof
    identity_id: String,
    /// Proof type (e.g., "kyc_verification", "credential")
    proof_type: String,
    /// Hash of the proof data (32 bytes hex)
    proof_hash: String,
    /// Optional metadata
    metadata: Option<serde_json::Value>,
    /// UTXO input - previous output hash (32 bytes hex) - REQUIRED
    previous_output_hash: String,
    /// UTXO input - output index - REQUIRED
    output_index: u32,
    /// UTXO input - signature (hex) - REQUIRED
    signature: String,
    /// UTXO input - public key (hex) - REQUIRED
    public_key: String,
    /// Optional nonce
    nonce: Option<u64>,
}

async fn anchor_proof<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Json(req): Json<AnchorProofRequest>,
) -> Result<Json<AnchorProofResponse>, ApiError> {
    // Validate identity_id
    let identity_bytes = hex::decode(&req.identity_id)
        .map_err(|_| ApiError::InvalidParams("Invalid identity_id format".to_string()))?;

    if identity_bytes.len() != 32 {
        return Err(ApiError::InvalidParams(
            "identity_id must be 32 bytes".to_string(),
        ));
    }

    let mut identity_id = [0u8; 32];
    identity_id.copy_from_slice(&identity_bytes);

    // Validate proof hash format
    let proof_hash_bytes = hex::decode(&req.proof_hash)
        .map_err(|_| ApiError::InvalidParams("Invalid proof hash format".to_string()))?;

    if proof_hash_bytes.len() != 32 {
        return Err(ApiError::InvalidParams(
            "Proof hash must be 32 bytes".to_string(),
        ));
    }

    let mut proof_hash = [0u8; 32];
    proof_hash.copy_from_slice(&proof_hash_bytes);

    // Parse proof type
    let proof_type = boundless_core::ProofType::from_str(&req.proof_type);

    // Encode metadata
    let metadata = if let Some(ref meta) = req.metadata {
        serde_json::to_vec(meta)
            .map_err(|_| ApiError::InvalidParams("Invalid metadata".to_string()))?
    } else {
        vec![]
    };

    // Create proof anchor data
    let proof_data =
        boundless_core::ProofAnchorData::new(identity_id, proof_type, proof_hash, metadata);

    // Validate proof data
    proof_data
        .validate()
        .map_err(|e| ApiError::InvalidParams(e))?;

    // Validate and parse UTXO input from client (REQUIRED)
    let previous_output_bytes = hex::decode(&req.previous_output_hash)
        .map_err(|_| ApiError::InvalidParams("Invalid previous_output_hash format".to_string()))?;

    if previous_output_bytes.len() != 32 {
        return Err(ApiError::InvalidParams(
            "previous_output_hash must be 32 bytes".to_string(),
        ));
    }

    let mut previous_output_hash = [0u8; 32];
    previous_output_hash.copy_from_slice(&previous_output_bytes);

    // Decode signature from client
    let signature_bytes = hex::decode(&req.signature)
        .map_err(|_| ApiError::InvalidParams("Invalid signature format".to_string()))?;

    if signature_bytes.is_empty() {
        return Err(ApiError::InvalidParams(
            "Signature cannot be empty".to_string(),
        ));
    }

    // Decode public key from client
    let public_key = hex::decode(&req.public_key)
        .map_err(|_| ApiError::InvalidParams("Invalid public_key format".to_string()))?;

    if public_key.is_empty() {
        return Err(ApiError::InvalidParams(
            "Public key cannot be empty".to_string(),
        ));
    }

    // Create proof anchor transaction with real UTXO input from client
    let tx_input = boundless_core::TxInput {
        previous_output_hash,
        output_index: req.output_index,
        signature: boundless_core::Signature::Classical(signature_bytes),
        public_key,
        nonce: req.nonce,
    };

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let tx = boundless_core::TransactionBuilder::create_proof_anchor(
        tx_input,
        proof_data.clone(),
        100, // Fee
        timestamp,
    )
    .map_err(|e| ApiError::InvalidParams(e))?;

    let tx_hash = tx.hash();
    let proof_id: [u8; 32] = sha3::Sha3_256::digest(&proof_hash).into();

    // Submit transaction to blockchain
    let chain = state.blockchain.read().await;
    match chain.submit_transaction(tx) {
        Ok(_) => {
            Ok(Json(AnchorProofResponse {
                proof_id: hex::encode(proof_id),
                tx_hash: hex::encode(tx_hash),
                block_height: None, // Not yet in a block
                anchored_at: timestamp,
            }))
        }
        Err(e) => Err(ApiError::InvalidParams(format!(
            "Failed to submit transaction: {}",
            e
        ))),
    }
}

#[derive(Debug, Deserialize)]
struct VerifyProofRequest {
    /// Proof hash to verify
    proof_hash: String,
}

async fn verify_proof<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Json(req): Json<VerifyProofRequest>,
) -> Result<Json<VerifyProofResponse>, ApiError> {
    // Validate proof hash format
    let proof_hash_bytes = hex::decode(&req.proof_hash)
        .map_err(|_| ApiError::InvalidParams("Invalid proof hash format".to_string()))?;

    if proof_hash_bytes.len() != 32 {
        return Err(ApiError::InvalidParams(
            "Proof hash must be 32 bytes".to_string(),
        ));
    }

    let mut proof_hash = [0u8; 32];
    proof_hash.copy_from_slice(&proof_hash_bytes);

    // Query blockchain state for proof
    let chain = state.blockchain.read().await;
    match chain.verify_proof_by_hash(&proof_hash) {
        Some(proof_anchor) => {
            // Proof exists and is valid
            Ok(Json(VerifyProofResponse {
                proof_hash: req.proof_hash,
                exists: true,
                anchored_at: Some(proof_anchor.timestamp),
                block_height: Some(proof_anchor.block_height),
                tx_hash: Some(hex::encode(proof_anchor.proof_id)), // Using proof_id as tx reference
            }))
        }
        None => {
            // Proof not found
            Ok(Json(VerifyProofResponse {
                proof_hash: req.proof_hash,
                exists: false,
                anchored_at: None,
                block_height: None,
                tx_hash: None,
            }))
        }
    }
}

async fn get_proof<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Path(proof_id): Path<String>,
) -> Result<Json<ProofResponse>, ApiError> {
    // Validate proof ID format
    let proof_id_bytes = hex::decode(&proof_id)
        .map_err(|_| ApiError::InvalidParams("Invalid proof ID format".to_string()))?;

    if proof_id_bytes.len() != 32 {
        return Err(ApiError::InvalidParams(
            "Proof ID must be 32 bytes".to_string(),
        ));
    }

    let mut proof_id_array = [0u8; 32];
    proof_id_array.copy_from_slice(&proof_id_bytes);

    // Query blockchain for proof
    let chain = state.blockchain.read().await;
    match chain.get_proof_by_id(&proof_id_array) {
        Some(proof_anchor) => {
            // Parse metadata if present
            let metadata = if !proof_anchor.metadata.is_empty() {
                serde_json::from_slice(&proof_anchor.metadata).ok()
            } else {
                None
            };

            Ok(Json(ProofResponse {
                proof_id: hex::encode(proof_anchor.proof_id),
                proof_type: proof_anchor.proof_type.as_str().to_string(),
                proof_hash: hex::encode(proof_anchor.proof_hash),
                identity_id: hex::encode(proof_anchor.identity_id),
                anchored_at: proof_anchor.timestamp,
                block_height: proof_anchor.block_height,
                tx_hash: hex::encode(proof_anchor.proof_id), // Using proof_id as tx reference
                metadata,
            }))
        }
        None => Err(ApiError::ProofNotFound(format!(
            "Proof not found: {}",
            proof_id
        ))),
    }
}

// ============================================================================
// Contract Endpoints (Phase 4)
// ============================================================================

#[derive(Debug, Deserialize)]
struct ContractQueryRequest {
    /// Contract address (hex-encoded 32 bytes)
    contract_address: String,
    /// Function name to call
    function_name: String,
    /// Function arguments (hex-encoded)
    args: String,
    /// Caller address (hex-encoded 32 bytes, optional)
    caller: Option<String>,
}

async fn query_contract<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Json(req): Json<ContractQueryRequest>,
) -> Result<Json<ContractQueryResponse>, ApiError> {
    // Validate and decode contract address
    let contract_address_bytes = hex::decode(&req.contract_address)
        .map_err(|_| ApiError::InvalidParams("Invalid contract address format".to_string()))?;

    if contract_address_bytes.len() != 32 {
        return Err(ApiError::InvalidParams(
            "Contract address must be 32 bytes".to_string(),
        ));
    }

    let mut contract_address = [0u8; 32];
    contract_address.copy_from_slice(&contract_address_bytes);

    // Decode function arguments
    let args_bytes = hex::decode(&req.args)
        .map_err(|_| ApiError::InvalidParams("Invalid args format".to_string()))?;

    // Decode caller address (use zero address if not provided)
    let caller = if let Some(caller_str) = req.caller {
        let caller_bytes = hex::decode(&caller_str)
            .map_err(|_| ApiError::InvalidParams("Invalid caller address format".to_string()))?;

        if caller_bytes.len() != 32 {
            return Err(ApiError::InvalidParams(
                "Caller address must be 32 bytes".to_string(),
            ));
        }

        let mut caller_array = [0u8; 32];
        caller_array.copy_from_slice(&caller_bytes);
        caller_array
    } else {
        [0u8; 32] // Default to zero address
    };

    // Execute contract query
    let chain = state.blockchain.read().await;

    match chain.query_contract(&contract_address, &req.function_name, &args_bytes, &caller) {
        Ok(result) => Ok(Json(ContractQueryResponse {
            result: hex::encode(result),
            success: true,
            error: None,
        })),
        Err(e) => Ok(Json(ContractQueryResponse {
            result: String::new(),
            success: false,
            error: Some(e),
        })),
    }
}

async fn get_contract_info<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Path(address_str): Path<String>,
) -> Result<Json<ContractInfoResponse>, ApiError> {
    // Validate and decode contract address
    let address_bytes = hex::decode(&address_str)
        .map_err(|_| ApiError::InvalidAddress("Invalid contract address format".to_string()))?;

    if address_bytes.len() != 32 {
        return Err(ApiError::InvalidAddress(
            "Contract address must be 32 bytes".to_string(),
        ));
    }

    let mut address = [0u8; 32];
    address.copy_from_slice(&address_bytes);

    // Query contract info from blockchain
    let chain = state.blockchain.read().await;

    match chain.get_contract(&address) {
        Some(contract) => Ok(Json(ContractInfoResponse {
            contract_address: hex::encode(contract.contract_address),
            wasm_bytecode_size: contract.wasm_bytecode.len(),
            deployer: hex::encode(contract.deployer),
            deployed_at_height: contract.deployed_at_height,
            deployed_at_tx: hex::encode(contract.deployed_at_tx),
        })),
        None => Err(ApiError::NotImplemented(format!(
            "Contract not found: {}",
            address_str
        ))),
    }
}

async fn get_contract_state<B: BlockchainRpc + Clone>(
    State(state): State<HttpBridgeState<B>>,
    Path(address_str): Path<String>,
) -> Result<Json<ContractStateResponse>, ApiError> {
    // Validate and decode contract address
    let address_bytes = hex::decode(&address_str)
        .map_err(|_| ApiError::InvalidAddress("Invalid contract address format".to_string()))?;

    if address_bytes.len() != 32 {
        return Err(ApiError::InvalidAddress(
            "Contract address must be 32 bytes".to_string(),
        ));
    }

    let mut address = [0u8; 32];
    address.copy_from_slice(&address_bytes);

    // Query contract state from blockchain
    let chain = state.blockchain.read().await;

    match chain.get_contract_state(&address) {
        Some(state) => {
            // Convert storage HashMap to hex-encoded key-value pairs
            let storage_entries: Vec<StorageEntry> = state
                .storage
                .iter()
                .map(|(key, value)| StorageEntry {
                    key: hex::encode(key),
                    value: hex::encode(value),
                })
                .collect();

            Ok(Json(ContractStateResponse {
                address: hex::encode(state.address),
                storage_quota: state.storage_quota,
                storage_used: state.storage_used,
                storage_entries,
                usage_percentage: state.usage_percentage(),
                last_modified: state.last_modified,
            }))
        }
        None => Err(ApiError::NotImplemented(format!(
            "Contract state not found: {}",
            address_str
        ))),
    }
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize)]
struct ChainInfoResponse {
    height: u64,
    best_block_hash: String,
    total_supply: u64,
    difficulty: u32,
}

#[derive(Debug, Serialize)]
struct BlockHeightResponse {
    height: u64,
}

#[derive(Debug, Serialize)]
struct BalanceResponse {
    address: String,
    balance: u64,
    nonce: u64,
}

#[derive(Debug, Serialize)]
struct UtxoListResponse {
    utxos: Vec<crate::types::UtxoData>,
}

#[derive(Debug, Serialize)]
struct SendTransactionResponse {
    tx_hash: String,
    success: bool,
    message: Option<String>,
}

#[derive(Debug, Serialize)]
struct TransactionResponse {
    tx_hash: String,
    block_height: Option<u64>,
    block_hash: Option<String>,
    timestamp: Option<u64>,
    from: String,
    to: String,
    amount: u64,
    fee: u64,
    status: String,
}

#[derive(Debug, Serialize)]
struct TransactionHistoryResponse {
    address: String,
    transactions: Vec<TransactionResponse>,
    total: u32,
    limit: u32,
    offset: u32,
}

#[derive(Debug, Serialize)]
struct BlockResponse {
    height: u64,
    hash: String,
    prev_hash: String,
    timestamp: u64,
    difficulty: u32,
    nonce: u64,
    transactions_count: usize,
}

impl BlockResponse {
    fn from_block(block: &boundless_core::Block) -> Self {
        Self {
            height: block.header.height,
            hash: hex::encode(block.hash()),
            prev_hash: hex::encode(block.header.previous_hash),
            timestamp: block.header.timestamp,
            difficulty: block.header.difficulty_target,
            nonce: block.header.nonce,
            transactions_count: block.transactions.len(),
        }
    }
}

#[derive(Debug, Serialize)]
struct AnchorProofResponse {
    proof_id: String,
    tx_hash: String,
    block_height: Option<u64>,
    anchored_at: u64,
}

#[derive(Debug, Serialize)]
struct VerifyProofResponse {
    proof_hash: String,
    exists: bool,
    anchored_at: Option<u64>,
    block_height: Option<u64>,
    tx_hash: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProofResponse {
    proof_id: String,
    proof_type: String,
    proof_hash: String,
    identity_id: String,
    anchored_at: u64,
    block_height: u64,
    tx_hash: String,
    metadata: Option<serde_json::Value>,
}

// Contract response types (Phase 4)

#[derive(Debug, Serialize)]
struct ContractQueryResponse {
    /// Hex-encoded result bytes
    result: String,
    /// Whether the query was successful
    success: bool,
    /// Error message if failed
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ContractInfoResponse {
    contract_address: String,
    wasm_bytecode_size: usize,
    deployer: String,
    deployed_at_height: u64,
    deployed_at_tx: String,
}

#[derive(Debug, Serialize)]
struct ContractStateResponse {
    address: String,
    storage_quota: u64,
    storage_used: u64,
    storage_entries: Vec<StorageEntry>,
    usage_percentage: f64,
    last_modified: u64,
}

#[derive(Debug, Serialize)]
struct StorageEntry {
    /// Hex-encoded key
    key: String,
    /// Hex-encoded value
    value: String,
}

// ============================================================================
// Error Handling
// ============================================================================

#[derive(Debug)]
enum ApiError {
    InvalidAddress(String),
    InvalidTransaction(String),
    InvalidParams(String),
    BlockNotFound(String),
    ProofNotFound(String),
    Serialization(String),
    NotImplemented(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::InvalidAddress(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InvalidTransaction(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InvalidParams(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::BlockNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::ProofNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Serialization(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::NotImplemented(msg) => (StatusCode::NOT_IMPLEMENTED, msg),
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limit() {
        assert_eq!(default_limit(), 50);
    }
}
