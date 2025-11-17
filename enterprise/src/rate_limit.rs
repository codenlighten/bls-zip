// Rate Limiting Module
// Provides IP-based and user-based rate limiting for API endpoints

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{EnterpriseError, Result};

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per IP address per window
    pub max_requests_per_ip: usize,

    /// Maximum requests per user per window
    pub max_requests_per_user: usize,

    /// Time window for rate limiting (seconds)
    pub window_secs: u64,

    /// Enable burst protection
    pub enable_burst_protection: bool,

    /// Burst allowance (additional requests allowed in burst)
    pub burst_size: usize,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_ip: 100,        // 100 requests per minute per IP
            max_requests_per_user: 200,      // 200 requests per minute per user
            window_secs: 60,                 // 1 minute window
            enable_burst_protection: true,
            burst_size: 10,                  // Allow 10 extra requests in burst
        }
    }
}

/// Rate limit entry tracking requests
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// Number of requests in current window
    request_count: usize,

    /// Window start time
    window_start: Instant,

    /// Burst tokens available
    burst_tokens: usize,

    /// Last request time
    last_request: Instant,
}

impl RateLimitEntry {
    fn new(burst_size: usize) -> Self {
        let now = Instant::now();
        Self {
            request_count: 0,
            window_start: now,
            burst_tokens: burst_size,
            last_request: now,
        }
    }

    /// Check if rate limit is exceeded
    fn is_exceeded(&self, max_requests: usize) -> bool {
        self.request_count >= max_requests
    }

    /// Reset window if expired
    fn reset_if_expired(&mut self, window_duration: Duration, burst_size: usize) {
        let now = Instant::now();
        if now.duration_since(self.window_start) >= window_duration {
            self.request_count = 0;
            self.window_start = now;
            self.burst_tokens = burst_size; // Refill burst tokens
        }
    }

    /// Increment request count
    fn increment(&mut self) {
        self.request_count += 1;
        self.last_request = Instant::now();
    }

    /// Try to consume a burst token
    fn try_consume_burst(&mut self) -> bool {
        if self.burst_tokens > 0 {
            self.burst_tokens -= 1;
            self.last_request = Instant::now();
            true
        } else {
            false
        }
    }

    /// Refill burst tokens gradually
    fn refill_burst_tokens(&mut self, max_burst: usize) {
        let now = Instant::now();
        let time_since_last = now.duration_since(self.last_request);

        // Refill 1 token per second
        let tokens_to_add = time_since_last.as_secs() as usize;
        self.burst_tokens = (self.burst_tokens + tokens_to_add).min(max_burst);
    }
}

/// Rate limiter
pub struct RateLimiter {
    /// Configuration
    config: RateLimitConfig,

    /// IP-based rate limits
    ip_limits: Arc<RwLock<HashMap<IpAddr, RateLimitEntry>>>,

    /// User-based rate limits
    user_limits: Arc<RwLock<HashMap<Uuid, RateLimitEntry>>>,
}

impl RateLimiter {
    /// Create a new rate limiter with default config
    pub fn new() -> Self {
        Self::with_config(RateLimitConfig::default())
    }

    /// Create a new rate limiter with custom config
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            config,
            ip_limits: Arc::new(RwLock::new(HashMap::new())),
            user_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if IP is rate limited
    pub async fn check_ip(&self, ip: IpAddr) -> Result<()> {
        let mut limits = self.ip_limits.write().await;

        let entry = limits
            .entry(ip)
            .or_insert_with(|| RateLimitEntry::new(self.config.burst_size));

        // Reset window if expired
        entry.reset_if_expired(
            Duration::from_secs(self.config.window_secs),
            self.config.burst_size,
        );

        // Check rate limit
        if entry.is_exceeded(self.config.max_requests_per_ip) {
            // Try burst protection if enabled
            if self.config.enable_burst_protection {
                entry.refill_burst_tokens(self.config.burst_size);
                if entry.try_consume_burst() {
                    return Ok(());
                }
            }

            return Err(EnterpriseError::InvalidInput(format!(
                "Rate limit exceeded for IP {}. Max {} requests per {} seconds",
                ip, self.config.max_requests_per_ip, self.config.window_secs
            )));
        }

        entry.increment();
        Ok(())
    }

