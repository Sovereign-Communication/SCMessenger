//! End-to-End Integration Tests for SCMessenger
//!
//! These tests verify the complete message flow across all layers:
//! 1. Identity generation
//! 2. Message encryption
//! 3. Store-and-forward persistence
//! 4. Transport layer handling
//! 5. Message decryption and delivery
//!
//! Run with: cargo test --test integration_e2e

use scmessenger_core::crypto::encrypt::{
    decrypt_message, encrypt_message, sign_envelope, verify_envelope,
};
use scmessenger_core::identity::{IdentityKeys, IdentityStore};
use scmessenger_core::message::{Envelope, Message, MessageType};
use scmessenger_core::store::backend::SledStorage;
use scmessenger_core::store::{Inbox, Outbox, QueuedMessage};
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_e2e_message_flow_two_peers() {
    // Test scenario: Alice sends a message to Bob
    // This verifies the complete encryption -> storage -> decryption flow

    // Step 1: Generate identities for Alice and Bob
    let alice_keys = IdentityKeys::generate();
    let bob_keys = IdentityKeys::generate();

    let alice_id = alice_keys.identity_id();
    let bob_id = bob_keys.identity_id();

    println!("Alice ID: {}", alice_id);
    println!("Bob ID: {}", bob_id);

    // Step 2: Alice creates a message for Bob
    let message_text = "Hello Bob, this is a test message from Alice!";
    // sender_id is canonicalized to the sender's Ed25519 public key.
    let message = Message::text(alice_keys.public_key_hex(), bob_id.clone(), message_text);

    // Step 3: Alice encrypts the message
    let message_bytes = bincode::serialize(&message).expect("Failed to serialize message");
    let bob_public_key = bob_keys.signing_key.verifying_key().to_bytes();

    let envelope = encrypt_message(&alice_keys.signing_key, &bob_public_key, &message_bytes)
        .expect("Failed to encrypt message");

    // Verify envelope structure
    assert_eq!(envelope.sender_public_key.len(), 32);
    assert_eq!(envelope.ephemeral_public_key.len(), 32);
    assert_eq!(envelope.nonce.len(), 24);
    assert!(!envelope.ciphertext.is_empty());

    // Step 4: Sign the envelope for relay verification
    let signed_envelope =
        sign_envelope(envelope.clone(), &alice_keys.signing_key).expect("Failed to sign envelope");

    // Verify signature
    verify_envelope(&signed_envelope).expect("Signature verification failed");

    // Step 5: Store in outbox (Alice's side)
    let mut alice_outbox = Outbox::new();
    let envelope_data = bincode::serialize(&signed_envelope).expect("Failed to serialize envelope");

    let queued_msg = QueuedMessage {
        version: 1,
        message_id: message.id.clone(),
        recipient_id: bob_id.clone(),
        envelope_data,
        queued_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        attempts: 0,
        next_retry_at: None,
        in_custody: false,
        custody_established_at: 0,
        state: scmessenger_core::store::outbox::MessageState::Enqueued,
    };

    alice_outbox
        .enqueue(queued_msg)
        .expect("Failed to enqueue message");
    assert_eq!(alice_outbox.total_count(), 1);

    // Step 6: Simulate delivery - Bob receives the envelope
    let bob_messages = alice_outbox.peek_for_peer(&bob_id);
    assert_eq!(bob_messages.len(), 1);

    let received_envelope_data = &bob_messages[0].envelope_data;
    let received_signed_envelope: scmessenger_core::message::SignedEnvelope =
        bincode::deserialize(received_envelope_data).expect("Failed to deserialize envelope");

    // Step 7: Verify envelope signature (relay would do this)
    verify_envelope(&received_signed_envelope).expect("Received envelope signature invalid");

    // Step 8: Bob decrypts the message
    let decrypted_bytes =
        decrypt_message(&bob_keys.signing_key, &received_signed_envelope.envelope)
            .expect("Failed to decrypt message");

    let received_message: Message =
        bincode::deserialize(&decrypted_bytes).expect("Failed to deserialize message");

    // Step 9: Store in inbox (Bob's side)
    let mut bob_inbox = Inbox::new();
    let received_msg = scmessenger_core::store::ReceivedMessage {
        version: 1,
        message_id: received_message.id.clone(),
        // Inbox is keyed by the stable identity_id, not the wire public key.
        sender_id: alice_keys.identity_id(),
        payload: received_message.payload.clone(),
        received_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        sender_public_key_hex: None,
    };

    assert!(bob_inbox.receive(received_msg), "Failed to receive message");
    assert_eq!(bob_inbox.total_count(), 1);

    // Step 10: Verify message content
    // sender_id carries Alice's canonical identifier: her Ed25519 public key.
    assert_eq!(received_message.sender_id, alice_keys.public_key_hex());
    assert_eq!(received_message.recipient_id, bob_id);
    assert_eq!(received_message.message_type, MessageType::Text);
    assert_eq!(received_message.text_content().unwrap(), message_text);

    // Step 11: Verify deduplication
    let duplicate_msg = scmessenger_core::store::ReceivedMessage {
        version: 1,
        message_id: received_message.id.clone(),
        // Same message, same canonical inbound key (identity_id).
        sender_id: alice_keys.identity_id(),
        payload: received_message.payload.clone(),
        received_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        sender_public_key_hex: None,
    };

    assert!(!bob_inbox.receive(duplicate_msg), "Deduplication failed");
    assert_eq!(bob_inbox.total_count(), 1); // Still only 1 message

    println!("[OK] E2E test passed: Complete message flow verified");
}

