// Post-Quantum Cryptography implementations using liboqs
use oqs::*;
use crate::error::CryptoError;

pub struct MlKem768 {
    kem: kem::Kem,
}

impl MlKem768 {
    pub fn new() -> std::result::Result<Self, CryptoError> {
        let kem = kem::Kem::new(kem::Algorithm::MlKem768)?;
        Ok(Self { kem })
    }

    pub fn keypair(&self) -> std::result::Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let (pk, sk) = self.kem.keypair()?;
        // Serialize using serde
        let pk_bytes = serde_json::to_vec(&pk)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        let sk_bytes = serde_json::to_vec(&sk)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        Ok((pk_bytes, sk_bytes))
    }

    pub fn encapsulate(&self, public_key_bytes: &[u8]) -> std::result::Result<(Vec<u8>, Vec<u8>), CryptoError> {
        // Deserialize public key
        let pk: kem::PublicKey = serde_json::from_slice(public_key_bytes)
            .map_err(|e| CryptoError::InvalidPublicKey)?;

        // Use KEM encapsulate which accepts &PublicKey
        let (ciphertext, shared_secret) = self.kem.encapsulate(&pk)?;
        Ok((ciphertext.into_vec(), shared_secret.into_vec()))
    }

    pub fn decapsulate(&self, secret_key_bytes: &[u8], ciphertext: &[u8]) -> std::result::Result<Vec<u8>, CryptoError> {
        // Deserialize secret key
        let sk: kem::SecretKey = serde_json::from_slice(secret_key_bytes)
            .map_err(|e| CryptoError::InvalidSecretKey)?;

        // Use KEM decapsulate
        let shared_secret = self.kem.decapsulate(&sk, ciphertext)?;
        Ok(shared_secret.into_vec())
    }
}

impl Default for MlKem768 {
    fn default() -> Self {
        Self::new().expect("Failed to initialize ML-KEM-768")
    }
}

pub struct MlDsa44 {
    sig: sig::Sig,
}

impl MlDsa44 {
    pub fn new() -> std::result::Result<Self, CryptoError> {
        let sig = sig::Sig::new(sig::Algorithm::Dilithium2)?;
        Ok(Self { sig })
    }

    pub fn keypair(&self) -> std::result::Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let (pk, sk) = self.sig.keypair()?;
        // Serialize using serde
        let pk_bytes = serde_json::to_vec(&pk)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        let sk_bytes = serde_json::to_vec(&sk)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        Ok((pk_bytes, sk_bytes))
    }

    pub fn sign(&self, message: &[u8], secret_key_bytes: &[u8]) -> std::result::Result<Vec<u8>, CryptoError> {
        // Deserialize secret key
        let sk: sig::SecretKey = serde_json::from_slice(secret_key_bytes)
            .map_err(|e| CryptoError::InvalidSecretKey)?;

        // Sign using owned type reference
        let signature = self.sig.sign(message, &sk)?;

        // Serialize signature
        let sig_bytes = serde_json::to_vec(&signature)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        Ok(sig_bytes)
    }

    pub fn verify(&self, message: &[u8], signature_bytes: &[u8], public_key_bytes: &[u8]) -> std::result::Result<bool, CryptoError> {
        // Deserialize all components
        let pk: sig::PublicKey = serde_json::from_slice(public_key_bytes)
            .map_err(|_| CryptoError::InvalidPublicKey)?;
        let signature: sig::Signature = serde_json::from_slice(signature_bytes)
            .map_err(|_| CryptoError::InvalidSignature)?;

        // Verify using owned type references
        match self.sig.verify(message, &signature, &pk) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

impl Default for MlDsa44 {
    fn default() -> Self {
        Self::new().expect("Failed to initialize ML-DSA-44")
    }
}

pub struct Falcon512 {
    sig: sig::Sig,
}

impl Falcon512 {
    pub fn new() -> std::result::Result<Self, CryptoError> {
        let sig = sig::Sig::new(sig::Algorithm::Falcon512)?;
        Ok(Self { sig })
    }

    pub fn keypair(&self) -> std::result::Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let (pk, sk) = self.sig.keypair()?;
        let pk_bytes = serde_json::to_vec(&pk)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        let sk_bytes = serde_json::to_vec(&sk)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        Ok((pk_bytes, sk_bytes))
    }

    pub fn sign(&self, message: &[u8], secret_key_bytes: &[u8]) -> std::result::Result<Vec<u8>, CryptoError> {
        let sk: sig::SecretKey = serde_json::from_slice(secret_key_bytes)
            .map_err(|e| CryptoError::InvalidSecretKey)?;
        let signature = self.sig.sign(message, &sk)?;
        let sig_bytes = serde_json::to_vec(&signature)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        Ok(sig_bytes)
    }

    pub fn verify(&self, message: &[u8], signature_bytes: &[u8], public_key_bytes: &[u8]) -> std::result::Result<bool, CryptoError> {
        let pk: sig::PublicKey = serde_json::from_slice(public_key_bytes)
            .map_err(|_| CryptoError::InvalidPublicKey)?;
        let signature: sig::Signature = serde_json::from_slice(signature_bytes)
            .map_err(|_| CryptoError::InvalidSignature)?;

        match self.sig.verify(message, &signature, &pk) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

impl Default for Falcon512 {
    fn default() -> Self {
        Self::new().expect("Failed to initialize Falcon-512")
    }
}
