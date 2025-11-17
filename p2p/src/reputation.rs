// Peer Reputation System
// Tracks peer behavior and implements automatic banning for misbehaving peers

use chrono::{DateTime, Duration, Utc};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// Types of violations that can affect peer reputation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Violation {
    /// Peer sent an invalid block
    InvalidBlock,

    /// Peer sent an invalid transaction
    InvalidTransaction,

    /// Peer exceeded rate limits
    RateLimitExceeded,

    /// Peer sent malformed protocol message
    MalformedMessage,

    /// Peer violated protocol rules
    ProtocolViolation,

    /// Peer failed to respond to request
    NoResponse,

    /// Peer sent duplicate data
    DuplicateData,

    /// Peer attempted known attack pattern
    AttackAttempt,
}

impl Violation {
    /// Get the reputation score penalty for this violation
    pub fn severity(&self) -> i32 {
        match self {
            Violation::InvalidBlock => -10,
            Violation::InvalidTransaction => -5,
            Violation::RateLimitExceeded => -2,
            Violation::MalformedMessage => -3,
            Violation::ProtocolViolation => -15,
            Violation::NoResponse => -1,
            Violation::DuplicateData => -2,
            Violation::AttackAttempt => -50, // Severe penalty
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Violation::InvalidBlock => "Sent invalid block",
            Violation::InvalidTransaction => "Sent invalid transaction",
            Violation::RateLimitExceeded => "Exceeded rate limits",
            Violation::MalformedMessage => "Sent malformed message",
            Violation::ProtocolViolation => "Violated protocol rules",
            Violation::NoResponse => "Failed to respond to request",
            Violation::DuplicateData => "Sent duplicate data",
            Violation::AttackAttempt => "Attempted known attack",
        }
    }
}

/// Reputation record for a single peer
#[derive(Debug, Clone)]
pub struct PeerReputation {
    /// Peer identifier
    pub peer_id: PeerId,

    /// Current reputation score (-100 to +100)
    pub reputation_score: i32,

    /// List of recent violations
    pub violations: Vec<ViolationRecord>,

    /// Timestamp of last violation
    pub last_violation: Option<DateTime<Utc>>,

    /// Ban expiration time (if banned)
    pub ban_until: Option<DateTime<Utc>>,

    /// Total number of successful interactions
    pub successful_interactions: u64,

    /// Timestamp when this peer was first seen
    pub first_seen: DateTime<Utc>,

    /// Timestamp of last successful interaction
    pub last_seen: DateTime<Utc>,
}

/// Record of a specific violation
#[derive(Debug, Clone)]
pub struct ViolationRecord {
    pub violation: Violation,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
}

impl PeerReputation {
    /// Create a new reputation record for a peer
    pub fn new(peer_id: PeerId) -> Self {
        let now = Utc::now();
        Self {
            peer_id,
            reputation_score: 0,
            violations: Vec::new(),
            last_violation: None,
            ban_until: None,
            successful_interactions: 0,
            first_seen: now,
            last_seen: now,
        }
    }

    /// Check if this peer is currently banned
    pub fn is_banned(&self) -> bool {
        if let Some(ban_until) = self.ban_until {
            Utc::now() < ban_until
        } else {
            false
        }
    }

    /// Get time remaining on ban
    pub fn ban_time_remaining(&self) -> Option<Duration> {
        if let Some(ban_until) = self.ban_until {
            let now = Utc::now();
            if ban_until > now {
                Some(ban_until - now)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Record a successful interaction
    pub fn record_success(&mut self) {
        self.successful_interactions += 1;
        self.last_seen = Utc::now();

        // Slowly improve reputation with successful interactions
        if self.reputation_score < 100 {
            self.reputation_score += 1;
        }
    }
}

/// Configuration for reputation system
#[derive(Debug, Clone)]
pub struct ReputationConfig {
    /// Reputation score threshold for automatic ban
    pub auto_ban_threshold: i32,

    /// Default ban duration in hours
    pub ban_duration_hours: i64,

    /// Maximum number of violations to track per peer
    pub max_violations_tracked: usize,

    /// Time window for rate limiting violations (seconds)
    pub violation_rate_window_secs: i64,

    /// Maximum violations within time window before extended ban
    pub max_violations_in_window: usize,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            auto_ban_threshold: -50,
            ban_duration_hours: 24,
            max_violations_tracked: 100,
            violation_rate_window_secs: 3600, // 1 hour
            max_violations_in_window: 10,
        }
    }
}

/// Peer reputation manager
pub struct ReputationManager {
    /// Map of peer IDs to their reputation records
    peers: HashMap<PeerId, PeerReputation>,

