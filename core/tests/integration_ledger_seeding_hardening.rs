// Regression tests for the ledger-seeding adversarial review of 2026-07-25
// (HANDOFF/review/LEDGER_SEEDING_ADVERSARIAL_REVIEW_2026-07-25.md).
//
// Covers, end to end rather than at the unit boundary:
//   F11 -- IronCore is the ledger owner: the constructor hydrates it from disk,
//          and a real connection populates `dialable_addresses()` with nobody
//          calling `record_connection` by hand. The pre-existing
//          `integration_ledger_convergence.rs` only passed because its test
//          body seeded the ledger itself, so it proved nothing about
//          production.
//   F6  -- the `/sc/ledger-exchange/1.0.0` RESPONSE is capped, address-filtered
//          and carries no `known_topics`.
//   F3  -- an SSRF/internal address sitting in our ledger is never disclosed to
//          a peer and never becomes a dial candidate.
//
// Networked cases are #[ignore] by default, matching the rest of the suite:
//   cargo test -p scmessenger-core --test integration_ledger_seeding_hardening \
//       -- --include-ignored

use libp2p::identity::Keypair;
use libp2p::Multiaddr;
use scmessenger_core::store::ledger_entry::{LedgerEntry, SharedPeerEntry};
use scmessenger_core::transport::swarm::{start_swarm, SwarmEvent2, SwarmHandle};
use scmessenger_core::IronCore;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::mpsc;

/// F11, disk half: `IronCore`'s persistent constructors must call
/// `LedgerManager::load()`.
///
/// Nothing in `core/src` ever did, so every restart began with an empty ledger:
/// `success_count` was always 0, `dialable_addresses()` was permanently empty,
/// and both the seed-dial proven tier and the ledger-exchange response shipped
/// nothing. Needs no networking.
#[test]
fn iron_core_constructor_hydrates_the_ledger_from_disk() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().to_string_lossy().to_string();

    {
        let core = IronCore::with_storage(path.clone());
        core.ledger_manager.record_connection(
            "/ip4/198.51.100.42/tcp/9001".to_string(),
            libp2p::PeerId::random().to_string(),
        );
        assert_eq!(core.ledger_manager.dialable_addresses().len(), 1);
    }

    let restarted = IronCore::with_storage(path);
    let dialable = restarted.ledger_manager.dialable_addresses();
    assert_eq!(
        dialable.len(),
        1,
        "IronCore did not load the persisted ledger; every restart starts blind"
    );
    assert_eq!(dialable[0].multiaddr, "/ip4/198.51.100.42/tcp/9001");
    assert!(dialable[0].success_count > 0);
}

/// F11, world-readable-temp-dir half: the storage-less constructor must not
/// write peer topology into `std::env::temp_dir()`.
#[test]
fn in_memory_core_has_no_on_disk_ledger() {
    let core = IronCore::new();
    let temp_ledger = std::env::temp_dir().join("ledger.json");
    let before = std::fs::metadata(&temp_ledger).ok().map(|m| m.len());

    core.ledger_manager.record_connection(
        "/ip4/198.51.100.77/tcp/9001".to_string(),
        libp2p::PeerId::random().to_string(),
    );

    assert_eq!(core.ledger_manager.dialable_addresses().len(), 1);
    let after = std::fs::metadata(&temp_ledger).ok().map(|m| m.len());
    assert_eq!(
        before, after,
        "in-memory IronCore wrote its ledger into the shared temp directory"
    );
}

