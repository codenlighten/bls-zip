// Boundless BLS WASM Runtime - Deterministic Smart Contract Execution
//
// This module provides a Wasmtime-based runtime for executing WebAssembly smart contracts
// with deterministic fuel metering, memory limits, and blockchain-specific host functions.
//
// Features:
// - Fuel-based gas metering for deterministic execution costs
// - Memory and stack limits for security
// - Host functions for storage, cryptography, and blockchain context
// - Pooling allocator for high-performance execution

pub mod config;
pub mod error;
pub mod host_functions;
pub mod runtime;

pub use config::{ExecutionResult, RuntimeConfig};
pub use error::WasmError;
pub use runtime::{ContractState, WasmRuntime};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_initialization() {
        let config = RuntimeConfig::default();
        let runtime = WasmRuntime::new(config);
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_config_presets() {
        let test_config = RuntimeConfig::for_testing();
        assert_eq!(test_config.max_fuel, 1_000_000_000);

        let prod_config = RuntimeConfig::for_production();
        assert_eq!(prod_config.max_fuel, 50_000_000);
    }
}
