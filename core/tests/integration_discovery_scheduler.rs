// Integration test: event-driven discovery scheduler (issue #469, T5).
//
// Exercises the public API only (no access to scheduler internals): a
// per-transport set of schedulers fed one shared event stream, with an
// injected clock and RNG.

use scmessenger_core::transport::{
    DiscoveryInputs, DiscoveryScheduler, JitterSource, NetworkEvent, Phase, PowerState,
    SchedulerClock, SchedulerConfig, TransportClass,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct TestClock(AtomicU64);
impl SchedulerClock for TestClock {
    fn now_ms(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

struct Lcg(AtomicU64);
impl JitterSource for Lcg {
    fn next_unit(&self) -> f64 {
        let next = self
            .0
            .load(Ordering::SeqCst)
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0.store(next, Ordering::SeqCst);
        (next >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn build_all() -> Vec<DiscoveryScheduler> {
    let clock = Arc::new(TestClock(AtomicU64::new(0)));
    TransportClass::ALL
        .iter()
        .map(|t| {
            DiscoveryScheduler::new(
                *t,
                SchedulerConfig::for_transport(*t),
                clock.clone(),
                Arc::new(Lcg(AtomicU64::new(7))),
            )
        })
        .collect()
}

fn dispatch(all: &[DiscoveryScheduler], ev: NetworkEvent) -> Vec<TransportClass> {
    all.iter()
        .filter(|s| s.on_event(ev))
        .map(|s| s.transport())
        .collect()
}

#[test]
fn integration_discovery_scheduler_ble_on_resets_only_ble() {
    let all = build_all();
    for s in &all {
        for _ in 0..12 {
            s.next_delay();
        }
        assert_eq!(s.snapshot().phase, Phase::Decay);
    }
    let reset = dispatch(&all, NetworkEvent::BleStateChanged { on: true });
    assert_eq!(reset, vec![TransportClass::Ble]);
    for s in &all {
        let snap = s.snapshot();
        if s.transport() == TransportClass::Ble {
            assert_eq!(snap.phase, Phase::Aggressive);
            assert_eq!(snap.interval_ms, 1_000);
        } else {
            assert_eq!(
                snap.phase,
                Phase::Decay,
                "{:?} must keep decaying",
                snap.transport
            );
        }
    }
}

#[test]
fn integration_discovery_scheduler_network_change_then_quiet_backoff_never_stops() {
    let all = build_all();
    let internet = all
        .iter()
        .find(|s| s.transport() == TransportClass::Internet)
        .expect("internet scheduler built");
    internet.set_inputs(DiscoveryInputs {
        connected_peers: 0,
        power: PowerState::Charging,
        foreground: true,
    });
    assert!(dispatch(&all, NetworkEvent::CellularChanged).contains(&TransportClass::Internet));

    let mut previous_base = 0;
    for _ in 0..1_000 {
        let before = internet.snapshot().interval_ms;
        assert!(before >= previous_base);
        previous_base = before;
        let d = internet.next_delay();
        assert!(d > Duration::ZERO);
        assert!(d.as_millis() as u64 <= internet.ceiling_ms());
    }
    // A later network change restarts aggressive probing immediately.
    assert!(internet.on_event(NetworkEvent::WifiChanged));
    assert_eq!(internet.snapshot().interval_ms, 500);
}
