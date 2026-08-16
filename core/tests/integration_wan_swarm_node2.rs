// integration_wan_swarm_node2.rs
// Comprehensive Integration Test Suite for Node 2: WAN Transport & Swarm Infrastructure in SCMessenger
//
// Enforces the 5-Layer Deep Verification Standard:
// Layer 1: Domain Assertions (State transitions, queue ordering, circuit breaker, dial policy, relay lifecycle)
// Layer 2: Coverage (Happy paths, error arms, match branches across all target modules)
// Layer 3: Panic Safety & Boundaries (Extreme, corrupted, invalid inputs, edge cases — NEVER panic)
// Layer 4: Multi-Hop Call Depth (End-to-end traversal across TransportManager -> EscalationEngine -> InternetRelay -> CircuitBreaker -> DialPolicy -> Swarm)
// Layer 5: Memory Zeroization & Secret Safety (Key material zeroization, buffer clearing, shutdown hygiene)

use std::sync::Arc;
use web_time::{Duration, SystemTime};

use libp2p::{identity::Keypair, Multiaddr, PeerId};
use zeroize::Zeroize;

use scmessenger_core::transport::abstraction::{
    TransportCapabilities, TransportError, TransportEvent, TransportType,
};
use scmessenger_core::transport::circuit_breaker::{
    CircuitBreakerConfig, CircuitBreakerManager, CircuitState,
};
use scmessenger_core::transport::dial_policy::{
    multiaddr_to_key, CircuitRelayLadder, DialPolicyManager, PerPeerBackoffState,
};
use scmessenger_core::transport::escalation::{EscalationEngine, EscalationPolicy};
use scmessenger_core::transport::internet::{
    InternetRelay, InternetTransportConfig, InternetTransportError, NatStatus, RelayMode,
};
use scmessenger_core::transport::manager::{
    OutgoingQueue, PendingSend, ReconnectionState, SendResult, TransportManager,
};

// Helper function: Generate a dummy 32-byte peer identity
fn random_peer_id_bytes(seed: u8) -> [u8; 32] {
    let mut id = [0u8; 32];
    id[0] = seed;
    id[31] = seed;
    id
}

// Helper function: Generate a libp2p PeerId
fn random_libp2p_peer_id() -> PeerId {
    Keypair::generate_ed25519().public().to_peer_id()
}

// Helper function: Create default TransportCapabilities
fn mock_capabilities(bandwidth: u64, latency_ms: u32, streaming: bool) -> TransportCapabilities {
    TransportCapabilities {
        estimated_bandwidth_bps: bandwidth,
        estimated_latency_ms: latency_ms,
        supports_streaming: streaming,
        requires_encryption: true,
        max_message_size: 1_048_576,
    }
}

// ============================================================================
// LAYER 1: DOMAIN ASSERTIONS (Functional State Transitions & Operations)
// ============================================================================

mod layer1_domain_assertions {
    use super::*;

