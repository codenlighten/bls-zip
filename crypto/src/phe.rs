// Partially Homomorphic Encryption using Paillier cryptosystem
// Custom implementation to avoid dependency conflicts with libp2p

use num_bigint::{BigUint, RandBigInt};
use num_traits::{One, Zero};
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use crate::error::CryptoError;

/// Paillier public key for encryption
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptionKey {
    n: BigUint,      // n = p * q
    g: BigUint,      // g = n + 1
    n_squared: BigUint,
}

/// Paillier private key for decryption
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecryptionKey {
    lambda: BigUint,  // λ = lcm(p-1, q-1)
    mu: BigUint,      // μ = (L(g^λ mod n²))^(-1) mod n
    n: BigUint,
    n_squared: BigUint,
}

/// Encrypted ciphertext
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ciphertext {
    value: BigUint,
}

/// Paillier PHE wrapper for privacy-preserving computations
pub struct PaillierPhe {
    key_size: usize,
}

impl PaillierPhe {
    /// Create a new Paillier instance with specified key size (bits)
    pub fn new(key_size: usize) -> Self {
        if key_size < 512 {
            panic!("Key size must be at least 512 bits for security");
        }
        Self { key_size }
    }

    /// Generate a Paillier keypair
    pub fn keypair(&self) -> Result<(EncryptionKey, DecryptionKey), CryptoError> {
        let mut rng = thread_rng();
        let half_size = self.key_size / 2;

        // Generate two large primes p and q
        let p = Self::generate_prime(&mut rng, half_size);
        let q = Self::generate_prime(&mut rng, half_size);

        // Compute n = p * q
        let n = &p * &q;
        let n_squared = &n * &n;

        // Compute λ = lcm(p-1, q-1)
        let p_minus_1 = &p - BigUint::one();
        let q_minus_1 = &q - BigUint::one();
        let lambda = Self::lcm(&p_minus_1, &q_minus_1);

        // g = n + 1 (simplified approach for better performance)
        let g = &n + BigUint::one();

        // Compute μ = (L(g^λ mod n²))^(-1) mod n
        let g_lambda = g.modpow(&lambda, &n_squared);
        let l_result = Self::l_function(&g_lambda, &n);
        let mu = Self::mod_inverse(&l_result, &n)?;

        let ek = EncryptionKey {
            n: n.clone(),
            g,
            n_squared: n_squared.clone(),
        };

        let dk = DecryptionKey {
            lambda,
            mu,
            n,
            n_squared,
        };

        Ok((ek, dk))
    }

    /// Encrypt a value
    pub fn encrypt(
        &self,
        plaintext: u64,
        encryption_key: &EncryptionKey,
    ) -> Result<Ciphertext, CryptoError> {
        let mut rng = thread_rng();
        let m = BigUint::from(plaintext);

        // Ensure plaintext is less than n
        if m >= encryption_key.n {
            return Err(CryptoError::PheError(
                "Plaintext must be less than n".to_string(),
            ));
        }

        // Select random r where r < n and gcd(r, n) = 1
        let r = rng.gen_biguint_range(&BigUint::one(), &encryption_key.n);

        // c = g^m * r^n mod n²
        let g_m = encryption_key.g.modpow(&m, &encryption_key.n_squared);
        let r_n = r.modpow(&encryption_key.n, &encryption_key.n_squared);
        let c = (g_m * r_n) % &encryption_key.n_squared;

        Ok(Ciphertext { value: c })
    }

    /// Decrypt a ciphertext
    pub fn decrypt(
        &self,
        ciphertext: &Ciphertext,
        decryption_key: &DecryptionKey,
    ) -> Result<u64, CryptoError> {
        // m = L(c^λ mod n²) * μ mod n
        let c_lambda = ciphertext
            .value
            .modpow(&decryption_key.lambda, &decryption_key.n_squared);
        let l_result = Self::l_function(&c_lambda, &decryption_key.n);
        let m = (l_result * &decryption_key.mu) % &decryption_key.n;

        // Convert BigUint to u64
        let value: u64 = m
            .to_u64_digits()
            .get(0)
            .copied()
            .ok_or_else(|| {
                CryptoError::DecryptionError("Value out of range for u64".to_string())
            })?;

        Ok(value)
    }

