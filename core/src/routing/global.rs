//! Layer 3 — Common Mycorrhizal Network (Global Routes)
//!
//! Sparse, demand-driven global route advertisements via internet-connected nodes.
//! Routes are propagated between cells through gateway peers, enabling messages to
//! traverse long distances via predictable, established paths.
//!
//! Key properties:
//! - **Sparse**: Only routes for hints we actively need or have received advertisements for
//! - **Demand-driven**: Route requests trigger discovery when routes are unknown
//! - **Redundant**: Multiple routes per destination for resilience
//! - **Versioned**: Sequence numbers prevent accepting stale route updates
//! - **Expiring**: Routes expire after TTL to avoid stale routing decisions

use super::local::PeerId;
use std::collections::HashMap;

/// A route advertisement — "I can reach this destination hint"
///
/// Sent via Layer 3 (global internet-connected peers) to propagate routing information
/// between distant cells. Each advertisement is independent and versioned.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RouteAdvertisement {
    /// The destination hint this route reaches
    pub destination_hint: [u8; 4],
    /// The next hop peer to forward through
    pub next_hop: PeerId,
    /// Total hops to destination (via this route)
    pub hop_count: u8,
    /// How reliable this route is (0.0 - 1.0)
    pub reliability: f64,
    /// When this route was last confirmed (unix timestamp)
    pub last_confirmed: u64,
    /// Sequence number for update ordering (higher = newer)
    pub sequence: u64,
    /// TTL in seconds (route expires after this)
    pub ttl: u32,
}

/// A pending route request
///
/// Tracks requests for unknown destinations. When a message arrives for an unknown hint,
/// we create a route request and disseminate it to internet-connected peers.
#[derive(Debug, Clone)]
pub struct RouteRequest {
    /// The destination hint we're looking for
    pub hint: [u8; 4],
    /// When the request was initiated
    pub requested_at: u64,
    /// How many times we've re-requested this route
    pub attempts: u32,
    /// Maximum re-attempts before giving up
    pub max_attempts: u32,
}

/// How long a route request stays pending before it is considered stale and a
/// new RouteDiscovery may be issued (must match cleanup()'s window).
const MAX_ROUTE_REQUEST_AGE_SECS: u64 = 300; // 5 minutes

/// Global routing table — sparse, demand-driven
///
/// Maintains route advertisements received from internet-connected peers. Each destination
/// hint can have multiple routes for redundancy. Routes are periodically cleaned up based on
/// expiry time.
#[derive(Debug, Clone)]
pub struct GlobalRoutes {
    /// hint → Vec<RouteAdvertisement> (multiple routes per destination for redundancy)
    routes: HashMap<[u8; 4], Vec<RouteAdvertisement>>,
    /// Maximum routes per destination (we keep the N best routes)
    max_routes_per_hint: usize,
    /// Maximum total routes (hard limit to prevent memory bloat)
    max_total_routes: usize,
    /// Our own advertisements to share with internet-connected peers
    /// (What this cell can reach via its local network + gateways)
    local_advertisements: Vec<RouteAdvertisement>,
    /// Pending route requests (hints we're looking for)
    pending_requests: HashMap<[u8; 4], RouteRequest>,
}

impl GlobalRoutes {
    /// Create a new, empty global routing table
    pub fn new() -> Self {
        GlobalRoutes {
            routes: HashMap::new(),
            max_routes_per_hint: 3,  // Keep up to 3 routes per destination
            max_total_routes: 10000, // Absolute cap to prevent memory issues
            local_advertisements: Vec::new(),
            pending_requests: HashMap::new(),
        }
    }

    /// Create with custom capacity limits
    pub fn with_limits(max_routes_per_hint: usize, max_total_routes: usize) -> Self {
        GlobalRoutes {
            routes: HashMap::new(),
            max_routes_per_hint,
            max_total_routes,
            local_advertisements: Vec::new(),
            pending_requests: HashMap::new(),
        }
    }

