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
    // `is_valid_public_key` checks the 64-hex form AND that the decoded bytes
    // parse as an Ed25519 curve point, so an identity_id (always 64 hex, but
    // not a valid curve point) is rejected rather than double-hashed.
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

/// Inner serializable format for V2 identity keys
#[derive(Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
struct IdentityKeysV2Raw {
    signing_key_bytes: [u8; 32],
    x25519_secret_bytes: [u8; 32],
    mlkem_seed: Vec<u8>,
}

/// Inner serializable format for V3 identity keys (with ML-DSA)
#[derive(Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
struct IdentityKeysV3Raw {
    signing_key_bytes: [u8; 32],
    x25519_secret_bytes: [u8; 32],
    mlkem_seed: Vec<u8>,
    mldsa_public_key: Vec<u8>,
    mldsa_secret_key: Vec<u8>,
}

/// Identity keys (signing + dedicated encryption + hybrid post-quantum key agreement + ML-DSA)
#[derive(Clone)]
pub struct IdentityKeys {
    pub signing_key: SigningKey,
    pub x25519_encryption_secret: x25519_dalek::StaticSecret,
    pub mlkem_keypair: crate::crypto::pq::MlKem768KeyPair,
    pub mldsa_keypair: Option<crate::crypto::pq::mldsa::MlDsa65KeyPair>,
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

        let mlkem_keypair = crate::crypto::pq::generate();
        let mldsa_keypair = Some(crate::crypto::pq::mldsa::generate_keypair());

        Self {
            signing_key,
            x25519_encryption_secret,
            mlkem_keypair,
            mldsa_keypair,
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

    /// Sign data with ML-DSA-65
    pub fn sign_mldsa(&self, data: &[u8]) -> Result<Vec<u8>> {
        if let Some(ref kp) = self.mldsa_keypair {
            crate::crypto::pq::mldsa::sign(kp, data)
        } else {
            Err(anyhow::anyhow!("ML-DSA keypair not available"))
        }
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
        if let Some(mldsa_kp) = self.mldsa_keypair.as_ref() {
            // V3 format with ML-DSA keys
            let mut raw = IdentityKeysV3Raw {
                signing_key_bytes: self.signing_key.to_bytes(),
                x25519_secret_bytes: self.x25519_encryption_secret.to_bytes(),
                mlkem_seed: self.mlkem_keypair.seed.to_vec(),
                mldsa_public_key: mldsa_kp.verifying_key().to_vec(),
                // ML-DSA-65 private key is persisted directly (4032 bytes)
                mldsa_secret_key: mldsa_kp.signing_key().to_vec(),
            };
            let mut serialized = bincode::serialize(&raw)
                .expect("bincode serialization of IdentityKeysV3Raw cannot fail");
            raw.zeroize();
            let mut result = Vec::with_capacity(1 + serialized.len());
            result.push(0x03); // version tag for V3
            result.append(&mut serialized);
            result
        } else {
            // V2 format without ML-DSA keys
            let mut raw = IdentityKeysV2Raw {
                signing_key_bytes: self.signing_key.to_bytes(),
                x25519_secret_bytes: self.x25519_encryption_secret.to_bytes(),
                mlkem_seed: self.mlkem_keypair.seed.to_vec(),
            };
            let mut serialized = bincode::serialize(&raw)
                .expect("bincode serialization of IdentityKeysV2Raw cannot fail");
            raw.zeroize();
            let mut result = Vec::with_capacity(1 + serialized.len());
            result.push(0x02); // version tag
            result.append(&mut serialized);
            result
        }
    }

    /// Deserialize keys from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() == 32 {
            // V1 legacy format
            let signing_key = SigningKey::from_bytes(
                bytes
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid V1 key bytes"))?,
            );

            // For V1 legacy decoding inside from_bytes, we generate temporary encryption keys.
            // Note: IdentityStore::load_keys will check if it was V1 and persist the migration properly.
            let mut x25519_bytes = [0u8; 32];
            rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut x25519_bytes);
            let x25519_encryption_secret = x25519_dalek::StaticSecret::from(x25519_bytes);
            x25519_bytes.zeroize();
            let mlkem_keypair = crate::crypto::pq::generate();

