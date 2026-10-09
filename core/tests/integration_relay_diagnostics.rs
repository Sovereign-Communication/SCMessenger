//! Integration tests for IronCore's relay-diagnostics methods
//! (get_all_relay_stats / get_fallback_relays / can_bootstrap_others /
//! get_healthy_relays), which used to be hardcoded stubs due to a type
//! mismatch between `iron_core`'s stored `relay::BootstrapManager` (the
//! QR-code/invite workflow) and `transport::bootstrap::BootstrapManager`
//! (relay health/circuit-breaker/fallback-relay tracking). IronCore now
//! holds a `transport::bootstrap::BootstrapManager` directly and these
//! methods delegate to it.

use libp2p::{Multiaddr, PeerId};
use scmessenger_core::transport::relay_health::RelayMetrics;
use scmessenger_core::IronCore;

fn make_core() -> IronCore {
    let dir = tempfile::tempdir().unwrap();
    IronCore::with_storage(dir.path().to_str().unwrap().to_string())
}

/// Fallback relay addresses are static/env-derived and must be available
/// immediately, without any live swarm wiring.
#[test]
fn get_fallback_relays_returns_empty_by_default() {
    let core = make_core();
    let relays = core.get_fallback_relays();
    assert!(
        relays.is_empty(),
        "fallback relays must be empty by default without hardcoded nodes"
    );
}

/// Health/stats start empty (no dial attempts have happened yet), and
/// become non-empty once fed through `relay_bootstrap_manager_handle()` —
/// proving these methods report on live state rather than a hardcoded
/// stub that could never change.
#[test]
fn relay_stats_and_health_reflect_recorded_events() {
    let core = make_core();

    assert!(core.get_all_relay_stats().is_empty());
    assert!(core.get_healthy_relays().is_empty());

    let addr: Multiaddr = "/ip4/1.2.3.4/tcp/9001".parse().unwrap();
    let peer_id = PeerId::random();

    {
        let handle = core.relay_bootstrap_manager_handle();
        let mut guard = handle.write();
        let mgr = guard
            .as_mut()
            .expect("bootstrap manager must be initialized by default");
        // `record_success` only updates metrics for already-known relays;
        // a relay must first be registered (e.g. on discovery) before dial
        // results can be recorded against it.
        mgr.relay_discovery_mut()
            .update_relay_metrics(RelayMetrics {
                peer_id,
                addresses: vec![addr.clone()],
                uptime_ratio: 0.5,
                avg_latency_ms: 100,
                bandwidth_estimate: 0,
                recent_connections: 0,
                recent_failures: 0,
                last_seen: 0,
                region: None,
                stability_score: 0.5,
            });
        mgr.relay_discovery_mut().record_success(&peer_id, 42);
        mgr.circuit_breaker().record_success(&addr.to_string());
    }

    let stats = core.get_all_relay_stats();
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].0, peer_id);

    let healthy = core.get_healthy_relays();
    assert_eq!(healthy, vec![addr.to_string()]);
}

/// `can_bootstrap_others` requires the core to be running with an
/// initialized identity.
#[test]
fn can_bootstrap_others_requires_running_identity() {
    let core = make_core();
    assert!(!core.can_bootstrap_others());

    core.grant_consent();
    core.initialize_identity().unwrap();
    assert!(
        !core.can_bootstrap_others(),
        "must not report bootstrap-capable before start()"
    );

    core.start().unwrap();
    assert!(core.can_bootstrap_others());
}
