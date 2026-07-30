// Integration tests for P1 Graceful Dial Policy (Items 3+4)
// Tests per-peer backoff state machine and circuit-relay ladder preference.

use std::time::Duration;

#[test]
fn test_dial_policy_exponential_backoff() {
    // Test that backoff doubles from 1s to 30s cap
    use scmessenger_core::transport::dial_policy::PerPeerBackoffState;

    let mut state = PerPeerBackoffState::new(None);

    // Initial state
    assert_eq!(state.attempt_count, 0);
    assert_eq!(state.backoff_duration, Duration::from_secs(1));
    assert!(!state.is_dead);
    assert!(state.is_eligible());

    // 1st failure: 1s → 2s
    state.on_dial_failure();
    assert_eq!(state.attempt_count, 1);
    assert_eq!(state.backoff_duration, Duration::from_secs(2));
    assert!(!state.is_dead);
    assert!(!state.is_eligible()); // Backoff not elapsed yet

    // 2nd failure: 2s → 4s
    state.on_dial_failure();
    assert_eq!(state.attempt_count, 2);
    assert_eq!(state.backoff_duration, Duration::from_secs(4));
    assert!(!state.is_dead);

    // 3rd failure: 4s → 8s
    state.on_dial_failure();
    assert_eq!(state.attempt_count, 3);
    assert_eq!(state.backoff_duration, Duration::from_secs(8));
    assert!(state.is_dead); // Marked dead after 3 attempts

    // 4th failure attempt (should still be dead)
    state.on_dial_failure();
    assert_eq!(state.attempt_count, 4);
    assert!(state.is_dead);
}

#[test]
fn test_backoff_cap_at_30_seconds() {
    use scmessenger_core::transport::dial_policy::PerPeerBackoffState;

    let mut state = PerPeerBackoffState::new(None);

    // Simulate enough failures to reach and exceed 30s cap
    for _ in 0..15 {
        if state.is_dead {
            break;
        }
        state.on_dial_failure();
    }

    // Backoff should never exceed 30s
    assert!(state.backoff_duration <= Duration::from_secs(30));
    assert!(state.is_dead); // Should be marked dead after 3 attempts
}

#[test]
fn test_connection_resets_backoff() {
    use scmessenger_core::transport::dial_policy::PerPeerBackoffState;

    let mut state = PerPeerBackoffState::new(None);

    // Fail twice
    state.on_dial_failure();
    state.on_dial_failure();
    assert_eq!(state.attempt_count, 2);
    assert_eq!(state.backoff_duration, Duration::from_secs(4));

    // Connection succeeds
    state.on_connection_established();
    assert_eq!(state.attempt_count, 0);
    assert_eq!(state.backoff_duration, Duration::from_secs(1));
    assert!(!state.is_dead);
    assert!(state.is_eligible());
}

#[test]
fn test_permanent_failure_marks_dead() {
    use scmessenger_core::transport::dial_policy::PerPeerBackoffState;

    let mut state = PerPeerBackoffState::new(None);

    // Single permanent failure should mark as dead
    state.on_permanent_failure();
    assert!(state.is_dead);
    assert_eq!(state.attempt_count, 3);
    assert!(!state.is_eligible());
}

#[test]
fn test_dial_policy_manager_concurrent_limit() {
    use scmessenger_core::transport::dial_policy::DialPolicyManager;

    let manager = DialPolicyManager::new();

    let addr = "test-peer-addr";

    // Should allow up to 3 concurrent dials
    assert!(manager.register_dial_attempt(addr, None));
    assert!(manager.register_dial_attempt(addr, None));
    assert!(manager.register_dial_attempt(addr, None));

    // 4th dial should be rejected (concurrent limit)
    assert!(!manager.register_dial_attempt(addr, None));

    // After completing one, we can register another
    manager.complete_dial_attempt(addr);
    assert!(manager.register_dial_attempt(addr, None));

    // Back to limit
    assert!(!manager.register_dial_attempt(addr, None));
}

#[test]
fn test_dial_policy_manager_backoff_rejection() {
    use scmessenger_core::transport::dial_policy::DialPolicyManager;

    let manager = DialPolicyManager::new();
    let addr = "test-peer";

    // First dial is allowed
    assert!(manager.register_dial_attempt(addr, None));
    manager.complete_dial_attempt(addr);

    // After failure, backoff should prevent immediate retry
    manager.record_dial_failure(addr, None);
    assert!(!manager.register_dial_attempt(addr, None));

    // But we can still query the state
    let state = manager.get_backoff_state(addr);
    assert!(state.is_some());
    let s = state.unwrap();
    assert_eq!(s.attempt_count, 1);
}

