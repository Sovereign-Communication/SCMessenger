use proptest::prelude::*;
use scmessenger_core::routing::OptimizedRoutingEngine;
use scmessenger_core::store::backend::SledStorage;
use scmessenger_core::store::{Outbox, QueuedMessage};
use scmessenger_core::{IronCore, MessageType};
use std::sync::Arc;

/// Stand up an initialised IronCore instance with a live routing engine
/// wired in (needed so `prepare_message` can produce real StoreAndCarry vs
/// live routing decisions instead of always defaulting to the outbox).
fn make_core_with_routing() -> IronCore {
    let core = IronCore::new();
    core.grant_consent();
    core.initialize_identity()
        .expect("identity initialization must succeed");
    *core.routing_engine_handle().write() = Some(OptimizedRoutingEngine::new([0u8; 32], [0u8; 8]));
    core
}

fn pubkey(node: &IronCore) -> String {
    node.get_identity_info()
        .public_key_hex
        .expect("node must be initialized before calling pubkey()")
}

/// Mirror `prepare_message_internal`'s recipient-hint derivation
/// (`blake3::hash(recipient_id.as_bytes())[0..8]`, hex-encoded) so tests can
/// mark/clear the same hint the routing engine's negative cache keys on.
fn recipient_hint_hex(recipient_pubkey_hex: &str) -> String {
    let hint: [u8; 8] = blake3::hash(recipient_pubkey_hex.as_bytes()).as_bytes()[0..8]
        .try_into()
        .unwrap_or([0u8; 8]);
    hex::encode(hint)
}

fn make_msg(id: &str, recipient: &str, attempts: u32) -> QueuedMessage {
    QueuedMessage {
        version: 1,
        message_id: id.to_string(),
        recipient_id: recipient.to_string(),
        envelope_data: vec![1, 2, 3, 4],
        queued_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        attempts,
        next_retry_at: None,
        in_custody: false,
        custody_established_at: 0,
        state: scmessenger_core::store::outbox::MessageState::Enqueued,
    }
}

#[test]
fn retry_attempt_state_persists_across_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir
        .path()
        .join("retry_outbox")
        .to_str()
        .unwrap()
        .to_string();

    {
        let backend = Arc::new(SledStorage::new(&path).unwrap());
        let mut outbox = Outbox::persistent(backend);
        outbox.enqueue(make_msg("msg-retry", "peer-a", 0)).unwrap();
        outbox.record_attempt("msg-retry");
        outbox.record_attempt("msg-retry");
        outbox.record_attempt("msg-retry");
    }

    {
        let backend = Arc::new(SledStorage::new(&path).unwrap());
        let outbox = Outbox::persistent(backend);
        let queued = outbox.peek_for_peer("peer-a");
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].message_id, "msg-retry");
        assert_eq!(queued[0].attempts, 3);
    }
}

#[test]
fn undelivered_message_does_not_transition_to_terminal_drop() {
    let mut outbox = Outbox::new();
    outbox
        .enqueue(make_msg("msg-undelivered", "peer-b", u32::MAX - 2))
        .unwrap();

    // Simulate repeated retry ticks over a long-lived undelivered flow.
    for _ in 0..50 {
        outbox.record_attempt("msg-undelivered");
    }

    let queued = outbox.peek_for_peer("peer-b");
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].message_id, "msg-undelivered");
    assert_eq!(queued[0].attempts, u32::MAX);
}

#[test]
fn test_custody_ownership_mutual_exclusion() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir
        .path()
        .join("retry_outbox")
        .to_str()
        .unwrap()
        .to_string();

    let backend = std::sync::Arc::new(SledStorage::new(&path).unwrap());
    let mut outbox = Outbox::persistent(backend);

    // Message is enqueued initially
    outbox
        .enqueue(make_msg("msg-custody", "peer-a", 0))
        .unwrap();
    let queued = outbox.peek_for_peer("peer-a");
    assert_eq!(queued.len(), 1);

    // When moved to Drift custody (StoreAndCarry), it is removed from the outbox queue
    let removed = outbox.remove("msg-custody");
    assert!(removed);

    let queued = outbox.peek_for_peer("peer-a");
    assert_eq!(queued.len(), 0);
}

// ============================================================================
// T2.5 — Outbox retry x Drift custody convergence, driven through the real
// IronCore send path (`prepare_message_internal`'s routing decision), not a
// hand-simulated Outbox/MeshStore pair.
// ============================================================================

/// Forcing a StoreAndCarry routing decision (recipient marked definitely
/// unreachable) must hand the message to drift custody and never enqueue it
/// in the active outbox - there is nothing there for the retry loop to keep
/// retrying.
#[test]
fn outbox_stops_retrying_when_route_is_store_and_carry() {
    let alice = make_core_with_routing();
    let bob = IronCore::new();
    bob.grant_consent();
    bob.initialize_identity()
        .expect("bob identity initialization must succeed");
    let bob_pubkey = pubkey(&bob);
    let hint_hex = recipient_hint_hex(&bob_pubkey);

    alice
        .routing_engine_handle()
        .write()
        .as_mut()
        .expect("routing engine must be set")
        .record_unreachable_peer(&hint_hex);

    let prepared = alice
        .prepare_message(
            bob_pubkey.clone(),
            "carried".to_string(),
            MessageType::Text,
            None,
        )
        .expect("prepare_message must succeed even when store-and-carry");

    assert!(
        alice.drift_contains(&prepared.message_id),
        "message must be handed off to drift custody when the route is StoreAndCarry"
    );
    assert!(
        !alice.outbox_contains_for_recipient(&bob_pubkey, &prepared.message_id),
        "message must NOT also be queued in the active outbox - nothing there to retry"
    );
    assert_eq!(alice.outbox_count(), 0);
    assert_eq!(alice.drift_store_size(), 1);
}

