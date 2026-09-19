//! Layer 2 — Rhizomorphs (Neighborhood Gossip)
//!
//! The neighborhood layer maintains summarized information about cells 2-3 hops away.
//! This includes:
//! - Gateway peers that connect to other cells
//! - Aggregated cell summaries of neighbors
//! - Hop-based routing via gateway peers
//! - Gossip exchange for knowledge propagation
//! - Stale entry cleanup to maintain freshness

use super::local::{CellSummary, PeerId, TransportType};
use std::collections::HashMap;
use web_time::SystemTime;

/// Energy class for hardware-aware routing (2-bit classification)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default,
)]
pub enum EnergyClass {
    /// Device is charging — lowest routing cost
    Charging,
    /// Battery > 50% — normal routing cost
    #[default]
    High,
    /// Battery 20-50% — elevated routing cost
    Low,
    /// Battery < 20% — highest routing cost, used only as last resort
    Critical,
}

impl EnergyClass {
    /// Cost multiplier for routing decisions
    pub fn cost_multiplier(&self) -> f64 {
        match self {
            EnergyClass::Charging => 0.5,
            EnergyClass::High => 1.0,
            EnergyClass::Low => 2.0,
            EnergyClass::Critical => 8.0,
        }
    }

    /// Classify from battery percentage and charging state
    pub fn from_battery(battery_pct: u8, is_charging: bool) -> Self {
        if is_charging {
            EnergyClass::Charging
        } else if battery_pct > 50 {
            EnergyClass::High
        } else if battery_pct > 20 {
            EnergyClass::Low
        } else {
            EnergyClass::Critical
        }
    }
}

/// A gateway must be at least this old (seconds) before eviction will
/// consider it: freshly-learned gateways carry current routing info and
/// must not be churned by a gossip burst that exceeds max_gateways.
const MIN_GATEWAY_AGE_BEFORE_EVICT_SECS: u64 = 30;

/// Information about a gateway peer that connects to other cells
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GatewayInfo {
    pub gateway_id: PeerId,
    pub cell_summary: CellSummary,
    pub hops_away: u8,
    pub last_updated: u64,
    pub transport: TransportType,
    /// Energy class of the gateway peer (for hardware-aware routing)
    /// Defaults to High for backward-compat with old peers that don't send energy class.
    #[serde(default)]
    pub energy_class: EnergyClass,
}

/// Neighborhood summary — what we know about cells 2-3 hops away
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NeighborhoodSummary {
    /// Total peers reachable through this neighborhood path
    pub total_reachable: u32,
    /// Recipient hints reachable (aggregated from cell summaries)
    pub reachable_hints: Vec<[u8; 8]>,
    /// Average reliability along this path
    pub path_reliability: f64,
    /// Number of hops in this path
    pub hop_count: u8,
    /// Freshness timestamp
    pub timestamp: u64,
}

/// Gossip message exchanged between peers for Layer 2 knowledge sharing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NeighborhoodGossip {
    pub local_summary: CellSummary,
    pub neighborhood_summaries: Vec<NeighborhoodSummary>,
    pub timestamp: u64,
    /// Energy class of the gossiping peer
    /// Defaults to High for backward-compat with old peers.
    #[serde(default)]
    pub energy_class: EnergyClass,
}

/// Layer 2: Neighborhood gossip table
/// Tracks information about cells 2-3 hops beyond our local cell
pub struct NeighborhoodTable {
    /// Gateway → what's beyond it
    gateways: HashMap<PeerId, GatewayInfo>,
    /// Aggregated summaries for reachable neighborhoods
    summaries: Vec<NeighborhoodSummary>,
    /// How stale can gateway info be before we discard it (seconds)
    max_staleness: u64,
    /// Maximum gateways to track
    max_gateways: usize,
    /// Maximum hop count we accept in gossip (prevent routing loops)
    max_hops: u8,
}

impl NeighborhoodTable {
    /// Create a new neighborhood table with default settings
    pub fn new() -> Self {
        Self {
            gateways: HashMap::new(),
            summaries: Vec::new(),
            max_staleness: 3600, // 1 hour
            max_gateways: 100,
            max_hops: 4,
        }
    }

