// Per-peer backoff state machine for graceful dial policy.
//
// This module implements P1 Item 3: Per-Peer Backoff State Machine (max 3 concurrent dials)
// and P1 Item 4: Prefer Circuit-Relay After Connection Established.
//
// Philosophy: Each peer maintains attempt_count, last_attempt_ts, and backoff_duration.
// The global dial orchestrator enforces max 3 concurrent outbound dials. Exponential
// backoff ranges from 1s to 30s (capped). On successful connection, backoff resets.
//
// Circuit-relay preference: Once a peer connects, we add circuit-relay multiaddrs
// to the candidate ladder in order: direct → relay → fallback.

use libp2p::{Multiaddr, PeerId};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::Arc;
use tracing::{debug, info, warn};
use web_time::{Duration, Instant};

/// How long a peer stays dead after 3 failed dial attempts before the
/// dial-policy auto-revives it. "Dead" is a bounded backoff state, not a
/// lifetime sentence: a peer that was down and comes back must be retried
/// within a minute (bootstrap sweep cadence), not held out until a
/// ConnectionEstablished/liveness event or the 1-hour hygiene prune.
/// 2026-09-03: 3-node validation showed the 5-minute dead cycle -- secondary
/// address failures dead-marked a peer whose live path identify kept
/// confirming. Fix A/B/C stop dead-marks on live peers; this window bounds
/// the dead state for genuinely unreachable peers.
///
/// Anti-hammer bound (review A3, 2026-09-03): a revive is NOT a free dial
/// burst. A revived entry starts from zero strikes, but the FIRST failure
/// immediately re-applies the 1s/2s/4s backoff ladder and the 3rd strike
/// re-marks it dead -- so a genuinely unreachable address gets at most ~3
/// attempts within the seconds after a revive, then stays dead until the
/// next 60s window. Worst case is ~3 dial attempts/minute per dead address,
/// and the window itself is a single shared constant used by both
/// `is_eligible` (read) and `maybe_revive` (mutate), so no path can observe
/// a different revive predicate.
pub const DEAD_REVIVE_AFTER: Duration = Duration::from_secs(60);

/// Per-peer backoff state tracking.
#[derive(Debug, Clone)]
pub struct PerPeerBackoffState {
    /// Number of failed dial attempts (0-3). At 3, peer is considered dead.
    pub attempt_count: u32,
    /// Timestamp of the last dial attempt to this peer.
    pub last_attempt_ts: Instant,
    /// Current backoff duration (1s → 2s → 4s → 8s → 16s → 30s capped).
    pub backoff_duration: Duration,
    /// Whether this peer is marked as dead (bounded: auto-revives after
    /// [`DEAD_REVIVE_AFTER`]).
    pub is_dead: bool,
    /// When the dead mark was applied; `None` when not dead. Drives the
    /// bounded auto-revive window.
    pub dead_since: Option<Instant>,
    /// Optional peer ID if known at registration time.
    pub peer_id: Option<PeerId>,
}

impl PerPeerBackoffState {
    /// Create a new backoff state with initial backoff of 1 second.
    pub fn new(peer_id: Option<PeerId>) -> Self {
        Self {
            attempt_count: 0,
            last_attempt_ts: Instant::now(),
            backoff_duration: Duration::from_secs(1),
            is_dead: false,
            dead_since: None,
            peer_id,
        }
    }

    /// Check if this peer is eligible for a dial attempt right now.
    ///
    /// Dead is bounded: once the revive window has elapsed the entry reads as
    /// eligible again (the caller then dials; the persistent state is revived
    /// by [`Self::maybe_revive`] inside `register_dial_attempt`).
    pub fn is_eligible(&self) -> bool {
        if self.is_dead {
            return self
                .dead_since
                .is_some_and(|since| since.elapsed() >= DEAD_REVIVE_AFTER);
        }
        if self.attempt_count >= 3 {
            return false;
        }
        // Allow the first attempt immediately.
        if self.attempt_count == 0 {
            return true;
        }
        Instant::now() >= self.last_attempt_ts + self.backoff_duration
    }

    /// Revive a dead entry whose window has elapsed, resetting strike count
    /// and backoff. Returns true when a revive actually happened.
    pub fn maybe_revive(&mut self) -> bool {
        if self.is_dead {
            if let Some(since) = self.dead_since {
                if since.elapsed() >= DEAD_REVIVE_AFTER {
                    self.is_dead = false;
                    self.dead_since = None;
                    self.attempt_count = 0;
                    self.backoff_duration = Duration::from_secs(1);
                    self.last_attempt_ts = Instant::now();
                    debug!(
                        peer_id=?self.peer_id,
                        "[DIAL-BACKOFF] Dead entry auto-revived after revive window"
                    );
                    return true;
                }
            }
        }
        false
    }

    /// Record a failed dial attempt: increment attempt_count and double backoff (capped at 30s).
    pub fn on_dial_failure(&mut self) {
        self.attempt_count += 1;
        self.last_attempt_ts = Instant::now();

        // Double the backoff duration, capped at 30 seconds.
        let doubled = self.backoff_duration.as_secs() * 2;
        self.backoff_duration = Duration::from_secs(doubled.min(30));

        debug!(
            peer_id=?self.peer_id,
            attempt_count=self.attempt_count,
            backoff_secs=self.backoff_duration.as_secs(),
            "[DIAL-BACKOFF] Incremented attempt count and backoff"
        );

        // After 3 attempts, mark as dead (bounded by DEAD_REVIVE_AFTER).
        if self.attempt_count >= 3 {
            warn!(
                peer_id=?self.peer_id,
                "[DIAL-BACKOFF] Peer marked as dead after 3 failed attempts"
            );
            self.is_dead = true;
            self.dead_since = Some(Instant::now());
        }
    }

    /// Record a permanent dial failure (mark peer as dead immediately).
    pub fn on_permanent_failure(&mut self) {
        self.is_dead = true;
        self.dead_since = Some(Instant::now());
        self.attempt_count = 3;
        warn!(
            peer_id=?self.peer_id,
            "[DIAL-BACKOFF] Peer marked as dead due to permanent failure"
        );
    }

    /// Reset backoff state on successful connection.
    pub fn on_connection_established(&mut self) {
        let old_attempt_count = self.attempt_count;
        self.attempt_count = 0;
        self.backoff_duration = Duration::from_secs(1);
        self.last_attempt_ts = Instant::now();
        self.is_dead = false;
        self.dead_since = None;

        info!(
            peer_id=?self.peer_id,
            prev_attempt_count=old_attempt_count,
            "[DIAL-BACKOFF] Reset backoff state after successful connection"
        );
    }
}

/// Global dial policy manager: tracks per-peer backoff state and enforces
/// concurrent dial limits (max 3 concurrent outbound dials to any peer).
#[derive(Debug, Clone)]
pub struct DialPolicyManager {
    /// Per-peer backoff state, keyed by peer address (stripped of /p2p/).
    /// Using String as key to handle addresses without peer IDs.
    peer_backoff: Arc<RwLock<HashMap<String, PerPeerBackoffState>>>,
    /// Count of in-flight (queued but not yet connected/failed) dials to each peer.
    /// Used to enforce max 3 concurrent dials per peer.
    concurrent_dials: Arc<RwLock<HashMap<String, u32>>>,
}

