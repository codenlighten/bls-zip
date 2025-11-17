// Node configuration
use crate::mempool::MempoolConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Network configuration
    pub network: NetworkConfig,

    /// Consensus configuration
    pub consensus: ConsensusConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// RPC configuration
    pub rpc: RpcConfig,

    /// Mempool configuration
    pub mempool: MempoolConfig,

    /// HIGH PRIORITY FIX: Mining configuration
    #[serde(default)]
    pub mining: MiningConfig,

    /// HIGH PRIORITY FIX: Security configuration
    #[serde(default)]
    pub security: SecurityConfig,

    /// HIGH PRIORITY FIX: Operational configuration
    #[serde(default)]
    pub operational: OperationalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// P2P listen address
    pub listen_addr: String,

    /// Bootstrap nodes
    pub bootnodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Target block time in seconds
    pub target_block_time_secs: u64,

    /// Difficulty adjustment interval in blocks
    pub difficulty_adjustment_interval: u64,

    /// Maximum adjustment factor
    pub max_adjustment_factor: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database path
    pub database_path: String,

    /// Cache size in MB
    pub cache_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    /// HTTP address
    pub http_addr: String,

    /// WebSocket address
    pub ws_addr: String,

    /// CORS allowed origins
    pub cors_allowed_origins: Vec<String>,
}

impl NodeConfig {
    /// Load configuration from file
    pub fn from_file(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Development configuration
    pub fn development() -> Self {
        Self {
            network: NetworkConfig {
                listen_addr: "/ip4/127.0.0.1/tcp/30333".to_string(),
                bootnodes: vec![],
            },
            consensus: ConsensusConfig {
                target_block_time_secs: 300,
                difficulty_adjustment_interval: 1008,
                max_adjustment_factor: 4,
            },
            storage: StorageConfig {
                database_path: "./data/db".to_string(),
                cache_size_mb: 128,
            },
            rpc: RpcConfig {
                http_addr: "127.0.0.1:9933".to_string(),
                ws_addr: "127.0.0.1:9944".to_string(),
                cors_allowed_origins: vec!["*".to_string()],
            },
            mempool: MempoolConfig::default(),
            mining: MiningConfig::default(),
            security: SecurityConfig::default(),
            operational: OperationalConfig::default(),
        }
    }

    pub fn mempool_config(&self) -> MempoolConfig {
        self.mempool.clone()
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                listen_addr: "/ip4/0.0.0.0/tcp/30333".to_string(),
                bootnodes: vec![],
            },
            consensus: ConsensusConfig {
                target_block_time_secs: 300,
                difficulty_adjustment_interval: 1008,
                max_adjustment_factor: 4,
            },
            storage: StorageConfig {
                database_path: "/var/lib/boundless/db".to_string(),
                cache_size_mb: 2048,
            },
            rpc: RpcConfig {
                http_addr: "127.0.0.1:9933".to_string(),
                ws_addr: "127.0.0.1:9944".to_string(),
                cors_allowed_origins: vec![],
            },
            mempool: MempoolConfig::default(),
            mining: MiningConfig::default(),
            security: SecurityConfig::default(),
            operational: OperationalConfig::default(),
        }
    }
}

/// HIGH PRIORITY FIX: Mining configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningConfig {
    /// Enable mining
    pub enabled: bool,

    /// Number of mining threads
    pub threads: usize,

    /// Coinbase reward address (hex-encoded 32 bytes)
    pub coinbase_address: Option<String>,
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threads: num_cpus::get(),
            coinbase_address: None,
        }
    }
}

/// HIGH PRIORITY FIX: Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable TLS for RPC
    pub enable_tls: bool,

    /// Maximum RPC request size in bytes
    pub max_request_size_bytes: usize,

    /// Rate limit: requests per minute per IP
    pub rate_limit_per_minute: u32,

    /// Enable authentication for RPC
    pub require_authentication: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_tls: false, // Should be true in production
            max_request_size_bytes: 1_000_000, // 1MB
            rate_limit_per_minute: 60,
            require_authentication: false, // Should be true in production
        }
    }
}

/// HIGH PRIORITY FIX: Operational configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalConfig {
    /// Enable metrics endpoint
    pub enable_metrics: bool,

    /// Metrics endpoint address
    pub metrics_addr: String,

    /// Enable health check endpoint
    pub enable_health_check: bool,

    /// Graceful shutdown timeout in seconds
    pub shutdown_timeout_secs: u64,

    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,

    /// Enable structured logging (JSON format)
    pub structured_logging: bool,

    /// HIGH PRIORITY FIX: Checkpoint interval in blocks
    /// Checkpoints prevent deep reorganizations and 51% attacks
    /// Set to 0 to disable auto-checkpointing (not recommended)
    pub checkpoint_interval: u64,
}

impl Default for OperationalConfig {
    fn default() -> Self {
        Self {
            enable_metrics: false,
            metrics_addr: "127.0.0.1:9615".to_string(),
            enable_health_check: true,
            shutdown_timeout_secs: 10,
            log_level: "info".to_string(),
            structured_logging: false,
            checkpoint_interval: 1000, // Checkpoint every 1000 blocks
        }
    }
}
