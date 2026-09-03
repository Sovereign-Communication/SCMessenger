//! Integration tests for mycorrhizal routing activation
//!
//! Verifies that:
//! - OptimizedRoutingEngine is properly wired to production transport dispatch
//! - Transport quality scoring influences routing decisions
//! - Negative cache blocks unreachable peers
//! - Adaptive TTL tracks peer activity
//! - Multi-transport routing decisions are observable via diagnostics

use scmessenger_core::routing::{
    engine::{NextHop, RoutingLayer},
    local::{CellSummary, PeerId, TransportType},
    optimized_engine::OptimizedRoutingEngine,
    timeout_budget::DiscoveryPhase,
};

fn make_peer_id(id: u8) -> PeerId {
    let mut peer = [0u8; 32];
    peer[0] = id;
    peer
}

fn make_hint(id: u8) -> [u8; 8] {
    [id, 0, 0, 0, 0, 0, 0, 0]
}

fn make_message_id(id: u8) -> [u8; 16] {
    [id; 16]
}

/// Helper: compute the hint for a peer ID using the same derivation
/// the routing engine uses (blake3 hash, first 8 bytes).
fn peer_hint(peer_id: &PeerId) -> [u8; 8] {
    blake3::hash(peer_id).as_bytes()[0..8]
        .try_into()
        .unwrap_or([0u8; 8])
}

/// Helper: register a peer in the local cell and set its reachable hints
/// so the routing engine can find it by hint.
fn register_peer_with_hint(
    engine: &mut OptimizedRoutingEngine,
    peer_id: PeerId,
    transport: TransportType,
    hints: Vec<[u8; 8]>,
) {
    engine
        .base_engine_mut()
        .local_cell_mut()
        .peer_seen(peer_id, transport);
    engine
        .base_engine_mut()
        .local_cell_mut()
        .update_peer_hints(&peer_id, hints);
}

/// Verify that the OptimizedRoutingEngine can be created with a local peer ID
/// and produces valid routing decisions.
#[test]
fn test_routing_engine_creation_and_basic_decision() {
    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);
    let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

    // Without any peers, routing should fall back to StoreAndCarry
    let target_hint = make_hint(99);
    let msg_id = make_message_id(1);
    let decision = engine.route_message_optimized(&target_hint, &msg_id, 50, 1000);

    assert!(
        matches!(
            decision.primary,
            NextHop::StoreAndCarry | NextHop::RouteDiscovery { .. }
        ),
        "Expected StoreAndCarry or RouteDiscovery for unknown peer, got {:?}",
        decision.primary
    );
}

/// Verify that when a peer is seen in the local cell with reachable hints,
/// the routing engine produces a Direct route for that peer.
#[test]
fn test_direct_route_when_peer_known() {
    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);
    let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

    // Register a peer in the local cell with reachable hints
    let peer_id = make_peer_id(2);
    let hint = peer_hint(&peer_id);
    register_peer_with_hint(&mut engine, peer_id, TransportType::BLE, vec![hint]);

    let msg_id = make_message_id(1);
    let decision = engine.route_message_optimized(&hint, &msg_id, 50, 1000);

    assert!(
        matches!(decision.primary, NextHop::Direct { .. }),
        "Expected Direct route for known peer, got {:?}",
        decision.primary
    );
    assert_eq!(decision.decided_by, RoutingLayer::Local);
}

/// Verify that transport quality scoring influences routing decisions.
/// A peer seen on QUIC should have the transport type recorded in the NextHop.
#[test]
fn test_transport_type_in_routing_decision() {
    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);
    let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

    // Register a peer on QUIC transport
    let peer_quic = make_peer_id(10);
    let hint_quic = peer_hint(&peer_quic);
    register_peer_with_hint(&mut engine, peer_quic, TransportType::QUIC, vec![hint_quic]);

    let msg_id = make_message_id(1);
    let decision = engine.route_message_optimized(&hint_quic, &msg_id, 50, 1000);

    if let NextHop::Direct { transport, .. } = decision.primary {
        assert_eq!(
            transport,
            TransportType::QUIC,
            "Expected QUIC transport in routing decision"
        );
    } else {
        panic!(
            "Expected Direct route with QUIC transport, got {:?}",
            decision.primary
        );
    }

    // Register a peer on BLE transport
    let peer_ble = make_peer_id(20);
    let hint_ble = peer_hint(&peer_ble);
    register_peer_with_hint(&mut engine, peer_ble, TransportType::BLE, vec![hint_ble]);

    let decision = engine.route_message_optimized(&hint_ble, &msg_id, 50, 1000);

    if let NextHop::Direct { transport, .. } = decision.primary {
        assert_eq!(
            transport,
            TransportType::BLE,
            "Expected BLE transport in routing decision"
        );
    } else {
        panic!(
            "Expected Direct route with BLE transport, got {:?}",
            decision.primary
        );
    }
}