impl DialPolicyManager {
    /// Create a new dial policy manager.
    pub fn new() -> Self {
        Self {
            peer_backoff: Arc::new(RwLock::new(HashMap::new())),
            concurrent_dials: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register the start of a dial attempt to a peer address.
    /// Returns true if the dial is allowed (backoff eligible + under concurrent limit).
    /// Returns false if the peer is backed off or at the concurrent dial limit.
    pub fn register_dial_attempt(&self, addr_key: &str, peer_id: Option<PeerId>) -> bool {
        let mut backoff = self.peer_backoff.write();
        let mut concurrent = self.concurrent_dials.write();

        // Ensure the peer has a backoff state entry.
        let state = backoff.entry(addr_key.to_string()).or_insert_with(|| {
            debug!(addr_key=%addr_key, "[DIAL-POLICY] Registering new peer backoff state");
            PerPeerBackoffState::new(peer_id)
        });

        // Bounded dead state: once the revive window has elapsed, clear the
        // dead mark so a peer that came back is dialed again (nimble
        // recovery instead of session-long exclusion).
        state.maybe_revive();

        // Check eligibility: not dead, attempt_count < 3, backoff elapsed.
        if !state.is_eligible() {
            debug!(
                addr_key=%addr_key,
                attempt_count=state.attempt_count,
                is_dead=state.is_dead,
                backoff_secs=state.backoff_duration.as_secs(),
                "[DIAL-POLICY] Peer is not eligible for dial attempt (backed off or dead)"
            );
            return false;
        }

        // Check concurrent dial limit (max 3 per peer).
        let dial_count = concurrent.entry(addr_key.to_string()).or_insert(0);
        if *dial_count >= 3 {
            debug!(
                addr_key=%addr_key,
                current_concurrent=*dial_count,
                "[DIAL-POLICY] Peer at concurrent dial limit (3/3)"
            );
            return false;
        }

        // Increment concurrent dial count and return success.
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
    /// This increments the attempt count and applies exponential backoff.
    pub fn record_dial_failure(&self, addr_key: &str, peer_id: Option<PeerId>) {
        let mut backoff = self.peer_backoff.write();
        let state = backoff
            .entry(addr_key.to_string())
            .or_insert_with(|| PerPeerBackoffState::new(peer_id));
        state.on_dial_failure();
    }

    /// Record a permanent dial failure for a peer address.
    /// This marks the peer as dead for this session (no retry).
    pub fn record_permanent_failure(&self, addr_key: &str, peer_id: Option<PeerId>) {
        let mut backoff = self.peer_backoff.write();
        let state = backoff
            .entry(addr_key.to_string())
            .or_insert_with(|| PerPeerBackoffState::new(peer_id));
        state.on_permanent_failure();
    }

    /// Reset backoff state for a peer after successful connection.
    pub fn reset_on_connection_established(&self, addr_key: &str, peer_id: Option<PeerId>) {
        let mut backoff = self.peer_backoff.write();
        let state = backoff
            .entry(addr_key.to_string())
            .or_insert_with(|| PerPeerBackoffState::new(peer_id));
        state.on_connection_established();
    }

    /// Reset backoff/dead state for EVERY address entry belonging to `peer_id`.
    ///
    /// Backoff entries are keyed by address, but an INBOUND connection's remote
    /// address (the peer's ephemeral port) differs from the address we dialed
    /// and marked dead — so the addr-keyed reset misses it. An established
    /// connection is proof of liveness regardless of transport path, so clear
    /// every entry attributed to this peer.
    ///
    /// Deliberate scope (review A1, 2026-09-03): the reset clears address-scoped
    /// dead marks when ANY path proves the peer is alive, rather than only the
    /// address that showed liveness. The alternative (keeping a stale address
    /// dead while the peer is demonstrably up) is exactly the 5-minute dead
    /// cycle being fixed -- a NAT-reflected address's dead mark suppressed
    /// hint-dials and relay pulls for a peer whose live path was fine. Cost of
    /// the broad reset: at most one dial attempt per stale address per revive
    /// window, immediately re-escalated by the failure path. Bounded: only
    /// entries whose stored peer_id matches, and only entries with is_dead or
    /// attempt_count > 0.
    pub fn reset_peer_backoff(&self, peer_id: PeerId) {
        let mut backoff = self.peer_backoff.write();
        let mut reset_count = 0u32;
        for (key, state) in backoff.iter_mut() {
            if state.peer_id == Some(peer_id) && (state.is_dead || state.attempt_count > 0) {
                state.on_connection_established();
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

    /// Get the current backoff state for a peer (for diagnostics/testing).
    pub fn get_backoff_state(&self, addr_key: &str) -> Option<PerPeerBackoffState> {
        self.peer_backoff.read().get(addr_key).cloned()
    }

    /// Prune old backoff entries (e.g., peers we haven't seen in a long time).
    /// Useful for memory hygiene.
    pub fn prune_old_entries(&self, max_age: Duration) {
        let now = Instant::now();
        let mut backoff = self.peer_backoff.write();
        let mut concurrent = self.concurrent_dials.write();

        let stale_peers: Vec<String> = backoff
            .iter()
            .filter(|(_, state)| now.duration_since(state.last_attempt_ts) > max_age)
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

/// Cap on relay records one node tracks (connected or not).
pub const RELAY_TRACKING_CAP: usize = 16;

/// Cap on relay CIRCUIT ADDRESSES EMITTED for ONE target peer, so the
/// synthesised dial ladder (3 direct ports + 1 last-good + these) stays far
/// inside the per-peer admission ceiling (`behaviour::MAX_ESTABLISHED_PER_PEER`).
pub const MAX_RELAY_LADDER_ADDRS: usize = 4;

/// Addresses remembered per relay. A relay that advertises hundreds of
/// listen addresses cannot widen the ladder or the table.
pub const MAX_ADDRS_PER_RELAY: usize = 4;

/// Identity slots: relays tracked per network group (IPv4 /24, IPv6 /48 of the
/// IP address the relay's AUTHENTICATED CONNECTION was observed from, never
/// an address the relay advertises about itself). A Sybil minting unlimited
/// peer ids from one network gets this many slots in total, not one per id.
/// Private, loopback, link-local and shared-address ranges carry no such
/// meaning (the same /24 is a different LAN on every site) and are exempt.
pub const MAX_RELAYS_PER_NETWORK_GROUP: usize = 2;

/// Re-registration of an already-tracked relay (identify re-fires every 60 s
/// per connection) is ignored inside this interval: it neither refreshes the
/// address list nor touches the relay's rank.
pub const RELAY_REREGISTER_MIN_INTERVAL: Duration = Duration::from_secs(300);

/// Admission of NEW relay identities is rate limited: at most
/// `NEW_RELAY_ADMISSIONS_PER_WINDOW` per `NEW_RELAY_WINDOW`.
pub const NEW_RELAY_ADMISSIONS_PER_WINDOW: usize = 4;
pub const NEW_RELAY_WINDOW: Duration = Duration::from_secs(60);

/// Of the admission budget, this many slots per window are held back for
/// identities with LOCALLY PROVEN history (see `record_proven_relay`). A
/// stream of fresh unproven identities can use at most
/// `NEW_RELAY_ADMISSIONS_PER_WINDOW - NEW_RELAY_PROVEN_RESERVED` of them, so
/// it cannot starve a returning relay that has earned its place.
pub const NEW_RELAY_PROVEN_RESERVED: usize = 2;

/// Peer ids remembered as having locally proven history after their record
/// was evicted (oldest forgotten first).
const PROVEN_HISTORY_CAP: usize = 64;

/// Longevity credit saturates here, so score cannot grow without bound.
const RELAY_UPTIME_CAP: Duration = Duration::from_secs(7 * 24 * 3600);

/// Each LOCALLY PROVEN relay service is worth this many seconds of uptime in
/// the score. Deliberately small: the signal is weaker than time served, and
/// a relay colluding with a peer it controls can manufacture it, so it only
/// breaks ties between relays of similar age. Self-asserted claims (a
/// `RelayResponse.accepted` flag, a reservation grant) are NOT a proof and are
/// never credited.
const RELAY_SUCCESS_WEIGHT_SECS: u64 = 60;
/// Credits counted per relay. With the weight above the proven component can
/// never exceed one hour of uptime-equivalent, against a seven day uptime cap.
const RELAY_SUCCESS_CAP: u32 = 60;
/// Credits for one relay are at least this far apart: a burst of proofs
/// counts once, so the cap cannot be reached quickly.
pub const RELAY_SUCCESS_MIN_INTERVAL: Duration = Duration::from_secs(30);
/// The proven component decays linearly to zero this long after the latest
/// credit: reputation must be re-earned, not banked forever.
pub const RELAY_SUCCESS_DECAY: Duration = Duration::from_secs(24 * 3600);

/// A full table only evicts a record that is disconnected or whose score is
/// below this (ten minutes of uptime, no proven relays). Established relays
/// are never displaced by newcomers.
const RELAY_EVICTABLE_SCORE_SECS: u64 = 600;

/// Why `add_relay` did or did not change the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayRegistration {
    /// New relay admitted.
    Admitted,
    /// Known relay; address list refreshed.
    Refreshed,
    /// Known relay seen again inside the re-registration interval; ignored.
    Ignored,
    /// New relay refused: admission rate limit.
    RateLimited,
    /// New relay refused: its network group already holds its slots.
    GroupFull,
    /// No observed remote IP for the relay's authenticated connection (it is
    /// itself reached over a circuit): its group cannot be established.
    NoObservedAddr,
    /// New relay refused: table full and every record is established.
    TableFull,
    /// No usable address supplied.
    NoUsableAddr,
}

#[derive(Debug, Clone)]
struct RelayRecord {
    peer: PeerId,
    addrs: Vec<Multiaddr>,
    group: Option<String>,
    first_seen: Instant,
    connected_since: Option<Instant>,
    uptime_banked: Duration,
    successes: u32,
    last_success: Option<Instant>,
    last_registration: Instant,
}

impl RelayRecord {
    fn uptime(&self, now: Instant) -> Duration {
        let live = self
            .connected_since
            .map(|since| now.saturating_duration_since(since))
            .unwrap_or_default();
        (self.uptime_banked + live).min(RELAY_UPTIME_CAP)
    }

    /// Proven-service component: capped, and decaying linearly to zero over
    /// [`RELAY_SUCCESS_DECAY`] since the latest credit.
    fn proven_secs(&self, now: Instant) -> u64 {
        let Some(last) = self.last_success else {
            return 0;
        };
        let age = now.saturating_duration_since(last);
        if age >= RELAY_SUCCESS_DECAY {
            return 0;
        }
        let full = u64::from(self.successes.min(RELAY_SUCCESS_CAP)) * RELAY_SUCCESS_WEIGHT_SECS;
        let remaining = (RELAY_SUCCESS_DECAY - age).as_secs();
        full.saturating_mul(remaining) / RELAY_SUCCESS_DECAY.as_secs().max(1)
    }

    /// Reputation score in seconds-equivalent: longevity plus the (small,
    /// capped, decaying) proven-service component.
    fn score(&self, now: Instant) -> u64 {
        self.uptime(now)
            .as_secs()
            .saturating_add(self.proven_secs(now))
    }

    fn connected(&self) -> bool {
        self.connected_since.is_some()
    }
}

#[derive(Debug, Default)]
struct RelayTable {
    records: Vec<RelayRecord>,
    /// Admission time and whether the admitted identity had proven history.
    recent_admissions: Vec<(Instant, bool)>,
    /// Peers that earned a proven credit, kept past record eviction.
    proven_history: Vec<PeerId>,
}

/// Whether `ip` is in a range whose /24 (or /48) says nothing about who
/// operates the host: the same private prefix exists on every LAN.
fn is_local_scope(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_unspecified()
                // 100.64.0.0/10 shared (carrier-grade NAT) address space.
                || (o[0] == 100 && (o[1] & 0xC0) == 0x40)
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_local_scope(IpAddr::V4(v4));
            }
            let first = v6.segments()[0];
            v6.is_loopback()
                || v6.is_unspecified()
                // fc00::/7 unique local, fe80::/10 link local.
                || (first & 0xFE00) == 0xFC00
                || (first & 0xFFC0) == 0xFE80
        }
    }
}

/// Network group of an observed remote IP: IPv4 /24 or IPv6 /48. `None` for
/// local-scope ranges, which are exempt from the per-group cap.
fn network_group_of_ip(ip: IpAddr) -> Option<String> {
    if is_local_scope(ip) {
        return None;
    }
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            Some(format!("v4:{}.{}.{}", o[0], o[1], o[2]))
        }
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(v4) => network_group_of_ip(IpAddr::V4(v4)),
            None => {
                let s = v6.segments();
                Some(format!("v6:{:x}:{:x}:{:x}", s[0], s[1], s[2]))
            }
        },
    }
}

/// The remote IP of a DIRECT connection address (`/ip4|ip6/..`), or `None`
/// when the address is relayed (it contains a circuit hop) or has no IP. This
/// is the only source of a relay's network group: it comes from the
/// connection the transport authenticated, not from what the peer claims.
pub fn observed_direct_ip(remote_addr: &Multiaddr) -> Option<IpAddr> {
    use libp2p::multiaddr::Protocol;
    let mut ip = None;
    for proto in remote_addr.iter() {
        match proto {
            Protocol::P2pCircuit => return None,
            Protocol::Ip4(v4) if ip.is_none() => ip = Some(IpAddr::V4(v4)),
            Protocol::Ip6(v6) if ip.is_none() => ip = Some(IpAddr::V6(v6)),
            _ => {}
        }
    }
    ip
}

/// The relay peer a circuit connection address runs through: the `/p2p/<id>`
/// immediately before `/p2p-circuit`. `None` for a non-circuit address.
pub fn circuit_relay_of(remote_addr: &Multiaddr) -> Option<PeerId> {
    use libp2p::multiaddr::Protocol;
    let mut last_p2p = None;
    for proto in remote_addr.iter() {
        match proto {
            Protocol::P2p(id) => last_p2p = Some(id),
            Protocol::P2pCircuit => return last_p2p,
            _ => {}
        }
    }
    None
}

/// Build the circuit address through `relay_pid` for `target`, or `None` when
/// the relay address is nested, portless or has no IP.
fn circuit_through(
    relay_pid: &PeerId,
    relay_addr: &Multiaddr,
    target: PeerId,
) -> Option<Multiaddr> {
    use libp2p::multiaddr::Protocol;
    // Identify can repeat /p2p and /p2p-circuit components when a peer has
    // already used a relay; appending another circuit suffix would create
    // nested/self-returning routes.
    if relay_addr
        .iter()
        .any(|proto| matches!(proto, Protocol::P2pCircuit))
    {
        return None;
    }
    // Preserve each transport component (IP, port, transport wrappers) so the
    // relay's concrete dialable prefix survives.
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
    if !(has_ip && has_port) {
        return None;
    }
    let mut circuit_addr = direct_addr;
    circuit_addr.push(Protocol::P2p(*relay_pid));
    circuit_addr.push(Protocol::P2pCircuit);
    circuit_addr.push(Protocol::P2p(target));
    Some(circuit_addr)
}

/// Circuit-relay ladder builder: adds relay addresses to a peer's dial candidates.
///
/// Once a peer is connected, we construct circuit-relay multiaddrs to that peer
/// through known relay peers. Every identified peer is a relay candidate and
/// identify re-fires every 60 s, so the set MUST resist Sybil
/// re-registration (Rule-8 review of #372, finding 1). Nothing is pinned or
/// statically trusted; reputation decides:
///
/// * Rank: uptime (connected time, capped) plus a small, capped, decaying
///   credit for LOCALLY PROVEN relay service (`record_proven_relay`: an
///   end-to-end authenticated connection that actually ran over the relay's
///   circuit). Self-asserted claims never score. Ties go to the OLDEST
///   record, never the newest, so a flood of fresh identities sorts behind
///   established relays.
/// * Re-registration of a tracked relay is rate limited
///   ([`RELAY_REREGISTER_MIN_INTERVAL`]) and never resets its record.
/// * Admission of new identities is rate limited
///   ([`NEW_RELAY_ADMISSIONS_PER_WINDOW`] per [`NEW_RELAY_WINDOW`]), with
///   [`NEW_RELAY_PROVEN_RESERVED`] of those slots reserved for identities
///   with locally proven history.
/// * Per-identity slots: at most [`MAX_RELAYS_PER_NETWORK_GROUP`] relays per
///   public IPv4 /24 (IPv6 /48) of the OBSERVED connection address, fixed at
///   admission; private ranges are exempt. [`MAX_ADDRS_PER_RELAY`] addresses
///   per relay.
/// * Eviction: a full table only displaces a disconnected or unproven record;
///   established relays are never evicted by newcomers.
/// * Emission is round-robin across ranked relays, so no single relay fills
///   the whole ladder.
pub struct CircuitRelayLadder {
    relays: Arc<RwLock<RelayTable>>,
}

impl CircuitRelayLadder {
    /// Create a new circuit-relay ladder.
    pub fn new() -> Self {
        Self {
            relays: Arc::new(RwLock::new(RelayTable::default())),
        }
    }

