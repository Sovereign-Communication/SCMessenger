//! Mobile FFI surface for the event-driven discovery scheduler (#469 T7/T8).
//!
//! Platform shells (Android, iOS) own the radios and the timers; the
//! scheduler in `transport::discovery_scheduler` owns the cadence policy.
//! This module is the thin bridge between them: a shell reports a
//! [`DiscoveryEvent`] (Bluetooth turned on, Wi-Fi changed, app foregrounded,
//! invite redeemed, ...) and asks [`DiscoveryCoordinator::next_delay_ms`] how
//! long to wait before the next scan or dial sweep of a given transport.
//!
//! Nothing here holds a literal interval: floors, decay and ceilings all come
//! from `DiscoveryScheduler`. There is no give-up state.

use std::sync::Arc;

use crate::transport::{
    DiscoveryInputs, DiscoveryScheduler, NetworkEvent, PowerState, TransportClass,
};

/// Transport whose discovery cadence is scheduled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum DiscoveryTransport {
    /// BLE scan / advertise restarts.
    Ble,
    /// mDNS / LAN discovery.
    Lan,
    /// Wi-Fi Direct / Wi-Fi Aware / Multipeer discovery.
    WifiDirect,
    /// Internet / cellular dial of ledger candidates.
    Ledger,
}

impl From<DiscoveryTransport> for TransportClass {
    fn from(t: DiscoveryTransport) -> Self {
        match t {
            DiscoveryTransport::Ble => TransportClass::Ble,
            DiscoveryTransport::Lan => TransportClass::Lan,
            DiscoveryTransport::WifiDirect => TransportClass::WifiDirect,
            DiscoveryTransport::Ledger => TransportClass::Internet,
        }
    }
}

impl From<TransportClass> for DiscoveryTransport {
    fn from(t: TransportClass) -> Self {
        match t {
            TransportClass::Ble => DiscoveryTransport::Ble,
            TransportClass::Lan => DiscoveryTransport::Lan,
            TransportClass::WifiDirect => DiscoveryTransport::WifiDirect,
            TransportClass::Internet => DiscoveryTransport::Ledger,
        }
    }
}

/// Platform event that may change what discovery should do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum DiscoveryEvent {
    /// Bluetooth adapter switched on (resets BLE to aggressive).
    BleOn,
    /// Bluetooth adapter switched off (recorded, resets nothing).
    BleOff,
    /// Wi-Fi interface or network changed (up, down, different network).
    WifiChanged,
    /// Cellular interface or network changed.
    CellularChanged,
    /// A local interface address appeared or disappeared.
    LanInterfaceChanged,
    /// App returned to the foreground.
    AppForeground,
    /// An invite was redeemed and the ledger gained seed entries.
    InviteRedeemed,
    /// A connection added a peer not seen before.
    NewPeerConnected,
    /// The connected peer count dropped to zero.
    AllPeersLost,
}

impl From<DiscoveryEvent> for NetworkEvent {
    fn from(e: DiscoveryEvent) -> Self {
        match e {
            DiscoveryEvent::BleOn => NetworkEvent::BleStateChanged { on: true },
            DiscoveryEvent::BleOff => NetworkEvent::BleStateChanged { on: false },
            DiscoveryEvent::WifiChanged => NetworkEvent::WifiChanged,
            DiscoveryEvent::CellularChanged => NetworkEvent::CellularChanged,
            DiscoveryEvent::LanInterfaceChanged => NetworkEvent::LanInterfaceChanged,
            DiscoveryEvent::AppForeground => NetworkEvent::AppForeground,
            DiscoveryEvent::InviteRedeemed => NetworkEvent::InviteRedeemed,
            DiscoveryEvent::NewPeerConnected => NetworkEvent::NewPeerConnected,
            DiscoveryEvent::AllPeersLost => NetworkEvent::AllPeersLost,
        }
    }
}

/// Device power state as reported by the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum DiscoveryPower {
    Charging,
    Normal,
    Low,
}

impl From<DiscoveryPower> for PowerState {
    fn from(p: DiscoveryPower) -> Self {
        match p {
            DiscoveryPower::Charging => PowerState::Charging,
            DiscoveryPower::Normal => PowerState::Normal,
            DiscoveryPower::Low => PowerState::Low,
        }
    }
}

