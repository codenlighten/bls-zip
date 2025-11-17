// HIGH PRIORITY FIX: Prometheus metrics for node observability
use anyhow::Result;
use hyper::{
    server::Server,
    service::{make_service_fn, service_fn},
    Body, Request, Response,
};
use prometheus::{
    Counter, Encoder, Gauge, IntCounter, IntGauge, Registry, TextEncoder,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Metrics collected by the node
pub struct NodeMetrics {
    /// Prometheus registry
    registry: Registry,

    // Blockchain metrics
    pub block_height: IntGauge,
    pub total_supply: Gauge,
    pub total_transactions: IntCounter,
    pub checkpoint_count: IntGauge,

    // Fork/reorg metrics
    pub fork_blocks_count: IntGauge,
    pub orphan_blocks_count: IntGauge,
    pub reorganizations_total: IntCounter,

    // Network metrics
    pub peer_count: IntGauge,
    pub inbound_messages_total: IntCounter,
    pub outbound_messages_total: IntCounter,

    // Mining metrics
    pub blocks_mined_total: IntCounter,
    pub mining_hash_rate: Gauge,

    // Mempool metrics
    pub mempool_size: IntGauge,
    pub mempool_bytes: IntGauge,

    // RPC metrics
    pub rpc_requests_total: IntCounter,
    pub rpc_errors_total: IntCounter,
}

impl NodeMetrics {
    /// Create a new metrics instance
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        // Blockchain metrics
        let block_height = IntGauge::new("blockchain_height", "Current blockchain height")?;
        let total_supply = Gauge::new("blockchain_total_supply", "Total BLS supply in circulation")?;
        let total_transactions = IntCounter::new("blockchain_transactions_total", "Total number of transactions processed")?;
        let checkpoint_count = IntGauge::new("blockchain_checkpoints_count", "Number of active checkpoints")?;

        // Fork/reorg metrics
        let fork_blocks_count = IntGauge::new("blockchain_fork_blocks_count", "Number of blocks on forks (not main chain)")?;
        let orphan_blocks_count = IntGauge::new("blockchain_orphan_blocks_count", "Number of orphan blocks (parent unknown)")?;
        let reorganizations_total = IntCounter::new("blockchain_reorganizations_total", "Total number of chain reorganizations")?;

        // Network metrics
        let peer_count = IntGauge::new("p2p_peer_count", "Number of connected peers")?;
        let inbound_messages_total = IntCounter::new("p2p_inbound_messages_total", "Total inbound P2P messages")?;
        let outbound_messages_total = IntCounter::new("p2p_outbound_messages_total", "Total outbound P2P messages")?;

        // Mining metrics
        let blocks_mined_total = IntCounter::new("mining_blocks_mined_total", "Total blocks mined by this node")?;
        let mining_hash_rate = Gauge::new("mining_hash_rate", "Current mining hash rate (H/s)")?;

        // Mempool metrics
        let mempool_size = IntGauge::new("mempool_transactions_count", "Number of transactions in mempool")?;
        let mempool_bytes = IntGauge::new("mempool_bytes", "Total size of mempool in bytes")?;

        // RPC metrics
        let rpc_requests_total = IntCounter::new("rpc_requests_total", "Total RPC requests received")?;
        let rpc_errors_total = IntCounter::new("rpc_errors_total", "Total RPC errors")?;

        // Register all metrics
        registry.register(Box::new(block_height.clone()))?;
        registry.register(Box::new(total_supply.clone()))?;
        registry.register(Box::new(total_transactions.clone()))?;
        registry.register(Box::new(checkpoint_count.clone()))?;

        registry.register(Box::new(fork_blocks_count.clone()))?;
        registry.register(Box::new(orphan_blocks_count.clone()))?;
        registry.register(Box::new(reorganizations_total.clone()))?;

        registry.register(Box::new(peer_count.clone()))?;
        registry.register(Box::new(inbound_messages_total.clone()))?;
        registry.register(Box::new(outbound_messages_total.clone()))?;

        registry.register(Box::new(blocks_mined_total.clone()))?;
        registry.register(Box::new(mining_hash_rate.clone()))?;

        registry.register(Box::new(mempool_size.clone()))?;
        registry.register(Box::new(mempool_bytes.clone()))?;

        registry.register(Box::new(rpc_requests_total.clone()))?;
        registry.register(Box::new(rpc_errors_total.clone()))?;

        Ok(Self {
            registry,
            block_height,
            total_supply,
            total_transactions,
            checkpoint_count,
            fork_blocks_count,
            orphan_blocks_count,
            reorganizations_total,
            peer_count,
            inbound_messages_total,
            outbound_messages_total,
            blocks_mined_total,
            mining_hash_rate,
            mempool_size,
            mempool_bytes,
            rpc_requests_total,
            rpc_errors_total,
        })
    }

    /// Gather and encode metrics in Prometheus format
    pub fn gather(&self) -> Result<Vec<u8>> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();

        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;

        Ok(buffer)
    }
}

/// Start Prometheus metrics HTTP server
pub async fn start_metrics_server(
    addr: SocketAddr,
    metrics: Arc<RwLock<NodeMetrics>>,
) -> Result<()> {
    info!("📊 Starting metrics server on {}", addr);

    let make_svc = make_service_fn(move |_conn| {
        let metrics = metrics.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                let metrics = metrics.clone();
                async move {
                    match req.uri().path() {
                        "/metrics" => {
                            let metrics_guard = metrics.read().await;
                            match metrics_guard.gather() {
                                Ok(buffer) => {
                                    Response::builder()
                                        .header("Content-Type", "text/plain; version=0.0.4")
                                        .body(Body::from(buffer))
                                }
                                Err(e) => {
                                    Response::builder()
                                        .status(500)
                                        .body(Body::from(format!("Error gathering metrics: {}", e)))
                                }
                            }
                        }
                        "/health" => {
                            Response::builder()
                                .body(Body::from("OK"))
                        }
                        _ => {
                            Response::builder()
                                .status(404)
                                .body(Body::from("Not Found\n\nAvailable endpoints:\n  /metrics - Prometheus metrics\n  /health - Health check"))
                        }
                    }
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    server.await?;

    Ok(())
}
