//! Deterministic acceptance fixtures for the CLI mesh path.
//!
//! These tests deliberately do not inject contacts, bootstrap nodes, or manual
//! connections. They model the data produced by ledger exchange and assert the
//! invariants the live five-node acceptance run must observe: every node learns
//! the other four nodes, the local node is never a dial target, discovered
//! peers receive a usable CLI route, and queued messages are released once per
//! discovered peer.
//!
//! T2 UNIFICATION (2026-08-31): ledger exchange is HEARSAY. It is recorded for
//! routing but is not a persistent dial candidate until a connection proves it
//! (the core store fires `record_connection` on outbound
//! `ConnectionEstablished` -- exactly what a successful first dial of a
//! wire-learned address does). The tests model that wire path: exchange ->
//! dial -> proven -> candidate.
//!
//! The live mDNS/swarm five-node test remains a separate network-gated run;
//! these tests keep CI deterministic while making its expected assertions
//! executable.

use libp2p::{identity::Keypair, PeerId};
use scmessenger_cli::ledger::ConnectionLedger;
use scmessenger_cli::transport_bridge::TransportBridge;
use scmessenger_core::store::outbox::MessageState;
use scmessenger_core::store::{LedgerManager, Outbox, QueuedMessage};
use scmessenger_core::transport::abstraction::TransportType;
use scmessenger_core::transport::SharedPeerEntry;
use std::collections::HashSet;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

const NODE_COUNT: usize = 5;

fn ledger() -> ConnectionLedger {
    ConnectionLedger::new(LedgerManager::ephemeral())
}

#[derive(Debug, Clone)]
struct FixtureNode {
    peer_id: PeerId,
    multiaddr: String,
}

fn fixture_nodes() -> Vec<FixtureNode> {
    (0..NODE_COUNT)
        .map(|index| FixtureNode {
            peer_id: Keypair::generate_ed25519().public().to_peer_id(),
            multiaddr: format!("/ip4/192.168.42.{}/tcp/{}", 10 + index, 9100 + index),
        })
        .collect()
}

fn shared_entries(nodes: &[FixtureNode]) -> Vec<SharedPeerEntry> {
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| SharedPeerEntry {
            multiaddr: node.multiaddr.clone(),
            last_peer_id: Some(node.peer_id.to_string()),
            last_seen: 1_700_000_000 + index as u64,
            known_topics: vec!["sc-mesh".to_string()],
        })
        .collect()
}

/// The node's own LAN address family: the private-class rule only dials
/// RFC1918 candidates on the same class as an address this node itself holds.
fn my_addrs() -> Vec<String> {
    vec!["/ip4/192.168.42.1/tcp/9100".to_string()]
}

/// The wire path a successful first dial of a wire-learned address takes:
/// the scheduler dials, the connection establishes, and the core store marks
/// the address locally verified. Mirrors `ConnectionEstablished` in the
/// swarm event loop.
fn prove_connection(ledger: &ConnectionLedger, node: &FixtureNode) {
    ledger.record_connection(&node.multiaddr, &node.peer_id.to_string());
}

fn queued_message(message_id: &str, recipient_id: &str) -> QueuedMessage {
    QueuedMessage {
        version: 1,
        message_id: message_id.to_string(),
        recipient_id: recipient_id.to_string(),
        envelope_data: vec![1, 2, 3, 4],
        queued_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        attempts: 0,
        next_retry_at: None,
        in_custody: false,
        custody_established_at: 0,
        state: MessageState::Enqueued,
    }
}

#[test]
fn five_node_no_manual_contact_ledger_converges_to_four_dial_candidates() {
    let nodes = fixture_nodes();
    let wire_entries = shared_entries(&nodes);
    let all_peer_ids: HashSet<PeerId> = nodes.iter().map(|node| node.peer_id).collect();

    for local in &nodes {
        let mut ledger = ledger();

        // This is the ledger-exchange input. No bootstrap, contact, or
        // locally verified connection is inserted first. Exchange is
        // HEARSAY: recorded for routing but not yet a dial candidate.
        assert_eq!(ledger.merge_shared_entries(&wire_entries), NODE_COUNT);
        assert!(
            ledger
                .dialable_addresses(Some(&local.peer_id.to_string()), &my_addrs())
                .is_empty(),
            "unproven exchange knowledge reached dialable_addresses()"
        );

        // The wire path: each node's exchange-learned addresses get dialed
        // (LedgerReceived handler); the five successful connections -- the
        // mesh's own traffic -- prove the entries.
        for node in &nodes {
            prove_connection(&ledger, node);
        }

        let dial_candidates =
            ledger.dialable_addresses(Some(&local.peer_id.to_string()), &my_addrs());
        assert_eq!(
            dial_candidates.len(),
            NODE_COUNT - 1,
            "node {} did not learn exactly the other four nodes: {dial_candidates:?}",
            local.peer_id
        );

        let candidate_ids: HashSet<PeerId> = dial_candidates
            .iter()
            .map(|(_, peer_id)| {
                PeerId::from_str(
                    peer_id
                        .as_deref()
                        .expect("every shared candidate must carry a peer id"),
                )
                .expect("shared peer id must be a valid libp2p PeerId")
            })
            .collect();

        assert_eq!(candidate_ids.len(), NODE_COUNT - 1);
        assert!(
            !candidate_ids.contains(&local.peer_id),
            "local node became a dial target"
        );
        assert_eq!(candidate_ids.len() + 1, all_peer_ids.len());
        assert!(candidate_ids.is_subset(&all_peer_ids));

        // Replaying a ledger response must be idempotent rather than creating
        // duplicate address records or additional dial targets.
        assert_eq!(ledger.merge_shared_entries(&wire_entries), 0);
        assert_eq!(
            ledger
                .dialable_addresses(Some(&local.peer_id.to_string()), &my_addrs())
                .len(),
            NODE_COUNT - 1
        );
    }
}