#[test]
fn test_e2e_persistent_message_flow() {
    // Test scenario: Verify message persistence across restarts
    // This tests that messages survive application crashes

    let dir = tempdir().expect("Failed to create temp dir");
    let alice_store_path = dir.path().join("alice_store");
    let bob_store_path = dir.path().join("bob_store");
    let outbox_path = dir.path().join("outbox");
    let inbox_path = dir.path().join("inbox");

    // Step 1: First session - Alice sends message
    let message_id = {
        // Generate identities
        let alice_keys = IdentityKeys::generate();
        let bob_keys = IdentityKeys::generate();

        // Persist Alice's identity
        let alice_backend = Arc::new(SledStorage::new(alice_store_path.to_str().unwrap()).unwrap());
        let alice_identity_store = IdentityStore::persistent(alice_backend);
        alice_identity_store
            .save_keys(&alice_keys)
            .expect("Failed to save Alice's keys");

        // Persist Bob's identity
        let bob_backend = Arc::new(SledStorage::new(bob_store_path.to_str().unwrap()).unwrap());
        let bob_identity_store = IdentityStore::persistent(bob_backend);
        bob_identity_store
            .save_keys(&bob_keys)
            .expect("Failed to save Bob's keys");

        // Create and encrypt message
        let message = Message::text(
            alice_keys.identity_id(),
            bob_keys.identity_id(),
            "Persistent message test",
        );
        let message_id = message.id.clone();

        let message_bytes = bincode::serialize(&message).unwrap();
        let bob_public = bob_keys.signing_key.verifying_key().to_bytes();
        let envelope =
            encrypt_message(&alice_keys.signing_key, &bob_public, &message_bytes).unwrap();
        let signed_envelope = sign_envelope(envelope, &alice_keys.signing_key).unwrap();
        let envelope_data = bincode::serialize(&signed_envelope).unwrap();

        // Store in persistent outbox
        let outbox_backend = Arc::new(SledStorage::new(outbox_path.to_str().unwrap()).unwrap());
        let mut outbox = Outbox::persistent(outbox_backend);

        let queued_msg = QueuedMessage {
            version: 1,
            message_id: message_id.clone(),
            recipient_id: bob_keys.identity_id(),
            envelope_data,
            queued_at: 0,
            attempts: 0,
            next_retry_at: None,
            in_custody: false,
            custody_established_at: 0,
            state: scmessenger_core::store::outbox::MessageState::Enqueued,
        };

        outbox.enqueue(queued_msg).expect("Failed to enqueue");
        assert_eq!(outbox.total_count(), 1);

        message_id
    };

    // Step 2: Simulate restart - All variables dropped, new session begins
    {
        // Load Alice's identity
        let alice_backend = Arc::new(SledStorage::new(alice_store_path.to_str().unwrap()).unwrap());
        let alice_identity_store = IdentityStore::persistent(alice_backend);
        let _alice_keys = alice_identity_store
            .load_keys()
            .expect("Failed to load keys")
            .expect("Alice's keys not found");

        // Load Bob's identity
        let bob_backend = Arc::new(SledStorage::new(bob_store_path.to_str().unwrap()).unwrap());
        let bob_identity_store = IdentityStore::persistent(bob_backend);
        let bob_keys = bob_identity_store
            .load_keys()
            .expect("Failed to load keys")
            .expect("Bob's keys not found");

        // Reopen persistent outbox
        let outbox_backend = Arc::new(SledStorage::new(outbox_path.to_str().unwrap()).unwrap());
        let outbox = Outbox::persistent(outbox_backend);

        // Verify message still in outbox
        assert_eq!(outbox.total_count(), 1);

        let bob_id = bob_keys.identity_id();
        let queued_messages = outbox.peek_for_peer(&bob_id);
        assert_eq!(queued_messages.len(), 1);
        assert_eq!(queued_messages[0].message_id, message_id);

        // Deliver message to Bob
        let envelope_data = &queued_messages[0].envelope_data;
        let signed_envelope: scmessenger_core::message::SignedEnvelope =
            bincode::deserialize(envelope_data).unwrap();

        let decrypted_bytes =
            decrypt_message(&bob_keys.signing_key, &signed_envelope.envelope).unwrap();
        let received_message: Message = bincode::deserialize(&decrypted_bytes).unwrap();

        // Store in persistent inbox
        let inbox_backend = Arc::new(SledStorage::new(inbox_path.to_str().unwrap()).unwrap());
        let mut inbox = Inbox::persistent(inbox_backend);

        let received_msg = scmessenger_core::store::ReceivedMessage {
            version: 1,
            message_id: received_message.id.clone(),
            sender_id: received_message.sender_id.clone(),
            payload: received_message.payload.clone(),
            received_at: 1,
            sender_public_key_hex: None,
        };

        inbox.receive(received_msg);
        assert_eq!(inbox.total_count(), 1);
    }

    // Step 3: Simulate another restart - Verify inbox persistence
    {
        let inbox_backend = Arc::new(SledStorage::new(inbox_path.to_str().unwrap()).unwrap());
        let inbox = Inbox::persistent(inbox_backend);

        // Message should still be there
        assert_eq!(inbox.total_count(), 1);

        // Should still be marked as duplicate
        assert!(inbox.is_duplicate(&message_id));

        let all_messages = inbox.all_messages();
        assert_eq!(all_messages.len(), 1);
        assert_eq!(all_messages[0].message_id, message_id);
    }

    println!("[OK] Persistent E2E test passed: Messages survive restarts");
}

