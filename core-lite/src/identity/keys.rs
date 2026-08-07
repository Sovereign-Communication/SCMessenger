// Cryptographic key management

use anyhow::Result;
use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Prefix for public key hex in logs and payloads to distinguish from identity_id
pub const PUBLIC_KEY_PREFIX: &str = "pk:";
/// Prefix for identity_id (blake3 hash) in logs and payloads to distinguish from public_key_hex
pub const IDENTITY_ID_PREFIX: &str = "id:";

/// Check if a 64-hex string looks like a public key (valid Ed25519 curve point)
pub fn is_valid_public_key(hex_str: &str) -> bool {
    if hex_str.len() != 64 || !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }
    if let Ok(bytes) = hex::decode(hex_str) {
        if let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice()) {
            return ed25519_dalek::VerifyingKey::from_bytes(&arr).is_ok();
        }
    }
    false
}

/// Check if a 64-hex string is a valid identity_id format (always valid if 64 hex chars)
pub fn is_valid_identity_id(hex_str: &str) -> bool {
    hex_str.len() == 64 && hex_str.chars().all(|c| c.is_ascii_hexdigit())
}

/// Derive the identity_id from a hex-encoded Ed25519 public key.
///
/// This is the single source of truth for the public_key -> identity_id
/// derivation, so callers outside this module never reimplement the hash.
/// Returns `None` when the input is not a valid Ed25519 public key: the
/// relation is one-way, so an identity_id can never be reversed back into a
/// public key, and hashing an arbitrary 64-hex string (e.g. an identity_id
/// itself, double-hashing) must not be allowed.
pub fn identity_id_from_public_key_hex(public_key_hex: &str) -> Option<String> {
    if !is_valid_public_key(public_key_hex) {
        return None;
    }
    let bytes = hex::decode(public_key_hex).ok()?;
    Some(hex::encode(blake3::hash(&bytes).as_bytes()))
}

/// Get the type of a 64-hex identifier for logging/debugging
pub fn identify_key_type(hex_str: &str) -> &'static str {
    if is_valid_public_key(hex_str) {
        "public_key"
    } else if is_valid_identity_id(hex_str) {
        "identity_id"
    } else {
        "unknown"
    }
}

/// Key pair for signing and verification
#[derive(Clone)]
pub struct KeyPair {
    pub signing_key: SigningKey,
}

impl KeyPair {
    /// Generate a new random key pair
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut secret_key_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret_key_bytes);
        let signing_key = SigningKey::from_bytes(&secret_key_bytes);
        secret_key_bytes.zeroize();
        Self { signing_key }
    }

    /// Get verifying key
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }
}

/// Serializable format for identity keys (V1 = Ed25519 + X25519)
#[derive(Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
struct IdentityKeysRaw {
    signing_key_bytes: [u8; 32],
    x25519_secret_bytes: [u8; 32],
}

/// Identity keys (signing + dedicated encryption)
#[derive(Clone)]
pub struct IdentityKeys {
    pub signing_key: SigningKey,
    pub x25519_encryption_secret: x25519_dalek::StaticSecret,
}

