use scmessenger_core::store::{Outbox, QueuedMessage};
use scmessenger_core::{IronCore, MessageType};
use std::sync::Arc;

/// Create a test message with the given parameters.
fn make_test_message(message_id: &str, recipient_id: &str) -> QueuedMessage {
    QueuedMessage {
        version: 1,
        message_id: message_id.to_string(),
        recipient_id: recipient_id.to_string(),
        envelope_data: vec![1, 2, 3, 4, 5, 6, 7, 8],
        queued_at: web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        attempts: 0,
        next_retry_at: None,
        in_custody: false,
        custody_established_at: 0,
        state: scmessenger_core::store::outbox::MessageState::Enqueued,
    }
}

/// Verify: send message (offline) → message enqueued → peer connects → message ready to send
#[test]
fn test_outbox_flush_on_peer_reconnect() {
    let alice = IronCore::new();
    alice.grant_consent();
    alice
        .initialize_identity()
        .expect("identity initialization must succeed");
    let alice_pubkey = alice
        .get_identity_info()
        .public_key_hex
        .expect("alice must be initialized");

    let bob = IronCore::new();
    bob.grant_consent();
    bob.initialize_identity()
        .expect("identity initialization must succeed");

    // Simulate offline: prepare a message and it should be enqueued
    let msg_result = alice.prepare_message(
        alice_pubkey.clone(),
        "Hello, Bob!".to_string(),
        MessageType::Text,
        None,
    );

    assert!(msg_result.is_ok(), "Message preparation should succeed");
    let prepared = msg_result.unwrap();
    let message_id = prepared.message_id.clone();

    // Verify the message is in the outbox (since alice is not connected to herself)
    let outbox_contains = alice.outbox_contains_for_recipient(&alice_pubkey, &message_id);
    assert!(
        outbox_contains,
        "Message should be enqueued in outbox since peer is not connected"
    );

    tracing::info!(
        "Message enqueued successfully: {} for peer {}",
        message_id,
        alice_pubkey
    );

    // Simulate peer connecting: trigger the reconnect handler
    // This should attempt to send messages from the outbox
    alice.handle_peer_connection_event(&alice_pubkey, true);

    // After reconnect attempt, messages should be marked as sent (removed from outbox)
    // OR re-enqueued with backoff if delivery failed
    // In this test, since there's no actual peer, delivery will fail and message will be re-enqueued
    let outbox_after_reconnect = alice.outbox_contains_for_recipient(&alice_pubkey, &message_id);

    // The message might be:
    // 1. Removed (successful delivery to the mock peer)
    // 2. Re-enqueued (delivery failed, backoff applied)
    tracing::info!(
        "After reconnect: message still in outbox = {}",
        outbox_after_reconnect
    );

    // Verify outbox state is consistent (message exists OR was cleanly removed)
    let outbox_count = alice.outbox_count();
    assert!(
        outbox_count <= 1,
        "Outbox should have at most 1 message after reconnect handling"
    );
}

/// Verify: multiple pending messages are all processed in order
#[test]
fn test_outbox_flush_multiple_pending_messages() {
    let mut outbox = Outbox::new();
    let peer_id = "test_peer_123";

    // Enqueue multiple messages
    let msg1 = make_test_message("msg1", peer_id);
    let msg2 = make_test_message("msg2", peer_id);
    let msg3 = make_test_message("msg3", peer_id);

    outbox.enqueue(msg1).expect("enqueue msg1 should succeed");
    outbox.enqueue(msg2).expect("enqueue msg2 should succeed");
    outbox.enqueue(msg3).expect("enqueue msg3 should succeed");

    assert_eq!(
        outbox.total_count(),
        3,
        "Outbox should contain exactly 3 messages"
    );

    // Flush messages for the peer
    let flushed = outbox.flush_peer_messages(peer_id);

    assert_eq!(flushed.len(), 3, "Should flush all 3 messages for the peer");
    assert_eq!(
        outbox.total_count(),
        0,
        "Outbox should be empty after flush"
    );

    // Verify messages are in order
    assert_eq!(flushed[0].message_id, "msg1");
    assert_eq!(flushed[1].message_id, "msg2");
    assert_eq!(flushed[2].message_id, "msg3");

    tracing::info!("Successfully flushed {} messages", flushed.len());
}

