//! Integration tests: Two-node in-process encrypt → send → receive → decrypt flow.
//!
//! These tests exercise the public `IronCore` API end-to-end without touching
//! any network or swarm machinery.  They are deliberately minimal: no tokio
//! runtime, no tempfiles, no libp2p — just pure crypto/message flow.
//!
//! Run with:
//!   cargo test --test integration_ironcore_roundtrip

use scmessenger_core::{IronCore, MessageType};

// ============================================================================
// Helpers
// ============================================================================

/// Stand up an initialised IronCore instance with a generated identity.
fn make_node() -> IronCore {
    let node = IronCore::new();
    node.grant_consent();
    node.initialize_identity()
        .expect("identity initialization must succeed");
    node
}

/// Use a live routing engine so the sender's message is placed in the active
/// outbox, allowing the receipt regression to exercise outbox convergence.
fn make_node_with_routing() -> IronCore {
    let node = make_node();
    *node.routing_engine_handle().write() = Some(
        scmessenger_core::routing::OptimizedRoutingEngine::new([0u8; 32], [0u8; 8]),
    );
    node
}

/// Return the hex-encoded Ed25519 public key for a node.
fn pubkey(node: &IronCore) -> String {
    node.get_identity_info()
        .public_key_hex
        .expect("node must be initialized before calling pubkey()")
}

// ============================================================================
// Test 1 — Happy-path roundtrip
// ============================================================================

/// Alice encrypts a message addressed to Bob; Bob decrypts it and recovers the
/// original plaintext.  The sender identity embedded in the decrypted `Message`
/// must match Alice's public key.
#[test]
fn test_two_node_message_roundtrip() {
    let alice = make_node_with_routing();
    let bob = make_node();

    let plaintext = "Hello Bob, this message is for your eyes only.";

    // Alice prepares (encrypts) the envelope.
    let prepared = alice
        .prepare_message(pubkey(&bob), plaintext.to_string(), MessageType::Text, None)
        .expect("prepare_message must succeed");

    assert!(
        !prepared.envelope_data.is_empty(),
        "envelope_bytes must not be empty"
    );

    // Bob decrypts the envelope.
    let received = bob
        .receive_message(prepared.envelope_data)
        .expect("receive_message must succeed");

    // Plaintext content must be recovered verbatim.
    assert_eq!(
        received.text_content().expect("message must carry text"),
        plaintext,
        "decrypted plaintext must match the original"
    );

    // The sender field must identify Alice, not Bob or anyone else.
    // Under identity canonicalization, message.sender_id carries the sender's
    // Ed25519 public key (not the Blake3 identity_id).
    assert_eq!(
        received.sender_id,
        pubkey(&alice),
        "decrypted message sender_id must equal Alice's public key"
    );
}

// ============================================================================
// Test 2 — Wrong recipient cannot decrypt
// ============================================================================

/// Eve intercepts an envelope that was encrypted for Bob.  Eve's attempt to
/// decrypt it must fail because she does not possess Bob's private key.
#[test]
fn test_wrong_recipient_cannot_decrypt() {
    let alice = make_node();
    let bob = make_node();
    let eve = make_node();

    let prepared = alice
        .prepare_message(
            pubkey(&bob),
            "Secret for Bob only".to_string(),
            MessageType::Text,
            None,
        )
        .expect("prepare_message must succeed");

    let result = eve.receive_message(prepared.envelope_data);
    assert!(
        result.is_err(),
        "Eve must not be able to decrypt a message encrypted for Bob"
    );
}

// ============================================================================
// Test 3 — Tampered ciphertext is rejected
// ============================================================================

