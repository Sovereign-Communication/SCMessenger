//! TRN-04 behavioural verification: the full custody lifecycle, driven through
//! the public store API against a REAL sled backend, including the steps that
//! are easiest to get wrong.
//!
//! Two things this test exists to pin down, both of which a unit test using
//! `in_memory()` and the private `put_message` cannot:
//!
//!   1. `mark_delivered` records the transition and then REMOVES the record.
//!      Custody ends at handover, so a delivered message leaves no row behind,
//!      and the delivery trail is the transition log rather than the message.
//!      (Discovering this is what this test did: the first version asserted the
//!      delivered row survived, and it failed. The production path was right
//!      and the assumption was wrong.)
//!   2. Retention is durable. The sweep must survive a store reopen, or
//!      retention is an in-memory illusion that resets when the node restarts.

use scmessenger_core::store::backend::{SledStorage, StorageBackend};
use scmessenger_core::store::relay_custody::{
    CustodyRetentionReport, CustodyState, RelayCustodyStore, CUSTODY_DEFAULT_MAX_AGE_MS,
};
use std::sync::Arc;
use tempfile::tempdir;

const SRC: &str = "source-peer";
const DEST: &str = "destination-peer";

fn open(path: &std::path::Path) -> RelayCustodyStore {
    let backend: Arc<dyn StorageBackend> =
        Arc::new(SledStorage::new(path.to_str().unwrap()).expect("open sled store"));
    RelayCustodyStore::persistent(backend)
}

fn accept(store: &RelayCustodyStore, message_id: &str) -> Result<(), String> {
    store
        .accept_custody(
            SRC.to_string(),
            DEST.to_string(),
            message_id.to_string(),
            vec![1, 2, 3, 4],
            None,
            None,
        )
        .map(|_| ())
}

/// States recorded for a custody id, in order.
fn states(store: &RelayCustodyStore, custody_id: &str) -> Vec<CustodyState> {
    store
        .transitions_for_custody(custody_id)
        .into_iter()
        .map(|t| t.to_state)
        .collect()
}

fn custody_id_for(store: &RelayCustodyStore, message_id: &str) -> String {
    store
        .pending_for_destination(DEST, 100)
        .into_iter()
        .find(|r| r.relay_message_id == message_id)
        .unwrap_or_else(|| panic!("{message_id} must be pending"))
        .custody_id
}