/// F11, production-path half: after a real dial, the dialer's ledger must show
/// a proven entry, with the test body never calling `record_connection`.
///
/// Before the fix `record_connection` had ZERO callers in `core/src`, so this
/// assertion was unsatisfiable in production no matter how long you waited.
#[tokio::test]
#[ignore = "requires real networking; run with --include-ignored"]
async fn dialing_a_peer_populates_the_dialer_ledger_without_manual_seeding() {
    let dir1 = TempDir::new().expect("tempdir 1");
    let dir2 = TempDir::new().expect("tempdir 2");
    let core1 = Arc::new(IronCore::with_storage(
        dir1.path().to_string_lossy().to_string(),
    ));
    let core2 = Arc::new(IronCore::with_storage(
        dir2.path().to_string_lossy().to_string(),
    ));

    assert!(
        core2.ledger_manager.dialable_addresses().is_empty(),
        "precondition: node 2 starts with a cold ledger"
    );

    let keypair1 = Keypair::generate_ed25519();
    let peer_id1 = libp2p::PeerId::from(keypair1.public());
    let keypair2 = Keypair::generate_ed25519();

    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut event_rx2) = mpsc::channel(256);

    let swarm1: SwarmHandle = start_swarm(
        keypair1,
        None,
        event_tx1,
        Some(Arc::downgrade(&core1)),
        false,
        None,
        scmessenger_core::transport::default_routing_engine_handle(),
    )
    .await
    .expect("Failed to start swarm1");

    let node1_addr = first_loopback_tcp(&mut event_rx1).await;

    let swarm2: SwarmHandle = start_swarm(
        keypair2,
        None,
        event_tx2,
        Some(Arc::downgrade(&core2)),
        false,
        None,
        scmessenger_core::transport::default_routing_engine_handle(),
    )
    .await
    .expect("Failed to start swarm2");

    // Keep both event channels drained so the bounded mpsc never backpressures
    // the swarm task.
    tokio::spawn(async move { while event_rx1.recv().await.is_some() {} });
    tokio::spawn(async move { while event_rx2.recv().await.is_some() {} });

    tokio::time::sleep(Duration::from_millis(500)).await;

    let mut dial_addr = node1_addr.clone();
    dial_addr.push(libp2p::multiaddr::Protocol::P2p(peer_id1));
    dial_or_already_connected(&swarm2, dial_addr).await;

    // Poll rather than sleep-and-hope.
    let mut dialable = Vec::new();
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_millis(250)).await;
        dialable = core2.ledger_manager.dialable_addresses();
        if !dialable.is_empty() {
            break;
        }
    }

    assert!(
        !dialable.is_empty(),
        "node 2 dialed node 1 successfully but its ledger stayed empty -- \
         nothing in production calls record_connection"
    );
    assert!(
        dialable
            .iter()
            .any(|e| e.peer_id.as_deref() == Some(&peer_id1.to_string())),
        "the proven entry does not carry the peer id we actually reached: {:?}",
        dialable
    );
    assert!(
        dialable.iter().all(|e| !e.multiaddr.contains("/p2p/")),
        "ledger keys must be peer-id-stripped: {:?}",
        dialable
    );

    let _ = swarm1.shutdown().await;
    let _ = swarm2.shutdown().await;
}

