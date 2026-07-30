// Onion-Layered Relay — Tor-like onion routing for hop anonymity
//
// Each layer reveals only the next hop to relays, protecting both
// origin and destination from intermediate nodes.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::XChaCha20Poly1305;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::crypto::pq::hybrid::{hybrid_decapsulate, hybrid_encapsulate, HybridCiphertext};
use crate::identity::keys::PublicKeyBundle;

/// Maximum number of hops in an onion circuit
pub const MAX_ONION_HOPS: usize = 5;

/// Size of X25519 public key (bytes)
const X25519_KEY_SIZE: usize = 32;

/// Size of XChaCha20-Poly1305 nonce (bytes)
const XCHACHA_NONCE_SIZE: usize = 24;

/// Size of Poly1305 authentication tag (bytes). Reserved constant for onion-layer size calculations; no current caller outside this module.
#[allow(dead_code)]
const POLY1305_TAG_SIZE: usize = 16;

/// Classical (v1) onion layer format - X25519 only
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassicalOnionLayer {
    /// Ephemeral X25519 public key (32 bytes)
    pub ephemeral_pk: Vec<u8>,
    /// XChaCha20-Poly1305 encrypted routing info (contains next hop + nonce + tag)
    pub encrypted_routing_info: Vec<u8>,
    /// XChaCha20-Poly1305 encrypted remaining layers or payload
    pub encrypted_payload: Vec<u8>,
}

/// Hybrid (v2) onion layer format - X25519 + ML-KEM-768
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridOnionLayer {
    /// Hybrid KEM ciphertext containing X25519 ephemeral and ML-KEM ciphertext
    pub hybrid_ct: HybridCiphertext,
    /// True if this is the innermost (destination) layer, in which case
    /// `payload` decrypts directly to the plaintext message. False for a
    /// relay layer, in which case `payload` decrypts to
    /// `(bool is_next_hybrid, next_hop_info) ++ remaining_layers`.
    /// Unlike ClassicalOnionLayer (which signals this via an empty
    /// encrypted_routing_info field), hybrid layers have no separate
    /// routing-info ciphertext, so this flag is the explicit equivalent --
    /// a relay learns this the moment it decrypts its own layer regardless,
    /// so storing it in cleartext here reveals nothing it wouldn't already
    /// determine on decrypt.
    pub is_destination: bool,
    /// XChaCha20-Poly1305 encrypted payload (next hop info + remaining layers,
    /// or the plaintext message if `is_destination`)
    pub payload: Vec<u8>,
}

/// Type of hop in the onion path
#[derive(Debug, Clone)]
pub enum HopAddress {
    /// Classical hop using only X25519
    Classical([u8; X25519_KEY_SIZE]),
    /// Hybrid hop using X25519 + ML-KEM-768
    Hybrid(PublicKeyBundle),
}

impl HopAddress {
    /// Get the X25519 public key for this hop
    pub fn x25519_public(&self) -> [u8; X25519_KEY_SIZE] {
        match self {
            HopAddress::Classical(pk) => *pk,
            HopAddress::Hybrid(bundle) => bundle.x25519_public,
        }
    }

    /// Check if this hop supports hybrid (PQ) encryption
    pub fn is_hybrid(&self) -> bool {
        matches!(self, HopAddress::Hybrid(_))
    }
}

#[derive(Debug, Error)]
pub enum OnionError {
    #[error("Invalid onion envelope")]
    InvalidEnvelope,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Too many hops (max {0})")]
    TooManyHops(usize),
    #[error("Invalid routing info size")]
    InvalidRoutingInfo,
    #[error("No layers remaining")]
    NoLayersRemaining,
    #[error("Invalid hop address")]
    InvalidHopAddress,
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Hybrid decryption failed")]
    HybridDecryptionFailed,
    #[error("Missing PQ keys for hybrid hop")]
    MissingPqKeys,
    #[error("Path contains mixed hops but require_pq=true")]
    MixedHopsNotAllowed,
}