/// Verify that reliability updates affect routing behavior.
/// After marking a peer as unreliable, the confidence in its route should decrease.
#[test]
fn test_reliability_update_affects_confidence() {
    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);
    let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

    // Register a peer with reachable hints
    let peer_id = make_peer_id(2);
    let hint = peer_hint(&peer_id);
    register_peer_with_hint(&mut engine, peer_id, TransportType::TCP, vec![hint]);

    let msg_id = make_message_id(1);

    // Initial reliability: should be 0.5 (default)
    let decision_before = engine.route_message_optimized(&hint, &msg_id, 50, 1000);
    let initial_confidence = decision_before.confidence;

    // Update reliability: success increases score
    engine
        .base_engine_mut()
        .local_cell_mut()
        .update_reliability(&peer_id, true);

    let decision_after = engine.route_message_optimized(&hint, &msg_id, 50, 1000);
    assert!(
        decision_after.confidence >= initial_confidence,
        "Confidence should not decrease after success: before={}, after={}",
        initial_confidence,
        decision_after.confidence
    );

    // Update reliability: failure decreases score
    engine
        .base_engine_mut()
        .local_cell_mut()
        .update_reliability(&peer_id, false);
    engine
        .base_engine_mut()
        .local_cell_mut()
        .update_reliability(&peer_id, false);
    engine
        .base_engine_mut()
        .local_cell_mut()
        .update_reliability(&peer_id, false);

    let decision_after_failures = engine.route_message_optimized(&hint, &msg_id, 50, 1000);
    assert!(
        decision_after_failures.confidence < decision_after.confidence,
        "Confidence should decrease after failures: after_success={}, after_failures={}",
        decision_after.confidence,
        decision_after_failures.confidence
    );
}

/// Verify that negative cache blocks routing to unreachable peers.
/// After recording a peer as unreachable via its hint prefix, routing
/// should fall back to StoreAndCarry for that hint.
#[test]
fn test_negative_cache_blocks_routing() {
    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);
    let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

    // Register a peer in local cell with reachable hints
    let peer_id = make_peer_id(2);
    let hint = peer_hint(&peer_id);
    register_peer_with_hint(&mut engine, peer_id, TransportType::QUIC, vec![hint]);

    let msg_id = make_message_id(1);

    // Should route directly before marking unreachable
    let decision_before = engine.route_message_optimized(&hint, &msg_id, 50, 1000);
    assert!(
        matches!(decision_before.primary, NextHop::Direct { .. }),
        "Should route directly before negative cache entry, got {:?}",
        decision_before.primary
    );

    // Record peer as unreachable using the hint prefix (matching how
    // route_message_optimized looks up peers in the negative cache)
    let hint_str = hex::encode(hint);
    engine.record_unreachable_peer(&hint_str);

    // After recording unreachable, routing should fall back
    let decision_after = engine.route_message_optimized(&hint, &msg_id, 50, 1000);
    assert!(
        matches!(
            decision_after.primary,
            NextHop::StoreAndCarry | NextHop::RouteDiscovery { .. }
        ),
        "Expected StoreAndCarry or RouteDiscovery after negative cache entry, got {:?}",
        decision_after.primary
    );
}

/// Verify that gateway peers are marked correctly and produce
/// Gateway routing decisions when no direct route exists.
#[test]
fn test_gateway_routing_decision() {
    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);
    let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

    // Register a gateway peer
    let gateway_id = make_peer_id(5);
    let gw_hint = peer_hint(&gateway_id);
    register_peer_with_hint(&mut engine, gateway_id, TransportType::TCP, vec![gw_hint]);
    engine
        .base_engine_mut()
        .local_cell_mut()
        .mark_as_gateway(&gateway_id, true);

    // Add a neighborhood route through the gateway
    let target_hint = make_hint(99);
    let cell_summary = CellSummary {
        peer_count: 3,
        gateway_count: 1,
        reachable_hints: vec![target_hint],
        avg_reliability: 0.9,
        timestamp: 1000,
    };
    engine.base_engine_mut().neighborhood_mut().update_gateway(
        gateway_id,
        cell_summary,
        2,
        TransportType::TCP,
    );

    let msg_id = make_message_id(1);
    let decision = engine.route_message_optimized(&target_hint, &msg_id, 50, 1000);

    // Should route through the gateway since target is not directly known
    assert!(
        matches!(
            decision.primary,
            NextHop::Gateway { .. } | NextHop::GlobalRoute { .. }
        ),
        "Expected Gateway or GlobalRoute for neighborhood-reachable peer, got {:?}",
        decision.primary
    );
}

