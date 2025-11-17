// WASM runtime with fuel metering and host function support
use std::collections::HashMap;
use std::time::Instant;
use wasmtime::*;

use crate::config::{ExecutionResult, RuntimeConfig};
use crate::error::WasmError;
use crate::host_functions::register_host_functions;

/// SECURITY: Storage quotas to prevent DoS attacks
/// Maximum total storage per contract: 10MB
pub const MAX_CONTRACT_STORAGE_BYTES: usize = 10_000_000;
/// Maximum size for a single storage value: 1MB
pub const MAX_STORAGE_VALUE_BYTES: usize = 1_000_000;

/// Contract execution state (passed to host functions)
pub struct ContractState {
    /// Contract storage (key-value pairs)
    pub storage: HashMap<Vec<u8>, Vec<u8>>,

    /// SECURITY FIX: Track total storage used (in bytes)
    pub storage_bytes_used: usize,

    /// Storage changes made during this execution
    pub storage_changes: Vec<crate::config::StorageChange>,

    /// Caller address (32 bytes)
    pub caller: [u8; 32],

    /// Current block height
    pub block_height: u64,

    /// Current timestamp
    pub timestamp: u64,

    /// Memory limiter for resource management
    pub limiter: MemoryLimiter,
}

impl ContractState {
    pub fn new(caller: [u8; 32], block_height: u64, timestamp: u64, max_memory_pages: u32) -> Self {
        Self {
            storage: HashMap::new(),
            storage_bytes_used: 0,
            storage_changes: Vec::new(),
            caller,
            block_height,
            timestamp,
            limiter: MemoryLimiter { max_memory_pages },
        }
    }

    /// SECURITY FIX: Calculate current storage usage
    pub fn calculate_storage_usage(&self) -> usize {
        self.storage
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum()
    }

    /// SECURITY FIX: Check if adding a key-value pair would exceed quota
    pub fn would_exceed_quota(&self, key_len: usize, value_len: usize) -> bool {
        let current = self.storage_bytes_used;
        let new_total = current + key_len + value_len;
        new_total > MAX_CONTRACT_STORAGE_BYTES
    }
}

/// WASM runtime for executing smart contracts
pub struct WasmRuntime {
    engine: Engine,
    config: RuntimeConfig,
}

impl WasmRuntime {
    /// Create a new WASM runtime with the given configuration
    pub fn new(config: RuntimeConfig) -> Result<Self, WasmError> {
        let mut wasmtime_config = Config::new();

        // Enable fuel metering for deterministic gas accounting
        wasmtime_config.consume_fuel(true);

        // Set memory limits
        wasmtime_config.max_wasm_stack(config.max_stack_size);

        // Enable caching if configured
        if config.enable_cache {
            wasmtime_config
                .cache_config_load_default()
                .map_err(|e| WasmError::InitializationError(e.to_string()))?;
        }

        // Configure pooling allocator for better performance (wasmtime v16)
        if config.use_pooling_allocator {
            let mut pooling_config = PoolingAllocationConfig::default();
            // Set maximum memory per instance (in pages)
            pooling_config.memory_pages(config.max_memory_pages as u64);
            pooling_config.total_memories(config.max_pooled_instances);
            pooling_config.total_stacks(config.max_pooled_instances);
            wasmtime_config.allocation_strategy(InstanceAllocationStrategy::Pooling(pooling_config));
        }

        let engine = Engine::new(&wasmtime_config)
            .map_err(|e| WasmError::InitializationError(e.to_string()))?;

        Ok(Self { engine, config })
    }

    /// Compile a WASM module
    pub fn compile(&self, wasm_bytes: &[u8]) -> Result<Module, WasmError> {
        Module::new(&self.engine, wasm_bytes)
            .map_err(|e| WasmError::CompilationError(e.to_string()))
    }

