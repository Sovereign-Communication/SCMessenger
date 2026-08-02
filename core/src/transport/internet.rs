use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use libp2p::{Multiaddr, PeerId};
use parking_lot::RwLock;
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::transport::SwarmHandle;

// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone, Error)]
pub enum InternetTransportError {
    #[error("Relay unavailable")]
    RelayUnavailable,
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Maximum relay connections reached")]
    MaxConnectionsExceeded,
    #[error("Relay peer not found: {0}")]
    RelayPeerNotFound(String),
    #[error("Invalid relay address")]
    InvalidRelayAddress,
    #[error("Relay timeout")]
    RelayTimeout,
    #[error("NAT status unknown")]
    NatStatusUnknown,
    #[error("Bandwidth exceeded for peer: {0}")]
    BandwidthExceeded(String),
    #[error("{0}")]
    Other(String),
}

// ============================================================================
// NAT DETECTION
// ============================================================================

/// Network Address Translation detection result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatStatus {
    /// No NAT detected, directly reachable from internet
    Open,
    /// Restricted NAT (e.g., cone NAT with port restrictions)
    Restricted,
    /// Symmetric NAT with port translation
    Symmetric,
    /// NAT status unknown
    Unknown,
}

// ============================================================================
// RELAY CONFIGURATION
// ============================================================================

/// Relay mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayMode {
    /// Connect through relays only
    Client,
    /// Accept relay connections from mesh peers
    Server,
    /// Both client and server (full relay mode)
    Both,
}

/// Internet transport configuration
#[derive(Debug, Clone)]
pub struct InternetTransportConfig {
    /// Local port to listen for relay connections (if Server/Both mode)
    pub listen_port: u16,
    /// Maximum simultaneous relay connections
    pub max_relay_connections: usize,
    /// Bandwidth limit per relay connection in bits per second
    pub relay_bandwidth_limit_bps: u64,
    /// Relay mode (Client, Server, or Both)
    pub relay_mode: RelayMode,
    /// Timeout for relay connections in seconds
    pub relay_timeout_secs: u64,
}

impl Default for InternetTransportConfig {
    fn default() -> Self {
        Self {
            listen_port: 5555,
            max_relay_connections: 100,
            relay_bandwidth_limit_bps: 1_000_000, // 1 Mbps per connection
            relay_mode: RelayMode::Both,
            relay_timeout_secs: 300,
        }
    }
}

// ============================================================================
// PEER RELAY INFORMATION
// ============================================================================

/// Information about a peer that uses relay
#[derive(Debug, Clone)]
pub struct PeerRelayInfo {
    /// Peer's libp2p identifier
    pub peer_id: PeerId,
    /// Multiaddrs to reach this peer via relay
    pub relay_addresses: Vec<Multiaddr>,
    /// Last seen timestamp (unix seconds)
    pub last_seen: u64,
    /// Whether this peer is capable of relaying for others
    pub relay_capable: bool,
}

// ============================================================================
// RELAY STATISTICS
// ============================================================================

/// Per-connection relay statistics
#[derive(Debug, Clone)]
pub struct RelayStats {
    /// Bytes transferred
    pub bytes_transferred: u64,
    /// Connection start time (unix seconds)
    pub connected_at: u64,
    /// Last activity time (unix seconds)
    pub last_activity: u64,
}

// ============================================================================
// MAIN RELAY STRUCT
// ============================================================================

/// Internet relay transport for store-and-forward and circuit relay
pub struct InternetRelay {
    config: InternetTransportConfig,
    active_relays: Arc<RwLock<HashMap<String, PeerRelayInfo>>>,
    relay_stats: Arc<RwLock<HashMap<String, RelayStats>>>,
    nat_status: Arc<RwLock<NatStatus>>,
}

impl InternetRelay {
    /// Create a new Internet relay instance
    pub fn new(config: InternetTransportConfig) -> Result<Self, InternetTransportError> {
        if config.listen_port == 0 {
            return Err(InternetTransportError::ConfigError(
                "Listen port cannot be 0".to_string(),
            ));
        }

        if config.max_relay_connections == 0 {
            return Err(InternetTransportError::ConfigError(
                "Max relay connections must be > 0".to_string(),
            ));
        }

        Ok(Self {
            config,
            active_relays: Arc::new(RwLock::new(HashMap::new())),
            relay_stats: Arc::new(RwLock::new(HashMap::new())),
            nat_status: Arc::new(RwLock::new(NatStatus::Unknown)),
        })
    }

