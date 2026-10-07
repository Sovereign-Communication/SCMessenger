// Event-driven discovery scheduler (issue #469, tasks T5/T6).
//
// Spec: docs/BOOTSTRAP_RENDEZVOUS_DECISION_469.md section 6.
//
// One `DiscoveryScheduler` exists per transport class (BLE, LAN, Wi-Fi
// Direct, internet/ledger dial). It decides *when to sweep*; the per-key
// `DialPolicyManager` still decides *which candidate is eligible*.
//
// Behaviour:
// - An affecting `NetworkEvent` resets the transport to aggressive: the
//   interval returns to the transport floor, `attempts` returns to zero, and
//   `on_event` returns `true` so the caller fires an attempt immediately
//   (the caller cancels and re-arms its pending timer).
// - Without events the interval decays: `interval' = ceil(interval * g)` with
//   `g` in [1.5, 2.0], then full jitter `uniform(0.5, 1.0) * interval` so a
//   fleet of nodes does not synchronise.
// - There is NO give-up state and NO literal ceiling. The ceiling is computed
//   from live inputs on every call:
//       ceiling = max(floor, base_ceiling * density_factor * power_factor)
//       density_factor = 1 + 0.25 * min(connected_peers, 16)        in [1, 5]
//       power_factor   = power(Charging 1.0 | Normal 1.5 | Low 3.0)
//                        * (foreground 1.0 | background 2.0)         in [1, 6]
//   With zero peers on a charging, foreground device the ceiling is exactly
//   `base_ceiling`, so the node keeps probing at a short steady interval
//   rather than going quiet. `base_ceiling` is per-transport configuration.
//
// Reset coalescing (Rule-8 review of #484, finding F1): a flapping peer or
// interface must not pin the node at the floor. Resets are rate-limited by a
// window derived from the floor: after `k` accepted resets without a proven
// connection (`record_success`), the next reset is accepted only once
// `floor * 2^(k-1)` ms (capped at the live ceiling) have passed since the
// previous one. An event inside the window is not dropped: it is remembered
// and applied when the window ends (`next_delay` caps its wait to that moment),
// so a real change is never lost, only delayed by at most the window. A quiet
// period of at least one ceiling, or `record_success`, restores immediate
// resets. There is still no give-up and no literal cap on the window.
//
// The scheduler is pure logic: time comes from an injected `SchedulerClock`
// and randomness from an injected `JitterSource`, so tests are deterministic.

use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

/// Minimum and maximum decay growth factor (spec section 6).
pub const GROWTH_MIN: f64 = 1.5;
pub const GROWTH_MAX: f64 = 2.0;
const GROWTH_DEFAULT: f64 = 1.75;

/// Peers beyond this count no longer raise the density factor further.
const DENSITY_PEER_CAP: u32 = 16;
/// Density factor added per connected peer (up to `DENSITY_PEER_CAP`).
const DENSITY_STEP: f64 = 0.25;

/// Transport class a scheduler instance belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportClass {
    /// BLE scan / advertise restarts.
    Ble,
    /// mDNS / LAN discovery.
    Lan,
    /// Wi-Fi Direct / Wi-Fi Aware / Multipeer style local radio discovery.
    WifiDirect,
    /// Internet / cellular dial of candidates from the ledger.
    Internet,
}

impl TransportClass {
    /// All transport classes.
    pub const ALL: [TransportClass; 4] = [
        TransportClass::Ble,
        TransportClass::Lan,
        TransportClass::WifiDirect,
        TransportClass::Internet,
    ];

    /// Name used in the `[DISCOVERY]` logging contract (spec section 6).
    pub fn as_str(self) -> &'static str {
        match self {
            TransportClass::Ble => "ble",
            TransportClass::Lan => "lan",
            TransportClass::WifiDirect => "wifi-direct",
            TransportClass::Internet => "ledger",
        }
    }
}

/// Events that change what discovery should do. Each event knows which
/// transports it affects (`NetworkEvent::affects`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkEvent {
    /// Bluetooth adapter changed state. Only the on-transition resets.
    BleStateChanged { on: bool },
    /// Wi-Fi interface or network changed (up, down, different network).
    WifiChanged,
    /// Cellular interface or network changed.
    CellularChanged,
    /// A local interface address appeared or disappeared (LAN change).
    LanInterfaceChanged,
    /// App returned to the foreground.
    AppForeground,
    /// An invite was redeemed and the ledger gained seed entries.
    InviteRedeemed,
    /// A ledger exchange delivered entries; only `new_entries > 0` matters.
    LedgerReceived { new_entries: u32 },
    /// A peer disconnected; `count_now` peers remain connected.
    PeerDisconnected { count_now: u32 },
    /// The connected peer count dropped to zero.
    AllPeersLost,
    /// A connection added a peer not seen before (decay restarts on success).
    NewPeerConnected,
}