    /// Homomorphic addition: E(m1) + E(m2) = E(m1 + m2)
    pub fn add(
        &self,
        encryption_key: &EncryptionKey,
        ct1: &Ciphertext,
        ct2: &Ciphertext,
    ) -> Ciphertext {
        let value = (&ct1.value * &ct2.value) % &encryption_key.n_squared;
        Ciphertext { value }
    }

    /// Homomorphic scalar multiplication: k * E(m) = E(k * m)
    pub fn mul(
        &self,
        encryption_key: &EncryptionKey,
        ciphertext: &Ciphertext,
        scalar: u64,
    ) -> Ciphertext {
        let k = BigUint::from(scalar);
        let value = ciphertext.value.modpow(&k, &encryption_key.n_squared);
        Ciphertext { value }
    }

    // Helper functions

    fn generate_prime(rng: &mut impl RandBigInt, bits: usize) -> BigUint {
        loop {
            let candidate = rng.gen_biguint(bits as u64);
            if Self::is_probably_prime(&candidate, 20) {
                return candidate;
            }
        }
    }

    fn is_probably_prime(n: &BigUint, k: usize) -> bool {
        use num_bigint::ToBigInt;

        if n <= &BigUint::one() {
            return false;
        }
        if n == &BigUint::from(2u32) || n == &BigUint::from(3u32) {
            return true;
        }
        if n % 2u32 == BigUint::zero() {
            return false;
        }

        // Write n-1 as 2^r * d
        let n_minus_1 = n - BigUint::one();
        let mut d = n_minus_1.clone();
        let mut r = 0u32;
        while &d % 2u32 == BigUint::zero() {
            d /= 2u32;
            r += 1;
        }

        let mut rng = thread_rng();
        'witness: for _ in 0..k {
            let a = rng.gen_biguint_range(&BigUint::from(2u32), &(n - BigUint::from(2u32)));
            let mut x = a.modpow(&d, n);

            if x == BigUint::one() || x == n_minus_1 {
                continue;
            }

            for _ in 0..r - 1 {
                x = x.modpow(&BigUint::from(2u32), n);
                if x == n_minus_1 {
                    continue 'witness;
                }
            }

            return false;
        }

        true
    }

    fn l_function(x: &BigUint, n: &BigUint) -> BigUint {
        (x - BigUint::one()) / n
    }

    fn gcd(a: &BigUint, b: &BigUint) -> BigUint {
        if b.is_zero() {
            a.clone()
        } else {
            Self::gcd(b, &(a % b))
        }
    }

    fn lcm(a: &BigUint, b: &BigUint) -> BigUint {
        (a * b) / Self::gcd(a, b)
    }

    fn mod_inverse(a: &BigUint, m: &BigUint) -> Result<BigUint, CryptoError> {
        use num_bigint::ToBigInt;
        use num_traits::Signed;

        let a_int = a.to_bigint().unwrap();
        let m_int = m.to_bigint().unwrap();

        let (mut t, mut new_t) = (num_bigint::BigInt::zero(), num_bigint::BigInt::one());
        let (mut r, mut new_r) = (m_int.clone(), a_int);

        while !new_r.is_zero() {
            let quotient = &r / &new_r;
            (t, new_t) = (new_t.clone(), t - &quotient * &new_t);
            (r, new_r) = (new_r.clone(), r - quotient * new_r);
        }

        if r > num_bigint::BigInt::one() {
            return Err(CryptoError::PheError(
                "Modular inverse does not exist".to_string(),
            ));
        }

        if t.is_negative() {
            t = t + m_int;
        }

        Ok(t.to_biguint().unwrap())
    }
}

impl Default for PaillierPhe {
    fn default() -> Self {
        Self::new(2048)
    }
}

/// Privacy-preserving sum aggregation
pub struct PrivateAggregator {
    phe: PaillierPhe,
}

impl PrivateAggregator {
    pub fn new() -> Self {
        Self {
            phe: PaillierPhe::default(),
        }
    }

    /// Compute sum of encrypted values without decrypting
    pub fn sum_encrypted(
        &self,
        encryption_key: &EncryptionKey,
        ciphertexts: &[Ciphertext],
    ) -> Result<Ciphertext, CryptoError> {
        if ciphertexts.is_empty() {
            return Err(CryptoError::PheError("Empty ciphertext list".to_string()));
        }

        let mut result = ciphertexts[0].clone();
        for ct in &ciphertexts[1..] {
            result = self.phe.add(encryption_key, &result, ct);
        }

        Ok(result)
    }