    /// Register a known relay peer with its advertised dial addresses.
    /// `observed_ip` is the remote IP of the relay's authenticated direct
    /// connection (see [`observed_direct_ip`]); it, not the advertised
    /// addresses, decides the relay's network group.
    pub fn add_relay(
        &self,
        relay_peer_id: PeerId,
        external_addrs: Vec<Multiaddr>,
        observed_ip: Option<IpAddr>,
    ) {
        let outcome = self.add_relay_at(relay_peer_id, external_addrs, observed_ip, Instant::now());
        debug!(
            relay_peer_id=%relay_peer_id,
            ?outcome,
            "[CIRCUIT-RELAY] Relay registration"
        );
    }

    /// `add_relay` with an explicit clock, for deterministic tests.
    pub fn add_relay_at(
        &self,
        relay_peer_id: PeerId,
        external_addrs: Vec<Multiaddr>,
        observed_ip: Option<IpAddr>,
        now: Instant,
    ) -> RelayRegistration {
        let mut addrs: Vec<Multiaddr> = Vec::with_capacity(MAX_ADDRS_PER_RELAY);
        for addr in external_addrs {
            if addrs.len() >= MAX_ADDRS_PER_RELAY {
                break;
            }
            if !addrs.contains(&addr) {
                addrs.push(addr);
            }
        }
        if addrs.is_empty() {
            return RelayRegistration::NoUsableAddr;
        }

        let mut table = self.relays.write();

        if let Some(record) = table.records.iter_mut().find(|r| r.peer == relay_peer_id) {
            // A reconnect after remove_relay resumes the uptime clock.
            if record.connected_since.is_none() {
                record.connected_since = Some(now);
            }
            if now.saturating_duration_since(record.last_registration)
                < RELAY_REREGISTER_MIN_INTERVAL
            {
                return RelayRegistration::Ignored;
            }
            // The group is frozen at admission: a refresh may update dial
            // addresses but can never hop the relay into another group (or
            // past the group cap) by re-advertising.
            record.addrs = addrs;
            record.last_registration = now;
            return RelayRegistration::Refreshed;
        }

        let Some(observed_ip) = observed_ip else {
            return RelayRegistration::NoObservedAddr;
        };

        let proven = table.proven_history.contains(&relay_peer_id);
        table
            .recent_admissions
            .retain(|(t, _)| now.saturating_duration_since(*t) < NEW_RELAY_WINDOW);
        let unproven_recent = table
            .recent_admissions
            .iter()
            .filter(|(_, was_proven)| !was_proven)
            .count();
        let unproven_budget =
            NEW_RELAY_ADMISSIONS_PER_WINDOW.saturating_sub(NEW_RELAY_PROVEN_RESERVED);
        if table.recent_admissions.len() >= NEW_RELAY_ADMISSIONS_PER_WINDOW
            || (!proven && unproven_recent >= unproven_budget)
        {
            return RelayRegistration::RateLimited;
        }

        let group = network_group_of_ip(observed_ip);
        if let Some(group) = &group {
            let in_group = table
                .records
                .iter()
                .filter(|r| r.group.as_ref() == Some(group))
                .count();
            if in_group >= MAX_RELAYS_PER_NETWORK_GROUP {
                return RelayRegistration::GroupFull;
            }
        }

        if table.records.len() >= RELAY_TRACKING_CAP {
            // Lowest-score evictable record; newest first on ties.
            let victim = table
                .records
                .iter()
                .enumerate()
                .filter(|(_, r)| !r.connected() || r.score(now) < RELAY_EVICTABLE_SCORE_SECS)
                .min_by(|(_, a), (_, b)| {
                    a.score(now)
                        .cmp(&b.score(now))
                        .then(b.first_seen.cmp(&a.first_seen))
                })
                .map(|(i, _)| i);
            match victim {
                Some(index) => {
                    table.records.remove(index);
                }
                None => return RelayRegistration::TableFull,
            }
        }

        table.recent_admissions.push((now, proven));
        table.records.push(RelayRecord {
            peer: relay_peer_id,
            addrs,
            group,
            first_seen: now,
            connected_since: Some(now),
            uptime_banked: Duration::ZERO,
            successes: 0,
            last_success: None,
            last_registration: now,
        });
        RelayRegistration::Admitted
    }