impl NetworkEvent {
    /// Short event name for the `[DISCOVERY]` log line.
    pub fn kind(&self) -> &'static str {
        match self {
            NetworkEvent::BleStateChanged { .. } => "BleStateChanged",
            NetworkEvent::WifiChanged => "WifiChanged",
            NetworkEvent::CellularChanged => "CellularChanged",
            NetworkEvent::LanInterfaceChanged => "LanInterfaceChanged",
            NetworkEvent::AppForeground => "AppForeground",
            NetworkEvent::InviteRedeemed => "InviteRedeemed",
            NetworkEvent::LedgerReceived { .. } => "LedgerReceived",
            NetworkEvent::PeerDisconnected { .. } => "PeerDisconnected",
            NetworkEvent::AllPeersLost => "AllPeersLost",
            NetworkEvent::NewPeerConnected => "NewPeerConnected",
        }
    }

    /// Whether this event should reset the given transport to aggressive.
    pub fn affects(&self, transport: TransportClass) -> bool {
        use TransportClass::{Ble, Internet, Lan, WifiDirect};
        match self {
            NetworkEvent::BleStateChanged { on } => *on && transport == Ble,
            NetworkEvent::WifiChanged | NetworkEvent::LanInterfaceChanged => {
                matches!(transport, Lan | WifiDirect | Internet)
            }
            NetworkEvent::CellularChanged | NetworkEvent::InviteRedeemed => transport == Internet,
            NetworkEvent::LedgerReceived { new_entries } => {
                *new_entries > 0 && transport == Internet
            }
            NetworkEvent::AppForeground
            | NetworkEvent::PeerDisconnected { .. }
            | NetworkEvent::AllPeersLost
            | NetworkEvent::NewPeerConnected => true,
        }
    }
}

/// Battery / charging state of the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    Charging,
    Normal,
    Low,
}

impl PowerState {
    fn factor(self) -> f64 {
        match self {
            PowerState::Charging => 1.0,
            PowerState::Normal => 1.5,
            PowerState::Low => 3.0,
        }
    }
}

/// Observed conditions that determine the decay ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscoveryInputs {
    pub connected_peers: u32,
    pub power: PowerState,
    pub foreground: bool,
}

impl Default for DiscoveryInputs {
    fn default() -> Self {
        Self {
            connected_peers: 0,
            power: PowerState::Normal,
            foreground: true,
        }
    }
}

/// Per-transport tuning. `base_ceiling_ms` is configuration with a
/// documented default, never the only behaviour (see the module header).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SchedulerConfig {
    /// Aggressive interval after a reset.
    pub floor_ms: u64,
    /// Ceiling at zero peers on a charging foreground device.
    pub base_ceiling_ms: u64,
    /// Decay growth factor, clamped into [`GROWTH_MIN`, `GROWTH_MAX`].
    pub growth: f64,
}

impl SchedulerConfig {
    /// Documented per-transport defaults.
    pub fn for_transport(transport: TransportClass) -> Self {
        let (floor_ms, base_ceiling_ms) = match transport {
            TransportClass::Ble => (1_000, 20_000),
            TransportClass::Lan => (1_000, 20_000),
            TransportClass::WifiDirect => (1_000, 30_000),
            TransportClass::Internet => (500, 30_000),
        };
        Self {
            floor_ms,
            base_ceiling_ms,
            growth: GROWTH_DEFAULT,
        }
    }

    fn sanitized(self) -> Self {
        let floor_ms = self.floor_ms.max(1);
        let growth = if self.growth.is_finite() {
            self.growth.clamp(GROWTH_MIN, GROWTH_MAX)
        } else {
            GROWTH_DEFAULT
        };
        Self {
            floor_ms,
            base_ceiling_ms: self.base_ceiling_ms.max(floor_ms),
            growth,
        }
    }
}

/// Injected monotonic clock (milliseconds).
pub trait SchedulerClock: Send + Sync {
    fn now_ms(&self) -> u64;
}

/// Injected jitter source returning a uniform value in [0, 1).
pub trait JitterSource: Send + Sync {
    fn next_unit(&self) -> f64;
}

/// Monotonic wall-clock implementation (native only).
#[cfg(not(target_arch = "wasm32"))]
pub struct SystemClock {
    start: std::time::Instant,
}

#[cfg(not(target_arch = "wasm32"))]
impl SystemClock {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl SchedulerClock for SystemClock {
    fn now_ms(&self) -> u64 {
        u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}

/// OS-seeded thread RNG jitter (native only).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct ThreadRngJitter;

#[cfg(not(target_arch = "wasm32"))]
impl JitterSource for ThreadRngJitter {
    fn next_unit(&self) -> f64 {
        use rand::Rng;
        rand::thread_rng().gen::<f64>()
    }
}

/// Scheduler phase, exposed for the node indicators (UI contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Aggressive,
    Decay,
}

impl Phase {
    pub fn as_str(self) -> &'static str {
        match self {
            Phase::Aggressive => "aggressive",
            Phase::Decay => "decay",
        }
    }
}

