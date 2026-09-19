// scmessenger-wasm — WebAssembly bindings for browser environments

pub mod connection_state;
pub mod daemon_bridge;
pub mod notification_manager;
pub mod transport;

use anyhow::Error;
use libp2p::{Multiaddr, PeerId};
use scmessenger_core::{
    IdentityInfo, IronCore as RustIronCore, NotificationDecision, NotificationMessageContext,
    NotificationUiState, SignatureResult,
};
use serde_json::{Map, Value};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DiscoveryMode {
    Normal,
    Cautious,
    Paranoid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct MeshSettings {
    pub relay_enabled: bool,
    pub max_relay_budget: u32,
    pub battery_floor: u8,
    pub ble_enabled: bool,
    pub wifi_aware_enabled: bool,
    pub wifi_direct_enabled: bool,
    pub internet_enabled: bool,
    pub discovery_mode: DiscoveryMode,
    pub onion_routing: bool,
    pub cover_traffic_enabled: bool,
    pub message_padding_enabled: bool,
    pub timing_obfuscation_enabled: bool,
    pub notifications_enabled: bool,
    pub notify_dm_enabled: bool,
    pub notify_dm_request_enabled: bool,
    pub notify_dm_in_foreground: bool,
    pub notify_dm_request_in_foreground: bool,
    pub sound_enabled: bool,
    pub badge_enabled: bool,
}

impl Default for MeshSettings {
    fn default() -> Self {
        Self {
            relay_enabled: true,
            max_relay_budget: 200,
            battery_floor: 20,
            ble_enabled: true,
            wifi_aware_enabled: true,
            wifi_direct_enabled: true,
            internet_enabled: true,
            discovery_mode: DiscoveryMode::Normal,
            onion_routing: false,
            cover_traffic_enabled: false,
            message_padding_enabled: false,
            timing_obfuscation_enabled: false,
            notifications_enabled: true,
            notify_dm_enabled: true,
            notify_dm_request_enabled: true,
            notify_dm_in_foreground: false,
            notify_dm_request_in_foreground: true,
            sound_enabled: true,
            badge_enabled: true,
        }
    }
}

impl From<MeshSettings> for scmessenger_core::MeshSettings {
    fn from(wasm: MeshSettings) -> Self {
        scmessenger_core::MeshSettings {
            require_pq: false,
            relay_enabled: wasm.relay_enabled,
            max_relay_budget: wasm.max_relay_budget,
            battery_floor: wasm.battery_floor,
            ble_enabled: wasm.ble_enabled,
            wifi_aware_enabled: wasm.wifi_aware_enabled,
            wifi_direct_enabled: wasm.wifi_direct_enabled,
            internet_enabled: wasm.internet_enabled,
            discovery_mode: match wasm.discovery_mode {
                DiscoveryMode::Normal => scmessenger_core::DiscoveryMode::Normal,
                DiscoveryMode::Cautious => scmessenger_core::DiscoveryMode::Cautious,
                DiscoveryMode::Paranoid => scmessenger_core::DiscoveryMode::Paranoid,
            },
            onion_routing: wasm.onion_routing,
            cover_traffic_enabled: wasm.cover_traffic_enabled,
            message_padding_enabled: wasm.message_padding_enabled,
            timing_obfuscation_enabled: wasm.timing_obfuscation_enabled,
            notifications_enabled: wasm.notifications_enabled,
            notify_dm_enabled: wasm.notify_dm_enabled,
            notify_dm_request_enabled: wasm.notify_dm_request_enabled,
            notify_dm_in_foreground: wasm.notify_dm_in_foreground,
            notify_dm_request_in_foreground: wasm.notify_dm_request_in_foreground,
            sound_enabled: wasm.sound_enabled,
            badge_enabled: wasm.badge_enabled,
        }
    }
}

pub struct MeshSettingsManager {
    /// Only read under `cfg(not(target_arch = "wasm32"))` in `load`/`save`;
    /// unused when actually compiled to wasm32 (browser storage path differs).
    #[allow(dead_code)]
    storage_path: String,
}

impl MeshSettingsManager {
    pub fn new(storage_path: String) -> Self {
        Self { storage_path }
    }

    pub fn load(&self) -> Result<MeshSettings, scmessenger_core::IronCoreError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let settings_file =
                std::path::PathBuf::from(&self.storage_path).join("mesh_settings.json");
            if settings_file.exists() {
                let data = std::fs::read_to_string(&settings_file)
                    .map_err(|_| scmessenger_core::IronCoreError::StorageError)?;
                let settings: MeshSettings = serde_json::from_str(&data)
                    .map_err(|_| scmessenger_core::IronCoreError::Internal)?;
                return Ok(settings);
            }
        }

        Ok(MeshSettings::default())
    }

    pub fn save(&self, settings: MeshSettings) -> Result<(), scmessenger_core::IronCoreError> {
        self.validate(settings.clone())?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let storage_path = std::path::PathBuf::from(&self.storage_path);
            std::fs::create_dir_all(&storage_path)
                .map_err(|_| scmessenger_core::IronCoreError::StorageError)?;

            let settings_file = storage_path.join("mesh_settings.json");
            let data = serde_json::to_string_pretty(&settings)
                .map_err(|_| scmessenger_core::IronCoreError::Internal)?;
            std::fs::write(&settings_file, data)
                .map_err(|_| scmessenger_core::IronCoreError::StorageError)?;
        }

        Ok(())
    }

    pub fn validate(&self, settings: MeshSettings) -> Result<(), scmessenger_core::IronCoreError> {
        if settings.relay_enabled && settings.max_relay_budget == 0 {
            return Err(scmessenger_core::IronCoreError::InvalidInput);
        }

        if !settings.ble_enabled
            && !settings.wifi_aware_enabled
            && !settings.wifi_direct_enabled
            && !settings.internet_enabled
        {
            return Err(scmessenger_core::IronCoreError::InvalidInput);
        }

        if settings.battery_floor > 50 {
            return Err(scmessenger_core::IronCoreError::InvalidInput);
        }

        Ok(())
    }
}