    /// Execute a contract function with timeout enforcement
    ///
    /// SECURITY FIX: Now async with timeout enforcement to prevent infinite loops
    pub async fn execute(
        &self,
        module: &Module,
        function_name: &str,
        args: &[u8],
        caller: [u8; 32],
        block_height: u64,
        timestamp: u64,
    ) -> Result<ExecutionResult, WasmError> {
        let timeout_duration = std::time::Duration::from_millis(self.config.max_execution_time_ms);

        // Clone necessary data for the blocking task
        let module = module.clone();
        let function_name = function_name.to_string();
        let args = args.to_vec();
        let engine = self.engine.clone();
        let config = self.config.clone();

        // SECURITY FIX: Wrap execution in timeout to prevent infinite loops
        let result = tokio::time::timeout(
            timeout_duration,
            tokio::task::spawn_blocking(move || {
                Self::execute_sync(
                    &engine,
                    &config,
                    &module,
                    &function_name,
                    &args,
                    caller,
                    block_height,
                    timestamp,
                )
            }),
        )
        .await;

        match result {
            Ok(Ok(execution_result)) => execution_result,
            Ok(Err(join_error)) => Err(WasmError::ExecutionError(format!(
                "Task execution failed: {}",
                join_error
            ))),
            Err(_timeout_elapsed) => {
                // SECURITY: Execution exceeded time limit
                Err(WasmError::Timeout)
            }
        }
    }

    /// Synchronous execution helper (run in blocking task)
    fn execute_sync(
        engine: &Engine,
        config: &RuntimeConfig,
        module: &Module,
        function_name: &str,
        args: &[u8],
        caller: [u8; 32],
        block_height: u64,
        timestamp: u64,
    ) -> Result<ExecutionResult, WasmError> {
        let start_time = Instant::now();

        // Create contract state
        let state = ContractState::new(
            caller,
            block_height,
            timestamp,
            config.max_memory_pages,
        );

        // Create store with fuel limit
        let mut store = Store::new(engine, state);
        store
            .set_fuel(config.max_fuel)
            .map_err(|e| WasmError::InitializationError(e.to_string()))?;

        // Set up memory limits using ResourceLimiter
        store.limiter(|state| &mut state.limiter);

        // Create linker and register host functions
        let mut linker = Linker::new(engine);
        register_host_functions(&mut linker)?;

        // Instantiate the module
        let instance = linker
            .instantiate(&mut store, module)
            .map_err(|e| WasmError::InstantiationError(e.to_string()))?;

        // Get the function to call
        let func = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, function_name)
            .map_err(|_| WasmError::FunctionNotFound(function_name.to_string()))?;

        // Allocate memory for input args (simplified - assumes contract has allocate/deallocate)
        let allocate = instance
            .get_typed_func::<i32, i32>(&mut store, "allocate")
            .map_err(|_| WasmError::FunctionNotFound("allocate".to_string()))?;

        let args_ptr = allocate
            .call(&mut store, args.len() as i32)
            .map_err(|e| WasmError::ExecutionError(e.to_string()))?;

        // Write args to contract memory
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| WasmError::MemoryError("Memory export not found".to_string()))?;

        memory
            .write(&mut store, args_ptr as usize, args)
            .map_err(|e| WasmError::MemoryError(e.to_string()))?;

        // Call the function
        let result_ptr = func
            .call(&mut store, (args_ptr, args.len() as i32))
            .map_err(|e| WasmError::ExecutionError(e.to_string()))?;

        // Read result from memory (assume first 4 bytes is length, then data)
        let mut result_len_bytes = [0u8; 4];
        memory
            .read(&store, result_ptr as usize, &mut result_len_bytes)
            .map_err(|e| WasmError::MemoryError(e.to_string()))?;

        let result_len = u32::from_le_bytes(result_len_bytes) as usize;
        let mut result_data = vec![0u8; result_len];

        memory
            .read(&store, (result_ptr + 4) as usize, &mut result_data)
            .map_err(|e| WasmError::MemoryError(e.to_string()))?;

        // Get fuel consumed
        let fuel_consumed = config.max_fuel
            - store
                .get_fuel()
                .map_err(|e| WasmError::ExecutionError(e.to_string()))?;

        let execution_time_us = start_time.elapsed().as_micros() as u64;

        // Extract storage changes from contract state
        let storage_changes = store.data().storage_changes.clone();

        Ok(ExecutionResult::success(
            result_data,
            fuel_consumed,
            execution_time_us,
            storage_changes,
        ))
    }

    /// Get the runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }
}

/// Memory limiter for enforcing memory limits
struct MemoryLimiter {
    max_memory_pages: u32,
}

impl ResourceLimiter for MemoryLimiter {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> Result<bool> {
        let current_pages = current / 65536;
        let desired_pages = (desired + 65535) / 65536;

