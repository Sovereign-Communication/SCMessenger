//! BLE GATT central path: discover SCMessenger peripherals, subscribe to notify, forward
//! decrypted payloads to the local Web UI as JSON-RPC `message_received`.
//!
//! **Advertising:** btleplug is central-oriented on desktop OSes; the CLI does not expose a
//! full peripheral GATT server here. Mobile/native peers remain peripherals; this node scans,
//! connects, and ingests notify payloads.

use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::{
    Central, CentralEvent, CharPropFlags, Manager as _, Peripheral as PeripheralApi, ScanFilter,
};
use btleplug::platform::{Manager, Peripheral};
use futures_util::StreamExt;
use scmessenger_core::drift::frame::{DriftFrame, FrameType};
use scmessenger_core::transport::ble::{GattFragmentHeader, GattReassembler};
use scmessenger_core::wasm_support::rpc::{notif_message_received, MessageReceivedParams};
use scmessenger_core::IronCore;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone)]
pub enum TrackingPeripheral {
    Real(Peripheral),
    #[cfg(test)]
    Mock(String),
}

impl TrackingPeripheral {
    pub fn address_string(&self) -> String {
        match self {
            Self::Real(p) => p.address().to_string(),
            #[cfg(test)]
            Self::Mock(mac) => mac.clone(),
        }
    }
}

use crate::server::{UiEvent, UiOutbound};

/// SCM GATT primary service UUID (must match `core/src/transport/ble/gatt.rs`).
const GATT_SERVICE_UUID: u128 = 0x0000_DF01_0000_1000_8000_0080_5F9B_34FB;
const MAX_CONNECT_ATTEMPTS: u32 = 3;
const CONNECT_BACKOFF_MS: u64 = 250;
const MAX_TRACKED_FAILURES: u32 = 6;
const MAX_INBOUND_FRAGMENTS: u16 = 1024;

static ACTIVE_PEERS: OnceLock<std::sync::Mutex<HashMap<String, TrackingPeripheral>>> =
    OnceLock::new();

fn get_active_peers() -> &'static std::sync::Mutex<HashMap<String, TrackingPeripheral>> {
    ACTIVE_PEERS.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

fn scm_service_uuid() -> Uuid {
    Uuid::from_u128(GATT_SERVICE_UUID)
}

fn scm_identity_uuid() -> Uuid {
    uuid_from_u16(0xDF02)
}

fn scm_notify_uuid() -> Uuid {
    uuid_from_u16(0xDF03)
}

/// Decode Drift-framed or raw envelope bytes; decrypt/verify via [`IronCore::receive_message`].
pub fn decode_ble_payload_for_ui(core: &IronCore, data: &[u8]) -> Option<MessageReceivedParams> {
    let payload: Vec<u8> = match DriftFrame::from_bytes(data) {
        Ok(f) => {
            if f.frame_type != FrameType::Data {
                return None;
            }
            f.payload
        }
        Err(_) => data.to_vec(),
    };
    let msg = core.receive_message(payload).ok()?;
    let from = msg.sender_id.clone();
    let content = msg.text_content().unwrap_or_default();
    let timestamp = msg.timestamp;
    let message_id = msg.id;
    Some(MessageReceivedParams {
        from,
        content,
        timestamp,
        message_id,
    })
}

fn push_message_to_ui(
    ui_tx: &tokio::sync::broadcast::Sender<UiOutbound>,
    p: MessageReceivedParams,
) {
    let legacy = UiEvent::MessageReceived {
        from: p.from.clone(),
        content: p.content.clone(),
        timestamp: p.timestamp,
        message_id: p.message_id.clone(),
    };
    let _ = ui_tx.send(UiOutbound::Legacy(legacy));
    let n = notif_message_received(p);
    if let Ok(v) = serde_json::to_value(&n) {
        let _ = ui_tx.send(UiOutbound::JsonRpc(v));
    }
}

struct PeerRegistryGuard {
    peer_id: Option<String>,
    mac_addr: String,
}