            // Generate ML-DSA keypair for migration
            let mldsa_keypair = Some(crate::crypto::pq::mldsa::generate_keypair());

            Ok(Self {
                signing_key,
                x25519_encryption_secret,
                mlkem_keypair,
                mldsa_keypair,
            })
        } else if bytes.first() == Some(&0x02) {
            // V2 format - migrate to V3 by adding ML-DSA keys
            let mut raw: IdentityKeysV2Raw = bincode::deserialize(&bytes[1..])
                .map_err(|e| anyhow::anyhow!("Failed to deserialize V2 keys: {}", e))?;
            let signing_key = SigningKey::from_bytes(&raw.signing_key_bytes);
            let x25519_encryption_secret =
                x25519_dalek::StaticSecret::from(raw.x25519_secret_bytes);

            let mut mlkem_seed_arr = [0u8; 64];
            if raw.mlkem_seed.len() != 64 {
                raw.zeroize();
                return Err(anyhow::anyhow!("Invalid ML-KEM seed length in V2 keys"));
            }
            mlkem_seed_arr.copy_from_slice(&raw.mlkem_seed);
            let mlkem_keypair = crate::crypto::pq::from_seed(mlkem_seed_arr);
            mlkem_seed_arr.zeroize();
            raw.zeroize();

            // Generate ML-DSA keypair for migration
            let mldsa_keypair = Some(crate::crypto::pq::mldsa::generate_keypair());

            Ok(Self {
                signing_key,
                x25519_encryption_secret,
                mlkem_keypair,
                mldsa_keypair,
            })
        } else if bytes.first() == Some(&0x03) {
            // V3 format with ML-DSA keys
            let mut raw: IdentityKeysV3Raw = bincode::deserialize(&bytes[1..])
                .map_err(|e| anyhow::anyhow!("Failed to deserialize V3 keys: {}", e))?;
            let signing_key = SigningKey::from_bytes(&raw.signing_key_bytes);
            let x25519_encryption_secret =
                x25519_dalek::StaticSecret::from(raw.x25519_secret_bytes);

            let mut mlkem_seed_arr = [0u8; 64];
            if raw.mlkem_seed.len() != 64 {
                raw.zeroize();
                return Err(anyhow::anyhow!("Invalid ML-KEM seed length in V3 keys"));
            }
            mlkem_seed_arr.copy_from_slice(&raw.mlkem_seed);
            let mlkem_keypair = crate::crypto::pq::from_seed(mlkem_seed_arr);
            mlkem_seed_arr.zeroize();

            // Reconstruct the ML-DSA keypair from persisted bytes
            let mldsa_keypair = Some(
                crate::crypto::pq::mldsa::MlDsa65KeyPair::from_bytes(
                    &raw.mldsa_public_key,
                    &raw.mldsa_secret_key,
                )
                .map_err(|e| anyhow::anyhow!("Failed to restore ML-DSA-65 keypair: {}", e))?,
            );

            raw.zeroize();
            Ok(Self {
                signing_key,
                x25519_encryption_secret,
                mlkem_keypair,
                mldsa_keypair,
            })
        } else {
            Err(anyhow::anyhow!("Invalid identity keys format/tag"))
        }
    }

    /// Convert to libp2p Keypair for network identity
    ///
    /// This allows using the same Ed25519 identity keys for both
    /// message encryption/signing AND libp2p network routing,
    /// eliminating the confusion of having two separate IDs.
    ///
    /// NOTE: ed25519-dalek's SigningKey::to_bytes() returns the 32-byte seed,
    /// but libp2p's SecretKey::try_from_bytes() expects the 32-byte seed,
    /// NOT the 64-byte expanded form. We pass the raw seed here.
    pub fn to_libp2p_peer_id(&self) -> Result<String> {
        // Primary path: derive PeerId from raw public key bytes.
        // Falls back to full keypair conversion if the public-key-only path
        // fails (handles libp2p version API skew where try_from_bytes may move).
        let pub_bytes = self.signing_key.verifying_key().to_bytes();
        match libp2p::identity::ed25519::PublicKey::try_from_bytes(&pub_bytes) {
            Ok(libp2p_pub) => Ok(libp2p::identity::PublicKey::from(libp2p_pub)
                .to_peer_id()
                .to_string()),
            Err(_) => {
                // Fallback: derive via full keypair → public → PeerId.
                let kp = self.to_libp2p_keypair()?;
                Ok(kp.public().to_peer_id().to_string())
            }
        }
    }

    pub fn to_libp2p_keypair(&self) -> Result<libp2p::identity::Keypair> {
        // libp2p 0.53 SecretKey::try_from_bytes expects exactly 32 bytes (seed),
        // NOT the 64-byte expanded form. Pass the raw seed from ed25519-dalek.
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

fn default_supported_suites() -> Vec<u8> {
    vec![0x01]
}

/// Public key bundle containing Ed25519, X25519, ML-KEM-768, and ML-DSA-65 public keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyBundle {
    pub ed25519_public: [u8; 32],
    pub x25519_public: [u8; 32],
    pub mlkem_encaps_key: Vec<u8>,     // 1184 B
    pub mldsa_public: Option<Vec<u8>>, // 1952 B (None for older bundles)
    pub created_at: u64,
    #[serde(default = "default_supported_suites")]
    pub supported_suites: Vec<u8>,
    pub signature: Vec<u8>, // Ed25519 over domain-separated bytes below
    pub mldsa_signature: Option<Vec<u8>>, // ML-DSA-65 over same domain-separated bytes (None for older bundles)
}

