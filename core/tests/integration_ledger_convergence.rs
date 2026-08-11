// Integration test for ledger convergence between two nodes (FARM WS-FARM-F1)
//
// Proves two in-process nodes converge their peer ledgers via ledger_exchange
// after connecting: a pre-existing entry in node 1's ledger, never directly
// dialed or discovered by node 2, still ends up in node 2's ledger purely via
// the ledger_exchange protocol.
//
// Test is #[ignore] by default (real networking) - run with:
//   cargo test -p scmessenger-core --test integration_ledger_convergence -- --include-ignored

use libp2p::identity::Keypair;
use libp2p::Multiaddr;
use scmessenger_core::transport::swarm::{start_swarm, SwarmEvent2, SwarmHandle};
use scmessenger_core::IronCore;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::mpsc;

/// CHOKE-POINT REFACTOR (2026-07-26): this test used to hand `share_ledger` a
/// payload it built itself, and started both swarms with `core_handle: None`.
/// `SwarmCommand::ShareLedger` no longer accepts a payload -- both directions of
/// `/sc/ledger-exchange/1.0.0` now build from
/// `LedgerManager::exchange_response_entries`, so the ledger has to be reachable
/// through `IronCore` for the node to have anything to say. That is the
/// production wiring, so the test now matches it.
#[tokio::test]
#[ignore = "requires real networking; run with --include-ignored"]
async fn test_ledger_convergence_between_nodes() {
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init()
        .ok();

    let dir1 = TempDir::new().expect("tempdir 1");
    let dir2 = TempDir::new().expect("tempdir 2");
    let core1 = Arc::new(IronCore::with_storage(
        dir1.path().to_string_lossy().to_string(),
    ));
    let core2 = Arc::new(IronCore::with_storage(
        dir2.path().to_string_lossy().to_string(),
    ));

    // Seed before either node can connect. Automatic discovery may establish
    // the connection immediately after swarm2 starts, and the first exchange
    // is intentionally deduplicated for the lifetime of that peer connection.
    let seeded_peer_id = libp2p::PeerId::random().to_string();
    core1
        .ledger_manager
        .record_connection("/ip4/1.2.3.4/tcp/9000".to_string(), seeded_peer_id);

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

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Collect all ListeningOn events in a bounded window so we can pick the
    // plain, directly-dialable TCP listen address. We must be careful because
    // multiple ListeningOn events arrive. We need to select a plain TCP
    // listener, preferring localhost (127.0.0.1 or ::1), and avoiding port 9002 (fixed WS port)
    // or any address containing /ws, /quic-v1, /p2p-circuit.
    let mut all_addrs: Vec<libp2p::Multiaddr> = Vec::new();
    tokio::time::timeout(Duration::from_secs(3), async {
        while let Some(event) = event_rx1.recv().await {
            if let SwarmEvent2::ListeningOn(addr) = event {
                all_addrs.push(addr);
                let has_loopback_tcp = all_addrs.iter().any(|a| {
                    let s = a.to_string();
                    s.contains("/127.0.0.1/")
                        && s.contains("/tcp/")
                        && !s.contains("/ws")
                        && !s.contains("/quic")
                });
                if has_loopback_tcp {
                    break;
                }
            }
        }
    })
    .await
    .ok();

    assert!(
        !all_addrs.is_empty(),
        "Node 1 should have at least one listen address"
    );

    let node1_addr = select_dialable_tcp_loopback(&all_addrs)
        .expect("No suitable plain TCP loopback address found among node1 listeners");

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

    tokio::time::sleep(Duration::from_millis(1500)).await;

    // Node 2's event loop: record whatever ledger entries it receives.
    let core2_for_task = core2.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx2.recv().await {
            if let SwarmEvent2::LedgerReceived { entries, .. } = event {
                for entry in entries {
                    if let Some(peer_id) = entry.last_peer_id {
                        core2_for_task
                            .ledger_manager
                            .record_connection(entry.multiaddr, peer_id);
                    }
                }
            }
        }
    });

    // Append /p2p/<peer_id1> so libp2p can associate the dial with a known PeerId.
    // Without this suffix, dial() succeeds but libp2p reports "no addresses for peer"
    // because it cannot track the connection against a specific PeerId.
    let mut dial_addr = node1_addr.clone();
    dial_addr.push(libp2p::multiaddr::Protocol::P2p(peer_id1));
    dial_or_already_connected(&swarm2, dial_addr).await;

    // No application layer calls share_ledger: ConnectionEstablished must
    // initiate convergence from the core for every platform.
    tokio::time::sleep(Duration::from_secs(3)).await;

    let dialable_addresses = core2.ledger_manager.dialable_addresses();
    let has_converged_entry = dialable_addresses
        .iter()
        .any(|entry| entry.multiaddr == "/ip4/1.2.3.4/tcp/9000");

    assert!(
        has_converged_entry,
        "Node 2's ledger should contain node 1's pre-existing entry via ledger_exchange"
    );

    let _ = swarm1.shutdown().await;
    let _ = swarm2.shutdown().await;
}

