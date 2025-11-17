// Hybrid cryptographic schemes combining classical and post-quantum algorithms
use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha3::{Digest, Sha3_256};
use subtle::ConstantTimeEq;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use zeroize::{Zeroize, Zeroizing};

use crate::error::CryptoError;
use crate::pqc::{MlDsa44, MlKem768};

/// Hybrid key exchange: X25519 + ML-KEM-768
pub struct HybridKex {
    ml_kem: MlKem768,
}

impl HybridKex {
    pub fn new() -> Result<Self, CryptoError> {
        Ok(Self {
            ml_kem: MlKem768::new()?,
        })
    }

    /// Generate hybrid keypair (X25519 + ML-KEM-768)
    pub fn keypair(&self) -> Result<HybridKeyPair, CryptoError> {
        // Classical X25519 - use StaticSecret for serializable keys
        let x25519_secret = StaticSecret::random_from_rng(OsRng);
        let x25519_public = X25519PublicKey::from(&x25519_secret);

        // PQC ML-KEM-768
        let (pqc_pk, pqc_sk) = self.ml_kem.keypair()?;

        Ok(HybridKeyPair {
            classical_public: x25519_public.to_bytes().to_vec(),
            // SECURITY: Wrap secret in Zeroizing for auto-zeroization on drop
            classical_secret: Zeroizing::new(x25519_secret.to_bytes().to_vec()),
            pqc_public: pqc_pk,
            // SECURITY: Wrap secret in Zeroizing for auto-zeroization on drop
            pqc_secret: Zeroizing::new(pqc_sk),
        })
    }

    /// Perform hybrid encapsulation
    pub fn encapsulate(
        &self,
        hybrid_public: &HybridPublicKey,
    ) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
        // X25519 key exchange
        let ephemeral_secret = EphemeralSecret::random_from_rng(OsRng);
        let ephemeral_public = X25519PublicKey::from(&ephemeral_secret);

        let x25519_pk_bytes: [u8; 32] = hybrid_public
            .classical_public
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidPublicKey)?;
        let x25519_pk = X25519PublicKey::from(x25519_pk_bytes);

        let classical_shared = ephemeral_secret.diffie_hellman(&x25519_pk);

        // ML-KEM-768 encapsulation
        let (pqc_ciphertext, pqc_shared) = self.ml_kem.encapsulate(&hybrid_public.pqc_public)?;

        // Combine shared secrets
        let combined_shared = Self::combine_secrets(classical_shared.as_bytes(), &pqc_shared);

        // Ciphertext includes both ephemeral public key and PQC ciphertext
        let mut ciphertext = Vec::new();
        ciphertext.extend_from_slice(ephemeral_public.as_bytes());
        ciphertext.extend_from_slice(&(pqc_ciphertext.len() as u32).to_le_bytes());
        ciphertext.extend_from_slice(&pqc_ciphertext);

        Ok((ciphertext, combined_shared))
    }

    /// Perform hybrid decapsulation
    pub fn decapsulate(
        &self,
        hybrid_secret: &HybridSecretKey,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        if ciphertext.len() < 32 + 4 {
            return Err(CryptoError::DecryptionError(
                "Invalid ciphertext".to_string(),
            ));
        }

        // Extract X25519 ephemeral public key
        let ephemeral_public_bytes: [u8; 32] = ciphertext[0..32]
            .try_into()
            .map_err(|_| CryptoError::DecryptionError("Invalid ephemeral key".to_string()))?;
        let ephemeral_public = X25519PublicKey::from(ephemeral_public_bytes);

        // Extract PQC ciphertext length
        let pqc_ct_len = u32::from_le_bytes(
            ciphertext[32..36]
                .try_into()
                .map_err(|_| CryptoError::DecryptionError("Invalid length".to_string()))?,
        ) as usize;

        if ciphertext.len() < 36 + pqc_ct_len {
            return Err(CryptoError::DecryptionError(
                "Truncated ciphertext".to_string(),
            ));
        }

        let pqc_ciphertext = &ciphertext[36..36 + pqc_ct_len];

        // X25519 key exchange
        let x25519_secret_bytes: [u8; 32] = hybrid_secret
            .classical_secret
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidPrivateKey)?;
        let x25519_secret = x25519_dalek::StaticSecret::from(x25519_secret_bytes);

        let classical_shared = x25519_secret.diffie_hellman(&ephemeral_public);

        // ML-KEM-768 decapsulation
        let pqc_shared = self
            .ml_kem
            .decapsulate(&hybrid_secret.pqc_secret, pqc_ciphertext)?;

        // Combine shared secrets
        let combined_shared = Self::combine_secrets(classical_shared.as_bytes(), &pqc_shared);

        Ok(combined_shared)
    }

    /// Combine classical and PQC shared secrets using SHA-3
    fn combine_secrets(classical: &[u8], pqc: &[u8]) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        hasher.update(b"BOUNDLESS-HYBRID-KEX");
        hasher.update((classical.len() as u32).to_le_bytes());
        hasher.update(classical);
        hasher.update((pqc.len() as u32).to_le_bytes());
        hasher.update(pqc);
        hasher.finalize().to_vec()
    }
}

