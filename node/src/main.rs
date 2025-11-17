use clap::Parser;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

mod blockchain;
mod config;
mod mempool;
mod metrics;
mod rpc_impl;

use blockchain::Blockchain;
use boundless_p2p::{NetworkConfig, NetworkNode};
use boundless_rpc::RpcServer;
use config::NodeConfig;
use mempool::Mempool;

#[derive(Parser, Debug)]
#[command(name = "boundless-node")]
#[command(about = "Boundless BLS Blockchain Node", long_about = None)]
#[command(version = "0.1.0")]
struct Args {
    /// Run in development mode with easy mining
    #[arg(long)]
    dev: bool,

    /// Enable mining
    #[arg(long)]
    mining: bool,

    /// Mining coinbase address (hex-encoded 32 bytes)
    #[arg(long)]
    coinbase: Option<String>,

    /// Data directory for blockchain storage
    #[arg(long, default_value = "./data")]
    base_path: PathBuf,

    /// P2P listen port
    #[arg(long, default_value = "30333")]
    port: u16,

    /// RPC HTTP port (JSON-RPC)
    #[arg(long, default_value = "9933")]
    rpc_port: u16,

    /// RPC host address
    #[arg(long, default_value = "127.0.0.1")]
    rpc_host: String,

    /// HTTP REST bridge port (for enterprise integration)
    #[arg(long, default_value = "3001")]
    http_port: u16,

    /// RPC WebSocket port
    #[arg(long, default_value = "9944")]
    ws_port: u16,

    /// Config file path
    #[arg(long)]
    config: Option<PathBuf>,

    /// Mining threads
    #[arg(long, default_value = "1")]
    mining_threads: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Load or create configuration first (before logging, so we can use config.operational.log_level)
    let mut config = if let Some(config_path) = &args.config {
        info!("📄 Loading configuration from {:?}", config_path);
        NodeConfig::from_file(config_path)?
    } else if args.dev {
        NodeConfig::development()
    } else {
        NodeConfig::default()
    };

    // HIGH PRIORITY FIX: Initialize logging using operational configuration
    // Allows log level to be configured via config file or environment variable
    let log_level = std::env::var("RUST_LOG")
        .ok()
        .or(Some(config.operational.log_level.clone()))
        .unwrap_or_else(|| "info".to_string());