    /// Add a route advertisement (received from a peer)
    ///
    /// Returns true if the route was added, false if rejected (e.g., too many routes,
    /// stale sequence number, or route count would exceed limits).
    pub fn add_route(&mut self, ad: RouteAdvertisement) -> bool {
        // Check global route count
        let current_total = self.routes.values().map(|v| v.len()).sum::<usize>();
        if current_total >= self.max_total_routes {
            return false;
        }

        let routes_for_hint = self.routes.entry(ad.destination_hint).or_default();

        // Check if we already have this exact route (same next_hop, same or newer sequence)
        if let Some(existing) = routes_for_hint.iter().find(|r| r.next_hop == ad.next_hop) {
            // Reject if this route is stale (lower sequence number)
            if ad.sequence <= existing.sequence {
                return false;
            }
            // Replace the old route with this newer one
            if let Some(pos) = routes_for_hint
                .iter()
                .position(|r| r.next_hop == ad.next_hop)
            {
                routes_for_hint[pos] = ad;
            }
            return true;
        }

        // If we're at the limit for this hint, check if this route is better than the worst one
        if routes_for_hint.len() >= self.max_routes_per_hint {
            // Sort to find the worst route (worst = fewest hops, lowest reliability, oldest)
            routes_for_hint.sort_by(|a, b| {
                // Primary: fewest hops
                if a.hop_count != b.hop_count {
                    return b.hop_count.cmp(&a.hop_count); // Descending (worst first)
                }
                // Secondary: lowest reliability
                if (a.reliability - b.reliability).abs() > 0.01 {
                    return a
                        .reliability
                        .partial_cmp(&b.reliability)
                        .unwrap_or(std::cmp::Ordering::Equal);
                }
                // Tertiary: oldest
                a.last_confirmed.cmp(&b.last_confirmed)
            });

            let worst = &routes_for_hint[0];

            // Only add if the new route is better than the worst one
            if ad.hop_count < worst.hop_count
                || (ad.hop_count == worst.hop_count && ad.reliability > worst.reliability)
                || (ad.hop_count == worst.hop_count
                    && (ad.reliability - worst.reliability).abs() <= 0.01
                    && ad.last_confirmed > worst.last_confirmed)
            {
                routes_for_hint.remove(0);
                routes_for_hint.push(ad);
                return true;
            }

            return false;
        }

        // Add the new route
        routes_for_hint.push(ad);
        true
    }

    /// Find all routes to a destination hint
    pub fn routes_for_hint(&self, hint: &[u8; 4]) -> Vec<&RouteAdvertisement> {
        self.routes
            .get(hint)
            .map(|routes| routes.iter().collect())
            .unwrap_or_default()
    }

    /// Best route for a hint
    ///
    /// Selection criteria (in order):
    /// 1. Fewest hops
    /// 2. Highest reliability
    /// 3. Most recently confirmed
    pub fn best_route_for_hint(&self, hint: &[u8; 4]) -> Option<&RouteAdvertisement> {
        self.routes.get(hint).and_then(|routes| {
            if routes.is_empty() {
                return None;
            }

            Some(
                routes
                    .iter()
                    .min_by(|a, b| {
                        // Primary: fewest hops
                        match a.hop_count.cmp(&b.hop_count) {
                            std::cmp::Ordering::Equal => {
                                // Secondary: highest reliability
                                match b.reliability.partial_cmp(&a.reliability) {
                                    Some(std::cmp::Ordering::Equal) | None => {
                                        // Tertiary: most recent
                                        b.last_confirmed.cmp(&a.last_confirmed)
                                    }
                                    Some(other) => other,
                                }
                            }
                            other => other,
                        }
                    })
                    .expect("routes non-empty checked above"),
            )
        })
    }

    /// Request a route for an unknown destination
    ///
    /// Creates a pending route request that should be disseminated to internet-connected peers.
    pub fn request_route(&mut self, hint: [u8; 4], now: u64) -> RouteRequest {
        let request = RouteRequest {
            hint,
            requested_at: now,
            attempts: 0,
            max_attempts: 5,
        };

        self.pending_requests.insert(hint, request.clone());
        request
    }

    /// Check if a route request is pending for a hint
    pub fn is_route_pending(&self, hint: &[u8; 4]) -> bool {
        self.pending_requests.contains_key(hint)
    }