    /// Create with custom max staleness
    pub fn with_max_staleness(max_staleness: u64) -> Self {
        Self {
            gateways: HashMap::new(),
            summaries: Vec::new(),
            max_staleness,
            max_gateways: 100,
            max_hops: 4,
        }
    }

    /// Update gateway info from a gossip exchange
    /// Called when we sync with a gateway peer and they share their cell summary
    pub fn update_gateway(
        &mut self,
        gateway_id: PeerId,
        cell_summary: CellSummary,
        hops: u8,
        transport: TransportType,
    ) {
        self.update_gateway_with_energy(
            gateway_id,
            cell_summary,
            hops,
            transport,
            EnergyClass::default(),
        );
    }

    /// Update gateway info with explicit energy class
    pub fn update_gateway_with_energy(
        &mut self,
        gateway_id: PeerId,
        cell_summary: CellSummary,
        hops: u8,
        transport: TransportType,
        energy_class: EnergyClass,
    ) {
        // Reject gossip with unrealistic hop counts
        if hops > self.max_hops {
            return;
        }

        let now = current_timestamp();

        self.gateways.insert(
            gateway_id,
            GatewayInfo {
                gateway_id,
                cell_summary,
                hops_away: hops,
                last_updated: now,
                transport,
                energy_class,
            },
        );

        // Check if we need to evict to stay under max_gateways
        if self.gateways.len() > self.max_gateways {
            self.evict_stalest_gateway(now);
        }

        // Rebuild neighborhood summaries
        self.rebuild_summaries();
    }

    /// Find gateways that might reach a recipient hint
    pub fn gateways_for_hint(&self, hint: &[u8; 8]) -> Vec<&GatewayInfo> {
        self.gateways
            .values()
            .filter(|g| g.cell_summary.reachable_hints.contains(hint))
            .collect()
    }

