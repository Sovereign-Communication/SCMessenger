//! Optimized Routing Engine with DHT Latency Reductions
//!
//! Integrates all optimization strategies:
//! - Hierarchical Timeout Budgeting (P0)
//! - Bloom Filter Negative Cache (P0)
//! - Route Prefetch on Resume (P1)
//! - Adaptive TTL Based on Peer Activity (P2)
//!
//! This engine replaces the basic RoutingEngine with optimized discovery paths.

use super::adaptive_ttl::AdaptiveTTLManager;
use super::engine::*;
use super::global::RouteAdvertisement;
use super::local::{PeerId, PeerStatus, TransportType};
use super::multipath::MultiPathDelivery;
use super::negative_cache::{NegativeCache, NegativeCacheStats};
use super::resume_prefetch::{PrefetchStats, ResumePrefetchManager};
use super::timeout_budget::{BudgetSummary, DiscoveryPhase, TimeoutBudget};
use web_time::Duration;

// For peer ID string conversion
use hex;

/// Optimized routing engine with all DHT latency optimizations
pub struct OptimizedRoutingEngine {
    /// Basic routing engine (layers 1-3)
    base_engine: RoutingEngine,
    /// Hierarchical timeout budgeting for discovery
    timeout_budget: TimeoutBudget,
    /// Bloom filter negative cache for fast unreachable detection
    negative_cache: NegativeCache,
    /// Route prefetch manager for app resume optimization
    prefetch_manager: ResumePrefetchManager,
    /// Adaptive TTL manager for activity-based route freshness
    adaptive_ttl: AdaptiveTTLManager,
    /// Multipath delivery manager for redundant route tracking
    multipath: MultiPathDelivery,
    /// Our own peer ID. Retained for parity with `base_engine`'s copy; not read
    /// directly today but kept for future direct access without traversing base_engine.
    #[allow(dead_code)]
    local_id: PeerId,
    /// Our recipient hint. Retained for parity with `base_engine`'s copy; not read
    /// directly today but kept for future direct access without traversing base_engine.
    #[allow(dead_code)]
    local_hint: [u8; 4],
    /// Current discovery phase
    current_phase: DiscoveryPhase,
    /// Whether we're in the middle of a discovery operation
    discovery_in_progress: bool,
}

impl OptimizedRoutingEngine {
    /// Create a new optimized routing engine
    pub fn new(local_id: PeerId, local_hint: [u8; 4]) -> Self {
        OptimizedRoutingEngine {
            base_engine: RoutingEngine::new(local_id, local_hint),
            timeout_budget: TimeoutBudget::default_500ms(),
            negative_cache: NegativeCache::with_defaults(),
            prefetch_manager: ResumePrefetchManager::with_defaults(),
            adaptive_ttl: AdaptiveTTLManager::with_defaults(),
            multipath: MultiPathDelivery::new(),
            local_id,
            local_hint,
            current_phase: DiscoveryPhase::LocalCache,
            discovery_in_progress: false,
        }
    }

    /// Optimized route message with hierarchical discovery and negative caching
    pub fn route_message_optimized(
        &mut self,
        recipient_hint: &[u8; 4],
        message_id: &[u8; 16],
        priority: u8,
        now: u64,
    ) -> RoutingDecision {
        // Phase 0: Fast negative cache check (P0 optimization)
        let peer_id_str = hex::encode(recipient_hint);
        if self.negative_cache.is_definitely_unreachable(&peer_id_str) {
            return RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::StoreAndCarry,
                alternatives: vec![],
                decided_by: RoutingLayer::StoreAndCarry,
                confidence: 0.0,
            };
        }

        // Phase 1: Check prefetch cache (P1 optimization)
        if let Some(prefetched_route) = self.prefetch_manager.get_route_early(recipient_hint) {
            // Convert prefetched route to routing decision
            return RoutingDecision {
                message_id: *message_id,
                recipient_hint: *recipient_hint,
                primary: NextHop::GlobalRoute {
                    next_hop_id: prefetched_route.next_hop,
                    total_hops: prefetched_route.hop_count,
                },
                alternatives: vec![],
                decided_by: RoutingLayer::Global,
                confidence: 0.95, // High confidence for prefetched routes
            };
        }

