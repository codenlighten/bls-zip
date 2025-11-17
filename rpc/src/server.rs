// JSON-RPC server implementation
use jsonrpsee::{
    server::{Server, ServerHandle},
    RpcModule,
};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::RwLock;
use tower::{Layer, Service, ServiceBuilder};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tracing::{info, warn};

use bincode;

use crate::{error::RpcError, types::*};

/// API Key Authentication Middleware
#[derive(Clone)]
struct ApiKeyLayer {
    api_keys: Arc<Vec<String>>,
}

impl ApiKeyLayer {
    fn new(api_keys: Vec<String>) -> Self {
        Self {
            api_keys: Arc::new(api_keys),
        }
    }
}

impl<S> Layer<S> for ApiKeyLayer {
    type Service = ApiKeyService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ApiKeyService {
            inner,
            api_keys: self.api_keys.clone(),
        }
    }
}

#[derive(Clone)]
struct ApiKeyService<S> {
    inner: S,
    api_keys: Arc<Vec<String>>,
}

impl<S, Body> Service<http::Request<Body>> for ApiKeyService<S>
where
    S: Service<http::Request<Body>, Response = http::Response<tower::BoxError>>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<Body>) -> Self::Future {
        // If no API keys configured, allow all requests
        if self.api_keys.is_empty() {
            let fut = self.inner.call(req);
            return Box::pin(fut);
        }

        // Check for API key in Authorization header
        let authorized = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|auth| {
                // Support both "Bearer <key>" and direct key
                if let Some(key) = auth.strip_prefix("Bearer ") {
                    Some(key)
                } else {
                    Some(auth)
                }
            })
            .map(|key| self.api_keys.iter().any(|valid_key| valid_key == key))
            .unwrap_or(false);

        if !authorized {
            // Return 401 Unauthorized
            let response = http::Response::builder()
                .status(http::StatusCode::UNAUTHORIZED)
                .body(tower::BoxError::from("Unauthorized: Invalid or missing API key"))
                .unwrap();

            return Box::pin(async move { Ok(response) });
        }

        let fut = self.inner.call(req);
        Box::pin(fut)
    }
}

/// Blockchain handle for RPC server
pub trait BlockchainRpc: Send + Sync {
    /// Get blockchain height
    fn height(&self) -> u64;

    /// Get best block hash
    fn best_block_hash(&self) -> [u8; 32];

    /// Get total supply
    fn total_supply(&self) -> u64;

    /// Get account balance
    fn get_balance(&self, address: &[u8; 32]) -> u64;

    /// Get account nonce
    fn get_nonce(&self, address: &[u8; 32]) -> u64;

    /// Get block by height
    fn get_block_by_height(&self, height: u64) -> Option<boundless_core::Block>;

    /// Get block by hash
    fn get_block_by_hash(&self, hash: &[u8; 32]) -> Option<boundless_core::Block>;

    /// Submit transaction to mempool
    fn submit_transaction(&self, tx: boundless_core::Transaction) -> Result<[u8; 32], String>;

    /// Get current difficulty target
    fn current_difficulty(&self) -> u32;

    /// Get transaction by hash
    fn get_transaction(&self, tx_hash: &[u8; 32]) -> Option<boundless_core::TransactionRecord>;

    /// Get transaction history for address
    fn get_address_transactions(
        &self,
        address: &[u8; 32],
        limit: usize,
        offset: usize,
    ) -> Vec<boundless_core::TransactionRecord>;

    /// Get transaction count for address
    fn get_address_tx_count(&self, address: &[u8; 32]) -> usize;

    /// Get proof by ID
    fn get_proof_by_id(&self, proof_id: &[u8; 32]) -> Option<boundless_core::ProofAnchor>;

    /// Verify proof by hash
    fn verify_proof_by_hash(&self, proof_hash: &[u8; 32]) -> Option<boundless_core::ProofAnchor>;