    /// Get best gateway for a hint (lowest hops, highest reliability)
    pub fn best_gateway_for_hint(&self, hint: &[u8; 8]) -> Option<&GatewayInfo> {
        self.gateways_for_hint(hint).into_iter().min_by(|a, b| {
            let cost_a = self.gateway_cost(a);
            let cost_b = self.gateway_cost(b);
            cost_a
                .partial_cmp(&cost_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Compute routing cost for a gateway: lower is better.
    /// cost = hops * energy_multiplier * (1 / reliability)
    fn gateway_cost(&self, gw: &GatewayInfo) -> f64 {
        let base_cost = gw.hops_away as f64;
        let energy_mult = gw.energy_class.cost_multiplier();
        let reliability = gw.cell_summary.avg_reliability.max(0.01);
        base_cost * energy_mult / reliability
    }

    /// Process incoming gossip from a peer (they share their neighborhood knowledge)
    pub fn process_gossip(&mut self, from_peer: PeerId, gossip: NeighborhoodGossip) {
        // The local summary tells us about the peer's local cell
        let hops = 1; // Direct peer is 1 hop away

        self.update_gateway(
            from_peer,
            gossip.local_summary,
            hops,
            // We don't know the transport from gossip, so we keep the existing one
            // In real implementation, this would be passed separately
            TransportType::TCP,
        );

        // Process their neighborhood knowledge
        let _now = current_timestamp();
        for neighbor_summary in gossip.neighborhood_summaries {
            // Add hops (they were N hops away, we're 1 hop from them)
            let our_hops = neighbor_summary.hop_count + 1;

            // Only accept if still within our limits
            if our_hops <= self.max_hops {
                // Find or create summary entry
                if let Some(existing) = self.summaries.iter_mut().find(|s| {
                    s.hop_count == our_hops && s.reachable_hints == neighbor_summary.reachable_hints
                }) {
                    // Update if fresher
                    if gossip.timestamp > existing.timestamp {
                        *existing = NeighborhoodSummary {
                            total_reachable: neighbor_summary.total_reachable,
                            reachable_hints: neighbor_summary.reachable_hints,
                            path_reliability: neighbor_summary.path_reliability,
                            hop_count: our_hops,
                            timestamp: gossip.timestamp,
                        };
                    }
                } else {
                    // Add new summary
                    self.summaries.push(NeighborhoodSummary {
                        total_reachable: neighbor_summary.total_reachable,
                        reachable_hints: neighbor_summary.reachable_hints,
                        path_reliability: neighbor_summary.path_reliability,
                        hop_count: our_hops,
                        timestamp: gossip.timestamp,
                    });
                }
            }
        }

        // Rebuild and deduplicate
        self.rebuild_summaries();
    }

    /// Generate gossip to share with a peer
    pub fn generate_gossip(&self, local_summary: CellSummary) -> NeighborhoodGossip {
        // Convert gateways to neighborhood summaries for gossip
        let mut neighborhood_summaries = self.summaries.clone();

        for gateway in self.gateways.values() {
            // Add each gateway as a neighborhood summary
            let summary = NeighborhoodSummary {
                total_reachable: gateway.cell_summary.peer_count,
                reachable_hints: gateway.cell_summary.reachable_hints.clone(),
                path_reliability: gateway.cell_summary.avg_reliability,
                hop_count: gateway.hops_away,
                timestamp: gateway.last_updated,
            };

            // Check if we already have this summary and only add if newer
            if !neighborhood_summaries.iter().any(|s| {
                s.reachable_hints == summary.reachable_hints && s.hop_count == summary.hop_count
            }) {
                neighborhood_summaries.push(summary);
            }
        }

        NeighborhoodGossip {
            local_summary,
            neighborhood_summaries,
            timestamp: current_timestamp(),
            energy_class: EnergyClass::default(),
        }
    }

    /// Clean up stale entries
    pub fn cleanup(&mut self, now: u64) -> usize {
        let initial_count = self.gateways.len();

        // Remove stale gateways
        self.gateways
            .retain(|_, gateway| now - gateway.last_updated <= self.max_staleness);

        // Remove stale neighborhood summaries
        self.summaries
            .retain(|summary| now - summary.timestamp <= self.max_staleness);

        initial_count - self.gateways.len()
    }

    /// Get all known gateways
    pub fn all_gateways(&self) -> Vec<&GatewayInfo> {
        self.gateways.values().collect()
    }

    /// Reachable hints aggregated across all neighborhoods
    pub fn all_reachable_hints(&self) -> Vec<[u8; 8]> {
        let mut hints = Vec::new();

        // Collect from gateways
        for gateway in self.gateways.values() {
            for hint in &gateway.cell_summary.reachable_hints {
                if !hints.contains(hint) {
                    hints.push(*hint);
                }
            }
        }

        // Collect from neighborhood summaries
        for summary in &self.summaries {
            for hint in &summary.reachable_hints {
                if !hints.contains(hint) {
                    hints.push(*hint);
                }
            }
        }

        hints
    }

    /// Get count of known gateways
    pub fn gateway_count(&self) -> usize {
        self.gateways.len()
    }

    /// Get count of neighborhood summaries
    pub fn summary_count(&self) -> usize {
        self.summaries.len()
    }

    /// Rebuild neighborhood summaries (deduplicate and clean)
    fn rebuild_summaries(&mut self) {
        // Deduplicate by reachable hints (prefer freshest)
        let mut unique_summaries: HashMap<Vec<[u8; 8]>, NeighborhoodSummary> = HashMap::new();

        for summary in self.summaries.drain(..) {
            let key = summary.reachable_hints.clone();
            unique_summaries
                .entry(key)
                .and_modify(|existing| {
                    if summary.timestamp > existing.timestamp {
                        *existing = summary.clone();
                    }
                })
                .or_insert(summary);
        }

        self.summaries = unique_summaries.into_values().collect();
    }

    /// Evict the gateway with the stalest timestamp
    fn evict_stalest_gateway(&mut self, now: u64) {
        if self.gateways.is_empty() {
            return;
        }

        // Evict the gateway with the oldest last_updated timestamp, but only
        // when it is actually stale: a burst of fresh gossip that pushes the
        // table over max_gateways must not churn current routing information
        // (cleanup() removes entries older than max_staleness every tick,
        // which resolves the over-capacity table as entries age).
        let gateway_to_evict = *self
            .gateways
            .values()
            .min_by_key(|g| g.last_updated)
            .map(|g| &g.gateway_id)
            .expect("checked gateways non-empty above");

        if now.saturating_sub(self.gateways[&gateway_to_evict].last_updated)
            < MIN_GATEWAY_AGE_BEFORE_EVICT_SECS
        {
            return;
        }

        self.gateways.remove(&gateway_to_evict);
    }
}

impl Default for NeighborhoodTable {
    fn default() -> Self {
        Self::new()
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

    fn make_cell_summary(hint_list: Vec<[u8; 8]>) -> CellSummary {
        CellSummary {
            peer_count: 5,
            gateway_count: 1,
            reachable_hints: hint_list,
            avg_reliability: 0.8,
            timestamp: current_timestamp(),
        }
    }

    #[test]
    fn test_update_gateway() {
        let mut table = NeighborhoodTable::new();
        let gateway_id = make_peer_id(1);
        let hints = vec![make_hint(100), make_hint(200)];
        let summary = make_cell_summary(hints.clone());

        table.update_gateway(gateway_id, summary, 1, TransportType::TCP);

        assert_eq!(table.gateway_count(), 1);
        let gateway = table.all_gateways()[0];
        assert_eq!(gateway.gateway_id, gateway_id);
        assert_eq!(gateway.hops_away, 1);
        assert_eq!(gateway.cell_summary.reachable_hints, hints);
    }

    #[test]
    fn test_gateways_for_hint() {
        let mut table = NeighborhoodTable::new();
        let gateway1 = make_peer_id(1);
        let gateway2 = make_peer_id(2);
        let gateway3 = make_peer_id(3);

        let hint_a = make_hint(100);
        let hint_b = make_hint(200);
        let hint_c = make_hint(300);

        table.update_gateway(
            gateway1,
            make_cell_summary(vec![hint_a, hint_b]),
            1,
            TransportType::TCP,
        );
        table.update_gateway(
            gateway2,
            make_cell_summary(vec![hint_b, hint_c]),
            1,
            TransportType::TCP,
        );
        table.update_gateway(
            gateway3,
            make_cell_summary(vec![hint_c]),
            1,
            TransportType::TCP,
        );

        let gateways_for_a = table.gateways_for_hint(&hint_a);
        assert_eq!(gateways_for_a.len(), 1);
        assert_eq!(gateways_for_a[0].gateway_id, gateway1);

        let gateways_for_b = table.gateways_for_hint(&hint_b);
        assert_eq!(gateways_for_b.len(), 2);

        let gateways_for_c = table.gateways_for_hint(&hint_c);
        assert_eq!(gateways_for_c.len(), 2);
    }

    #[test]
    fn test_best_gateway_for_hint_prefers_fewer_hops() {
        let mut table = NeighborhoodTable::new();
        let gateway_2_hops = make_peer_id(1);
        let gateway_1_hop = make_peer_id(2);
        let hint = make_hint(100);

        table.update_gateway(
            gateway_2_hops,
            make_cell_summary(vec![hint]),
            2,
            TransportType::TCP,
        );
        table.update_gateway(
            gateway_1_hop,
            make_cell_summary(vec![hint]),
            1,
            TransportType::TCP,
        );

        let best = table.best_gateway_for_hint(&hint).unwrap();
        assert_eq!(best.gateway_id, gateway_1_hop);
        assert_eq!(best.hops_away, 1);
    }

    #[test]
    fn test_best_gateway_for_hint_prefers_higher_reliability() {
        let mut table = NeighborhoodTable::new();
        let gateway_low_reliability = make_peer_id(1);
        let gateway_high_reliability = make_peer_id(2);
        let hint = make_hint(100);

        let mut summary_low = make_cell_summary(vec![hint]);
        summary_low.avg_reliability = 0.6;

        let mut summary_high = make_cell_summary(vec![hint]);
        summary_high.avg_reliability = 0.9;

        table.update_gateway(gateway_low_reliability, summary_low, 1, TransportType::TCP);
        table.update_gateway(
            gateway_high_reliability,
            summary_high,
            1,
            TransportType::TCP,
        );

        let best = table.best_gateway_for_hint(&hint).unwrap();
        assert_eq!(best.gateway_id, gateway_high_reliability);
    }

    #[test]
    fn test_process_gossip_adds_neighborhood_info() {
        let mut table = NeighborhoodTable::new();
        let peer_id = make_peer_id(1);

        let hint_local = make_hint(100);
        let hint_neighborhood = make_hint(200);

        let local_summary = make_cell_summary(vec![hint_local]);
        let neighborhood_summary = NeighborhoodSummary {
            total_reachable: 10,
            reachable_hints: vec![hint_neighborhood],
            path_reliability: 0.7,
            hop_count: 1,
            timestamp: current_timestamp(),
        };

        let gossip = NeighborhoodGossip {
            local_summary,
            neighborhood_summaries: vec![neighborhood_summary],
            timestamp: current_timestamp(),
            energy_class: EnergyClass::default(),
        };

        table.process_gossip(peer_id, gossip);

        // Should have 1 gateway (the direct peer)
        assert_eq!(table.gateway_count(), 1);

        // Should have 1 neighborhood summary (their neighborhood)
        assert_eq!(table.summary_count(), 1);

        // Check the summary was processed correctly
        let summaries = &table.summaries;
        assert_eq!(summaries[0].hop_count, 2); // We're 1 hop from peer, peer is 1 hop from neighbor
        assert_eq!(summaries[0].reachable_hints, vec![hint_neighborhood]);
    }

    #[test]
    fn test_process_gossip_respects_max_hops() {
        let mut table = NeighborhoodTable::with_max_staleness(3600);
        table.max_hops = 2;

        let peer_id = make_peer_id(1);

        let local_summary = make_cell_summary(vec![make_hint(100)]);

        // Create neighborhood info that claims to be 3 hops away (beyond our max)
        let distant_summary = NeighborhoodSummary {
            total_reachable: 10,
            reachable_hints: vec![make_hint(300)],
            path_reliability: 0.5,
            hop_count: 3, // This exceeds max_hops (2)
            timestamp: current_timestamp(),
        };

        let gossip = NeighborhoodGossip {
            local_summary,
            neighborhood_summaries: vec![distant_summary],
            timestamp: current_timestamp(),
            energy_class: EnergyClass::default(),
        };

        table.process_gossip(peer_id, gossip);

        // Should not add the distant summary since it would exceed max_hops
        assert_eq!(table.summary_count(), 0);
    }

    #[test]
    fn test_cleanup_removes_stale_entries() {
        let mut table = NeighborhoodTable::with_max_staleness(100);
        let gateway_id = make_peer_id(1);

        table.update_gateway(
            gateway_id,
            make_cell_summary(vec![make_hint(100)]),
            1,
            TransportType::TCP,
        );

        assert_eq!(table.gateway_count(), 1);

        // Advance time beyond max staleness
        let now = current_timestamp() + 101;
        let evicted = table.cleanup(now);

        assert_eq!(evicted, 1);
        assert_eq!(table.gateway_count(), 0);
    }

    #[test]
    fn test_generate_gossip() {
        let mut table = NeighborhoodTable::new();
        let gateway_id = make_peer_id(1);
        table.update_gateway(
            gateway_id,
            make_cell_summary(vec![make_hint(100)]),
            2,
            TransportType::TCP,
        );

        let local_summary = make_cell_summary(vec![make_hint(50)]);
        let gossip = table.generate_gossip(local_summary.clone());

        assert_eq!(gossip.local_summary.reachable_hints, vec![make_hint(50)]);
        assert_eq!(gossip.neighborhood_summaries.len(), 1);
    }

    #[test]
    fn test_all_reachable_hints() {
        let mut table = NeighborhoodTable::new();
        let hint_a = make_hint(100);
        let hint_b = make_hint(200);
        let hint_c = make_hint(300);

        table.update_gateway(
            make_peer_id(1),
            make_cell_summary(vec![hint_a, hint_b]),
            1,
            TransportType::TCP,
        );
        table.update_gateway(
            make_peer_id(2),
            make_cell_summary(vec![hint_b, hint_c]),
            1,
            TransportType::TCP,
        );

        let all_hints = table.all_reachable_hints();
        assert_eq!(all_hints.len(), 3);
        assert!(all_hints.contains(&hint_a));
        assert!(all_hints.contains(&hint_b));
        assert!(all_hints.contains(&hint_c));
    }

    #[test]
    fn test_reject_gossip_with_excessive_hops() {
        let mut table = NeighborhoodTable::with_max_staleness(3600);
        table.max_hops = 3;

        let gateway_id = make_peer_id(1);
        let mut summary = make_cell_summary(vec![make_hint(100)]);
        summary.timestamp = current_timestamp();

        // Try to add gateway with hop count exceeding max
        table.update_gateway(gateway_id, summary.clone(), 5, TransportType::TCP);

        // Should be rejected
        assert_eq!(table.gateway_count(), 0);

        // Adding with valid hop count should work
        table.update_gateway(gateway_id, summary, 2, TransportType::TCP);
        assert_eq!(table.gateway_count(), 1);
    }

    #[test]
    fn test_max_gateways_eviction() {
        let mut table = NeighborhoodTable::new();
        table.max_gateways = 3;

        let gateway1 = make_peer_id(1);
        let gateway2 = make_peer_id(2);
        let gateway3 = make_peer_id(3);
        let gateway4 = make_peer_id(4);

        table.update_gateway(
            gateway1,
            make_cell_summary(vec![make_hint(1)]),
            1,
            TransportType::TCP,
        );
        // Manually backdate gateway1 to make it the stalest
        if let Some(g) = table.gateways.get_mut(&gateway1) {
            g.last_updated -= 1000;
        }
        table.update_gateway(
            gateway2,
            make_cell_summary(vec![make_hint(2)]),
            1,
            TransportType::TCP,
        );
        table.update_gateway(
            gateway3,
            make_cell_summary(vec![make_hint(3)]),
            1,
            TransportType::TCP,
        );

        assert_eq!(table.gateway_count(), 3);

        // Adding 4th gateway should evict gateway1 (the stalest)
        table.update_gateway(
            gateway4,
            make_cell_summary(vec![make_hint(4)]),
            1,
            TransportType::TCP,
        );

        assert_eq!(table.gateway_count(), 3);
        assert!(table
            .all_gateways()
            .iter()
            .any(|g| g.gateway_id == gateway4));
    }

    #[test]
    fn test_evict_stalest_skips_fresh_burst() {
        let mut table = NeighborhoodTable::new();
        table.max_gateways = 3;

        let g1 = make_peer_id(1);
        let g2 = make_peer_id(2);
        let g3 = make_peer_id(3);
        let g4 = make_peer_id(4);

        table.update_gateway(
            g1,
            make_cell_summary(vec![make_hint(1)]),
            1,
            TransportType::TCP,
        );
        // Backdate g1 past the minimum eviction age so it is genuinely stale
        if let Some(g) = table.gateways.get_mut(&g1) {
            g.last_updated -= MIN_GATEWAY_AGE_BEFORE_EVICT_SECS + 10;
        }
        table.update_gateway(
            g2,
            make_cell_summary(vec![make_hint(2)]),
            1,
            TransportType::TCP,
        );
        table.update_gateway(
            g3,
            make_cell_summary(vec![make_hint(3)]),
            1,
            TransportType::TCP,
        );
        assert_eq!(table.gateway_count(), 3);

        // Adding a 4th evicts g1 -- it is genuinely stale
        table.update_gateway(
            g4,
            make_cell_summary(vec![make_hint(4)]),
            1,
            TransportType::TCP,
        );
        assert_eq!(table.gateway_count(), 3);
        assert!(!table.all_gateways().iter().any(|g| g.gateway_id == g1));
    }

    #[test]
    fn test_evict_stalest_defers_fresh_burst() {
        let mut table = NeighborhoodTable::new();
        table.max_gateways = 3;

        let g1 = make_peer_id(1);
        let g2 = make_peer_id(2);
        let g3 = make_peer_id(3);
        let g4 = make_peer_id(4);

        table.update_gateway(
            g1,
            make_cell_summary(vec![make_hint(1)]),
            1,
            TransportType::TCP,
        );
        table.update_gateway(
            g2,
            make_cell_summary(vec![make_hint(2)]),
            1,
            TransportType::TCP,
        );
        table.update_gateway(
            g3,
            make_cell_summary(vec![make_hint(3)]),
            1,
            TransportType::TCP,
        );
        assert_eq!(table.gateway_count(), 3);

        // All gateways are fresh (recently learned): adding a 4th defers
        // eviction rather than churning current routing information.
        table.update_gateway(
            g4,
            make_cell_summary(vec![make_hint(4)]),
            1,
            TransportType::TCP,
        );
        assert_eq!(table.gateway_count(), 4);
    }

    #[test]
    fn test_gossip_exchange_propagation() {
        // Scenario: A tells B about C's neighborhood, B should know about C
        let mut table_b = NeighborhoodTable::new();

        let peer_a = make_peer_id(1);
        let hint_c = make_hint(300);

        // A's gossip says it knows about a neighbor C with hint_c
        let a_local_summary = make_cell_summary(vec![make_hint(100)]);
        let a_neighborhood = NeighborhoodSummary {
            total_reachable: 5,
            reachable_hints: vec![hint_c],
            path_reliability: 0.75,
            hop_count: 1, // A is 1 hop from C
            timestamp: current_timestamp(),
        };

        let gossip = NeighborhoodGossip {
            local_summary: a_local_summary,
            neighborhood_summaries: vec![a_neighborhood],
            timestamp: current_timestamp(),
            energy_class: EnergyClass::default(),
        };

        // B processes this gossip
        table_b.process_gossip(peer_a, gossip);

        // B should now know about hint_c (through A)
        let hints = table_b.all_reachable_hints();
        assert!(hints.contains(&hint_c));

        // The route to hint_c should be 2 hops (B->A->C)
        let summaries_with_hint_c: Vec<_> = table_b
            .summaries
            .iter()
            .filter(|s| s.reachable_hints.contains(&hint_c))
            .collect();
        assert!(!summaries_with_hint_c.is_empty());
        assert_eq!(summaries_with_hint_c[0].hop_count, 2);
    }

    #[test]
    fn test_deduplication_prefers_fresh_data() {
        let mut table = NeighborhoodTable::new();
        let peer_id = make_peer_id(1);
        let hint = make_hint(100);

        let now = current_timestamp();

        // Create two versions of the same neighborhood summary
        let old_summary = NeighborhoodSummary {
            total_reachable: 5,
            reachable_hints: vec![hint],
            path_reliability: 0.5,
            hop_count: 1,
            timestamp: now - 100, // Older
        };

        let new_summary = NeighborhoodSummary {
            total_reachable: 5,
            reachable_hints: vec![hint],
            path_reliability: 0.9, // Better reliability
            hop_count: 1,
            timestamp: now, // Newer
        };

        // Process old version first
        let gossip1 = NeighborhoodGossip {
            local_summary: make_cell_summary(vec![]),
            neighborhood_summaries: vec![old_summary],
            timestamp: now - 100,
            energy_class: EnergyClass::default(),
        };
        table.process_gossip(peer_id, gossip1);

        // Then process newer version
        let gossip2 = NeighborhoodGossip {
            local_summary: make_cell_summary(vec![]),
            neighborhood_summaries: vec![new_summary],
            timestamp: now,
            energy_class: EnergyClass::default(),
        };
        table.process_gossip(peer_id, gossip2);

        // Should keep the newer version with better reliability
        let summaries_with_hint: Vec<_> = table
            .summaries
            .iter()
            .filter(|s| s.reachable_hints.contains(&hint))
            .collect();
        assert_eq!(summaries_with_hint.len(), 1);
        assert_eq!(summaries_with_hint[0].path_reliability, 0.9);
    }
}