    /// Configuration
    config: ReputationConfig,
}

impl ReputationManager {
    /// Create a new reputation manager with default config
    pub fn new() -> Self {
        Self::with_config(ReputationConfig::default())
    }

    /// Create a new reputation manager with custom config
    pub fn with_config(config: ReputationConfig) -> Self {
        Self {
            peers: HashMap::new(),
            config,
        }
    }

    /// Get or create reputation record for a peer
    fn get_or_create(&mut self, peer_id: &PeerId) -> &mut PeerReputation {
        self.peers
            .entry(peer_id.clone())
            .or_insert_with(|| PeerReputation::new(peer_id.clone()))
    }

    /// Record a violation for a peer
    pub fn record_violation(
        &mut self,
        peer_id: &PeerId,
        violation: Violation,
        details: Option<String>,
    ) {
        let severity = violation.severity();
        let description = violation.description();
        let max_violations = self.config.max_violations_tracked;
        let auto_ban_threshold = self.config.auto_ban_threshold;
        let ban_duration = self.config.ban_duration_hours;

        let peer = self.get_or_create(peer_id);

        // Add violation to record
        peer.violations.push(ViolationRecord {
            violation: violation.clone(),
            timestamp: Utc::now(),
            details: details.clone(),
        });

        // Trim old violations if needed
        if peer.violations.len() > max_violations {
            peer.violations.remove(0);
        }

        // Update reputation score
        peer.reputation_score += severity;
        peer.last_violation = Some(Utc::now());

        warn!(
            "Peer {} violated protocol: {} (score: {} -> {}, details: {:?})",
            peer_id,
            description,
            peer.reputation_score - severity,
            peer.reputation_score,
            details
        );

        // Check if peer should be banned
        if peer.reputation_score <= auto_ban_threshold {
            self.ban_peer(peer_id, ban_duration);
        }

        // Check for rapid violation pattern
        self.check_violation_rate(peer_id);
    }

    /// Check if peer is violating too frequently
    fn check_violation_rate(&mut self, peer_id: &PeerId) {
        let rate_window = self.config.violation_rate_window_secs;
        let max_violations_in_window = self.config.max_violations_in_window;
        let ban_duration = self.config.ban_duration_hours;

        if let Some(peer) = self.peers.get(peer_id) {
            let now = Utc::now();
            let window_start = now - Duration::seconds(rate_window);

            let recent_violations = peer
                .violations
                .iter()
                .filter(|v| v.timestamp > window_start)
                .count();

            if recent_violations >= max_violations_in_window {
                warn!(
                    "Peer {} has {} violations in {}s window, extending ban",
                    peer_id, recent_violations, rate_window
                );

                // Extended ban for repeated violations
                self.ban_peer(peer_id, ban_duration * 2);
            }
        }
    }

    /// Ban a peer for a specified duration
    pub fn ban_peer(&mut self, peer_id: &PeerId, duration_hours: i64) {
        let peer = self.get_or_create(peer_id);
        let ban_until = Utc::now() + Duration::hours(duration_hours);
        peer.ban_until = Some(ban_until);

        info!(
            "Peer {} banned until {} (duration: {}h, score: {})",
            peer_id,
            ban_until.format("%Y-%m-%d %H:%M:%S"),
            duration_hours,
            peer.reputation_score
        );
    }