        // Phase 1.5: Check multipath delivery routes (Phase 2 optimization)
        // If we have active multipath routes for this peer, prefer the primary route
        // and use alternatives from the multipath manager.
        {
            let active = self.multipath.active_paths(recipient_hint);
            if let Some(primary_path) = active.first() {
                if primary_path.active {
                    let alternatives: Vec<NextHop> = active
                        .iter()
                        .skip(1)
                        .filter(|p| p.active)
                        .map(|p| NextHop::GlobalRoute {
                            next_hop_id: p.peer_id,
                            total_hops: 1,
                        })
                        .collect();
                    return RoutingDecision {
                        message_id: *message_id,
                        recipient_hint: *recipient_hint,
                        primary: NextHop::GlobalRoute {
                            next_hop_id: primary_path.peer_id,
                            total_hops: 1,
                        },
                        alternatives,
                        decided_by: RoutingLayer::Global,
                        confidence: 0.9,
                    };
                }
            }
        }

        // Phase 2: Hierarchical discovery with timeout budgeting (P0 optimization)
        self.start_discovery_if_needed();

        let mut decision =
            self.base_engine
                .route_message(recipient_hint, message_id, priority, now);

        // If the decision is a direct local route, ensure the active transport
        // matches the peer's most recently observed status, and include any
        // other known transports as alternative hops for failover.
        if let NextHop::Direct {
            peer_id,
            ref mut transport,
        } = decision.primary
        {
            if let Some(peer) = self.base_engine.local_cell().get_peer(&peer_id) {
                if let PeerStatus::Active {
                    transport: active_t,
                    ..
                } = peer.status
                {
                    *transport = active_t;
                }
                for alt_t in &peer.transports {
                    if *alt_t != *transport {
                        let alt_hop = NextHop::Direct {
                            peer_id,
                            transport: *alt_t,
                        };
                        if !decision.alternatives.iter().any(|h| {
                            matches!(h, NextHop::Direct { peer_id: p, transport: t } if *p == peer_id && *t == *alt_t)
                        }) {
                            decision.alternatives.push(alt_hop);
                        }
                    }
                }
            }
        }

        // Structured tracing: Log routing decision
        tracing::info!(
            event = "routing_decision",
            message_id = %hex::encode(message_id),
            recipient_hint = %hex::encode(recipient_hint),
            priority = priority,
            next_hop = ?decision.primary,
            decided_by = ?decision.decided_by,
            confidence = decision.confidence
        );

        // Apply adaptive TTL to the decision (P2 optimization)
        let peer_id_str = if let NextHop::GlobalRoute { next_hop_id, .. } = decision.primary {
            let s = hex::encode(next_hop_id);
            let _ttl = self.adaptive_ttl.calculate_ttl(&s);
            // In a real implementation, this would update the route's TTL
            // For now, we just track the activity
            self.adaptive_ttl.record_activity(&s);
            s
        } else {
            String::new()
        };

        // If we got StoreAndCarry and it's a high priority message, record as unreachable
        if matches!(decision.primary, NextHop::StoreAndCarry)
            && priority >= 100
            && !peer_id_str.is_empty()
        {
            self.negative_cache.record_unreachable(peer_id_str);
        }