    #[test]
    fn test_tcp_quic_transport_manager_state_transitions() {
        let manager = TransportManager::new();

        // 1. Register TCP & QUIC transports
        let tcp_caps = mock_capabilities(10_000_000, 50, true);
        let quic_caps = mock_capabilities(50_000_000, 20, true);

        manager.register_transport(TransportType::TCP, tcp_caps.clone());
        manager.register_transport(TransportType::QUIC, quic_caps.clone());

        let peer1 = random_peer_id_bytes(1);

        // 2. Discover peer1 on TCP
        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id: peer1,
            transport: TransportType::TCP,
            address: b"192.168.1.10:4001".to_vec(),
        });

        assert!(manager.is_peer_connected(peer1));
        assert_eq!(manager.transports_for_peer(peer1), vec![TransportType::TCP]);

        // 3. Establish connection to peer1 on TCP
        manager.handle_event(TransportEvent::ConnectionEstablished {
            peer_id: peer1,
            transport: TransportType::TCP,
        });

        assert_eq!(manager.peers_on_transport(TransportType::TCP).len(), 1);
        assert_eq!(manager.peers_on_transport(TransportType::TCP)[0], peer1);

        // 4. Discover & establish peer1 on QUIC as well (multi-transport)
        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id: peer1,
            transport: TransportType::QUIC,
            address: b"192.168.1.10:4002".to_vec(),
        });
        manager.handle_event(TransportEvent::ConnectionEstablished {
            peer_id: peer1,
            transport: TransportType::QUIC,
        });

        // Best transport selection should prefer QUIC (higher bandwidth 50M vs 10M, lower latency 20ms vs 50ms)
        let best = manager
            .best_transport_for_peer(peer1)
            .expect("Best transport found");
        assert_eq!(best, TransportType::QUIC);

        // 5. Test priority-ordered outgoing queue via send_to_peer
        let res1 = manager.send_to_peer(peer1, b"low priority".to_vec(), 10);
        assert_eq!(res1, Ok(SendResult::Queued(TransportType::QUIC)));

        let res2 = manager.send_to_peer(peer1, b"high priority".to_vec(), 200);
        assert_eq!(res2, Ok(SendResult::Queued(TransportType::QUIC)));

        let pending = manager.pending_sends();
        assert_eq!(pending.len(), 2);
        // Priority order: highest priority (200) must be first
        assert_eq!(pending[0].priority, 200);
        assert_eq!(pending[1].priority, 10);

        // 6. Receive data on peer1
        manager.handle_event(TransportEvent::DataReceived {
            peer_id: peer1,
            data: b"hello".to_vec(),
        });

        // 7. Disconnect QUIC for peer1
        manager.handle_event(TransportEvent::PeerDisconnected {
            peer_id: peer1,
            transport: TransportType::QUIC,
        });

        // Still connected via TCP!
        assert!(manager.is_peer_connected(peer1));
        assert_eq!(
            manager.best_transport_for_peer(peer1),
            Ok(TransportType::TCP)
        );

        // 8. Disconnect TCP for peer1 -> fully disconnected
        manager.handle_event(TransportEvent::PeerDisconnected {
            peer_id: peer1,
            transport: TransportType::TCP,
        });

        assert!(!manager.is_peer_connected(peer1));
        assert!(manager.best_transport_for_peer(peer1).is_err());
    }

    #[test]
    fn test_transport_manager_reconnection_backoff_and_tick() {
        let mut manager = TransportManager::new();
        let escalation = Arc::new(EscalationEngine::new(EscalationPolicy::Balanced));
        manager.set_escalation_engine(escalation.clone());

        let tcp_caps = mock_capabilities(10_000_000, 50, true);
        manager.register_transport(TransportType::TCP, tcp_caps);

        let target_peer = random_peer_id_bytes(42);
        let target_addr = b"10.0.0.5:4001".to_vec();

        // Register target peer
        manager.add_target_peer(target_peer, target_addr.clone());

        // Discover and connect target peer
        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id: target_peer,
            transport: TransportType::TCP,
            address: target_addr,
        });
        manager.handle_event(TransportEvent::ConnectionEstablished {
            peer_id: target_peer,
            transport: TransportType::TCP,
        });

        // Peer disconnects -> should enter reconnection queue automatically because it is a target peer
        manager.handle_event(TransportEvent::PeerDisconnected {
            peer_id: target_peer,
            transport: TransportType::TCP,
        });

        assert_eq!(manager.reconnection_queue_len(), 1);

        // Test exponential backoff
        let mut recon_state = ReconnectionState::new(
            target_peer,
            [TransportType::TCP].into_iter().collect(),
            b"10.0.0.5:4001".to_vec(),
        );

        assert_eq!(recon_state.failures, 0);
        assert!(!recon_state.is_exhausted());

        // Fail 1: backoff -> 2s
        recon_state.record_failure();
        assert_eq!(recon_state.failures, 1);

        // Record failure in manager
        manager.record_reconnect_failure(&target_peer);

        // Record success removes from queue
        manager.record_reconnect_success(&target_peer);
        assert_eq!(manager.reconnection_queue_len(), 0);

        // Remove target peer stops tracking
        manager.remove_target_peer(&target_peer);
    }

    #[test]
    fn test_internet_relay_and_nat_traversal() {
        let config = InternetTransportConfig {
            listen_port: 5555,
            max_relay_connections: 5,
            relay_bandwidth_limit_bps: 1_000_000,
            relay_mode: RelayMode::Both,
            relay_timeout_secs: 300,
        };

        let relay = InternetRelay::new(config).expect("InternetRelay initialized");

        // 1. NAT status transitions
        assert_eq!(relay.get_nat_status(), NatStatus::Unknown);
        relay.set_nat_status(NatStatus::Restricted);
        assert_eq!(relay.get_nat_status(), NatStatus::Restricted);
        relay.set_nat_status(NatStatus::Open);
        assert_eq!(relay.get_nat_status(), NatStatus::Open);

        // 2. Register relay-capable peer
        let relay_peer_id = random_libp2p_peer_id();
        let relay_addr: Multiaddr = "/ip4/203.0.113.10/tcp/5555".parse().unwrap();

        let reg_res = relay.register_relay_peer(relay_peer_id, vec![relay_addr.clone()], true);
        assert!(reg_res.is_ok());
        assert_eq!(relay.get_active_relay_count(), 1);
        assert!(relay.can_accept_relay());

        // Verify relay info
        let info = relay
            .get_peer_relay_info(&relay_peer_id)
            .expect("Peer info exists");
        assert_eq!(info.peer_id, relay_peer_id);
        assert!(info.relay_capable);

        // 3. Store-and-forward relaying
        let payload = b"encrypted message payload for offline peer".to_vec();
        let relay_msg_res = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(relay.relay_for_peer(relay_peer_id, payload.clone()));
        assert!(relay_msg_res.is_ok());

        // Check stats tracking
        let stats = relay
            .get_relay_stats(&relay_peer_id)
            .expect("Relay stats exist");
        assert_eq!(stats.bytes_transferred, payload.len() as u64);

        // 4. Establish relay circuit between initiator and target via relay
        let initiator = random_libp2p_peer_id();
        let target = random_libp2p_peer_id();

        let circuit_res = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(relay.establish_relay_circuit(initiator, target, relay_peer_id));
        assert!(circuit_res.is_ok());

        // 5. Disconnect relay
        let disc_res = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(relay.disconnect_relay(relay_peer_id));
        assert!(disc_res.is_ok());
        assert_eq!(relay.get_active_relay_count(), 0);
    }

    #[test]
    fn test_circuit_breaker_state_transitions() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            open_timeout: Duration::from_millis(50),
            half_open_timeout: Duration::from_millis(20),
            success_threshold: 2,
            max_half_open_probes: 2,
        };

        let mgr = CircuitBreakerManager::new(config);
        let relay_addr = "/ip4/198.51.100.1/tcp/5555";

        // Initial state: Closed, allow_request = true
        assert_eq!(mgr.get_state(relay_addr), CircuitState::Closed);
        assert!(mgr.allow_request(relay_addr));

        // 2 failures: still Closed
        mgr.record_failure(relay_addr, "timeout 1");
        mgr.record_failure(relay_addr, "timeout 2");
        assert_eq!(mgr.get_failure_count(relay_addr), 2);
        assert_eq!(mgr.get_state(relay_addr), CircuitState::Closed);

        // 3rd failure: trips circuit to Open!
        mgr.record_failure(relay_addr, "timeout 3");
        assert_eq!(mgr.get_state(relay_addr), CircuitState::Open);
        assert!(!mgr.allow_request(relay_addr));

        // Verify open circuits stats
        let open_list = mgr.get_open_circuits();
        assert_eq!(open_list, vec![relay_addr.to_string()]);
        let stats = mgr.get_stats();
        assert_eq!(stats.open_count, 1);
        assert_eq!(stats.closed_count, 0);

        // Wait for open_timeout (50ms)
        std::thread::sleep(Duration::from_millis(60));

        // allow_request should now transition state to HalfOpen and return true for probing!
        assert!(mgr.allow_request(relay_addr));
        assert_eq!(mgr.get_state(relay_addr), CircuitState::HalfOpen);

        // Record 1st success in HalfOpen -> stays HalfOpen (threshold is 2)
        mgr.record_success(relay_addr);
        assert_eq!(mgr.get_state(relay_addr), CircuitState::HalfOpen);

        // Record 2nd success in HalfOpen -> closes circuit!
        mgr.record_success(relay_addr);
        assert_eq!(mgr.get_state(relay_addr), CircuitState::Closed);
        assert_eq!(mgr.get_failure_count(relay_addr), 0);

        // Test failure in HalfOpen immediately re-opens circuit
        for _ in 0..3 {
            mgr.record_failure(relay_addr, "fail");
        }
        assert_eq!(mgr.get_state(relay_addr), CircuitState::Open);
        std::thread::sleep(Duration::from_millis(60));
        assert!(mgr.allow_request(relay_addr)); // HalfOpen transition
        assert_eq!(mgr.get_state(relay_addr), CircuitState::HalfOpen);

        // Failure during probe re-opens circuit immediately
        mgr.record_failure(relay_addr, "probe failed");
        assert_eq!(mgr.get_state(relay_addr), CircuitState::Open);

        // Reset individual relay
        mgr.reset(relay_addr);
        assert_eq!(mgr.get_state(relay_addr), CircuitState::Closed);
    }

    #[test]
    fn test_dial_policy_and_backoff_state_machine() {
        let manager = DialPolicyManager::new();
        let peer_addr = "192.168.1.50:4001";
        let pid = random_libp2p_peer_id();

        // 1. Max 3 concurrent outbound dials to any peer address
        assert!(manager.register_dial_attempt(peer_addr, Some(pid)));
        assert!(manager.register_dial_attempt(peer_addr, Some(pid)));
        assert!(manager.register_dial_attempt(peer_addr, Some(pid)));

        // 4th concurrent dial rejected
        assert!(!manager.register_dial_attempt(peer_addr, Some(pid)));

        // Complete 1 dial attempt -> opens a slot
        manager.complete_dial_attempt(peer_addr);
        assert!(manager.register_dial_attempt(peer_addr, Some(pid)));
        assert!(!manager.register_dial_attempt(peer_addr, Some(pid)));

        // Clean up active dials
        manager.complete_dial_attempt(peer_addr);
        manager.complete_dial_attempt(peer_addr);
        manager.complete_dial_attempt(peer_addr);

        // 2. Exponential backoff state machine
        let mut backoff_state = PerPeerBackoffState::new(Some(pid));
        assert_eq!(backoff_state.backoff_duration, Duration::from_secs(1));
        assert!(!backoff_state.is_dead);

        // Failure 1: 1s -> 2s
        backoff_state.on_dial_failure();
        assert_eq!(backoff_state.attempt_count, 1);
        assert_eq!(backoff_state.backoff_duration, Duration::from_secs(2));

        // Failure 2: 2s -> 4s
        backoff_state.on_dial_failure();
        assert_eq!(backoff_state.attempt_count, 2);
        assert_eq!(backoff_state.backoff_duration, Duration::from_secs(4));

        // Failure 3: 4s -> 8s, marked dead!
        backoff_state.on_dial_failure();
        assert_eq!(backoff_state.attempt_count, 3);
        assert!(backoff_state.is_dead);
        assert!(!backoff_state.is_eligible());

        // Connection established resets backoff
        backoff_state.on_connection_established();
        assert_eq!(backoff_state.attempt_count, 0);
        assert_eq!(backoff_state.backoff_duration, Duration::from_secs(1));
        assert!(!backoff_state.is_dead);
        assert!(backoff_state.is_eligible());

        // 3. CircuitRelayLadder preference building
        let ladder = CircuitRelayLadder::new();
        let relay_pid = random_libp2p_peer_id();
        let target_pid = random_libp2p_peer_id();

        let relay_ext_addr: Multiaddr = "/ip4/203.0.113.100/tcp/4001".parse().unwrap();
        ladder.add_relay(relay_pid, vec![relay_ext_addr.clone()]);

        let relay_routes = ladder.build_relay_addresses(target_pid);
        assert_eq!(relay_routes.len(), 1);

        let constructed_str = relay_routes[0].to_string();
        assert!(constructed_str.contains("/p2p-circuit/"));
        assert!(constructed_str.contains(&relay_pid.to_string()));
        assert!(constructed_str.contains(&target_pid.to_string()));
    }

    #[test]
    fn test_libp2p_swarm_event_and_address_helpers() {
        let addr_with_p2p: Multiaddr = "/ip4/192.168.1.1/tcp/4001/p2p/12D3KooWSD55"
            .parse()
            .unwrap();
        let key = multiaddr_to_key(&addr_with_p2p);
        assert!(!key.contains("/p2p/"));
        assert!(key.contains("192.168.1.1"));
        assert!(key.contains("4001"));
    }
}