/// F6 + F3: the unauthenticated ledger-exchange RESPONSE must be capped, must
/// not carry `known_topics`, and must not disclose non-routable addresses.
#[tokio::test]
#[ignore = "requires real networking; run with --include-ignored"]
async fn ledger_exchange_response_is_capped_topic_free_and_address_filtered() {
    // Re-review NEW-2: the previous version of this list contained only
    // loopback and link-local, which is exactly why the RFC1918 disclosure bug
    // passed it. `record_connection` is deliberately unfiltered, so every LAN
    // peer we have ever dialed is a proven, disclosable entry, and the reply
    // path used to run in `NetworkMode::Local` -- the mode that skips the
    // private-range check.
    const HOSTILE: &[&str] = &[
        "/ip4/169.254.169.254/tcp/80",
        "/ip4/127.0.0.1/tcp/8080",
        "/ip6/::1/tcp/8080",
        "/ip4/192.168.7.7/tcp/9001",
        "/ip4/10.13.37.2/tcp/9001",
        "/ip4/172.20.1.1/tcp/9001",
        "/ip6/fd00::1/tcp/9001",
        "/dns4/nas.corp.internal/tcp/443",
    ];

    let dir1 = TempDir::new().expect("tempdir 1");
    let dir2 = TempDir::new().expect("tempdir 2");

    // Seed node 1's ledger on disk so the topics are populated (nothing in core
    // writes `LedgerEntry::topics` today, and an empty-topics assertion against
    // an always-empty field would be vacuous). Constructing IronCore
    // afterwards also exercises the F11 `load()` wiring.
    let mut seeded: Vec<LedgerEntry> = Vec::new();
    // Proven, but nothing a stranger should ever hear about. FIRST in the file,
    // deliberately: `exchange_response_entries` filters and then takes 64, so
    // putting these after 100 routable peers would let the cap hide a filter
    // that does not work -- the needles below would pass vacuously.
    for addr in HOSTILE {
        seeded.push(LedgerEntry {
            multiaddr: addr.to_string(),
            peer_id: Some(libp2p::PeerId::random().to_string()),
            public_key: None,
            nickname: None,
            success_count: 3,
            failure_count: 0,
            last_seen: Some(1_700_000_000_000),
            topics: vec!["sc-family-chat".to_string()],
            locally_verified: true,
            is_bootstrap: false,
            first_seen: Some(1_700_000_000_000),
            observed_peer_ids: Vec::new(),
            label: None,
        });
    }
    // 100 proven, routable peers -- more than the 64 response cap.
    for i in 0..100u32 {
        seeded.push(LedgerEntry {
            multiaddr: format!("/ip4/198.51.{}.{}/tcp/9001", i / 256, i % 256),
            peer_id: Some(libp2p::PeerId::random().to_string()),
            public_key: None,
            nickname: None,
            success_count: 3,
            failure_count: 0,
            last_seen: Some(1_700_000_000_000),
            topics: vec!["sc-family-chat".to_string(), "sc-activists".to_string()],
            locally_verified: true,
            is_bootstrap: false,
            first_seen: Some(1_700_000_000_000),
            observed_peer_ids: Vec::new(),
            label: None,
        });
    }
    std::fs::create_dir_all(dir1.path()).expect("create ledger dir");
    std::fs::write(
        dir1.path().join("ledger.json"),
        serde_json::to_string_pretty(&seeded).expect("serialize seeded ledger"),
    )
    .expect("write seeded ledger");

    let core1 = Arc::new(IronCore::with_storage(
        dir1.path().to_string_lossy().to_string(),
    ));
    let core2 = Arc::new(IronCore::with_storage(
        dir2.path().to_string_lossy().to_string(),
    ));
    assert_eq!(
        core1.ledger_manager.dialable_addresses().len(),
        seeded.len(),
        "IronCore did not hydrate the seeded ledger"
    );

    let keypair1 = Keypair::generate_ed25519();
    let peer_id1 = libp2p::PeerId::from(keypair1.public());
    let keypair2 = Keypair::generate_ed25519();

    let (event_tx1, mut event_rx1) = mpsc::channel(256);
    let (event_tx2, mut event_rx2) = mpsc::channel(256);

    let swarm1: SwarmHandle = start_swarm(
        keypair1,
        None,
        event_tx1,
        Some(Arc::downgrade(&core1)),
        false,
        None,
        scmessenger_core::transport::default_routing_engine_handle(),
    )
    .await
    .expect("Failed to start swarm1");

    let node1_addr = first_loopback_tcp(&mut event_rx1).await;
    tokio::spawn(async move { while event_rx1.recv().await.is_some() {} });

    let swarm2: SwarmHandle = start_swarm(
        keypair2,
        None,
        event_tx2,
        Some(Arc::downgrade(&core2)),
        false,
        None,
        scmessenger_core::transport::default_routing_engine_handle(),
    )
    .await
    .expect("Failed to start swarm2");

    let responses: Arc<parking_lot::Mutex<Vec<Vec<SharedPeerEntry>>>> =
        Arc::new(parking_lot::Mutex::new(Vec::new()));
    let responses_task = responses.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx2.recv().await {
            if let SwarmEvent2::LedgerReceived { entries, .. } = event {
                responses_task.lock().push(entries);
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(500)).await;
    let mut dial_addr = node1_addr.clone();
    dial_addr.push(libp2p::multiaddr::Protocol::P2p(peer_id1));
    dial_or_already_connected(&swarm2, dial_addr).await;
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Node 2 initiates; node 1's application layer never calls share_ledger.
    swarm2
        .share_ledger(peer_id1)
        .await
        .expect("Failed to share ledger");

    let mut received: Vec<SharedPeerEntry> = Vec::new();
    for _ in 0..40 {
        tokio::time::sleep(Duration::from_millis(250)).await;
        received = responses.lock().iter().flatten().cloned().collect();
        if !received.is_empty() {
            break;
        }
    }

    assert!(
        !received.is_empty(),
        "node 2 received no reciprocal ledger at all"
    );
    assert!(
        received.len() <= 64,
        "response exceeded the 64-record cap: {}",
        received.len()
    );
    assert!(
        received.iter().all(|e| e.known_topics.is_empty()),
        "known_topics leaked group membership to an unauthenticated peer"
    );
    for needle in [
        "127.0.0.1",
        "169.254.169.254",
        "/ip6/::1/",
        // NEW-2 needles. Each of these is a live internal host:port plus, via
        // `last_peer_id`, the identity of the neighbour listening on it.
        "192.168.",
        "/ip4/10.",
        "172.20.",
        "fd00::",
        "corp.internal",
    ] {
        assert!(
            !received.iter().any(|e| e.multiaddr.contains(needle)),
            "non-disclosable address containing {} was disclosed over the wire: {:?}",
            needle,
            received
                .iter()
                .map(|e| e.multiaddr.as_str())
                .collect::<Vec<_>>()
        );
    }

    let _ = swarm1.shutdown().await;
    let _ = swarm2.shutdown().await;
}

/// NEW-1, end to end through `IronCore`: a DNS-form address supplied by a
/// remote must not survive anywhere on the remote-supplied path.
///
/// The attack the finding describes: publish `A evil.example -> 169.254.169.254`
/// (or any internal host), put `/dns4/evil.example/tcp/80` in a ledger-exchange
/// entry or an invite `seed_ledger`, and every IPv4/IPv6 rule is skipped because
/// the old filter set `has_transport = true` for DNS and validated nothing. The
/// desktop swarm wires a real resolver, so it resolves and dials -- and the zone
/// can be re-pointed between probes, which turns one entry into a scanner.
#[test]
fn dns_addresses_from_remote_peers_are_refused_everywhere() {
    let dir = TempDir::new().expect("tempdir");
    let core = IronCore::with_storage(dir.path().to_string_lossy().to_string());

    let hostile: Vec<scmessenger_core::store::ledger_entry::SeedLedgerEntry> = [
        "/dns4/evil.example/tcp/80",
        "/dns6/evil.example/tcp/80",
        "/dns/evil.example/tcp/80",
        "/dnsaddr/evil.example",
        "/dns4/evil.example/tcp/443/p2p-circuit",
    ]
    .iter()
    .map(|a| scmessenger_core::store::ledger_entry::SeedLedgerEntry {
        multiaddr: a.to_string(),
    })
    .collect();

    assert_eq!(
        core.ledger_manager.import_seed_entries(hostile),
        0,
        "an invite seeded DNS names into the ledger"
    );
    assert!(
        core.ledger_manager.seed_addresses(64).is_empty(),
        "a DNS name became a seed-dial candidate"
    );
}

/// F3/NEW-1 at the INGESTION choke point, through `IronCore`.
///
/// `dns_addresses_from_remote_peers_are_refused_everywhere` above covers the
/// invite path. This covers the other writer, and the one that produces PROVEN
/// entries: `record_connection`. The gate used to live in the callers, which is
/// how `cli/src/main.rs:2034` ended up without it while `:2996` had it.
#[test]
fn dns_addresses_cannot_be_recorded_as_proven_connections() {
    let dir = TempDir::new().expect("tempdir");
    let core = IronCore::with_storage(dir.path().to_string_lossy().to_string());

    for addr in [
        "/dns4/evil.example/tcp/80",
        "/dns6/evil.example/tcp/80",
        "/dns/evil.example/tcp/80",
        "/dnsaddr/evil.example",
        "/dns4/evil.example/tcp/443/p2p-circuit",
    ] {
        core.ledger_manager
            .record_connection(addr.to_string(), libp2p::PeerId::random().to_string());
        core.ledger_manager.annotate_identity(
            addr.to_string(),
            libp2p::PeerId::random().to_string(),
            None,
            None,
        );
    }

    assert!(
        core.ledger_manager.dialable_addresses().is_empty(),
        "a DNS name became a PROVEN ledger entry: {:?}",
        core.ledger_manager
            .dialable_addresses()
            .iter()
            .map(|e| e.multiaddr.as_str())
            .collect::<Vec<_>>()
    );
    assert!(core.ledger_manager.seed_addresses(64).is_empty());
    assert!(core.ledger_manager.get_preferred_relays(64).is_empty());
    assert!(core.ledger_manager.export_seed_entries(64).is_empty());
    assert!(core
        .ledger_manager
        .exchange_response_entries(64, "stranger", &[])
        .is_empty());

    // A real address still works, so this is a gate and not an outage.
    core.ledger_manager.record_connection(
        "/ip4/198.51.100.4/tcp/9001".to_string(),
        libp2p::PeerId::random().to_string(),
    );
    assert_eq!(core.ledger_manager.dialable_addresses().len(), 1);
}

/// NEW-2, the REQUEST half. `SwarmCommand::ShareLedger` no longer carries a
/// payload; both directions of `/sc/ledger-exchange/1.0.0` build from
/// `exchange_response_entries`. This asserts the property that makes the two
/// doors equivalent: whatever the node would SEND is exactly what it would
/// REPLY, filtered identically, for the same requester.
///
/// The CLI's deleted `to_shared_entries()` had none of these filters -- no cap,
/// no `success_count > 0`, no address filter, and it copied `known_topics`
/// verbatim -- and it fired on every peer connection from three sites.
#[test]
fn the_exchange_request_and_response_payloads_are_the_same_function() {
    let dir = TempDir::new().expect("tempdir");
    let core = IronCore::with_storage(dir.path().to_string_lossy().to_string());

    // A ledger with everything the old request path would have disclosed.
    for addr in [
        "/ip4/192.168.1.20/tcp/9001",
        "/ip4/10.0.2.16/tcp/9001",
        "/ip4/100.64.7.7/tcp/9001",
        "/ip4/127.0.0.1/tcp/8080",
        "/ip4/169.254.169.254/tcp/80",
    ] {
        core.ledger_manager
            .record_connection(addr.to_string(), libp2p::PeerId::random().to_string());
    }
    // Unproven, wire-learned entries: never disclosable in either direction.
    core.ledger_manager.import_seed_entries(vec![
        scmessenger_core::store::ledger_entry::SeedLedgerEntry {
            multiaddr: "/ip4/203.0.113.44/tcp/9001".to_string(),
        },
    ]);
    // 100 proven, routable peers, more than the 64 cap.
    for i in 0..100u32 {
        core.ledger_manager.record_connection(
            format!("/ip4/198.51.{}.{}/tcp/9001", i / 256, i % 256),
            libp2p::PeerId::random().to_string(),
        );
    }

    let requester = "12D3KooWSHj3RRbBjD15g6wekV8y3mm57Pobmps2g2WJm6F67Lay";
    let payload = core
        .ledger_manager
        .exchange_response_entries(64, requester, &[]);

    assert_eq!(payload.len(), 64, "the shared cap was not applied");
    assert!(
        payload.iter().all(|e| e.known_topics.is_empty()),
        "known_topics leaked into the exchange payload"
    );
    for needle in [
        "192.168.",
        "/ip4/10.",
        "100.64.",
        "127.0.0.1",
        "169.254.",
        "203.0.113.44",
    ] {
        assert!(
            !payload.iter().any(|e| e.multiaddr.contains(needle)),
            "{needle} would be sent to a peer we merely handshaked with"
        );
    }
}

/// NEW-2, end to end without networking: a node whose entire proven ledger is
/// LAN neighbours must answer a ledger exchange with nothing, and must not bake
/// those neighbours into an invite either (NEW-7).
#[test]
fn lan_only_node_discloses_nothing_to_a_stranger() {
    let dir = TempDir::new().expect("tempdir");
    let core = IronCore::with_storage(dir.path().to_string_lossy().to_string());

    for addr in [
        "/ip4/192.168.1.20/tcp/9001",
        "/ip4/192.168.1.21/tcp/9001",
        "/ip4/10.0.2.16/tcp/9001",
        "/ip4/172.19.4.4/tcp/9001",
        "/ip6/fd00::5/tcp/9001",
        "/ip4/127.0.0.1/tcp/8080",
    ] {
        core.ledger_manager
            .record_connection(addr.to_string(), libp2p::PeerId::random().to_string());
    }
    assert_eq!(core.ledger_manager.dialable_addresses().len(), 6);

    let disclosed = core.ledger_manager.exchange_response_entries(
        64,
        "12D3KooWSHj3RRbBjD15g6wekV8y3mm57Pobmps2g2WJm6F67Lay",
        &[],
    );
    assert!(
        disclosed.is_empty(),
        "internal subnets, live host:ports and neighbour peer ids disclosed to a \
         stranger: {:?}",
        disclosed
            .iter()
            .map(|e| (e.multiaddr.as_str(), e.last_peer_id.as_deref()))
            .collect::<Vec<_>>()
    );

    let invite_seeds = core.ledger_manager.export_seed_entries(64);
    assert!(
        invite_seeds.is_empty(),
        "invite QR carried internal addresses: {:?}",
        invite_seeds
    );
}

/// NEW-5, on the wire: the handler must cap how many entries one message can
/// make it process, before it clones them, forwards them to the app layer and
/// loops each one through the recency map and Kademlia.
///
/// Observed at the receiver's application boundary, which is the only externally
/// visible edge of that loop.
///
/// SPOKEN BY THE HOSTILE PEER, NOT BY A `SwarmHandle` (choke-point refactor
/// 2026-07-26). This used to call `swarm2.share_ledger(peer_id1, flood)` with a
/// 4000-entry payload of its own construction. `SwarmCommand::ShareLedger` no
/// longer accepts a payload -- that was the second, unfiltered disclosure door
/// (re-review NEW-2) -- so an oversized request can only come from something
/// that speaks the protocol directly, which is what an attacker is anyway.
#[tokio::test]
#[ignore = "requires real networking; run with --include-ignored"]
async fn oversized_ledger_exchange_request_is_capped_before_processing() {
    const OFFERED: usize = 4000;
    const CAP: usize = 64;

    let dir1 = TempDir::new().expect("tempdir 1");
    let core1 = Arc::new(IronCore::with_storage(
        dir1.path().to_string_lossy().to_string(),
    ));

    let keypair1 = Keypair::generate_ed25519();
    let peer_id1 = libp2p::PeerId::from(keypair1.public());

    let (event_tx1, mut event_rx1) = mpsc::channel(256);

    let swarm1: SwarmHandle = start_swarm(
        keypair1,
        None,
        event_tx1,
        Some(Arc::downgrade(&core1)),
        false,
        None,
        scmessenger_core::transport::default_routing_engine_handle(),
    )
    .await
    .expect("Failed to start swarm1");

    let node1_addr = first_loopback_tcp(&mut event_rx1).await;

    // Node 1 is the receiver under test: record the size of every batch its
    // application layer is handed.
    let batches: Arc<parking_lot::Mutex<Vec<usize>>> =
        Arc::new(parking_lot::Mutex::new(Vec::new()));
    let batches_task = batches.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx1.recv().await {
            if let SwarmEvent2::LedgerReceived { entries, .. } = event {
                batches_task.lock().push(entries.len());
            }
        }
    });

    let mut dial_addr = node1_addr.clone();
    dial_addr.push(libp2p::multiaddr::Protocol::P2p(peer_id1));
    hostile::flood_ledger_exchange(dial_addr, peer_id1, 1, OFFERED).await;

    let seen = batches.lock().clone();
    assert!(
        !seen.is_empty(),
        "node 1 never received the ledger-exchange request at all"
    );
    assert!(
        seen.iter().all(|n| *n <= CAP),
        "handler processed an uncapped batch: {seen:?} (offered {OFFERED})"
    );

    let _ = swarm1.shutdown().await;
}