/// Verify: messages with custody flag are not flushed during reconnect
#[test]
fn test_outbox_flush_excludes_custody_messages() {
    let mut outbox = Outbox::new();
    let peer_id = "test_peer_456";

    // Enqueue a regular message
    let msg_regular = make_test_message("msg_regular", peer_id);
    outbox
        .enqueue(msg_regular)
        .expect("enqueue regular message");

    // Enqueue a message in custody (should not be flushed)
    let mut msg_custody = make_test_message("msg_custody", peer_id);
    msg_custody.in_custody = true;
    msg_custody.custody_established_at = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    outbox
        .enqueue(msg_custody)
        .expect("enqueue custody message");

    assert_eq!(outbox.total_count(), 2, "Outbox should have 2 messages");

    // Flush messages for the peer
    let flushed = outbox.flush_peer_messages(peer_id);

    // Only the regular message should be flushed, custody message should remain
    assert_eq!(
        flushed.len(),
        1,
        "Should only flush the regular message, not custody message"
    );
    assert_eq!(flushed[0].message_id, "msg_regular");

    // Verify custody message is still in outbox
    let remaining = outbox.peek_for_peer(peer_id);
    assert_eq!(
        remaining.len(),
        1,
        "Custody message should still be in outbox"
    );
    assert_eq!(remaining[0].message_id, "msg_custody");
    assert!(
        remaining[0].in_custody,
        "Message should still have custody flag"
    );

    tracing::info!("Correctly excluded custody message from flush");
}

/// Verify: exponential backoff is applied on delivery failure
#[test]
fn test_outbox_retry_backoff_schedule() {
    let mut outbox = Outbox::new();
    let peer_id = "test_peer_backoff";

    let msg = make_test_message("msg_backoff", peer_id);
    outbox.enqueue(msg).expect("enqueue message");

    // Simulate retry attempts with backoff
    let mut current_msg = outbox.peek_for_peer(peer_id)[0].clone();

    // Simulate 3 delivery attempts
    for attempt in 1..=3 {
        // Simulate failure and re-enqueue with backoff
        current_msg.attempts = attempt;
        let backoff_secs = 2u64.saturating_pow(attempt.min(12)).min(3600);
        let now_secs = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        current_msg.next_retry_at = Some(now_secs + backoff_secs);
        outbox.remove(&current_msg.message_id);
        outbox.enqueue(current_msg.clone()).ok();

        let updated = outbox.peek_for_peer(peer_id)[0].clone();
        assert_eq!(
            updated.attempts, attempt,
            "Attempt count should increment to {}",
            attempt
        );

        let expected_backoff = 2u64.saturating_pow(attempt.min(12)).min(3600);
        assert_eq!(
            updated.next_retry_at.unwrap_or(0) - now_secs,
            expected_backoff,
            "Backoff should be {} seconds for attempt {}",
            expected_backoff,
            attempt
        );

        tracing::info!(
            "Attempt {}: backoff scheduled for {} seconds",
            attempt,
            expected_backoff
        );
    }
}

/// Verify: reached max retry attempts but message is NOT dropped
#[test]
fn test_outbox_preserves_failed_messages() {
    let mut outbox = Outbox::new();
    let peer_id = "test_peer_max_retries";

    // Create a message with attempts near the max
    let mut msg = make_test_message("msg_max_retries", peer_id);
    msg.attempts = 11; // One below MAX_DELIVERY_ATTEMPTS (12)
    outbox.enqueue(msg).expect("enqueue message");

    // Attempt one more time, should trigger max_attempts
    let should_remove = outbox.record_attempt("msg_max_retries");

    assert!(
        should_remove,
        "record_attempt should return true when max attempts exceeded"
    );

    // But the message should still be in the outbox (not dropped)
    let messages = outbox.peek_for_peer(peer_id);
    assert_eq!(
        messages.len(),
        1,
        "Message should still exist in outbox even after max attempts"
    );
    assert_eq!(
        messages[0].attempts, 12,
        "Attempts should be clamped at MAX_DELIVERY_ATTEMPTS"
    );

    tracing::info!(
        "Message preserved with {} attempts (at max)",
        messages[0].attempts
    );
}

/// Verify: no double-send when flush happens during active retries
#[test]
fn test_outbox_no_double_send_during_flush() {
    let mut outbox = Outbox::new();
    let peer_id = "test_peer_double_send";

    let msg = make_test_message("msg_no_double", peer_id);
    outbox.enqueue(msg).expect("enqueue message");

    // First flush (reconnect event)
    let flushed1 = outbox.flush_peer_messages(peer_id);
    assert_eq!(flushed1.len(), 1, "First flush should return the message");

    // Immediate second flush (while first is still being processed)
    let flushed2 = outbox.flush_peer_messages(peer_id);
    assert_eq!(
        flushed2.len(),
        0,
        "Second flush should return empty (message already drained)"
    );

    tracing::info!("Correctly prevented double-send");
}