/// Once a route is restored (negative-cache entry cleared), new messages to
/// that recipient must flow through the live outbox again - but the message
/// that was already handed to drift custody while the route was down must
/// stay there untouched, with no window where both stores own either
/// message. `mark_message_sent` (the delivery-receipt convergence path)
/// clears whichever store actually holds a given message, regardless of
/// which one that is.
#[test]
fn restored_route_sends_new_messages_via_outbox_without_disturbing_existing_custody() {
    let alice = make_core_with_routing();
    let bob = IronCore::new();
    bob.grant_consent();
    bob.initialize_identity()
        .expect("bob identity initialization must succeed");
    let bob_pubkey = pubkey(&bob);
    let hint_hex = recipient_hint_hex(&bob_pubkey);

    // First message: Bob unreachable -> StoreAndCarry -> drift custody.
    alice
        .routing_engine_handle()
        .write()
        .as_mut()
        .unwrap()
        .record_unreachable_peer(&hint_hex);
    let custodied = alice
        .prepare_message(
            bob_pubkey.clone(),
            "stuck in custody".to_string(),
            MessageType::Text,
            None,
        )
        .expect("prepare_message must succeed");
    assert!(alice.drift_contains(&custodied.message_id));

    // Route restored: clear the negative-cache entry (equivalent to the
    // peer coming back online / a fresh direct route being confirmed).
    alice
        .routing_engine_handle()
        .write()
        .as_mut()
        .unwrap()
        .clear_unreachable_peer(&hint_hex);

    // A second message to the same recipient should now go through the live
    // outbox - exactly one delivery path per message, no duplication.
    let live = alice
        .prepare_message(
            bob_pubkey.clone(),
            "route is back".to_string(),
            MessageType::Text,
            None,
        )
        .expect("prepare_message must succeed");

    assert!(
        alice.outbox_contains_for_recipient(&bob_pubkey, &live.message_id),
        "message must be queued in the active outbox once the route is direct again"
    );
    assert!(
        !alice.drift_contains(&live.message_id),
        "the new message must not also land in drift custody"
    );
    assert!(
        alice.drift_contains(&custodied.message_id),
        "the earlier custodied message must remain in drift - restoring the route \
         doesn't retroactively touch already-custodied messages"
    );
    assert!(
        !alice.outbox_contains_for_recipient(&bob_pubkey, &custodied.message_id),
        "the earlier custodied message must never also appear in the outbox"
    );

    // Delivery-receipt convergence clears whichever store actually holds
    // the message, by message_id, regardless of which one that is.
    assert!(alice.mark_message_sent(custodied.message_id.clone()));
    assert!(!alice.drift_contains(&custodied.message_id));
    assert!(alice.mark_message_sent(live.message_id.clone()));
    assert!(!alice.outbox_contains_for_recipient(&bob_pubkey, &live.message_id));
}

#[derive(Debug, Clone)]
enum CustodyOp {
    SendWhileUnreachable,
    SendWhileReachable,
    DeliverPreviouslySent(usize),
}

proptest::proptest! {
    /// State-transition property: across any sequence of routing-decision
    /// flips (recipient marked unreachable / reachable) and out-of-order
    /// delivery confirmations, no message the real IronCore send path
    /// prepared is ever simultaneously present in both the active outbox
    /// and drift custody.
    #[test]
    fn property_message_never_owned_by_both_outbox_and_drift(
        ops in proptest::collection::vec(
            prop_oneof![
                Just(CustodyOp::SendWhileUnreachable),
                Just(CustodyOp::SendWhileReachable),
                (0usize..20).prop_map(CustodyOp::DeliverPreviouslySent),
            ],
            1..30,
        )
    ) {
        let alice = make_core_with_routing();
        let bob = IronCore::new();
        bob.grant_consent();
        bob.initialize_identity().expect("bob identity initialization must succeed");
        let bob_pubkey = pubkey(&bob);
        let hint_hex = recipient_hint_hex(&bob_pubkey);

        let mut sent_ids: Vec<String> = Vec::new();

        for op in ops {
            match op {
                CustodyOp::SendWhileUnreachable => {
                    alice.routing_engine_handle().write().as_mut().unwrap()
                        .record_unreachable_peer(&hint_hex);
                    if let Ok(prepared) = alice.prepare_message(
                        bob_pubkey.clone(), "x".to_string(), MessageType::Text, None,
                    ) {
                        sent_ids.push(prepared.message_id);
                    }
                }
                CustodyOp::SendWhileReachable => {
                    alice.routing_engine_handle().write().as_mut().unwrap()
                        .clear_unreachable_peer(&hint_hex);
                    if let Ok(prepared) = alice.prepare_message(
                        bob_pubkey.clone(), "y".to_string(), MessageType::Text, None,
                    ) {
                        sent_ids.push(prepared.message_id);
                    }
                }
                CustodyOp::DeliverPreviouslySent(idx) => {
                    if !sent_ids.is_empty() {
                        let i = idx % sent_ids.len();
                        alice.mark_message_sent(sent_ids[i].clone());
                    }
                }
            }

            for id in &sent_ids {
                let in_outbox = alice.outbox_contains_for_recipient(&bob_pubkey, id);
                let in_drift = alice.drift_contains(id);
                proptest::prop_assert!(
                    !(in_outbox && in_drift),
                    "message {} owned by both outbox and drift custody simultaneously",
                    id
                );
            }
        }
    }
}
