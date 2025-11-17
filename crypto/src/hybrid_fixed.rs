// Hybrid cryptographic schemes combining classical and post-quantum algorithms
use x25519_dalek::{StaticSecret, PublicKey as X25519PublicKey, SharedSecret};
use ed25519_dalek::{Signer, Verifier, Signature as Ed25519Signature, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use sha3::{Digest, Sha3_256};
use serde::{Serialize, Deserialize};

use crate::pqc::{MlKem768, MlDsa44};
use crate::error::CryptoError;

/// Hybrid key exchange: X25519 + ML-KEM-768
pub struct HybridKex {
    ml_kem: MlKem768,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridKeyPair {
    pub classical_public: Vec<u8>,
    pub classical_secret: Vec<u8>,
    pub pqc_public: Vec<u8>,
    pub pqc_secret: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridPublicKey {
    pub classical_public: Vec<u8>,
    pub pqc_public: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct HybridSecretKey {
    pub classical_secret: Vec<u8>,
    pub pqc_secret: Vec<u8>,
}

impl HybridKex {
    pub fn new() -> Result<Self, CryptoError> {
        Ok(Self {
            ml_kem: MlKem768::new()?,
        })
    }

    pub fn keypair(&self) -> Result<HybridKeyPair, CryptoError> {
        let x25519_secret = StaticSecret::random_from_rng(OsRng);
        let x25519_public = X25519PublicKey::from(&x25519_secret);
        let (pqc_pk, pqc_sk) = self.ml_kem.keypair()?;
        let secret_bytes = x25519_secret.to_bytes();

        Ok(HybridKeyPair {
            classical_public: x25519_public.to_bytes().to_vec(),
            classical_secret: secret_bytes.to_vec(),
            pqc_public: pqc_pk,
            pqc_secret: pqc_sk,
        })
    }

    pub fn encapsulate(
        &self,
        hybrid_public: &HybridPublicKey,
    ) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        let ephemeral_secret = StaticSecret::random_from_rng(OsRng);
        let ephemeral_public = X25519PublicKey::from(&ephemeral_secret);

        let x25519_pk_bytes: [u8; 32] = hybrid_public.classical_public
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidPublicKey)?;
        let x25519_pk = X25519PublicKey::from(x25519_pk_bytes);
        let classical_shared = ephemeral_secret.diffie_hellman(&x25519_pk);
        let (pqc_ciphertext, pqc_shared) = self.ml_kem.encapsulate(&hybrid_public.pqc_public)?;
        let combined_shared = Self::combine_secrets(classical_shared.as_bytes(), &pqc_shared);

        let mut ciphertext = Vec::new();
        ciphertext.extend_from_slice(ephemeral_public.as_bytes());
        ciphertext.extend_from_slice(&(pqc_ciphertext.len() as u32).to_le_bytes());
        ciphertext.extend_from_slice(&pqc_ciphertext);

        Ok((ciphertext, combined_shared))
    }

    pub fn decapsulate(
        &self,
        hybrid_secret: &HybridSecretKey,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        if ciphertext.len() < 36 {
            return Err(CryptoError::InvalidCiphertext);
        }

        let ephemeral_public_bytes: [u8; 32] = ciphertext[0..32]
            .try_into()
            .map_err(|_| CryptoError::InvalidCiphertext)?;
        let ephemeral_public = X25519PublicKey::from(ephemeral_public_bytes);

        let pqc_ct_len = u32::from_le_bytes(
            ciphertext[32..36].try_into().map_err(|_| CryptoError::InvalidCiphertext)?
        ) as usize;

        if ciphertext.len() < 36 + pqc_ct_len {
            return Err(CryptoError::InvalidCiphertext);
        }

        let pqc_ciphertext = &ciphertext[36..36 + pqc_ct_len];
        let secret_bytes: [u8; 32] = hybrid_secret.classical_secret
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidSecretKey)?;
        let x25519_secret = StaticSecret::from(secret_bytes);
        let classical_shared = x25519_secret.diffie_hellman(&ephemeral_public);
        let pqc_shared = self.ml_kem.decapsulate(&hybrid_secret.pqc_secret, pqc_ciphertext)?;
        let combined_shared = Self::combine_secrets(classical_shared.as_bytes(), &pqc_shared);

        Ok(combined_shared)
    }

    fn combine_secrets(classical: &[u8], pqc: &[u8]) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        hasher.update(b"HYBRID_KEX");
        hasher.update(classical);
        hasher.update(pqc);
        hasher.finalize().to_vec()
    }
}

pub struct HybridSignature {
    ml_dsa: MlDsa44,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridSignatureKeyPair {
    pub classical_public: Vec<u8>,
    pub classical_secret: Vec<u8>,
    pub pqc_public: Vec<u8>,
    pub pqc_secret: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridSignaturePublicKey {
    pub classical_public: Vec<u8>,
    pub pqc_public: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridSignatureData {
    pub classical_signature: Vec<u8>,
    pub pqc_signature: Vec<u8>,
}

impl HybridSignature {
    pub fn new() -> Result<Self, CryptoError> {
        Ok(Self {
            ml_dsa: MlDsa44::new()?,
        })
    }

    pub fn keypair(&self) -> Result<HybridSignatureKeyPair, CryptoError> {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let (pqc_pk, pqc_sk) = self.ml_dsa.keypair()?;

        Ok(HybridSignatureKeyPair {
            classical_public: verifying_key.to_bytes().to_vec(),
            classical_secret: signing_key.to_bytes().to_vec(),
            pqc_public: pqc_pk,
            pqc_secret: pqc_sk,
        })
    }

    pub fn sign(
        &self,
        message: &[u8],
        keypair: &HybridSignatureKeyPair,
    ) -> Result<HybridSignatureData, CryptoError> {
        let signing_key_bytes: [u8; 32] = keypair.classical_secret
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidSecretKey)?;
        let signing_key = SigningKey::from_bytes(&signing_key_bytes);
        let classical_sig = signing_key.sign(message);
        let pqc_sig = self.ml_dsa.sign(message, &keypair.pqc_secret)?;

        Ok(HybridSignatureData {
            classical_signature: classical_sig.to_bytes().to_vec(),
            pqc_signature: pqc_sig,
        })
    }

    pub fn verify(
        &self,
        message: &[u8],
        signature: &HybridSignatureData,
        public_key: &HybridSignaturePublicKey,
    ) -> Result<bool, CryptoError> {
        let verifying_key_bytes: [u8; 32] = public_key.classical_public
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidPublicKey)?;
        let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes)
            .map_err(|_| CryptoError::InvalidPublicKey)?;

        let ed25519_sig_bytes: [u8; 64] = signature.classical_signature
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidSignature)?;
        let ed25519_sig = Ed25519Signature::from_bytes(&ed25519_sig_bytes);
        let classical_valid = verifying_key.verify(message, &ed25519_sig).is_ok();
        let pqc_valid = self.ml_dsa.verify(message, &signature.pqc_signature, &public_key.pqc_public)?;

        Ok(classical_valid && pqc_valid)
    }
}