#[wasm_bindgen]
pub fn init_logging() {
    console_error_panic_hook::set_once();
    #[cfg(target_arch = "wasm32")]
    {
        static LOG_INIT: std::sync::Once = std::sync::Once::new();
        LOG_INIT.call_once(|| {
            tracing_wasm::set_as_global_default();
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum IronCoreMode {
    /// Full mode: local identity initialization and local mesh operations
    #[default]
    Full,
    /// Daemon mode: identity and operations route through local CLI daemon via JSON-RPC
    Daemon,
}

#[wasm_bindgen]
pub struct IronCore {
    inner: std::sync::Arc<RustIronCore>,
    /// Buffer of successfully decoded messages waiting to be drained by JS.
    rx_messages: Rc<RefCell<Vec<WasmMessage>>>,
    /// Active libp2p swarm handle for browser networking.
    swarm_handle: Rc<RefCell<Option<scmessenger_core::transport::SwarmHandle>>>,
    /// Settings manager for persistence (uses localStorage path or in-memory).
    settings_manager: Option<MeshSettingsManager>,
    /// Cached in-memory settings for the current session.
    settings: Rc<RefCell<MeshSettings>>,
    /// Operating mode: Full or Daemon
    mode: Rc<RefCell<IronCoreMode>>,
    /// Daemon socket URL for JSON-RPC communication (when in Daemon mode)
    daemon_socket_url: Rc<RefCell<Option<String>>>,
}

#[wasm_bindgen]
impl IronCore {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        init_logging();
        // Web defaults: always plugged in, internet-only transport
        let defaults = MeshSettings {
            battery_floor: 0,           // Web = always plugged in
            ble_enabled: false,         // No BLE in browser
            wifi_aware_enabled: false,  // No WiFi Aware in browser
            wifi_direct_enabled: false, // No WiFi Direct in browser
            internet_enabled: true,
            ..MeshSettings::default()
        };
        let core = Self {
            inner: std::sync::Arc::new(RustIronCore::new()),
            rx_messages: Rc::new(RefCell::new(Vec::new())),
            swarm_handle: Rc::new(RefCell::new(None)),
            settings_manager: None,
            settings: Rc::new(RefCell::new(defaults.clone())),
            mode: Rc::new(RefCell::new(IronCoreMode::Full)),
            daemon_socket_url: Rc::new(RefCell::new(None)),
        };

        // P1_CORE_001: Sync drift state
        if defaults.relay_enabled {
            core.inner.drift_activate();
        }

        core
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[wasm_bindgen(js_name = withStorage)]
    pub fn with_storage(storage_path: String) -> Self {
        init_logging();
        let manager = MeshSettingsManager::new(storage_path.clone());
        let loaded = manager.load().unwrap_or_else(|_| MeshSettings {
            battery_floor: 0,
            ble_enabled: false,
            wifi_aware_enabled: false,
            wifi_direct_enabled: false,
            internet_enabled: true,
            ..MeshSettings::default()
        });
        let core = Self {
            inner: std::sync::Arc::new(RustIronCore::with_storage(storage_path)),
            rx_messages: Rc::new(RefCell::new(Vec::new())),
            swarm_handle: Rc::new(RefCell::new(None)),
            settings_manager: Some(manager),
            settings: Rc::new(RefCell::new(loaded.clone())),
            mode: Rc::new(RefCell::new(IronCoreMode::Full)),
            daemon_socket_url: Rc::new(RefCell::new(None)),
        };

        // P1_CORE_001: Sync drift state
        if loaded.relay_enabled {
            core.inner.drift_activate();
        }

        core
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_name = withStorage)]
    pub fn with_storage(storage_path: String) -> Self {
        init_logging();
        let manager = MeshSettingsManager::new(storage_path.clone());
        let loaded = manager.load().unwrap_or_else(|_| MeshSettings {
            battery_floor: 0,
            ble_enabled: false,
            wifi_aware_enabled: false,
            wifi_direct_enabled: false,
            internet_enabled: true,
            ..MeshSettings::default()
        });
        let core = Self {
            inner: std::sync::Arc::new(RustIronCore::new()),
            rx_messages: Rc::new(RefCell::new(Vec::new())),
            swarm_handle: Rc::new(RefCell::new(None)),
            settings_manager: Some(manager),
            settings: Rc::new(RefCell::new(loaded.clone())),
            mode: Rc::new(RefCell::new(IronCoreMode::Full)),
            daemon_socket_url: Rc::new(RefCell::new(None)),
        };

        // P1_CORE_001: Sync drift state
        if loaded.relay_enabled {
            core.inner.drift_activate();
        }

        core
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_name = withStorageAsync)]
    pub async fn with_storage_async(storage_path: String) -> Self {
        init_logging();
        let manager = MeshSettingsManager::new(storage_path.clone());
        let loaded = manager.load().unwrap_or_else(|_| MeshSettings {
            battery_floor: 0,
            ble_enabled: false,
            wifi_aware_enabled: false,
            wifi_direct_enabled: false,
            internet_enabled: true,
            ..MeshSettings::default()
        });
        Self {
            inner: std::sync::Arc::new(RustIronCore::new()),
            rx_messages: Rc::new(RefCell::new(Vec::new())),
            swarm_handle: Rc::new(RefCell::new(None)),
            settings_manager: Some(manager),
            settings: Rc::new(RefCell::new(loaded)),
            mode: Rc::new(RefCell::new(IronCoreMode::Full)),
            daemon_socket_url: Rc::new(RefCell::new(None)),
        }
    }

    pub fn start(&self) -> Result<(), JsValue> {
        self.inner
            .start()
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    pub fn stop(&self) {
        self.inner.stop();
    }

    #[wasm_bindgen(js_name = isRunning)]
    pub fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    #[wasm_bindgen(js_name = getIdentityInfo)]
    pub fn get_identity_info(&self) -> JsValue {
        let info = self.inner.get_identity_info();
        to_js_value_safe(&WasmIdentityInfo::from(info))
    }

    /// Set the operating mode for this IronCore instance.
    /// In Daemon mode, the WASM client routes all operations through a local CLI daemon
    /// via JSON-RPC instead of performing operations locally.
    #[wasm_bindgen(js_name = setIronCoreMode)]
    pub fn set_iron_core_mode(&self, mode: JsValue) -> Result<(), JsValue> {
        let new_mode: IronCoreMode = serde_wasm_bindgen::from_value(mode)
            .map_err(|e| js_value_from_str(&format!("Invalid mode: {}", e)))?;
        *self.mode.borrow_mut() = new_mode;
        Ok(())
    }

    /// Get the current IronCore operating mode.
    #[wasm_bindgen(js_name = getIronCoreMode)]
    pub fn get_iron_core_mode(&self) -> JsValue {
        let mode = *self.mode.borrow();
        to_js_value_safe(&mode)
    }

    /// Set the daemon WebSocket socket URL for JSON-RPC communication.
    /// This is required when operating in Daemon mode.
    #[wasm_bindgen(js_name = setDaemonSocketUrl)]
    pub fn set_daemon_socket_url(&self, url: String) {
        *self.daemon_socket_url.borrow_mut() = Some(url);
    }

    /// Get the current daemon socket URL.
    #[wasm_bindgen(js_name = getDaemonSocketUrl)]
    pub fn get_daemon_socket_url(&self) -> JsValue {
        let url = self.daemon_socket_url.borrow();
        match url.as_ref() {
            Some(u) => to_js_value_safe(&u),
            None => JsValue::NULL,
        }
    }

    /// Initialize identity in local (non-daemon) mode.
    /// This generates a new identity or loads an existing one from storage.
    /// In Daemon mode, call `initializeIdentityFromDaemon` instead.
    #[wasm_bindgen(js_name = initializeIdentity)]
    pub fn initialize_identity(&self) -> Result<(), JsValue> {
        // Clone values to avoid borrowing issues
        let mode = *self.mode.borrow();
        if mode == IronCoreMode::Daemon {
            let url = self.daemon_socket_url.borrow().clone();
            let daemon_url = url.as_deref().unwrap_or("unknown");
            return Err(js_value_from_str(&format!(
                "Identity initialization refused: IronCore is in Daemon mode. \
                 Identity must be obtained from daemon at {} via getIdentityFromDaemon().",
                daemon_url
            )));
        }

        self.inner
            .start() // Ensure core is running first
            .map_err(|e| js_value_from_str(&format!("{}", e)))?;

        self.inner
            .initialize_identity()
            .map_err(|e| js_value_from_str(&format!("{}", e)))?;

        Ok(())
    }

    /// Initialize identity from daemon when in Daemon mode.
    /// This fetches the identity from the local CLI daemon via JSON-RPC.
    #[wasm_bindgen(js_name = initializeIdentityFromDaemon)]
    pub async fn initialize_identity_from_daemon(&self) -> Result<(), JsValue> {
        let mode = *self.mode.borrow();

        if mode != IronCoreMode::Daemon {
            return Err(js_value_from_str(
                "initializeIdentityFromDaemon() called but not in Daemon mode. Use initializeIdentity() instead.",
            ));
        }

        // Clone the URL to avoid borrowing issues
        let url_clone = self.daemon_socket_url.borrow().clone();
        let Some(daemon_url) = url_clone.as_ref() else {
            return Err(js_value_from_str(
                "No daemon socket URL set. Call setDaemonSocketUrl() first.",
            ));
        };

        // In the current architecture, the daemon handles identity management.
        // This method serves as a placeholder/gate that enforces the daemon-only mode.
        // The actual identity is obtained by the daemon and exposed via WebSocket notifications
        // or through getIdentityFromDaemon() calls.
        //
        // For now, we verify that the daemon connection is configured and return success.
        // The WASM client should NOT perform local identity initialization in this mode.
        tracing::info!(
            "IronCore initialized in Daemon mode - identity managed by daemon at {}",
            daemon_url
        );

        Ok(())
    }

    /// Get identity info from the local IronCore.
    /// In Daemon mode, this may return a placeholder until the daemon provides identity.
    #[wasm_bindgen(js_name = getIdentityFromDaemon)]
    pub fn get_identity_from_daemon(&self) -> Result<JsValue, JsValue> {
        let mode = *self.mode.borrow();

        if mode == IronCoreMode::Daemon {
            // Clone the URL to avoid borrowing issues
            let url_clone = self.daemon_socket_url.borrow().clone();
            // In daemon mode, identity info should come from the daemon.
            // Return a placeholder indicating we need to wait for daemon identity.
            let daemon_url = url_clone.as_deref().unwrap_or("unknown");
            return Err(js_value_from_str(&format!(
                "Identity pending from daemon at {}. \
                 Ensure daemon is running and identity has been established.",
                daemon_url
            )));
        }

        // Full mode: return actual identity info from local core
        let info = self.inner.get_identity_info();
        serde_wasm_bindgen::to_value(&WasmIdentityInfo::from(info))
            .map_err(|e| js_value_from_str(&format!("Failed to serialize identity info: {}", e)))
    }

    #[wasm_bindgen(js_name = signData)]
    pub fn sign_data(&self, data: Vec<u8>) -> Result<JsValue, JsValue> {
        self.inner
            .sign_data(data)
            .map(|sig| to_js_value_safe(&WasmSignatureResult::from(sig)))
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = verifySignature)]
    pub fn verify_signature(
        &self,
        data: Vec<u8>,
        signature: Vec<u8>,
        public_key_hex: String,
    ) -> Result<bool, JsValue> {
        self.inner
            .verify_signature(data, signature, public_key_hex)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// Prepare an encrypted message envelope.
    ///
    /// In **Full mode**: Encrypts locally using the local identity and returns the envelope bytes.
    /// In **Daemon mode**: Routes the request to the CLI daemon via JSON-RPC.
    ///                 Returns the envelope bytes prepared by the daemon.
    #[wasm_bindgen(js_name = prepareMessage)]
    pub fn prepare_message(
        &self,
        recipient_public_key_hex: String,
        text: String,
    ) -> Result<Vec<u8>, JsValue> {
        let mode = *self.mode.borrow();
        let url = self.daemon_socket_url.borrow().clone();

        if mode == IronCoreMode::Daemon {
            let daemon_url = url.as_deref().unwrap_or("unknown");

            // In daemon mode, we route prepare_message to the daemon.
            // For now, we return an error indicating daemon is required.
            // The full implementation would make a JSON-RPC call to the daemon
            // and return the envelope from the daemon's response.
            return Err(js_value_from_str(&format!(
                "prepareMessage() blocked: IronCore is in Daemon mode. \
                 Message preparation must be done by the daemon at {}. \
                 Use sendPreparedMessageFromDaemon() after daemon returns envelope.",
                daemon_url
            )));
        }

        ensure_mesh_participation_enabled(self.settings.borrow().relay_enabled)?;
        self.inner
            .prepare_message(
                recipient_public_key_hex,
                text,
                scmessenger_core::MessageType::Text,
                None,
            )
            .map(|pm| pm.envelope_data)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = receiveMessage)]
    pub fn receive_message(&self, envelope_bytes: Vec<u8>) -> Result<JsValue, JsValue> {
        ensure_mesh_participation_enabled(self.settings.borrow().relay_enabled)?;
        self.inner
            .receive_message(envelope_bytes)
            .map(|msg| {
                to_js_value_safe(&WasmMessage {
                    id: msg.id.clone(),
                    sender_id: msg.sender_id.clone(),
                    sender_peer_id: None,
                    text: msg.text_content(),
                    timestamp: msg.timestamp,
                })
            })
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = outboxCount)]
    pub fn outbox_count(&self) -> u32 {
        self.inner.outbox_count()
    }

    /// Flush the outbox for a specific peer, returning the count of messages drained.
    /// The caller should then send each message via the swarm transport.
    #[wasm_bindgen(js_name = flushOutboxForPeer)]
    pub fn flush_outbox_for_peer(&self, peer_id: String) -> usize {
        self.inner.flush_outbox_for_peer(&peer_id).len()
    }

    #[wasm_bindgen(js_name = inboxCount)]
    pub fn inbox_count(&self) -> u32 {
        self.inner.inbox_count()
    }

    /// Start libp2p swarm networking for the browser client.
    ///
    /// `bootstrapAddrs` must be a JS array of libp2p multiaddr strings.
    #[wasm_bindgen(js_name = startSwarm)]
    pub async fn start_swarm(&self, bootstrap_addrs: JsValue) -> Result<(), JsValue> {
        let bootstrap_addrs = parse_bootstrap_addrs(bootstrap_addrs)?;
        start_swarm_runtime(
            std::sync::Arc::clone(&self.inner),
            Rc::clone(&self.rx_messages),
            Rc::clone(&self.settings),
            Rc::clone(&self.swarm_handle),
            bootstrap_addrs,
        )
        .await
    }

    /// Stop libp2p swarm networking for the browser client.
    #[wasm_bindgen(js_name = stopSwarm)]
    pub async fn stop_swarm(&self) -> Result<(), JsValue> {
        let maybe_handle = self.swarm_handle.borrow_mut().take();
        if let Some(handle) = maybe_handle {
            handle
                .shutdown()
                .await
                .map_err(|e| js_value_from_str(&format!("Failed to stop swarm: {}", e)))?;
        }
        Ok(())
    }

    /// Send a prepared encrypted envelope to a connected libp2p peer.
    #[wasm_bindgen(js_name = sendPreparedEnvelope)]
    pub async fn send_prepared_envelope(
        &self,
        peer_id: String,
        envelope_bytes: Vec<u8>,
    ) -> Result<(), JsValue> {
        ensure_mesh_participation_enabled(self.settings.borrow().relay_enabled)?;

        let peer_id: PeerId = peer_id
            .parse()
            .map_err(|e| js_value_from_str(&format!("Invalid peer ID: {}", e)))?;

        let handle = self
            .swarm_handle
            .borrow()
            .clone()
            .ok_or_else(|| js_value_from_str("Swarm is not running"))?;

        handle
            .send_message(peer_id, envelope_bytes, None, None)
            .await
            .map_err(|e| js_value_from_str(&format!("Failed to send envelope: {}", e)))
    }

    /// Get currently connected peer IDs.
    #[wasm_bindgen(js_name = getPeers)]
    pub async fn get_peers(&self) -> Result<JsValue, JsValue> {
        let handle = self
            .swarm_handle
            .borrow()
            .clone()
            .ok_or_else(|| js_value_from_str("Swarm is not running"))?;

        let peers = handle
            .get_peers()
            .await
            .map_err(|e| js_value_from_str(&format!("Failed to get peers: {}", e)))?;

        let peer_strings: Vec<String> = peers.into_iter().map(|p| p.to_string()).collect();
        serde_wasm_bindgen::to_value(&peer_strings)
            .map_err(|e| js_value_from_str(&format!("Failed to serialize peers: {}", e)))
    }

    /// Get externally observed addresses (if available) for diagnostics parity.
    #[wasm_bindgen(js_name = getExternalAddresses)]
    pub async fn get_external_addresses(&self) -> Result<JsValue, JsValue> {
        let handle = self
            .swarm_handle
            .borrow()
            .clone()
            .ok_or_else(|| js_value_from_str("Swarm is not running"))?;

        let addrs = handle
            .get_external_addresses()
            .await
            .map_err(|e| js_value_from_str(&format!("Failed to get external addresses: {}", e)))?;

        let addr_strings: Vec<String> = addrs.into_iter().map(|a| a.to_string()).collect();
        serde_wasm_bindgen::to_value(&addr_strings).map_err(|e| {
            js_value_from_str(&format!("Failed to serialize external addresses: {}", e))
        })
    }

    #[wasm_bindgen(js_name = getConnectionPathState)]
    pub async fn get_connection_path_state(&self) -> Result<String, JsValue> {
        let maybe_handle = self.swarm_handle.borrow().clone();
        let Some(handle) = maybe_handle else {
            return Ok("Disconnected".to_string());
        };

        let peers = handle
            .get_peers()
            .await
            .map_err(|e| js_value_from_str(&format!("Failed to get peers: {}", e)))?;
        if peers.is_empty() {
            return Ok("Bootstrapping".to_string());
        }

        let listeners = handle
            .get_listeners()
            .await
            .map_err(|e| js_value_from_str(&format!("Failed to get listeners: {}", e)))?;
        if listeners.is_empty() {
            Ok("RelayFallback".to_string())
        } else {
            Ok("DirectPreferred".to_string())
        }
    }

    #[wasm_bindgen(js_name = exportDiagnostics)]
    pub async fn export_diagnostics(&self) -> Result<String, JsValue> {
        // Clone the handle out of the lock before any await so the RefCell borrow
        // is not held across the suspension point.
        let handle_opt = self.swarm_handle.borrow().clone();
        let peers = if let Some(handle) = handle_opt {
            handle
                .get_peers()
                .await
                .map(|p| p.into_iter().map(|id| id.to_string()).collect::<Vec<_>>())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let connection_path = self.get_connection_path_state().await?;
        let mut m = Map::new();
        m.insert("running".to_string(), self.is_running().into());
        m.insert("connection_path_state".to_string(), connection_path.into());
        m.insert("peers".to_string(), peers.into());
        m.insert("inbox_count".to_string(), self.inbox_count().into());
        m.insert("outbox_count".to_string(), self.outbox_count().into());
        m.insert(
            "relay_enabled".to_string(),
            self.settings.borrow().relay_enabled.into(),
        );
        m.insert("timestamp_ms".to_string(), js_sys::Date::now().into());
        let payload = Value::Object(m);

        Ok(payload.to_string())
    }

    // ── Topic Management ─────────────────────────────────────────────────

    /// Subscribe to a gossipsub topic.
    #[wasm_bindgen(js_name = subscribeTopic)]
    pub async fn subscribe_topic(&self, topic: String) -> Result<(), JsValue> {
        let handle = self
            .swarm_handle
            .borrow()
            .clone()
            .ok_or_else(|| js_value_from_str("Swarm is not running"))?;

        handle
            .subscribe_topic(topic)
            .await
            .map_err(|e: anyhow::Error| {
                js_value_from_str(&format!("Failed to subscribe topic: {}", e))
            })?;
        Ok(())
    }

    /// Unsubscribe from a gossipsub topic.
    #[wasm_bindgen(js_name = unsubscribeTopic)]
    pub async fn unsubscribe_topic(&self, topic: String) -> Result<(), JsValue> {
        let handle = self
            .swarm_handle
            .borrow()
            .clone()
            .ok_or_else(|| js_value_from_str("Swarm is not running"))?;

        handle
            .unsubscribe_topic(topic)
            .await
            .map_err(|e: anyhow::Error| {
                js_value_from_str(&format!("Failed to unsubscribe topic: {}", e))
            })?;
        Ok(())
    }

    /// Publish data to a gossipsub topic.
    #[wasm_bindgen(js_name = publishTopic)]
    pub async fn publish_topic(&self, topic: String, data: Vec<u8>) -> Result<(), JsValue> {
        let handle = self
            .swarm_handle
            .borrow()
            .clone()
            .ok_or_else(|| js_value_from_str("Swarm is not running"))?;

        handle
            .publish_topic(topic, data)
            .await
            .map_err(|e: anyhow::Error| {
                js_value_from_str(&format!("Failed to publish topic: {}", e))
            })?;
        Ok(())
    }

    // ── Network Operations ───────────────────────────────────────────────

    /// Dial a remote peer by multiaddr.
    #[wasm_bindgen(js_name = dial)]
    pub async fn dial(&self, multiaddr: String) -> Result<(), JsValue> {
        let handle = self
            .swarm_handle
            .borrow()
            .clone()
            .ok_or_else(|| js_value_from_str("Swarm is not running"))?;

        let addr: Multiaddr = multiaddr
            .parse()
            .map_err(|e| js_value_from_str(&format!("Invalid multiaddr: {}", e)))?;

        handle
            .dial(addr)
            .await
            .map_err(|e: anyhow::Error| js_value_from_str(&format!("Failed to dial: {}", e)))?;
        Ok(())
    }

    /// Send data to all currently connected peers.
    #[wasm_bindgen(js_name = sendToAllPeers)]
    pub async fn send_to_all_peers(&self, data: Vec<u8>) -> Result<(), JsValue> {
        let handle = self
            .swarm_handle
            .borrow()
            .clone()
            .ok_or_else(|| js_value_from_str("Swarm is not running"))?;

        let peers = handle
            .get_peers()
            .await
            .map_err(|e| js_value_from_str(&format!("Failed to get peers: {}", e)))?;

        let mut sent_count: usize = 0;
        let mut failures: Vec<String> = Vec::new();

        for peer_id in peers {
            match handle.send_message(peer_id, data.clone(), None, None).await {
                Ok(()) => {
                    sent_count += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to send to peer {}: {}", peer_id, e);
                    failures.push(format!("{}: {}", peer_id, e));
                }
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(js_value_from_str(&format!(
                "Failed to send to some peers. Sent to {} peers successfully. Failures: [{}]",
                sent_count,
                failures.join(", ")
            )))
        }
    }

    /// Get current swarm listeners.
    #[wasm_bindgen(js_name = getListeners)]
    pub async fn get_listeners(&self) -> Result<JsValue, JsValue> {
        let handle = self
            .swarm_handle
            .borrow()
            .clone()
            .ok_or_else(|| js_value_from_str("Swarm is not running"))?;

        let listeners: Vec<Multiaddr> =
            handle.get_listeners().await.map_err(|e: anyhow::Error| {
                js_value_from_str(&format!("Failed to get listeners: {}", e))
            })?;

        let listener_strings: Vec<String> = listeners
            .into_iter()
            .map(|a: Multiaddr| a.to_string())
            .collect();
        serde_wasm_bindgen::to_value(&listener_strings)
            .map_err(|e| js_value_from_str(&format!("Failed to serialize listeners: {}", e)))
    }

    /// Get the NAT status. In browser environments this always returns "unknown".
    #[wasm_bindgen(js_name = getNatStatus)]
    pub fn get_nat_status(&self) -> String {
        "unknown".to_string()
    }

    #[wasm_bindgen(js_name = getDriftState)]
    pub fn get_drift_state(&self) -> String {
        self.inner.drift_network_state()
    }

    #[wasm_bindgen(js_name = getDriftStoreSize)]
    pub fn get_drift_store_size(&self) -> u32 {
        self.inner.drift_store_size()
    }

    #[wasm_bindgen(js_name = getAuditLog)]
    pub fn get_audit_log(&self) -> JsValue {
        let log = self.inner.get_audit_log();
        serde_wasm_bindgen::to_value(&log).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = getAuditEventsSince)]
    pub fn get_audit_events_since(&self, since: u64) -> JsValue {
        let events = self.inner.get_audit_events_since(since);
        serde_wasm_bindgen::to_value(&events).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = getPeerReputation)]
    pub fn get_peer_reputation(&self, peer_id: String) -> f64 {
        self.inner.get_peer_reputation(peer_id.clone())
    }

    #[wasm_bindgen(js_name = getEnhancedPeerReputation)]
    pub fn get_enhanced_peer_reputation(&self, peer_id: String) -> JsValue {
        let (score, confidence, flagged) = self.inner.get_enhanced_peer_reputation(peer_id);
        serde_wasm_bindgen::to_value(&(score, confidence, flagged)).unwrap_or(JsValue::NULL)
    }

    /// Get the combined overall abuse score for a peer.
    /// Combines base reputation and spam confidence into a single score.
    #[wasm_bindgen(js_name = getOverallScore)]
    pub fn get_overall_score(&self, peer_id: String) -> Option<f64> {
        self.inner.abuse_overall_score(&peer_id)
    }

    #[wasm_bindgen(js_name = getPrivacyConfig)]
    pub fn get_privacy_config(&self) -> JsValue {
        let config = self.inner.privacy_config();
        serde_wasm_bindgen::to_value(&config).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(js_name = setPrivacyConfig)]
    pub fn set_privacy_config(&self, js_config: JsValue) -> Result<(), JsValue> {
        let config: scmessenger_core::privacy::PrivacyConfig =
            serde_wasm_bindgen::from_value(js_config)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let json = serde_json::to_string(&config)
            .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))?;
        self.inner
            .set_privacy_config(json)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    /// Validate the given settings against invariant rules.
    #[wasm_bindgen(js_name = validateSettings)]
    pub fn validate_settings(&self, js_settings: JsValue) -> Result<(), JsValue> {
        let wasm_settings: WasmMeshSettings = serde_wasm_bindgen::from_value(js_settings)
            .map_err(|e| js_value_from_str(&format!("Invalid settings: {}", e)))?;
        let settings: MeshSettings = wasm_settings.into();

        if let Some(ref mgr) = self.settings_manager {
            mgr.validate(settings)
                .map_err(|e| js_value_from_str(&format!("Validation failed: {:?}", e)))?;
        } else {
            return Err(js_value_from_str(
                "Validation failed: settings manager not initialized",
            ));
        }

        Ok(())
    }

    /// DEPRECATED shim for pre-0.1.2 clients.
    ///
    /// This maps a relay URL to a libp2p websocket multiaddr and starts the
    /// wasm swarm path.  Unlike the original fire-and-forget API, this version
    /// is `async` and propagates start errors back to the JS caller so callers
    /// can detect and handle failures rather than silently missing them.
    ///
    /// Prefer `startSwarm(bootstrapAddrs)` for new code.
    #[wasm_bindgen(js_name = startReceiveLoop)]
    pub async fn start_receive_loop(&self, relay_url: String) -> Result<(), JsValue> {
        tracing::warn!(
            "startReceiveLoop(relayUrl) is deprecated; use startSwarm(bootstrapAddrs) instead"
        );
        let relay_multiaddr = relay_url_to_multiaddr(&relay_url)
            .map_err(|e| js_value_from_str(&format!("Invalid relay URL: {}", e)))?;

        start_swarm_runtime(
            std::sync::Arc::clone(&self.inner),
            Rc::clone(&self.rx_messages),
            Rc::clone(&self.settings),
            Rc::clone(&self.swarm_handle),
            vec![relay_multiaddr],
        )
        .await
    }

    /// Drain and return all messages that have arrived since the last call.
    ///
    /// Returns a `js_sys::Array` of plain JS objects with the same shape as
    /// the object returned by `receiveMessage`:
    /// `{ id, senderId, text, timestamp }`.
    ///
    /// The internal buffer is cleared on each call; messages are not duplicated
    /// across successive calls.
    #[wasm_bindgen(js_name = drainReceivedMessages)]
    pub fn drain_received_messages(&self) -> js_sys::Array {
        let mut buf = self.rx_messages.borrow_mut();
        let drained: Vec<WasmMessage> = buf.drain(..).collect();
        drop(buf);

        let array = js_sys::Array::new();
        for msg in drained {
            if let Ok(js_val) = serde_wasm_bindgen::to_value(&msg) {
                array.push(&js_val);
            }
        }
        array
    }

    // ── Settings Management ──────────────────────────────────────────────

    /// Return the current MeshSettings as a JS object.
    #[wasm_bindgen(js_name = getSettings)]
    pub fn get_settings(&self) -> JsValue {
        let s = self.settings.borrow();
        to_js_value_safe(&WasmMeshSettings::from(s.clone()))
    }

    /// Apply a partial or full settings update from JS.
    /// Accepts a JS object matching the WasmMeshSettings shape.
    #[wasm_bindgen(js_name = updateSettings)]
    pub fn update_settings(&self, js_settings: JsValue) -> Result<(), JsValue> {
        let wasm_settings: WasmMeshSettings = serde_wasm_bindgen::from_value(js_settings)
            .map_err(|e| js_value_from_str(&format!("Invalid settings: {}", e)))?;
        let settings: MeshSettings = wasm_settings.into();

        // Persist if we have a storage manager
        if let Some(ref mgr) = self.settings_manager {
            mgr.save(settings.clone())
                .map_err(|e| js_value_from_str(&format!("Failed to save settings: {:?}", e)))?;
        }

        *self.settings.borrow_mut() = settings.clone();

        // P1_CORE_001: Sync drift protocol state with relay toggle
        if settings.relay_enabled {
            self.inner.drift_activate();
        } else {
            self.inner.drift_deactivate();
        }

        Ok(())
    }

    /// Return the default settings for the Web platform.
    #[wasm_bindgen(js_name = getDefaultSettings)]
    pub fn get_default_settings(&self) -> JsValue {
        to_js_value_safe(&WasmMeshSettings::from(MeshSettings {
            battery_floor: 0,
            ble_enabled: false,
            wifi_aware_enabled: false,
            wifi_direct_enabled: false,
            internet_enabled: true,
            ..MeshSettings::default()
        }))
    }

    #[wasm_bindgen(js_name = classifyNotification)]
    pub fn classify_notification(
        &self,
        js_message: JsValue,
        js_ui_state: JsValue,
    ) -> Result<JsValue, JsValue> {
        let message: WasmNotificationMessageContext = serde_wasm_bindgen::from_value(js_message)
            .map_err(|e| js_value_from_str(&format!("Invalid notification message: {}", e)))?;
        let ui_state: WasmNotificationUiState = serde_wasm_bindgen::from_value(js_ui_state)
            .map_err(|e| js_value_from_str(&format!("Invalid notification UI state: {}", e)))?;
        let settings: scmessenger_core::MeshSettings = self.settings.borrow().clone().into();
        let decision = self
            .inner
            .classify_notification(message.into(), ui_state.into(), settings);
        serde_wasm_bindgen::to_value(&WasmNotificationDecision::from(decision))
            .map_err(|e| js_value_from_str(&format!("Failed to serialize decision: {}", e)))
    }

    /// Unregister a notification endpoint by ID.
    #[wasm_bindgen(js_name = unregisterNotificationEndpoint)]
    pub fn unregister_notification_endpoint(&self, endpoint_id: String) -> Result<(), JsValue> {
        self.inner
            .unregister_notification_endpoint(&endpoint_id)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// Touch (update last-seen timestamp for) a notification endpoint.
    #[wasm_bindgen(js_name = touchNotificationEndpoint)]
    pub fn touch_notification_endpoint(&self, endpoint_id: String) -> Result<(), JsValue> {
        self.inner
            .touch_notification_endpoint(&endpoint_id)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    // ── Identity Management ──────────────────────────────────────────────

    /// Set the nickname for the local identity.
    #[wasm_bindgen(js_name = setNickname)]
    pub fn set_nickname(&self, nickname: String) -> Result<(), JsValue> {
        self.inner
            .set_nickname(nickname)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// Export the local identity as a backup string (for import on another device).
    #[wasm_bindgen(js_name = exportIdentityBackup)]
    pub fn export_identity_backup(&self, passphrase: String) -> Result<String, JsValue> {
        self.inner
            .export_identity_backup(passphrase)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// Import an identity from an encrypted backup string produced by `exportIdentityBackup`.
    /// Requires the same passphrase that was used during export.
    #[wasm_bindgen(js_name = importIdentityBackup)]
    pub fn import_identity_backup(
        &self,
        backup: String,
        passphrase: String,
    ) -> Result<(), JsValue> {
        self.inner
            .import_identity_backup(backup, passphrase)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// Derive the Ed25519 public key hex from a libp2p PeerId string.
    #[wasm_bindgen(js_name = extractPublicKeyFromPeerId)]
    pub fn extract_public_key_from_peer_id(&self, peer_id: String) -> Result<String, JsValue> {
        self.inner
            .extract_public_key_from_peer_id(peer_id)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    // ── Messaging (extended) ─────────────────────────────────────────────

    /// Prepare an encrypted message envelope and return both the message ID
    /// and the raw envelope bytes. Use the ID to track delivery receipts.
    #[wasm_bindgen(js_name = prepareMessageWithId)]
    pub fn prepare_message_with_id(
        &self,
        recipient_public_key_hex: String,
        text: String,
    ) -> Result<JsValue, JsValue> {
        ensure_mesh_participation_enabled(self.settings.borrow().relay_enabled)?;
        self.inner
            .prepare_message_with_id(
                recipient_public_key_hex,
                text,
                scmessenger_core::MessageType::Text,
                None,
            )
            .map(|p| {
                to_js_value_safe(&WasmPreparedMessage {
                    message_id: p.message_id,
                    envelope_data: p.envelope_data,
                })
            })
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// Prepare a delivery receipt envelope to send back to the original sender.
    /// Call this after successfully decoding a received message.
    #[wasm_bindgen(js_name = prepareReceipt)]
    pub fn prepare_receipt(
        &self,
        recipient_public_key_hex: String,
        message_id: String,
    ) -> Result<Vec<u8>, JsValue> {
        self.inner
            .prepare_receipt(recipient_public_key_hex, message_id)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// Generate a cover traffic payload — random bytes that look like an
    /// encrypted message. Broadcast via `sendPreparedEnvelope` or the
    /// swarm's send-to-all to obscure real traffic patterns.
    /// `sizeBytes` is clamped to [16, 1024].
    #[wasm_bindgen(js_name = prepareCoverTraffic)]
    pub fn prepare_cover_traffic(&self, size_bytes: u32) -> Result<Vec<u8>, JsValue> {
        self.inner
            .prepare_cover_traffic(size_bytes)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// Remove a message from the Rust outbox after it has been delivered.
    /// Returns `true` if the message was found and removed, `false` if not found.
    #[wasm_bindgen(js_name = markMessageSent)]
    pub fn mark_message_sent(&self, message_id: String) -> bool {
        self.inner.mark_message_sent(message_id)
    }

    #[wasm_bindgen(js_name = getContactManager)]
    pub fn get_contact_manager(&self) -> WasmContactManager {
        WasmContactManager {
            inner: self.inner.contacts_store_manager(),
        }
    }

    #[wasm_bindgen(js_name = getHistoryManager)]
    pub fn get_history_manager(&self) -> WasmHistoryManager {
        WasmHistoryManager {
            inner: self.inner.history_store_manager(),
        }
    }

    /// Return the federated nickname for a contact (the nickname advertised by the peer).
    /// Returns None if the contact is not found or has no federated nickname set.
    #[wasm_bindgen(js_name = contactFederatedNickname)]
    pub fn contact_federated_nickname(&self, peer_id: String) -> Option<String> {
        self.inner.contact_federated_nickname(peer_id)
    }

    /// Get the signable data for an invite token.
    /// Returns serialized token data (without signature) suitable for Ed25519 signing.
    /// The token_bytes must be a bincode-serialized InviteToken.
    #[wasm_bindgen(js_name = getInviteSignableData)]
    pub fn get_invite_signable_data(&self, token_bytes: Vec<u8>) -> Result<Vec<u8>, JsValue> {
        self.inner
            .invite_get_signable_data(token_bytes)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    // ── Identity Resolution ──────────────────────────────────────────────

    /// Resolve any identifier format (peer ID, identity hash, public key hex)
    /// to the canonical public_key_hex.
    #[wasm_bindgen(js_name = resolveIdentity)]
    pub fn resolve_identity(&self, any_id: String) -> Result<String, JsValue> {
        self.inner
            .resolve_identity(any_id)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// Resolve any identifier format to the identity_id (Blake3 hash).
    #[wasm_bindgen(js_name = resolveToIdentityId)]
    pub fn resolve_to_identity_id(&self, any_id: String) -> Result<String, JsValue> {
        self.inner
            .resolve_to_identity_id(any_id)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    // ── Blocking ─────────────────────────────────────────────────────────

    /// Block a peer by ID, with an optional reason string.
    #[wasm_bindgen(js_name = blockPeer)]
    pub fn block_peer(&self, peer_id: String, reason: Option<String>) -> Result<(), JsValue> {
        self.inner
            .block_peer(peer_id, None, reason)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// Unblock a previously blocked peer.
    #[wasm_bindgen(js_name = unblockPeer)]
    pub fn unblock_peer(&self, peer_id: String) -> Result<(), JsValue> {
        self.inner
            .unblock_peer(peer_id, None)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// Block a peer AND delete all their stored messages (cascade purge).
    /// Future payloads from this peer are dropped at the ingress layer.
    #[wasm_bindgen(js_name = blockAndDeletePeer)]
    pub fn block_and_delete_peer(
        &self,
        peer_id: String,
        reason: Option<String>,
    ) -> Result<(), JsValue> {
        self.inner
            .block_and_delete_peer(peer_id, None, reason)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// Check whether a peer is currently blocked.
    #[wasm_bindgen(js_name = isPeerBlocked)]
    pub fn is_peer_blocked(&self, peer_id: String) -> Result<bool, JsValue> {
        self.inner
            .is_peer_blocked(peer_id, None)
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// List all blocked peers. Returns a JS array of BlockedIdentity objects.
    #[wasm_bindgen(js_name = listBlockedPeers)]
    pub fn list_blocked_peers(&self) -> Result<js_sys::Array, JsValue> {
        let list = self
            .inner
            .list_blocked_peers_raw()
            .map_err(|e| js_value_from_str(&format!("{}", e)))?;
        let array = js_sys::Array::new();
        for item in list {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("peerId"),
                &JsValue::from_str(&item.peer_id),
            )
            .map_err(|e| js_value_from_str(&format!("Failed to set peerId: {:?}", e)))?;
            if let Some(did) = item.device_id.as_deref() {
                js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("deviceId"),
                    &JsValue::from_str(did),
                )
                .map_err(|e| js_value_from_str(&format!("Failed to set deviceId: {:?}", e)))?;
            }
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("blockedAt"),
                &JsValue::from_f64(item.blocked_at as f64),
            )
            .map_err(|e| js_value_from_str(&format!("Failed to set blockedAt: {:?}", e)))?;
            if let Some(reason) = item.reason.as_deref() {
                js_sys::Reflect::set(
                    &obj,
                    &JsValue::from_str("reason"),
                    &JsValue::from_str(reason),
                )
                .map_err(|e| js_value_from_str(&format!("Failed to set reason: {:?}", e)))?;
            }
            if let Some(notes) = item.notes.as_deref() {
                js_sys::Reflect::set(&obj, &JsValue::from_str("notes"), &JsValue::from_str(notes))
                    .map_err(|e| js_value_from_str(&format!("Failed to set notes: {:?}", e)))?;
            }
            js_sys::Reflect::set(
                &obj,
                &JsValue::from_str("isDeleted"),
                &JsValue::from_bool(item.is_deleted),
            )
            .map_err(|e| js_value_from_str(&format!("Failed to set isDeleted: {:?}", e)))?;
            array.push(&obj);
        }
        Ok(array)
    }

    /// Get the count of blocked peers.
    #[wasm_bindgen(js_name = blockedCount)]
    pub fn blocked_count(&self) -> Result<u32, JsValue> {
        self.inner
            .blocked_count()
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    // ── Device & Registration ────────────────────────────────────────────

    /// Get the installation-local device ID (UUIDv4).
    #[wasm_bindgen(js_name = getDeviceId)]
    pub fn get_device_id(&self) -> Option<String> {
        self.inner.get_device_id()
    }

    /// Get the seniority timestamp for this installation.
    #[wasm_bindgen(js_name = getSeniorityTimestamp)]
    pub fn get_seniority_timestamp(&self) -> Option<u64> {
        self.inner.get_seniority_timestamp()
    }

    /// Get the registration state for a given identity.
    #[wasm_bindgen(js_name = getRegistrationState)]
    pub fn get_registration_state(&self, identity_id: String) -> JsValue {
        let info = self.inner.get_registration_state(identity_id);
        to_js_value_safe(&WasmRegistrationStateInfo {
            state: info.state,
            device_id: info.device_id,
            seniority_timestamp: info.seniority_timestamp,
        })
    }

    // ── DSPy Integrity ────────────────────────────────────────────────────

    /// Compute a BLAKE3 hash of the given data.
    /// Returns the 32-byte hash as a hex-encoded string.
    #[wasm_bindgen(js_name = blake3Hash)]
    pub fn blake3_hash(&self, data: Vec<u8>) -> String {
        let hash = self.inner.dspy_blake3_hash(&data);
        hash.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Get a DSPy signature description by role name.
    /// Returns the signature description string if the role is found.
    #[wasm_bindgen(js_name = getSignature)]
    pub fn get_signature(&self, role: String) -> Option<String> {
        self.inner.dspy_get_signature(&role)
    }

    // ── Maintenance & Logging ────────────────────────────────────────────

    /// Perform storage maintenance (quota enforcement, cleanup).
    #[wasm_bindgen(js_name = performMaintenance)]
    pub fn perform_maintenance(&self) -> Result<(), JsValue> {
        self.inner
            .perform_maintenance()
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    /// Update disk usage statistics for quota enforcement.
    #[wasm_bindgen(js_name = updateDiskStats)]
    pub fn update_disk_stats(&self, total_bytes: u64, free_bytes: u64) {
        self.inner.update_disk_stats(total_bytes, free_bytes);
    }

    /// Get current disk usage statistics.
    #[wasm_bindgen(js_name = getDiskStats)]
    pub fn get_disk_stats(&self) -> JsValue {
        let stats = self.inner.get_disk_stats();
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &obj,
            &js_sys::JsString::from("totalBytes"),
            &js_sys::Number::from(stats.total_bytes as f64),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &js_sys::JsString::from("freeBytes"),
            &js_sys::Number::from(stats.free_bytes as f64),
        )
        .ok();
        js_sys::Reflect::set(
            &obj,
            &js_sys::JsString::from("appDataBytes"),
            &js_sys::Number::from(stats.app_data_bytes as f64),
        )
        .ok();
        obj.into()
    }

    /// Record a log line into the core log manager.
    #[wasm_bindgen(js_name = recordLog)]
    pub fn record_log(&self, line: String) {
        self.inner.record_log(line);
    }

    /// Export all recorded log entries as a single string.
    #[wasm_bindgen(js_name = exportLogs)]
    pub fn export_logs(&self) -> Result<String, JsValue> {
        self.inner
            .export_logs()
            .map_err(|e| js_value_from_str(&format!("{}", e)))
    }

    // ── Peer Notifications ───────────────────────────────────────────────

    /// Notify the core that a peer was discovered on the network.
    #[wasm_bindgen(js_name = notifyPeerDiscovered)]
    pub fn notify_peer_discovered(&self, peer_id: String) {
        self.inner.notify_peer_discovered(peer_id);
    }

    /// Notify the core that a peer disconnected from the network.
    #[wasm_bindgen(js_name = notifyPeerDisconnected)]
    pub fn notify_peer_disconnected(&self, peer_id: String) {
        self.inner.notify_peer_disconnected(peer_id);
    }
}

#[wasm_bindgen]
pub struct WasmContactManager {
    inner: scmessenger_core::store::ContactManager,
}

#[wasm_bindgen]
impl WasmContactManager {
    #[wasm_bindgen(js_name = add)]
    pub fn add(&self, js_contact: JsValue) -> Result<(), JsValue> {
        let contact: scmessenger_core::store::Contact = serde_wasm_bindgen::from_value(js_contact)
            .map_err(|e| js_value_from_str(&format!("Invalid contact: {}", e)))?;
        self.inner
            .add(contact)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))
    }

    #[wasm_bindgen(js_name = get)]
    pub fn get(&self, peer_id: String) -> Result<JsValue, JsValue> {
        let contact = self
            .inner
            .get(peer_id)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))?;
        Ok(to_js_value_safe(&contact))
    }

    #[wasm_bindgen(js_name = remove)]
    pub fn remove(&self, peer_id: String) -> Result<(), JsValue> {
        self.inner
            .remove(peer_id)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))
    }

    #[wasm_bindgen(js_name = list)]
    pub fn list(&self) -> Result<js_sys::Array, JsValue> {
        let list = self
            .inner
            .list()
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))?;
        let array = js_sys::Array::new();
        for item in list {
            array.push(&to_js_value_safe(&item));
        }
        Ok(array)
    }

    #[wasm_bindgen(js_name = count)]
    pub fn count(&self) -> u32 {
        self.inner.count()
    }

    #[wasm_bindgen(js_name = setLocalNickname)]
    pub fn set_local_nickname(
        &self,
        peer_id: String,
        nickname: Option<String>,
    ) -> Result<(), JsValue> {
        self.inner
            .set_local_nickname(peer_id, nickname)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))
    }

    /// Search contacts by query string (matches peer_id, nickname, notes).
    #[wasm_bindgen(js_name = search)]
    pub fn search(&self, query: String) -> Result<js_sys::Array, JsValue> {
        let results = self
            .inner
            .search(query)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))?;
        let array = js_sys::Array::new();
        for item in results {
            array.push(&to_js_value_safe(&item));
        }
        Ok(array)
    }

    /// Set the federated (broadcast) nickname for a contact.
    #[wasm_bindgen(js_name = setNickname)]
    pub fn set_nickname(&self, peer_id: String, nickname: Option<String>) -> Result<(), JsValue> {
        self.inner
            .set_nickname(peer_id, nickname)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))
    }

    /// Update the last-seen timestamp for a contact.
    #[wasm_bindgen(js_name = updateLastSeen)]
    pub fn update_last_seen(&self, peer_id: String) -> Result<(), JsValue> {
        self.inner
            .update_last_seen(peer_id)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))
    }

    /// Update the last known device ID for a contact.
    #[wasm_bindgen(js_name = updateDeviceId)]
    pub fn update_device_id(
        &self,
        peer_id: String,
        device_id: Option<String>,
    ) -> Result<(), JsValue> {
        self.inner
            .update_last_known_device_id(peer_id, device_id)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))
    }

    /// Flush contact data to persistent storage.
    #[wasm_bindgen(js_name = flush)]
    pub fn flush(&self) {
        self.inner.flush();
    }
}