impl Drop for PeerRegistryGuard {
    fn drop(&mut self) {
        if let Some(ref peer_id) = self.peer_id {
            if let Ok(mut map) = get_active_peers().lock() {
                let should_remove = match map.get(peer_id) {
                    Some(active_p) => active_p.address_string() == self.mac_addr,
                    None => false,
                };
                if should_remove {
                    map.remove(peer_id);
                    tracing::info!("BLE removed peer ID {} from active registry", peer_id);
                } else {
                    tracing::debug!(
                        "BLE peer ID {} retained in registry (MAC rotated from {})",
                        peer_id,
                        self.mac_addr
                    );
                }
            }
        }
    }
}

async fn connect_with_backoff(peripheral: &Peripheral, addr: &str) -> Result<(), String> {
    for attempt in 0..MAX_CONNECT_ATTEMPTS {
        let is_connected = match peripheral.is_connected().await {
            Ok(connected) => connected,
            Err(e) => {
                tracing::debug!("BLE connection state unavailable for {}: {}", addr, e);
                false
            }
        };
        if is_connected {
            return Ok(());
        }

        match peripheral.connect().await {
            Ok(()) => return Ok(()),
            Err(e) if attempt + 1 < MAX_CONNECT_ATTEMPTS => {
                let delay_ms = CONNECT_BACKOFF_MS << attempt;
                tracing::debug!(
                    "BLE connect attempt {}/{} failed for {}: {}; retrying in {}ms",
                    attempt + 1,
                    MAX_CONNECT_ATTEMPTS,
                    addr,
                    e,
                    delay_ms
                );
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }
            Err(e) => {
                return Err(format!(
                    "BLE connect failed for {} after {} attempts: {}",
                    addr, MAX_CONNECT_ATTEMPTS, e
                ));
            }
        }
    }

    Err(format!(
        "BLE connect failed for {} after {} attempts",
        addr, MAX_CONNECT_ATTEMPTS
    ))
}

#[derive(Default)]
struct IngressReassembly {
    total_fragments: Option<u16>,
    fragments: HashMap<u16, Vec<u8>>,
}

impl IngressReassembly {
    /// Accept one DF03 GATT fragment and return a complete message when all
    /// fragments for this connection have arrived.
    fn push(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        let header = GattFragmentHeader::from_bytes(data).ok()?;
        if header.total_fragments > MAX_INBOUND_FRAGMENTS {
            self.total_fragments = None;
            self.fragments.clear();
            tracing::warn!(
                "BLE DF03 fragment count {} exceeds the inbound limit {}",
                header.total_fragments,
                MAX_INBOUND_FRAGMENTS
            );
            return None;
        }
        if self.total_fragments != Some(header.total_fragments) || header.fragment_index == 0 {
            self.total_fragments = Some(header.total_fragments);
            self.fragments.clear();
        }
        self.fragments.insert(header.fragment_index, data.to_vec());

        let total = header.total_fragments as usize;
        if self.fragments.len() != total {
            return None;
        }

        let ordered = (0..header.total_fragments)
            .map(|index| self.fragments.get(&index).cloned())
            .collect::<Option<Vec<_>>>()?;
        let message = GattReassembler::reassemble(&ordered).ok()?;
        self.total_fragments = None;
        self.fragments.clear();
        Some(message)
    }
}

