#![cfg(not(target_arch = "wasm32"))]

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use libp2p::{Multiaddr, PeerId};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

use crate::settings::MeshSettings;

use crate::transport::wifi_aware::{
    WifiAwareConfig, WifiAwareError, WifiAwarePlatformBridge, WifiAwareTransport,
};
use crate::transport::wifi_direct::{PlatformWifiDirectBridge, WifiDirectTransport};
use crate::transport::SwarmHandle;

// MOBILE SERVICE
// ============================================================================

#[derive(Debug, Clone)]
pub struct MeshServiceConfig {
    pub discovery_interval_ms: u32,
    pub battery_floor_pct: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Stopping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionPathState {
    Disconnected,
    Bootstrapping,
    DirectPreferred,
    RelayFallback,
    RelayOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MotionState {
    #[default]
    Still,
    Walking,
    Running,
    Automotive,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProximityTransport {
    Ble,
    WifiAware,
    WifiDirect,
    Multipeer,
}

impl ProximityTransport {
    pub fn max_payload_size(&self) -> usize {
        match self {
            ProximityTransport::Ble => 512,
            ProximityTransport::WifiAware => 2048,
            ProximityTransport::WifiDirect => 4096,
            ProximityTransport::Multipeer => 4096,
        }
    }
}

impl fmt::Display for ProximityTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProximityTransport::Ble => write!(f, "BLE"),
            ProximityTransport::WifiAware => write!(f, "WiFiAware"),
            ProximityTransport::WifiDirect => write!(f, "WiFiDirect"),
            ProximityTransport::Multipeer => write!(f, "Multipeer"),
        }
    }
}

/// Network connectivity type reported by the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, uniffi::Enum)]
pub enum NetworkType {
    /// No connectivity.
    None,
    /// WiFi connection present.
    Wifi,
    /// Cellular data (any generation).
    Cellular,
    /// Both WiFi and cellular available.
    WifiAndCellular,
    /// Unknown / not yet reported.
    #[default]
    Unknown,
}

/// Snapshot of device state as reported by the platform layer.
///
/// This is the canonical state record stored inside `MeshService`.
/// It is richer than `DeviceProfile` (which is the UniFFI-facing input type)
/// and drives the threshold-based behavior adjustments.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct DeviceState {
    /// Battery level 0–100.
    pub battery_level: u8,
    /// True while the device is plugged in / wirelessly charging.
    pub is_charging: bool,
    /// Active network type.
    pub network_type: NetworkType,
    /// Motion context reported by the platform accelerometer/activity API.
    pub motion_state: MotionState,
}

impl DeviceState {
    /// Construct from the UniFFI-facing `DeviceProfile`.
    pub fn from_profile(profile: &DeviceProfile) -> Self {
        let network_type = match (profile.has_wifi, profile.is_charging) {
            (true, _) => NetworkType::Wifi,
            (false, _) => NetworkType::Cellular,
        };
        Self {
            battery_level: profile.battery_pct,
            is_charging: profile.is_charging,
            network_type,
            motion_state: profile.motion_state,
        }
    }
}

/// Recommended behavior adjustments derived from the current `DeviceState`.
///
/// Callers (swarm thread, scan schedulers, relay logic) should query
/// `MeshService::recommended_behavior()` and honour these hints.
#[derive(Debug, Clone, uniffi::Record)]
pub struct BehaviorAdjustment {
    /// Suggested BLE / WiFi-Aware scan interval in milliseconds.
    /// Higher value = less frequent scanning = less battery drain.
    pub scan_interval_ms: u32,
    /// Whether relay duty should be active at all.
    pub relay_enabled: bool,
    /// Relay message budget (messages per hour, 0 means relay disabled).
    pub relay_budget: u32,
    /// True when the device should operate in the absolute minimum mode
    /// (battery critically low and not charging).
    pub minimal_operation: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ServiceStats {
    pub peers_discovered: u32,
    pub messages_relayed: u32,
    pub bytes_transferred: u64,
    pub uptime_secs: u64,
}

/// Mobile mesh service wrapper integrating IronCore with mobile lifecycle.
///
/// Uses `parking_lot::Mutex` throughout — unlike `std::sync::Mutex` it never
/// poisons on panic, eliminating the PoisonError cascade that previously
/// caused a fatal crash when `start_swarm` panicked while holding `core`.
#[derive(uniffi::Object)]
pub struct MeshService {
    _config: Mutex<MeshServiceConfig>,
    state: Mutex<ServiceState>,
    stats: Arc<Mutex<ServiceStats>>,
    pub(crate) nearby_ble_peers: Arc<Mutex<HashSet<String>>>,
    core: std::sync::Arc<Mutex<Option<std::sync::Arc<crate::IronCore>>>>,
    platform_bridge: std::sync::Arc<Mutex<Option<Box<dyn PlatformBridge>>>>,
    storage_path: Option<String>,
    log_directory: Option<String>,
    swarm_bridge: std::sync::Arc<SwarmBridge>,
    nat_status: std::sync::Arc<Mutex<String>>,
    relay_budget: std::sync::Arc<Mutex<u32>>,
    swarm_headless_mode: std::sync::Arc<Mutex<Option<bool>>>,
    current_device_profile: Mutex<Option<DeviceProfile>>,
    device_state: RwLock<Option<DeviceState>>,
    auto_adjust: Arc<AutoAdjustEngine>,
    wifi_aware_bridge: Arc<Mutex<Option<Arc<PlatformWifiAwareBridge>>>>,
    wifi_direct_bridge: Arc<Mutex<Option<Arc<PlatformWifiDirectBridge>>>>,
    wifi_aware_transport: Arc<Mutex<Option<Arc<crate::transport::wifi_aware::WifiAwareTransport>>>>,
    wifi_direct_transport:
        Arc<Mutex<Option<Arc<crate::transport::wifi_direct::WifiDirectTransport>>>>,
    /// Platform-provided delegate for decentralized protocol events (Phase 4).
    external_delegate: Arc<Mutex<Option<Box<dyn crate::CoreDelegate>>>>,
}

#[uniffi::export]
impl MeshService {
    #[uniffi::constructor]
    pub fn new(config: MeshServiceConfig) -> Self {
        Self {
            _config: Mutex::new(config),
            state: Mutex::new(ServiceState::Stopped),
            stats: Arc::new(Mutex::new(ServiceStats::default())),
            core: std::sync::Arc::new(Mutex::new(None)),
            platform_bridge: std::sync::Arc::new(Mutex::new(None)),
            storage_path: None,
            log_directory: None,
            swarm_bridge: std::sync::Arc::new(SwarmBridge::new()),
            nat_status: std::sync::Arc::new(Mutex::new("unknown".to_string())),
            relay_budget: std::sync::Arc::new(Mutex::new(200)),
            swarm_headless_mode: std::sync::Arc::new(Mutex::new(None)),
            current_device_profile: Mutex::new(None),
            device_state: RwLock::new(None),
            auto_adjust: Arc::new(AutoAdjustEngine::new()),
            nearby_ble_peers: Arc::new(Mutex::new(HashSet::new())),
            external_delegate: Arc::new(Mutex::new(None)),
            wifi_aware_bridge: Arc::new(Mutex::new(None)),
            wifi_direct_bridge: Arc::new(Mutex::new(None)),
            wifi_aware_transport: Arc::new(Mutex::new(None)),
            wifi_direct_transport: Arc::new(Mutex::new(None)),
        }
    }

    /// Create MeshService with persistent storage
    #[uniffi::constructor]
    pub fn with_storage(config: MeshServiceConfig, storage_path: String) -> Self {
        Self {
            _config: Mutex::new(config),
            state: Mutex::new(ServiceState::Stopped),
            stats: Arc::new(Mutex::new(ServiceStats::default())),
            core: std::sync::Arc::new(Mutex::new(None)),
            platform_bridge: std::sync::Arc::new(Mutex::new(None)),
            storage_path: Some(storage_path),
            log_directory: None,
            swarm_bridge: std::sync::Arc::new(SwarmBridge::new()),
            nat_status: std::sync::Arc::new(Mutex::new("unknown".to_string())),
            relay_budget: std::sync::Arc::new(Mutex::new(200)),
            swarm_headless_mode: std::sync::Arc::new(Mutex::new(None)),
            current_device_profile: Mutex::new(None),
            device_state: RwLock::new(None),
            auto_adjust: Arc::new(AutoAdjustEngine::new()),
            nearby_ble_peers: Arc::new(Mutex::new(HashSet::new())),
            external_delegate: Arc::new(Mutex::new(None)),
            wifi_aware_bridge: Arc::new(Mutex::new(None)),
            wifi_direct_bridge: Arc::new(Mutex::new(None)),
            wifi_aware_transport: Arc::new(Mutex::new(None)),
            wifi_direct_transport: Arc::new(Mutex::new(None)),
        }
    }