/// NEW-5, ordering half: the token bucket must gate the WORK, not just the
/// reply.
///
/// The bucket used to be consulted after the handler had already cloned the
/// request, `await`ed it into the bounded event channel, walked every entry into
/// the recency map and Kademlia, and subscribed gossipsub to every
/// attacker-supplied topic string. Only the disclosure was rate limited.
///
/// This CANNOT be driven by a second `SwarmHandle`: `SwarmCommand::ShareLedger`
/// suppresses repeats via `ledger_exchanged_peers`, so an honest node sends at
/// most one request per peer and any loop over `share_ledger` would pass
/// vacuously. A real attacker is under no such obligation, so the test speaks
/// the protocol directly.
///
/// The application-layer `LedgerReceived` event is the observable proxy for "the
/// handler did the per-request work": with the bucket checked first, an
/// over-quota peer produces no event at all.
#[tokio::test]
#[ignore = "requires real networking; run with --include-ignored"]
async fn over_quota_exchange_requests_do_no_per_entry_work() {
    const REQUESTS: usize = 24;
    // Burst is RELAY_PEER_BUCKET_BURST_CAPACITY (20) * 0.1 = 2 tokens, refilling
    // at 4.0 * 0.1 = 0.4/s. The requests are sent back to back and the drain
    // window below is a few seconds, so a handful of tokens at most.
    const TOLERATED_BATCHES: usize = 6;

    let dir1 = TempDir::new().expect("tempdir 1");
    let core1 = Arc::new(IronCore::with_storage(
        dir1.path().to_string_lossy().to_string(),
    ));

    let keypair1 = Keypair::generate_ed25519();
    let peer_id1 = libp2p::PeerId::from(keypair1.public());
    let (event_tx1, mut event_rx1) = mpsc::channel(256);

    let swarm1: SwarmHandle = start_swarm(
        keypair1,
        None,
        event_tx1,
        Some(Arc::downgrade(&core1)),
        false,
        None,
        scmessenger_core::transport::default_routing_engine_handle(),
    )
    .await
    .expect("Failed to start swarm1");

    let node1_addr = first_loopback_tcp(&mut event_rx1).await;

    let batches: Arc<parking_lot::Mutex<usize>> = Arc::new(parking_lot::Mutex::new(0));
    let batches_task = batches.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx1.recv().await {
            if let SwarmEvent2::LedgerReceived { .. } = event {
                *batches_task.lock() += 1;
            }
        }
    });

    let mut dial_addr = node1_addr.clone();
    dial_addr.push(libp2p::multiaddr::Protocol::P2p(peer_id1));
    hostile::flood_ledger_exchange(dial_addr, peer_id1, REQUESTS, 16).await;

    let observed = *batches.lock();
    assert!(
        observed >= 1,
        "the first exchange from a new peer must still be processed"
    );
    assert!(
        observed <= TOLERATED_BATCHES,
        "{observed} of {REQUESTS} over-quota requests were processed in full; \
         the bucket is gating the reply, not the work"
    );

    let _ = swarm1.shutdown().await;
}