/// One scheduler per transport class, driven by platform events.
#[derive(uniffi::Object)]
pub struct DiscoveryCoordinator {
    schedulers: Vec<(TransportClass, Arc<DiscoveryScheduler>)>,
}

impl Default for DiscoveryCoordinator {
    fn default() -> Self {
        Self::with_schedulers(
            TransportClass::ALL
                .iter()
                .map(|t| (*t, Arc::new(DiscoveryScheduler::with_system_time(*t))))
                .collect(),
        )
    }
}

impl DiscoveryCoordinator {
    /// Build from explicit schedulers (tests inject a fake clock this way).
    pub fn with_schedulers(schedulers: Vec<(TransportClass, Arc<DiscoveryScheduler>)>) -> Self {
        Self { schedulers }
    }

    fn scheduler(&self, transport: DiscoveryTransport) -> Option<&Arc<DiscoveryScheduler>> {
        let class: TransportClass = transport.into();
        self.schedulers
            .iter()
            .find(|(t, _)| *t == class)
            .map(|(_, s)| s)
    }
}

#[uniffi::export]
impl DiscoveryCoordinator {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Feed a platform event to every scheduler. Returns the transports that
    /// were reset to aggressive: for each, the caller must cancel its pending
    /// timer and run an attempt immediately.
    pub fn on_event(&self, event: DiscoveryEvent) -> Vec<DiscoveryTransport> {
        let ev: NetworkEvent = event.into();
        self.schedulers
            .iter()
            .filter(|(_, s)| s.on_event(ev))
            .map(|(t, _)| DiscoveryTransport::from(*t))
            .collect()
    }

    /// Replace the observed conditions that drive every scheduler's ceiling.
    pub fn set_inputs(&self, connected_peers: u32, power: DiscoveryPower, foreground: bool) {
        let inputs = DiscoveryInputs {
            connected_peers,
            power: power.into(),
            foreground,
        };
        for (_, s) in &self.schedulers {
            s.set_inputs(inputs);
        }
    }

    /// Milliseconds to wait before the next attempt on `transport`, then
    /// advances the decay. Always positive; there is no give-up signal.
    pub fn next_delay_ms(&self, transport: DiscoveryTransport) -> u64 {
        match self.scheduler(transport) {
            Some(s) => u64::try_from(s.next_delay().as_millis())
                .unwrap_or(u64::MAX)
                .max(1),
            None => SchedulerFallback::DELAY_MS,
        }
    }

    /// Milliseconds until a flap-damped (coalesced) reset falls due, if any.
    /// A caller on a long timer must also wake then.
    pub fn pending_reset_due_ms(&self, transport: DiscoveryTransport) -> Option<u64> {
        self.scheduler(transport)
            .and_then(|s| s.pending_reset_due_in())
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
    }

    /// Apply the coalesced reset now; `true` means attempt immediately.
    pub fn apply_pending_reset(&self, transport: DiscoveryTransport) -> bool {
        self.scheduler(transport)
            .is_some_and(|s| s.apply_pending_reset())
    }
}

/// Unreachable in practice (the coordinator always builds all four classes);
/// keeps the FFI total instead of panicking.
struct SchedulerFallback;

impl SchedulerFallback {
    const DELAY_MS: u64 = 1_000;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{Phase, SchedulerClock, SchedulerConfig, SchedulerSnapshot};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct FakeClock(AtomicU64);
    impl SchedulerClock for FakeClock {
        fn now_ms(&self) -> u64 {
            self.0.load(Ordering::SeqCst)
        }
    }

    struct MidJitter;
    impl crate::transport::JitterSource for MidJitter {
        fn next_unit(&self) -> f64 {
            1.0
        }
    }

    /// Test-only view of a transport's scheduler state (no platform needs it).
    fn snapshot(c: &DiscoveryCoordinator, t: DiscoveryTransport) -> SchedulerSnapshot {
        c.scheduler(t)
            .map(|s| s.snapshot())
            .expect("coordinator builds all four classes")
    }

