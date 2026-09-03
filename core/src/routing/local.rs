//! Layer 1 — Mycelium (Local Cell Topology)
//!
//! The local cell maintains complete topology awareness of all directly reachable peers.
//! This includes:
//! - Real-time peer status (Active, Stale, Dormant)
//! - Transport connectivity (BLE, WiFi, TCP, QUIC)
//! - Recipient reachability via recipient hints
//! - Peer reliability scores based on relay success
//! - Gateway detection for Layer 2 gossip propagation

use std::collections::HashMap;
use web_time::SystemTime;

/// 32-byte Ed25519 public key as peer identifier
pub type PeerId = [u8; 32];

/// Transport method used to reach a peer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TransportType {
    BLE,
    WiFiAware,
    WiFiDirect,
    TCP,
    QUIC,
    /// Relayed circuit (libp2p p2p-circuit through a helper node)
    Circuit,
}

/// Status of a known peer in the local cell
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PeerStatus {
    /// Peer is currently reachable (last seen within timeout)
    Active {
        last_seen: u64,
        transport: TransportType,
    },
    /// Peer was recently seen but connection dropped
    Stale {
        last_seen: u64,
        last_transport: TransportType,
    },
    /// Peer is known but hasn't been seen in a long time
    Dormant { last_seen: u64 },
}

/// Information about a peer in the local cell
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub status: PeerStatus,
    /// Recipient hints this peer can deliver to (from their announcements)
    pub reachable_hints: Vec<[u8; 4]>,
    /// Number of messages this peer has (from sync metadata)
    pub message_count: u32,
    /// Observed relay quality (0.0 = terrible, 1.0 = perfect)
    pub reliability_score: f64,
    /// Available transports to reach this peer
    pub transports: Vec<TransportType>,
    /// Is this peer a gateway to other cells? (has connections beyond our local range)
    pub is_gateway: bool,
    /// Number of successful syncs with this peer
    pub sync_count: u32,
    /// Average sync time in milliseconds
    pub avg_sync_ms: u64,
}

/// Summary of local cell state (for gossip exchange)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CellSummary {
    pub peer_count: u32,
    pub gateway_count: u32,
    pub reachable_hints: Vec<[u8; 4]>,
    pub avg_reliability: f64,
    pub timestamp: u64,
}

/// Events emitted by the local cell during tick
#[derive(Debug, Clone, PartialEq)]
pub enum PeerEvent {
    PeerBecameActive(PeerId),
    PeerBecameStale(PeerId),
    PeerBecameDormant(PeerId),
    PeerEvicted(PeerId),
}

/// Layer 1: Complete local cell topology
/// Knows every peer within direct communication range
pub struct LocalCell {
    /// Our own peer ID
    local_id: PeerId,
    /// All known peers in the local cell
    peers: HashMap<PeerId, PeerInfo>,
    /// Timeout before Active → Stale (seconds)
    active_timeout: u64,
    /// Timeout before Stale → Dormant (seconds)
    stale_timeout: u64,
    /// Maximum peers to track
    max_peers: usize,
}

impl LocalCell {
    /// Create a new local cell with default timeouts
    pub fn new(local_id: PeerId) -> Self {
        Self {
            local_id,
            peers: HashMap::new(),
            active_timeout: 300, // 5 minutes
            stale_timeout: 1800, // 30 minutes
            max_peers: 1000,
        }
    }

    /// Create a new local cell with custom timeouts
    pub fn with_timeouts(local_id: PeerId, active_timeout: u64, stale_timeout: u64) -> Self {
        Self {
            local_id,
            peers: HashMap::new(),
            active_timeout,
            stale_timeout,
            max_peers: 1000,
        }
    }

    /// Record a peer sighting (called on every contact)
    pub fn peer_seen(&mut self, peer_id: PeerId, transport: TransportType) {
        let now = current_timestamp();

        if let Some(peer) = self.peers.get_mut(&peer_id) {
            // Update existing peer
            peer.status = PeerStatus::Active {
                last_seen: now,
                transport,
            };
            // Add transport if not already known
            if !peer.transports.contains(&transport) {
                peer.transports.push(transport);
            }
        } else {
            // Check if we need to evict to stay under max_peers
            if self.peers.len() >= self.max_peers {
                self.evict_lowest_reliability();
            }

            // Create new peer info
            self.peers.insert(
                peer_id,
                PeerInfo {
                    peer_id,
                    status: PeerStatus::Active {
                        last_seen: now,
                        transport,
                    },
                    reachable_hints: Vec::new(),
                    message_count: 0,
                    reliability_score: 0.5, // Start neutral
                    transports: vec![transport],
                    is_gateway: false,
                    sync_count: 0,
                    avg_sync_ms: 0,
                },
            );
        }
    }