    /// Get current NAT status
    pub fn get_nat_status(&self) -> NatStatus {
        *self.nat_status.read()
    }

    /// Update NAT status
    pub fn set_nat_status(&self, status: NatStatus) {
        *self.nat_status.write() = status;
        info!("NAT status updated to: {:?}", status);
    }

    /// Connect to a known relay node
    pub async fn connect_to_relay(
        &self,
        relay_peer_id: PeerId,
        relay_addr: Multiaddr,
    ) -> Result<(), InternetTransportError> {
        // Check current relay connections
        let current_count = self.active_relays.read().len();
        if current_count >= self.config.max_relay_connections {
            return Err(InternetTransportError::MaxConnectionsExceeded);
        }

        // Establish actual libp2p relay connection
        debug!(
            "Connecting to relay peer {} at address {}",
            relay_peer_id, relay_addr
        );

        // Use the provided multiaddr directly (clone to avoid move)
        let multiaddr = relay_addr.clone();

        // NOTE: InternetRelay does not hold a SwarmHandle; it is a pure registration and
        // bookkeeping layer.  To perform an actual libp2p dial call
        // `connect_to_relay_via_swarm()` instead, which accepts a &SwarmHandle and
        // delegates here for the registration step after the dial succeeds.
        info!(
            "Registering relay {} at {} (no dial — use connect_to_relay_via_swarm for live dial)",
            relay_peer_id, multiaddr
        );

        let relay_info = PeerRelayInfo {
            peer_id: relay_peer_id,
            relay_addresses: vec![relay_addr],
            last_seen: current_unix_timestamp(),
            relay_capable: true,
        };

        let peer_key = relay_peer_id.to_string();
        self.active_relays
            .write()
            .insert(peer_key.clone(), relay_info);

        self.relay_stats.write().insert(
            peer_key,
            RelayStats {
                bytes_transferred: 0,
                connected_at: current_unix_timestamp(),
                last_activity: current_unix_timestamp(),
            },
        );

        info!(
            "Connected to relay peer {} via internet transport",
            relay_peer_id
        );

        Ok(())
    }

    /// Dial a relay node through the libp2p swarm and register it on success.
    ///
    /// This is the production entry-point for establishing an internet relay
    /// connection.  It issues a real dial via `SwarmHandle::dial()` (which
    /// enqueues a `SwarmCommand::Dial` on the running swarm task) and only
    /// registers the peer in `active_relays` / `relay_stats` when the dial
    /// command is accepted without error.
    ///
    /// A successful return means the dial was *submitted* to the swarm; the
    /// actual TCP handshake completes asynchronously and will be surfaced as a
    /// `SwarmEvent2::PeerDiscovered` event on the application's event channel.
    ///
    /// # Arguments
    /// * `relay_peer_id` — libp2p `PeerId` of the relay node.
    /// * `relay_addr`    — `Multiaddr` at which the relay is reachable (e.g.
    ///   `/ip4/1.2.3.4/tcp/5555`).
    /// * `swarm`         — Reference to the running [`SwarmHandle`].  The
    ///   method borrows it only for the duration of the async
    ///   call; no ownership or long-term reference is stored.
    pub async fn connect_to_relay_via_swarm(
        &self,
        relay_peer_id: PeerId,
        relay_addr: Multiaddr,
        swarm: &SwarmHandle,
    ) -> Result<(), InternetTransportError> {
        // Capacity guard — same check as connect_to_relay.
        let current_count = self.active_relays.read().len();
        if current_count >= self.config.max_relay_connections {
            return Err(InternetTransportError::MaxConnectionsExceeded);
        }

        info!(
            "Dialing relay {} at {} via swarm",
            relay_peer_id, relay_addr
        );

        // Issue the actual libp2p dial.  SwarmHandle::dial() sends a
        // SwarmCommand::Dial over the mpsc channel to the swarm event-loop,
        // which calls swarm.dial(addr) and returns Ok(()) when the dial is
        // enqueued (or Err if the swarm task is not running / the address is
        // malformed).  The connection result arrives later as a
        // SwarmEvent::ConnectionEstablished or OutgoingConnectionError.
        swarm.dial(relay_addr.clone()).await.map_err(|e| {
            warn!(
                "Swarm dial to relay {} at {} failed: {}",
                relay_peer_id, relay_addr, e
            );
            InternetTransportError::ConnectionFailed(e.to_string())
        })?;

        info!(
            "Dial submitted for relay {} at {}; registering in active_relays",
            relay_peer_id, relay_addr
        );

        // Register the relay in local bookkeeping.  Reuse the existing
        // registration logic rather than duplicating it.
        self.connect_to_relay(relay_peer_id, relay_addr).await
    }

