//! Layer 1 — Mycelium (Local Cell Topology)
//!
//! The local cell maintains complete topology awareness of all directly reachable peers.
//! This includes:
//! - Real-time peer status (Active, Stale, Dormant)
//! - Transport connectivity (BLE, WiFi, TCP, QUIC)
//! - Recipient reachability via recipient hints
//! - Peer reliability scores based on relay success
//! - Gateway detection for Layer 2 gossip propagation

use std::collections::{HashMap, HashSet};
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
    pub reachable_hints: Vec<[u8; 8]>,
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
    pub reachable_hints: Vec<[u8; 8]>,
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
    /// Hint -> (peer -> announce time, unix secs). Kept consistent with every
    /// `PeerInfo::reachable_hints` on update, evict, expire and peer removal.
    hint_index: HashMap<[u8; 8], HashMap<PeerId, u64>>,
    /// Number of indexed non-direct (announced) hint entries.
    announced_total: usize,
    /// Seconds an announced hint lives without a re-announce.
    hint_ttl: u64,
}

/// Hint budget floor: every peer, however new, may announce this many hints.
const HINT_BUDGET_FLOOR: usize = 8;
/// Extra hints a fully trusted peer (reliability 1.0) earns above the floor.
const HINT_BUDGET_TRUST_SPAN: usize = 120;
/// Average announced hints per tracked peer; scales the global budget with
/// the routing table size.
const HINT_BUDGET_PER_PEER_AVG: usize = 64;

