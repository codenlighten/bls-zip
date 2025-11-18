// Enterprise Multipass - Error Types

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum EnterpriseError {
    // Identity & Attestation errors
    #[error("Identity not found: {0}")]
    IdentityNotFound(String),

    #[error("KYC verification failed: {0}")]
    KycFailed(String),

    #[error("Attestation invalid or expired")]
    AttestationInvalid,

    // Wallet errors
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    // Auth/SSO errors
    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Session expired")]
    SessionExpired,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid token")]
    InvalidToken,

    // Application errors
    #[error("Application not found: {0}")]
    ApplicationNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    // Asset & Market errors
    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Order not found: {0}")]
    OrderNotFound(String),

    #[error("Insufficient liquidity")]
    InsufficientLiquidity,

    // Hardware errors
    #[error("Hardware device not found: {0}")]
    HardwareNotFound(String),

    #[error("Hardware device revoked or lost")]
    HardwareRevoked,

    // Blockchain interaction errors
    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    #[error("Failed to anchor proof on chain: {0}")]
    ChainAnchorFailed(String),

    #[error("Failed to connect to blockchain")]
    BlockchainConnectionError,

    // Database errors
    #[error("Database error: {0}")]
    DatabaseError(String),

    // Cryptography errors
    #[error("Cryptography error: {0}")]
    CryptoError(String),

    // General errors
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

// FIX M-10: Sanitize database errors to prevent information leakage
impl From<sqlx::Error> for EnterpriseError {
    fn from(err: sqlx::Error) -> Self {
        // Log the real error for debugging (server-side only)
        tracing::error!("Database error: {:?}", err);

        // Return generic error message (no table/column/constraint details)
        EnterpriseError::DatabaseError("Database operation failed".to_string())
    }
}

// FIX M-10: Helper to sanitize generic errors (for use with .map_err)
impl EnterpriseError {
    pub fn from_db_error<E: std::fmt::Display + std::fmt::Debug>(err: E) -> Self {
        // Log the real error for debugging (server-side only)
        tracing::error!("Database error: {:?}", err);

        // Return generic error message
        EnterpriseError::DatabaseError("Database operation failed".to_string())
    }
}

// Note: boundless_storage and boundless_core are not available in the enterprise package
// impl From<boundless_storage::StorageError> for EnterpriseError {
//     fn from(err: boundless_storage::StorageError) -> Self {
//         EnterpriseError::BlockchainError(err.to_string())
//     }
// }

// impl From<boundless_core::error::CoreError> for EnterpriseError {
//     fn from(err: boundless_core::error::CoreError) -> Self {
//         EnterpriseError::BlockchainError(err.to_string())
//     }
// }

// Axum response conversion
impl IntoResponse for EnterpriseError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            EnterpriseError::IdentityNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            EnterpriseError::WalletNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            EnterpriseError::ApplicationNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            EnterpriseError::AssetNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            EnterpriseError::OrderNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            EnterpriseError::HardwareNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),

            EnterpriseError::AuthenticationFailed => (StatusCode::UNAUTHORIZED, self.to_string()),
            EnterpriseError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
            EnterpriseError::SessionExpired => (StatusCode::UNAUTHORIZED, self.to_string()),
            EnterpriseError::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string()),

            EnterpriseError::Unauthorized(_) => (StatusCode::FORBIDDEN, self.to_string()),
            EnterpriseError::PermissionDenied(_) => (StatusCode::FORBIDDEN, self.to_string()),

            EnterpriseError::InvalidInput(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            EnterpriseError::ValidationError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            EnterpriseError::KycFailed(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            EnterpriseError::AttestationInvalid => (StatusCode::BAD_REQUEST, self.to_string()),
            EnterpriseError::InsufficientBalance => (StatusCode::BAD_REQUEST, self.to_string()),
            EnterpriseError::InsufficientLiquidity => (StatusCode::BAD_REQUEST, self.to_string()),
            EnterpriseError::HardwareRevoked => (StatusCode::BAD_REQUEST, self.to_string()),

            EnterpriseError::TransactionFailed(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            EnterpriseError::BlockchainError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            EnterpriseError::ChainAnchorFailed(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            EnterpriseError::BlockchainConnectionError => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            EnterpriseError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            EnterpriseError::CryptoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            EnterpriseError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),

            EnterpriseError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            EnterpriseError::NotImplemented(_) => (StatusCode::NOT_IMPLEMENTED, self.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, EnterpriseError>;