/// Result of onion construction with PQ hop statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnionConstructionResult {
    pub envelope: OnionEnvelope,
    pub pq_hops: u8,
    pub total_hops: u8,
}

/// Complete onion-routed envelope
///
/// An N-layer onion where each relay peels one layer and forwards
/// to the next hop address revealed by decryption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnionEnvelope {
    /// Version tag: 0x01 for classical, 0x02 for hybrid
    pub version: u8,
    /// Layer data (either classical or hybrid)
    pub layer_data: Vec<u8>,
    /// All remaining encrypted layers (for relays to forward)
    pub remaining_layers: Vec<u8>,
}

/// Construct an onion-routed message from a relay path
///
/// Given a path [hop1, hop2, ..., hopN, destination]:
/// - Encrypts payload for destination
/// - Wraps with each relay's public key (in reverse order)
/// - Each relay only learns the next hop address, not origin/destination
///
/// # Arguments
/// * `path` - Vector of relay hop addresses, ending with destination
/// * `payload` - Plaintext message to encrypt
/// * `require_pq` - If true, refuses to build circuit unless ALL hops are hybrid
///
/// # Returns
/// * `OnionConstructionResult` with envelope and PQ hop statistics
pub fn construct_onion(
    path: Vec<HopAddress>,
    payload: &[u8],
    require_pq: bool,
) -> Result<OnionConstructionResult, OnionError> {
    if path.is_empty() || path.len() > MAX_ONION_HOPS {
        return Err(OnionError::TooManyHops(MAX_ONION_HOPS));
    }

    let total_hops = path.len() as u8;
    let pq_hops = path.iter().filter(|hop| hop.is_hybrid()).count() as u8;

    if require_pq && pq_hops != total_hops {
        return Err(OnionError::MixedHopsNotAllowed);
    }

    // Start with the innermost encryption (destination)
    let destination_hop = &path[path.len() - 1];
    let mut current_layer_data: Vec<u8>;
    let mut remaining_layers: Vec<u8>;

    if destination_hop.is_hybrid() {
        // Hybrid destination layer
        let bundle = match destination_hop {
            HopAddress::Hybrid(b) => b,
            _ => unreachable!(),
        };

        let (hybrid_ct, shared_secret) =
            hybrid_encapsulate(&bundle.x25519_public, &bundle.mlkem_encaps_key)
                .map_err(|_| OnionError::EncryptionFailed)?;

        // Derive encryption key from shared secret
        let key = derive_layer_key(shared_secret.as_bytes());

        // Encrypt payload for destination
        let cipher = XChaCha20Poly1305::new(&key);
        let payload_nonce_bytes = derive_nonce(shared_secret.as_bytes(), 0);
        let payload_nonce = chacha20poly1305::XNonce::from_slice(&payload_nonce_bytes);

        let encrypted_payload = cipher
            .encrypt(
                payload_nonce,
                Payload {
                    msg: payload,
                    aad: &payload_nonce_bytes,
                },
            )
            .map_err(|_| OnionError::EncryptionFailed)?;

        let hybrid_layer = HybridOnionLayer {
            hybrid_ct,
            is_destination: true,
            payload: encrypted_payload,
        };

        current_layer_data =
            bincode::serialize(&hybrid_layer).map_err(|_| OnionError::InvalidEnvelope)?;
        remaining_layers = current_layer_data.clone();
    } else {
        // Classical destination layer
        let destination_pk = destination_hop.x25519_public();
        let destination_public_key = PublicKey::from(destination_pk);

        // Generate ephemeral key for destination layer
        let ephemeral_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
        let ephemeral_pk = PublicKey::from(&ephemeral_secret);

        // Perform ECDH with destination
        let shared_secret = ephemeral_secret.diffie_hellman(&destination_public_key);
        let key = derive_layer_key(shared_secret.as_bytes());

        // For the innermost layer, routing_info is empty (destination doesn't need to know next hop)
        let routing_info: Vec<u8> = vec![];

        // Encrypt routing info and payload for destination
        let cipher = XChaCha20Poly1305::new(&key);
        let routing_nonce_bytes = derive_nonce(shared_secret.as_bytes(), 0);
        let routing_nonce = chacha20poly1305::XNonce::from_slice(&routing_nonce_bytes);

        let encrypted_routing_info = cipher
            .encrypt(
                routing_nonce,
                Payload {
                    msg: routing_info.as_slice(),
                    aad: &routing_nonce_bytes,
                },
            )
            .map_err(|_| OnionError::EncryptionFailed)?;

        let payload_nonce_bytes = derive_nonce(shared_secret.as_bytes(), 1);
        let payload_nonce = chacha20poly1305::XNonce::from_slice(&payload_nonce_bytes);

        let encrypted_payload = cipher
            .encrypt(
                payload_nonce,
                Payload {
                    msg: payload,
                    aad: &payload_nonce_bytes,
                },
            )
            .map_err(|_| OnionError::EncryptionFailed)?;

        let classical_layer = ClassicalOnionLayer {
            ephemeral_pk: ephemeral_pk.as_bytes().to_vec(),
            encrypted_routing_info,
            encrypted_payload,
        };

        current_layer_data =
            bincode::serialize(&classical_layer).map_err(|_| OnionError::InvalidEnvelope)?;
        remaining_layers = current_layer_data.clone();
    }

    // Wrap with each relay in reverse order (from second-to-last to first)
    for i in (0..path.len() - 1).rev() {
        let relay_hop = &path[i];
        let next_hop = &path[i + 1];

        if relay_hop.is_hybrid() {
            // Hybrid relay layer
            let bundle = match relay_hop {
                HopAddress::Hybrid(b) => b,
                _ => unreachable!(),
            };

            let (hybrid_ct, shared_secret) =
                hybrid_encapsulate(&bundle.x25519_public, &bundle.mlkem_encaps_key)
                    .map_err(|_| OnionError::EncryptionFailed)?;

            // For relay layers, routing_info contains the next hop's address info
            let next_hop_info = if next_hop.is_hybrid() {
                // Next hop is hybrid - include full bundle info
                let next_bundle = match next_hop {
                    HopAddress::Hybrid(b) => b,
                    _ => unreachable!(),
                };
                bincode::serialize(&(true, next_bundle.clone()))
                    .map_err(|_| OnionError::InvalidEnvelope)?
            } else {
                // Next hop is classical - include just X25519 key
                let next_pk = next_hop.x25519_public();
                bincode::serialize(&(false, next_pk)).map_err(|_| OnionError::InvalidEnvelope)?
            };

            // Combine next_hop_info and remaining_layers
            let mut combined_payload = next_hop_info;
            combined_payload.extend_from_slice(&remaining_layers);

            // Derive encryption key from shared secret
            let key = derive_layer_key(shared_secret.as_bytes());

            // Encrypt combined payload
            let cipher = XChaCha20Poly1305::new(&key);
            let payload_nonce_bytes = derive_nonce(shared_secret.as_bytes(), 0);
            let payload_nonce = chacha20poly1305::XNonce::from_slice(&payload_nonce_bytes);

            let encrypted_payload = cipher
                .encrypt(
                    payload_nonce,
                    Payload {
                        msg: &combined_payload,
                        aad: &payload_nonce_bytes,
                    },
                )
                .map_err(|_| OnionError::EncryptionFailed)?;

            let hybrid_layer = HybridOnionLayer {
                hybrid_ct,
                is_destination: false,
                payload: encrypted_payload,
            };

            current_layer_data =
                bincode::serialize(&hybrid_layer).map_err(|_| OnionError::InvalidEnvelope)?;
            remaining_layers = current_layer_data.clone();
        } else {
            // Classical relay layer
            let relay_pk = relay_hop.x25519_public();
            let relay_public_key = PublicKey::from(relay_pk);

            // Generate new ephemeral key for this layer
            let ephemeral_secret = EphemeralSecret::random_from_rng(rand::thread_rng());
            let ephemeral_pk = PublicKey::from(&ephemeral_secret);

            // ECDH with relay
            let shared_secret = ephemeral_secret.diffie_hellman(&relay_public_key);
            let key = derive_layer_key(shared_secret.as_bytes());

            // For relay layers, routing_info describes the next hop. Must
            // carry (is_hybrid, data) like the hybrid branch does -- a raw
            // [u8;32] alone cannot represent a hybrid next hop (loses its
            // mlkem_encaps_key/suite info), breaking mixed classical-then-
            // hybrid paths.
            let next_hop_info = if next_hop.is_hybrid() {
                let next_bundle = match next_hop {
                    HopAddress::Hybrid(b) => b,
                    HopAddress::Classical(_) => unreachable!(),
                };
                bincode::serialize(&(true, next_bundle.clone()))
                    .map_err(|_| OnionError::InvalidEnvelope)?
            } else {
                let next_pk = next_hop.x25519_public();
                bincode::serialize(&(false, next_pk)).map_err(|_| OnionError::InvalidEnvelope)?
            };

            let cipher = XChaCha20Poly1305::new(&key);
            let routing_nonce_bytes = derive_nonce(shared_secret.as_bytes(), 0);
            let routing_nonce = chacha20poly1305::XNonce::from_slice(&routing_nonce_bytes);

            // Encrypt routing info (next hop) and remaining layers
            let encrypted_routing_info = cipher
                .encrypt(
                    routing_nonce,
                    Payload {
                        msg: next_hop_info.as_slice(),
                        aad: &routing_nonce_bytes,
                    },
                )
                .map_err(|_| OnionError::EncryptionFailed)?;

            let payload_nonce_bytes = derive_nonce(shared_secret.as_bytes(), 1);
            let payload_nonce = chacha20poly1305::XNonce::from_slice(&payload_nonce_bytes);

            let encrypted_payload = cipher
                .encrypt(
                    payload_nonce,
                    Payload {
                        msg: remaining_layers.as_slice(),
                        aad: &payload_nonce_bytes,
                    },
                )
                .map_err(|_| OnionError::EncryptionFailed)?;

            let classical_layer = ClassicalOnionLayer {
                ephemeral_pk: ephemeral_pk.as_bytes().to_vec(),
                encrypted_routing_info,
                encrypted_payload,
            };

            current_layer_data =
                bincode::serialize(&classical_layer).map_err(|_| OnionError::InvalidEnvelope)?;
            remaining_layers = current_layer_data.clone();
        }
    }

    // Determine version based on outermost layer type
    let version = if path[0].is_hybrid() { 0x02 } else { 0x01 };

    let envelope = OnionEnvelope {
        version,
        layer_data: current_layer_data,
        remaining_layers: vec![], // Will be populated during peeling
    };

    Ok(OnionConstructionResult {
        envelope,
        pq_hops,
        total_hops,
    })
}