impl LocalCell {
    /// Create a new local cell with default timeouts
    pub fn new(local_id: PeerId) -> Self {
        Self {
            local_id,
            peers: HashMap::new(),
            active_timeout: 300, // 5 minutes
            stale_timeout: 1800, // 30 minutes
            max_peers: 1000,
            hint_index: HashMap::new(),
            announced_total: 0,
            hint_ttl: 1800,
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
            hint_index: HashMap::new(),
            announced_total: 0,
            hint_ttl: stale_timeout,
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
    ///
    /// Bounded: the announced list is deduped and trimmed to the peer's
    /// reliability-derived budget before any state changes, then the global
    /// budget (scaled by routing table size) is enforced by evicting the
    /// lowest-reliability, oldest hints. The peer's direct hint is exempt.
    pub fn update_peer_hints(&mut self, peer_id: &PeerId, hints: Vec<[u8; 8]>) {
        self.update_peer_hints_at(peer_id, hints, current_timestamp());
    }

    /// Per-peer hint budget: a small floor that scales with reliability.
    pub fn peer_hint_budget(reliability: f64) -> usize {
        HINT_BUDGET_FLOOR + (reliability.clamp(0.0, 1.0) * HINT_BUDGET_TRUST_SPAN as f64) as usize
    }

    /// Largest per-peer budget any peer can earn (reliability 1.0). Callers
    /// use it to bound work before taking a lock.
    pub fn max_peer_hint_budget() -> usize {
        Self::peer_hint_budget(1.0)
    }

    /// Global announced-hint budget, derived from the routing table size.
    pub fn global_hint_budget(&self) -> usize {
        self.peers.len().max(1) * HINT_BUDGET_PER_PEER_AVG
    }

    /// Number of indexed (hint, peer) entries.
    pub fn indexed_hint_count(&self) -> usize {
        self.hint_index.values().map(|m| m.len()).sum()
    }

    fn update_peer_hints_at(&mut self, peer_id: &PeerId, hints: Vec<[u8; 8]>, now: u64) {
        let Some(peer) = self.peers.get_mut(peer_id) else {
            return;
        };
        let direct = crate::identity::keys::routing_hint_from_public_key(peer_id);
        let budget = Self::peer_hint_budget(peer.reliability_score);
        // Dedupe and trim first; nothing below runs per announced duplicate.
        let mut seen: HashSet<[u8; 8]> = HashSet::new();
        let mut keep_direct = peer.reachable_hints.contains(&direct);
        let mut kept: Vec<[u8; 8]> = Vec::new();
        for hint in hints {
            if hint == direct {
                keep_direct = true;
            } else if kept.len() < budget && seen.insert(hint) {
                kept.push(hint);
            }
        }
        let old = std::mem::take(&mut peer.reachable_hints);
        for hint in &old {
            if *hint != direct && !seen.contains(hint) {
                self.index_remove(peer_id, hint);
            }
        }
        for hint in &kept {
            self.index_insert(peer_id, hint, now);
        }
        if keep_direct {
            kept.push(direct);
            self.index_insert(peer_id, &direct, now);
        }
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.reachable_hints = kept;
        }
        self.enforce_global_hint_budget();
    }

    fn index_insert(&mut self, peer_id: &PeerId, hint: &[u8; 8], now: u64) {
        let prev = self
            .hint_index
            .entry(*hint)
            .or_default()
            .insert(*peer_id, now);
        if prev.is_none() && *hint != crate::identity::keys::routing_hint_from_public_key(peer_id) {
            self.announced_total += 1;
        }
    }

    fn index_remove(&mut self, peer_id: &PeerId, hint: &[u8; 8]) {
        if let Some(holders) = self.hint_index.get_mut(hint) {
            if holders.remove(peer_id).is_some()
                && *hint != crate::identity::keys::routing_hint_from_public_key(peer_id)
            {
                self.announced_total = self.announced_total.saturating_sub(1);
            }
            if holders.is_empty() {
                self.hint_index.remove(hint);
            }
        }
    }

    /// Drop announced hints (never direct hints) over the global budget,
    /// lowest-reliability first, then oldest announcement first.
    fn enforce_global_hint_budget(&mut self) {
        let budget = self.global_hint_budget();
        if self.announced_total <= budget {
            return;
        }
        let excess = self.announced_total - budget;
        let mut ranked: Vec<(f64, u64, PeerId, [u8; 8])> = Vec::new();
        for (hint, holders) in &self.hint_index {
            for (peer_id, at) in holders {
                if *hint == crate::identity::keys::routing_hint_from_public_key(peer_id) {
                    continue;
                }
                let score = self.peers.get(peer_id).map_or(0.0, |p| p.reliability_score);
                ranked.push((score, *at, *peer_id, *hint));
            }
        }
        ranked.sort_by(|a, b| {
            a.0.total_cmp(&b.0)
                .then_with(|| a.1.cmp(&b.1))
                .then_with(|| a.2.cmp(&b.2))
                .then_with(|| a.3.cmp(&b.3))
        });
        for (_, _, peer_id, hint) in ranked.into_iter().take(excess) {
            self.index_remove(&peer_id, &hint);
            if let Some(peer) = self.peers.get_mut(&peer_id) {
                peer.reachable_hints.retain(|h| *h != hint);
            }
        }
    }

    /// Remove announced hints not re-announced within the TTL. Direct hints
    /// never expire.
    pub fn expire_hints(&mut self, now: u64) {
        let ttl = self.hint_ttl;
        let mut expired: Vec<(PeerId, [u8; 8])> = Vec::new();
        for (hint, holders) in &self.hint_index {
            for (peer_id, at) in holders {
                if now.saturating_sub(*at) > ttl
                    && *hint != crate::identity::keys::routing_hint_from_public_key(peer_id)
                {
                    expired.push((*peer_id, *hint));
                }
            }
        }
        for (peer_id, hint) in expired {
            self.index_remove(&peer_id, &hint);
            if let Some(peer) = self.peers.get_mut(&peer_id) {
                peer.reachable_hints.retain(|h| *h != hint);
            }
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
    ///
    /// V040-T13 F7-C: the engine's `route_message` takes `local_peers[0]` as
    /// the primary next hop on the strength of a documented contract that this
    /// list is "already sorted by reliability in LocalCell". Sort by the shared
    /// reliability policy (score, then peer_id), never HashMap order.
    pub fn peers_for_hint(&self, hint: &[u8; 8]) -> Vec<&PeerInfo> {
        let Some(holders) = self.hint_index.get(hint) else {
            return Vec::new();
        };
        let now = current_timestamp();
        let mut peers: Vec<&PeerInfo> = holders
            .iter()
            .filter_map(|(peer_id, at)| {
                let peer = self.peers.get(peer_id)?;
                let fresh = now.saturating_sub(*at) <= self.hint_ttl
                    || *hint == crate::identity::keys::routing_hint_from_public_key(peer_id);
                (fresh && matches!(peer.status, PeerStatus::Active { .. })).then_some(peer)
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

        self.expire_hints(now);

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

        let all_hints: Vec<[u8; 8]> = self.hint_index.keys().copied().collect();

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

        // Same ordering as sort_by_reliability: lowest score first, peer-id
        // tie-break, so equal-score eviction is deterministic, not
        // HashMap-iteration-order dependent.
        let mut ranked: Vec<&PeerInfo> = self.peers.values().collect();
        Self::sort_by_reliability(&mut ranked);
        let peer_to_evict = *ranked
            .last()
            .map(|p| &p.peer_id)
            .expect("checked non-empty above");

        if let Some(removed) = self.peers.remove(&peer_to_evict) {
            for hint in &removed.reachable_hints {
                self.index_remove(&peer_to_evict, hint);
            }
        }
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

    fn make_hint(n: u32) -> [u8; 8] {
        let mut hint = [0u8; 8];
        hint[..4].copy_from_slice(&n.to_le_bytes());
        hint
    }

    fn hint_test_cell(peers: u8) -> LocalCell {
        let mut cell = LocalCell::new(make_peer_id(250));
        for n in 1..=peers {
            cell.peer_seen(make_peer_id(n), TransportType::TCP);
        }
        cell
    }

    fn flood(from: u32, count: u32) -> Vec<[u8; 8]> {
        (from..from + count).map(make_hint).collect()
    }

    /// reachable_hints and the index describe exactly the same set, and the
    /// announced counter matches the non-direct entries.
    fn assert_index_consistent(cell: &LocalCell) {
        let mut announced = 0usize;
        for (hint, holders) in &cell.hint_index {
            assert!(!holders.is_empty(), "empty holder set left in index");
            for peer_id in holders.keys() {
                let peer = cell.peers.get(peer_id).expect("indexed peer must exist");
                assert!(peer.reachable_hints.contains(hint));
                if *hint != crate::identity::keys::routing_hint_from_public_key(peer_id) {
                    announced += 1;
                }
            }
        }
        assert_eq!(announced, cell.announced_total);
        let listed: usize = cell.peers.values().map(|p| p.reachable_hints.len()).sum();
        assert_eq!(listed, cell.indexed_hint_count());
    }

    #[test]
    fn hint_flood_stays_within_peer_and_global_budget() {
        let mut cell = hint_test_cell(1);
        let peer = make_peer_id(1);
        cell.update_peer_hints(&peer, flood(0, 100_000));
        let budget = LocalCell::peer_hint_budget(cell.get_peer(&peer).unwrap().reliability_score);
        assert!(cell.get_peer(&peer).unwrap().reachable_hints.len() <= budget);
        assert!(cell.announced_total <= cell.global_hint_budget());
        assert_eq!(cell.indexed_hint_count(), budget);
        assert_index_consistent(&cell);
        // Lookup is index-bound: a flooded-out hint has no holders.
        assert!(cell.peers_for_hint(&make_hint(99_999)).is_empty());
        assert_eq!(cell.peers_for_hint(&make_hint(0)).len(), 1);
    }

    #[test]
    fn many_peers_flooding_stay_within_global_budget() {
        let mut cell = hint_test_cell(10);
        for n in 1..=10u8 {
            cell.update_peer_hints(&make_peer_id(n), flood(n as u32 * 1000, 5_000));
        }
        assert!(cell.announced_total <= cell.global_hint_budget());
        assert_index_consistent(&cell);
    }

    #[test]
    fn budget_scales_with_reliability_and_has_a_floor() {
        assert_eq!(LocalCell::peer_hint_budget(0.0), HINT_BUDGET_FLOOR);
        assert!(LocalCell::peer_hint_budget(0.5) > LocalCell::peer_hint_budget(0.0));
        assert_eq!(
            LocalCell::peer_hint_budget(1.0),
            LocalCell::max_peer_hint_budget()
        );
        assert_eq!(LocalCell::peer_hint_budget(f64::NAN), HINT_BUDGET_FLOOR);
    }

    #[test]
    fn global_eviction_takes_lowest_reliability_first() {
        let mut cell = hint_test_cell(2);
        let (low, high) = (make_peer_id(1), make_peer_id(2));
        for _ in 0..6 {
            cell.update_reliability(&high, true);
        }
        let high_budget = LocalCell::max_peer_hint_budget();
        let low_budget = LocalCell::peer_hint_budget(0.5);
        assert!(high_budget + low_budget > cell.global_hint_budget());
        cell.update_peer_hints_at(&low, flood(0, low_budget as u32), 100);
        cell.update_peer_hints_at(&high, flood(10_000, high_budget as u32), 200);
        let excess = high_budget + low_budget - cell.global_hint_budget();
        assert_eq!(cell.announced_total, cell.global_hint_budget());
        assert_eq!(
            cell.get_peer(&high).unwrap().reachable_hints.len(),
            high_budget
        );
        assert_eq!(
            cell.get_peer(&low).unwrap().reachable_hints.len(),
            low_budget - excess
        );
        assert_index_consistent(&cell);
    }

    #[test]
    fn global_eviction_takes_oldest_first_at_equal_reliability() {
        let mut cell = hint_test_cell(2);
        let (old, new) = (make_peer_id(1), make_peer_id(2));
        let budget = LocalCell::peer_hint_budget(0.5);
        assert!(2 * budget > cell.global_hint_budget());
        cell.update_peer_hints_at(&old, flood(0, budget as u32), 100);
        cell.update_peer_hints_at(&new, flood(10_000, budget as u32), 200);
        let excess = 2 * budget - cell.global_hint_budget();
        assert_eq!(cell.get_peer(&new).unwrap().reachable_hints.len(), budget);
        assert_eq!(
            cell.get_peer(&old).unwrap().reachable_hints.len(),
            budget - excess
        );
        assert_index_consistent(&cell);
    }

    #[test]
    fn announced_hints_expire_after_ttl_and_reannounce_refreshes() {
        let mut cell = hint_test_cell(1);
        let peer = make_peer_id(1);
        let ttl = cell.hint_ttl;
        cell.update_peer_hints_at(&peer, vec![make_hint(1)], 1_000);
        cell.update_peer_hints_at(&peer, vec![make_hint(1), make_hint(2)], 2_000);
        // make_hint(1) was refreshed at 2_000; neither is expired yet.
        cell.expire_hints(1_000 + ttl + 1);
        assert_eq!(cell.indexed_hint_count(), 2);
        cell.expire_hints(2_000 + ttl + 1);
        assert_eq!(cell.indexed_hint_count(), 0);
        assert!(cell.get_peer(&peer).unwrap().reachable_hints.is_empty());
        assert_index_consistent(&cell);
        // Lookup also ignores an expired hint before the next tick.
        let stale_at = current_timestamp() - ttl - 1;
        cell.update_peer_hints_at(&peer, vec![make_hint(3)], stale_at);
        assert!(cell.peers_for_hint(&make_hint(3)).is_empty());
    }

    #[test]
    fn direct_hint_survives_flood_expiry_and_reannounce_without_it() {
        let mut cell = hint_test_cell(1);
        let peer = make_peer_id(1);
        let direct = crate::identity::keys::routing_hint_from_public_key(&peer);
        cell.update_peer_hints_at(&peer, vec![direct], 100);
        cell.update_peer_hints_at(&peer, flood(0, 100_000), 200);
        assert!(cell
            .get_peer(&peer)
            .unwrap()
            .reachable_hints
            .contains(&direct));
        cell.update_peer_hints_at(&peer, vec![make_hint(7)], 300);
        cell.expire_hints(300 + cell.hint_ttl + 1);
        assert!(cell
            .get_peer(&peer)
            .unwrap()
            .reachable_hints
            .contains(&direct));
        assert_eq!(cell.peers_for_hint(&direct).len(), 1);
        assert_index_consistent(&cell);
    }

    #[test]
    fn index_stays_consistent_after_peer_removal() {
        let mut cell = hint_test_cell(3);
        cell.max_peers = 3;
        for n in 1..=3u8 {
            cell.update_peer_hints(&make_peer_id(n), flood(n as u32 * 100, 20));
        }
        cell.update_reliability(&make_peer_id(1), false);
        cell.update_reliability(&make_peer_id(2), true);
        cell.update_reliability(&make_peer_id(3), true);
        // A fourth peer forces eviction of the lowest-reliability peer (1).
        cell.peer_seen(make_peer_id(4), TransportType::TCP);
        assert!(cell.get_peer(&make_peer_id(1)).is_none());
        assert!(cell.peers_for_hint(&make_hint(100)).is_empty());
        assert!(!cell.hint_index.contains_key(&make_hint(100)));
        assert_index_consistent(&cell);
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

    /// V040-T13 F7-C: `peers_for_hint` must return the most reliable peer
    /// first -- the engine takes `[0]` as the primary next hop on the strength
    /// of that contract. HashMap iteration order is not a substitute.
    #[test]
    fn test_peers_for_hint_sorted_by_reliability() {
        let local_id = make_peer_id(1);
        let mut cell = LocalCell::new(local_id);
        let hint = make_hint(7);
        let weak_peer = make_peer_id(2);
        let strong_peer = make_peer_id(3);

        cell.peer_seen(weak_peer, TransportType::BLE);
        cell.peer_seen(strong_peer, TransportType::BLE);
        cell.update_peer_hints(&weak_peer, vec![hint]);
        cell.update_peer_hints(&strong_peer, vec![hint]);
        // strong_peer scores higher than weak_peer regardless of insertion
        // order: update_reliability bumps the score 0.1 per success.
        cell.update_reliability(&strong_peer, true);
        cell.update_reliability(&strong_peer, true);
        cell.update_reliability(&weak_peer, true);

        let matches = cell.peers_for_hint(&hint);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].peer_id, strong_peer, "highest reliability first");
        assert_eq!(matches[1].peer_id, weak_peer);
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