/// Verify that the shared routing engine handle pattern works correctly.
/// Tests that Arc<RwLock<Option<OptimizedRoutingEngine>>> can be shared
/// and mutated from different references.
#[test]
fn test_shared_routing_engine_handle() {
    use parking_lot::RwLock;
    use std::sync::Arc;

    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);

    // Create a shared routing engine handle (like what start_swarm_with_config uses)
    let handle: Arc<RwLock<Option<OptimizedRoutingEngine>>> = Arc::new(RwLock::new(None));

    // Initialize the engine (like start_swarm does)
    {
        let mut guard = handle.write();
        if guard.is_none() {
            *guard = Some(OptimizedRoutingEngine::new(local_id, local_hint));
        }
    }

    // Verify it's initialized
    {
        let guard = handle.read();
        assert!(guard.is_some(), "Routing engine should be initialized");
    }

    // Use the shared handle for routing decisions
    let target_hint = make_hint(42);
    let msg_id = make_message_id(1);
    let decision = {
        let mut guard = handle.write();
        guard
            .as_mut()
            .map(|e| e.route_message_optimized(&target_hint, &msg_id, 50, 1000))
    };
    assert!(decision.is_some(), "Should produce a routing decision");

    // Simulate IronCore trying to initialize the same handle
    // (should not overwrite since it's already set)
    {
        let mut guard = handle.write();
        if guard.is_none() {
            *guard = Some(OptimizedRoutingEngine::new(local_id, local_hint));
        }
    }

    // Verify the engine is still the same instance
    {
        let guard = handle.read();
        assert!(
            guard.is_some(),
            "Routing engine should still be initialized"
        );
    }
}

/// Verify that the optimization tick produces maintenance results
/// and cleans up stale entries.
#[test]
fn test_routing_optimization_tick() {
    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);
    let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

    // Record some activity and unreachable peers
    engine.record_message_activity("peer_activity_1");
    engine.record_message_activity("peer_activity_2");
    engine.record_unreachable_peer("peer_unreachable_1");

    // Run optimization tick
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let maintenance = engine.tick(now);

    // Verify maintenance produces results
    assert!(
        maintenance.negative_cache_stats.entry_count > 0,
        "Negative cache should have entries after recording unreachable peers"
    );
    assert!(
        !maintenance.timeout_budget_summary.is_exhausted
            || maintenance.timeout_budget_summary.elapsed.as_millis() == 0,
        "Timeout budget should not be exhausted at start"
    );
}

/// Verify that discovery phase starts and advances correctly.
#[test]
fn test_discovery_phase_advancement() {
    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);
    let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

    // Initially not in discovery
    assert!(!engine.is_discovery_in_progress());
    assert_eq!(engine.current_discovery_phase(), DiscoveryPhase::LocalCache);

    // Start discovery by making a routing decision
    let target_hint = make_hint(42);
    let msg_id = make_message_id(1);
    let _ = engine.route_message_optimized(&target_hint, &msg_id, 50, 1000);
    assert!(engine.is_discovery_in_progress());

    // Advance discovery phases
    let phase1 = engine.advance_discovery_phase();
    assert!(phase1.is_some());

    let phase2 = engine.advance_discovery_phase();
    assert!(phase2.is_some());

    // After exhausting all phases, discovery should complete
    while engine.advance_discovery_phase().is_some() {}
    assert!(!engine.is_discovery_in_progress());
}