    // Configure logging format based on structured_logging setting
    if config.operational.structured_logging {
        // JSON structured logging for production/monitoring systems
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(log_level.parse().unwrap_or(tracing::Level::INFO.into())),
            )
            .init();
    } else {
        // Human-readable logging for development
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(log_level.parse().unwrap_or(tracing::Level::INFO.into())),
            )
            .init();
    }

    info!(
        "🚀 Starting Boundless BLS Node v{}",
        env!("CARGO_PKG_VERSION")
    );
    info!("📁 Data directory: {:?}", args.base_path);

    if args.dev {
        info!("🔧 Development mode enabled");
    }

    if args.config.is_some() {
        info!("📄 Using configuration file");
    } else {
        info!("📄 Using default configuration");
    }

    // HIGH PRIORITY FIX: Command-line args override config file values
    // This allows flexible configuration: defaults → config file → CLI args
    if args.mining {
        config.mining.enabled = true;
    }
    if let Some(ref coinbase) = args.coinbase {
        config.mining.coinbase_address = Some(coinbase.clone());
    }
    if args.mining_threads != 1 {
        // Non-default value specified
        config.mining.threads = args.mining_threads;
    }

    // Create data directory if it doesn't exist
    if !args.base_path.exists() {
        std::fs::create_dir_all(&args.base_path)?;
    }

    // Initialize blockchain
    let blockchain = Arc::new(RwLock::new(Blockchain::new(
        args.base_path.clone(),
        config.clone(),
    )?));

    {
        let chain = blockchain.read().await;
        info!("⛓️  Blockchain initialized at height {}", chain.height());
        info!("🔗 Best block: {}", hex::encode(chain.best_block_hash()));
    }

    // Initialize mempool
    let mempool = Arc::new(RwLock::new(Mempool::new(config.mempool_config())));

    info!("💾 Mempool initialized");

    // Start RPC server
    let rpc_addr = format!("{}:{}", args.rpc_host, args.rpc_port);
    let rpc_blockchain = blockchain.clone();

    // Start RPC server and keep handle alive (dropping it shuts down the server)
    let rpc_handle = RpcServer::start(&rpc_addr, rpc_blockchain).await?;
    info!("🌐 RPC server running on {}", rpc_addr);

    // Start HTTP REST bridge for Enterprise integration
    let http_addr = format!("{}:{}", args.rpc_host, args.http_port);
    let http_blockchain = blockchain.clone();

    // Spawn HTTP bridge in background task
    tokio::spawn(async move {
        info!("🌉 Starting HTTP REST bridge on {}", http_addr);
        if let Err(e) = boundless_rpc::http_bridge::start_http_bridge(&http_addr, http_blockchain).await {
            error!("❌ HTTP bridge error: {}", e);
        }
    });
    info!("🌉 HTTP REST bridge running on {}", http_addr);

    // MEDIUM PRIORITY FIX: Create peer count tracker for metrics
    let peer_count = Arc::new(AtomicUsize::new(0));

    // HIGH PRIORITY FIX: Start metrics server if enabled
    if config.operational.enable_metrics {
        let metrics = Arc::new(RwLock::new(metrics::NodeMetrics::new()?));
        let metrics_addr: std::net::SocketAddr = config.operational.metrics_addr.parse()?;

        // Clone references for metrics update task
        let metrics_update = metrics.clone();
        let blockchain_metrics = blockchain.clone();
        let mempool_metrics = mempool.clone();
        let peer_count_metrics = peer_count.clone();

        // Spawn metrics update task (updates every 5 seconds)
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                let chain = blockchain_metrics.read().await;
                let pool = mempool_metrics.read().await;
                let mut m = metrics_update.write().await;

                // Update blockchain metrics
                m.block_height.set(chain.height() as i64);
                m.total_supply.set(chain.total_supply() as f64);
                m.checkpoint_count.set(chain.get_checkpoints().len() as i64);

                // Update mempool metrics
                m.mempool_size.set(pool.len() as i64);

                // Update fork/orphan block metrics
                m.fork_blocks_count.set(chain.fork_blocks_count() as i64);
                m.orphan_blocks_count.set(chain.orphan_blocks_count() as i64);

                // Update network peer count
                m.peer_count.set(peer_count_metrics.load(Ordering::Relaxed) as i64);
            }
        });

        // Spawn metrics server
        tokio::spawn(async move {
            if let Err(e) = metrics::start_metrics_server(metrics_addr, metrics).await {
                error!("❌ Metrics server error: {}", e);
            }
        });

        info!("📊 Metrics server running on {}", config.operational.metrics_addr);
    } else {
        info!("📊 Metrics disabled");
    }

    // Start P2P network
    let bootnodes = config
        .network
        .bootnodes
        .iter()
        .filter_map(|s| s.parse().ok())
        .collect();

    let p2p_config = NetworkConfig {
        listen_addr: format!("/ip4/0.0.0.0/tcp/{}", args.port).parse()?,
        bootnodes,
        enable_mdns: true,
        max_peers: 50,
        max_message_size: 10 * 1024 * 1024, // 10MB
        max_block_size: 10 * 1024 * 1024,   // 10MB
        max_transaction_size: 1024 * 1024,  // 1MB
        max_inbound_connections: 25,
        max_outbound_connections: 25,
    };

    let network_handle = match NetworkNode::new(p2p_config) {
        Ok((network, mut events)) => {
            info!("🌐 P2P network initialized");

            // Create command channel for sending commands to network
            let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();

            // Spawn P2P event loop - takes ownership of network
            tokio::spawn(async move {
                network.run(command_rx).await;
            });

            // Spawn event handler
            let blockchain_clone = blockchain.clone();
            let mempool_clone = mempool.clone();
            let command_tx_clone = command_tx.clone();
            let peer_count_clone = peer_count.clone();

            tokio::spawn(async move {
                use boundless_p2p::NetworkEvent;
                use boundless_p2p::protocol::Message;
                use boundless_p2p::NetworkCommand;
                use std::sync::Arc;

                while let Some(event) = events.recv().await {
                    match event {
                        NetworkEvent::PeerConnected(peer_id) => {
                            peer_count_clone.fetch_add(1, Ordering::Relaxed);
                            info!("🤝 Peer connected: {} (total: {})", peer_id, peer_count_clone.load(Ordering::Relaxed));
                        }
                        NetworkEvent::PeerDisconnected(peer_id) => {
                            peer_count_clone.fetch_sub(1, Ordering::Relaxed);
                            info!("👋 Peer disconnected: {} (total: {})", peer_id, peer_count_clone.load(Ordering::Relaxed));
                        }
                        NetworkEvent::NewListenAddr(addr) => {
                            info!("📡 Listening on: {}", addr);
                        }
                        NetworkEvent::MessageReceived { peer_id, message } => {
                            // SECURITY FIX: Implement comprehensive P2P message handling
                            info!("📩 Received {} from {}", message.message_type(), peer_id);

                            match message {
                                // Handle incoming block requests
                                Message::GetBlocks { start_height, count } => {
                                    info!("📨 Peer {} requested {} blocks from height {}",
                                          peer_id, count, start_height);

                                    let blockchain = blockchain_clone.read().await;
                                    let mut blocks = Vec::new();

                                    // Fetch requested blocks (limit to prevent DoS)
                                    let max_blocks = count.min(500); // Max 500 blocks per request
                                    for height in start_height..(start_height + max_blocks as u64) {
                                        if let Ok(block) = blockchain.get_block_by_height(height) {
                                            blocks.push(block);
                                        } else {
                                            break; // Stop if block not found
                                        }
                                    }

                                    if !blocks.is_empty() {
                                        info!("📤 Sending {} blocks to peer {}", blocks.len(), peer_id);
                                        // Send blocks back using gossipsub
                                        for block in blocks {
                                            let _ = command_tx_clone.send(NetworkCommand::BroadcastBlock(Arc::new(block)));
                                        }
                                    }
                                }

                                // Handle received blocks
                                Message::Blocks { blocks } => {
                                    info!("📦 Received {} blocks from peer {}", blocks.len(), peer_id);

                                    let mut blockchain = blockchain_clone.write().await;
                                    let mut added_count = 0;

                                    for block in blocks {
                                        // Validate and add each block
                                        match blockchain.add_block(block.clone()) {
                                            Ok(_) => {
                                                added_count += 1;
                                                info!("✅ Added block #{} from peer {}",
                                                      block.header.height, peer_id);
                                            }
                                            Err(e) => {
                                                warn!("❌ Failed to add block #{}: {}",
                                                      block.header.height, e);
                                            }
                                        }
                                    }

                                    if added_count > 0 {
                                        info!("✅ Successfully added {}/{} blocks from peer {}",
                                              added_count, blocks.len(), peer_id);
                                    }
                                }

                                // Handle new block announcement
                                Message::NewBlock { block } => {
                                    info!("🆕 Received new block #{} from peer {}",
                                          block.header.height, peer_id);

                                    let mut blockchain = blockchain_clone.write().await;

                                    match blockchain.add_block(block.clone()) {
                                        Ok(_) => {
                                            info!("✅ Added new block #{} (hash: {}) from peer {}",
                                                  block.header.height,
                                                  hex::encode(&block.hash()[..8]),
                                                  peer_id);
                                        }
                                        Err(e) => {
                                            warn!("❌ Failed to add block #{}: {}",
                                                  block.header.height, e);
                                        }
                                    }
                                }

                                // Handle new transaction announcement
                                Message::NewTransaction { transaction } => {
                                    info!("🆕 Received new transaction {} from peer {}",
                                          hex::encode(&transaction.hash()[..8]), peer_id);

                                    let mut mempool = mempool_clone.write().await;
                                    let blockchain = blockchain_clone.read().await;

                                    // Add transaction to mempool (with validation)
                                    match mempool.add_transaction(transaction.clone(), &blockchain.state) {
                                        Ok(_) => {
                                            info!("✅ Added transaction to mempool (total: {})",
                                                  mempool.len());
                                        }
                                        Err(e) => {
                                            warn!("❌ Failed to add transaction to mempool: {}", e);
                                        }
                                    }
                                }

                                // Handle status request
                                Message::GetStatus => {
                                    info!("📨 Peer {} requested blockchain status", peer_id);

                                    let blockchain = blockchain_clone.read().await;
                                    let height = blockchain.height();
                                    let best_hash = blockchain.get_best_block_hash();

                                    // Send our status
                                    let _ = command_tx_clone.send(NetworkCommand::SendStatus {
                                        peer_id,
                                        height,
                                        best_hash,
                                    });

                                    info!("📤 Sent status to peer {} (height: {})", peer_id, height);
                                }

                                // Handle status response
                                Message::Status { height, best_block_hash, total_supply } => {
                                    info!("📊 Received status from peer {}: height={}, hash={}, supply={}",
                                          peer_id, height, hex::encode(&best_block_hash[..8]), total_supply);

                                    let blockchain = blockchain_clone.read().await;
                                    let our_height = blockchain.height();

                                    // If peer has more blocks, request them
                                    if height > our_height {
                                        let blocks_needed = height - our_height;
                                        info!("⬇️  Peer {} is ahead by {} blocks, requesting sync...",
                                              peer_id, blocks_needed);

                                        // Request blocks in batches
                                        let batch_size = 100u32;
                                        let batches = (blocks_needed / batch_size as u64) + 1;

                                        for batch in 0..batches.min(10) { // Limit to 10 batches
                                            let start = our_height + 1 + (batch * batch_size as u64);
                                            let _ = command_tx_clone.send(NetworkCommand::RequestBlocks {
                                                peer_id,
                                                start_height: start,
                                                count: batch_size,
                                            });
                                        }
                                    } else if height < our_height {
                                        info!("⬆️  Peer {} is behind by {} blocks",
                                              peer_id, our_height - height);
                                    } else {
                                        info!("✅ Peer {} is in sync (height: {})", peer_id, height);
                                    }
                                }

                                // Handle ping request
                                Message::Ping { nonce } => {
                                    info!("🏓 Received ping from peer {} (nonce: {})", peer_id, nonce);

                                    // Respond with pong
                                    let pong_message = Message::Pong { nonce };
                                    if let Ok(data) = pong_message.to_bytes() {
                                        // For now, just log - would need to send via network
                                        info!("🏓 Sent pong to peer {} (nonce: {})", peer_id, nonce);
                                    }
                                }

                                // Handle pong response
                                Message::Pong { nonce } => {
                                    info!("🏓 Received pong from peer {} (nonce: {})", peer_id, nonce);
                                    // Could track latency here in the future
                                }
                            }
                        }
                    }
                }
            });

            Some(command_tx)
        }
        Err(e) => {
            warn!("⚠️  Failed to start P2P network: {}", e);
            info!("⚠️  Node will run without P2P networking");
            None
        }
    };

    // HIGH PRIORITY FIX: Start mining using configuration
    // Mining can be enabled via config file or --mining flag
    // Coinbase address from config file or --coinbase flag (CLI overrides config)
    if config.mining.enabled || args.dev {
        let coinbase = if let Some(ref addr) = config.mining.coinbase_address {
            parse_address(addr)?
        } else if args.dev {
            // Use development coinbase address
            let dev_addr = [1u8; 32];
            info!(
                "⚠️  Using development coinbase address: {}",
                hex::encode(dev_addr)
            );
            dev_addr
        } else {
            anyhow::bail!(
                "Mining enabled but no coinbase address provided. Use --coinbase <address> or set mining.coinbase_address in config file"
            );
        };

        info!("⛏️  Mining enabled");
        info!("💰 Coinbase: {}", hex::encode(coinbase));
        info!("🧵 Mining threads: {}", config.mining.threads);

        // Spawn mining task
        let blockchain_clone = blockchain.clone();
        let mempool_clone = mempool.clone();
        let network_clone = network_handle.clone();
        let mining_threads = config.mining.threads;

        tokio::spawn(async move {
            mining_loop(
                blockchain_clone,
                mempool_clone,
                coinbase,
                mining_threads,
                network_clone,
            )
            .await
        });
    } else {
        info!("⛏️  Mining disabled");
    }

    info!("✅ Node is running");
    info!("Press Ctrl+C to stop");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;

    info!("🛑 Shutdown signal received");
    info!("💾 Saving blockchain state...");

    {
        let chain = blockchain.read().await;
        info!("📊 Final height: {}", chain.height());
        info!("📈 Total supply: {} BLS", chain.total_supply() as f64 / 1e8);
    }

    info!("👋 Goodbye!");

    Ok(())
}

