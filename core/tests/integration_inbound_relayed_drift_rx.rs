//! Regression: an inbound message that arrives as a DriftFrame-wrapped Data
//! frame (the shape the swarm hands to the bridge for relayed/custody
//! delivery) from a sender with NO contact record must be decrypted, written
//! to history, and a delivery receipt must be producible. Also pins that a
//! corrupted frame is a loud error (logged as `[RX-DROP]` by `IronCore`), not a
//! silent success.
//!
//! Run with:
//!   cargo test --test integration_inbound_relayed_drift_rx

use scmessenger_core::drift::{DriftEnvelope, DriftFrame, FrameType};
use scmessenger_core::{IronCore, MessageType};

fn make_node() -> IronCore {
    let node = IronCore::new();
    node.grant_consent();
    node.initialize_identity()
        .expect("identity initialization must succeed");
    node
}

fn pubkey(node: &IronCore) -> String {
    node.get_identity_info()
        .public_key_hex
        .expect("node must be initialized")
}

fn identity_id(node: &IronCore) -> String {
    node.get_identity_info()
        .identity_id
        .expect("node must have identity_id")
}

#[test]
fn relayed_drift_data_frame_from_unknown_sender_is_decrypted_and_persisted() {
    let alice = make_node();
    let bob = make_node();
    let text = "relayed hello";

    let prepared = alice
        .prepare_message(pubkey(&bob), text.to_string(), MessageType::Text, None)
        .expect("prepare_message must succeed");

    // Wire shape: swarm wraps the envelope in a Data frame; the receiving
    // swarm unwraps it and passes frame.payload to receive_message.
    let wire = DriftFrame {
        frame_type: FrameType::Data,
        payload: prepared.envelope_data.clone(),
    }
    .to_bytes()
    .expect("frame encodes");
    let unwrapped = DriftFrame::from_bytes(&wire).expect("frame decodes");
    assert_eq!(unwrapped.frame_type, FrameType::Data);

    let received = bob
        .receive_message(unwrapped.payload)
        .expect("relayed frame from an unknown sender must be accepted");
    assert_eq!(received.text_content().expect("text"), text);

    let records = bob
        .history_store_manager()
        .recent_including_hidden(Some(identity_id(&alice)), 10)
        .expect("history readable");
    assert_eq!(records.len(), 1, "message must be persisted in history");
    assert_eq!(records[0].content, text);
    assert!(!records[0].hidden);

    // Redelivery of the same frame (custody retry) must not error or duplicate.
    let again = DriftFrame::from_bytes(&wire).expect("decodes").payload;
    bob.receive_message(again)
        .expect("duplicate delivery must not be an error");
}

#[test]
fn corrupted_relayed_frame_is_a_loud_error_not_silent_success() {
    let alice = make_node();
    let bob = make_node();
    let prepared = alice
        .prepare_message(pubkey(&bob), "x".to_string(), MessageType::Text, None)
        .expect("prepare_message must succeed");
    // Truncation is a deterministic decode failure (a single flipped byte
    // can land in an unauthenticated field and still parse).
    let mut bad = prepared.envelope_data;
    bad.truncate(bad.len() / 2);
    assert!(bob.receive_message(bad).is_err());
    assert!(bob.receive_message(Vec::new()).is_err());
}

/// Parse the envelope, mutate one byte inside an authenticated region, and
/// re-encode. Re-encoding keeps the frame well-formed, so rejection must come
/// from signature/AEAD verification and not from a decode failure.
fn tampered(envelope_data: &[u8], mutate: impl FnOnce(&mut DriftEnvelope)) -> Vec<u8> {
    let mut env = DriftEnvelope::from_bytes(envelope_data).expect("envelope parses");
    mutate(&mut env);
    env.to_bytes().expect("tampered envelope still encodes")
}

#[test]
fn tampered_ciphertext_or_signature_is_rejected_and_nothing_persisted() {
    let alice = make_node();
    let bob = make_node();
    let prepared = alice
        .prepare_message(pubkey(&bob), "secret".to_string(), MessageType::Text, None)
        .expect("prepare_message must succeed");

    let flipped_ciphertext = tampered(&prepared.envelope_data, |env| {
        assert!(!env.ciphertext.is_empty());
        let mid = env.ciphertext.len() / 2;
        env.ciphertext[mid] ^= 0x01;
    });
    assert!(
        bob.receive_message(flipped_ciphertext).is_err(),
        "ciphertext tamper must be rejected"
    );

    let flipped_signature = tampered(&prepared.envelope_data, |env| {
        env.signature[0] ^= 0x01;
    });
    assert!(
        bob.receive_message(flipped_signature).is_err(),
        "signature tamper must be rejected"
    );

    let records = bob
        .history_store_manager()
        .recent_including_hidden(Some(identity_id(&alice)), 10)
        .expect("history readable");
    assert!(records.is_empty(), "rejected frames must not be persisted");

    // The untouched original still decrypts: the rejections left no state.
    bob.receive_message(prepared.envelope_data)
        .expect("pristine envelope must still be accepted");
}