/// Read-only view of scheduler state.
#[derive(Debug, Clone, PartialEq)]
pub struct SchedulerSnapshot {
    pub transport: TransportClass,
    pub phase: Phase,
    /// Current un-jittered interval (what the next `next_delay` decays from).
    pub interval_ms: u64,
    /// Ceiling computed from the current inputs.
    pub ceiling_ms: u64,
    /// Attempts since the last reset.
    pub attempts: u32,
    /// Total resets since creation.
    pub resets: u64,
    /// Clock reading at the most recent `next_delay` call, if any.
    pub last_attempt_at_ms: Option<u64>,
    /// Kind of the most recent affecting event, if any.
    pub last_event: Option<&'static str>,
    /// Events absorbed by reset coalescing since creation.
    pub coalesced: u64,
    /// Current minimum spacing between accepted resets (0 before the first).
    pub reset_window_ms: u64,
}

struct State {
    interval_ms: u64,
    attempts: u32,
    resets: u64,
    inputs: DiscoveryInputs,
    last_attempt_at_ms: Option<u64>,
    last_event: Option<&'static str>,
    /// Clock reading of the last accepted reset.
    last_reset_at_ms: Option<u64>,
    /// Accepted resets since the last proven connection or quiet period.
    unproven_resets: u32,
    /// An event was coalesced; apply a reset when the window ends.
    pending_reset: Option<&'static str>,
    coalesced: u64,
}

/// Minimum spacing between accepted resets after `unproven` accepted resets
/// without a proven connection: `floor * 2^(unproven-1)`, never above the
/// live ceiling and never below the floor.
fn coalesce_window_ms(config: &SchedulerConfig, unproven: u32, ceiling_ms: u64) -> u64 {
    let shift = unproven.saturating_sub(1).min(32);
    config
        .floor_ms
        .saturating_mul(1u64 << shift)
        .min(ceiling_ms)
        .max(config.floor_ms)
}

/// Computes the decay ceiling from configuration and live inputs.
pub fn compute_ceiling_ms(config: &SchedulerConfig, inputs: &DiscoveryInputs) -> u64 {
    let peers = inputs.connected_peers.min(DENSITY_PEER_CAP);
    let density_factor = 1.0 + DENSITY_STEP * f64::from(peers);
    let background_factor = if inputs.foreground { 1.0 } else { 2.0 };
    let power_factor = inputs.power.factor() * background_factor;
    let raw = (config.base_ceiling_ms as f64) * density_factor * power_factor;
    (raw.round() as u64).max(config.floor_ms)
}

/// Event-driven discovery scheduler for one transport.
///
/// Usage: run an attempt, call [`next_delay`](Self::next_delay), wait that
/// long; if [`on_event`](Self::on_event) returns `true` while waiting, abandon
/// the wait and run an attempt immediately.
pub struct DiscoveryScheduler {
    transport: TransportClass,
    config: SchedulerConfig,
    clock: Arc<dyn SchedulerClock>,
    rng: Arc<dyn JitterSource>,
    state: RwLock<State>,
}

impl DiscoveryScheduler {
    pub fn new(
        transport: TransportClass,
        config: SchedulerConfig,
        clock: Arc<dyn SchedulerClock>,
        rng: Arc<dyn JitterSource>,
    ) -> Self {
        let config = config.sanitized();
        Self {
            transport,
            config,
            clock,
            rng,
            state: RwLock::new(State {
                interval_ms: config.floor_ms,
                attempts: 0,
                resets: 0,
                inputs: DiscoveryInputs::default(),
                last_attempt_at_ms: None,
                last_event: None,
                last_reset_at_ms: None,
                unproven_resets: 0,
                pending_reset: None,
                coalesced: 0,
            }),
        }
    }

