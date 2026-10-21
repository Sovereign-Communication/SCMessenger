// Enhanced relay selection and fallback discovery
//
// Replaces traditional bootstrap with dynamic priority relay system.
// All nodes are relays, prioritized by:
// - Historical stability and uptime metrics
// - Resource availability (bandwidth, storage, processing)
// - Network connectivity and geographic distribution
// - Real-time performance indicators
//
// Scoring is purely behavioural (uptime, latency, stability, recent
// failures). There is no node-class term: whether a node has an identity does
// not affect its rank. An always-on node simply earns a high score through
// its uptime and recency (#469, docs/rules/NODE_MODEL.md).

use libp2p::{Multiaddr, PeerId};
use std::collections::{HashMap, VecDeque};
use tracing::{debug, info};
use web_time::{Duration, SystemTime, UNIX_EPOCH};

/// Relay node stability metrics for priority calculation
#[derive(Debug, Clone)]
pub struct RelayMetrics {
    /// Peer ID of the relay
    pub peer_id: PeerId,
    /// Known multiaddresses
    pub addresses: Vec<Multiaddr>,
    /// Historical uptime percentage (0.0-1.0)
    pub uptime_ratio: f64,
    /// Average response latency in milliseconds
    pub avg_latency_ms: u64,
    /// Bandwidth capacity estimate (bytes/second)
    pub bandwidth_estimate: u64,
    /// Number of successful connections in last 24h
    pub recent_connections: u32,
    /// Number of failed connection attempts in last 24h
    pub recent_failures: u32,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Geographic region hint for distribution
    pub region: Option<String>,
    /// Relay stability score (0.0-1.0)
    pub stability_score: f64,
}

impl RelayMetrics {
    /// Calculate priority score for relay selection
    pub fn priority_score(&self) -> f64 {
        let base_score = self.uptime_ratio * 0.4
            + (1.0 - (self.avg_latency_ms as f64 / 1000.0).min(1.0)) * 0.3
            + self.stability_score * 0.3;

        // Penalty for recent failures
        let failure_penalty = (self.recent_failures as f64 / 100.0).min(0.2);

        (base_score - failure_penalty).clamp(0.0, 1.0)
    }

    /// Check if relay is considered healthy
    pub fn is_healthy(&self) -> bool {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let age_hours = (now_ms - self.last_seen) / (1000 * 60 * 60);

        self.uptime_ratio >= 0.8
            && self.stability_score >= 0.7
            && age_hours < 24
            && self.recent_failures < self.recent_connections
    }
}

/// Enhanced relay discovery system
#[derive(Debug)]
pub struct RelayDiscovery {
    /// Known relay metrics, indexed by peer ID
    relay_metrics: HashMap<PeerId, RelayMetrics>,
    /// Cached priority-ordered relay list
    priority_cache: VecDeque<PeerId>,
    /// Last cache update timestamp
    cache_updated: SystemTime,
    /// Cache validity duration
    cache_ttl: Duration,
    /// Fallback relay addresses for initial connectivity
    fallback_relays: Vec<Multiaddr>,
}

impl RelayDiscovery {
    pub fn new(fallback_relays: Vec<Multiaddr>) -> Self {
        Self {
            relay_metrics: HashMap::new(),
            priority_cache: VecDeque::new(),
            cache_updated: UNIX_EPOCH,
            cache_ttl: Duration::from_secs(300), // 5 minute cache
            fallback_relays,
        }
    }

    /// Add or update relay metrics
    pub fn update_relay_metrics(&mut self, metrics: RelayMetrics) {
        self.relay_metrics.insert(metrics.peer_id, metrics);
        self.invalidate_cache();
    }

    /// Record successful connection to relay
    pub fn record_success(&mut self, peer_id: &PeerId, latency_ms: u64) {
        if let Some(metrics) = self.relay_metrics.get_mut(peer_id) {
            metrics.recent_connections += 1;
            metrics.avg_latency_ms = (metrics.avg_latency_ms + latency_ms) / 2;
            metrics.last_seen = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            // Improve stability score on success
            metrics.stability_score = (metrics.stability_score + 0.1).min(1.0);
        }
        self.invalidate_cache();
    }

    /// Record failed connection attempt
    pub fn record_failure(&mut self, peer_id: &PeerId, reason: &str) {
        debug!("Recording relay failure for {}: {}", peer_id, reason);

        if let Some(metrics) = self.relay_metrics.get_mut(peer_id) {
            metrics.recent_failures += 1;

            // Degrade stability score on failure
            metrics.stability_score = (metrics.stability_score - 0.1).max(0.0);
        }
        self.invalidate_cache();
    }