/// A minimal peer that speaks `/sc/ledger-exchange/1.0.0` and nothing else, so
/// tests can exercise the inbound handler the way an attacker would rather than
/// the way `SwarmHandle` politely does.
mod hostile {
    use super::*;
    use libp2p::futures::StreamExt;
    use libp2p::request_response::{self, ProtocolSupport};
    use libp2p::swarm::SwarmEvent as Libp2pSwarmEvent;
    use libp2p::StreamProtocol;
    use scmessenger_core::store::ledger_entry::{LedgerExchangeRequest, LedgerExchangeResponse};

    type Behaviour =
        request_response::cbor::Behaviour<LedgerExchangeRequest, LedgerExchangeResponse>;

    /// Dial `addr`, then fire `requests` back-to-back ledger-exchange requests
    /// of `entries` peers each, then drain for long enough that anything the
    /// target was going to do has happened.
    pub async fn flood_ledger_exchange(
        addr: Multiaddr,
        target: libp2p::PeerId,
        requests: usize,
        entries: usize,
    ) {
        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )
            .expect("hostile tcp transport")
            .with_behaviour(|_| {
                Behaviour::new(
                    [(
                        StreamProtocol::new("/sc/ledger-exchange/1.0.0"),
                        ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                )
            })
            .expect("hostile behaviour")
            .build();

        let local = *swarm.local_peer_id();
        swarm.dial(addr).expect("hostile dial");

        // Wait for the connection, then flood.
        let connected = tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                if let Libp2pSwarmEvent::ConnectionEstablished { peer_id, .. } =
                    swarm.select_next_some().await
                {
                    if peer_id == target {
                        return true;
                    }
                }
            }
        })
        .await
        .unwrap_or(false);
        assert!(connected, "hostile peer never connected to the target");

        let payload: Vec<SharedPeerEntry> = (0..entries)
            .map(|i| SharedPeerEntry {
                multiaddr: format!("/ip4/198.51.100.{}/tcp/9001", (i % 254) + 1),
                last_peer_id: Some(libp2p::PeerId::random().to_string()),
                last_seen: 1_700_000_000,
                known_topics: vec![format!("sc-attacker-topic-{i}")],
            })
            .collect();

        for _ in 0..requests {
            swarm.behaviour_mut().send_request(
                &target,
                LedgerExchangeRequest {
                    version_tag: 1,
                    peers: payload.clone(),
                    sender_peer_id: local.to_string(),
                    version: 1,
                },
            );
        }

        // Drive the swarm so the requests actually go out, and deliberately do
        // NOT answer the target's own reciprocal request -- otherwise the
        // target would emit a `LedgerReceived` for our response too and the
        // count under test would not be attributable to our requests.
        let _ = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let _ = swarm.select_next_some().await;
            }
        })
        .await;
    }
}

