use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use libp2p::PeerId;
use parking_lot::RwLock;
use thiserror::Error;
use tracing::{info, warn};

// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone, Error)]
pub enum WifiAwareError {
    #[error("WiFi Aware unavailable on this device")]
    Unavailable,
    #[error("Service discovery failed: {0}")]
    DiscoveryFailed(String),
    #[error("Data path creation failed: {0}")]
    DataPathFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Platform bridge error: {0}")]
    PlatformError(String),
    #[error("State transition error: {0}")]
    InvalidStateTransition(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
}

// ============================================================================
// SERVICE INFO TLV ENCODING/DECODING
// ============================================================================

pub const TLV_TYPE_PORT: u8 = 0x01;

pub fn encode_port_tlv(port: u16) -> Vec<u8> {
    vec![TLV_TYPE_PORT, 2, (port >> 8) as u8, (port & 0xff) as u8]
}

pub fn decode_port_tlv(service_info: &[u8]) -> Option<u16> {
    let mut i = 0;
    let mut found_port: Option<u16> = None;
    while i + 1 < service_info.len() {
        let tlv_type = service_info[i];
        let tlv_len = service_info[i + 1] as usize;
        let value_end = i.checked_add(2)?.checked_add(tlv_len)?;
        if value_end > service_info.len() {
            break;
        }
        if tlv_type == TLV_TYPE_PORT && tlv_len == 2 {
            let port = ((service_info[i + 2] as u16) << 8) | (service_info[i + 3] as u16);
            if found_port.is_some() {
                return None;
            }
            found_port = Some(port);
        }
        i = value_end;
    }
    found_port
}

// ============================================================================
// CONFIGURATION
// ============================================================================

/// WiFi Aware transport configuration
#[derive(Debug, Clone)]
pub struct WifiAwareConfig {
    /// Service name to publish/discover (e.g., "SCMesh")
    pub service_name: String,
    /// Optional service info to include in beacons (will be encrypted)
    pub service_info: Vec<u8>,
    /// Match filter criteria for discovery (optional)
    pub match_filter: Option<Vec<u8>>,
    /// Enable publishing this node's service
    pub publish_enabled: bool,
    /// Enable subscribing to peer services
    pub subscribe_enabled: bool,
    /// Maximum simultaneous data paths
    pub max_data_paths: usize,
    /// Port to advertise in service_info TLV for per-peer negotiation
    pub listen_port: Option<u16>,
}

impl Default for WifiAwareConfig {
    fn default() -> Self {
        Self {
            service_name: "SCMesh".to_string(),
            service_info: Vec::new(),
            match_filter: None,
            publish_enabled: true,
            subscribe_enabled: true,
            max_data_paths: 10,
            listen_port: None,
        }
    }
}

// ============================================================================
// STATE MANAGEMENT
// ============================================================================

/// WiFi Aware capability and connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiAwareState {
    /// WiFi Aware not available on this device
    Unavailable,
    /// WiFi Aware available but not active
    Available,
    /// Service is being published
    Publishing,
    /// Subscribed to peer services
    Subscribing,
    /// Active data path established
    DataPathActive,
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Information about an established WiFi Aware data path
#[derive(Debug, Clone)]
pub struct DataPathInfo {
    /// Peer's libp2p identifier
    pub peer_id: PeerId,
    /// IP address for data transfer on this path
    pub ip_address: String,
    /// Port for data transfer
    pub port: u16,
    /// Estimated bandwidth in bits per second
    pub bandwidth_estimate: u64,
    /// Whether this is a publisher (us) or subscriber (peer)
    pub is_publisher: bool,
}

/// Discovered peer information
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Peer's libp2p identifier
    pub peer_id: PeerId,
    /// Service information (encrypted)
    pub service_info: Vec<u8>,
    /// RSSI signal strength (typically -40 to -120 dBm)
    pub rssi: i32,
}

// ============================================================================
// PLATFORM BRIDGE TRAIT
// ============================================================================

