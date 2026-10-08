//! Speculative Delegate Pre-warming
//!
//! Register with delegates BEFORE going to sleep, and pre-warm delegate connections.
//! This reduces wake-up latency from 1000ms to 0ms (already connected and registered).
//!
//! # Design Principles
//!
//! 1. **Proactive registration**: Register with delegates before sleep
//! 2. **Connection pre-warming**: Establish connections before they're needed
//! 3. **Redundancy**: Register with multiple delegates for reliability
//! 4. **Lifecycle-aware**: Hook into iOS/Android app lifecycle events

use std::collections::{HashMap, VecDeque};
use web_time::{Duration, Instant};

use libp2p::PeerId;

/// Information about a delegate node
#[derive(Debug, Clone)]
pub struct DelegateInfo {
    /// The delegate's peer ID
    pub peer_id: PeerId,
    /// The delegate's multiaddress
    pub multiaddr: String,
    /// Reliability score (0.0 - 1.0)
    pub reliability_score: f64,
    /// Last time we confirmed this delegate is available
    pub last_confirmed: Instant,
    /// Maximum number of peers this delegate can handle
    pub max_peers: usize,
    /// Current number of peers connected to this delegate
    pub current_peers: usize,
}

impl DelegateInfo {
    /// Create a new delegate info
    pub fn new(peer_id: PeerId, multiaddr: String) -> Self {
        DelegateInfo {
            peer_id,
            multiaddr,
            reliability_score: 0.8, // Default reliability
            last_confirmed: Instant::now(),
            max_peers: 100,
            current_peers: 0,
        }
    }

    /// Update delegate information
    pub fn update(&mut self, reliability: f64, current_peers: usize) {
        self.reliability_score = reliability;
        self.current_peers = current_peers;
        self.last_confirmed = Instant::now();
    }

    /// Check if delegate has capacity
    pub fn has_capacity(&self) -> bool {
        self.current_peers < self.max_peers
    }

    /// Calculate delegate score for selection.
    ///
    /// Purely behavioural: every node is an equal relay, so there is no
    /// node-class bonus. The score is the locally measured reliability.
    pub fn score(&self) -> f64 {
        self.reliability_score
    }
}

/// Pre-warmed connection to a delegate
#[derive(Debug, Clone)]
pub struct WarmConnection {
    /// The delegate ID
    pub delegate_id: PeerId,
    /// When the connection was established
    pub connected_at: Instant,
    /// Last keepalive timestamp
    pub last_keepalive: Instant,
    /// Connection status
    pub is_active: bool,
    /// Registration status
    pub is_registered: bool,
}

impl WarmConnection {
    /// Create a new warm connection
    pub fn new(delegate_id: PeerId) -> Self {
        let now = Instant::now();
        WarmConnection {
            delegate_id,
            connected_at: now,
            last_keepalive: now,
            is_active: true,
            is_registered: false,
        }
    }

    /// Update keepalive timestamp
    pub fn update_keepalive(&mut self) {
        self.last_keepalive = Instant::now();
    }

    /// Mark as registered
    pub fn mark_registered(&mut self) {
        self.is_registered = true;
    }

    /// Check if connection is stale
    pub fn is_stale(&self, stale_timeout: Duration) -> bool {
        self.last_keepalive.elapsed() > stale_timeout
    }
}

/// Manager for delegate pre-warming
#[derive(Debug)]
pub struct DelegatePrewarmManager {
    /// Current delegate assignments
    delegates: Vec<DelegateInfo>,
    /// Pre-warmed connections to delegates
    warm_connections: HashMap<PeerId, WarmConnection>,
    /// When to next refresh delegate connections
    next_refresh: Instant,
    /// Configuration
    config: DelegatePrewarmConfig,
    /// Registration queue for new delegates
    registration_queue: VecDeque<PeerId>,
}

/// Configuration for delegate pre-warming
#[derive(Debug, Clone)]
pub struct DelegatePrewarmConfig {
    /// Maximum number of delegates to maintain
    pub max_delegates: usize,
    /// Minimum number of delegates to maintain
    pub min_delegates: usize,
    /// How often to refresh delegate connections
    pub refresh_interval: Duration,
    /// How often to send keepalives
    pub keepalive_interval: Duration,
    /// When to consider a connection stale
    pub stale_timeout: Duration,
    /// Maximum registration attempts
    pub max_registration_attempts: u32,
}