    /// Update reachable hints for a peer (from their PeerAnnouncement)
    pub fn update_peer_hints(&mut self, peer_id: &PeerId, hints: Vec<[u8; 4]>) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.reachable_hints = hints;
        }
    }

    /// Mark peer as gateway (they have connections beyond local range)
    pub fn mark_as_gateway(&mut self, peer_id: &PeerId, is_gateway: bool) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.is_gateway = is_gateway;
        }
    }

    /// Record a successful sync with timing
    pub fn record_sync(
        &mut self,
        peer_id: &PeerId,
        sync_duration_ms: u64,
        messages_exchanged: u32,
    ) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            let old_avg = peer.avg_sync_ms;
            let old_count = peer.sync_count;

            // Update sync count and average (exponential moving average)
            peer.sync_count += 1;
            peer.avg_sync_ms =
                (old_avg * old_count as u64 + sync_duration_ms) / peer.sync_count as u64;

            // Update message count
            peer.message_count = messages_exchanged;
        }
    }

    /// Update reliability score based on relay outcome
    pub fn update_reliability(&mut self, peer_id: &PeerId, success: bool) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            let increment = if success { 0.1 } else { -0.15 };
            peer.reliability_score = (peer.reliability_score + increment).clamp(0.0, 1.0);
        }
    }

    /// Find active peers that might reach a recipient, ordered by the shared
    /// reliability policy used by all local-cell selectors.
    pub fn peers_for_hint(&self, hint: &[u8; 4]) -> Vec<&PeerInfo> {
        let mut peers: Vec<&PeerInfo> = self
            .peers
            .values()
            .filter(|peer| {
                matches!(peer.status, PeerStatus::Active { .. })
                    && peer.reachable_hints.contains(hint)
            })
            .collect();
        Self::sort_by_reliability(&mut peers);
        peers
    }

    /// Get all active peers sorted by the shared reliability policy.
    pub fn active_peers(&self) -> Vec<&PeerInfo> {
        let mut peers: Vec<&PeerInfo> = self
            .peers
            .values()
            .filter(|peer| matches!(peer.status, PeerStatus::Active { .. }))
            .collect();
        Self::sort_by_reliability(&mut peers);
        peers
    }

    fn sort_by_reliability(peers: &mut Vec<&PeerInfo>) {
        peers.sort_by(|a, b| {
            b.reliability_score
                .total_cmp(&a.reliability_score)
                .then_with(|| a.peer_id.cmp(&b.peer_id))
        });
    }

    /// Get gateway peers (for Layer 2 gossip)
    pub fn gateway_peers(&self) -> Vec<&PeerInfo> {
        self.peers
            .values()
            .filter(|p| p.is_gateway && matches!(p.status, PeerStatus::Active { .. }))
            .collect()
    }

    /// Run periodic cleanup: promote/demote peer statuses based on timeouts
    pub fn tick(&mut self, now: u64) -> Vec<PeerEvent> {
        let mut events = Vec::new();

        let peers_to_update: Vec<PeerId> = self.peers.keys().copied().collect();

        for peer_id in peers_to_update {
            if let Some(peer) = self.peers.get_mut(&peer_id) {
                match &peer.status {
                    PeerStatus::Active {
                        last_seen,
                        transport,
                    } => {
                        if now - last_seen > self.active_timeout {
                            let last_transport = *transport;
                            peer.status = PeerStatus::Stale {
                                last_seen: *last_seen,
                                last_transport,
                            };
                            events.push(PeerEvent::PeerBecameStale(peer_id));
                        }
                    }
                    PeerStatus::Stale { last_seen, .. } => {
                        if now - last_seen > self.stale_timeout {
                            peer.status = PeerStatus::Dormant {
                                last_seen: *last_seen,
                            };
                            events.push(PeerEvent::PeerBecameDormant(peer_id));
                        }
                    }
                    PeerStatus::Dormant { .. } => {
                        // Keep dormant peers around for a while (could be brought back)
                    }
                }
            }
        }

        events
    }

    /// Get our local cell summary (for Layer 2 gossip exchange)
    pub fn summarize(&self) -> CellSummary {
        let gateway_count = self.peers.values().filter(|p| p.is_gateway).count() as u32;

        let mut all_hints = Vec::new();
        for peer in self.peers.values() {
            for hint in &peer.reachable_hints {
                if !all_hints.contains(hint) {
                    all_hints.push(*hint);
                }
            }
        }

        let avg_reliability = if self.peers.is_empty() {
            0.0
        } else {
            let sum: f64 = self.peers.values().map(|p| p.reliability_score).sum();
            sum / self.peers.len() as f64
        };

        CellSummary {
            peer_count: self.peers.len() as u32,
            gateway_count,
            reachable_hints: all_hints,
            avg_reliability,
            timestamp: current_timestamp(),
        }
    }

    /// Get the count of all known peers
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Get the count of active peers
    pub fn active_count(&self) -> usize {
        self.peers
            .values()
            .filter(|p| matches!(p.status, PeerStatus::Active { .. }))
            .count()
    }

    /// Get a specific peer by ID
    pub fn get_peer(&self, id: &PeerId) -> Option<&PeerInfo> {
        self.peers.get(id)
    }

    /// Evict the peer with the lowest reliability score
    fn evict_lowest_reliability(&mut self) {
        if self.peers.is_empty() {
            return;
        }

        let peer_to_evict = *self
            .peers
            .values()
            .min_by(|a, b| a.reliability_score.total_cmp(&b.reliability_score))
            .map(|p| &p.peer_id)
            .expect("checked non-empty above");

        self.peers.remove(&peer_to_evict);
    }

    pub fn local_id(&self) -> PeerId {
        self.local_id
    }
}