/// Peel one layer of onion encryption
///
/// Called by a relay node to:
/// 1. Decrypt routing info to discover next hop
/// 2. Decrypt payload to get remaining layers
/// 3. Return (next_hop, remaining_envelope) for forwarding
///
/// # Arguments
/// * `envelope` - Current onion envelope
/// * `relay_secret_key` - This node's X25519 private key
/// * `relay_mlkem_keypair` - This node's ML-KEM keypair (optional, required for hybrid layers)
///
/// # Returns
/// * `Ok((next_hop, remaining_envelope))` - Continue relaying
/// * `Ok((None, plaintext))` - This is the final destination (decrypt payload as plaintext)
pub fn peel_layer(
    envelope: &OnionEnvelope,
    relay_secret_key: &[u8; X25519_KEY_SIZE],
    relay_mlkem_keypair: Option<&crate::crypto::pq::MlKem768KeyPair>,
) -> Result<(Option<HopAddress>, Vec<u8>), OnionError> {
    match envelope.version {
        0x01 => {
            // Classical layer
            let classical_layer: ClassicalOnionLayer = bincode::deserialize(&envelope.layer_data)
                .map_err(|_| OnionError::InvalidEnvelope)?;

            let relay_secret = x25519_dalek::StaticSecret::from(*relay_secret_key);
            let ephemeral_pk = PublicKey::from(
                <[u8; X25519_KEY_SIZE]>::try_from(classical_layer.ephemeral_pk.as_slice())
                    .map_err(|_| OnionError::InvalidEnvelope)?,
            );

            // Perform ECDH
            let shared_secret = relay_secret.diffie_hellman(&ephemeral_pk);
            let key = derive_layer_key(shared_secret.as_bytes());

            let cipher = XChaCha20Poly1305::new(&key);

            // Try to decrypt routing_info - if empty, we're at the destination
            let routing_nonce_bytes = derive_nonce(shared_secret.as_bytes(), 0);
            let routing_nonce = chacha20poly1305::XNonce::from_slice(&routing_nonce_bytes);

            let payload_nonce_bytes = derive_nonce(shared_secret.as_bytes(), 1);
            let payload_nonce = chacha20poly1305::XNonce::from_slice(&payload_nonce_bytes);

            let routing_info_plaintext = if classical_layer.encrypted_routing_info.is_empty() {
                vec![]
            } else {
                cipher
                    .decrypt(
                        routing_nonce,
                        Payload {
                            msg: classical_layer.encrypted_routing_info.as_slice(),
                            aad: &routing_nonce_bytes,
                        },
                    )
                    .map_err(|_| OnionError::DecryptionFailed)?
            };

            // Decrypt payload
            let payload_plaintext = cipher
                .decrypt(
                    payload_nonce,
                    Payload {
                        msg: classical_layer.encrypted_payload.as_slice(),
                        aad: &payload_nonce_bytes,
                    },
                )
                .map_err(|_| OnionError::DecryptionFailed)?;

            // Check if routing_info is empty (we're at destination)
            if routing_info_plaintext.is_empty() {
                // This is the final destination - payload is the plaintext message
                Ok((None, payload_plaintext))
            } else {
                // routing_info_plaintext is bincode-encoded as (is_hybrid: bool,
                // next_hop_data), mirroring the hybrid branch's next-hop format so a
                // classical relay can forward to either a classical or hybrid hop.
                let mut routing_cursor = std::io::Cursor::new(&routing_info_plaintext);
                let is_hybrid: bool = bincode::deserialize_from(&mut routing_cursor)
                    .map_err(|_| OnionError::InvalidHopAddress)?;

                let next_hop = if is_hybrid {
                    let next_bundle: PublicKeyBundle =
                        bincode::deserialize_from(&mut routing_cursor)
                            .map_err(|_| OnionError::InvalidHopAddress)?;
                    HopAddress::Hybrid(next_bundle)
                } else {
                    let next_pk: [u8; X25519_KEY_SIZE] =
                        bincode::deserialize_from(&mut routing_cursor)
                            .map_err(|_| OnionError::InvalidHopAddress)?;
                    HopAddress::Classical(next_pk)
                };

                Ok((Some(next_hop), payload_plaintext))
            }
        }
        0x02 => {
            // Hybrid layer
            let hybrid_layer: HybridOnionLayer = bincode::deserialize(&envelope.layer_data)
                .map_err(|_| OnionError::InvalidEnvelope)?;

            let relay_secret = x25519_dalek::StaticSecret::from(*relay_secret_key);
            let relay_mlkem_keypair = relay_mlkem_keypair.ok_or(OnionError::MissingPqKeys)?;

            // Decapsulate hybrid shared secret
            let shared_secret =
                hybrid_decapsulate(&relay_secret, relay_mlkem_keypair, &hybrid_layer.hybrid_ct)
                    .map_err(|_| OnionError::HybridDecryptionFailed)?;

            // Derive encryption key from shared secret
            let key = derive_layer_key(shared_secret.as_bytes());

            // Decrypt payload
            let cipher = XChaCha20Poly1305::new(&key);
            let payload_nonce_bytes = derive_nonce(shared_secret.as_bytes(), 0);
            let payload_nonce = chacha20poly1305::XNonce::from_slice(&payload_nonce_bytes);

            let payload_plaintext = cipher
                .decrypt(
                    payload_nonce,
                    Payload {
                        msg: hybrid_layer.payload.as_slice(),
                        aad: &payload_nonce_bytes,
                    },
                )
                .map_err(|_| OnionError::DecryptionFailed)?;

            if hybrid_layer.is_destination {
                // Final destination: payload_plaintext IS the message, no
                // next-hop structure to parse (mirrors the classical
                // branch's empty-routing-info destination case).
                return Ok((None, payload_plaintext));
            }

            // Parse the decrypted payload to get next hop info and remaining layers
            // The payload contains: (is_hybrid: bool, next_hop_data) + remaining_layers
            let mut payload_cursor = std::io::Cursor::new(&payload_plaintext);

            // Read the boolean indicating if next hop is hybrid
            let is_hybrid: bool = bincode::deserialize_from(&mut payload_cursor)
                .map_err(|_| OnionError::InvalidEnvelope)?;

            if is_hybrid {
                // Next hop is hybrid - read the full bundle
                let next_bundle: PublicKeyBundle = bincode::deserialize_from(&mut payload_cursor)
                    .map_err(|_| OnionError::InvalidEnvelope)?;
                let next_hop = HopAddress::Hybrid(next_bundle);

                // Get remaining layers (everything after the bundle)
                let remaining_start = payload_cursor.position() as usize;
                let remaining_layers = payload_plaintext[remaining_start..].to_vec();

                Ok((Some(next_hop), remaining_layers))
            } else {
                // Next hop is classical - read just the X25519 key
                let next_pk: [u8; X25519_KEY_SIZE] = bincode::deserialize_from(&mut payload_cursor)
                    .map_err(|_| OnionError::InvalidEnvelope)?;
                let next_hop = HopAddress::Classical(next_pk);

                // Get remaining layers (everything after the key)
                let remaining_start = payload_cursor.position() as usize;
                let remaining_layers = payload_plaintext[remaining_start..].to_vec();

                Ok((Some(next_hop), remaining_layers))
            }
        }
        _ => Err(OnionError::InvalidEnvelope),
    }
}