impl IdentityKeys {
    /// Generate new identity keys
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut secret_key_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret_key_bytes);
        let signing_key = SigningKey::from_bytes(&secret_key_bytes);
        secret_key_bytes.zeroize();

        let mut x25519_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut x25519_bytes);
        let x25519_encryption_secret = x25519_dalek::StaticSecret::from(x25519_bytes);
        x25519_bytes.zeroize();

        Self {
            signing_key,
            x25519_encryption_secret,
        }
    }

    /// Get public key as hex
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.signing_key.verifying_key().to_bytes())
    }

    /// Get public key as hex with prefix for logging distinction
    pub fn public_key_hex_prefixed(&self) -> String {
        format!("{}{}", PUBLIC_KEY_PREFIX, self.public_key_hex())
    }

    /// Get identity ID (Blake3 hash of public key)
    pub fn identity_id(&self) -> String {
        let public_key = self.signing_key.verifying_key().to_bytes();
        let hash = blake3::hash(&public_key);
        hex::encode(hash.as_bytes())
    }

    /// Get identity ID with prefix for logging distinction
    pub fn identity_id_prefixed(&self) -> String {
        format!("{}{}", IDENTITY_ID_PREFIX, self.identity_id())
    }

    /// Sign data with Ed25519
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let signature = self.signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    /// Verify Ed25519 signature
    pub fn verify(data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool> {
        let verifying_key = VerifyingKey::from_bytes(
            public_key
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid public key"))?,
        )?;

        let sig = Ed25519Signature::from_bytes(
            signature
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid signature"))?,
        );

        Ok(verifying_key.verify(data, &sig).is_ok())
    }

    /// Serialize keys to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let raw = IdentityKeysRaw {
            signing_key_bytes: self.signing_key.to_bytes(),
            x25519_secret_bytes: self.x25519_encryption_secret.to_bytes(),
        };
        let mut serialized = bincode::serialize(&raw)
            .expect("bincode serialization of IdentityKeysRaw cannot fail");
        // Zeroize raw after serialization
        let mut raw = raw;
        raw.zeroize();
        
        let mut result = Vec::with_capacity(1 + serialized.len());
        result.push(0x01); // version tag
        result.append(&mut serialized);
        result
    }

    /// Deserialize keys from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() == 32 {
            // V1 legacy format (raw seed)
            let signing_key = SigningKey::from_bytes(
                bytes
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid V1 key bytes"))?,
            );

            let mut x25519_bytes = [0u8; 32];
            rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut x25519_bytes);
            let x25519_encryption_secret = x25519_dalek::StaticSecret::from(x25519_bytes);
            x25519_bytes.zeroize();

            Ok(Self {
                signing_key,
                x25519_encryption_secret,
            })
        } else if bytes.first() == Some(&0x01) {
            // V1 tagged format
            let raw: IdentityKeysRaw = bincode::deserialize(&bytes[1..])
                .map_err(|e| anyhow::anyhow!("Failed to deserialize V1 keys: {}", e))?;
            let signing_key = SigningKey::from_bytes(&raw.signing_key_bytes);
            let x25519_encryption_secret = x25519_dalek::StaticSecret::from(raw.x25519_secret_bytes);

            Ok(Self {
                signing_key,
                x25519_encryption_secret,
            })
        } else {
            Err(anyhow::anyhow!("Invalid identity keys format/tag"))
        }
    }

    /// Convert to libp2p Keypair for network identity
    pub fn to_libp2p_peer_id(&self) -> Result<String> {
        let pub_bytes = self.signing_key.verifying_key().to_bytes();
        match libp2p::identity::ed25519::PublicKey::try_from_bytes(&pub_bytes) {
            Ok(libp2p_pub) => Ok(libp2p::identity::PublicKey::from(libp2p_pub)
                .to_peer_id()
                .to_string()),
            Err(_) => {
                let kp = self.to_libp2p_keypair()?;
                Ok(kp.public().to_peer_id().to_string())
            }
        }
    }

    pub fn to_libp2p_keypair(&self) -> Result<libp2p::identity::Keypair> {
        let mut seed = self.signing_key.to_bytes();
        let ed25519_secret = libp2p::identity::ed25519::SecretKey::try_from_bytes(&mut seed)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to convert Ed25519 secret key to libp2p format: {}",
                    e
                )
            })?;
        let ed25519_keypair = libp2p::identity::ed25519::Keypair::from(ed25519_secret);
        Ok(libp2p::identity::Keypair::from(ed25519_keypair))
    }
}