#[wasm_bindgen]
pub struct WasmHistoryManager {
    inner: scmessenger_core::store::HistoryManager,
}

#[derive(serde::Serialize)]
struct WasmHistoryStats {
    total_messages: u32,
    sent_count: u32,
    received_count: u32,
    undelivered_count: u32,
}

#[wasm_bindgen]
impl WasmHistoryManager {
    #[wasm_bindgen(js_name = add)]
    pub fn add(&self, js_record: JsValue) -> Result<(), JsValue> {
        let record: scmessenger_core::store::MessageRecord =
            serde_wasm_bindgen::from_value(js_record)
                .map_err(|e| js_value_from_str(&format!("Invalid record: {}", e)))?;
        self.inner
            .add(record)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))
    }

    #[wasm_bindgen(js_name = recent)]
    pub fn recent(
        &self,
        peer_filter: Option<String>,
        limit: u32,
    ) -> Result<js_sys::Array, JsValue> {
        let records = self
            .inner
            .recent(peer_filter, limit)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))?;
        let array = js_sys::Array::new();
        for rec in records {
            array.push(&to_js_value_safe(&rec));
        }
        Ok(array)
    }

    #[wasm_bindgen(js_name = conversation)]
    pub fn conversation(&self, peer_id: String, limit: u32) -> Result<js_sys::Array, JsValue> {
        let records = self
            .inner
            .conversation(peer_id, limit)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))?;
        let array = js_sys::Array::new();
        for rec in records {
            array.push(&to_js_value_safe(&rec));
        }
        Ok(array)
    }

    #[wasm_bindgen(js_name = clear)]
    pub fn clear(&self) -> Result<(), JsValue> {
        self.inner
            .clear()
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))
    }

    #[wasm_bindgen(js_name = stats)]
    pub fn stats(&self) -> Result<JsValue, JsValue> {
        let stats = self
            .inner
            .stats()
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))?;
        let payload = WasmHistoryStats {
            total_messages: stats.total_messages,
            sent_count: stats.sent_count,
            received_count: stats.received_count,
            undelivered_count: stats.undelivered_count,
        };
        Ok(to_js_value_safe(&payload))
    }

    #[wasm_bindgen(js_name = count)]
    pub fn count(&self) -> u32 {
        self.inner.count()
    }

    #[wasm_bindgen(js_name = enforceRetention)]
    pub fn enforce_retention(&self, max_messages: u32) -> Result<u32, JsValue> {
        self.inner
            .enforce_retention(max_messages)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))
    }

    #[wasm_bindgen(js_name = pruneBefore)]
    pub fn prune_before(&self, before_timestamp: u64) -> Result<u32, JsValue> {
        self.inner
            .prune_before(before_timestamp)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))
    }

    /// Get a single message record by ID.
    #[wasm_bindgen(js_name = get)]
    pub fn get(&self, id: String) -> Result<JsValue, JsValue> {
        let record = self
            .inner
            .get(id)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))?;
        Ok(to_js_value_safe(&record))
    }

    /// Search message history by query string.
    #[wasm_bindgen(js_name = search)]
    pub fn search(&self, query: String, limit: u32) -> Result<js_sys::Array, JsValue> {
        let records = self
            .inner
            .search(query, limit)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))?;
        let array = js_sys::Array::new();
        for rec in records {
            array.push(&to_js_value_safe(&rec));
        }
        Ok(array)
    }

    /// Mark a message as delivered by ID.
    #[wasm_bindgen(js_name = markDelivered)]
    pub fn mark_delivered(&self, id: String) -> Result<(), JsValue> {
        self.inner
            .mark_delivered(id)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))
    }

    /// Clear all messages in a conversation with a specific peer.
    /// Alias: this is equivalent to `removeConversation` in the core API.
    #[wasm_bindgen(js_name = clearConversation)]
    pub fn clear_conversation(&self, peer_id: String) -> Result<(), JsValue> {
        self.inner
            .remove_conversation(peer_id)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))
    }

    /// Delete a single message by ID.
    #[wasm_bindgen(js_name = delete)]
    pub fn delete(&self, id: String) -> Result<(), JsValue> {
        self.inner
            .delete(id)
            .map_err(|e| js_value_from_str(&format!("{:?}", e)))
    }

    /// Flush history data to persistent storage.
    #[wasm_bindgen(js_name = flush)]
    pub fn flush(&self) {
        self.inner.flush();
    }
}

