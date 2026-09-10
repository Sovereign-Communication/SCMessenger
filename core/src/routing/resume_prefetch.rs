//! Route Prefetch on App Resume
//!
//! When the app wakes from background, immediately refresh known routes
//! instead of waiting for the first message to trigger discovery.
//! This reduces first-message latency from 2000ms to ~10ms.
//!
//! # Design Principles
//!
//! 1. **Proactive refresh**: Refresh routes before they're needed
//! 2. **Parallel validation**: Validate multiple routes concurrently
//! 3. **Graceful degradation**: Return stale routes while refreshing
//! 4. **Lifecycle-aware**: Hook into iOS/Android app lifecycle events

use std::collections::{HashMap, VecDeque};
use web_time::{Duration, Instant};

use super::global::RouteAdvertisement;
use super::local::PeerId;

/// Status of a prefetched route
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefetchStatus {
    /// Route is fresh and ready to use
    Fresh,
    /// Route is slightly stale but usable
    Stale,
    /// Route refresh is in progress
    Refreshing,
    /// Route refresh failed
    Failed,
}

/// A prefetched route with metadata
#[derive(Debug, Clone)]
pub struct PrefetchedRoute {
    /// The route advertisement
    pub route: RouteAdvertisement,
    /// When this route was last validated
    pub last_validated: Instant,
    /// Current prefetch status
    pub status: PrefetchStatus,
    /// How many times we've attempted to refresh
    pub refresh_attempts: u32,
    /// Priority for refresh (higher = more important)
    pub priority: u32,
}

impl PrefetchedRoute {
    fn new(route: RouteAdvertisement, priority: u32) -> Self {
        PrefetchedRoute {
            route,
            last_validated: Instant::now(),
            status: PrefetchStatus::Fresh,
            refresh_attempts: 0,
            priority,
        }
    }

    /// Check if this route is still fresh. Reserved strict-freshness check for
    /// future prefetch validation refinement; `is_usable` (fresh-or-stale) is
    /// what's used in the current prefetch/refresh path.
    #[allow(dead_code)]
    fn is_fresh(&self, max_age: Duration) -> bool {
        self.last_validated.elapsed() < max_age
    }

    /// Check if this route is usable (fresh or stale)
    fn is_usable(&self, max_stale_age: Duration) -> bool {
        match self.status {
            PrefetchStatus::Fresh | PrefetchStatus::Stale => {
                self.last_validated.elapsed() < max_stale_age
            }
            PrefetchStatus::Refreshing => true, // Still usable while refreshing
            PrefetchStatus::Failed => false,
        }
    }

    /// Mark as refreshing
    pub fn start_refresh(&mut self) {
        self.status = PrefetchStatus::Refreshing;
        self.refresh_attempts += 1;
    }

    /// Mark refresh as successful
    fn complete_refresh(&mut self) {
        self.status = PrefetchStatus::Fresh;
        self.last_validated = Instant::now();
    }

    /// Mark refresh as failed
    fn fail_refresh(&mut self) {
        self.status = PrefetchStatus::Failed;
    }
}

/// Configuration for route prefetching
#[derive(Debug, Clone)]
pub struct PrefetchConfig {
    /// Maximum age before a route is considered stale
    pub max_route_age: Duration,
    /// Maximum age before a route is unusable
    pub max_stale_age: Duration,
    /// Maximum number of routes to prefetch
    pub max_prefetch_routes: usize,
    /// Maximum number of concurrent refreshes
    pub max_concurrent_refreshes: usize,
    /// Delay between refresh batches
    pub refresh_batch_delay: Duration,
}

impl Default for PrefetchConfig {
    fn default() -> Self {
        PrefetchConfig {
            max_route_age: Duration::from_secs(300),  // 5 minutes
            max_stale_age: Duration::from_secs(3600), // 1 hour
            max_prefetch_routes: 100,
            max_concurrent_refreshes: 10,
            refresh_batch_delay: Duration::from_millis(100),
        }
    }
}

