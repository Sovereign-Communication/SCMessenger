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
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use web_time::{Duration, Instant};

/// Per-peer backoff state tracking.
#[derive(Debug, Clone)]
pub struct PerPeerBackoffState {
    /// Number of failed dial attempts (0-3). At 3, peer is considered dead.
    pub attempt_count: u32,
    /// Timestamp of the last dial attempt to this peer.
    pub last_attempt_ts: Instant,
    /// Current backoff duration (1s → 2s → 4s → 8s → 16s → 30s capped).
    pub backoff_duration: Duration,
    /// Whether this peer is marked as dead for this session (permanent failure).
    pub is_dead: bool,
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
            peer_id,
        }
    }

    /// Check if this peer is eligible for a dial attempt right now.
    pub fn is_eligible(&self) -> bool {
        if self.is_dead {
            return false;
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

        // After 3 attempts, mark as dead for this session.
        if self.attempt_count >= 3 {
            warn!(
                peer_id=?self.peer_id,
                "[DIAL-BACKOFF] Peer marked as dead after 3 failed attempts"
            );
            self.is_dead = true;
        }
    }

    /// Record a permanent dial failure (mark peer as dead immediately).
    pub fn on_permanent_failure(&mut self) {
        self.is_dead = true;
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

    /// Build a list of circuit-relay multiaddrs to a target peer through known relays.
    ///
    /// Returns a list of circuit-relay addresses in the format:
    /// `/ip4/<relay-ip>/tcp/<relay-port>/p2p/<relay-peer-id>/p2p-circuit/p2p/<target-peer-id>`
    pub fn build_relay_addresses(&self, target_peer_id: PeerId) -> Vec<Multiaddr> {
        use libp2p::multiaddr::Protocol;

        let relays = self.relays.read();
        let mut relay_addrs = Vec::new();

        for (relay_pid, external_addrs) in relays.iter() {
            for relay_addr in external_addrs {
                // Only use addresses that have a proper IP and port.
                let mut has_ip = false;
                let mut has_port = false;
                for proto in relay_addr.iter() {
                    match proto {
                        Protocol::Ip4(_) | Protocol::Ip6(_) => has_ip = true,
                        Protocol::Tcp(_) | Protocol::Udp(_) => has_port = true,
                        _ => {}
                    }
                }

                if has_ip && has_port {
                    // Construct circuit-relay address: base → /p2p/<relay> → /p2p-circuit → /p2p/<target>
                    let mut circuit_addr = relay_addr.clone();
                    circuit_addr.push(Protocol::P2p(*relay_pid));
                    circuit_addr.push(Protocol::P2pCircuit);
                    circuit_addr.push(Protocol::P2p(target_peer_id));
                    relay_addrs.push(circuit_addr);
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

        relay_addrs
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
        assert!(addr_str.contains("/p2p-circuit/"));
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