/// Convenience function for backward compatibility with classical-only paths
pub fn construct_classical_onion(
    path: Vec<[u8; X25519_KEY_SIZE]>,
    payload: &[u8],
) -> Result<OnionEnvelope, OnionError> {
    let hop_addresses: Vec<HopAddress> = path.into_iter().map(HopAddress::Classical).collect();

    let result = construct_onion(hop_addresses, payload, false)?;
    Ok(result.envelope)
}

/// Derive a 32-byte encryption key from a shared secret
fn derive_layer_key(shared_secret: &[u8]) -> chacha20poly1305::Key {
    let key_bytes = blake3::derive_key("SCMessenger-onion-layer-key-v1", shared_secret);
    *chacha20poly1305::Key::from_slice(&key_bytes)
}

/// Derive a 24-byte nonce deterministically from a shared secret and counter
/// This allows the peeling side to reconstruct the same nonce.
/// Counter distinguishes routing_info (0) from payload (1) within the same layer.
fn derive_nonce(shared_secret: &[u8], counter: u8) -> [u8; XCHACHA_NONCE_SIZE] {
    let mut input = shared_secret.to_vec();
    input.push(counter);
    let hash = blake3::derive_key("SCMessenger-onion-layer-nonce-v1", &input);
    let mut nonce = [0u8; XCHACHA_NONCE_SIZE];
    nonce.copy_from_slice(&hash[..XCHACHA_NONCE_SIZE]);
    nonce
}