/// Alice creates a valid envelope for Bob.  An adversary flips a byte in the
/// middle of the ciphertext.  Bob's decryption attempt must return an error
/// because the AEAD authentication tag will not match the modified ciphertext.
#[test]
fn test_envelope_signature_verification() {
    let alice = make_node();
    let bob = make_node();

    let mut prepared = alice
        .prepare_message(
            pubkey(&bob),
            "Tamper me if you dare".to_string(),
            MessageType::Text,
            None,
        )
        .expect("prepare_message must succeed");

    // Flip a byte well into the payload (past any headers / nonce material).
    // The envelope is bincode-encoded; the ciphertext lives toward the end.
    // Flipping any byte inside the AEAD ciphertext will invalidate the tag.
    let tamper_index = prepared.envelope_data.len() / 2;
    prepared.envelope_data[tamper_index] ^= 0xFF;

    let result = bob.receive_message(prepared.envelope_data);
    assert!(
        result.is_err(),
        "Bob must reject a tampered envelope (AEAD authentication failure)"
    );
}

// ============================================================================
// Test 4 — Deduplication: replaying the same envelope is rejected
// ============================================================================

/// Bob receives the same envelope twice.  The second delivery must be rejected
/// by the inbox deduplication layer, not silently accepted.
#[test]
fn test_duplicate_delivery_rejected() {
    let alice = make_node();
    let bob = make_node();

    let prepared = alice
        .prepare_message(
            pubkey(&bob),
            "Once is enough".to_string(),
            MessageType::Text,
            None,
        )
        .expect("prepare_message must succeed");

    // First delivery succeeds.
    bob.receive_message(prepared.envelope_data.clone())
        .expect("first delivery must succeed");

    // Second delivery of the identical envelope must be accepted (for receipt re-dispatch)
    let result = bob.receive_message(prepared.envelope_data);
    assert!(
        result.is_ok(),
        "duplicate envelope delivery should be accepted to re-dispatch callbacks"
    );

    // Bob's inbox must contain exactly one copy of the message.
    assert_eq!(
        bob.inbox_count(),
        1,
        "inbox must hold exactly one message despite the replay attempt"
    );
}

// ============================================================================
// Test 5 — Multiple independent messages flow correctly
// ============================================================================

/// Alice sends three distinct messages to Bob.  All three must be decryptable
/// and must arrive with the correct content in order.
#[test]
fn test_multiple_messages_roundtrip() {
    let alice = make_node();
    let bob = make_node();
    let bob_pubkey = pubkey(&bob);

    let messages = ["First message", "Second message", "Third message"];

    for expected_text in &messages {
        let prepared = alice
            .prepare_message(
                bob_pubkey.clone(),
                expected_text.to_string(),
                MessageType::Text,
                None,
            )
            .expect("prepare_message must succeed");

        let received = bob
            .receive_message(prepared.envelope_data)
            .expect("receive_message must succeed");

        assert_eq!(
            received.text_content().expect("message must carry text"),
            *expected_text,
            "decrypted text must match the sent text"
        );
    }

    assert_eq!(
        bob.inbox_count(),
        messages.len() as u32,
        "bob's inbox must hold all received messages"
    );
}

// ============================================================================
// Test 6 — Self-message: a node can send to itself
// ============================================================================

/// A node encrypts a message addressed to its own public key and then decrypts
/// it.  This validates that the ECDH key-derivation path works when sender and
/// recipient share the same Ed25519 signing key.
#[test]
fn test_self_message_roundtrip() {
    let node = make_node();

    let plaintext = "Note to self";

    let prepared = node
        .prepare_message(
            pubkey(&node),
            plaintext.to_string(),
            MessageType::Text,
            None,
        )
        .expect("prepare_message to self must succeed");

    let received = node
        .receive_message(prepared.envelope_data)
        .expect("receive_message from self must succeed");

    assert_eq!(
        received
            .text_content()
            .expect("self-message must carry text"),
        plaintext,
        "self-message plaintext must be recovered verbatim"
    );
}

// ============================================================================
// Test 7 — Empty-string body is handled gracefully
// ============================================================================

/// An empty payload must survive the full encrypt / decrypt round-trip without
/// panicking or returning an error.
#[test]
fn test_empty_payload_roundtrip() {
    let alice = make_node();
    let bob = make_node();

    let prepared = alice
        .prepare_message(pubkey(&bob), "".to_string(), MessageType::Text, None)
        .expect("prepare_message with empty body must succeed");

    let received = bob
        .receive_message(prepared.envelope_data)
        .expect("receive_message of empty body must succeed");

    assert_eq!(
        received.text_content().unwrap_or_default(),
        "",
        "empty payload must round-trip as empty string"
    );
}

