// Per-message encryption: X25519 ECDH + XChaCha20-Poly1305
//
// Flow:
// 1. Convert sender's Ed25519 signing key → X25519 static secret
// 2. Generate ephemeral X25519 keypair
// 3. ECDH: ephemeral_secret × recipient_x25519_public → shared_secret
// 4. KDF: Blake3::derive_key(shared_secret) → symmetric_key
// 5. Encrypt: XChaCha20-Poly1305(symmetric_key, random_nonce, plaintext)
// 6. Output: Envelope { sender_pub, ephemeral_pub, nonce, ciphertext }
//
// Recipient reverses:
// 1. Convert recipient's Ed25519 key → X25519 static secret
// 2. ECDH: recipient_secret × ephemeral_public → shared_secret
// 3. KDF: same derivation → symmetric_key
// 4. Decrypt: XChaCha20-Poly1305(symmetric_key, nonce, ciphertext)

use anyhow::{bail, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use zeroize::Zeroize;

/// KDF context string for deriving encryption keys from ECDH shared secrets.
/// Changing this breaks compatibility with all existing messages.
pub const KDF_CONTEXT: &str = "iron-core v2 message encryption 2026-02-05";

/// Convert an Ed25519 signing key to an X25519 static secret for ECDH.
///
/// Ed25519 and X25519 share the same underlying curve (Curve25519),
/// so we can derive X25519 keys from Ed25519 keys deterministically.
/// The conversion uses the clamped SHA-512 hash of the Ed25519 secret key,
/// which is how Ed25519 internally derives its scalar.
pub fn ed25519_to_x25519_secret(signing_key: &SigningKey) -> StaticSecret {
    // Ed25519 secret scalar is SHA-512(secret_key_bytes)[0..32], clamped.
    // x25519-dalek StaticSecret expects the raw 32-byte secret and does its own clamping.
    let mut hash = <sha2::Sha512 as sha2::Digest>::digest(signing_key.to_bytes());
    let mut secret_bytes = [0u8; 32];
    secret_bytes.copy_from_slice(&hash[..32]);

    let secret = StaticSecret::from(secret_bytes);

    // Zeroize intermediates
    secret_bytes.zeroize();
    hash.as_mut_slice().zeroize();

    secret
}

/// Validate that a hex-encoded Ed25519 public key is well-formed.
///
/// Checks:
/// 1. Hex decoding succeeds
/// 2. Length is exactly 32 bytes
/// 3. Bytes represent a valid compressed Ed25519 point
///
/// Returns an error with a specific message if validation fails.
pub fn validate_ed25519_public_key(public_key_hex: &str) -> Result<()> {
    use curve25519_dalek::edwards::CompressedEdwardsY;

    // Decode hex
    let public_key_bytes = hex::decode(public_key_hex)
        .map_err(|_| anyhow::anyhow!("Invalid hex encoding in public key"))?;

    // Check length
    if public_key_bytes.len() != 32 {
        bail!(
            "Public key must be exactly 32 bytes (64 hex characters), got {} bytes ({} hex characters)",
            public_key_bytes.len(),
            public_key_hex.len()
        );
    }

    // Validate Ed25519 format by attempting decompression
    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&public_key_bytes);

    let compressed = CompressedEdwardsY::from_slice(&key_array)
        .map_err(|_| anyhow::anyhow!("Invalid Ed25519 public key format"))?;

    compressed.decompress().ok_or_else(|| {
        anyhow::anyhow!("Public key is not a valid Ed25519 point (decompression failed)")
    })?;

    Ok(())
}

/// Convert an Ed25519 verifying (public) key to an X25519 public key.
///
/// Uses the birational map from Ed25519 (twisted Edwards) to X25519 (Montgomery).
/// This is the standard conversion: u = (1 + y) / (1 - y) mod p.
pub fn ed25519_public_to_x25519(public_key_bytes: &[u8; 32]) -> Result<X25519PublicKey> {
    use curve25519_dalek::edwards::CompressedEdwardsY;

    let compressed = CompressedEdwardsY::from_slice(public_key_bytes)
        .map_err(|_| anyhow::anyhow!("Invalid Ed25519 public key"))?;

    let edwards_point = compressed
        .decompress()
        .ok_or_else(|| anyhow::anyhow!("Failed to decompress Ed25519 public key"))?;

    let montgomery = edwards_point.to_montgomery();
    Ok(X25519PublicKey::from(montgomery.to_bytes()))
}

/// Derive a symmetric encryption key from an ECDH shared secret using Blake3.
fn derive_key(shared_secret: &[u8]) -> [u8; 32] {
    blake3::derive_key(KDF_CONTEXT, shared_secret)
}

/// Encrypt a plaintext message for a recipient.
///
/// # Arguments
/// * `sender_signing_key` - Sender's Ed25519 signing key (for sender identification)
/// * `recipient_public_key` - Recipient's Ed25519 public key bytes (32 bytes)
/// * `plaintext` - The message bytes to encrypt
///
/// # Returns
/// An `Envelope` containing everything needed for decryption.
pub fn encrypt_message(
    sender_signing_key: &SigningKey,
    recipient_public_key: &[u8; 32],
    plaintext: &[u8],
) -> Result<crate::message::Envelope> {
    // Convert recipient's Ed25519 public key to X25519
    let recipient_x25519 = ed25519_public_to_x25519(recipient_public_key)?;

    // Generate ephemeral X25519 keypair for this message
    let ephemeral_secret = EphemeralSecret::random_from_rng(rand::rngs::OsRng);
    let ephemeral_public = X25519PublicKey::from(&ephemeral_secret);

    // ECDH: ephemeral_secret × recipient_public → shared_secret
    let shared_secret = ephemeral_secret.diffie_hellman(&recipient_x25519);

    // KDF: derive symmetric key
    let mut symmetric_key = derive_key(shared_secret.as_bytes());

    // Generate random nonce (24 bytes for XChaCha20)
    let mut nonce_bytes = [0u8; 24];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    // Encrypt with AAD (Additional Authenticated Data)
    // Bind sender public key as AAD to prevent sender spoofing
    let sender_public_bytes = sender_signing_key.verifying_key().to_bytes();
    let cipher = XChaCha20Poly1305::new_from_slice(&symmetric_key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad: &sender_public_bytes,
            },
        )
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Zeroize key material
    symmetric_key.zeroize();

    Ok(crate::message::Envelope {
        sender_public_key: sender_signing_key.verifying_key().to_bytes().to_vec(),
        ephemeral_public_key: ephemeral_public.to_bytes().to_vec(),
        nonce: nonce_bytes.to_vec(),
        ciphertext,
        ratchet_dh_public: None,
        ratchet_message_number: None,
    })
}