async fn subscribe_ingress_for_peripheral(
    peripheral: Peripheral,
    core: Arc<IronCore>,
    ui_tx: tokio::sync::broadcast::Sender<UiOutbound>,
) {
    let addr = peripheral.address().to_string();
    if let Err(e) = connect_with_backoff(&peripheral, &addr).await {
        tracing::debug!("BLE connect skipped/failed for {}: {}", addr, e);
        return;
    }
    if let Err(e) = peripheral.discover_services().await {
        tracing::warn!("BLE discover_services failed for {}: {}", addr, e);
        let _ = peripheral.disconnect().await;
        return;
    }
    if !peripheral
        .services()
        .iter()
        .any(|service| service.uuid == scm_service_uuid())
    {
        tracing::debug!("BLE SCM service {} missing on {}", scm_service_uuid(), addr);
        let _ = peripheral.disconnect().await;
        return;
    }

    // Try to read identity data to register peer_id
    // PeerRegistryGuard will remove peer_id from ACTIVE_PEERS automatically when this background
    // streaming task terminates (via early return, disconnect, or stream completion).
    let mut guard = PeerRegistryGuard {
        peer_id: None,
        mac_addr: addr.clone(),
    };
    let identity_uuid = scm_identity_uuid();
    let id_char = peripheral
        .characteristics()
        .iter()
        .find(|c| {
            c.service_uuid == scm_service_uuid()
                && c.uuid == identity_uuid
                && c.properties.contains(CharPropFlags::READ)
        })
        .cloned();

    if let Some(id_c) = id_char {
        match peripheral.read(&id_c).await {
            Ok(bytes) => {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    let peer_id = val
                        .get("peer_id")
                        .or_else(|| val.get("libp2p_peer_id"))
                        .and_then(|v| v.as_str());
                    if let Some(peer_id) = peer_id {
                        if let Ok(parsed_peer_id) = peer_id.parse::<libp2p::PeerId>() {
                            let peer_id_str = parsed_peer_id.to_string();
                            let mut map = get_active_peers()
                                .lock()
                                .expect("ACTIVE_PEERS lock poisoned");
                            if let Some(existing) = map.get(&peer_id_str) {
                                tracing::info!("BLE correlated rotated MAC {} to existing logical peer ID {} (was {})", addr, peer_id_str, existing.address_string());
                            } else {
                                tracing::info!(
                                    "BLE mapped peripheral {} to peer ID {}",
                                    addr,
                                    peer_id_str
                                );
                            }
                            map.insert(
                                peer_id_str.clone(),
                                TrackingPeripheral::Real(peripheral.clone()),
                            );
                            guard.peer_id = Some(peer_id_str);
                        } else {
                            tracing::warn!(
                                "BLE received invalid peer ID '{}' from {}",
                                peer_id,
                                addr
                            );
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("BLE failed to read identity char for {}: {}", addr, e);
            }
        }
    } else {
        tracing::debug!("BLE no identity char {} on {}", identity_uuid, addr);
    }

    let notify_uuid = scm_notify_uuid();
    let ch = peripheral
        .characteristics()
        .iter()
        .find(|c| {
            c.service_uuid == scm_service_uuid()
                && c.uuid == notify_uuid
                && c.properties.contains(CharPropFlags::NOTIFY)
        })
        .cloned();
    let Some(ch) = ch else {
        tracing::debug!("BLE no notify char {:} on {}", notify_uuid, addr);
        let _ = peripheral.disconnect().await;
        return;
    };
    if let Err(e) = peripheral.subscribe(&ch).await {
        tracing::warn!("BLE subscribe failed for {}: {}", addr, e);
        let _ = peripheral.disconnect().await;
        return;
    }
    tracing::info!(
        "BLE GATT notify subscribed on {} (SCM ingress for thin client WebSocket)",
        addr
    );

    let mut stream = match peripheral.notifications().await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("BLE notifications stream failed: {}", e);
            return;
        }
    };

    let mut reassembly = IngressReassembly::default();
    while let Some(note) = stream.next().await {
        if note.uuid != notify_uuid {
            continue;
        }
        let payload = if GattFragmentHeader::from_bytes(&note.value).is_ok() {
            match reassembly.push(&note.value) {
                Some(payload) => payload,
                None => continue,
            }
        } else {
            // Keep compatibility with peers that send one complete envelope
            // in a single notification instead of using GATT fragmentation.
            note.value
        };
        if let Some(params) = decode_ble_payload_for_ui(core.as_ref(), &payload) {
            push_message_to_ui(&ui_tx, params);
        }
    }
}