    fn is_aggressive(c: &DiscoveryCoordinator, t: DiscoveryTransport) -> bool {
        snapshot(c, t).phase == Phase::Aggressive
    }

    fn coordinator(clock: Arc<FakeClock>) -> DiscoveryCoordinator {
        DiscoveryCoordinator::with_schedulers(
            TransportClass::ALL
                .iter()
                .map(|t| {
                    (
                        *t,
                        Arc::new(DiscoveryScheduler::new(
                            *t,
                            SchedulerConfig::for_transport(*t),
                            clock.clone(),
                            Arc::new(MidJitter),
                        )),
                    )
                })
                .collect(),
        )
    }

    #[test]
    fn ble_on_resets_only_ble_to_aggressive() {
        let clock = Arc::new(FakeClock(AtomicU64::new(1_000_000)));
        let c = coordinator(clock);
        let reset = c.on_event(DiscoveryEvent::BleOn);
        assert!(reset.contains(&DiscoveryTransport::Ble));
        assert!(!reset.contains(&DiscoveryTransport::Ledger));
        assert!(is_aggressive(&c, DiscoveryTransport::Ble));
    }

    #[test]
    fn ble_off_resets_nothing() {
        let clock = Arc::new(FakeClock(AtomicU64::new(1_000_000)));
        let c = coordinator(clock);
        assert!(c.on_event(DiscoveryEvent::BleOff).is_empty());
    }

    #[test]
    fn delay_decays_after_an_event_and_never_hits_zero() {
        let clock = Arc::new(FakeClock(AtomicU64::new(1_000_000)));
        let c = coordinator(clock);
        c.on_event(DiscoveryEvent::BleOn);
        let first = c.next_delay_ms(DiscoveryTransport::Ble);
        let mut last = first;
        for _ in 0..12 {
            last = c.next_delay_ms(DiscoveryTransport::Ble);
            assert!(last >= 1);
        }
        assert!(last >= first, "decay is monotone with jitter pinned high");
        let snap = snapshot(&c, DiscoveryTransport::Ble);
        assert!(snap.phase != Phase::Aggressive);
        assert!(snap.interval_ms <= snap.ceiling_ms);
    }

    #[test]
    fn event_after_quiet_period_returns_to_floor() {
        let clock = Arc::new(FakeClock(AtomicU64::new(1_000_000)));
        let c = coordinator(clock.clone());
        c.on_event(DiscoveryEvent::BleOn);
        for _ in 0..10 {
            c.next_delay_ms(DiscoveryTransport::Ble);
        }
        let decayed = snapshot(&c, DiscoveryTransport::Ble).interval_ms;
        clock.0.fetch_add(10 * 60 * 1000, Ordering::SeqCst);
        let reset = c.on_event(DiscoveryEvent::BleOn);
        assert!(reset.contains(&DiscoveryTransport::Ble));
        let snap = snapshot(&c, DiscoveryTransport::Ble);
        assert!(snap.phase == Phase::Aggressive);
        assert!(snap.interval_ms < decayed);
    }

    #[test]
    fn invite_redeemed_resets_ledger_transport() {
        let clock = Arc::new(FakeClock(AtomicU64::new(1_000_000)));
        let c = coordinator(clock);
        let reset = c.on_event(DiscoveryEvent::InviteRedeemed);
        assert!(reset.contains(&DiscoveryTransport::Ledger));
    }

    #[test]
    fn set_inputs_changes_ceiling() {
        let clock = Arc::new(FakeClock(AtomicU64::new(1_000_000)));
        let c = coordinator(clock);
        c.set_inputs(0, DiscoveryPower::Charging, true);
        let low = snapshot(&c, DiscoveryTransport::Lan).ceiling_ms;
        c.set_inputs(12, DiscoveryPower::Low, false);
        let high = snapshot(&c, DiscoveryTransport::Lan).ceiling_ms;
        assert!(high > low);
    }