#[test]
fn test_e2e_multi_peer_scenario() {
    // Test scenario: Alice broadcasts to Bob and Carol
    // Verifies multi-recipient handling and message fanout

    let alice_keys = IdentityKeys::generate();
    let bob_keys = IdentityKeys::generate();
    let carol_keys = IdentityKeys::generate();

    let alice_id = alice_keys.identity_id();
    let bob_id = bob_keys.identity_id();
    let carol_id = carol_keys.identity_id();

    // Alice creates messages for both Bob and Carol
    let message_to_bob = Message::text(alice_id.clone(), bob_id.clone(), "Hi Bob!");
    let message_to_carol = Message::text(alice_id.clone(), carol_id.clone(), "Hi Carol!");

    // Encrypt both messages
    let bob_public = bob_keys.signing_key.verifying_key().to_bytes();
    let carol_public = carol_keys.signing_key.verifying_key().to_bytes();

    let bob_envelope = encrypt_message(
        &alice_keys.signing_key,
        &bob_public,
        &bincode::serialize(&message_to_bob).unwrap(),
    )
    .unwrap();

    let carol_envelope = encrypt_message(
        &alice_keys.signing_key,
        &carol_public,
        &bincode::serialize(&message_to_carol).unwrap(),
    )
    .unwrap();

    // Queue both messages in outbox
    let mut outbox = Outbox::new();

    outbox
        .enqueue(QueuedMessage {
            version: 1,
            message_id: message_to_bob.id.clone(),
            recipient_id: bob_id.clone(),
            envelope_data: bincode::serialize(&bob_envelope).unwrap(),
            queued_at: 0,
            attempts: 0,
            next_retry_at: None,
            in_custody: false,
            custody_established_at: 0,
            state: scmessenger_core::store::outbox::MessageState::Enqueued,
        })
        .unwrap();

    outbox
        .enqueue(QueuedMessage {
            version: 1,
            message_id: message_to_carol.id.clone(),
            recipient_id: carol_id.clone(),
            envelope_data: bincode::serialize(&carol_envelope).unwrap(),
            queued_at: 0,
            attempts: 0,
            next_retry_at: None,
            in_custody: false,
            custody_established_at: 0,
            state: scmessenger_core::store::outbox::MessageState::Enqueued,
        })
        .unwrap();

    assert_eq!(outbox.total_count(), 2);
    assert_eq!(outbox.peer_count(), 2);

    // Bob receives his message
    let bob_messages = outbox.peek_for_peer(&bob_id);
    assert_eq!(bob_messages.len(), 1);

    let bob_envelope: Envelope = bincode::deserialize(&bob_messages[0].envelope_data).unwrap();
    let bob_decrypted = decrypt_message(&bob_keys.signing_key, &bob_envelope).unwrap();
    let bob_msg: Message = bincode::deserialize(&bob_decrypted).unwrap();

    assert_eq!(bob_msg.text_content().unwrap(), "Hi Bob!");

    // Carol receives her message
    let carol_messages = outbox.peek_for_peer(&carol_id);
    assert_eq!(carol_messages.len(), 1);

    let carol_envelope: Envelope = bincode::deserialize(&carol_messages[0].envelope_data).unwrap();
    let carol_decrypted = decrypt_message(&carol_keys.signing_key, &carol_envelope).unwrap();
    let carol_msg: Message = bincode::deserialize(&carol_decrypted).unwrap();

    assert_eq!(carol_msg.text_content().unwrap(), "Hi Carol!");

    println!("[OK] Multi-peer E2E test passed: Message fanout works correctly");
}