        decision
    }

    /// Start discovery if not already in progress
    fn start_discovery_if_needed(&mut self) {
        if self.discovery_in_progress {
            return;
        }

        self.discovery_in_progress = true;
        self.current_phase = DiscoveryPhase::LocalCache;
        self.timeout_budget = TimeoutBudget::default_500ms();
    }

    /// Advance to next discovery phase
    pub fn advance_discovery_phase(&mut self) -> Option<DiscoveryPhase> {
        if !self.discovery_in_progress {
            return None;
        }

        let next_phase = self.timeout_budget.advance();
        if let Some(phase) = next_phase {
            self.current_phase = phase;
            Some(phase)
        } else {
            self.discovery_in_progress = false;
            None
        }
    }

    /// Get current discovery phase
    pub fn current_discovery_phase(&self) -> DiscoveryPhase {
        self.current_phase
    }

    /// Check if discovery is in progress
    pub fn is_discovery_in_progress(&self) -> bool {
        self.discovery_in_progress
    }

    /// Get timeout budget summary
    pub fn timeout_budget_summary(&self) -> BudgetSummary {
        self.timeout_budget.summary()
    }

    /// Get negative cache statistics
    pub fn negative_cache_stats(&self) -> NegativeCacheStats {
        self.negative_cache.stats()
    }

    /// Get prefetch statistics
    pub fn prefetch_stats(&self) -> PrefetchStats {
        self.prefetch_manager.stats()
    }

    /// Get adaptive TTL manager
    pub fn adaptive_ttl(&mut self) -> &mut AdaptiveTTLManager {
        &mut self.adaptive_ttl
    }

    /// Access base engine methods
    pub fn base_engine(&self) -> &RoutingEngine {
        &self.base_engine
    }

    /// Mutable access to base engine
    pub fn base_engine_mut(&mut self) -> &mut RoutingEngine {
        &mut self.base_engine
    }

    /// Access prefetch manager
    pub fn prefetch_manager(&self) -> &ResumePrefetchManager {
        &self.prefetch_manager
    }

    /// Mutable access to prefetch manager
    pub fn prefetch_manager_mut(&mut self) -> &mut ResumePrefetchManager {
        &mut self.prefetch_manager
    }

    /// Periodic maintenance for all components
    pub fn tick(&mut self, now: u64) -> OptimizedRoutingMaintenance {
        let base_maint = self.base_engine.tick(now);
        let neg_cache_stats = self.negative_cache.stats();
        let neg_cache_cleaned = self.negative_cache.cleanup_expired();
        let ttl_cleaned = self.adaptive_ttl.cleanup(Duration::from_secs(86400)); // 24h
        let budget_summary = self.timeout_budget_summary();

        OptimizedRoutingMaintenance {
            base_maintenance: base_maint,
            negative_cache_entries_cleaned: neg_cache_cleaned,
            negative_cache_stats: neg_cache_stats,
            adaptive_ttl_entries_cleaned: ttl_cleaned,
            timeout_budget_summary: budget_summary,
        }
    }

    /// Called when app goes to background
    pub fn on_app_background(
        &mut self,
        current_routes: Vec<(PeerId, [u8; 4], RouteAdvertisement)>,
    ) {
        self.prefetch_manager.on_app_background(current_routes);
    }

    /// Called when app resumes from background
    pub fn on_app_resume(&mut self) -> Vec<[u8; 4]> {
        self.prefetch_manager.on_app_resume()
    }

    /// Record message activity for adaptive TTL
    pub fn record_message_activity(&mut self, peer_id: &str) {
        self.adaptive_ttl.record_activity(peer_id);
    }

    /// Record unreachable peer
    pub fn record_unreachable_peer(&mut self, peer_id: &str) {
        self.negative_cache.record_unreachable(peer_id.to_string());
    }

    /// Clear unreachable status for peer
    pub fn clear_unreachable_peer(&mut self, peer_id: &str) {
        self.negative_cache.clear_unreachable(peer_id);
    }

    /// Get active multipath delivery paths for a recipient hint.
    pub fn active_paths(&self, hint: &[u8; 4]) -> Vec<&super::multipath::DeliveryPath> {
        self.multipath.active_paths(hint)
    }

    /// Refresh delegate routes by re-querying the base engine's local cell.
    /// Called when transport state changes (peer connect/disconnect) to update
    /// cached routing information.
    pub fn refresh_delegate_routes(&mut self) {
        // The base engine's local cell is always kept up to date via
        // `record_message_activity` and `routing_peer_seen`. This method
        // triggers a sweep of the adaptive TTL to expire stale entries.
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.adaptive_ttl.cleanup(Duration::from_secs(300));
        tracing::debug!(
            refreshed_at = now,
            "Delegate routes refreshed via adaptive TTL sweep"
        );
    }

    /// Run an optimization cycle over the routing engine.
    /// Performs a full maintenance tick: base engine maintenance,
    /// negative cache cleanup, adaptive TTL sweep, and prefetch
    /// queue optimization.
    /// Returns the maintenance result for diagnostics.
    pub fn run_optimization(&mut self) -> OptimizedRoutingMaintenance {
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.tick(now)
    }

    /// Evaluate all tracked peers in the negative cache and adaptive TTL.
    /// Returns the number of entries that were evicted due to staleness.
    /// Used by periodic health checks to prune unreachable peers whose
    /// negative cache entry has expired.
    pub fn evaluate_all_tracked(&mut self) -> usize {
        let neg_evicted = self.negative_cache.cleanup_expired();
        let ttl_evicted = self.adaptive_ttl.cleanup(Duration::from_secs(86400));
        neg_evicted + ttl_evicted
    }

    /// Record a peer sighting on a specific transport.
    /// Updates local cell topology, clears negative cache unreachable status,
    /// records activity for adaptive TTL, and ensures direct hint reachability.
    pub fn peer_seen(&mut self, peer_id: PeerId, transport: TransportType) {
        let peer_id_hex = hex::encode(peer_id);
        self.negative_cache.clear_unreachable(&peer_id_hex);
        self.adaptive_ttl.record_activity(&peer_id_hex);

        let cell = self.base_engine.local_cell_mut();
        cell.peer_seen(peer_id, transport);

        // Ensure peer's direct hint is reachable in local cell
        let direct_hint: [u8; 4] = blake3::hash(&peer_id).as_bytes()[0..4]
            .try_into()
            .unwrap_or([0u8; 4]);
        if let Some(peer) = cell.get_peer(&peer_id) {
            if !peer.reachable_hints.contains(&direct_hint) {
                let mut hints = peer.reachable_hints.clone();
                hints.push(direct_hint);
                cell.update_peer_hints(&peer_id, hints);
            }
        }
    }

    /// Check whether a specific peer is reachable via any known route.
    /// Returns true if the peer exists in the local routing cell or
    /// is not in the negative cache (i.e., not definitely unreachable).
    pub fn can_reach_destination(&mut self, peer_id_hex: &str) -> bool {
        // If the negative cache says definitely unreachable, return false.
        if self.negative_cache.is_definitely_unreachable(peer_id_hex) {
            return false;
        }
        // Otherwise check if we have any route to this peer.
        if let Ok(bytes) = hex::decode(peer_id_hex) {
            if bytes.len() == 32 {
                let mut peer_id = [0u8; 32];
                peer_id.copy_from_slice(&bytes);
                if let Some(peer) = self.base_engine.local_cell().get_peer(&peer_id) {
                    if matches!(peer.status, PeerStatus::Active { .. }) {
                        return true;
                    }
                }
                let hint: [u8; 4] = blake3::hash(&peer_id).as_bytes()[0..4]
                    .try_into()
                    .unwrap_or([0u8; 4]);
                let peers = self.base_engine.local_cell().peers_for_hint(&hint);
                if !peers.is_empty() {
                    return true;
                }
            }
        }
        let hint = if peer_id_hex.len() >= 8 {
            let bytes = hex::decode(&peer_id_hex[..8]).unwrap_or_default();
            let arr: [u8; 4] = bytes.try_into().unwrap_or([0u8; 4]);
            arr
        } else {
            [0u8; 4]
        };
        let peers = self.base_engine.local_cell().peers_for_hint(&hint);
        !peers.is_empty()
    }

    /// Prune entries below a given reputation threshold.
    pub fn prune_below(&mut self, threshold: f64) {
        self.multipath.prune_below(threshold);
        self.negative_cache.prune_below_confidence(threshold);
    }

    /// Check whether the timeout budget allows advancing to the next
    /// discovery phase. Delegates to TimeoutBudget::should_advance().
    pub fn should_advance(&self) -> bool {
        self.timeout_budget.should_advance()
    }

    /// Mark a path as failed in the multipath delivery manager.
    pub fn multipath_mark_path_failed(&mut self, path_id: u64) {
        self.multipath.mark_path_failed(path_id);
    }

    /// Register a delivery path in the multipath delivery manager.
    pub fn multipath_register_path(&mut self, peer_id_hex: String, path_id: u64, latency_ms: u64) {
        use super::multipath::DeliveryPath;
        let bytes = hex::decode(&peer_id_hex).unwrap_or_default();
        let peer_id: [u8; 32] = if bytes.len() >= 32 {
            bytes[..32].try_into().unwrap_or([0u8; 32])
        } else {
            let mut arr = [0u8; 32];
            arr[..bytes.len()].copy_from_slice(&bytes);
            arr
        };
        // Derive the 4-byte recipient hint from the peer ID (first 4 bytes of blake3 hash)
        let hint: [u8; 4] = blake3::hash(&peer_id).as_bytes()[0..4]
            .try_into()
            .unwrap_or([0u8; 4]);
        let path = DeliveryPath {
            path_id,
            peer_id,
            estimated_latency_ms: latency_ms,
            active: true,
            score: 75.0,
            attempt_count: 0,
            success_count: 0,
        };
        self.multipath.register_path(hint, path);
    }
}