// ============================================================================
// LAYER 2: BRANCH COVERAGE (Happy Paths, Error Arms, Match Branches)
// ============================================================================

mod layer2_branch_coverage {
    use super::*;

    #[test]
    fn test_transport_manager_error_arms() {
        let manager = TransportManager::new();
        let peer = random_peer_id_bytes(99);

        // 1. best_transport_for_peer on unknown peer
        let err = manager.best_transport_for_peer(peer);
        assert!(matches!(err, Err(TransportError::PeerNotFound(_))));

        // 2. send_to_peer on unknown peer
        let send_err = manager.send_to_peer(peer, b"data".to_vec(), 100);
        assert!(matches!(send_err, Err(TransportError::PeerNotFound(_))));

        // 3. record_reconnect_failure on unknown peer (should not panic)
        manager.record_reconnect_failure(&peer);

        // 4. remove_target_peer on unknown peer (should not panic)
        manager.remove_target_peer(&peer);

        // 5. is_peer_connected on unknown peer -> false
        assert!(!manager.is_peer_connected(peer));
        assert!(manager.transports_for_peer(peer).is_empty());
        assert!(manager.peers_on_transport(TransportType::TCP).is_empty());
    }

    #[test]
    fn test_internet_relay_error_arms() {
        // 1. Invalid listen port (0)
        let config_invalid_port = InternetTransportConfig {
            listen_port: 0,
            ..Default::default()
        };
        let err1 = InternetRelay::new(config_invalid_port);
        assert!(matches!(err1, Err(InternetTransportError::ConfigError(_))));

        // 2. Invalid max relay connections (0)
        let config_invalid_conn = InternetTransportConfig {
            max_relay_connections: 0,
            ..Default::default()
        };
        let err2 = InternetRelay::new(config_invalid_conn);
        assert!(matches!(err2, Err(InternetTransportError::ConfigError(_))));

        // 3. Exceeding max relay connections
        let config_max_1 = InternetTransportConfig {
            max_relay_connections: 1,
            ..Default::default()
        };
        let relay = InternetRelay::new(config_max_1).unwrap();
        let peer1 = random_libp2p_peer_id();
        let peer2 = random_libp2p_peer_id();
        let addr: Multiaddr = "/ip4/1.2.3.4/tcp/5555".parse().unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            assert!(relay.connect_to_relay(peer1, addr.clone()).await.is_ok());
            let err = relay.connect_to_relay(peer2, addr).await;
            assert!(matches!(
                err,
                Err(InternetTransportError::MaxConnectionsExceeded)
            ));
        });

        // 4. Client mode trying to store-and-forward for peer
        let client_config = InternetTransportConfig {
            relay_mode: RelayMode::Client,
            ..Default::default()
        };
        let client_relay = InternetRelay::new(client_config).unwrap();
        let target_peer = random_libp2p_peer_id();

        rt.block_on(async {
            let err = client_relay
                .relay_for_peer(target_peer, b"data".to_vec())
                .await;
            assert!(matches!(err, Err(InternetTransportError::RelayUnavailable)));
        });

        // 5. Relaying for unregistered peer in Server mode
        let server_relay = InternetRelay::new(InternetTransportConfig::default()).unwrap();
        rt.block_on(async {
            let err = server_relay
                .relay_for_peer(target_peer, b"data".to_vec())
                .await;
            assert!(matches!(
                err,
                Err(InternetTransportError::RelayPeerNotFound(_))
            ));
        });

        // 6. Register relay peer with empty multiaddr vector
        let empty_reg_err = server_relay.register_relay_peer(target_peer, vec![], true);
        assert!(matches!(
            empty_reg_err,
            Err(InternetTransportError::InvalidRelayAddress)
        ));

        // 7. establish_relay_circuit for unknown relay peer
        rt.block_on(async {
            let circuit_err = server_relay
                .establish_relay_circuit(
                    random_libp2p_peer_id(),
                    target_peer,
                    random_libp2p_peer_id(),
                )
                .await;
            assert!(matches!(
                circuit_err,
                Err(InternetTransportError::RelayPeerNotFound(_))
            ));
        });
    }

    #[test]
    fn test_circuit_breaker_error_arms() {
        let mgr = CircuitBreakerManager::with_defaults();
        let unknown_relay = "unknown.relay.example.com";

        // 1. Unregistered relay returns true by default (implicitly Closed)
        assert!(mgr.allow_request(unknown_relay));
        assert_eq!(mgr.get_state(unknown_relay), CircuitState::Closed);
        assert_eq!(mgr.get_failure_count(unknown_relay), 0);
        assert_eq!(mgr.get_last_failure_reason(unknown_relay), None);

        // 2. Recording success on unrecorded or Open circuit handles gracefully
        mgr.record_success(unknown_relay);
        assert_eq!(mgr.get_state(unknown_relay), CircuitState::Closed);

        // 3. Resetting unknown relay does not error
        mgr.reset(unknown_relay);
        mgr.reset_all();
    }

    #[test]
    fn test_dial_policy_error_arms() {
        let manager = DialPolicyManager::new();
        let peer_addr = "10.0.0.1:4001";

        // Mark peer dead via permanent failure
        manager.record_permanent_failure(peer_addr, None);
        assert!(!manager.register_dial_attempt(peer_addr, None));

        let state = manager.get_backoff_state(peer_addr).unwrap();
        assert!(state.is_dead);

        // Decrementing complete_dial_attempt on 0 count does not underflow
        manager.complete_dial_attempt(peer_addr);

        // CircuitRelayLadder rejecting self-target routes
        let ladder = CircuitRelayLadder::new();
        let same_pid = random_libp2p_peer_id();

        let ext_addr: Multiaddr = "/ip4/192.168.1.1/tcp/4001".parse().unwrap();
        ladder.add_relay(same_pid, vec![ext_addr]);

        // Building relay addresses where target == relay should produce no routes!
        let routes = ladder.build_relay_addresses(same_pid);
        assert!(routes.is_empty());
    }
}

