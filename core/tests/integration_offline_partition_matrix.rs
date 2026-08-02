use libp2p::PeerId;
use scmessenger_core::store::backend::SledStorage;
use scmessenger_core::store::QueuedMessage;
use scmessenger_core::store::{CustodyState, Outbox, RelayCustodyStore};
use scmessenger_core::transport::mesh_routing::{
    MultiPathDelivery, ROUTE_REASON_DIRECT_FIRST, ROUTE_REASON_RELAY_RECENCY_SUCCESS,
};
use std::sync::Arc;

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn queued_message(message_id: &str, recipient_id: &str) -> QueuedMessage {
    QueuedMessage {
        version: 1,
        message_id: message_id.to_string(),
        recipient_id: recipient_id.to_string(),
        envelope_data: vec![7, 1, 7, 1],
        queued_at: now_secs(),
        attempts: 0,
        next_retry_at: None,
        in_custody: false,
        custody_established_at: 0,
        state: scmessenger_core::store::outbox::MessageState::Enqueued,
    }
}

#[test]
fn offline_partition_custody_and_outbox_survive_restart_until_reconnect_delivery() {
    let dir = tempfile::tempdir().unwrap();
    let outbox_path = dir.path().join("outbox").to_string_lossy().to_string();
    let custody_path = dir.path().join("custody").to_string_lossy().to_string();

    let recipient_id = "peer-offline-partition";
    let message_id = "ws12-msg-001";
    let relay_message_id = "ws12-relay-msg-001";

    let accepted_custody_id = {
        let outbox_backend = Arc::new(SledStorage::new(&outbox_path).unwrap());
        let mut outbox = Outbox::persistent(outbox_backend);
        outbox
            .enqueue(queued_message(message_id, recipient_id))
            .expect("sender should queue message while recipient is offline");

        let custody_backend = Arc::new(SledStorage::new(&custody_path).unwrap());
        let custody = RelayCustodyStore::persistent(custody_backend);
        let accepted = custody
            .accept_custody(
                "peer-sender".to_string(),
                recipient_id.to_string(),
                relay_message_id.to_string(),
                vec![9, 9, 9],
                None,
                None,
            )
            .expect("relay should accept custody while destination is offline");

        for attempt in 1..=3u32 {
            outbox.record_attempt(message_id);
            custody
                .mark_dispatching(
                    recipient_id,
                    &accepted.custody_id,
                    "partition_dispatch_attempt",
                )
                .expect("dispatch transition should be recorded");
            custody
                .mark_dispatch_failed(recipient_id, &accepted.custody_id, "partition_still_active")
                .expect("partition failure should return custody to accepted");

            let pending = custody.pending_for_destination(recipient_id, 10);
            assert_eq!(pending.len(), 1);
            assert_eq!(pending[0].state, CustodyState::Accepted);
            assert_eq!(pending[0].delivery_attempts, attempt);
        }

        accepted.custody_id
    };

    {
        let outbox_backend = Arc::new(SledStorage::new(&outbox_path).unwrap());
        let mut outbox = Outbox::persistent(outbox_backend);
        let queued = outbox.peek_for_peer(recipient_id);
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].message_id, message_id);
        assert_eq!(queued[0].attempts, 3);

        let custody_backend = Arc::new(SledStorage::new(&custody_path).unwrap());
        let custody = RelayCustodyStore::persistent(custody_backend);
        let pending = custody.pending_for_destination(recipient_id, 10);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].custody_id, accepted_custody_id);
        assert_eq!(pending[0].state, CustodyState::Accepted);

        custody
            .mark_dispatching(recipient_id, &accepted_custody_id, "reconnect_dispatch")
            .expect("reconnect should resume dispatching");
        custody
            .mark_delivered(recipient_id, &accepted_custody_id, "recipient_reconnected")
            .expect("reconnect should complete delivery");
        assert!(outbox.remove(message_id));

        let transitions = custody.transitions_for_custody(&accepted_custody_id);
        assert!(
            transitions
                .iter()
                .any(|transition| transition.reason == "partition_still_active"),
            "partition retries should be auditable"
        );
        assert_eq!(
            transitions.last().map(|transition| transition.to_state),
            Some(CustodyState::Delivered)
        );
    }

    {
        let outbox_backend = Arc::new(SledStorage::new(&outbox_path).unwrap());
        let outbox = Outbox::persistent(outbox_backend);
        assert!(outbox.peek_for_peer(recipient_id).is_empty());

        let custody_backend = Arc::new(SledStorage::new(&custody_path).unwrap());
        let custody = RelayCustodyStore::persistent(custody_backend);
        assert!(custody.pending_for_destination(recipient_id, 10).is_empty());
    }
}

#[test]
fn partition_recency_recovery_prefers_fresh_relays_deterministically() {
    let mut delivery = MultiPathDelivery::new();
    let now = now_secs();
    let target = PeerId::random();
    let relay_a = PeerId::random();
    let relay_b = PeerId::random();

    delivery.add_relay(relay_a);
    delivery.add_relay(relay_b);
    delivery.record_success("seed-a", vec![relay_a, target], 120);
    delivery.record_success("seed-b", vec![relay_b, target], 120);

    // During partition, relay A has fresher recipient visibility.
    delivery.record_recipient_seen_via_relay(relay_a, target, now.saturating_sub(10));
    delivery.record_recipient_seen_via_relay(relay_b, target, now.saturating_sub(20));
    let during_partition = delivery.ranked_routes(&target, 3);
    let during_partition_repeat = delivery.ranked_routes(&target, 3);

    assert_eq!(during_partition[0].path, vec![target]);
    assert_eq!(during_partition[0].reason_code, ROUTE_REASON_DIRECT_FIRST);
    assert_eq!(during_partition[1].path[0], relay_a);
    assert_eq!(
        during_partition[1].reason_code,
        ROUTE_REASON_RELAY_RECENCY_SUCCESS
    );
    assert_eq!(
        during_partition[1].path, during_partition_repeat[1].path,
        "ordering must remain deterministic across repeated reads"
    );

    // After partition heal, fresher signal via relay B should deterministically win.
    delivery.record_recipient_seen_via_relay(relay_b, target, now);
    let after_heal = delivery.ranked_routes(&target, 3);
    let after_heal_repeat = delivery.ranked_routes(&target, 3);
    assert_eq!(after_heal[0].path, vec![target]);
    assert_eq!(after_heal[1].path[0], relay_b);
    assert_eq!(
        after_heal[1].path, after_heal_repeat[1].path,
        "post-heal ordering must remain deterministic"
    );
}