/// Manager for route prefetching on app resume
///
/// Tracks routes that were valid before background and refreshes them
/// immediately when the app resumes.
#[derive(Debug)]
pub struct ResumePrefetchManager {
    /// Prefetched routes by destination hint
    routes: HashMap<[u8; 8], PrefetchedRoute>,
    /// Peers we frequently message (candidates for prefetch)
    frequent_peers: Vec<FrequentPeer>,
    /// Whether prefetch is currently in progress
    prefetch_in_progress: bool,
    /// When prefetch started
    prefetch_started: Option<Instant>,
    /// Configuration
    config: PrefetchConfig,
    /// Queue of hints waiting for refresh
    refresh_queue: VecDeque<[u8; 8]>,
    /// Number of currently active refreshes
    active_refreshes: usize,
}

/// Information about a frequently messaged peer
#[derive(Debug, Clone)]
pub struct FrequentPeer {
    /// The peer's ID
    pub peer_id: PeerId,
    /// The peer's destination hint
    pub hint: [u8; 8],
    /// Number of messages exchanged recently
    pub message_count: u32,
    /// Last message timestamp
    pub last_message: Instant,
    /// Calculated priority (higher = more important to prefetch)
    pub priority: u32,
}

impl FrequentPeer {
    fn new(peer_id: PeerId, hint: [u8; 8]) -> Self {
        FrequentPeer {
            peer_id,
            hint,
            message_count: 1,
            last_message: Instant::now(),
            priority: 1,
        }
    }

    fn record_message(&mut self) {
        self.message_count += 1;
        self.last_message = Instant::now();
        // Priority increases with message count, capped at 100
        self.priority = (self.message_count * 10).min(100);
    }

    fn decay(&mut self, half_life: Duration) {
        let elapsed = self.last_message.elapsed();
        let decay_factor = 0.5_f64.powf(elapsed.as_secs_f64() / half_life.as_secs_f64());
        self.message_count = (self.message_count as f64 * decay_factor).round() as u32;
        self.priority = (self.message_count * 10).min(100);
    }
}