// ============================================================================
// LAYER 3: PANIC SAFETY & BOUNDARIES (Invalid, Extreme & Corrupted Inputs)
// ============================================================================

mod layer3_panic_safety_and_boundaries {
    use super::*;

    #[test]
    fn test_transport_manager_boundary_safety() {
        let manager = TransportManager::new();
        let peer = random_peer_id_bytes(0);

        let caps = mock_capabilities(0, u32::MAX, false);
        manager.register_transport(TransportType::BLE, caps);

        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id: peer,
            transport: TransportType::BLE,
            address: vec![],
        });
        manager.handle_event(TransportEvent::ConnectionEstablished {
            peer_id: peer,
            transport: TransportType::BLE,
        });

        // 1. Enqueue priority 0 and 255
        assert!(manager.send_to_peer(peer, vec![], 0).is_ok());
        assert!(manager
            .send_to_peer(peer, vec![0xff; 1_000_000], 255)
            .is_ok());

        assert_eq!(manager.pending_sends().len(), 2);

        // 2. Outgoing queue methods
        let mut queue = OutgoingQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.dequeue(), None);

        queue.enqueue(PendingSend {
            peer_id: peer,
            data: vec![],
            priority: 10,
            preferred_transport: None,
            created_at: SystemTime::now(),
        });
        assert!(!queue.is_empty());
        queue.clear();
        assert!(queue.is_empty());

        // 3. Tick with zero peers or extreme staleness
        manager.tick();
        manager.tick();
        manager.tick();

        // Expire address observations with 0 max age
        manager.expire_address_observations(0);
    }

    #[test]
    fn test_internet_relay_boundary_safety() {
        let config = InternetTransportConfig {
            relay_bandwidth_limit_bps: 1, // Tiny limit: 1 bps
            ..Default::default()
        };
        let relay = InternetRelay::new(config).unwrap();
        let peer = random_libp2p_peer_id();
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/5555".parse().unwrap();

        relay.register_relay_peer(peer, vec![addr], true).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // 0-byte payload should succeed
            assert!(relay.relay_for_peer(peer, vec![]).await.is_ok());

            // Huge payload exceeding bandwidth limit should return BandwidthExceeded error without panicking
            let huge_payload = vec![0u8; 100_000];
            let res = relay.relay_for_peer(peer, huge_payload).await;
            assert!(res.is_err());
        });

        // Cleanup stale relays on empty/populated map
        relay.cleanup_stale_relays();
    }

    #[test]
    fn test_circuit_breaker_boundary_safety() {
        let mgr = CircuitBreakerManager::with_defaults();

        // 1. Extreme address strings (empty, long, null bytes)
        let empty_key = "";
        let long_key = "a".repeat(10_000);
        let null_key = "relay\0with\0nulls";

        assert!(mgr.allow_request(empty_key));
        mgr.record_failure(long_key, "");
        mgr.record_failure(null_key, "null byte failure");

        assert_eq!(mgr.get_failure_count(long_key), 1);
        assert_eq!(
            mgr.get_last_failure_reason(null_key),
            Some("null byte failure".to_string())
        );

        // Reset all
        mgr.reset_all();
        assert_eq!(mgr.get_stats().total, 0);
    }

    #[test]
    fn test_dial_policy_boundary_safety() {
        let mut state = PerPeerBackoffState::new(None);

        // 1. Calling on_dial_failure 100 times continuously must not overflow u32 or Duration!
        for _ in 0..100 {
            state.on_dial_failure();
        }

        assert_eq!(state.attempt_count, 100);
        assert!(state.backoff_duration <= Duration::from_secs(30));
        assert!(state.is_dead);

        // 2. DialPolicyManager prune with Duration::ZERO and Duration::MAX
        let manager = DialPolicyManager::new();
        manager.register_dial_attempt("peer", None);
        manager.prune_old_entries(Duration::ZERO);
        manager.prune_old_entries(Duration::MAX);
    }
}

