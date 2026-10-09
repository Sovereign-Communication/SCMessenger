// Per-peer backoff state machine for graceful dial policy.
//
// Event-driven peer revival (operator directive: no static caps, never give
// up). A peer is NEVER dead-marked. Repeated failures push it into a
// `Dormant` state, which only means "the next attempt waits for a wake event
// or for its jittered backoff to elapse". Retries never stop.
//
// Wake events ([`WakeTrigger`]): inbound connection or dial from the peer,
// identify, a newly learned address, a network-change event from the merged
// discovery scheduler, or an outbound message queued for the peer. A wake
// grants a ONE-SHOT credit: the next attempt is pulled forward to the backoff
// floor, but the backoff ladder is NOT reset (a failed woken attempt resumes
// the ladder where it was). Each (address, trigger) source has a token bucket
// whose refill period is derived from the observed event frequency, clamped
// by the stability-derived ceiling, so a flood of wakes cannot hold an
// address at the floor. mDNS wakes are honoured only for peers with prior
// authenticated history (mDNS is an unauthenticated LAN claim).
//
// Backoff: exponential from the floor, full jitter (uniform 0.5..1.0 of the
// nominal interval), with a ceiling derived from observed network stability
// (EWMA of recent connect success vs failure) rather than a literal.
//
// Circuit-relay preference: Once a peer connects, we add circuit-relay multiaddrs
// to the candidate ladder in order: direct -> relay -> fallback.

use libp2p::{Multiaddr, PeerId};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, info};
use web_time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Backoff floor: first retry interval and the minimum gap between two
/// attempts to the same address, even when wake events arrive in a flood.
/// This also bounds attempts per minute per address (<= 60) under any wake
/// pattern.
pub const BACKOFF_FLOOR: Duration = Duration::from_secs(1);

/// Scale for the stability-derived ceiling (like the discovery scheduler's
/// `base_ceiling`): the ceiling is this value multiplied by a factor in
/// [1, 4] that grows as the recent connect success rate falls.
pub const BACKOFF_CEILING_BASE: Duration = Duration::from_secs(30);

/// Consecutive failures after which a peer is reported as `Dormant`. This is
/// a labelling threshold only: it never blocks or stops dialing.
const DORMANT_AFTER_FAILURES: u32 = 3;

/// EWMA weight of the newest connect outcome in the stability estimate.
const STABILITY_ALPHA: f64 = 0.2;

/// Initial per-address stability estimate (no evidence either way).
const STABILITY_PRIOR: f64 = 0.5;

/// Events that wake a dormant (or backed-off) peer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WakeTrigger {
    /// The peer (or its host) connected to us.
    InboundConnection,
    /// The peer (or its host) attempted to dial us.
    InboundDial,
    /// Identify exchanged with the peer.
    Identify,
    /// A new address for the peer was learned (ledger, identify, BLE).
    AddressLearned,
    /// The peer was sighted via mDNS. mDNS is an unauthenticated LAN
    /// broadcast, so this wakes only peers with prior authenticated history.
    MdnsDiscovered,
    /// Network-change event from the discovery scheduler.
    NetworkChange,
    /// An outbound message was queued for the peer.
    OutboundQueued,
}

impl WakeTrigger {
    /// Stable name for the `[DIAL] wake` marker.
    pub fn as_str(self) -> &'static str {
        match self {
            WakeTrigger::InboundConnection => "inbound_connection",
            WakeTrigger::InboundDial => "inbound_dial",
            WakeTrigger::Identify => "identify",
            WakeTrigger::AddressLearned => "address_learned",
            WakeTrigger::MdnsDiscovered => "mdns_discovered",
            WakeTrigger::NetworkChange => "network_change",
            WakeTrigger::OutboundQueued => "outbound_queued",
        }
    }
}

/// Uniform value in [0, 1) for backoff jitter. SplitMix64 over a process-wide
/// counter mixed with wall-clock nanos: cheap, dependency-free, and works on
/// every target (including wasm32). Not for cryptographic use.
pub(crate) fn jitter_unit() -> f64 {
    static COUNTER: AtomicU64 = AtomicU64::new(0x9E37_79B9_7F4A_7C15);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let mut z = COUNTER
        .fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed)
        .wrapping_add(nanos);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64
}

/// Apply full jitter: uniform(0.5, 1.0) of `nominal`.
pub(crate) fn jittered(nominal: Duration) -> Duration {
    nominal.mul_f64(0.5 + 0.5 * jitter_unit())
}

/// Token bucket for the wake credits of one (address, trigger) source.
///
/// Capacity is a single credit (a wake is a one-shot credit, not a stream).
/// The refill period is derived, not a literal: it is the EWMA of the observed
/// gap between events from this source, clamped to `[ceiling / 4, ceiling]`
/// where `ceiling` is the stability-derived backoff ceiling. A rare source
/// therefore always finds its credit refilled, while a flooding source
/// (gap -> 0) is held to one credit per quarter-ceiling no matter how fast it
/// fires. Rejected events still update the observed frequency.
#[derive(Debug, Clone)]
pub(crate) struct WakeBucket {
    tokens: f64,
    refilled_at: Instant,
    last_event: Option<Instant>,
    gap_ewma: Option<Duration>,
}

impl WakeBucket {
    pub(crate) fn new(now: Instant) -> Self {
        Self {
            tokens: 1.0,
            refilled_at: now,
            last_event: None,
            gap_ewma: None,
        }
    }