fn js_value_from_str(message: &str) -> JsValue {
    #[cfg(target_arch = "wasm32")]
    {
        JsValue::from_str(message)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = message;
        JsValue::NULL
    }
}

/// Safely convert a serializable value to JsValue, returning JsValue::NULL on error.
/// This prevents panics from serde_wasm_bindgen::to_value() failures.
fn to_js_value_safe<T: serde::Serialize>(value: &T) -> JsValue {
    serde_wasm_bindgen::to_value(value).unwrap_or(JsValue::NULL)
}

fn parse_bootstrap_addrs(value: JsValue) -> Result<Vec<String>, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(Vec::new());
    }

    serde_wasm_bindgen::from_value(value)
        .map_err(|e| js_value_from_str(&format!("bootstrapAddrs must be string[]: {}", e)))
}

fn relay_url_to_multiaddr(relay_url: &str) -> Result<String, String> {
    let (is_secure, rest) = if let Some(rest) = relay_url.strip_prefix("wss://") {
        (true, rest)
    } else if let Some(rest) = relay_url.strip_prefix("ws://") {
        (false, rest)
    } else {
        return Err("URL must start with ws:// or wss://".to_string());
    };

    let authority = rest
        .split('/')
        .next()
        .ok_or_else(|| "Missing relay host".to_string())?;
    if authority.is_empty() {
        return Err("Missing relay host".to_string());
    }

    let (host, port) = if let Some((host, port_str)) = authority.rsplit_once(':') {
        if host.contains(':') && !host.starts_with('[') {
            (authority.to_string(), if is_secure { 443 } else { 80 })
        } else {
            let parsed_port = port_str
                .parse::<u16>()
                .map_err(|_| format!("Invalid relay port: {}", port_str))?;
            (
                host.trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string(),
                parsed_port,
            )
        }
    } else {
        (
            authority
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string(),
            if is_secure { 443 } else { 80 },
        )
    };

    let host_segment = if host.parse::<std::net::Ipv4Addr>().is_ok() {
        format!("/ip4/{}", host)
    } else if host.parse::<std::net::Ipv6Addr>().is_ok() {
        format!("/ip6/{}", host)
    } else {
        format!("/dns4/{}", host)
    };

    let ws_segment = if is_secure { "wss" } else { "ws" };
    Ok(format!("{}/tcp/{}/{}", host_segment, port, ws_segment))
}

