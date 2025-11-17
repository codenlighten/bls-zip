// Boundless Enterprise - Post-Quantum Cryptography Module
//
// Uses ML-DSA (Dilithium) for signatures and ML-KEM (Kyber) for key encapsulation
// Compatible with Boundless blockchain's PQC implementation

use pqcrypto_dilithium::dilithium5 as dilithium;
use pqcrypto_kyber::kyber1024 as kyber;
use pqcrypto_traits::kem::{Ciphertext as KemCiphertext, PublicKey as KemPublicKey, SecretKey as KemSecretKey, SharedSecret as KemSharedSecret};
use pqcrypto_traits::sign::{PublicKey as SignPublicKey, SecretKey as SignSecretKey, SignedMessage, DetachedSignature};
use sha3::{Digest, Sha3_256};
use zeroize::Zeroizing;

use crate::error::{EnterpriseError, Result};

/// Post-Quantum Key Pair for Digital Signatures (Dilithium)
#[derive(Clone)]
pub struct PqcKeyPair {
    pub public_key: Vec<u8>,
    secret_key: Zeroizing<Vec<u8>>,  // Automatically zeroed on drop
}

impl PqcKeyPair {
    /// Generate a new PQC key pair using Dilithium5 (ML-DSA)
    pub fn generate() -> Result<Self> {
        let (pk, sk) = dilithium::keypair();

        Ok(Self {
            public_key: pk.as_bytes().to_vec(),
            secret_key: Zeroizing::new(sk.as_bytes().to_vec()),
        })
    }

    /// Create from existing key material
    pub fn from_bytes(public_key: Vec<u8>, secret_key: Vec<u8>) -> Result<Self> {
        // Validate key sizes
        if public_key.len() != dilithium::public_key_bytes() {
            return Err(EnterpriseError::CryptoError(
                format!("Invalid public key size: expected {}, got {}",
                    dilithium::public_key_bytes(), public_key.len())
            ));
        }

        if secret_key.len() != dilithium::secret_key_bytes() {
            return Err(EnterpriseError::CryptoError(
                format!("Invalid secret key size: expected {}, got {}",
                    dilithium::secret_key_bytes(), secret_key.len())
            ));
        }

        Ok(Self {
            public_key,
            secret_key: Zeroizing::new(secret_key),
        })
    }

    /// Sign a message using the private key
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        // Reconstruct secret key for signing
        let sk = dilithium::SecretKey::from_bytes(&self.secret_key)
            .map_err(|e| EnterpriseError::CryptoError(format!("Failed to reconstruct secret key: {:?}", e)))?;

        // Sign the message
        let signed_msg = dilithium::sign(message, &sk);

        Ok(signed_msg.as_bytes().to_vec())
    }

    /// Sign a message and return detached signature
    pub fn sign_detached(&self, message: &[u8]) -> Result<Vec<u8>> {
        // Reconstruct secret key
        let sk = dilithium::SecretKey::from_bytes(&self.secret_key)
            .map_err(|e| EnterpriseError::CryptoError(format!("Failed to reconstruct secret key: {:?}", e)))?;

        // Create detached signature
        let sig = dilithium::detached_sign(message, &sk);

        Ok(sig.as_bytes().to_vec())
    }

    /// Get the public key bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.public_key
    }

    /// Get the secret key bytes (use with caution)
    pub fn secret_key_bytes(&self) -> &[u8] {
        &self.secret_key
    }

    /// Derive a Boundless-compatible address from the public key
    /// FIX: Use full 32-byte SHA3-256 hash (64 hex chars) to align with blockchain spec
    pub fn derive_address(&self) -> String {
        // Hash the public key using SHA3-256
        let mut hasher = Sha3_256::new();
        hasher.update(&self.public_key);
        let hash = hasher.finalize();

        // Use full 32 bytes encoded as hex (64 characters)
        // Boundless blockchain expects 32-byte addresses
        hex::encode(&hash)
    }
}

/// Verify a signature using the public key
pub struct PqcSignature;