/// Generate a Signal-style safety number from two public keys.
///
/// Returns a 60-digit numeric string (12 groups of 5 digits, space-separated).
/// The number is order-independent (sorted keys) so both sides display identically.
pub fn safety_number(our_pubkey_hex: &str, their_pubkey_hex: &str) -> Result<String> {
    let our_bytes = hex::decode(our_pubkey_hex)
        .map_err(|e| anyhow::anyhow!("Invalid our pubkey hex: {}", e))?;
    let their_bytes = hex::decode(their_pubkey_hex)
        .map_err(|e| anyhow::anyhow!("Invalid their pubkey hex: {}", e))?;

    if our_bytes.len() != 32 || their_bytes.len() != 32 {
        return Err(anyhow::anyhow!("Public keys must be 32 bytes"));
    }

    // Sort keys to ensure order-independence
    let (first, second) = if our_bytes <= their_bytes {
        (&our_bytes, &their_bytes)
    } else {
        (&their_bytes, &our_bytes)
    };

    // blake3(first || second)
    let mut hasher = blake3::Hasher::new();
    hasher.update(first);
    hasher.update(second);
    let hash = hasher.finalize();
    let hash_bytes = hash.as_bytes();

    // Convert hash bytes to decimal digits
    let mut digits = String::with_capacity(71); // 60 digits + 11 spaces
    for group in 0..12 {
        let offset = (group * 2) % 24;
        let val = u16::from_be_bytes([hash_bytes[offset], hash_bytes[offset + 1]]) as u32;
        let group_val = val % 100000;
        if group > 0 {
            digits.push(' ');
        }
        digits.push_str(&format!("{:05}", group_val));
    }

    Ok(digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let keys = IdentityKeys::generate();
        let public_hex = keys.public_key_hex();
        let id = keys.identity_id();

        assert_eq!(public_hex.len(), 64);
        assert_eq!(id.len(), 64);
    }

    #[test]
    fn test_signing() {
        let keys = IdentityKeys::generate();
        let data = b"test message";

        let signature = keys.sign(data).unwrap();
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_verification() {
        let keys = IdentityKeys::generate();
        let data = b"test message";

        let signature = keys.sign(data).unwrap();
        let public_key = keys.signing_key.verifying_key().to_bytes();

        let valid = IdentityKeys::verify(data, &signature, &public_key).unwrap();
        assert!(valid);

        let invalid = IdentityKeys::verify(b"wrong data", &signature, &public_key).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_serialization() {
        let keys = IdentityKeys::generate();
        let bytes = keys.to_bytes();

        let restored = IdentityKeys::from_bytes(&bytes).unwrap();

        assert_eq!(keys.public_key_hex(), restored.public_key_hex());
        assert_eq!(keys.identity_id(), restored.identity_id());
    }

    #[test]
    fn test_libp2p_peer_id_derivation() {
        let keys = IdentityKeys::generate();
        let peer_id = keys
            .to_libp2p_peer_id()
            .expect("Peer ID derivation should succeed");

        assert!(
            peer_id.starts_with("12D3Koo"),
            "Ed25519 Peer ID should start with '12D3Koo', got: {}",
            peer_id
        );

        assert!(
            peer_id.parse::<libp2p::PeerId>().is_ok(),
            "Derived Peer ID must be parseable as libp2p::PeerId"
        );
    }

    #[test]
    fn test_identity_hash_differs_from_public_key() {
        let keys = IdentityKeys::generate();
        let pk_hex = keys.public_key_hex();
        let id_hash = keys.identity_id();

        assert_eq!(pk_hex.len(), 64);
        assert_eq!(id_hash.len(), 64);
        assert_ne!(
            pk_hex, id_hash,
            "Identity hash must differ from the raw public key"
        );
    }

    #[test]
    fn test_safety_number_is_order_independent_and_deterministic() {
        let a = IdentityKeys::generate().public_key_hex();
        let b = IdentityKeys::generate().public_key_hex();

        let ab = safety_number(&a, &b).unwrap();
        let ba = safety_number(&b, &a).unwrap();
        assert_eq!(ab, ba, "safety number must not depend on argument order");

        assert_eq!(ab, safety_number(&a, &b).unwrap());

        let groups: Vec<&str> = ab.split(' ').collect();
        assert_eq!(groups.len(), 12);
        for group in groups {
            assert_eq!(group.len(), 5);
            assert!(group.chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn test_safety_number_differs_for_different_key_pairs() {
        let a = IdentityKeys::generate().public_key_hex();
        let b = IdentityKeys::generate().public_key_hex();
        let c = IdentityKeys::generate().public_key_hex();

        assert_ne!(
            safety_number(&a, &b).unwrap(),
            safety_number(&a, &c).unwrap()
        );
    }

    #[test]
    fn test_safety_number_rejects_malformed_keys() {
        assert!(safety_number("not-hex", "also-not-hex").is_err());
        assert!(safety_number("abcd", "abcd").is_err());
    }
}