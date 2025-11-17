// Boundless Enterprise - Encrypted Keystore
//
// Stores private keys encrypted with AES-256-GCM
// Master encryption key must be set in MASTER_ENCRYPTION_KEY environment variable
//
// SECURITY: Generate master key with: openssl rand -hex 32

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;
use zeroize::Zeroizing;

use crate::error::{EnterpriseError, Result};

/// Encrypted key material
#[derive(Clone)]
pub struct EncryptedKey {
    /// Base64-encoded ciphertext
    pub ciphertext: String,
    /// Base64-encoded nonce (96 bits / 12 bytes)
    pub nonce: String,
}

/// Keystore for encrypting and decrypting private keys
pub struct Keystore {
    cipher: Aes256Gcm,
}

impl Keystore {
    /// Initialize keystore with master encryption key from environment
    pub fn new() -> Result<Self> {
        let master_key_hex = std::env::var("MASTER_ENCRYPTION_KEY")
            .map_err(|_| EnterpriseError::CryptoError(
                "MASTER_ENCRYPTION_KEY environment variable not set. Generate with: openssl rand -hex 32".to_string()
            ))?;

        Self::from_hex_key(&master_key_hex)
    }

    /// Initialize keystore with a hex-encoded master key
    pub fn from_hex_key(master_key_hex: &str) -> Result<Self> {
        // Decode hex master key
        let master_key = hex::decode(master_key_hex)
            .map_err(|e| EnterpriseError::CryptoError(
                format!("Invalid MASTER_ENCRYPTION_KEY hex: {}", e)
            ))?;

        if master_key.len() != 32 {
            return Err(EnterpriseError::CryptoError(
                format!("MASTER_ENCRYPTION_KEY must be 32 bytes (64 hex chars), got {}", master_key.len())
            ));
        }

        // Create AES-256-GCM cipher
        let cipher = Aes256Gcm::new_from_slice(&master_key)
            .map_err(|e| EnterpriseError::CryptoError(
                format!("Failed to initialize AES-256-GCM: {}", e)
            ))?;

        Ok(Self { cipher })
    }

    /// Encrypt a private key for storage
    pub fn encrypt_key(&self, private_key: &[u8]) -> Result<EncryptedKey> {
        // Generate random nonce (96 bits for GCM)
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the private key
        let ciphertext = self.cipher.encrypt(nonce, private_key)
            .map_err(|e| EnterpriseError::CryptoError(
                format!("Encryption failed: {}", e)
            ))?;

        // Encode to base64 for storage
        Ok(EncryptedKey {
            ciphertext: BASE64.encode(&ciphertext),
            nonce: BASE64.encode(&nonce_bytes),
        })
    }

    /// Decrypt a private key from storage
    pub fn decrypt_key(&self, encrypted: &EncryptedKey) -> Result<Zeroizing<Vec<u8>>> {
        // Decode from base64
        let ciphertext = BASE64.decode(&encrypted.ciphertext)
            .map_err(|e| EnterpriseError::CryptoError(
                format!("Invalid ciphertext base64: {}", e)
            ))?;

        let nonce_bytes = BASE64.decode(&encrypted.nonce)
            .map_err(|e| EnterpriseError::CryptoError(
                format!("Invalid nonce base64: {}", e)
            ))?;

        if nonce_bytes.len() != 12 {
            return Err(EnterpriseError::CryptoError(
                format!("Invalid nonce size: expected 12 bytes, got {}", nonce_bytes.len())
            ));
        }

        let nonce = Nonce::from_slice(&nonce_bytes);

        // Decrypt the private key
        let plaintext = self.cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| EnterpriseError::CryptoError(
                format!("Decryption failed (wrong key or corrupted data): {}", e)
            ))?;