    /// Get UTXOs (unspent transaction outputs) for an address
    fn get_utxos(&self, address: &[u8; 32]) -> Vec<crate::types::UtxoData>;
}

/// RPC server
pub struct RpcServer;

pub type RpcServerHandle = ServerHandle;

impl RpcServer {
    /// Start the RPC server with security middleware
    pub async fn start<B: BlockchainRpc + 'static>(
        addr: &str,
        blockchain: Arc<RwLock<B>>,
    ) -> anyhow::Result<RpcServerHandle> {
        info!("🌐 Starting RPC server on {}", addr);

        // SECURITY FIX: Configure CORS
        let cors = Self::configure_cors();

        // SECURITY FIX: Configure rate limiting (100 requests per minute per IP)
        let rate_limit = Self::configure_rate_limiting();

        // SECURITY FIX: Configure API key authentication
        let api_auth = Self::configure_api_keys();

        // Build server with security middleware
        let server = Server::builder()
            .set_http_middleware(
                ServiceBuilder::new()
                    .layer(api_auth)  // Auth first (reject unauthorized early)
                    .layer(cors)      // Then CORS
                    .layer(rate_limit) // Then rate limiting
            )
            .build(addr.parse::<SocketAddr>()?)
            .await?;

        let mut module = RpcModule::new(blockchain);

        // Register RPC methods
        Self::register_methods(&mut module)?;

        let handle = server.start(module);

        info!("✅ RPC server started on {} with security middleware", addr);

        Ok(handle)
    }

    /// Configure API key authentication
    ///
    /// SECURITY: Validates API keys from RPC_API_KEYS environment variable
    /// Format: comma-separated list of keys
    /// If not set, authentication is DISABLED (for development only)
    fn configure_api_keys() -> ApiKeyLayer {
        let api_keys_str = std::env::var("RPC_API_KEYS")
            .unwrap_or_else(|_| String::new());

        if api_keys_str.is_empty() {
            warn!(
                "RPC_API_KEYS not set - API KEY AUTHENTICATION DISABLED! \
                This is INSECURE for production. Set RPC_API_KEYS environment variable."
            );
            return ApiKeyLayer::new(vec![]);
        }

        let api_keys: Vec<String> = api_keys_str
            .split(',')
            .map(|key| key.trim().to_string())
            .filter(|key| !key.is_empty())
            .collect();

        if api_keys.is_empty() {
            warn!("RPC_API_KEYS is empty - authentication disabled");
        } else {
            info!("RPC API Key Authentication: {} keys configured", api_keys.len());
        }

        ApiKeyLayer::new(api_keys)
    }

    /// Configure CORS with environment-based origin whitelist
    ///
    /// SECURITY: Only allows specific origins from RPC_CORS_ORIGINS environment variable
    /// Format: comma-separated list of origins (e.g., "http://localhost:3000,https://app.example.com")
    /// Default: No origins allowed (secure by default)
    fn configure_cors() -> CorsLayer {
        let allowed_origins_str = std::env::var("RPC_CORS_ORIGINS")
            .unwrap_or_else(|_| String::new());

        if allowed_origins_str.is_empty() {
            warn!(
                "RPC_CORS_ORIGINS not set - CORS will reject all origins. \
                Set RPC_CORS_ORIGINS environment variable for production use."
            );
        }

        // Parse origins from comma-separated string
        let allowed_origins: Vec<_> = allowed_origins_str
            .split(',')
            .filter_map(|origin| {
                let trimmed = origin.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    match trimmed.parse() {
                        Ok(header) => {
                            info!("RPC CORS: Allowing origin: {}", trimmed);
                            Some(header)
                        }
                        Err(e) => {
                            warn!("RPC CORS: Invalid origin '{}': {}", trimmed, e);
                            None
                        }
                    }
                }
            })
            .collect();

        CorsLayer::new()
            .allow_origin(AllowOrigin::list(allowed_origins))
            .allow_methods([
                tower_http::cors::Method::GET,
                tower_http::cors::Method::POST,
                tower_http::cors::Method::OPTIONS,
            ])
            .allow_headers([
                tower_http::cors::Header::from_static("content-type"),
                tower_http::cors::Header::from_static("authorization"),
            ])
            .allow_credentials(false)
    }

    /// Configure rate limiting
    ///
    /// SECURITY: Limits requests per IP address to prevent DoS attacks
    /// Default: 100 requests per minute per IP
    /// Configure via RPC_RATE_LIMIT environment variable
    fn configure_rate_limiting() -> GovernorLayer<SmartIpKeyExtractor> {
        let rate_limit: u32 = std::env::var("RPC_RATE_LIMIT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        info!("RPC Rate Limit: {} requests per minute per IP", rate_limit);

        let governor_conf = Box::new(
            GovernorConfigBuilder::default()
                .per_second(rate_limit as u64 / 60) // Convert per-minute to per-second
                .burst_size(rate_limit)
                .finish()
                .unwrap(),
        );

        GovernorLayer {
            config: Box::leak(governor_conf),
        }
    }

    /// Register all RPC methods
    fn register_methods<B: BlockchainRpc + 'static>(
        module: &mut RpcModule<Arc<RwLock<B>>>,
    ) -> anyhow::Result<()> {
        // chain_getBlockHeight
        module.register_async_method("chain_getBlockHeight", |_, blockchain| async move {
            let chain = blockchain.read().await;
            Ok::<u64, RpcError>(chain.height())
        })?;

        // chain_getBestBlockHash
        module.register_async_method("chain_getBestBlockHash", |_, blockchain| async move {
            let chain = blockchain.read().await;
            let hash = chain.best_block_hash();
            Ok::<String, RpcError>(hex::encode(hash))
        })?;

        // chain_getInfo
        module.register_async_method("chain_getInfo", |_, blockchain| async move {
            let chain = blockchain.read().await;
            Ok::<ChainInfo, RpcError>(ChainInfo {
                height: chain.height(),
                best_block_hash: hex::encode(chain.best_block_hash()),
                total_supply: chain.total_supply(),
                difficulty: chain.current_difficulty(), // Fixed: Get actual current difficulty
            })
        })?;

        // chain_getBlockByHeight
        module.register_async_method(
            "chain_getBlockByHeight",
            |params, blockchain| async move {
                let height: u64 = params.one()?;
                let chain = blockchain.read().await;

                match chain.get_block_by_height(height) {
                    Some(block) => Ok::<BlockInfo, RpcError>(BlockInfo::from_block(&block)),
                    None => Err(RpcError::BlockNotFound(format!("height {}", height))),
                }
            },
        )?;

        // chain_getBlockByHash
        module.register_async_method("chain_getBlockByHash", |params, blockchain| async move {
            let hash_str: String = params.one()?;
            let hash_bytes = hex::decode(&hash_str)
                .map_err(|_| RpcError::InvalidParams("Invalid hash format".to_string()))?;

            if hash_bytes.len() != 32 {
                return Err(RpcError::InvalidParams("Hash must be 32 bytes".to_string()));
            }

            let mut hash = [0u8; 32];
            hash.copy_from_slice(&hash_bytes);

            let chain = blockchain.read().await;

            match chain.get_block_by_hash(&hash) {
                Some(block) => Ok::<BlockInfo, RpcError>(BlockInfo::from_block(&block)),
                None => Err(RpcError::BlockNotFound(hash_str)),
            }
        })?;

        // chain_getBalance
        module.register_async_method("chain_getBalance", |params, blockchain| async move {
            let address_str: String = params.one()?;
            let address_bytes = hex::decode(&address_str)
                .map_err(|_| RpcError::InvalidAddress("Invalid address format".to_string()))?;

            if address_bytes.len() != 32 {
                return Err(RpcError::InvalidAddress(
                    "Address must be 32 bytes".to_string(),
                ));
            }

            let mut address = [0u8; 32];
            address.copy_from_slice(&address_bytes);

            let chain = blockchain.read().await;

            Ok::<BalanceInfo, RpcError>(BalanceInfo {
                address: address_str,
                balance: chain.get_balance(&address),
                nonce: chain.get_nonce(&address),
            })
        })?;

        // chain_submitTransaction
        module.register_async_method(
            "chain_submitTransaction",
            |params, blockchain| async move {
                let tx_hex: String = params.one()?;
                let tx_bytes = hex::decode(&tx_hex)
                    .map_err(|_| RpcError::InvalidParams("Invalid transaction hex".to_string()))?;

                let tx: boundless_core::Transaction = bincode::deserialize(&tx_bytes)
                    .map_err(|e| RpcError::Serialization(e.to_string()))?;

                let chain = blockchain.read().await;

                match chain.submit_transaction(tx) {
                    Ok(hash) => Ok::<SubmitTxResponse, RpcError>(SubmitTxResponse {
                        tx_hash: hex::encode(hash),
                    }),
                    Err(e) => Err(RpcError::InvalidTransaction(e)),
                }
            },
        )?;

        // system_health
        module.register_async_method("system_health", |_, _blockchain| async move {
            Ok::<serde_json::Value, RpcError>(serde_json::json!({
                "is_syncing": false,
                "should_have_peers": true,
                "peers": 0,
            }))
        })?;

        // system_version
        module.register_method("system_version", |_, _| {
            Ok::<String, RpcError>(env!("CARGO_PKG_VERSION").to_string())
        })?;

        info!(
            "📝 Registered {} RPC methods",
            module.method_names().count()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBlockchain {
        height: u64,
        total_supply: u64,
    }

    impl BlockchainRpc for MockBlockchain {
        fn height(&self) -> u64 {
            self.height
        }

        fn best_block_hash(&self) -> [u8; 32] {
            [0u8; 32]
        }

        fn total_supply(&self) -> u64 {
            self.total_supply
        }

        fn get_balance(&self, _address: &[u8; 32]) -> u64 {
            1000
        }

        fn get_nonce(&self, _address: &[u8; 32]) -> u64 {
            0
        }

        fn get_block_by_height(&self, _height: u64) -> Option<boundless_core::Block> {
            None
        }

        fn get_block_by_hash(&self, _hash: &[u8; 32]) -> Option<boundless_core::Block> {
            None
        }

        fn submit_transaction(&self, tx: boundless_core::Transaction) -> Result<[u8; 32], String> {
            Ok(tx.hash())
        }

        fn current_difficulty(&self) -> u32 {
            0x1f0fffff // Genesis difficulty for testing
        }

        fn get_transaction(
            &self,
            _tx_hash: &[u8; 32],
        ) -> Option<boundless_core::TransactionRecord> {
            None
        }

        fn get_address_transactions(
            &self,
            _address: &[u8; 32],
            _limit: usize,
            _offset: usize,
        ) -> Vec<boundless_core::TransactionRecord> {
            vec![]
        }

        fn get_address_tx_count(&self, _address: &[u8; 32]) -> usize {
            0
        }

        fn get_proof_by_id(&self, _proof_id: &[u8; 32]) -> Option<boundless_core::ProofAnchor> {
            None
        }

        fn verify_proof_by_hash(
            &self,
            _proof_hash: &[u8; 32],
        ) -> Option<boundless_core::ProofAnchor> {
            None
        }

        fn get_utxos(&self, _address: &[u8; 32]) -> Vec<crate::types::UtxoData> {
            vec![]
        }
    }

    #[tokio::test]
    async fn test_rpc_server_creation() {
        let blockchain = Arc::new(RwLock::new(MockBlockchain {
            height: 100,
            total_supply: 5_000_000_000,
        }));

        // We can't actually bind to the port in tests, but we can create the module
        let mut module = RpcModule::new(blockchain);
        assert!(RpcServer::register_methods(&mut module).is_ok());
        assert!(module.method_names().count() > 0);
    }
}