async fn first_loopback_tcp(rx: &mut mpsc::Receiver<SwarmEvent2>) -> Multiaddr {
    let mut all_addrs: Vec<Multiaddr> = Vec::new();
    tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(event) = rx.recv().await {
            if let SwarmEvent2::ListeningOn(addr) = event {
                all_addrs.push(addr);
                if select_dialable_tcp_loopback(&all_addrs).is_some() {
                    break;
                }
            }
        }
    })
    .await
    .ok();

    select_dialable_tcp_loopback(&all_addrs)
        .expect("no plain TCP loopback listen address was reported")
}

/// Dial `addr`, tolerating the case where mDNS already connected the two nodes.
async fn dial_or_already_connected(swarm: &SwarmHandle, addr: Multiaddr) {
    if let Err(e) = swarm.dial(addr).await {
        let msg = e.to_string();
        let already_connected =
            msg.contains("already connected") || msg.contains("dial is in progress");
        assert!(already_connected, "Failed to dial: {}", msg);
        eprintln!("[INFO] Explicit dial skipped, peers already connected: {msg}");
    }
}

/// Pick a plain TCP loopback listener, avoiding the fixed WS port 9002.
fn select_dialable_tcp_loopback(addrs: &[Multiaddr]) -> Option<Multiaddr> {
    let mut loopback_ephemeral: Option<Multiaddr> = None;
    let mut loopback_any_tcp: Option<Multiaddr> = None;

    for addr in addrs {
        let s = addr.to_string();
        if s.contains("/ws") || s.contains("/quic") || s.contains("/p2p-circuit") {
            continue;
        }

        let mut has_tcp = false;
        let mut tcp_port: u16 = 0;
        let mut is_loopback = false;
        for proto in addr.iter() {
            match proto {
                libp2p::multiaddr::Protocol::Ip4(ip) => {
                    if ip == std::net::Ipv4Addr::LOCALHOST {
                        is_loopback = true;
                    }
                }
                libp2p::multiaddr::Protocol::Tcp(p) => {
                    has_tcp = true;
                    tcp_port = p;
                }
                _ => {}
            }
        }

        if !has_tcp || !is_loopback {
            continue;
        }
        if loopback_any_tcp.is_none() {
            loopback_any_tcp = Some(addr.clone());
        }
        if tcp_port != 9002 && loopback_ephemeral.is_none() {
            loopback_ephemeral = Some(addr.clone());
        }
    }

    loopback_ephemeral.or(loopback_any_tcp)
}