    /// Credit a relay for service WE proved locally: an end-to-end
    /// authenticated connection to a third peer that actually ran over this
    /// relay's circuit (or a delivery receipt for a message sent via it).
    /// Never call this for something the relay merely claims (a
    /// `RelayResponse.accepted` flag, a granted reservation): those are
    /// forgeable and would let a Sybil outrank honest relays.
    ///
    /// Returns `true` when a credit was recorded (it is rate limited per
    /// relay by [`RELAY_SUCCESS_MIN_INTERVAL`] and capped).
    pub fn record_proven_relay(&self, relay_peer_id: &PeerId) -> bool {
        self.record_proven_relay_at(relay_peer_id, Instant::now())
    }

    /// `record_proven_relay` with an explicit clock, for deterministic tests.
    pub fn record_proven_relay_at(&self, relay_peer_id: &PeerId, now: Instant) -> bool {
        let mut table = self.relays.write();
        let credited = match table.records.iter_mut().find(|r| &r.peer == relay_peer_id) {
            Some(record) => {
                let too_soon = record.last_success.is_some_and(|last| {
                    now.saturating_duration_since(last) < RELAY_SUCCESS_MIN_INTERVAL
                });
                if too_soon {
                    false
                } else {
                    record.successes = record.successes.saturating_add(1).min(RELAY_SUCCESS_CAP);
                    record.last_success = Some(now);
                    true
                }
            }
            None => false,
        };
        if credited && !table.proven_history.contains(relay_peer_id) {
            if table.proven_history.len() >= PROVEN_HISTORY_CAP {
                table.proven_history.remove(0);
            }
            table.proven_history.push(*relay_peer_id);
        }
        credited
    }

