// Post-Quantum Cryptography implementations using liboqs
use oqs::*;
use crate::error::CryptoError;

pub struct MlKem768 {
    kem: kem::Kem,
}

impl MlKem768 {
    pub fn new() -> Result<Self, CryptoError> {
        let kem = kem::Kem::new(kem::Algorithm::MlKem768)?;
        Ok(Self { kem })
    }

    pub fn keypair(&self) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let (pk, sk) = self.kem.keypair()?;
        Ok((pk.into_vec(), sk.into_vec()))
    }

    pub fn encapsulate(&self, public_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let (ciphertext, shared_secret) = self.kem.encaps(public_key)?;
        Ok((ciphertext.into_vec(), shared_secret.into_vec()))
    }

    pub fn decapsulate(&self, secret_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let shared_secret = self.kem.decaps(secret_key, ciphertext)?;
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
    pub fn new() -> Result<Self, CryptoError> {
        let sig = sig::Sig::new(sig::Algorithm::Dilithium2)?;
        Ok(Self { sig })
    }

    pub fn keypair(&self) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let (pk, sk) = self.sig.keypair()?;
        Ok((pk.into_vec(), sk.into_vec()))
    }

    pub fn sign(&self, message: &[u8], secret_key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Create owned SecretKey from bytes
        let sk = sig::SecretKey::from_bytes(secret_key)
            .ok_or(CryptoError::InvalidSecretKey)?;
        let signature = self.sig.sign(message, &sk)?;
        Ok(signature.into_vec())
    }

    pub fn verify(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool, CryptoError> {
        // Create owned types from bytes
        let pk = sig::PublicKey::from_bytes(public_key)
            .ok_or(CryptoError::InvalidPublicKey)?;
        let sig_owned = sig::Signature::from_bytes(signature)
            .ok_or(CryptoError::InvalidSignature)?;

        match self.sig.verify(message, &sig_owned, &pk) {
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
    pub fn new() -> Result<Self, CryptoError> {
        let sig = sig::Sig::new(sig::Algorithm::Falcon512)?;
        Ok(Self { sig })
    }

    pub fn keypair(&self) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let (pk, sk) = self.sig.keypair()?;
        Ok((pk.into_vec(), sk.into_vec()))
    }

    pub fn sign(&self, message: &[u8], secret_key: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let sk = sig::SecretKey::from_bytes(secret_key)
            .ok_or(CryptoError::InvalidSecretKey)?;
        let signature = self.sig.sign(message, &sk)?;
        Ok(signature.into_vec())
    }

    pub fn verify(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool, CryptoError> {
        let pk = sig::PublicKey::from_bytes(public_key)
            .ok_or(CryptoError::InvalidPublicKey)?;
        let sig_owned = sig::Signature::from_bytes(signature)
            .ok_or(CryptoError::InvalidSignature)?;

        match self.sig.verify(message, &sig_owned, &pk) {
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