    /// Get priority-ordered list of healthy relays
    pub fn get_priority_relays(&mut self, limit: usize) -> Vec<&RelayMetrics> {
        if self.cache_updated.elapsed().unwrap_or_default() > self.cache_ttl {
            self.rebuild_cache();
        }

        self.priority_cache
            .iter()
            .filter_map(|peer_id| self.relay_metrics.get(peer_id))
            .filter(|metrics| metrics.is_healthy())
            .take(limit)
            .collect()
    }

    /// Get fallback relay addresses for initial connectivity
    pub fn get_fallback_relays(&self) -> &[Multiaddr] {
        &self.fallback_relays
    }

    /// Add fallback relay address
    pub fn add_fallback_relay(&mut self, addr: Multiaddr) {
        if !self.fallback_relays.contains(&addr) {
            self.fallback_relays.push(addr.clone());
            info!("Added fallback relay: {}", addr);
        }
    }

    /// Get total number of known relays
    pub fn relay_count(&self) -> usize {
        self.relay_metrics.len()
    }

    /// Get number of healthy relays
    pub fn healthy_relay_count(&self) -> usize {
        self.relay_metrics
            .values()
            .filter(|metrics| metrics.is_healthy())
            .count()
    }

    /// Get all relay metrics (for diagnostics).
    /// Returns a list of (peer_id, metrics) pairs for all known relays.
    pub fn get_all_metrics(&self) -> Vec<(&PeerId, &RelayMetrics)> {
        self.relay_metrics.iter().collect()
    }

    /// Remove stale relay entries
    pub fn cleanup_stale_relays(&mut self, max_age_hours: u64) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let cutoff_ms = now_ms - (max_age_hours * 60 * 60 * 1000);

        let stale_peers: Vec<PeerId> = self
            .relay_metrics
            .iter()
            .filter_map(|(peer_id, metrics)| {
                if metrics.last_seen < cutoff_ms {
                    Some(*peer_id)
                } else {
                    None
                }
            })
            .collect();

        for peer_id in stale_peers {
            debug!("Removing stale relay: {}", peer_id);
            self.relay_metrics.remove(&peer_id);
        }