/// Send a SCMessenger message envelope over BLE to the registered peripheral
pub async fn send_ble_message(recipient_peer_id: &str, data: &[u8]) -> Result<(), String> {
    let peripheral = {
        let guard = get_active_peers()
            .lock()
            .expect("ACTIVE_PEERS lock poisoned");
        guard.get(recipient_peer_id).cloned()
    };

    let Some(TrackingPeripheral::Real(peripheral)) = peripheral else {
        #[cfg(target_os = "windows")]
        {
            return Err(format!(
                "Windows BLE recipient-specific send unsupported for peer {}: no exact logical recipient/session is registered",
                recipient_peer_id
            ));
        }
        #[cfg(not(target_os = "windows"))]
        return Err("Peer not connected over BLE".to_string());
    };

    connect_with_backoff(&peripheral, recipient_peer_id).await?;
    peripheral
        .discover_services()
        .await
        .map_err(|e| format!("GATT service discovery failed: {}", e))?;
    if !peripheral
        .services()
        .iter()
        .any(|service| service.uuid == scm_service_uuid())
    {
        return Err("SCM GATT service not found on peer".to_string());
    }

    let msg_char_uuid = scm_notify_uuid(); // 0xDF03
    let ch = peripheral
        .characteristics()
        .iter()
        .find(|c| {
            c.service_uuid == scm_service_uuid()
                && c.uuid == msg_char_uuid
                && (c.properties.contains(CharPropFlags::WRITE)
                    || c.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE))
        })
        .cloned();

    let Some(ch) = ch else {
        return Err("Message characteristic not found on peer".to_string());
    };

    // Fragment the message using GattFragmenter from scmessenger_core
    let fragments = scmessenger_core::transport::ble::GattFragmenter::fragment(data)
        .map_err(|e| format!("Fragmentation error: {:?}", e))?;

    tracing::info!(
        "BLE: sending {} fragments to {}",
        fragments.len(),
        recipient_peer_id
    );
    for fragment in fragments {
        let write_type = if ch
            .properties
            .contains(CharPropFlags::WRITE_WITHOUT_RESPONSE)
        {
            btleplug::api::WriteType::WithoutResponse
        } else {
            btleplug::api::WriteType::WithResponse
        };

        peripheral
            .write(&ch, &fragment, write_type)
            .await
            .map_err(|e| format!("GATT write failed: {}", e))?;
    }

    Ok(())
}