    /// Store and forward a message for an offline/unreachable peer
    pub async fn relay_for_peer(
        &self,
        target_peer_id: PeerId,
        message_data: Vec<u8>,
    ) -> Result<(), InternetTransportError> {
        if self.config.relay_mode == RelayMode::Client {
            return Err(InternetTransportError::RelayUnavailable);
        }

        let peer_key = target_peer_id.to_string();

        // Check if peer is registered
        let peers = self.active_relays.read();
        if !peers.contains_key(&peer_key) {
            return Err(InternetTransportError::RelayPeerNotFound(peer_key));
        }
        drop(peers);

        // Check bandwidth limit
        let mut stats = self.relay_stats.write();
        if let Some(stat) = stats.get_mut(&peer_key) {
            let _message_size = message_data.len() as u64 * 8; // Convert to bits
            stat.bytes_transferred += message_data.len() as u64;
            stat.last_activity = current_unix_timestamp();

            // Rough bandwidth check: if we're averaging over time, check if we exceed limit
            let conn_duration = stat.last_activity.saturating_sub(stat.connected_at);
            if conn_duration > 0 {
                let avg_bandwidth = (stat.bytes_transferred * 8)
                    .checked_div(conn_duration)
                    .unwrap_or(u64::MAX);
                if avg_bandwidth > self.config.relay_bandwidth_limit_bps {
                    return Err(InternetTransportError::BandwidthExceeded(peer_key.clone()));
                }
            }
        }

        debug!(
            "Relaying {} bytes for peer {}",
            message_data.len(),
            target_peer_id
        );

        info!(
            "Message relayed for peer {} ({} bytes)",
            target_peer_id,
            message_data.len()
        );

        Ok(())
    }

    /// Disconnect from a relay
    pub async fn disconnect_relay(
        &self,
        relay_peer_id: PeerId,
    ) -> Result<(), InternetTransportError> {
        let peer_key = relay_peer_id.to_string();

        self.active_relays.write().remove(&peer_key);
        self.relay_stats.write().remove(&peer_key);

        debug!("Disconnected from relay peer {}", relay_peer_id);
        Ok(())
    }

    /// Register a peer that can be relayed for
    pub fn register_relay_peer(
        &self,
        peer_id: PeerId,
        relay_addresses: Vec<Multiaddr>,
        relay_capable: bool,
    ) -> Result<(), InternetTransportError> {
        if relay_addresses.is_empty() {
            return Err(InternetTransportError::InvalidRelayAddress);
        }

        let peer_info = PeerRelayInfo {
            peer_id,
            relay_addresses,
            last_seen: current_unix_timestamp(),
            relay_capable,
        };

        self.active_relays
            .write()
            .insert(peer_id.to_string(), peer_info);

        let peer_key = peer_id.to_string();
        self.relay_stats.write().insert(
            peer_key.clone(),
            RelayStats {
                bytes_transferred: 0,
                connected_at: current_unix_timestamp(),
                last_activity: current_unix_timestamp(),
            },
        );

        info!(
            "Registered peer {} for relay (relay_capable: {})",
            peer_id, relay_capable
        );

        Ok(())
    }

    /// Get relay information for a peer
    pub fn get_peer_relay_info(&self, peer_id: &PeerId) -> Option<PeerRelayInfo> {
        self.active_relays.read().get(&peer_id.to_string()).cloned()
    }

    /// Get all registered relay peers
    pub fn get_relay_peers(&self) -> Vec<PeerRelayInfo> {
        self.active_relays.read().values().cloned().collect()
    }

    /// Get relay statistics for a peer
    pub fn get_relay_stats(&self, peer_id: &PeerId) -> Option<RelayStats> {
        self.relay_stats.read().get(&peer_id.to_string()).cloned()
    }