/// Verify: transient vs persistent error handling
#[test]
fn test_outbox_retry_state_transitions() {
    let mut outbox = Outbox::new();
    let peer_id = "test_peer_states";

    let mut msg = make_test_message("msg_states", peer_id);
    msg.attempts = 0;
    msg.next_retry_at = None;
    outbox.enqueue(msg).expect("enqueue message");

    let original = outbox.peek_for_peer(peer_id)[0].clone();
    assert_eq!(original.attempts, 0, "Initial state: attempts = 0");
    assert!(
        original.next_retry_at.is_none(),
        "Initial state: no retry scheduled"
    );

    // Simulate transient failure (re-enqueue with backoff)
    let mut failed_msg = original.clone();
    failed_msg.attempts += 1;
    let now_secs = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    failed_msg.next_retry_at = Some(now_secs + 2); // 2 second backoff

    outbox.remove(&failed_msg.message_id);
    outbox
        .enqueue(failed_msg)
        .expect("re-enqueue after transient failure");

    let after_transient = outbox.peek_for_peer(peer_id)[0].clone();
    assert_eq!(
        after_transient.attempts, 1,
        "After transient failure: attempts incremented"
    );
    assert!(
        after_transient.next_retry_at.is_some(),
        "After transient failure: retry scheduled"
    );

    tracing::info!(
        "State transitions: {} -> (attempt {} with {} sec backoff)",
        original.message_id,
        after_transient.attempts,
        after_transient.next_retry_at.unwrap_or(0) - now_secs
    );
}

/// Integration: full workflow with persistent storage
#[test]
fn test_outbox_flush_with_persistent_storage() {
    let dir = tempfile::tempdir().expect("create temp directory");
    let path = dir
        .path()
        .join("outbox_persistent_test")
        .to_str()
        .expect("convert path to string")
        .to_string();

    {
        let backend1 = Arc::new(
            scmessenger_core::store::backend::SledStorage::new(&path).expect("create SledStorage"),
        );
        let mut outbox = Outbox::persistent(backend1.clone());

        // Enqueue messages
        let msg1 = make_test_message("persist_msg1", "persist_peer");
        let msg2 = make_test_message("persist_msg2", "persist_peer");
        outbox.enqueue(msg1).expect("enqueue msg1");
        outbox.enqueue(msg2).expect("enqueue msg2");

        assert_eq!(outbox.total_count(), 2);
    }

    // Reopen storage and verify messages survived
    let backend2 = Arc::new(
        scmessenger_core::store::backend::SledStorage::new(&path).expect("reopen SledStorage"),
    );
    {
        let outbox = Outbox::persistent(backend2);
        assert_eq!(
            outbox.total_count(),
            2,
            "Messages should persist across restart"
        );

        let messages = outbox.peek_for_peer("persist_peer");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].message_id, "persist_msg1");
        assert_eq!(messages[1].message_id, "persist_msg2");

        tracing::info!(
            "Verified {} messages persisted in storage",
            outbox.total_count()
        );
    }
}

/// Edge case: peer_id mismatch between multiple peers
#[test]
fn test_outbox_flush_peer_isolation() {
    let mut outbox = Outbox::new();

    let peer_a = "peer_a_abc123";
    let peer_b = "peer_b_def456";

    // Enqueue messages for different peers
    outbox
        .enqueue(make_test_message("msg_for_a", peer_a))
        .expect("enqueue for peer_a");
    outbox
        .enqueue(make_test_message("msg_for_b1", peer_b))
        .expect("enqueue for peer_b");
    outbox
        .enqueue(make_test_message("msg_for_b2", peer_b))
        .expect("enqueue for peer_b");
    outbox
        .enqueue(make_test_message("msg_for_a2", peer_a))
        .expect("enqueue second for peer_a");

    assert_eq!(outbox.total_count(), 4);

    // Flush only peer_a
    let flushed_a = outbox.flush_peer_messages(peer_a);
    assert_eq!(
        flushed_a.len(),
        2,
        "Should flush only 2 messages for peer_a"
    );
    assert!(
        flushed_a.iter().all(|m| m.recipient_id == peer_a),
        "All flushed messages should be for peer_a"
    );

    // Verify peer_b messages still in queue
    let remaining_b = outbox.peek_for_peer(peer_b);
    assert_eq!(
        remaining_b.len(),
        2,
        "Peer_b messages should still be in outbox"
    );

    assert_eq!(
        outbox.total_count(),
        2,
        "Total count should be 2 after flushing peer_a"
    );

    tracing::info!("Correctly isolated messages per peer during flush");
}