    /// Check if user is rate limited
    pub async fn check_user(&self, user_id: Uuid) -> Result<()> {
        let mut limits = self.user_limits.write().await;

        let entry = limits
            .entry(user_id)
            .or_insert_with(|| RateLimitEntry::new(self.config.burst_size));

        // Reset window if expired
        entry.reset_if_expired(
            Duration::from_secs(self.config.window_secs),
            self.config.burst_size,
        );

        // Check rate limit
        if entry.is_exceeded(self.config.max_requests_per_user) {
            // Try burst protection if enabled
            if self.config.enable_burst_protection {
                entry.refill_burst_tokens(self.config.burst_size);
                if entry.try_consume_burst() {
                    return Ok(());
                }
            }

            return Err(EnterpriseError::InvalidInput(format!(
                "Rate limit exceeded for user {}. Max {} requests per {} seconds",
                user_id, self.config.max_requests_per_user, self.config.window_secs
            )));
        }

        entry.increment();
        Ok(())
    }

    /// Check both IP and user rate limits
    pub async fn check_combined(&self, ip: IpAddr, user_id: Option<Uuid>) -> Result<()> {
        // Always check IP
        self.check_ip(ip).await?;

        // Check user if authenticated
        if let Some(uid) = user_id {
            self.check_user(uid).await?;
        }

        Ok(())
    }

    /// Clean up old entries (should be called periodically)
    pub async fn cleanup_expired(&self) {
        let window_duration = Duration::from_secs(self.config.window_secs);
        let cleanup_threshold = window_duration * 2; // Keep entries for 2x window duration

        // Cleanup IP limits
        {
            let mut limits = self.ip_limits.write().await;
            limits.retain(|_, entry| {
                Instant::now().duration_since(entry.last_request) < cleanup_threshold
            });
        }

        // Cleanup user limits
        {
            let mut limits = self.user_limits.write().await;
            limits.retain(|_, entry| {
                Instant::now().duration_since(entry.last_request) < cleanup_threshold
            });
        }
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> RateLimitStats {
        let ip_count = self.ip_limits.read().await.len();
        let user_count = self.user_limits.read().await.len();

        RateLimitStats {
            tracked_ips: ip_count,
            tracked_users: user_count,
            config: self.config.clone(),
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limit statistics
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    pub tracked_ips: usize,
    pub tracked_users: usize,
    pub config: RateLimitConfig,
}

/// Helper function to start periodic cleanup task
pub fn start_cleanup_task(limiter: Arc<RateLimiter>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
        loop {
            interval.tick().await;
            limiter.cleanup_expired().await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_ip_rate_limit() {
        let config = RateLimitConfig {
            max_requests_per_ip: 5,
            max_requests_per_user: 10,
            window_secs: 60,
            enable_burst_protection: false,
            burst_size: 0,
        };

        let limiter = RateLimiter::with_config(config);
        let ip = IpAddr::from_str("127.0.0.1").unwrap();

        // First 5 requests should succeed
        for _ in 0..5 {
            assert!(limiter.check_ip(ip).await.is_ok());
        }

        // 6th request should fail
        assert!(limiter.check_ip(ip).await.is_err());
    }

    #[tokio::test]
    async fn test_user_rate_limit() {
        let config = RateLimitConfig {
            max_requests_per_ip: 100,
            max_requests_per_user: 3,
            window_secs: 60,
            enable_burst_protection: false,
            burst_size: 0,
        };

        let limiter = RateLimiter::with_config(config);
        let user_id = Uuid::new_v4();

        // First 3 requests should succeed
        for _ in 0..3 {
            assert!(limiter.check_user(user_id).await.is_ok());
        }

        // 4th request should fail
        assert!(limiter.check_user(user_id).await.is_err());
    }

    #[tokio::test]
    async fn test_burst_protection() {
        let config = RateLimitConfig {
            max_requests_per_ip: 5,
            max_requests_per_user: 10,
            window_secs: 60,
            enable_burst_protection: true,
            burst_size: 3,
        };

        let limiter = RateLimiter::with_config(config);
        let ip = IpAddr::from_str("127.0.0.1").unwrap();

        // First 5 requests (normal limit)
        for _ in 0..5 {
            assert!(limiter.check_ip(ip).await.is_ok());
        }

        // Next 3 requests should succeed via burst tokens
        for _ in 0..3 {
            assert!(limiter.check_ip(ip).await.is_ok());
        }

        // 9th request should fail (all burst tokens consumed)
        assert!(limiter.check_ip(ip).await.is_err());
    }

    #[tokio::test]
    async fn test_cleanup() {
        let limiter = RateLimiter::new();
        let ip = IpAddr::from_str("127.0.0.1").unwrap();

        limiter.check_ip(ip).await.unwrap();

        let stats = limiter.get_stats().await;
        assert_eq!(stats.tracked_ips, 1);

        limiter.cleanup_expired().await;

        // Entry should still be there (not expired yet)
        let stats = limiter.get_stats().await;
        assert_eq!(stats.tracked_ips, 1);
    }
}