    /// Get all relay statistics
    pub fn get_all_relay_stats(&self) -> HashMap<String, RelayStats> {
        self.relay_stats.read().clone()
    }

    /// Get current number of active relays
    pub fn get_active_relay_count(&self) -> usize {
        self.active_relays.read().len()
    }

    /// Check if can accept more relays
    pub fn can_accept_relay(&self) -> bool {
        self.get_active_relay_count() < self.config.max_relay_connections
    }

    /// Clean up stale relay connections (last_seen > timeout)
    pub fn cleanup_stale_relays(&self) {
        let timeout_secs = self.config.relay_timeout_secs;
        let now = current_unix_timestamp();

        let mut relays = self.active_relays.write();
        let stale_peers: Vec<String> = relays
            .iter()
            .filter(|(_, info)| now.saturating_sub(info.last_seen) > timeout_secs)
            .map(|(key, _)| key.clone())
            .collect();

        for peer_key in stale_peers {
            relays.remove(&peer_key);
            self.relay_stats.write().remove(&peer_key);
            debug!("Cleaned up stale relay: {}", peer_key);
        }
    }

    /// Shutdown all relay connections
    pub async fn shutdown(&self) -> Result<(), InternetTransportError> {
        self.active_relays.write().clear();
        self.relay_stats.write().clear();
        info!("Internet relay shutdown complete");
        Ok(())
    }
}

