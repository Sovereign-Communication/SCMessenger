use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use libp2p::PeerId;
use parking_lot::RwLock;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Clone, Error)]
pub enum WifiDirectError {
    #[error("WiFi Direct unavailable on this device")]
    Unavailable,
    #[error("Peer discovery failed: {0}")]
    DiscoveryFailed(String),
    #[error("Group creation failed: {0}")]
    GroupFailed(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Platform bridge error: {0}")]
    PlatformError(String),
    #[error("State transition error: {0}")]
    InvalidStateTransition(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiDirectState {
    Unavailable,
    Idle,
    Discovering,
    Connecting,
    GroupOwner,
    GroupClient,
}

#[derive(Debug, Clone)]
pub struct WifiDirectPeer {
    pub peer_id: PeerId,
    pub device_name: String,
    pub device_address: String,
    pub rssi: i32,
}

#[derive(Debug, Clone)]
pub struct GroupInfo {
    pub group_owner: bool,
    pub group_owner_ip: Option<String>,
    pub client_ips: Vec<String>,
    pub interface_name: String,
    pub port: Option<u16>,
}

/// Android's `WifiP2pConfig.groupOwnerIntent` ranges 0-15; a device more likely
/// to stay powered and near mains (charging or with plenty of battery) should
/// bid higher so it wins group-owner negotiation and becomes the relay point.
pub const WIFI_DIRECT_GO_INTENT_PREFERRED: i32 = 7;
pub const WIFI_DIRECT_GO_INTENT_CLIENT: i32 = 0;

/// Mirrors the Kotlin-side decision in `WifiDirectTransport.kt`: prefer to be
/// group owner when charging or above 50% battery, otherwise prefer client.
pub fn compute_group_owner_intent(is_charging: bool, battery_pct: u8) -> i32 {
    if is_charging || battery_pct > 50 {
        WIFI_DIRECT_GO_INTENT_PREFERRED
    } else {
        WIFI_DIRECT_GO_INTENT_CLIENT
    }
}

#[async_trait]
pub trait WifiDirectPlatformBridge: Send + Sync {
    async fn is_available(&self) -> Result<bool, WifiDirectError>;
    async fn discover_peers(&self) -> Result<(), WifiDirectError>;
    async fn stop_discovery(&self) -> Result<(), WifiDirectError>;
    async fn connect(&self, device_address: &str) -> Result<(), WifiDirectError>;
    async fn create_group(&self, group_name: &str) -> Result<(), WifiDirectError>;
    async fn remove_group(&self) -> Result<(), WifiDirectError>;
    fn set_on_peers_changed(&self, callback: Box<dyn Fn(Vec<WifiDirectPeer>) + Send + Sync>);
    fn set_on_connection_info(&self, callback: Box<dyn Fn(GroupInfo) + Send + Sync>);
    fn set_on_message_received(&self, callback: Box<dyn Fn(String, Vec<u8>) + Send + Sync>);
}

#[cfg(not(target_arch = "wasm32"))]
type PeersChangedCallback = Box<dyn Fn(Vec<WifiDirectPeer>) + Send + Sync>;
#[cfg(not(target_arch = "wasm32"))]
type ConnectionInfoCallback = Box<dyn Fn(GroupInfo) + Send + Sync>;

#[cfg(not(target_arch = "wasm32"))]
pub struct PlatformWifiDirectBridge {
    platform_bridge: std::sync::Arc<parking_lot::Mutex<Option<Box<dyn crate::PlatformBridge>>>>,
    discovered_peers: Arc<parking_lot::Mutex<HashMap<String, WifiDirectPeer>>>,
    group_info: Arc<parking_lot::Mutex<Option<GroupInfo>>>,
    on_peers_changed: Arc<parking_lot::Mutex<Option<PeersChangedCallback>>>,
    on_connection_info: Arc<parking_lot::Mutex<Option<ConnectionInfoCallback>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl PlatformWifiDirectBridge {
    pub fn new_platform_ref(
        platform_bridge: std::sync::Arc<parking_lot::Mutex<Option<Box<dyn crate::PlatformBridge>>>>,
    ) -> Self {
        Self {
            platform_bridge,
            discovered_peers: Arc::new(parking_lot::Mutex::new(HashMap::new())),
            group_info: Arc::new(parking_lot::Mutex::new(None)),
            on_peers_changed: Arc::new(parking_lot::Mutex::new(None)),
            on_connection_info: Arc::new(parking_lot::Mutex::new(None)),
        }
    }

    fn with_platform<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&dyn crate::PlatformBridge) -> R,
    {
        self.platform_bridge.lock().as_ref().map(|b| f(b.as_ref()))
    }

    pub fn handle_peers_changed(&self, peers: Vec<WifiDirectPeer>) {
        {
            let mut map = self.discovered_peers.lock();
            map.clear();
            for peer in peers.clone() {
                map.insert(peer.device_address.clone(), peer);
            }
        }
        if let Some(cb) = self.on_peers_changed.lock().as_ref() {
            cb(peers);
        }
    }

    pub fn handle_connection_info(&self, info: GroupInfo) {
        *self.group_info.lock() = Some(info.clone());
        if let Some(cb) = self.on_connection_info.lock().as_ref() {
            cb(info);
        }
    }

    pub fn get_group_info(&self) -> Option<GroupInfo> {
        self.group_info.lock().clone()
    }
}

#[async_trait]
#[cfg(not(target_arch = "wasm32"))]
impl WifiDirectPlatformBridge for PlatformWifiDirectBridge {
    async fn is_available(&self) -> Result<bool, WifiDirectError> {
        Ok(self.with_platform(|_| true).unwrap_or(false))
    }

    async fn discover_peers(&self) -> Result<(), WifiDirectError> {
        let ok = self
            .with_platform(|b| b.wifi_direct_discover_peers())
            .unwrap_or(false);
        if ok {
            Ok(())
        } else {
            Err(WifiDirectError::DiscoveryFailed(
                "Platform rejected peer discovery".into(),
            ))
        }
    }

    async fn stop_discovery(&self) -> Result<(), WifiDirectError> {
        if let Some(b) = self.platform_bridge.lock().as_ref() {
            b.wifi_direct_stop_discovery();
        }
        Ok(())
    }

    async fn connect(&self, device_address: &str) -> Result<(), WifiDirectError> {
        let ok = self
            .with_platform(|b| b.wifi_direct_connect(device_address.to_string()))
            .unwrap_or(false);
        if ok {
            Ok(())
        } else {
            Err(WifiDirectError::ConnectionFailed(
                "Platform rejected connection".into(),
            ))
        }
    }

    async fn create_group(&self, group_name: &str) -> Result<(), WifiDirectError> {
        let ok = self
            .with_platform(|b| b.wifi_direct_create_group(group_name.to_string()))
            .unwrap_or(false);
        if ok {
            Ok(())
        } else {
            Err(WifiDirectError::GroupFailed(
                "Platform rejected group creation".into(),
            ))
        }
    }

    async fn remove_group(&self) -> Result<(), WifiDirectError> {
        if let Some(b) = self.platform_bridge.lock().as_ref() {
            b.wifi_direct_remove_group();
        }
        Ok(())
    }

    fn set_on_peers_changed(&self, callback: Box<dyn Fn(Vec<WifiDirectPeer>) + Send + Sync>) {
        *self.on_peers_changed.lock() = Some(callback);
    }
    fn set_on_connection_info(&self, callback: Box<dyn Fn(GroupInfo) + Send + Sync>) {
        *self.on_connection_info.lock() = Some(callback);
    }
    // PERIMETER-ALLOW-UNDERSCORE: WiFi Direct data does not arrive via a
    // discrete message callback on this bridge -- the GO/client link is a
    // direct TCP dial instead (see HANDOFF/plans/P1-15_transport_matrix_audit.md
    // and HANDOFF/plans/P1-17_windows_wifi_direct_design.md, both already
    // documenting this as a deliberate no-op, not a gap). `PlatformWifiDirectBridge`
    // has no `on_message_received` field to store it in.
    fn set_on_message_received(&self, _callback: Box<dyn Fn(String, Vec<u8>) + Send + Sync>) {}
}

pub struct WifiDirectTransport {
    state: Arc<RwLock<WifiDirectState>>,
    bridge: Arc<dyn WifiDirectPlatformBridge>,
    discovered_peers: Arc<RwLock<HashMap<String, WifiDirectPeer>>>,
    group_info: Arc<RwLock<Option<GroupInfo>>>,
}

impl WifiDirectTransport {
    pub fn new(bridge: Arc<dyn WifiDirectPlatformBridge>) -> Self {
        Self {
            state: Arc::new(RwLock::new(WifiDirectState::Idle)),
            bridge,
            discovered_peers: Arc::new(RwLock::new(HashMap::new())),
            group_info: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get_state(&self) -> WifiDirectState {
        *self.state.read()
    }

    pub async fn initialize(&self) -> Result<(), WifiDirectError> {
        let available = self.bridge.is_available().await?;
        if !available {
            *self.state.write() = WifiDirectState::Unavailable;
            return Err(WifiDirectError::Unavailable);
        }
        info!("WiFi Direct initialized and available");
        Ok(())
    }

    pub async fn start_discovery(&self) -> Result<(), WifiDirectError> {
        if self.get_state() == WifiDirectState::Unavailable {
            return Err(WifiDirectError::Unavailable);
        }
        self.bridge.discover_peers().await?;
        *self.state.write() = WifiDirectState::Discovering;
        info!("WiFi Direct peer discovery started");
        Ok(())
    }

    pub async fn stop_discovery(&self) -> Result<(), WifiDirectError> {
        self.bridge.stop_discovery().await?;
        if self.get_state() == WifiDirectState::Discovering {
            *self.state.write() = WifiDirectState::Idle;
        }
        Ok(())
    }

    pub async fn connect_to_peer(&self, device_address: &str) -> Result<(), WifiDirectError> {
        if self.get_state() == WifiDirectState::Unavailable {
            return Err(WifiDirectError::Unavailable);
        }
        self.bridge.connect(device_address).await?;
        *self.state.write() = WifiDirectState::Connecting;
        info!("WiFi Direct connecting to {}", device_address);
        Ok(())
    }

    pub async fn create_group(&self, group_name: &str) -> Result<(), WifiDirectError> {
        if self.get_state() == WifiDirectState::Unavailable {
            return Err(WifiDirectError::Unavailable);
        }
        self.bridge.create_group(group_name).await?;
        *self.state.write() = WifiDirectState::GroupOwner;
        info!("WiFi Direct group created: {}", group_name);
        Ok(())
    }

    pub async fn remove_group(&self) -> Result<(), WifiDirectError> {
        self.bridge.remove_group().await?;
        *self.state.write() = WifiDirectState::Idle;
        *self.group_info.write() = None;
        info!("WiFi Direct group removed");
        Ok(())
    }

    pub fn register_peer(&self, peer: WifiDirectPeer) {
        self.discovered_peers
            .write()
            .insert(peer.device_address.clone(), peer);
    }

    pub fn get_discovered_peers(&self) -> Vec<WifiDirectPeer> {
        self.discovered_peers.read().values().cloned().collect()
    }

    pub fn set_group_info(&self, info: GroupInfo) {
        if info.group_owner {
            *self.state.write() = WifiDirectState::GroupOwner;
        } else {
            *self.state.write() = WifiDirectState::GroupClient;
        }
        *self.group_info.write() = Some(info);
    }

    pub fn get_group_info(&self) -> Option<GroupInfo> {
        self.group_info.read().clone()
    }

    pub fn wire_callbacks(&self) {
        let discovered_peers = self.discovered_peers.clone();
        self.bridge
            .set_on_peers_changed(Box::new(move |peers: Vec<WifiDirectPeer>| {
                let mut map = discovered_peers.write();
                map.clear();
                for peer in peers {
                    map.insert(peer.device_address.clone(), peer);
                }
            }));

        let group_info = self.group_info.clone();
        let state = self.state.clone();
        self.bridge
            .set_on_connection_info(Box::new(move |info: GroupInfo| {
                if info.group_owner {
                    *state.write() = WifiDirectState::GroupOwner;
                } else {
                    *state.write() = WifiDirectState::GroupClient;
                }
                *group_info.write() = Some(info);
            }));
    }

    pub async fn shutdown(&self) -> Result<(), WifiDirectError> {
        let _ = self.remove_group().await;
        let _ = self.stop_discovery().await;
        *self.state.write() = WifiDirectState::Idle;
        info!("WiFi Direct transport shutdown");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockWifiDirectBridge {
        available: bool,
    }

    impl MockWifiDirectBridge {
        fn new(available: bool) -> Self {
            Self { available }
        }
    }

    #[async_trait]
    impl WifiDirectPlatformBridge for MockWifiDirectBridge {
        async fn is_available(&self) -> Result<bool, WifiDirectError> {
            Ok(self.available)
        }
        async fn discover_peers(&self) -> Result<(), WifiDirectError> {
            if self.available {
                Ok(())
            } else {
                Err(WifiDirectError::Unavailable)
            }
        }
        async fn stop_discovery(&self) -> Result<(), WifiDirectError> {
            Ok(())
        }
        // PERIMETER-ALLOW-UNDERSCORE: test-only mock of WifiDirectPlatformBridge;
        // MockWifiDirectBridge is a fixed available/unavailable stub and does
        // not model per-call arguments.
        async fn connect(&self, _device_address: &str) -> Result<(), WifiDirectError> {
            if self.available {
                Ok(())
            } else {
                Err(WifiDirectError::Unavailable)
            }
        }
        // PERIMETER-ALLOW-UNDERSCORE: test-only mock, see `connect` above.
        async fn create_group(&self, _group_name: &str) -> Result<(), WifiDirectError> {
            if self.available {
                Ok(())
            } else {
                Err(WifiDirectError::Unavailable)
            }
        }
        async fn remove_group(&self) -> Result<(), WifiDirectError> {
            Ok(())
        }
        // PERIMETER-ALLOW-UNDERSCORE: test-only mock, see `connect` above.
        fn set_on_peers_changed(&self, _callback: Box<dyn Fn(Vec<WifiDirectPeer>) + Send + Sync>) {}
        // PERIMETER-ALLOW-UNDERSCORE: test-only mock, see `connect` above.
        fn set_on_connection_info(&self, _callback: Box<dyn Fn(GroupInfo) + Send + Sync>) {}
        // PERIMETER-ALLOW-UNDERSCORE: test-only mock, see `connect` above.
        fn set_on_message_received(&self, _callback: Box<dyn Fn(String, Vec<u8>) + Send + Sync>) {}
    }

    #[tokio::test]
    async fn test_wifi_direct_initialization() {
        let bridge = Arc::new(MockWifiDirectBridge::new(true));
        let transport = WifiDirectTransport::new(bridge);
        assert_eq!(transport.get_state(), WifiDirectState::Idle);
        assert!(transport.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_wifi_direct_unavailable() {
        let bridge = Arc::new(MockWifiDirectBridge::new(false));
        let transport = WifiDirectTransport::new(bridge);
        assert!(transport.initialize().await.is_err());
        assert_eq!(transport.get_state(), WifiDirectState::Unavailable);
    }

    #[tokio::test]
    async fn test_wifi_direct_discovery() {
        let bridge = Arc::new(MockWifiDirectBridge::new(true));
        let transport = WifiDirectTransport::new(bridge);
        transport.initialize().await.unwrap();
        assert!(transport.start_discovery().await.is_ok());
        assert_eq!(transport.get_state(), WifiDirectState::Discovering);
        assert!(transport.stop_discovery().await.is_ok());
        assert_eq!(transport.get_state(), WifiDirectState::Idle);
    }

    #[tokio::test]
    async fn test_wifi_direct_create_group() {
        let bridge = Arc::new(MockWifiDirectBridge::new(true));
        let transport = WifiDirectTransport::new(bridge);
        transport.initialize().await.unwrap();
        assert!(transport.create_group("SCMeshGroup").await.is_ok());
        assert_eq!(transport.get_state(), WifiDirectState::GroupOwner);
        assert!(transport.remove_group().await.is_ok());
        assert_eq!(transport.get_state(), WifiDirectState::Idle);
    }

    #[test]
    fn test_group_owner_intent_charging_prefers_owner() {
        // Charging, regardless of battery level -> bid to be group owner.
        assert_eq!(
            compute_group_owner_intent(true, 10),
            WIFI_DIRECT_GO_INTENT_PREFERRED
        );
    }

    #[test]
    fn test_group_owner_intent_high_battery_prefers_owner() {
        // Not charging but well-charged -> still bid to be group owner.
        assert_eq!(
            compute_group_owner_intent(false, 51),
            WIFI_DIRECT_GO_INTENT_PREFERRED
        );
    }

    #[test]
    fn test_group_owner_intent_low_battery_prefers_client() {
        // Not charging and battery at/below threshold -> prefer client role.
        assert_eq!(
            compute_group_owner_intent(false, 50),
            WIFI_DIRECT_GO_INTENT_CLIENT
        );
        assert_eq!(
            compute_group_owner_intent(false, 5),
            WIFI_DIRECT_GO_INTENT_CLIENT
        );
    }
}
