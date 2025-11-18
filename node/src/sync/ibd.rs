// Initial Block Download (IBD) Orchestrator
// Implements efficient blockchain synchronization
//
// Key improvements over naive sync:
// 1. Headers-first: Download and validate headers before full blocks
// 2. Parallel downloads: Fetch multiple blocks concurrently
// 3. Peer selection: Choose best peers based on chainwork
// 4. Efficient state management: Track progress and retry failures

use anyhow::Result;
use boundless_core::{Block, BlockHeader};
use libp2p::PeerId;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Maximum number of headers to download in a single batch
const MAX_HEADERS_BATCH: usize = 2000;

/// Maximum number of blocks to download in parallel
const MAX_PARALLEL_BLOCKS: usize = 16;

/// Timeout for block download from a single peer
const BLOCK_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(30);

/// IBD synchronization state
#[derive(Debug, Clone, PartialEq)]
pub enum SyncState {
    /// Node is fully synced
    Synced,

    /// Downloading block headers
    DownloadingHeaders {
        /// Current height of headers chain
        headers_height: u64,
        /// Target height (best known from peers)
        target_height: u64,
    },

    /// Downloading full blocks
    DownloadingBlocks {
        /// Height of last fully downloaded block
        blocks_height: u64,
        /// Height of downloaded headers
        headers_height: u64,
    },

    /// Validating downloaded blocks
    Validating {
        /// Current validation height
        current_height: u64,
        /// Total blocks to validate
        total_blocks: u64,
    },
}

/// Information about a peer's blockchain state
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer ID
    pub peer_id: PeerId,

    /// Reported blockchain height
    pub height: u64,

    /// Reported best block hash
    pub best_hash: [u8; 32],

    /// Cumulative chainwork (for selecting best peer)
    pub chainwork: u64,

    /// Last time we received data from this peer
    pub last_seen: Instant,

    /// Number of failed requests to this peer
    pub failures: u32,
}

impl PeerInfo {
    pub fn new(peer_id: PeerId, height: u64, best_hash: [u8; 32], chainwork: u64) -> Self {
        Self {
            peer_id,
            height,
            best_hash,
            chainwork,
            last_seen: Instant::now(),
            failures: 0,
        }
    }

    /// Check if peer is responsive
    pub fn is_responsive(&self) -> bool {
        self.failures < 3 && self.last_seen.elapsed() < Duration::from_secs(60)
    }

    /// Check if peer has better chain than us
    pub fn has_better_chain(&self, our_chainwork: u64) -> bool {
        self.chainwork > our_chainwork && self.is_responsive()
    }
}

/// Request for downloading a block
#[derive(Debug, Clone)]
struct BlockRequest {
    /// Block height
    height: u64,

    /// Expected block hash (from headers)
    expected_hash: [u8; 32],

    /// Peer assigned to download this block
    peer_id: Option<PeerId>,

    /// Time when download was requested
    requested_at: Option<Instant>,

    /// Number of retry attempts
    retries: u32,
}

impl BlockRequest {
    fn new(height: u64, expected_hash: [u8; 32]) -> Self {
        Self {
            height,
            expected_hash,
            peer_id: None,
            requested_at: None,
            retries: 0,
        }
    }

    /// Check if request has timed out
    fn is_timed_out(&self) -> bool {
        if let Some(requested_at) = self.requested_at {
            requested_at.elapsed() > BLOCK_DOWNLOAD_TIMEOUT
        } else {
            false
        }
    }

    /// Assign peer and mark as requested
    fn assign_peer(&mut self, peer_id: PeerId) {
        self.peer_id = Some(peer_id);
        self.requested_at = Some(Instant::now());
    }

    /// Reset for retry
    fn reset_for_retry(&mut self) {
        self.peer_id = None;
        self.requested_at = None;
        self.retries += 1;
    }
}