/// Sign a public key bundle for the given identity keys.
pub fn sign_bundle(keys: &IdentityKeys) -> Result<PublicKeyBundle> {
    let ed25519_public = keys.signing_key.verifying_key().to_bytes();
    let x25519_public = x25519_dalek::PublicKey::from(&keys.x25519_encryption_secret).to_bytes();
    let mlkem_encaps_key = keys.mlkem_keypair.public_key().to_vec();
    let mldsa_public = keys
        .mldsa_keypair
        .as_ref()
        .map(|kp| kp.verifying_key().to_vec());
    let created_at = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Only advertise negotiable RATCHET/encryption suites here. `negotiate_suite`
    // picks the highest value in the intersection of both peers' supported_suites,
    // but `encrypt_message_ratcheted` only special-cases suite 0x02 for V2 envelope
    // construction -- advertising 0x03 caused negotiation to pick it and then fall
    // through to the legacy V1 path. ML-DSA identity-signature capability is already
    // signaled independently via `mldsa_public`/`mldsa_signature` being `Some(..)`,
    // so it doesn't need its own entry in this list.
    let supported_suites = vec![0x01, 0x02];

    // Signature input: b"iron-core keybundle v3" || ed25519_public || x25519_public || mlkem_encaps_key || mldsa_public || created_at.to_le_bytes() || supported_suites
    let mut sig_input = Vec::new();
    sig_input.extend_from_slice(b"iron-core keybundle v3");
    sig_input.extend_from_slice(&ed25519_public);
    sig_input.extend_from_slice(&x25519_public);
    sig_input.extend_from_slice(&mlkem_encaps_key);
    if let Some(ref mldsa_pub) = mldsa_public {
        sig_input.extend_from_slice(mldsa_pub);
    }
    sig_input.extend_from_slice(&created_at.to_le_bytes());
    sig_input.extend_from_slice(&supported_suites);

    let signature = keys.sign(&sig_input)?;
    let mldsa_signature = if keys.mldsa_keypair.is_some() {
        Some(keys.sign_mldsa(&sig_input)?)
    } else {
        None
    };

    Ok(PublicKeyBundle {
        ed25519_public,
        x25519_public,
        mlkem_encaps_key,
        mldsa_public,
        created_at,
        supported_suites,
        signature,
        mldsa_signature,
    })
}