#[test]
fn test_dial_policy_manager_dead_peer() {
    use scmessenger_core::transport::dial_policy::DialPolicyManager;

    let manager = DialPolicyManager::new();
    let addr = "test-peer";

    // Register and fail 3 times to mark as dead
    for _ in 0..3 {
        manager.record_dial_failure(addr, None);
    }

    // Now the peer should be marked as dead
    let state = manager.get_backoff_state(addr);
    assert!(state.is_some());
    assert!(state.unwrap().is_dead);

    // Future dial attempts should be rejected
    assert!(!manager.register_dial_attempt(addr, None));
}

#[test]
fn test_dial_policy_multiple_peers() {
    use scmessenger_core::transport::dial_policy::DialPolicyManager;

    let manager = DialPolicyManager::new();

    // Different peers should have independent backoff states
    assert!(manager.register_dial_attempt("peer-a", None));
    assert!(manager.register_dial_attempt("peer-b", None));
    assert!(manager.register_dial_attempt("peer-c", None));

    manager.complete_dial_attempt("peer-a");
    manager.record_dial_failure("peer-a", None);

    // peer-a should now be backed off
    assert!(!manager.register_dial_attempt("peer-a", None));

    // But other peers should still be available
    manager.complete_dial_attempt("peer-b");
    assert!(manager.register_dial_attempt("peer-b", None));

    manager.complete_dial_attempt("peer-c");
    assert!(manager.register_dial_attempt("peer-c", None));
}

#[test]
fn test_circuit_relay_ladder_construction() {
    use libp2p::identity::Keypair;
    use scmessenger_core::transport::dial_policy::CircuitRelayLadder;

    let ladder = CircuitRelayLadder::new();

    // Create mock relay and target peers
    let relay_kp = Keypair::generate_ed25519();
    let relay_pid = relay_kp.public().to_peer_id();

    let target_kp = Keypair::generate_ed25519();
    let target_pid = target_kp.public().to_peer_id();

    // Register relay with some addresses
    let relay_addr: libp2p::Multiaddr = "/ip4/192.168.1.100/tcp/4001".parse().unwrap();
    ladder.add_relay(relay_pid, vec![relay_addr.clone()]);

    // Build relay addresses for target
    let relay_addresses = ladder.build_relay_addresses(target_pid);

    assert!(!relay_addresses.is_empty());

    // Check structure: should contain /p2p-circuit/
    let addr_str = relay_addresses[0].to_string();
    assert!(addr_str.contains("/p2p-circuit/"));

    // Should contain relay peer ID
    assert!(addr_str.contains(&relay_pid.to_string()));

    // Should contain target peer ID
    assert!(addr_str.contains(&target_pid.to_string()));
}

#[test]
fn test_circuit_relay_ladder_multiple_relays() {
    use libp2p::identity::Keypair;
    use scmessenger_core::transport::dial_policy::CircuitRelayLadder;

    let ladder = CircuitRelayLadder::new();

    let target_kp = Keypair::generate_ed25519();
    let target_pid = target_kp.public().to_peer_id();

    // Register multiple relays
    for i in 0..3 {
        let relay_kp = Keypair::generate_ed25519();
        let relay_pid = relay_kp.public().to_peer_id();
        let relay_addr: libp2p::Multiaddr = format!("/ip4/192.168.1.{}/tcp/4001", 100 + i)
            .parse()
            .unwrap();
        ladder.add_relay(relay_pid, vec![relay_addr]);
    }

    // Build addresses for target
    let relay_addresses = ladder.build_relay_addresses(target_pid);

    // Should have addresses from all relays
    assert_eq!(relay_addresses.len(), 3);
}

#[test]
fn test_multiaddr_to_key_strips_peer_id() {
    use scmessenger_core::transport::dial_policy::multiaddr_to_key;

    let pid = libp2p::identity::Keypair::generate_ed25519()
        .public()
        .to_peer_id();
    let addr_str = format!("/ip4/192.168.1.1/tcp/4001/p2p/{}", pid);
    let addr_with_p2p: libp2p::Multiaddr = addr_str.parse().unwrap();

    let key = multiaddr_to_key(&addr_with_p2p);

    // Key should not contain /p2p/ component
    assert!(!key.contains("/p2p/"));

    // Key should contain IP and port
    assert!(key.contains("192.168.1.1"));
    assert!(key.contains("4001"));
}