    /// Create MeshService with persistent storage and structured tracing
    #[uniffi::constructor]
    pub fn with_storage_and_logs(
        config: MeshServiceConfig,
        storage_path: String,
        log_directory: String,
    ) -> Self {
        Self {
            _config: Mutex::new(config),
            state: Mutex::new(ServiceState::Stopped),
            stats: Arc::new(Mutex::new(ServiceStats::default())),
            core: std::sync::Arc::new(Mutex::new(None)),
            platform_bridge: std::sync::Arc::new(Mutex::new(None)),
            storage_path: Some(storage_path),
            log_directory: Some(log_directory),
            swarm_bridge: std::sync::Arc::new(SwarmBridge::new()),
            nat_status: std::sync::Arc::new(Mutex::new("unknown".to_string())),
            relay_budget: std::sync::Arc::new(Mutex::new(200)),
            swarm_headless_mode: std::sync::Arc::new(Mutex::new(None)),
            current_device_profile: Mutex::new(None),
            device_state: RwLock::new(None),
            auto_adjust: Arc::new(AutoAdjustEngine::new()),
            nearby_ble_peers: Arc::new(Mutex::new(HashSet::new())),
            external_delegate: Arc::new(Mutex::new(None)),
            wifi_aware_bridge: Arc::new(Mutex::new(None)),
            wifi_direct_bridge: Arc::new(Mutex::new(None)),
            wifi_aware_transport: Arc::new(Mutex::new(None)),
            wifi_direct_transport: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(self: Arc<Self>) -> Result<(), crate::IronCoreError> {
        let mut state = self.state.lock();

        if *state == ServiceState::Running || *state == ServiceState::Starting {
            return Err(crate::IronCoreError::AlreadyRunning);
        }

        *state = ServiceState::Starting;
        drop(state);

        tracing::info!(
            "MeshService::start: storage_path={:?}, log_directory={:?}",
            self.storage_path,
            self.log_directory
        );

        // Initialize IronCore
        #[cfg(not(target_arch = "wasm32"))]
        let core = if let Some(ref log_dir) = self.log_directory {
            if let Some(ref path) = self.storage_path {
                tracing::info!("MeshService::start: Creating IronCore::with_storage_and_logs");
                let core = crate::IronCore::with_storage_and_logs(path.clone(), log_dir.clone());
                tracing::info!("MeshService::start: IronCore::with_storage_and_logs completed");
                core
            } else {
                tracing::info!("MeshService::start: Creating IronCore::new (no storage path)");
                crate::IronCore::new()
            }
        } else if let Some(ref path) = self.storage_path {
            tracing::info!(
                "MeshService::start: Creating IronCore::with_storage at {:?}",
                path
            );
            let core = crate::IronCore::with_storage(path.clone());
            tracing::info!("MeshService::start: IronCore::with_storage completed");
            core
        } else {
            tracing::info!("MeshService::start: Creating IronCore::new (no storage)");
            crate::IronCore::new()
        };

        #[cfg(target_arch = "wasm32")]
        let core = crate::IronCore::new();

        // Start the core
        core.start()?;
        let core = Arc::new(core);

        // Register this service as the core delegate for all protocol events
        core.set_delegate(Some(Box::new(MeshServiceCoreDelegate {
            service: Arc::downgrade(&self),
        })));

        // Load identity metadata into service profile
        let id_manager = core.identity_id();
        let device_id = core.device_id();

        if let (Some(id), Some(device_id)) = (id_manager, device_id) {
            let mut profile = self.current_device_profile.lock();
            *profile = Some(DeviceProfile {
                peer_id: Some(id),
                device_id: Some(device_id),
                ..DeviceProfile::default()
            });
        }

        // Store the core instance
        *self.core.lock() = Some(core.clone());

        // P1_CORE_001: Activate drift if relaying is enabled
        let budget = *self.relay_budget.lock();
        if budget > 0 {
            core.drift_activate();
        }

        // Initialize WiFi Aware and WiFi Direct transports if enabled and platform bridge is set
        if self.platform_bridge.lock().is_some() {
            let aware_bridge = Arc::new(PlatformWifiAwareBridge::new_platform_ref(
                self.platform_bridge.clone(),
            ));
            *self.wifi_aware_bridge.lock() = Some(aware_bridge.clone());
            tracing::info!("WiFi Aware bridge adapter initialized");

            let direct_bridge = Arc::new(PlatformWifiDirectBridge::new_platform_ref(
                self.platform_bridge.clone(),
            ));
            *self.wifi_direct_bridge.lock() = Some(direct_bridge.clone());
            tracing::info!("WiFi Direct bridge adapter initialized");

            // Load settings using MeshSettingsManager
            let settings = if let Some(ref path) = self.storage_path {
                let manager = MeshSettingsManager::new(path.clone());
                manager.load().unwrap_or_default()
            } else {
                MeshSettings::default()
            };

            // WiFi Aware Transport
            if settings.wifi_aware_enabled {
                let config = WifiAwareConfig {
                    publish_enabled: true,
                    subscribe_enabled: true,
                    ..Default::default()
                };
                if let Ok(transport) = WifiAwareTransport::new(config, aware_bridge) {
                    let transport = Arc::new(transport);
                    let transport_clone = transport.clone();
                    let rt = self.swarm_bridge.get_runtime_handle();
                    rt.spawn(async move {
                        if let Err(e) = transport_clone.initialize().await {
                            tracing::error!("WiFi Aware transport initialization failed: {:?}", e);
                        } else {
                            transport_clone.wire_discovery_callback();
                            if let Err(e) = transport_clone.publish_service().await {
                                tracing::error!("WiFi Aware publish failed: {:?}", e);
                            }
                            if let Err(e) = transport_clone.subscribe().await {
                                tracing::error!("WiFi Aware subscribe failed: {:?}", e);
                            }
                        }
                    });
                    *self.wifi_aware_transport.lock() = Some(transport);
                }
            }

            // WiFi Direct Transport
            if settings.wifi_direct_enabled {
                let transport = WifiDirectTransport::new(direct_bridge);
                let transport = Arc::new(transport);
                let transport_clone = transport.clone();
                let rt = self.swarm_bridge.get_runtime_handle();
                rt.spawn(async move {
                    if let Err(e) = transport_clone.initialize().await {
                        tracing::error!("WiFi Direct transport initialization failed: {:?}", e);
                    } else {
                        transport_clone.wire_callbacks();
                        if let Err(e) = transport_clone.start_discovery().await {
                            tracing::error!("WiFi Direct start discovery failed: {:?}", e);
                        }
                    }
                });
                *self.wifi_direct_transport.lock() = Some(transport);
            }
        }

        // Update state
        *self.state.lock() = ServiceState::Running;

        tracing::info!("MeshService started");
        Ok(())
    }

    /// Register an external delegate for protocol events (messages, discovery).
    pub fn set_delegate(&self, delegate: Option<Box<dyn crate::CoreDelegate>>) {
        *self.external_delegate.lock() = delegate;
    }

    pub fn stop(&self) {
        let mut state = self.state.lock();

        if *state == ServiceState::Stopped {
            return;
        }

        *state = ServiceState::Stopping;
        drop(state);

        // Stop the core and clear the reference atomically
        let core = self.core.lock().take();
        if let Some(core) = core {
            core.stop();
        }

        // Shutdown the swarm bridge gracefully
        self.swarm_bridge.shutdown_blocking();

        // Clear headless mode
        *self.swarm_headless_mode.lock() = None;

        // Update state
        *self.state.lock() = ServiceState::Stopped;

        tracing::info!("MeshService stopped");
    }

    pub fn pause(&self) {
        tracing::info!("MeshService paused (activity reduced)");
        if let Some(bridge) = self.platform_bridge.lock().as_ref() {
            bridge.on_entering_background();
        }
    }

    pub fn resume(&self) {
        tracing::info!("MeshService resumed (full activity)");
        if let Some(bridge) = self.platform_bridge.lock().as_ref() {
            bridge.on_entering_foreground();
        }
    }

    pub fn get_state(&self) -> ServiceState {
        *self.state.lock()
    }

    pub fn get_stats(&self) -> ServiceStats {
        let mut stats = self.stats.lock().clone();
        let peers = self.get_swarm_bridge().get_peers_blocking();
        stats.peers_discovered = peers.len() as u32;
        stats
    }

    pub fn reset_stats(&self) {
        *self.stats.lock() = ServiceStats::default();
        tracing::info!("MeshService stats reset");
    }

    pub fn set_platform_bridge(&self, bridge: Option<Box<dyn PlatformBridge>>) {
        *self.platform_bridge.lock() = bridge;
    }

    /// Update keepalive interval for a peer connection.
    ///
    /// Async FFI (Issue 5): exported to Kotlin as a `suspend fun`.
    pub async fn update_keepalive(
        &self,
        peer_id: String,
        interval_secs: u64,
    ) -> Result<(), crate::IronCoreError> {
        let peer_id_parsed: PeerId = peer_id
            .parse()
            .map_err(|_| crate::IronCoreError::InvalidInput)?;
        let handle = self
            .swarm_bridge
            .handle
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NetworkError)?;
        handle
            .update_keepalive(peer_id_parsed, interval_secs)
            .await
            .map_err(|_| crate::IronCoreError::NetworkError)
    }

    /// Get current NAT status string.
    pub fn get_nat_status(&self) -> String {
        self.nat_status.lock().clone()
    }

    pub fn get_connection_path_state(&self) -> ConnectionPathState {
        let peers = self.swarm_bridge.get_peers_blocking();
        let listeners = self.swarm_bridge.get_listeners_blocking();
        let nat = self.nat_status.lock().clone();

        if peers.is_empty() {
            return ConnectionPathState::Disconnected;
        }

        if !listeners.is_empty() && nat != "symmetric" {
            return ConnectionPathState::DirectPreferred;
        }

        ConnectionPathState::RelayOnly
    }

    pub fn export_diagnostics(&self) -> String {
        let stats = self.get_stats();
        let drift_state = if let Some(core) = self.core.lock().as_ref() {
            core.drift_network_state()
        } else {
            "Dormant".to_string()
        };
        let drift_store_size = if let Some(core) = self.core.lock().as_ref() {
            core.drift_store_size()
        } else {
            0
        };
        let mut payload = serde_json::Value::Object(serde_json::Map::from_iter([
            (
                "service_state".into(),
                serde_json::Value::from(format!("{:?}", self.get_state())),
            ),
            (
                "connection_path_state".into(),
                serde_json::Value::from(format!("{:?}", self.get_connection_path_state())),
            ),
            (
                "nat_status".into(),
                serde_json::Value::from(self.get_nat_status()),
            ),
            (
                "peers".into(),
                serde_json::to_value(self.swarm_bridge.get_peers_blocking())
                    .unwrap_or(serde_json::Value::Null),
            ),
            (
                "listeners".into(),
                serde_json::to_value(self.swarm_bridge.get_listeners_blocking())
                    .unwrap_or(serde_json::Value::Null),
            ),
            (
                "external_addrs".into(),
                serde_json::to_value(self.swarm_bridge.get_external_addresses_blocking())
                    .unwrap_or(serde_json::Value::Null),
            ),
            (
                "relay_budget".into(),
                serde_json::Value::from(*self.relay_budget.lock()),
            ),
            ("drift_state".into(), serde_json::Value::from(drift_state)),
            (
                "drift_store_size".into(),
                serde_json::Value::from(drift_store_size),
            ),
            (
                "timestamp_ms".into(),
                serde_json::Value::from(current_timestamp()),
            ),
        ]));
        payload["stats"] = serde_json::Value::Object(serde_json::Map::from_iter([
            (
                "peers_discovered".into(),
                serde_json::Value::from(stats.peers_discovered),
            ),
            (
                "messages_relayed".into(),
                serde_json::Value::from(stats.messages_relayed),
            ),
            (
                "bytes_transferred".into(),
                serde_json::Value::from(stats.bytes_transferred),
            ),
            (
                "uptime_secs".into(),
                serde_json::Value::from(stats.uptime_secs),
            ),
        ]));

        payload.to_string()
    }

    pub fn start_swarm(
        &self,
        listen_addr: String,
        bootstrap_addrs: Vec<String>,
    ) -> Result<(), crate::IronCoreError> {
        // Extract keys while holding the lock, then DROP the lock before any
        // runtime/thread work.  This is critical: if anything below panics
        // while the lock is held, parking_lot will NOT poison it (unlike
        // std::sync::Mutex), but releasing early is still the safest pattern.
        let (libp2p_keys, headless_mode) = self.resolve_swarm_keypair_and_mode()?;

        let has_existing_handle = self.swarm_bridge.handle.lock().is_some();
        let existing_mode = *self.swarm_headless_mode.lock();
        if has_existing_handle {
            if existing_mode == Some(headless_mode) {
                tracing::info!(
                    "Swarm already running in {} mode; skipping restart",
                    if headless_mode { "headless" } else { "full" }
                );
                return Ok(());
            }

            tracing::info!(
                "Swarm mode change requested ({} -> {}); restarting swarm",
                if existing_mode == Some(true) {
                    "headless"
                } else {
                    "full"
                },
                if headless_mode { "headless" } else { "full" }
            );
            self.swarm_bridge.shutdown_blocking();
            *self.swarm_bridge.handle.lock() = None;
            *self.swarm_headless_mode.lock() = None;
        }

        tracing::info!(
            "Starting Swarm with PeerID: {}",
            libp2p_keys.public().to_peer_id()
        );
        eprintln!(
            "=== OWN_IDENTITY: {} ===",
            libp2p_keys.public().to_peer_id()
        );

        let listen_multiaddr: Option<libp2p::Multiaddr> = if listen_addr.is_empty() {
            None
        } else {
            Some(
                listen_addr
                    .parse()
                    .map_err(|_| crate::IronCoreError::InvalidInput)?,
            )
        };
        let expected_listen_addr = listen_multiaddr.clone();

        // Parse bootstrap multiaddr strings into Multiaddr objects
        let parsed_bootstrap: Vec<libp2p::Multiaddr> = bootstrap_addrs
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();
        if !parsed_bootstrap.is_empty() {
            tracing::info!(
                "[MOBILE] Mobile bridge: {} bootstrap addrs configured",
                parsed_bootstrap.len()
            );
        }

        let swarm_bridge = self.swarm_bridge.clone();
        let core = self.core.clone();
        let relay_budget_init = self.relay_budget.clone();
        let nat_status = self.nat_status.clone();
        let swarm_mode_state = self.swarm_headless_mode.clone();
        let service_storage_path = self.storage_path.clone();
        let stats = self.stats.clone();

        // TCP-listener-zombie fix: the OS socket bind happens asynchronously
        // inside the swarm task, so returning Ok(()) here used to mean "the
        // thread was spawned", not "we are listening". This channel carries
        // the real startup outcome (first NewListenAddr, or a bind failure)
        // back to this FFI call so callers can no longer observe a
        // "Running" service with no listener.
        let (startup_tx, startup_rx) = std::sync::mpsc::sync_channel::<Result<(), String>>(1);
        let await_listener = listen_multiaddr.is_some();

        // Spawn a dedicated OS thread that owns its own Tokio runtime.
        // This is the safest approach for mobile: we cannot rely on being
        // called from a Tokio context, and we must not hold any Mutex across
        // the thread boundary.
        std::thread::Builder::new()
            .name("scm-swarm".to_string())
            .spawn(move || {
                #[cfg(not(target_arch = "wasm32"))]
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .thread_name("scm-swarm-worker")
                    .build();

                #[cfg(target_arch = "wasm32")]
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();

                match rt {
                    Ok(rt) => {
                        rt.block_on(async move {
                            let mut startup_signal = Some(startup_tx);
                            let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(100);

                            let iron_core_handle = {
                                let core_guard = core.lock();
                                core_guard.clone()
                            };

                            // Extract both the Weak<IronCore> and routing engine handle before
                            // iron_core_handle is consumed by the closure below.
                            let routing_engine_handle = iron_core_handle.as_ref()
                                .map(|c| c.routing_engine_handle())
                                .unwrap_or_else(crate::transport::swarm::default_routing_engine_handle);
                            let core_weak = iron_core_handle.map(|c| {
                                Arc::downgrade(&c)
                            });
                            let preferred_port = listen_multiaddr.as_ref().and_then(|addr| {
                                  addr.iter().find_map(|p| match p {
                                      libp2p::multiaddr::Protocol::Tcp(port) => Some(port),
                                      _ => None,
                                  })
                              });
                              let multiport_config = crate::transport::multiport::MultiPortConfig {
                                  preferred_port,
                                  ..Default::default()
                              };

                              match crate::transport::start_swarm_with_config(
                                  libp2p_keys,
                                  listen_multiaddr,
                                  event_tx,
                                  Some(multiport_config),
                                  parsed_bootstrap.clone(),
                                  service_storage_path,
                                  core_weak,
                                  headless_mode,
                                  None, // Use default discovery config (Open/mDNS enabled)
                                  routing_engine_handle,
                              )
                            .await
                            {
                                Ok(handle) => {
                                    tracing::info!("Swarm started, wiring bridge");
                                    swarm_bridge.set_handle(handle.clone());
                                    *swarm_mode_state.lock() = Some(headless_mode);
                                    if !await_listener {
                                        // No listen address requested: nothing to
                                        // wait for, report startup success now.
                                        if let Some(tx) = startup_signal.take() {
                                            let _ = tx.try_send(Ok(()));
                                        }
                                    }
                                    // Apply stored relay budget
                                    let budget = *relay_budget_init.lock();
                                    if let Err(e) = handle.set_relay_budget(budget).await {
                                        tracing::warn!(
                                            "Failed to set initial relay budget: {:?}",
                                            e
                                        );
                                    }

                                    // NAT hole-punch Priority 1: proactively dial a seed peer on
                                    // startup so an outbound NAT mapping exists before any
                                    // inbound circuit-relay traffic arrives. A failure here must
                                    // never fail service startup.
                                    //
                                    // Not gated on `parsed_bootstrap` any more: the swarm now
                                    // sources seed candidates from the connection ledger first,
                                    // so a node with an empty startup address list but a warm
                                    // ledger must still get its hole punch. `Ok(())` means an
                                    // actual connection was established, not merely a queued
                                    // dial, so the log line is no longer a false positive.
                                    //
                                    // DEADLOCK SAFETY: this MUST NOT be awaited inline. Waiting
                                    // for a real ConnectionEstablished can take until the swarm's
                                    // 10s pending-dial sweep fires. `event_rx` is a bounded
                                    // channel (capacity 100) and the swarm emits events with
                                    // awaited sends from the same select! task that owns that
                                    // sweep. Blocking here stops the drain below, the channel
                                    // fills, the swarm task blocks on send, and the sweep that
                                    // would release this reply can never run -- a permanent
                                    // startup deadlock, reachable on an ordinary LAN via mDNS
                                    // event volume. Detach it so the drain starts immediately.
                                    {
                                        let seed_handle = handle.clone();
                                        tokio::spawn(async move {
                                            match seed_handle.connect_to_seed_peers().await {
                                                Ok(()) => {
                                                    tracing::info!("Connected to seed peer")
                                                }
                                                Err(e) => tracing::warn!(
                                                    "No seed peer reachable at startup (non-fatal): {:?}",
                                                    e
                                                ),
                                            }
                                        });
                                    }
                                    while let Some(event) = event_rx.recv().await {
                                        match event {
                                            crate::transport::SwarmEvent::MessageReceived {
                                                peer_id,
                                                envelope_data,
                                            } => {
                                                let core_guard = core.lock();
                                                if let Some(core_ref) = core_guard.as_ref() {
                                                    match core_ref.receive_message(envelope_data.clone()) {
                                                        Ok(msg) => {
                                                            if msg.message_type == crate::message::MessageType::OnionRelay {
                                                                // RELAY: Forward to next hop
                                                                let next_hop_hex = msg.recipient_id.clone();
                                                                let payload = msg.payload.clone();

                                                                eprintln!("[IronCore] [RELAY] Onion relay: forwarding to {}", next_hop_hex);
                                                                if let Ok(next_hop_bytes) = hex::decode(&next_hop_hex) {
                                                                    if let Ok(libp2p_pk) = libp2p::identity::ed25519::PublicKey::try_from_bytes(&next_hop_bytes[..32]) {
                                                                        let next_peer_id = libp2p::PeerId::from_public_key(&libp2p::identity::PublicKey::from(libp2p_pk));

                                                                        let bridge_clone = swarm_bridge.clone();
                                                                        let stats_clone = stats.clone();
                                                                        let core_owned = core_ref.clone();
                                                                        let spawn_res = bridge_clone.get_runtime_handle().spawn(async move {
                                                                            // B1_CORE_ENTRY_006: Apply timing jitter to thwart correlation attacks
                                                                            let delay_ms = core_owned.relay_jitter_delay("Normal".to_string());
                                                                            if delay_ms > 0 {
                                                                                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                                                                            }
                                                                            let _ = bridge_clone.send_message(next_peer_id.to_string(), payload, None, None).await;
                                                                        });
                                                                        drop(spawn_res);

                                                                        let mut s = stats_clone.lock();
                                                                        s.messages_relayed += 1;
                                                                    }
                                                                }
                                                            } else {
                                                                tracing::info!(
                                                                    "Received message {} from {}",
                                                                    msg.id,
                                                                    peer_id
                                                                );
                                                                eprintln!(
                                                                    "[IronCore] [OK] Received message {} from {} (type={:?})",
                                                                    msg.id,
                                                                    peer_id,
                                                                    msg.message_type
                                                                );
                                                            }
                                                        }
                                                        Err(e) => {
                                                            let err_detail = format!("{:?}", e);
                                                            tracing::warn!(
                                                                "receive_message error from {}: {}",
                                                                peer_id,
                                                                err_detail
                                                            );
                                                            // CRITICAL: eprintln! is the ONLY way to surface
                                                            // errors on mobile — tracing goes to /dev/null.
                                                            eprintln!(
                                                                "[IronCore] [ERROR] receive_message FAILED from {}: {} (envelope_len={})",
                                                                peer_id,
                                                                err_detail,
                                                                envelope_data.len()
                                                            );
                                                        }
                                                    }
                                                } else {
                                                    eprintln!(
                                                        "[IronCore] [ERROR] receive_message SKIPPED from {}: core not initialized",
                                                        peer_id
                                                    );
                                                }
                                            }
                                            crate::transport::SwarmEvent::PeerDiscovered(
                                                peer_id,
                                            ) => {
                                                tracing::info!(
                                                    "Peer discovered via Swarm: {}",
                                                    peer_id
                                                );
                                                let core_guard = core.lock();
                                                if let Some(core_ref) = core_guard.as_ref() {
                                                    core_ref.notify_peer_discovered(
                                                        peer_id.to_string(),
                                                    );
                                                }
                                            }
                                            crate::transport::SwarmEvent::PeerDisconnected(
                                                peer_id,
                                            ) => {
                                                tracing::info!(
                                                    "Peer disconnected via Swarm: {}",
                                                    peer_id
                                                );
                                                let core_guard = core.lock();
                                                if let Some(core_ref) = core_guard.as_ref() {
                                                    core_ref.notify_peer_disconnected(
                                                        peer_id.to_string(),
                                                    );
                                                }
                                            }
                                            crate::transport::SwarmEvent::PeerIdentified {
                                                peer_id,
                                                public_key,
                                                agent_version,
                                                listen_addrs,
                                                ..
                                            } => {
                                                let registration_request = if headless_mode {
                                                    None
                                                } else {
                                                    let core_guard = core.lock();
                                                    core_guard
                                                        .as_ref()
                                                        .and_then(|core_ref| {
                                                            core_ref.build_registration_request().ok()
                                                        })
                                                };
                                                if let Some(request) = registration_request {
                                                    if let Err(err) =
                                                        handle.register_identity(peer_id, request).await
                                                    {
                                                        tracing::warn!(
                                                            "Failed to register local identity with {}: {:?}",
                                                            peer_id,
                                                            err
                                                        );
                                                    }
                                                }
                                                tracing::info!(
                                                    "Peer identified via Swarm: {} (agent: {})",
                                                    peer_id,
                                                    agent_version
                                                );
                                                let core_guard = core.lock();
                                                if let Some(core_ref) = core_guard.as_ref() {
                                                    #[cfg(not(target_arch = "wasm32"))]
                                                    {
                                                        // Annotate identity in ledger for each
                                                        // listen address.
                                                        //
                                                        // Review F3: `listen_addrs` is whatever
                                                        // the remote chose to put in its
                                                        // Identify response. Unfiltered, a peer
                                                        // could have us persist -- and later
                                                        // dial and re-gossip --
                                                        // /ip4/169.254.169.254/tcp/80 or a
                                                        // loopback service on our own host.
                                                        //
                                                        // Re-review NEW-1: `DnsPolicy::Reject`,
                                                        // because a peer that advertises
                                                        // /dns4/evil.example/tcp/80 in Identify
                                                        // picks the resolved IP at dial time and
                                                        // can re-point it between probes -- the
                                                        // IP rules above never run on a name.
                                                        let mut accepted = Vec::new();
                                                        for addr in &listen_addrs {
                                                            let addr_str = addr.to_string();
                                                            let mode = if let Some(engine) = &core_ref.escalation_engine {
                                                                if engine.current_transport() == crate::mobile_bridge::TransportType::Cellular {
                                                                    crate::transport::addr_filter::NetworkMode::Public
                                                                } else {
                                                                    crate::transport::addr_filter::NetworkMode::Local
                                                                }
                                                            } else {
                                                                crate::transport::addr_filter::NetworkMode::Local
                                                            };
                                                            if !crate::transport::addr_filter::is_dialable_multiaddr(
                                                                &addr_str,
                                                                mode,
                                                                crate::transport::addr_filter::DnsPolicy::Reject,
                                                            ) {
                                                                tracing::debug!(
                                                                    "Ignoring non-routable Identify address from {}: {}",
                                                                    peer_id,
                                                                    addr_str
                                                                );
                                                                continue;
                                                            }
                                                            accepted.push((
                                                                addr_str,
                                                                peer_id.to_string(),
                                                                public_key.clone(),
                                                                None, // Nickname not available in Identify
                                                            ));
                                                        }
                                                        if !accepted.is_empty() {
                                                            core_ref.ledger_manager.annotate_identities_batch(accepted);
                                                        }
                                                    }

                                                    if let Some(delegate) =
                                                        core_ref.delegate.read().as_ref()
                                                    {
                                                        let addrs_str: Vec<String> = listen_addrs
                                                            .iter()
                                                            .map(|a| a.to_string())
                                                            .collect();
                                                        delegate.on_peer_identified(
                                                            peer_id.to_string(),
                                                            agent_version,
                                                            addrs_str,
                                                        );
                                                    }
                                                }
                                            }
                                            crate::transport::SwarmEvent::NatStatusChanged(
                                                status,
                                            ) => {
                                                tracing::info!("[NAT] NAT status updated: {}", status);
                                                *nat_status.lock() = status;
                                            }
                                            crate::transport::SwarmEvent::PortMapping(status) => {
                                                tracing::info!("[NET] Port mapping updated: {}", status);
                                            }
                                            crate::transport::SwarmEvent::AbuseSignalDetected {
                                                peer_id,
                                                signal,
                                            } => {
                                                tracing::info!(
                                                    "Abuse signal detected from {}: {}",
                                                    peer_id,
                                                    signal
                                                );
                                                let core_guard = core.lock();
                                                if let Some(core_ref) = core_guard.as_ref() {
                                                    core_ref.record_abuse_signal(
                                                        peer_id.to_string(),
                                                        signal,
                                                    );
                                                }
                                            }
                                            crate::transport::SwarmEvent::LedgerReceived {
                                                from_peer: _,
                                                entries,
                                            } => {
                                                let core_guard = core.lock();
                                                if let Some(core_ref) = core_guard.as_ref() {
                                                    // Review F3: this is the ONLY live writer of
                                                    // wire-learned ledger entries, fed straight
                                                    // from /sc/ledger-exchange/1.0.0 data sent by
                                                    // any connected peer. Whatever lands here
                                                    // becomes a seed-dial candidate and is
                                                    // re-gossiped, so it is filtered before it is
                                                    // stored, not after.
                                                    #[cfg(not(target_arch = "wasm32"))]
                                                    {
                                                        let mut accepted = Vec::new();
                                                    for entry in entries {
                                                        let stripped =
                                                            crate::transport::addr_filter::strip_peer_id(
                                                                &entry.multiaddr,
                                                            );
                                                        // `DnsPolicy::Reject` (re-review NEW-1):
                                                        // this is the exact path the finding
                                                        // describes -- a ledger-exchange entry
                                                        // naming /dns4/evil.example/tcp/80 was
                                                        // stored here, became a seed-dial
                                                        // candidate, and the desktop swarm wires
                                                        // a real resolver.
                                                        let mode = if let Some(engine) = &core_ref.escalation_engine {
                                                            if engine.current_transport() == crate::mobile_bridge::TransportType::Cellular {
                                                                crate::transport::addr_filter::NetworkMode::Public
                                                            } else {
                                                                crate::transport::addr_filter::NetworkMode::Local
                                                            }
                                                        } else {
                                                            crate::transport::addr_filter::NetworkMode::Local
                                                        };
                                                        if !crate::transport::addr_filter::is_dialable_multiaddr(
                                                            &stripped,
                                                            mode,
                                                            crate::transport::addr_filter::DnsPolicy::Reject,
                                                        ) {
                                                            tracing::debug!(
                                                                "Dropping non-routable ledger entry from the wire: {}",
                                                                entry.multiaddr
                                                            );
                                                            continue;
                                                        }
                                                        if let Some(peer_id) = entry.last_peer_id {
                                                            accepted.push((
                                                                stripped,
                                                                peer_id,
                                                                None,
                                                                None,
                                                            ));
                                                        }
                                                    }
                                                    if !accepted.is_empty() {
                                                        core_ref.ledger_manager.annotate_identities_batch(accepted);
                                                    }
                                                    }
                                                }
                                            }
                                            crate::transport::SwarmEvent::ListeningOn(
                                                addr,
                                            ) => {
                                                tracing::info!("Swarm listening on {}", addr);
                                                eprintln!("[IronCore] [OK] Swarm listening on {}", addr);
                                                let is_expected = expected_listen_addr.as_ref() == Some(&addr);
                                                // If we bound to 0.0.0.0, the emitted addr will be a specific interface IP.
                                                // So we must also accept any TCP address if it was our primary transport.
                                                let is_primary_tcp = addr.iter().any(|p| matches!(p, libp2p::multiaddr::Protocol::Tcp(_)));

                                                if is_expected || is_primary_tcp || !await_listener {
                                                    if let Some(tx) = startup_signal.take() {
                                                        let _ = tx.try_send(Ok(()));
                                                    }
                                                } else {
                                                    tracing::debug!("Ignoring ListeningOn for incidental/relay address: {}", addr);
                                                }
                                            }
                                            crate::transport::SwarmEvent::ListenerFailed {
                                                listener_id,
                                                error,
                                            } => {
                                                tracing::error!(
                                                    "Swarm listener {} failed: {}",
                                                    listener_id,
                                                    error
                                                );
                                                eprintln!(
                                                    "[IronCore] [ERROR] Swarm listener {} failed: {}",
                                                    listener_id,
                                                    error
                                                );
                                                // F4: Ignore listener failures for the startup signal to avoid
                                                // false-failures from QUIC/WS/relay listeners. Genuine TCP bind
                                                // failures will correctly trigger the 15s startup timeout.
                                            }
                                            crate::transport::SwarmEvent::RelayCircuitEstablished => {
                                                tracing::info!("Relay circuit established");
                                            }
                                            crate::transport::SwarmEvent::RelayCircuitBroken => {
                                                tracing::info!("Relay circuit broken");
                                            }
                                            other => {
                                                tracing::debug!("Swarm event: {:?}", other);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    *swarm_mode_state.lock() = None;
                                    tracing::error!("Failed to start swarm: {:?}", e);
                                    let err_msg = format!("START_SWARM_WITH_CONFIG ERROR: {:?}", e);
                                    let _ = std::fs::write("/data/data/com.scmessenger.android/files/swarm_error.txt", &err_msg);
                                    if let Some(tx) = startup_signal.take() {
                                        let _ = tx.try_send(Err(err_msg));
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to create swarm Tokio runtime: {}", e);
                        let err_msg = format!("failed to create swarm Tokio runtime: {}", e);
                        let _ = std::fs::write("/data/data/com.scmessenger.android/files/swarm_error.txt", &err_msg);
                        let _ = startup_tx.try_send(Err(err_msg));
                    }
                }
            })
            .map_err(|_| crate::IronCoreError::Internal)?;

        // Block until the swarm reports its true startup outcome: either the
        // first listener is actually bound (NewListenAddr), the bind failed
        // (ListenerFailed), or swarm construction itself errored. Callers must
        // invoke this from a background thread/dispatcher, never the UI thread.
        match startup_rx.recv_timeout(std::time::Duration::from_secs(15)) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => {
                tracing::error!("Swarm startup failed: {}", e);
                eprintln!("[IronCore] [ERROR] Swarm startup failed: {}", e);
                self.swarm_bridge.shutdown_blocking();
                *self.swarm_bridge.handle.lock() = None;
                *self.swarm_headless_mode.lock() = None;
                Err(crate::IronCoreError::NetworkError)
            }
            Err(_) => {
                tracing::error!("Swarm startup timed out waiting for first listener");
                eprintln!("[IronCore] [ERROR] Swarm startup timed out waiting for first listener");
                self.swarm_bridge.shutdown_blocking();
                *self.swarm_bridge.handle.lock() = None;
                *self.swarm_headless_mode.lock() = None;
                Err(crate::IronCoreError::NetworkError)
            }
        }
    }

    pub fn get_swarm_bridge(&self) -> std::sync::Arc<SwarmBridge> {
        self.swarm_bridge.clone()
    }

    pub fn update_device_state(&self, profile: DeviceProfile) {
        let new_state = DeviceState::from_profile(&profile);

        // Read old state for transition logging (cheap read-lock).
        let old_state = self.device_state.read().clone();

        // Log any meaningful transitions before storing the new state.
        if let Some(ref old) = old_state {
            if old.battery_level != new_state.battery_level {
                tracing::debug!(
                    "Battery level changed: {}% → {}%",
                    old.battery_level,
                    new_state.battery_level
                );
            }
            if old.is_charging != new_state.is_charging {
                tracing::info!(
                    "Charging state changed: {} → {}",
                    old.is_charging,
                    new_state.is_charging
                );
            }
            if old.network_type != new_state.network_type {
                tracing::info!(
                    "Network type changed: {:?} → {:?}",
                    old.network_type,
                    new_state.network_type
                );
            }
            if old.motion_state != new_state.motion_state {
                tracing::info!(
                    "Motion state changed: {:?} → {:?}",
                    old.motion_state,
                    new_state.motion_state
                );
            }

            // Threshold-crossing events deserve explicit log entries.
            let was_critical = old.battery_level <= 10 && !old.is_charging;
            let is_critical = new_state.battery_level <= 10 && !new_state.is_charging;
            let was_low = old.battery_level <= 20 && !old.is_charging;
            let is_low = new_state.battery_level <= 20 && !new_state.is_charging;

            if !was_critical && is_critical {
                tracing::warn!(
                    "Battery CRITICAL ({}%, not charging) — entering minimal operation",
                    new_state.battery_level
                );
            } else if was_critical && !is_critical {
                tracing::info!(
                    "Battery recovered from critical ({}%{})",
                    new_state.battery_level,
                    if new_state.is_charging {
                        ", charging"
                    } else {
                        ""
                    }
                );
            } else if !was_low && is_low {
                tracing::warn!(
                    "Battery LOW ({}%, not charging) — reducing scan and relay activity",
                    new_state.battery_level
                );
            } else if was_low && !is_low {
                tracing::info!(
                    "Battery recovered from low ({}%{})",
                    new_state.battery_level,
                    if new_state.is_charging {
                        ", charging"
                    } else {
                        ""
                    }
                );
            }
        } else {
            // First report — just log the initial state.
            tracing::info!(
                "Device state initialised: battery={}% charging={} network={:?} motion={:?}",
                new_state.battery_level,
                new_state.is_charging,
                new_state.network_type,
                new_state.motion_state
            );
        }

        // Persist the new DeviceState.
        *self.device_state.write() = Some(new_state.clone());

        // Also keep the legacy DeviceProfile for callers that still use it.
        *self.current_device_profile.lock() = Some(profile.clone());

        // Derive and apply behavior adjustments using the new engine.
        let adj_profile = self.auto_adjust.compute_profile(profile.clone());
        let ble_adj = self.auto_adjust.compute_ble_adjustment(adj_profile);
        let relay_adj = self.auto_adjust.compute_relay_adjustment(adj_profile);

        tracing::info!(
            "Behavior adjustment computed: profile={:?}, scan={}ms, advertise={}ms, relay_budget={}",
            adj_profile,
            ble_adj.scan_interval_ms,
            ble_adj.advertise_interval_ms,
            relay_adj.max_per_hour
        );

        // Derive and apply behavior adjustments (legacy path for now).
        let adj = Self::compute_behavior(&new_state);

        if adj.minimal_operation {
            tracing::warn!(
                "Applying MINIMAL operation mode (battery={}%)",
                new_state.battery_level
            );
        }

        // Apply relay budget from the new engine (this fulfills the 'wiring'
        // requirement). Non-blocking variant: update_device_state is invoked
        // from sync platform callbacks (battery/network receivers) and must
        // not wait on the swarm reply.
        self.set_relay_budget_nonblocking(relay_adj.max_per_hour);

        // P0_RELIABILITY_001: Notify platform bridge of state change if it's subscribed.
        // This ensures the platform (Android/iOS) UI stays in sync with core adjustments.
        if let Some(bridge) = self.platform_bridge.lock().as_ref() {
            bridge.on_battery_changed(profile.battery_pct, profile.is_charging);
            bridge.on_network_changed(profile.has_wifi, false); // Cellular not in profile yet
            bridge.on_motion_changed(profile.motion_state);
        }

        // B1_CORE_ENTRY_007: Periodic routing engine maintenance
        // Advance the routing engine by one tick to maintain up-to-date routing state.
        // Called on device state changes to ensure routing stays synchronized with network conditions.
        let _ = self.routing_tick();
    }

    /// Return the recommended behavior adjustments for the *current* device state.
    ///
    /// Returns `None` if no device state has been reported yet.
    pub fn recommended_behavior(&self) -> Option<BehaviorAdjustment> {
        self.device_state
            .read()
            .as_ref()
            .map(Self::compute_behavior)
    }

    /// Return a clone of the most recently stored `DeviceState`, if any.
    pub fn get_device_state(&self) -> Option<DeviceState> {
        self.device_state.read().clone()
    }

    /// Async FFI (Issue 5): exported to Kotlin as a `suspend fun`.
    pub async fn set_relay_budget(&self, messages_per_hour: u32) {
        self.apply_relay_budget_state(messages_per_hour);

        // If swarm is already running, forward the budget update immediately.
        // Bind the clone to a local first: an `if let` on `.lock().clone()`
        // would keep the (non-Send) guard alive across the await.
        let handle = self.swarm_bridge.handle.lock().clone();
        if let Some(handle) = handle {
            handle.set_relay_budget(messages_per_hour).await.ok();
        }
    }

    /// Access the auto-adjustment engine to set overrides or query current profile.
    pub fn get_auto_adjust_engine(&self) -> std::sync::Arc<AutoAdjustEngine> {
        self.auto_adjust.clone()
    }

    pub fn on_peer_discovered(&self, peer_id: String) {
        let mut stats = self.stats.lock();
        stats.peers_discovered += 1;
        tracing::info!("Peer discovered: {}", peer_id);
    }

    /// B1_CORE_ENTRY_009: Production caller for ratchet_reset_session
    /// Reset the ratchet session for a peer when they disconnect.
    /// This ensures fresh keys when they reconnect, providing forward secrecy.
    pub fn on_peer_disconnected(&self, peer_id: String) {
        tracing::info!("Peer disconnected: {}", peer_id);
        // Reset ratchet session for the disconnected peer to force re-key on reconnection
        if let Some(core) = self.get_core() {
            core.ratchet_reset_session(peer_id);
        }
    }

    pub fn on_data_received(&self, peer_id: String, data: Vec<u8>) {
        let mut stats = self.stats.lock();
        stats.bytes_transferred += data.len() as u64;
        drop(stats);

        eprintln!(
            "[IronCore] on_data_received from {} ({} bytes)",
            peer_id,
            data.len()
        );
        if let Some(core) = self.get_core() {
            match core.receive_message(data) {
                Ok(msg) => {
                    if msg.message_type == crate::message::MessageType::OnionRelay {
                        // RELAY: Forward to next hop
                        let next_hop_hex = msg.recipient_id.clone();
                        let payload = msg.payload.clone();

                        eprintln!(
                            "[IronCore] [RELAY] BLE Onion relay: forwarding to {}",
                            next_hop_hex
                        );

                        // For BLE, we might want to try both BLE and Internet
                        let bridge_clone = self.swarm_bridge.clone();
                        let spawn_res = bridge_clone.get_runtime_handle().spawn(async move {
                            let _ = bridge_clone
                                .send_message(next_hop_hex, payload, None, None)
                                .await;
                        });
                        drop(spawn_res);

                        let mut stats = self.stats.lock();
                        stats.messages_relayed += 1;
                    } else {
                        tracing::info!("Message received from {}: {:?}", peer_id, msg.id);
                        eprintln!(
                            "[IronCore] [OK] BLE message received from {}: {}",
                            peer_id, msg.id
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to process received message: {:?}", e);
                    eprintln!(
                        "[IronCore] [ERROR] BLE receive_message FAILED from {}: {:?}",
                        peer_id, e
                    );
                }
            }
        } else {
            eprintln!(
                "[IronCore] [ERROR] on_data_received SKIPPED from {}: core not initialized",
                peer_id
            );
        }
    }

    pub fn on_battery_changed(&self, battery_pct: u8, is_charging: bool) {
        let mut profile = self
            .current_device_profile
            .lock()
            .clone()
            .unwrap_or_default();
        profile.battery_pct = battery_pct;
        profile.is_charging = is_charging;
        self.update_device_state(profile);
    }

    pub fn on_network_changed(&self, has_wifi: bool, _has_cellular: bool) {
        let mut profile = self
            .current_device_profile
            .lock()
            .clone()
            .unwrap_or_default();
        profile.has_wifi = has_wifi;
        // profile doesn't have has_cellular yet, but we've ingested it.
        self.update_device_state(profile);
    }

    pub fn on_motion_changed(&self, motion: MotionState) {
        let mut profile = self
            .current_device_profile
            .lock()
            .clone()
            .unwrap_or_default();
        profile.motion_state = motion;
        self.update_device_state(profile);
    }

    pub fn on_entering_background(&self) {
        tracing::info!("App entering background; reducing activity level");
        self.pause();
    }

    pub fn on_entering_foreground(&self) {
        tracing::info!("App entering foreground; restoring activity level");
        self.resume();
    }

    pub fn on_ble_data_received(&self, peer_id: String, data: Vec<u8>) {
        self.on_proximity_data_received(peer_id, ProximityTransport::Ble, data);
    }

    pub fn on_proximity_data_received(
        &self,
        peer_id: String,
        transport: ProximityTransport,
        data: Vec<u8>,
    ) {
        tracing::info!("{} data received from {}", transport, peer_id);
        if data.len() > transport.max_payload_size() {
            tracing::warn!(
                "{} payload from {} exceeds max ({} > {}), dropping",
                transport,
                peer_id,
                data.len(),
                transport.max_payload_size()
            );
            return;
        }
        if transport == ProximityTransport::Ble {
            self.nearby_ble_peers.lock().insert(peer_id.clone());
        }
        self.on_data_received(peer_id, data);
    }

    /// Helper to get the core instance exposed to UniFFI
    pub fn get_core(&self) -> Option<std::sync::Arc<crate::IronCore>> {
        self.core.lock().clone()
    }

    /// Run a bounded drift maintenance cycle within the given time budget.
    pub fn run_maintenance_cycle(&self, budget_ms: u32) -> String {
        if let Some(core) = self.get_core() {
            core.run_maintenance_cycle(budget_ms)
        } else {
            r#"{"work_done":0,"elapsed_ms":0,"budget_ms":0,"remaining":false}"#.to_string()
        }
    }

    pub fn on_wifi_aware_peer_discovered(&self, peer_id: String, service_info: Vec<u8>, rssi: i32) {
        if let Some(transport) = self.wifi_aware_transport.lock().as_ref() {
            transport.add_discovered_peer(peer_id.clone(), service_info.clone(), rssi);
        }
        if let Some(aware_bridge) = self.wifi_aware_bridge.lock().as_ref() {
            aware_bridge.handle_service_discovered(peer_id.clone(), service_info, rssi);
        }

        let transport_opt = self.wifi_aware_transport.lock().clone();
        let swarm_bridge = self.swarm_bridge.clone();
        let core_opt = self.get_core();
        if let Some(transport) = transport_opt {
            let rt = swarm_bridge.get_runtime_handle();
            rt.spawn(async move {
                if let Ok(peer_id_parsed) = peer_id.parse::<libp2p::PeerId>() {
                    let peer_id_bytes = peer_id_parsed.to_bytes();
                    let remote_pubkey_bytes: Vec<u8> = if peer_id_bytes.len() >= 32 {
                        peer_id_bytes[peer_id_bytes.len() - 32..].to_vec()
                    } else {
                        vec![0u8; 32]
                    };
                    let pmk_result = core_opt
                        .as_ref()
                        .ok_or(crate::IronCoreError::NotInitialized)
                        .and_then(|core| core.derive_wifi_aware_pmk(remote_pubkey_bytes));
                    let pmk_vec = match pmk_result {
                        Ok(pmk) => pmk,
                        Err(_) => return,
                    };
                    let mut pmk = [0u8; 32];
                    if pmk_vec.len() != 32 {
                        return;
                    }
                    pmk.copy_from_slice(&pmk_vec);
                    if let Ok(path_info) = transport.create_data_path(peer_id_parsed, &pmk).await {
                        let multiaddr_str = if path_info.ip_address.contains(':') {
                            format!("/ip6/{}/tcp/{}", path_info.ip_address, path_info.port)
                        } else {
                            format!("/ip4/{}/tcp/{}", path_info.ip_address, path_info.port)
                        };
                        let _ = swarm_bridge.dial(multiaddr_str).await;
                    }
                }
            });
        }
    }

    pub fn on_wifi_aware_data_path_confirmed(
        &self,
        peer_id: String,
        ip_address: String,
        port: u16,
    ) {
        if let Some(aware_bridge) = self.wifi_aware_bridge.lock().as_ref() {
            aware_bridge.handle_data_path_confirmed(peer_id, ip_address, port);
        }
    }

    pub fn on_wifi_direct_peer_discovered(
        &self,
        peer_id: String,
        device_name: String,
        device_address: String,
        rssi: i32,
    ) {
        if let Ok(peer_id_parsed) = peer_id.parse::<libp2p::PeerId>() {
            let peer = crate::transport::wifi_direct::WifiDirectPeer {
                peer_id: peer_id_parsed,
                device_name,
                device_address,
                rssi,
            };
            if let Some(transport) = self.wifi_direct_transport.lock().as_ref() {
                transport.register_peer(peer.clone());
            }
            if let Some(direct_bridge) = self.wifi_direct_bridge.lock().as_ref() {
                direct_bridge.handle_peers_changed(vec![peer]);
            }
        }
    }

    pub fn on_wifi_direct_connection_info(
        &self,
        _peer_id: String,
        group_owner_ip: String,
        is_group_owner: bool,
    ) {
        let info = crate::transport::wifi_direct::GroupInfo {
            group_owner: is_group_owner,
            group_owner_ip: Some(group_owner_ip.clone()),
            client_ips: vec![],
            interface_name: "wlan0".to_string(),
            port: None,
        };

        if let Some(transport) = self.wifi_direct_transport.lock().as_ref() {
            transport.set_group_info(info.clone());
        }
        if let Some(direct_bridge) = self.wifi_direct_bridge.lock().as_ref() {
            direct_bridge.handle_connection_info(info);
        }

        if !is_group_owner {
            let swarm_bridge = self.swarm_bridge.clone();
            let rt = swarm_bridge.get_runtime_handle();
            rt.spawn(async move {
                let multiaddr_str = format!("/ip4/{}/tcp/9001", group_owner_ip);
                let _ = swarm_bridge.dial(multiaddr_str).await;
            });
        }
    }

    pub fn export_identity_backup(
        &self,
        passphrase: String,
    ) -> Result<String, crate::IronCoreError> {
        let core = self
            .core
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NotInitialized)?;
        core.export_identity_backup(passphrase)
    }

    pub fn export_identity_backup_with_salt(
        &self,
        passphrase: String,
        salt: Vec<u8>,
    ) -> Result<String, crate::IronCoreError> {
        let core = self
            .core
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NotInitialized)?;
        core.export_identity_backup_with_salt(passphrase, Some(salt))
    }

    /// Export identity backup using fast Blake3 KDF (for device-bound
    /// auto-backups with high-entropy random passphrases). Sub-millisecond.
    pub fn export_identity_backup_fast(
        &self,
        passphrase: String,
    ) -> Result<String, crate::IronCoreError> {
        let core = self
            .core
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NotInitialized)?;
        core.export_identity_backup_fast(passphrase)
    }

    /// Export identity backup using fast Blake3 KDF with a custom salt.
    pub fn export_identity_backup_fast_with_salt(
        &self,
        passphrase: String,
        salt: Vec<u8>,
    ) -> Result<String, crate::IronCoreError> {
        let core = self
            .core
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NotInitialized)?;
        core.export_identity_backup_fast_with_salt(passphrase, Some(salt))
    }

    pub fn import_identity_backup(
        &self,
        backup: String,
        passphrase: String,
    ) -> Result<(), crate::IronCoreError> {
        let core = self
            .core
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NotInitialized)?;
        core.import_identity_backup(backup, passphrase)
    }

    // Group 1: IronCore entrypoints (methods not in #[uniffi::export] block)
    // -----------------------------------------------------------------------

    /// Prepare a message with onion routing layers.
    /// Wraps the envelope in multiple layers of encryption for anonymous delivery.
    pub fn prepare_onion_message(
        &self,
        envelope_data: Vec<u8>,
        relay_public_keys_json: String,
    ) -> Result<Vec<u8>, crate::IronCoreError> {
        let core = self
            .get_core()
            .ok_or(crate::IronCoreError::NotInitialized)?;
        core.prepare_onion_message(envelope_data, relay_public_keys_json)
    }

    /// Peel one layer of an onion-routed envelope (relay-side operation).
    /// Decodes the next hop and removes one encryption layer.
    pub fn peel_onion_layer(
        &self,
        onion_data: Vec<u8>,
        relay_secret_key: Vec<u8>,
    ) -> Result<crate::PeelResult, crate::IronCoreError> {
        let core = self
            .get_core()
            .ok_or(crate::IronCoreError::NotInitialized)?;
        core.peel_onion_layer(onion_data, relay_secret_key)
    }

    /// Return a random available port for temporary listeners.
    pub fn random_port(&self) -> u16 {
        let core = self.get_core().unwrap_or_else(|| {
            // Fallback: create a temporary core just for this operation
            std::sync::Arc::new(crate::IronCore::new())
        });
        core.random_port()
    }

    /// Return the number of active ratchet sessions.
    pub fn ratchet_session_count(&self) -> u32 {
        let core = self.get_core().unwrap_or_else(|| {
            // Fallback: create a temporary core just for this operation
            std::sync::Arc::new(crate::IronCore::new())
        });
        core.ratchet_session_count()
    }

    /// Check if a ratchet session exists for the given peer.
    pub fn ratchet_has_session(&self, peer_id: String) -> bool {
        let core = self.get_core().unwrap_or_else(|| {
            // Fallback: create a temporary core just for this operation
            std::sync::Arc::new(crate::IronCore::new())
        });
        core.ratchet_has_session(peer_id)
    }

    /// Force-reset the ratchet session for a peer (re-key).
    pub fn ratchet_reset_session(&self, peer_id: String) {
        if let Some(core) = self.get_core() {
            core.ratchet_reset_session(peer_id);
        }
    }

    /// Advance the routing engine by one tick. Returns state snapshot as JSON.
    pub fn routing_tick(&self) -> String {
        let core = self.get_core().unwrap_or_else(|| {
            // Fallback: create a temporary core just for this operation
            std::sync::Arc::new(crate::IronCore::new())
        });
        core.routing_tick()
    }

    /// Check if service is running
    pub fn is_running(&self) -> bool {
        *self.state.lock() == ServiceState::Running
    }

    /// Get all connection statistics from the transport health monitor.
    /// Returns peer-by-peer connection stats for diagnostics.
    pub fn get_all_connection_stats(&self) -> std::collections::HashMap<String, String> {
        let core = self.get_core().unwrap_or_else(|| {
            // Fallback: create a temporary core just for this operation
            std::sync::Arc::new(crate::IronCore::new())
        });
        let stats = core.get_all_connection_stats();
        // Convert HashMap<PeerId, ConnectionStats> to String map for UniFFI
        stats
            .into_iter()
            .map(|(peer_id, conn_stats)| {
                (
                    peer_id.to_string(),
                    format!(
                        "state={:?},duration_ms={},messages_sent={},message_failures={},bytes_sent={},bytes_received={},avg_latency_ms={},connection_attempts={},successful_connections={},connection_failures={}",
                        conn_stats.state,
                        conn_stats.duration_ms,
                        conn_stats.messages_sent,
                        conn_stats.message_failures,
                        conn_stats.bytes_sent,
                        conn_stats.bytes_received,
                        conn_stats.avg_latency_ms,
                        conn_stats.connection_attempts,
                        conn_stats.successful_connections,
                        conn_stats.connection_failures
                    ),
                )
            })
            .collect()
    }

    /// Helper to dispatch a packet via BLE bridge
    pub fn dispatch_ble_packet(&self, peer_id: String, data: Vec<u8>) {
        self.dispatch_proximity_packet(peer_id, ProximityTransport::Ble, data);
    }

    /// Helper to dispatch a packet via any proximity transport
    pub fn dispatch_proximity_packet(
        &self,
        peer_id: String,
        transport: ProximityTransport,
        data: Vec<u8>,
    ) {
        if data.len() > transport.max_payload_size() {
            tracing::warn!(
                "{} payload to {} exceeds max ({} > {}), dropping",
                transport,
                peer_id,
                data.len(),
                transport.max_payload_size()
            );
            return;
        }
        if let Some(bridge) = self.platform_bridge.lock().as_ref() {
            bridge.send_proximity_packet(peer_id, transport, data);
        }
    }
}

// Non-UniFFI internal methods for MeshService
impl MeshService {
    /// Apply the local (synchronous) side of a relay-budget change: persist
    /// the budget and toggle drift protocol state. Shared by the async FFI
    /// `set_relay_budget` and the internal non-blocking variant.
    fn apply_relay_budget_state(&self, messages_per_hour: u32) {
        tracing::info!("Relay budget set: {} msgs/hour", messages_per_hour);
        *self.relay_budget.lock() = messages_per_hour;

        // P1_CORE_001: Sync drift protocol state with relay budget
        if let Some(core) = self.core.lock().as_ref() {
            if messages_per_hour > 0 {
                core.drift_activate();
            } else {
                core.drift_deactivate();
            }
        }
    }

    /// Relay-budget update for internal callers on synchronous paths
    /// (e.g. `update_device_state`, which runs inside platform battery /
    /// network callbacks). Applies local state immediately and forwards the
    /// budget to the swarm as a spawned task instead of awaiting it.
    pub(crate) fn set_relay_budget_nonblocking(&self, messages_per_hour: u32) {
        self.apply_relay_budget_state(messages_per_hour);

        if let Some(handle) = self.swarm_bridge.handle.lock().clone() {
            let rt = self.swarm_bridge.get_runtime_handle();
            rt.spawn(async move {
                let _ = handle.set_relay_budget(messages_per_hour).await;
            });
        }
    }

    /// Compute recommended behavior from a device state snapshot.
    ///
    /// This is a pure function — no side-effects — so callers can call it at
    /// any time without acquiring locks.
    pub fn compute_behavior(state: &DeviceState) -> BehaviorAdjustment {
        let battery = state.battery_level;
        let charging = state.is_charging;

        // Minimal mode: critical battery and not charging.
        if battery <= 10 && !charging {
            return BehaviorAdjustment {
                scan_interval_ms: 30_000, // 30 s — barely alive
                relay_enabled: false,
                relay_budget: 0,
                minimal_operation: true,
            };
        }

        // Low battery: reduce everything but keep messaging alive.
        if battery <= 20 && !charging {
            return BehaviorAdjustment {
                scan_interval_ms: 10_000, // 10 s
                relay_enabled: false,     // no relay duty when low
                relay_budget: 0,
                minimal_operation: false,
            };
        }

        // Stationary with good battery or charging: maximise relay duty.
        let stationary = matches!(state.motion_state, MotionState::Still);
        if charging || (battery >= 50 && stationary) {
            return BehaviorAdjustment {
                scan_interval_ms: 500, // very frequent
                relay_enabled: true,
                relay_budget: 200,
                minimal_operation: false,
            };
        }

        // Normal operation (battery 21–49, not charging, possibly moving).
        BehaviorAdjustment {
            scan_interval_ms: 2_000, // 2 s
            relay_enabled: true,
            relay_budget: 100,
            minimal_operation: false,
        }
    }

    fn resolve_swarm_keypair_and_mode(
        &self,
    ) -> Result<(libp2p::identity::Keypair, bool), crate::IronCoreError> {
        let identity_keypair = {
            let core_guard = self.core.lock();
            let core = core_guard
                .as_ref()
                .ok_or(crate::IronCoreError::NotInitialized)?;
            core.get_libp2p_keypair().ok()
        };

        if let Some(keypair) = identity_keypair {
            return Ok((keypair, false));
        }

        tracing::info!("No identity keypair available; using persisted headless network key");
        let keypair = self.load_or_create_headless_network_keypair()?;
        Ok((keypair, true))
    }

    fn load_or_create_headless_network_keypair(
        &self,
    ) -> Result<libp2p::identity::Keypair, crate::IronCoreError> {
        const HEADLESS_KEY_FILE: &str = "relay_network_key.pb";

        let Some(storage_path) = self.storage_path.as_ref() else {
            tracing::warn!("MeshService has no storage path; using ephemeral headless keypair");
            return Ok(libp2p::identity::Keypair::generate_ed25519());
        };

        let storage_dir = std::path::PathBuf::from(storage_path);
        std::fs::create_dir_all(&storage_dir).map_err(|_| crate::IronCoreError::StorageError)?;
        let key_path = storage_dir.join(HEADLESS_KEY_FILE);

        if key_path.exists() {
            let bytes = std::fs::read(&key_path).map_err(|_| crate::IronCoreError::StorageError)?;
            match libp2p::identity::Keypair::from_protobuf_encoding(&bytes) {
                Ok(keypair) => return Ok(keypair),
                Err(err) => {
                    tracing::warn!(
                        "Failed to decode headless network key at {} ({}); rotating key",
                        key_path.display(),
                        err
                    );
                }
            }
        }

        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let encoded = keypair
            .to_protobuf_encoding()
            .map_err(|_| crate::IronCoreError::Internal)?;
        std::fs::write(&key_path, encoded).map_err(|_| crate::IronCoreError::StorageError)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600));
        }

        Ok(keypair)
    }
}

/// Internal helper to bridge IronCore events back to MeshService and its external delegate.
struct MeshServiceCoreDelegate {
    service: std::sync::Weak<MeshService>,
}

impl crate::CoreDelegate for MeshServiceCoreDelegate {
    fn on_peer_discovered(&self, peer_id: String) {
        if let Some(service) = self.service.upgrade() {
            service.on_peer_discovered(peer_id.clone());
            if let Some(delegate) = service.external_delegate.lock().as_ref() {
                delegate.on_peer_discovered(peer_id);
            }
        }
    }

    fn on_peer_disconnected(&self, peer_id: String) {
        if let Some(service) = self.service.upgrade() {
            service.on_peer_disconnected(peer_id.clone());
            if let Some(delegate) = service.external_delegate.lock().as_ref() {
                delegate.on_peer_disconnected(peer_id);
            }
        }
    }

    fn on_peer_identified(
        &self,
        peer_id: String,
        agent_version: String,
        listen_addrs: Vec<String>,
    ) {
        if let Some(service) = self.service.upgrade() {
            if let Some(delegate) = service.external_delegate.lock().as_ref() {
                delegate.on_peer_identified(peer_id, agent_version, listen_addrs);
            }
        }
    }

    fn on_message_received(
        &self,
        sender_id: String,
        sender_public_key_hex: String,
        message_id: String,
        sender_timestamp: u64,
        data: Vec<u8>,
    ) {
        if let Some(service) = self.service.upgrade() {
            if let Some(delegate) = service.external_delegate.lock().as_ref() {
                delegate.on_message_received(
                    sender_id,
                    sender_public_key_hex,
                    message_id,
                    sender_timestamp,
                    data,
                );
            }
        }
    }

    fn on_receipt_received(&self, message_id: String, status: String) {
        if let Some(service) = self.service.upgrade() {
            if let Some(delegate) = service.external_delegate.lock().as_ref() {
                delegate.on_receipt_received(message_id, status);
            }
        }
    }
}

// PlatformBridge callback trait (implemented by mobile platforms)
pub trait PlatformBridge: Send + Sync {
    fn on_battery_changed(&self, battery_pct: u8, is_charging: bool);
    fn on_network_changed(&self, has_wifi: bool, has_cellular: bool);
    fn on_motion_changed(&self, motion: MotionState);
    fn on_ble_data_received(&self, peer_id: String, data: Vec<u8>);
    fn on_entering_background(&self);
    fn on_entering_foreground(&self);
    fn send_ble_packet(&self, peer_id: String, data: Vec<u8>);
    fn on_proximity_data_received(
        &self,
        peer_id: String,
        transport: ProximityTransport,
        data: Vec<u8>,
    );
    fn send_proximity_packet(&self, peer_id: String, transport: ProximityTransport, data: Vec<u8>);
    fn wifi_aware_publish(&self, service_name: String, service_info: Vec<u8>) -> bool;
    fn wifi_aware_subscribe(&self, service_name: String) -> bool;
    fn wifi_aware_create_data_path(&self, peer_id: String, pmk: Vec<u8>) -> bool;
    fn wifi_aware_stop(&self);
    fn wifi_direct_discover_peers(&self) -> bool;
    fn wifi_direct_stop_discovery(&self);
    fn wifi_direct_connect(&self, device_address: String) -> bool;
    fn wifi_direct_create_group(&self, group_name: String) -> bool;
    fn wifi_direct_remove_group(&self);
}

pub trait WifiAwareCallback: Send + Sync {
    fn on_service_discovered(&self, peer_id: String, service_info: Vec<u8>, rssi: i32);
    fn on_data_path_confirmed(&self, peer_id: String, ip_address: String, port: u16);
}

// ============================================================================
// WIFI AWARE PLATFORM BRIDGE ADAPTER
// ============================================================================

/// Adapter that bridges the synchronous UniFFI PlatformBridge to the async
/// WifiAwarePlatformBridge trait used by WifiAwareTransport.
///
/// Control commands (publish, subscribe, create_data_path) are forwarded to
/// the platform via PlatformBridge methods. Callbacks from the platform are
/// routed through channels to satisfy async await patterns.
#[allow(clippy::type_complexity)]
pub struct PlatformWifiAwareBridge {
    platform_bridge: std::sync::Arc<Mutex<Option<Box<dyn PlatformBridge>>>>,
    discovered_peers: Arc<Mutex<HashMap<String, (Vec<u8>, i32)>>>,
    data_path_results:
        Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<std::net::SocketAddr>>>>,
    on_service_discovered: Arc<Mutex<Option<Box<dyn Fn(String, Vec<u8>, i32) + Send + Sync>>>>,
}

impl PlatformWifiAwareBridge {
    pub fn new_platform_ref(
        platform_bridge: std::sync::Arc<Mutex<Option<Box<dyn PlatformBridge>>>>,
    ) -> Self {
        Self {
            platform_bridge,
            discovered_peers: Arc::new(Mutex::new(HashMap::new())),
            data_path_results: Arc::new(Mutex::new(HashMap::new())),
            on_service_discovered: Arc::new(Mutex::new(None)),
        }
    }

    fn with_platform<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&dyn PlatformBridge) -> R,
    {
        self.platform_bridge.lock().as_ref().map(|b| f(b.as_ref()))
    }

    pub fn handle_service_discovered(&self, peer_id: String, service_info: Vec<u8>, rssi: i32) {
        self.discovered_peers
            .lock()
            .insert(peer_id.clone(), (service_info.clone(), rssi));
        if let Some(cb) = self.on_service_discovered.lock().as_ref() {
            cb(peer_id, service_info, rssi);
        }
    }

    pub fn handle_data_path_confirmed(&self, peer_id: String, ip_address: String, port: u16) {
        // Build SocketAddr from the parsed IpAddr rather than formatting
        // "ip:port" and parsing that as a whole: an unbracketed IPv6 string
        // formatted that way (e.g. "fe80::1234:8765") is not valid SocketAddr
        // syntax (IPv6 needs "[ip]:port"), so every IPv6 confirmation would
        // silently fail to parse and never resolve create_data_path's future.
        match ip_address.parse::<std::net::IpAddr>() {
            Ok(ip) => {
                let addr = std::net::SocketAddr::new(ip, port);
                let mut results = self.data_path_results.lock();
                if let Some(tx) = results.remove(&peer_id) {
                    let _ = tx.send(addr);
                }
            }
            Err(e) => {
                tracing::warn!(
                    "WiFi Aware data path confirmed with unparseable IP '{}': {}",
                    ip_address,
                    e
                );
            }
        }
    }

    pub fn get_discovered_peer(&self, peer_id: &str) -> Option<(Vec<u8>, i32)> {
        self.discovered_peers.lock().get(peer_id).cloned()
    }
}

#[async_trait]
impl WifiAwarePlatformBridge for PlatformWifiAwareBridge {
    async fn is_available(&self) -> Result<bool, WifiAwareError> {
        Ok(self.with_platform(|_| true).unwrap_or(false))
    }