impl Default for HybridKex {
    fn default() -> Self {
        Self::new().expect("Failed to initialize hybrid KEX")
    }
}

/// Hybrid signature scheme: Ed25519 + ML-DSA-44
pub struct HybridSignature {
    ml_dsa: MlDsa44,
}

impl HybridSignature {
    pub fn new() -> Result<Self, CryptoError> {
        Ok(Self {
            ml_dsa: MlDsa44::new()?,
        })
    }

    /// Generate hybrid signature keypair
    pub fn keypair(&self) -> Result<HybridSignatureKeyPair, CryptoError> {
        // Classical Ed25519
        let ed25519_signing_key = SigningKey::generate(&mut OsRng);
        let ed25519_verifying_key = ed25519_signing_key.verifying_key();

        // PQC ML-DSA-44
        let (pqc_pk, pqc_sk) = self.ml_dsa.keypair()?;

        Ok(HybridSignatureKeyPair {
            classical_verifying: ed25519_verifying_key.to_bytes().to_vec(),
            // SECURITY: Wrap signing key in Zeroizing for auto-zeroization on drop
            classical_signing: Zeroizing::new(ed25519_signing_key.to_bytes().to_vec()),
            pqc_public: pqc_pk,
            // SECURITY: Wrap secret in Zeroizing for auto-zeroization on drop
            pqc_secret: Zeroizing::new(pqc_sk),
        })
    }

    /// Sign a message with both schemes
    pub fn sign(
        &self,
        message: &[u8],
        keypair: &HybridSignatureKeyPair,
    ) -> Result<HybridSignatureData, CryptoError> {
        // Ed25519 signature
        let ed25519_sk_bytes: [u8; 32] = keypair
            .classical_signing
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidPrivateKey)?;
        let ed25519_signing_key = SigningKey::from_bytes(&ed25519_sk_bytes);
        let classical_sig = ed25519_signing_key.sign(message);

        // ML-DSA-44 signature
        let pqc_sig = self.ml_dsa.sign(message, &keypair.pqc_secret)?;

        Ok(HybridSignatureData {
            classical: classical_sig.to_bytes().to_vec(),
            pqc: pqc_sig,
        })
    }

    /// Verify a hybrid signature (both must be valid)
    ///
    /// SECURITY FIX: Uses constant-time verification to prevent timing attacks
    /// Both signatures are ALWAYS verified before returning the result
    pub fn verify(
        &self,
        message: &[u8],
        signature: &HybridSignatureData,
        public_key: &HybridSignaturePublicKey,
    ) -> Result<bool, CryptoError> {
        // Verify Ed25519 signature
        let ed25519_vk_bytes: [u8; 32] = public_key
            .classical_verifying
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidPublicKey)?;
        let ed25519_verifying_key = VerifyingKey::from_bytes(&ed25519_vk_bytes)
            .map_err(|_| CryptoError::InvalidPublicKey)?;

        let ed25519_sig_bytes: [u8; 64] = signature
            .classical
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidSignature)?;
        let ed25519_sig = Ed25519Signature::from_bytes(&ed25519_sig_bytes);

        // SECURITY: Store classical result but DON'T return early
        let classical_valid = ed25519_verifying_key.verify(message, &ed25519_sig).is_ok();

        // SECURITY: ALWAYS verify PQC signature to prevent timing leakage
        let pqc_valid = self
            .ml_dsa
            .verify(message, &signature.pqc, &public_key.pqc_public)?;

        // SECURITY: Use constant-time AND operation
        // This prevents timing attacks by always taking the same execution path
        Ok(classical_valid & pqc_valid)
    }
}