/// Initial Block Download Orchestrator
///
/// Manages efficient blockchain synchronization using:
/// - Headers-first sync to validate chain before downloading blocks
/// - Parallel block downloads from multiple peers
/// - Intelligent peer selection based on chainwork
/// - Automatic retry and timeout handling
pub struct IbdOrchestrator {
    /// Current synchronization state
    state: SyncState,

    /// Known peers and their blockchain state
    peers: HashMap<PeerId, PeerInfo>,

    /// Downloaded block headers (height -> header)
    headers: HashMap<u64, BlockHeader>,

    /// Pending block download requests
    pending_blocks: VecDeque<BlockRequest>,

    /// Blocks currently being downloaded (height -> request)
    downloading_blocks: HashMap<u64, BlockRequest>,

    /// Downloaded blocks waiting to be applied (height -> block)
    downloaded_blocks: HashMap<u64, Block>,

    /// Current blockchain height (fully validated)
    current_height: u64,

    /// Current chainwork
    current_chainwork: u64,

    /// Headers for the best chain
    best_chain_headers: Vec<BlockHeader>,
}

impl IbdOrchestrator {
    /// Create a new IBD orchestrator
    pub fn new(current_height: u64, current_chainwork: u64) -> Self {
        Self {
            state: SyncState::Synced,
            peers: HashMap::new(),
            headers: HashMap::new(),
            pending_blocks: VecDeque::new(),
            downloading_blocks: HashMap::new(),
            downloaded_blocks: HashMap::new(),
            current_height,
            current_chainwork,
            best_chain_headers: Vec::new(),
        }
    }

    /// Get current sync state
    pub fn state(&self) -> &SyncState {
        &self.state
    }

    /// Check if node is synced
    pub fn is_synced(&self) -> bool {
        matches!(self.state, SyncState::Synced)
    }

    /// Update peer information
    pub fn update_peer(&mut self, peer_id: PeerId, height: u64, best_hash: [u8; 32], chainwork: u64) {
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            peer.height = height;
            peer.best_hash = best_hash;
            peer.chainwork = chainwork;
            peer.last_seen = Instant::now();
        } else {
            let peer_info = PeerInfo::new(peer_id, height, best_hash, chainwork);
            self.peers.insert(peer_id, peer_info);
        }

