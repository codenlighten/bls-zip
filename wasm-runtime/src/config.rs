// Runtime configuration for WASM execution
use serde::{Deserialize, Serialize};

/// Configuration for the WASM runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Maximum fuel units per contract call (gas metering)
    pub max_fuel: u64,

    /// Maximum memory pages (64KB each)
    pub max_memory_pages: u32,

    /// Maximum stack size in bytes
    pub max_stack_size: usize,

    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,

    /// Enable compilation cache
    pub enable_cache: bool,

    /// Use pooling allocator for performance
    pub use_pooling_allocator: bool,

    /// Maximum number of instances in pool
    pub max_pooled_instances: u32,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            // 100 million fuel units (adjustable per transaction)
            max_fuel: 100_000_000,

            // 16MB max memory (256 pages * 64KB)
            max_memory_pages: 256,

            // 1MB stack
            max_stack_size: 1024 * 1024,

            // SECURITY FIX: 10 second timeout (was 30s) - prevents infinite loops
            max_execution_time_ms: 10_000,

            // Enable caching for performance
            enable_cache: true,

            // Use pooling for better performance
            use_pooling_allocator: true,

            // Pool up to 100 instances
            max_pooled_instances: 100,
        }
    }
}

impl RuntimeConfig {
    /// Create a config for testing (more permissive)
    pub fn for_testing() -> Self {
        Self {
            max_fuel: 1_000_000_000,
            max_memory_pages: 512,
            max_stack_size: 2 * 1024 * 1024,
            max_execution_time_ms: 60_000,
            enable_cache: false,
            use_pooling_allocator: false,
            max_pooled_instances: 10,
        }
    }

    /// Create a strict config for production
    pub fn for_production() -> Self {
        Self {
            max_fuel: 50_000_000,
            max_memory_pages: 128,
            max_stack_size: 512 * 1024,
            max_execution_time_ms: 10_000,
            enable_cache: true,
            use_pooling_allocator: true,
            max_pooled_instances: 200,
        }
    }
}

/// Execution result with gas accounting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Return value from the contract
    pub return_value: Vec<u8>,

    /// Fuel consumed during execution
    pub fuel_consumed: u64,

    /// Execution time in microseconds
    pub execution_time_us: u64,

    /// Whether the execution was successful
    pub success: bool,

    /// Error message if execution failed
    pub error: Option<String>,
}

impl ExecutionResult {
    pub fn success(return_value: Vec<u8>, fuel_consumed: u64, execution_time_us: u64) -> Self {
        Self {
            return_value,
            fuel_consumed,
            execution_time_us,
            success: true,
            error: None,
        }
    }

    pub fn failure(error: String, fuel_consumed: u64, execution_time_us: u64) -> Self {
        Self {
            return_value: Vec::new(),
            fuel_consumed,
            execution_time_us,
            success: false,
            error: Some(error),
        }
    }
}
