//! Routing Decision Engine — Unified Layer Integration
//!
//! Combines all three routing layers (Local, Neighborhood, Global) to make optimal routing
//! decisions for every message. Implements mycorrhizal network principles: try direct paths
//! first, then nearby relays, then distant routes, and finally store-and-carry.
//!
//! Decision algorithm for every message:
//! 1. **Layer 1 (Mycelium)**: Check local cell for direct peer
//! 2. **Layer 2 (Rhizomorphs)**: Check neighborhood gateways if local misses
//! 3. **Layer 3 (CMN)**: Check global routes if neighborhood misses
//! 4. **Store-and-Carry**: No known route — hold until one appears or request discovery

use super::global::GlobalRoutes;
use super::local::{LocalCell, PeerId, TransportType};
use super::neighborhood::NeighborhoodTable;

/// Where to send a message next
#[derive(Debug, Clone)]
pub enum NextHop {
    /// Deliver directly to a local peer
    Direct {
        peer_id: PeerId,
        transport: TransportType,
    },
    /// Forward through a gateway peer (Layer 2 path)
    Gateway {
        gateway_id: PeerId,
        transport: TransportType,
        hops_remaining: u8,
    },
    /// Forward through a global route (Layer 3 path)
    GlobalRoute { next_hop_id: PeerId, total_hops: u8 },
    /// Store and wait — no route known, carry until one appears
    StoreAndCarry,
    /// Broadcast route request — ask the network "who can reach this?"
    RouteDiscovery { hint: [u8; 8] },
}

/// Which routing layer decided the route
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingLayer {
    /// Layer 1 — direct peer in local cell
    Local,
    /// Layer 2 — via gateway in neighborhood
    Neighborhood,
    /// Layer 3 — via internet-connected global route
    Global,
    /// No route known — epidemic/opportunistic delivery
    StoreAndCarry,
}

/// Full routing decision with metadata
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    /// Which message this route is for
    pub message_id: [u8; 16],
    /// The recipient we're trying to reach
    pub recipient_hint: [u8; 8],
    /// Primary path (where to try first)
    pub primary: NextHop,
    /// Alternative paths (for redundant delivery of high-priority messages)
    pub alternatives: Vec<NextHop>,
    /// Which layer provided the decision
    pub decided_by: RoutingLayer,
    /// Confidence in this route (0.0 - 1.0)
    pub confidence: f64,
}

/// Result of periodic maintenance tick
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoutingMaintenance {
    /// Peers moved from stale to active
    pub peers_promoted: usize,
    /// Peers moved from active to stale
    pub peers_demoted: usize,
    /// Stale gateways removed
    pub stale_gateways_removed: usize,
    /// Expired routes removed
    pub expired_routes_removed: usize,
}

/// Routing status summary
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoutingSummary {
    /// Total peers in local cell
    pub local_peers: usize,
    /// Active peers (recently seen)
    pub active_peers: usize,
    /// Gateway peers in local cell
    pub gateway_peers: usize,
    /// Neighboring cells (via gateways)
    pub neighborhood_cells: usize,
    /// Total global routes
    pub global_routes: usize,
    /// Unique recipient hints we can reach
    pub total_reachable_hints: usize,
}

/// The unified routing engine
///
/// Manages all three routing layers and provides a single API for routing decisions.
/// Handles periodic maintenance across all layers (peer promotions, demotions, cleanup).
pub struct RoutingEngine {
    /// Layer 1: local cell topology
    local: LocalCell,
    /// Layer 2: neighborhood gateways
    neighborhood: NeighborhoodTable,
    /// Layer 3: global routes via internet peers
    global: GlobalRoutes,
    /// Our own peer ID (Ed25519 public key)
    _local_id: PeerId,
    /// Our recipient hint (first 8 bytes of blake3(our_pk))
    local_hint: [u8; 8],
}

impl RoutingEngine {
    /// Create a new routing engine
    pub fn new(local_id: PeerId, local_hint: [u8; 8]) -> Self {
        RoutingEngine {
            local: LocalCell::new(local_id),
            neighborhood: NeighborhoodTable::new(),
            global: GlobalRoutes::new(),
            _local_id: local_id,
            local_hint,
        }
    }