    /// A relay's authenticated connection is gone: stop offering it, bank its
    /// uptime. The record (and its reputation) is retained so a reconnect does
    /// not restart from zero, but it becomes evictable.
    pub fn remove_relay(&self, relay_peer_id: &PeerId) {
        self.remove_relay_at(relay_peer_id, Instant::now());
    }

    /// `remove_relay` with an explicit clock, for deterministic tests.
    pub fn remove_relay_at(&self, relay_peer_id: &PeerId, now: Instant) {
        if let Some(record) = self
            .relays
            .write()
            .records
            .iter_mut()
            .find(|r| &r.peer == relay_peer_id)
        {
            if let Some(since) = record.connected_since.take() {
                record.uptime_banked += now.saturating_duration_since(since);
            }
        }
    }

    /// Number of tracked relay records (connected or not).
    pub fn tracked_relays(&self) -> usize {
        self.relays.read().records.len()
    }

    /// Build circuit-relay multiaddrs to a target peer through known relays.
    ///
    /// Returns at most [`MAX_RELAY_LADDER_ADDRS`] addresses in the format
    /// `/ip4/<relay-ip>/tcp/<relay-port>/p2p/<relay-peer-id>/p2p-circuit/p2p/<target-peer-id>`,
    /// best-reputation relay first, round-robin across relays.
    pub fn build_relay_addresses(&self, target_peer_id: PeerId) -> Vec<Multiaddr> {
        self.build_relay_addresses_at(target_peer_id, Instant::now())
    }

    /// `build_relay_addresses` with an explicit clock, for deterministic tests.
    pub fn build_relay_addresses_at(&self, target_peer_id: PeerId, now: Instant) -> Vec<Multiaddr> {
        let table = self.relays.read();

        // A relay cannot provide a useful circuit to itself: a self-target
        // creates a path that returns to the originating node.
        let mut ranked: Vec<&RelayRecord> = table
            .records
            .iter()
            .filter(|r| r.connected() && r.peer != target_peer_id)
            .collect();
        ranked.sort_by(|a, b| {
            b.score(now)
                .cmp(&a.score(now))
                .then(a.first_seen.cmp(&b.first_seen))
                .then(a.peer.to_bytes().cmp(&b.peer.to_bytes()))
        });

        let per_relay: Vec<Vec<Multiaddr>> = ranked
            .iter()
            .map(|r| {
                r.addrs
                    .iter()
                    .filter_map(|a| circuit_through(&r.peer, a, target_peer_id))
                    .collect()
            })
            .collect();

        let mut out: Vec<Multiaddr> = Vec::with_capacity(MAX_RELAY_LADDER_ADDRS);
        let mut seen: HashSet<Multiaddr> = HashSet::new();
        for round in 0..MAX_ADDRS_PER_RELAY {
            for circuits in &per_relay {
                if out.len() >= MAX_RELAY_LADDER_ADDRS {
                    break;
                }
                if let Some(circuit) = circuits.get(round) {
                    if seen.insert(circuit.clone()) {
                        out.push(circuit.clone());
                    }
                }
            }
        }

        if !out.is_empty() {
            debug!(
                target_peer_id=%target_peer_id,
                relay_count=out.len(),
                cap=MAX_RELAY_LADDER_ADDRS,
                "[CIRCUIT-RELAY] Built relay addresses for target, best reputation first"
            );
        }
        out
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
        assert!(!state.is_dead);
    }

    #[test]
    fn test_exponential_backoff_progression() {
        let mut state = PerPeerBackoffState::new(None);

        // 1st failure: 1s → 2s
        state.on_dial_failure();
        assert_eq!(state.attempt_count, 1);
        assert_eq!(state.backoff_duration, Duration::from_secs(2));

        // 2nd failure: 2s → 4s
        state.on_dial_failure();
        assert_eq!(state.attempt_count, 2);
        assert_eq!(state.backoff_duration, Duration::from_secs(4));

        // 3rd failure: 4s → 8s
        state.on_dial_failure();
        assert_eq!(state.attempt_count, 3);
        assert_eq!(state.backoff_duration, Duration::from_secs(8));
        assert!(state.is_dead); // Marked as dead after 3 attempts
    }

    #[test]
    fn test_backoff_cap_at_30s() {
        let mut state = PerPeerBackoffState::new(None);

        // Simulate many failures to reach the 30s cap.
        for _ in 0..10 {
            state.on_dial_failure();
            if state.is_dead {
                break;
            }
        }

        // Check that backoff never exceeds 30s.
        assert!(state.backoff_duration <= Duration::from_secs(30));
    }

    #[test]
    fn test_eligibility_check() {
        let state = PerPeerBackoffState::new(None);
        assert!(state.is_eligible()); // Initially eligible

        let mut state = PerPeerBackoffState::new(None);
        state.on_dial_failure();
        state.on_dial_failure();
        state.on_dial_failure();
        assert!(!state.is_eligible()); // Dead after 3 attempts
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
        assert!(!state.is_dead);
    }

    #[test]
    fn test_permanent_failure() {
        let mut state = PerPeerBackoffState::new(None);
        state.on_permanent_failure();
        assert!(state.is_dead);
        assert_eq!(state.attempt_count, 3);
    }

    #[test]
    fn test_dead_revive_after_window() {
        let mut state = PerPeerBackoffState::new(None);
        state.on_dial_failure();
        state.on_dial_failure();
        state.on_dial_failure();
        assert!(state.is_dead);
        assert!(state.dead_since.is_some());
        assert!(!state.is_eligible()); // within the revive window

        // Simulate the revive window elapsing (clock is wall-time based).
        state.dead_since = Some(Instant::now() - DEAD_REVIVE_AFTER - Duration::from_secs(1));
        assert!(state.is_eligible()); // bounded dead: eligible again after the window

        // maybe_revive clears the dead mark persistently and resets strikes.
        assert!(state.maybe_revive());
        assert!(!state.is_dead);
        assert_eq!(state.attempt_count, 0);
        assert_eq!(state.backoff_duration, Duration::from_secs(1));
        assert!(!state.maybe_revive()); // already alive: no-op
    }