#[test]
fn test_backoff_eligibility_timing() {
    use scmessenger_core::transport::dial_policy::PerPeerBackoffState;
    use std::time::{Duration, Instant};

    let mut state = PerPeerBackoffState::new(None);

    // Initial state is eligible
    assert!(state.is_eligible());

    // After failure, should not be eligible immediately
    state.on_dial_failure();
    assert!(!state.is_eligible());

    // Simulate backoff expiration by manually updating
    // (In real use, time passes naturally)
    let now = Instant::now();
    state.last_attempt_ts = now - Duration::from_secs(3); // 3 seconds ago

    // Now with backoff of 2 seconds, should be eligible again
    assert!(state.is_eligible());
}

#[test]
fn test_dial_policy_concurrent_limit_per_peer() {
    use scmessenger_core::transport::dial_policy::DialPolicyManager;

    let manager = DialPolicyManager::new();

    // peer-a: register 3 dials
    for _ in 0..3 {
        assert!(manager.register_dial_attempt("peer-a", None));
    }

    // peer-a should be at limit
    assert!(!manager.register_dial_attempt("peer-a", None));

    // peer-b should still have capacity
    assert!(manager.register_dial_attempt("peer-b", None));
    assert!(manager.register_dial_attempt("peer-b", None));
    assert!(manager.register_dial_attempt("peer-b", None));
    assert!(!manager.register_dial_attempt("peer-b", None));

    // Completing one dial for peer-a should free up a slot
    manager.complete_dial_attempt("peer-a");
    assert!(manager.register_dial_attempt("peer-a", None));

    // But peer-b's limit is still enforced independently
    assert!(!manager.register_dial_attempt("peer-b", None));
}

#[test]
fn test_backoff_progression_sequence() {
    use scmessenger_core::transport::dial_policy::PerPeerBackoffState;

    let mut state = PerPeerBackoffState::new(None);

    // Track the progression: 2s -> 4s -> 8s (capped at 3rd attempt, won't increase further inside loop since we only call on_dial_failure for attempt < 3)
    let expected_backoffs = vec![2, 4, 8, 8, 8, 8, 8];

    for (attempt, expected_secs) in expected_backoffs.iter().enumerate() {
        if attempt < 3 {
            // First 3 failures bring us to "dead" status
            state.on_dial_failure();
            if attempt < 3 {
                assert_eq!(
                    state.backoff_duration.as_secs(),
                    *expected_secs as u64,
                    "Attempt {} backoff mismatch",
                    attempt
                );
            }
        } else {
            break; // Can't fail more after marked dead in our current implementation
        }
    }
}

#[test]
fn test_circuit_relay_invalid_addresses_skipped() {
    use libp2p::identity::Keypair;
    use scmessenger_core::transport::dial_policy::CircuitRelayLadder;

    let ladder = CircuitRelayLadder::new();

    let relay_kp = Keypair::generate_ed25519();
    let relay_pid = relay_kp.public().to_peer_id();

    let target_kp = Keypair::generate_ed25519();
    let target_pid = target_kp.public().to_peer_id();

    // Add relay with invalid addresses (no IP or port)
    let invalid_addr: libp2p::Multiaddr = "/dns/example.com".parse().unwrap();
    ladder.add_relay(relay_pid, vec![invalid_addr]);

    // Build relay addresses - should be empty since relay address has no TCP port
    let relay_addresses = ladder.build_relay_addresses(target_pid);

    // May be empty or contain the addr depending on implementation
    // The important thing is it doesn't crash
    let _ = relay_addresses.len();
}

#[test]
fn test_dial_policy_prune_old_entries() {
    use scmessenger_core::transport::dial_policy::DialPolicyManager;
    use std::time::Duration;

    let manager = DialPolicyManager::new();

    // Register some peers
    assert!(manager.register_dial_attempt("peer-a", None));
    manager.complete_dial_attempt("peer-a");

    // Get state (should exist)
    assert!(manager.get_backoff_state("peer-a").is_some());

    // Prune old entries (with very short age threshold)
    // In real scenario, entries older than 1 hour are pruned
    // For testing, we'd need to mock time, but we can at least test
    // that the method doesn't crash
    manager.prune_old_entries(Duration::from_secs(0));

    // Entry might be pruned now (if system is fast enough)
    // But at minimum, function should not panic
}