    /// Check if a route request is pending AND still fresh (younger than the
    /// staleness window). A stale pending request must not block a new
    /// RouteDiscovery decision.
    pub fn is_route_pending_fresh(&self, hint: &[u8; 4], now: u64) -> bool {
        match self.pending_requests.get(hint) {
            Some(req) => now.saturating_sub(req.requested_at) <= MAX_ROUTE_REQUEST_AGE_SECS,
            None => false,
        }
    }

    /// Increment attempts for a pending route request
    ///
    /// Returns true if the request is still active, false if max attempts exceeded.
    pub fn increment_route_request_attempts(&mut self, hint: &[u8; 4]) -> bool {
        if let Some(req) = self.pending_requests.get_mut(hint) {
            req.attempts += 1;
            req.attempts < req.max_attempts
        } else {
            false
        }
    }

    /// Remove a route request after it's been resolved or abandoned
    pub fn resolve_route_request(&mut self, hint: &[u8; 4]) {
        self.pending_requests.remove(hint);
    }

    /// Add local advertisements
    ///
    /// Called periodically to update what this cell advertises to the global network.
    /// Typically called after local cell or neighborhood state changes.
    pub fn update_local_advertisements(
        &mut self,
        reachable_hints: Vec<[u8; 4]>,
        local_id: &PeerId,
        now: u64,
    ) {
        self.local_advertisements.clear();

        for hint in reachable_hints {
            // Create an advertisement: this cell reaches this hint via itself (0 hops from here)
            let ad = RouteAdvertisement {
                destination_hint: hint,
                next_hop: *local_id,
                hop_count: 0, // We directly reach this hint
                reliability: 1.0,
                last_confirmed: now,
                sequence: 1,
                ttl: 3600, // Valid for 1 hour
            };
            self.local_advertisements.push(ad);
        }
    }

    /// Get advertisements to share with peers
    pub fn get_advertisements(&self) -> &[RouteAdvertisement] {
        &self.local_advertisements
    }

    /// Remove expired routes
    ///
    /// Called periodically (e.g., every 10-30 seconds). Returns number of routes removed.
    pub fn cleanup(&mut self, now: u64) -> usize {
        let mut removed = 0;

        self.routes.retain(|_, routes| {
            routes.retain(|route| {
                let age = now.saturating_sub(route.last_confirmed);
                let is_expired = age >= (route.ttl as u64);
                if is_expired {
                    removed += 1;
                }
                !is_expired
            });
            !routes.is_empty()
        });

        // Also clean up old route requests (older than 5 minutes)
        self.pending_requests
            .retain(|_, req| now.saturating_sub(req.requested_at) <= MAX_ROUTE_REQUEST_AGE_SECS);

        removed
    }