#[test]
fn test_e2e_sender_spoofing_prevention() {
    // Test scenario: Attacker tries to spoof sender identity
    // This tests the AAD binding security feature

    let alice_keys = IdentityKeys::generate();
    let bob_keys = IdentityKeys::generate();
    let attacker_keys = IdentityKeys::generate();

    // Alice encrypts a message for Bob
    let message = Message::text(
        alice_keys.identity_id(),
        bob_keys.identity_id(),
        "Secret message",
    );

    let bob_public = bob_keys.signing_key.verifying_key().to_bytes();
    let mut envelope = encrypt_message(
        &alice_keys.signing_key,
        &bob_public,
        &bincode::serialize(&message).unwrap(),
    )
    .unwrap();

    // Attacker intercepts and tries to replace sender public key
    envelope.sender_public_key = attacker_keys
        .signing_key
        .verifying_key()
        .to_bytes()
        .to_vec();

    // Bob tries to decrypt - should fail due to AAD mismatch
    let result = decrypt_message(&bob_keys.signing_key, &envelope);
    assert!(
        result.is_err(),
        "AAD binding should prevent sender spoofing"
    );

    println!("[OK] Security test passed: Sender spoofing prevented by AAD binding");
}

#[test]
fn test_e2e_relay_verification() {
    // Test scenario: Relay verifies envelope signature without decryption
    // This tests the envelope signature security feature

    let alice_keys = IdentityKeys::generate();
    let bob_keys = IdentityKeys::generate();
    let attacker_keys = IdentityKeys::generate();

    // Alice creates and signs an envelope
    let message = Message::text(
        alice_keys.identity_id(),
        bob_keys.identity_id(),
        "Relay test",
    );

    let bob_public = bob_keys.signing_key.verifying_key().to_bytes();
    let envelope = encrypt_message(
        &alice_keys.signing_key,
        &bob_public,
        &bincode::serialize(&message).unwrap(),
    )
    .unwrap();

    let signed_envelope = sign_envelope(envelope.clone(), &alice_keys.signing_key).unwrap();

    // Relay verifies signature (without decryption)
    assert!(
        verify_envelope(&signed_envelope).is_ok(),
        "Valid signature should verify"
    );

    // Attacker tries to tamper with envelope
    let mut tampered = signed_envelope.clone();
    if let Some(byte) = tampered.envelope.ciphertext.last_mut() {
        *byte ^= 0xFF;
    }

    // Relay detects tampering
    assert!(
        verify_envelope(&tampered).is_err(),
        "Tampered envelope should fail verification"
    );

    // Attacker tries to forge signature
    let forged_envelope = envelope.clone();
    let forged_signed = sign_envelope(forged_envelope, &attacker_keys.signing_key).unwrap();

    // Can't use Alice's public key with attacker's signature
    let mut forgery_attempt = forged_signed;
    forgery_attempt.envelope.sender_public_key =
        alice_keys.signing_key.verifying_key().to_bytes().to_vec();

    assert!(
        verify_envelope(&forgery_attempt).is_err(),
        "Forged signature should fail verification"
    );

    println!("[OK] Relay security test passed: Envelope signatures prevent tampering");
}