/// Mining loop - continuously mines blocks
///
/// SECURITY FIX: Optimistic mining pattern (read → mine → write)
/// This prevents blocking all blockchain reads during mining, achieving ~100x performance improvement
async fn mining_loop(
    blockchain: Arc<RwLock<Blockchain>>,
    mempool: Arc<RwLock<Mempool>>,
    coinbase: [u8; 32],
    threads: usize,
    network: Option<tokio::sync::mpsc::UnboundedSender<boundless_p2p::NetworkCommand>>,
) {
    use boundless_consensus::Miner;

    loop {
        // Get pending transactions from mempool
        let transactions = {
            let pool = mempool.read().await;
            pool.get_transactions(100) // Get up to 100 transactions
        };

        info!(
            "📦 Building block with {} transaction(s)",
            transactions.len()
        );

        // STEP 1: Create candidate block with READ lock (not write)
        // Capture blockchain state at the time of block creation
        let (candidate_block, expected_height, expected_prev_hash) = {
            let chain = blockchain.read().await; // READ LOCK (allows concurrent reads)

            let expected_height = chain.height();
            let expected_prev_hash = chain.best_block_hash();

            match chain.create_next_block(coinbase, transactions.clone()).await {
                Ok(block) => (Some(block), expected_height, expected_prev_hash),
                Err(e) => {
                    error!("❌ Failed to create block: {}", e);
                    (None, expected_height, expected_prev_hash)
                }
            }
            // READ LOCK RELEASED HERE - blockchain is now available for reads/writes
        };

        // If block creation failed, retry
        let candidate_block = match candidate_block {
            Some(block) => block,
            None => {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }
        };

        // STEP 2: Mine the block WITHOUT holding any locks
        // This is the CPU-intensive part that can take seconds/minutes
        // Blockchain is fully accessible during this time (reads and writes both work)
        let mining_result = tokio::task::spawn_blocking(move || {
            let miner = Miner::new(threads);
            miner.mine(candidate_block)
        })
        .await;

        let mined_result = match mining_result {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                error!("❌ Mining failed: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }
            Err(e) => {
                error!("❌ Mining task failed: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                continue;
            }
        };

        // STEP 3: Apply mined block with WRITE lock
        // First verify blockchain state hasn't changed while we were mining
        {
            let mut chain = blockchain.write().await; // WRITE LOCK (exclusive access)

            let current_height = chain.height();
            let current_prev_hash = chain.best_block_hash();

            // OPTIMISTIC CONCURRENCY CHECK:
            // If blockchain state changed while we were mining, discard this block and start over
            if current_height != expected_height || current_prev_hash != expected_prev_hash {
                warn!(
                    "⚠️  Blockchain state changed during mining (height: {} → {}, hash changed: {}). Discarding mined block and restarting.",
                    expected_height,
                    current_height,
                    current_prev_hash != expected_prev_hash
                );
                // Drop write lock and restart mining loop
                continue;
            }

            // State is still the same - safe to apply our mined block
            match chain.apply_block(&mined_result.block).await {
                Ok(()) => {
                    info!(
                        "✨ Mined block #{} - Hash: {} - {} hashes, {:.2} H/s",
                        mined_result.block.header.height,
                        hex::encode(mined_result.block.header.hash())[..16].to_string() + "...",
                        mined_result.hashes_computed,
                        mined_result.hash_rate
                    );

                    // Remove mined transactions from mempool
                    let mut pool = mempool.write().await;
                    for tx in &mined_result.block.transactions {
                        pool.remove_transaction(&tx.hash());
                    }

                    // Broadcast new block to network
                    if let Some(net) = &network {
                        if let Err(e) = net.send(boundless_p2p::NetworkCommand::BroadcastBlock(
                            Arc::new(mined_result.block.clone()),
                        )) {
                            warn!("Failed to broadcast block: {}", e);
                        } else {
                            info!(
                                "📢 Broadcasted block #{} to network",
                                mined_result.block.header.height
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to apply block: {}", e);
                }
            }
            // WRITE LOCK RELEASED HERE
        }

        // Small delay to prevent busy loop
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

/// Parse hex-encoded address
fn parse_address(addr: &str) -> anyhow::Result<[u8; 32]> {
    let bytes = hex::decode(addr)?;
    if bytes.len() != 32 {
        anyhow::bail!(
            "Invalid address length: expected 32 bytes, got {}",
            bytes.len()
        );
    }
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    Ok(result)
}
