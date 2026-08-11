//! Deterministic acceptance fixtures for the CLI mesh path.
//!
//! These tests deliberately do not inject contacts, call `record_connection`,
//! or manually dial a peer. They model the data produced by ledger exchange
//! and assert the invariants the live five-node acceptance run must observe:
//! every node learns the other four nodes, the local node is never a dial
//! target, discovered peers receive a usable CLI route, and queued messages
//! are released once per discovered peer.
//!
//! The live mDNS/swarm five-node test remains a separate network-gated run;
//! these tests keep CI deterministic while making its expected assertions
//! executable.

use libp2p::{identity::Keypair, PeerId};
use scmessenger_cli::ledger::ConnectionLedger;
use scmessenger_cli::transport_bridge::TransportBridge;
use scmessenger_core::store::outbox::MessageState;
use scmessenger_core::store::{Outbox, QueuedMessage};
use scmessenger_core::transport::abstraction::TransportType;
use scmessenger_core::transport::SharedPeerEntry;
use std::collections::HashSet;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

const NODE_COUNT: usize = 5;

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
        let mut ledger = ConnectionLedger::default();

        // This is the ledger-exchange input. No bootstrap, contact, or
        // locally verified connection is inserted first.
        assert_eq!(ledger.merge_shared_entries(&wire_entries), NODE_COUNT);

        let dial_candidates = ledger.dialable_addresses(Some(&local.peer_id.to_string()));
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
                .dialable_addresses(Some(&local.peer_id.to_string()))
                .len(),
            NODE_COUNT - 1
        );
    }
}

#[test]
fn five_node_shared_candidates_survive_cli_ledger_restart_without_contact_injection() {
    let nodes = fixture_nodes();
    let local = &nodes[0];
    let mut ledger = ConnectionLedger::default();
    assert_eq!(
        ledger.merge_shared_entries(&shared_entries(&nodes)),
        NODE_COUNT
    );

    let data_dir = tempfile::tempdir().expect("temporary CLI ledger directory");
    ledger.save(data_dir.path()).expect("save shared ledger");

    let restored = ConnectionLedger::load(data_dir.path()).expect("reload shared ledger");
    let candidates = restored.dialable_addresses(Some(&local.peer_id.to_string()));

    assert_eq!(
        candidates.len(),
        NODE_COUNT - 1,
        "restarting the CLI lost shared mesh candidates: {candidates:?}"
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
    let mut ledger = ConnectionLedger::default();
    assert_eq!(
        ledger.merge_shared_entries(&shared_entries(&nodes)),
        NODE_COUNT
    );

    let candidates = ledger.dialable_addresses(Some(&local.peer_id.to_string()));
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