impl PqcSignature {
    /// Verify a signed message
    pub fn verify(public_key: &[u8], signed_message: &[u8]) -> Result<bool> {
        // Reconstruct public key
        let pk = dilithium::PublicKey::from_bytes(public_key)
            .map_err(|e| EnterpriseError::CryptoError(format!("Invalid public key: {:?}", e)))?;

        // Reconstruct signed message
        let signed = dilithium::SignedMessage::from_bytes(signed_message)
            .map_err(|e| EnterpriseError::CryptoError(format!("Invalid signed message: {:?}", e)))?;

        // Verify and open the message
        match dilithium::open(&signed, &pk) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Verify a detached signature
    pub fn verify_detached(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool> {
        // Reconstruct public key
        let pk = dilithium::PublicKey::from_bytes(public_key)
            .map_err(|e| EnterpriseError::CryptoError(format!("Invalid public key: {:?}", e)))?;

        // Reconstruct signature
        let sig = dilithium::DetachedSignature::from_bytes(signature)
            .map_err(|e| EnterpriseError::CryptoError(format!("Invalid signature: {:?}", e)))?;

        // Verify the detached signature
        match dilithium::verify_detached_signature(&sig, message, &pk) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Key Encapsulation Mechanism (KEM) using Kyber
pub struct PqcKem;

impl PqcKem {
    /// Generate a KEM key pair for encryption
    pub fn keypair() -> (Vec<u8>, Zeroizing<Vec<u8>>) {
        let (pk, sk) = kyber::keypair();
        (
            pk.as_bytes().to_vec(),
            Zeroizing::new(sk.as_bytes().to_vec()),
        )
    }

    /// Encapsulate a shared secret for the recipient's public key
    pub fn encapsulate(public_key: &[u8]) -> Result<(Vec<u8>, Zeroizing<Vec<u8>>)> {
        // Reconstruct public key
        let pk = kyber::PublicKey::from_bytes(public_key)
            .map_err(|e| EnterpriseError::CryptoError(format!("Invalid KEM public key: {:?}", e)))?;

        // Encapsulate to generate shared secret and ciphertext
        let (ss, ct) = kyber::encapsulate(&pk);

        Ok((
            ct.as_bytes().to_vec(),  // Ciphertext to send
            Zeroizing::new(ss.as_bytes().to_vec()),  // Shared secret
        ))
    }

    /// Decapsulate the shared secret using the private key
    pub fn decapsulate(secret_key: &[u8], ciphertext: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
        // Reconstruct secret key
        let sk = kyber::SecretKey::from_bytes(secret_key)
            .map_err(|e| EnterpriseError::CryptoError(format!("Invalid KEM secret key: {:?}", e)))?;

        // Reconstruct ciphertext
        let ct = kyber::Ciphertext::from_bytes(ciphertext)
            .map_err(|e| EnterpriseError::CryptoError(format!("Invalid ciphertext: {:?}", e)))?;

        // Decapsulate to recover shared secret
        let ss = kyber::decapsulate(&ct, &sk);

        Ok(Zeroizing::new(ss.as_bytes().to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = PqcKeyPair::generate().unwrap();
        assert!(!keypair.public_key.is_empty());
        assert!(!keypair.secret_key.is_empty());
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = PqcKeyPair::generate().unwrap();
        let message = b"Test message for signing";

        // Sign the message
        let signed_msg = keypair.sign(message).unwrap();

        // Verify the signature
        let is_valid = PqcSignature::verify(&keypair.public_key, &signed_msg).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_detached_signature() {
        let keypair = PqcKeyPair::generate().unwrap();
        let message = b"Test message for detached signature";

        // Sign with detached signature
        let signature = keypair.sign_detached(message).unwrap();

        // Verify detached signature
        let is_valid = PqcSignature::verify_detached(
            &keypair.public_key,
            message,
            &signature
        ).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_address_derivation() {
        let keypair = PqcKeyPair::generate().unwrap();
        let address = keypair.derive_address();

        // Address should be 64 hex characters (32 bytes)
        assert_eq!(address.len(), 64);
        // Verify it's valid hex
        assert!(hex::decode(&address).is_ok());
    }

    #[test]
    fn test_kem_encapsulation() {
        let (pk, sk) = PqcKem::keypair();

        // Encapsulate
        let (ciphertext, shared_secret1) = PqcKem::encapsulate(&pk).unwrap();

        // Decapsulate
        let shared_secret2 = PqcKem::decapsulate(&sk, &ciphertext).unwrap();

        // Shared secrets should match
        assert_eq!(&*shared_secret1, &*shared_secret2);
    }
}