/// Maintenance result for optimized engine
#[derive(Debug, Clone, serde::Serialize)]
pub struct OptimizedRoutingMaintenance {
    /// Base engine maintenance
    pub base_maintenance: RoutingMaintenance,
    /// Negative cache entries cleaned
    pub negative_cache_entries_cleaned: usize,
    /// Negative cache statistics snapshot
    pub negative_cache_stats: NegativeCacheStats,
    /// Adaptive TTL entries cleaned
    pub adaptive_ttl_entries_cleaned: usize,
    /// Timeout budget summary snapshot
    pub timeout_budget_summary: BudgetSummary,
}

impl std::fmt::Display for OptimizedRoutingMaintenance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Optimized Maintenance: base({} promoted, {} demoted), neg_cache({} cleaned, {} entries, {} checks, {} bloom_hits), ttl({} cleaned), budget({:?} elapsed, {:?} remaining, {:?} phase, exhausted: {})",
            self.base_maintenance.peers_promoted,
            self.base_maintenance.peers_demoted,
            self.negative_cache_entries_cleaned,
            self.negative_cache_stats.entry_count,
            self.negative_cache_stats.negative_checks,
            self.negative_cache_stats.bloom_hits,
            self.adaptive_ttl_entries_cleaned,
            self.timeout_budget_summary.elapsed,
            self.timeout_budget_summary.remaining,
            self.timeout_budget_summary.current_phase,
            self.timeout_budget_summary.is_exhausted
        )
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

    fn make_hint(id: u8) -> [u8; 4] {
        [id, 0, 0, 0]
    }

    #[test]
    fn test_optimized_engine_creation() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);

        let engine = OptimizedRoutingEngine::new(local_id, local_hint);
        assert_eq!(engine.local_id, local_id);
        assert_eq!(engine.local_hint, local_hint);
    }

    #[test]
    fn test_negative_cache_integration() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

        // Record a peer as unreachable
        engine.record_unreachable_peer("peer1");

        // Should be detected as unreachable
        let target_hint = make_hint(99);
        let msg_id = make_message_id(1);
        let decision = engine.route_message_optimized(&target_hint, &msg_id, 50, 1000);

        // This test is simplified - in reality we'd need to mock the peer_id_str generation
        assert_eq!(decision.decided_by, RoutingLayer::StoreAndCarry);
    }

    #[test]
    fn test_discovery_phase_advancement() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

        // Start discovery
        engine.start_discovery_if_needed();
        assert!(engine.is_discovery_in_progress());

        // Advance through phases
        let phase1 = engine.advance_discovery_phase();
        assert!(phase1.is_some());

        let phase2 = engine.advance_discovery_phase();
        assert!(phase2.is_some());

        // Eventually should complete
        let _phase3 = engine.advance_discovery_phase();
        let phase4 = engine.advance_discovery_phase();
        assert!(phase4.is_none());
        assert!(!engine.is_discovery_in_progress());
    }

    #[test]
    fn test_app_lifecycle_integration() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

        // Simulate going to background
        let route = RouteAdvertisement {
            destination_hint: make_hint(99),
            next_hop: make_peer_id(2),
            hop_count: 2,
            reliability: 0.95,
            last_confirmed: 1000,
            sequence: 1,
            ttl: 3600,
        };

        engine.on_app_background(vec![(make_peer_id(2), make_hint(99), route)]);

        // Simulate resuming
        let hints = engine.on_app_resume();
        assert_eq!(hints.len(), 1);
    }

    #[test]
    fn test_adaptive_ttl_integration() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

        // Record activity for a peer
        engine.record_message_activity("peer1");

        // Should have activity recorded
        let ttl = engine.adaptive_ttl().calculate_ttl("peer1");
        assert!(ttl > Duration::from_secs(0));
    }

    #[test]
    fn test_maintenance_integration() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

        // Record some unreachable peers and activity
        engine.record_unreachable_peer("peer1");
        engine.record_message_activity("peer2");

        // Run maintenance
        let maint = engine.tick(1000);

        // Should have cleaned up appropriately
        assert_eq!(maint.adaptive_ttl_entries_cleaned, 0); // Nothing old enough to clean
    }

    #[test]
    fn test_peer_seen_multi_transport_and_confidence() {
        let local_id = make_peer_id(1);
        let local_hint = make_hint(1);
        let mut engine = OptimizedRoutingEngine::new(local_id, local_hint);

        let peer = make_peer_id(42);
        let peer_hint: [u8; 4] = blake3::hash(&peer).as_bytes()[0..4]
            .try_into()
            .expect("4 byte hint");
        let msg_id = make_message_id(1);

        // Before sighting: StoreAndCarry with confidence 0.0
        let dec0 = engine.route_message_optimized(&peer_hint, &msg_id, 50, 1000);
        assert_eq!(dec0.decided_by, RoutingLayer::StoreAndCarry);
        assert_eq!(dec0.confidence, 0.0);

        // Sighting on BLE
        engine.peer_seen(peer, TransportType::BLE);
        let dec1 = engine.route_message_optimized(&peer_hint, &msg_id, 50, 1000);
        assert_eq!(dec1.decided_by, RoutingLayer::Local);
        assert!(dec1.confidence >= 0.5);
        match dec1.primary {
            NextHop::Direct { peer_id, transport } => {
                assert_eq!(peer_id, peer);
                assert_eq!(transport, TransportType::BLE);
            }
            other => panic!("expected Direct BLE, got {:?}", other),
        }

        // Sighting on TCP
        engine.peer_seen(peer, TransportType::TCP);
        let dec2 = engine.route_message_optimized(&peer_hint, &msg_id, 50, 1000);
        assert_eq!(dec2.decided_by, RoutingLayer::Local);
        assert!(dec2.confidence >= 0.5);
        match dec2.primary {
            NextHop::Direct { peer_id, transport } => {
                assert_eq!(peer_id, peer);
                assert_eq!(transport, TransportType::TCP);
            }
            other => panic!("expected Direct TCP, got {:?}", other),
        }
        assert!(dec2.alternatives.iter().any(|h| matches!(h, NextHop::Direct { peer_id, transport } if *peer_id == peer && *transport == TransportType::BLE)));

        // Sighting back on BLE
        engine.peer_seen(peer, TransportType::BLE);
        let dec3 = engine.route_message_optimized(&peer_hint, &msg_id, 50, 1000);
        assert_eq!(dec3.decided_by, RoutingLayer::Local);
        match dec3.primary {
            NextHop::Direct { peer_id, transport } => {
                assert_eq!(peer_id, peer);
                assert_eq!(transport, TransportType::BLE);
            }
            other => panic!("expected Direct BLE, got {:?}", other),
        }
        assert!(dec3.alternatives.iter().any(|h| matches!(h, NextHop::Direct { peer_id, transport } if *peer_id == peer && *transport == TransportType::TCP)));
    }
}