#[test]
fn five_node_shared_candidates_survive_core_ledger_reload_without_contact_injection() {
    let nodes = fixture_nodes();
    let local = &nodes[0];
    let dir = tempfile::tempdir().expect("temporary core ledger directory");
    let storage = dir.path().join("storage");

    // Handle A: the node that merged the exchange. The core store persists
    // on every mutation, so ledger.json lands on disk immediately.
    let mut ledger =
        ConnectionLedger::new(LedgerManager::new(storage.to_string_lossy().to_string()));
    assert_eq!(
        ledger.merge_shared_entries(&shared_entries(&nodes)),
        NODE_COUNT
    );

    // Durability: a restarting process will load this file.
    let ledger_file = storage.join("ledger.json");
    assert!(
        ledger_file.exists(),
        "core ledger did not persist ledger.json"
    );
    let persisted = std::fs::read_to_string(&ledger_file).expect("read persisted ledger");
    for node in &nodes {
        assert!(
            persisted.contains(&node.multiaddr),
            "persisted ledger lost {}",
            node.multiaddr
        );
    }

    // The wire path proves the four other nodes.
    for node in &nodes {
        if node.peer_id != local.peer_id {
            prove_connection(&ledger, node);
        }
    }

    // Handle B: a fresh session over the same storage path (the in-process
    // mirror of a restart) sees the surviving shared candidates.
    let restored = ConnectionLedger::new(LedgerManager::new(storage.to_string_lossy().to_string()));
    let candidates = restored.dialable_addresses(Some(&local.peer_id.to_string()), &my_addrs());

    assert_eq!(
        candidates.len(),
        NODE_COUNT - 1,
        "reloading the shared ledger lost mesh candidates: {candidates:?}"
    );
    assert!(candidates.iter().all(|(_, peer_id)| peer_id.is_some()));
    assert!(candidates
        .iter()
        .all(|(_, peer_id)| peer_id.as_deref() != Some(local.peer_id.to_string().as_str())));
    assert_eq!(restored.all_known_topics(), vec!["sc-mesh".to_string()]);
}

#[test]
fn five_node_discovery_registers_routes_and_flushes_each_queued_message_once() {
    let nodes = fixture_nodes();
    let local = &nodes[0];
    let mut ledger = ledger();
    assert_eq!(
        ledger.merge_shared_entries(&shared_entries(&nodes)),
        NODE_COUNT
    );
    for node in &nodes {
        prove_connection(&ledger, node);
    }

    let candidates = ledger.dialable_addresses(Some(&local.peer_id.to_string()), &my_addrs());
    assert_eq!(candidates.len(), NODE_COUNT - 1);
    let mut bridge = TransportBridge::new();
    let mut outbox = Outbox::new();

    for (index, (_, peer_id)) in candidates.iter().enumerate() {
        let peer_id = PeerId::from_str(
            peer_id
                .as_deref()
                .expect("discovered route must have a peer id"),
        )
        .expect("discovered route must have a valid peer id");

        // Mirrors the CLI PeerDiscovered handler's capability registration;
        // the source of the peer is still the shared ledger, not manual input.
        bridge.register_peer(peer_id, vec![TransportType::Internet, TransportType::Local]);
        assert!(bridge.can_reach_destination(&peer_id));
        assert!(bridge.find_best_path(&peer_id).is_some());

        outbox
            .enqueue(queued_message(
                &format!("mesh-{index}"),
                &peer_id.to_string(),
            ))
            .expect("enqueue message for discovered peer");
    }

    assert_eq!(outbox.pending().len(), NODE_COUNT - 1);

    for (_, peer_id) in candidates {
        let peer_id = peer_id.expect("candidate peer id");
        let flushed = outbox.flush_peer_messages(&peer_id);
        assert_eq!(
            flushed.len(),
            1,
            "peer {peer_id} did not receive exactly one outbox flush"
        );
        assert_eq!(flushed[0].recipient_id, peer_id);
    }

    assert!(outbox.pending().is_empty());
    assert_eq!(outbox.total_count(), 0);
}