fn ensure_mesh_participation_enabled(relay_enabled: bool) -> Result<(), JsValue> {
    if relay_enabled {
        return Ok(());
    }
    Err(js_value_from_str(
        "Messaging blocked: mesh participation is disabled (relay toggle OFF)",
    ))
}

async fn start_swarm_runtime(
    inner: std::sync::Arc<RustIronCore>,
    rx_messages: Rc<RefCell<Vec<WasmMessage>>>,
    settings: Rc<RefCell<MeshSettings>>,
    swarm_handle: Rc<RefCell<Option<scmessenger_core::transport::SwarmHandle>>>,
    bootstrap_addrs: Vec<String>,
) -> Result<(), JsValue> {
    if swarm_handle.borrow().is_some() {
        return Err(js_value_from_str("Swarm is already running"));
    }

    if !inner.is_running() {
        inner
            .start()
            .map_err(|e| js_value_from_str(&format!("Failed to start core: {}", e)))?;
    }

    let (libp2p_keys, headless_mode) = resolve_swarm_keypair_and_mode(inner.as_ref())?;

    let bootstrap_multiaddrs: Vec<Multiaddr> = bootstrap_addrs
        .iter()
        .map(|raw| {
            raw.parse::<Multiaddr>().map_err(|e| {
                js_value_from_str(&format!("Invalid bootstrap multiaddr '{}': {}", raw, e))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(100);
    let handle: scmessenger_core::transport::SwarmHandle =
        scmessenger_core::transport::start_swarm_with_config(
            libp2p_keys,
            None,
            event_tx,
            None,
            bootstrap_multiaddrs,
            None,
            Some(std::sync::Arc::downgrade(&inner)),
            headless_mode,
            None, // discovery_config
            scmessenger_core::transport::default_routing_engine_handle(),
        )
        .await
        .map_err(|e: anyhow::Error| js_value_from_str(&format!("Failed to start swarm: {}", e)))?;

    *swarm_handle.borrow_mut() = Some(handle);

    let swarm_handle_for_loop: Rc<RefCell<Option<scmessenger_core::transport::SwarmHandle>>> =
        Rc::clone(&swarm_handle);
    wasm_bindgen_futures::spawn_local(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                scmessenger_core::transport::SwarmEvent::MessageReceived {
                    peer_id,
                    envelope_data,
                } => {
                    if !settings.borrow().relay_enabled {
                        tracing::debug!("Dropping swarm inbound frame: relay toggle OFF");
                        continue;
                    }

                    match inner.receive_message(envelope_data) {
                        Ok(msg) => {
                            rx_messages.borrow_mut().push(WasmMessage {
                                id: msg.id.clone(),
                                sender_id: msg.sender_id.clone(),
                                sender_peer_id: Some(peer_id.to_string()),
                                text: msg.text_content(),
                                timestamp: msg.timestamp,
                            });
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to decode swarm message from {}: {:?}",
                                peer_id,
                                e
                            );
                        }
                    }
                }
                scmessenger_core::transport::SwarmEvent::PeerDiscovered(peer_id) => {
                    inner.notify_peer_discovered(peer_id.to_string());
                    // Flush any queued outbox messages for this peer
                    let pid_str = peer_id.to_string();
                    let flushed = inner.flush_outbox_for_peer(&pid_str);
                    if !flushed.is_empty() {
                        tracing::info!(
                            event = "wasm_outbox_flushed",
                            peer = %pid_str,
                            count = flushed.len()
                        );
                    }
                }
                scmessenger_core::transport::SwarmEvent::PeerDisconnected(peer_id) => {
                    inner.notify_peer_disconnected(peer_id.to_string());
                }
                scmessenger_core::transport::SwarmEvent::PeerIdentified { peer_id, .. } => {
                    tracing::info!("Swarm identified peer {}", peer_id);
                }
                scmessenger_core::transport::SwarmEvent::ListenerFailed { listener_id, error } => {
                    tracing::warn!("Swarm listener {} failed: {}", listener_id, error);
                }
                scmessenger_core::transport::SwarmEvent::AddressReflected { .. }
                | scmessenger_core::transport::SwarmEvent::ListeningOn(_)
                | scmessenger_core::transport::SwarmEvent::PortMapping(_)
                | scmessenger_core::transport::SwarmEvent::TopicDiscovered { .. }
                | scmessenger_core::transport::SwarmEvent::LedgerReceived { .. }
                | scmessenger_core::transport::SwarmEvent::NatStatusChanged(_)
                | scmessenger_core::transport::SwarmEvent::AbuseSignalDetected { .. }
                | scmessenger_core::transport::SwarmEvent::RelayCircuitEstablished
                | scmessenger_core::transport::SwarmEvent::RelayCircuitBroken => {}
            }
        }

        *swarm_handle_for_loop.borrow_mut() = None;
        tracing::info!("WASM swarm event loop terminated");
    });

    Ok(())
}

