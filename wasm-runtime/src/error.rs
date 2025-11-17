// Error types for WASM runtime operations
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WasmError {
    #[error("Runtime initialization failed: {0}")]
    InitializationError(String),

    #[error("Module compilation failed: {0}")]
    CompilationError(String),

    #[error("Module instantiation failed: {0}")]
    InstantiationError(String),

    #[error("Function execution failed: {0}")]
    ExecutionError(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Out of fuel: consumed {consumed}, limit {limit}")]
    OutOfFuel { consumed: u64, limit: u64 },

    #[error("Memory allocation failed: {0}")]
    MemoryError(String),

    #[error("Invalid WASM module: {0}")]
    InvalidModule(String),

    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },

    #[error("Contract execution exceeded time limit")]
    Timeout,

    #[error("Contract execution exceeded memory limit")]
    MemoryLimitExceeded,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Invalid contract call data")]
    InvalidCallData,

    #[error("Host function error: {0}")]
    HostFunctionError(String),
}

impl From<anyhow::Error> for WasmError {
    fn from(err: anyhow::Error) -> Self {
        WasmError::ExecutionError(err.to_string())
    }
}
