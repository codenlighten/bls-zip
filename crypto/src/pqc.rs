// Post-Quantum Cryptography implementations using liboqs
use crate::error::CryptoError;
use oqs::{kem, sig};

// Type alias to avoid conflict with oqs::Result
type Result<T> = std::result::Result<T, CryptoError>;

/// ML-KEM-768 key encapsulation mechanism
pub struct MlKem768 {
    kem: kem::Kem,
}

impl MlKem768 {
    /// Create a new ML-KEM-768 instance
    pub fn new() -> Result<Self> {
        let kem = kem::Kem::new(kem::Algorithm::MlKem768)
            .map_err(|e| CryptoError::PqcError(e.to_string()))?;
        Ok(Self { kem })
    }

    /// Generate a keypair
    pub fn keypair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let (pk, sk) = self
            .kem
            .keypair()
            .map_err(|e| CryptoError::PqcError(e.to_string()))?;
        Ok((pk.into_vec(), sk.into_vec()))
    }

    /// Encapsulate a shared secret using the public key
    pub fn encapsulate(&self, public_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let pk = self
            .kem
            .public_key_from_bytes(public_key)
            .ok_or(CryptoError::InvalidPublicKey)?;
        let (ciphertext, shared_secret) = self
            .kem
            .encapsulate(pk)
            .map_err(|e| CryptoError::PqcError(e.to_string()))?;
        Ok((ciphertext.into_vec(), shared_secret.into_vec()))
    }

    /// Decapsulate a shared secret using the private key
    pub fn decapsulate(&self, secret_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
        let sk = self
            .kem
            .secret_key_from_bytes(secret_key)
            .ok_or(CryptoError::InvalidSecretKey)?;
        let ct = self
            .kem
            .ciphertext_from_bytes(ciphertext)
            .ok_or(CryptoError::InvalidCiphertext)?;
        let shared_secret = self
            .kem
            .decapsulate(sk, ct)
            .map_err(|e| CryptoError::PqcError(e.to_string()))?;
        Ok(shared_secret.into_vec())
    }
}

impl Default for MlKem768 {
    fn default() -> Self {
        Self::new().expect("Failed to initialize ML-KEM-768")
    }
}

/// ML-DSA-44 digital signature algorithm (formerly Dilithium2)
pub struct MlDsa44 {
    sig: sig::Sig,
}

impl MlDsa44 {
    /// Create a new ML-DSA-44 instance
    pub fn new() -> Result<Self> {
        let sig = sig::Sig::new(sig::Algorithm::Dilithium2)
            .map_err(|e| CryptoError::PqcError(e.to_string()))?;
        Ok(Self { sig })
    }

    /// Generate a keypair
    pub fn keypair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let (pk, sk) = self
            .sig
            .keypair()
            .map_err(|e| CryptoError::PqcError(e.to_string()))?;
        Ok((pk.into_vec(), sk.into_vec()))
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8], secret_key: &[u8]) -> Result<Vec<u8>> {
        let sk = self
            .sig
            .secret_key_from_bytes(secret_key)
            .ok_or(CryptoError::InvalidSecretKey)?;
        let signature = self
            .sig
            .sign(message, sk)
            .map_err(|e| CryptoError::PqcError(e.to_string()))?;
        Ok(signature.into_vec())
    }

    /// Verify a signature
    ///
    /// SECURITY FIX: Properly propagates errors instead of masking them
    pub fn verify(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool> {
        let pk = self
            .sig
            .public_key_from_bytes(public_key)
            .ok_or(CryptoError::InvalidPublicKey)?;
        let sig_ref = self
            .sig
            .signature_from_bytes(signature)
            .ok_or(CryptoError::InvalidSignature)?;

        match self.sig.verify(message, sig_ref, pk) {
            Ok(()) => Ok(true),
            Err(e) => {
                // SECURITY FIX: Propagate error with context instead of hiding it
                Err(CryptoError::SignatureVerificationFailed(format!(
                    "ML-DSA-44 verification failed: {}",
                    e
                )))
            }
        }
    }
}

impl Default for MlDsa44 {
    fn default() -> Self {
        Self::new().expect("Failed to initialize ML-DSA-44")
    }
}

/// Falcon-512 digital signature algorithm
pub struct Falcon512 {
    sig: sig::Sig,
}

impl Falcon512 {
    /// Create a new Falcon-512 instance
    pub fn new() -> Result<Self> {
        let sig = sig::Sig::new(sig::Algorithm::Falcon512)
            .map_err(|e| CryptoError::PqcError(e.to_string()))?;
        Ok(Self { sig })
    }

    /// Generate a keypair
    pub fn keypair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let (pk, sk) = self
            .sig
            .keypair()
            .map_err(|e| CryptoError::PqcError(e.to_string()))?;
        Ok((pk.into_vec(), sk.into_vec()))
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8], secret_key: &[u8]) -> Result<Vec<u8>> {
        let sk = self
            .sig
            .secret_key_from_bytes(secret_key)
            .ok_or(CryptoError::InvalidSecretKey)?;
        let signature = self
            .sig
            .sign(message, sk)
            .map_err(|e| CryptoError::PqcError(e.to_string()))?;
        Ok(signature.into_vec())
    }

    /// Verify a signature
    ///
    /// SECURITY FIX: Properly propagates errors instead of masking them
    pub fn verify(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool> {
        let pk = self
            .sig
            .public_key_from_bytes(public_key)
            .ok_or(CryptoError::InvalidPublicKey)?;
        let sig_ref = self
            .sig
            .signature_from_bytes(signature)
            .ok_or(CryptoError::InvalidSignature)?;

        match self.sig.verify(message, sig_ref, pk) {
            Ok(()) => Ok(true),
            Err(e) => {
                // SECURITY FIX: Propagate error with context instead of hiding it
                Err(CryptoError::SignatureVerificationFailed(format!(
                    "Falcon-512 verification failed: {}",
                    e
                )))
            }
        }
    }
}

impl Default for Falcon512 {
    fn default() -> Self {
        Self::new().expect("Failed to initialize Falcon-512")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_kem_768() {
        let kem = MlKem768::new().unwrap();
        let (pk, sk) = kem.keypair().unwrap();

        let (ciphertext, ss1) = kem.encapsulate(&pk).unwrap();
        let ss2 = kem.decapsulate(&sk, &ciphertext).unwrap();

        assert_eq!(ss1, ss2);
    }

    #[test]
    fn test_ml_dsa_44() {
        let dsa = MlDsa44::new().unwrap();
        let (pk, sk) = dsa.keypair().unwrap();

        let message = b"Hello, Boundless BLS!";
        let signature = dsa.sign(message, &sk).unwrap();

        assert!(dsa.verify(message, &signature, &pk).unwrap());

        // Wrong message should return error (security fix: errors are propagated)
        let wrong_message = b"Wrong message";
        assert!(dsa.verify(wrong_message, &signature, &pk).is_err());
    }

    #[test]
    fn test_falcon_512() {
        let falcon = Falcon512::new().unwrap();
        let (pk, sk) = falcon.keypair().unwrap();

        let message = b"Falcon test message";
        let signature = falcon.sign(message, &sk).unwrap();

        assert!(falcon.verify(message, &signature, &pk).unwrap());
    }
}
