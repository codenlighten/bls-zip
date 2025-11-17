// Network service with channel-based communication
use boundless_core::{Block, Transaction};
use libp2p::PeerId;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Commands that can be sent to the network service
#[derive(Debug, Clone)]
pub enum NetworkCommand {
    /// Broadcast a block to all peers
    BroadcastBlock(Arc<Block>),

    /// Broadcast a transaction to all peers
    BroadcastTransaction(Arc<Transaction>),

    /// Send status to a specific peer
    SendStatus {
        peer_id: PeerId,
        height: u64,
        best_hash: [u8; 32],
    },

    /// Request blocks from a peer
    RequestBlocks {
        peer_id: PeerId,
        start_height: u64,
        count: u32,
    },
}

/// Events that the network service emits
#[derive(Debug)]
pub enum NetworkEvent {
    /// New peer connected
    PeerConnected(PeerId),

    /// Peer disconnected
    PeerDisconnected(PeerId),

    /// Received a block from a peer
    BlockReceived { peer_id: PeerId, block: Block },

    /// Received a transaction from a peer
    TransactionReceived {
        peer_id: PeerId,
        transaction: Transaction,
    },

    /// Received status from a peer
    StatusReceived {
        peer_id: PeerId,
        height: u64,
        best_hash: [u8; 32],
    },

    /// Received block request from a peer
    BlocksRequested {
        peer_id: PeerId,
        start_height: u64,
        count: u32,
    },

    /// Local listening address
    ListeningOn(libp2p::Multiaddr),
}

/// Handle to communicate with the network service
#[derive(Clone)]
pub struct NetworkHandle {
    command_tx: mpsc::UnboundedSender<NetworkCommand>,
}

impl NetworkHandle {
    pub fn new(command_tx: mpsc::UnboundedSender<NetworkCommand>) -> Self {
        Self { command_tx }
    }

    /// Broadcast a block to all connected peers
    pub fn broadcast_block(&self, block: Arc<Block>) -> anyhow::Result<()> {
        self.command_tx
            .send(NetworkCommand::BroadcastBlock(block))?;
        Ok(())
    }

    /// Broadcast a transaction to all connected peers
    pub fn broadcast_transaction(&self, tx: Arc<Transaction>) -> anyhow::Result<()> {
        self.command_tx
            .send(NetworkCommand::BroadcastTransaction(tx))?;
        Ok(())
    }

    /// Send our blockchain status to a peer
    pub fn send_status(
        &self,
        peer_id: PeerId,
        height: u64,
        best_hash: [u8; 32],
    ) -> anyhow::Result<()> {
        self.command_tx.send(NetworkCommand::SendStatus {
            peer_id,
            height,
            best_hash,
        })?;
        Ok(())
    }

    /// Request blocks from a peer
    pub fn request_blocks(
        &self,
        peer_id: PeerId,
        start_height: u64,
        count: u32,
    ) -> anyhow::Result<()> {
        self.command_tx.send(NetworkCommand::RequestBlocks {
            peer_id,
            start_height,
            count,
        })?;
        Ok(())
    }
}