    /// Compute weighted sum: Σ(w_i * E(x_i))
    pub fn weighted_sum(
        &self,
        encryption_key: &EncryptionKey,
        ciphertexts: &[Ciphertext],
        weights: &[u64],
    ) -> Result<Ciphertext, CryptoError> {
        if ciphertexts.len() != weights.len() {
            return Err(CryptoError::PheError("Mismatched lengths".to_string()));
        }

        if ciphertexts.is_empty() {
            return Err(CryptoError::PheError("Empty input".to_string()));
        }

        let mut result = self.phe.mul(encryption_key, &ciphertexts[0], weights[0]);

        for i in 1..ciphertexts.len() {
            let weighted = self.phe.mul(encryption_key, &ciphertexts[i], weights[i]);
            result = self.phe.add(encryption_key, &result, &weighted);
        }

        Ok(result)
    }
}

impl Default for PrivateAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paillier_encryption() {
        let phe = PaillierPhe::new(1024);
        let (ek, dk) = phe.keypair().unwrap();

        let plaintext = 42u64;
        let ciphertext = phe.encrypt(plaintext, &ek).unwrap();
        let decrypted = phe.decrypt(&ciphertext, &dk).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_homomorphic_addition() {
        let phe = PaillierPhe::new(1024);
        let (ek, dk) = phe.keypair().unwrap();

        let m1 = 15u64;
        let m2 = 27u64;

        let ct1 = phe.encrypt(m1, &ek).unwrap();
        let ct2 = phe.encrypt(m2, &ek).unwrap();

        let ct_sum = phe.add(&ek, &ct1, &ct2);
        let sum = phe.decrypt(&ct_sum, &dk).unwrap();

        assert_eq!(sum, m1 + m2);
    }

    #[test]
    fn test_homomorphic_multiplication() {
        let phe = PaillierPhe::new(1024);
        let (ek, dk) = phe.keypair().unwrap();

        let m = 7u64;
        let k = 6u64;

        let ct = phe.encrypt(m, &ek).unwrap();
        let ct_mul = phe.mul(&ek, &ct, k);
        let result = phe.decrypt(&ct_mul, &dk).unwrap();

        assert_eq!(result, m * k);
    }

    #[test]
    fn test_private_aggregation() {
        let aggregator = PrivateAggregator::new();
        let phe = PaillierPhe::new(1024);
        let (ek, dk) = phe.keypair().unwrap();

        let values = vec![10u64, 20, 30, 40];
        let ciphertexts: Vec<_> = values
            .iter()
            .map(|&v| phe.encrypt(v, &ek).unwrap())
            .collect();

        let ct_sum = aggregator.sum_encrypted(&ek, &ciphertexts).unwrap();
        let sum = phe.decrypt(&ct_sum, &dk).unwrap();

        assert_eq!(sum, values.iter().sum());
    }

    #[test]
    fn test_weighted_sum() {
        let aggregator = PrivateAggregator::new();
        let phe = PaillierPhe::new(1024);
        let (ek, dk) = phe.keypair().unwrap();

        let values = vec![5u64, 10, 15];
        let weights = vec![2u64, 3, 4];

        let ciphertexts: Vec<_> = values
            .iter()
            .map(|&v| phe.encrypt(v, &ek).unwrap())
            .collect();

        let ct_weighted = aggregator.weighted_sum(&ek, &ciphertexts, &weights).unwrap();
        let result = phe.decrypt(&ct_weighted, &dk).unwrap();

        let expected: u64 = values.iter().zip(&weights).map(|(v, w)| v * w).sum();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_large_values() {
        let phe = PaillierPhe::new(2048);
        let (ek, dk) = phe.keypair().unwrap();

        let m1 = 1_000_000u64;
        let m2 = 2_000_000u64;

        let ct1 = phe.encrypt(m1, &ek).unwrap();
        let ct2 = phe.encrypt(m2, &ek).unwrap();

        let ct_sum = phe.add(&ek, &ct1, &ct2);
        let sum = phe.decrypt(&ct_sum, &dk).unwrap();

        assert_eq!(sum, m1 + m2);
    }
}