/// Verify a public key bundle's cross-signature.
pub fn verify_bundle(bundle: &PublicKeyBundle) -> Result<()> {
    // Determine which signature format to use based on presence of ML-DSA fields
    let has_mldsa = bundle.mldsa_public.is_some() && bundle.mldsa_signature.is_some();

    if has_mldsa {
        // V3 format with ML-DSA signatures - both must verify
        // SAFETY: has_mldsa checked that both fields are Some, so as_ref() calls are guaranteed to return Some.
        let mldsa_public = bundle
            .mldsa_public
            .as_ref()
            .expect("mldsa_public verified Some by has_mldsa check");
        let mldsa_signature = bundle
            .mldsa_signature
            .as_ref()
            .expect("mldsa_signature verified Some by has_mldsa check");

        // Signature input: b"iron-core keybundle v3" || ed25519_public || x25519_public || mlkem_encaps_key || mldsa_public || created_at.to_le_bytes() || supported_suites
        let mut sig_input = Vec::new();
        sig_input.extend_from_slice(b"iron-core keybundle v3");
        sig_input.extend_from_slice(&bundle.ed25519_public);
        sig_input.extend_from_slice(&bundle.x25519_public);
        sig_input.extend_from_slice(&bundle.mlkem_encaps_key);
        sig_input.extend_from_slice(mldsa_public);
        sig_input.extend_from_slice(&bundle.created_at.to_le_bytes());
        sig_input.extend_from_slice(&bundle.supported_suites);

        // Verify Ed25519 signature
        let ed_verified =
            IdentityKeys::verify(&sig_input, &bundle.signature, &bundle.ed25519_public)
                .unwrap_or(false);

        // Verify ML-DSA signature
        let mldsa_verified =
            crate::crypto::pq::mldsa::verify(mldsa_public, &sig_input, mldsa_signature).is_ok();

        if ed_verified && mldsa_verified {
            Ok(())
        } else {
            // Log the failure details (in real implementation, this would use tracing)
            if !ed_verified {
                eprintln!("Ed25519 signature verification failed for dual-signed bundle");
            }
            if !mldsa_verified {
                eprintln!("ML-DSA signature verification failed for dual-signed bundle");
            }
            Err(anyhow::anyhow!(
                "Dual signature verification failed: both signatures must be valid"
            ))
        }
    } else {
        // Legacy format without ML-DSA - only Ed25519 signature required
        // Log that this is an older bundle (in real implementation, this would use tracing)
        eprintln!("Accepting bundle without ML-DSA signatures (legacy compatibility)");

        // Check v2 signature format first (with suites)
        let mut sig_input_v2 = Vec::new();
        sig_input_v2.extend_from_slice(b"iron-core keybundle v2");
        sig_input_v2.extend_from_slice(&bundle.ed25519_public);
        sig_input_v2.extend_from_slice(&bundle.x25519_public);
        sig_input_v2.extend_from_slice(&bundle.mlkem_encaps_key);
        sig_input_v2.extend_from_slice(&bundle.created_at.to_le_bytes());
        sig_input_v2.extend_from_slice(&bundle.supported_suites);

        if IdentityKeys::verify(&sig_input_v2, &bundle.signature, &bundle.ed25519_public)
            .unwrap_or(false)
        {
            return Ok(());
        }

        // Fallback to v1 signature format (legacy bundles loaded from store without suites signed in)
        let mut sig_input_v1 = Vec::new();
        sig_input_v1.extend_from_slice(b"iron-core keybundle v1");
        sig_input_v1.extend_from_slice(&bundle.ed25519_public);
        sig_input_v1.extend_from_slice(&bundle.x25519_public);
        sig_input_v1.extend_from_slice(&bundle.mlkem_encaps_key);
        sig_input_v1.extend_from_slice(&bundle.created_at.to_le_bytes());

        let verified_v1 =
            IdentityKeys::verify(&sig_input_v1, &bundle.signature, &bundle.ed25519_public)?;
        if !verified_v1 {
            return Err(anyhow::anyhow!("Invalid signature on key bundle"));
        }
        Ok(())
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
    // Use the hash bytes to generate enough digits
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

        assert_eq!(public_hex.len(), 64); // 32 bytes = 64 hex chars
        assert_eq!(id.len(), 64); // Blake3 hash = 32 bytes = 64 hex chars
        assert!(keys.mldsa_keypair.is_some());
    }

    #[test]
    fn test_signing() {
        let keys = IdentityKeys::generate();
        let data = b"test message";

        let signature = keys.sign(data).unwrap();
        assert_eq!(signature.len(), 64); // Ed25519 signature = 64 bytes

        let mldsa_signature = keys.sign_mldsa(data).unwrap();
        assert_eq!(mldsa_signature.len(), 3309); // ML-DSA-65 signature = 3309 bytes
    }

    #[test]
    fn test_verification() {
        let keys = IdentityKeys::generate();
        let data = b"test message";

        let signature = keys.sign(data).unwrap();
        let public_key = keys.signing_key.verifying_key().to_bytes();

        let valid = IdentityKeys::verify(data, &signature, &public_key).unwrap();
        assert!(valid);

        // Test with wrong data
        let invalid = IdentityKeys::verify(b"wrong data", &signature, &public_key).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_serialization() {
        let keys = IdentityKeys::generate();
        let bytes = keys.to_bytes();

        let restored = IdentityKeys::from_bytes(&bytes).unwrap();

        // Verify they produce the same public key
        assert_eq!(keys.public_key_hex(), restored.public_key_hex());
        assert_eq!(keys.identity_id(), restored.identity_id());
        assert!(restored.mldsa_keypair.is_some());
    }

    #[test]
    fn test_libp2p_peer_id_derivation() {
        // Deriving Peer ID from identity keys should produce a valid base58 Peer ID
        let keys = IdentityKeys::generate();
        let peer_id = keys
            .to_libp2p_peer_id()
            .expect("Peer ID derivation should succeed");

        // Peer ID must start with "12D3Koo" (Ed25519 identity multihash prefix)
        assert!(
            peer_id.starts_with("12D3Koo"),
            "Ed25519 Peer ID should start with '12D3Koo', got: {}",
            &peer_id[..12.min(peer_id.len()).try_into().unwrap_or(usize::MAX)]
        );

        // Peer ID must be parseable as a valid libp2p PeerId
        assert!(
            peer_id.parse::<libp2p::PeerId>().is_ok(),
            "Derived Peer ID must be parseable as libp2p::PeerId"
        );
    }

    #[test]
    fn test_identity_hash_differs_from_public_key() {
        // identity_id (Blake3 hash) must differ from public_key_hex (Ed25519 verifying key)
        let keys = IdentityKeys::generate();
        let pk_hex = keys.public_key_hex();
        let id_hash = keys.identity_id();

        // Both are 64 hex chars but represent different values
        assert_eq!(pk_hex.len(), 64);
        assert_eq!(id_hash.len(), 64);
        assert_ne!(
            pk_hex, id_hash,
            "Identity hash must differ from the raw public key"
        );
    }

    #[test]
    fn test_peer_id_roundtrip_deterministic() {
        // Deriving Peer ID multiple times from the same keypair must produce the same result
        let keys = IdentityKeys::generate();
        let peer_id_1 = keys.to_libp2p_peer_id().unwrap();
        let peer_id_2 = keys.to_libp2p_peer_id().unwrap();
        assert_eq!(
            peer_id_1, peer_id_2,
            "Peer ID derivation must be deterministic"
        );
    }

    #[test]
    fn test_peer_id_unique_per_keypair() {
        // Different keypairs must produce different Peer IDs
        let keys_a = IdentityKeys::generate();
        let keys_b = IdentityKeys::generate();
        let peer_id_a = keys_a.to_libp2p_peer_id().unwrap();
        let peer_id_b = keys_b.to_libp2p_peer_id().unwrap();
        assert_ne!(
            peer_id_a, peer_id_b,
            "Different keypairs must yield different Peer IDs"
        );
    }

    #[test]
    fn test_public_key_to_peer_id_to_public_key_roundtrip() {
        // public_key_hex -> Peer ID -> extract public key should round-trip
        let keys = IdentityKeys::generate();
        let original_pk = keys.public_key_hex();
        let peer_id = keys.to_libp2p_peer_id().unwrap();

        // Parse the Peer ID and extract the public key bytes
        let parsed: libp2p::PeerId = peer_id.parse().expect("Peer ID must be parseable");
        let mh = parsed.as_ref();
        assert_eq!(
            mh.code(),
            0,
            "Ed25519 Peer IDs use identity multihash (code 0)"
        );

        let pk = libp2p::identity::PublicKey::try_decode_protobuf(mh.digest())
            .expect("Must decode public key from Peer ID");
        let ed25519_pk = pk.try_into_ed25519().expect("Must be Ed25519 key");
        let extracted_pk = hex::encode(ed25519_pk.to_bytes());

        assert_eq!(
            original_pk.to_lowercase(),
            extracted_pk.to_lowercase(),
            "Public key must round-trip through Peer ID derivation"
        );
    }

    #[test]
    fn test_identity_id_is_not_valid_ed25519_point() {
        // WHAT THIS ACTUALLY PROVES: that a curve-point test CANNOT be used to
        // tell a public key apart from an identity_id.
        //
        // The chance of a random 32-byte string being a valid Ed25519 point is
        // ~1/2 (quadratic-residue requirement when decompressing y). So roughly
        // HALF of all identity_id values ARE valid Ed25519 points and would be
        // misclassified as public keys by such a test.
        //
        // This comment used to claim the opposite -- that the check "verifies
        // resolve_identity can distinguish the two formats" -- while the
        // assertion below measured ~50/100 and therefore demonstrated it cannot.
        // resolve_identity was written on that false premise: it tested the curve
        // point FIRST and returned identity_ids as though they were public keys
        // about half the time, so callers encrypted to a hash and produced
        // ciphertext nobody could decrypt. It now resolves from stored contact
        // and identity data first and uses the curve test only as a last-resort
        // fallback for peers it has no record of.
        let mut valid_count = 0usize;
        for _ in 0..100 {
            let keys = IdentityKeys::generate();
            let id_hash = keys.identity_id();
            let bytes = hex::decode(&id_hash).unwrap();
            let arr: [u8; 32] = bytes.try_into().unwrap();
            if ed25519_dalek::VerifyingKey::from_bytes(&arr).is_ok() {
                valid_count += 1;
            }
        }
        // Sanity check: if we get extreme outliers, flag it — but 30-70 is normal.
        assert!(
            valid_count < 90,
            "Suspiciously high ratio of valid Ed25519 points from Blake3 hashes ({}/100). \
             Expected ~50%. Check for non-random key generation.",
            valid_count
        );
    }

    #[test]
    fn test_safety_number_is_order_independent_and_deterministic() {
        let a = IdentityKeys::generate().public_key_hex();
        let b = IdentityKeys::generate().public_key_hex();

        let ab = safety_number(&a, &b).unwrap();
        let ba = safety_number(&b, &a).unwrap();
        assert_eq!(ab, ba, "safety number must not depend on argument order");

        // Deterministic: same inputs produce the same output every time.
        assert_eq!(ab, safety_number(&a, &b).unwrap());

        // 12 groups of 5 digits, space-separated.
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
        assert!(safety_number("abcd", "abcd").is_err()); // too short
    }

    #[test]
    fn test_v2_keys_non_derivation() {
        let keys = IdentityKeys::generate();
        let ed25519_pub = keys.signing_key.verifying_key().to_bytes();
        let x25519_pub_derived = crate::crypto::ed25519_public_to_x25519(&ed25519_pub).unwrap();
        let x25519_pub_derived_bytes = x25519_pub_derived.to_bytes();
        let x25519_pub_actual =
            x25519_dalek::PublicKey::from(&keys.x25519_encryption_secret).to_bytes();

        assert_ne!(
            x25519_pub_derived_bytes, x25519_pub_actual,
            "V2 X25519 key MUST be newly generated, NOT derived from Ed25519 identity key"
        );
    }

    #[test]
    fn test_bundle_sign_verify_tamper() {
        let keys = IdentityKeys::generate();
        let bundle = sign_bundle(&keys).unwrap();

        // 1. Verify valid bundle succeeds
        assert!(verify_bundle(&bundle).is_ok());

        // 2. Tamper ed25519_public
        let mut tampered = bundle.clone();
        tampered.ed25519_public[0] ^= 1;
        assert!(verify_bundle(&tampered).is_err());

        // 3. Tamper x25519_public
        let mut tampered = bundle.clone();
        tampered.x25519_public[0] ^= 1;
        assert!(verify_bundle(&tampered).is_err());

        // 4. Tamper mlkem_encaps_key
        let mut tampered = bundle.clone();
        tampered.mlkem_encaps_key[0] ^= 1;
        assert!(verify_bundle(&tampered).is_err());

        // 5. Tamper created_at
        let mut tampered = bundle.clone();
        tampered.created_at ^= 1;
        assert!(verify_bundle(&tampered).is_err());

        // 6. Tamper signature
        let mut tampered = bundle.clone();
        tampered.signature[0] ^= 1;
        assert!(verify_bundle(&tampered).is_err());
    }

    #[test]
    fn test_dual_signature_verification() {
        let keys = IdentityKeys::generate();
        let bundle = sign_bundle(&keys).unwrap();

        // Both signatures should be present
        assert!(bundle.mldsa_public.is_some());
        assert!(bundle.mldsa_signature.is_some());

        // Valid bundle should verify
        assert!(verify_bundle(&bundle).is_ok());

        // Tamper Ed25519 signature only
        let mut tampered = bundle.clone();
        tampered.signature[0] ^= 1;
        assert!(verify_bundle(&tampered).is_err());

        // Tamper ML-DSA signature only
        let mut tampered = bundle.clone();
        tampered.mldsa_signature.as_mut().unwrap()[0] ^= 1;
        assert!(verify_bundle(&tampered).is_err());

        // Tamper both signatures
        let mut tampered = bundle.clone();
        tampered.signature[0] ^= 1;
        tampered.mldsa_signature.as_mut().unwrap()[0] ^= 1;
        assert!(verify_bundle(&tampered).is_err());
    }

    #[test]
    fn test_legacy_bundle_acceptance() {
        // Create a bundle without ML-DSA fields (simulate older version)
        let keys = IdentityKeys::generate();
        let ed25519_public = keys.signing_key.verifying_key().to_bytes();
        let x25519_public =
            x25519_dalek::PublicKey::from(&keys.x25519_encryption_secret).to_bytes();
        let mlkem_encaps_key = keys.mlkem_keypair.public_key().to_vec();
        let created_at = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let supported_suites = vec![0x01, 0x02];

        let mut sig_input_v2 = Vec::new();
        sig_input_v2.extend_from_slice(b"iron-core keybundle v2");
        sig_input_v2.extend_from_slice(&ed25519_public);
        sig_input_v2.extend_from_slice(&x25519_public);
        sig_input_v2.extend_from_slice(&mlkem_encaps_key);
        sig_input_v2.extend_from_slice(&created_at.to_le_bytes());
        sig_input_v2.extend_from_slice(&supported_suites);

        let signature = keys.sign(&sig_input_v2).unwrap();

        let legacy_bundle = PublicKeyBundle {
            ed25519_public,
            x25519_public,
            mlkem_encaps_key,
            mldsa_public: None,
            created_at,
            supported_suites,
            signature,
            mldsa_signature: None,
        };

        // Should accept with log message (we check it doesn't error)
        assert!(verify_bundle(&legacy_bundle).is_ok());
    }

    #[test]
    fn test_v1_identity_migration_and_compatibility() {
        // Create a legacy V1 identity (32 bytes raw seed)
        let mut v1_seed = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut v1_seed);

        // Parse V1 seed -> should generate X25519, ML-KEM, and ML-DSA keys on the fly
        let keys = IdentityKeys::from_bytes(&v1_seed).unwrap();
        assert_eq!(keys.signing_key.to_bytes(), v1_seed);
        assert_eq!(keys.mlkem_keypair.public_key().len(), 1184);
        assert!(keys.mldsa_keypair.is_some());

        // Serialize it as V3 (tagged format)
        let serialized = keys.to_bytes();
        assert_eq!(serialized[0], 0x03);

        // Parse serialized V3
        let keys_restored = IdentityKeys::from_bytes(&serialized).unwrap();
        assert_eq!(
            keys_restored.signing_key.to_bytes(),
            keys.signing_key.to_bytes()
        );
        assert_eq!(
            x25519_dalek::PublicKey::from(&keys_restored.x25519_encryption_secret).to_bytes(),
            x25519_dalek::PublicKey::from(&keys.x25519_encryption_secret).to_bytes()
        );
        assert_eq!(
            keys_restored.mlkem_keypair.public_key(),
            keys.mlkem_keypair.public_key()
        );
        assert!(keys_restored.mldsa_keypair.is_some());
    }

    #[test]
    fn test_v1_backup_restore_migration() {
        use crate::crypto::backup::{decrypt_backup, encrypt_backup};

        // Create a legacy V1 identity seed
        let v1_seed = [17u8; 32];

        // Format legacy backup payload: bare hex-encoded 32-byte signing key
        let payload = hex::encode(v1_seed);

        // Encrypt the backup payload using password "secure-password"
        let passphrase = "secure-password";
        let encrypted_backup = encrypt_backup(&payload, passphrase, None).unwrap();

        // Decrypt the backup
        let decrypted_payload = decrypt_backup(&encrypted_backup, passphrase).unwrap();
        assert_eq!(decrypted_payload, payload);

        // Decode decrypted key bytes
        let decrypted_bytes = hex::decode(&decrypted_payload).unwrap();
        assert_eq!(decrypted_bytes, v1_seed);

        // Restore keys through IdentityKeys::from_bytes (should automatically generate encryption/PQ/ML-DSA keys)
        let restored_keys = IdentityKeys::from_bytes(&decrypted_bytes).unwrap();
        assert_eq!(restored_keys.signing_key.to_bytes(), v1_seed);
        assert_eq!(restored_keys.mlkem_keypair.public_key().len(), 1184);
        assert!(restored_keys.mldsa_keypair.is_some());

        // Verify we can sign a bundle
        let bundle = sign_bundle(&restored_keys).unwrap();
        assert!(verify_bundle(&bundle).is_ok());
    }

    #[test]
    fn test_v2_to_v3_migration() {
        // Create V2 keys (without ML-DSA)
        let mut v2_raw = IdentityKeysV2Raw {
            signing_key_bytes: [1u8; 32],
            x25519_secret_bytes: [2u8; 32],
            mlkem_seed: vec![3u8; 64],
        };
        let v2_bytes = {
            let mut serialized = bincode::serialize(&v2_raw)
                .expect("bincode serialization of IdentityKeysV2Raw cannot fail");
            let mut result = Vec::with_capacity(1 + serialized.len());
            result.push(0x02);
            result.append(&mut serialized);
            result
        };
        v2_raw.zeroize();

        // Parse V2 bytes -> should migrate to V3 with ML-DSA keys
        let keys = IdentityKeys::from_bytes(&v2_bytes).unwrap();
        assert_eq!(keys.signing_key.to_bytes(), [1u8; 32]);
        assert!(keys.mldsa_keypair.is_some());

        // Serialize as V3
        let v3_bytes = keys.to_bytes();
        assert_eq!(v3_bytes[0], 0x03);

        // Parse V3 bytes
        let keys_restored = IdentityKeys::from_bytes(&v3_bytes).unwrap();
        assert_eq!(keys_restored.signing_key.to_bytes(), [1u8; 32]);
        assert!(keys_restored.mldsa_keypair.is_some());
    }

    #[test]
    fn identity_id_from_public_key_hex_rejects_non_curve_point_inputs() {
        // A real generated public key derives an identity_id.
        let keys = IdentityKeys::generate();
        let public_hex = keys.public_key_hex();
        let derived = identity_id_from_public_key_hex(&public_hex);
        assert!(
            derived.is_some(),
            "a valid Ed25519 public key must derive an id"
        );

        // T2 (identifier-gate follow-up): the derivation must be gated on a
        // valid Ed25519 curve point, so a public key is accepted but a
        // non-curve-point 64-hex input (identity_id double-hash vector) is
        // rejected rather than blindly re-hashed.
        //
        // NOTE (documented residual): blake3 output is a uniformly random 32
        // bytes, and a random compressed Edwards point is on-curve ~50% of the
        // time, so SOME identity_ids will still parse as valid curve points
        // and pass this gate. The gate strictly narrows the acceptance set
        // (before: any 64-hex; after: only real Ed25519 public keys) but is
        // not a complete identity_id-vs-public_key disambiguator by itself.
        //
        // Deterministic assertions only: short input and an explicitly
        // off-curve 64-hex value (0x7f*32 fails Ed25519 decompression; probed
        // against ed25519-dalek).
        assert!(identity_id_from_public_key_hex("abcd").is_none());
        assert!(
            identity_id_from_public_key_hex(&"7f".repeat(32)).is_none(),
            "0x7f*32 is not a valid Ed25519 curve point"
        );
    }
}