    async fn publish_service(
        &self,
        service_name: &str,
        service_info: &[u8],
    ) -> Result<(), WifiAwareError> {
        let ok = self
            .with_platform(|b| {
                b.wifi_aware_publish(service_name.to_string(), service_info.to_vec())
            })
            .unwrap_or(false);
        if ok {
            Ok(())
        } else {
            Err(WifiAwareError::PlatformError("Publish failed".into()))
        }
    }

    async fn subscribe_to_services(
        &self,
        service_name: &str,
        _match_filter: Option<&[u8]>,
    ) -> Result<(), WifiAwareError> {
        let ok = self
            .with_platform(|b| b.wifi_aware_subscribe(service_name.to_string()))
            .unwrap_or(false);
        if ok {
            Ok(())
        } else {
            Err(WifiAwareError::PlatformError("Subscribe failed".into()))
        }
    }

    async fn unpublish_service(&self) -> Result<(), WifiAwareError> {
        if let Some(b) = self.platform_bridge.lock().as_ref() {
            b.wifi_aware_stop();
        }
        Ok(())
    }

    async fn unsubscribe_from_services(&self) -> Result<(), WifiAwareError> {
        if let Some(b) = self.platform_bridge.lock().as_ref() {
            b.wifi_aware_stop();
        }
        Ok(())
    }