        if !self.relay_metrics.is_empty() {
            self.invalidate_cache();
        }
    }

    /// Invalidate priority cache
    fn invalidate_cache(&mut self) {
        self.cache_updated = UNIX_EPOCH;
    }

    /// Rebuild priority cache based on current metrics
    fn rebuild_cache(&mut self) {
        let mut relays: Vec<(&PeerId, &RelayMetrics)> = self.relay_metrics.iter().collect();

        // Sort by priority score (highest first)
        relays.sort_by(|a, b| {
            b.1.priority_score()
                .partial_cmp(&a.1.priority_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.priority_cache = relays.into_iter().map(|(peer_id, _)| *peer_id).collect();

        self.cache_updated = SystemTime::now();

        info!(
            "Rebuilt relay priority cache: {} relays, {} healthy",
            self.priority_cache.len(),
            self.healthy_relay_count()
        );
    }
}

/// Fallback relay discovery when priority relays fail
pub struct RelayFallback {
    /// Attempted relay addresses and their failure count
    attempted_relays: HashMap<Multiaddr, u32>,
    /// Maximum retry attempts per relay
    max_retries: u32,
    /// Backoff base duration in seconds
    backoff_base_secs: u64,
}

impl RelayFallback {
    pub fn new(max_retries: u32) -> Self {
        Self {
            attempted_relays: HashMap::new(),
            max_retries,
            backoff_base_secs: 5,
        }
    }

    /// Check if relay should be retried
    pub fn should_retry(&self, addr: &Multiaddr) -> bool {
        self.attempted_relays
            .get(addr)
            .map(|count| *count < self.max_retries)
            .unwrap_or(true)
    }

    /// Record relay attempt
    pub fn record_attempt(&mut self, addr: &Multiaddr) {
        let count = self.attempted_relays.entry(addr.clone()).or_insert(0);
        *count += 1;
    }

    /// Get backoff delay for retry
    pub fn get_backoff_delay(&self, addr: &Multiaddr) -> Duration {
        let attempts = self.attempted_relays.get(addr).unwrap_or(&0);
        let delay_secs = self.backoff_base_secs * (2u64.pow(*attempts));
        Duration::from_secs(delay_secs.min(300)) // Max 5 minute backoff
    }

    /// Reset attempt counters (call on successful connection)
    pub fn reset_attempts(&mut self) {
        self.attempted_relays.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity;

    #[test]
    fn test_relay_metrics_priority_score() {
        let peer_id = identity::Keypair::generate_ed25519().public().to_peer_id();

        let metrics = RelayMetrics {
            peer_id,
            addresses: vec![],
            uptime_ratio: 0.95,
            avg_latency_ms: 50,
            bandwidth_estimate: 1_000_000,
            recent_connections: 100,
            recent_failures: 5,
            last_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            region: Some("us-east".to_string()),
            stability_score: 0.9,
        };

        let score = metrics.priority_score();
        assert!(score > 0.8, "High-quality relay should have high score");
        assert!(
            metrics.is_healthy(),
            "Metrics should indicate healthy relay"
        );
    }

    #[test]
    fn test_priority_score_is_purely_behavioural() {
        // Scoring has no node-class input: two relays with identical
        // behavioural metrics score identically, and an always-on relay
        // (high uptime, fresh, low latency) outranks a flaky one purely on
        // behaviour.
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let make = |uptime: f64, latency: u64, failures: u32, stability: f64| RelayMetrics {
            peer_id: identity::Keypair::generate_ed25519().public().to_peer_id(),
            addresses: vec![],
            uptime_ratio: uptime,
            avg_latency_ms: latency,
            bandwidth_estimate: 1_000_000,
            recent_connections: 100,
            recent_failures: failures,
            last_seen: now_ms,
            region: None,
            stability_score: stability,
        };

        let a = make(0.9, 100, 2, 0.85);
        let b = make(0.9, 100, 2, 0.85);
        assert!((a.priority_score() - b.priority_score()).abs() < f64::EPSILON);

        // Exact weights: 0.4 uptime + 0.3 latency + 0.3 stability - failures.
        let expected = 0.9 * 0.4 + (1.0 - 0.1) * 0.3 + 0.85 * 0.3 - 0.02;
        assert!((a.priority_score() - expected).abs() < 1e-9);

        let always_on = make(0.99, 40, 0, 0.97);
        let flaky = make(0.82, 400, 30, 0.72);
        assert!(always_on.priority_score() > flaky.priority_score());
    }

    #[test]
    fn test_relay_discovery_priority_ordering() {
        let mut discovery = RelayDiscovery::new(vec![]);

        // Add relays with different quality metrics
        let high_quality = RelayMetrics {
            peer_id: identity::Keypair::generate_ed25519().public().to_peer_id(),
            addresses: vec![],
            uptime_ratio: 0.98,
            avg_latency_ms: 30,
            bandwidth_estimate: 10_000_000,
            recent_connections: 200,
            recent_failures: 1,
            last_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            region: Some("us-west".to_string()),
            stability_score: 0.95,
        };

        let low_quality = RelayMetrics {
            peer_id: identity::Keypair::generate_ed25519().public().to_peer_id(),
            addresses: vec![],
            uptime_ratio: 0.85, // Increased from 0.75 to pass health check (> 0.8)
            avg_latency_ms: 200,
            bandwidth_estimate: 1_000_000,
            recent_connections: 50,
            recent_failures: 20,
            last_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            region: Some("eu-central".to_string()),
            stability_score: 0.75, // Increased from 0.6 to pass health check (>= 0.7)
        };

        discovery.update_relay_metrics(low_quality.clone());
        discovery.update_relay_metrics(high_quality.clone());

        let priority_relays = discovery.get_priority_relays(2);

        // High quality relay should be first
        assert_eq!(priority_relays[0].peer_id, high_quality.peer_id);
        assert!(priority_relays[0].priority_score() > priority_relays[1].priority_score());
    }

    #[test]
    fn test_relay_fallback_backoff() {
        let mut fallback = RelayFallback::new(3);
        let addr: Multiaddr = "/ip4/1.2.3.4/tcp/4001".parse().unwrap();

        assert!(fallback.should_retry(&addr));

        fallback.record_attempt(&addr);
        let delay1 = fallback.get_backoff_delay(&addr);

        fallback.record_attempt(&addr);
        let delay2 = fallback.get_backoff_delay(&addr);

        assert!(delay2 > delay1, "Backoff delay should increase");

        fallback.record_attempt(&addr);
        fallback.record_attempt(&addr);

        assert!(
            !fallback.should_retry(&addr),
            "Should stop retrying after max attempts"
        );
    }
}