#[test]
fn custody_lifecycle_is_correct_and_survives_a_restart() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("custody");

    let delivered_custody_id;
    let abandoned_custody_id;

    // ---- session 1: accept -> dispatch -> deliver -----------------------
    {
        let store = open(&path);

        let accepted = store
            .accept_custody(
                SRC.to_string(),
                DEST.to_string(),
                "msg-delivered".to_string(),
                vec![1, 2, 3, 4],
                None,
                None,
            )
            .expect("custody must be accepted for an offline destination");
        delivered_custody_id = accepted.custody_id.clone();
        assert_eq!(accepted.state, CustodyState::Accepted);
        assert_eq!(
            store.pending_for_destination(DEST, 10).len(),
            1,
            "an accepted record must be pending for its destination"
        );

        // Dispatch: the record leaves the pending set but is not gone.
        store
            .mark_dispatching(DEST, &delivered_custody_id, "verification_dispatch")
            .expect("mark_dispatching must succeed");
        assert_eq!(
            states(&store, &delivered_custody_id),
            vec![CustodyState::Accepted, CustodyState::Dispatching],
            "mark_dispatching must append a Dispatching transition"
        );
        assert!(
            store.pending_for_destination(DEST, 10).is_empty(),
            "a dispatching record must not still be offered as pending"
        );
        assert!(
            store.has_message_for_destination(DEST, "msg-delivered"),
            "a dispatching record must still exist in storage"
        );

        // Deliver: custody ends here, so the record is removed.
        store
            .mark_delivered(DEST, &delivered_custody_id, "delivered_receipt")
            .expect("mark_delivered must succeed");
        assert_eq!(
            states(&store, &delivered_custody_id),
            vec![
                CustodyState::Accepted,
                CustodyState::Dispatching,
                CustodyState::Delivered
            ],
            "the audit trail must record the whole lifecycle in order"
        );
        assert!(
            !store.has_message_for_destination(DEST, "msg-delivered"),
            "mark_delivered must REMOVE the record: custody ends at handover, \
             and the delivery trail is the transition log"
        );
        assert!(store.pending_for_destination(DEST, 10).is_empty());

        // A sweep must have nothing to say about an already-delivered message,
        // even with a window of zero tolerance.
        let report = store
            .purge_expired_custody(1)
            .expect("sweep must not error");
        assert_eq!(
            report.purged_records, 0,
            "nothing may be purged when the only record was delivered (report: {report:?})"
        );

        // ---- the abandoned one ------------------------------------------
        accept(&store, "msg-abandoned").expect("accept");
        abandoned_custody_id = custody_id_for(&store, "msg-abandoned");

        drop(store);
    }

    // ---- session 2: reopen; nothing may have been an in-memory illusion ---
    {
        let store = open(&path);

        assert_eq!(
            states(&store, &delivered_custody_id),
            vec![
                CustodyState::Accepted,
                CustodyState::Dispatching,
                CustodyState::Delivered
            ],
            "the delivery trail must persist across a store reopen"
        );
        assert!(
            !store.has_message_for_destination(DEST, "msg-delivered"),
            "the delivered record must stay removed across a reopen"
        );

        // The abandoned record survived the reopen as pending...
        let pending = store.pending_for_destination(DEST, 10);
        assert_eq!(pending.len(), 1, "the abandoned record must be pending");
        assert_eq!(pending[0].relay_message_id, "msg-abandoned");

        // ...and now expiry runs against a record that really was persisted.
        std::thread::sleep(std::time::Duration::from_millis(5));
        let report: CustodyRetentionReport = store
            .purge_expired_custody(1)
            .expect("sweep must not error");

        assert_eq!(
            report.expired, 1,
            "exactly the abandoned record must expire (report: {report:?})"
        );
        assert_eq!(report.purged_records, 1);
        assert!(
            report.purged_bytes > 0,
            "reclaimed bytes must be reported (got {})",
            report.purged_bytes
        );

        // The drop is attributable, not silent.
        assert_eq!(
            states(&store, &abandoned_custody_id),
            vec![CustodyState::Accepted, CustodyState::Expired],
            "expiry must transition from the state the record was actually in"
        );
        assert!(
            store
                .transitions_for_custody(&abandoned_custody_id)
                .iter()
                .any(|t| t.reason == "custody_expired"),
            "the expiry transition must carry the custody_expired reason"
        );
        assert!(store.pending_for_destination(DEST, 10).is_empty());

        // The default 7-day window must leave a fresh record alone.
        accept(&store, "msg-fresh").expect("accept");
        let default_report = store
            .purge_expired_custody(CUSTODY_DEFAULT_MAX_AGE_MS)
            .expect("sweep");
        assert_eq!(
            default_report.purged_records, 0,
            "a freshly accepted record must not be expired by the default window"
        );
        assert_eq!(store.pending_for_destination(DEST, 10).len(), 1);

        drop(store);
    }

    // ---- session 3: the purge really was durable --------------------------
    {
        let store = open(&path);
        let pending = store.pending_for_destination(DEST, 10);
        assert_eq!(pending.len(), 1, "only the fresh record should remain");
        assert_eq!(pending[0].relay_message_id, "msg-fresh");
        assert!(
            !store.has_message_for_destination(DEST, "msg-abandoned"),
            "an expired record must stay gone after a reopen"
        );
        assert_eq!(
            states(&store, &abandoned_custody_id),
            vec![CustodyState::Accepted, CustodyState::Expired],
            "the expiry transition must persist too, so the drop stays attributable"
        );
    }
}

#[test]
fn custody_ingestion_guards_reject_on_a_real_persistent_store() {
    let dir = tempdir().expect("temp dir");
    let store = open(&dir.path().join("custody"));

    // Unbounded destination: previously stored and indexed at any length.
    assert!(
        store
            .accept_custody(
                SRC.to_string(),
                "d".repeat(129),
                "msg-1".to_string(),
                vec![1],
                None,
                None
            )
            .is_err(),
        "an oversized destination must be refused"
    );

    // The key separator: without this, destination "a_b" aliases destination
    // "a" onto the same storage key AND into the same prefix scan.
    assert!(
        store
            .accept_custody(
                SRC.to_string(),
                "a_b".to_string(),
                "msg-2".to_string(),
                vec![1],
                None,
                None
            )
            .is_err(),
        "a destination containing the key separator must be refused"
    );

    // Self-relay.
    assert!(
        store
            .accept_custody(
                DEST.to_string(),
                DEST.to_string(),
                "msg-3".to_string(),
                vec![1],
                None,
                None
            )
            .is_err(),
        "a custody hop from X to X must be refused"
    );

    // Control characters would let a caller forge audit/log structure.
    assert!(
        store
            .accept_custody(
                "source\npeer".to_string(),
                DEST.to_string(),
                "msg-4".to_string(),
                vec![1],
                None,
                None
            )
            .is_err(),
        "a control character in an identifier must be refused"
    );

    // And the normal path still works on the same store afterwards, so the
    // guards are not simply refusing everything.
    accept(&store, "msg-ok").expect("a well-formed identifier must still be accepted");
    assert_eq!(store.pending_for_destination(DEST, 10).len(), 1);

    // Dedup must still collapse a repeat of the same (destination, message id).
    accept(&store, "msg-ok").expect("repeat accept must not error");
    assert_eq!(
        store.pending_for_destination(DEST, 10).len(),
        1,
        "re-accepting the same message id must not create a second record"
    );

    // A refused ingestion must not have left anything behind.
    assert!(
        store.has_message_for_destination("a_b", "msg-2") == false,
        "a refused destination must not appear in the store"
    );
}