    async fn create_data_path(
        &self,
        peer_id: &str,
        pmk: &[u8; 32],
    ) -> Result<std::net::SocketAddr, WifiAwareError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.data_path_results
            .lock()
            .insert(peer_id.to_string(), tx);

        let ok = self
            .with_platform(|b| b.wifi_aware_create_data_path(peer_id.to_string(), pmk.to_vec()))
            .unwrap_or(false);

        if !ok {
            self.data_path_results.lock().remove(peer_id);
            return Err(WifiAwareError::DataPathFailed(
                "Platform rejected data path creation".into(),
            ));
        }

        // Await (not block) the confirmation: this runs on a shared tokio
        // worker thread, and a blocking wait here would starve other tasks
        // (including the swarm's own event loop) whenever multiple peers are
        // discovered concurrently.
        tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| {
                self.data_path_results.lock().remove(peer_id);
                WifiAwareError::DataPathFailed("Data path confirmation timed out".into())
            })?
            .map_err(|_| WifiAwareError::DataPathFailed("Confirmation sender dropped".into()))
    }

    async fn close_data_path(&self, _peer_id: &str) -> Result<(), WifiAwareError> {
        Ok(())
    }

    fn set_on_service_discovered(&self, callback: Box<dyn Fn(String, Vec<u8>, i32) + Send + Sync>) {
        *self.on_service_discovered.lock() = Some(callback);
    }