/// Item 3 of the v0.4.0 ledger work: the ledger-exchange RESPONSE must be
/// populated by the swarm itself from `IronCore::ledger_manager`.
///
/// Before this fix `swarm.rs` answered every `/sc/ledger-exchange/1.0.0`
/// request with `peers: Vec::new()`, on the assumption that the application
/// layer would follow up with a `ShareLedger` command. Only the CLI ever did,
/// so a phone received ledgers and answered with silence. This test asserts
/// convergence in BOTH directions from a single initiation, with neither node's
/// application layer calling `share_ledger` on the responding side.
#[tokio::test]
#[ignore = "requires real networking; run with --include-ignored"]
async fn test_ledger_exchange_response_is_reciprocated_from_core() {
    use scmessenger_core::IronCore;
    use tempfile::TempDir;

    tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init()
        .ok();

    // Each node gets its own IronCore (and therefore its own LedgerManager),
    // wired into the swarm as a core handle.
    let dir1 = TempDir::new().expect("tempdir 1");
    let dir2 = TempDir::new().expect("tempdir 2");
    let core1 = Arc::new(IronCore::with_storage(
        dir1.path().to_string_lossy().to_string(),
    ));
    let core2 = Arc::new(IronCore::with_storage(
        dir2.path().to_string_lossy().to_string(),
    ));

    // A proven entry in each ledger that the other node can only learn via
    // ledger exchange.
    const NODE1_ONLY_ADDR: &str = "/ip4/8.8.8.8/tcp/9000";
    const NODE2_ONLY_ADDR: &str = "/ip4/1.1.1.1/tcp/9000";
    core1.ledger_manager.record_connection(
        NODE1_ONLY_ADDR.to_string(),
        libp2p::PeerId::random().to_string(),
    );
    core2.ledger_manager.record_connection(
        NODE2_ONLY_ADDR.to_string(),
        libp2p::PeerId::random().to_string(),
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

    tokio::time::sleep(Duration::from_millis(500)).await;

    // Drain node 1's listen addresses, and remember any ledger entries node 1
    // receives so we can assert the request direction too.
    let mut all_addrs: Vec<Multiaddr> = Vec::new();
    let node1_received: Arc<parking_lot::Mutex<Vec<String>>> =
        Arc::new(parking_lot::Mutex::new(Vec::new()));
    tokio::time::timeout(Duration::from_secs(3), async {
        while let Some(event) = event_rx1.recv().await {
            if let SwarmEvent2::ListeningOn(addr) = event {
                all_addrs.push(addr);
                let has_loopback_tcp = all_addrs.iter().any(|a| {
                    let s = a.to_string();
                    s.contains("/127.0.0.1/")
                        && s.contains("/tcp/")
                        && !s.contains("/ws")
                        && !s.contains("/quic")
                });
                if has_loopback_tcp {
                    break;
                }
            }
        }
    })
    .await
    .ok();

    let node1_addr = select_dialable_tcp_loopback(&all_addrs)
        .expect("No suitable plain TCP loopback address found among node1 listeners");

    let node1_received_task = node1_received.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx1.recv().await {
            if let SwarmEvent2::LedgerReceived { entries, .. } = event {
                let mut seen = node1_received_task.lock();
                for entry in entries {
                    seen.push(entry.multiaddr);
                }
            }
        }
    });

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

    let node2_received: Arc<parking_lot::Mutex<Vec<String>>> =
        Arc::new(parking_lot::Mutex::new(Vec::new()));
    let node2_received_task = node2_received.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx2.recv().await {
            if let SwarmEvent2::LedgerReceived { entries, .. } = event {
                let mut seen = node2_received_task.lock();
                for entry in entries {
                    seen.push(entry.multiaddr);
                }
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let mut dial_addr = node1_addr.clone();
    dial_addr.push(libp2p::multiaddr::Protocol::P2p(peer_id1));
    dial_or_already_connected(&swarm2, dial_addr).await;
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // ONLY node 2 initiates. Node 1's application layer never calls
    // share_ledger -- the swarm must answer out of core1's ledger by itself.
    //
    // Node 2's REQUEST payload is likewise built inside the swarm from
    // `exchange_response_entries`, so this also asserts that the request door
    // and the response door are the same door (re-review NEW-2).
    let outbound = core2
        .ledger_manager
        .exchange_response_entries(64, &peer_id1.to_string(), &[]);
    assert!(
        outbound.iter().any(|e| e.multiaddr == NODE2_ONLY_ADDR),
        "node 2 should be offering its own seeded entry; got {:?}",
        outbound
            .iter()
            .map(|e| e.multiaddr.as_str())
            .collect::<Vec<_>>()
    );
    swarm2
        .share_ledger(peer_id1)
        .await
        .expect("Failed to share ledger");

    tokio::time::sleep(Duration::from_secs(3)).await;

    // Request direction (worked before this change).
    assert!(
        node1_received.lock().iter().any(|a| a == NODE2_ONLY_ADDR),
        "node 1 should have received node 2's entry from the exchange request; got {:?}",
        node1_received.lock()
    );

    // Response direction (the reciprocity gap this change closes).
    assert!(
        node2_received.lock().iter().any(|a| a == NODE1_ONLY_ADDR),
        "node 2 should have received node 1's entry in the exchange RESPONSE \
         without node 1's app layer calling share_ledger; got {:?}",
        node2_received.lock()
    );

    let _ = swarm1.shutdown().await;
    let _ = swarm2.shutdown().await;
}

/// Dial `addr`, tolerating the case where the two nodes have already found each
/// other.
///
/// Both nodes run on the same host with mDNS enabled, so libp2p frequently
/// connects them before the explicit dial and then rejects it with
/// `PeerCondition::Disconnected` / `NotDialing`. That is a connected outcome,
/// not a failure — panicking on it made this whole file fail on any machine
/// where mDNS wins the race. Genuine dial failures still surface, because the
/// convergence assertions below cannot pass without a live connection.
async fn dial_or_already_connected(swarm: &SwarmHandle, addr: Multiaddr) {
    // The test creates this loopback target itself; use the narrowly scoped
    // trusted-local path rather than weakening the production untrusted filter.
    if let Err(e) = swarm.dial_trusted_local_proxy(addr).await {
        let msg = e.to_string();
        let already_connected =
            msg.contains("already connected") || msg.contains("dial is in progress");
        assert!(already_connected, "Failed to dial: {}", msg);
        eprintln!("[INFO] Explicit dial skipped, peers already connected: {msg}");
    }
}

/// Select a plain TCP loopback address from a list of ListeningOn multiaddrs.
///
/// Picks the first address matching ALL of:
///   - contains /ip4/127.0.0.1  (loopback -- this is an in-process localhost test)
///   - contains /tcp/<port>     (plain TCP, not QUIC)
///   - port != 9002             (hardcoded WS listener port shared by both nodes)
///   - no /ws, /wss, /quic-v1, /p2p-circuit protocol components
///
/// Falls back to any 127.0.0.1/tcp address if the port-9002 filter is too
/// aggressive (should not happen in practice), then to any /ip4 + /tcp addr.
fn select_dialable_tcp_loopback(addrs: &[Multiaddr]) -> Option<Multiaddr> {
    // Classify each address.
    let mut loopback_ephemeral: Option<Multiaddr> = None;
    let mut loopback_any_tcp: Option<Multiaddr> = None;
    let mut any_plain_tcp: Option<Multiaddr> = None;

    for addr in addrs {
        let s = addr.to_string();

        // Reject non-plain-TCP transports.
        if s.contains("/ws")
            || s.contains("/wss")
            || s.contains("/quic")
            || s.contains("/p2p-circuit")
        {
            continue;
        }

        // Must have /tcp.
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

        if !has_tcp {
            continue;
        }

        if any_plain_tcp.is_none() {
            any_plain_tcp = Some(addr.clone());
        }

        if is_loopback {
            if loopback_any_tcp.is_none() {
                loopback_any_tcp = Some(addr.clone());
            }
            // Prefer ephemeral port (not the hardcoded WS port 9002).
            if tcp_port != 9002 && loopback_ephemeral.is_none() {
                loopback_ephemeral = Some(addr.clone());
            }
        }
    }

    loopback_ephemeral.or(loopback_any_tcp).or(any_plain_tcp)
}