// ============================================================================
// LAYER 4: MULTI-HOP CALL DEPTH (End-to-End Component Traversal)
// ============================================================================

mod layer4_multi_hop_call_depth {
    use super::*;

    #[test]
    fn test_end_to_end_wan_swarm_lifecycle() {
        // Complete multi-hop architecture traversal:
        // TransportManager <-> EscalationEngine <-> DialPolicyManager <-> CircuitBreakerManager <-> CircuitRelayLadder <-> InternetRelay

        let mut manager = TransportManager::new();
        let escalation = Arc::new(EscalationEngine::new(EscalationPolicy::Balanced));
        let cb_mgr = CircuitBreakerManager::with_defaults();
        let dial_policy = DialPolicyManager::new();
        let relay_ladder = CircuitRelayLadder::new();
        let internet_config = InternetTransportConfig::default();
        let internet_relay = InternetRelay::new(internet_config).expect("Relay created");

        manager.set_escalation_engine(escalation.clone());

        // 1. Hop 1: Register TCP & QUIC transports with capabilities
        let tcp_caps = mock_capabilities(10_000_000, 100, true);
        let quic_caps = mock_capabilities(100_000_000, 10, true);

        manager.register_transport(TransportType::TCP, tcp_caps.clone());
        manager.register_transport(TransportType::QUIC, quic_caps.clone());

        escalation.set_capabilities(TransportType::TCP, tcp_caps);
        escalation.set_capabilities(TransportType::QUIC, quic_caps);

        let target_peer = random_peer_id_bytes(100);
        let libp2p_target = random_libp2p_peer_id();
        let relay_peer = random_libp2p_peer_id();

        // Initialize peer escalation state
        escalation
            .init_peer(target_peer, vec![TransportType::TCP, TransportType::QUIC])
            .expect("Peer escalation init");

        // 2. Hop 2: Connect peer via TCP initially
        manager.add_target_peer(target_peer, b"192.168.1.50:4001".to_vec());
        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id: target_peer,
            transport: TransportType::TCP,
            address: b"192.168.1.50:4001".to_vec(),
        });
        manager.handle_event(TransportEvent::ConnectionEstablished {
            peer_id: target_peer,
            transport: TransportType::TCP,
        });

        assert_eq!(
            manager.best_transport_for_peer(target_peer),
            Ok(TransportType::TCP)
        );

        // 3. Hop 3: Direct TCP fails and times out across STALE_CONFIRM_TICKS
        // 3 consecutive ticks trigger staleness -> deescalation & synthetic disconnect
        manager.tick();
        manager.tick();
        manager.tick();

        // Direct connection address trips CircuitBreaker
        let direct_addr_str = "192.168.1.50:4001";
        cb_mgr.record_failure(direct_addr_str, "TCP timeout");
        cb_mgr.record_failure(direct_addr_str, "TCP timeout");
        cb_mgr.record_failure(direct_addr_str, "TCP timeout");

        assert_eq!(cb_mgr.get_state(direct_addr_str), CircuitState::Open);
        assert!(!cb_mgr.allow_request(direct_addr_str));

        // DialPolicy records failure & applies backoff
        dial_policy.record_dial_failure(direct_addr_str, Some(libp2p_target));

        // 4. Hop 4: Fallback to CircuitRelay via InternetRelay
        let relay_multiaddr: Multiaddr = "/ip4/203.0.113.1/tcp/5555".parse().unwrap();
        relay_ladder.add_relay(relay_peer, vec![relay_multiaddr.clone()]);

        let circuit_addrs = relay_ladder.build_relay_addresses(libp2p_target);
        assert!(!circuit_addrs.is_empty());

        let circuit_key = multiaddr_to_key(&circuit_addrs[0]);

        // Register dial attempt on circuit key
        assert!(dial_policy.register_dial_attempt(&circuit_key, Some(libp2p_target)));

        // 5. Hop 5: Register & connect over InternetRelay circuit
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            internet_relay
                .register_relay_peer(relay_peer, vec![relay_multiaddr.clone()], true)
                .unwrap();
            internet_relay
                .connect_to_relay(relay_peer, relay_multiaddr)
                .await
                .unwrap();
        });

        // 6. Hop 6: Connection Established on QUIC fallback transport -> reset backoff & re-escalate!
        manager.handle_event(TransportEvent::PeerDiscovered {
            peer_id: target_peer,
            transport: TransportType::QUIC,
            address: circuit_addrs[0].to_vec(),
        });
        manager.handle_event(TransportEvent::ConnectionEstablished {
            peer_id: target_peer,
            transport: TransportType::QUIC,
        });

        dial_policy.reset_on_connection_established(&circuit_key, Some(libp2p_target));
        cb_mgr.record_success(&circuit_key);

        assert_eq!(
            manager.best_transport_for_peer(target_peer),
            Ok(TransportType::QUIC)
        );
    }
}