    fn set_on_message_received(&self, _callback: Box<dyn Fn(String, Vec<u8>) + Send + Sync>) {}

    fn set_on_data_path_confirmed(
        &self,
        _callback: Box<dyn Fn(String, std::net::SocketAddr) + Send + Sync>,
    ) {
    }
}

// ============================================================================
// AUTO-ADJUST ENGINE
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct DeviceProfile {
    pub battery_pct: u8,
    pub is_charging: bool,
    pub has_wifi: bool,
    pub motion_state: MotionState,
    pub peer_id: Option<String>,
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustmentProfile {
    Maximum,
    High,
    Standard,
    Reduced,
    Minimal,
}

#[derive(Debug, Clone)]
pub struct BleAdjustment {
    pub scan_interval_ms: u32,
    pub advertise_interval_ms: u32,
    pub tx_power_dbm: i8,
}

#[derive(Debug, Clone)]
pub struct RelayAdjustment {
    pub max_per_hour: u32,
    pub priority_threshold: u8,
    pub max_payload_bytes: u32,
}

#[derive(uniffi::Object)]
pub struct AutoAdjustEngine {
    ble_scan_override: Mutex<Option<u32>>,
    relay_max_override: Mutex<Option<u32>>,
}

impl Default for AutoAdjustEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl AutoAdjustEngine {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            ble_scan_override: Mutex::new(None),
            relay_max_override: Mutex::new(None),
        }
    }

    pub fn compute_profile(&self, device: DeviceProfile) -> AdjustmentProfile {
        // Logic from core/src/mobile/auto_adjust.rs
        if device.is_charging && device.has_wifi {
            AdjustmentProfile::Maximum
        } else if device.battery_pct > 50 {
            AdjustmentProfile::High
        } else if device.battery_pct > 30 {
            AdjustmentProfile::Standard
        } else if device.battery_pct > 15 {
            AdjustmentProfile::Reduced
        } else {
            AdjustmentProfile::Minimal
        }
    }

    pub fn compute_ble_adjustment(&self, profile: AdjustmentProfile) -> BleAdjustment {
        let (scan_interval, advertise_interval, tx_power) = match profile {
            AdjustmentProfile::Maximum => (500, 100, 4),
            AdjustmentProfile::High => (1000, 200, 0),
            AdjustmentProfile::Standard => (2000, 500, -4),
            AdjustmentProfile::Reduced => (5000, 1000, -8),
            AdjustmentProfile::Minimal => (10000, 2000, -12),
        };

        BleAdjustment {
            scan_interval_ms: (*self.ble_scan_override.lock()).unwrap_or(scan_interval),
            advertise_interval_ms: advertise_interval,
            tx_power_dbm: tx_power,
        }
    }

    pub fn compute_relay_adjustment(&self, profile: AdjustmentProfile) -> RelayAdjustment {
        let (max_per_hour, priority_threshold, max_payload) = match profile {
            AdjustmentProfile::Maximum => (1000, 0, 65536),
            AdjustmentProfile::High => (500, 50, 32768),
            AdjustmentProfile::Standard => (200, 100, 16384),
            AdjustmentProfile::Reduced => (100, 150, 8192),
            AdjustmentProfile::Minimal => (50, 200, 4096),
        };

        RelayAdjustment {
            max_per_hour: (*self.relay_max_override.lock()).unwrap_or(max_per_hour),
            priority_threshold,
            max_payload_bytes: max_payload,
        }
    }

    pub fn override_ble_scan_interval(&self, interval_ms: u32) {
        *self.ble_scan_override.lock() = Some(interval_ms);
    }

    pub fn override_ble_advertise_interval(&self, interval_ms: Option<u16>) {
        // The bridge AutoAdjustEngine stores BLE overrides as a single scan interval.
        // Advertise interval is derived from the profile in compute_ble_adjustment,
        // so we store it alongside the scan override if both are present.
        // For now, map the advertise interval to the scan override field
        // since the bridge type only tracks one BLE interval override.
        if let Some(v) = interval_ms {
            *self.ble_scan_override.lock() = Some(v as u32);
        }
    }

    pub fn override_relay_max_per_hour(&self, max: u32) {
        *self.relay_max_override.lock() = Some(max);
    }

    pub fn override_relay_priority_threshold(&self, threshold: Option<u8>) {
        // Map priority threshold to relay max override. The bridge stores
        // relay overrides as max-per-hour. Higher thresholds mean fewer relays,
        // so we use a heuristic: threshold * 5 as the max relay count.
        if let Some(v) = threshold {
            *self.relay_max_override.lock() = Some(v as u32 * 5);
        }
    }

    pub fn clear_overrides(&self) {
        *self.ble_scan_override.lock() = None;
        *self.relay_max_override.lock() = None;
    }
}

// ============================================================================
// MESH SETTINGS MANAGER
// ============================================================================

#[derive(uniffi::Object)]
pub struct MeshSettingsManager {
    storage_path: std::path::PathBuf,
}

#[uniffi::export]
impl MeshSettingsManager {
    #[uniffi::constructor]
    pub fn new(storage_path: String) -> Self {
        Self {
            storage_path: std::path::PathBuf::from(storage_path),
        }
    }

    pub fn load(&self) -> Result<MeshSettings, crate::IronCoreError> {
        let settings_file = self.storage_path.join("mesh_settings.json");
        if settings_file.exists() {
            let data = std::fs::read_to_string(&settings_file)
                .map_err(|_| crate::IronCoreError::StorageError)?;
            let settings: MeshSettings =
                serde_json::from_str(&data).map_err(|_| crate::IronCoreError::Internal)?;
            Ok(settings)
        } else {
            Ok(MeshSettings::default())
        }
    }

    pub fn save(&self, settings: MeshSettings) -> Result<(), crate::IronCoreError> {
        self.validate(settings.clone())?;

        std::fs::create_dir_all(&self.storage_path)
            .map_err(|_| crate::IronCoreError::StorageError)?;

        let settings_file = self.storage_path.join("mesh_settings.json");
        let data =
            serde_json::to_string_pretty(&settings).map_err(|_| crate::IronCoreError::Internal)?;
        std::fs::write(&settings_file, data).map_err(|_| crate::IronCoreError::StorageError)?;

        Ok(())
    }

    pub fn validate(&self, settings: MeshSettings) -> Result<(), crate::IronCoreError> {
        // NOTE: relay_enabled controls BOTH sending and receiving
        // When false, ALL communication stops (bidirectional shutdown)
        // This enforces the relay=messaging principle in practice

        // If relay is enabled, max_relay_budget must be > 0
        if settings.relay_enabled && settings.max_relay_budget == 0 {
            return Err(crate::IronCoreError::InvalidInput);
        }

        // At least one transport must be enabled
        if !settings.ble_enabled
            && !settings.wifi_aware_enabled
            && !settings.wifi_direct_enabled
            && !settings.internet_enabled
        {
            return Err(crate::IronCoreError::InvalidInput);
        }

        // Battery floor must be reasonable
        if settings.battery_floor > 50 {
            return Err(crate::IronCoreError::InvalidInput);
        }

        Ok(())
    }

    pub fn default_settings(&self) -> MeshSettings {
        MeshSettings::default()
    }
}