    /// Scheduler with default config, system clock and thread RNG.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_system_time(transport: TransportClass) -> Self {
        Self::new(
            transport,
            SchedulerConfig::for_transport(transport),
            Arc::new(SystemClock::new()),
            Arc::new(ThreadRngJitter),
        )
    }

    pub fn transport(&self) -> TransportClass {
        self.transport
    }

    /// Replace the observed conditions that drive the ceiling.
    pub fn set_inputs(&self, inputs: DiscoveryInputs) {
        self.state.write().inputs = inputs;
    }

    pub fn inputs(&self) -> DiscoveryInputs {
        self.state.read().inputs
    }

    /// Feed an event. Returns `true` when it affects this transport and the
    /// schedule was reset to aggressive: the caller must cancel its pending
    /// timer and attempt immediately. Returns `false` (no state change)
    /// otherwise.
    pub fn on_event(&self, event: NetworkEvent) -> bool {
        if !event.affects(self.transport) {
            return false;
        }
        let now = self.clock.now_ms();
        let accepted = {
            let mut st = self.state.write();
            let ceiling = compute_ceiling_ms(&self.config, &st.inputs);
            let mut allowed = true;
            if let Some(last) = st.last_reset_at_ms {
                let elapsed = now.saturating_sub(last);
                if elapsed >= ceiling {
                    // A full quiet ceiling proves stability: forgive history.
                    st.unproven_resets = 0;
                }
                let window = coalesce_window_ms(&self.config, st.unproven_resets, ceiling);
                if elapsed < window {
                    st.coalesced = st.coalesced.saturating_add(1);
                    st.pending_reset = Some(event.kind());
                    allowed = false;
                }
            }
            if allowed {
                Some(self.apply_reset(&mut st, now, event.kind()))
            } else {
                None
            }
        };
        match accepted {
            Some((interval_ms, attempts, peers)) => {
                tracing::info!(
                    "[DISCOVERY] event={} transport={} phase=aggressive interval_ms={} attempt={} peers={}",
                    event.kind(),
                    self.transport.as_str(),
                    interval_ms,
                    attempts,
                    peers
                );
                true
            }
            None => {
                tracing::debug!(
                    "[DISCOVERY] event={} transport={} coalesced (flap damping); reset deferred to window end",
                    event.kind(),
                    self.transport.as_str()
                );
                false
            }
        }
    }

    /// Record that a connection was proven stable (for example a peer still
    /// connected across consecutive observations). Restores immediate reset
    /// responsiveness by clearing the coalescing history.
    pub fn record_success(&self) {
        self.state.write().unproven_resets = 0;
    }

    /// Clock reading at which the coalesced reset (if any) becomes due.
    fn pending_due_ms(&self, st: &State) -> Option<u64> {
        let (Some(_), Some(last)) = (st.pending_reset, st.last_reset_at_ms) else {
            return None;
        };
        let ceiling = compute_ceiling_ms(&self.config, &st.inputs);
        let window = coalesce_window_ms(&self.config, st.unproven_resets, ceiling);
        Some(last.saturating_add(window))
    }

    /// Time until a coalesced (deferred) reset falls due, or `None` when no
    /// reset is pending. `Some(ZERO)` means it is already due. A caller
    /// sleeping on a long timer must also wake at this instant, otherwise a
    /// real change absorbed by flap damping is only noticed when the current
    /// sleep (up to the ceiling) ends.
    pub fn pending_reset_due_in(&self) -> Option<Duration> {
        let now = self.clock.now_ms();
        let st = self.state.read();
        self.pending_due_ms(&st)
            .map(|due| Duration::from_millis(due.saturating_sub(now)))
    }

    /// Apply the coalesced reset now. Call when the timer from
    /// [`pending_reset_due_in`](Self::pending_reset_due_in) fired. Returns
    /// `true` when a reset was applied (the caller must attempt immediately),
    /// `false` when nothing was pending.
    pub fn apply_pending_reset(&self) -> bool {
        let now = self.clock.now_ms();
        let applied = {
            let mut st = self.state.write();
            let pending = st.pending_reset;
            pending.map(|kind| (kind, self.apply_reset(&mut st, now, kind)))
        };
        match applied {
            Some((kind, (interval_ms, attempts, peers))) => {
                tracing::info!(
                    "[DISCOVERY] event={} transport={} phase=aggressive interval_ms={} attempt={} peers={} (deferred reset applied at window end)",
                    kind,
                    self.transport.as_str(),
                    interval_ms,
                    attempts,
                    peers
                );
                true
            }
            None => false,
        }
    }

    /// Apply an accepted reset. Returns (interval, attempts, peers) for logging.
    fn apply_reset(&self, st: &mut State, now: u64, kind: &'static str) -> (u64, u32, u32) {
        st.interval_ms = self.config.floor_ms;
        st.attempts = 0;
        st.resets = st.resets.saturating_add(1);
        st.last_event = Some(kind);
        st.last_reset_at_ms = Some(now);
        st.unproven_resets = st.unproven_resets.saturating_add(1);
        st.pending_reset = None;
        (st.interval_ms, st.attempts, st.inputs.connected_peers)
    }

    /// Delay to wait before the next attempt, then advance the decay. Call
    /// once after each attempt. Never returns a give-up signal: the result is
    /// always a positive duration bounded by the current computed ceiling.
    pub fn next_delay(&self) -> Duration {
        let now = self.clock.now_ms();
        let unit = {
            let u = self.rng.next_unit();
            if u.is_finite() {
                u.clamp(0.0, 1.0)
            } else {
                0.5
            }
        };
        let (delay_ms, phase, base_ms, attempt, peers) = {
            let mut st = self.state.write();
            let ceiling = compute_ceiling_ms(&self.config, &st.inputs);
            // A coalesced event is applied the moment its window has ended;
            // until then the wait below is capped so it ends exactly there.
            let mut cap_ms = u64::MAX;
            if let (Some(kind), Some(due)) = (st.pending_reset, self.pending_due_ms(&st)) {
                if now >= due {
                    self.apply_reset(&mut st, now, kind);
                } else {
                    cap_ms = due - now;
                }
            }
            let base = st.interval_ms.min(ceiling).max(self.config.floor_ms);
            let delay = (((base as f64) * (0.5 + 0.5 * unit)).round() as u64)
                .min(cap_ms)
                .max(1);
            let phase = if st.attempts == 0 {
                Phase::Aggressive
            } else {
                Phase::Decay
            };
            let grown = ((base as f64) * self.config.growth).ceil() as u64;
            st.interval_ms = grown.min(ceiling).max(base);
            st.attempts = st.attempts.saturating_add(1);
            st.last_attempt_at_ms = Some(now);
            (delay, phase, base, st.attempts, st.inputs.connected_peers)
        };
        tracing::debug!(
            "[DISCOVERY] event=Rearm transport={} phase={} interval_ms={} attempt={} peers={}",
            self.transport.as_str(),
            phase.as_str(),
            base_ms,
            attempt,
            peers
        );
        Duration::from_millis(delay_ms)
    }

    /// Ceiling for the current inputs.
    pub fn ceiling_ms(&self) -> u64 {
        compute_ceiling_ms(&self.config, &self.state.read().inputs)
    }

    pub fn snapshot(&self) -> SchedulerSnapshot {
        let st = self.state.read();
        SchedulerSnapshot {
            transport: self.transport,
            phase: if st.attempts == 0 {
                Phase::Aggressive
            } else {
                Phase::Decay
            },
            interval_ms: st.interval_ms,
            ceiling_ms: compute_ceiling_ms(&self.config, &st.inputs),
            attempts: st.attempts,
            resets: st.resets,
            last_attempt_at_ms: st.last_attempt_at_ms,
            last_event: st.last_event,
            coalesced: st.coalesced,
            reset_window_ms: if st.last_reset_at_ms.is_some() {
                coalesce_window_ms(
                    &self.config,
                    st.unproven_resets,
                    compute_ceiling_ms(&self.config, &st.inputs),
                )
            } else {
                0
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct ManualClock(AtomicU64);
    impl ManualClock {
        fn new() -> Arc<Self> {
            Arc::new(Self(AtomicU64::new(0)))
        }
        fn advance(&self, ms: u64) {
            self.0.fetch_add(ms, Ordering::SeqCst);
        }
    }
    impl SchedulerClock for ManualClock {
        fn now_ms(&self) -> u64 {
            self.0.load(Ordering::SeqCst)
        }
    }

    /// Deterministic splitmix64-based jitter source.
    struct SeededJitter(AtomicU64);
    impl SeededJitter {
        fn new(seed: u64) -> Arc<Self> {
            Arc::new(Self(AtomicU64::new(seed)))
        }
    }
    impl JitterSource for SeededJitter {
        fn next_unit(&self) -> f64 {
            let s = self
                .0
                .fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::SeqCst)
                .wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = s;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            (z >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    /// Jitter source returning a constant (for exact-bound checks).
    struct FixedJitter(f64);
    impl JitterSource for FixedJitter {
        fn next_unit(&self) -> f64 {
            self.0
        }
    }

    fn sched(t: TransportClass) -> (DiscoveryScheduler, Arc<ManualClock>) {
        let clock = ManualClock::new();
        let s = DiscoveryScheduler::new(
            t,
            SchedulerConfig::for_transport(t),
            clock.clone(),
            SeededJitter::new(42),
        );
        (s, clock)
    }

    fn sched_with_jitter(t: TransportClass, j: f64) -> DiscoveryScheduler {
        DiscoveryScheduler::new(
            t,
            SchedulerConfig::for_transport(t),
            ManualClock::new(),
            Arc::new(FixedJitter(j)),
        )
    }

    #[test]
    fn starts_aggressive_at_floor() {
        let (s, _) = sched(TransportClass::Ble);
        let snap = s.snapshot();
        assert_eq!(snap.phase, Phase::Aggressive);
        assert_eq!(snap.interval_ms, 1_000);
        assert_eq!(snap.attempts, 0);
        assert_eq!(snap.resets, 0);
        assert_eq!(snap.last_attempt_at_ms, None);
    }

    #[test]
    fn affecting_event_resets_to_aggressive_and_signals_immediate_attempt() {
        let (s, _) = sched(TransportClass::Ble);
        for _ in 0..8 {
            s.next_delay();
        }
        assert!(s.snapshot().interval_ms > 1_000);
        assert_eq!(s.snapshot().phase, Phase::Decay);

        assert!(s.on_event(NetworkEvent::BleStateChanged { on: true }));
        let snap = s.snapshot();
        assert_eq!(snap.interval_ms, 1_000);
        assert_eq!(snap.attempts, 0);
        assert_eq!(snap.phase, Phase::Aggressive);
        assert_eq!(snap.resets, 1);
        assert_eq!(snap.last_event, Some("BleStateChanged"));
    }

    #[test]
    fn unaffected_events_leave_state_untouched() {
        let (s, _) = sched(TransportClass::Ble);
        for _ in 0..5 {
            s.next_delay();
        }
        let before = s.snapshot();
        assert!(!s.on_event(NetworkEvent::BleStateChanged { on: false }));
        assert!(!s.on_event(NetworkEvent::WifiChanged));
        assert!(!s.on_event(NetworkEvent::CellularChanged));
        assert!(!s.on_event(NetworkEvent::LanInterfaceChanged));
        assert!(!s.on_event(NetworkEvent::InviteRedeemed));
        assert!(!s.on_event(NetworkEvent::LedgerReceived { new_entries: 3 }));
        assert_eq!(s.snapshot(), before);
    }

    #[test]
    fn event_affects_table() {
        use TransportClass::*;
        let table: [(NetworkEvent, [bool; 4]); 10] = [
            // order: Ble, Lan, WifiDirect, Internet
            (
                NetworkEvent::BleStateChanged { on: true },
                [true, false, false, false],
            ),
            (
                NetworkEvent::BleStateChanged { on: false },
                [false, false, false, false],
            ),
            (NetworkEvent::WifiChanged, [false, true, true, true]),
            (NetworkEvent::CellularChanged, [false, false, false, true]),
            (NetworkEvent::LanInterfaceChanged, [false, true, true, true]),
            (NetworkEvent::AppForeground, [true, true, true, true]),
            (NetworkEvent::InviteRedeemed, [false, false, false, true]),
            (
                NetworkEvent::LedgerReceived { new_entries: 0 },
                [false, false, false, false],
            ),
            (
                NetworkEvent::PeerDisconnected { count_now: 2 },
                [true, true, true, true],
            ),
            (NetworkEvent::AllPeersLost, [true, true, true, true]),
        ];
        for (event, expected) in table {
            for (t, want) in [Ble, Lan, WifiDirect, Internet].into_iter().zip(expected) {
                assert_eq!(event.affects(t), want, "{:?} vs {:?}", event, t);
            }
        }
        assert!(NetworkEvent::LedgerReceived { new_entries: 1 }.affects(Internet));
        assert!(NetworkEvent::NewPeerConnected.affects(Ble));
    }

    #[test]
    fn base_interval_decays_monotonically_to_ceiling_and_holds() {
        let (s, _) = sched(TransportClass::Internet);
        let ceiling = s.ceiling_ms();
        let mut prev = s.snapshot().interval_ms;
        let mut reached = false;
        for _ in 0..64 {
            s.next_delay();
            let now = s.snapshot().interval_ms;
            assert!(now >= prev, "decay must be monotone: {} < {}", now, prev);
            assert!(now <= ceiling);
            if now == ceiling {
                reached = true;
            }
            prev = now;
        }
        assert!(reached, "interval must reach the computed ceiling");
        assert_eq!(s.snapshot().interval_ms, ceiling);
    }

    #[test]
    fn growth_is_within_spec_band() {
        for growth in [0.0, 1.0, 1.5, 1.75, 2.0, 5.0, f64::NAN, f64::INFINITY] {
            let cfg = SchedulerConfig {
                floor_ms: 1_000,
                base_ceiling_ms: 1_000_000,
                growth,
            };
            let s = DiscoveryScheduler::new(
                TransportClass::Lan,
                cfg,
                ManualClock::new(),
                Arc::new(FixedJitter(1.0)),
            );
            let mut prev = s.snapshot().interval_ms;
            for _ in 0..5 {
                s.next_delay();
                let now = s.snapshot().interval_ms;
                let ratio = now as f64 / prev as f64;
                assert!(
                    (GROWTH_MIN - 0.01..=GROWTH_MAX + 0.01).contains(&ratio),
                    "growth {} gave ratio {}",
                    growth,
                    ratio
                );
                prev = now;
            }
        }
    }

    #[test]
    fn jitter_stays_within_half_to_full_interval() {
        let (s, _) = sched(TransportClass::WifiDirect);
        for _ in 0..200 {
            let base = s.snapshot().interval_ms.min(s.ceiling_ms());
            let d = s.next_delay().as_millis() as u64;
            assert!(d >= base / 2, "delay {} below half of {}", d, base);
            assert!(d <= base, "delay {} above interval {}", d, base);
            assert!(d >= 1);
        }
    }

    #[test]
    fn jitter_extremes_hit_exact_bounds() {
        let lo = sched_with_jitter(TransportClass::Ble, 0.0);
        assert_eq!(lo.next_delay(), Duration::from_millis(500));
        let hi = sched_with_jitter(TransportClass::Ble, 1.0);
        assert_eq!(hi.next_delay(), Duration::from_millis(1_000));
        // Out-of-range / non-finite jitter is sanitised, never panics.
        let nan = sched_with_jitter(TransportClass::Ble, f64::NAN);
        let d = nan.next_delay();
        assert!(d >= Duration::from_millis(500) && d <= Duration::from_millis(1_000));
        let big = sched_with_jitter(TransportClass::Ble, 7.0);
        assert_eq!(big.next_delay(), Duration::from_millis(1_000));
    }

    #[test]
    fn same_seed_gives_same_sequence() {
        let a = sched(TransportClass::Lan).0;
        let b = sched(TransportClass::Lan).0;
        for _ in 0..50 {
            assert_eq!(a.next_delay(), b.next_delay());
        }
    }

    #[test]
    fn jitter_actually_varies_delays() {
        let (s, _) = sched(TransportClass::Internet);
        // Run to the ceiling plateau, then sample: the same base must give
        // several distinct delays.
        for _ in 0..30 {
            s.next_delay();
        }
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..50 {
            seen.insert(s.next_delay());
        }
        assert!(seen.len() > 10, "jitter must desynchronise nodes");
    }

    #[test]
    fn zero_peers_charging_foreground_ceiling_is_base() {
        let cfg = SchedulerConfig::for_transport(TransportClass::Ble);
        let inputs = DiscoveryInputs {
            connected_peers: 0,
            power: PowerState::Charging,
            foreground: true,
        };
        assert_eq!(compute_ceiling_ms(&cfg, &inputs), cfg.base_ceiling_ms);
    }

    #[test]
    fn ceiling_rises_with_density_and_saturates() {
        let cfg = SchedulerConfig::for_transport(TransportClass::Lan);
        let at = |peers| {
            compute_ceiling_ms(
                &cfg,
                &DiscoveryInputs {
                    connected_peers: peers,
                    power: PowerState::Charging,
                    foreground: true,
                },
            )
        };
        let mut prev = at(0);
        for p in 1..=DENSITY_PEER_CAP {
            let c = at(p);
            assert!(c > prev, "density must raise the ceiling at {} peers", p);
            prev = c;
        }
        assert_eq!(at(DENSITY_PEER_CAP), 5 * cfg.base_ceiling_ms);
        assert_eq!(at(10_000), at(DENSITY_PEER_CAP));
    }

    #[test]
    fn ceiling_rises_with_low_power_and_background() {
        let cfg = SchedulerConfig::for_transport(TransportClass::Internet);
        let at = |power, foreground| {
            compute_ceiling_ms(
                &cfg,
                &DiscoveryInputs {
                    connected_peers: 0,
                    power,
                    foreground,
                },
            )
        };
        assert!(at(PowerState::Normal, true) > at(PowerState::Charging, true));
        assert!(at(PowerState::Low, true) > at(PowerState::Normal, true));
        assert!(at(PowerState::Charging, false) > at(PowerState::Charging, true));
        assert_eq!(
            at(PowerState::Low, false),
            6 * cfg.base_ceiling_ms,
            "worst case is bounded by the factor product"
        );
    }

    #[test]
    fn plateau_follows_inputs_in_both_directions() {
        let (s, _) = sched(TransportClass::Internet);
        for _ in 0..40 {
            s.next_delay();
        }
        let low = s.snapshot().interval_ms;
        assert_eq!(low, s.ceiling_ms());

        s.set_inputs(DiscoveryInputs {
            connected_peers: 8,
            power: PowerState::Low,
            foreground: false,
        });
        for _ in 0..40 {
            s.next_delay();
        }
        let high = s.snapshot().interval_ms;
        assert!(high > low, "denser/low-power plateau must be longer");
        assert_eq!(high, s.ceiling_ms());

        // Conditions improve: the very next delay obeys the smaller ceiling.
        s.set_inputs(DiscoveryInputs {
            connected_peers: 0,
            power: PowerState::Charging,
            foreground: true,
        });
        let d = s.next_delay().as_millis() as u64;
        assert!(
            d <= low,
            "delay {} must respect the shrunken ceiling {}",
            d,
            low
        );
    }

    #[test]
    fn never_gives_up_and_every_delay_is_positive_and_bounded() {
        let (s, clock) = sched(TransportClass::Ble);
        for i in 0..20_000u32 {
            let d = s.next_delay();
            assert!(d > Duration::ZERO, "attempt {} returned zero", i);
            assert!(d.as_millis() as u64 <= s.ceiling_ms());
            clock.advance(d.as_millis() as u64);
        }
        let snap = s.snapshot();
        assert_eq!(snap.attempts, 20_000);
        assert_eq!(snap.phase, Phase::Decay);
        // Still fully responsive after a very long quiet period.
        assert!(s.on_event(NetworkEvent::AppForeground));
        assert_eq!(s.snapshot().interval_ms, 1_000);
    }

    #[test]
    fn attempt_counter_saturates_instead_of_overflowing() {
        let (s, _) = sched(TransportClass::Lan);
        s.state.write().attempts = u32::MAX;
        s.next_delay();
        assert_eq!(s.snapshot().attempts, u32::MAX);
    }

    #[test]
    fn last_attempt_time_comes_from_injected_clock() {
        let (s, clock) = sched(TransportClass::Lan);
        clock.advance(1234);
        s.next_delay();
        assert_eq!(s.snapshot().last_attempt_at_ms, Some(1234));
    }

    #[test]
    fn repeated_resets_keep_restarting_from_floor() {
        let (s, clock) = sched(TransportClass::Internet);
        for round in 1..=5u64 {
            for _ in 0..10 {
                s.next_delay();
            }
            // Spaced past the (widening) coalescing window, but not proven.
            clock.advance(s.ceiling_ms());
            assert!(s.on_event(NetworkEvent::CellularChanged));
            assert_eq!(s.snapshot().interval_ms, 500);
            assert_eq!(s.snapshot().resets, round);
        }
    }

    #[test]
    fn event_storm_is_bounded_by_coalescing() {
        let (s, clock) = sched(TransportClass::Internet);
        let mut accepted = 0u64;
        // 10_000 flap events, one per millisecond, over ten seconds.
        for _ in 0..10_000 {
            if s.on_event(NetworkEvent::WifiChanged) {
                accepted += 1;
            }
            clock.advance(1);
        }
        // Windows 500, 1000, 2000, 4000, 8000 ms: at most five resets fit.
        assert!((1..=6).contains(&accepted), "storm accepted {}", accepted);
        let snap = s.snapshot();
        assert_eq!(snap.resets, accepted);
        assert_eq!(snap.coalesced, 10_000 - accepted);
    }

    #[test]
    fn window_widens_with_unproven_resets_up_to_ceiling() {
        let (s, clock) = sched(TransportClass::Internet);
        let mut prev = 0;
        for _ in 0..9 {
            assert!(s.on_event(NetworkEvent::WifiChanged));
            let w = s.snapshot().reset_window_ms;
            assert!(w >= prev, "window must not shrink: {} < {}", w, prev);
            assert!(w <= s.ceiling_ms());
            prev = w;
            clock.advance(w);
        }
        assert_eq!(prev, s.ceiling_ms());
    }

    #[test]
    fn real_change_after_quiet_period_resets_immediately() {
        let (s, clock) = sched(TransportClass::Internet);
        for _ in 0..8 {
            s.on_event(NetworkEvent::WifiChanged);
            clock.advance(10);
        }
        assert!(s.snapshot().coalesced > 0);
        clock.advance(s.ceiling_ms());
        assert!(s.on_event(NetworkEvent::CellularChanged));
        assert_eq!(s.snapshot().interval_ms, 500);
        // History was forgiven: the next window is the floor again.
        assert_eq!(s.snapshot().reset_window_ms, 500);
    }

    #[test]
    fn record_success_restores_immediate_resets() {
        let (s, clock) = sched(TransportClass::Internet);
        assert!(s.on_event(NetworkEvent::WifiChanged));
        clock.advance(600);
        assert!(s.on_event(NetworkEvent::WifiChanged));
        clock.advance(600);
        // Window is now 1000 ms: an event 600 ms after the last reset is damped.
        assert!(!s.on_event(NetworkEvent::WifiChanged));
        s.record_success();
        // Same instant, but proven history gone: window is the floor again.
        assert!(s.on_event(NetworkEvent::WifiChanged));
    }

    #[test]
    fn coalesced_event_is_applied_when_window_ends_not_lost() {
        let (s, clock) = sched(TransportClass::Internet);
        assert!(s.on_event(NetworkEvent::WifiChanged));
        clock.advance(600);
        assert!(s.on_event(NetworkEvent::WifiChanged));
        // Window is now 1000 ms from t=600. Inside it: damped, remembered.
        clock.advance(100);
        assert!(!s.on_event(NetworkEvent::CellularChanged));
        // The wait is capped to end at the window boundary (900 ms away).
        assert!(s.next_delay().as_millis() as u64 <= 900);
        clock.advance(900);
        let resets = s.snapshot().resets;
        s.next_delay();
        assert_eq!(s.snapshot().resets, resets + 1);
        assert_eq!(s.snapshot().last_event, Some("CellularChanged"));
    }

    #[test]
    fn pending_reset_due_time_is_exposed_and_applied() {
        let (s, clock) = sched(TransportClass::Internet);
        assert_eq!(s.pending_reset_due_in(), None);
        assert!(!s.apply_pending_reset());
        // Flap, then a real change 300 ms later (floor is 500 ms): coalesced.
        assert!(s.on_event(NetworkEvent::WifiChanged));
        clock.advance(300);
        assert!(!s.on_event(NetworkEvent::CellularChanged));
        // Due exactly at the end of the 500 ms window: 200 ms from now.
        assert_eq!(s.pending_reset_due_in(), Some(Duration::from_millis(200)));
        let resets = s.snapshot().resets;
        assert!(s.apply_pending_reset());
        assert_eq!(s.snapshot().resets, resets + 1);
        assert_eq!(s.snapshot().last_event, Some("CellularChanged"));
        assert_eq!(s.pending_reset_due_in(), None);
    }

    #[test]
    fn degenerate_config_is_sanitised() {
        let cfg = SchedulerConfig {
            floor_ms: 0,
            base_ceiling_ms: 0,
            growth: 1.75,
        };
        let s = DiscoveryScheduler::new(
            TransportClass::Ble,
            cfg,
            ManualClock::new(),
            Arc::new(FixedJitter(0.5)),
        );
        for _ in 0..10 {
            assert!(s.next_delay() >= Duration::from_millis(1));
        }
    }

    #[test]
    fn log_names_follow_the_contract() {
        assert_eq!(TransportClass::Ble.as_str(), "ble");
        assert_eq!(TransportClass::Lan.as_str(), "lan");
        assert_eq!(TransportClass::Internet.as_str(), "ledger");
        assert_eq!(Phase::Aggressive.as_str(), "aggressive");
        assert_eq!(Phase::Decay.as_str(), "decay");
    }
}