    /// THE CORE FUNCTION: Decide where to send a message
    ///
    /// Implements the mycorrhizal lookup order:
    /// 1. Check local cell (Layer 1) — do I know this recipient?
    /// 2. Check neighborhood (Layer 2) — does a gateway know?
    /// 3. Check global routes (Layer 3) — is there a known path?
    /// 4. Store-and-carry — hold until a route appears
    pub fn route_message(
        &self,
        recipient_hint: &[u8; 8],
        message_id: &[u8; 16],
        priority: u8,
        _now: u64,
    ) -> RoutingDecision {
        // Layer 1: Check local cell
        let local_peers = self.local.peers_for_hint(recipient_hint);
        if !local_peers.is_empty() {
            // Found direct peer(s)
            // V040-T13 F7-C: `peers_for_hint` now actually delivers on this
            // contract (it sorts by reliability, highest first).
            let best_peer = local_peers[0];
            let transport = best_peer
                .transports
                .first()
                .copied()
                .unwrap_or(TransportType::BLE);

            return RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::Direct {
                    peer_id: best_peer.peer_id,
                    transport,
                },
                alternatives: self.collect_alternative_hops(recipient_hint, RoutingLayer::Local),
                decided_by: RoutingLayer::Local,
                confidence: best_peer.reliability_score.min(0.98), // Very high confidence for direct peers
            };
        }