impl Default for DelegatePrewarmConfig {
    fn default() -> Self {
        DelegatePrewarmConfig {
            max_delegates: 5,
            min_delegates: 2,
            refresh_interval: Duration::from_secs(300), // 5 minutes
            keepalive_interval: Duration::from_secs(60), // 1 minute
            stale_timeout: Duration::from_secs(120),    // 2 minutes
            max_registration_attempts: 3,
        }
    }
}

impl DelegatePrewarmManager {
    /// Create a new delegate pre-warm manager
    pub fn new(config: DelegatePrewarmConfig) -> Self {
        DelegatePrewarmManager {
            delegates: Vec::new(),
            warm_connections: HashMap::new(),
            next_refresh: Instant::now(),
            config,
            registration_queue: VecDeque::new(),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(DelegatePrewarmConfig::default())
    }

    /// Add a known delegate
    pub fn add_delegate(&mut self, delegate: DelegateInfo) {
        self.delegates.push(delegate);
    }

    /// Called before app goes to background (iOS/Android lifecycle)
    ///
    /// Pre-warms connections to delegates before the app goes to sleep.
    pub async fn prewarm_for_background(&mut self) {
        // 1. Select top-N reliable delegates
        let best_delegates = self.select_best_delegates(self.config.max_delegates);

        // 2. Establish connections to them NOW
        for delegate_id in best_delegates {
            if let std::collections::hash_map::Entry::Vacant(e) =
                self.warm_connections.entry(delegate_id)
            {
                // In a real implementation, this would establish the actual connection
                // For now, we simulate it
                let connection = WarmConnection::new(delegate_id);
                e.insert(connection);
                self.registration_queue.push_back(delegate_id);
            }
        }

        // 3. Send registration to each delegate
        while let Some(delegate_id) = self.registration_queue.pop_front() {
            if let Some(connection) = self.warm_connections.get_mut(&delegate_id) {
                // In a real implementation, this would send the registration message
                // For now, we simulate successful registration
                connection.mark_registered();
            }
        }
    }

    /// Called when app comes to foreground
    ///
    /// Refreshes delegate routes and validates connections.
    pub async fn refresh_delegate_routes(&mut self) {
        // 1. Re-validate all delegate routes
        let mut valid_delegates = Vec::new();

        for delegate in &self.delegates {
            // In a real implementation, this would validate the route
            // For now, we assume all delegates are valid
            valid_delegates.push(delegate.clone());
        }

        self.delegates = valid_delegates;

        // 2. Update global route advertisements
        // In a real implementation, this would update the DHT

        // 3. Warm the predictive cache with delegate routes
        // In a real implementation, this would update the routing engine

        self.next_refresh = Instant::now() + self.config.refresh_interval;
    }

    /// Select the best delegates from known delegates
    fn select_best_delegates(&self, count: usize) -> Vec<PeerId> {
        let mut delegates: Vec<_> = self
            .delegates
            .iter()
            .filter(|d| d.has_capacity())
            .map(|d| (d.peer_id, d.score()))
            .collect();

        // Sort by score descending
        delegates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        delegates
            .iter()
            .take(count)
            .map(|(peer_id, _)| *peer_id)
            .collect()
    }

    /// Periodic maintenance
    pub fn tick(&mut self, now: Instant) -> DelegatePrewarmStats {
        let mut stats = DelegatePrewarmStats::default();

        // Check if it's time to refresh
        if now >= self.next_refresh {
            self.next_refresh = now + self.config.refresh_interval;
            stats.needs_refresh = true;
        }

        // Check for stale connections
        let stale_connections: Vec<PeerId> = self
            .warm_connections
            .iter()
            .filter(|&(_, conn)| conn.is_stale(self.config.stale_timeout))
            .map(|(&id, _)| id)
            .collect();

        for stale_id in stale_connections {
            self.warm_connections.remove(&stale_id);
            stats.stale_connections_removed += 1;
        }

        stats.active_connections = self.warm_connections.len();
        stats.registered_delegates = self
            .warm_connections
            .values()
            .filter(|conn| conn.is_registered)
            .count();

        stats
    }

    /// Get statistics
    pub fn stats(&self) -> DelegatePrewarmStats {
        DelegatePrewarmStats {
            active_connections: self.warm_connections.len(),
            registered_delegates: self
                .warm_connections
                .values()
                .filter(|conn| conn.is_registered)
                .count(),
            stale_connections_removed: 0,
            needs_refresh: Instant::now() >= self.next_refresh,
        }
    }

    /// Get the number of active connections
    pub fn active_connection_count(&self) -> usize {
        self.warm_connections.len()
    }

    /// Get the number of registered delegates
    pub fn registered_delegate_count(&self) -> usize {
        self.warm_connections
            .values()
            .filter(|conn| conn.is_registered)
            .count()
    }
}

/// Statistics for delegate pre-warming
#[derive(Debug, Clone, Default)]
pub struct DelegatePrewarmStats {
    /// Number of active connections
    pub active_connections: usize,
    /// Number of registered delegates
    pub registered_delegates: usize,
    /// Number of stale connections removed
    pub stale_connections_removed: usize,
    /// Whether a refresh is needed
    pub needs_refresh: bool,
}

impl std::fmt::Display for DelegatePrewarmStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Delegate Pre-warm: {} active, {} registered, {} stale removed, refresh needed: {}",
            self.active_connections,
            self.registered_delegates,
            self.stale_connections_removed,
            self.needs_refresh
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_delegate(id: u8) -> DelegateInfo {
        // Create a test peer ID using libp2p's test utilities
        use libp2p::identity::Keypair;
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        DelegateInfo::new(peer_id, format!("/ip4/127.0.0.1/tcp/{}", 10000 + id as u16))
    }