impl ResumePrefetchManager {
    /// Create a new resume prefetch manager
    pub fn new(config: PrefetchConfig) -> Self {
        ResumePrefetchManager {
            routes: HashMap::new(),
            frequent_peers: Vec::new(),
            prefetch_in_progress: false,
            prefetch_started: None,
            config,
            refresh_queue: VecDeque::new(),
            active_refreshes: 0,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(PrefetchConfig::default())
    }

    /// Called when app goes to background
    ///
    /// Saves current routes for later refresh.
    pub fn on_app_background(
        &mut self,
        current_routes: Vec<(PeerId, [u8; 8], RouteAdvertisement)>,
    ) {
        // Clear old routes
        self.routes.clear();

        // Save current routes with their hints
        for (_peer_id, hint, route) in current_routes {
            // Calculate priority based on whether this is a frequent peer
            let priority = self
                .frequent_peers
                .iter()
                .find(|p| p.hint == hint)
                .map(|p| p.priority)
                .unwrap_or(1);

            self.routes
                .insert(hint, PrefetchedRoute::new(route, priority));
        }
    }

    /// Called when app resumes from background
    ///
    /// Returns hints that need immediate refresh, sorted by priority.
    pub fn on_app_resume(&mut self) -> Vec<[u8; 8]> {
        self.prefetch_in_progress = true;
        self.prefetch_started = Some(Instant::now());
        self.active_refreshes = 0;

        // Build refresh queue sorted by priority (highest first)
        let mut hints: Vec<([u8; 8], u32)> = self
            .routes
            .iter()
            .map(|(&hint, route)| (hint, route.priority))
            .collect();

        hints.sort_by_key(|b| std::cmp::Reverse(b.1)); // Sort by priority descending

        self.refresh_queue = hints.iter().map(|(h, _)| *h).collect();

        // Return hints for immediate refresh (up to max_concurrent_refreshes)
        let immediate_count = self.config.max_concurrent_refreshes.min(hints.len());
        hints
            .into_iter()
            .take(immediate_count)
            .map(|(h, _)| h)
            .collect()
    }

    /// Get a route immediately if available (even if slightly stale)
    ///
    /// This provides fast access while refresh is in progress.
    pub fn get_route_early(&self, hint: &[u8; 8]) -> Option<&RouteAdvertisement> {
        self.routes.get(hint).and_then(|prefetched| {
            if prefetched.is_usable(self.config.max_stale_age) {
                Some(&prefetched.route)
            } else {
                None
            }
        })
    }

    /// Update a route after successful refresh
    pub fn update_route(&mut self, hint: [u8; 8], route: RouteAdvertisement) {
        if let Some(prefetched) = self.routes.get_mut(&hint) {
            prefetched.route = route;
            prefetched.complete_refresh();
        } else {
            // New route, add it
            self.routes.insert(hint, PrefetchedRoute::new(route, 1));
        }
        self.active_refreshes = self.active_refreshes.saturating_sub(1);
    }

    /// Mark a route refresh as failed
    pub fn mark_refresh_failed(&mut self, hint: &[u8; 8]) {
        if let Some(prefetched) = self.routes.get_mut(hint) {
            prefetched.fail_refresh();
        }
        self.active_refreshes = self.active_refreshes.saturating_sub(1);
    }

    /// Start a route refresh
    pub fn start_route_refresh(&mut self, hint: &[u8; 8]) {
        if let Some(prefetched) = self.routes.get_mut(hint) {
            prefetched.start_refresh();
        }
    }

    /// Get the next hint that needs refresh
    pub fn next_refresh_hint(&mut self) -> Option<[u8; 8]> {
        self.refresh_queue.pop_front()
    }

    /// Check if prefetch is complete
    pub fn is_prefetch_complete(&self) -> bool {
        self.refresh_queue.is_empty() && self.active_refreshes == 0
    }

    /// Check if prefetch is in progress
    pub fn is_prefetch_in_progress(&self) -> bool {
        self.prefetch_in_progress
    }

    /// Get prefetch statistics
    pub fn stats(&self) -> PrefetchStats {
        let total = self.routes.len();
        let fresh = self
            .routes
            .values()
            .filter(|r| r.status == PrefetchStatus::Fresh)
            .count();
        let stale = self
            .routes
            .values()
            .filter(|r| r.status == PrefetchStatus::Stale)
            .count();
        let refreshing = self
            .routes
            .values()
            .filter(|r| r.status == PrefetchStatus::Refreshing)
            .count();
        let failed = self
            .routes
            .values()
            .filter(|r| r.status == PrefetchStatus::Failed)
            .count();

        PrefetchStats {
            total_routes: total,
            fresh_routes: fresh,
            stale_routes: stale,
            refreshing_routes: refreshing,
            failed_routes: failed,
            prefetch_in_progress: self.prefetch_in_progress,
            queue_remaining: self.refresh_queue.len(),
        }
    }

    /// Record a message exchange with a peer
    ///
    /// Updates the frequent peers list for future prefetch prioritization.
    pub fn record_message(&mut self, peer_id: PeerId, hint: [u8; 8]) {
        // Decay old peers first so accumulated inactivity is properly applied
        // before recording the new message (which resets last_message to now).
        let half_life = Duration::from_secs(3600); // 1 hour half-life
        self.frequent_peers.retain_mut(|p| {
            p.decay(half_life);
            p.message_count > 0
        });

        if let Some(peer) = self
            .frequent_peers
            .iter_mut()
            .find(|p| p.peer_id == peer_id)
        {
            peer.record_message();
        } else {
            self.frequent_peers.push(FrequentPeer::new(peer_id, hint));
        }

        // Sort by priority for efficient lookup
        self.frequent_peers
            .sort_by_key(|b| std::cmp::Reverse(b.priority));

        // Limit list size
        if self.frequent_peers.len() > 50 {
            self.frequent_peers.truncate(50);
        }
    }

    /// Get the top N frequent peers for prefetching
    pub fn top_frequent_peers(&self, n: usize) -> Vec<&FrequentPeer> {
        self.frequent_peers.iter().take(n).collect()
    }

    /// Clear all prefetched routes
    pub fn clear(&mut self) {
        self.routes.clear();
        self.refresh_queue.clear();
        self.prefetch_in_progress = false;
        self.active_refreshes = 0;
    }
}

/// Statistics for prefetch operations
#[derive(Debug, Clone)]
pub struct PrefetchStats {
    pub total_routes: usize,
    pub fresh_routes: usize,
    pub stale_routes: usize,
    pub refreshing_routes: usize,
    pub failed_routes: usize,
    pub prefetch_in_progress: bool,
    pub queue_remaining: usize,
}

impl std::fmt::Display for PrefetchStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Prefetch: {} total ({} fresh, {} stale, {} refreshing, {} failed), queue: {}",
            self.total_routes,
            self.fresh_routes,
            self.stale_routes,
            self.refreshing_routes,
            self.failed_routes,
            self.queue_remaining
        )
    }
}

