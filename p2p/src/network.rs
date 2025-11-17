// P2P network node implementation
use async_trait::async_trait;
use futures::StreamExt;
use libp2p::{
    core::upgrade,
    gossipsub, identity, mdns, noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, StreamProtocol, Swarm, Transport,
};
use std::collections::HashMap;
use std::io;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::service::NetworkCommand;
use crate::{peer::PeerInfo, protocol::Message};

/// Gossipsub topics
const TOPIC_BLOCKS: &str = "/boundless/blocks/1.0.0";
const TOPIC_TRANSACTIONS: &str = "/boundless/transactions/1.0.0";

/// Request-response codec for Boundless protocol
#[derive(Debug, Clone)]
struct BoundlessCodec {
    max_message_size: usize,
}

impl Default for BoundlessCodec {
    fn default() -> Self {
        Self {
            max_message_size: 10 * 1024 * 1024, // 10MB default
        }
    }
}

#[async_trait]
impl request_response::Codec for BoundlessCodec {
    type Protocol = StreamProtocol;
    type Request = Message;
    type Response = Message;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;

        // SECURITY: Enforce maximum message size to prevent DoS attacks
        let mut buf = Vec::new();
        let bytes_read = io
            .take(self.max_message_size as u64)
            .read_to_end(&mut buf)
            .await?;

        if bytes_read >= self.max_message_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Message size {} exceeds maximum {}",
                    bytes_read, self.max_message_size
                ),
            ));
        }

        Message::from_bytes(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;

        // SECURITY: Enforce maximum message size to prevent DoS attacks
        let mut buf = Vec::new();
        let bytes_read = io
            .take(self.max_message_size as u64)
            .read_to_end(&mut buf)
            .await?;

        if bytes_read >= self.max_message_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Message size {} exceeds maximum {}",
                    bytes_read, self.max_message_size
                ),
            ));
        }

        Message::from_bytes(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;
        let data = req
            .to_bytes()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&data).await?;
        io.close().await
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;
        let data = res
            .to_bytes()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&data).await?;
        io.close().await
    }
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Listen address
    pub listen_addr: Multiaddr,

    /// Bootstrap nodes
    pub bootnodes: Vec<Multiaddr>,

    /// Enable mDNS for local peer discovery
    pub enable_mdns: bool,

    /// Maximum peers
    pub max_peers: usize,

    /// SECURITY: Message size limits
    /// Maximum message size (10MB for blocks)
    pub max_message_size: usize,

    /// Maximum block size (10MB)
    pub max_block_size: usize,

    /// Maximum transaction size (1MB)
    pub max_transaction_size: usize,

    /// Maximum inbound connections
    pub max_inbound_connections: usize,

    /// Maximum outbound connections
    pub max_outbound_connections: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/30333".parse().unwrap(),
            bootnodes: Vec::new(),
            enable_mdns: true,
            max_peers: 50,
            // SECURITY: Default size limits to prevent DoS attacks
            max_message_size: 10 * 1024 * 1024, // 10MB
            max_block_size: 10 * 1024 * 1024,   // 10MB
            max_transaction_size: 1024 * 1024,  // 1MB
            max_inbound_connections: 30,
            max_outbound_connections: 20,
        }
    }
}

/// Network events
#[derive(Debug)]
pub enum NetworkEvent {
    /// New peer connected
    PeerConnected(PeerId),

    /// Peer disconnected
    PeerDisconnected(PeerId),

    /// Message received from peer
    MessageReceived { peer_id: PeerId, message: Message },

    /// Local address discovered
    NewListenAddr(Multiaddr),
}

/// Network behaviour
#[derive(NetworkBehaviour)]
struct BoundlessBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    request_response: request_response::Behaviour<BoundlessCodec>,
}

/// P2P network node
pub struct NetworkNode {
    swarm: Swarm<BoundlessBehaviour>,
    peers: HashMap<PeerId, PeerInfo>,
    event_tx: mpsc::UnboundedSender<NetworkEvent>,
    blocks_topic: gossipsub::IdentTopic,
    transactions_topic: gossipsub::IdentTopic,
    /// Peers subscribed to blocks topic
    blocks_subscribers: std::collections::HashSet<PeerId>,
    /// Peers subscribed to transactions topic
    tx_subscribers: std::collections::HashSet<PeerId>,
    /// HIGH PRIORITY FIX: Pending request-response channels
    /// Maps (peer_id, request content) to response channel for answering requests
    pending_requests: HashMap<
        PeerId,
        Vec<request_response::ResponseChannel<Message>>,
    >,
}