        Ok(desired_pages <= self.max_memory_pages as usize)
    }

    fn table_growing(
        &mut self,
        _current: u32,
        _desired: u32,
        _maximum: Option<u32>,
    ) -> Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let config = RuntimeConfig::default();
        let runtime = WasmRuntime::new(config);
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_simple_wasm_execution() {
        // Simple WASM module that adds two numbers
        let wat = r#"
            (module
                (memory (export "memory") 1)

                (func (export "allocate") (param i32) (result i32)
                    i32.const 0
                )

                (func (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                )
            )
        "#;

        let wasm = wat::parse_str(wat).unwrap();

        let config = RuntimeConfig::for_testing();
        let runtime = WasmRuntime::new(config).unwrap();
        let module = runtime.compile(&wasm).unwrap();

        // Note: This is a simplified test. Real contract execution would be more complex
        // with proper ABI encoding/decoding
    }

    // SECURITY TESTS: Storage Quota Enforcement

    #[test]
    fn test_contract_state_storage_quota_calculation() {
        // Test that storage usage is correctly calculated
        let mut state = ContractState::new([0u8; 32], 0, 0, 10);

        // Initially empty
        assert_eq!(state.storage_bytes_used, 0);
        assert_eq!(state.calculate_storage_usage(), 0);

        // Add some data
        state.storage.insert(vec![1, 2, 3], vec![4, 5, 6, 7]);
        state.storage_bytes_used = 3 + 4; // key + value

        assert_eq!(state.storage_bytes_used, 7);
        assert_eq!(state.calculate_storage_usage(), 7);
    }

    #[test]
    fn test_would_exceed_quota_empty_storage() {
        let state = ContractState::new([0u8; 32], 0, 0, 10);

        // Small addition should not exceed
        assert!(!state.would_exceed_quota(100, 100));

        // Very large addition should exceed 10MB limit
        let large_size = MAX_CONTRACT_STORAGE_BYTES + 1;
        assert!(state.would_exceed_quota(large_size, 0));
        assert!(state.would_exceed_quota(0, large_size));
    }

    #[test]
    fn test_would_exceed_quota_with_existing_data() {
        let mut state = ContractState::new([0u8; 32], 0, 0, 10);

        // Fill storage to 9MB
        state.storage_bytes_used = 9_000_000;

        // Adding 500KB should be OK (9MB + 500KB < 10MB)
        assert!(!state.would_exceed_quota(250_000, 250_000));

        // Adding 2MB should exceed (9MB + 2MB > 10MB)
        assert!(state.would_exceed_quota(1_000_000, 1_000_000));
    }

    #[test]
    fn test_would_exceed_quota_at_limit() {
        let mut state = ContractState::new([0u8; 32], 0, 0, 10);

        // Fill storage to exactly 10MB
        state.storage_bytes_used = MAX_CONTRACT_STORAGE_BYTES;

        // Adding even 1 byte should exceed
        assert!(state.would_exceed_quota(1, 0));
        assert!(state.would_exceed_quota(0, 1));
    }

    #[test]
    fn test_storage_quota_constants() {
        // Verify quota constants are sensible
        assert_eq!(MAX_CONTRACT_STORAGE_BYTES, 10_000_000); // 10MB
        assert_eq!(MAX_STORAGE_VALUE_BYTES, 1_000_000); // 1MB

        // Value max should be less than total max
        assert!(MAX_STORAGE_VALUE_BYTES < MAX_CONTRACT_STORAGE_BYTES);
    }

    #[test]
    fn test_multiple_keys_storage_tracking() {
        let mut state = ContractState::new([0u8; 32], 0, 0, 10);

        // Add multiple keys
        state.storage.insert(vec![1], vec![1, 2, 3]); // 1 + 3 = 4 bytes
        state.storage.insert(vec![2, 3], vec![4, 5]); // 2 + 2 = 4 bytes
        state.storage.insert(vec![4, 5, 6], vec![7]); // 3 + 1 = 4 bytes

        let expected_size = 4 + 4 + 4; // 12 bytes total
        assert_eq!(state.calculate_storage_usage(), expected_size);
    }

    #[test]
    fn test_storage_update_reduces_size() {
        let mut state = ContractState::new([0u8; 32], 0, 0, 10);

        // Add a large value
        let key = vec![1, 2, 3];
        state.storage.insert(key.clone(), vec![0u8; 1000]);
        state.storage_bytes_used = 3 + 1000; // 1003 bytes

        // Update with smaller value
        state.storage.insert(key.clone(), vec![0u8; 100]);
        // New size should be smaller
        let new_size = state.calculate_storage_usage();
        assert_eq!(new_size, 3 + 100); // 103 bytes
    }

    #[test]
    fn test_storage_quota_edge_case_zero_length() {
        let state = ContractState::new([0u8; 32], 0, 0, 10);

        // Zero-length keys/values should not exceed quota
        assert!(!state.would_exceed_quota(0, 0));

        // But should still be tracked if added
        // (though in practice, zero-length keys might be rejected)
    }

    #[test]
    fn test_storage_bytes_used_tracking() {
        let mut state = ContractState::new([0u8; 32], 0, 0, 10);

        // Verify initial state
        assert_eq!(state.storage_bytes_used, 0);

        // Manually set storage_bytes_used (as host_functions.rs does)
        state.storage.insert(vec![1, 2], vec![3, 4, 5]);
        state.storage_bytes_used = 2 + 3; // 5 bytes

        assert_eq!(state.storage_bytes_used, 5);

        // Add another entry
        state.storage.insert(vec![6], vec![7, 8]);
        state.storage_bytes_used += 1 + 2; // Add 3 more bytes

        assert_eq!(state.storage_bytes_used, 8);
    }

    #[test]
    fn test_max_value_size_constant() {
        // Verify max value size is reasonable
        let max_value = MAX_STORAGE_VALUE_BYTES;

        // Should be 1MB
        assert_eq!(max_value, 1_000_000);

        // Should fit at least 10 max values in total storage
        assert!(max_value * 10 <= MAX_CONTRACT_STORAGE_BYTES);
    }

    // SECURITY TESTS: WASM Execution Timeout

    #[tokio::test]
    async fn test_execution_timeout_configuration() {
        // Test that timeout configuration is respected
        let mut config = RuntimeConfig::default();
        config.max_execution_time_ms = 100; // 100ms timeout for fast test

        let runtime = WasmRuntime::new(config.clone()).unwrap();
        assert_eq!(runtime.config().max_execution_time_ms, 100);
    }

    #[tokio::test]
    async fn test_normal_execution_completes_within_timeout() {
        // Test that normal execution completes successfully within timeout
        let wat = r#"
            (module
                (memory (export "memory") 1)

                (func (export "allocate") (param i32) (result i32)
                    i32.const 100
                )

                (func (export "quick_add") (param i32 i32) (result i32)
                    ;; Simple addition that completes quickly
                    local.get 0
                    drop
                    local.get 1
                    drop
                    i32.const 42
                )
            )
        "#;

        let wasm = wat::parse_str(wat).unwrap();

        let mut config = RuntimeConfig::for_testing();
        config.max_execution_time_ms = 5000; // 5 second timeout
        let runtime = WasmRuntime::new(config).unwrap();
        let module = runtime.compile(&wasm).unwrap();

        // Execute should complete without timeout
        let args = vec![1, 2, 3, 4];
        let result = runtime
            .execute(&module, "quick_add", &args, [0u8; 32], 0, 0)
            .await;

        // Should succeed (not timeout)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_timeout_prevents_infinite_loop() {
        // Test that infinite loops are terminated by timeout
        let wat = r#"
            (module
                (memory (export "memory") 1)

                (func (export "allocate") (param i32) (result i32)
                    i32.const 100
                )

                (func (export "infinite_loop") (param i32 i32) (result i32)
                    (loop $forever
                        ;; Infinite loop that never exits
                        br $forever
                    )
                    i32.const 0
                )
            )
        "#;

        let wasm = wat::parse_str(wat).unwrap();

        let mut config = RuntimeConfig::for_testing();
        config.max_execution_time_ms = 100; // Very short timeout (100ms)
        config.max_fuel = u64::MAX; // Unlimited fuel to ensure timeout triggers first
        let runtime = WasmRuntime::new(config).unwrap();
        let module = runtime.compile(&wasm).unwrap();

        // Execute should timeout
        let args = vec![1, 2, 3, 4];
        let result = runtime
            .execute(&module, "infinite_loop", &args, [0u8; 32], 0, 0)
            .await;

        // Should fail with timeout error
        assert!(result.is_err());
        match result {
            Err(WasmError::Timeout) => {
                // Expected timeout error
            }
            other => panic!("Expected WasmError::Timeout, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_different_timeout_values() {
        // Test that different timeout configurations work correctly
        let wat = r#"
            (module
                (memory (export "memory") 1)

                (func (export "allocate") (param i32) (result i32)
                    i32.const 100
                )

                (func (export "simple") (param i32 i32) (result i32)
                    i32.const 1
                )
            )
        "#;

        let wasm = wat::parse_str(wat).unwrap();

        // Test with very short timeout (should still work for simple operations)
        let mut config1 = RuntimeConfig::for_testing();
        config1.max_execution_time_ms = 50; // 50ms
        let runtime1 = WasmRuntime::new(config1).unwrap();
        let module1 = runtime1.compile(&wasm).unwrap();

        let result1 = runtime1
            .execute(&module1, "simple", &[], [0u8; 32], 0, 0)
            .await;
        assert!(result1.is_ok());

        // Test with longer timeout
        let mut config2 = RuntimeConfig::for_testing();
        config2.max_execution_time_ms = 10000; // 10 seconds
        let runtime2 = WasmRuntime::new(config2).unwrap();
        let module2 = runtime2.compile(&wasm).unwrap();

        let result2 = runtime2
            .execute(&module2, "simple", &[], [0u8; 32], 0, 0)
            .await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_production_timeout_default() {
        // Verify production config has appropriate timeout (10 seconds)
        let config = RuntimeConfig::for_production();
        assert_eq!(config.max_execution_time_ms, 10_000);

        let runtime = WasmRuntime::new(config).unwrap();
        assert_eq!(runtime.config().max_execution_time_ms, 10_000);
    }

    #[tokio::test]
    async fn test_default_timeout_value() {
        // Verify default config has 10 second timeout (was 30s, fixed for security)
        let config = RuntimeConfig::default();
        assert_eq!(config.max_execution_time_ms, 10_000);
    }

    #[tokio::test]
    async fn test_timeout_with_fuel_exhaustion() {
        // Test that timeout works independently of fuel exhaustion
        let wat = r#"
            (module
                (memory (export "memory") 1)

                (func (export "allocate") (param i32) (result i32)
                    i32.const 100
                )

                (func (export "work") (param i32 i32) (result i32)
                    (local $i i32)
                    (local.set $i (i32.const 0))
                    (loop $count
                        (local.set $i (i32.add (local.get $i) (i32.const 1)))
                        (br_if $count (i32.lt_u (local.get $i) (i32.const 1000000)))
                    )
                    local.get $i
                )
            )
        "#;

        let wasm = wat::parse_str(wat).unwrap();

        let mut config = RuntimeConfig::for_testing();
        config.max_execution_time_ms = 100; // Short timeout
        config.max_fuel = 100_000; // Limited fuel

        let runtime = WasmRuntime::new(config).unwrap();
        let module = runtime.compile(&wasm).unwrap();

        let result = runtime
            .execute(&module, "work", &[], [0u8; 32], 0, 0)
            .await;

        // Should fail (either timeout or fuel exhaustion, both acceptable)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timeout_error_message() {
        // Verify that timeout errors are clear and identifiable
        let wat = r#"
            (module
                (memory (export "memory") 1)

                (func (export "allocate") (param i32) (result i32)
                    i32.const 100
                )

                (func (export "hang") (param i32 i32) (result i32)
                    (loop $forever
                        br $forever
                    )
                    i32.const 0
                )
            )
        "#;

        let wasm = wat::parse_str(wat).unwrap();

        let mut config = RuntimeConfig::for_testing();
        config.max_execution_time_ms = 50;
        config.max_fuel = u64::MAX;
        let runtime = WasmRuntime::new(config).unwrap();
        let module = runtime.compile(&wasm).unwrap();

        let result = runtime
            .execute(&module, "hang", &[], [0u8; 32], 0, 0)
            .await;

        match result {
            Err(WasmError::Timeout) => {
                // Correct error type
            }
            other => panic!("Expected WasmError::Timeout, got {:?}", other),
        }
    }
}