/// Platform-specific WiFi Aware API abstraction
///
/// Implementers provide actual WiFi Aware API calls for their platform.
/// This is typically implemented by platform-specific code (e.g., iOS/Android bindings).
#[async_trait]
pub trait WifiAwarePlatformBridge: Send + Sync {
    /// Check if WiFi Aware is available on this device
    async fn is_available(&self) -> Result<bool, WifiAwareError>;

    /// Publish a service with the given name and info
    async fn publish_service(
        &self,
        service_name: &str,
        service_info: &[u8],
    ) -> Result<(), WifiAwareError>;

    /// Subscribe to services matching the criteria
    async fn subscribe_to_services(
        &self,
        service_name: &str,
        match_filter: Option<&[u8]>,
    ) -> Result<(), WifiAwareError>;

    /// Stop publishing the service
    async fn unpublish_service(&self) -> Result<(), WifiAwareError>;

    /// Stop subscribing to services
    async fn unsubscribe_from_services(&self) -> Result<(), WifiAwareError>;

    /// Create a data path to a discovered peer
    async fn create_data_path(
        &self,
        peer_id: &str,
        pmk: &[u8; 32],
    ) -> Result<SocketAddr, WifiAwareError>;

    /// Close a data path
    async fn close_data_path(&self, peer_id: &str) -> Result<(), WifiAwareError>;

    /// Register callback for service discovered event
    fn set_on_service_discovered(&self, callback: Box<dyn Fn(String, Vec<u8>, i32) + Send + Sync>);

    /// Register callback for message received on data path
    fn set_on_message_received(&self, callback: Box<dyn Fn(String, Vec<u8>) + Send + Sync>);

    /// Register callback for data path confirmation
    fn set_on_data_path_confirmed(&self, callback: Box<dyn Fn(String, SocketAddr) + Send + Sync>);
}

// ============================================================================
// MOCK PLATFORM BRIDGE (for testing)
// ============================================================================

/// Mock implementation of WifiAwarePlatformBridge for testing
#[cfg(test)]
pub struct MockWifiAwareBridge {
    available: bool,
    discovered_peers: Arc<RwLock<Vec<DiscoveredPeer>>>,
}