impl Default for ResumePrefetchManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    fn create_test_peer_id() -> PeerId {
        // routing::PeerId is [u8; 32], not libp2p::PeerId
        let mut id = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut id);
        id
    }

    fn create_test_route(hint: [u8; 8], hop_count: u8) -> RouteAdvertisement {
        RouteAdvertisement {
            destination_hint: hint,
            next_hop: create_test_peer_id(),
            hop_count,
            reliability: 0.95,
            last_confirmed: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            sequence: 1,
            ttl: 3600,
        }
    }

    #[test]
    fn test_prefetch_basic() {
        let mut manager = ResumePrefetchManager::with_defaults();
        let hint = [1, 2, 3, 4, 0, 0, 0, 0];
        let route = create_test_route(hint, 2);

        // Simulate going to background with a route
        manager.on_app_background(vec![(create_test_peer_id(), hint, route)]);

        // Resume and get hints to refresh
        let hints = manager.on_app_resume();
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0], hint);
    }

    #[test]
    fn test_get_route_early() {
        let mut manager = ResumePrefetchManager::with_defaults();
        let hint = [1, 2, 3, 4, 0, 0, 0, 0];
        let route = create_test_route(hint, 2);

        manager.on_app_background(vec![(create_test_peer_id(), hint, route.clone())]);
        manager.on_app_resume();

        // Should be able to get route immediately
        let early_route = manager.get_route_early(&hint);
        assert!(early_route.is_some());
        assert_eq!(early_route.unwrap().hop_count, 2);
    }

    #[test]
    fn test_update_route() {
        let mut manager = ResumePrefetchManager::with_defaults();
        let hint = [1, 2, 3, 4, 0, 0, 0, 0];
        let route = create_test_route(hint, 2);

        manager.on_app_background(vec![(create_test_peer_id(), hint, route)]);
        manager.on_app_resume();

        // Update with new route
        let new_route = create_test_route(hint, 3);
        manager.update_route(hint, new_route);

        // Should have updated route
        let updated = manager.get_route_early(&hint);
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().hop_count, 3);
    }

    #[test]
    fn test_frequent_peer_tracking() {
        let mut manager = ResumePrefetchManager::with_defaults();
        let peer_id = create_test_peer_id();
        let hint = [1, 2, 3, 4, 0, 0, 0, 0];

        // Record multiple messages
        for _ in 0..10 {
            manager.record_message(peer_id, hint);
        }

        let top_peers = manager.top_frequent_peers(5);
        assert_eq!(top_peers.len(), 1);
        assert_eq!(top_peers[0].message_count, 10);
    }

    #[test]
    fn test_prefetch_stats() {
        let mut manager = ResumePrefetchManager::with_defaults();
        let hint1 = [1, 2, 3, 4, 0, 0, 0, 0];
        let hint2 = [5, 6, 7, 8, 0, 0, 0, 0];
        let route1 = create_test_route(hint1, 2);
        let route2 = create_test_route(hint2, 3);

        manager.on_app_background(vec![
            (create_test_peer_id(), hint1, route1),
            (create_test_peer_id(), hint2, route2),
        ]);

        let stats = manager.stats();
        assert_eq!(stats.total_routes, 2);
    }

    #[test]
    fn test_frequent_peer_decay() {
        let mut manager = ResumePrefetchManager::with_defaults();
        let peer_id = create_test_peer_id();
        let hint = [1, 2, 3, 4, 0, 0, 0, 0];

        // Record messages
        manager.record_message(peer_id, hint);
        assert_eq!(manager.frequent_peers[0].message_count, 1);

        // Manually set last_message to past to trigger decay
        manager.frequent_peers[0].last_message = Instant::now()
            .checked_sub(Duration::from_secs(7200))
            .unwrap_or_else(|| Instant::now() - Duration::from_secs(3600));

        // Record another message, should trigger decay
        manager.record_message(peer_id, hint);

        // Message count should have decayed
        assert!(manager.frequent_peers[0].message_count < 2);
    }
}