fn resolve_swarm_keypair_and_mode(
    inner: &RustIronCore,
) -> Result<(libp2p::identity::Keypair, bool), JsValue> {
    if let Some(identity_keys) = inner.get_identity_keys() {
        let libp2p_keys: libp2p::identity::Keypair =
            identity_keys.to_libp2p_keypair().map_err(|e: Error| {
                js_value_from_str(&format!(
                    "Failed to derive libp2p keypair from identity: {}",
                    e
                ))
            })?;
        return Ok((libp2p_keys, false));
    }

    tracing::info!("No identity available; starting swarm in relay-only mode");
    Ok((libp2p::identity::Keypair::generate_ed25519(), true))
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmIdentityInfo {
    identity_id: Option<String>,
    public_key_hex: Option<String>,
    device_id: Option<String>,
    seniority_timestamp: Option<u64>,
    initialized: bool,
    nickname: Option<String>,
    libp2p_peer_id: Option<String>,
}

impl From<IdentityInfo> for WasmIdentityInfo {
    fn from(info: IdentityInfo) -> Self {
        Self {
            identity_id: info.identity_id,
            public_key_hex: info.public_key_hex,
            device_id: info.device_id,
            seniority_timestamp: info.seniority_timestamp,
            initialized: info.initialized,
            nickname: info.nickname,
            libp2p_peer_id: info.libp2p_peer_id,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmSignatureResult {
    signature: Vec<u8>,
    public_key_hex: String,
}

impl From<SignatureResult> for WasmSignatureResult {
    fn from(sig: SignatureResult) -> Self {
        Self {
            signature: sig.signature,
            public_key_hex: sig.public_key_hex,
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct WasmRegistrationStateInfo {
    state: String,
    device_id: Option<String>,
    seniority_timestamp: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmPreparedMessage {
    message_id: String,
    envelope_data: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmMessage {
    id: String,
    sender_id: String,
    sender_peer_id: Option<String>,
    text: Option<String>,
    timestamp: u64,
}

/// Web-facing MeshSettings with camelCase field names for JS interop.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmMeshSettings {
    relay_enabled: bool,
    max_relay_budget: u32,
    battery_floor: u8,
    ble_enabled: bool,
    wifi_aware_enabled: bool,
    wifi_direct_enabled: bool,
    internet_enabled: bool,
    discovery_mode: String,
    onion_routing: bool,
    cover_traffic_enabled: bool,
    message_padding_enabled: bool,
    timing_obfuscation_enabled: bool,
    notifications_enabled: bool,
    notify_dm_enabled: bool,
    notify_dm_request_enabled: bool,
    notify_dm_in_foreground: bool,
    notify_dm_request_in_foreground: bool,
    sound_enabled: bool,
    badge_enabled: bool,
}

impl From<MeshSettings> for WasmMeshSettings {
    fn from(s: MeshSettings) -> Self {
        Self {
            relay_enabled: s.relay_enabled,
            max_relay_budget: s.max_relay_budget,
            battery_floor: s.battery_floor,
            ble_enabled: s.ble_enabled,
            wifi_aware_enabled: s.wifi_aware_enabled,
            wifi_direct_enabled: s.wifi_direct_enabled,
            internet_enabled: s.internet_enabled,
            discovery_mode: match s.discovery_mode {
                DiscoveryMode::Normal => "normal".to_string(),
                DiscoveryMode::Cautious => "cautious".to_string(),
                DiscoveryMode::Paranoid => "paranoid".to_string(),
            },
            onion_routing: s.onion_routing,
            cover_traffic_enabled: s.cover_traffic_enabled,
            message_padding_enabled: s.message_padding_enabled,
            timing_obfuscation_enabled: s.timing_obfuscation_enabled,
            notifications_enabled: s.notifications_enabled,
            notify_dm_enabled: s.notify_dm_enabled,
            notify_dm_request_enabled: s.notify_dm_request_enabled,
            notify_dm_in_foreground: s.notify_dm_in_foreground,
            notify_dm_request_in_foreground: s.notify_dm_request_in_foreground,
            sound_enabled: s.sound_enabled,
            badge_enabled: s.badge_enabled,
        }
    }
}

impl From<WasmMeshSettings> for MeshSettings {
    fn from(w: WasmMeshSettings) -> Self {
        Self {
            relay_enabled: w.relay_enabled,
            max_relay_budget: w.max_relay_budget,
            battery_floor: w.battery_floor,
            ble_enabled: w.ble_enabled,
            wifi_aware_enabled: w.wifi_aware_enabled,
            wifi_direct_enabled: w.wifi_direct_enabled,
            internet_enabled: w.internet_enabled,
            discovery_mode: match w.discovery_mode.as_str() {
                "cautious" => DiscoveryMode::Cautious,
                "paranoid" => DiscoveryMode::Paranoid,
                _ => DiscoveryMode::Normal,
            },
            onion_routing: w.onion_routing,
            cover_traffic_enabled: w.cover_traffic_enabled,
            message_padding_enabled: w.message_padding_enabled,
            timing_obfuscation_enabled: w.timing_obfuscation_enabled,
            notifications_enabled: w.notifications_enabled,
            notify_dm_enabled: w.notify_dm_enabled,
            notify_dm_request_enabled: w.notify_dm_request_enabled,
            notify_dm_in_foreground: w.notify_dm_in_foreground,
            notify_dm_request_in_foreground: w.notify_dm_request_in_foreground,
            sound_enabled: w.sound_enabled,
            badge_enabled: w.badge_enabled,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmNotificationMessageContext {
    conversation_id: Option<String>,
    sender_peer_id: String,
    message_id: String,
    explicit_dm_request: Option<bool>,
    sender_is_known_contact: bool,
    has_existing_conversation: bool,
    is_self_originated: bool,
    is_duplicate: bool,
    already_seen: bool,
    is_blocked: bool,
}

impl From<WasmNotificationMessageContext> for NotificationMessageContext {
    fn from(value: WasmNotificationMessageContext) -> Self {
        Self {
            conversation_id: value.conversation_id,
            sender_peer_id: value.sender_peer_id,
            message_id: value.message_id,
            explicit_dm_request: value.explicit_dm_request,
            sender_is_known_contact: value.sender_is_known_contact,
            has_existing_conversation: value.has_existing_conversation,
            is_self_originated: value.is_self_originated,
            is_duplicate: value.is_duplicate,
            already_seen: value.already_seen,
            is_blocked: value.is_blocked,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmNotificationUiState {
    app_in_foreground: bool,
    active_conversation_id: Option<String>,
}

impl From<WasmNotificationUiState> for NotificationUiState {
    fn from(value: WasmNotificationUiState) -> Self {
        Self {
            app_in_foreground: value.app_in_foreground,
            active_conversation_id: value.active_conversation_id,
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct WasmNotificationDecision {
    kind: String,
    conversation_id: String,
    sender_peer_id: String,
    message_id: String,
    should_alert: bool,
    suppression_reason: Option<String>,
}

impl From<NotificationDecision> for WasmNotificationDecision {
    fn from(value: NotificationDecision) -> Self {
        let kind = match value.kind {
            scmessenger_core::NotificationKind::DirectMessage => "directMessage",
            scmessenger_core::NotificationKind::DirectMessageRequest => "directMessageRequest",
            scmessenger_core::NotificationKind::None => "none",
        }
        .to_string();

        Self {
            kind,
            conversation_id: value.conversation_id,
            sender_peer_id: value.sender_peer_id,
            message_id: value.message_id,
            should_alert: value.should_alert,
            suppression_reason: value.suppression_reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scmessenger_core::store::{Contact, MessageDirection, MessageRecord};
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_wasm_core_creation() {
        let core = IronCore::new();
        core.start().unwrap();
        assert!(core.is_running());
        core.stop();
        assert!(!core.is_running());
    }

    #[wasm_bindgen_test]
    fn test_wasm_identity() {
        let core = RustIronCore::new();
        core.grant_consent();
        core.initialize_identity().unwrap();
        let info = core.get_identity_info();
        assert!(info.initialized);
    }

    #[test]
    fn test_desktop_identity_flow_exposes_metadata_after_init() {
        let core = RustIronCore::with_storage(temp_storage_path("identity-flow"));
        core.start().unwrap();
        core.grant_consent();
        core.initialize_identity().unwrap();
        core.set_nickname("Desktop Node".to_string()).unwrap();

        let info = core.get_identity_info();
        assert!(info.initialized, "identity should be initialized");
        assert!(
            info.public_key_hex.is_some(),
            "initialized identity should expose public key"
        );
        assert!(
            info.device_id.is_some(),
            "identity should expose local device_id"
        );
        assert!(
            info.seniority_timestamp.is_some(),
            "identity should expose local seniority timestamp"
        );
        assert_eq!(
            info.nickname.as_deref(),
            Some("Desktop Node"),
            "nickname update should round-trip through identity surface"
        );
        assert!(
            info.libp2p_peer_id.is_some() || info.identity_id.is_some(),
            "identity surface should provide stable peer or identity metadata"
        );
    }

    #[test]
    fn test_relay_url_to_multiaddr_ws_defaults() {
        let addr = relay_url_to_multiaddr("ws://relay.example.com").unwrap();
        assert_eq!(addr, "/dns4/relay.example.com/tcp/80/ws");
    }

    #[test]
    fn test_relay_url_to_multiaddr_wss_defaults() {
        let addr = relay_url_to_multiaddr("wss://relay.example.com").unwrap();
        assert_eq!(addr, "/dns4/relay.example.com/tcp/443/wss");
    }

    #[test]
    fn test_relay_url_to_multiaddr_ipv4_port() {
        let addr = relay_url_to_multiaddr("wss://1.2.3.4:7443/mesh").unwrap();
        assert_eq!(addr, "/ip4/1.2.3.4/tcp/7443/wss");
    }

    #[test]
    fn test_relay_url_to_multiaddr_rejects_http() {
        let err = relay_url_to_multiaddr("https://relay.example.com").unwrap_err();
        assert!(err.contains("ws:// or wss://"));
    }

    #[test]
    fn test_desktop_role_resolution_defaults_to_relay_only_without_identity() {
        let core = RustIronCore::new();
        let (_, relay_only) = resolve_swarm_keypair_and_mode(&core).unwrap();
        assert!(
            relay_only,
            "desktop should start relay-only when identity is absent"
        );

        core.grant_consent();
        core.initialize_identity().unwrap();
        let (_, full_role) = resolve_swarm_keypair_and_mode(&core).unwrap();
        assert!(
            !full_role,
            "desktop should switch to full role after identity initialization"
        );
    }

    #[test]
    fn test_desktop_relay_only_flow_blocks_outbound_message_prepare() {
        let err = ensure_mesh_participation_enabled(false);
        assert!(
            err.is_err(),
            "relay-only mode should block message prepare path"
        );
        assert!(ensure_mesh_participation_enabled(true).is_ok());
    }

    #[test]
    fn test_desktop_contacts_and_messaging_interaction_flow() {
        let core = RustIronCore::with_storage(temp_storage_path("contacts-flow"));
        core.start().unwrap();
        core.grant_consent();
        core.initialize_identity().unwrap();

        let contact_manager = core.contacts_store_manager();
        let history = core.history_store_manager();

        // UNIFICATION_V2 canonicalizes a contact's stored peer_id to its public-key
        // hex even when the add-time peer id is a libp2p id (the real key is
        // derived from it). Use the initialized identity's own genuine public key
        // hex as both peer id and public key so the whole desktop flow keys on the
        // canonical hex form that add()/get() actually store and read.
        let key_hex = core
            .get_identity_info()
            .public_key_hex
            .expect("initialized identity exposes public key hex");

        let mut contact = Contact::new(key_hex.clone(), key_hex.clone());
        contact.local_nickname = Some("Alice".to_string());
        contact.last_seen = Some(1);
        contact_manager.add(contact).unwrap();

        // Verify the specific contact exists (count() may include other sled table keys).
        let fetched = contact_manager.get(key_hex.clone()).unwrap();
        assert!(fetched.is_some(), "contact should exist after add");
        assert_eq!(fetched.unwrap().local_nickname, Some("Alice".to_string()));

        let outbound = MessageRecord {
            id: "desktop-outbound-1".to_string(),
            direction: MessageDirection::Sent,
            peer_id: key_hex.clone(),
            content: "hello from desktop".to_string(),
            timestamp: 2,
            sender_timestamp: 2,
            delivered: false,
            hidden: false,
        };
        history.add(outbound).unwrap();

        let conversation = history.conversation(key_hex, 20).unwrap();
        assert_eq!(
            conversation.len(),
            1,
            "contact conversation should include saved outbound record"
        );
    }

    #[test]
    fn test_desktop_mesh_dashboard_stats_update_with_message_flow() {
        let core = RustIronCore::with_storage(temp_storage_path("dashboard-stats"));
        let history = core.history_store_manager();

        let sent = MessageRecord {
            id: "stats-sent".to_string(),
            direction: MessageDirection::Sent,
            peer_id: "peer-a".to_string(),
            content: "pending".to_string(),
            timestamp: 10,
            sender_timestamp: 10,
            delivered: false,
            hidden: false,
        };
        let received = MessageRecord {
            id: "stats-recv".to_string(),
            direction: MessageDirection::Received,
            peer_id: "peer-b".to_string(),
            content: "ack".to_string(),
            timestamp: 11,
            sender_timestamp: 11,
            delivered: true,
            hidden: false,
        };

        history.add(sent).unwrap();
        history.add(received).unwrap();

        let stats = history.stats().unwrap();
        assert_eq!(stats.total_messages, 2);
        assert_eq!(stats.sent_count, 1);
        assert_eq!(stats.received_count, 1);
        assert_eq!(stats.undelivered_count, 1);
    }

    // Notification Manager Tests
    #[wasm_bindgen_test]
    fn test_notification_manager_creation() {
        let manager = notification_manager::NotificationManager::new();
        assert!(manager.is_supported());
    }

    fn temp_storage_path(label: &str) -> String {
        let ts = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("scm-wasm-{}-{}-{}", label, ts, std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        path.to_string_lossy().to_string()
    }
}
