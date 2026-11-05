//! Integration test settling, by EXECUTION (not static analysis), whether the V2 ingress
//! signature-verification fix in `IronCore::receive_message` (commit 2aadf489) can be
//! bypassed by downgrading a forged envelope to the legacy untagged-bincode V1 wire format.
//!
//! Adversarial review claim: `receive_message`'s ingress fallback rejects unsigned V2
//! envelopes (`matches!(decoded, WireEnvelope::V2(_))`) but does nothing equivalent for
//! unsigned V1 envelopes. An attacker (Mallory) who holds ONLY Alice's and Bob's published
//! public bundles can:
//!   1. Build a raw legacy `Envelope` with `ratchet_dh_public: None`,
//!      `ratchet_message_number: None`, and `sender_public_key` set to Alice's real public key.
//!   2. Encrypt to Bob using ordinary ephemeral X25519 ECDH (the pre-ratchet `encrypt_message`
//!      algorithm), with the message AAD bound to Alice's public key bytes (which Mallory
//!      knows, because it's public).
//!   3. bincode-serialize the raw `Envelope` (no `SignedEnvelope` wrapper, no Drift framing).
//!      Because bincode writes a 64-bit little-endian length prefix ahead of the first
//!      `Vec<u8>` field (`sender_public_key`, always 32 bytes), `envelope_data[0] == 0x20`,
//!      which is neither `DRIFT_VERSION` (0x01) nor `WIRE_TAG_V2` (0x02).
//!   4. Feed those bytes directly into `IronCore::receive_message`.
//!
//! If Bob decrypts this and attributes the message to Alice, the V2 ingress fix did not close
//! the hole -- it only moved it to the V1 fallback. If Bob's `receive_message` rejects it, the
//! reviewer's claim is refuted for the CURRENT code, whatever the reason.
//!
//! Run with:
//!   cargo test -p scmessenger-core --test test_v1_bincode_downgrade_forgery

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use rand::RngCore;
use scmessenger_core::crypto::encrypt::{ed25519_public_to_x25519, KDF_CONTEXT};
use scmessenger_core::identity::{sign_bundle, IdentityKeys};
use scmessenger_core::message::codec::encode_message;
use scmessenger_core::message::{Envelope, Message, MessageType};
use scmessenger_core::IronCore;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};