// ============================================================================
// Test 8 — Receipt round-trip fires CoreDelegate::on_receipt_received
// ============================================================================

struct TestReceiptDelegate {
    receipts: std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>>,
    generic_messages: std::sync::Arc<std::sync::Mutex<usize>>,
}

impl scmessenger_core::CoreDelegate for TestReceiptDelegate {
    fn on_peer_discovered(&self, _peer_id: String) {}
    fn on_peer_disconnected(&self, _peer_id: String) {}
    fn on_peer_identified(
        &self,
        _peer_id: String,
        _agent_version: String,
        _listen_addrs: Vec<String>,
    ) {
    }
    fn on_message_received(
        &self,
        _sender_id: String,
        _sender_public_key_hex: String,
        _message_id: String,
        _sender_timestamp: u64,
        _data: Vec<u8>,
    ) {
        *self.generic_messages.lock().unwrap() += 1;
    }
    fn on_receipt_received(&self, message_id: String, status: String) {
        self.receipts.lock().unwrap().push((message_id, status));
    }
}

/// A receipt generated by Bob for a message from Alice round-trips back to Alice
/// and triggers CoreDelegate::on_receipt_received with status "Delivered".
#[test]
fn test_receipt_roundtrip_flips_state() {
    let alice = make_node();
    let bob = make_node();

    // Register delegate on Alice
    let receipts = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let generic_messages = std::sync::Arc::new(std::sync::Mutex::new(0usize));
    let delegate = TestReceiptDelegate {
        receipts: std::sync::Arc::clone(&receipts),
        generic_messages: std::sync::Arc::clone(&generic_messages),
    };
    alice.set_delegate(Some(Box::new(delegate)));

    // 1. Alice sends text message to Bob
    let prepared = alice
        .prepare_message(
            pubkey(&bob),
            "Hello Bob".to_string(),
            MessageType::Text,
            None,
        )
        .expect("prepare_message must succeed");

    // 2. Bob receives Alice's message
    let received_msg = bob
        .receive_message(prepared.envelope_data)
        .expect("receive_message must succeed");

    // 3. Bob prepares an encrypted receipt envelope for Alice
    let receipt_envelope = bob
        .prepare_receipt(pubkey(&alice), received_msg.id.clone())
        .expect("prepare_receipt must succeed");
    assert!(
        alice.outbox_contains_for_recipient(&pubkey(&bob), &prepared.message_id),
        "the original message must be pending before its delivery receipt arrives"
    );

    // 4. Alice receives Bob's receipt envelope
    let received_receipt = alice
        .receive_message(receipt_envelope)
        .expect("receive_message for receipt must succeed");
    assert_eq!(
        received_receipt.message_type,
        MessageType::Receipt,
        "prepare_receipt must return an encrypted receipt message envelope"
    );
    let decoded_receipt = scmessenger_core::decode_receipt(received_receipt.payload)
        .expect("received receipt payload must use the canonical receipt codec");
    assert_eq!(decoded_receipt.message_id, received_msg.id);

    // 5. The valid receipt keeps its dedicated callback/outbox behavior, but
    // never enters the user-facing message pipeline.
    assert_eq!(alice.inbox_count(), 0, "receipts must not enter the inbox");
    assert_eq!(
        alice.history_store_manager().count(),
        0,
        "receipts must not enter message history"
    );
    assert_eq!(
        *generic_messages.lock().unwrap(),
        0,
        "receipts must not trigger the generic message callback"
    );

    // A malformed receipt is still protocol metadata: receive_message returns
    // it to the caller for the receipt branch, but must not surface its payload.
    let malformed_envelope = bob
        .prepare_message(
            pubkey(&alice),
            "{malformed".to_string(),
            MessageType::Receipt,
            None,
        )
        .expect("prepare malformed receipt must succeed");
    let malformed_receipt = alice
        .receive_message(malformed_envelope.envelope_data)
        .expect("malformed receipt envelope must still decrypt");
    assert_eq!(
        malformed_receipt.message_type,
        MessageType::Receipt,
        "malformed receipt must remain available to the receipt branch"
    );
    assert_eq!(
        alice.inbox_count(),
        0,
        "malformed receipts must not enter the inbox"
    );
    assert_eq!(
        alice.history_store_manager().count(),
        0,
        "malformed receipts must not enter message history"
    );
    assert_eq!(
        *generic_messages.lock().unwrap(),
        0,
        "malformed receipts must not trigger the generic message callback"
    );

    let recorded = receipts.lock().unwrap();
    assert_eq!(
        recorded.len(),
        1,
        "Alice should receive exactly 1 receipt callback"
    );
    assert_eq!(
        recorded[0].0, received_msg.id,
        "Receipt message_id must match original message ID"
    );
    assert_eq!(
        recorded[0].1, "Delivered",
        "Receipt status must be Delivered"
    );
    assert!(
        !alice.outbox_contains_for_recipient(&pubkey(&bob), &prepared.message_id),
        "a Delivered receipt must clear the matching sender outbox entry"
    );
}

