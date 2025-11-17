// Peer information and management
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer ID (libp2p PeerId as string)
    pub peer_id: String,

    /// Multiaddresses
    pub addresses: Vec<String>,

    /// Protocol version
    pub protocol_version: u32,

    /// Node version
    pub node_version: String,

    /// Last seen timestamp
    pub last_seen: u64,

    /// Connection status
    pub connected: bool,

    /// Blockchain height
    pub height: u64,

    /// Best block hash
    pub best_block_hash: Option<[u8; 32]>,
}

impl PeerInfo {
    /// Create new peer info
    pub fn new(peer_id: String) -> Self {
        Self {
            peer_id,
            addresses: Vec::new(),
            protocol_version: 1,
            node_version: env!("CARGO_PKG_VERSION").to_string(),
            last_seen: current_timestamp(),
            connected: false,
            height: 0,
            best_block_hash: None,
        }
    }

    /// Update last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = current_timestamp();
    }

    /// Mark as connected
    pub fn mark_connected(&mut self) {
        self.connected = true;
        self.update_last_seen();
    }

    /// Mark as disconnected
    pub fn mark_disconnected(&mut self) {
        self.connected = false;
    }

    /// Update blockchain status
    pub fn update_status(&mut self, height: u64, best_block_hash: [u8; 32]) {
        self.height = height;
        self.best_block_hash = Some(best_block_hash);
        self.update_last_seen();
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