impl NetworkNode {
    /// Create a new network node
    pub fn new(
        config: NetworkConfig,
    ) -> anyhow::Result<(Self, mpsc::UnboundedReceiver<NetworkEvent>)> {
        info!("🌍 Initializing P2P network node");

        // Create identity
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!("🆔 Local PeerId: {}", local_peer_id);

        // Create transport
        let transport = tcp::tokio::Transport::default()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        // Create gossipsub topics
        let blocks_topic = gossipsub::IdentTopic::new(TOPIC_BLOCKS);
        let transactions_topic = gossipsub::IdentTopic::new(TOPIC_TRANSACTIONS);

        // Create gossipsub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()?;

        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .map_err(|e| anyhow::anyhow!("{}", e))?;

        // Subscribe to topics
        gossipsub.subscribe(&blocks_topic)?;
        gossipsub.subscribe(&transactions_topic)?;

        info!("📢 Subscribed to gossipsub topics");
        info!("   - Blocks: {}", TOPIC_BLOCKS);
        info!("   - Transactions: {}", TOPIC_TRANSACTIONS);

        // Create mDNS
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        // Create request-response protocol
        let req_resp_config = request_response::Config::default();
        let protocols = std::iter::once((
            StreamProtocol::new("/boundless/req-resp/1.0.0"),
            ProtocolSupport::Full,
        ));
        let request_response = request_response::Behaviour::new(protocols, req_resp_config);

        info!("🔄 Request-response protocol initialized");

        // Create behaviour
        let behaviour = BoundlessBehaviour {
            gossipsub,
            mdns,
            request_response,
        };

        // Create swarm
        let mut swarm = Swarm::new(
            transport,
            behaviour,
            local_peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );

        // Listen on address
        swarm.listen_on(config.listen_addr.clone())?;

        info!("👂 Listening on {}", config.listen_addr);

        // Connect to bootnodes
        for bootnode in &config.bootnodes {
            if let Err(e) = swarm.dial(bootnode.clone()) {
                warn!("Failed to dial bootnode {}: {}", bootnode, e);
            }
        }

        // Create event channel
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Ok((
            Self {
                swarm,
                peers: HashMap::new(),
                event_tx,
                blocks_topic,
                transactions_topic,
                blocks_subscribers: std::collections::HashSet::new(),
                tx_subscribers: std::collections::HashSet::new(),
                pending_requests: HashMap::new(),
            },
            event_rx,
        ))
    }