/// D4 coalescing: inbound history is stored under the sender's canonical
/// identity_id while a contact / thread lookup is often keyed by the same
/// identity's public_key_hex flavor. Querying the conversation by EITHER form
/// must return the same messages, so a single identity never appears as two
/// split threads.
#[test]
fn test_history_conversation_coalesces_pubkey_and_identity_flavors() {
    let alice = make_node();
    let bob = make_node();

    let bob_msg = bob
        .prepare_message(
            pubkey(&alice),
            "hello from bob (D4 coalesce)".to_string(),
            MessageType::Text,
            None,
        )
        .expect("prepare must succeed");
    let received = alice
        .receive_message(bob_msg.envelope_data)
        .expect("receive must succeed");

    // The stored inbound record is keyed by Bob's canonical identity_id.
    let record = alice
        .history_store_manager()
        .get(received.id.clone())
        .expect("history get must not error")
        .expect("inbound message must be recorded in history");
    let identity_id = record.peer_id;

    let by_identity = alice
        .history_store_manager()
        .conversation(identity_id.clone(), 50)
        .expect("conversation by identity_id must not error");
    assert_eq!(
        by_identity.len(),
        1,
        "inbound message must coalesce under the identity_id it was stored with"
    );

    let by_pubkey = alice
        .history_store_manager()
        .conversation(pubkey(&bob), 50)
        .expect("conversation by public key must not error");
    assert!(
        !by_pubkey.is_empty(),
        "D4: the public_key_hex flavor must reach the identity_id-keyed records (no split thread)"
    );
    assert_eq!(
        by_pubkey.len(),
        by_identity.len(),
        "D4: a thread keyed by public_key_hex must coalesce exactly with the identity_id-keyed records"
    );

    // No over-match: a THIRD party's public key must not reach Bob's records.
    let carol = make_node();
    let carol_thread = alice
        .history_store_manager()
        .conversation(pubkey(&carol), 50)
        .expect("conversation by carol pubkey must not error");
    assert!(
        carol_thread.is_empty(),
        "D4: a stranger's public key must not coalesce into Bob's thread"
    );

    // Delete symmetry: a thread visible under the pubkey flavor must be
    // deletable by that flavor (regression for the remove_conversation hole).
    alice
        .history_store_manager()
        .remove_conversation(pubkey(&bob))
        .expect("remove_conversation by pubkey flavor must not error");
    assert!(
        alice
            .history_store_manager()
            .conversation(pubkey(&bob), 50)
            .expect("conversation after removal must not error")
            .is_empty(),
        "D4: remove_conversation by pubkey flavor must delete the coalesced thread"
    );
    assert!(
        alice
            .history_store_manager()
            .conversation(identity_id.clone(), 50)
            .expect("conversation by identity_id after removal must not error")
            .is_empty(),
        "D4: removal by either flavor must empty the thread"
    );
}