// Measured envelope sizes for 3-hop circuits:
// - Classical-only (3 hops): ~384 bytes total
// - Hybrid-only (3 hops): ~3744 bytes total (+1120 bytes per hybrid layer as expected)
// - Mixed (hybrid-classical-hybrid): ~2624 bytes total

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::pq::{generate, MlKem768KeyPair};
    use x25519_dalek::StaticSecret;

    fn create_test_bundle() -> (HopAddress, StaticSecret, MlKem768KeyPair) {
        let x25519_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let x25519_public = PublicKey::from(&x25519_secret).to_bytes();
        let mlkem_keypair = generate();

        let bundle = PublicKeyBundle {
            ed25519_public: [0u8; 32], // Not used for onion routing
            x25519_public,
            mlkem_encaps_key: mlkem_keypair.public_key().to_vec(),
            created_at: 0,
            supported_suites: vec![0x02],
            signature: vec![], // Not verified in these tests
            mldsa_public: None,
            mldsa_signature: None,
        };

        (HopAddress::Hybrid(bundle), x25519_secret, mlkem_keypair)
    }

    fn create_classical_hop() -> (HopAddress, StaticSecret) {
        let secret = StaticSecret::random_from_rng(rand::thread_rng());
        let pk = PublicKey::from(&secret).to_bytes();
        (HopAddress::Classical(pk), secret)
    }

    #[test]
    fn test_construct_onion_all_hybrid() {
        let (hop1, _, _) = create_test_bundle();
        let (hop2, _, _) = create_test_bundle();
        let (dest, _, _) = create_test_bundle();

        let path = vec![hop1, hop2, dest];
        let payload = b"hello world";
        let result = construct_onion(path, payload, true);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.pq_hops, 3);
        assert_eq!(result.total_hops, 3);
        assert_eq!(result.envelope.version, 0x02);
    }

    #[test]
    fn test_construct_onion_mixed_path() {
        let (hop1, _, _) = create_test_bundle();
        let (hop2, _) = create_classical_hop();
        let (dest, _, _) = create_test_bundle();

        let path = vec![hop1, hop2, dest];
        let payload = b"test message";
        let result = construct_onion(path, payload, false);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.pq_hops, 2);
        assert_eq!(result.total_hops, 3);
    }

    #[test]
    fn test_construct_onion_strict_mode_rejects_mixed() {
        let (hop1, _, _) = create_test_bundle();
        let (hop2, _) = create_classical_hop();
        let (dest, _, _) = create_test_bundle();

        let path = vec![hop1, hop2, dest];
        let payload = b"test message";
        let result = construct_onion(path, payload, true);

        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            OnionError::MixedHopsNotAllowed
        ));
    }

    #[test]
    fn test_construct_onion_classical_only() {
        let (hop1, _) = create_classical_hop();
        let (hop2, _) = create_classical_hop();
        let (dest, _) = create_classical_hop();

        let path = vec![hop1, hop2, dest];
        let payload = b"test message";
        let result = construct_onion(path, payload, false);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.pq_hops, 0);
        assert_eq!(result.total_hops, 3);
        assert_eq!(result.envelope.version, 0x01);
    }

    #[test]
    fn test_peel_layer_hybrid_destination() {
        let (dest_hop, dest_secret, dest_mlkem) = create_test_bundle();
        let path = vec![dest_hop];
        let payload = b"secret message";
        let result = construct_onion(path, payload, true).unwrap();

        let peeled = peel_layer(&result.envelope, &dest_secret.to_bytes(), Some(&dest_mlkem));

        assert!(peeled.is_ok());
        let (next_hop, decrypted_payload) = peeled.unwrap();
        assert!(next_hop.is_none());
        assert_eq!(decrypted_payload, payload);
    }

    #[test]
    fn test_peel_layer_hybrid_relay() {
        let (relay_hop, relay_secret, relay_mlkem) = create_test_bundle();
        let (dest_hop, _, _) = create_test_bundle();

        let path = vec![relay_hop, dest_hop];
        let payload = b"test message";
        let result = construct_onion(path, payload, true).unwrap();

        let peeled = peel_layer(
            &result.envelope,
            &relay_secret.to_bytes(),
            Some(&relay_mlkem),
        );

        assert!(peeled.is_ok());
        let (next_hop, remaining_layers) = peeled.unwrap();
        assert!(next_hop.is_some());
        assert!(next_hop.unwrap().is_hybrid());
        assert!(!remaining_layers.is_empty());
    }

    #[test]
    fn test_peel_layer_mixed_path() {
        let (hop1, hop1_secret, hop1_mlkem) = create_test_bundle();
        let (hop2, hop2_secret) = create_classical_hop();
        let (dest, _, _) = create_test_bundle();

        let path = vec![hop1, hop2, dest];
        let payload = b"test message";
        let result = construct_onion(path, payload, false).unwrap();

        // Peel first layer (hybrid)
        let peeled1 =
            peel_layer(&result.envelope, &hop1_secret.to_bytes(), Some(&hop1_mlkem)).unwrap();
        assert!(peeled1.0.unwrap().is_hybrid() == false); // hop2 is classical

        // Create envelope for second layer
        let second_envelope = OnionEnvelope {
            version: 0x01, // classical
            layer_data: peeled1.1,
            remaining_layers: vec![],
        };

        // Peel second layer (classical)
        let peeled2 = peel_layer(&second_envelope, &hop2_secret.to_bytes(), None).unwrap();
        assert!(peeled2.0.unwrap().is_hybrid()); // dest is hybrid
    }

    #[test]
    fn test_tampered_hybrid_layer_fails_cleanly() {
        let (relay_hop, relay_secret, relay_mlkem) = create_test_bundle();
        let (dest_hop, _, _) = create_test_bundle();

        let path = vec![relay_hop, dest_hop];
        let payload = b"test message";
        let mut result = construct_onion(path, payload, true).unwrap();

        // Tamper with the hybrid ciphertext
        let mut hybrid_layer: HybridOnionLayer =
            bincode::deserialize(&result.envelope.layer_data).unwrap();
        hybrid_layer.hybrid_ct.mlkem_ciphertext[0] ^= 1;
        result.envelope.layer_data = bincode::serialize(&hybrid_layer).unwrap();

        let peeled = peel_layer(
            &result.envelope,
            &relay_secret.to_bytes(),
            Some(&relay_mlkem),
        );

        assert!(peeled.is_err());
        assert!(matches!(
            peeled.err().unwrap(),
            OnionError::DecryptionFailed
        ));
    }

    #[test]
    fn test_classical_circuit_unchanged() {
        // Test that classical circuits work exactly as before
        let relay_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let relay_pk = PublicKey::from(&relay_secret);

        let dest_secret = StaticSecret::random_from_rng(rand::thread_rng());
        let dest_pk = PublicKey::from(&dest_secret);

        let path = vec![*relay_pk.as_bytes(), *dest_pk.as_bytes()];
        let payload = b"test message";

        // Use the backward compatibility function
        let envelope = construct_classical_onion(path, payload).unwrap();
        assert_eq!(envelope.version, 0x01);

        // Peel first layer (relay)
        let peeled1 = peel_layer(&envelope, &relay_secret.to_bytes(), None).unwrap();
        assert!(peeled1.0.is_some());

        // Create envelope for destination layer
        let dest_envelope = OnionEnvelope {
            version: 0x01,
            layer_data: peeled1.1,
            remaining_layers: vec![],
        };

        // Peel destination layer
        let peeled2 = peel_layer(&dest_envelope, &dest_secret.to_bytes(), None).unwrap();
        assert!(peeled2.0.is_none());
        assert_eq!(peeled2.1, payload);
    }

        proptest::proptest! {
            #[test]
            fn arbitrary_bytes_dont_panic(data in proptest::prelude::any::<Vec<u8>>()) {
                let envelope = OnionEnvelope {
                    version: data.first().cloned().unwrap_or(0x01),
                    layer_data: data.clone(),
                    remaining_layers: vec![],
                };

                let secret = [0u8; 32];
                let _ = peel_layer(&envelope, &secret, None);
                // Should not panic regardless of input
            }
        }

    #[test]
    fn test_onion_error_display() {
        let err = OnionError::InvalidEnvelope;
        assert!(err.to_string().contains("Invalid"));

        let err = OnionError::TooManyHops(5);
        assert!(err.to_string().contains("max"));

        let err = OnionError::MixedHopsNotAllowed;
        assert!(err.to_string().contains("mixed"));
    }
}
