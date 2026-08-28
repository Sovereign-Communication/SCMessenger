// Transport Layer Health Monitoring
//
// Provides comprehensive monitoring and diagnostics for the transport layer,
// including connection state tracking, performance metrics, and failure detection.

use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};
use web_time::{SystemTime, UNIX_EPOCH};

/// Connection state for a peer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    /// Connection is being established
    Connecting,
    /// Connection is established and active
    Connected,
    /// Connection is in the process of disconnecting
    Disconnecting,
    /// Connection has been disconnected
    Disconnected,
    /// Connection attempt failed
    Failed,
}

/// Connection statistics for a peer
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// Peer ID
    pub peer_id: PeerId,
    /// Connection state
    pub state: ConnectionState,
    /// Connection duration in milliseconds
    pub duration_ms: u64,
    /// Number of successful messages sent
    pub messages_sent: u64,
    /// Number of failed message attempts
    pub message_failures: u64,
    /// Number of bytes sent
    pub bytes_sent: u64,
    /// Number of bytes received
    pub bytes_received: u64,
    /// Average latency in milliseconds
    pub avg_latency_ms: u64,
    /// Last activity timestamp
    pub last_activity: u64,
    /// Number of connection attempts
    pub connection_attempts: u32,
    /// Number of successful connections
    pub successful_connections: u32,
    /// Number of connection failures
    pub connection_failures: u32,
    /// Current multiaddress
    pub current_address: Option<Multiaddr>,
    /// Known multiaddresses
    pub known_addresses: Vec<Multiaddr>,
}