// ============================================================================
// MESSAGE HISTORY
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageDirection {
    Sent,
    Received,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MessageStatus {
    #[default]
    Queued,
    InCustody,
    Sent,
    Delivered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: String,
    pub direction: MessageDirection,
    pub peer_id: String,
    pub content: String,
    pub timestamp: u64,
    #[serde(default)]
    pub sender_timestamp: u64,
    pub delivered: bool,
    #[serde(default)]
    pub status: MessageStatus,
    #[serde(default)]
    pub hidden: bool,
}

impl MessageRecord {
    fn adjust_legacy_timestamps(mut self) -> Self {
        if self.sender_timestamp == 0 {
            self.sender_timestamp = self.timestamp;
        }
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct HistoryStats {
    pub total_messages: u32,
    pub sent_count: u32,
    pub received_count: u32,
    pub undelivered_count: u32,
}

#[derive(uniffi::Object)]
pub struct HistoryManager {
    db: Arc<Mutex<sled::Db>>,
}

#[uniffi::export]
impl HistoryManager {
    #[uniffi::constructor]
    pub fn new(storage_path: String) -> Result<Self, crate::IronCoreError> {
        let path = std::path::PathBuf::from(storage_path).join("history.db");
        let db = sled::Config::default()
            .path(path)
            .mode(sled::Mode::LowSpace)
            .use_compression(false)
            .open()
            .map_err(|_| crate::IronCoreError::StorageError)?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    pub fn add(&self, record: MessageRecord) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock();
        let key = record.id.as_bytes();
        let value = serde_json::to_vec(&record).map_err(|_| crate::IronCoreError::Internal)?;
        db.insert(key, value)
            .map_err(|_| crate::IronCoreError::StorageError)?;
        Ok(())
    }

    pub fn get(&self, id: String) -> Result<Option<MessageRecord>, crate::IronCoreError> {
        let db = self.db.lock();
        if let Some(data) = db
            .get(id.as_bytes())
            .map_err(|_| crate::IronCoreError::StorageError)?
        {
            let record: MessageRecord =
                serde_json::from_slice(&data).map_err(|_| crate::IronCoreError::Internal)?;
            Ok(Some(record.adjust_legacy_timestamps()))
        } else {
            Ok(None)
        }
    }

    pub fn recent(
        &self,
        peer_filter: Option<String>,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, crate::IronCoreError> {
        self.recent_internal(peer_filter, limit, false)
    }

    /// Like `recent()` but also returns messages that are hidden due to the
    /// sender being blocked.  Used by administrative / evidentiary access paths.
    pub fn recent_including_hidden(
        &self,
        peer_filter: Option<String>,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, crate::IronCoreError> {
        self.recent_internal(peer_filter, limit, true)
    }

    fn recent_internal(
        &self,
        peer_filter: Option<String>,
        limit: u32,
        include_hidden: bool,
    ) -> Result<Vec<MessageRecord>, crate::IronCoreError> {
        let db = self.db.lock();
        let mut records = Vec::new();

        for item in db.iter() {
            let (_, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            let record = record.adjust_legacy_timestamps();

            // Evidentiary retention: skip hidden messages in normal queries.
            if record.hidden && !include_hidden {
                continue;
            }

            if let Some(ref peer) = peer_filter {
                if &record.peer_id == peer {
                    records.push(record);
                }
            } else {
                records.push(record);
            }
        }

        // Do not rely on sled key order (message IDs are not time-ordered).
        // Sort explicitly so callers receive newest records first.
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| b.id.cmp(&a.id)));
        if records.len() > limit as usize {
            records.truncate(limit as usize);
        }

        Ok(records)
    }

    pub fn conversation(
        &self,
        peer_id: String,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, crate::IronCoreError> {
        self.recent(Some(peer_id), limit)
    }

    pub fn remove_conversation(&self, peer_id: String) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock();
        let mut keys_to_remove = Vec::new();

        for item in db.iter() {
            let (key, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            let record = record.adjust_legacy_timestamps();

            if record.peer_id.eq_ignore_ascii_case(&peer_id) {
                keys_to_remove.push(key);
            }
        }

        for key in keys_to_remove {
            db.remove(key)
                .map_err(|_| crate::IronCoreError::StorageError)?;
        }

        Ok(())
    }

    pub fn search(
        &self,
        query: String,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, crate::IronCoreError> {
        let db = self.db.lock();
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for item in db.iter() {
            if results.len() >= limit as usize {
                break;
            }

            let (_, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            let record = record.adjust_legacy_timestamps();

            // Evidentiary retention: skip hidden messages in search results.
            if record.hidden {
                continue;
            }

            if record.content.to_lowercase().contains(&query_lower) {
                results.push(record);
            }
        }

        Ok(results)
    }

    /// Unhide all stored messages for a given peer (called on unblock).
    pub fn unhide_messages_for_peer(&self, peer_id: String) -> Result<u32, crate::IronCoreError> {
        let db = self.db.lock();
        let mut to_update: Vec<(Vec<u8>, MessageRecord)> = Vec::new();

        for item in db.iter() {
            let (key, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            if record.hidden && record.peer_id.eq_ignore_ascii_case(&peer_id) {
                to_update.push((key.to_vec(), record));
            }
        }

        let count = to_update.len() as u32;
        for (key, mut record) in to_update {
            record.hidden = false;
            let updated =
                serde_json::to_vec(&record).map_err(|_| crate::IronCoreError::Internal)?;
            db.insert(key, updated)
                .map_err(|_| crate::IronCoreError::StorageError)?;
        }
        Ok(count)
    }

    /// Hide all stored messages for a given peer (called on block).
    pub fn hide_messages_for_peer(&self, peer_id: String) -> Result<u32, crate::IronCoreError> {
        let db = self.db.lock();
        let mut to_update: Vec<(Vec<u8>, MessageRecord)> = Vec::new();

        for item in db.iter() {
            let (key, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            if !record.hidden && record.peer_id.eq_ignore_ascii_case(&peer_id) {
                to_update.push((key.to_vec(), record));
            }
        }

        let count = to_update.len() as u32;
        for (key, mut record) in to_update {
            record.hidden = true;
            let updated =
                serde_json::to_vec(&record).map_err(|_| crate::IronCoreError::Internal)?;
            db.insert(key, updated)
                .map_err(|_| crate::IronCoreError::StorageError)?;
        }
        Ok(count)
    }

    pub fn mark_delivered(&self, id: String) -> Result<(), crate::IronCoreError> {
        if let Some(mut record) = self.get(id.clone())? {
            record.delivered = true;
            self.add(record)?;
        }
        Ok(())
    }

    pub fn clear(&self) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock();
        db.clear().map_err(|_| crate::IronCoreError::StorageError)?;
        Ok(())
    }

    pub fn clear_conversation(&self, peer_id: String) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock();
        let mut to_delete = Vec::new();

        for item in db.iter() {
            let (key, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            let record = record.adjust_legacy_timestamps();
            // P0_SECURITY_001: Case-insensitive peer ID matching to match generic HistoryManager behavior
            if record.peer_id.eq_ignore_ascii_case(&peer_id) {
                to_delete.push(key.to_vec());
            }
        }

        for key in to_delete {
            db.remove(key)
                .map_err(|_| crate::IronCoreError::StorageError)?;
        }

        Ok(())
    }

    pub fn stats(&self) -> Result<HistoryStats, crate::IronCoreError> {
        let db = self.db.lock();
        let mut stats = HistoryStats::default();

        for item in db.iter() {
            let (_, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            let record = record.adjust_legacy_timestamps();

            stats.total_messages += 1;
            match record.direction {
                MessageDirection::Sent => stats.sent_count += 1,
                MessageDirection::Received => stats.received_count += 1,
            }
            if !record.delivered {
                stats.undelivered_count += 1;
            }
        }

        Ok(stats)
    }

    pub fn count(&self) -> u32 {
        let db = self.db.lock();
        db.len() as u32
    }

    pub fn flush(&self) {
        let db = self.db.lock();
        let _ = db.flush();
    }

    /// Enforce a maximum message retention cap.
    ///
    /// Keeps the `max_messages` most recent messages (by timestamp) and
    /// removes the rest.  Returns the number of pruned records.
    pub fn enforce_retention(&self, max_messages: u32) -> Result<u32, crate::IronCoreError> {
        let db = self.db.lock();
        let total = db.len();
        if total <= max_messages as usize {
            return Ok(0);
        }

        // Collect all (key, timestamp) pairs
        let mut entries: Vec<(Vec<u8>, u64)> = Vec::with_capacity(total);
        for item in db.iter() {
            let (key, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            entries.push((key.to_vec(), record.timestamp));
        }

        // Sort by timestamp descending (newest first)
        entries.sort_by_key(|b| std::cmp::Reverse(b.1));

        // Remove everything after max_messages
        let mut pruned: u32 = 0;
        for (key, _) in entries.into_iter().skip(max_messages as usize) {
            db.remove(key)
                .map_err(|_| crate::IronCoreError::StorageError)?;
            pruned += 1;
        }

        Ok(pruned)
    }

    /// Remove all messages with timestamp before the given Unix epoch seconds.
    ///
    /// Returns the number of pruned records.
    pub fn prune_before(&self, before_timestamp: u64) -> Result<u32, crate::IronCoreError> {
        let db = self.db.lock();
        let mut keys_to_remove = Vec::new();

        for item in db.iter() {
            let (key, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            if record.timestamp < before_timestamp {
                keys_to_remove.push(key.to_vec());
            }
        }

        let pruned = keys_to_remove.len() as u32;
        for key in keys_to_remove {
            db.remove(key)
                .map_err(|_| crate::IronCoreError::StorageError)?;
        }

        Ok(pruned)
    }

    pub fn delete(&self, id: String) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock();
        db.remove(id.as_bytes())
            .map_err(|_| crate::IronCoreError::StorageError)?;
        Ok(())
    }
}

// ============================================================================
// CONNECTION LEDGER
// ============================================================================
// LedgerEntry and LedgerManager have been moved to crate::store::ledger_entry

// ============================================================================
// SWARM BRIDGE
// ============================================================================

/// Bridge between UniFFI (synchronous) and SwarmHandle (async).
///
/// This bridge provides synchronous wrappers around async SwarmHandle operations
/// using tokio::runtime::Handle to block on futures when necessary.
#[derive(uniffi::Object)]
pub struct SwarmBridge {
    handle: Arc<Mutex<Option<SwarmHandle>>>,
    captured_handle: Option<tokio::runtime::Handle>,
    /// Shared BLE peer set from MeshService for dual-stack delivery.
    pub nearby_ble_peers: Arc<Mutex<HashSet<String>>>,
    /// Callback for dispatching BLE packets to the platform layer.
    #[allow(clippy::type_complexity)]
    dispatch_ble_fn: Arc<Mutex<Option<Arc<dyn Fn(String, Vec<u8>) + Send + Sync>>>>,
    /// Callback for dispatching proximity packets (any transport) to the platform layer.
    #[allow(clippy::type_complexity)]
    dispatch_proximity_fn:
        Arc<Mutex<Option<Arc<dyn Fn(String, ProximityTransport, Vec<u8>) + Send + Sync>>>>,
}

impl Default for SwarmBridge {
    fn default() -> Self {
        Self::new()
    }
}
// [CRITICAL] Global runtime for network operations on mobile.
// We need this because many mobile callback threads aren't in a tokio context.
static GLOBAL_RT: parking_lot::RwLock<Option<tokio::runtime::Runtime>> =
    parking_lot::RwLock::new(None);

fn get_global_runtime() -> tokio::runtime::Handle {
    let rt_read = GLOBAL_RT.read();
    if let Some(rt) = &*rt_read {
        return rt.handle().clone();
    }
    drop(rt_read);

    let mut rt_write = GLOBAL_RT.write();
    if let Some(rt) = &*rt_write {
        return rt.handle().clone();
    }

    tracing::info!("Initializing global Tokio runtime for mobile mesh...");
    #[cfg(not(target_arch = "wasm32"))]
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            tracing::error!(
                "Failed to create multi-thread Tokio runtime: {}, falling back to current-thread",
                e
            );
            // SAFETY: Tokio runtime initialization is critical infrastructure. If both multi-thread and
            // current-thread runtimes fail to initialize, the application cannot function and must panic.
            // This fallback path should be extremely rare and indicates a system-level resource exhaustion.
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create any Tokio runtime — critical failure")
        }
    };

    #[cfg(target_arch = "wasm32")]
    let rt = {
        // SAFETY: Tokio runtime initialization is critical infrastructure. On WASM, only a current-thread
        // runtime is available. If this fails, the application cannot function and must panic.
        // This should be extremely rare and indicates a system-level initialization failure.
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime on WASM")
    };
    let handle = rt.handle().clone();
    *rt_write = Some(rt);
    handle
}

#[uniffi::export]
impl SwarmBridge {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            handle: Arc::new(Mutex::new(None)),
            captured_handle: Some(get_global_runtime()),
            nearby_ble_peers: Arc::new(Mutex::new(HashSet::new())),
            dispatch_ble_fn: Arc::new(Mutex::new(None)),
            dispatch_proximity_fn: Arc::new(Mutex::new(None)),
        }
    }

    /// Send an encrypted message envelope to a peer.
    ///
    /// `recipient_identity_id` and `intended_device_id` are WS13 tight-pair metadata.
    /// Pass `None` for both if the caller has no device record for the recipient.
    ///
    /// Async FFI (Issue 5): exported to Kotlin as a `suspend fun`; awaits the
    /// swarm reply instead of parking the calling thread in `block_on`.
    pub async fn send_message(
        &self,
        peer_id: String,
        data: Vec<u8>,
        recipient_identity_id: Option<String>,
        intended_device_id: Option<String>,
    ) -> Result<(), crate::IronCoreError> {
        // Clone handle and drop guard before awaiting to prevent deadlock
        let handle = self
            .handle
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NetworkError)?;

        // Parse peer ID
        let peer_id_parsed =
            PeerId::from_str(&peer_id).map_err(|_| crate::IronCoreError::InvalidInput)?;

        // P0_MESH_004: Dual-stack delivery via BLE if peer is nearby
        if self.nearby_ble_peers.lock().contains(&peer_id) {
            tracing::info!(
                "Dual-stack delivery: sending message to {} via BLE",
                peer_id
            );
            self.dispatch_ble_packet(peer_id, data.clone());
        }

        handle
            .send_message(
                peer_id_parsed,
                data,
                recipient_identity_id,
                intended_device_id,
            )
            .await
            .map_err(|_| crate::IronCoreError::NetworkError)
    }

    /// Send an encrypted message envelope and return the raw swarm error string
    /// on failure so adapters can classify retryable vs terminal rejection.
    ///
    /// Async FFI (Issue 5): exported to Kotlin as a `suspend fun`.
    pub async fn send_message_status(
        &self,
        peer_id: String,
        data: Vec<u8>,
        recipient_identity_id: Option<String>,
        intended_device_id: Option<String>,
    ) -> Option<String> {
        let handle = match self.handle.lock().clone() {
            Some(handle) => handle,
            None => return Some("swarm_bridge_unavailable".to_string()),
        };

        let peer_id_parsed = match PeerId::from_str(&peer_id) {
            Ok(peer_id) => peer_id,
            Err(_) => return Some("invalid_peer_id".to_string()),
        };

        // P0_MESH_004: Dual-stack delivery via BLE if peer is nearby
        if self.nearby_ble_peers.lock().contains(&peer_id) {
            tracing::info!(
                "Dual-stack delivery: sending message to {} via BLE",
                peer_id
            );
            self.dispatch_ble_packet(peer_id, data.clone());
        }

        handle
            .send_message(
                peer_id_parsed,
                data,
                recipient_identity_id,
                intended_device_id,
            )
            .await
            .err()
            .map(|err| err.to_string())
    }

    /// Send an encrypted message envelope to ALL connected peers.
    /// Since messages are encrypted for a specific recipient, broadcasting to all peers is safe.
    /// Only the intended recipient can decrypt the payload.
    ///
    /// Async FFI (Issue 5): exported to Kotlin as a `suspend fun`.
    pub async fn send_to_all_peers(&self, data: Vec<u8>) -> Result<(), crate::IronCoreError> {
        let handle = self
            .handle
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NetworkError)?;

        let peers = handle.get_peers().await.unwrap_or_default();

        // P0_MESH_004: Dual-stack broadcast via BLE
        let ble_peers = self.nearby_ble_peers.lock().clone();
        for peer_id in ble_peers {
            tracing::info!("Broadcasting message to {} via BLE", peer_id);
            self.dispatch_ble_packet(peer_id, data.clone());
        }

        let mut sent = 0usize;
        for peer_id in peers {
            match handle.send_message(peer_id, data.clone(), None, None).await {
                Ok(()) => sent += 1,
                Err(e) => {
                    tracing::warn!("send_to_all_peers: failed to send to {}: {:?}", peer_id, e)
                }
            }
        }

        // We count success if at least one peer (of either transport) was reachable.
        // Actually, we don't track success of dispatch_ble_packet because it's a bridge call.

        tracing::info!("send_to_all_peers: sent to {} libp2p peers", sent);
        Ok(())
    }

    /// Dial a peer at a multiaddress.
    ///
    /// Async FFI (Issue 5): exported to Kotlin as a `suspend fun`. Safe to
    /// call from any context, including `rt.spawn`'d futures — it awaits the
    /// dial instead of blocking the calling thread (the old sync version
    /// panicked with "Cannot start a runtime from within a runtime" there).
    pub async fn dial(&self, multiaddr: String) -> Result<(), crate::IronCoreError> {
        let handle = self
            .handle
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NetworkError)?;

        let addr =
            Multiaddr::from_str(&multiaddr).map_err(|_| crate::IronCoreError::InvalidInput)?;

        handle.dial(addr).await.map_err(|e| {
            let err_str = e.to_string().to_lowercase();
            if err_str.contains("dialing self") || err_str.contains("dialself") {
                crate::IronCoreError::DialSelf
            } else if err_str.contains("no addresses") || err_str.contains("noaddresses") {
                crate::IronCoreError::NoAddresses
            } else if err_str.contains("connection limit") || err_str.contains("connectionlimit") {
                crate::IronCoreError::ConnectionLimit
            } else if err_str.contains("not supported") || err_str.contains("multiaddrnotsupported")
            {
                crate::IronCoreError::MultiaddrNotSupported
            } else if err_str.contains("io") {
                crate::IronCoreError::IoError
            } else {
                crate::IronCoreError::NetworkError
            }
        })
    }

    pub async fn get_peers(&self) -> Vec<String> {
        let handle = match self.handle.lock().clone() {
            Some(h) => h,
            None => return Vec::new(),
        };

        handle
            .get_peers()
            .await
            .unwrap_or_default()
            .iter()
            .map(|peer_id| peer_id.to_string())
            .collect()
    }

    pub async fn get_listeners(&self) -> Vec<String> {
        let handle = match self.handle.lock().clone() {
            Some(h) => h,
            None => return Vec::new(),
        };

        handle
            .get_listeners()
            .await
            .unwrap_or_default()
            .iter()
            .map(|addr| addr.to_string())
            .collect()
    }

    pub async fn get_external_addresses(&self) -> Vec<String> {
        let handle = match self.handle.lock().clone() {
            Some(h) => h,
            None => return Vec::new(),
        };

        handle
            .get_external_addresses()
            .await
            .unwrap_or_default()
            .iter()
            .map(|addr| addr.to_string())
            .collect()
    }

    pub async fn get_topics(&self) -> Vec<String> {
        let handle = match self.handle.lock().clone() {
            Some(h) => h,
            None => return Vec::new(),
        };

        handle.get_topics().await.unwrap_or_default()
    }

    /// Subscribe to a Gossipsub topic.
    pub async fn subscribe_topic(&self, topic: String) -> Result<(), crate::IronCoreError> {
        let handle = self
            .handle
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NetworkError)?;

        handle
            .subscribe_topic(topic)
            .await
            .map_err(|_| crate::IronCoreError::NetworkError)
    }

    pub async fn unsubscribe_topic(&self, topic: String) -> Result<(), crate::IronCoreError> {
        let handle = self
            .handle
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NetworkError)?;

        handle
            .unsubscribe_topic(topic)
            .await
            .map_err(|_| crate::IronCoreError::NetworkError)
    }

    pub async fn publish_topic(
        &self,
        topic: String,
        data: Vec<u8>,
    ) -> Result<(), crate::IronCoreError> {
        let handle = self
            .handle
            .lock()
            .clone()
            .ok_or(crate::IronCoreError::NetworkError)?;

        handle
            .publish_topic(topic, data)
            .await
            .map_err(|_| crate::IronCoreError::NetworkError)
    }

    pub async fn shutdown(&self) {
        // Bind the clone to a local first: an `if let` on `.lock().clone()`
        // would keep the (non-Send) guard alive across the await.
        let handle = self.handle.lock().clone();
        if let Some(handle) = handle {
            let _ = handle.shutdown().await;
        }
    }
}

// Non-UniFFI internal methods for SwarmBridge
impl SwarmBridge {
    /// Set the SwarmHandle for this bridge.
    /// This must be called after starting the swarm to wire up network operations.
    pub fn set_handle(&self, handle: SwarmHandle) {
        *self.handle.lock() = Some(handle);
    }

    /// Internal helper to get the runtime handle for spawning
    pub fn get_runtime_handle(&self) -> tokio::runtime::Handle {
        self.captured_handle
            .clone()
            .unwrap_or_else(get_global_runtime)
    }

    // ------------------------------------------------------------------
    // Blocking snapshots for internal *synchronous* Rust callers only
    // (diagnostics/state queries invoked from sync FFI paths like
    // get_connection_path_state / export_diagnostics / MeshService::stop).
    // These MUST NOT be called from an async context — they block the
    // calling thread on the swarm reply. The FFI surface itself is async
    // (Issue 5); do not re-export these through UniFFI.
    // ------------------------------------------------------------------

    pub(crate) fn get_peers_blocking(&self) -> Vec<String> {
        let handle = match self.handle.lock().clone() {
            Some(h) => h,
            None => return Vec::new(),
        };
        let rt = self.get_runtime_handle();
        rt.block_on(handle.get_peers())
            .unwrap_or_default()
            .iter()
            .map(|peer_id| peer_id.to_string())
            .collect()
    }

    pub(crate) fn get_listeners_blocking(&self) -> Vec<String> {
        let handle = match self.handle.lock().clone() {
            Some(h) => h,
            None => return Vec::new(),
        };
        let rt = self.get_runtime_handle();
        rt.block_on(handle.get_listeners())
            .unwrap_or_default()
            .iter()
            .map(|addr| addr.to_string())
            .collect()
    }

    pub(crate) fn get_external_addresses_blocking(&self) -> Vec<String> {
        let handle = match self.handle.lock().clone() {
            Some(h) => h,
            None => return Vec::new(),
        };
        let rt = self.get_runtime_handle();
        rt.block_on(handle.get_external_addresses())
            .unwrap_or_default()
            .iter()
            .map(|addr| addr.to_string())
            .collect()
    }

    #[allow(dead_code)]
    pub(crate) fn get_topics_blocking(&self) -> Vec<String> {
        let handle = match self.handle.lock().clone() {
            Some(h) => h,
            None => return Vec::new(),
        };
        let rt = self.get_runtime_handle();
        rt.block_on(handle.get_topics()).unwrap_or_default()
    }

    pub(crate) fn shutdown_blocking(&self) {
        if let Some(handle) = self.handle.lock().clone() {
            let rt = self.get_runtime_handle();
            let _ = rt.block_on(handle.shutdown());
        }
    }

    /// Dispatch a BLE packet to the platform layer.
    pub fn dispatch_ble_packet(&self, peer_id: String, data: Vec<u8>) {
        self.dispatch_proximity_packet(peer_id, ProximityTransport::Ble, data);
    }

    /// Dispatch a proximity packet via any transport to the platform layer.
    pub fn dispatch_proximity_packet(
        &self,
        peer_id: String,
        transport: ProximityTransport,
        data: Vec<u8>,
    ) {
        if let Some(ref f) = *self.dispatch_proximity_fn.lock() {
            f(peer_id, transport, data);
        } else if let Some(ref f) = *self.dispatch_ble_fn.lock() {
            // Fallback: if only BLE callback is set, use it for BLE transport
            if transport == ProximityTransport::Ble {
                f(peer_id, data);
            }
        }
    }

    /// Set the BLE dispatch callback.
    #[allow(clippy::type_complexity)]
    pub fn set_dispatch_ble_fn(&self, f: Option<Arc<dyn Fn(String, Vec<u8>) + Send + Sync>>) {
        *self.dispatch_ble_fn.lock() = f;
    }

    /// Set the proximity dispatch callback (supports all transports).
    #[allow(clippy::type_complexity)]
    pub fn set_dispatch_proximity_fn(
        &self,
        f: Option<Arc<dyn Fn(String, ProximityTransport, Vec<u8>) + Send + Sync>>,
    ) {
        *self.dispatch_proximity_fn.lock() = f;
    }
}

static ESCALATION_ENGINE: std::sync::OnceLock<Arc<crate::transport::escalation::EscalationEngine>> =
    std::sync::OnceLock::new();

fn get_escalation_engine() -> &'static Arc<crate::transport::escalation::EscalationEngine> {
    ESCALATION_ENGINE.get_or_init(|| {
        Arc::new(crate::transport::escalation::EscalationEngine::new(
            crate::transport::escalation::EscalationPolicy::Balanced,
        ))
    })
}