    /// Run the network event loop (takes ownership, spawned in dedicated task)
    pub async fn run(
        mut self,
        mut command_rx: tokio::sync::mpsc::UnboundedReceiver<crate::service::NetworkCommand>,
    ) {
        info!("▶️  Starting P2P network event loop");

        loop {
            tokio::select! {
                Some(command) = command_rx.recv() => {
                    self.handle_command(command).await;
                }
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, command: NetworkCommand) {
        match command {
            NetworkCommand::BroadcastBlock(block) => {
                // FIXME: Removed subscriber check - gossipsub will handle message routing

                let message = Message::NewBlock {
                    block: (*block).clone(),
                };
                if let Ok(data) = message.to_bytes() {
                    match self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(self.blocks_topic.clone(), data)
                    {
                        Ok(_) => {
                            info!(
                                "📢 Broadcasted block #{} to {} peer(s)",
                                block.header.height,
                                self.blocks_subscribers.len()
                            );
                        }
                        Err(gossipsub::PublishError::InsufficientPeers) => {
                            // This can happen transiently, just log at debug level
                        }
                        Err(e) => {
                            warn!("Failed to broadcast block: {}", e);
                        }
                    }
                }
            }

            NetworkCommand::BroadcastTransaction(tx) => {
                let message = Message::NewTransaction {
                    transaction: (*tx).clone(),
                };
                if let Ok(data) = message.to_bytes() {
                    match self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(self.transactions_topic.clone(), data)
                    {
                        Ok(_) => {
                            info!(
                                "📢 Broadcasted transaction to {} peer(s)",
                                self.tx_subscribers.len()
                            );
                        }
                        Err(gossipsub::PublishError::InsufficientPeers) => {
                            // This can happen transiently, just log at debug level
                        }
                        Err(e) => {
                            warn!("Failed to broadcast transaction: {}", e);
                        }
                    }
                }
            }

            NetworkCommand::SendStatus {
                peer_id,
                height,
                best_hash,
            } => {
                let message = Message::Status {
                    height,
                    best_block_hash: best_hash,
                    total_supply: 0,
                };
                if let Ok(data) = message.to_bytes() {
                    if let Err(e) = self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(self.blocks_topic.clone(), data)
                    {
                        warn!("Failed to send status: {}", e);
                    }
                }
            }

            NetworkCommand::RequestBlocks {
                peer_id,
                start_height,
                count,
            } => {
                let message = Message::GetBlocks {
                    start_height,
                    count,
                };
                if let Ok(data) = message.to_bytes() {
                    if let Err(e) = self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(self.blocks_topic.clone(), data)
                    {
                        warn!("Failed to request blocks: {}", e);
                    }
                }
            }
        }
    }

    /// Handle swarm events
    async fn handle_swarm_event(&mut self, event: SwarmEvent<BoundlessBehaviourEvent>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("📡 Listening on {}", address);
                let _ = self.event_tx.send(NetworkEvent::NewListenAddr(address));
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("🤝 Connected to peer: {}", peer_id);
                let peer_info = PeerInfo::new(peer_id.to_string());
                self.peers.insert(peer_id, peer_info);
                let _ = self.event_tx.send(NetworkEvent::PeerConnected(peer_id));
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("👋 Disconnected from peer: {}", peer_id);
                self.peers.remove(&peer_id);
                let _ = self.event_tx.send(NetworkEvent::PeerDisconnected(peer_id));
            }
            SwarmEvent::Behaviour(BoundlessBehaviourEvent::Gossipsub(event)) => {
                match event {
                    gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id,
                        message,
                    } => {
                        // Deserialize and forward message
                        match Message::from_bytes(&message.data) {
                            Ok(msg) => {
                                info!("📩 Received {} from {}", msg.message_type(), peer_id);
                                let _ = self.event_tx.send(NetworkEvent::MessageReceived {
                                    peer_id,
                                    message: msg,
                                });
                            }
                            Err(e) => {
                                warn!("Failed to deserialize message from {}: {}", peer_id, e);
                            }
                        }
                    }
                    gossipsub::Event::Subscribed { peer_id, topic } => {
                        info!("✅ Peer {} subscribed to topic {}", peer_id, topic);
                        // Track subscription
                        if topic.as_str() == TOPIC_BLOCKS {
                            self.blocks_subscribers.insert(peer_id);
                            info!(
                                "   Total blocks subscribers: {}",
                                self.blocks_subscribers.len()
                            );
                        } else if topic.as_str() == TOPIC_TRANSACTIONS {
                            self.tx_subscribers.insert(peer_id);
                            info!("   Total tx subscribers: {}", self.tx_subscribers.len());
                        }
                    }
                    gossipsub::Event::Unsubscribed { peer_id, topic } => {
                        info!("❌ Peer {} unsubscribed from topic {}", peer_id, topic);
                        // Remove subscription
                        if topic.as_str() == TOPIC_BLOCKS {
                            self.blocks_subscribers.remove(&peer_id);
                        } else if topic.as_str() == TOPIC_TRANSACTIONS {
                            self.tx_subscribers.remove(&peer_id);
                        }
                    }
                    gossipsub::Event::GossipsubNotSupported { peer_id } => {
                        warn!("⚠️  Peer {} does not support gossipsub", peer_id);
                    }
                    _ => {}
                }
            }
            SwarmEvent::Behaviour(BoundlessBehaviourEvent::Mdns(event)) => match event {
                mdns::Event::Discovered(list) => {
                    for (peer_id, multiaddr) in list {
                        info!("🔍 Discovered peer {} at {}", peer_id, multiaddr);
                        if let Err(e) = self.swarm.dial(multiaddr.clone()) {
                            warn!("Failed to dial discovered peer: {}", e);
                        }
                    }
                }
                mdns::Event::Expired(list) => {
                    for (peer_id, _) in list {
                        info!("⏰ Peer {} expired from mDNS", peer_id);
                    }
                }
            },
            // HIGH PRIORITY FIX: Handle request-response protocol events
            SwarmEvent::Behaviour(BoundlessBehaviourEvent::RequestResponse(event)) => {
                use request_response::{Event, Message};
                match event {
                    Event::Message { peer, message } => match message {
                        Message::Request {
                            request_id,
                            request,
                            channel,
                        } => {
                            info!(
                                "📨 Received request {:?} from peer {}: {}",
                                request_id,
                                peer,
                                request.message_type()
                            );

                            // HIGH PRIORITY FIX: Store the response channel for later use
                            // The node will process the request and send response via the channel
                            self.pending_requests
                                .entry(peer)
                                .or_insert_with(Vec::new)
                                .push(channel);

                            // Forward request to node for processing
                            let _ = self.event_tx.send(NetworkEvent::MessageReceived {
                                peer_id: peer,
                                message: request,
                            });
                        }
                        Message::Response {
                            request_id,
                            response,
                        } => {
                            info!(
                                "📨 Received response {:?} from peer {}: {}",
                                request_id,
                                peer,
                                response.message_type()
                            );

                            // Forward response to node for processing
                            let _ = self.event_tx.send(NetworkEvent::MessageReceived {
                                peer_id: peer,
                                message: response,
                            });
                        }
                    },
                    Event::OutboundFailure {
                        peer,
                        request_id,
                        error,
                    } => {
                        warn!(
                            "❌ Outbound request {:?} to peer {} failed: {}",
                            request_id, peer, error
                        );
                    }
                    Event::InboundFailure {
                        peer,
                        request_id,
                        error,
                    } => {
                        warn!(
                            "❌ Inbound request {:?} from peer {} failed: {}",
                            request_id, peer, error
                        );
                    }
                    Event::ResponseSent { peer, request_id } => {
                        info!(
                            "✅ Response to request {:?} sent to peer {}",
                            request_id, peer
                        );
                    }
                }
            }
            _ => {}
        }
    }

    /// Get connected peers
    pub fn peers(&self) -> &HashMap<PeerId, PeerInfo> {
        &self.peers
    }

    /// Get peer count
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Broadcast a new block to all peers
    pub fn broadcast_block(&mut self, block: boundless_core::Block) -> anyhow::Result<()> {
        let message = Message::NewBlock { block };
        let data = message.to_bytes()?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.blocks_topic.clone(), data)
            .map_err(|e| anyhow::anyhow!("Failed to publish block: {:?}", e))?;

        info!("📢 Broadcasted new block");
        Ok(())
    }

    /// Broadcast a new transaction to all peers
    pub fn broadcast_transaction(
        &mut self,
        transaction: boundless_core::Transaction,
    ) -> anyhow::Result<()> {
        let message = Message::NewTransaction { transaction };
        let data = message.to_bytes()?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.transactions_topic.clone(), data)
            .map_err(|e| anyhow::anyhow!("Failed to publish transaction: {:?}", e))?;

        info!("📢 Broadcasted new transaction");
        Ok(())
    }

    /// Request blocks from a peer
    pub fn request_blocks(
        &mut self,
        peer_id: PeerId,
        start_height: u64,
        count: u32,
    ) -> anyhow::Result<()> {
        let message = Message::GetBlocks {
            start_height,
            count,
        };

        // Use request-response protocol for direct peer communication
        let request_id = self
            .swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, message);

        info!(
            "📨 Sent request {:?} to peer {} for {} blocks starting from height {}",
            request_id, peer_id, count, start_height
        );
        Ok(())
    }

    /// Send blocks to a peer
    pub fn send_blocks(&mut self, blocks: Vec<boundless_core::Block>) -> anyhow::Result<()> {
        let message = Message::Blocks {
            blocks: blocks.clone(),
        };
        let data = message.to_bytes()?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.blocks_topic.clone(), data)
            .map_err(|e| anyhow::anyhow!("Failed to send blocks: {:?}", e))?;

        info!("📤 Sent {} blocks", blocks.len());
        Ok(())
    }

    /// Get blockchain status request
    pub fn request_status(&mut self) -> anyhow::Result<()> {
        let message = Message::GetStatus;
        let data = message.to_bytes()?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.blocks_topic.clone(), data)
            .map_err(|e| anyhow::anyhow!("Failed to request status: {:?}", e))?;

        info!("📨 Requested network status");
        Ok(())
    }

    /// Send blockchain status
    pub fn send_status(
        &mut self,
        height: u64,
        best_block_hash: [u8; 32],
        total_supply: u64,
    ) -> anyhow::Result<()> {
        let message = Message::Status {
            height,
            best_block_hash,
            total_supply,
        };
        let data = message.to_bytes()?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.blocks_topic.clone(), data)
            .map_err(|e| anyhow::anyhow!("Failed to send status: {:?}", e))?;

        info!("📤 Sent blockchain status (height: {})", height);
        Ok(())
    }

    /// HIGH PRIORITY FIX: Send response to a pending request
    /// This uses the request-response protocol to send a direct response to a peer
    pub fn send_response_to_peer(
        &mut self,
        peer_id: PeerId,
        response: Message,
    ) -> anyhow::Result<()> {
        // Get and remove the oldest pending request channel for this peer
        if let Some(channels) = self.pending_requests.get_mut(&peer_id) {
            if let Some(channel) = channels.pop() {
                // Send response through request-response channel
                if self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, response.clone())
                    .is_ok()
                {
                    info!(
                        "📤 Sent {} response to peer {} via request-response",
                        response.message_type(),
                        peer_id
                    );

                    // Clean up empty channel list
                    if channels.is_empty() {
                        self.pending_requests.remove(&peer_id);
                    }

                    return Ok(());
                } else {
                    warn!("❌ Failed to send response to peer {} (channel closed)", peer_id);
                }
            }
        }

        // Fallback: No pending request channel found, this shouldn't happen
        // but we can still try to use gossipsub as a fallback
        warn!(
            "⚠️  No pending request channel for peer {}, using gossipsub fallback",
            peer_id
        );

        // Fallback to gossipsub (less efficient but ensures delivery)
        let data = response.to_bytes()?;
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.blocks_topic.clone(), data)
            .map_err(|e| anyhow::anyhow!("Failed to send response: {:?}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.max_peers, 50);
        assert!(config.enable_mdns);
    }
}