    #[test]
    fn test_delegate_creation() {
        let delegate = create_test_delegate(1);
        assert!((delegate.score() - delegate.reliability_score).abs() < f64::EPSILON);
        assert!(delegate.has_capacity());
    }

    #[test]
    fn test_delegate_selection() {
        let mut manager = DelegatePrewarmManager::with_defaults();

        // Add some delegates
        manager.add_delegate(create_test_delegate(1));
        manager.add_delegate(create_test_delegate(2));
        manager.add_delegate(create_test_delegate(3));

        // Select best delegates
        let best = manager.select_best_delegates(2);
        assert_eq!(best.len(), 2);
    }

    #[test]
    fn test_delegate_score_is_purely_behavioural() {
        // No node-class bonus: score equals measured reliability.
        let mut a = create_test_delegate(1);
        let mut b = create_test_delegate(2);
        a.update(0.9, 0);
        b.update(0.4, 0);
        assert!((a.score() - 0.9).abs() < f64::EPSILON);
        assert!((b.score() - 0.4).abs() < f64::EPSILON);

        // Selection follows measured reliability only.
        let best_id = a.peer_id;
        let mut manager = DelegatePrewarmManager::with_defaults();
        manager.add_delegate(b);
        manager.add_delegate(a);
        assert_eq!(manager.select_best_delegates(1), vec![best_id]);
    }

    #[test]
    fn test_prewarm_for_background() {
        let mut manager = DelegatePrewarmManager::with_defaults();

        // Add a delegate
        let delegate = create_test_delegate(1);
        manager.add_delegate(delegate);

        // Pre-warm for background
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            manager.prewarm_for_background().await;
        });

        // Should have a warm connection
        assert_eq!(manager.active_connection_count(), 1);
        assert_eq!(manager.registered_delegate_count(), 1);
    }

    #[test]
    fn test_tick_maintenance() {
        let mut manager = DelegatePrewarmManager::with_defaults();

        // Add a delegate and create a connection
        let delegate_id = create_test_delegate(1).peer_id;
        manager.add_delegate(create_test_delegate(1));
        manager
            .warm_connections
            .insert(delegate_id, WarmConnection::new(delegate_id));

        // Tick should not remove active connections
        let now = Instant::now();
        let stats = manager.tick(now);
        assert_eq!(stats.active_connections, 1);
    }

    #[test]
    fn test_stats() {
        let mut manager = DelegatePrewarmManager::with_defaults();

        // Add delegates
        manager.add_delegate(create_test_delegate(1));
        manager.add_delegate(create_test_delegate(2));

        let stats = manager.stats();
        assert_eq!(stats.active_connections, 0); // No connections yet
        assert!(stats.needs_refresh); // New manager needs refresh immediately
    }
}