/// Get the recommended proximity transport for a peer based on current state.
/// Consults the EscalationEngine when available, falls back to BLE.
#[uniffi::export]
pub fn recommended_transport(peer_id: String) -> ProximityTransport {
    // Parse peer_id as bytes for EscalationEngine lookup
    if let Ok(bytes) = hex::decode(&peer_id) {
        if bytes.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            let engine = get_escalation_engine();
            if let Some(t) = engine.current_transport(arr) {
                return match t {
                    crate::transport::abstraction::TransportType::BLE => ProximityTransport::Ble,
                    crate::transport::abstraction::TransportType::WiFiAware => {
                        ProximityTransport::WifiAware
                    }
                    crate::transport::abstraction::TransportType::WiFiDirect => {
                        ProximityTransport::WifiDirect
                    }
                    _ => ProximityTransport::Ble,
                };
            }
        }
    }
    ProximityTransport::Ble
}

/// Update the available transports list for a peer in the authoritative EscalationEngine.
#[uniffi::export]
pub fn update_peer_transports(peer_id: String, transports: Vec<ProximityTransport>) {
    if let Ok(bytes) = hex::decode(&peer_id) {
        if bytes.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            let core_transports: Vec<crate::transport::abstraction::TransportType> = transports
                .iter()
                .map(|t| match t {
                    ProximityTransport::Ble => crate::transport::abstraction::TransportType::BLE,
                    ProximityTransport::WifiAware => {
                        crate::transport::abstraction::TransportType::WiFiAware
                    }
                    ProximityTransport::WifiDirect => {
                        crate::transport::abstraction::TransportType::WiFiDirect
                    }
                    ProximityTransport::Multipeer => {
                        crate::transport::abstraction::TransportType::Internet
                    }
                })
                .collect();
            let engine = get_escalation_engine();
            if engine.init_peer(arr, core_transports.clone()).is_err() {
                let _ = engine.update_available_transports(arr, core_transports);
            }
        }
    }
}

/// Generate a Signal-style safety number from two public keys (Ed25519 hex).
/// Returns a 60-digit numeric string. Order-independent so both sides match.
/// Returns an empty string if either key is malformed - an all-zero fallback
/// looked like a real (matching) safety number that a user could "verify",
/// which is unsafe for a value whose entire purpose is tamper detection.
/// Callers must treat "" as an error state, not a value to display.
#[uniffi::export]
pub fn safety_number(our_pubkey_hex: String, their_pubkey_hex: String) -> String {
    crate::identity::keys::safety_number(&our_pubkey_hex, &their_pubkey_hex).unwrap_or_default()
}

// ============================================================================
// UNIFFI EXPORTS: ABUSE ENGINE & DEVICE PROTECTION
// ============================================================================

/// Check if a peer is exempt from auto-blocking.
#[uniffi::export]
pub fn auto_block_is_exempt(peer_id: String) -> bool {
    let core = crate::IronCore::new();
    core.auto_block_is_exempt(&peer_id)
}

/// Exclude a peer from automatic blocking rules.
#[uniffi::export]
pub fn auto_block_exempt_peer(peer_id: String) {
    let core = crate::IronCore::new();
    core.auto_block_exempt_peer(peer_id);
}

/// Remove a peer from the auto-block exemption list.
#[uniffi::export]
pub fn auto_block_unexempt_peer(peer_id: String) {
    let core = crate::IronCore::new();
    core.auto_block_unexempt_peer(&peer_id);
}

/// Get the current reputation score for a peer (0.0 to 100.0).
#[uniffi::export]
pub fn get_reputation_score(peer_id: String) -> f64 {
    let core = crate::IronCore::new();
    core.get_reputation_score(&peer_id)
}

/// Check if a peer's reputation score is within suspicious bounds.
#[uniffi::export]
pub fn is_peer_suspicious(peer_id: String) -> bool {
    let core = crate::IronCore::new();
    core.is_peer_suspicious(&peer_id)
}

/// Check if a peer's reputation score is abusive.
#[uniffi::export]
pub fn is_peer_abusive(peer_id: String) -> bool {
    let core = crate::IronCore::new();
    core.is_peer_abusive(&peer_id)
}

/// Get spam confidence score for a peer (0.0 to 1.0).
#[uniffi::export]
pub fn detect_spam_confidence(peer_id: String) -> f64 {
    let core = crate::IronCore::new();
    core.detect_spam_confidence(&peer_id)
}

/// Inspect binary envelope data to determine if content violates spam heuristics.
#[uniffi::export]
pub fn is_content_suspicious(envelope_data: Vec<u8>) -> bool {
    let core = crate::IronCore::new();
    core.is_content_suspicious(&envelope_data)
}

/// Prune stale peer records from the spam detection engine.
#[uniffi::export]
pub fn prune_stale_spam_peers(max_entries: u32) -> u32 {
    let core = crate::IronCore::new();
    core.prune_stale_spam_peers(max_entries as usize) as u32
}

/// Associate a device ID with a peer ID in the blocked manager.
#[uniffi::export]
pub fn register_device_id(peer_id: String, device_id: String) -> bool {
    let core = crate::IronCore::new();
    core.register_device_id(&peer_id, &device_id).is_ok()
}

/// Retrieve all known device IDs associated with a peer ID.
#[uniffi::export]
pub fn get_known_devices(peer_id: String) -> Vec<String> {
    let core = crate::IronCore::new();
    core.get_known_devices(&peer_id)
}

/// Check if a specific device ID has been blocked.
#[uniffi::export]
pub fn is_device_blocked(peer_id: String, device_id: String) -> bool {
    let core = crate::IronCore::new();
    core.is_device_blocked(&peer_id, &device_id)
}

