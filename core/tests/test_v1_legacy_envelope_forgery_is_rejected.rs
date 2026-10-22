//! Integration test for the P0 identified in the adversarial review of PR #221
//! (posted 2026-08-23T12:07:51Z, verdict BLOCK): PR #221 only rejected
//! unsigned *V2* wire envelopes at ingress. An attacker who bincode-serializes
//! a raw legacy `Envelope` directly (skipping the Drift wire format entirely)
//! and sets `sender_public_key` to a victim's real identity key still slipped
//! through `IronCore::receive_message` as an unsigned `WireEnvelope::V1`,
//! because `sender_public_key` is only ever used as AAD -- it binds a value
//! to the ciphertext but never proves the sender possessed the corresponding
//! private key.
//!
//! This test builds exactly the forgery the review describes: an ordinary
//! ephemeral X25519 ECDH encryption (no ratchet, no Drift signature) with the
//! victim's real public key spoofed into `sender_public_key`, bincode-
//! serialized directly (not wrapped in `WireEnvelope`, not run through the
//! Drift signer) -- and proves `IronCore::receive_message` now rejects it.
//!
//! A companion test proves an honest, properly Drift-signed V1 legacy send
//! (the real send path taken by `IronCore::prepare_message` when the
//! recipient has no published V2 bundle) still succeeds, so the ingress fix
//! cannot be passing by rejecting everything.
//!
//! Run with:
//!   cargo test -p scmessenger-core --test test_v1_legacy_envelope_forgery_is_rejected

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use rand::RngCore;
use scmessenger_core::crypto::encrypt::{ed25519_public_to_x25519, KDF_CONTEXT};
use scmessenger_core::identity::IdentityKeys;
use scmessenger_core::message::codec::{decode_wire_envelope, encode_message};
use scmessenger_core::message::{Envelope, Message, MessageType, WireEnvelope};
use scmessenger_core::IronCore;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};

