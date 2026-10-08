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

/// Read-only scheduler view for the node indicators (never an absence claim).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct DiscoverySnapshot {
    /// `true` right after a reset, before the first decayed re-arm.
    pub aggressive: bool,
    /// Current un-jittered interval in milliseconds.
    pub interval_ms: u64,
    /// Ceiling computed from the live inputs, in milliseconds.
    pub ceiling_ms: u64,
    /// Attempts since the last reset.
    pub attempts: u32,
    /// Total resets since creation.
    pub resets: u64,
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

    /// A ledger exchange delivered `new_entries` entries (only > 0 matters).
    pub fn on_ledger_received(&self, new_entries: u32) -> Vec<DiscoveryTransport> {
        let ev = NetworkEvent::LedgerReceived { new_entries };
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

    /// Record a proven-stable connection (clears flap-damping history).
    pub fn record_success(&self, transport: DiscoveryTransport) {
        if let Some(s) = self.scheduler(transport) {
            s.record_success();
        }
    }

    /// Current scheduler state for the node indicators.
    pub fn snapshot(&self, transport: DiscoveryTransport) -> DiscoverySnapshot {
        match self.scheduler(transport) {
            Some(s) => {
                let snap = s.snapshot();
                DiscoverySnapshot {
                    aggressive: snap.phase == crate::transport::Phase::Aggressive,
                    interval_ms: snap.interval_ms,
                    ceiling_ms: snap.ceiling_ms,
                    attempts: snap.attempts,
                    resets: snap.resets,
                }
            }
            None => DiscoverySnapshot {
                aggressive: true,
                interval_ms: SchedulerFallback::DELAY_MS,
                ceiling_ms: SchedulerFallback::DELAY_MS,
                attempts: 0,
                resets: 0,
            },
        }
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
    use crate::transport::{SchedulerClock, SchedulerConfig};
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
        assert!(c.snapshot(DiscoveryTransport::Ble).aggressive);
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
        let snap = c.snapshot(DiscoveryTransport::Ble);
        assert!(!snap.aggressive);
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
        let decayed = c.snapshot(DiscoveryTransport::Ble).interval_ms;
        clock.0.fetch_add(10 * 60 * 1000, Ordering::SeqCst);
        let reset = c.on_event(DiscoveryEvent::BleOn);
        assert!(reset.contains(&DiscoveryTransport::Ble));
        let snap = c.snapshot(DiscoveryTransport::Ble);
        assert!(snap.aggressive);
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
    fn ledger_received_with_zero_entries_resets_nothing_for_ledger() {
        let clock = Arc::new(FakeClock(AtomicU64::new(1_000_000)));
        let c = coordinator(clock);
        assert!(!c
            .on_ledger_received(0)
            .contains(&DiscoveryTransport::Ledger));
    }

    #[test]
    fn set_inputs_changes_ceiling() {
        let clock = Arc::new(FakeClock(AtomicU64::new(1_000_000)));
        let c = coordinator(clock);
        c.set_inputs(0, DiscoveryPower::Charging, true);
        let low = c.snapshot(DiscoveryTransport::Lan).ceiling_ms;
        c.set_inputs(12, DiscoveryPower::Low, false);
        let high = c.snapshot(DiscoveryTransport::Lan).ceiling_ms;
        assert!(high > low);
    }
}