    #[test]
    fn dropping_the_coordinator_releases_every_scheduler() {
        // Lifecycle pairing (#469): the coordinator spawns no task and owns no
        // OS handle, so dropping the last reference must free its schedulers.
        // A leaked clone would keep the strong count above one.
        let clock = Arc::new(FakeClock(AtomicU64::new(1_000_000)));
        let held: Vec<Arc<DiscoveryScheduler>> = TransportClass::ALL
            .iter()
            .map(|t| {
                Arc::new(DiscoveryScheduler::new(
                    *t,
                    SchedulerConfig::for_transport(*t),
                    clock.clone(),
                    Arc::new(MidJitter),
                ))
            })
            .collect();
        let c = DiscoveryCoordinator::with_schedulers(
            TransportClass::ALL
                .iter()
                .copied()
                .zip(held.iter().cloned())
                .collect(),
        );
        assert!(held.iter().all(|s| Arc::strong_count(s) == 2));
        drop(c);
        assert!(held.iter().all(|s| Arc::strong_count(s) == 1));
    }

    #[test]
    fn events_after_many_cycles_keep_delays_positive() {
        // Repeated start/stop style churn (event, drain delays, event) never
        // yields a zero delay and never panics: no give-up state accumulates.
        let clock = Arc::new(FakeClock(AtomicU64::new(1_000_000)));
        let c = coordinator(clock.clone());
        for _ in 0..20 {
            c.on_event(DiscoveryEvent::WifiChanged);
            for _ in 0..5 {
                assert!(c.next_delay_ms(DiscoveryTransport::Lan) >= 1);
            }
            clock.0.fetch_add(60_000, Ordering::SeqCst);
        }
    }

    // ---- #469 T7/T8 behaviour tests: events reach the scheduler and cadence
    // ---- is driven by events only (no static timers).

    fn floor_ms(t: DiscoveryTransport) -> u64 {
        SchedulerConfig::for_transport(t.into()).floor_ms
    }

    const EVERY_TRANSPORT: [DiscoveryTransport; 4] = [
        DiscoveryTransport::Ble,
        DiscoveryTransport::Lan,
        DiscoveryTransport::WifiDirect,
        DiscoveryTransport::Ledger,
    ];

    fn decay_every_transport(c: &DiscoveryCoordinator) {
        for t in EVERY_TRANSPORT {
            for _ in 0..4 {
                c.next_delay_ms(t);
            }
            assert!(snapshot(c, t).interval_ms > floor_ms(t));
        }
    }

    #[test]
    fn new_peer_connected_resets_every_transport_to_its_floor() {
        let c = coordinator(Arc::new(FakeClock(AtomicU64::new(1_000_000))));
        decay_every_transport(&c);
        let reset = c.on_event(DiscoveryEvent::NewPeerConnected);
        for t in EVERY_TRANSPORT {
            assert!(reset.contains(&t), "{:?} not reset", t);
            let snap = snapshot(&c, t);
            assert_eq!(snap.phase, Phase::Aggressive);
            assert_eq!(snap.interval_ms, floor_ms(t));
        }
    }

    #[test]
    fn all_peers_lost_resets_every_transport_to_its_floor() {
        let c = coordinator(Arc::new(FakeClock(AtomicU64::new(1_000_000))));
        decay_every_transport(&c);
        let reset = c.on_event(DiscoveryEvent::AllPeersLost);
        for t in EVERY_TRANSPORT {
            assert!(reset.contains(&t), "{:?} not reset", t);
            assert_eq!(snapshot(&c, t).interval_ms, floor_ms(t));
        }
    }

    #[test]
    fn app_foreground_resets_every_transport() {
        let c = coordinator(Arc::new(FakeClock(AtomicU64::new(1_000_000))));
        decay_every_transport(&c);
        let reset = c.on_event(DiscoveryEvent::AppForeground);
        assert_eq!(reset.len(), EVERY_TRANSPORT.len());
    }

    #[test]
    fn wifi_changed_resets_lan_wifi_direct_and_ledger_but_not_ble() {
        let c = coordinator(Arc::new(FakeClock(AtomicU64::new(1_000_000))));
        decay_every_transport(&c);
        let reset = c.on_event(DiscoveryEvent::WifiChanged);
        assert!(!reset.contains(&DiscoveryTransport::Ble));
        for t in [
            DiscoveryTransport::Lan,
            DiscoveryTransport::WifiDirect,
            DiscoveryTransport::Ledger,
        ] {
            assert!(reset.contains(&t), "{:?} not reset", t);
            assert_eq!(snapshot(&c, t).interval_ms, floor_ms(t));
        }
        assert!(
            snapshot(&c, DiscoveryTransport::Ble).interval_ms > floor_ms(DiscoveryTransport::Ble)
        );
    }