/// Decrypt an envelope using the recipient's signing key.
///
/// # Arguments
/// * `recipient_signing_key` - Recipient's Ed25519 signing key
/// * `envelope` - The encrypted envelope
///
/// # Returns
/// The decrypted plaintext bytes.
pub fn decrypt_message(
    recipient_signing_key: &SigningKey,
    envelope: &crate::message::Envelope,
) -> Result<Vec<u8>> {
    // Validate envelope fields
    if envelope.ephemeral_public_key.len() != 32 {
        bail!("Invalid ephemeral public key length");
    }
    if envelope.nonce.len() != 24 {
        bail!("Invalid nonce length");
    }

    // Convert recipient's Ed25519 signing key to X25519 static secret
    let recipient_x25519_secret = ed25519_to_x25519_secret(recipient_signing_key);

    // Reconstruct ephemeral public key
    let mut ephemeral_bytes = [0u8; 32];
    ephemeral_bytes.copy_from_slice(&envelope.ephemeral_public_key);
    let ephemeral_public = X25519PublicKey::from(ephemeral_bytes);

    // ECDH: recipient_secret × ephemeral_public → shared_secret
    let shared_secret = recipient_x25519_secret.diffie_hellman(&ephemeral_public);

    // KDF: same derivation as encryption
    let mut symmetric_key = derive_key(shared_secret.as_bytes());

    // Reconstruct nonce
    let nonce = XNonce::from_slice(&envelope.nonce);

    // Decrypt with AAD (must match the sender public key used during encryption)
    // This prevents sender spoofing attacks
    if envelope.sender_public_key.len() != 32 {
        bail!("Invalid sender public key length");
    }
    let cipher = XChaCha20Poly1305::new_from_slice(&symmetric_key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

    let plaintext = cipher
        .decrypt(
            nonce,
            Payload {
                msg: envelope.ciphertext.as_ref(),
                aad: envelope.sender_public_key.as_ref(),
            },
        )
        .map_err(|_| {
            anyhow::anyhow!(
                "Decryption failed: invalid ciphertext, wrong key, or tampered sender public key"
            )
        })?;

    // Zeroize key material
    symmetric_key.zeroize();

    Ok(plaintext)
}

/// Decrypt a ratcheted envelope using a RatchetSession.
///
/// The envelope must have `ratchet_dh_public` and `ratchet_message_number` set.
/// The caller provides a mutable reference to the active `RatchetSession` for
/// this peer, which will advance the ratchet state as a side effect.
pub fn decrypt_message_ratcheted(
    session: &mut crate::crypto::RatchetSession,
    envelope: &crate::message::Envelope,
) -> Result<Vec<u8>> {
    let dh_public = envelope
        .ratchet_dh_public
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Ratcheted envelope missing ratchet_dh_public field"))?;
    let message_number = envelope.ratchet_message_number.ok_or_else(|| {
        anyhow::anyhow!("Ratcheted envelope missing ratchet_message_number field")
    })?;

    if envelope.nonce.len() != 24 {
        bail!("Invalid nonce length in ratcheted envelope");
    }

    // Use sender public key as AAD (same binding as legacy path)
    let aad = envelope.sender_public_key.as_slice();

    let plaintext = session.decrypt(
        dh_public,
        message_number,
        &envelope.nonce,
        &envelope.ciphertext,
        aad,
    )?;

    session.peer_confirmed = true;
    Ok(plaintext)
}

pub fn decrypt_message_ratcheted_v2(
    session: &mut crate::crypto::RatchetSession,
    envelope: &crate::message::EnvelopeV2,
) -> Result<Vec<u8>> {
    let dh_public = envelope
        .ratchet_dh_public
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Ratcheted V2 envelope missing ratchet_dh_public field"))?;
    let message_number = envelope.ratchet_message_number.ok_or_else(|| {
        anyhow::anyhow!("Ratcheted V2 envelope missing ratchet_message_number field")
    })?;

    if envelope.nonce.len() != 24 {
        bail!("Invalid nonce length in ratcheted V2 envelope");
    }

    let aad = envelope.sender_public_key.as_slice();

    // Verify transcript hash if the peer hasn't been confirmed yet
    if !session.peer_confirmed {
        if let (Some(expected_hash), Some(envelope_hash)) =
            (&session.transcript_hash, &envelope.transcript_hash)
        {
            if expected_hash.as_slice() != envelope_hash.as_slice() {
                bail!("Transcript hash mismatch");
            }
        }
    }

    // Handle ongoing PQ ratchet fields (PQ-hybrid sessions only, suite 0x02
    // or 0x03 -- see RatchetSession::is_pq_hybrid). The bootstrap
    // ciphertext (first message, !peer_confirmed) is already consumed by
    // init_as_receiver_hybrid / init_as_receiver_hybrid_suite02 at session
    // setup -- only process pq fields here for post-confirmation messages,
    // which represent a genuine PQ ratchet step from perform_pq_ratchet_step,
    // not the initial bootstrap.
    if session.is_pq_hybrid() && session.peer_confirmed {
        // The anti-stripping check only applies at genuine cadence boundaries
        // (mirrors encrypt_message_ratcheted's own `current_message_number % 100
        // == 0` trigger). pq_their_encaps_key is already `Some` from the initial
        // bootstrap handshake, so checking on every message would flag every
        // ordinary in-between message as a stripping attempt -- PQ fields are
        // only ever expected on the message numbers that actually trigger a
        // fresh PQ ratchet step.
        let expects_pq_fields = message_number > 0 && message_number % 100 == 0;
        if expects_pq_fields {
            session.validate_pq_fields_present(envelope.pq_kem_ciphertext.is_some())?;
        }
        if let (Some(pq_kem_ciphertext), Some(pq_encaps_key)) =
            (&envelope.pq_kem_ciphertext, &envelope.pq_encaps_key)
        {
            session.handle_incoming_pq_fields(pq_kem_ciphertext, pq_encaps_key)?;
        }
    }

    let plaintext = session.decrypt(
        dh_public,
        message_number,
        &envelope.nonce,
        &envelope.ciphertext,
        aad,
    )?;

    session.peer_confirmed = true;
    Ok(plaintext)
}

/// Encrypt a plaintext message using an existing Double Ratchet session.
///
/// Unlike `encrypt_message` (which uses per-message ephemeral ECDH), this
/// uses the ratchet's sending chain to derive the message key, providing
/// forward secrecy: compromising a key only reveals messages from that
/// chain step forward, not past messages.
///
/// # Arguments
/// * `sender_signing_key` - Sender's Ed25519 signing key (for sender identification)
/// * `session` - Mutable reference to an active `RatchetSession` for the recipient
/// * `plaintext` - The message bytes to encrypt
///
/// # Returns
/// An `Envelope` with `ratchet_dh_public` and `ratchet_message_number` set,
/// indicating this envelope uses the Double Ratchet protocol.
pub fn encrypt_message_ratcheted(
    sender_signing_key: &SigningKey,
    session: &mut crate::crypto::RatchetSession,
    plaintext: &[u8],
) -> Result<crate::message::WireEnvelope> {
    let sender_public_bytes = sender_signing_key.verifying_key().to_bytes();
    let result = session.encrypt(plaintext, &sender_public_bytes)?;

    // PQ ratchet cadence: trigger every 100 messages after peer confirmation.
    // This provides ongoing PQ forward secrecy beyond just the bootstrap.
    let mut pq_kem_ciphertext = None;
    let mut pq_encaps_key = None;

    if session.is_pq_hybrid() {
        if !session.peer_confirmed {
            if let Some(hct) = &session.bootstrap_hct {
                pq_kem_ciphertext = Some(hct.mlkem_ciphertext.clone());
                // Include our current ML-KEM public key so peer can encapsulate with it
                if let Some(ref kp) = session.pq_our_keypair {
                    pq_encaps_key = Some(kp.public_key().to_vec());
                }
            }
        } else if let Some((_, chain_index)) = session.sending_chain_state() {
            // chain_index is the next message number to be sent.
            let current_message_number = chain_index.saturating_sub(1);
            if current_message_number > 0 && current_message_number % 100 == 0 {
                if let Ok((ct, encaps_key)) = session.perform_pq_ratchet_step() {
                    pq_kem_ciphertext = Some(ct);
                    pq_encaps_key = Some(encaps_key);
                }
            }
        }

        let ephemeral_public_key = if !session.peer_confirmed {
            if let Some(hct) = &session.bootstrap_hct {
                hct.x25519_ephemeral_public.to_vec()
            } else {
                result.our_dh_public.to_vec()
            }
        } else {
            result.our_dh_public.to_vec()
        };

        let transcript_hash = if !session.peer_confirmed {
            session.transcript_hash.map(|h| h.to_vec())
        } else {
            None
        };

        Ok(crate::message::WireEnvelope::V2(
            crate::message::EnvelopeV2 {
                // Reflect the SESSION's actual negotiated suite on the wire
                // instead of hardcoding one value -- suite 0x02 and 0x03
                // sessions both reach this branch (is_pq_hybrid() is true for
                // both). `unwrap_or(0x02)` is an unreachable-in-practice
                // defensive fallback: `is_pq_hybrid()` being true guarantees
                // `negotiated_suite` is `Some(0x02)` or `Some(0x03)` here.
                suite: session.negotiated_suite.unwrap_or(0x02),
                sender_public_key: sender_public_bytes.to_vec(),
                ephemeral_public_key,
                nonce: result.nonce,
                ciphertext: result.ciphertext,
                ratchet_dh_public: Some(result.our_dh_public.to_vec()),
                ratchet_message_number: Some(result.message_number),
                pq_kem_ciphertext,
                pq_encaps_key,
                transcript_hash,
            },
        ))
    } else {
        Ok(crate::message::WireEnvelope::V1(crate::message::Envelope {
            sender_public_key: sender_public_bytes.to_vec(),
            ephemeral_public_key: result.our_dh_public.to_vec(),
            nonce: result.nonce,
            ciphertext: result.ciphertext,
            ratchet_dh_public: Some(result.our_dh_public.to_vec()),
            ratchet_message_number: Some(result.message_number),
        }))
    }
}

/// Determine the appropriate encryption method based on peer capabilities and configuration.
///
/// This function implements the gating logic specified in PQC-08 to retire legacy static-ECDH sends.
///
/// # Arguments
/// * `recipient_bundle` - The recipient's public key bundle (None for v1 peers)
/// * `session_exists` - Whether a ratchet session already exists for this peer
/// * `require_pq` - Whether PQ encryption is strictly required (PQC-04)
/// * `peer_id` - The peer identifier for logging purposes
///
/// # Returns
/// * `Ok(true)` - Use ratcheted encryption (hybrid or classical)
/// * `Ok(false)` - Use legacy static-ECDH encryption (only allowed for v1 peers without sessions when require_pq=false)
/// * `Err(_)` - Error cases (v2 peer falling back to legacy, or require_pq=true with v1 peer)
fn should_use_ratcheted_encryption(
    recipient_bundle: Option<&crate::identity::PublicKeyBundle>,
    session_exists: bool,
    require_pq: bool,
    peer_id: &str,
) -> Result<bool> {
    match recipient_bundle {
        Some(bundle) => {
            // V2 peer (has bundle). Suite 0x02 and 0x03 are both PQ-hybrid
            // suites (see crypto::negotiation::HYBRID_SUITE_IDS) -- a peer
            // advertising EITHER one supports hybrid ratchet establishment.
            // The exact suite is re-negotiated (and dispatched to the correct
            // derivation) inside get_or_create_session_hybrid /
            // create_receiver_session_hybrid; this is only the coarse gate
            // deciding whether to attempt the hybrid path at all.
            if bundle
                .supported_suites
                .iter()
                .any(|s| crate::crypto::negotiation::HYBRID_SUITE_IDS.contains(s))
            {
                // Peer supports a hybrid ratchet suite - always use ratcheted encryption
                // Session establishment vs. reuse is handled by the caller
                Ok(true)
            } else {
                // V2 peer but doesn't support any hybrid suite - treat as v1
                if session_exists {
                    Ok(true)
                } else if require_pq {
                    Err(anyhow::anyhow!(
                        "V1 peer {} cannot be sent to when require_pq=true",
                        peer_id
                    ))
                } else {
                    Ok(false)
                }
            }
        }
        None => {
            // V1 peer (no bundle)
            if session_exists {
                Ok(true)
            } else if require_pq {
                Err(anyhow::anyhow!(
                    "V1 peer {} cannot be sent to when require_pq=true",
                    peer_id
                ))
            } else {
                Ok(false)
            }
        }
    }
}

/// Encrypt a message with automatic ratchet session selection.
///
/// If a ratchet session already exists for the peer, uses it (forward secrecy).
/// Otherwise, falls back to per-message ECDH encryption (backward compatible).
///
/// # Arguments
/// * `sender_signing_key` - Sender's Ed25519 signing key
/// * `recipient_bundle` - Recipient's public key bundle (None for V1 peers)
/// * `recipient_public_key_fallback` - Recipient's Ed25519 public key bytes (32 bytes) for legacy path
/// * `plaintext` - The message bytes to encrypt
/// * `session_manager` - Optional ratchet session manager
/// * `peer_id` - Peer identifier for ratchet session lookup
/// * `our_bundle` - Our public key bundle (None for V1 senders)
/// * `our_x25519_secret` - Our dedicated X25519 encryption secret (required for hybrid
///   sessions; used for the static-static DH sender-authentication step instead of an
///   Ed25519-derived key, avoiding cross-protocol reuse of the signing key)
/// * `require_pq` - Whether PQ encryption is strictly required
/// * `audit_log` - Audit log manager for logging legacy sends
///
/// # Returns
/// An `Envelope` encrypted with the best available method.
#[allow(clippy::too_many_arguments)]
pub fn encrypt_with_ratchet_fallback(
    sender_signing_key: &SigningKey,
    recipient_bundle: Option<&crate::identity::PublicKeyBundle>,
    recipient_public_key_fallback: &[u8; 32], // Legacy path for V1
    plaintext: &[u8],
    session_manager: Option<&mut crate::crypto::RatchetSessionManager>,
    peer_id: &str,
    our_bundle: Option<&crate::identity::PublicKeyBundle>,
    our_x25519_secret: Option<&x25519_dalek::StaticSecret>,
    require_pq: bool,
    audit_log: Option<&mut crate::observability::AuditLog>,
) -> Result<crate::message::WireEnvelope> {
    let session_exists = session_manager
        .as_ref()
        .is_some_and(|mgr| mgr.has_session(peer_id));

    match should_use_ratcheted_encryption(recipient_bundle, session_exists, require_pq, peer_id)? {
        true => {
            // Use ratcheted encryption
            if let Some(manager) = session_manager {
                // If we have bundles for both sides, try hybrid session
                if let (Some(our_b), Some(their_b)) = (our_bundle, recipient_bundle) {
                    let our_x25519 = our_x25519_secret.ok_or_else(|| {
                        anyhow::anyhow!(
                            "Hybrid session for peer {} requires our dedicated X25519 secret",
                            peer_id
                        )
                    })?;
                    let session = manager.get_or_create_session_hybrid(
                        peer_id,
                        sender_signing_key,
                        our_x25519,
                        our_b,
                        their_b,
                    )?;
                    return encrypt_message_ratcheted(sender_signing_key, session, plaintext);
                } else {
                    // Fallback to classical V1
                    let their_x25519 = crate::crypto::encrypt::ed25519_public_to_x25519(
                        recipient_public_key_fallback,
                    )?;
                    let session = manager.get_or_create_session(
                        peer_id,
                        sender_signing_key,
                        &their_x25519,
                    )?;
                    return encrypt_message_ratcheted(sender_signing_key, session, plaintext);
                }
            }
            // This shouldn't happen since session_exists was true
            bail!("Session exists but no session manager provided");
        }
        false => {
            // Use legacy static-ECDH encryption
            // Log audit event for legacy static ECDH send
            if let Some(audit) = audit_log {
                audit.append(
                    crate::observability::AuditEventType::LegacyStaticEcdhSend,
                    None,
                    Some(peer_id.to_string()),
                    None,
                );
            }

            Ok(crate::message::WireEnvelope::V1(encrypt_message(
                sender_signing_key,
                recipient_public_key_fallback,
                plaintext,
            )?))
        }
    }
}

/// Decrypt an envelope using the best available method.
///
/// If the envelope has ratchet fields (`ratchet_dh_public` and
/// `ratchet_message_number`), decrypts using the Double Ratchet protocol.
/// Otherwise, falls back to per-message ECDH decryption.
///
/// # Arguments
/// * `recipient_signing_key` - Recipient's Ed25519 signing key (for legacy path)
/// * `envelope` - The encrypted envelope
/// * `session_manager` - Optional ratchet session manager (for ratcheted path)
///
/// # Returns
/// The decrypted plaintext bytes.
/// Build (or replace) a receiver-side hybrid/classical V2 session from an
/// inbound envelope's bootstrap material. Used both for first-contact session
/// establishment and for SESSION DIVERGENCE RECOVERY: if the sender restarted
/// (memory-only sender sessions) its next bootstrap ciphertext cannot be
/// decrypted against our stale session, and we rebuild from the bootstrap
/// fields carried on the wire instead of dropping the message.
#[allow(clippy::too_many_arguments)]
fn init_receiver_v2_session(
    manager: &mut crate::crypto::RatchetSessionManager,
    peer_id: &str,
    recipient_signing_key: &SigningKey,
    recipient_x25519_secret: Option<&x25519_dalek::StaticSecret>,
    our_mlkem_keypair: Option<&crate::crypto::pq::MlKem768KeyPair>,
    our_bundle: Option<&crate::identity::PublicKeyBundle>,
    sender_bundle: Option<&crate::identity::PublicKeyBundle>,
    envelope_v2: &crate::message::EnvelopeV2,
) -> Result<()> {
    if let (Some(our_k), Some(our_b), Some(their_b)) =
        (our_mlkem_keypair, our_bundle, sender_bundle)
    {
        let hct = if let Some(mlk) = &envelope_v2.pq_kem_ciphertext {
            let mut e = [0u8; 32];
            e.copy_from_slice(&envelope_v2.ephemeral_public_key);
            let mut m = [0u8; 1088];
            m.copy_from_slice(mlk);
            Some(crate::crypto::pq::hybrid::HybridCiphertext {
                x25519_ephemeral_public: e,
                mlkem_ciphertext: m.to_vec(),
            })
        } else {
            None
        };

        manager.create_receiver_session_hybrid(
            peer_id,
            recipient_signing_key,
            recipient_x25519_secret
                .ok_or_else(|| anyhow::anyhow!("V2 session requires recipient x25519 secret"))?,
            our_k,
            our_b,
            their_b,
            hct.as_ref(),
        )?;
        Ok(())
    } else {
        bail!("V2 ratcheted envelope received, but missing keys/bundles for hybrid init");
    }
}

pub fn decrypt_with_ratchet_fallback(
    recipient_signing_key: &SigningKey,
    recipient_x25519_secret: Option<&x25519_dalek::StaticSecret>,
    wire_envelope: &crate::message::WireEnvelope,
    session_manager: Option<&mut crate::crypto::RatchetSessionManager>,
    our_mlkem_keypair: Option<&crate::crypto::pq::MlKem768KeyPair>,
    our_bundle: Option<&crate::identity::PublicKeyBundle>,
    sender_bundle: Option<&crate::identity::PublicKeyBundle>,
) -> Result<Vec<u8>> {
    match wire_envelope {
        crate::message::WireEnvelope::V1(envelope) => {
            if is_ratcheted_envelope(envelope) {
                if let Some(manager) = session_manager {
                    let peer_hash = blake3::hash(&envelope.sender_public_key);
                    let peer_id = hex::encode(peer_hash.as_bytes());

                    if !manager.has_session(&peer_id) {
                        // Create a classical receiver session if it doesn't exist
                        let mut sender_ed = [0u8; 32];
                        sender_ed.copy_from_slice(&envelope.sender_public_key);
                        let sender_x25519 =
                            crate::crypto::encrypt::ed25519_public_to_x25519(&sender_ed)?;
                        manager.create_receiver_session(
                            &peer_id,
                            recipient_signing_key,
                            &sender_x25519,
                        )?;
                    }

                    let first_error = match manager
                        .get_session_mut(&peer_id)
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Session exists but cannot be retrieved for peer {}",
                                peer_id
                            )
                        })
                        .and_then(|session| decrypt_message_ratcheted(session, envelope))
                    {
                        Ok(plaintext) => return Ok(plaintext),
                        Err(e) => e,
                    };

                    // SESSION DIVERGENCE RECOVERY: classical receiver sessions are
                    // deterministically derivable from both parties' static keys,
                    // so a sender whose session store was wiped (daemon restart)
                    // can be re-synced by rebuilding ours.
                    tracing::warn!(
                        "V1 ratchet decrypt failed for peer {}.. ({}); rebuilding receiver session from static keys",
                        &peer_id[..16.min(peer_id.len())],
                        first_error
                    );
                    let mut sender_ed = [0u8; 32];
                    sender_ed.copy_from_slice(&envelope.sender_public_key);
                    let sender_x25519 =
                        crate::crypto::encrypt::ed25519_public_to_x25519(&sender_ed)?;
                    manager.create_receiver_session(
                        &peer_id,
                        recipient_signing_key,
                        &sender_x25519,
                    )?;
                    return match manager.get_session_mut(&peer_id) {
                        Some(session) => match decrypt_message_ratcheted(session, envelope) {
                            Ok(plaintext) => {
                                tracing::warn!(
                                    "V1 session re-established for peer {}..; message recovered",
                                    &peer_id[..16.min(peer_id.len())]
                                );
                                Ok(plaintext)
                            }
                            Err(retry_err) => {
                                tracing::warn!(
                                    "V1 session rebuild did not help for peer {}..: {}",
                                    &peer_id[..16.min(peer_id.len())],
                                    retry_err
                                );
                                Err(first_error)
                            }
                        },
                        None => Err(first_error),
                    };
                }
                bail!("Ratcheted V1 envelope received but no active ratchet session");
            }
            decrypt_message(recipient_signing_key, envelope)
        }
        crate::message::WireEnvelope::V2(envelope_v2) => {
            if let Some(manager) = session_manager {
                let peer_hash = blake3::hash(&envelope_v2.sender_public_key);
                let peer_id = hex::encode(peer_hash.as_bytes());
                let peer_short = &peer_id[..16.min(peer_id.len())];

                if !manager.has_session(&peer_id) {
                    init_receiver_v2_session(
                        manager,
                        &peer_id,
                        recipient_signing_key,
                        recipient_x25519_secret,
                        our_mlkem_keypair,
                        our_bundle,
                        sender_bundle,
                        envelope_v2,
                    )?;
                }

                let first_error = match manager
                    .get_session_mut(&peer_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Session exists but cannot be retrieved for peer {}",
                            peer_id
                        )
                    })
                    .and_then(|session| decrypt_message_ratcheted_v2(session, envelope_v2))
                {
                    Ok(plaintext) => return Ok(plaintext),
                    Err(e) => e,
                };

                // SESSION DIVERGENCE RECOVERY: the sender's ratchet sessions live
                // only in memory on some platforms, so a sender-side restart
                // silently forks the session. Its next bootstrap ciphertext
                // (carrying pq/transcript bootstrap fields) can never decrypt
                // against our stale session -- rebuild the receiver session from
                // the bootstrap material instead of dropping the message.
                //
                // Precedent: the phone already re-keys by pushing identity_sync
                // automatically; this heals the same divergence inbound, without
                // needing a round-trip.
                tracing::warn!(
                    "V2 ratchet decrypt failed for peer {}.. ({}); attempting session re-establishment from bootstrap fields",
                    peer_short,
                    first_error
                );
                match init_receiver_v2_session(
                    manager,
                    &peer_id,
                    recipient_signing_key,
                    recipient_x25519_secret,
                    our_mlkem_keypair,
                    our_bundle,
                    sender_bundle,
                    envelope_v2,
                ) {
                    Ok(()) => match manager.get_session_mut(&peer_id) {
                        Some(session) => match decrypt_message_ratcheted_v2(session, envelope_v2) {
                            Ok(plaintext) => {
                                tracing::warn!(
                                    "V2 session re-established from bootstrap for peer {}..; message recovered",
                                    peer_short
                                );
                                return Ok(plaintext);
                            }
                            Err(retry_err) => {
                                tracing::warn!(
                                    "V2 session rebuild did not help for peer {}..: {}",
                                    peer_short,
                                    retry_err
                                );
                                return Err(first_error);
                            }
                        },
                        None => return Err(first_error),
                    },
                    Err(init_err) => {
                        tracing::warn!(
                            "V2 session re-establishment unavailable for peer {}..: {}",
                            peer_short,
                            init_err
                        );
                    }
                }
                return Err(first_error);
            }
            bail!("V2 envelope received but no session manager available");
        }
    }
}

