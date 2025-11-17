// RPC error types
use jsonrpsee::types::ErrorObjectOwned;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<RpcError> for ErrorObjectOwned {
    fn from(err: RpcError) -> Self {
        let code = match &err {
            RpcError::InvalidParams(_) => -32602,
            RpcError::BlockNotFound(_) => -32001,
            RpcError::TransactionNotFound(_) => -32002,
            RpcError::InvalidAddress(_) => -32003,
            RpcError::InvalidTransaction(_) => -32004,
            RpcError::Internal(_) | RpcError::Serialization(_) => -32603,
        };

        ErrorObjectOwned::owned(code, err.to_string(), None::<String>)
    }
}

impl From<ErrorObjectOwned> for RpcError {
    fn from(err: ErrorObjectOwned) -> Self {
        RpcError::InvalidParams(err.message().to_string())
    }
}