    #[test]
    fn cellular_changed_resets_only_the_ledger_transport() {
        let c = coordinator(Arc::new(FakeClock(AtomicU64::new(1_000_000))));
        let reset = c.on_event(DiscoveryEvent::CellularChanged);
        assert_eq!(reset, vec![DiscoveryTransport::Ledger]);
    }

    #[test]
    fn lan_interface_changed_resets_lan_wifi_direct_and_ledger() {
        let c = coordinator(Arc::new(FakeClock(AtomicU64::new(1_000_000))));
        let reset = c.on_event(DiscoveryEvent::LanInterfaceChanged);
        assert_eq!(reset.len(), 3);
        assert!(!reset.contains(&DiscoveryTransport::Ble));
    }

    #[test]
    fn invite_redeemed_after_decay_returns_the_ledger_to_floor_and_leaves_ble_decayed() {
        let c = coordinator(Arc::new(FakeClock(AtomicU64::new(1_000_000))));
        decay_every_transport(&c);
        let reset = c.on_event(DiscoveryEvent::InviteRedeemed);
        assert_eq!(reset, vec![DiscoveryTransport::Ledger]);
        assert_eq!(
            snapshot(&c, DiscoveryTransport::Ledger).interval_ms,
            floor_ms(DiscoveryTransport::Ledger)
        );
        assert_eq!(snapshot(&c, DiscoveryTransport::Ble).phase, Phase::Decay);
    }

    #[test]
    fn first_delay_after_ble_on_is_the_floor_then_widens_without_passing_the_ceiling() {
        let c = coordinator(Arc::new(FakeClock(AtomicU64::new(1_000_000))));
        c.on_event(DiscoveryEvent::BleOn);
        let ceiling = snapshot(&c, DiscoveryTransport::Ble).ceiling_ms;
        let delays: Vec<u64> = (0..12)
            .map(|_| c.next_delay_ms(DiscoveryTransport::Ble))
            .collect();
        assert_eq!(delays[0], floor_ms(DiscoveryTransport::Ble));
        assert!(delays[1] > delays[0], "interval must widen after the floor");
        assert!(delays.windows(2).all(|w| w[1] >= w[0]));
        assert!(delays.iter().all(|d| *d <= ceiling));
        assert_eq!(*delays.last().unwrap_or(&0), ceiling);
    }

    #[test]
    fn elapsed_time_alone_never_resets_the_schedule() {
        // Backoff is event-driven only: a quiet hour must not return the
        // scheduler to aggressive, queue a reset, or change the reset count.
        let clock = Arc::new(FakeClock(AtomicU64::new(1_000_000)));
        let c = coordinator(clock.clone());
        c.on_event(DiscoveryEvent::BleOn);
        for _ in 0..6 {
            c.next_delay_ms(DiscoveryTransport::Ble);
        }
        let before = snapshot(&c, DiscoveryTransport::Ble);
        clock.0.fetch_add(60 * 60 * 1000, Ordering::SeqCst);
        let delay = c.next_delay_ms(DiscoveryTransport::Ble);
        let after = snapshot(&c, DiscoveryTransport::Ble);
        assert_eq!(after.resets, before.resets);
        assert_eq!(after.phase, Phase::Decay);
        assert!(delay > floor_ms(DiscoveryTransport::Ble));
        assert_eq!(c.pending_reset_due_ms(DiscoveryTransport::Ble), None);
        assert!(!c.apply_pending_reset(DiscoveryTransport::Ble));
    }

    #[test]
    fn set_inputs_moves_the_ceiling_but_never_resets_the_schedule() {
        let c = coordinator(Arc::new(FakeClock(AtomicU64::new(1_000_000))));
        c.on_event(DiscoveryEvent::BleOn);
        c.next_delay_ms(DiscoveryTransport::Ble);
        let before = snapshot(&c, DiscoveryTransport::Ble);
        c.set_inputs(8, DiscoveryPower::Low, false);
        let after = snapshot(&c, DiscoveryTransport::Ble);
        assert!(after.ceiling_ms > before.ceiling_ms);
        assert_eq!(after.resets, before.resets);
        assert_eq!(after.attempts, before.attempts);
        assert_eq!(after.interval_ms, before.interval_ms);
    }