/// Determine if an envelope was encrypted using the Double Ratchet protocol.
pub fn is_ratcheted_envelope(envelope: &crate::message::Envelope) -> bool {
    envelope.ratchet_dh_public.is_some() && envelope.ratchet_message_number.is_some()
}

/// Sign an envelope with the sender's signing key.
///
/// Creates a canonical serialization of the envelope and signs it with Ed25519.
/// This allows relays to verify envelope authenticity without decryption.
///
/// # Arguments
/// * `envelope` - The encrypted envelope to sign
/// * `sender_signing_key` - Sender's Ed25519 signing key
///
/// # Returns
/// A `SignedEnvelope` containing the envelope and its signature.
pub fn sign_envelope(
    envelope: crate::message::Envelope,
    sender_signing_key: &SigningKey,
) -> Result<crate::message::SignedEnvelope> {
    // Create canonical representation for signing
    // We sign the serialized envelope to cover all fields
    let envelope_bytes = bincode::serialize(&envelope)
        .map_err(|e| anyhow::anyhow!("Failed to serialize envelope: {}", e))?;

    // Sign the envelope bytes
    let signature = sender_signing_key.sign(&envelope_bytes);

    Ok(crate::message::SignedEnvelope {
        envelope,
        signature: signature.to_bytes().to_vec(),
    })
}