#[cfg(test)]
impl MockWifiAwareBridge {
    pub fn new(available: bool) -> Self {
        Self {
            available,
            discovered_peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add_discovered_peer(&self, peer: DiscoveredPeer) {
        self.discovered_peers.write().push(peer);
    }
}

#[cfg(test)]
#[async_trait]
impl WifiAwarePlatformBridge for MockWifiAwareBridge {
    async fn is_available(&self) -> Result<bool, WifiAwareError> {
        Ok(self.available)
    }

    // PERIMETER-ALLOW-UNDERSCORE: test-only mock of WifiAwarePlatformBridge;
    // MockWifiAwareBridge is a fixed available/unavailable stub and does not
    // model per-call arguments.
    async fn publish_service(
        &self,
        service_name: &str,
        _service_info: &[u8],
    ) -> Result<(), WifiAwareError> {
        if !self.available {
            return Err(WifiAwareError::Unavailable);
        }
        let _ = service_name;
        Ok(())
    }

    // PERIMETER-ALLOW-UNDERSCORE: test-only mock, see `publish_service` above.
    async fn subscribe_to_services(
        &self,
        service_name: &str,
        _match_filter: Option<&[u8]>,
    ) -> Result<(), WifiAwareError> {
        if !self.available {
            return Err(WifiAwareError::Unavailable);
        }
        let _ = service_name;
        Ok(())
    }

    async fn unpublish_service(&self) -> Result<(), WifiAwareError> {
        Ok(())
    }

    async fn unsubscribe_from_services(&self) -> Result<(), WifiAwareError> {
        Ok(())
    }

    // PERIMETER-ALLOW-UNDERSCORE: test-only mock, see `publish_service` above.
    async fn create_data_path(
        &self,
        peer_id: &str,
        _pmk: &[u8; 32],
    ) -> Result<SocketAddr, WifiAwareError> {
        let _ = peer_id;
        Ok("192.168.100.1:5000".parse().unwrap())
    }

    async fn close_data_path(&self, peer_id: &str) -> Result<(), WifiAwareError> {
        let _ = peer_id;
        Ok(())
    }

    // PERIMETER-ALLOW-UNDERSCORE: test-only mock, see `publish_service` above.
    fn set_on_service_discovered(
        &self,
        _callback: Box<dyn Fn(String, Vec<u8>, i32) + Send + Sync>,
    ) {
    }

    // PERIMETER-ALLOW-UNDERSCORE: test-only mock, see `publish_service` above.
    fn set_on_message_received(&self, _callback: Box<dyn Fn(String, Vec<u8>) + Send + Sync>) {}

    // PERIMETER-ALLOW-UNDERSCORE: test-only mock, see `publish_service` above.
    fn set_on_data_path_confirmed(&self, _callback: Box<dyn Fn(String, SocketAddr) + Send + Sync>) {
    }
}

// ============================================================================
// MAIN TRANSPORT STRUCT
// ============================================================================

/// WiFi Aware transport for high-bandwidth P2P communication
pub struct WifiAwareTransport {
    config: WifiAwareConfig,
    state: Arc<RwLock<WifiAwareState>>,
    bridge: Arc<dyn WifiAwarePlatformBridge>,
    data_paths: Arc<RwLock<HashMap<String, DataPathInfo>>>,
    discovered_peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,
}

impl WifiAwareTransport {
    /// Create a new WiFi Aware transport
    pub fn new(
        config: WifiAwareConfig,
        bridge: Arc<dyn WifiAwarePlatformBridge>,
    ) -> Result<Self, WifiAwareError> {
        if config.service_name.is_empty() {
            return Err(WifiAwareError::InvalidConfig(
                "Service name cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(WifiAwareState::Available)),
            bridge,
            data_paths: Arc::new(RwLock::new(HashMap::new())),
            discovered_peers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get current WiFi Aware state
    pub fn get_state(&self) -> WifiAwareState {
        *self.state.read()
    }

    /// Initialize and check WiFi Aware availability
    pub async fn initialize(&self) -> Result<(), WifiAwareError> {
        let available = self.bridge.is_available().await.map_err(|e| {
            warn!("WiFi Aware availability check failed: {}", e);
            e
        })?;

        if !available {
            *self.state.write() = WifiAwareState::Unavailable;
            return Err(WifiAwareError::Unavailable);
        }

        info!("WiFi Aware initialized and available");
        Ok(())
    }

    /// Publish this node's service to nearby peers
    pub async fn publish_service(&self) -> Result<(), WifiAwareError> {
        if !self.config.publish_enabled {
            return Err(WifiAwareError::InvalidConfig(
                "Publishing not enabled in config".to_string(),
            ));
        }

        let state = self.get_state();
        if state == WifiAwareState::Unavailable {
            return Err(WifiAwareError::Unavailable);
        }

        let mut info = self.config.service_info.clone();
        if let Some(port) = self.config.listen_port {
            info.extend_from_slice(&encode_port_tlv(port));
        }
        self.bridge
            .publish_service(&self.config.service_name, &info)
            .await?;

        *self.state.write() = WifiAwareState::Publishing;
        info!("Published WiFi Aware service: {}", self.config.service_name);
        Ok(())
    }

    /// Subscribe to discover nearby mesh peers
    pub async fn subscribe(&self) -> Result<(), WifiAwareError> {
        if !self.config.subscribe_enabled {
            return Err(WifiAwareError::InvalidConfig(
                "Subscription not enabled in config".to_string(),
            ));
        }

        let state = self.get_state();
        if state == WifiAwareState::Unavailable {
            return Err(WifiAwareError::Unavailable);
        }

        self.bridge
            .subscribe_to_services(
                &self.config.service_name,
                self.config.match_filter.as_deref(),
            )
            .await?;

        *self.state.write() = WifiAwareState::Subscribing;
        info!(
            "Subscribed to WiFi Aware service: {}",
            self.config.service_name
        );
        Ok(())
    }

    /// Create a data path to a discovered peer
    pub async fn create_data_path(
        &self,
        peer_id: PeerId,
        pmk: &[u8; 32],
    ) -> Result<DataPathInfo, WifiAwareError> {
        let current_state = self.get_state();
        if current_state == WifiAwareState::Unavailable {
            return Err(WifiAwareError::Unavailable);
        }

        let peer_id_str = peer_id.to_string();

        // Check if peer is discovered (guard dropped before any await)
        let peer = {
            let discovered = self.discovered_peers.read();
            discovered
                .get(&peer_id_str)
                .ok_or_else(|| WifiAwareError::PeerNotFound(peer_id_str.clone()))?
                .clone()
        };

        // Check if we're at capacity
        {
            let paths = self.data_paths.read();
            if paths.len() >= self.config.max_data_paths {
                return Err(WifiAwareError::DataPathFailed(
                    "Maximum data paths reached".to_string(),
                ));
            }
        }

        // Create data path via platform bridge (no locks held during await)
        let socket_addr = self.bridge.create_data_path(&peer_id_str, pmk).await?;

        let data_path_info = DataPathInfo {
            peer_id,
            ip_address: socket_addr.ip().to_string(),
            port: socket_addr.port(),
            bandwidth_estimate: estimate_bandwidth_from_rssi(peer.rssi),
            is_publisher: false,
        };

        self.data_paths
            .write()
            .insert(peer_id_str.clone(), data_path_info.clone());
        *self.state.write() = WifiAwareState::DataPathActive;

        info!(
            "Created WiFi Aware data path to peer {}: {}:{}",
            peer_id_str, data_path_info.ip_address, data_path_info.port
        );

        Ok(data_path_info)
    }

    /// Close a data path to a peer
    pub async fn close_data_path(&self, peer_id: PeerId) -> Result<(), WifiAwareError> {
        let peer_id_str = peer_id.to_string();

        self.bridge.close_data_path(&peer_id_str).await?;

        self.data_paths.write().remove(&peer_id_str);

        info!("Closed WiFi Aware data path to peer {}", peer_id_str);
        Ok(())
    }

    /// Get information about an active data path
    pub fn get_data_path(&self, peer_id: &PeerId) -> Option<DataPathInfo> {
        self.data_paths.read().get(&peer_id.to_string()).cloned()
    }

    /// Extract the advertised listen port from a discovered peer's service_info.
    pub fn get_peer_listen_port(&self, peer_id: &PeerId) -> Option<u16> {
        let discovered = self.discovered_peers.read();
        let peer = discovered.get(&peer_id.to_string())?;
        decode_port_tlv(&peer.service_info)
    }

    /// Get all active data paths
    pub fn get_active_data_paths(&self) -> Vec<DataPathInfo> {
        self.data_paths.read().values().cloned().collect()
    }

    /// Get discovered peers
    pub fn get_discovered_peers(&self) -> Vec<DiscoveredPeer> {
        self.discovered_peers.read().values().cloned().collect()
    }

    /// Register a discovered peer
    pub fn register_peer(&self, peer: DiscoveredPeer) {
        self.discovered_peers
            .write()
            .insert(peer.peer_id.to_string(), peer);
    }

    /// Add a discovered peer from a platform callback.
    /// This is the production wiring for peer discovery events — called
    /// when the platform bridge fires a service discovery event.
    /// Converts the raw callback data into a `DiscoveredPeer` and registers it.
    pub fn add_discovered_peer(&self, peer_id_str: String, service_info: Vec<u8>, rssi: i32) {
        let peer_id: PeerId = match peer_id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                warn!("Failed to parse discovered peer ID: {}", peer_id_str);
                return;
            }
        };
        let peer = DiscoveredPeer {
            peer_id,
            service_info,
            rssi,
        };
        self.register_peer(peer);
        info!("Discovered WiFi Aware peer: {} (RSSI: {})", peer_id, rssi);
    }

    /// Wire the platform bridge discovery callback to automatically register
    /// discovered peers. Call this after `initialize()` to start receiving
    /// discovery events.
    pub fn wire_discovery_callback(&self) {
        let discovered_peers = self.discovered_peers.clone();
        self.bridge.set_on_service_discovered(Box::new(
            move |peer_id_str: String, service_info: Vec<u8>, rssi: i32| {
                let peer_id: PeerId = match peer_id_str.parse() {
                    Ok(id) => id,
                    Err(_) => {
                        warn!("Failed to parse discovered peer ID in callback");
                        return;
                    }
                };
                let peer = DiscoveredPeer {
                    peer_id,
                    service_info,
                    rssi,
                };
                discovered_peers.write().insert(peer_id.to_string(), peer);
                info!(
                    "Auto-discovered WiFi Aware peer: {} (RSSI: {})",
                    peer_id, rssi
                );
            },
        ));
    }

    /// Shutdown WiFi Aware transport
    pub async fn shutdown(&self) -> Result<(), WifiAwareError> {
        // Close all data paths
        let paths: Vec<_> = self.data_paths.read().values().map(|p| p.peer_id).collect();

        for peer_id in paths {
            let _ = self.close_data_path(peer_id).await;
        }

        // Unpublish and unsubscribe
        let _ = self.bridge.unpublish_service().await;
        let _ = self.bridge.unsubscribe_from_services().await;

        *self.state.write() = WifiAwareState::Available;
        info!("WiFi Aware transport shutdown complete");
        Ok(())
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Estimate bandwidth from WiFi Aware signal strength (RSSI)
/// RSSI ranges from -40 (excellent) to -120 (poor) dBm
fn estimate_bandwidth_from_rssi(rssi: i32) -> u64 {
    // Simple linear model: higher RSSI = higher bandwidth
    // -40 dBm → ~100 Mbps
    // -80 dBm → ~10 Mbps
    // -120 dBm → ~1 Mbps
    const MAX_BANDWIDTH: i32 = 100_000_000; // 100 Mbps
    const MIN_BANDWIDTH: i32 = 1_000_000; // 1 Mbps
    const RSSI_EXCELLENT: i32 = -40;
    const RSSI_POOR: i32 = -120;

    let rssi = rssi.clamp(RSSI_POOR, RSSI_EXCELLENT);
    let ratio = (rssi - RSSI_POOR) as f64 / (RSSI_EXCELLENT - RSSI_POOR) as f64;
    let bandwidth = MIN_BANDWIDTH as f64 + (MAX_BANDWIDTH - MIN_BANDWIDTH) as f64 * ratio;

    bandwidth as u64
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_port_tlv_roundtrip() {
        let port = 5000u16;
        let encoded = encode_port_tlv(port);
        assert_eq!(encoded.len(), 4);
        assert_eq!(encoded[0], TLV_TYPE_PORT);
        assert_eq!(encoded[1], 2);
        assert_eq!(decode_port_tlv(&encoded), Some(port));
    }

    #[test]
    fn test_decode_port_tlv_none_when_absent() {
        let info = vec![0x02, 0x01, 0x42];
        assert_eq!(decode_port_tlv(&info), None);
    }

    #[test]
    fn test_decode_port_tlv_with_multiple_tlvs() {
        let port = 5000u16;
        let mut info = encode_port_tlv(port);
        info.extend_from_slice(&[0x02, 0x01, 0x42]);
        assert_eq!(decode_port_tlv(&info), Some(port));
    }

    #[test]
    fn test_decode_port_tlv_truncated() {
        let info = vec![TLV_TYPE_PORT, 0x02, 0x12];
        assert_eq!(decode_port_tlv(&info), None);
    }

    #[test]
    fn test_decode_port_tlv_rejects_duplicates() {
        let mut info = encode_port_tlv(5000);
        info.extend_from_slice(&encode_port_tlv(6000));
        assert_eq!(decode_port_tlv(&info), None);
    }

    #[tokio::test]
    async fn test_wifi_aware_initialization() {
        let bridge = Arc::new(MockWifiAwareBridge::new(true));
        let transport = WifiAwareTransport::new(WifiAwareConfig::default(), bridge)
            .expect("Failed to create transport");

        assert_eq!(transport.get_state(), WifiAwareState::Available);
        assert!(transport.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_wifi_aware_unavailable() {
        let bridge = Arc::new(MockWifiAwareBridge::new(false));
        let transport = WifiAwareTransport::new(WifiAwareConfig::default(), bridge)
            .expect("Failed to create transport");

        assert!(transport.initialize().await.is_err());
        assert_eq!(transport.get_state(), WifiAwareState::Unavailable);
    }

    #[tokio::test]
    async fn test_publish_service() {
        let bridge = Arc::new(MockWifiAwareBridge::new(true));
        let transport = WifiAwareTransport::new(WifiAwareConfig::default(), bridge)
            .expect("Failed to create transport");

        transport.initialize().await.unwrap();
        assert!(transport.publish_service().await.is_ok());
        assert_eq!(transport.get_state(), WifiAwareState::Publishing);
    }

    #[tokio::test]
    async fn test_subscribe_service() {
        let bridge = Arc::new(MockWifiAwareBridge::new(true));
        let transport = WifiAwareTransport::new(WifiAwareConfig::default(), bridge)
            .expect("Failed to create transport");

        transport.initialize().await.unwrap();
        assert!(transport.subscribe().await.is_ok());
        assert_eq!(transport.get_state(), WifiAwareState::Subscribing);
    }

    #[tokio::test]
    async fn test_publish_disabled() {
        let mut config = WifiAwareConfig::default();
        config.publish_enabled = false;

        let bridge = Arc::new(MockWifiAwareBridge::new(true));
        let transport =
            WifiAwareTransport::new(config, bridge).expect("Failed to create transport");

        transport.initialize().await.unwrap();
        assert!(transport.publish_service().await.is_err());
    }

    #[tokio::test]
    async fn test_invalid_config() {
        let mut config = WifiAwareConfig::default();
        config.service_name.clear();

        let bridge = Arc::new(MockWifiAwareBridge::new(true));
        assert!(WifiAwareTransport::new(config, bridge).is_err());
    }

    #[tokio::test]
    async fn test_create_data_path() {
        let bridge = Arc::new(MockWifiAwareBridge::new(true));
        let transport = WifiAwareTransport::new(WifiAwareConfig::default(), bridge)
            .expect("Failed to create transport");

        transport.initialize().await.unwrap();

        let peer_id = PeerId::random();
        let peer = DiscoveredPeer {
            peer_id,
            service_info: vec![1, 2, 3],
            rssi: -60,
        };
        transport.register_peer(peer);

        let pmk = [0u8; 32];
        let result = transport.create_data_path(peer_id, &pmk).await;

        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path.peer_id, peer_id);
        assert!(path.bandwidth_estimate > 0);
    }

    #[tokio::test]
    async fn test_data_path_not_found() {
        let bridge = Arc::new(MockWifiAwareBridge::new(true));
        let transport = WifiAwareTransport::new(WifiAwareConfig::default(), bridge)
            .expect("Failed to create transport");

        transport.initialize().await.unwrap();

        let peer_id = PeerId::random();
        let pmk = [0u8; 32];
        let result = transport.create_data_path(peer_id, &pmk).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_close_data_path() {
        let bridge = Arc::new(MockWifiAwareBridge::new(true));
        let transport = WifiAwareTransport::new(WifiAwareConfig::default(), bridge)
            .expect("Failed to create transport");

        transport.initialize().await.unwrap();

        let peer_id = PeerId::random();
        let peer = DiscoveredPeer {
            peer_id,
            service_info: vec![],
            rssi: -70,
        };
        transport.register_peer(peer);

        let pmk = [0u8; 32];
        transport.create_data_path(peer_id, &pmk).await.unwrap();

        assert!(transport.close_data_path(peer_id).await.is_ok());
        assert!(transport.get_data_path(&peer_id).is_none());
    }

    #[tokio::test]
    async fn test_get_active_data_paths() {
        let bridge = Arc::new(MockWifiAwareBridge::new(true));
        let transport = WifiAwareTransport::new(WifiAwareConfig::default(), bridge)
            .expect("Failed to create transport");

        transport.initialize().await.unwrap();

        let peer_id1 = PeerId::random();
        let peer_id2 = PeerId::random();

        transport.register_peer(DiscoveredPeer {
            peer_id: peer_id1,
            service_info: vec![],
            rssi: -60,
        });

        transport.register_peer(DiscoveredPeer {
            peer_id: peer_id2,
            service_info: vec![],
            rssi: -70,
        });

        let pmk = [0u8; 32];
        let _ = transport.create_data_path(peer_id1, &pmk).await;
        let _ = transport.create_data_path(peer_id2, &pmk).await;

        let paths = transport.get_active_data_paths();
        assert_eq!(paths.len(), 2);
    }

    #[tokio::test]
    async fn test_max_data_paths_limit() {
        let mut config = WifiAwareConfig::default();
        config.max_data_paths = 2;

        let bridge = Arc::new(MockWifiAwareBridge::new(true));
        let transport =
            WifiAwareTransport::new(config, bridge).expect("Failed to create transport");

        transport.initialize().await.unwrap();

        let pmk = [0u8; 32];
        let mut peer_ids = Vec::new();

        for i in 0..3 {
            let peer_id = PeerId::random();
            peer_ids.push(peer_id);
            transport.register_peer(DiscoveredPeer {
                peer_id,
                service_info: vec![i],
                rssi: -60,
            });
        }

        assert!(transport.create_data_path(peer_ids[0], &pmk).await.is_ok());
        assert!(transport.create_data_path(peer_ids[1], &pmk).await.is_ok());
        assert!(transport.create_data_path(peer_ids[2], &pmk).await.is_err());
    }

    #[tokio::test]
    async fn test_shutdown() {
        let bridge = Arc::new(MockWifiAwareBridge::new(true));
        let transport = WifiAwareTransport::new(WifiAwareConfig::default(), bridge)
            .expect("Failed to create transport");

        transport.initialize().await.unwrap();
        assert!(transport.shutdown().await.is_ok());
        assert_eq!(transport.get_state(), WifiAwareState::Available);
    }

    #[test]
    fn test_bandwidth_estimation() {
        // Test excellent signal
        let bandwidth_excellent = estimate_bandwidth_from_rssi(-40);
        assert!(bandwidth_excellent > 50_000_000); // > 50 Mbps

        // Test poor signal
        let bandwidth_poor = estimate_bandwidth_from_rssi(-120);
        assert!(bandwidth_poor < 10_000_000); // < 10 Mbps

        // Test intermediate signal
        let bandwidth_mid = estimate_bandwidth_from_rssi(-80);
        assert!(bandwidth_mid > 1_000_000); // > 1 Mbps
        assert!(bandwidth_mid < 100_000_000); // < 100 Mbps
    }

    #[test]
    fn test_get_discovered_peers() {
        let bridge = Arc::new(MockWifiAwareBridge::new(true));
        let transport = WifiAwareTransport::new(WifiAwareConfig::default(), bridge)
            .expect("Failed to create transport");

        let peer1 = DiscoveredPeer {
            peer_id: PeerId::random(),
            service_info: vec![1],
            rssi: -60,
        };
        let peer2 = DiscoveredPeer {
            peer_id: PeerId::random(),
            service_info: vec![2],
            rssi: -70,
        };

        transport.register_peer(peer1);
        transport.register_peer(peer2);

        let peers = transport.get_discovered_peers();
        assert_eq!(peers.len(), 2);
    }

    #[test]
    fn test_config_validation() {
        let config = WifiAwareConfig {
            service_name: String::new(),
            ..Default::default()
        };

        let bridge = Arc::new(MockWifiAwareBridge::new(true));
        assert!(WifiAwareTransport::new(config, bridge).is_err());
    }
}
