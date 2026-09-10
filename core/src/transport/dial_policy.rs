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
