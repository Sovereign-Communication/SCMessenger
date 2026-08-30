//! BLE GATT central path: discover SCMessenger peripherals, subscribe to notify, forward
//! decrypted payloads to the local Web UI as JSON-RPC `message_received`.
//!
//! **Advertising:** btleplug is central-oriented on desktop OSes; the CLI does not expose a
//! full peripheral GATT server here. Mobile/native peers remain peripherals; this node scans,
//! connects, and ingests notify payloads.

use btleplug::api::{
    Central, CentralEvent, CharPropFlags, Manager as _, Peripheral as PeripheralApi, ScanFilter,
};
use btleplug::platform::{Manager, Peripheral};
use futures_util::{FutureExt, StreamExt};
use scmessenger_core::drift::frame::{DriftFrame, FrameType};
use scmessenger_core::drift::DriftEnvelope;
use scmessenger_core::transport::ble::{GattFragmentHeader, GattReassembler};
use scmessenger_core::wasm_support::rpc::{notif_message_received, MessageReceivedParams};
use scmessenger_core::IronCore;
use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
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

use crate::ble_ids::{GATT_SERVICE_UUID, IDENTITY_CHAR_UUID, MESSAGE_CHAR_UUID};
use crate::server::{UiEvent, UiOutbound};

static ACTIVE_PEERS: OnceLock<std::sync::Mutex<HashMap<String, TrackingPeripheral>>> =
    OnceLock::new();

fn get_active_peers() -> &'static std::sync::Mutex<HashMap<String, TrackingPeripheral>> {
    ACTIVE_PEERS.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

#[cfg(target_os = "macos")]
fn note_macos_btleplug_gatt_unavailable() {
    static LOGGED: OnceLock<()> = OnceLock::new();
    if LOGGED.set(()).is_ok() {
        tracing::warn!(
            "BLE: macOS CoreBluetooth GATT unavailable with btleplug 0.11.8; continuing scan-only with backoff"
        );
    }
}

fn scm_service_uuid() -> Uuid {
    Uuid::from_u128(GATT_SERVICE_UUID)
}

fn scm_identity_uuid() -> Uuid {
    Uuid::from_u128(IDENTITY_CHAR_UUID)
}

fn scm_notify_uuid() -> Uuid {
    Uuid::from_u128(MESSAGE_CHAR_UUID)
}

fn peer_suffix(peer_id: &str) -> String {
    let suffix: String = peer_id.chars().rev().take(8).collect();
    suffix.chars().rev().collect()
}

fn diagnostic_message_id(data: &[u8]) -> String {
    let envelope = DriftFrame::from_bytes(data)
        .ok()
        .filter(|frame| frame.frame_type == FrameType::Data)
        .map(|frame| frame.payload)
        .unwrap_or_else(|| data.to_vec());

    DriftEnvelope::from_bytes(&envelope)
        .map(|parsed| Uuid::from_bytes(parsed.message_id).to_string())
        .unwrap_or_else(|_| "unavailable".to_string())
}

