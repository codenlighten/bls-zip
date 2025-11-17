use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

mod blockchain;
mod mempool;
mod config;
mod rpc_impl;

use blockchain::Blockchain;
use mempool::Mempool;
use config::NodeConfig;
use boundless_rpc::RpcServer;
use boundless_p2p::{NetworkNode, NetworkConfig, NetworkHandle, Message};
use boundless_p2p::network::NetworkEvent;
use libp2p::PeerId;

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

    /// RPC HTTP port
    #[arg(long, default_value = "9933")]
    rpc_port: u16,

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
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    let args = Args::parse();

    info!("🚀 Starting Boundless BLS Node v{}", env!("CARGO_PKG_VERSION"));
    info!("📁 Data directory: {:?}", args.base_path);

    // Load or create configuration
    let config = if let Some(config_path) = args.config {
        NodeConfig::from_file(&config_path)?
    } else if args.dev {
        info!("🔧 Development mode enabled");
        NodeConfig::development()
    } else {
        NodeConfig::default()
    };

    // Create data directory if it doesn't exist
    if !args.base_path.exists() {
        std::fs::create_dir_all(&args.base_path)?;
    }

    // Initialize blockchain
    let blockchain = Arc::new(RwLock::new(
        Blockchain::new(args.base_path.clone(), config.clone())?
    ));

    {
        let chain = blockchain.read().await;
        info!("⛓️  Blockchain initialized at height {}", chain.height());
        info!("🔗 Best block: {}", hex::encode(chain.best_block_hash()));
    }

    // Initialize mempool
    let mempool = Arc::new(RwLock::new(Mempool::new(config.mempool_config())));

    info!("💾 Mempool initialized");


    // Start RPC server
    let rpc_addr = format!("{}:{}", "0.0.0.0", args.rpc_port);
    let rpc_blockchain = blockchain.clone();

    // Start RPC server and keep handle alive (dropping it shuts down the server)
    let rpc_handle = RpcServer::start(&rpc_addr, rpc_blockchain).await?;
    info!("🌐 RPC server running on {}", rpc_addr);














    // Start P2P network
    let p2p_config = NetworkConfig {
        listen_addr: format!("/ip4/0.0.0.0/tcp/{}", args.port).parse()?,
        bootnodes: vec![], // TODO: Add bootnodes from config
        enable_mdns: true,
        max_peers: 50,
    };

    // Create command channel for network communication
    let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
    let network_handle = boundless_p2p::NetworkHandle::new(command_tx);

    match NetworkNode::new(p2p_config) {
        Ok((network, mut events)) => {
            info!("🌐 P2P network initialized");

            // Spawn dedicated network task (owns the Swarm)
            tokio::spawn(async move {
                network.run(command_rx).await;
            });

            // Clone handle for event handler
            let handle_clone = network_handle.clone();

            // Spawn event handler
            let blockchain_clone = blockchain.clone();
            let mempool_clone = mempool.clone();

            tokio::spawn(async move {
                while let Some(event) = events.recv().await {
                    match event {
                        NetworkEvent::PeerConnected(peer_id) => {
                            info!("🤝 Peer connected: {}", peer_id);
                        }
                        NetworkEvent::PeerDisconnected(peer_id) => {
                            info!("👋 Peer disconnected: {}", peer_id);
                        }
//                         NetworkEvent::BlockReceived { peer_id, block } => {
//                             info!("📦 Received block #{} from {}", block.header.height, peer_id);
//                             
//                             let mut chain = blockchain_clone.write().await;
//                             match chain.apply_block(&block).await {
//                                 Ok(_) => {
//                                     info!("✅ Applied block #{} from network", block.header.height);
//                                 }
//                                 Err(e) => {
//                                     warn!("Failed to apply block: {}", e);
//                                 }
//                             }
//                         }
//                         NetworkEvent::TransactionReceived { peer_id, transaction } => {
//                             info!("💵 Received transaction from {}", peer_id);
//                             
//                             let mut mp = mempool_clone.write().await;
//                             if let Err(e) = mp.add_transaction(transaction) {
//                                 warn!("Failed to add transaction: {}", e);
//                             }
//                         }
//                         NetworkEvent::StatusReceived { peer_id, height, best_hash } => {
//                             info!("📊 Peer {} at height {}", peer_id, height);
//                         }
//                         NetworkEvent::BlocksRequested { peer_id, start_height, count } => {
//                             info!("📨 Peer {} requested {} blocks from {}", peer_id, count, start_height);
//                         }
//                         NetworkEvent::ListeningOn(addr) => {
//                             info!("📡 Listening on: {}", addr);
//                         }
//                     }
                }
            });

            Some(network_handle.clone())

        }
        Err(e) => {
            warn!("⚠️  Failed to start P2P network: {}", e);
            info!("⚠️  Node will run without P2P networking");
            None
        }
    };

    // Start mining if enabled
    if args.mining || args.dev {
        let coinbase = if let Some(addr) = args.coinbase {
            parse_address(&addr)?
        } else if args.dev {
            // Use development coinbase address
            let dev_addr = [1u8; 32];
            info!("⚠️  Using development coinbase address: {}", hex::encode(dev_addr));
            dev_addr
        } else {
            anyhow::bail!("Mining enabled but no coinbase address provided. Use --coinbase <address>");
        };

        info!("⛏️  Mining enabled");
        info!("💰 Coinbase: {}", hex::encode(coinbase));
        info!("🧵 Mining threads: {}", args.mining_threads);

        // Spawn mining task
        let blockchain_clone = blockchain.clone();
        let mempool_clone = mempool.clone();
        let network_clone = network_handle.clone();

        tokio::spawn(async move {
            mining_loop(blockchain_clone, mempool_clone, coinbase, args.mining_threads, network_clone).await
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

/// Handle network messages from peers
async fn handle_network_message(
    peer_id: PeerId,
    message: Message,
    blockchain: Arc<RwLock<Blockchain>>,
    mempool: Arc<RwLock<Mempool>>,
    network: Arc<RwLock<NetworkNode>>,
) {
    match message {
        Message::NewBlock { block } => {
            info!("📦 Received new block #{} from {}", block.header.height, peer_id);

            // Validate and apply block
            let mut chain = blockchain.write().await;
            match chain.apply_block(&block).await {
                Ok(()) => {
                    info!("✅ Applied block #{} from network", block.header.height);

                    // Remove transactions from mempool
                    let mut pool = mempool.write().await;
                    for tx in &block.transactions {
                        pool.remove_transaction(&tx.hash());
                    }
                }
                Err(e) => {
                    warn!("❌ Failed to apply received block: {}", e);
                }
            }
        }
        Message::NewTransaction { transaction } => {
            info!("💸 Received new transaction from {}", peer_id);

            // Add to mempool
            let mut pool = mempool.write().await;
            if let Err(e) = pool.add_transaction(transaction) {
                warn!("Failed to add transaction to mempool: {}", e);
            } else {
                info!("✅ Added transaction to mempool");
            }
        }
        Message::GetBlocks { start_height, count } => {
            info!("📨 Received block request from {} (height: {}, count: {})", peer_id, start_height, count);

            // Fetch requested blocks
            let chain = blockchain.read().await;
            let mut blocks = Vec::new();

            for height in start_height..(start_height + count as u64) {
                if let Some(block) = chain.get_block(height) {
                    blocks.push(block);
                } else {
                    break;
                }
            }

            if !blocks.is_empty() {
                let mut net = network.write().await;
                if let Err(e) = net.send_blocks(blocks.clone()) {
                    warn!("Failed to send blocks: {}", e);
                } else {
                    info!("📤 Sent {} blocks to network", blocks.len());
                }
            }
        }
        Message::Blocks { blocks } => {
            let total_blocks = blocks.len();
            info!("📦 Received {} blocks from {}", total_blocks, peer_id);

            // Apply blocks in order
            let mut chain = blockchain.write().await;
            let mut applied_count = 0;

            for block in blocks {
                match chain.apply_block(&block).await {
                    Ok(()) => {
                        applied_count += 1;

                        // Remove transactions from mempool
                        let mut pool = mempool.write().await;
                        for tx in &block.transactions {
                            pool.remove_transaction(&tx.hash());
                        }
                    }
                    Err(e) => {
                        warn!("Failed to apply block #{}: {}", block.header.height, e);
                        break; // Stop on first error
                    }
                }
            }

            info!("✅ Applied {}/{} blocks from network", applied_count, total_blocks);
        }
        Message::GetStatus => {
            info!("📨 Received status request from {}", peer_id);

            // Send our blockchain status
            let chain = blockchain.read().await;
            let mut net = network.write().await;

            if let Err(e) = net.send_status(
                chain.height(),
                chain.best_block_hash(),
                chain.total_supply(),
            ) {
                warn!("Failed to send status: {}", e);
            }
        }
        Message::Status { height, best_block_hash, total_supply } => {
            info!("📊 Peer {} status: height={}, supply={}", peer_id, height, total_supply);

            // Check if peer is ahead of us
            let chain = blockchain.read().await;
            let our_height = chain.height();

            if height > our_height {
                info!("⬇️  Peer is ahead (our height: {}, peer height: {})", our_height, height);

                // Request missing blocks
                let blocks_to_request = (height - our_height).min(100) as u32;
                let mut net = network.write().await;

                if let Err(e) = net.request_blocks(peer_id, our_height + 1, blocks_to_request) {
                    warn!("Failed to request blocks: {}", e);
                } else {
                    info!("📨 Requesting {} blocks from height {}", blocks_to_request, our_height + 1);
                }
            }
        }
        Message::Ping { nonce } => {
            // Respond with pong (future implementation)
        }
        Message::Pong { nonce } => {
            // Handle pong (future implementation)
        }
    }
}

/// Mining loop - continuously mines blocks
async fn mining_loop(
    blockchain: Arc<RwLock<Blockchain>>,
    mempool: Arc<RwLock<Mempool>>,
    coinbase: [u8; 32],
    threads: usize,
    network: Option<NetworkHandle>,
) {
    use boundless_consensus::Miner;

    let miner = Miner::new(threads);

    loop {
        // Get pending transactions from mempool
        let transactions = {
            let pool = mempool.read().await;
            pool.get_transactions(100) // Get up to 100 transactions
        };

        info!("📦 Building block with {} transaction(s)", transactions.len());

        // Mine new block
        let result = {
            let mut chain = blockchain.write().await;
            match chain.create_next_block(coinbase, transactions).await {
                Ok(block) => {
                    match miner.mine(block) {
                        Ok(result) => Some(result),
                        Err(e) => {
                            error!("❌ Mining failed: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to create block: {}", e);
                    None
                }
            }
        };

        if let Some(result) = result {
            // Apply mined block to blockchain
            let mut chain = blockchain.write().await;
            match chain.apply_block(&result.block).await {
                Ok(()) => {
                    info!(
                        "✨ Mined block #{} - Hash: {} - {} hashes, {:.2} H/s",
                        result.block.header.height,
                        hex::encode(result.block.header.hash())[..16].to_string() + "...",
                        result.hashes_computed,
                        result.hash_rate
                    );

                    // Remove mined transactions from mempool
                    let mut pool = mempool.write().await;
                    for tx in &result.block.transactions {
                        pool.remove_transaction(&tx.hash());
                    }

                    // Broadcast new block to network
                    if let Some(net) = &network {
                        use std::sync::Arc;
                        if let Err(e) = net.broadcast_block(Arc::new(result.block.clone())) {
                            warn!("Failed to broadcast block: {}", e);
                        } else {
                            info!("📢 Broadcasted block #{} to network", result.block.header.height);
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Failed to apply block: {}", e);
                }
            }
        }

        // Small delay to prevent busy loop on errors
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

/// Parse hex-encoded address
fn parse_address(addr: &str) -> anyhow::Result<[u8; 32]> {
    let bytes = hex::decode(addr)?;
    if bytes.len() != 32 {
        anyhow::bail!("Invalid address length: expected 32 bytes, got {}", bytes.len());
    }
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    Ok(result)
}