#[test]
fn test_v1_bincode_downgrade_forgery() {
    // =========================================================================
    // STEP 1: Alice publishes her bundle, then her private key material is DROPPED.
    // Everything the attacker (Mallory) can see from here on is public.
    // =========================================================================
    let alice_pubkey_bytes: [u8; 32];
    let alice_pubkey_hex: String;
    let alice_public_bundle;
    {
        let alice_keys = IdentityKeys::generate();
        alice_public_bundle = sign_bundle(&alice_keys).expect("Alice bundle signing must succeed");
        alice_pubkey_bytes = alice_keys.signing_key.verifying_key().to_bytes();
        alice_pubkey_hex = alice_keys.public_key_hex();
        // `alice_keys` (and its private signing/X25519 secrets) is dropped at the
        // end of this block. Mallory never has access to it again.
    }

    // =========================================================================
    // STEP 2: Bob is a fully initialized IronCore node with Alice's bundle imported
    // as a contact (the normal precondition for attributing an inbound message to her).
    // =========================================================================
    let bob_node = IronCore::new();
    bob_node.grant_consent();
    bob_node
        .initialize_identity()
        .expect("Bob node initialization must succeed");

    let bob_node_keys = bob_node
        .get_identity_keys()
        .expect("Bob node must have keys");
    let bob_pubkey_bytes = bob_node_keys.signing_key.verifying_key().to_bytes();

    bob_node
        .contacts_store_manager()
        .save_contact_bundle(&alice_pubkey_hex, &alice_public_bundle)
        .expect("Saving Alice's public bundle in Bob's contact store must succeed");

    // =========================================================================
    // STEP 3: Mallory (holding only Alice's and Bob's PUBLIC key bytes) forges a raw
    // legacy V1 Envelope, mirroring the pre-ratchet `encrypt_message` algorithm exactly,
    // but without ever touching Alice's private key.
    // =========================================================================
    let bob_x25519_public =
        ed25519_public_to_x25519(&bob_pubkey_bytes).expect("Derive Bob's X25519 public key");

    let mallory_ephemeral_secret = EphemeralSecret::random_from_rng(&mut rand::rngs::OsRng);
    let mallory_ephemeral_public = X25519PublicKey::from(&mallory_ephemeral_secret);

    // ECDH: mallory_ephemeral_secret x bob_x25519_public -> shared_secret
    let shared_secret = mallory_ephemeral_secret.diffie_hellman(&bob_x25519_public);

    // Same KDF as the legacy `encrypt_message` path.
    let symmetric_key = blake3::derive_key(KDF_CONTEXT, shared_secret.as_bytes());

    let mut nonce_bytes = [0u8; 24];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let forged_payload_text = "MALLORY_V1_DOWNGRADE_FORGED_PAYLOAD";
    let forged_message = Message {
        id: uuid::Uuid::new_v4().to_string(),
        sender_id: alice_pubkey_hex.clone(),
        recipient_id: bob_node.get_identity_info().public_key_hex.clone().unwrap(),
        message_type: MessageType::Text,
        payload: forged_payload_text.as_bytes().to_vec(),
        timestamp: scmessenger_core::util::unix_time_secs(),
    };
    let forged_message_bytes =
        encode_message(&forged_message).expect("Message encoding must succeed");

    let cipher = XChaCha20Poly1305::new_from_slice(&symmetric_key)
        .expect("Cipher construction must succeed");

    // AAD bound to Alice's PUBLIC key bytes -- Mallory knows these; they were published.
    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: &forged_message_bytes,
                aad: &alice_pubkey_bytes,
            },
        )
        .expect("Forged encryption must succeed");

    let forged_envelope = Envelope {
        sender_public_key: alice_pubkey_bytes.to_vec(),
        ephemeral_public_key: mallory_ephemeral_public.to_bytes().to_vec(),
        nonce: nonce_bytes.to_vec(),
        ciphertext,
        ratchet_dh_public: None,
        ratchet_message_number: None,
    };

    let forged_wire_bytes =
        bincode::serialize(&forged_envelope).expect("Bincode serialization must succeed");

    // Sanity-check the premise of the claimed attack: the leading byte must NOT be
    // DRIFT_VERSION or WIRE_TAG_V2, so the Drift and V2-signed-envelope decode paths are
    // both skipped and this falls all the way to the untagged-bincode V1 fallback.
    assert_ne!(
        forged_wire_bytes[0],
        scmessenger_core::drift::DRIFT_VERSION,
        "Precondition of the claimed attack: must not look like a Drift envelope"
    );
    assert_ne!(
        forged_wire_bytes[0],
        scmessenger_core::message::WIRE_TAG_V2,
        "Precondition of the claimed attack: must not look like a tagged V2 envelope"
    );

    // =========================================================================
    // STEP 4: Feed the forged bytes directly into Bob's IronCore::receive_message.
    // =========================================================================
    let receive_result = bob_node.receive_message(forged_wire_bytes);

    // =========================================================================
    // STEP 5: Settle the claim.
    //
    // `IronCore::receive_message` MUST reject this: it is unsigned, and Mallory never
    // possessed Alice's private key. If this assertion fails, the V1 downgrade hole has
    // regressed.
    // =========================================================================
    assert!(
        receive_result.is_err(),
        "P0 REGRESSION: IronCore::receive_message accepted a forged, unsigned V1 envelope \
         and attributed it to Alice without Alice's private key ever being used. Got: {:?}",
        receive_result.ok().map(|m| {
            let text = m.text_content();
            (m.sender_id, text)
        })
    );
    tracing::debug!(
        "V1 bincode-downgrade forgery correctly rejected: {:?}",
        receive_result.err()
    );
}