fn diagnostic_message_hash(data: &[u8]) -> String {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn fragment_metadata(data: &[u8]) -> (Option<usize>, Option<usize>) {
    GattFragmentHeader::from_bytes(data)
        .ok()
        .map(|header| {
            (
                Some(header.fragment_index as usize),
                Some(header.total_fragments as usize),
            )
        })
        .unwrap_or((None, None))
}

/// One peer's inbound GATT fragment buffer. A single BLE message may be split into
/// `total_fragments` notify notifications (each: 4-byte `GattFragmentHeader` + payload
/// chunk); collect them here and emit the reassembled envelope once all are present.
struct PeerReassembly {
    total_fragments: usize,
    fragments: HashMap<u16, Vec<u8>>,
    last_seen: std::time::Instant,
}

/// Drop a buffered stream that never completes this long after its last fragment — a
/// lost mid-message fragment must not pin memory (or a stale buffer) indefinitely.
const REASSEMBLY_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(30);

/// Upper bound on distinct retained fragments per peer (bounds a hostile/erroneous stream).
const REASSEMBLY_MAX_FRAGMENTS: usize = 4096;

impl PeerReassembly {
    fn new(total_fragments: usize) -> Self {
        Self {
            total_fragments,
            fragments: HashMap::new(),
            last_seen: std::time::Instant::now(),
        }
    }

    fn is_expired(&self) -> bool {
        self.last_seen.elapsed() > REASSEMBLY_MAX_AGE
    }

    /// Store one fragment; return the fully reassembled envelope once every fragment in
    /// `0..total` is present (in index order), or `None` while still awaiting fragments.
    fn insert(&mut self, index: u16, payload: Vec<u8>) -> Option<Vec<u8>> {
        self.last_seen = std::time::Instant::now();
        if self.fragments.len() >= REASSEMBLY_MAX_FRAGMENTS {
            return None; // hostile/erroneous stream: give up cleanly
        }
        self.fragments.insert(index, payload);
        if self.fragments.len() < self.total_fragments {
            return None; // still awaiting more fragments
        }

        let mut ordered = Vec::with_capacity(self.total_fragments);
        for i in 0..self.total_fragments as u16 {
            // Reconstruct each stored chunk back into a headed fragment so the existing
            // core reassembler (same convention as the write path and ble_windows.rs)
            // verifies ordering and concatenates payloads.
            let chunk = match self.fragments.remove(&i) {
                Some(c) => c,
                None => {
                    self.fragments.clear();
                    return None;
                }
            };
            let mut full = GattFragmentHeader::new(self.total_fragments as u16, i)
                .ok()?
                .to_bytes()
                .to_vec();
            full.extend_from_slice(&chunk);
            ordered.push(full);
        }
        GattReassembler::reassemble(&ordered).ok()
    }
}

/// Route one raw GATT notification through reassembly, returning `Some(bytes)` when a
/// complete message is ready to decode — never partial bytes. Senders that do not
/// fragment (no parseable header) pass straight through unchanged.
fn ingest_ble_notification(buffer: &mut Option<PeerReassembly>, value: &[u8]) -> Option<Vec<u8>> {
    let header = match GattFragmentHeader::from_bytes(value) {
        Ok(header) => header,
        Err(_) => return Some(value.to_vec()), // unfragmented / legacy message
    };
    let payload = if value.len() > 4 {
        value[4..].to_vec()
    } else {
        Vec::new()
    };

    let total = header.total_fragments as usize;
    if total == 1 {
        // Single-fragment message: header + full envelope; supersedes a partial stream.
        *buffer = None;
        return Some(payload);
    }

    // Multi-fragment stream: reuse an active buffer, or restart when the declared
    // partition changed or the prior stream went stale.
    let needs_new = match buffer.as_ref() {
        Some(b) => b.is_expired() || b.total_fragments != total,
        None => true,
    };
    if needs_new {
        *buffer = Some(PeerReassembly::new(total));
    }
    let b = buffer.as_mut()?;
    b.insert(header.fragment_index, payload)
}

fn log_ble_payload_diagnostic(
    route: &str,
    peer: &str,
    data: &[u8],
    fragment_index: Option<usize>,
    fragment_count: Option<usize>,
    terminal_result: &str,
) {
    tracing::info!(
        message_id = %diagnostic_message_id(data),
        message_hash = %diagnostic_message_hash(data),
        fragment_index = ?fragment_index,
        fragment_count = ?fragment_count,
        peer_suffix = %peer_suffix(peer),
        route = route,
        terminal_result = terminal_result,
        "BLE payload diagnostic"
    );
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

async fn subscribe_ingress_for_peripheral(
    peripheral: Peripheral,
    core: Arc<IronCore>,
    ui_tx: tokio::sync::broadcast::Sender<UiOutbound>,
) {
    let addr = peripheral.address().to_string();
    if let Err(e) = peripheral.connect().await {
        tracing::debug!("BLE connect skipped/failed for {}: {}", addr, e);
        return;
    }
    if let Err(e) = peripheral.discover_services().await {
        tracing::warn!("BLE discover_services failed for {}: {}", addr, e);
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
        .find(|c| c.uuid == identity_uuid && c.properties.contains(CharPropFlags::READ))
        .cloned();

    if let Some(id_c) = id_char {
        match peripheral.read(&id_c).await {
            Ok(bytes) => {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    if let Some(peer_id) = val.get("peer_id").and_then(|v| v.as_str()) {
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
        .find(|c| c.uuid == notify_uuid && c.properties.contains(CharPropFlags::NOTIFY))
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

    // Buffer an inbound fragmented GATT message per peripheral until all its fragments
    // arrive, so a truncated envelope is reassembled rather than decoded/verified against
    // partial bytes (the P0 ble_gatt_ingress truncation bug).
    let mut reassembly: Option<PeerReassembly> = None;
    while let Some(note) = stream.next().await {
        let peer = guard.peer_id.as_deref().unwrap_or(&addr);

        // Fast path: a complete, unfragmented envelope decodes directly.
        if let Some(params) = decode_ble_payload_for_ui(core.as_ref(), &note.value) {
            reassembly = None; // a whole message supersedes any earlier partial stream
            let (fragment_index, fragment_count) = fragment_metadata(&note.value);
            log_ble_payload_diagnostic(
                "ble_gatt_ingress",
                peer,
                &note.value,
                fragment_index,
                fragment_count,
                "received_and_decrypted",
            );
            push_message_to_ui(&ui_tx, params);
            continue;
        }

        // Not a complete envelope: route through fragment reassembly so a truncated
        // inbound message is reconstructed before decode, never verified on partial bytes.
        if let Some(message) = ingest_ble_notification(&mut reassembly, &note.value) {
            let (fragment_index, fragment_count) = fragment_metadata(&note.value);
            if let Some(params) = decode_ble_payload_for_ui(core.as_ref(), &message) {
                reassembly = None;
                log_ble_payload_diagnostic(
                    "ble_gatt_ingress",
                    peer,
                    &note.value,
                    fragment_index,
                    fragment_count,
                    "received_and_decrypted",
                );
                push_message_to_ui(&ui_tx, params);
            } else {
                reassembly = None;
                log_ble_payload_diagnostic(
                    "ble_gatt_ingress",
                    peer,
                    &note.value,
                    fragment_index,
                    fragment_count,
                    "decode_or_decrypt_error",
                );
            }
        } else {
            let (fragment_index, fragment_count) = fragment_metadata(&note.value);
            log_ble_payload_diagnostic(
                "ble_gatt_ingress",
                peer,
                &note.value,
                fragment_index,
                fragment_count,
                "awaiting_fragments",
            );
        }
    }
}

/// Send a SCMessenger message envelope over BLE to the registered peripheral
pub async fn send_ble_message(recipient_peer_id: &str, data: &[u8]) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = (recipient_peer_id, data);
        note_macos_btleplug_gatt_unavailable();
        return Err("BLE unavailable on macOS".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        if crate::ble_windows::get_message_characteristic().is_some() {
            if let Err(e) = crate::ble_windows::send_windows_ble_notification(data).await {
                log_ble_payload_diagnostic(
                    "ble_windows_gatt",
                    recipient_peer_id,
                    data,
                    None,
                    None,
                    "windows_notification_error",
                );
                tracing::debug!("Windows BLE: outgoing notification failed: {:?}", e);
            } else {
                log_ble_payload_diagnostic(
                    "ble_windows_gatt",
                    recipient_peer_id,
                    data,
                    None,
                    None,
                    "windows_notification_complete",
                );
                return Ok(());
            }
        }
    }

    let peripheral = {
        let guard = get_active_peers()
            .lock()
            .expect("ACTIVE_PEERS lock poisoned");
        guard.get(recipient_peer_id).cloned()
    };

    let Some(TrackingPeripheral::Real(peripheral)) = peripheral else {
        log_ble_payload_diagnostic(
            "ble_gatt_central",
            recipient_peer_id,
            data,
            None,
            None,
            "peer_not_connected",
        );
        return Err("Peer not connected over BLE".to_string());
    };

    let msg_char_uuid = scm_notify_uuid(); // 0xDF03
    let ch = peripheral
        .characteristics()
        .iter()
        .find(|c| {
            c.uuid == msg_char_uuid
                && (c.properties.contains(CharPropFlags::WRITE)
                    || c.properties.contains(CharPropFlags::WRITE_WITHOUT_RESPONSE))
        })
        .cloned();

    let Some(ch) = ch else {
        log_ble_payload_diagnostic(
            "ble_gatt_central",
            recipient_peer_id,
            data,
            None,
            None,
            "message_characteristic_missing",
        );
        return Err("Message characteristic not found on peer".to_string());
    };

    // Fragment the message using GattFragmenter from scmessenger_core
    let fragments = match scmessenger_core::transport::ble::GattFragmenter::fragment(data) {
        Ok(fragments) => fragments,
        Err(e) => {
            log_ble_payload_diagnostic(
                "ble_gatt_central",
                recipient_peer_id,
                data,
                None,
                None,
                "fragmentation_error",
            );
            return Err(format!("Fragmentation error: {:?}", e));
        }
    };

    tracing::info!(
        message_id = %diagnostic_message_id(data),
        message_hash = %diagnostic_message_hash(data),
        fragment_index = ?None::<usize>,
        fragment_count = fragments.len(),
        peer_suffix = %peer_suffix(recipient_peer_id),
        route = "ble_gatt_central",
        terminal_result = "writing",
        "BLE outbound payload"
    );
    for (fragment_index, fragment) in fragments.iter().enumerate() {
        let write_type = if ch
            .properties
            .contains(CharPropFlags::WRITE_WITHOUT_RESPONSE)
        {
            btleplug::api::WriteType::WithoutResponse
        } else {
            btleplug::api::WriteType::WithResponse
        };

        if let Err(e) = peripheral.write(&ch, fragment, write_type).await {
            log_ble_payload_diagnostic(
                "ble_gatt_central",
                recipient_peer_id,
                data,
                Some(fragment_index),
                Some(fragments.len()),
                "gatt_write_error",
            );
            return Err(format!("GATT write failed: {}", e));
        }
        log_ble_payload_diagnostic(
            "ble_gatt_central",
            recipient_peer_id,
            data,
            Some(fragment_index),
            Some(fragments.len()),
            "gatt_write_ok",
        );
    }

    log_ble_payload_diagnostic(
        "ble_gatt_central",
        recipient_peer_id,
        data,
        None,
        Some(fragments.len()),
        "gatt_write_complete",
    );
    Ok(())
}

/// Run until process exit: scan for SCM service, connect + notify per peripheral.
pub async fn run_ble_central_ingress(
    core: Arc<IronCore>,
    ui_tx: tokio::sync::broadcast::Sender<UiOutbound>,
) {
    #[cfg(target_os = "macos")]
    let _ = (&core, &ui_tx);

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

        let manager = match AssertUnwindSafe(Manager::new()).catch_unwind().await {
            Ok(Ok(manager)) => manager,
            Ok(Err(error)) => {
                tracing::warn!(
                    route = "ble_gatt_central",
                    terminal_result = "manager_init_error",
                    error = %error,
                    "BLE central unavailable"
                );
                return;
            }
            Err(_) => {
                tracing::error!(
                    route = "ble_gatt_central",
                    terminal_result = "manager_init_panic_isolated",
                    "BLE central unavailable"
                );
                return;
            }
        };
        let adapters = match AssertUnwindSafe(manager.adapters()).catch_unwind().await {
            Ok(Ok(adapters)) => adapters,
            Ok(Err(error)) => {
                tracing::warn!(
                    route = "ble_gatt_central",
                    terminal_result = "adapter_enumeration_error",
                    error = %error,
                    "BLE central unavailable"
                );
                return;
            }
            Err(_) => {
                tracing::error!(
                    route = "ble_gatt_central",
                    terminal_result = "adapter_enumeration_panic_isolated",
                    "BLE central unavailable"
                );
                return;
            }
        };
        let Some(adapter) = adapters.first() else {
            tracing::warn!(
                route = "ble_gatt_central",
                terminal_result = "no_adapter",
                "BLE central unavailable"
            );
            return;
        };

        let svc = scm_service_uuid();
        // Windows/WinRT: the adapter object is often not ready to scan for a
        // brief window right after Manager::new()/adapters() returns (the
        // underlying BluetoothLEAdvertisementWatcher hasn't finished
        // initializing). start_scan() then fails with HRESULT 0x800710DF
        // ("device is not ready for use"). This is transient, not fatal —
        // retry a few times with backoff before giving up.
        const SCAN_START_RETRIES: u32 = 5;
        let mut scan_started = false;
        for attempt in 0..SCAN_START_RETRIES {
            match AssertUnwindSafe(adapter.start_scan(ScanFilter::default()))
                .catch_unwind()
                .await
            {
                Ok(Ok(())) => {
                    scan_started = true;
                    break;
                }
                Ok(Err(e)) => {
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
                            route = "ble_gatt_central",
                            terminal_result = "scan_start_error",
                            attempts = SCAN_START_RETRIES,
                            error = %e,
                            "BLE central unavailable"
                        );
                    }
                }
                Err(_) => {
                    tracing::error!(
                        route = "ble_gatt_central",
                        terminal_result = "scan_start_panic_isolated",
                        attempt = attempt + 1,
                        "BLE central unavailable"
                    );
                    return;
                }
            }
        }
        if !scan_started {
            return;
        }
        tracing::info!(
            "BLE scan active (wide open, manually filtering to SCM service {})",
            svc
        );

        let mut events = match AssertUnwindSafe(adapter.events()).catch_unwind().await {
            Ok(Ok(events)) => events,
            Ok(Err(error)) => {
                tracing::warn!(
                    route = "ble_gatt_central",
                    terminal_result = "event_stream_error",
                    error = %error,
                    "BLE central unavailable"
                );
                return;
            }
            Err(_) => {
                tracing::error!(
                    route = "ble_gatt_central",
                    terminal_result = "event_stream_panic_isolated",
                    "BLE central unavailable"
                );
                return;
            }
        };

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
            let id_key = format!("{:?}", id);
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

            let peripheral = match AssertUnwindSafe(adapter.peripheral(&id))
                .catch_unwind()
                .await
            {
                Ok(Ok(peripheral)) => peripheral,
                Ok(Err(_)) | Err(_) => {
                    let mut guard = tracked.lock().await;
                    if let Some(state) = guard.get_mut(&id_key) {
                        state.active = false;
                    }
                    continue;
                }
            };

            // In a background task, query properties so we don't block the main event stream.
            // On macOS, keep the advertisement/discovery half of central operation, but do not
            // enter btleplug's CoreBluetooth GATT connect path (0.11.8 can panic in its delegate
            // after repeated service-discovery callbacks instead of returning an error).
            #[cfg(not(target_os = "macos"))]
            let core_c = Arc::clone(&core);
            #[cfg(not(target_os = "macos"))]
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
                    #[cfg(target_os = "macos")]
                    {
                        note_macos_btleplug_gatt_unavailable();
                        success = false;
                    }

                    #[cfg(not(target_os = "macos"))]
                    {
                        tracing::info!("BLE found matching peripheral: {}", key);
                        let start_time = std::time::Instant::now();
                        let session_result = AssertUnwindSafe(subscribe_ingress_for_peripheral(
                            peripheral, core_c, ui_c,
                        ))
                        .catch_unwind()
                        .await;
                        if session_result.is_err() {
                            tracing::error!(
                                route = "ble_gatt_ingress",
                                peer_suffix = %peer_suffix(&key),
                                terminal_result = "session_panic_isolated",
                                "BLE peripheral session ended with an isolated panic"
                            );
                        }
                        // subscribe_ingress_for_peripheral returns only when the stream
                        // ends or an error occurs. A session that stayed connected past a
                        // threshold is a normal disconnect (peer out of range), not a
                        // backoff-worthy failure; only rapid failures (< threshold) back off.
                        let session_duration = start_time.elapsed();
                        if session_duration < std::time::Duration::from_secs(10) {
                            success = false;
                        }
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
                        state.failures += 1;
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
        let (fragment_index, fragment_count) = fragment_metadata(data);
        log_ble_payload_diagnostic(
            "ble_gatt_ingress",
            "unknown",
            data,
            fragment_index,
            fragment_count,
            "received_and_decrypted",
        );
        push_message_to_ui(ui_tx, params);
    } else {
        let (fragment_index, fragment_count) = fragment_metadata(data);
        log_ble_payload_diagnostic(
            "ble_gatt_ingress",
            "unknown",
            data,
            fragment_index,
            fragment_count,
            "decode_or_decrypt_error",
        );
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
            tracing::warn!(
                "BLE: peripheral advertising for service {:x} is not implemented on this platform \
                 (known limitation, not a bug — see tasks/T1.8/progress.md). This CLI still discovers \
                 and connects to BLE peripherals normally (mobile/native peers); it just cannot itself \
                 be discovered by another desktop CLI over BLE.",
                GATT_SERVICE_UUID
            );

            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            }
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
    fn ingest_reassembles_fragmented_envelope() {
        // Payload larger than a single 512-byte GATT characteristic write, so the
        // sender would split it; verify reassembly reconstructs the original bytes
        // even when fragments arrive out of order.
        let original: Vec<u8> = (0..2000u32).map(|i| (i % 251) as u8).collect();
        let fragments = scmessenger_core::transport::ble::GattFragmenter::fragment(&original)
            .expect("fragment");
        assert!(fragments.len() > 1);

        let mut buffer = None;
        let expected = fragments.len();
        for (order, i) in [1usize, 0, 2, 3, 4, 5, 6]
            .into_iter()
            .take(expected)
            .enumerate()
        {
            let out = ingest_ble_notification(&mut buffer, &fragments[i]);
            if order + 1 == expected {
                assert_eq!(out.expect("final fragment completes reassembly"), original);
            } else {
                assert!(out.is_none(), "should still be awaiting fragments");
            }
        }
    }

    #[test]
    fn ingest_single_fragment_strips_header() {
        let header = scmessenger_core::transport::ble::GattFragmentHeader::new(1, 0).unwrap();
        let mut value = header.to_bytes().to_vec();
        value.extend_from_slice(b"hello");
        let mut buffer = None;
        assert_eq!(
            ingest_ble_notification(&mut buffer, &value),
            Some(b"hello".to_vec())
        );
    }

    #[test]
    fn ingest_unfragmented_passthrough() {
        // First 4 bytes encode total=0 (invalid fragment header), so the whole value
        // must pass through unchanged rather than be swallowed into a fragment buffer.
        let mut value = vec![0u8; 4];
        value.extend_from_slice(b"envelope-tail");
        let mut buffer = None;
        assert_eq!(ingest_ble_notification(&mut buffer, &value), Some(value));
    }

    #[test]
    fn ingest_partition_change_restarts_buffer() {
        let mut buffer = None;
        // A two-fragment stream starts but never completes.
        let f1 = {
            let h = scmessenger_core::transport::ble::GattFragmentHeader::new(2, 0).unwrap();
            let mut v = h.to_bytes().to_vec();
            v.extend_from_slice(&[0xAA; 200]);
            v
        };
        assert!(ingest_ble_notification(&mut buffer, &f1).is_none());

        // A differently-partitioned (total=3) stream begins -> buffer must restart.
        let g0 = {
            let h = scmessenger_core::transport::ble::GattFragmentHeader::new(3, 0).unwrap();
            let mut v = h.to_bytes().to_vec();
            v.extend_from_slice(&[0xBB; 200]);
            v
        };
        assert!(ingest_ble_notification(&mut buffer, &g0).is_none());
        let g1 = {
            let h = scmessenger_core::transport::ble::GattFragmentHeader::new(3, 1).unwrap();
            let mut v = h.to_bytes().to_vec();
            v.extend_from_slice(&[0xBB; 200]);
            v
        };
        let g2 = {
            let h = scmessenger_core::transport::ble::GattFragmentHeader::new(3, 2).unwrap();
            let mut v = h.to_bytes().to_vec();
            v.extend_from_slice(&[0xBB; 200]);
            v
        };
        assert!(ingest_ble_notification(&mut buffer, &g1).is_none());
        let assembled = ingest_ble_notification(&mut buffer, &g2).expect("complete total=3");
        assert_eq!(assembled, vec![0xBB; 600]);
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

    #[cfg(target_os = "macos")]
    #[tokio::test]
    async fn macos_ble_send_reports_unavailable() {
        assert_eq!(
            send_ble_message("ignored", &[]).await,
            Err("BLE unavailable on macOS".to_string())
        );
    }
}