#[test]
fn test_v1_legacy_forged_unsigned_envelope_is_rejected() {
    // =========================================================================
    // STEP 1: Victim (Alice) exists only as a real, known public key. Mallory
    // never has Alice's private key -- only what an attacker could obtain
    // from two published public bundles (per the confirmed original P0).
    // =========================================================================
    let alice_keys = IdentityKeys::generate();
    let alice_pubkey_hex = alice_keys.public_key_hex();
    let alice_ed25519_public = alice_keys.signing_key.verifying_key().to_bytes();

    // Bob is the real target: a fully initialized IronCore node.
    let bob_node = IronCore::new();
    bob_node.grant_consent();
    bob_node
        .initialize_identity()
        .expect("Bob node initialization must succeed");
    let bob_keys = bob_node
        .get_identity_keys()
        .expect("Bob node must have keys");
    let bob_ed25519_public = bob_keys.signing_key.verifying_key().to_bytes();

    // =========================================================================
    // STEP 2: Mallory forges a message attributed to Alice, using ordinary
    // ephemeral X25519 ECDH to Bob's real key -- no ratchet session, no
    // Drift signature, no possession of Alice's private key anywhere.
    // =========================================================================
    let forged_message = Message {
        id: uuid::Uuid::new_v4().to_string(),
        sender_id: alice_pubkey_hex.clone(), // Mallory attributes sender to Alice
        recipient_id: bob_node.get_identity_info().public_key_hex.clone().unwrap(),
        message_type: MessageType::Text,
        payload: b"MALLORY_FORGED_LEGACY_V1_IMPERSONATION".to_vec(),
        timestamp: scmessenger_core::util::unix_time_secs(),
    };
    let forged_message_bytes =
        encode_message(&forged_message).expect("Message encoding must succeed");

    let bob_x25519_public = ed25519_public_to_x25519(&bob_ed25519_public)
        .expect("Deriving Bob's X25519 public key must succeed");

    let mallory_ephemeral_secret = EphemeralSecret::random_from_rng(&mut rand::rngs::OsRng);
    let mallory_ephemeral_public = X25519PublicKey::from(&mallory_ephemeral_secret);
    let shared_secret = mallory_ephemeral_secret.diffie_hellman(&bob_x25519_public);
    let symmetric_key = blake3::derive_key(KDF_CONTEXT, shared_secret.as_bytes());

    let mut nonce_bytes = [0u8; 24];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let cipher = XChaCha20Poly1305::new_from_slice(&symmetric_key)
        .expect("Cipher construction must succeed");
    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: &forged_message_bytes,
                // AAD is the ONLY place sender_public_key is bound -- it does
                // not prove Mallory possesses Alice's private key.
                aad: &alice_ed25519_public,
            },
        )
        .expect("Forged encryption must succeed");

    let forged_envelope = Envelope {
        sender_public_key: alice_ed25519_public.to_vec(),
        ephemeral_public_key: mallory_ephemeral_public.to_bytes().to_vec(),
        nonce: nonce_bytes.to_vec(),
        ciphertext,
        ratchet_dh_public: None,
        ratchet_message_number: None,
    };

    // Raw bincode serialization directly, bypassing the Drift wire format
    // entirely (no version-byte prefix, no signature) -- exactly what an
    // attacker who does not go through `IronCore::prepare_message` would
    // send.
    let forged_wire_bytes =
        bincode::serialize(&forged_envelope).expect("Bincode serialization must succeed");

    // Sanity: confirm this reproduces the exact bypass conditions described
    // in the review -- not Drift-prefixed, and still decodable as an
    // unsigned WireEnvelope::V1 by the codec fallback (i.e. the forgery is
    // realistic, not a strawman that already fails to decode).
    assert_ne!(
        forged_wire_bytes[0],
        scmessenger_core::drift::DRIFT_VERSION,
        "forged bytes must not accidentally collide with the Drift version byte"
    );
    let decoded = decode_wire_envelope(&forged_wire_bytes)
        .expect("forged envelope must still decode as an unsigned WireEnvelope::V1");
    assert!(
        matches!(decoded, WireEnvelope::V1(_)),
        "forged envelope must decode as V1, confirming the bypass path is exercised"
    );

    // =========================================================================
    // STEP 3: Bob's real ingress entry point MUST reject this.
    // =========================================================================
    let receive_result = bob_node.receive_message(forged_wire_bytes);
    assert!(
        receive_result.is_err(),
        "IronCore::receive_message MUST REJECT an unsigned, forged legacy V1 envelope at ingress"
    );
}

#[test]
fn test_v1_legacy_honest_sender_succeeds() {
    // =========================================================================
    // Companion test: an honest V1 legacy send (the real send path taken by
    // `IronCore::prepare_message` when the recipient has no published V2
    // bundle, i.e. `should_use_ratcheted_encryption` returns `Ok(false)`)
    // must still be delivered. This proves the ingress fix isn't passing by
    // rejecting everything.
    // =========================================================================
    let alice_node = IronCore::new();
    alice_node.grant_consent();
    alice_node
        .initialize_identity()
        .expect("Alice node initialization must succeed");
    let alice_pubkey_hex = alice_node.get_identity_info().public_key_hex.unwrap();

    let bob_node = IronCore::new();
    bob_node.grant_consent();
    bob_node
        .initialize_identity()
        .expect("Bob node initialization must succeed");
    let bob_pubkey_hex = bob_node.get_identity_info().public_key_hex.unwrap();

    // Deliberately do NOT exchange/save V2 bundles, so
    // `should_use_ratcheted_encryption` takes the legacy static-ECDH (V1)
    // branch rather than the hybrid ratchet branch.
    let prepared = alice_node
        .prepare_message(
            bob_pubkey_hex.clone(),
            "hello from the honest legacy path".to_string(),
            MessageType::Text,
            None,
        )
        .expect("Honest V1 legacy prepare_message must succeed");

    let receive_result = bob_node.receive_message(prepared.envelope_data);
    assert!(
        receive_result.is_ok(),
        "IronCore::receive_message must accept an honest, Drift-signed V1 legacy envelope: {:?}",
        receive_result.err()
    );

    let received_message = receive_result.unwrap();
    assert_eq!(received_message.sender_id, alice_pubkey_hex);
    assert_eq!(
        received_message
            .text_content()
            .expect("text payload must decode"),
        "hello from the honest legacy path"
    );
}