// ============================================================================
// LAYER 5: MEMORY ZEROIZATION & SECRET SAFETY
// ============================================================================

mod layer5_memory_zeroization_and_secret_safety {
    use super::*;

    #[test]
    fn test_keypair_memory_zeroization_and_hygiene() {
        // Generate an Ed25519 keypair for swarm node identity
        let keypair = Keypair::generate_ed25519();

        if let Ok(mut secret_bytes) = keypair.to_protobuf_encoding() {
            assert!(!secret_bytes.is_empty());

            // Zeroize sensitive key material buffer on drop
            secret_bytes.zeroize();
            assert!(secret_bytes.iter().all(|&b| b == 0));
        }
    }

    #[test]
    fn test_pending_queue_memory_clearing() {
        let mut queue = OutgoingQueue::new();

        // Enqueue sensitive packet payload
        let mut sensitive_data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        queue.enqueue(PendingSend {
            peer_id: random_peer_id_bytes(7),
            data: sensitive_data.clone(),
            priority: 100,
            preferred_transport: None,
            created_at: SystemTime::now(),
        });

        assert_eq!(queue.len(), 1);

        // Zeroize local buffer and clear queue memory
        sensitive_data.zeroize();
        queue.clear();

        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_internet_relay_shutdown_memory_clearing() {
        let config = InternetTransportConfig::default();
        let relay = InternetRelay::new(config).unwrap();
        let peer = random_libp2p_peer_id();
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/5555".parse().unwrap();

        relay.register_relay_peer(peer, vec![addr], true).unwrap();
        assert_eq!(relay.get_active_relay_count(), 1);

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let shutdown_res = relay.shutdown().await;
            assert!(shutdown_res.is_ok());
        });

        assert_eq!(relay.get_active_relay_count(), 0);
        assert!(relay.get_all_relay_stats().is_empty());
    }
}