/// Run until process exit: scan for SCM service, connect + notify per peripheral.
pub async fn run_ble_central_ingress(
    core: Arc<IronCore>,
    ui_tx: tokio::sync::broadcast::Sender<UiOutbound>,
) {
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let _ = (core, ui_tx);
        tracing::debug!("BLE central ingress: unsupported OS");
        return;
    }

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        tracing::info!(
            "BLE: CLI GATT central for service {:x} (peripheral advertising via btleplug not enabled).",
            GATT_SERVICE_UUID
        );

        let manager = match Manager::new().await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("BLE Manager::new failed: {}", e);
                return;
            }
        };
        let adapters = match manager.adapters().await {
            Ok(a) => a,
            Err(e) => {
                tracing::warn!("BLE adapters() failed: {}", e);
                return;
            }
        };
        let Some(adapter) = adapters.first() else {
            tracing::warn!("BLE: no adapters");
            return;
        };

        let svc = scm_service_uuid();
        let mut events = match adapter.events().await {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("BLE events() failed: {}", e);
                return;
            }
        };
        // Windows/WinRT: the adapter object is often not ready to scan for a
        // brief window right after Manager::new()/adapters() returns (the
        // underlying BluetoothLEAdvertisementWatcher hasn't finished
        // initializing). start_scan() then fails with HRESULT 0x800710DF
        // ("device is not ready for use"). This is transient, not fatal —
        // retry a few times with backoff before giving up.
        const SCAN_START_RETRIES: u32 = 5;
        let mut scan_started = false;
        for attempt in 0..SCAN_START_RETRIES {
            match adapter
                .start_scan(ScanFilter {
                    services: vec![svc],
                })
                .await
            {
                Ok(()) => {
                    scan_started = true;
                    break;
                }
                Err(e) => {
                    if attempt + 1 < SCAN_START_RETRIES {
                        let delay_ms = 300u64 << attempt;
                        tracing::debug!(
                            "BLE start_scan attempt {}/{} failed ({}), retrying in {}ms",
                            attempt + 1,
                            SCAN_START_RETRIES,
                            e,
                            delay_ms
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                    } else {
                        tracing::warn!(
                            "BLE start_scan failed after {} attempts: {}",
                            SCAN_START_RETRIES,
                            e
                        );
                    }
                }
            }
        }
        if !scan_started {
            return;
        }
        tracing::info!("BLE scan active, filtered to SCM service {}", svc);

        // Track peripherals with backoff state to prevent spin-looping on unreachable devices
        struct PeripheralState {
            active: bool,
            failures: u32,
            cooldown_until: Option<std::time::Instant>,
        }
        let tracked: Arc<Mutex<std::collections::HashMap<String, PeripheralState>>> =
            Arc::new(Mutex::new(std::collections::HashMap::new()));

        while let Some(evt) = events.next().await {
            tracing::debug!("BLE central event received: {:?}", evt);

            // Extract the peripheral ID from ANY variant that contains it
            let id = match &evt {
                CentralEvent::DeviceDiscovered(id) => id.clone(),
                CentralEvent::DeviceUpdated(id) => id.clone(),
                CentralEvent::ManufacturerDataAdvertisement { id, .. } => id.clone(),
                CentralEvent::ServiceDataAdvertisement { id, .. } => id.clone(),
                CentralEvent::ServicesAdvertisement { id, .. } => id.clone(),
                _ => continue,
            };

            // Throttle processing per device with exponential backoff
            let id_key = id.to_string();
            {
                let mut guard = tracked.lock().await;
                // Bound memory against unbounded growth under BLE MAC rotation:
                // sweep idle-safe (inactive, no failures) or expired-cooldown
                // entries before growing past a cap.
                if guard.len() > 2048 {
                    let now = std::time::Instant::now();
                    guard.retain(|_, s| {
                        (s.active || s.failures != 0) && s.cooldown_until.is_none_or(|t| t > now)
                    });
                }
                let state = guard.entry(id_key.clone()).or_insert(PeripheralState {
                    active: false,
                    failures: 0,
                    cooldown_until: None,
                });

                if state.active {
                    continue; // Busy connecting or actively tracked
                }

                // Respect backoff cooldown for previously failed peripherals
                if let Some(cooldown) = state.cooldown_until {
                    if std::time::Instant::now() < cooldown {
                        continue;
                    }
                }

                state.active = true;
            }

            let peripheral = match adapter.peripheral(&id).await {
                Ok(p) => p,
                Err(_) => {
                    let mut guard = tracked.lock().await;
                    if let Some(state) = guard.get_mut(&id_key) {
                        state.active = false;
                    }
                    continue;
                }
            };

            // In a background task, query properties so we don't block the main event stream
            let core_c = Arc::clone(&core);
            let ui_c = ui_tx.clone();
            let track = Arc::clone(&tracked);
            let key = id_key.clone();
            let target_svc = svc;

            tokio::spawn(async move {
                let mut is_match = false;

                // 1. Quick check if events gave us immediate evidence
                match &evt {
                    CentralEvent::ServicesAdvertisement { services, .. }
                        if services.contains(&target_svc) =>
                    {
                        is_match = true;
                    }
                    CentralEvent::ServiceDataAdvertisement { service_data, .. }
                        if service_data.contains_key(&target_svc) =>
                    {
                        is_match = true;
                    }
                    _ => {}
                }

                // 2. Explicit property poll if event variant was generic
                if !is_match {
                    if let Ok(Some(props)) = peripheral.properties().await {
                        if props.services.contains(&target_svc)
                            || props.service_data.contains_key(&target_svc)
                        {
                            is_match = true;
                        }
                    }
                }

                let mut success = true;
                if is_match {
                    tracing::info!("BLE found matching peripheral: {}", key);
                    let start_time = std::time::Instant::now();
                    subscribe_ingress_for_peripheral(peripheral, core_c, ui_c).await;
                    // subscribe_ingress_for_peripheral returns only when the stream
                    // ends or an error occurs. A session that stayed connected past a
                    // threshold is a normal disconnect (peer out of range), not a
                    // backoff-worthy failure; only rapid failures (< threshold) back off.
                    let session_duration = start_time.elapsed();
                    if session_duration < std::time::Duration::from_secs(10) {
                        success = false;
                    }
                }

                // Update backoff state
                let mut guard = track.lock().await;
                if let Some(state) = guard.get_mut(&key) {
                    state.active = false;
                    if success || !is_match {
                        // Non-matching peripherals or successful connections reset backoff
                        state.failures = 0;
                        state.cooldown_until = None;
                    } else {
                        state.failures = state.failures.saturating_add(1).min(MAX_TRACKED_FAILURES);
                        // Exponential backoff: 2s, 4s, 8s, 16s, 32s, 60s cap
                        let backoff_secs = (1u64 << state.failures.min(6)).min(60);
                        state.cooldown_until = Some(
                            std::time::Instant::now()
                                + std::time::Duration::from_secs(backoff_secs),
                        );
                        tracing::debug!(
                            "BLE backoff for {} set to {}s (failure #{})",
                            key,
                            backoff_secs,
                            state.failures
                        );
                    }
                }
            });
        }
    }
}

