// Boundless BLS Cryptography - Post-Quantum and Hybrid Cryptographic Systems
//
// This module provides PQC implementations using NIST-standardized algorithms:
// - ML-KEM-768 (FIPS 203) for key encapsulation
// - ML-DSA-44 (FIPS 204) for digital signatures
// - Falcon-512 for compact signatures
// - Hybrid schemes combining classical (X25519, Ed25519) with PQC
// - Paillier PHE for privacy-preserving computations

pub mod error;
pub mod hybrid;
pub mod pqc;
pub mod phe;  // Re-enabled with custom implementation (no dependency conflicts)

pub use error::CryptoError;
pub use hybrid::{
    HybridKex, HybridKeyPair, HybridPublicKey, HybridSecretKey, HybridSignature,
    HybridSignatureData, HybridSignatureKeyPair, HybridSignaturePublicKey,
};
pub use pqc::{Falcon512, MlDsa44, MlKem768};
pub use phe::{Ciphertext, DecryptionKey, EncryptionKey, PaillierPhe, PrivateAggregator};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pqc_availability() {
        // Ensure all PQC algorithms are available
        assert!(MlKem768::new().is_ok());
        assert!(MlDsa44::new().is_ok());
        assert!(Falcon512::new().is_ok());
    }

    #[test]
    fn test_hybrid_availability() {
        // Ensure hybrid schemes are available
        assert!(HybridKex::new().is_ok());
        assert!(HybridSignature::new().is_ok());
    }
}