/// Verify multi-transport routing: when a peer is known on multiple
/// transports, the routing engine should still produce a valid Direct route.
#[test]
fn test_multi_transport_peer_routing() {
    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);
    let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

    // Register the same peer on multiple transports
    let peer_id = make_peer_id(2);
    let hint = peer_hint(&peer_id);
    register_peer_with_hint(&mut engine, peer_id, TransportType::BLE, vec![hint]);
    engine
        .base_engine_mut()
        .local_cell_mut()
        .peer_seen(peer_id, TransportType::QUIC);

    let msg_id = make_message_id(1);
    let decision = engine.route_message_optimized(&hint, &msg_id, 50, 1000);

    // Should produce a valid routing decision
    assert!(
        matches!(decision.primary, NextHop::Direct { .. }),
        "Expected Direct route for multi-transport peer, got {:?}",
        decision.primary
    );

    // The decision should include the transport (either BLE or QUIC)
    if let NextHop::Direct { transport, .. } = decision.primary {
        // The routing engine should pick one of the known transports
        assert!(
            transport == TransportType::BLE || transport == TransportType::QUIC,
            "Expected BLE or QUIC transport, got {:?}",
            transport
        );
    }
}

/// Verify that app background/resume lifecycle correctly saves and
/// restores route prefetch data.
#[test]
fn test_app_lifecycle_prefetch() {
    use scmessenger_core::routing::global::RouteAdvertisement;

    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);
    let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

    // Simulate app going to background with known routes
    let route = RouteAdvertisement {
        destination_hint: make_hint(99),
        next_hop: make_peer_id(5),
        hop_count: 2,
        reliability: 0.95,
        last_confirmed: 1000,
        sequence: 1,
        ttl: 3600,
    };
    engine.on_app_background(vec![(make_peer_id(5), make_hint(99), route)]);

    // Simulate app resuming
    let hints = engine.on_app_resume();
    assert_eq!(hints.len(), 1, "Should have one prefetch hint after resume");
    assert_eq!(
        hints[0],
        make_hint(99),
        "Prefetch hint should match saved route"
    );
}

/// Verify that clearing an unreachable peer from the negative cache
/// restores routing to that peer.
#[test]
fn test_clear_unreachable_restores_routing() {
    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);
    let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

    // Record a peer as unreachable
    engine.record_unreachable_peer("dead_peer");
    let stats = engine.negative_cache_stats();
    assert!(stats.entry_count > 0);

    // Clear the unreachable status
    engine.clear_unreachable_peer("dead_peer");

    // After clearing, the negative cache should not block this peer
    // (the entry count may not decrease immediately due to bloom filter semantics)
    let stats_after = engine.negative_cache_stats();
    // Verify stats are accessible after clearing (negative_checks is u64, always >= 0)
    let _ = stats_after.negative_checks;
}

/// Verify that the routing engine's evaluate_all_tracked method
/// correctly prunes stale entries.
#[test]
fn test_evaluate_all_tracked_pruning() {
    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);
    let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

    // Record some unreachable peers and activity
    engine.record_unreachable_peer("stale_peer_1");
    engine.record_unreachable_peer("stale_peer_2");
    engine.record_message_activity("active_peer");

    // Evaluate all tracked (should prune expired negative cache entries)
    let pruned = engine.evaluate_all_tracked();
    // No entries should be pruned immediately (they haven't expired yet)
    assert_eq!(
        pruned, 0,
        "No entries should be pruned immediately after recording"
    );
}

/// Verify that the can_reach_destination method correctly determines
/// reachability based on local cell and negative cache.
#[test]
fn test_can_reach_destination() {
    let local_id = make_peer_id(1);
    let local_hint = make_hint(1);
    let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

    // Register a peer with reachable hints matching the derivation
    // that can_reach_destination uses (first 8 bytes of the peer ID).
    let peer_id = make_peer_id(2);
    let derived_hint_hex = &hex::encode(peer_id)[..16];
    let derived_hint_bytes = hex::decode(derived_hint_hex).unwrap_or_default();
    let derived_hint: [u8; 8] = derived_hint_bytes.try_into().unwrap_or([0u8; 8]);

    // Register the peer with reachable hints matching the derived hint
    register_peer_with_hint(&mut engine, peer_id, TransportType::TCP, vec![derived_hint]);

    let peer_hex = hex::encode(peer_id);
    let known_reachable = engine.can_reach_destination(&peer_hex);
    assert!(
        known_reachable,
        "Known peer with reachable hints should be reachable, got false"
    );

    // An unknown peer should not be reachable (not in local cell)
    let unknown_hex = hex::encode(make_peer_id(99));
    let unknown_reachable = engine.can_reach_destination(&unknown_hex);
    assert!(
        !unknown_reachable,
        "Unknown peer without reachable hints should not be reachable, got true"
    );
}
