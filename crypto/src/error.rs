// Error types for cryptographic operations
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("PQC operation failed: {0}")]
    PqcError(String),

    #[error("Key generation failed: {0}")]
    KeyGeneration(String),

    #[error("Signature verification failed")]
    InvalidSignature,

    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionError(String),

    #[error("Decryption failed: {0}")]
    DecryptionError(String),

    #[error("Key exchange failed: {0}")]
    KeyExchangeError(String),

    #[error("Invalid public key")]
    InvalidPublicKey,

    #[error("Invalid private key")]
    InvalidPrivateKey,

    #[error("Invalid secret key")]
    InvalidSecretKey,

    #[error("Invalid ciphertext")]
    InvalidCiphertext,

    #[error("Hybrid scheme mismatch")]
    HybridMismatch,

    #[error("PHE operation failed: {0}")]
    PheError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<oqs::Error> for CryptoError {
    fn from(err: oqs::Error) -> Self {
        CryptoError::PqcError(err.to_string())
    }
}