/// Helper to decode BLE message and push to UI. Used by both central and peripheral paths.
pub fn handle_incoming_ble_payload(
    core: &IronCore,
    ui_tx: &tokio::sync::broadcast::Sender<UiOutbound>,
    data: &[u8],
) {
    if let Some(params) = decode_ble_payload_for_ui(core, data) {
        push_message_to_ui(ui_tx, params);
    }
}

/// Run peripheral advertising.
pub async fn run_ble_peripheral_advertising(
    core: Arc<IronCore>,
    ui_tx: tokio::sync::broadcast::Sender<UiOutbound>,
) {
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = crate::ble_windows::run_windows_ble_peripheral(core, ui_tx).await {
            tracing::error!("BLE: Windows GATT server / advertising error: {:?}", e);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let _ = (core, ui_tx);
            tracing::warn!(
                "BLE capability unsupported: btleplug provides no desktop peripheral GATT server for service {:x}; advertising is not active. The CLI remains a GATT central for mobile/native peers.",
                GATT_SERVICE_UUID
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scmessenger_core::IronCore as CoreIron;

    #[test]
    fn decode_rejects_short_buffer() {
        let core = CoreIron::new();
        let _ = core.start();
        let junk = [0u8; 4];
        assert!(decode_ble_payload_for_ui(&core, &junk).is_none());
    }

    #[test]
    fn test_mac_rotation_continuity() {
        // Clear global state to ensure clean test
        get_active_peers().lock().unwrap().clear();

        let peer_id = "12D3KooWMqHj8dM6zY7vVv2K4n3nF2T1oT1w2Z3a4b5c6d7e8f9".to_string();
        let old_mac = "AA:BB:CC:DD:EE:FF".to_string();
        let new_mac = "11:22:33:44:55:66".to_string();

        // 1. Initial connection
        let mut guard1 = PeerRegistryGuard {
            peer_id: None,
            mac_addr: old_mac.clone(),
        };
        {
            let mut map = get_active_peers().lock().unwrap();
            map.insert(peer_id.clone(), TrackingPeripheral::Mock(old_mac.clone()));
            guard1.peer_id = Some(peer_id.clone());
        }

        assert_eq!(
            get_active_peers()
                .lock()
                .unwrap()
                .get(&peer_id)
                .unwrap()
                .address_string(),
            old_mac
        );

        // 2. MAC rotates mid-session (new connection established before old one drops)
        let mut guard2 = PeerRegistryGuard {
            peer_id: None,
            mac_addr: new_mac.clone(),
        };
        {
            let mut map = get_active_peers().lock().unwrap();
            // Correlate
            if let Some(existing) = map.get(&peer_id) {
                assert_eq!(existing.address_string(), old_mac);
            } else {
                panic!("Expected existing peer");
            }
            map.insert(peer_id.clone(), TrackingPeripheral::Mock(new_mac.clone()));
            guard2.peer_id = Some(peer_id.clone());
        }

        // 3. Old connection drops
        drop(guard1);

        // 4. Assert session continuity (peer is still tracked with new MAC)
        let map = get_active_peers().lock().unwrap();
        assert!(
            map.contains_key(&peer_id),
            "Peer should still be in registry"
        );
        assert_eq!(map.get(&peer_id).unwrap().address_string(), new_mac);
    }
}