    /// Manually unban a peer
    pub fn unban_peer(&mut self, peer_id: &PeerId) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.ban_until = None;
            info!("Peer {} unbanned manually", peer_id);
        }
    }

    /// Check if a peer is banned
    pub fn is_banned(&self, peer_id: &PeerId) -> bool {
        self.peers
            .get(peer_id)
            .map(|p| p.is_banned())
            .unwrap_or(false)
    }

    /// Get reputation score for a peer
    pub fn get_reputation(&self, peer_id: &PeerId) -> i32 {
        self.peers
            .get(peer_id)
            .map(|p| p.reputation_score)
            .unwrap_or(0)
    }

    /// Record a successful interaction
    pub fn record_success(&mut self, peer_id: &PeerId) {
        let peer = self.get_or_create(peer_id);
        peer.record_success();
    }

    /// Get peer reputation details
    pub fn get_peer_details(&self, peer_id: &PeerId) -> Option<&PeerReputation> {
        self.peers.get(peer_id)
    }

    /// Get list of all banned peers
    pub fn get_banned_peers(&self) -> Vec<PeerId> {
        self.peers
            .values()
            .filter(|p| p.is_banned())
            .map(|p| p.peer_id.clone())
            .collect()
    }

    /// Clean up expired bans and old peer records
    pub fn cleanup_expired(&mut self) {
        let now = Utc::now();

        // Remove expired bans
        for peer in self.peers.values_mut() {
            if let Some(ban_until) = peer.ban_until {
                if ban_until <= now {
                    peer.ban_until = None;
                    info!("Ban expired for peer {}", peer.peer_id);
                }
            }
        }

        // Optional: Remove very old peer records that haven't been seen in 30 days
        let cutoff = now - Duration::days(30);
        self.peers.retain(|peer_id, peer| {
            if peer.last_seen < cutoff && peer.reputation_score >= 0 {
                info!("Removing old peer record: {}", peer_id);
                false
            } else {
                true
            }
        });
    }

    /// Get statistics about peer reputation
    pub fn get_statistics(&self) -> ReputationStats {
        let total_peers = self.peers.len();
        let banned_count = self.peers.values().filter(|p| p.is_banned()).count();
        let negative_reputation_count = self
            .peers
            .values()
            .filter(|p| p.reputation_score < 0)
            .count();
        let total_violations: usize = self.peers.values().map(|p| p.violations.len()).sum();

        let avg_reputation = if total_peers > 0 {
            let sum: i32 = self.peers.values().map(|p| p.reputation_score).sum();
            sum as f64 / total_peers as f64
        } else {
            0.0
        };

        ReputationStats {
            total_peers,
            banned_count,
            negative_reputation_count,
            total_violations,
            avg_reputation,
        }
    }
}

impl Default for ReputationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about peer reputation system
#[derive(Debug, Clone)]
pub struct ReputationStats {
    pub total_peers: usize,
    pub banned_count: usize,
    pub negative_reputation_count: usize,
    pub total_violations: usize,
    pub avg_reputation: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_peer_id() -> PeerId {
        PeerId::random()
    }

    #[test]
    fn test_new_peer_has_zero_reputation() {
        let peer_id = create_test_peer_id();
        let peer = PeerReputation::new(peer_id.clone());

        assert_eq!(peer.reputation_score, 0);
        assert!(!peer.is_banned());
        assert_eq!(peer.violations.len(), 0);
    }

    #[test]
    fn test_violation_decreases_reputation() {
        let mut manager = ReputationManager::new();
        let peer_id = create_test_peer_id();

        manager.record_violation(&peer_id, Violation::InvalidTransaction, None);

        assert_eq!(manager.get_reputation(&peer_id), -5);
    }

    #[test]
    fn test_auto_ban_on_low_reputation() {
        let mut manager = ReputationManager::new();
        let peer_id = create_test_peer_id();

        // Record enough violations to trigger auto-ban
        for _ in 0..10 {
            manager.record_violation(&peer_id, Violation::InvalidBlock, None);
        }

        assert!(manager.is_banned(&peer_id));
    }

    #[test]
    fn test_successful_interactions_improve_reputation() {
        let mut manager = ReputationManager::new();
        let peer_id = create_test_peer_id();

        // Record a violation first
        manager.record_violation(&peer_id, Violation::InvalidTransaction, None);
        assert_eq!(manager.get_reputation(&peer_id), -5);

        // Record successful interactions
        for _ in 0..10 {
            manager.record_success(&peer_id);
        }

        assert_eq!(manager.get_reputation(&peer_id), 5);
    }

    #[test]
    fn test_attack_attempt_severe_penalty() {
        let mut manager = ReputationManager::new();
        let peer_id = create_test_peer_id();

        manager.record_violation(
            &peer_id,
            Violation::AttackAttempt,
            Some("Attempted double-spend".into()),
        );

        assert_eq!(manager.get_reputation(&peer_id), -50);
        assert!(manager.is_banned(&peer_id));
    }

    #[test]
    fn test_manual_unban() {
        let mut manager = ReputationManager::new();
        let peer_id = create_test_peer_id();

        manager.ban_peer(&peer_id, 24);
        assert!(manager.is_banned(&peer_id));

        manager.unban_peer(&peer_id);
        assert!(!manager.is_banned(&peer_id));
    }
}