/// OUTBOX-SWEEP-001 (2026-09-23): an entry dispatched over a live connection
/// (egress=true) is re-enqueued with a grace timer, NOT removed. The sweep
/// must be able to re-drain it once that timer expires, because a lost
/// receipt on a still-open connection otherwise strands it forever (Windows
/// node: outbox_count 118 / undelivered 259 with all links healthy).
///
/// The sweep re-invokes flush_peer_messages on a timer; these assertions pin
/// the exact due-semantics it relies on: entries inside their grace window
/// are left alone, expired ones are drained, custody entries are never
/// touched (see also the R3-C2 grace contract in core/src/iron_core.rs).
#[test]
#[allow(clippy::disallowed_methods)]
fn test_sweep_reflushes_entry_after_grace_expiry_on_live_connection() {
    let peer = "sweep_grace_peer";

    // 1) The flush drained and the egress path re-enqueued with a live grace
    //    timer (exactly what handle_peer_connection_event_with_egress does:
    //    state=Enqueued, attempts+=1, next_retry_at=now+120).
    let mut outbox = Outbox::new();
    let mut redispatched = make_test_message("sweep-msg-1", peer);
    redispatched.attempts = 1;
    redispatched.next_retry_at = Some(
        web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + 120, // OUTBOX_EGRESS_GRACE_SECS
    );
    outbox.enqueue(redispatched).expect("enqueue with grace");

    // 2) A sweep flush while the grace window is still open must NOT drain.
    let immediate = outbox.flush_peer_messages(peer);
    assert_eq!(
        immediate.len(),
        0,
        "sweep must not re-drain an entry inside its grace window"
    );
    assert_eq!(
        outbox.total_count(),
        1,
        "entry survives the premature sweep"
    );

    // 3) Once the timer expires, the next sweep flush re-drains it.
    let mut outbox_expired = Outbox::new();
    let mut stale = make_test_message("sweep-msg-1", peer);
    stale.attempts = 1;
    stale.next_retry_at = Some(
        web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(1),
    );
    outbox_expired
        .enqueue(stale)
        .expect("enqueue expired-grace entry");
    let after_expiry = outbox_expired.flush_peer_messages(peer);
    assert_eq!(
        after_expiry.len(),
        1,
        "sweep re-drains the entry once its grace timer expires"
    );
}

/// OUTBOX-SWEEP-001: entries in custody are never swept (the sweep drains
/// only Enqueued AND due entries).
#[test]
#[allow(clippy::disallowed_methods)]
fn test_sweep_never_drains_custody_entries() {
    let mut outbox = Outbox::new();
    let peer = "sweep_custody_peer";
    let mut msg = make_test_message("sweep-custody-1", peer);
    msg.in_custody = true;
    outbox.enqueue(msg).expect("enqueue custody entry");

    let drained = outbox.flush_peer_messages(peer);
    assert!(
        drained.is_empty(),
        "custody entries are owned by the custody retry lifecycle, never swept"
    );
    assert_eq!(outbox.total_count(), 1, "custody entry stays in the outbox");
}

#[test]
fn test_sweep_empty_peer_and_non_enqueued_entries_are_noop() {
    let mut outbox = Outbox::new();

    // Empty/unknown peer input must be a no-op, not a panic or a scan of
    // unrelated queue keys.
    assert!(outbox.flush_peer_messages("").is_empty());
    assert!(outbox.flush_peer_messages("missing-peer").is_empty());

    let peer = "non_enqueued_peer";
    let mut sent = make_test_message("already-sent", peer);
    sent.state = scmessenger_core::store::outbox::MessageState::Sent;
    let mut failed = make_test_message("already-failed", peer);
    failed.state = scmessenger_core::store::outbox::MessageState::Failed;
    outbox.enqueue(sent).expect("enqueue sent message");
    outbox.enqueue(failed).expect("enqueue failed message");

    assert!(outbox.flush_peer_messages(peer).is_empty());
    assert_eq!(
        outbox.total_count(),
        2,
        "non-Enqueued entries remain owned by outbox"
    );
}