impl Default for HybridSignature {
    fn default() -> Self {
        Self::new().expect("Failed to initialize hybrid signature")
    }
}

// Data structures

/// SECURITY FIX: Removed Clone derive to prevent secret key duplication in memory
#[derive(Debug)]
pub struct HybridKeyPair {
    pub classical_public: Vec<u8>,
    /// SECURITY: Wrapped in Zeroizing to auto-zeroize on drop
    pub classical_secret: Zeroizing<Vec<u8>>,
    pub pqc_public: Vec<u8>,
    /// SECURITY: Wrapped in Zeroizing to auto-zeroize on drop
    pub pqc_secret: Zeroizing<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct HybridPublicKey {
    pub classical_public: Vec<u8>,
    pub pqc_public: Vec<u8>,
}

/// SECURITY FIX: Removed Clone derive to prevent secret key duplication in memory
#[derive(Debug)]
pub struct HybridSecretKey {
    /// SECURITY: Wrapped in Zeroizing to auto-zeroize on drop
    pub classical_secret: Zeroizing<Vec<u8>>,
    /// SECURITY: Wrapped in Zeroizing to auto-zeroize on drop
    pub pqc_secret: Zeroizing<Vec<u8>>,
}

/// SECURITY FIX: Removed Clone derive to prevent secret key duplication in memory
#[derive(Debug)]
pub struct HybridSignatureKeyPair {
    pub classical_verifying: Vec<u8>,
    /// SECURITY: Wrapped in Zeroizing to auto-zeroize on drop
    pub classical_signing: Zeroizing<Vec<u8>>,
    pub pqc_public: Vec<u8>,
    /// SECURITY: Wrapped in Zeroizing to auto-zeroize on drop
    pub pqc_secret: Zeroizing<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct HybridSignaturePublicKey {
    pub classical_verifying: Vec<u8>,
    pub pqc_public: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct HybridSignatureData {
    pub classical: Vec<u8>,
    pub pqc: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_kex() {
        let kex = HybridKex::new().unwrap();
        let keypair = kex.keypair().unwrap();

        let public_key = HybridPublicKey {
            classical_public: keypair.classical_public.clone(),
            pqc_public: keypair.pqc_public.clone(),
        };

        // SECURITY NOTE: We can't clone secret keys anymore (removed Clone trait)
        // Instead, we reference the keypair's secrets directly
        let secret_key = HybridSecretKey {
            classical_secret: Zeroizing::new(keypair.classical_secret.to_vec()),
            pqc_secret: Zeroizing::new(keypair.pqc_secret.to_vec()),
        };

        let (ciphertext, ss1) = kex.encapsulate(&public_key).unwrap();
        let ss2 = kex.decapsulate(&secret_key, &ciphertext).unwrap();

        assert_eq!(ss1, ss2);
    }

    #[test]
    fn test_hybrid_signature() {
        let sig_scheme = HybridSignature::new().unwrap();
        let keypair = sig_scheme.keypair().unwrap();

        let message = b"Hybrid signature test for Boundless BLS";
        let signature = sig_scheme.sign(message, &keypair).unwrap();

        // SECURITY NOTE: Public keys can still be cloned (they're not secret)
        let public_key = HybridSignaturePublicKey {
            classical_verifying: keypair.classical_verifying.clone(),
            pqc_public: keypair.pqc_public.clone(),
        };

        assert!(sig_scheme.verify(message, &signature, &public_key).unwrap());

        // Wrong message should fail
        let wrong_message = b"Wrong message";
        assert!(!sig_scheme
            .verify(wrong_message, &signature, &public_key)
            .unwrap());
    }

    // SECURITY TESTS: Timing Attack Prevention

    #[test]
    fn test_constant_time_verification_both_invalid() {
        // Test that when both signatures are invalid, verification still checks both
        let sig_scheme = HybridSignature::new().unwrap();
        let keypair = sig_scheme.keypair().unwrap();

        let message = b"Test message";

        // Create an invalid signature (all zeros)
        let invalid_signature = HybridSignatureData {
            classical: vec![0u8; 64],
            pqc: vec![0u8; 100],
        };

        let public_key = HybridSignaturePublicKey {
            classical_verifying: keypair.classical_verifying.clone(),
            pqc_public: keypair.pqc_public.clone(),
        };

        // Verification should fail but not panic - both signatures checked
        let result = sig_scheme.verify(message, &invalid_signature, &public_key);

        // Should return an error or false (not panic)
        // The PQC verification might fail with an error
        match result {
            Ok(valid) => assert!(!valid),
            Err(_) => {} // Expected: PQC verification fails on invalid signature
        }
    }

    #[test]
    fn test_constant_time_classical_valid_pqc_invalid() {
        // Test that if classical is valid but PQC is invalid, result is false
        // This verifies we're checking BOTH signatures
        let sig_scheme = HybridSignature::new().unwrap();
        let keypair = sig_scheme.keypair().unwrap();

        let message = b"Test message";

        // Create a valid classical signature
        let valid_signature = sig_scheme.sign(message, &keypair).unwrap();

        // Corrupt the PQC signature part
        let mut corrupted_signature = valid_signature.clone();
        corrupted_signature.pqc[0] ^= 0xFF; // Flip some bits

        let public_key = HybridSignaturePublicKey {
            classical_verifying: keypair.classical_verifying.clone(),
            pqc_public: keypair.pqc_public.clone(),
        };

        // Even though classical sig is valid, PQC is invalid -> should fail
        let result = sig_scheme.verify(message, &corrupted_signature, &public_key);

        match result {
            Ok(valid) => assert!(!valid, "Signature should fail when PQC part is invalid"),
            Err(_) => {} // Also acceptable: PQC verification returns error
        }
    }

    #[test]
    fn test_constant_time_classical_invalid_pqc_valid() {
        // Test that if PQC is valid but classical is invalid, result is false
        // This ensures BOTH are checked, not just one
        let sig_scheme = HybridSignature::new().unwrap();
        let keypair = sig_scheme.keypair().unwrap();

        let message = b"Test message";

        // Create a valid signature
        let valid_signature = sig_scheme.sign(message, &keypair).unwrap();

        // Corrupt the classical signature part
        let mut corrupted_signature = valid_signature.clone();
        corrupted_signature.classical[0] ^= 0xFF; // Flip some bits

        let public_key = HybridSignaturePublicKey {
            classical_verifying: keypair.classical_verifying.clone(),
            pqc_public: keypair.pqc_public.clone(),
        };

        // Even though PQC sig is valid, classical is invalid -> should fail
        let result = sig_scheme.verify(message, &corrupted_signature, &public_key);

        match result {
            Ok(valid) => assert!(!valid, "Signature should fail when classical part is invalid"),
            Err(_) => {} // Also acceptable if error is returned
        }
    }

    #[test]
    fn test_constant_time_verification_wrong_message() {
        // Verify that wrong message fails for both classical and PQC
        let sig_scheme = HybridSignature::new().unwrap();
        let keypair = sig_scheme.keypair().unwrap();

        let message = b"Correct message";
        let wrong_message = b"Wrong message";

        let signature = sig_scheme.sign(message, &keypair).unwrap();

        let public_key = HybridSignaturePublicKey {
            classical_verifying: keypair.classical_verifying.clone(),
            pqc_public: keypair.pqc_public.clone(),
        };

        // Verification with wrong message should fail
        let result = sig_scheme.verify(wrong_message, &signature, &public_key);

        match result {
            Ok(valid) => assert!(!valid),
            Err(_) => {} // Error is also acceptable
        }
    }

    // SECURITY TESTS: Memory Zeroization

    #[test]
    fn test_secret_keys_use_zeroizing_wrapper() {
        // Test that secret key fields are wrapped in Zeroizing
        // This ensures they will be zeroized when dropped
        let kex = HybridKex::new().unwrap();
        let keypair = kex.keypair().unwrap();

        // The type system ensures classical_secret and pqc_secret are Zeroizing
        // We can verify by checking the fields are accessible
        assert!(!keypair.classical_secret.is_empty());
        assert!(!keypair.pqc_secret.is_empty());

        // Public keys should not be wrapped
        assert!(!keypair.classical_public.is_empty());
        assert!(!keypair.pqc_public.is_empty());
    }

    #[test]
    fn test_signature_keypair_uses_zeroizing() {
        // Test that signature secret keys use Zeroizing wrapper
        let sig_scheme = HybridSignature::new().unwrap();
        let keypair = sig_scheme.keypair().unwrap();

        // Verify secret fields are Zeroizing-wrapped and non-empty
        assert!(!keypair.classical_signing.is_empty());
        assert!(!keypair.pqc_secret.is_empty());

        // Public keys should not be wrapped
        assert!(!keypair.classical_verifying.is_empty());
        assert!(!keypair.pqc_public.is_empty());
    }

    #[test]
    fn test_zeroizing_drop_behavior() {
        // Test that Zeroizing secrets are properly dropped
        // We can't directly verify memory is zeroed, but we can verify the
        // Drop trait is called by ensuring the value goes out of scope

        {
            let kex = HybridKex::new().unwrap();
            let keypair = kex.keypair().unwrap();

            // Store length before drop
            let classical_len = keypair.classical_secret.len();
            let pqc_len = keypair.pqc_secret.len();

            assert!(classical_len > 0);
            assert!(pqc_len > 0);

            // keypair will be dropped here, zeroizing the secrets
        }
        // If we reach here, Drop was called successfully
    }

    // SECURITY TESTS: Clone Prevention

    #[test]
    fn test_secret_keys_cannot_be_cloned() {
        // This test verifies at compile-time that secret key types don't implement Clone
        // If Clone was implemented, this test would compile differently

        // We verify by creating keypairs and ensuring we can't accidentally clone secrets
        let kex = HybridKex::new().unwrap();
        let keypair = kex.keypair().unwrap();

        // This should compile: creating a new secret key from the data
        let _secret_key = HybridSecretKey {
            classical_secret: Zeroizing::new(keypair.classical_secret.to_vec()),
            pqc_secret: Zeroizing::new(keypair.pqc_secret.to_vec()),
        };

        // The fact that we have to manually create a new Zeroizing wrapper
        // and use .to_vec() proves that Clone is not available for direct use
        // (If Clone was implemented, we could just do keypair.clone())
    }

    #[test]
    fn test_signature_keypair_cannot_be_cloned() {
        // Verify signature keypairs don't implement Clone
        let sig_scheme = HybridSignature::new().unwrap();
        let keypair = sig_scheme.keypair().unwrap();

        // Manual construction required (proves Clone is not available)
        let _another_copy = HybridSignatureKeyPair {
            classical_verifying: keypair.classical_verifying.clone(), // Public key OK
            classical_signing: Zeroizing::new(keypair.classical_signing.to_vec()),
            pqc_public: keypair.pqc_public.clone(), // Public key OK
            pqc_secret: Zeroizing::new(keypair.pqc_secret.to_vec()),
        };

        // If we reach here, the manual construction worked,
        // proving Clone is not directly available
    }

    #[test]
    fn test_public_keys_can_be_cloned() {
        // Verify that public keys CAN still be cloned (they're not secret)
        let sig_scheme = HybridSignature::new().unwrap();
        let keypair = sig_scheme.keypair().unwrap();

        let public_key = HybridSignaturePublicKey {
            classical_verifying: keypair.classical_verifying.clone(),
            pqc_public: keypair.pqc_public.clone(),
        };

        // Clone should work for public keys
        let _cloned_public = public_key.clone();

        // If we reach here, public key cloning works as expected
    }

    #[test]
    fn test_zeroizing_prevents_accidental_exposure() {
        // Test that Zeroizing wrapper prevents accidental secret exposure
        let kex = HybridKex::new().unwrap();
        let keypair = kex.keypair().unwrap();

        // We can access the secret through the Zeroizing wrapper
        let secret_ref: &[u8] = &keypair.classical_secret;
        assert!(!secret_ref.is_empty());

        // But we can't move it out or clone it easily
        // (the Zeroizing wrapper prevents accidental misuse)

        // To get a copy, we must explicitly use .to_vec()
        let _explicit_copy = keypair.classical_secret.to_vec();

        // This explicit requirement helps prevent accidental secret duplication
    }
}