    /// Remove all routes that go through a specific peer
    ///
    /// Called when a peer disconnects or becomes unreliable.
    /// Returns the number of routes removed.
    pub fn remove_routes_via(&mut self, peer_id: &PeerId) -> usize {
        let mut removed = 0;

        self.routes.retain(|_, routes| {
            routes.retain(|route| {
                if route.next_hop == *peer_id {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
            !routes.is_empty()
        });

        removed
    }

    /// Total number of routes in the table
    pub fn route_count(&self) -> usize {
        self.routes.values().map(|v| v.len()).sum()
    }

    /// Check if we have any route for a destination hint
    pub fn has_route_for(&self, hint: &[u8; 4]) -> bool {
        self.routes.get(hint).is_some_and(|r| !r.is_empty())
    }

    /// Number of unique destination hints we have routes for
    pub fn unique_destination_count(&self) -> usize {
        self.routes.len()
    }

    /// Number of pending route requests
    pub fn pending_request_count(&self) -> usize {
        self.pending_requests.len()
    }

    /// Get all pending route requests
    pub fn pending_requests(&self) -> Vec<&RouteRequest> {
        self.pending_requests.values().collect()
    }
}

impl Default for GlobalRoutes {
    fn default() -> Self {
        Self::new()
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

    fn make_hint(id: u8) -> [u8; 4] {
        [id, 0, 0, 0]
    }

    fn make_route(
        hint: [u8; 4],
        next_hop: PeerId,
        hops: u8,
        reliability: f64,
        last_confirmed: u64,
        sequence: u64,
    ) -> RouteAdvertisement {
        RouteAdvertisement {
            destination_hint: hint,
            next_hop,
            hop_count: hops,
            reliability,
            last_confirmed,
            sequence,
            ttl: 3600,
        }
    }

    #[test]
    fn test_add_route_basic() {
        let mut table = GlobalRoutes::new();
        let hint = make_hint(1);
        let peer = make_peer_id(10);

        let route = make_route(hint, peer, 3, 0.95, 1000, 1);
        assert!(table.add_route(route.clone()));

        assert!(table.has_route_for(&hint));
        assert_eq!(table.route_count(), 1);
        assert_eq!(table.routes_for_hint(&hint).len(), 1);
    }

    #[test]
    fn test_add_multiple_routes_same_destination() {
        let mut table = GlobalRoutes::new();
        let hint = make_hint(1);

        let route1 = make_route(hint, make_peer_id(10), 3, 0.95, 1000, 1);
        let route2 = make_route(hint, make_peer_id(11), 4, 0.85, 1000, 1);
        let route3 = make_route(hint, make_peer_id(12), 2, 0.99, 1000, 1);

        assert!(table.add_route(route1));
        assert!(table.add_route(route2));
        assert!(table.add_route(route3));

        assert_eq!(table.routes_for_hint(&hint).len(), 3);
        assert_eq!(table.route_count(), 3);
    }

    #[test]
    fn test_best_route_selection_by_hops() {
        let mut table = GlobalRoutes::new();
        let hint = make_hint(1);

        let route1 = make_route(hint, make_peer_id(10), 5, 0.95, 1000, 1);
        let route2 = make_route(hint, make_peer_id(11), 2, 0.85, 1000, 1);
        let route3 = make_route(hint, make_peer_id(12), 3, 0.99, 1000, 1);

        table.add_route(route1);
        table.add_route(route2);
        table.add_route(route3);

        let best = table.best_route_for_hint(&hint).unwrap();
        assert_eq!(best.hop_count, 2);
        assert_eq!(best.next_hop, make_peer_id(11));
    }

    #[test]
    fn test_best_route_selection_by_reliability() {
        let mut table = GlobalRoutes::new();
        let hint = make_hint(1);

        let route1 = make_route(hint, make_peer_id(10), 3, 0.85, 1000, 1);
        let route2 = make_route(hint, make_peer_id(11), 3, 0.99, 1000, 1);
        let route3 = make_route(hint, make_peer_id(12), 3, 0.75, 1000, 1);

        table.add_route(route1);
        table.add_route(route2);
        table.add_route(route3);

        let best = table.best_route_for_hint(&hint).unwrap();
        assert_eq!(best.hop_count, 3);
        assert_eq!(best.reliability, 0.99);
        assert_eq!(best.next_hop, make_peer_id(11));
    }

    #[test]
    fn test_best_route_selection_by_recency() {
        let mut table = GlobalRoutes::new();
        let hint = make_hint(1);

        let route1 = make_route(hint, make_peer_id(10), 3, 0.95, 1000, 1);
        let route2 = make_route(hint, make_peer_id(11), 3, 0.95, 2000, 1);
        let route3 = make_route(hint, make_peer_id(12), 3, 0.95, 500, 1);

        table.add_route(route1);
        table.add_route(route2);
        table.add_route(route3);

        let best = table.best_route_for_hint(&hint).unwrap();
        assert_eq!(best.last_confirmed, 2000);
        assert_eq!(best.next_hop, make_peer_id(11));
    }

    #[test]
    fn test_reject_stale_route_update() {
        let mut table = GlobalRoutes::new();
        let hint = make_hint(1);
        let peer = make_peer_id(10);

        let route1 = make_route(hint, peer, 3, 0.95, 1000, 5);
        assert!(table.add_route(route1));

        // Try to add a route with the same next_hop but older sequence
        let route2 = make_route(hint, peer, 2, 0.99, 2000, 3);
        assert!(!table.add_route(route2));

        // Should still have the original route
        let best = table.best_route_for_hint(&hint).unwrap();
        assert_eq!(best.hop_count, 3);
        assert_eq!(best.sequence, 5);
    }

    #[test]
    fn test_accept_newer_route_update() {
        let mut table = GlobalRoutes::new();
        let hint = make_hint(1);
        let peer = make_peer_id(10);

        let route1 = make_route(hint, peer, 3, 0.95, 1000, 5);
        assert!(table.add_route(route1));

        // Update with newer sequence but worse hops
        let route2 = make_route(hint, peer, 5, 0.90, 2000, 7);
        assert!(table.add_route(route2));

        // Should have the newer route
        let best = table.best_route_for_hint(&hint).unwrap();
        assert_eq!(best.hop_count, 5);
        assert_eq!(best.sequence, 7);
    }

    #[test]
    fn test_max_routes_per_hint() {
        let mut table = GlobalRoutes::with_limits(2, 10000);
        let hint = make_hint(1);

        let route1 = make_route(hint, make_peer_id(1), 3, 0.90, 1000, 1);
        let route2 = make_route(hint, make_peer_id(2), 2, 0.95, 1000, 1);
        // route3 is worse than both (more hops, lower reliability)
        let route3 = make_route(hint, make_peer_id(3), 6, 0.70, 1000, 1);

        assert!(table.add_route(route1)); // Add first
        assert!(table.add_route(route2)); // Add second
        assert!(!table.add_route(route3)); // Reject third (worse than the worst of first two)

        assert_eq!(table.routes_for_hint(&hint).len(), 2);
    }

    #[test]
    fn test_max_routes_per_hint_replacement() {
        let mut table = GlobalRoutes::with_limits(2, 10000);
        let hint = make_hint(1);

        let route1 = make_route(hint, make_peer_id(1), 5, 0.80, 1000, 1);
        let route2 = make_route(hint, make_peer_id(2), 4, 0.85, 1000, 1);
        let route3 = make_route(hint, make_peer_id(3), 2, 0.95, 1000, 1); // Better than both

        assert!(table.add_route(route1));
        assert!(table.add_route(route2));
        assert!(table.add_route(route3)); // Should replace the worst (route1)

        assert_eq!(table.routes_for_hint(&hint).len(), 2);
        let routes = table.routes_for_hint(&hint);
        assert!(routes.iter().any(|r| r.next_hop == make_peer_id(3)));
        assert!(!routes.iter().any(|r| r.next_hop == make_peer_id(1)));
    }

    #[test]
    fn test_max_total_routes() {
        let mut table = GlobalRoutes::with_limits(10, 5);
        let hint1 = make_hint(1);
        let hint2 = make_hint(2);
        let hint3 = make_hint(3);

        // Add 3 routes to hint1
        assert!(table.add_route(make_route(hint1, make_peer_id(1), 1, 0.9, 1000, 1)));
        assert!(table.add_route(make_route(hint1, make_peer_id(2), 1, 0.9, 1000, 1)));
        assert!(table.add_route(make_route(hint1, make_peer_id(3), 1, 0.9, 1000, 1)));

        // Add 2 routes to hint2
        assert!(table.add_route(make_route(hint2, make_peer_id(4), 1, 0.9, 1000, 1)));
        assert!(table.add_route(make_route(hint2, make_peer_id(5), 1, 0.9, 1000, 1)));

        // We're at limit, next route should be rejected
        assert!(!table.add_route(make_route(hint3, make_peer_id(6), 1, 0.9, 1000, 1)));

        assert_eq!(table.route_count(), 5);
    }

    #[test]
    fn test_cleanup_expired_routes() {
        let mut table = GlobalRoutes::new();
        let hint = make_hint(1);

        let route1 = make_route(hint, make_peer_id(1), 1, 0.9, 1000, 1); // expires at 1000 + 3600
        let route2 = make_route(hint, make_peer_id(2), 1, 0.9, 2000, 1); // expires at 2000 + 3600

        table.add_route(route1);
        table.add_route(route2);

        assert_eq!(table.route_count(), 2);

        // Clean at time 4600 (route1 should be expired)
        let removed = table.cleanup(4600);
        assert_eq!(removed, 1);
        assert_eq!(table.route_count(), 1);
        assert_eq!(table.routes_for_hint(&hint)[0].next_hop, make_peer_id(2));

        // Clean at time 5600 (route2 should be expired)
        let removed = table.cleanup(5600);
        assert_eq!(removed, 1);
        assert_eq!(table.route_count(), 0);
    }

    #[test]
    fn test_remove_routes_via_peer() {
        let mut table = GlobalRoutes::new();
        let hint1 = make_hint(1);
        let hint2 = make_hint(2);
        let peer_x = make_peer_id(10);
        let peer_y = make_peer_id(11);

        table.add_route(make_route(hint1, peer_x, 2, 0.9, 1000, 1));
        table.add_route(make_route(hint1, peer_y, 3, 0.8, 1000, 1));
        table.add_route(make_route(hint2, peer_x, 2, 0.9, 1000, 1));
        table.add_route(make_route(hint2, peer_y, 4, 0.7, 1000, 1));

        assert_eq!(table.route_count(), 4);

        let removed = table.remove_routes_via(&peer_x);
        assert_eq!(removed, 2);
        assert_eq!(table.route_count(), 2);

        // Should only have routes via peer_y
        assert_eq!(table.routes_for_hint(&hint1)[0].next_hop, peer_y);
        assert_eq!(table.routes_for_hint(&hint2)[0].next_hop, peer_y);
    }

    #[test]
    fn test_route_request_creation() {
        let mut table = GlobalRoutes::new();
        let hint = make_hint(1);

        assert!(!table.is_route_pending(&hint));

        let req = table.request_route(hint, 1000);
        assert_eq!(req.hint, hint);
        assert_eq!(req.requested_at, 1000);
        assert_eq!(req.attempts, 0);
        assert!(table.is_route_pending(&hint));
    }

    #[test]
    fn test_route_request_pending_freshness() {
        let mut table = GlobalRoutes::new();
        let hint = make_hint(1);

        table.request_route(hint, 1000);

        // Fresh pending request blocks re-discovery
        assert!(table.is_route_pending_fresh(&hint, 1000));
        assert!(table.is_route_pending_fresh(&hint, 1000 + MAX_ROUTE_REQUEST_AGE_SECS - 1));

        // Stale pending request (older than the window) must not block
        assert!(!table.is_route_pending_fresh(&hint, 1000 + MAX_ROUTE_REQUEST_AGE_SECS + 1));
        // Raw entry is still pending until cleanup() runs
        assert!(table.is_route_pending(&hint));
    }

    #[test]
    fn test_route_request_attempts() {
        let mut table = GlobalRoutes::new();
        let hint = make_hint(1);

        table.request_route(hint, 1000);

        let mut still_active = true;
        for _ in 0..5 {
            still_active = table.increment_route_request_attempts(&hint);
            if !still_active {
                break;
            }
        }

        assert!(!still_active); // After 5 attempts, should return false
    }

    #[test]
    fn test_route_request_resolution() {
        let mut table = GlobalRoutes::new();
        let hint = make_hint(1);

        table.request_route(hint, 1000);
        assert!(table.is_route_pending(&hint));

        table.resolve_route_request(&hint);
        assert!(!table.is_route_pending(&hint));
    }

    #[test]
    fn test_update_local_advertisements() {
        let mut table = GlobalRoutes::new();
        let local_id = make_peer_id(1);
        let hints = vec![make_hint(1), make_hint(2), make_hint(3)];

        table.update_local_advertisements(hints.clone(), &local_id, 1000);

        let ads = table.get_advertisements();
        assert_eq!(ads.len(), 3);

        for ad in ads {
            assert_eq!(ad.next_hop, local_id);
            assert_eq!(ad.hop_count, 0);
            assert_eq!(ad.reliability, 1.0);
            assert_eq!(ad.last_confirmed, 1000);
        }
    }

    #[test]
    fn test_local_advertisements_cleared_on_update() {
        let mut table = GlobalRoutes::new();
        let local_id = make_peer_id(1);

        table.update_local_advertisements(vec![make_hint(1), make_hint(2)], &local_id, 1000);
        assert_eq!(table.get_advertisements().len(), 2);

        table.update_local_advertisements(vec![make_hint(3)], &local_id, 2000);
        assert_eq!(table.get_advertisements().len(), 1);
        assert_eq!(table.get_advertisements()[0].destination_hint, make_hint(3));
    }

    #[test]
    fn test_unique_destination_count() {
        let mut table = GlobalRoutes::new();

        let hint1 = make_hint(1);
        let hint2 = make_hint(2);

        table.add_route(make_route(hint1, make_peer_id(1), 1, 0.9, 1000, 1));
        table.add_route(make_route(hint1, make_peer_id(2), 2, 0.8, 1000, 1));
        table.add_route(make_route(hint2, make_peer_id(3), 1, 0.9, 1000, 1));

        assert_eq!(table.unique_destination_count(), 2);
        assert_eq!(table.route_count(), 3);
    }

    #[test]
    fn test_routes_for_unknown_hint() {
        let table = GlobalRoutes::new();
        let hint = make_hint(99);

        assert_eq!(table.routes_for_hint(&hint).len(), 0);
        assert!(!table.has_route_for(&hint));
        assert!(table.best_route_for_hint(&hint).is_none());
    }

    #[test]
    fn test_pending_requests_cleanup() {
        let mut table = GlobalRoutes::new();
        let hint1 = make_hint(1);
        let hint2 = make_hint(2);

        table.request_route(hint1, 1000);
        table.request_route(hint2, 1100);

        assert_eq!(table.pending_request_count(), 2);

        // Cleanup at time 1400 (max_request_age = 300)
        table.cleanup(1400);
        assert_eq!(table.pending_request_count(), 1); // hint2 is still pending (age 300)

        table.cleanup(1401);
        assert_eq!(table.pending_request_count(), 0); // hint2 is now expired
    }

    #[test]
    fn test_pending_requests_getter() {
        let mut table = GlobalRoutes::new();
        let hint1 = make_hint(1);
        let hint2 = make_hint(2);

        table.request_route(hint1, 1000);
        table.request_route(hint2, 1100);

        let pending = table.pending_requests();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_default_construction() {
        let table = GlobalRoutes::default();
        assert_eq!(table.route_count(), 0);
        assert_eq!(table.unique_destination_count(), 0);
        assert_eq!(table.pending_request_count(), 0);
    }

    #[test]
    fn test_complex_routing_scenario() {
        let mut table = GlobalRoutes::with_limits(3, 10000);
        let hint_a = make_hint(1);
        let hint_b = make_hint(2);

        // Build multiple paths to hint_a
        table.add_route(make_route(hint_a, make_peer_id(10), 5, 0.8, 1000, 1));
        table.add_route(make_route(hint_a, make_peer_id(11), 3, 0.95, 1100, 1));
        table.add_route(make_route(hint_a, make_peer_id(12), 4, 0.85, 1200, 1));

        // Build paths to hint_b
        table.add_route(make_route(hint_b, make_peer_id(20), 2, 0.99, 1000, 1));
        table.add_route(make_route(hint_b, make_peer_id(21), 6, 0.7, 1000, 1));

        // Best to hint_a should be via peer_id(11) (3 hops, 0.95 reliability)
        let best_a = table.best_route_for_hint(&hint_a).unwrap();
        assert_eq!(best_a.next_hop, make_peer_id(11));

        // Best to hint_b should be via peer_id(20) (2 hops, 0.99 reliability)
        let best_b = table.best_route_for_hint(&hint_b).unwrap();
        assert_eq!(best_b.next_hop, make_peer_id(20));

        // Simulate peer 11 disconnecting
        table.remove_routes_via(&make_peer_id(11));
        let new_best_a = table.best_route_for_hint(&hint_a).unwrap();
        assert_eq!(new_best_a.next_hop, make_peer_id(12)); // Next best is via 12

        // hint_b should be unaffected
        let still_best_b = table.best_route_for_hint(&hint_b).unwrap();
        assert_eq!(still_best_b.next_hop, make_peer_id(20));
    }
}