        // Layer 2: Check neighborhood
        if let Some(gateway_info) = self.neighborhood.best_gateway_for_hint(recipient_hint) {
            return RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::Gateway {
                    gateway_id: gateway_info.gateway_id,
                    transport: gateway_info.transport,
                    hops_remaining: gateway_info.hops_away,
                },
                alternatives: self
                    .collect_alternative_hops(recipient_hint, RoutingLayer::Neighborhood),
                decided_by: RoutingLayer::Neighborhood,
                confidence: 0.85_f64 - (gateway_info.hops_away as f64 * 0.05), // Confidence decreases with hops
            };
        }

        // Layer 3: Check global routes
        if let Some(route) = self.global.best_route_for_hint(recipient_hint) {
            return RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::GlobalRoute {
                    next_hop_id: route.next_hop,
                    total_hops: route.hop_count,
                },
                alternatives: self.collect_alternative_hops(recipient_hint, RoutingLayer::Global),
                decided_by: RoutingLayer::Global,
                confidence: route.reliability, // Use route's own reliability metric
            };
        }

        // Layer 4: No route known
        // Check if we should request a route or just store-and-carry
        let should_request = !self.global.is_route_pending(recipient_hint) && priority >= 100;

        if should_request {
            RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::RouteDiscovery {
                    hint: *recipient_hint,
                },
                alternatives: vec![],
                decided_by: RoutingLayer::StoreAndCarry,
                confidence: 0.0, // No confidence yet
            }
        } else {
            RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::StoreAndCarry,
                alternatives: vec![],
                decided_by: RoutingLayer::StoreAndCarry,
                confidence: 0.0,
            }
        }
    }

    /// Route with redundant paths for high-priority messages
    ///
    /// Returns a routing decision with multiple alternative paths to increase
    /// the chance of successful delivery for critical messages.
    pub fn route_redundant(
        &self,
        recipient_hint: &[u8; 8],
        message_id: &[u8; 16],
        priority: u8,
        redundancy: usize,
        now: u64,
    ) -> RoutingDecision {
        let mut decision = self.route_message(recipient_hint, message_id, priority, now);

        // Collect additional alternative paths based on redundancy level
        if redundancy > 1 {
            decision.alternatives =
                self.collect_alternative_hops_count(recipient_hint, redundancy - 1);
        }

        decision
    }

    /// Is this message addressed to us?
    pub fn is_for_us(&self, recipient_hint: &[u8; 8]) -> bool {
        recipient_hint == &self.local_hint
    }

    // Access individual routing layers (mutable and immutable)

    /// Immutable access to local cell
    pub fn local_cell(&self) -> &LocalCell {
        &self.local
    }

    /// Mutable access to local cell
    pub fn local_cell_mut(&mut self) -> &mut LocalCell {
        &mut self.local
    }

    /// Immutable access to neighborhood table
    pub fn neighborhood(&self) -> &NeighborhoodTable {
        &self.neighborhood
    }

    /// Mutable access to neighborhood table
    pub fn neighborhood_mut(&mut self) -> &mut NeighborhoodTable {
        &mut self.neighborhood
    }

    /// Immutable access to global routes
    pub fn global_routes(&self) -> &GlobalRoutes {
        &self.global
    }

    /// Mutable access to global routes
    pub fn global_routes_mut(&mut self) -> &mut GlobalRoutes {
        &mut self.global
    }

    /// Periodic maintenance across all layers
    ///
    /// Should be called regularly (e.g., every 10-30 seconds). Handles:
    /// - Peer status transitions (Active → Stale → Dormant)
    /// - Stale gateway removal
    /// - Expired route cleanup
    /// - Pending route request management
    pub fn tick(&mut self, now: u64) -> RoutingMaintenance {
        // Layer 1: Process peer status changes
        let events = self.local.tick(now);
        let peers_promoted = events
            .iter()
            .filter(|e| matches!(e, super::local::PeerEvent::PeerBecameActive(_)))
            .count();
        let peers_demoted = events
            .iter()
            .filter(|e| matches!(e, super::local::PeerEvent::PeerBecameStale(_)))
            .count();

        // Layer 2: Cleanup stale gateways
        let stale_gateways_removed = self.neighborhood.cleanup(now);

        // Layer 3: Cleanup expired routes
        let expired_routes_removed = self.global.cleanup(now);

        RoutingMaintenance {
            peers_promoted,
            peers_demoted,
            stale_gateways_removed,
            expired_routes_removed,
        }
    }

    /// Generate a complete routing summary (for diagnostics/UI)
    pub fn routing_summary(&self) -> RoutingSummary {
        let local_summary = self.local.summarize();
        let neighborhood_cells = self.neighborhood.gateway_count();
        let global_routes = self.global.route_count();

        RoutingSummary {
            local_peers: self.local.peer_count(),
            active_peers: self.local.active_count(),
            gateway_peers: local_summary.gateway_count as usize,
            neighborhood_cells,
            global_routes,
            total_reachable_hints: self.count_reachable_hints(),
        }
    }

    // Helper methods

    /// Collect alternative hops for a destination
    fn collect_alternative_hops(&self, hint: &[u8; 8], skip_layer: RoutingLayer) -> Vec<NextHop> {
        let mut alternatives = Vec::new();

        // From Layer 1 (if not the primary)
        if skip_layer != RoutingLayer::Local {
            let local_peers = self.local.peers_for_hint(hint);
            if !local_peers.is_empty() {
                for peer in local_peers.iter().take(1) {
                    if let Some(transport) = peer.transports.first() {
                        alternatives.push(NextHop::Direct {
                            peer_id: peer.peer_id,
                            transport: *transport,
                        });
                    }
                }
            }
        }

        // From Layer 2 (if not the primary)
        if skip_layer != RoutingLayer::Neighborhood {
            if let Some(gw) = self.neighborhood.best_gateway_for_hint(hint) {
                alternatives.push(NextHop::Gateway {
                    gateway_id: gw.gateway_id,
                    transport: gw.transport,
                    hops_remaining: gw.hops_away,
                });
            }
        }

        // From Layer 3 (if not the primary)
        if skip_layer != RoutingLayer::Global {
            if let Some(route) = self.global.best_route_for_hint(hint) {
                alternatives.push(NextHop::GlobalRoute {
                    next_hop_id: route.next_hop,
                    total_hops: route.hop_count,
                });
            }
        }

        alternatives
    }

    /// Collect N alternative hops
    fn collect_alternative_hops_count(&self, hint: &[u8; 8], count: usize) -> Vec<NextHop> {
        let mut alternatives = Vec::new();

        // Layer 1
        let local_peers = self.local.peers_for_hint(hint);
        for peer in local_peers.iter().take(count.min(local_peers.len())) {
            if let Some(transport) = peer.transports.first() {
                alternatives.push(NextHop::Direct {
                    peer_id: peer.peer_id,
                    transport: *transport,
                });
            }
        }

        // Layer 2
        if alternatives.len() < count {
            if let Some(gw) = self.neighborhood.best_gateway_for_hint(hint) {
                alternatives.push(NextHop::Gateway {
                    gateway_id: gw.gateway_id,
                    transport: gw.transport,
                    hops_remaining: gw.hops_away,
                });
            }
        }

        // Layer 3
        if alternatives.len() < count {
            if let Some(route) = self.global.best_route_for_hint(hint) {
                alternatives.push(NextHop::GlobalRoute {
                    next_hop_id: route.next_hop,
                    total_hops: route.hop_count,
                });
            }
        }

        alternatives
    }

    /// Count unique reachable hints across all layers
    fn count_reachable_hints(&self) -> usize {
        let mut hints = std::collections::HashSet::new();

        // From local cell
        let local_summary = self.local.summarize();
        for hint in &local_summary.reachable_hints {
            hints.insert(*hint);
        }

        // From neighborhood
        for hint in self.neighborhood.all_reachable_hints() {
            hints.insert(hint);
        }

        hints.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_peer_id(id: u8) -> PeerId {
        let mut peer = [0u8; 32];
        peer[0] = id;
        peer
    }

    fn make_message_id(id: u8) -> [u8; 16] {
        [id; 16]
    }

    fn make_hint(id: u8) -> [u8; 8] {
        [id, 0, 0, 0, 0, 0, 0, 0]
    }

    #[test]
    fn test_routing_engine_creation() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);

        let engine = RoutingEngine::new(local_id, local_hint);
        assert_eq!(engine._local_id, local_id);
        assert_eq!(engine.local_hint, local_hint);
    }

    #[test]
    fn test_is_for_us() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let other_hint = make_hint(2);

        let engine = RoutingEngine::new(local_id, local_hint);

        assert!(engine.is_for_us(&local_hint));
        assert!(!engine.is_for_us(&other_hint));
    }

    #[test]
    fn test_route_to_unknown_destination() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let engine = RoutingEngine::new(local_id, local_hint);

        let target_hint = make_hint(99);
        let msg_id = make_message_id(1);

        let decision = engine.route_message(&target_hint, &msg_id, 50, 1000);

        assert_eq!(decision.decided_by, RoutingLayer::StoreAndCarry);
        assert!(matches!(decision.primary, NextHop::StoreAndCarry));
        assert_eq!(decision.confidence, 0.0);
    }

    #[test]
    fn test_routing_summary() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let engine = RoutingEngine::new(local_id, local_hint);

        let _summary = engine.routing_summary();
        // Verifies routing_summary() returns without panic
    }

    #[test]
    fn test_tick_returns_maintenance() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let mut engine = RoutingEngine::new(local_id, local_hint);

        let _maintenance = engine.tick(1000);
        // Verifies tick() returns without panic
    }

    #[test]
    fn test_layer_access() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let mut engine = RoutingEngine::new(local_id, local_hint);

        // Immutable access
        let _ = engine.local_cell();
        let _ = engine.neighborhood();
        let _ = engine.global_routes();

        // Mutable access
        let _ = engine.local_cell_mut();
        let _ = engine.neighborhood_mut();
        let _ = engine.global_routes_mut();
    }

    #[test]
    fn test_route_redundant() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let engine = RoutingEngine::new(local_id, local_hint);

        let target_hint = make_hint(99);
        let msg_id = make_message_id(1);

        let decision = engine.route_redundant(&target_hint, &msg_id, 50, 3, 1000);

        // Should have same primary as non-redundant route
        assert_eq!(decision.decided_by, RoutingLayer::StoreAndCarry);
    }

    #[test]
    fn test_high_priority_message_with_no_route() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let engine = RoutingEngine::new(local_id, local_hint);

        let target_hint = make_hint(99);
        let msg_id = make_message_id(1);

        // High priority should trigger route discovery request
        let decision = engine.route_message(&target_hint, &msg_id, 100, 1000);

        // Should request route discovery
        assert!(matches!(decision.primary, NextHop::RouteDiscovery { .. }));
    }

    #[test]
    fn test_routing_layer_enum() {
        assert_ne!(RoutingLayer::Local, RoutingLayer::Neighborhood);
        assert_ne!(RoutingLayer::Neighborhood, RoutingLayer::Global);
        assert_ne!(RoutingLayer::Global, RoutingLayer::StoreAndCarry);
    }

    #[test]
    fn test_next_hop_direct() {
        let peer_id = make_peer_id(10);
        let hop = NextHop::Direct {
            peer_id,
            transport: TransportType::BLE,
        };

        match hop {
            NextHop::Direct {
                peer_id: p,
                transport: t,
            } => {
                assert_eq!(p, peer_id);
                assert_eq!(t, TransportType::BLE);
            }
            _ => panic!("Expected Direct hop"),
        }
    }

    #[test]
    fn test_next_hop_gateway() {
        let gateway_id = make_peer_id(20);
        let hop = NextHop::Gateway {
            gateway_id,
            transport: TransportType::WiFiDirect,
            hops_remaining: 3,
        };

        match hop {
            NextHop::Gateway {
                gateway_id: g,
                transport: t,
                hops_remaining: h,
            } => {
                assert_eq!(g, gateway_id);
                assert_eq!(t, TransportType::WiFiDirect);
                assert_eq!(h, 3);
            }
            _ => panic!("Expected Gateway hop"),
        }
    }

    #[test]
    fn test_next_hop_global_route() {
        let next_hop_id = make_peer_id(30);
        let hop = NextHop::GlobalRoute {
            next_hop_id,
            total_hops: 5,
        };

        match hop {
            NextHop::GlobalRoute {
                next_hop_id: n,
                total_hops: h,
            } => {
                assert_eq!(n, next_hop_id);
                assert_eq!(h, 5);
            }
            _ => panic!("Expected GlobalRoute hop"),
        }
    }

    #[test]
    fn test_next_hop_store_and_carry() {
        let hop = NextHop::StoreAndCarry;
        assert!(matches!(hop, NextHop::StoreAndCarry));
    }

    #[test]
    fn test_next_hop_route_discovery() {
        let hint = make_hint(99);
        let hop = NextHop::RouteDiscovery { hint };
        assert!(matches!(hop, NextHop::RouteDiscovery { .. }));
    }

    #[test]
    fn test_routing_decision_structure() {
        let msg_id = make_message_id(1);
        let hint = make_hint(99);

        let decision = RoutingDecision {
            message_id: msg_id,
            recipient_hint: hint,
            primary: NextHop::StoreAndCarry,
            alternatives: vec![],
            decided_by: RoutingLayer::StoreAndCarry,
            confidence: 0.0,
        };

        assert_eq!(decision.message_id, msg_id);
        assert_eq!(decision.recipient_hint, hint);
        assert_eq!(decision.decided_by, RoutingLayer::StoreAndCarry);
        assert_eq!(decision.confidence, 0.0);
    }

    #[test]
    fn test_maintenance_structure() {
        let maint = RoutingMaintenance {
            peers_promoted: 5,
            peers_demoted: 2,
            stale_gateways_removed: 1,
            expired_routes_removed: 3,
        };

        assert_eq!(maint.peers_promoted, 5);
        assert_eq!(maint.peers_demoted, 2);
        assert_eq!(maint.stale_gateways_removed, 1);
        assert_eq!(maint.expired_routes_removed, 3);
    }

    #[test]
    fn test_routing_summary_structure() {
        let summary = RoutingSummary {
            local_peers: 10,
            active_peers: 8,
            gateway_peers: 2,
            neighborhood_cells: 5,
            global_routes: 100,
            total_reachable_hints: 500,
        };

        assert_eq!(summary.local_peers, 10);
        assert_eq!(summary.active_peers, 8);
        assert_eq!(summary.gateway_peers, 2);
        assert_eq!(summary.neighborhood_cells, 5);
        assert_eq!(summary.global_routes, 100);
        assert_eq!(summary.total_reachable_hints, 500);
    }

    #[test]
    fn test_multiple_routing_decisions_independent() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let engine = RoutingEngine::new(local_id, local_hint);

        let msg1_id = make_message_id(1);
        let msg2_id = make_message_id(2);
        let hint1 = make_hint(10);
        let hint2 = make_hint(20);

        let decision1 = engine.route_message(&hint1, &msg1_id, 50, 1000);
        let decision2 = engine.route_message(&hint2, &msg2_id, 50, 1000);

        assert_eq!(decision1.message_id, msg1_id);
        assert_eq!(decision2.message_id, msg2_id);
        assert_ne!(decision1.recipient_hint, decision2.recipient_hint);
    }

    #[test]
    fn test_engine_thread_safety() {
        // Verify the types can be sent between threads
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let engine = RoutingEngine::new(local_id, local_hint);

        let handle = std::thread::spawn(move || {
            let msg_id = make_message_id(1);
            let hint = make_hint(99);
            engine.route_message(&hint, &msg_id, 50, 1000)
        });

        let decision = handle.join().unwrap();
        assert_eq!(decision.decided_by, RoutingLayer::StoreAndCarry);
    }
}