/// Helper function to get current unix timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_peer_id(n: u8) -> PeerId {
        let mut id = [0u8; 32];
        id[0] = n;
        id
    }

    fn make_hint(n: u32) -> [u8; 4] {
        n.to_le_bytes()
    }

    #[test]
    fn test_peer_seen_creates_active_peer() {
        let local_id = make_peer_id(1);
        let mut cell = LocalCell::new(local_id);
        let peer_id = make_peer_id(2);

        cell.peer_seen(peer_id, TransportType::BLE);

        assert_eq!(cell.peer_count(), 1);
        let peer = cell.get_peer(&peer_id).unwrap();
        assert!(matches!(peer.status, PeerStatus::Active { .. }));
        assert!(peer.transports.contains(&TransportType::BLE));
    }

    #[test]
    fn test_active_timeout_progression() {
        let local_id = make_peer_id(1);
        let mut cell = LocalCell::with_timeouts(local_id, 100, 200);
        let peer_id = make_peer_id(2);

        let now = current_timestamp();
        cell.peer_seen(peer_id, TransportType::BLE);

        // Initially active
        assert!(matches!(
            cell.get_peer(&peer_id).unwrap().status,
            PeerStatus::Active { .. }
        ));

        // After 101 seconds, should be stale
        let events = cell.tick(now + 101);
        assert!(events.contains(&PeerEvent::PeerBecameStale(peer_id)));
        assert!(matches!(
            cell.get_peer(&peer_id).unwrap().status,
            PeerStatus::Stale { .. }
        ));

        // After 201 more seconds, should be dormant
        let events = cell.tick(now + 301);
        assert!(events.contains(&PeerEvent::PeerBecameDormant(peer_id)));
        assert!(matches!(
            cell.get_peer(&peer_id).unwrap().status,
            PeerStatus::Dormant { .. }
        ));
    }

    #[test]
    fn test_update_peer_hints() {
        let local_id = make_peer_id(1);
        let mut cell = LocalCell::new(local_id);
        let peer_id = make_peer_id(2);

        cell.peer_seen(peer_id, TransportType::BLE);

        let hints = vec![make_hint(100), make_hint(200)];
        cell.update_peer_hints(&peer_id, hints.clone());

        let peer = cell.get_peer(&peer_id).unwrap();
        assert_eq!(peer.reachable_hints, hints);
    }

    #[test]
    fn test_peers_for_hint() {
        let local_id = make_peer_id(1);
        let mut cell = LocalCell::new(local_id);
        let peer1 = make_peer_id(2);
        let peer2 = make_peer_id(3);
        let peer3 = make_peer_id(4);

        cell.peer_seen(peer1, TransportType::BLE);
        cell.peer_seen(peer2, TransportType::BLE);
        cell.peer_seen(peer3, TransportType::BLE);

        let hint_a = make_hint(100);
        let hint_b = make_hint(200);

        cell.update_peer_hints(&peer1, vec![hint_a]);
        cell.update_peer_hints(&peer2, vec![hint_a, hint_b]);
        cell.update_peer_hints(&peer3, vec![hint_b]);

        let peers_for_a = cell.peers_for_hint(&hint_a);
        assert_eq!(peers_for_a.len(), 2);
        assert!(peers_for_a.iter().any(|p| p.peer_id == peer1));
        assert!(peers_for_a.iter().any(|p| p.peer_id == peer2));

        let peers_for_b = cell.peers_for_hint(&hint_b);
        assert_eq!(peers_for_b.len(), 2);
        assert!(peers_for_b.iter().any(|p| p.peer_id == peer2));
        assert!(peers_for_b.iter().any(|p| p.peer_id == peer3));
    }

    #[test]
    fn test_reliability_scoring() {
        let local_id = make_peer_id(1);
        let mut cell = LocalCell::new(local_id);
        let peer_id = make_peer_id(2);

        cell.peer_seen(peer_id, TransportType::BLE);
        let initial_score = cell.get_peer(&peer_id).unwrap().reliability_score;

        // Success should increase score
        cell.update_reliability(&peer_id, true);
        let after_success = cell.get_peer(&peer_id).unwrap().reliability_score;
        assert!(after_success > initial_score);

        // Failure should decrease score
        cell.update_reliability(&peer_id, false);
        let after_failure = cell.get_peer(&peer_id).unwrap().reliability_score;
        assert!(after_failure < after_success);

        // Scores should be clamped to [0.0, 1.0]
        for _ in 0..20 {
            cell.update_reliability(&peer_id, false);
        }
        let final_score = cell.get_peer(&peer_id).unwrap().reliability_score;
        assert!(final_score >= 0.0);
        assert!(final_score <= 1.0);
    }

    #[test]
    fn test_gateway_detection() {
        let local_id = make_peer_id(1);
        let mut cell = LocalCell::new(local_id);
        let peer_id = make_peer_id(2);

        cell.peer_seen(peer_id, TransportType::BLE);
        assert!(!cell.get_peer(&peer_id).unwrap().is_gateway);

        cell.mark_as_gateway(&peer_id, true);
        assert!(cell.get_peer(&peer_id).unwrap().is_gateway);

        // Gateway peers should appear in gateway_peers()
        let gateways = cell.gateway_peers();
        assert_eq!(gateways.len(), 1);
        assert_eq!(gateways[0].peer_id, peer_id);
    }

    #[test]
    fn test_max_peers_eviction() {
        let local_id = make_peer_id(1);
        let mut cell = LocalCell::with_timeouts(local_id, 100, 200);
        cell.max_peers = 3;

        let peer1 = make_peer_id(2);
        let peer2 = make_peer_id(3);
        let peer3 = make_peer_id(4);
        let peer4 = make_peer_id(5);

        cell.peer_seen(peer1, TransportType::BLE);
        cell.update_reliability(&peer1, true); // high score

        cell.peer_seen(peer2, TransportType::BLE);
        cell.update_reliability(&peer2, true);

        cell.peer_seen(peer3, TransportType::BLE);
        // peer3 starts at 0.5 score

        assert_eq!(cell.peer_count(), 3);

        // Adding peer4 should evict peer3 (lowest score)
        cell.peer_seen(peer4, TransportType::BLE);

        assert_eq!(cell.peer_count(), 3);
        assert!(cell.get_peer(&peer3).is_none());
        assert!(cell.get_peer(&peer4).is_some());
    }

    #[test]
    fn test_record_sync() {
        let local_id = make_peer_id(1);
        let mut cell = LocalCell::new(local_id);
        let peer_id = make_peer_id(2);

        cell.peer_seen(peer_id, TransportType::BLE);

        cell.record_sync(&peer_id, 100, 5);
        let peer = cell.get_peer(&peer_id).unwrap();
        assert_eq!(peer.sync_count, 1);
        assert_eq!(peer.avg_sync_ms, 100);
        assert_eq!(peer.message_count, 5);

        cell.record_sync(&peer_id, 200, 10);
        let peer = cell.get_peer(&peer_id).unwrap();
        assert_eq!(peer.sync_count, 2);
        assert_eq!(peer.avg_sync_ms, 150); // (100 + 200) / 2
        assert_eq!(peer.message_count, 10);
    }

    #[test]
    fn test_cell_summary_generation() {
        let local_id = make_peer_id(1);
        let mut cell = LocalCell::new(local_id);
        let peer1 = make_peer_id(2);
        let peer2 = make_peer_id(3);

        cell.peer_seen(peer1, TransportType::BLE);
        cell.peer_seen(peer2, TransportType::BLE);
        cell.mark_as_gateway(&peer1, true);

        let hint_a = make_hint(100);
        cell.update_peer_hints(&peer1, vec![hint_a]);
        cell.update_peer_hints(&peer2, vec![hint_a, make_hint(200)]);

        let summary = cell.summarize();
        assert_eq!(summary.peer_count, 2);
        assert_eq!(summary.gateway_count, 1);
        assert_eq!(summary.reachable_hints.len(), 2);
        assert!(summary.reachable_hints.contains(&hint_a));
    }

    #[test]
    fn active_peer_selection_contract() {
        let local_id = make_peer_id(1);
        let mut cell = LocalCell::new(local_id);
        let peer1 = make_peer_id(2);
        let peer2 = make_peer_id(3);
        let peer3 = make_peer_id(4);

        cell.peer_seen(peer1, TransportType::BLE);
        cell.peer_seen(peer2, TransportType::BLE);
        cell.peer_seen(peer3, TransportType::BLE);

        cell.update_reliability(&peer1, true);
        cell.update_reliability(&peer1, true);
        cell.update_reliability(&peer2, true);
        cell.update_reliability(&peer3, false);

        let active = cell.active_peers();
        assert_eq!(
            active.iter().map(|peer| peer.peer_id).collect::<Vec<_>>(),
            vec![peer1, peer2, peer3]
        );

        let hint = make_hint(100);
        cell.update_peer_hints(&peer1, vec![hint]);
        cell.update_peer_hints(&peer2, vec![hint]);
        assert_eq!(cell.peers_for_hint(&hint)[0].peer_id, peer1);
    }

    #[test]
    fn test_stale_peers_not_in_peers_for_hint() {
        let local_id = make_peer_id(1);
        let mut cell = LocalCell::with_timeouts(local_id, 100, 200);
        let peer_id = make_peer_id(2);

        cell.peer_seen(peer_id, TransportType::BLE);
        let hint = make_hint(100);
        cell.update_peer_hints(&peer_id, vec![hint]);

        // Active peer should be found
        assert_eq!(cell.peers_for_hint(&hint).len(), 1);

        // Make peer stale
        let now = current_timestamp();
        cell.tick(now + 101);

        // Stale peer should not be found
        assert_eq!(cell.peers_for_hint(&hint).len(), 0);
    }

    #[test]
    fn test_multiple_transports() {
        let local_id = make_peer_id(1);
        let mut cell = LocalCell::new(local_id);
        let peer_id = make_peer_id(2);

        cell.peer_seen(peer_id, TransportType::BLE);
        cell.peer_seen(peer_id, TransportType::WiFiDirect);
        cell.peer_seen(peer_id, TransportType::TCP);

        let peer = cell.get_peer(&peer_id).unwrap();
        assert_eq!(peer.transports.len(), 3);
        assert!(peer.transports.contains(&TransportType::BLE));
        assert!(peer.transports.contains(&TransportType::WiFiDirect));
        assert!(peer.transports.contains(&TransportType::TCP));
    }

    #[test]
    fn test_local_id_preserved() {
        let local_id = make_peer_id(42);
        let cell = LocalCell::new(local_id);
        assert_eq!(cell.local_id(), local_id);
    }
}