        // Check if we need to start syncing
        self.check_sync_needed();
    }

    /// Remove a peer (on disconnect)
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peers.remove(peer_id);

        // Cancel any downloads from this peer
        self.cancel_downloads_from_peer(peer_id);
    }

    /// Check if sync is needed and start if necessary
    fn check_sync_needed(&mut self) {
        // Find best peer
        let best_peer = self.peers.values()
            .filter(|p| p.has_better_chain(self.current_chainwork))
            .max_by_key(|p| p.chainwork);

        if let Some(best_peer) = best_peer {
            if best_peer.height > self.current_height {
                info!(
                    "📥 Starting IBD: current height {} -> target height {} (peer: {})",
                    self.current_height,
                    best_peer.height,
                    best_peer.peer_id
                );

                self.state = SyncState::DownloadingHeaders {
                    headers_height: self.current_height,
                    target_height: best_peer.height,
                };
            }
        }
    }

    /// Process downloaded block headers
    ///
    /// Returns: (accepted_count, best_chain_tip)
    pub fn process_headers(&mut self, headers: Vec<BlockHeader>) -> Result<(usize, Option<[u8; 32]>)> {
        if headers.is_empty() {
            return Ok((0, None));
        }

        let mut accepted = 0;
        let mut tip_hash = None;

        for header in headers {
            let height = header.height;
            let hash = header.hash();

            // Validate header (basic checks)
            // TODO: Full validation including PoW, difficulty, timestamps

            // Store header
            self.headers.insert(height, header.clone());
            accepted += 1;
            tip_hash = Some(hash);

            // Update best chain
            if height > self.current_height {
                self.best_chain_headers.push(header);
            }
        }

        // Transition to block download if we have headers
        if !self.best_chain_headers.is_empty() {
            let headers_height = self.best_chain_headers.last().unwrap().height;
            self.state = SyncState::DownloadingBlocks {
                blocks_height: self.current_height,
                headers_height,
            };

            // Queue blocks for download
            self.queue_block_downloads();
        }

        Ok((accepted, tip_hash))
    }

    /// Queue blocks for download based on headers
    fn queue_block_downloads(&mut self) {
        for header in &self.best_chain_headers {
            if header.height > self.current_height {
                let request = BlockRequest::new(header.height, header.hash());
                self.pending_blocks.push_back(request);
            }
        }

        info!("📋 Queued {} blocks for download", self.pending_blocks.len());
    }

    /// Get next block download requests (up to MAX_PARALLEL_BLOCKS)
    ///
    /// Returns list of (height, hash, peer_id) for blocks to download
    pub fn get_next_download_requests(&mut self) -> Vec<(u64, [u8; 32], PeerId)> {
        let mut requests = Vec::new();

        // Check for timed-out downloads and retry
        self.retry_timed_out_downloads();

        // Get available peers sorted by chainwork
        let mut available_peers: Vec<&PeerInfo> = self.peers.values()
            .filter(|p| p.is_responsive())
            .collect();
        available_peers.sort_by_key(|p| std::cmp::Reverse(p.chainwork));

        if available_peers.is_empty() {
            return requests;
        }

        // Assign pending blocks to peers (round-robin for load balancing)
        let mut peer_index = 0;

        while self.downloading_blocks.len() < MAX_PARALLEL_BLOCKS {
            if let Some(mut request) = self.pending_blocks.pop_front() {
                // Skip if already downloading
                if self.downloading_blocks.contains_key(&request.height) {
                    continue;
                }

                // Assign to next available peer
                if let Some(peer) = available_peers.get(peer_index % available_peers.len()) {
                    request.assign_peer(peer.peer_id);
                    requests.push((request.height, request.expected_hash, peer.peer_id));

                    self.downloading_blocks.insert(request.height, request);
                    peer_index += 1;
                }
            } else {
                break;
            }
        }

        if !requests.is_empty() {
            debug!("📥 Requesting {} blocks from {} peers", requests.len(), available_peers.len());
        }

        requests
    }

    /// Retry timed-out downloads
    fn retry_timed_out_downloads(&mut self) {
        let timed_out: Vec<u64> = self.downloading_blocks.iter()
            .filter(|(_, req)| req.is_timed_out())
            .map(|(height, _)| *height)
            .collect();

        for height in timed_out {
            if let Some(mut request) = self.downloading_blocks.remove(&height) {
                warn!("⏱️  Block {} download timed out, retrying", height);

                // Mark peer as failed
                if let Some(peer_id) = &request.peer_id {
                    if let Some(peer) = self.peers.get_mut(peer_id) {
                        peer.failures += 1;
                    }
                }

                // Retry if not exceeded max attempts
                if request.retries < 3 {
                    request.reset_for_retry();
                    self.pending_blocks.push_front(request);
                } else {
                    warn!("❌ Block {} exceeded max retries, giving up", height);
                }
            }
        }
    }

    /// Cancel downloads from a specific peer
    fn cancel_downloads_from_peer(&mut self, peer_id: &PeerId) {
        let cancelled: Vec<u64> = self.downloading_blocks.iter()
            .filter(|(_, req)| req.peer_id.as_ref() == Some(peer_id))
            .map(|(height, _)| *height)
            .collect();

        for height in cancelled {
            if let Some(mut request) = self.downloading_blocks.remove(&height) {
                warn!("📦 Cancelling block {} download from disconnected peer", height);
                request.reset_for_retry();
                self.pending_blocks.push_front(request);
            }
        }
    }

    /// Process a downloaded block
    pub fn process_downloaded_block(&mut self, block: Block) -> Result<bool> {
        let height = block.header.height;

        // Verify this block was requested
        if let Some(request) = self.downloading_blocks.remove(&height) {
            // Verify hash matches expected
            let block_hash = block.header.hash();
            if block_hash != request.expected_hash {
                warn!("❌ Block {} hash mismatch: expected {:?}, got {:?}",
                    height, request.expected_hash, block_hash);

                // Re-queue for download
                let mut retry_request = request.clone();
                retry_request.reset_for_retry();
                self.pending_blocks.push_front(retry_request);

                return Ok(false);
            }

            // Store downloaded block
            self.downloaded_blocks.insert(height, block);

            // Mark peer as successful
            if let Some(peer_id) = &request.peer_id {
                if let Some(peer) = self.peers.get_mut(peer_id) {
                    peer.last_seen = Instant::now();
                    peer.failures = 0;
                }
            }

            debug!("✅ Downloaded block {} from {:?}", height, request.peer_id);

            // Check if we can apply sequential blocks
            self.try_apply_blocks()?;

            return Ok(true);
        }

        warn!("⚠️  Received unrequested block at height {}", height);
        Ok(false)
    }

    /// Try to apply downloaded blocks sequentially
    fn try_apply_blocks(&mut self) -> Result<()> {
        // Apply blocks in order starting from current_height + 1
        let mut next_height = self.current_height + 1;

        while let Some(_block) = self.downloaded_blocks.get(&next_height) {
            // Block would be applied to blockchain here
            // For now, just track progress

            self.downloaded_blocks.remove(&next_height);
            self.current_height = next_height;
            next_height += 1;

            debug!("✅ Applied block at height {}", self.current_height);
        }

        // Check if sync is complete
        if self.pending_blocks.is_empty()
            && self.downloading_blocks.is_empty()
            && !self.best_chain_headers.is_empty()
            && self.current_height >= self.best_chain_headers.last().unwrap().height
        {
            info!("🎉 IBD complete! Synced to height {}", self.current_height);
            self.state = SyncState::Synced;
            self.best_chain_headers.clear();
        }

        Ok(())
    }

    /// Get sync progress (percentage)
    pub fn sync_progress(&self) -> f64 {
        match &self.state {
            SyncState::Synced => 100.0,
            SyncState::DownloadingHeaders { headers_height, target_height } => {
                if *target_height == 0 {
                    0.0
                } else {
                    (*headers_height as f64 / *target_height as f64) * 50.0 // Headers = 50% of sync
                }
            }
            SyncState::DownloadingBlocks { blocks_height, headers_height } => {
                if *headers_height == 0 {
                    50.0
                } else {
                    50.0 + (*blocks_height as f64 / *headers_height as f64) * 50.0
                }
            }
            SyncState::Validating { current_height, total_blocks } => {
                if *total_blocks == 0 {
                    90.0
                } else {
                    90.0 + (*current_height as f64 / *total_blocks as f64) * 10.0
                }
            }
        }
    }

    /// Get number of peers
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Get number of responsive peers
    pub fn responsive_peer_count(&self) -> usize {
        self.peers.values().filter(|p| p.is_responsive()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ibd_orchestrator_creation() {
        let ibd = IbdOrchestrator::new(0, 0);
        assert!(ibd.is_synced());
        assert_eq!(ibd.peer_count(), 0);
    }

    #[test]
    fn test_peer_management() {
        let mut ibd = IbdOrchestrator::new(0, 0);
        let peer_id = PeerId::random();

        ibd.update_peer(peer_id, 100, [1u8; 32], 1000);
        assert_eq!(ibd.peer_count(), 1);

        ibd.remove_peer(&peer_id);
        assert_eq!(ibd.peer_count(), 0);
    }

    #[test]
    fn test_sync_progress() {
        let ibd = IbdOrchestrator::new(0, 0);
        assert_eq!(ibd.sync_progress(), 100.0);
    }
}