    #[test]
    fn test_manager_revives_dead_entry_on_register() {
        let manager = DialPolicyManager::new();
        let key = "/ip4/10.0.0.9/tcp/9000".to_string();
        manager.register_dial_attempt(&key, None);
        manager.record_dial_failure(&key, None);
        manager.record_dial_failure(&key, None);
        manager.record_dial_failure(&key, None);
        let dead = manager.get_backoff_state(&key).expect("state");
        assert!(dead.is_dead);
        assert!(!manager.register_dial_attempt(&key, None)); // window not elapsed

        // Force the dead_since back so the window has elapsed.
        {
            let mut backoff = manager.peer_backoff.write();
            if let Some(st) = backoff.get_mut(&key) {
                st.dead_since =
                    Some(web_time::Instant::now() - DEAD_REVIVE_AFTER - Duration::from_secs(1));
            }
        }
        assert!(manager.register_dial_attempt(&key, None)); // revived -> allowed
        let revived = manager.get_backoff_state(&key).expect("state");
        assert!(!revived.is_dead);
        assert_eq!(revived.attempt_count, 0);
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
    fn test_reset_peer_backoff_clears_dead_state_on_any_addr_entry() {
        use libp2p::identity::Keypair;

        let manager = DialPolicyManager::new();
        let pid = Keypair::generate_ed25519().public().to_peer_id();

        // Peer gets marked dead after 3 failures on its dialed LAN address.
        let lan_addr = "/ip4/192.168.1.50/tcp/4001";
        for _ in 0..3 {
            manager.record_dial_failure(lan_addr, Some(pid));
        }
        assert!(!manager.register_dial_attempt(lan_addr, Some(pid)));

        // An INBOUND connection arrives from an ephemeral remote address:
        // resetting only that address must NOT revive the dead LAN entry.
        let ephemeral = "/ip4/192.168.1.50/tcp/51234";
        manager.reset_on_connection_established(ephemeral, Some(pid));
        assert!(!manager.register_dial_attempt(lan_addr, Some(pid)));

        // Peer-wide liveness reset (what ConnectionEstablished now does) must.
        manager.reset_peer_backoff(pid);
        let state = manager.get_backoff_state(lan_addr).expect("entry exists");
        assert_eq!(state.attempt_count, 0);
        assert!(!state.is_dead);
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
        let observed = observed_direct_ip(&relay_addr);
        ladder.add_relay(relay_pid, vec![relay_addr], observed);

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
        let observed = observed_direct_ip(&relay_addr);
        ladder.add_relay(relay_pid, vec![relay_addr], observed);

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
            "192.168.1.101".parse().ok(),
        );
        ladder.add_relay(
            other_relay_pid,
            vec!["/ip4/192.168.1.102/tcp/4001"
                .parse()
                .expect("direct relay fixture is valid")],
            "192.168.1.102".parse().ok(),
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

    // ---- Relay-ladder hostile-registration tests (Rule-8 review of #372, F1) ----

    fn fresh_pid() -> PeerId {
        libp2p::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id()
    }

    /// Public address in a distinct /24 per `net`, so group caps do not
    /// interfere (44.0.0.0/8 is public; private ranges are group-exempt).
    fn net_addr(net: u8, host: u8) -> Multiaddr {
        format!("/ip4/44.{net}.0.{host}/tcp/4001")
            .parse()
            .expect("fixture addr")
    }

    /// Register with the observed IP taken from the first address, i.e. an
    /// honest relay that advertises the address it is actually reached at.
    fn add_at(
        ladder: &CircuitRelayLadder,
        pid: PeerId,
        addrs: Vec<Multiaddr>,
        at: Instant,
    ) -> RelayRegistration {
        let observed = addrs.first().and_then(observed_direct_ip);
        ladder.add_relay_at(pid, addrs, observed, at)
    }

    /// Admit `count` relays one admission-window apart so the admission rate
    /// limit never interferes with fixture setup. Returns the ids.
    fn seed_relays(
        ladder: &CircuitRelayLadder,
        base: Instant,
        first_net: u8,
        count: usize,
    ) -> Vec<PeerId> {
        (0..count)
            .map(|i| {
                let pid = fresh_pid();
                let at = base + NEW_RELAY_WINDOW * (i as u32);
                assert_eq!(
                    add_at(&ladder, pid, vec![net_addr(first_net + i as u8, 1)], at),
                    RelayRegistration::Admitted
                );
                pid
            })
            .collect()
    }

    #[test]
    fn sybil_flood_cannot_displace_established_relays_in_the_ladder() {
        let ladder = CircuitRelayLadder::new();
        let t0 = Instant::now();
        // Two honest relays, long-lived; the later one has locally proven
        // service (three credits, spaced past the per-relay interval), which
        // outweighs its 60 s later start but is far below an uptime cap.
        let honest = seed_relays(&ladder, t0, 1, 2);
        for k in 0..3u32 {
            assert!(ladder.record_proven_relay_at(
                &honest[1],
                t0 + Duration::from_secs(500) + RELAY_SUCCESS_MIN_INTERVAL * k
            ));
        }

        // A Sybil mints 200 identities, each on its own network, hammering
        // registration for ten minutes after the honest relays were admitted.
        let sybil_start = t0 + Duration::from_secs(600);
        let mut admitted_sybils = 0;
        for i in 0..200u32 {
            let at = sybil_start + Duration::from_millis(u64::from(i) * 50);
            let outcome = add_at(
                &ladder,
                fresh_pid(),
                vec![net_addr(100 + (i % 100) as u8, (i / 100) as u8 + 1)],
                at,
            );
            if outcome == RelayRegistration::Admitted {
                admitted_sybils += 1;
            }
        }
        assert!(
            admitted_sybils <= NEW_RELAY_ADMISSIONS_PER_WINDOW,
            "admission rate limit must cap a burst, admitted {admitted_sybils}"
        );

        let target = fresh_pid();
        let now = sybil_start + Duration::from_secs(30);
        let built = ladder.build_relay_addresses_at(target, now);
        assert!(built.len() <= MAX_RELAY_LADDER_ADDRS);
        let first = built[0].to_string();
        let second = built[1].to_string();
        assert!(
            first.contains(&honest[1].to_string()),
            "proven long-lived relay must rank first, got {first}"
        );
        assert!(
            second.contains(&honest[0].to_string()),
            "older relay must outrank unproven newcomers, got {second}"
        );
    }

    #[test]
    fn same_network_sybil_is_capped_to_the_group_slots() {
        let ladder = CircuitRelayLadder::new();
        let t0 = Instant::now();
        let mut outcomes = Vec::new();
        // Many identities from one /24, spaced past the admission window.
        for i in 0..10u32 {
            outcomes.push(add_at(
                &ladder,
                fresh_pid(),
                vec![net_addr(7, (i + 1) as u8)],
                t0 + NEW_RELAY_WINDOW * i,
            ));
        }
        let admitted = outcomes
            .iter()
            .filter(|o| **o == RelayRegistration::Admitted)
            .count();
        assert_eq!(admitted, MAX_RELAYS_PER_NETWORK_GROUP);
        assert!(outcomes.contains(&RelayRegistration::GroupFull));
        assert_eq!(ladder.tracked_relays(), MAX_RELAYS_PER_NETWORK_GROUP);
    }

    #[test]
    fn reregistration_is_rate_limited_and_does_not_reset_rank() {
        let ladder = CircuitRelayLadder::new();
        let t0 = Instant::now();
        let pid = fresh_pid();
        assert_eq!(
            add_at(&ladder, pid, vec![net_addr(1, 1)], t0),
            RelayRegistration::Admitted
        );
        // Identify fires every 60 s: all inside the interval are ignored and
        // must not replace the address list.
        for k in 1..5u32 {
            assert_eq!(
                add_at(
                    &ladder,
                    pid,
                    vec![net_addr(2, 9)],
                    t0 + Duration::from_secs(60) * k
                ),
                RelayRegistration::Ignored
            );
        }
        let built = ladder.build_relay_addresses_at(fresh_pid(), t0 + Duration::from_secs(300));
        assert!(built[0].to_string().starts_with("/ip4/44.1.0.1/"));
        // After the interval a refresh is accepted, but first_seen (rank) is kept.
        assert_eq!(
            add_at(
                &ladder,
                pid,
                vec![net_addr(2, 9)],
                t0 + RELAY_REREGISTER_MIN_INTERVAL
            ),
            RelayRegistration::Refreshed
        );
        let built = ladder.build_relay_addresses_at(fresh_pid(), t0 + Duration::from_secs(301));
        assert!(built[0].to_string().starts_with("/ip4/44.2.0.9/"));
        assert_eq!(ladder.tracked_relays(), 1);
    }

    #[test]
    fn full_table_never_evicts_established_relays_for_newcomers() {
        let ladder = CircuitRelayLadder::new();
        let t0 = Instant::now();
        let established = seed_relays(&ladder, t0, 1, RELAY_TRACKING_CAP);
        assert_eq!(ladder.tracked_relays(), RELAY_TRACKING_CAP);

        // Well past the evictable score, a newcomer is refused outright.
        let later = t0 + NEW_RELAY_WINDOW * (RELAY_TRACKING_CAP as u32 + 20);
        assert_eq!(
            add_at(&ladder, fresh_pid(), vec![net_addr(200, 1)], later),
            RelayRegistration::TableFull
        );
        let built = ladder.build_relay_addresses_at(fresh_pid(), later);
        for circuit in &built {
            assert!(established
                .iter()
                .any(|pid| circuit.to_string().contains(&pid.to_string())));
        }
    }

    #[test]
    fn disconnected_unproven_records_are_evictable_but_keep_reputation_on_reconnect() {
        let ladder = CircuitRelayLadder::new();
        let t0 = Instant::now();
        let ids = seed_relays(&ladder, t0, 1, RELAY_TRACKING_CAP);
        // One relay proves itself, then drops; another just drops.
        let drop_at = t0 + NEW_RELAY_WINDOW * (RELAY_TRACKING_CAP as u32 + 20);
        assert!(ladder.record_proven_relay_at(&ids[0], drop_at - Duration::from_secs(5)));
        ladder.remove_relay_at(&ids[0], drop_at);
        ladder.remove_relay_at(&ids[1], drop_at);

        // A newcomer takes the lowest-score disconnected record (ids[1]: no
        // successes), not the proven one.
        assert_eq!(
            add_at(&ladder, fresh_pid(), vec![net_addr(220, 1)], drop_at),
            RelayRegistration::Admitted
        );
        assert_eq!(ladder.tracked_relays(), RELAY_TRACKING_CAP);
        // ids[0] reconnects: reputation survived (success still counted).
        assert_eq!(
            add_at(
                &ladder,
                ids[0],
                vec![net_addr(1, 1)],
                drop_at + Duration::from_secs(1)
            ),
            RelayRegistration::Refreshed
        );
        let built = ladder.build_relay_addresses_at(fresh_pid(), drop_at + Duration::from_secs(2));
        assert!(built[0].to_string().contains(&ids[0].to_string()));
    }

    #[test]
    fn one_relay_advertising_many_addresses_cannot_fill_the_ladder() {
        let ladder = CircuitRelayLadder::new();
        let t0 = Instant::now();
        let greedy = fresh_pid();
        let addrs: Vec<Multiaddr> = (1..=50u8).map(|h| net_addr(5, h)).collect();
        add_at(&ladder, greedy, addrs, t0);
        let other = fresh_pid();
        add_at(&ladder, other, vec![net_addr(6, 1)], t0 + NEW_RELAY_WINDOW);
        let built = ladder.build_relay_addresses_at(fresh_pid(), t0 + Duration::from_secs(120));
        assert!(built.len() <= MAX_RELAY_LADDER_ADDRS);
        assert!(
            built
                .iter()
                .any(|a| a.to_string().contains(&other.to_string())),
            "a second relay must keep a slot against an address-flooding relay"
        );
        assert!(
            built
                .iter()
                .filter(|a| a.to_string().contains(&greedy.to_string()))
                .count()
                <= MAX_RELAY_LADDER_ADDRS - 1
        );
    }

    #[test]
    fn ladder_width_is_inside_the_per_peer_admission_ceiling() {
        // 3 direct ports + 1 last-good + the relay half must fit under the
        // real admission cap used by behaviour.rs (not a copy of it).
        let ladder_width = 3 + 1 + MAX_RELAY_LADDER_ADDRS as u32;
        assert!(
            ladder_width <= crate::transport::behaviour::MAX_ESTABLISHED_PER_PEER,
            "a full dial ladder must be admissible by the per-peer cap"
        );
    }

    // ---- Rule-8 batch 2 (#489): forgeable reputation, group derivation, budget ----

    fn ip(s: &str) -> IpAddr {
        s.parse().expect("fixture ip")
    }

    /// The reputation signal is local proof only: a relay that merely claims
    /// service (nothing calls `record_proven_relay`) gains nothing, so an
    /// honest proven relay outranks it even when the claimant is older.
    #[test]
    fn unproven_claims_gain_nothing_and_honest_proven_relay_outranks() {
        let ladder = CircuitRelayLadder::new();
        let t0 = Instant::now();
        let claimant = fresh_pid();
        let honest = fresh_pid();
        // The claimant is admitted a window EARLIER (older, higher uptime).
        assert_eq!(
            add_at(&ladder, claimant, vec![net_addr(1, 1)], t0),
            RelayRegistration::Admitted
        );
        let h_at = t0 + Duration::from_secs(120);
        assert_eq!(
            add_at(&ladder, honest, vec![net_addr(2, 1)], h_at),
            RelayRegistration::Admitted
        );
        let now = h_at + Duration::from_secs(1800);
        let rank = |l: &CircuitRelayLadder| {
            l.build_relay_addresses_at(fresh_pid(), now)
                .first()
                .map(ToString::to_string)
                .unwrap_or_default()
        };
        // Without any local proof the older relay leads on uptime alone.
        assert!(rank(&ladder).contains(&claimant.to_string()));
        // Credits for an unknown relay are ignored outright.
        assert!(!ladder.record_proven_relay_at(&fresh_pid(), now));
        // Locally proven service, spaced credits: the honest relay overtakes.
        for k in 0..5u32 {
            assert!(ladder.record_proven_relay_at(
                &honest,
                h_at + Duration::from_secs(60) + RELAY_SUCCESS_MIN_INTERVAL * k
            ));
        }
        assert!(rank(&ladder).contains(&honest.to_string()));
    }

    #[test]
    fn proven_credit_is_rate_limited_capped_small_and_decays() {
        let ladder = CircuitRelayLadder::new();
        let t0 = Instant::now();
        let pid = fresh_pid();
        assert_eq!(
            add_at(&ladder, pid, vec![net_addr(1, 1)], t0),
            RelayRegistration::Admitted
        );
        let credit_at = t0 + Duration::from_secs(100);
        assert!(ladder.record_proven_relay_at(&pid, credit_at));
        // A burst inside the minimum interval counts once.
        for ms in [1u64, 500, 29_000] {
            assert!(!ladder.record_proven_relay_at(&pid, credit_at + Duration::from_millis(ms)));
        }
        // Far more attempts than the cap, spaced legally.
        let mut at = credit_at;
        for _ in 0..(RELAY_SUCCESS_CAP * 3) {
            at = at + RELAY_SUCCESS_MIN_INTERVAL;
            ladder.record_proven_relay_at(&pid, at);
        }
        let table = ladder.relays.read();
        let rec = &table.records[0];
        // Total proven contribution is bounded below one hour-equivalent and
        // far below the uptime cap.
        let cap_secs = u64::from(RELAY_SUCCESS_CAP) * RELAY_SUCCESS_WEIGHT_SECS;
        assert!(rec.proven_secs(at) <= cap_secs);
        assert!(cap_secs < RELAY_UPTIME_CAP.as_secs() / 100);
        // Linear decay to zero without new credits.
        let half = at + RELAY_SUCCESS_DECAY / 2;
        assert!(rec.proven_secs(half) <= cap_secs / 2 + 1);
        assert!(rec.proven_secs(half) > 0);
        assert_eq!(rec.proven_secs(at + RELAY_SUCCESS_DECAY), 0);
    }

    #[test]
    fn group_comes_from_observed_ip_not_advertised_addresses() {
        let ladder = CircuitRelayLadder::new();
        let t0 = Instant::now();
        // Three Sybils all connect from 203.0.113.0/24 but each ADVERTISES an
        // address in a different network: the advertised address is ignored
        // for grouping, so the group cap still binds.
        let outcomes: Vec<_> = (0..4u32)
            .map(|i| {
                ladder.add_relay_at(
                    fresh_pid(),
                    vec![net_addr(10 + i as u8, 1)],
                    Some(ip(&format!("203.0.113.{}", i + 1))),
                    t0 + NEW_RELAY_WINDOW * i,
                )
            })
            .collect();
        let admitted = outcomes
            .iter()
            .filter(|o| **o == RelayRegistration::Admitted)
            .count();
        assert_eq!(admitted, MAX_RELAYS_PER_NETWORK_GROUP);
        assert!(outcomes.contains(&RelayRegistration::GroupFull));
    }

    #[test]
    fn relay_without_an_observed_direct_ip_is_refused() {
        let ladder = CircuitRelayLadder::new();
        assert_eq!(
            ladder.add_relay_at(fresh_pid(), vec![net_addr(1, 1)], None, Instant::now()),
            RelayRegistration::NoObservedAddr
        );
        assert_eq!(ladder.tracked_relays(), 0);
        let circuit: Multiaddr =
            format!("/ip4/198.51.100.7/tcp/4001/p2p/{}/p2p-circuit", fresh_pid())
                .parse()
                .expect("fixture addr");
        assert_eq!(observed_direct_ip(&circuit), None);
        let direct: Multiaddr = "/ip4/198.51.100.7/tcp/4001".parse().expect("fixture addr");
        assert_eq!(observed_direct_ip(&direct), Some(ip("198.51.100.7")));
    }

    #[test]
    fn refresh_cannot_hop_a_relay_into_another_group() {
        let ladder = CircuitRelayLadder::new();
        let t0 = Instant::now();
        let victim_net = Some(ip("203.0.113.9"));
        let own_net = Some(ip("198.51.100.9"));
        // Two relays fill 203.0.113.0/24; the attacker relay sits elsewhere.
        for i in 0..MAX_RELAYS_PER_NETWORK_GROUP as u32 {
            assert_eq!(
                ladder.add_relay_at(
                    fresh_pid(),
                    vec![net_addr(1, 1)],
                    victim_net,
                    t0 + NEW_RELAY_WINDOW * i
                ),
                RelayRegistration::Admitted
            );
        }
        let hopper = fresh_pid();
        let at = t0 + NEW_RELAY_WINDOW * 5;
        assert_eq!(
            ladder.add_relay_at(hopper, vec![net_addr(2, 1)], own_net, at),
            RelayRegistration::Admitted
        );
        // After the re-registration interval it refreshes while observed from
        // the (full) victim group: the group was frozen at admission, so the
        // table still holds exactly the group counts it admitted.
        assert_eq!(
            ladder.add_relay_at(
                hopper,
                vec![net_addr(3, 1)],
                victim_net,
                at + RELAY_REREGISTER_MIN_INTERVAL
            ),
            RelayRegistration::Refreshed
        );
        let table = ladder.relays.read();
        let rec = table
            .records
            .iter()
            .find(|r| r.peer == hopper)
            .expect("hopper tracked");
        assert_eq!(rec.group, network_group_of_ip(ip("198.51.100.9")));
        let in_victim_group = table
            .records
            .iter()
            .filter(|r| r.group == network_group_of_ip(ip("203.0.113.9")))
            .count();
        assert_eq!(in_victim_group, MAX_RELAYS_PER_NETWORK_GROUP);
    }

    #[test]
    fn unproven_flood_cannot_monopolise_the_admission_budget() {
        let ladder = CircuitRelayLadder::new();
        let t0 = Instant::now();
        // A returning relay earned history, then lost its record.
        let veteran = fresh_pid();
        assert_eq!(
            add_at(&ladder, veteran, vec![net_addr(90, 1)], t0),
            RelayRegistration::Admitted
        );
        assert!(ladder.record_proven_relay_at(&veteran, t0 + Duration::from_secs(10)));
        {
            let mut table = ladder.relays.write();
            table.records.clear();
            table.recent_admissions.clear();
        }
        // A burst of fresh identities inside one window.
        let burst = t0 + Duration::from_secs(1000);
        let mut admitted = 0;
        for i in 0..50u32 {
            if add_at(
                &ladder,
                fresh_pid(),
                vec![net_addr(100 + i as u8, 1)],
                burst + Duration::from_millis(u64::from(i)),
            ) == RelayRegistration::Admitted
            {
                admitted += 1;
            }
        }
        assert_eq!(
            admitted,
            NEW_RELAY_ADMISSIONS_PER_WINDOW - NEW_RELAY_PROVEN_RESERVED,
            "unproven identities must leave the reserved slots free"
        );
        // The veteran still gets in inside the same window.
        assert_eq!(
            add_at(
                &ladder,
                veteran,
                vec![net_addr(90, 1)],
                burst + Duration::from_millis(100)
            ),
            RelayRegistration::Admitted
        );
        // And the total is still bounded by the window budget.
        let mut extra = 0;
        for i in 0..20u32 {
            if add_at(
                &ladder,
                fresh_pid(),
                vec![net_addr(150 + i as u8, 1)],
                burst + Duration::from_millis(200 + u64::from(i)),
            ) == RelayRegistration::Admitted
            {
                extra += 1;
            }
        }
        assert_eq!(extra, 0, "unproven budget stays exhausted for the window");
    }

    #[test]
    fn private_and_local_ranges_are_exempt_from_the_group_cap() {
        let ladder = CircuitRelayLadder::new();
        let t0 = Instant::now();
        // Same RFC1918 /24 (different physical LANs): all admitted.
        for i in 0..4u32 {
            assert_eq!(
                ladder.add_relay_at(
                    fresh_pid(),
                    vec![net_addr(1, 1)],
                    Some(ip(&format!("192.168.1.{}", i + 10))),
                    t0 + NEW_RELAY_WINDOW * i
                ),
                RelayRegistration::Admitted
            );
        }
        for local in [
            "10.1.2.3",
            "172.16.5.5",
            "169.254.7.7",
            "127.0.0.1",
            "100.64.1.1",
            "fd12:3456:789a::1",
            "fe80::1",
            "::1",
            "::ffff:192.168.0.5",
        ] {
            assert_eq!(
                network_group_of_ip(ip(local)),
                None,
                "{local} is local scope"
            );
        }
        // Public ranges still group, v4 by /24 and v6 by /48.
        assert_eq!(
            network_group_of_ip(ip("203.0.113.9")),
            Some("v4:203.0.113".to_string())
        );
        assert_eq!(
            network_group_of_ip(ip("2001:db8:1:2::9")),
            Some("v6:2001:db8:1".to_string())
        );
        assert_eq!(
            network_group_of_ip(ip("::ffff:203.0.113.9")),
            Some("v4:203.0.113".to_string())
        );
    }

    #[test]
    fn circuit_relay_of_names_the_relay_before_the_circuit_hop() {
        let relay = fresh_pid();
        let target = fresh_pid();
        let via: Multiaddr =
            format!("/ip4/198.51.100.7/tcp/4001/p2p/{relay}/p2p-circuit/p2p/{target}")
                .parse()
                .expect("fixture addr");
        assert_eq!(circuit_relay_of(&via), Some(relay));
        let direct: Multiaddr = format!("/ip4/198.51.100.7/tcp/4001/p2p/{relay}")
            .parse()
            .expect("fixture addr");
        assert_eq!(circuit_relay_of(&direct), None);
    }
}