impl ConnectionStats {
    /// Create new connection stats
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            peer_id,
            state: ConnectionState::Disconnected,
            duration_ms: 0,
            messages_sent: 0,
            message_failures: 0,
            bytes_sent: 0,
            bytes_received: 0,
            avg_latency_ms: 0,
            last_activity: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            connection_attempts: 0,
            successful_connections: 0,
            connection_failures: 0,
            current_address: None,
            known_addresses: Vec::new(),
        }
    }

    /// Update connection state
    pub fn update_state(&mut self, state: ConnectionState) {
        self.state = state;
        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    /// Record successful message
    pub fn record_message_success(&mut self, bytes: u64, latency_ms: u64) {
        self.messages_sent += 1;
        self.bytes_sent += bytes;

        // Update average latency (moving average)
        if self.messages_sent > 1 {
            self.avg_latency_ms = (self.avg_latency_ms + latency_ms) / 2;
        } else {
            self.avg_latency_ms = latency_ms;
        }

        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    /// Record failed message
    pub fn record_message_failure(&mut self) {
        self.message_failures += 1;
        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    /// Record bytes received
    pub fn record_bytes_received(&mut self, bytes: u64) {
        self.bytes_received += bytes;
        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    /// Record connection attempt
    pub fn record_connection_attempt(&mut self) {
        self.connection_attempts += 1;
        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    /// Record successful connection
    pub fn record_successful_connection(&mut self) {
        self.successful_connections += 1;
        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    /// Record connection failure
    pub fn record_connection_failure(&mut self) {
        self.connection_failures += 1;
        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    /// Update current address
    pub fn update_current_address(&mut self, address: Multiaddr) {
        self.current_address = Some(address.clone());
        if !self.known_addresses.contains(&address) {
            self.known_addresses.push(address);
        }
        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    /// Check if connection is healthy
    pub fn is_healthy(&self) -> bool {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let age_ms = now_ms - self.last_activity;

        // Connection is healthy if:
        // 1. It's currently connected
        // 2. Has recent activity (< 30 seconds)
        // 3. Has reasonable success rate (> 70%)
        self.state == ConnectionState::Connected
            && age_ms < 30000
            && self.successful_connections > 0
            && (self.connection_attempts == 0
                || (self.successful_connections as f64 / self.connection_attempts as f64) > 0.7)
    }

    /// Calculate connection quality score (0.0-1.0)
    pub fn quality_score(&self) -> f64 {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let age_ms = now_ms - self.last_activity;
        let age_factor = if age_ms < 10000 {
            1.0
        } else if age_ms < 30000 {
            0.8
        } else if age_ms < 60000 {
            0.5
        } else {
            0.2
        };

        let success_rate = if self.connection_attempts > 0 {
            self.successful_connections as f64 / self.connection_attempts as f64
        } else {
            1.0
        };

        let message_success_rate = if self.messages_sent + self.message_failures > 0 {
            self.messages_sent as f64 / (self.messages_sent + self.message_failures) as f64
        } else {
            1.0
        };

        // Weighted score: connection success (40%), message success (40%), recency (20%)
        (success_rate * 0.4 + message_success_rate * 0.4 + age_factor * 0.2).clamp(0.0, 1.0)
    }
}

/// Transport health monitor
pub struct TransportHealthMonitor {
    /// Connection statistics by peer ID
    connection_stats: Arc<Mutex<HashMap<PeerId, ConnectionStats>>>,
    /// Global transport metrics
    global_metrics: Arc<Mutex<GlobalTransportMetrics>>,
    /// Connection state change callbacks
    #[allow(clippy::type_complexity)]
    state_change_callbacks: Arc<Mutex<Vec<Box<dyn Fn(PeerId, ConnectionState) + Send + Sync>>>>,
}

impl std::fmt::Debug for TransportHealthMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransportHealthMonitor")
            .field("connection_stats", &self.connection_stats)
            .field("global_metrics", &self.global_metrics)
            .field(
                "state_change_callbacks",
                &format!(
                    "{} callbacks",
                    self.state_change_callbacks
                        .lock()
                        .expect("parking_lot Mutex never poisons")
                        .len()
                ),
            )
            .finish()
    }
}

impl Default for TransportHealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportHealthMonitor {
    /// Create a new transport health monitor
    pub fn new() -> Self {
        Self {
            connection_stats: Arc::new(Mutex::new(HashMap::new())),
            global_metrics: Arc::new(Mutex::new(GlobalTransportMetrics::new())),
            state_change_callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Update connection state
    pub fn update_connection_state(&self, peer_id: PeerId, state: ConnectionState) {
        let mut stats = self
            .connection_stats
            .lock()
            .expect("parking_lot Mutex never poisons");
        let entry = stats
            .entry(peer_id)
            .or_insert_with(|| ConnectionStats::new(peer_id));
        entry.update_state(state.clone());

        // Notify callbacks
        let callbacks = self
            .state_change_callbacks
            .lock()
            .expect("parking_lot Mutex never poisons");
        for callback in callbacks.iter() {
            callback(peer_id, state.clone());
        }

        // Update global metrics
        let mut global = self
            .global_metrics
            .lock()
            .expect("parking_lot Mutex never poisons");
        global.record_connection_state_change(&state);

        match state {
            ConnectionState::Connected => {
                info!("Connection established to peer: {}", peer_id);
            }
            ConnectionState::Disconnected => {
                warn!("Connection lost to peer: {}", peer_id);
            }
            ConnectionState::Failed => {
                error!("Connection failed to peer: {}", peer_id);
            }
            _ => {}
        }
    }

    /// Record successful message
    pub fn record_message_success(&self, peer_id: PeerId, bytes: u64, latency_ms: u64) {
        let mut stats = self
            .connection_stats
            .lock()
            .expect("parking_lot Mutex never poisons");
        if let Some(entry) = stats.get_mut(&peer_id) {
            entry.record_message_success(bytes, latency_ms);
        }

        let mut global = self
            .global_metrics
            .lock()
            .expect("parking_lot Mutex never poisons");
        global.record_message_success(bytes, latency_ms);
    }

    /// Record failed message
    pub fn record_message_failure(&self, peer_id: PeerId) {
        let mut stats = self
            .connection_stats
            .lock()
            .expect("parking_lot Mutex never poisons");
        if let Some(entry) = stats.get_mut(&peer_id) {
            entry.record_message_failure();
        }

        let mut global = self
            .global_metrics
            .lock()
            .expect("parking_lot Mutex never poisons");
        global.record_message_failure();
    }

    /// Record bytes received
    pub fn record_bytes_received(&self, peer_id: PeerId, bytes: u64) {
        let mut stats = self
            .connection_stats
            .lock()
            .expect("parking_lot Mutex never poisons");
        if let Some(entry) = stats.get_mut(&peer_id) {
            entry.record_bytes_received(bytes);
        }

        let mut global = self
            .global_metrics
            .lock()
            .expect("parking_lot Mutex never poisons");
        global.record_bytes_received(bytes);
    }

    /// Record connection attempt
    pub fn record_connection_attempt(&self, peer_id: PeerId) {
        let mut stats = self
            .connection_stats
            .lock()
            .expect("parking_lot Mutex never poisons");
        let entry = stats
            .entry(peer_id)
            .or_insert_with(|| ConnectionStats::new(peer_id));
        entry.record_connection_attempt();

        let mut global = self
            .global_metrics
            .lock()
            .expect("parking_lot Mutex never poisons");
        global.record_connection_attempt();
    }

    /// Record successful connection
    pub fn record_successful_connection(&self, peer_id: PeerId) {
        let mut stats = self
            .connection_stats
            .lock()
            .expect("parking_lot Mutex never poisons");
        if let Some(entry) = stats.get_mut(&peer_id) {
            entry.record_successful_connection();
        }

        let mut global = self
            .global_metrics
            .lock()
            .expect("parking_lot Mutex never poisons");
        global.record_successful_connection();
    }

    /// Record connection failure
    pub fn record_connection_failure(&self, peer_id: PeerId) {
        let mut stats = self
            .connection_stats
            .lock()
            .expect("parking_lot Mutex never poisons");
        if let Some(entry) = stats.get_mut(&peer_id) {
            entry.record_connection_failure();
        }

        let mut global = self
            .global_metrics
            .lock()
            .expect("parking_lot Mutex never poisons");
        global.record_connection_failure();
    }

    /// Update current address
    pub fn update_current_address(&self, peer_id: PeerId, address: Multiaddr) {
        let mut stats = self
            .connection_stats
            .lock()
            .expect("parking_lot Mutex never poisons");
        if let Some(entry) = stats.get_mut(&peer_id) {
            entry.update_current_address(address);
        }
    }

    /// Get connection stats for a peer
    pub fn get_connection_stats(&self, peer_id: &PeerId) -> Option<ConnectionStats> {
        let stats = self
            .connection_stats
            .lock()
            .expect("parking_lot Mutex never poisons");
        stats.get(peer_id).cloned()
    }

    /// Get all connection stats
    pub fn get_all_connection_stats(&self) -> HashMap<PeerId, ConnectionStats> {
        let stats = self
            .connection_stats
            .lock()
            .expect("parking_lot Mutex never poisons");
        stats.clone()
    }

    /// Get global transport metrics
    pub fn get_global_metrics(&self) -> GlobalTransportMetrics {
        let global = self
            .global_metrics
            .lock()
            .expect("parking_lot Mutex never poisons");
        global.clone()
    }

    /// Get healthy connections
    pub fn get_healthy_connections(&self) -> Vec<PeerId> {
        let stats = self
            .connection_stats
            .lock()
            .expect("parking_lot Mutex never poisons");
        stats
            .iter()
            .filter(|(_, stat)| stat.is_healthy())
            .map(|(peer_id, _)| *peer_id)
            .collect()
    }

    /// Get unhealthy connections
    pub fn get_unhealthy_connections(&self) -> Vec<PeerId> {
        let stats = self
            .connection_stats
            .lock()
            .expect("parking_lot Mutex never poisons");
        stats
            .iter()
            .filter(|(_, stat)| !stat.is_healthy())
            .map(|(peer_id, _)| *peer_id)
            .collect()
    }

    /// Register connection state change callback
    pub fn register_state_change_callback<F>(&self, callback: F)
    where
        F: Fn(PeerId, ConnectionState) + Send + Sync + 'static,
    {
        let mut callbacks = self
            .state_change_callbacks
            .lock()
            .expect("parking_lot Mutex never poisons");
        callbacks.push(Box::new(callback));
    }

    /// Clean up stale connection entries
    pub fn cleanup_stale_connections(&self, max_age_secs: u64) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let cutoff_ms = now_ms - (max_age_secs * 1000);

        let mut stats = self
            .connection_stats
            .lock()
            .expect("parking_lot Mutex never poisons");
        let stale_peers: Vec<PeerId> = stats
            .iter()
            .filter(|(_, stat)| stat.last_activity < cutoff_ms)
            .map(|(peer_id, _)| *peer_id)
            .collect();

        for peer_id in stale_peers {
            debug!("Removing stale connection entry for peer: {}", peer_id);
            stats.remove(&peer_id);
        }
    }
}

/// Global transport metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalTransportMetrics {
    /// Total connection attempts
    pub total_connection_attempts: u64,
    /// Total successful connections
    pub total_successful_connections: u64,
    /// Total connection failures
    pub total_connection_failures: u64,
    /// Total messages sent
    pub total_messages_sent: u64,
    /// Total message failures
    pub total_message_failures: u64,
    /// Total bytes sent
    pub total_bytes_sent: u64,
    /// Total bytes received
    pub total_bytes_received: u64,
    /// Average message latency in milliseconds (moving average)
    #[serde(default)]
    pub avg_latency_ms: u64,
    /// Current active connections
    pub current_active_connections: u32,
    /// Peak active connections
    pub peak_active_connections: u32,
    /// Connection state counts
    pub connection_state_counts: HashMap<ConnectionState, u32>,
    /// Start timestamp
    pub start_timestamp: u64,
}

impl Default for GlobalTransportMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalTransportMetrics {
    /// Create new global transport metrics
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            total_connection_attempts: 0,
            total_successful_connections: 0,
            total_connection_failures: 0,
            total_messages_sent: 0,
            total_message_failures: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            avg_latency_ms: 0,
            current_active_connections: 0,
            peak_active_connections: 0,
            connection_state_counts: HashMap::new(),
            start_timestamp: now,
        }
    }

    /// Record connection attempt
    pub fn record_connection_attempt(&mut self) {
        self.total_connection_attempts += 1;
    }

    /// Record successful connection
    pub fn record_successful_connection(&mut self) {
        self.total_successful_connections += 1;
        self.current_active_connections += 1;
        if self.current_active_connections > self.peak_active_connections {
            self.peak_active_connections = self.current_active_connections;
        }
    }

    /// Record connection failure
    pub fn record_connection_failure(&mut self) {
        self.total_connection_failures += 1;
    }

    /// Record message success
    pub fn record_message_success(&mut self, bytes: u64, latency_ms: u64) {
        self.total_messages_sent += 1;
        self.total_bytes_sent += bytes;

        // Update average latency (moving average), mirroring the per-peer
        // TransportHealth::record_message_success.
        if self.total_messages_sent > 1 {
            self.avg_latency_ms = (self.avg_latency_ms + latency_ms) / 2;
        } else {
            self.avg_latency_ms = latency_ms;
        }
    }

    /// Record message failure
    pub fn record_message_failure(&mut self) {
        self.total_message_failures += 1;
    }

    /// Record bytes received
    pub fn record_bytes_received(&mut self, bytes: u64) {
        self.total_bytes_received += bytes;
    }

    /// Record connection state change
    pub fn record_connection_state_change(&mut self, state: &ConnectionState) {
        match state {
            ConnectionState::Connected => {
                self.current_active_connections += 1;
                if self.current_active_connections > self.peak_active_connections {
                    self.peak_active_connections = self.current_active_connections;
                }
            }
            ConnectionState::Disconnected | ConnectionState::Failed
                if self.current_active_connections > 0 =>
            {
                self.current_active_connections -= 1;
            }
            _ => {}
        }

        let count = self
            .connection_state_counts
            .entry(state.clone())
            .or_insert(0);
        *count += 1;
    }

    /// Calculate overall transport health score (0.0-1.0)
    pub fn health_score(&self) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let uptime_secs = (now - self.start_timestamp) / 1000;
        if uptime_secs == 0 {
            return 1.0; // Just started
        }

        let connection_success_rate = if self.total_connection_attempts > 0 {
            self.total_successful_connections as f64 / self.total_connection_attempts as f64
        } else {
            1.0
        };

        let message_success_rate = if self.total_messages_sent + self.total_message_failures > 0 {
            self.total_messages_sent as f64
                / (self.total_messages_sent + self.total_message_failures) as f64
        } else {
            1.0
        };

        let connection_stability = if self.total_successful_connections > 0 {
            let avg_duration = if self.current_active_connections > 0 {
                uptime_secs / self.current_active_connections as u64
            } else {
                0
            };
            (avg_duration as f64 / 60.0).min(1.0) // Cap at 1 minute average
        } else {
            1.0
        };

        // Weighted score: connection success (30%), message success (40%), stability (30%)
        (connection_success_rate * 0.3 + message_success_rate * 0.4 + connection_stability * 0.3)
            .clamp(0.0, 1.0)
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        (now - self.start_timestamp) / 1000
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity;

    #[test]
    fn test_connection_stats_quality_score() {
        let peer_id = identity::Keypair::generate_ed25519().public().to_peer_id();
        let mut stats = ConnectionStats::new(peer_id);

        // Simulate good connection
        stats.update_state(ConnectionState::Connected);
        stats.record_successful_connection();
        stats.record_message_success(1024, 50);
        stats.record_message_success(2048, 60);

        let score = stats.quality_score();
        assert!(
            score > 0.8,
            "Good connection should have high quality score"
        );
        assert!(stats.is_healthy(), "Good connection should be healthy");
    }

    #[test]
    fn test_connection_stats_unhealthy() {
        let peer_id = identity::Keypair::generate_ed25519().public().to_peer_id();
        let mut stats = ConnectionStats::new(peer_id);

        // Simulate bad connection
        stats.record_connection_attempt();
        stats.record_connection_attempt();
        stats.record_connection_failure();
        stats.record_message_failure();

        let score = stats.quality_score();
        assert!(score < 0.5, "Bad connection should have low quality score");
        assert!(!stats.is_healthy(), "Bad connection should not be healthy");
    }

    #[test]
    fn test_transport_health_monitor() {
        let monitor = TransportHealthMonitor::new();
        let peer_id = identity::Keypair::generate_ed25519().public().to_peer_id();

        // Test connection lifecycle
        monitor.update_connection_state(peer_id, ConnectionState::Connecting);
        monitor.record_connection_attempt(peer_id);
        monitor.update_connection_state(peer_id, ConnectionState::Connected);
        monitor.record_successful_connection(peer_id);
        monitor.record_message_success(peer_id, 1024, 50);

        // Verify stats
        let stats = monitor.get_connection_stats(&peer_id);
        assert!(stats.is_some(), "Should have connection stats");
        let stats = stats.expect("just asserted is_some");
        assert_eq!(stats.state, ConnectionState::Connected);
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.successful_connections, 1);

        // Verify global metrics
        let metrics = monitor.get_global_metrics();
        assert_eq!(metrics.total_successful_connections, 1);
        assert_eq!(metrics.total_messages_sent, 1);
    }

    #[test]
    fn test_global_metrics_health_score() {
        let mut metrics = GlobalTransportMetrics::new();

        // Simulate good transport performance
        for _ in 0..10 {
            metrics.record_connection_attempt();
            metrics.record_successful_connection();
            metrics.record_message_success(1024, 50);
        }
        assert_eq!(
            metrics.avg_latency_ms, 50,
            "moving average of uniform latency"
        );

        let score = metrics.health_score();
        assert!(
            score > 0.9,
            "Good transport metrics should have high health score"
        );
    }
}