        // Return with automatic zeroing on drop
        Ok(Zeroizing::new(plaintext))
    }

    /// Encrypt multiple key components (for key pairs with multiple parts)
    pub fn encrypt_keypair(&self, public_key: &[u8], private_key: &[u8]) -> Result<(Vec<u8>, EncryptedKey)> {
        let encrypted_private = self.encrypt_key(private_key)?;

        Ok((
            public_key.to_vec(),
            encrypted_private,
        ))
    }

    /// Re-encrypt a key with a new master key (for key rotation)
    pub fn reencrypt_key(
        &self,
        encrypted: &EncryptedKey,
        new_keystore: &Keystore,
    ) -> Result<EncryptedKey> {
        // Decrypt with old key
        let plaintext = self.decrypt_key(encrypted)?;

        // Re-encrypt with new key
        new_keystore.encrypt_key(&plaintext)
    }
}

/// Default implementation tries to load from environment
impl Default for Keystore {
    fn default() -> Self {
        Self::new().expect("Failed to initialize keystore: MASTER_ENCRYPTION_KEY not set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_master_key() -> String {
        // Generate a test master key (32 bytes = 64 hex chars)
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()
    }

    #[test]
    fn test_keystore_initialization() {
        let keystore = Keystore::from_hex_key(&test_master_key()).unwrap();
        // If we get here, initialization succeeded
        assert!(true);
    }

    #[test]
    fn test_encrypt_decrypt_key() {
        let keystore = Keystore::from_hex_key(&test_master_key()).unwrap();
        let original_key = b"this_is_a_test_private_key_12345678";

        // Encrypt
        let encrypted = keystore.encrypt_key(original_key).unwrap();
        assert!(!encrypted.ciphertext.is_empty());
        assert!(!encrypted.nonce.is_empty());

        // Decrypt
        let decrypted = keystore.decrypt_key(&encrypted).unwrap();
        assert_eq!(&*decrypted, original_key);
    }

    #[test]
    fn test_different_nonces() {
        let keystore = Keystore::from_hex_key(&test_master_key()).unwrap();
        let key = b"test_key";

        // Encrypt the same key twice
        let encrypted1 = keystore.encrypt_key(key).unwrap();
        let encrypted2 = keystore.encrypt_key(key).unwrap();

        // Nonces should be different (probabilistically)
        assert_ne!(encrypted1.nonce, encrypted2.nonce);

        // But both should decrypt to the same value
        let decrypted1 = keystore.decrypt_key(&encrypted1).unwrap();
        let decrypted2 = keystore.decrypt_key(&encrypted2).unwrap();
        assert_eq!(&*decrypted1, key);
        assert_eq!(&*decrypted2, key);
    }

    #[test]
    fn test_wrong_key_fails() {
        let keystore1 = Keystore::from_hex_key(&test_master_key()).unwrap();
        let keystore2 = Keystore::from_hex_key(
            "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
        ).unwrap();

        let key = b"secret_key";
        let encrypted = keystore1.encrypt_key(key).unwrap();

        // Decrypting with wrong keystore should fail
        let result = keystore2.decrypt_key(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_reencryption() {
        let keystore1 = Keystore::from_hex_key(&test_master_key()).unwrap();
        let keystore2 = Keystore::from_hex_key(
            "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
        ).unwrap();

        let original_key = b"rotate_this_key";

        // Encrypt with first keystore
        let encrypted1 = keystore1.encrypt_key(original_key).unwrap();

        // Re-encrypt with second keystore
        let encrypted2 = keystore1.reencrypt_key(&encrypted1, &keystore2).unwrap();

        // Decrypt with second keystore
        let decrypted = keystore2.decrypt_key(&encrypted2).unwrap();
        assert_eq!(&*decrypted, original_key);
    }

    #[test]
    fn test_keypair_encryption() {
        let keystore = Keystore::from_hex_key(&test_master_key()).unwrap();
        let public_key = b"public_key_bytes_here";
        let private_key = b"private_key_bytes_here";

        let (pub_stored, priv_encrypted) = keystore
            .encrypt_keypair(public_key, private_key)
            .unwrap();

        // Public key stored unencrypted
        assert_eq!(&pub_stored, public_key);

        // Private key can be decrypted
        let priv_decrypted = keystore.decrypt_key(&priv_encrypted).unwrap();
        assert_eq!(&*priv_decrypted, private_key);
    }
}