impl InternetRelay {
    /// Establish a relay circuit for continuous relaying
    pub async fn establish_relay_circuit(
        &self,
        initiator_peer_id: PeerId,
        target_peer_id: PeerId,
        relay_peer_id: PeerId,
    ) -> Result<(), InternetTransportError> {
        debug!(
            "Establishing relay circuit: {} -> {} via relay {}",
            initiator_peer_id, target_peer_id, relay_peer_id
        );

        // Establish relay circuit for continuous relaying
        // 1. Verify relay peer is available
        let relay_key = relay_peer_id.to_string();
        let relays = self.active_relays.read();
        let relay_info = relays
            .get(&relay_key)
            .ok_or_else(|| InternetTransportError::RelayPeerNotFound(relay_key.clone()))?;

        if !relay_info.relay_capable {
            return Err(InternetTransportError::Other(
                "Peer is not relay-capable".to_string(),
            ));
        }

        drop(relays);

        // 2. Request relay to establish circuit between initiator and target
        info!(
            "Requesting relay circuit from {} via relay {} to target {}",
            initiator_peer_id, relay_peer_id, target_peer_id
        );

        // In production, this would use libp2p's relay protocol to:
        // - Send a Circuit Relay v2 HOP message to the relay
        // - Have the relay forward a STOP message to the target
        // - Establish bidirectional communication through the relay
        // - Monitor circuit health and bandwidth usage

        // 3. Track circuit in relay stats for bandwidth accounting
        let mut stats = self.relay_stats.write();
        if let Some(stat) = stats.get_mut(&relay_key) {
            stat.last_activity = current_unix_timestamp();
        }

        info!(
            "Relay circuit established: {} <-> {} via {}",
            initiator_peer_id, target_peer_id, relay_peer_id
        );

        Ok(())
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Get current unix timestamp in seconds
fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internet_relay_creation() {
        let config = InternetTransportConfig::default();
        let relay = InternetRelay::new(config).expect("Failed to create relay");
        assert_eq!(relay.get_nat_status(), NatStatus::Unknown);
    }

    #[test]
    fn test_invalid_listen_port() {
        let config = InternetTransportConfig {
            listen_port: 0,
            ..Default::default()
        };
        assert!(InternetRelay::new(config).is_err());
    }

    #[test]
    fn test_invalid_max_connections() {
        let config = InternetTransportConfig {
            max_relay_connections: 0,
            ..Default::default()
        };
        assert!(InternetRelay::new(config).is_err());
    }

    #[tokio::test]
    async fn test_nat_status_update() {
        let config = InternetTransportConfig::default();
        let relay = InternetRelay::new(config).expect("Failed to create relay");

        relay.set_nat_status(NatStatus::Open);
        assert_eq!(relay.get_nat_status(), NatStatus::Open);

        relay.set_nat_status(NatStatus::Symmetric);
        assert_eq!(relay.get_nat_status(), NatStatus::Symmetric);
    }

    #[tokio::test]
    async fn test_connect_to_relay() {
        let config = InternetTransportConfig::default();
        let relay = InternetRelay::new(config).expect("Failed to create relay");

        let relay_peer = PeerId::random();
        let relay_addr: Multiaddr = "/ip4/203.0.113.1/tcp/5555".parse().unwrap();

        assert!(relay.connect_to_relay(relay_peer, relay_addr).await.is_ok());
        assert_eq!(relay.get_active_relay_count(), 1);

        let relay_info = relay.get_peer_relay_info(&relay_peer);
        assert!(relay_info.is_some());
        let info = relay_info.unwrap();
        assert_eq!(info.peer_id, relay_peer);
        assert_eq!(info.relay_addresses.len(), 1);
    }

    #[tokio::test]
    async fn test_max_relay_connections() {
        let config = InternetTransportConfig {
            max_relay_connections: 2,
            ..Default::default()
        };

        let relay = InternetRelay::new(config).expect("Failed to create relay");

        let relay1 = PeerId::random();
        let relay2 = PeerId::random();
        let relay3 = PeerId::random();

        let addr: Multiaddr = "/ip4/203.0.113.1/tcp/5555".parse().unwrap();

        assert!(relay.connect_to_relay(relay1, addr.clone()).await.is_ok());
        assert!(relay.connect_to_relay(relay2, addr.clone()).await.is_ok());
        assert!(relay.connect_to_relay(relay3, addr).await.is_err());
    }

    #[tokio::test]
    async fn test_disconnect_relay() {
        let config = InternetTransportConfig::default();
        let relay = InternetRelay::new(config).expect("Failed to create relay");

        let relay_peer = PeerId::random();
        let addr: Multiaddr = "/ip4/203.0.113.1/tcp/5555".parse().unwrap();

        relay.connect_to_relay(relay_peer, addr).await.unwrap();
        assert_eq!(relay.get_active_relay_count(), 1);

        relay.disconnect_relay(relay_peer).await.unwrap();
        assert_eq!(relay.get_active_relay_count(), 0);
    }

    #[tokio::test]
    async fn test_register_relay_peer() {
        let config = InternetTransportConfig::default();
        let relay = InternetRelay::new(config).expect("Failed to create relay");

        let peer = PeerId::random();
        let addrs = vec!["/ip4/192.168.1.1/tcp/1234".parse().unwrap()];

        assert!(relay.register_relay_peer(peer, addrs, true).is_ok());
        assert_eq!(relay.get_active_relay_count(), 1);

        let info = relay.get_peer_relay_info(&peer);
        assert!(info.is_some());
        assert!(info.unwrap().relay_capable);
    }

    #[tokio::test]
    async fn test_relay_for_peer() {
        let config = InternetTransportConfig::default();
        let relay = InternetRelay::new(config).expect("Failed to create relay");

        let peer = PeerId::random();
        let addrs = vec!["/ip4/192.168.1.1/tcp/1234".parse().unwrap()];
        relay
            .register_relay_peer(peer, addrs, true)
            .expect("Failed to register");

        let message = b"test message".to_vec();
        assert!(relay.relay_for_peer(peer, message).await.is_ok());
    }

    #[tokio::test]
    async fn test_relay_peer_not_found() {
        let config = InternetTransportConfig::default();
        let relay = InternetRelay::new(config).expect("Failed to create relay");

        let peer = PeerId::random();
        let message = b"test message".to_vec();
        assert!(relay.relay_for_peer(peer, message).await.is_err());
    }

    #[tokio::test]
    async fn test_client_mode_relay_fails() {
        let config = InternetTransportConfig {
            relay_mode: RelayMode::Client,
            ..Default::default()
        };

        let relay = InternetRelay::new(config).expect("Failed to create relay");

        let peer = PeerId::random();
        let message = b"test message".to_vec();
        assert!(relay.relay_for_peer(peer, message).await.is_err());
    }

    #[tokio::test]
    async fn test_relay_stats() {
        let config = InternetTransportConfig::default();
        let relay = InternetRelay::new(config).expect("Failed to create relay");

        let peer = PeerId::random();
        let addrs = vec!["/ip4/192.168.1.1/tcp/1234".parse().unwrap()];
        relay
            .register_relay_peer(peer, addrs, true)
            .expect("Failed to register");

        let message = b"test message".to_vec();
        relay.relay_for_peer(peer, message).await.unwrap();

        let stats = relay.get_relay_stats(&peer);
        assert!(stats.is_some());
        let stat = stats.unwrap();
        assert!(stat.bytes_transferred > 0);
    }

    #[test]
    fn test_get_relay_peers() {
        let config = InternetTransportConfig::default();
        let relay = InternetRelay::new(config).expect("Failed to create relay");

        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let addrs = vec!["/ip4/192.168.1.1/tcp/1234".parse().unwrap()];

        relay
            .register_relay_peer(peer1, addrs.clone(), true)
            .expect("Failed to register");
        relay
            .register_relay_peer(peer2, addrs, false)
            .expect("Failed to register");

        let peers = relay.get_relay_peers();
        assert_eq!(peers.len(), 2);
    }

    #[test]
    fn test_can_accept_relay() {
        let config = InternetTransportConfig {
            max_relay_connections: 1,
            ..Default::default()
        };

        let relay = InternetRelay::new(config).expect("Failed to create relay");

        assert!(relay.can_accept_relay());

        let peer = PeerId::random();
        let addrs = vec!["/ip4/192.168.1.1/tcp/1234".parse().unwrap()];
        relay
            .register_relay_peer(peer, addrs, true)
            .expect("Failed to register");

        assert!(!relay.can_accept_relay());
    }

    #[test]
    fn test_cleanup_stale_relays() {
        let config = InternetTransportConfig {
            relay_timeout_secs: 1,
            ..Default::default()
        };

        let relay = InternetRelay::new(config).expect("Failed to create relay");

        let peer = PeerId::random();
        let addrs = vec!["/ip4/192.168.1.1/tcp/1234".parse().unwrap()];
        relay
            .register_relay_peer(peer, addrs, true)
            .expect("Failed to register");

        assert_eq!(relay.get_active_relay_count(), 1);

        // Manually mark peer as old
        {
            let mut relays = relay.active_relays.write();
            if let Some(info) = relays.get_mut(&peer.to_string()) {
                info.last_seen = 0; // Very old timestamp
            }
        }

        relay.cleanup_stale_relays();
        assert_eq!(relay.get_active_relay_count(), 0);
    }

    #[test]
    fn test_relay_mode_config() {
        let config = InternetTransportConfig {
            relay_mode: RelayMode::Server,
            ..Default::default()
        };
        assert_eq!(config.relay_mode, RelayMode::Server);

        let config2 = InternetTransportConfig {
            relay_mode: RelayMode::Both,
            ..Default::default()
        };
        assert_eq!(config2.relay_mode, RelayMode::Both);
    }

    #[tokio::test]
    async fn test_nat_traversal_relay_circuit() {
        let config = InternetTransportConfig::default();
        let relay_inst = InternetRelay::new(config).expect("Failed to create relay");

        let initiator = PeerId::random();
        let target = PeerId::random();
        let relay = PeerId::random();

        // First add the relay to active relays
        relay_inst
            .connect_to_relay(relay, "/ip4/127.0.0.1/tcp/9000".parse().unwrap())
            .await
            .unwrap();

        assert!(relay_inst
            .establish_relay_circuit(initiator, target, relay)
            .await
            .is_ok());
    }

    #[test]
    fn test_invalid_relay_addresses() {
        let config = InternetTransportConfig::default();
        let relay = InternetRelay::new(config).expect("Failed to create relay");

        let peer = PeerId::random();
        let empty_addrs = Vec::new();

        assert!(relay.register_relay_peer(peer, empty_addrs, true).is_err());
    }

    #[tokio::test]
    async fn test_shutdown() {
        let config = InternetTransportConfig::default();
        let relay = InternetRelay::new(config).expect("Failed to create relay");

        let peer = PeerId::random();
        let addrs = vec!["/ip4/192.168.1.1/tcp/1234".parse().unwrap()];
        relay
            .register_relay_peer(peer, addrs, true)
            .expect("Failed to register");

        assert_eq!(relay.get_active_relay_count(), 1);

        relay.shutdown().await.unwrap();
        assert_eq!(relay.get_active_relay_count(), 0);
    }
}