/// Verify a signed envelope's signature.
///
/// Checks that the signature matches the envelope content and was created
/// by the sender whose public key is in the envelope. This allows relays
/// to reject forged envelopes without decrypting them.
///
/// # Arguments
/// * `signed_envelope` - The signed envelope to verify
///
/// # Returns
/// `Ok(())` if signature is valid, `Err` otherwise.
pub fn verify_envelope(signed_envelope: &crate::message::SignedEnvelope) -> Result<()> {
    // Extract sender's public key from envelope
    if signed_envelope.envelope.sender_public_key.len() != 32 {
        bail!("Invalid sender public key length");
    }

    let mut sender_public_bytes = [0u8; 32];
    sender_public_bytes.copy_from_slice(&signed_envelope.envelope.sender_public_key);

    let verifying_key = VerifyingKey::from_bytes(&sender_public_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid sender public key: {}", e))?;

    // Verify signature length
    if signed_envelope.signature.len() != 64 {
        bail!("Invalid signature length");
    }

    let mut signature_bytes = [0u8; 64];
    signature_bytes.copy_from_slice(&signed_envelope.signature);

    let signature = Ed25519Signature::from_bytes(&signature_bytes);

    // Create canonical representation (same as signing)
    let envelope_bytes = bincode::serialize(&signed_envelope.envelope)
        .map_err(|e| anyhow::anyhow!("Failed to serialize envelope: {}", e))?;

    // Verify signature
    verifying_key
        .verify(&envelope_bytes, &signature)
        .map_err(|e| anyhow::anyhow!("Signature verification failed: {}", e))?;

    Ok(())
}

/// Sign a v2 envelope.
/// We sign the tagged envelope representation (0x02 || bincode(EnvelopeV2)) to authenticate the version tag.
pub fn sign_envelope_v2(
    envelope: crate::message::EnvelopeV2,
    sender_signing_key: &SigningKey,
) -> Result<crate::message::SignedEnvelopeV2> {
    let envelope_bytes = bincode::serialize(&envelope)
        .map_err(|e| anyhow::anyhow!("Failed to serialize envelope V2: {}", e))?;

    // Tagged byte sequence: tag byte included in signed data
    let mut tagged_bytes = Vec::with_capacity(envelope_bytes.len() + 1);
    tagged_bytes.push(crate::message::WIRE_TAG_V2);
    tagged_bytes.extend_from_slice(&envelope_bytes);

    let signature = sender_signing_key.sign(&tagged_bytes);

    Ok(crate::message::SignedEnvelopeV2 {
        envelope,
        signature: signature.to_bytes().to_vec(),
    })
}

/// Verify a signed v2 envelope's signature.
pub fn verify_envelope_v2(signed_envelope: &crate::message::SignedEnvelopeV2) -> Result<()> {
    if signed_envelope.envelope.sender_public_key.len() != 32 {
        bail!("Invalid sender public key length in V2 envelope");
    }

    let mut sender_public_bytes = [0u8; 32];
    sender_public_bytes.copy_from_slice(&signed_envelope.envelope.sender_public_key);

    let verifying_key = VerifyingKey::from_bytes(&sender_public_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid sender public key: {}", e))?;

    if signed_envelope.signature.len() != 64 {
        bail!("Invalid V2 signature length");
    }

    let mut signature_bytes = [0u8; 64];
    signature_bytes.copy_from_slice(&signed_envelope.signature);

    let signature = Ed25519Signature::from_bytes(&signature_bytes);

    // Recreate tagged bytes (tag byte + serialized envelope V2)
    let envelope_bytes = bincode::serialize(&signed_envelope.envelope)
        .map_err(|e| anyhow::anyhow!("Failed to serialize envelope V2: {}", e))?;

    let mut tagged_bytes = Vec::with_capacity(envelope_bytes.len() + 1);
    tagged_bytes.push(crate::message::WIRE_TAG_V2);
    tagged_bytes.extend_from_slice(&envelope_bytes);

    verifying_key
        .verify(&tagged_bytes, &signature)
        .map_err(|e| anyhow::anyhow!("V2 signature verification failed: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::{AuditEventType, AuditLog};
    use ed25519_dalek::SigningKey;

    fn generate_keypair() -> SigningKey {
        let mut secret = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret);
        let key = SigningKey::from_bytes(&secret);
        secret.zeroize();
        key
    }

    #[test]
    fn test_decrypt_fallback_recovers_after_sender_session_restart() {
        // Regression guard for Windows->phone silent drop after daemon restarts:
        // sender ratchet sessions are memory-only, so a sender restart forks the
        // session. The receiver's stale session must NOT cause a silent drop --
        // decrypt_with_ratchet_fallback must re-establish from the bootstrap
        // fields carried on the new session's first ciphertext(s).
        use crate::crypto::RatchetSessionManager;

        let alice_key = generate_keypair();
        let bob_key = generate_keypair();
        let alice_x25519_secret = ed25519_to_x25519_secret(&alice_key);
        let bob_x25519_secret = ed25519_to_x25519_secret(&bob_key);
        let bob_mlkem = crate::crypto::pq::generate();

        let alice_bundle = crate::identity::PublicKeyBundle {
            ed25519_public: alice_key.verifying_key().to_bytes(),
            x25519_public: x25519_dalek::PublicKey::from(&alice_x25519_secret).to_bytes(),
            mlkem_encaps_key: crate::crypto::pq::generate().public_key().to_vec(),
            created_at: 0,
            supported_suites: vec![0x03],
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };
        let bob_bundle = crate::identity::PublicKeyBundle {
            ed25519_public: bob_key.verifying_key().to_bytes(),
            x25519_public: x25519_dalek::PublicKey::from(&bob_x25519_secret).to_bytes(),
            mlkem_encaps_key: bob_mlkem.public_key().to_vec(),
            created_at: 0,
            supported_suites: vec![0x03],
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };

        let mut bob_sessions = RatchetSessionManager::new();

        // Round 1: normal in-session message establishes Bob's receiver session.
        let mut alice_sessions_1 = RatchetSessionManager::new();
        let wire1 = {
            let session = alice_sessions_1
                .get_or_create_session_hybrid(
                    "bob",
                    &alice_key,
                    &alice_x25519_secret,
                    &alice_bundle,
                    &bob_bundle,
                )
                .unwrap();
            encrypt_message_ratcheted(&alice_key, session, b"before restart").unwrap()
        };
        let pt1 = decrypt_with_ratchet_fallback(
            &bob_key,
            Some(&bob_x25519_secret),
            &wire1,
            Some(&mut bob_sessions),
            Some(&bob_mlkem),
            Some(&bob_bundle),
            Some(&alice_bundle),
        )
        .unwrap();
        assert_eq!(pt1, b"before restart");

        // Round 2: Alice's process restarts -- her session store (memory-only)
        // is wiped and a BRAND-NEW sender session is created.
        let mut fresh_alice_sessions = RatchetSessionManager::new();
        let wire2 = {
            let session = fresh_alice_sessions
                .get_or_create_session_hybrid(
                    "bob",
                    &alice_key,
                    &alice_x25519_secret,
                    &alice_bundle,
                    &bob_bundle,
                )
                .unwrap();
            encrypt_message_ratcheted(&alice_key, session, b"after restart").unwrap()
        };

        // The whole point of the fix: the fallback RECOVERS against Bob's
        // stale session instead of failing.
        let pt2 = decrypt_with_ratchet_fallback(
            &bob_key,
            Some(&bob_x25519_secret),
            &wire2,
            Some(&mut bob_sessions),
            Some(&bob_mlkem),
            Some(&bob_bundle),
            Some(&alice_bundle),
        )
        .unwrap();
        assert_eq!(pt2, b"after restart");
    }

    #[test]
    fn test_should_use_ratcheted_encryption_v2_peer_with_session() {
        let bundle = crate::identity::PublicKeyBundle {
            ed25519_public: [0u8; 32],
            x25519_public: [0u8; 32],
            mlkem_encaps_key: vec![0u8; 32],
            created_at: 0,
            supported_suites: vec![0x02],
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };

        let result = should_use_ratcheted_encryption(Some(&bundle), true, false, "test_peer");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_should_use_ratcheted_encryption_v2_peer_no_session() {
        let bundle = crate::identity::PublicKeyBundle {
            ed25519_public: [0u8; 32],
            x25519_public: [0u8; 32],
            mlkem_encaps_key: vec![0u8; 32],
            created_at: 0,
            supported_suites: vec![0x02],
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };

        let result = should_use_ratcheted_encryption(Some(&bundle), false, false, "test_peer");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_should_use_ratcheted_encryption_v1_peer_with_session() {
        let result = should_use_ratcheted_encryption(None, true, false, "test_peer");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_should_use_ratcheted_encryption_v1_peer_no_session_no_require_pq() {
        let result = should_use_ratcheted_encryption(None, false, false, "test_peer");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_should_use_ratcheted_encryption_v1_peer_no_session_require_pq() {
        let result = should_use_ratcheted_encryption(None, false, true, "test_peer");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("require_pq=true"));
    }

    #[test]
    fn test_should_use_ratcheted_encryption_v2_peer_no_suite_02_with_session() {
        let bundle = crate::identity::PublicKeyBundle {
            ed25519_public: [0u8; 32],
            x25519_public: [0u8; 32],
            mlkem_encaps_key: vec![0u8; 32],
            created_at: 0,
            supported_suites: vec![0x01], // Only supports v1
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };

        let result = should_use_ratcheted_encryption(Some(&bundle), true, false, "test_peer");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_should_use_ratcheted_encryption_v2_peer_no_suite_02_no_session_no_require_pq() {
        let bundle = crate::identity::PublicKeyBundle {
            ed25519_public: [0u8; 32],
            x25519_public: [0u8; 32],
            mlkem_encaps_key: vec![0u8; 32],
            created_at: 0,
            supported_suites: vec![0x01], // Only supports v1
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };

        let result = should_use_ratcheted_encryption(Some(&bundle), false, false, "test_peer");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_should_use_ratcheted_encryption_v2_peer_no_suite_02_no_session_require_pq() {
        let bundle = crate::identity::PublicKeyBundle {
            ed25519_public: [0u8; 32],
            x25519_public: [0u8; 32],
            mlkem_encaps_key: vec![0u8; 32],
            created_at: 0,
            supported_suites: vec![0x01], // Only supports v1
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };

        let result = should_use_ratcheted_encryption(Some(&bundle), false, true, "test_peer");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("require_pq=true"));
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Hello, this is a secret message!";

        let envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();
        let decrypted = decrypt_message(&recipient_key, &envelope).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_recipient_fails() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let wrong_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Secret message";
        let envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();

        // Wrong recipient should fail to decrypt
        let result = decrypt_message(&wrong_key, &envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Secret message";
        let mut envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();

        // Tamper with ciphertext
        if let Some(byte) = envelope.ciphertext.last_mut() {
            *byte ^= 0xFF;
        }

        let result = decrypt_message(&recipient_key, &envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_messages_different_ciphertext() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let env1 = encrypt_message(&sender_key, &recipient_public, b"message 1").unwrap();
        let env2 = encrypt_message(&sender_key, &recipient_public, b"message 1").unwrap();

        // Same plaintext should produce different ciphertext (different ephemeral keys + nonces)
        assert_ne!(env1.ciphertext, env2.ciphertext);
        assert_ne!(env1.ephemeral_public_key, env2.ephemeral_public_key);
        assert_ne!(env1.nonce, env2.nonce);
    }

    #[test]
    fn test_sender_public_key_in_envelope() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let envelope = encrypt_message(&sender_key, &recipient_public, b"hello").unwrap();

        assert_eq!(
            envelope.sender_public_key,
            sender_key.verifying_key().to_bytes().to_vec()
        );
    }

    #[test]
    fn test_empty_plaintext() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let envelope = encrypt_message(&sender_key, &recipient_public, b"").unwrap();
        let decrypted = decrypt_message(&recipient_key, &envelope).unwrap();

        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_large_plaintext() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = vec![0x42u8; 60_000]; // 60 KB
        let envelope = encrypt_message(&sender_key, &recipient_public, &plaintext).unwrap();
        let decrypted = decrypt_message(&recipient_key, &envelope).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_invalid_envelope_nonce() {
        let recipient_key = generate_keypair();
        let envelope = crate::message::Envelope {
            sender_public_key: vec![0u8; 32],
            ephemeral_public_key: vec![0u8; 32],
            nonce: vec![0u8; 12], // Wrong size (should be 24)
            ciphertext: vec![0u8; 32],
            ratchet_dh_public: None,
            ratchet_message_number: None,
        };

        let result = decrypt_message(&recipient_key, &envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_aad_binding_prevents_sender_spoofing() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let attacker_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Secret message";
        let mut envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();

        // Attacker tries to replace sender public key with their own
        envelope.sender_public_key = attacker_key.verifying_key().to_bytes().to_vec();

        // Decryption should fail due to AAD mismatch
        let result = decrypt_message(&recipient_key, &envelope);
        assert!(
            result.is_err(),
            "AAD binding should prevent sender spoofing"
        );
    }

    #[test]
    fn test_sign_and_verify_envelope() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Test message for signing";
        let envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();

        // Sign the envelope
        let signed_envelope = sign_envelope(envelope, &sender_key).unwrap();

        // Verify the signature
        let result = verify_envelope(&signed_envelope);
        assert!(result.is_ok(), "Valid signature should verify successfully");
    }

    #[test]
    fn test_tampered_envelope_fails_verification() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Test message";
        let envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();
        let mut signed_envelope = sign_envelope(envelope, &sender_key).unwrap();

        // Tamper with the ciphertext
        if let Some(byte) = signed_envelope.envelope.ciphertext.last_mut() {
            *byte ^= 0xFF;
        }

        // Verification should fail
        let result = verify_envelope(&signed_envelope);
        assert!(
            result.is_err(),
            "Tampered envelope should fail verification"
        );
    }

    #[test]
    fn test_forged_signature_fails_verification() {
        let sender_key = generate_keypair();
        let attacker_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Test message";
        let envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();

        // Attacker signs with their own key
        let mut signed_envelope = sign_envelope(envelope.clone(), &attacker_key).unwrap();

        // Replace sender public key with original sender (attempting forgery)
        signed_envelope.envelope.sender_public_key = sender_key.verifying_key().to_bytes().to_vec();

        // Verification should fail (signature doesn't match sender_public_key)
        let result = verify_envelope(&signed_envelope);
        assert!(result.is_err(), "Forged signature should fail verification");
    }

    #[test]
    fn test_relay_can_verify_without_decrypting() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Secret message that relay can't read";
        let envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();
        let signed_envelope = sign_envelope(envelope, &sender_key).unwrap();

        // Relay can verify authenticity without knowing recipient's key
        let verification = verify_envelope(&signed_envelope);
        assert!(
            verification.is_ok(),
            "Relay should be able to verify envelope"
        );

        // But relay still can't decrypt (would need recipient's key)
        // This demonstrates the purpose: relays can reject forged messages
        // without being able to read the content
    }

    #[test]
    fn test_audit_log_legacy_static_ecdh_send() {
        let mut audit_log = AuditLog::new();
        let sender_key = generate_keypair();
        let recipient_public = [0u8; 32]; // dummy

        let result = encrypt_with_ratchet_fallback(
            &sender_key,
            None, // v1 peer
            &recipient_public,
            b"test",
            None, // no session manager
            "test_peer",
            None,  // no our bundle
            None,  // no our x25519 secret
            false, // require_pq = false
            Some(&mut audit_log),
        );

        assert!(result.is_ok());
        assert_eq!(audit_log.events.len(), 1);
        assert_eq!(
            audit_log.events[0].event_type,
            AuditEventType::LegacyStaticEcdhSend
        );
        assert_eq!(audit_log.events[0].peer_id, Some("test_peer".to_string()));
    }
}
