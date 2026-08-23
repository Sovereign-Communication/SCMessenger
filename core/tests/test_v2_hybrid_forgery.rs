//! Integration test for V2 hybrid handshake authentication analysis.
//!
//! Test: `test_v2_hybrid_envelope_forgeable_without_sender_key`
//!
//! Investigates whether an attacker (Mallory) holding ONLY published public key material
//! (Alice's and Bob's public bundles) can forge a V2 hybrid envelope that Bob decrypts
//! successfully and attributes to Alice.

use rand::RngCore;
use scmessenger_core::crypto::negotiation::negotiate_suite;
use scmessenger_core::crypto::ratchet::RatchetSession;
use scmessenger_core::crypto::{decrypt_with_ratchet_fallback, RatchetSessionManager};
use scmessenger_core::drift::{DriftEnvelope, EnvelopeType, DRIFT_VERSION};
use scmessenger_core::identity::{sign_bundle, IdentityKeys, PublicKeyBundle};
use scmessenger_core::message::{
    decode_message, encode_message, EnvelopeV2, Message, MessageType, WireEnvelope,
};
use scmessenger_core::IronCore;

#[test]
fn test_v2_hybrid_envelope_forgeable_without_sender_key() {
    // =========================================================================
    // STEP 1: Key Generation & Distribution of Public Material
    // =========================================================================

    // Alice (the impersonated victim): generate identity keys and public bundle.
    let (alice_public_bundle, alice_pubkey_hex) = {
        let alice_keys = IdentityKeys::generate();
        let bundle = sign_bundle(&alice_keys).expect("Alice bundle signing must succeed");
        let hex_pk = alice_keys.public_key_hex();
        // Alice's private signing key and encryption secrets are dropped here.
        // They are completely out of scope for the rest of the test.
        (bundle, hex_pk)
    };

    // Bob (the recipient node): generate identity and public bundle.
    let (bob_keys, bob_public_bundle, bob_pubkey_hex) = {
        let keys = IdentityKeys::generate();
        let bundle = sign_bundle(&keys).expect("Bob bundle signing must succeed");
        let hex_pk = keys.public_key_hex();
        (keys, bundle, hex_pk)
    };

    // Mallory (the attacker): holds ONLY the public bundles of Alice and Bob.
    // Explicit assertion for review: Mallory has no access to `alice_keys`.
    let mallory_known_alice_bundle: &PublicKeyBundle = &alice_public_bundle;
    let mallory_known_bob_bundle: &PublicKeyBundle = &bob_public_bundle;

    // =========================================================================
    // STEP 2: Attacker (Mallory) Constructs Forged V2 Hybrid Handshake & Message
    // =========================================================================

    // 2a. Mallory computes suite negotiation and transcript hash from public bundles
    let (suite, transcript_hash) = negotiate_suite(
        &mallory_known_alice_bundle.supported_suites,
        &mallory_known_bob_bundle.supported_suites,
        &mallory_known_alice_bundle.ed25519_public,
        &mallory_known_bob_bundle.ed25519_public,
    )
    .expect("Negotiation from public bundles must succeed");
    assert_eq!(suite, 0x02, "Suite must be 0x02 (hybrid post-quantum)");

    // 2b. Mallory generates her own dummy signing key because `init_as_sender_hybrid`
    // accepts a signing key parameter (which it ignores: `_our_signing_key`).
    let mut mallory_dummy_signing_key_bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut mallory_dummy_signing_key_bytes);
    let mallory_dummy_signing_key =
        ed25519_dalek::SigningKey::from_bytes(&mallory_dummy_signing_key_bytes);

    // 2c. Mallory initializes a sender hybrid ratchet session to Bob's public bundle.
    // Notice: NO Alice private key is used!
    let mut mallory_sender_session = RatchetSession::init_as_sender_hybrid(
        &mallory_dummy_signing_key,
        mallory_known_bob_bundle,
        transcript_hash,
    )
    .expect("Sender hybrid session initialization must succeed without Alice's secret key");

    // 2d. Mallory drafts a forged message claiming to be from Alice to Bob.
    let forged_payload_text = "AUTHENTICATED_COMMAND: Transfer 1000000 credits to Mallory.";
    let forged_message = Message {
        id: uuid::Uuid::new_v4().to_string(),
        sender_id: alice_pubkey_hex.clone(),
        recipient_id: bob_pubkey_hex.clone(),
        message_type: MessageType::Text,
        payload: forged_payload_text.as_bytes().to_vec(),
        timestamp: scmessenger_core::util::unix_time_secs(),
    };
    let forged_message_bytes =
        encode_message(&forged_message).expect("Encoding forged message must succeed");

    // 2e. Mallory encrypts using the ratchet session, setting AAD to Alice's public key
    let encrypt_result = mallory_sender_session
        .encrypt(
            &forged_message_bytes,
            &mallory_known_alice_bundle.ed25519_public,
        )
        .expect("Ratchet encryption with Alice AAD must succeed");

    let bootstrap_hct = mallory_sender_session
        .bootstrap_hct
        .as_ref()
        .expect("Bootstrap HCT must exist");
    let pq_our_keypair = mallory_sender_session
        .pq_our_keypair
        .as_ref()
        .expect("PQ keypair must exist");

    // 2f. Mallory constructs EnvelopeV2 with Alice's public key as sender_public_key
    let forged_envelope_v2 = EnvelopeV2 {
        suite: 0x02,
        sender_public_key: mallory_known_alice_bundle.ed25519_public.to_vec(),
        ephemeral_public_key: bootstrap_hct.x25519_ephemeral_public.to_vec(),
        nonce: encrypt_result.nonce.clone(),
        ciphertext: encrypt_result.ciphertext.clone(),
        ratchet_dh_public: Some(encrypt_result.our_dh_public.to_vec()),
        ratchet_message_number: Some(encrypt_result.message_number),
        pq_kem_ciphertext: Some(bootstrap_hct.mlkem_ciphertext.clone()),
        pq_encaps_key: Some(pq_our_keypair.public_key().to_vec()),
        transcript_hash: Some(transcript_hash.to_vec()),
    };

    let forged_wire_v2 = WireEnvelope::V2(forged_envelope_v2.clone());

    // 2g. Mallory packages the forged envelope into the Drift binary wire format.
    // Since Mallory does not possess Alice's Ed25519 private key, she inserts a dummy/zero signature.
    let forged_drift_envelope = DriftEnvelope {
        version: DRIFT_VERSION,
        envelope_type: EnvelopeType::EncryptedMessage,
        compressed: false,
        message_id: *uuid::Uuid::parse_str(&forged_message.id)
            .unwrap()
            .as_bytes(),
        recipient_hint: DriftEnvelope::hint_from_public_key(
            &mallory_known_bob_bundle.ed25519_public,
        ),
        created_at: scmessenger_core::util::unix_time_secs() as u32,
        ttl_expiry: 0,
        hop_count: 0,
        priority: 128,
        sender_public_key: mallory_known_alice_bundle.ed25519_public,
        ephemeral_public_key: bootstrap_hct.x25519_ephemeral_public,
        nonce: encrypt_result
            .nonce
            .clone()
            .try_into()
            .expect("24-byte nonce"),
        signature: [0u8; 64], // Dummy signature: Mallory cannot sign for Alice!
        ciphertext: encrypt_result.ciphertext,
        ratchet_dh_public: Some(encrypt_result.our_dh_public),
        ratchet_message_number: Some(encrypt_result.message_number),
        suite: Some(0x02),
        pq_kem_ciphertext: Some(bootstrap_hct.mlkem_ciphertext.clone()),
        pq_encaps_key: Some(pq_our_keypair.public_key().to_vec()),
        transcript_hash: Some(transcript_hash.to_vec()),
    };
    let forged_drift_wire_bytes = forged_drift_envelope
        .to_bytes()
        .expect("Drift envelope serialization must succeed");

    // =========================================================================
    // STEP 3: Bob Decrypts the Inbound Message
    // =========================================================================

    // --- LEVEL 1: Primitive-level test (`decrypt_with_ratchet_fallback`) ---
    // 1a. Decrypt directly from WireEnvelope::V2
    let mut bob_session_manager_direct = RatchetSessionManager::new();
    let direct_decrypt_result = decrypt_with_ratchet_fallback(
        &bob_keys.signing_key,
        Some(&bob_keys.x25519_encryption_secret),
        &forged_wire_v2,
        Some(&mut bob_session_manager_direct),
        Some(&bob_keys.mlkem_keypair),
        Some(&bob_public_bundle),
        Some(&alice_public_bundle),
    );
    assert!(
        direct_decrypt_result.is_ok(),
        "Direct WireEnvelope::V2 primitive decryption failed: {:?}",
        direct_decrypt_result.err()
    );

    // 1b. Decrypt decoded from Drift wire bytes
    let decoded_wire =
        scmessenger_core::message::codec::decode_wire_envelope(&forged_drift_wire_bytes)
            .expect("Drift wire decoding to WireEnvelope::V2 must succeed");

    let mut bob_session_manager = RatchetSessionManager::new();
    let primitive_decrypt_result = decrypt_with_ratchet_fallback(
        &bob_keys.signing_key,
        Some(&bob_keys.x25519_encryption_secret),
        &decoded_wire,
        Some(&mut bob_session_manager),
        Some(&bob_keys.mlkem_keypair),
        Some(&bob_public_bundle),
        Some(&alice_public_bundle),
    );

    assert!(
        primitive_decrypt_result.is_ok(),
        "Primitive decryption unexpectedly failed: {:?}",
        primitive_decrypt_result.err()
    );

    let decrypted_primitive_bytes = primitive_decrypt_result.unwrap();
    let recovered_primitive_message =
        decode_message(&decrypted_primitive_bytes).expect("Decoded message from primitive");
    assert_eq!(
        recovered_primitive_message.sender_id, alice_pubkey_hex,
        "Primitive level decrypted message attributes sender to Alice"
    );
    assert_eq!(
        recovered_primitive_message
            .text_content()
            .expect("text content"),
        forged_payload_text
    );

    // --- LEVEL 2: High-level full system test (`IronCore::receive_message`) ---
    let bob_node = IronCore::new();
    bob_node.grant_consent();
    bob_node
        .initialize_identity()
        .expect("Bob node initialization must succeed");

    // Bob imports Alice's published public bundle into his contacts store
    let bob_node_keys = bob_node
        .get_identity_keys()
        .expect("Bob node must have keys");
    bob_node
        .contacts_store_manager()
        .save_contact_bundle(&alice_pubkey_hex, &alice_public_bundle)
        .expect("Saving Alice's public bundle in Bob's contact store must succeed");

    // Re-forge the envelope specifically addressed to bob_node's actual initialized identity
    let bob_node_bundle =
        sign_bundle(&bob_node_keys).expect("Signing bob_node's bundle must succeed");
    let (node_suite, node_transcript_hash) = negotiate_suite(
        &mallory_known_alice_bundle.supported_suites,
        &bob_node_bundle.supported_suites,
        &mallory_known_alice_bundle.ed25519_public,
        &bob_node_bundle.ed25519_public,
    )
    .expect("Suite negotiation with bob_node must succeed");
    assert_eq!(node_suite, 0x02);

    let mut mallory_session_for_node = RatchetSession::init_as_sender_hybrid(
        &mallory_dummy_signing_key,
        &bob_node_bundle,
        node_transcript_hash,
    )
    .expect("Sender hybrid session init for bob_node must succeed");

    let node_forged_message = Message {
        id: uuid::Uuid::new_v4().to_string(),
        sender_id: alice_pubkey_hex.clone(),
        recipient_id: bob_node.get_identity_info().public_key_hex.unwrap(),
        message_type: MessageType::Text,
        payload: forged_payload_text.as_bytes().to_vec(),
        timestamp: scmessenger_core::util::unix_time_secs(),
    };
    let node_forged_bytes =
        encode_message(&node_forged_message).expect("Encoding node forged message");

    let node_encrypt_result = mallory_session_for_node
        .encrypt(
            &node_forged_bytes,
            &mallory_known_alice_bundle.ed25519_public,
        )
        .expect("Ratchet encrypt for node");

    let node_bootstrap_hct = mallory_session_for_node.bootstrap_hct.as_ref().unwrap();
    let node_pq_keypair = mallory_session_for_node.pq_our_keypair.as_ref().unwrap();

    let node_forged_drift_envelope = DriftEnvelope {
        version: DRIFT_VERSION,
        envelope_type: EnvelopeType::EncryptedMessage,
        compressed: false,
        message_id: *uuid::Uuid::parse_str(&node_forged_message.id)
            .unwrap()
            .as_bytes(),
        recipient_hint: DriftEnvelope::hint_from_public_key(&bob_node_bundle.ed25519_public),
        created_at: scmessenger_core::util::unix_time_secs() as u32,
        ttl_expiry: 0,
        hop_count: 0,
        priority: 128,
        sender_public_key: mallory_known_alice_bundle.ed25519_public,
        ephemeral_public_key: node_bootstrap_hct.x25519_ephemeral_public,
        nonce: node_encrypt_result
            .nonce
            .clone()
            .try_into()
            .expect("24-byte nonce"),
        signature: [0u8; 64], // Forged / zero signature
        ciphertext: node_encrypt_result.ciphertext,
        ratchet_dh_public: Some(node_encrypt_result.our_dh_public),
        ratchet_message_number: Some(node_encrypt_result.message_number),
        suite: Some(0x02),
        pq_kem_ciphertext: Some(node_bootstrap_hct.mlkem_ciphertext.clone()),
        pq_encaps_key: Some(node_pq_keypair.public_key().to_vec()),
        transcript_hash: Some(node_transcript_hash.to_vec()),
    };
    let node_forged_drift_bytes = node_forged_drift_envelope
        .to_bytes()
        .expect("Serialize node forged drift envelope");

    // Feed the forged drift envelope directly into Bob's IronCore::receive_message
    let receive_result = bob_node.receive_message(node_forged_drift_bytes);

    // =========================================================================
    // STEP 4: Assertions on the Outcome
    // =========================================================================

    // If receive_message succeeds: the claim is CONFIRMED (forgery successful).
    // If receive_message fails: the claim is REFUTED.
    assert!(
        receive_result.is_ok(),
        "IronCore::receive_message failed on forged envelope: {:?}",
        receive_result.err()
    );

    let received_message = receive_result.unwrap();

    // Verify Bob accepted the forged message and attributed it to Alice
    assert_eq!(
        received_message.sender_id, alice_pubkey_hex,
        "Forged message sender_id must be Alice's public key"
    );
    assert_eq!(
        received_message.text_content().expect("Text payload"),
        forged_payload_text,
        "Decrypted plaintext must match forged payload"
    );
}
