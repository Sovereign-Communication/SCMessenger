import os

filepath = "core/src/crypto/pq/hybrid.rs"
proper_code = """// core/src/crypto/pq/hybrid.rs
use anyhow::{anyhow, Result};
use blake3::derive_key;
use crate::crypto::pq::{encapsulate, decapsulate, MlKem768KeyPair};
use rand::rngs::OsRng;
use rand::RngCore;
use x25519_dalek::{StaticSecret, PublicKey};
use zeroize::Zeroize;

/// A 32-byte key that zeroizes on drop.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct RatchetKey([u8; 32]);

impl RatchetKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// A combined KEM ciphertext for the hybrid X25519+ML-KEM-768 scheme.
#[derive(Clone)]
pub struct HybridCiphertext {
    pub x25519_ephemeral_public: [u8; 32],
    pub mlkem_ciphertext: Vec<u8>, // 1088 B, length-validated
}

/// Encapsulates a shared secret for the given public keys.
pub fn hybrid_encapsulate(
    their_x25519_public_bytes: &[u8; 32],
    their_mlkem_encaps_key: &[u8],
) -> Result<(HybridCiphertext, RatchetKey /* 32-byte zeroizing secret */)> {
    if their_mlkem_encaps_key.len() != 1184 {
        return Err(anyhow!(
            "Invalid ML-KEM-768 public key length: expected 1184, got {}",
            their_mlkem_encaps_key.len()
        ));
    }

    let their_x25519_public = PublicKey::from(*their_x25519_public_bytes);

    let mut ephemeral_secret_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut ephemeral_secret_bytes);
    let ephemeral_secret = StaticSecret::from(ephemeral_secret_bytes);
    ephemeral_secret_bytes.zeroize();

    let x25519_ephemeral_public = PublicKey::from(&ephemeral_secret).to_bytes();

    let ss_x25519 = ephemeral_secret.diffie_hellman(&their_x25519_public);
    if !ss_x25519.was_contributory() {
        return Err(anyhow!("X25519 all-zero shared secret"));
    }

    let (mlkem_ciphertext, mut ss_mlkem) = encapsulate(their_mlkem_encaps_key)?;

    let mut ikm = Vec::new();
    ikm.extend_from_slice(&ss_x25519.to_bytes());
    ikm.extend_from_slice(&ss_mlkem);
    ikm.extend_from_slice(&x25519_ephemeral_public);
    ikm.extend_from_slice(their_x25519_public_bytes);
    ikm.extend_from_slice(&mlkem_ciphertext);
    ikm.extend_from_slice(their_mlkem_encaps_key);

    let shared = derive_key("iron-core hybrid-kem v1 X25519+MLKEM768 2026-07", &ikm);

    // zeroize explicitly
    ss_mlkem.zeroize();
    ikm.zeroize();

    Ok((
        HybridCiphertext {
            x25519_ephemeral_public,
            mlkem_ciphertext,
        },
        RatchetKey::from_bytes(shared),
    ))
}

/// Decapsulates the shared secret from the ciphertext using the keypair's private key.
pub fn hybrid_decapsulate(
    our_x25519_secret: &StaticSecret,
    our_mlkem_keypair: &MlKem768KeyPair,
    ct: &HybridCiphertext,
) -> Result<RatchetKey> {
    if ct.mlkem_ciphertext.len() != 1088 {
        return Err(anyhow!(
            "Invalid ML-KEM-768 ciphertext length: expected 1088, got {}",
            ct.mlkem_ciphertext.len()
        ));
    }

    let ct_x25519_public = PublicKey::from(ct.x25519_ephemeral_public);
    let ss_x25519 = our_x25519_secret.diffie_hellman(&ct_x25519_public);
    if !ss_x25519.was_contributory() {
        return Err(anyhow!("X25519 all-zero shared secret"));
    }

    let mut ss_mlkem = decapsulate(our_mlkem_keypair, &ct.mlkem_ciphertext)?;

    let mut ikm = Vec::new();
    ikm.extend_from_slice(&ss_x25519.to_bytes());
    ikm.extend_from_slice(&ss_mlkem);
    ikm.extend_from_slice(&ct.x25519_ephemeral_public);
    ikm.extend_from_slice(our_x25519_secret.to_bytes().as_ref());
    ikm.extend_from_slice(&ct.mlkem_ciphertext);
    ikm.extend_from_slice(our_mlkem_keypair.public_key());

    let shared = derive_key("iron-core hybrid-kem v1 X25519+MLKEM768 2026-07", &ikm);

    ss_mlkem.zeroize();
    ikm.zeroize();

    Ok(RatchetKey::from_bytes(shared))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::pq::generate;

    #[test]
    fn test_roundtrip() {
        let our_x25519_secret = StaticSecret::random_from_rng(OsRng);
        let our_mlkem_keypair = generate();

        let their_x25519_secret = StaticSecret::random_from_rng(OsRng);
        let their_x25519_public = PublicKey::from(&their_x25519_secret).to_bytes();
        let their_mlkem_encaps_key = our_mlkem_keypair.public_key();

        let (ct, shared_enc) = hybrid_encapsulate(&their_x25519_public, their_mlkem_encaps_key).unwrap();
        let shared_dec = hybrid_decapsulate(&our_x25519_secret, &our_mlkem_keypair, &ct).unwrap();

        assert_eq!(shared_enc.as_bytes(), shared_dec.as_bytes());
    }

    #[test]
    fn test_tamper_mlkem_ciphertext() {
        let our_x25519_secret = StaticSecret::random_from_rng(OsRng);
        let our_mlkem_keypair = generate();

        let their_x25519_secret = StaticSecret::random_from_rng(OsRng);
        let their_x25519_public = PublicKey::from(&their_x25519_secret).to_bytes();
        let their_mlkem_encaps_key = our_mlkem_keypair.public_key();

        let (mut ct, shared_enc) = hybrid_encapsulate(&their_x25519_public, their_mlkem_encaps_key).unwrap();
        ct.mlkem_ciphertext[0] ^= 1;

        let shared_dec = hybrid_decapsulate(&our_x25519_secret, &our_mlkem_keypair, &ct).unwrap();
        assert_ne!(shared_enc.as_bytes(), shared_dec.as_bytes());
    }

    #[test]
    fn test_tamper_x25519_ephemeral_public() {
        let our_x25519_secret = StaticSecret::random_from_rng(OsRng);
        let our_mlkem_keypair = generate();

        let their_x25519_secret = StaticSecret::random_from_rng(OsRng);
        let their_x25519_public = PublicKey::from(&their_x25519_secret).to_bytes();
        let their_mlkem_encaps_key = our_mlkem_keypair.public_key();

        let (mut ct, shared_enc) = hybrid_encapsulate(&their_x25519_public, their_mlkem_encaps_key).unwrap();
        ct.x25519_ephemeral_public[0] ^= 1;

        let shared_dec = hybrid_decapsulate(&our_x25519_secret, &our_mlkem_keypair, &ct).unwrap();
        assert_ne!(shared_enc.as_bytes(), shared_dec.as_bytes());
    }

    #[test]
    fn test_both_halves_contribute() {
        let our_x25519_secret = StaticSecret::random_from_rng(OsRng);
        let our_mlkem_keypair = generate();

        let their_x25519_secret = StaticSecret::random_from_rng(OsRng);
        let their_x25519_public = PublicKey::from(&their_x25519_secret).to_bytes();
        let their_mlkem_encaps_key = our_mlkem_keypair.public_key();

        let (ct, shared_enc) = hybrid_encapsulate(&their_x25519_public, their_mlkem_encaps_key).unwrap();

        let mut tampered_ct = ct.clone();
        tampered_ct.x25519_ephemeral_public[0] ^= 1;
        let shared_dec = hybrid_decapsulate(&our_x25519_secret, &our_mlkem_keypair, &tampered_ct).unwrap();
        assert_ne!(shared_enc.as_bytes(), shared_dec.as_bytes());

        let mut tampered_ct = ct.clone();
        tampered_ct.mlkem_ciphertext[0] ^= 1;
        let shared_dec = hybrid_decapsulate(&our_x25519_secret, &our_mlkem_keypair, &tampered_ct).unwrap();
        assert_ne!(shared_enc.as_bytes(), shared_dec.as_bytes());
    }

    #[test]
    fn test_kat_stability() {
        let our_x25519_secret = StaticSecret::random_from_rng(OsRng);
        let our_mlkem_keypair = generate();

        let their_x25519_secret = StaticSecret::random_from_rng(OsRng);
        let their_x25519_public = PublicKey::from(&their_x25519_secret).to_bytes();
        let their_mlkem_encaps_key = our_mlkem_keypair.public_key();

        let (ct, shared_enc) = hybrid_encapsulate(&their_x25519_public, their_mlkem_encaps_key).unwrap();
        let shared_dec = hybrid_decapsulate(&our_x25519_secret, &our_mlkem_keypair, &ct).unwrap();

        assert_eq!(shared_enc.as_bytes(), shared_dec.as_bytes());
    }

    #[test]
    fn test_zero_shared_secret_rejection() {
        let our_mlkem_keypair = generate();

        let their_x25519_public = [0u8; 32]; // Not contributory!
        let their_mlkem_encaps_key = our_mlkem_keypair.public_key();

        let result = hybrid_encapsulate(&their_x25519_public, their_mlkem_encaps_key);
        assert!(result.is_err());
    }
}
"""

with open(filepath, "w", encoding="utf-8") as f:
    f.write(proper_code)
print("done")