    /// Consume one credit if available.
    pub(crate) fn admit(&mut self, now: Instant, ceiling: Duration) -> bool {
        let ceiling = ceiling.max(BACKOFF_FLOOR);
        if let Some(last) = self.last_event {
            let gap = now.saturating_duration_since(last);
            self.gap_ewma = Some(match self.gap_ewma {
                Some(prev) => prev.mul_f64(1.0 - STABILITY_ALPHA) + gap.mul_f64(STABILITY_ALPHA),
                None => gap,
            });
        }
        self.last_event = Some(now);

        let min_period = (ceiling / 4).max(BACKOFF_FLOOR);
        let period = self.gap_ewma.unwrap_or(ceiling).clamp(min_period, ceiling);
        let elapsed = now.saturating_duration_since(self.refilled_at);
        self.tokens = (self.tokens + elapsed.as_secs_f64() / period.as_secs_f64()).min(1.0);
        self.refilled_at = now;
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Last 8 characters of a peer id, for log markers.
fn short_peer(peer_id: Option<PeerId>) -> String {
    match peer_id {
        Some(p) => {
            let s = p.to_string();
            let skip = s.chars().count().saturating_sub(8);
            s.chars().skip(skip).collect()
        }
        None => "unknown".to_string(),
    }
}

/// Backoff ceiling derived from the recent connect success rate in [0, 1].
/// A healthy network keeps the ceiling near the base (retries stay brisk); a
/// failing network stretches it up to 4x so a dead link is not hammered, while
/// wake events still bring the next attempt forward immediately.
pub fn ceiling_for_success_rate(success_rate: f64) -> Duration {
    let rate = if success_rate.is_finite() {
        success_rate.clamp(0.0, 1.0)
    } else {
        0.5
    };
    BACKOFF_CEILING_BASE
        .mul_f64(1.0 + 3.0 * (1.0 - rate))
        .max(BACKOFF_FLOOR)
}

/// Per-peer backoff state tracking.
#[derive(Debug, Clone)]
pub struct PerPeerBackoffState {
    /// Consecutive failed dial attempts. Unbounded: there is no failure
    /// threshold at which dialing stops.
    pub attempt_count: u32,
    /// Timestamp of the last dial attempt to this peer.
    pub last_attempt_ts: Instant,
    /// Current nominal backoff duration (1s doubling up to the ceiling).
    pub backoff_duration: Duration,
    /// Earliest time of the next attempt (jittered backoff, or pulled
    /// forward by a wake event).
    pub next_attempt_at: Instant,
    /// `Dormant`: the next attempt waits for a wake event or the jittered
    /// backoff. Not a terminal state; cleared by wake or success.
    pub dormant: bool,
    /// Optional peer ID if known at registration time.
    pub peer_id: Option<PeerId>,
    /// Per-address EWMA of connect outcomes in [0, 1] (1 = all succeed).
    /// Drives THIS address's backoff ceiling, so dead ledger-learned
    /// addresses stretch only their own ladder.
    success_rate: f64,
    /// True once a connection through this entry (or, via
    /// `reset_peer_backoff`, any entry of the same peer) was established over
    /// an authenticated transport.
    authenticated_before: bool,
    /// Wake-credit buckets, one per trigger (bounded by the trigger count).
    wake_buckets: HashMap<WakeTrigger, WakeBucket>,
}

impl PerPeerBackoffState {
    /// Create a new backoff state with initial backoff of 1 second.
    pub fn new(peer_id: Option<PeerId>) -> Self {
        Self::new_at(peer_id, Instant::now())
    }

    fn new_at(peer_id: Option<PeerId>, now: Instant) -> Self {
        Self {
            attempt_count: 0,
            last_attempt_ts: now,
            backoff_duration: BACKOFF_FLOOR,
            next_attempt_at: now,
            dormant: false,
            peer_id,
            success_rate: STABILITY_PRIOR,
            authenticated_before: false,
            wake_buckets: HashMap::new(),
        }
    }

    /// Fold a connect outcome into this address's stability estimate.
    fn record_outcome(&mut self, success: bool) {
        let sample = if success { 1.0 } else { 0.0 };
        self.success_rate = (1.0 - STABILITY_ALPHA) * self.success_rate + STABILITY_ALPHA * sample;
    }

    /// Backoff ceiling for this address, from its own stability estimate.
    pub fn ceiling(&self) -> Duration {
        ceiling_for_success_rate(self.success_rate)
    }

    /// Check if this peer is eligible for a dial attempt right now.
    pub fn is_eligible(&self) -> bool {
        self.is_eligible_at(Instant::now())
    }

    /// Eligibility at an explicit time (fake-clock friendly).
    pub fn is_eligible_at(&self, now: Instant) -> bool {
        now >= self.next_attempt_at
    }

    /// Record a failed dial attempt using the default ceiling.
    pub fn on_dial_failure(&mut self) {
        self.on_dial_failure_at(Instant::now(), BACKOFF_CEILING_BASE);
    }

    /// Record a failed dial attempt: increment the count, double the nominal
    /// backoff up to `ceiling`, schedule the next attempt with full jitter.
    pub fn on_dial_failure_at(&mut self, now: Instant, ceiling: Duration) {
        self.attempt_count = self.attempt_count.saturating_add(1);
        self.last_attempt_ts = now;

        let ceiling = ceiling.max(BACKOFF_FLOOR);
        let doubled = self.backoff_duration.saturating_mul(2);
        self.backoff_duration = doubled.min(ceiling);
        let wait = jittered(self.backoff_duration);
        self.next_attempt_at = now + wait;

        debug!(
            peer_id=?self.peer_id,
            attempt_count=self.attempt_count,
            backoff_ms=self.backoff_duration.as_millis() as u64,
            "[DIAL-BACKOFF] Incremented attempt count and backoff"
        );

        if self.attempt_count >= DORMANT_AFTER_FAILURES && !self.dormant {
            self.dormant = true;
            info!(
                "[DIAL] dormant peer={} reason=repeated_failures next_wake=event|backoff_ms={}",
                short_peer(self.peer_id),
                wait.as_millis()
            );
        }
    }

    /// Record a permanent-looking dial failure (for example an unsupported
    /// address). The peer goes Dormant at the current ceiling; it is NOT
    /// dead and is retried on the next wake event or backoff expiry.
    pub fn on_permanent_failure(&mut self) {
        self.on_permanent_failure_at(Instant::now(), BACKOFF_CEILING_BASE);
    }

    /// See [`Self::on_permanent_failure`].
    pub fn on_permanent_failure_at(&mut self, now: Instant, ceiling: Duration) {
        let ceiling = ceiling.max(BACKOFF_FLOOR);
        self.attempt_count = self.attempt_count.max(DORMANT_AFTER_FAILURES);
        self.last_attempt_ts = now;
        self.backoff_duration = ceiling;
        let wait = jittered(ceiling);
        self.next_attempt_at = now + wait;
        self.dormant = true;
        info!(
            "[DIAL] dormant peer={} reason=permanent_failure next_wake=event|backoff_ms={}",
            short_peer(self.peer_id),
            wait.as_millis()
        );
    }

    /// Grant a one-shot wake credit: pull the next attempt forward to the
    /// backoff floor after the last attempt. The backoff ladder is NOT reset
    /// (`backoff_duration` and `attempt_count` are kept), so a woken attempt
    /// that fails resumes the ladder where it was. Credits are limited by a
    /// per-trigger token bucket (see [`WakeBucket`]); mDNS wakes need prior
    /// authenticated history. Returns true when a credit was granted
    /// (something was waiting).
    pub fn wake_at(&mut self, now: Instant, trigger: WakeTrigger) -> bool {
        let authenticated = self.authenticated_before;
        self.wake_at_with(now, trigger, authenticated)
    }

    /// As [`Self::wake_at`], with peer-level authenticated history supplied
    /// by the caller (history may sit on a sibling address entry).
    fn wake_at_with(
        &mut self,
        now: Instant,
        trigger: WakeTrigger,
        peer_authenticated: bool,
    ) -> bool {
        if self.attempt_count == 0 && !self.dormant {
            return false;
        }
        if trigger == WakeTrigger::MdnsDiscovered && !peer_authenticated {
            debug!(
                "[DIAL] wake ignored peer={} trigger={} reason=no_authenticated_history",
                short_peer(self.peer_id),
                trigger.as_str()
            );
            return false;
        }
        let ceiling = self.ceiling();
        let admitted = self
            .wake_buckets
            .entry(trigger)
            .or_insert_with(|| WakeBucket::new(now))
            .admit(now, ceiling);
        if !admitted {
            debug!(
                "[DIAL] wake rate-limited peer={} trigger={}",
                short_peer(self.peer_id),
                trigger.as_str()
            );
            return false;
        }
        let was_dormant = self.dormant;
        let earliest = (self.last_attempt_ts + BACKOFF_FLOOR).max(now);
        if earliest < self.next_attempt_at {
            self.next_attempt_at = earliest;
        }
        self.dormant = false;
        if was_dormant {
            info!(
                "[DIAL] wake peer={} trigger={}",
                short_peer(self.peer_id),
                trigger.as_str()
            );
        } else {
            debug!(
                "[DIAL] wake peer={} trigger={}",
                short_peer(self.peer_id),
                trigger.as_str()
            );
        }
        true
    }

    /// Reset backoff state on successful connection.
    pub fn on_connection_established(&mut self) {
        self.on_connection_established_at(Instant::now());
    }

    fn on_connection_established_at(&mut self, now: Instant) {
        let old_attempt_count = self.attempt_count;
        self.attempt_count = 0;
        self.backoff_duration = BACKOFF_FLOOR;
        self.last_attempt_ts = now;
        self.next_attempt_at = now;
        self.dormant = false;
        self.authenticated_before = true;

        info!(
            peer_id=?self.peer_id,
            prev_attempt_count=old_attempt_count,
            "[DIAL-BACKOFF] Reset backoff state after successful connection"
        );
    }
}

/// Global dial policy manager: tracks per-peer backoff state and enforces
/// concurrent dial limits per address.
#[derive(Debug, Clone)]
pub struct DialPolicyManager {
    /// Per-peer backoff state, keyed by peer address (stripped of /p2p/).
    /// Using String as key to handle addresses without peer IDs.
    peer_backoff: Arc<RwLock<HashMap<String, PerPeerBackoffState>>>,
    /// Count of in-flight (queued but not yet connected/failed) dials to each peer.
    concurrent_dials: Arc<RwLock<HashMap<String, u32>>>,
    /// Offset added to the monotonic clock. Always zero in production;
    /// tests advance it to simulate hours passing without sleeping.
    clock_skew: Arc<RwLock<Duration>>,
}

impl DialPolicyManager {
    /// Create a new dial policy manager.
    pub fn new() -> Self {
        Self {
            peer_backoff: Arc::new(RwLock::new(HashMap::new())),
            concurrent_dials: Arc::new(RwLock::new(HashMap::new())),
            clock_skew: Arc::new(RwLock::new(Duration::ZERO)),
        }
    }

    fn now(&self) -> Instant {
        Instant::now() + *self.clock_skew.read()
    }

    /// Advance the manager's clock (test hook for fake-clock scenarios).
    #[doc(hidden)]
    pub fn advance_clock(&self, by: Duration) {
        let mut skew = self.clock_skew.write();
        *skew = skew.saturating_add(by);
    }

    /// Backoff ceiling for one address, derived from that address's own
    /// observed connect stability (never from other peers' failures).
    pub fn ceiling_for(&self, addr_key: &str) -> Duration {
        self.peer_backoff
            .read()
            .get(addr_key)
            .map(PerPeerBackoffState::ceiling)
            .unwrap_or_else(|| ceiling_for_success_rate(STABILITY_PRIOR))
    }

    /// Register the start of a dial attempt to a peer address.
    /// Returns true if the dial is allowed (backoff eligible + under concurrent limit).
    /// Returns false if the address is waiting out its backoff or at the
    /// concurrent dial limit.
    pub fn register_dial_attempt(&self, addr_key: &str, peer_id: Option<PeerId>) -> bool {
        let now = self.now();
        let mut backoff = self.peer_backoff.write();
        let mut concurrent = self.concurrent_dials.write();

        let state = backoff.entry(addr_key.to_string()).or_insert_with(|| {
            debug!(addr_key=%addr_key, "[DIAL-POLICY] Registering new peer backoff state");
            PerPeerBackoffState::new_at(peer_id, now)
        });

        if !state.is_eligible_at(now) {
            debug!(
                addr_key=%addr_key,
                attempt_count=state.attempt_count,
                dormant=state.dormant,
                backoff_ms=state.backoff_duration.as_millis() as u64,
                "[DIAL-POLICY] Peer is waiting for backoff or a wake event"
            );
            return false;
        }

        // Check concurrent dial limit (max 3 per address).
        let dial_count = concurrent.entry(addr_key.to_string()).or_insert(0);
        if *dial_count >= 3 {
            debug!(
                addr_key=%addr_key,
                current_concurrent=*dial_count,
                "[DIAL-POLICY] Peer at concurrent dial limit (3/3)"
            );
            return false;
        }

        *dial_count += 1;
        debug!(
            addr_key=%addr_key,
            concurrent_count=*dial_count,
            "[DIAL-POLICY] Dial attempt registered (concurrent dial count)"
        );
        true
    }

    /// Record the completion of a dial attempt (whether it succeeds or fails).
    /// Must be called once per successful register_dial_attempt.
    pub fn complete_dial_attempt(&self, addr_key: &str) {
        let mut concurrent = self.concurrent_dials.write();
        if let Some(count) = concurrent.get_mut(addr_key) {
            if *count > 0 {
                *count -= 1;
                debug!(
                    addr_key=%addr_key,
                    remaining_concurrent=*count,
                    "[DIAL-POLICY] Dial attempt completed (decremented concurrent count)"
                );
            }
        }
    }

    /// Record a transient dial failure for a peer address.
    /// Increments the attempt count and applies jittered exponential backoff.
    pub fn record_dial_failure(&self, addr_key: &str, peer_id: Option<PeerId>) {
        let now = self.now();
        let mut backoff = self.peer_backoff.write();
        let state = backoff
            .entry(addr_key.to_string())
            .or_insert_with(|| PerPeerBackoffState::new_at(peer_id, now));
        state.record_outcome(false);
        let ceiling = state.ceiling();
        state.on_dial_failure_at(now, ceiling);
    }

    /// Record a permanent-looking dial failure for a peer address. The
    /// address goes Dormant (wake event or backoff); it is never dead.
    pub fn record_permanent_failure(&self, addr_key: &str, peer_id: Option<PeerId>) {
        let now = self.now();
        let mut backoff = self.peer_backoff.write();
        let state = backoff
            .entry(addr_key.to_string())
            .or_insert_with(|| PerPeerBackoffState::new_at(peer_id, now));
        state.record_outcome(false);
        let ceiling = state.ceiling();
        state.on_permanent_failure_at(now, ceiling);
    }

    /// Reset backoff state for a peer after successful connection.
    pub fn reset_on_connection_established(&self, addr_key: &str, peer_id: Option<PeerId>) {
        let now = self.now();
        let mut backoff = self.peer_backoff.write();
        let state = backoff
            .entry(addr_key.to_string())
            .or_insert_with(|| PerPeerBackoffState::new_at(peer_id, now));
        state.record_outcome(true);
        state.on_connection_established_at(now);
    }

    /// Reset backoff/dormant state for EVERY address entry belonging to `peer_id`.
    ///
    /// Backoff entries are keyed by address, but an INBOUND connection's remote
    /// address (the peer's ephemeral port) differs from the address we dialed,
    /// so the addr-keyed reset misses it. An established connection is proof
    /// of liveness regardless of transport path, so clear every entry
    /// attributed to this peer. Only entries whose stored peer_id matches and
    /// that are dormant or have failures are touched.
    pub fn reset_peer_backoff(&self, peer_id: PeerId) {
        let now = self.now();
        let mut backoff = self.peer_backoff.write();
        let mut reset_count = 0u32;
        for (key, state) in backoff.iter_mut() {
            if state.peer_id != Some(peer_id) {
                continue;
            }
            // An established connection is authenticated history for the
            // peer on every address entry, failing or not.
            state.authenticated_before = true;
            if state.dormant || state.attempt_count > 0 {
                state.record_outcome(true);
                state.on_connection_established_at(now);
                reset_count += 1;
                debug!(addr_key=%key, "[DIAL-POLICY] Peer-level liveness reset cleared backoff entry");
            }
        }
        if reset_count > 0 {
            info!(
                peer_id=%peer_id,
                entries_reset=reset_count,
                "[DIAL-POLICY] Cleared dial backoff on established connection"
            );
        }
    }

    /// Wake every address entry belonging to `peer_id`. Returns how many
    /// entries were waiting and are now due (after the floor gap).
    pub fn wake_peer(&self, peer_id: PeerId, trigger: WakeTrigger) -> usize {
        let now = self.now();
        let mut backoff = self.peer_backoff.write();
        // Authenticated history is a property of the peer: it may sit on a
        // sibling address entry (for example the inbound ephemeral address).
        let authenticated = backoff
            .values()
            .any(|s| s.peer_id == Some(peer_id) && s.authenticated_before);
        backoff
            .values_mut()
            .filter(|s| s.peer_id == Some(peer_id))
            .map(|s| s.wake_at_with(now, trigger, authenticated))
            .filter(|changed| *changed)
            .count()
    }

    /// Wake one address entry (for sightings that carry only an address,
    /// for example an inbound dial or a BLE/mDNS sighting).
    pub fn wake_addr(&self, addr_key: &str, trigger: WakeTrigger) -> bool {
        let now = self.now();
        let mut backoff = self.peer_backoff.write();
        backoff
            .get_mut(addr_key)
            .is_some_and(|s| s.wake_at(now, trigger))
    }

    /// Wake every waiting entry (network-change events from the discovery
    /// scheduler). Returns how many entries were woken.
    pub fn wake_all(&self, trigger: WakeTrigger) -> usize {
        let now = self.now();
        let mut backoff = self.peer_backoff.write();
        backoff
            .values_mut()
            .map(|s| s.wake_at(now, trigger))
            .filter(|changed| *changed)
            .count()
    }

    /// Consume a discovery-scheduler event: events that reflect a genuine
    /// LOCAL change (interface, radio, foreground, ledger/invite news) wake
    /// waiting peers. Peer-loss events do not (a loss is not a reason to
    /// expect reachability), and `NewPeerConnected` does not: a remote party
    /// (including a Sybil) can trigger it cheaply.
    pub fn on_network_event(&self, event: &super::discovery_scheduler::NetworkEvent) -> usize {
        use super::discovery_scheduler::NetworkEvent;
        match event {
            NetworkEvent::PeerDisconnected { .. }
            | NetworkEvent::AllPeersLost
            | NetworkEvent::NewPeerConnected => 0,
            NetworkEvent::BleStateChanged { on } if !*on => 0,
            NetworkEvent::LedgerReceived { new_entries } if *new_entries == 0 => 0,
            _ => self.wake_all(WakeTrigger::NetworkChange),
        }
    }

    /// Get the current backoff state for a peer (for diagnostics/testing).
    pub fn get_backoff_state(&self, addr_key: &str) -> Option<PerPeerBackoffState> {
        self.peer_backoff.read().get(addr_key).cloned()
    }

    /// Prune old backoff entries (e.g., peers we haven't seen in a long time).
    /// Useful for memory hygiene; a pruned peer is simply dialed fresh.
    pub fn prune_old_entries(&self, max_age: Duration) {
        let now = self.now();
        let mut backoff = self.peer_backoff.write();
        let mut concurrent = self.concurrent_dials.write();

        let stale_peers: Vec<String> = backoff
            .iter()
            .filter(|(_, state)| now.saturating_duration_since(state.last_attempt_ts) > max_age)
            .map(|(key, _)| key.clone())
            .collect();

        for peer_key in stale_peers {
            backoff.remove(&peer_key);
            concurrent.remove(&peer_key);
            debug!(peer_key=%peer_key, "[DIAL-POLICY] Pruned stale backoff entry");
        }
    }
}

impl Default for DialPolicyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility function to extract the address key from a Multiaddr (strip /p2p/ component).
pub fn multiaddr_to_key(addr: &Multiaddr) -> String {
    use libp2p::multiaddr::Protocol;
    let stripped: Multiaddr = addr
        .iter()
        .filter(|p| !matches!(p, Protocol::P2p(_)))
        .collect();
    stripped.to_string()
}

/// A known relay peer: its peer ID plus its external addresses.
type RelayEntry = (PeerId, Vec<Multiaddr>);

/// Circuit-relay ladder builder: adds relay addresses to a peer's dial candidates.
///
/// Once a peer is connected, we construct circuit-relay multiaddrs to that peer
/// through known relay peers. This improves connectivity for future dials.
pub struct CircuitRelayLadder {
    /// List of known relay peers (peer ID + their external addresses).
    relays: Arc<RwLock<Vec<RelayEntry>>>,
}

impl CircuitRelayLadder {
    /// Create a new circuit-relay ladder.
    pub fn new() -> Self {
        Self {
            relays: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a known relay peer with its external addresses.
    pub fn add_relay(&self, relay_peer_id: PeerId, external_addrs: Vec<Multiaddr>) {
        let mut relays = self.relays.write();

        // Remove any stale entry for this relay.
        relays.retain(|(pid, _)| pid != &relay_peer_id);

        debug!(
            relay_peer_id=%relay_peer_id,
            addr_count=external_addrs.len(),
            "[CIRCUIT-RELAY] Registered relay peer"
        );
        relays.push((relay_peer_id, external_addrs));
    }

    /// Remove a relay after its authenticated connection is gone.
    pub fn remove_relay(&self, relay_peer_id: &PeerId) {
        self.relays
            .write()
            .retain(|(peer_id, _)| peer_id != relay_peer_id);
    }

    /// Build a list of circuit-relay multiaddrs to a target peer through known relays.
    ///
    /// Returns a list of circuit-relay addresses in the format:
    /// `/ip4/<relay-ip>/tcp/<relay-port>/p2p/<relay-peer-id>/p2p-circuit/p2p/<target-peer-id>`
    pub fn build_relay_addresses(&self, target_peer_id: PeerId) -> Vec<Multiaddr> {
        use libp2p::multiaddr::Protocol;

        let relays = self.relays.read();
        let mut relay_addrs = HashSet::new();

        for (relay_pid, external_addrs) in relays.iter() {
            // A relay cannot provide a useful circuit to itself. More
            // importantly, accepting a self-target here creates a circuit
            // path that returns to the originating node and multiplies during
            // mesh growth.
            if relay_pid == &target_peer_id {
                continue;
            }
            for relay_addr in external_addrs {
                // Only use direct addresses with a proper IP and port. Identify
                // can repeat /p2p and /p2p-circuit components when a peer has
                // already used a relay; appending another circuit suffix would
                // create nested/self-returning routes.
                if relay_addr
                    .iter()
                    .any(|proto| matches!(proto, Protocol::P2pCircuit))
                {
                    continue;
                }
                // Preserve each transport component (IP, port, transport wrappers)
                // in direct_addr so the relay's concrete dialable prefix survives.
                // Rust match arms do not fall through; failing to push in the IP
                // and port arms strips the prefix and produces undialable addresses.
                let mut direct_addr = Multiaddr::empty();
                let mut has_ip = false;
                let mut has_port = false;
                for proto in relay_addr.iter() {
                    match proto {
                        Protocol::Ip4(_) | Protocol::Ip6(_) => {
                            has_ip = true;
                            direct_addr.push(proto);
                        }
                        Protocol::Tcp(_) | Protocol::Udp(_) => {
                            has_port = true;
                            direct_addr.push(proto);
                        }
                        Protocol::P2p(_) => {}
                        other => direct_addr.push(other),
                    }
                }

                if has_ip && has_port {
                    // Construct circuit-relay address: base -> /p2p/<relay> -> /p2p-circuit -> /p2p/<target>
                    let mut circuit_addr = direct_addr;
                    circuit_addr.push(Protocol::P2p(*relay_pid));
                    circuit_addr.push(Protocol::P2pCircuit);
                    circuit_addr.push(Protocol::P2p(target_peer_id));
                    relay_addrs.insert(circuit_addr);
                }
            }
        }

        if !relay_addrs.is_empty() {
            debug!(
                target_peer_id=%target_peer_id,
                relay_count=relay_addrs.len(),
                "[CIRCUIT-RELAY] Built relay addresses for target"
            );
        }

        relay_addrs.into_iter().collect()
    }
}

impl Default for CircuitRelayLadder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_state_creation() {
        let state = PerPeerBackoffState::new(None);
        assert_eq!(state.attempt_count, 0);
        assert_eq!(state.backoff_duration, Duration::from_secs(1));
        assert!(!state.dormant);
        assert!(state.is_eligible());
    }

    #[test]
    fn test_exponential_backoff_progression() {
        let mut state = PerPeerBackoffState::new(None);

        // 1st failure: 1s -> 2s
        state.on_dial_failure();
        assert_eq!(state.attempt_count, 1);
        assert_eq!(state.backoff_duration, Duration::from_secs(2));

        // 2nd failure: 2s -> 4s
        state.on_dial_failure();
        assert_eq!(state.attempt_count, 2);
        assert_eq!(state.backoff_duration, Duration::from_secs(4));
        assert!(!state.dormant);

        // 3rd failure: 4s -> 8s, peer goes Dormant (not dead).
        state.on_dial_failure();
        assert_eq!(state.attempt_count, 3);
        assert_eq!(state.backoff_duration, Duration::from_secs(8));
        assert!(state.dormant);
    }

    #[test]
    fn test_backoff_cap_at_ceiling_and_never_stops() {
        let mut state = PerPeerBackoffState::new(None);
        for _ in 0..50 {
            state.on_dial_failure();
        }
        // Default ceiling is the base; the backoff never exceeds it and the
        // failure counter keeps counting (no give-up threshold).
        assert!(state.backoff_duration <= BACKOFF_CEILING_BASE);
        assert_eq!(state.attempt_count, 50);
    }

    #[test]
    fn test_jittered_backoff_within_bounds() {
        let t0 = Instant::now();
        for _ in 0..200 {
            let mut state = PerPeerBackoffState::new_at(None, t0);
            state.on_dial_failure_at(t0, BACKOFF_CEILING_BASE);
            // nominal 2s, jitter uniform(0.5, 1.0) => wait in [1s, 2s].
            let wait = state.next_attempt_at.duration_since(t0);
            assert!(wait >= Duration::from_secs(1), "wait too short: {wait:?}");
            assert!(wait <= Duration::from_secs(2), "wait too long: {wait:?}");
        }
    }

    #[test]
    fn test_ceiling_follows_stability() {
        let stable = ceiling_for_success_rate(1.0);
        let flaky = ceiling_for_success_rate(0.0);
        assert_eq!(stable, BACKOFF_CEILING_BASE);
        assert_eq!(flaky, BACKOFF_CEILING_BASE * 4);
        assert!(ceiling_for_success_rate(0.5) > stable);
        assert!(ceiling_for_success_rate(0.5) < flaky);
        assert_eq!(
            ceiling_for_success_rate(f64::NAN),
            ceiling_for_success_rate(0.5)
        );
    }

    #[test]
    fn test_eligibility_check() {
        let state = PerPeerBackoffState::new(None);
        assert!(state.is_eligible()); // Initially eligible

        let mut state = PerPeerBackoffState::new(None);
        state.on_dial_failure();
        state.on_dial_failure();
        state.on_dial_failure();
        // Dormant: waiting for a wake event or its jittered backoff.
        assert!(state.dormant);
        assert!(!state.is_eligible());
        // The backoff alone brings it back; it is never permanently out.
        assert!(state.is_eligible_at(state.next_attempt_at));
    }

    #[test]
    fn test_connection_established_reset() {
        let mut state = PerPeerBackoffState::new(None);
        state.on_dial_failure();
        state.on_dial_failure();
        assert_eq!(state.attempt_count, 2);

        state.on_connection_established();
        assert_eq!(state.attempt_count, 0);
        assert_eq!(state.backoff_duration, Duration::from_secs(1));
        assert!(!state.dormant);
        assert!(state.is_eligible());
    }

    #[test]
    fn test_permanent_failure_goes_dormant_not_dead() {
        let mut state = PerPeerBackoffState::new(None);
        let t0 = Instant::now();
        state.on_permanent_failure_at(t0, BACKOFF_CEILING_BASE);
        assert!(state.dormant);
        assert_eq!(state.attempt_count, 3);
        assert!(!state.is_eligible_at(t0));
        // Backoff expiry alone revives eligibility.
        assert!(state.is_eligible_at(t0 + BACKOFF_CEILING_BASE));
    }

    #[test]
    fn test_wake_pulls_next_attempt_forward_and_clears_dormant() {
        let t0 = Instant::now();
        let mut state = PerPeerBackoffState::new_at(None, t0);
        for _ in 0..8 {
            state.on_dial_failure_at(t0, BACKOFF_CEILING_BASE);
        }
        assert!(state.dormant);
        assert!(!state.is_eligible_at(t0 + BACKOFF_FLOOR));

        let ladder_before = state.backoff_duration;
        let attempts_before = state.attempt_count;
        assert!(state.wake_at(t0, WakeTrigger::Identify));
        assert!(!state.dormant);
        // One-shot credit: the ladder is NOT reset.
        assert_eq!(state.backoff_duration, ladder_before);
        assert_eq!(state.attempt_count, attempts_before);
        // Due after the floor gap, not before: wake storms stay bounded.
        assert!(!state.is_eligible_at(t0));
        assert!(state.is_eligible_at(t0 + BACKOFF_FLOOR));

        // A fresh entry has nothing to wake.
        let mut fresh = PerPeerBackoffState::new_at(None, t0);
        assert!(!fresh.wake_at(t0, WakeTrigger::Identify));
    }

    #[test]
    fn test_manager_unreachable_for_hours_is_redialed_on_wake() {
        let manager = DialPolicyManager::new();
        let key = "/ip4/10.0.0.9/tcp/9000";
        assert!(manager.register_dial_attempt(key, None));
        manager.complete_dial_attempt(key);

        // Unreachable "for hours": a failure, then a long fake-clock stretch
        // with a failed attempt each time it is due.
        let mut attempts = 0u32;
        for _ in 0..200 {
            manager.record_dial_failure(key, None);
            manager.advance_clock(manager.ceiling_for(key)); // backoff alone suffices
            assert!(
                manager.register_dial_attempt(key, None),
                "backoff alone must always bring the peer back"
            );
            manager.complete_dial_attempt(key);
            attempts += 1;
        }
        assert_eq!(attempts, 200);
        let st = manager.get_backoff_state(key).expect("state");
        assert!(st.dormant);
        assert!(st.backoff_duration <= manager.ceiling_for(key));

        // Now backed off with the peer unreachable; a wake event makes it
        // eligible again at the floor, long before the ceiling.
        manager.record_dial_failure(key, None);
        assert!(!manager.register_dial_attempt(key, None));
        assert!(manager.wake_addr(key, WakeTrigger::InboundDial));
        manager.advance_clock(BACKOFF_FLOOR);
        assert!(manager.register_dial_attempt(key, None));
    }

    #[test]
    fn test_attempts_per_minute_bounded_under_wake_flood() {
        let manager = DialPolicyManager::new();
        let key = "/ip4/10.0.0.10/tcp/9000";
        let mut attempts = 0u32;
        // Simulate 60 s in 100 ms steps; every step wakes the peer and every
        // granted attempt fails.
        for _ in 0..600 {
            manager.wake_addr(key, WakeTrigger::NetworkChange);
            if manager.register_dial_attempt(key, None) {
                manager.complete_dial_attempt(key);
                manager.record_dial_failure(key, None);
                attempts += 1;
            }
            manager.advance_clock(Duration::from_millis(100));
        }
        // Floor of 1 s between attempts => at most 60 per minute (+1 for the
        // initial immediate attempt).
        assert!(
            attempts <= 61,
            "attempts per minute not bounded: {attempts}"
        );
        assert!(attempts >= 1);
    }

    #[test]
    fn test_wake_is_one_shot_credit_ladder_resumes_after_failed_attempt() {
        let t0 = Instant::now();
        let mut state = PerPeerBackoffState::new_at(None, t0);
        for _ in 0..8 {
            state.on_dial_failure_at(t0, BACKOFF_CEILING_BASE);
        }
        let ladder = state.backoff_duration;
        assert!(state.wake_at(t0, WakeTrigger::Identify));
        let t1 = t0 + BACKOFF_FLOOR;
        assert!(state.is_eligible_at(t1));
        // The woken attempt fails: the next wait follows the preserved
        // ladder (at least half the nominal interval), not the floor.
        state.on_dial_failure_at(t1, BACKOFF_CEILING_BASE);
        let wait = state.next_attempt_at.duration_since(t1);
        assert!(state.backoff_duration >= ladder);
        assert!(wait >= ladder.mul_f64(0.5), "ladder was reset: {wait:?}");
    }

    #[test]
    fn test_wake_bucket_limits_a_flood_per_source() {
        let t0 = Instant::now();
        let mut state = PerPeerBackoffState::new_at(None, t0);
        for _ in 0..8 {
            state.on_dial_failure_at(t0, BACKOFF_CEILING_BASE);
        }
        let ceiling = state.ceiling();
        let min_period = (ceiling / 4).max(BACKOFF_FLOOR);
        let mut admitted = 0u64;
        // 100 ms flood for 200 s.
        for i in 0..2000u32 {
            let now = t0 + Duration::from_millis(100) * i;
            if state.wake_at(now, WakeTrigger::NetworkChange) {
                admitted += 1;
            }
        }
        let allowed =
            1 + (Duration::from_secs(200).as_secs_f64() / min_period.as_secs_f64()) as u64;
        assert!(admitted >= 1);
        assert!(
            admitted <= allowed,
            "admitted {admitted} > allowed {allowed}"
        );
        // Sources are independent: another trigger still has its own credit.
        assert!(state.wake_at(t0 + Duration::from_secs(200), WakeTrigger::Identify));
    }

    #[test]
    fn test_rare_wake_source_is_always_admitted() {
        let t0 = Instant::now();
        let mut state = PerPeerBackoffState::new_at(None, t0);
        for _ in 0..4 {
            state.on_dial_failure_at(t0, BACKOFF_CEILING_BASE);
        }
        let ceiling = state.ceiling();
        for i in 0..5u32 {
            let now = t0 + ceiling * (i + 1);
            assert!(state.wake_at(now, WakeTrigger::NetworkChange), "event {i}");
        }
    }

    #[test]
    fn test_mdns_wake_needs_authenticated_history() {
        use libp2p::identity::Keypair;
        let manager = DialPolicyManager::new();
        let stranger = Keypair::generate_ed25519().public().to_peer_id();
        let known = Keypair::generate_ed25519().public().to_peer_id();
        let stranger_key = "/ip4/192.168.1.90/tcp/4001";
        let known_key = "/ip4/192.168.1.91/tcp/4001";
        for _ in 0..5 {
            manager.record_dial_failure(stranger_key, Some(stranger));
        }
        // `known` connected once (authenticated), then went away.
        manager.reset_on_connection_established(known_key, Some(known));
        for _ in 0..5 {
            manager.record_dial_failure(known_key, Some(known));
        }
        assert_eq!(
            manager.wake_peer(stranger, WakeTrigger::MdnsDiscovered),
            0,
            "unauthenticated mDNS claim must not wake"
        );
        assert!(
            manager
                .get_backoff_state(stranger_key)
                .expect("state")
                .dormant
        );
        assert_eq!(manager.wake_peer(known, WakeTrigger::MdnsDiscovered), 1);
    }

    #[test]
    fn test_authenticated_history_on_sibling_entry_counts_for_mdns() {
        use libp2p::identity::Keypair;
        let manager = DialPolicyManager::new();
        let pid = Keypair::generate_ed25519().public().to_peer_id();
        let dialed = "/ip4/192.168.1.60/tcp/4001";
        for _ in 0..5 {
            manager.record_dial_failure(dialed, Some(pid));
        }
        // Inbound connection from an ephemeral address proves the peer.
        manager.reset_peer_backoff(pid);
        for _ in 0..5 {
            manager.record_dial_failure(dialed, Some(pid));
        }
        assert_eq!(manager.wake_peer(pid, WakeTrigger::MdnsDiscovered), 1);
    }

    #[test]
    fn test_dead_address_does_not_stretch_other_addresses_ceiling() {
        let manager = DialPolicyManager::new();
        for _ in 0..40 {
            manager.record_dial_failure("/ip4/203.0.113.9/tcp/1", None);
        }
        let fresh = ceiling_for_success_rate(STABILITY_PRIOR);
        assert!(manager.ceiling_for("/ip4/203.0.113.9/tcp/1") > fresh);
        assert_eq!(manager.ceiling_for("/ip4/198.51.100.2/tcp/1"), fresh);
        manager.record_dial_failure("/ip4/198.51.100.2/tcp/1", None);
        assert!(
            manager.ceiling_for("/ip4/198.51.100.2/tcp/1")
                < manager.ceiling_for("/ip4/203.0.113.9/tcp/1")
        );
    }

    #[test]
    fn test_each_wake_trigger_wakes_a_dormant_peer() {
        use libp2p::identity::Keypair;
        let triggers = [
            WakeTrigger::InboundConnection,
            WakeTrigger::InboundDial,
            WakeTrigger::Identify,
            WakeTrigger::AddressLearned,
            WakeTrigger::NetworkChange,
            WakeTrigger::OutboundQueued,
        ];
        for trigger in triggers {
            let manager = DialPolicyManager::new();
            let pid = Keypair::generate_ed25519().public().to_peer_id();
            let key = "/ip4/192.168.1.77/tcp/4001";
            for _ in 0..6 {
                manager.record_dial_failure(key, Some(pid));
            }
            assert!(manager.get_backoff_state(key).expect("state").dormant);
            assert!(!manager.register_dial_attempt(key, Some(pid)));

            assert_eq!(manager.wake_peer(pid, trigger), 1, "{}", trigger.as_str());
            assert!(!manager.get_backoff_state(key).expect("state").dormant);
            manager.advance_clock(BACKOFF_FLOOR);
            assert!(
                manager.register_dial_attempt(key, Some(pid)),
                "trigger {} did not make the peer dialable",
                trigger.as_str()
            );
        }
    }

    #[test]
    fn test_network_event_wakes_all_but_peer_loss_does_not() {
        use super::super::discovery_scheduler::NetworkEvent;
        let manager = DialPolicyManager::new();
        for k in ["a", "b", "c"] {
            for _ in 0..4 {
                manager.record_dial_failure(k, None);
            }
        }
        assert_eq!(
            manager.on_network_event(&NetworkEvent::PeerDisconnected { count_now: 2 }),
            0
        );
        assert_eq!(manager.on_network_event(&NetworkEvent::AllPeersLost), 0);
        // A remote party (Sybil) can trigger this cheaply: never a wake.
        assert_eq!(manager.on_network_event(&NetworkEvent::NewPeerConnected), 0);
        assert_eq!(
            manager.on_network_event(&NetworkEvent::LedgerReceived { new_entries: 0 }),
            0
        );
        assert_eq!(manager.on_network_event(&NetworkEvent::WifiChanged), 3);
        for k in ["a", "b", "c"] {
            assert!(!manager.get_backoff_state(k).expect("state").dormant);
        }
    }

    #[test]
    fn test_record_permanent_failure_is_not_terminal() {
        let manager = DialPolicyManager::new();
        let key = "10.0.0.1:4001";
        manager.record_permanent_failure(key, None);
        assert!(!manager.register_dial_attempt(key, None));
        assert!(manager.get_backoff_state(key).expect("state").dormant);
        manager.advance_clock(manager.ceiling_for(key));
        assert!(manager.register_dial_attempt(key, None));
    }
    #[test]
    fn test_dial_policy_manager_registration() {
        let manager = DialPolicyManager::new();

        // First dial should succeed.
        assert!(manager.register_dial_attempt("addr1", None));

        // Can register multiple dials to the same peer (up to 3).
        assert!(manager.register_dial_attempt("addr1", None));
        assert!(manager.register_dial_attempt("addr1", None));

        // 4th dial should fail (concurrent limit).
        assert!(!manager.register_dial_attempt("addr1", None));
    }

    #[test]
    fn test_concurrent_dial_limit() {
        let manager = DialPolicyManager::new();

        let addr = "peer1";

        // Register 3 concurrent dials.
        assert!(manager.register_dial_attempt(addr, None));
        assert!(manager.register_dial_attempt(addr, None));
        assert!(manager.register_dial_attempt(addr, None));

        // 4th should fail.
        assert!(!manager.register_dial_attempt(addr, None));

        // After completing one, we can register another.
        manager.complete_dial_attempt(addr);
        assert!(manager.register_dial_attempt(addr, None));
    }

    #[test]
    fn test_backoff_eligibility() {
        let manager = DialPolicyManager::new();
        let addr = "peer1";

        // First dial succeeds.
        assert!(manager.register_dial_attempt(addr, None));
        manager.complete_dial_attempt(addr);

        // After failure, backoff should prevent immediate re-dial.
        manager.record_dial_failure(addr, None);
        assert!(!manager.register_dial_attempt(addr, None));
    }

    #[test]
    fn test_reset_peer_backoff_clears_dormant_state_on_any_addr_entry() {
        use libp2p::identity::Keypair;

        let manager = DialPolicyManager::new();
        let pid = Keypair::generate_ed25519().public().to_peer_id();

        // Peer goes Dormant after repeated failures on its dialed LAN address.
        let lan_addr = "/ip4/192.168.1.50/tcp/4001";
        for _ in 0..3 {
            manager.record_dial_failure(lan_addr, Some(pid));
        }
        assert!(!manager.register_dial_attempt(lan_addr, Some(pid)));

        // An INBOUND connection arrives from an ephemeral remote address:
        // resetting only that address must NOT touch the dormant LAN entry.
        let ephemeral = "/ip4/192.168.1.50/tcp/51234";
        manager.reset_on_connection_established(ephemeral, Some(pid));
        assert!(!manager.register_dial_attempt(lan_addr, Some(pid)));

        // Peer-wide liveness reset (what ConnectionEstablished does) must.
        manager.reset_peer_backoff(pid);
        let state = manager.get_backoff_state(lan_addr).expect("entry exists");
        assert_eq!(state.attempt_count, 0);
        assert!(!state.dormant);
        assert!(manager.register_dial_attempt(lan_addr, Some(pid)));
    }
    #[test]
    fn test_circuit_relay_ladder() {
        let ladder = CircuitRelayLadder::new();

        // Create a mock relay with some addresses.
        let relay_pid = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let relay_addr: Multiaddr = "/ip4/192.168.1.100/tcp/4001".parse().unwrap();
        ladder.add_relay(relay_pid, vec![relay_addr]);

        // Build relay addresses for a target peer.
        let target_pid = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let relay_addresses = ladder.build_relay_addresses(target_pid);

        assert!(!relay_addresses.is_empty());
        // Check that the circuit relay address contains both relay and target peer IDs.
        let addr_str = relay_addresses[0].to_string();
        assert!(addr_str.starts_with("/ip4/192.168.1.100/tcp/4001"));
        assert!(addr_str.contains("/p2p-circuit/"));
    }

    #[test]
    fn circuit_relay_ladder_preserves_transport_prefix() {
        let ladder = CircuitRelayLadder::new();
        let relay_pid = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let target_pid = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();

        let prefix = "/ip4/192.168.1.100/tcp/4001";
        let relay_addr: Multiaddr = prefix.parse().unwrap();
        ladder.add_relay(relay_pid, vec![relay_addr]);

        let relay_addresses = ladder.build_relay_addresses(target_pid);
        assert_eq!(relay_addresses.len(), 1);

        let addr_str = relay_addresses[0].to_string();
        let expected = format!("{prefix}/p2p/{relay_pid}/p2p-circuit/p2p/{target_pid}");
        assert_eq!(addr_str, expected);
        assert!(
            addr_str.starts_with(prefix),
            "relay circuit address must preserve concrete transport prefix, got: {addr_str}"
        );
    }

    #[test]
    fn circuit_relay_ladder_rejects_nested_and_self_targeted_routes() {
        let ladder = CircuitRelayLadder::new();
        let relay_pid = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let other_relay_pid = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let target_pid = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();

        ladder.add_relay(
            relay_pid,
            vec![
                format!("/ip4/192.168.1.100/tcp/4001/p2p/{relay_pid}/p2p-circuit/p2p/{target_pid}")
                    .parse()
                    .expect("nested relay fixture is valid"),
                "/ip4/192.168.1.101/tcp/4001"
                    .parse()
                    .expect("direct relay fixture is valid"),
            ],
        );
        ladder.add_relay(
            other_relay_pid,
            vec!["/ip4/192.168.1.102/tcp/4001"
                .parse()
                .expect("direct relay fixture is valid")],
        );

        let routes = ladder.build_relay_addresses(target_pid);
        assert_eq!(routes.len(), 2);
        assert!(routes
            .iter()
            .all(|addr| { addr.to_string().matches("/p2p-circuit/").count() == 1 }));
        assert!(ladder.build_relay_addresses(relay_pid).iter().all(|addr| {
            !addr
                .to_string()
                .contains(&format!("/p2p/{relay_pid}/p2p-circuit/p2p/{relay_pid}"))
        }));
    }

    #[test]
    fn test_multiaddr_to_key() {
        let pid = libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let addr_str = format!("/ip4/192.168.1.1/tcp/4001/p2p/{}", pid);
        let addr: Multiaddr = addr_str.parse().unwrap();
        let key = multiaddr_to_key(&addr);
        assert!(!key.contains("/p2p/"));
        assert!(key.contains("192.168.1.1"));
        assert!(key.contains("4001"));
    }
}