fn current_timestamp() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ledger_entry::LedgerManager;
    use tempfile::tempdir;

    // -----------------------------------------------------------------------
    // DeviceState / BehaviorAdjustment tests
    // -----------------------------------------------------------------------

    fn make_state(battery: u8, charging: bool, motion: MotionState) -> DeviceState {
        DeviceState {
            battery_level: battery,
            is_charging: charging,
            network_type: NetworkType::Wifi,
            motion_state: motion,
        }
    }

    // -----------------------------------------------------------------------
    // safety_number (S5)
    // -----------------------------------------------------------------------

    /// S5: a malformed key must produce an empty string, not an all-zero
    /// 60-digit number that looks like a real (matching) safety number a
    /// user could mistakenly "verify".
    #[test]
    fn test_safety_number_returns_empty_string_on_malformed_keys() {
        assert_eq!(safety_number("not-hex".to_string(), "junk".to_string()), "");
    }

    #[test]
    fn test_safety_number_is_order_independent_for_valid_keys() {
        let a = hex::encode([1u8; 32]);
        let b = hex::encode([2u8; 32]);

        let forward = safety_number(a.clone(), b.clone());
        let backward = safety_number(b, a);

        assert!(!forward.is_empty());
        assert_eq!(forward, backward);
    }

    #[test]
    fn test_fresh_install_without_identity_resolves_headless_mode_with_persisted_key() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();

        let service = Arc::new(MeshService::with_storage(
            MeshServiceConfig {
                discovery_interval_ms: 5_000,
                battery_floor_pct: 20,
            },
            path.clone(),
        ));
        service.clone().start().unwrap();

        let (first_keypair, first_headless) = service.resolve_swarm_keypair_and_mode().unwrap();
        assert!(
            first_headless,
            "fresh install should default to headless mode"
        );

        let key_path = std::path::Path::new(&path).join("relay_network_key.pb");
        assert!(
            key_path.exists(),
            "headless key should persist on first resolve"
        );
        service.stop();

        let reloaded = Arc::new(MeshService::with_storage(
            MeshServiceConfig {
                discovery_interval_ms: 5_000,
                battery_floor_pct: 20,
            },
            path,
        ));
        reloaded.clone().start().unwrap();
        let (second_keypair, second_headless) = reloaded.resolve_swarm_keypair_and_mode().unwrap();
        assert!(second_headless);
        assert_eq!(
            first_keypair.public().to_peer_id(),
            second_keypair.public().to_peer_id(),
            "headless key should be stable across restarts"
        );
    }

    #[test]
    fn start_swarm_success_installs_bridge_handle_before_returning() {
        let dir = tempdir().unwrap();
        let service = Arc::new(MeshService::with_storage(
            MeshServiceConfig {
                discovery_interval_ms: 5_000,
                battery_floor_pct: 20,
            },
            dir.path().to_str().unwrap().to_string(),
        ));
        service.clone().start().unwrap();

        service
            .start_swarm("/ip4/127.0.0.1/tcp/0".to_string(), Vec::new())
            .expect("a successful swarm start must wait for the bridge handle");

        assert!(
            service.get_swarm_bridge().handle.lock().is_some(),
            "start_swarm must not report success before SwarmBridge is usable"
        );
        service.stop();
    }

    // -----------------------------------------------------------------------
    // PlatformWifiAwareBridge::handle_data_path_confirmed tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_handle_data_path_confirmed_resolves_ipv4() {
        let bridge = PlatformWifiAwareBridge::new_platform_ref(Arc::new(Mutex::new(None)));
        let (tx, rx) = tokio::sync::oneshot::channel();
        bridge
            .data_path_results
            .lock()
            .insert("peer-1".to_string(), tx);

        bridge.handle_data_path_confirmed("peer-1".to_string(), "127.0.0.1".to_string(), 4242);

        let addr = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(rx)
            .expect("oneshot must resolve");
        assert_eq!(addr, "127.0.0.1:4242".parse().unwrap());
    }

    #[test]
    fn test_handle_data_path_confirmed_resolves_ipv6_link_local() {
        // Regression test: building SocketAddr via format!("{ip}:{port}") and
        // parsing the whole string fails for any IPv6 address (needs
        // "[ip]:port" bracket syntax), so this used to silently swallow every
        // WiFi Aware confirmation with an IPv6 address and time out.
        let bridge = PlatformWifiAwareBridge::new_platform_ref(Arc::new(Mutex::new(None)));
        let (tx, rx) = tokio::sync::oneshot::channel();
        bridge
            .data_path_results
            .lock()
            .insert("peer-2".to_string(), tx);

        bridge.handle_data_path_confirmed("peer-2".to_string(), "fe80::1234".to_string(), 8765);

        let addr = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(rx)
            .expect("oneshot must resolve for an IPv6 address");
        assert_eq!(addr, "[fe80::1234]:8765".parse().unwrap());
    }

    #[test]
    fn test_handle_data_path_confirmed_ignores_unparseable_ip_without_panicking() {
        let bridge = PlatformWifiAwareBridge::new_platform_ref(Arc::new(Mutex::new(None)));
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        bridge
            .data_path_results
            .lock()
            .insert("peer-3".to_string(), tx);

        bridge.handle_data_path_confirmed("peer-3".to_string(), "not-an-ip".to_string(), 1);

        // A malformed IP must not resolve (or drop) the pending confirmation:
        // the sender is left in place in data_path_results, matching the
        // original pre-fix behavior for a parse failure.
        assert!(
            rx.try_recv().is_err(),
            "malformed IP must not resolve the pending confirmation"
        );
        assert!(bridge.data_path_results.lock().contains_key("peer-3"));
    }

    #[test]
    fn test_identity_creation_upgrades_resolved_mode_from_headless_to_full() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();

        let service = Arc::new(MeshService::with_storage(
            MeshServiceConfig {
                discovery_interval_ms: 5_000,
                battery_floor_pct: 20,
            },
            path,
        ));
        service.clone().start().unwrap();

        let (_, headless_before) = service.resolve_swarm_keypair_and_mode().unwrap();
        assert!(headless_before);

        let core = service
            .get_core()
            .expect("core should be available after start");
        core.grant_consent();
        core.initialize_identity().unwrap();

        let (full_keypair, headless_after) = service.resolve_swarm_keypair_and_mode().unwrap();
        assert!(
            !headless_after,
            "identity initialization should upgrade to full mode"
        );

        let identity_keypair = core.get_libp2p_keypair().unwrap();
        assert_eq!(
            full_keypair.public().to_peer_id(),
            identity_keypair.public().to_peer_id(),
            "full mode should use identity-derived libp2p keypair"
        );
    }

    #[test]
    fn test_connection_path_state_disconnected_by_default() {
        let service = MeshService::new(MeshServiceConfig {
            discovery_interval_ms: 5_000,
            battery_floor_pct: 20,
        });

        *service.swarm_headless_mode.lock() = Some(true);
        let headless_state = service.get_connection_path_state();

        *service.swarm_headless_mode.lock() = Some(false);
        let full_state = service.get_connection_path_state();

        assert_eq!(
            headless_state, full_state,
            "connection-path semantics should not differ by role mode"
        );
        assert_eq!(headless_state, ConnectionPathState::Disconnected);
    }

    #[test]
    fn test_compute_behavior_minimal_mode() {
        // <= 10% and not charging → minimal operation
        let adj = MeshService::compute_behavior(&make_state(10, false, MotionState::Still));
        assert!(adj.minimal_operation);
        assert!(!adj.relay_enabled);
        assert_eq!(adj.relay_budget, 0);
        assert!(adj.scan_interval_ms >= 10_000);

        // Charging saves it even at 5%
        let adj_charging = MeshService::compute_behavior(&make_state(5, true, MotionState::Still));
        assert!(!adj_charging.minimal_operation);
    }

    #[test]
    fn test_compute_behavior_low_battery() {
        // 20% not charging → no relay, not minimal
        let adj = MeshService::compute_behavior(&make_state(20, false, MotionState::Walking));
        assert!(!adj.minimal_operation);
        assert!(!adj.relay_enabled);
        assert_eq!(adj.relay_budget, 0);
        assert!(adj.scan_interval_ms > 2_000);

        // 21% not charging → normal
        let adj21 = MeshService::compute_behavior(&make_state(21, false, MotionState::Walking));
        assert!(adj21.relay_enabled);
    }

    #[test]
    fn test_compute_behavior_stationary_good_battery() {
        // Stationary + battery >= 50 → maximum relay
        let adj = MeshService::compute_behavior(&make_state(60, false, MotionState::Still));
        assert!(adj.relay_enabled);
        assert_eq!(adj.relay_budget, 200);
        assert!(adj.scan_interval_ms <= 500);
    }

    #[test]
    fn test_compute_behavior_charging_always_full() {
        // Charging at any battery level → full relay
        let adj = MeshService::compute_behavior(&make_state(15, true, MotionState::Automotive));
        assert!(adj.relay_enabled);
        assert_eq!(adj.relay_budget, 200);
    }

    #[test]
    fn test_compute_behavior_normal_operation() {
        // 30% not charging, moving → normal
        let adj = MeshService::compute_behavior(&make_state(30, false, MotionState::Walking));
        assert!(adj.relay_enabled);
        assert_eq!(adj.relay_budget, 100);
        assert_eq!(adj.scan_interval_ms, 2_000);
    }

    #[test]
    fn test_device_state_from_profile() {
        let profile = DeviceProfile {
            battery_pct: 55,
            is_charging: false,
            has_wifi: true,
            motion_state: MotionState::Still,
            peer_id: None,
            device_id: None,
        };
        let state = DeviceState::from_profile(&profile);
        assert_eq!(state.battery_level, 55);
        assert!(!state.is_charging);
        assert_eq!(state.network_type, NetworkType::Wifi);
        assert_eq!(state.motion_state, MotionState::Still);
    }

    #[test]
    fn test_update_device_state_stores_state() {
        let svc = MeshService::new(MeshServiceConfig {
            discovery_interval_ms: 1000,
            battery_floor_pct: 20,
        });

        assert!(svc.get_device_state().is_none());
        assert!(svc.recommended_behavior().is_none());

        let profile = DeviceProfile {
            battery_pct: 80,
            is_charging: false,
            has_wifi: true,
            motion_state: MotionState::Still,
            peer_id: None,
            device_id: None,
        };
        svc.update_device_state(profile);

        let state = svc.get_device_state().unwrap();
        assert_eq!(state.battery_level, 80);

        let adj = svc.recommended_behavior().unwrap();
        assert!(adj.relay_enabled);
        assert_eq!(adj.relay_budget, 200); // stationary + good battery
    }

    #[test]
    fn test_update_device_state_transitions() {
        let svc = MeshService::new(MeshServiceConfig {
            discovery_interval_ms: 1000,
            battery_floor_pct: 20,
        });

        // First update
        svc.update_device_state(DeviceProfile {
            battery_pct: 50,
            is_charging: false,
            has_wifi: true,
            motion_state: MotionState::Walking,
            peer_id: None,
            device_id: None,
        });

        // Transition to low battery
        svc.update_device_state(DeviceProfile {
            battery_pct: 15,
            is_charging: false,
            has_wifi: false,
            motion_state: MotionState::Walking,
            peer_id: None,
            device_id: None,
        });

        let adj = svc.recommended_behavior().unwrap();
        assert!(!adj.relay_enabled);
        assert_eq!(adj.relay_budget, 0);
        assert!(!adj.minimal_operation);

        // Transition to critical battery
        svc.update_device_state(DeviceProfile {
            battery_pct: 8,
            is_charging: false,
            has_wifi: false,
            motion_state: MotionState::Still,
            peer_id: None,
            device_id: None,
        });

        let adj = svc.recommended_behavior().unwrap();
        assert!(adj.minimal_operation);
    }

    #[test]
    fn test_connection_path_state_disconnected_without_peers() {
        let svc = MeshService::new(MeshServiceConfig {
            discovery_interval_ms: 5_000,
            battery_floor_pct: 20,
        });
        assert_eq!(
            svc.get_connection_path_state(),
            ConnectionPathState::Disconnected
        );
    }

    #[test]
    fn test_export_diagnostics_contains_state_fields() {
        let svc = MeshService::new(MeshServiceConfig {
            discovery_interval_ms: 5_000,
            battery_floor_pct: 20,
        });
        let json = svc.export_diagnostics();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("service_state").is_some());
        assert!(v.get("connection_path_state").is_some());
        assert!(v.get("nat_status").is_some());
        assert!(v.get("timestamp_ms").is_some());
    }

    #[test]
    fn test_get_swarm_bridge_initialization() {
        let svc = MeshService::new(MeshServiceConfig {
            discovery_interval_ms: 5_000,
            battery_floor_pct: 20,
        });
        let bridge = svc.get_swarm_bridge();
        // Initial bridge should have no handle set yet
        assert!(bridge.get_peers_blocking().is_empty());
        assert!(bridge.get_topics_blocking().is_empty());
    }

    #[test]
    fn test_history_manager_persists_across_restart() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();

        {
            let history = HistoryManager::new(path.clone()).unwrap();
            history
                .add(MessageRecord {
                    id: "msg-persist-1".to_string(),
                    direction: MessageDirection::Sent,
                    peer_id: "peer-one".to_string(),
                    content: "hello".to_string(),
                    timestamp: 1_777_000_000,
                    sender_timestamp: 1_777_000_000,
                    delivered: false,
                    status: MessageStatus::default(),
                    hidden: false,
                })
                .unwrap();
            history.mark_delivered("msg-persist-1".to_string()).unwrap();
            assert_eq!(history.count(), 1);
        }

        let reloaded = HistoryManager::new(path).unwrap();
        let record = reloaded
            .get("msg-persist-1".to_string())
            .unwrap()
            .expect("message record should persist");
        assert_eq!(record.peer_id, "peer-one");
        assert!(record.delivered);
    }

    #[test]
    fn test_history_manager_recent_sorts_by_timestamp_not_key_order() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let history = HistoryManager::new(path).unwrap();

        history
            .add(MessageRecord {
                id: "z_old".to_string(),
                direction: MessageDirection::Sent,
                peer_id: "peer-a".to_string(),
                content: "old".to_string(),
                timestamp: 100,
                sender_timestamp: 100,
                delivered: false,
                status: MessageStatus::default(),
                hidden: false,
            })
            .unwrap();
        history
            .add(MessageRecord {
                id: "a_new".to_string(),
                direction: MessageDirection::Sent,
                peer_id: "peer-a".to_string(),
                content: "new".to_string(),
                timestamp: 200,
                sender_timestamp: 200,
                delivered: false,
                status: MessageStatus::default(),
                hidden: false,
            })
            .unwrap();
        history
            .add(MessageRecord {
                id: "m_other".to_string(),
                direction: MessageDirection::Received,
                peer_id: "peer-b".to_string(),
                content: "other".to_string(),
                timestamp: 300,
                sender_timestamp: 300,
                delivered: true,
                status: MessageStatus::Delivered,
                hidden: false,
            })
            .unwrap();

        let latest_any = history.recent(None, 1).unwrap();
        assert_eq!(latest_any.len(), 1);
        assert_eq!(latest_any[0].id, "m_other");

        let peer_a = history.recent(Some("peer-a".to_string()), 2).unwrap();
        assert_eq!(peer_a.len(), 2);
        assert_eq!(peer_a[0].id, "a_new");
        assert_eq!(peer_a[1].id, "z_old");
    }

    // -----------------------------------------------------------------------
    // Existing tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_ledger_preferred_relays() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let ledger = LedgerManager::new(path);

        // Valid libp2p peer ids -- record_connection rejects unparseable
        // peer ids (v2a-2 wire-validation), so fixtures use real PeerIds.
        let peer1 = libp2p::PeerId::random().to_string();
        let peer2 = libp2p::PeerId::random().to_string();

        // Add some entries
        ledger.record_connection("/ip4/1.2.3.4/tcp/1000".to_string(), peer1.clone());
        ledger.record_connection("/ip4/1.2.3.4/tcp/1000".to_string(), peer1.clone()); // Make it successful

        // Simulate time passing and another peer
        std::thread::sleep(web_time::Duration::from_millis(10));
        ledger.record_connection("/ip4/5.6.7.8/tcp/2000".to_string(), peer2.clone());
        ledger.record_connection("/ip4/5.6.7.8/tcp/2000".to_string(), peer2.clone());

        let preferred = ledger.get_preferred_relays(10);
        assert_eq!(preferred.len(), 2);

        // Peer 2 should be first because it was seen last
        assert_eq!(preferred[0].peer_id, Some(peer2.clone()));
        assert_eq!(preferred[1].peer_id, Some(peer1.clone()));

        let limited = ledger.get_preferred_relays(1);
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].peer_id, Some(peer2));
    }

    #[test]
    fn test_mesh_settings_default() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let manager = MeshSettingsManager::new(path);
        let settings = manager.default_settings();

        assert!(settings.relay_enabled);
        assert_eq!(settings.max_relay_budget, 200);
        assert_eq!(settings.battery_floor, 20);
        assert!(settings.ble_enabled);
        assert!(!settings.wifi_aware_enabled);
        assert!(!settings.wifi_direct_enabled);
        assert!(settings.internet_enabled);
        assert_eq!(settings.discovery_mode, crate::DiscoveryMode::Normal);
    }

    #[test]
    fn message_status_monotone_progress() {
        // Valid transitions: Queued → InCustody/Sent → Delivered
        // Never regresses: Delivered never downgrades
        let status = MessageStatus::default();
        assert_eq!(status, MessageStatus::Queued);

        // Queued → InCustody (valid)
        let custody = MessageStatus::InCustody;
        assert!(custody as u8 > MessageStatus::Queued as u8);

        // Queued → Sent (valid)
        let sent = MessageStatus::Sent;
        assert!(sent as u8 > MessageStatus::Queued as u8);

        // Sent → Delivered (valid)
        let delivered = MessageStatus::Delivered;
        assert!(delivered as u8 > MessageStatus::Sent as u8);

        // Delivered is highest — no regression possible
        assert_eq!(delivered as u8, 3);
    }

    #[test]
    fn message_status_serialization_roundtrip() {
        let status = MessageStatus::Delivered;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: MessageStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    // -----------------------------------------------------------------------
    // T1.1: generalized FFI proximity-data plane
    // -----------------------------------------------------------------------

    /// Mock `PlatformBridge` recording every outbound send and every inbound
    /// delivery request it observes, so tests can assert round-trip behavior
    /// without a real BLE/WiFi Aware/WiFi Direct stack.
    #[derive(Default)]
    struct MockPlatformBridge {
        sent_packets: Mutex<Vec<(String, ProximityTransport, Vec<u8>)>>,
    }

    impl PlatformBridge for MockPlatformBridge {
        fn on_battery_changed(&self, _battery_pct: u8, _is_charging: bool) {}
        fn on_network_changed(&self, _has_wifi: bool, _has_cellular: bool) {}
        fn on_motion_changed(&self, _motion: MotionState) {}
        fn on_ble_data_received(&self, _peer_id: String, _data: Vec<u8>) {}
        fn on_entering_background(&self) {}
        fn on_entering_foreground(&self) {}
        fn send_ble_packet(&self, peer_id: String, data: Vec<u8>) {
            self.send_proximity_packet(peer_id, ProximityTransport::Ble, data);
        }
        fn on_proximity_data_received(
            &self,
            _peer_id: String,
            _transport: ProximityTransport,
            _data: Vec<u8>,
        ) {
        }
        fn send_proximity_packet(
            &self,
            peer_id: String,
            transport: ProximityTransport,
            data: Vec<u8>,
        ) {
            self.sent_packets.lock().push((peer_id, transport, data));
        }
        fn wifi_aware_publish(&self, _service_name: String, _service_info: Vec<u8>) -> bool {
            true
        }
        fn wifi_aware_subscribe(&self, _service_name: String) -> bool {
            true
        }
        fn wifi_aware_create_data_path(&self, _peer_id: String, _pmk: Vec<u8>) -> bool {
            true
        }
        fn wifi_aware_stop(&self) {}
        fn wifi_direct_discover_peers(&self) -> bool {
            true
        }
        fn wifi_direct_stop_discovery(&self) {}
        fn wifi_direct_connect(&self, _device_address: String) -> bool {
            true
        }
        fn wifi_direct_create_group(&self, _group_name: String) -> bool {
            true
        }
        fn wifi_direct_remove_group(&self) {}
    }

    fn test_mesh_service_config() -> MeshServiceConfig {
        MeshServiceConfig {
            discovery_interval_ms: 5_000,
            battery_floor_pct: 20,
        }
    }

    fn all_proximity_transports() -> [ProximityTransport; 4] {
        [
            ProximityTransport::Ble,
            ProximityTransport::WifiAware,
            ProximityTransport::WifiDirect,
            ProximityTransport::Multipeer,
        ]
    }

    /// Oversize payloads must be rejected (dropped, not silently truncated,
    /// not panicking) for every `ProximityTransport` variant, matching each
    /// transport's `max_payload_size`.
    #[test]
    fn proximity_oversize_payload_rejected_per_transport_outbound() {
        let service = MeshService::new(test_mesh_service_config());
        let bridge = std::sync::Arc::new(MockPlatformBridge::default());
        service.set_platform_bridge(Some(Box::new(MockBridgeHandle(bridge.clone()))));

        for transport in all_proximity_transports() {
            let max = transport.max_payload_size();
            let oversize_data = vec![0xAB; max + 1];
            service.dispatch_proximity_packet(
                "peer-oversize".to_string(),
                transport,
                oversize_data,
            );
        }

        // None of the oversize sends should have reached the bridge.
        assert!(
            bridge.sent_packets.lock().is_empty(),
            "oversize payloads must never be forwarded to the platform bridge"
        );
    }

    /// Oversize inbound data must be dropped (not processed / not panicking)
    /// for every `ProximityTransport` variant.
    #[test]
    fn proximity_oversize_payload_rejected_per_transport_inbound() {
        let service = MeshService::new(test_mesh_service_config());

        for transport in all_proximity_transports() {
            let max = transport.max_payload_size();
            let oversize_data = vec![0xCD; max + 1];
            // Must not panic; core is not initialized so there is nothing to
            // observe beyond "did not crash" plus size-gate behavior, which
            // on_proximity_data_received enforces before touching the core.
            service.on_proximity_data_received(
                "peer-oversize-in".to_string(),
                transport,
                oversize_data,
            );
        }
    }

    /// Exactly-at-limit payloads must be accepted (not rejected) for every
    /// transport, i.e. the size check is `>`, not `>=`.
    #[test]
    fn proximity_payload_at_exact_limit_is_forwarded() {
        let service = MeshService::new(test_mesh_service_config());
        let bridge = std::sync::Arc::new(MockPlatformBridge::default());
        service.set_platform_bridge(Some(Box::new(MockBridgeHandle(bridge.clone()))));

        for transport in all_proximity_transports() {
            let max = transport.max_payload_size();
            let data = vec![0x11; max];
            service.dispatch_proximity_packet("peer-exact".to_string(), transport, data);
        }

        let sent = bridge.sent_packets.lock();
        assert_eq!(
            sent.len(),
            all_proximity_transports().len(),
            "at-limit payloads for every transport must be forwarded"
        );
    }

    /// Round-trip: dispatching a packet for a given transport calls the
    /// bridge's `send_proximity_packet` with the same peer id, transport tag,
    /// and bytes; and `send_ble_packet` (legacy) is a thin wrapper that
    /// produces an equivalent `Ble`-tagged call.
    #[test]
    fn proximity_round_trip_via_mock_bridge_ble_and_wifi_aware() {
        let service = MeshService::new(test_mesh_service_config());
        let bridge = std::sync::Arc::new(MockPlatformBridge::default());
        service.set_platform_bridge(Some(Box::new(MockBridgeHandle(bridge.clone()))));

        // Ble via the generic dispatch path.
        service.dispatch_proximity_packet(
            "peer-ble".to_string(),
            ProximityTransport::Ble,
            b"hello-ble".to_vec(),
        );
        // WifiAware via the generic dispatch path.
        service.dispatch_proximity_packet(
            "peer-aware".to_string(),
            ProximityTransport::WifiAware,
            b"hello-aware".to_vec(),
        );
        // Legacy BLE-named helper must still work and route as Ble.
        service.dispatch_ble_packet("peer-ble-legacy".to_string(), b"legacy-ble".to_vec());

        let sent = bridge.sent_packets.lock();
        assert_eq!(sent.len(), 3);

        assert_eq!(sent[0].0, "peer-ble");
        assert_eq!(sent[0].1, ProximityTransport::Ble);
        assert_eq!(sent[0].2, b"hello-ble".to_vec());

        assert_eq!(sent[1].0, "peer-aware");
        assert_eq!(sent[1].1, ProximityTransport::WifiAware);
        assert_eq!(sent[1].2, b"hello-aware".to_vec());

        assert_eq!(sent[2].0, "peer-ble-legacy");
        assert_eq!(sent[2].1, ProximityTransport::Ble);
        assert_eq!(sent[2].2, b"legacy-ble".to_vec());
    }

    /// Thin wrapper for the mock: `PlatformBridge` requires `Box<dyn
    /// PlatformBridge>` ownership at the `MeshService` boundary, while tests
    /// want to keep observing the shared mock via `Arc` after handing
    /// ownership over. This indirection lets the test retain a handle.
    struct MockBridgeHandle(std::sync::Arc<MockPlatformBridge>);

    impl PlatformBridge for MockBridgeHandle {
        fn on_battery_changed(&self, battery_pct: u8, is_charging: bool) {
            self.0.on_battery_changed(battery_pct, is_charging);
        }
        fn on_network_changed(&self, has_wifi: bool, has_cellular: bool) {
            self.0.on_network_changed(has_wifi, has_cellular);
        }
        fn on_motion_changed(&self, motion: MotionState) {
            self.0.on_motion_changed(motion);
        }
        fn on_ble_data_received(&self, peer_id: String, data: Vec<u8>) {
            self.0.on_ble_data_received(peer_id, data);
        }
        fn on_entering_background(&self) {
            self.0.on_entering_background();
        }
        fn on_entering_foreground(&self) {
            self.0.on_entering_foreground();
        }
        fn send_ble_packet(&self, peer_id: String, data: Vec<u8>) {
            self.0.send_ble_packet(peer_id, data);
        }
        fn on_proximity_data_received(
            &self,
            peer_id: String,
            transport: ProximityTransport,
            data: Vec<u8>,
        ) {
            self.0.on_proximity_data_received(peer_id, transport, data);
        }
        fn send_proximity_packet(
            &self,
            peer_id: String,
            transport: ProximityTransport,
            data: Vec<u8>,
        ) {
            self.0.send_proximity_packet(peer_id, transport, data);
        }
        fn wifi_aware_publish(&self, service_name: String, service_info: Vec<u8>) -> bool {
            self.0.wifi_aware_publish(service_name, service_info)
        }
        fn wifi_aware_subscribe(&self, service_name: String) -> bool {
            self.0.wifi_aware_subscribe(service_name)
        }
        fn wifi_aware_create_data_path(&self, peer_id: String, pmk: Vec<u8>) -> bool {
            self.0.wifi_aware_create_data_path(peer_id, pmk)
        }
        fn wifi_aware_stop(&self) {
            self.0.wifi_aware_stop();
        }
        fn wifi_direct_discover_peers(&self) -> bool {
            self.0.wifi_direct_discover_peers()
        }
        fn wifi_direct_stop_discovery(&self) {
            self.0.wifi_direct_stop_discovery();
        }
        fn wifi_direct_connect(&self, device_address: String) -> bool {
            self.0.wifi_direct_connect(device_address)
        }
        fn wifi_direct_create_group(&self, group_name: String) -> bool {
            self.0.wifi_direct_create_group(group_name)
        }
        fn wifi_direct_remove_group(&self) {
            self.0.wifi_direct_remove_group();
        }
    }
}