    #[test]
    fn a_flap_inside_the_window_is_deferred_and_applied_when_the_window_ends() {
        let clock = Arc::new(FakeClock(AtomicU64::new(1_000_000)));
        let c = coordinator(clock.clone());
        assert_eq!(
            c.on_event(DiscoveryEvent::BleOn),
            vec![DiscoveryTransport::Ble]
        );
        for _ in 0..3 {
            c.next_delay_ms(DiscoveryTransport::Ble);
        }
        clock.0.fetch_add(100, Ordering::SeqCst);
        assert!(c.on_event(DiscoveryEvent::BleOn).is_empty());
        let floor = floor_ms(DiscoveryTransport::Ble);
        assert_eq!(
            c.pending_reset_due_ms(DiscoveryTransport::Ble),
            Some(floor - 100)
        );
        // The long decayed wait is cut short to the deferred reset.
        assert_eq!(c.next_delay_ms(DiscoveryTransport::Ble), floor - 100);
        clock.0.fetch_add(floor, Ordering::SeqCst);
        assert!(c.apply_pending_reset(DiscoveryTransport::Ble));
        assert!(is_aggressive(&c, DiscoveryTransport::Ble));
        assert!(!c.apply_pending_reset(DiscoveryTransport::Ble));
        assert_eq!(c.pending_reset_due_ms(DiscoveryTransport::Ble), None);
    }

    #[test]
    fn every_discovery_event_maps_to_its_network_event() {
        let table = [
            (DiscoveryEvent::BleOn, "BleStateChanged"),
            (DiscoveryEvent::BleOff, "BleStateChanged"),
            (DiscoveryEvent::WifiChanged, "WifiChanged"),
            (DiscoveryEvent::CellularChanged, "CellularChanged"),
            (DiscoveryEvent::LanInterfaceChanged, "LanInterfaceChanged"),
            (DiscoveryEvent::AppForeground, "AppForeground"),
            (DiscoveryEvent::InviteRedeemed, "InviteRedeemed"),
            (DiscoveryEvent::NewPeerConnected, "NewPeerConnected"),
            (DiscoveryEvent::AllPeersLost, "AllPeersLost"),
        ];
        for (event, kind) in table {
            assert_eq!(NetworkEvent::from(event).kind(), kind);
        }
        assert!(matches!(
            NetworkEvent::from(DiscoveryEvent::BleOn),
            NetworkEvent::BleStateChanged { on: true }
        ));
        assert!(matches!(
            NetworkEvent::from(DiscoveryEvent::BleOff),
            NetworkEvent::BleStateChanged { on: false }
        ));
    }

    #[test]
    fn transports_and_power_levels_map_to_the_scheduler_types() {
        assert_eq!(
            TransportClass::from(DiscoveryTransport::Ledger),
            TransportClass::Internet
        );
        for t in EVERY_TRANSPORT {
            assert_eq!(DiscoveryTransport::from(TransportClass::from(t)), t);
        }
        assert_eq!(
            PowerState::from(DiscoveryPower::Charging),
            PowerState::Charging
        );
        assert_eq!(PowerState::from(DiscoveryPower::Normal), PowerState::Normal);
        assert_eq!(PowerState::from(DiscoveryPower::Low), PowerState::Low);
    }

    #[test]
    fn a_coordinator_without_a_scheduler_still_returns_a_positive_delay_and_no_reset() {
        let c = DiscoveryCoordinator::with_schedulers(Vec::new());
        assert!(c.next_delay_ms(DiscoveryTransport::Ble) >= 1);
        assert!(c.on_event(DiscoveryEvent::BleOn).is_empty());
        assert_eq!(c.pending_reset_due_ms(DiscoveryTransport::Ble), None);
        assert!(!c.apply_pending_reset(DiscoveryTransport::Ble));
        c.set_inputs(3, DiscoveryPower::Normal, true);
    }
}
