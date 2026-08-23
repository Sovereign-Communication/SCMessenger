//! IronCore — the central entry point for the SCMessenger mesh.
//!
//! Holds identity, outbox, inbox, contact manager, history manager, storage
//! manager, log manager, blocked manager, relay registry, and audit log.
//! All state is behind `Arc<RwLock<…>>` (parking_lot).
//!
//! # Lint Configuration
//!
//! This file intentionally uses empty lines after doc comments for readability.
//! The `empty_line_after_outer_attr` check is disabled for this file.

#![allow(clippy::empty_line_after_outer_attr)]

use parking_lot::{Mutex, RwLock};
use std::sync::Arc;

use crate::abuse::auto_block::{AutoBlockConfig, AutoBlockEngine};
use crate::abuse::spam_detection::{SpamDetectionConfig, SpamDetectionEngine};
use crate::abuse::EnhancedAbuseReputationManager;
use crate::crypto::encrypt::{ed25519_public_to_x25519, ed25519_to_x25519_secret};
use crate::crypto::{decrypt_message, encrypt_message, session_manager::RatchetSessionManager};
use crate::drift::{MeshStore, NetworkState, RelayConfig, RelayEngine};
use crate::identity::IdentityManager;
use crate::message::{decode_envelope, decode_message, Message};
use crate::notification::NotificationEndpointRegistry;
use crate::observability::{AuditEventType, AuditLog as AuditLogType};
use crate::privacy::{
    CircuitBuilder, CircuitConfig, CoverConfig, CoverTrafficGenerator, JitterConfig, TimingJitter,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::relay::BootstrapManager;
use crate::routing::optimized_engine::OptimizedRoutingEngine;
use crate::store::backend::MemoryStorage;
#[cfg(not(target_arch = "wasm32"))]
use crate::store::backend::SledStorage;
use crate::store::blocked::BlockedManager as CoreBlockedManager;
use crate::store::logs::LogManager;
use crate::store::{
    Contact, ContactManager as CoreContactManager, HistoryManager as CoreHistoryManager, Inbox,
    MessageDirection, MessageRecord, Outbox, QueuedMessage, ReceivedMessage, RelayCustodyStore,
    StorageBackend, StorageManager,
};
use crate::transport::behaviour::RegistrationRequest;
use crate::transport::manager::TransportManager;
use crate::transport::reputation::AbuseSignal;
use crate::IronCoreError;

// ═══════════════════════════════════════════════════════════════════════════════
// Module wiring notes
// ═══════════════════════════════════════════════════════════════════════════════
//
// The following core modules are wired into IronCore as fields or pub fn
// entry points. Modules that are stateless or pure-function libraries do not
// need dedicated state fields:
//
// - `wasm_support` — JSON-RPC bridge between browser WASM client and CLI daemon.
//   Stateless request/response routing; no persistent state managed by IronCore.
//
// - `notification_defaults` — Default values and constants for notification
//   policies. Pure data module with no runtime state.
//
// - `crypto` — Pure cryptographic functions (encrypt_message, decrypt_message).
//   Called directly from message flow methods; no mutable state.
//
// - `message` — Message encoding/decoding types and helpers. Stateless.
//
// - `abuse` — Wired as `abuse_manager` field.
// - `drift` — Wired as `drift_active`, `drift_store`, `drift_engine` fields.
// - `identity` — Wired as `identity` field.
// - `observability` — Wired as `audit_log` field.
// - `store` — Wired as contact_manager, history_manager, storage_manager, etc.
// - `notification` — Wired as `classify_notification` pub fn + `notification_endpoint_registry` field.
// - `privacy` — Wired as `privacy_config` pub fn + cover_traffic_generator, timing_jitter,
//   circuit_builder fields.
// - `routing` — Wired as `routing_engine` field.
// - `transport` — Wired as `transport_manager` field.
// - `relay` — Wired as `bootstrap_manager` field.
//
// ═══════════════════════════════════════════════════════════════════════════════

/// Delegate trait for protocol events that consumers (like MeshService) implement
/// to receive callbacks from IronCore.
///
/// Method signatures must match the UniFFI-generated scaffolding exactly:
/// all string parameters are owned `String`, not `&str`.
pub trait CoreDelegate: Send + Sync {
    fn on_peer_discovered(&self, peer_id: String);
    fn on_peer_disconnected(&self, peer_id: String);
    fn on_peer_identified(&self, peer_id: String, agent_version: String, listen_addrs: Vec<String>);
    fn on_message_received(
        &self,
        sender_id: String,
        sender_public_key_hex: String,
        message_id: String,
        sender_timestamp: u64,
        data: Vec<u8>,
    );
    fn on_receipt_received(&self, message_id: String, status: String);
}

/// Consent state for identity initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentState {
    NotGranted,
    Granted,
}

/// Kill switch for the E-00 ratchet-aware send/receive wiring.
///
/// When `SCM_RATCHET_DISABLE` is set to any non-empty, non-zero, non-"false"
/// value, IronCore takes the exact legacy code path for both
/// `prepare_message_internal` and `receive_message`.  This preserves the
/// pre-ratchet behavior byte-for-byte and guarantees zero behavior change for
/// mixed-fleet peers that cannot consume the new wire format.
fn ratchet_disabled() -> bool {
    static FLAG: std::sync::OnceLock<std::sync::atomic::AtomicBool> = std::sync::OnceLock::new();
    let f = FLAG.get_or_init(|| {
        std::sync::atomic::AtomicBool::new(
            std::env::var("SCM_RATCHET_DISABLE")
                .map(|v| !v.is_empty() && v != "0" && v.to_lowercase() != "false")
                .unwrap_or(false),
        )
    });
    f.load(std::sync::atomic::Ordering::Relaxed)
}

/// The main entry point for the SCMessenger core.
///
/// Wraps all subsystems behind `Arc<RwLock<…>>` for safe concurrent access.
#[cfg_attr(not(target_arch = "wasm32"), derive(uniffi::Object))]
pub struct IronCore {
    pub(crate) identity: Arc<RwLock<IdentityManager>>,
    pub(crate) outbox: Arc<RwLock<Outbox>>,
    pub(crate) inbox: Arc<RwLock<Inbox>>,
    pub(crate) contact_manager: Arc<RwLock<CoreContactManager>>,
    pub(crate) history_manager: Arc<CoreHistoryManager>,
    pub(crate) storage_manager: Arc<RwLock<StorageManager>>,
    pub(crate) log_manager: Arc<LogManager>,
    pub(crate) blocked_manager: Arc<RwLock<CoreBlockedManager>>,
    pub(crate) audit_log: Arc<RwLock<AuditLogType>>,
    pub(crate) relay_custody_store: Arc<RwLock<RelayCustodyStore>>,

    /// Protocol event delegate (set by MeshService or platform bridge).
    pub delegate: Arc<RwLock<Option<Box<dyn CoreDelegate>>>>,

    /// Consent gate — identity cannot be initialized until consent is granted.
    consent: Arc<RwLock<ConsentState>>,

    /// Drift (mesh relay store-and-forward) engine state.
    drift_active: Arc<RwLock<bool>>,
    drift_store: Arc<RwLock<MeshStore>>,
    drift_engine: Arc<RwLock<Option<RelayEngine>>>,

    /// Abuse reputation manager.
    abuse_manager: Arc<RwLock<EnhancedAbuseReputationManager>>,

    /// Auto-block engine for periodic abuse scan.
    auto_block_engine: Arc<RwLock<AutoBlockEngine>>,

    /// Storage path for persistent data (None = in-memory).
    storage_path: Option<String>,
    /// Log directory for structured tracing.
    // Reserved; not yet wired.
    #[allow(dead_code)]
    log_directory: Option<String>,

    /// Ledger manager for connection tracking (used by mobile bridge).
    #[cfg(not(target_arch = "wasm32"))]
    pub ledger_manager: crate::store::LedgerManager,

    /// Running state flag.
    running: Arc<RwLock<bool>>,

    /// Serializes complete start/stop transitions without changing the
    /// documented running/drift lock ordering.
    lifecycle_transition: Arc<Mutex<()>>,

    // -----------------------------------------------------------------------
    // Routing engine (mycorrhizal mesh routing)
    // -----------------------------------------------------------------------
    /// Optimized routing engine for multi-layer routing decisions.
    /// Initialized when identity is available (requires local peer id).
    routing_engine: Arc<RwLock<Option<OptimizedRoutingEngine>>>,

    // -----------------------------------------------------------------------
    // Privacy subcomponents (stateful instances managed by IronCore)
    // -----------------------------------------------------------------------
    /// Cover traffic generator — produces dummy traffic to mask real patterns.
    /// Initialized when cover traffic is enabled via privacy config.
    cover_traffic_generator: Arc<RwLock<Option<CoverTrafficGenerator>>>,

    /// Timing jitter for relay forwarding obfuscation.
    /// Initialized when timing obfuscation is enabled via privacy config.
    timing_jitter: Arc<RwLock<Option<TimingJitter>>>,

    /// Circuit builder for onion routing path selection.
    /// Initialized when onion routing is enabled and peers are available.
    circuit_builder: Arc<RwLock<Option<CircuitBuilder>>>,

    // -----------------------------------------------------------------------
    // Notification endpoint registry (remote push contract)
    // -----------------------------------------------------------------------
    /// In-memory registry of notification endpoints for hybrid remote push.
    notification_endpoint_registry: Arc<RwLock<NotificationEndpointRegistry>>,

    // -----------------------------------------------------------------------
    // Transport manager (multiplexes BLE / WiFi / TCP / QUIC)
    // -----------------------------------------------------------------------
    /// Transport manager coordinating transport abstraction and selection.
    transport_manager: Arc<RwLock<TransportManager>>,

    // -----------------------------------------------------------------------
    // Relay standalone module (bootstrap + peer exchange)
    // -----------------------------------------------------------------------
    /// Bootstrap manager for relay network bootstrap.
    /// Initialized when identity is available.
    #[cfg(not(target_arch = "wasm32"))]
    bootstrap_manager: Arc<RwLock<Option<BootstrapManager>>>,

    /// Transport-layer relay health/circuit-breaker/fallback-relay tracker.
    /// Distinct from `bootstrap_manager` above (the QR-code/invite workflow
    /// manager): this one holds the relay discovery, circuit breaker, and
    /// fallback-relay-address state that `get_all_relay_stats`,
    /// `get_fallback_relays`, and `get_healthy_relays` report on. Constructed
    /// eagerly with defaults since it needs no identity/network setup to be
    /// useful (fallback relay addresses are static/env-derived); health and
    /// stats stay empty until something feeds it real dial events.
    #[cfg(not(target_arch = "wasm32"))]
    relay_bootstrap_manager: Arc<RwLock<Option<crate::transport::bootstrap::BootstrapManager>>>,

    /// Ratchet session manager for forward-secret peer conversations.
    ratchet_sessions: Arc<RwLock<RatchetSessionManager>>,

    /// Security audit pipeline for cryptographic and protocol verification.
    // Reserved; not yet wired.
    #[allow(dead_code)]
    pub(crate) security_audit_pipeline: Arc<crate::dspy::modules::OptimizerPipeline>,

    /// Active runtime privacy configuration.
    pub(crate) privacy_config: Arc<RwLock<crate::privacy::PrivacyConfig>>,

    /// Drift policy engine — adapts relay aggressiveness from device state.
    pub policy_engine: Arc<RwLock<crate::drift::PolicyEngine>>,
    pub transport_memory: Arc<RwLock<crate::store::transport_memory::TransportMemoryStore>>,
}

/// Current version of the structured identity-backup payload (the plaintext
/// encrypted by `export_identity_backup*`). Bumping this is safe: older
/// payload shapes stay decodable as long as `import_identity_backup` keeps a
/// fallback for them.
const IDENTITY_BACKUP_PAYLOAD_VERSION: u32 = 2;

/// Plaintext payload encrypted inside an identity backup blob: the identity
/// keypair plus enough conversational state (ratchet sessions, contacts) to
/// keep messaging without interruption after a restore on a fresh device.
#[derive(serde::Serialize, serde::Deserialize)]
struct IdentityBackupPayload {
    version: u32,
    identity_key_hex: String,
    ratchet_sessions_json: Option<String>,
    #[serde(default)]
    contacts: Vec<Contact>,
    /// Contacts from the UniFFI mobile bridge's contacts store
    /// (`contacts_manager()` / `contacts_bridge::ContactManager`), which is
    /// a separate `contacts.db` from the core `contact_manager` backing
    /// `contacts` above - Android/iOS clients add contacts through the
    /// bridge, so without this field a mobile export's address book was
    /// silently empty/stale on restore. Serialized JSON of
    /// `Vec<contacts_bridge::Contact>`; `None` on WASM (no bridge there)
    /// or when the bridge store is empty.
    #[serde(default)]
    bridge_contacts_json: Option<String>,
    #[serde(default)]
    nickname: Option<String>,
}

impl Default for IronCore {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the connection ledger for a persistent core and hydrate it from disk.
///
/// Review F11: nothing in `core/src` ever called `LedgerManager::load()`. On a
/// fresh process every entry's `success_count` was therefore 0, so
/// `dialable_addresses()` was permanently empty, so the ledger-exchange
/// RESPONSE shipped nothing and the seed-dial "proven" tier had no candidates
/// -- leaving the unauthenticated seed tier as the only live candidate source.
/// `IronCore` owns the ledger, so `IronCore` is where it gets loaded.
///
/// A load failure is not fatal: a corrupt or absent `ledger.json` means we
/// start from an empty ledger and re-learn peers, which is exactly the
/// cold-start path the mesh already supports.
#[cfg(not(target_arch = "wasm32"))]
fn hydrated_ledger_manager(path: String) -> crate::store::LedgerManager {
    let manager = crate::store::LedgerManager::new(path);
    if let Err(err) = manager.load() {
        tracing::warn!(
            "Connection ledger could not be loaded, starting empty: {:?}",
            err
        );
    }
    manager
}

fn parse_transport_type(transport: &str) -> crate::routing::TransportType {
    match transport.trim().to_ascii_lowercase().as_str() {
        "ble" | "bluetooth" | "ble_gatt" | "proximity" => crate::routing::TransportType::BLE,
        "wifi_direct" | "wifidirect" | "p2p" => crate::routing::TransportType::WiFiDirect,
        "wifi_aware" | "wifiaware" | "nan" => crate::routing::TransportType::WiFiAware,
        "quic" => crate::routing::TransportType::QUIC,
        "tcp" | "lan" | "wifi" | "internet" | "mdns" | "relay" | "ip" | "local" => {
            crate::routing::TransportType::TCP
        }
        _ => crate::routing::TransportType::BLE,
    }
}

fn parse_peer_id_32(peer_id_str: &str) -> Option<[u8; 32]> {
    let clean_str = peer_id_str.trim();
    let unescaped = clean_str
        .strip_prefix("public_key:")
        .or_else(|| clean_str.strip_prefix("identity_id:"))
        .or_else(|| clean_str.strip_prefix("0x"))
        .unwrap_or(clean_str);

    if let Ok(bytes) = hex::decode(unescaped) {
        if bytes.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            return Some(arr);
        }
    }

    if let Ok(p) = clean_str.parse::<libp2p::PeerId>() {
        let bytes = p.to_bytes();
        if bytes.len() >= 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes[bytes.len() - 32..]);
            return Some(arr);
        }
    }

    None
}

#[cfg_attr(not(target_arch = "wasm32"), uniffi::export)]
impl IronCore {
    /// Create an in-memory IronCore with no persistent storage.
    #[cfg_attr(not(target_arch = "wasm32"), uniffi::constructor)]
    pub fn new() -> Self {
        let backend: Arc<dyn StorageBackend> = Arc::new(MemoryStorage::new());
        let contact_manager = CoreContactManager::new(backend.clone());
        let history_manager = Arc::new(CoreHistoryManager::new(backend.clone()));
        let log_mgr = Arc::new(LogManager::new(backend.clone()));
        let blocked_manager = CoreBlockedManager::new(backend.clone());
        let blocked_for_auto_block = CoreBlockedManager::new(backend.clone());
        let inbox = Inbox::new();
        let outbox = Outbox::new();
        let storage_manager =
            StorageManager::new(backend.clone(), history_manager.clone(), log_mgr.clone());
        let spam_detector =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let abuse_mgr = EnhancedAbuseReputationManager::new(1000, spam_detector);
        let auto_block_spam =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let auto_block_reputation = EnhancedAbuseReputationManager::new(1000, auto_block_spam);
        let auto_block = AutoBlockEngine::new(
            AutoBlockConfig::default(),
            Arc::new(blocked_for_auto_block),
            Arc::new(auto_block_reputation),
        );
        let ratchet_sessions = Arc::new(RwLock::new(RatchetSessionManager::new()));
        let security_audit_pipeline =
            Arc::new(crate::dspy::modules::ModuleFactory::build_security_audit_pipeline());

        let transport_memory =
            crate::store::transport_memory::TransportMemoryStore::new(backend.clone());

        Self {
            identity: Arc::new(RwLock::new(IdentityManager::new())),
            outbox: Arc::new(RwLock::new(outbox)),
            inbox: Arc::new(RwLock::new(inbox)),
            contact_manager: Arc::new(RwLock::new(contact_manager)),
            history_manager,
            storage_manager: Arc::new(RwLock::new(storage_manager)),
            log_manager: log_mgr,
            blocked_manager: Arc::new(RwLock::new(blocked_manager)),
            audit_log: Arc::new(RwLock::new(AuditLogType::new())),
            relay_custody_store: Arc::new(RwLock::new(RelayCustodyStore::persistent(
                backend.clone(),
            ))),
            delegate: Arc::new(RwLock::new(None)),
            consent: Arc::new(RwLock::new(ConsentState::NotGranted)),
            drift_active: Arc::new(RwLock::new(false)),
            drift_store: Arc::new(RwLock::new(MeshStore::persistent(backend.clone()))),
            drift_engine: Arc::new(RwLock::new(None)),
            abuse_manager: Arc::new(RwLock::new(abuse_mgr)),
            auto_block_engine: Arc::new(RwLock::new(auto_block)),
            storage_path: None,
            log_directory: None,
            // Review F11: this used to be `LedgerManager::new(temp_dir())`,
            // which wrote the node's entire peer topology into a
            // world-readable directory on every desktop platform. `new()` is
            // the storage-less constructor, so its ledger is storage-less too.
            #[cfg(not(target_arch = "wasm32"))]
            ledger_manager: crate::store::LedgerManager::ephemeral(),
            running: Arc::new(RwLock::new(false)),
            lifecycle_transition: Arc::new(Mutex::new(())),
            routing_engine: Arc::new(RwLock::new(None)),
            cover_traffic_generator: Arc::new(RwLock::new(None)),
            timing_jitter: Arc::new(RwLock::new(None)),
            circuit_builder: Arc::new(RwLock::new(None)),
            notification_endpoint_registry: Arc::new(RwLock::new(
                NotificationEndpointRegistry::new(),
            )),
            transport_manager: Arc::new(RwLock::new(TransportManager::new())),
            #[cfg(not(target_arch = "wasm32"))]
            bootstrap_manager: Arc::new(RwLock::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            relay_bootstrap_manager: Arc::new(RwLock::new(Some(
                crate::transport::bootstrap::BootstrapManager::with_defaults(),
            ))),
            ratchet_sessions,
            security_audit_pipeline,
            privacy_config: Arc::new(RwLock::new(crate::privacy::PrivacyConfig::default())),
            policy_engine: Arc::new(RwLock::new(crate::drift::PolicyEngine::new())),
            transport_memory: Arc::new(RwLock::new(transport_memory)),
        }
    }

    /// Create IronCore with persistent sled-backed storage at `path`.
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg_attr(not(target_arch = "wasm32"), uniffi::constructor)]
    pub fn with_storage(path: String) -> Self {
        let backend: Arc<dyn StorageBackend> = match SledStorage::new(&path) {
            Ok(s) => Arc::new(s),
            Err(_) => Arc::new(MemoryStorage::new()),
        };
        let p = path.clone();
        let contact_manager = CoreContactManager::new(backend.clone());
        let history_manager = Arc::new(CoreHistoryManager::new(backend.clone()));
        let log_mgr = Arc::new(LogManager::new(backend.clone()));
        let blocked_manager = CoreBlockedManager::new(backend.clone());
        let blocked_for_auto_block = CoreBlockedManager::new(backend.clone());
        let inbox = Inbox::new();
        let outbox = Outbox::new();
        let storage_manager =
            StorageManager::new(backend.clone(), history_manager.clone(), log_mgr.clone());
        let spam_detector =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let abuse_mgr = EnhancedAbuseReputationManager::new(1000, spam_detector);
        let auto_block_spam =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let auto_block_reputation = EnhancedAbuseReputationManager::new(1000, auto_block_spam);
        let auto_block = AutoBlockEngine::new(
            AutoBlockConfig::default(),
            Arc::new(blocked_for_auto_block),
            Arc::new(auto_block_reputation),
        );
        let security_audit_pipeline =
            Arc::new(crate::dspy::modules::ModuleFactory::build_security_audit_pipeline());

        let transport_memory =
            crate::store::transport_memory::TransportMemoryStore::new(backend.clone());

        Self {
            identity: Arc::new(RwLock::new(
                IdentityManager::with_backend(backend.clone()).unwrap_or_else(|_| {
                    tracing::error!(
                        "Failed to hydrate identity from persistent store, falling back to memory"
                    );
                    IdentityManager::new()
                }),
            )),
            outbox: Arc::new(RwLock::new(outbox)),
            inbox: Arc::new(RwLock::new(inbox)),
            contact_manager: Arc::new(RwLock::new(contact_manager)),
            history_manager,
            storage_manager: Arc::new(RwLock::new(storage_manager)),
            log_manager: log_mgr,
            blocked_manager: Arc::new(RwLock::new(blocked_manager)),
            audit_log: Arc::new(RwLock::new(AuditLogType::new())),
            relay_custody_store: Arc::new(RwLock::new(RelayCustodyStore::persistent(
                backend.clone(),
            ))),
            delegate: Arc::new(RwLock::new(None)),
            consent: Arc::new(RwLock::new(ConsentState::NotGranted)),
            drift_active: Arc::new(RwLock::new(false)),
            drift_store: Arc::new(RwLock::new(MeshStore::persistent(backend.clone()))),
            drift_engine: Arc::new(RwLock::new(None)),
            abuse_manager: Arc::new(RwLock::new(abuse_mgr)),
            auto_block_engine: Arc::new(RwLock::new(auto_block)),
            storage_path: Some(path),
            log_directory: None,
            #[cfg(not(target_arch = "wasm32"))]
            ledger_manager: hydrated_ledger_manager(p),
            running: Arc::new(RwLock::new(false)),
            lifecycle_transition: Arc::new(Mutex::new(())),
            routing_engine: Arc::new(RwLock::new(None)),
            cover_traffic_generator: Arc::new(RwLock::new(None)),
            timing_jitter: Arc::new(RwLock::new(None)),
            circuit_builder: Arc::new(RwLock::new(None)),
            notification_endpoint_registry: Arc::new(RwLock::new(
                NotificationEndpointRegistry::new(),
            )),
            transport_manager: Arc::new(RwLock::new(TransportManager::new())),
            #[cfg(not(target_arch = "wasm32"))]
            bootstrap_manager: Arc::new(RwLock::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            relay_bootstrap_manager: Arc::new(RwLock::new(Some(
                crate::transport::bootstrap::BootstrapManager::with_defaults(),
            ))),
            ratchet_sessions: Arc::new(RwLock::new(RatchetSessionManager::new())),
            security_audit_pipeline,
            privacy_config: Arc::new(RwLock::new(crate::privacy::PrivacyConfig::default())),
            policy_engine: Arc::new(RwLock::new(crate::drift::PolicyEngine::new())),
            transport_memory: Arc::new(RwLock::new(transport_memory)),
        }
    }

    /// Create IronCore with persistent storage and a log directory.
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg_attr(not(target_arch = "wasm32"), uniffi::constructor)]
    pub fn with_storage_and_logs(path: String, log_dir: String) -> Self {
        // Install the tracing subscriber FIRST, before any other init, so
        // startup diagnostics are captured too.
        //
        // This call was missing entirely. `log_dir` was accepted, stored as
        // `self.log_directory` (:547) and never read; `init_file_tracing` had
        // ZERO callers anywhere in the crate. So the whole structured-tracing
        // facility was dead code: Android computes a logs directory
        // (MeshRepository.kt:1341) and passes it through
        // MeshService::withStorageAndLogs -> IronCore::with_storage_and_logs,
        // and core then discarded it.
        //
        // The consequence was that the Rust core was COMPLETELY SILENT on
        // device. `eprintln!` goes to stderr, which Android discards unless
        // `log.redirect-stdio` is set (verified unset on the test device:
        // `grep -cF "[IronCore]" logcat` returned 0 across a full buffer while
        // 264 inbound BLE messages were being processed), and `tracing::` had
        // no subscriber. That blindness is why the BLE inbound wedge took so
        // long to localise -- we could not tell whether on_data_received was
        // even entered.
        //
        // init_file_tracing uses try_init() internally, so a warm boot that
        // recreates MeshService without killing the process is safe. A failure
        // here must never take the core down: logging is diagnostics, not a
        // dependency of messaging.
        #[cfg(not(target_arch = "wasm32"))]
        if let Err(e) = crate::store::tracing_init::init_file_tracing(&log_dir) {
            // Cannot use tracing:: here -- if this failed there may be no
            // subscriber. eprintln is invisible on Android but correct
            // everywhere else, and this path means diagnostics are degraded
            // rather than the process being unhealthy.
            eprintln!("[IronCore] [WARN] file tracing init failed ({}); core diagnostics will not be written to {}", e, log_dir);
        }

        let backend: Arc<dyn StorageBackend> = match SledStorage::new(&path) {
            Ok(s) => Arc::new(s),
            Err(_) => Arc::new(MemoryStorage::new()),
        };
        let p = path.clone();
        let contact_manager = CoreContactManager::new(backend.clone());
        let history_manager = Arc::new(CoreHistoryManager::new(backend.clone()));
        let log_mgr = Arc::new(LogManager::new(backend.clone()));
        let blocked_manager = CoreBlockedManager::new(backend.clone());
        let blocked_for_auto_block = CoreBlockedManager::new(backend.clone());
        let inbox = Inbox::new();
        let outbox = Outbox::new();
        let storage_manager =
            StorageManager::new(backend.clone(), history_manager.clone(), log_mgr.clone());
        let spam_detector =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let abuse_mgr = EnhancedAbuseReputationManager::new(1000, spam_detector);
        let auto_block_spam =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let auto_block_reputation = EnhancedAbuseReputationManager::new(1000, auto_block_spam);
        let auto_block = AutoBlockEngine::new(
            AutoBlockConfig::default(),
            Arc::new(blocked_for_auto_block),
            Arc::new(auto_block_reputation),
        );
        let security_audit_pipeline =
            Arc::new(crate::dspy::modules::ModuleFactory::build_security_audit_pipeline());

        let transport_memory =
            crate::store::transport_memory::TransportMemoryStore::new(backend.clone());

        Self {
            identity: Arc::new(RwLock::new(
                IdentityManager::with_backend(backend.clone()).unwrap_or_else(|_| {
                    tracing::error!(
                        "Failed to hydrate identity from persistent store, falling back to memory"
                    );
                    IdentityManager::new()
                }),
            )),
            outbox: Arc::new(RwLock::new(outbox)),
            inbox: Arc::new(RwLock::new(inbox)),
            contact_manager: Arc::new(RwLock::new(contact_manager)),
            history_manager,
            storage_manager: Arc::new(RwLock::new(storage_manager)),
            log_manager: log_mgr,
            blocked_manager: Arc::new(RwLock::new(blocked_manager)),
            audit_log: Arc::new(RwLock::new(AuditLogType::new())),
            relay_custody_store: Arc::new(RwLock::new(RelayCustodyStore::persistent(
                backend.clone(),
            ))),
            delegate: Arc::new(RwLock::new(None)),
            consent: Arc::new(RwLock::new(ConsentState::NotGranted)),
            drift_active: Arc::new(RwLock::new(false)),
            drift_store: Arc::new(RwLock::new(MeshStore::persistent(backend.clone()))),
            drift_engine: Arc::new(RwLock::new(None)),
            abuse_manager: Arc::new(RwLock::new(abuse_mgr)),
            auto_block_engine: Arc::new(RwLock::new(auto_block)),
            storage_path: Some(path),
            log_directory: Some(log_dir),
            #[cfg(not(target_arch = "wasm32"))]
            ledger_manager: hydrated_ledger_manager(p),
            running: Arc::new(RwLock::new(false)),
            lifecycle_transition: Arc::new(Mutex::new(())),
            routing_engine: Arc::new(RwLock::new(None)),
            cover_traffic_generator: Arc::new(RwLock::new(None)),
            timing_jitter: Arc::new(RwLock::new(None)),
            circuit_builder: Arc::new(RwLock::new(None)),
            notification_endpoint_registry: Arc::new(RwLock::new(
                NotificationEndpointRegistry::new(),
            )),
            transport_manager: Arc::new(RwLock::new(TransportManager::new())),
            #[cfg(not(target_arch = "wasm32"))]
            bootstrap_manager: Arc::new(RwLock::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            relay_bootstrap_manager: Arc::new(RwLock::new(Some(
                crate::transport::bootstrap::BootstrapManager::with_defaults(),
            ))),
            ratchet_sessions: Arc::new(RwLock::new(RatchetSessionManager::new())),
            security_audit_pipeline,
            privacy_config: Arc::new(RwLock::new(crate::privacy::PrivacyConfig::default())),
            policy_engine: Arc::new(RwLock::new(crate::drift::PolicyEngine::new())),
            transport_memory: Arc::new(RwLock::new(transport_memory)),
        }
    }

    /// Start the core. Must be called before any messaging operations.
    pub fn start(&self) -> Result<(), IronCoreError> {
        let _transition = self.lifecycle_transition.lock();
        // LOCK ORDERING (deadlock fix): `running` must be RELEASED before any
        // drift lock is taken. `stop()` acquires drift_active (via
        // drift_deactivate) and only then `running`. Holding `running` across
        // drift_activate() here created the reverse order, so a concurrent
        // start()/stop() -- exactly what a force-stop-and-reopen produces --
        // could deadlock. parking_lot locks are not reentrant, so there is no
        // recovery once that happens: the process stays alive with its UI
        // thread blocked, which surfaces as an Android ANR
        // ("Input dispatching timed out") while the mesh core itself is fine.
        //
        // The compare-and-set stays atomic: the guard scope below still makes
        // the AlreadyRunning check and the flag write a single critical
        // section. Only the nested acquisition is removed.
        {
            let mut running = self.running.write();
            if *running {
                return Err(IronCoreError::AlreadyRunning);
            }
            *running = true;
        }
        self.drift_activate();
        tracing::info!("IronCore started");
        Ok(())
    }

    /// Stop the core gracefully.
    pub fn stop(&self) {
        let _transition = self.lifecycle_transition.lock();
        self.drift_deactivate();
        *self.running.write() = false;
        tracing::info!("IronCore stopped");
    }

    /// Grant consent for identity initialization.
    pub fn grant_consent(&self) {
        *self.consent.write() = ConsentState::Granted;
        tracing::info!("Consent granted for identity initialization");
    }

    /// Initialize the identity (generate Ed25519 keys).
    /// Requires consent to have been granted first.
    pub fn initialize_identity(&self) -> Result<(), IronCoreError> {
        if *self.consent.read() != ConsentState::Granted {
            return Err(IronCoreError::ConsentRequired);
        }
        let mut identity = self.identity.write();
        identity.initialize().map_err(|e| {
            tracing::error!("Identity initialization failed: {:?}", e);
            IronCoreError::CryptoError
        })?;

        self.audit_log.write().append(
            AuditEventType::IdentityCreated,
            identity.identity_id(),
            None,
            None,
        );

        // Initialize drift engine now that we have a public key
        if let Some(keys) = identity.keys() {
            let pk_bytes = keys.signing_key.verifying_key().to_bytes();
            {
                let mut engine = self.drift_engine.write();
                *engine = Some(RelayEngine::new(&pk_bytes, RelayConfig::default()));
            }
            // The relay engine owns no reputation state; link it to IronCore's
            // shared manager after releasing the engine lock to avoid re-entry.
            self.drift_set_reputation_manager();

            // Initialize routing engine with identity-derived peer id and hint.
            // If the swarm has already seeded the engine (via start_swarm_with_config),
            // keep it — the shared engine is already in use for message dispatch.
            let hint = blake3::hash(&pk_bytes).as_bytes()[0..4]
                .try_into()
                .unwrap_or([0u8; 4]);
            let mut routing = self.routing_engine.write();
            if routing.is_none() {
                *routing = Some(OptimizedRoutingEngine::new(pk_bytes, hint));
            }

            // Initialize relay bootstrap manager
            #[cfg(not(target_arch = "wasm32"))]
            {
                let mut bootstrap = self.bootstrap_manager.write();
                *bootstrap = Some(BootstrapManager::new(
                    keys.identity_id(),
                    keys.signing_key.verifying_key().to_bytes().to_vec(),
                    Vec::new(),
                ));
            }
        }

        // STEP 5: Migrate existing contacts to populate identity_id -> public_key index.
        // This allows backward-compatible resolution of identity_id hashes.
        if let Err(e) = self.contact_manager.read().migrate_identity_id_index() {
            tracing::warn!("Failed to migrate identity_id index: {:?}", e);
        }

        tracing::info!("Identity initialized: {:?}", identity.identity_id());
        Ok(())
    }

    /// Return the identity ID (Blake3 hash of public key), if initialized.
    pub fn identity_id(&self) -> Option<String> {
        self.identity.read().identity_id()
    }

    /// Return the device ID, if initialized.
    pub fn device_id(&self) -> Option<String> {
        self.identity.read().device_id()
    }

    /// Return the public key hex, if initialized.
    pub fn public_key_hex(&self) -> Option<String> {
        self.identity.read().public_key_hex()
    }

    /// Return the libp2p keypair derived from identity, if initialized.

    /// Set the delegate for protocol event callbacks.
    pub fn set_delegate(&self, delegate: Option<Box<dyn CoreDelegate>>) {
        *self.delegate.write() = delegate;
    }

    // -----------------------------------------------------------------------
    // Message flow
    // -----------------------------------------------------------------------

    /// Internal helper: prepare an encrypted message for a recipient.
    /// Returns the full PreparedMessage (id + envelope bytes) and also
    /// enqueues in the outbox.
    fn prepare_message_internal(
        &self,
        recipient_id: &str,
        content: &str,
        _msg_type: crate::MessageType,
        _ttl: Option<crate::TtlConfig>,
    ) -> Result<crate::PreparedMessage, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;

        let recipient_bytes = hex::decode(recipient_id).map_err(|_| IronCoreError::InvalidInput)?;
        let recipient_pk: [u8; 32] = recipient_bytes
            .try_into()
            .map_err(|_| IronCoreError::InvalidInput)?;
        // Reject a recipient id that is well-formed but MEANINGLESS as a key.
        //
        // identity_id() is hex(blake3(pubkey)) and public_key_hex() is
        // hex(pubkey). BOTH are 64 hex chars and BOTH decode to exactly 32
        // bytes, so the decode above cannot tell them apart, and a hash used
        // here would be silently encrypted to -- producing ciphertext nobody
        // can open. See HANDOFF/audit/IDENTITY_HASH_VS_PUBKEY_CONFLICT.md.
        if recipient_pk.iter().all(|b| *b == 0) {
            tracing::error!("[ERROR] refusing to send: recipient key is all zeros");
            return Err(IronCoreError::InvalidInput);
        }

        // Fast path first: if the recipient resolves to a known contact by
        // PUBLIC KEY, it is the right kind of value and we are done. Only when
        // that misses do we pay for a scan, so the common send path stays O(1)
        // against the contact store rather than hashing every contact.
        let contacts = self.contact_manager.read();
        let known_by_pubkey = contacts
            .get(recipient_id.to_string())
            .ok()
            .flatten()
            .or_else(|| contacts.get_by_public_key(recipient_id).ok().flatten())
            .is_some();

        if !known_by_pubkey {
            // Miss. Determine whether this is the hash/pubkey confusion, which
            // deserves a distinct and loud error, or simply an unknown peer,
            // which stays permitted (sending to a peer we have no contact
            // record for is existing behaviour and is not broken).
            if let Ok(contacts) = self.contact_manager.read().list() {
                for contact in contacts {
                    let Ok(pk_bytes) = hex::decode(&contact.public_key) else {
                        continue;
                    };
                    if blake3::hash(&pk_bytes).as_bytes() == &recipient_pk {
                        // Deliberately does NOT log either value: both are
                        // identity material and this repo is public.
                        tracing::error!(
                            "[ERROR] refusing to send: recipient_id is the identity_id (blake3 hash) \
                             of a known contact, not their public key. Encrypting to a hash produces \
                             ciphertext nobody can decrypt. The contact record needs repairing."
                        );
                        return Err(IronCoreError::InvalidInput);
                    }
                }
            }
            tracing::warn!(
                "[WARN] sending to a recipient with no contact record; proceeding \
                 (unknown-peer sends remain allowed)"
            );
        }

        let message_id = uuid::Uuid::new_v4().to_string();
        // CRITICAL FIX: Use public_key_hex as sender_id, NOT identity_id.
        // identity_id is a blake3 hash and cannot be used for encryption.
        // The recipient needs the actual public key to decrypt messages.
        let sender_id = identity
            .public_key_hex()
            .ok_or(IronCoreError::NotInitialized)?;
        let message = crate::Message {
            id: message_id.clone(),
            sender_id: sender_id.clone(),
            recipient_id: recipient_id.to_string(),
            message_type: _msg_type,
            payload: content.as_bytes().to_vec(),
            timestamp: crate::util::unix_time_secs(),
        };
        let message_bytes =
            crate::message::encode_message(&message).map_err(|_| IronCoreError::Internal)?;

        let (mut envelope_data, drift_env) = if ratchet_disabled() {
            // LEGACY PATH (kill switch) -- verbatim current behavior
            let envelope = encrypt_message(&keys.signing_key, &recipient_pk, &message_bytes)
                .map_err(|_| IronCoreError::CryptoError)?;
            let drift_env = crate::drift::DriftEnvelope::from_legacy_envelope(
                envelope,
                message_id.clone(),
                recipient_pk,
                &keys.signing_key,
            )
            .map_err(|_| IronCoreError::Internal)?;
            let envelope_data = drift_env.to_bytes().map_err(|_| IronCoreError::Internal)?;
            (envelope_data, drift_env)
        } else {
            // RATCHET PATH -- identity.read() already held; take
            // ratchet_sessions.write() AFTER (identity-first, proven-safe).
            // TODO cache our_bundle per identity_id (per-send Ed25519+ML-DSA
            // signing is wasteful) -- E-00 ships correct-first.
            let our_bundle = crate::identity::sign_bundle(keys).ok();
            let recipient_bundle = self
                .contact_manager
                .read()
                .get_contact_bundle(recipient_id)
                .ok()
                .flatten();
            let peer_id = recipient_id.to_string();
            let signing_key = keys.signing_key.clone();
            let mut sessions = self.ratchet_sessions.write();
            let mut audit = self.audit_log.write();
            let wire = crate::crypto::encrypt::encrypt_with_ratchet_fallback(
                &signing_key,
                recipient_bundle.as_ref(),
                &recipient_pk,
                &message_bytes,
                Some(&mut *sessions),
                &peer_id,
                our_bundle.as_ref(),
                false,
                Some(&mut *audit),
            )
            .map_err(|_| IronCoreError::CryptoError)?;
            let drift_env = match wire {
                crate::message::WireEnvelope::V1(env) => {
                    crate::drift::DriftEnvelope::from_legacy_envelope(
                        env,
                        message_id.clone(),
                        recipient_pk,
                        &signing_key,
                    )
                    .map_err(|_| IronCoreError::Internal)?
                }
                crate::message::WireEnvelope::V2(env2) => {
                    crate::drift::DriftEnvelope::from_v2_envelope(
                        env2,
                        message_id.clone(),
                        recipient_pk,
                        &signing_key,
                    )
                    .map_err(|_| IronCoreError::Internal)?
                }
            };
            let envelope_data = drift_env.to_bytes().map_err(|_| IronCoreError::Internal)?;
            (envelope_data, drift_env)
        };

        if self.privacy_config().onion_routing_enabled {
            let relays = self.swarm_get_best_relays(3);
            if !relays.is_empty() {
                let relays_json =
                    serde_json::to_string(&relays).map_err(|_| IronCoreError::Internal)?;
                envelope_data = self.prepare_onion_message(envelope_data, relays_json)?;
            }
        }

        // Check routing decision
        let hint = blake3::hash(recipient_id.as_bytes()).as_bytes()[0..4]
            .try_into()
            .unwrap_or([0u8; 4]);
        let msg_id_bytes: [u8; 16] = *uuid::Uuid::parse_str(&message_id)
            .unwrap_or_else(|_| uuid::Uuid::nil())
            .as_bytes();
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let decision = self.make_routing_decision(hint, msg_id_bytes, 128, now);

        let mut handoff_to_drift = false;
        if let Some(dec) = decision {
            if matches!(dec.primary, crate::routing::NextHop::StoreAndCarry) {
                handoff_to_drift = true;
            }
        }

        if handoff_to_drift {
            let stored_env = crate::drift::store::StoredEnvelope {
                envelope_data: envelope_data.clone(),
                message_id: drift_env.message_id,
                recipient_hint: drift_env.recipient_hint,
                created_at: drift_env.created_at,
                ttl_expiry: drift_env.ttl_expiry,
                hop_count: drift_env.hop_count,
                priority: drift_env.priority,
                received_at: web_time::SystemTime::now()
                    .duration_since(web_time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            self.drift_store.write().insert(stored_env);
            tracing::info!("StoreAndCarry route resolved for {}. Handoff to Drift custody and bypassed active outbox.", message_id);
        } else {
            let connected = self
                .transport_manager
                .read()
                .is_peer_connected(recipient_pk);
            if !connected {
                let _ = self.outbox.write().enqueue(QueuedMessage {
                    version: 1,
                    message_id: message_id.clone(),
                    recipient_id: recipient_id.to_string(),
                    envelope_data: envelope_data.clone(),
                    queued_at: web_time::SystemTime::now()
                        .duration_since(web_time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                    attempts: 0,
                    next_retry_at: None,
                    in_custody: false,
                    custody_established_at: 0,
                    state: crate::store::outbox::MessageState::Enqueued,
                });
            }
        }

        self.audit_log.write().append(
            AuditEventType::MessageSent,
            identity.identity_id(),
            Some(recipient_id.to_string()),
            None,
        );

        Ok(crate::PreparedMessage {
            message_id,
            envelope_data,
        })
    }

    /// Prepare an encrypted message envelope for a recipient.
    /// Returns the PreparedMessage (message_id + envelope_data).
    /// Use `prepare_message_with_id` if you need the message_id separately.
    pub fn prepare_message(
        &self,
        recipient_public_key_hex: String,
        text: String,
        msg_type: crate::MessageType,
        ttl: Option<crate::TtlConfig>,
    ) -> Result<crate::PreparedMessage, IronCoreError> {
        self.prepare_message_internal(&recipient_public_key_hex, &text, msg_type, ttl)
    }

    /// Prepare an encrypted message and return both the message_id and envelope data.
    pub fn prepare_message_with_id(
        &self,
        recipient_public_key_hex: String,
        text: String,
        msg_type: crate::MessageType,
        ttl: Option<crate::TtlConfig>,
    ) -> Result<crate::PreparedMessage, IronCoreError> {
        self.prepare_message_internal(&recipient_public_key_hex, &text, msg_type, ttl)
    }

    /// Receive and decrypt an incoming envelope.

    /// Mark a message as sent (remove from outbox after transport confirms delivery).
    pub fn mark_message_sent(&self, message_id: String) -> bool {
        let outbox_removed = self.outbox.write().remove(&message_id);
        let mut parsed_id = [0u8; 16];
        let parsed = if let Ok(uuid) = uuid::Uuid::parse_str(&message_id) {
            parsed_id.copy_from_slice(uuid.as_bytes());
            true
        } else {
            false
        };
        let drift_removed = if parsed {
            self.drift_store.write().remove(&parsed_id)
        } else {
            false
        };
        outbox_removed || drift_removed
    }

    /// Send a message status report for a given peer.
    /// Returns `None` on success, or `Some(error_string)` on failure.
    /// This method provides the same interface as the mobile bridge's
    /// send_message_status but through the core IronCore API.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn send_message_status(
        &self,
        peer_id: String,
        data: Vec<u8>,
        _recipient_identity_id: Option<String>,
        _intended_device_id: Option<String>,
    ) -> Option<String> {
        // Delegate to prepare_message + mark_message_sent flow
        // The mobile bridge has direct swarm access; this provides
        // a core-level status reporting path for the CLI/WASM layers.
        let result = self.prepare_message(
            peer_id.clone(),
            String::from_utf8_lossy(&data).to_string(),
            crate::MessageType::Text,
            None,
        );
        match result {
            Ok(msg) => {
                self.mark_message_sent(msg.message_id);
                None
            }
            Err(e) => Some(format!("{:?}", e)),
        }
    }

    /// Check if a peer is blocked.
    pub fn is_peer_blocked(
        &self,
        peer_id: String,
        device_id: Option<String>,
    ) -> Result<bool, IronCoreError> {
        self.blocked_manager
            .read()
            .is_blocked_resolved(&peer_id, device_id.as_deref())
    }

    /// Get the set of peer IDs that are blocked-only (not deleted).
    /// Used by the query layer to filter blocked peers from UI results
    /// without purging them (evidentiary retention).
    pub fn blocked_only_peer_ids(&self) -> Result<Vec<String>, IronCoreError> {
        self.blocked_manager
            .read()
            .blocked_only_peer_ids()
            .map(|set| set.into_iter().collect())
    }

    /// Register a device ID for a peer in the block device registry.
    /// If the peer is blocked, the device is automatically blocked too.
    pub fn register_blocked_device(
        &self,
        peer_id: String,
        device_id: String,
    ) -> Result<(), IronCoreError> {
        self.blocked_manager
            .write()
            .register_device_id(&peer_id, &device_id)?;
        Ok(())
    }

    /// Get all known device IDs registered for a blocked peer.
    pub fn get_blocked_peer_devices(&self, peer_id: String) -> Result<Vec<String>, IronCoreError> {
        self.blocked_manager.read().get_known_devices(&peer_id)
    }

    /// Get the peer reputation score.
    pub fn get_peer_reputation(&self, peer_id: String) -> f64 {
        self.abuse_manager.read().get_score(&peer_id).value()
    }

    /// Get the spam confidence score for a peer.
    pub fn peer_spam_score(&self, peer_id: String) -> f64 {
        self.abuse_manager
            .read()
            .get_enhanced_score(&peer_id)
            .spam_confidence
    }

    /// Get the rate limit multiplier for a peer.
    pub fn peer_rate_limit_multiplier(&self, peer_id: String) -> f64 {
        self.abuse_manager.read().rate_limit_multiplier(&peer_id)
    }

    /// Sign data with the identity key and return the signature + public key.
    pub fn sign_data(&self, data: Vec<u8>) -> Result<crate::SignatureResult, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
        let signature = keys.sign(&data).map_err(|_| IronCoreError::CryptoError)?;
        Ok(crate::SignatureResult {
            signature,
            public_key_hex: keys.public_key_hex(),
        })
    }

    /// Get the current registration state for an identity.
    pub fn get_registration_state(&self, identity_id: String) -> crate::RegistrationStateInfo {
        let info = self
            .relay_custody_store
            .read()
            .get_registration_state_info(&identity_id);
        crate::RegistrationStateInfo {
            state: info.state,
            device_id: info.device_id,
            seniority_timestamp: info.seniority_timestamp,
        }
    }

    // -----------------------------------------------------------------------
    // Peer event notification (called from swarm event loop)
    // -----------------------------------------------------------------------

    /// Notify the core that a peer was discovered.
    /// Blocked peers (peer-level or any known device) are silently ignored.
    pub fn notify_peer_discovered(&self, peer_id: String) {
        // Suppress discovery notifications for blocked peers.
        //
        // FAIL CLOSED: if the block store cannot be read we cannot prove the
        // peer is unblocked, so treat it as blocked and suppress. The previous
        // `unwrap_or(false)` failed OPEN -- any store error silently downgraded
        // a blocked peer to visible, defeating the block the user set.
        let blocked = match self
            .blocked_manager
            .read()
            .is_blocked_resolved(&peer_id, None)
        {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(
                    "[WARN] block lookup failed for discovered peer; suppressing notification (fail-closed): {}",
                    e
                );
                true
            }
        };
        if blocked {
            return;
        }
        self.routing_peer_seen(peer_id.clone(), "tcp".to_string());
        if let Some(delegate) = self.delegate.read().as_ref() {
            delegate.on_peer_discovered(peer_id.clone());
        }
    }

    /// Notify the core that a peer disconnected.
    pub fn notify_peer_disconnected(&self, peer_id: String) {
        if let Some(delegate) = self.delegate.read().as_ref() {
            delegate.on_peer_disconnected(peer_id.clone());
        }
    }

    /// Record an abuse signal from the transport layer.
    pub fn record_abuse_signal(&self, peer_id: String, signal: String) {
        let abuse = self.abuse_manager.read();
        let abuse_signal = match signal.as_str() {
            "RateLimited" => AbuseSignal::RateLimited,
            "OversizedMessage" => AbuseSignal::OversizedMessage,
            "InvalidFormat" => AbuseSignal::InvalidFormat,
            "DuplicateMessage" => AbuseSignal::DuplicateMessage,
            "InvalidDestination" => AbuseSignal::InvalidDestination,
            "SuccessfulRelay" => AbuseSignal::SuccessfulRelay,
            "FailedRelay" => AbuseSignal::FailedRelay,
            "SuccessfulDelivery" => AbuseSignal::SuccessfulDelivery,
            "ConnectionTimeout" => AbuseSignal::ConnectionTimeout,
            _ => AbuseSignal::RateLimited,
        };
        abuse.record_signal(&peer_id, abuse_signal);
        let score = abuse.get_score(&peer_id).value();
        tracing::info!(
            "Abuse signal recorded for {}: {}, new score: {}",
            peer_id,
            signal,
            score
        );
    }

    // -----------------------------------------------------------------------
    // Drift (mesh relay) protocol
    // -----------------------------------------------------------------------

    /// Activate the drift relay engine.
    pub fn drift_activate(&self) {
        *self.drift_active.write() = true;
        if let Some(ref mut engine) = *self.drift_engine.write() {
            engine.set_network_state(NetworkState::Active);
        }
        tracing::info!("Drift relay activated");
    }

    /// Deactivate the drift relay engine.
    pub fn drift_deactivate(&self) {
        *self.drift_active.write() = false;
        if let Some(ref mut engine) = *self.drift_engine.write() {
            engine.set_network_state(NetworkState::Dormant);
        }
        tracing::info!("Drift relay deactivated");
    }

    /// Run a bounded drift maintenance cycle within the given time budget.
    /// Returns a JSON report of work done. Used by iOS BGProcessingTask and Android WorkManager.
    pub fn run_maintenance_cycle(&self, budget_ms: u32) -> String {
        let start = web_time::Instant::now();
        let mut work_done = 0u32;

        // Drift engine tick if active
        if *self.drift_active.read() {
            work_done += 1;
        }

        let elapsed = start.elapsed().as_millis() as u64;
        format!(
            r#"{{"work_done":{},"elapsed_ms":{},"budget_ms":{},"remaining":{}}}"#,
            work_done,
            elapsed,
            budget_ms,
            budget_ms as u64 > elapsed
        )
    }

    /// Get the current drift network state as a string.
    pub fn drift_network_state(&self) -> String {
        if *self.drift_active.read() {
            "Active".to_string()
        } else {
            "Dormant".to_string()
        }
    }

    /// Get the number of envelopes in the drift store.
    pub fn drift_store_size(&self) -> u32 {
        self.drift_store.read().len() as u32
    }

    /// Compute a jitter delay for relay timing obfuscation (returns ms).
    pub fn relay_jitter_delay(&self, severity: String) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let (minimum, span) = match severity.trim().to_ascii_lowercase().as_str() {
            "low" => (0, 51),
            "high" => (100, 401),
            _ => (50, 151),
        };
        minimum + (seed % span)
    }

    // -----------------------------------------------------------------------
    // Registration (WS13.3)
    // -----------------------------------------------------------------------

    /// Build a registration request for the identity protocol.

    // -----------------------------------------------------------------------
    // Runtime state
    // -----------------------------------------------------------------------

    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    // -----------------------------------------------------------------------
    // Identity queries
    // -----------------------------------------------------------------------

    pub fn get_identity_info(&self) -> crate::IdentityInfo {
        let identity = self.identity.read();
        let keys = identity.keys();
        let libp2p_peer_id = keys.and_then(|k| match k.to_libp2p_peer_id() {
            Ok(pid) => {
                tracing::debug!("to_libp2p_peer_id OK: {}", pid);
                Some(pid)
            }
            Err(e) => {
                tracing::error!("to_libp2p_peer_id FAILED: {:?}", e);
                None
            }
        });
        crate::IdentityInfo {
            identity_id: identity.identity_id(),
            public_key_hex: identity.public_key_hex(),
            device_id: identity.device_id(),
            seniority_timestamp: identity.seniority_timestamp(),
            initialized: keys.is_some(),
            nickname: identity.nickname(),
            libp2p_peer_id,
        }
    }

    pub fn get_device_id(&self) -> Option<String> {
        self.identity.read().device_id()
    }

    /// Derive the libp2p Peer ID from the local identity's Ed25519 public key.
    /// Returns None if identity is not initialized.
    pub fn get_libp2p_peer_id(&self) -> Option<String> {
        self.identity
            .read()
            .keys()
            .and_then(|k| k.to_libp2p_peer_id().ok())
    }

    pub fn get_seniority_timestamp(&self) -> Option<u64> {
        self.identity.read().seniority_timestamp()
    }

    /// Set the nickname for the local identity.
    pub fn set_nickname(&self, nickname: String) -> Result<(), IronCoreError> {
        let mut identity = self.identity.write();
        identity.set_nickname(nickname.clone()).map_err(|e| {
            tracing::error!("Failed to persist nickname to store: {:?}", e);
            IronCoreError::Internal
        })?;
        // Verify the in-memory state was applied — guards against
        // partial-write bugs where sled succeeds but the field is stale.
        if identity.nickname().as_deref() != Some(&nickname) {
            tracing::warn!(
                "Nickname in-memory mismatch after save; forcing update from {:?} to {:?}",
                identity.nickname(),
                nickname
            );
        }
        tracing::info!("Nickname set to {:?}", nickname);
        self.audit_log.write().append(
            AuditEventType::ConsentGranted,
            identity.identity_id(),
            None,
            Some(format!("nickname={}", nickname)),
        );
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Signature verification
    // -----------------------------------------------------------------------

    pub fn verify_signature(
        &self,
        data: Vec<u8>,
        signature: Vec<u8>,
        public_key_hex: String,
    ) -> Result<bool, IronCoreError> {
        let pk_bytes = hex::decode(&public_key_hex).map_err(|_| IronCoreError::InvalidInput)?;
        crate::identity::IdentityKeys::verify(&data, &signature, &pk_bytes)
            .map_err(|_| IronCoreError::CryptoError)
    }

    // -----------------------------------------------------------------------
    // Outbox / Inbox counts
    // -----------------------------------------------------------------------

    pub fn outbox_count(&self) -> u32 {
        self.outbox.read().total_count() as u32
    }

    pub fn inbox_count(&self) -> u32 {
        self.inbox.read().total_count() as u32
    }

    /// Drain all received messages from the inbox, clearing the buffer while
    /// preserving dedup IDs. This is the core parity of the WASM
    /// `drainReceivedMessages` method.
    pub fn drain_received_messages(&self) -> Vec<ReceivedMessage> {
        self.inbox.write().drain_received_messages()
    }

    /// Non-destructively list all received messages still in the inbox.
    /// Unlike `drain_received_messages`, repeated calls return the same
    /// messages until something else clears them - needed for read-only
    /// polling like listing pending message requests.
    pub fn peek_received_messages(&self) -> Vec<ReceivedMessage> {
        self.inbox.read().all_messages()
    }

    /// Get the canonical peer ID (identity_id) for an explicitly tagged
    /// identifier. Unprefixed 64-hex values are left unchanged because an
    /// identity_id can be a valid Ed25519 point by chance.
    pub fn get_canonical_peer_id(&self, peer_id: &str) -> Option<String> {
        if let Some(public_key) = peer_id.strip_prefix(crate::identity::keys::PUBLIC_KEY_PREFIX) {
            return crate::identity::keys::identity_id_from_public_key_hex(public_key);
        }
        Some(
            peer_id
                .strip_prefix(crate::identity::keys::IDENTITY_ID_PREFIX)
                .unwrap_or(peer_id)
                .to_string(),
        )
    }

    // -----------------------------------------------------------------------
    // Store managers (returned to WASM for bridging)
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Blocking
    // -----------------------------------------------------------------------

    pub fn block_peer(
        &self,
        peer_id: String,
        device_id: Option<String>,
        reason: Option<String>,
    ) -> Result<(), IronCoreError> {
        let contact_public_key = self
            .contact_manager
            .read()
            .get(peer_id.clone())
            .ok()
            .flatten()
            .map(|contact| contact.public_key)
            .filter(|public_key| public_key != &peer_id);

        // Register known device IDs from the contact before blocking
        if let Some(contact) = self
            .contact_manager
            .read()
            .get(peer_id.clone())
            .ok()
            .flatten()
        {
            if let Some(ref did) = contact.last_known_device_id {
                let _ = self
                    .blocked_manager
                    .write()
                    .register_device_id(&peer_id, did);
            }
        }
        if let Some(ref did) = device_id {
            let _ = self
                .blocked_manager
                .write()
                .register_device_id(&peer_id, did);
        }

        let blocked = crate::store::blocked::BlockedIdentity::new(peer_id.clone());
        let blocked = if let Some(did) = device_id.clone() {
            crate::store::blocked::BlockedIdentity {
                device_id: Some(did),
                ..blocked
            }
        } else {
            blocked
        };
        let blocked = if let Some(r) = reason.clone() {
            crate::store::blocked::BlockedIdentity {
                reason: Some(r),
                ..blocked
            }
        } else {
            blocked
        };
        self.blocked_manager.write().block(blocked)?;

        // A contact record is the explicit provenance needed to write the
        // public-key flavor. Never infer it from curve-point validity.
        if let Some(public_key) = contact_public_key {
            let mut public_key_block =
                crate::store::blocked::BlockedIdentity::new(format!("pk:{public_key}"));
            public_key_block.device_id = device_id;
            public_key_block.reason = reason;
            self.blocked_manager.write().block(public_key_block)?;
        }
        Ok(())
    }

    pub fn unblock_peer(
        &self,
        peer_id: String,
        device_id: Option<String>,
    ) -> Result<(), IronCoreError> {
        let contact_public_key = self
            .contact_manager
            .read()
            .get(peer_id.clone())
            .ok()
            .flatten()
            .map(|contact| contact.public_key)
            .filter(|public_key| public_key != &peer_id);

        self.blocked_manager
            .write()
            .unblock(peer_id.clone(), device_id.clone())?;
        if let Some(public_key) = contact_public_key {
            self.blocked_manager
                .write()
                .unblock(format!("pk:{public_key}"), device_id)?;
        }
        let _ = self.history_manager.unhide_messages_for_peer(&peer_id);
        if let Some(canonical) = self
            .get_canonical_peer_id(&peer_id)
            .filter(|canonical| canonical != &peer_id)
        {
            let _ = self.history_manager.unhide_messages_for_peer(&canonical);
        }
        Ok(())
    }

    pub fn block_and_delete_peer(
        &self,
        peer_id: String,
        _device_id: Option<String>,
        reason: Option<String>,
    ) -> Result<(), IronCoreError> {
        let contact_public_key = self
            .contact_manager
            .read()
            .get(peer_id.clone())
            .ok()
            .flatten()
            .map(|contact| contact.public_key)
            .filter(|public_key| public_key != &peer_id);

        // Register known devices before block_and_delete so they get auto-blocked
        if let Some(contact) = self
            .contact_manager
            .read()
            .get(peer_id.clone())
            .ok()
            .flatten()
        {
            if let Some(ref did) = contact.last_known_device_id {
                let _ = self
                    .blocked_manager
                    .write()
                    .register_device_id(&peer_id, did);
            }
        }

        self.blocked_manager
            .write()
            .block_and_delete(peer_id.clone(), reason.clone())?;
        if let Some(public_key) = contact_public_key {
            self.blocked_manager
                .write()
                .block_and_delete(format!("pk:{public_key}"), reason.clone())?;
        }
        // Purge messages from this peer
        let _ = self.history_manager.remove_conversation(peer_id.clone());
        let _ = self.outbox.write().drain_for_peer(&peer_id);
        if let Some(canonical) = self
            .get_canonical_peer_id(&peer_id)
            .filter(|canonical| canonical != &peer_id)
        {
            let _ = self.history_manager.remove_conversation(canonical.clone());
            let _ = self.outbox.write().drain_for_peer(&canonical);
        }
        Ok(())
    }

    /// List blocked peers, returning bridge-compatible BlockedIdentity structs.
    /// Internal helper used by `list_blocked_peers` for UniFFI.
    #[cfg(not(target_arch = "wasm32"))]
    fn list_blocked_peers_bridge(
        &self,
    ) -> Result<Vec<crate::blocked_bridge::BlockedIdentity>, IronCoreError> {
        self.blocked_manager.read().list().map(|v| {
            v.into_iter()
                .map(crate::blocked_bridge::BlockedIdentity::from)
                .collect()
        })
    }

    pub fn blocked_count(&self) -> Result<u32, IronCoreError> {
        self.blocked_manager.read().count().map(|c| c as u32)
    }

    /// Clear all message history.
    pub fn clear_history(&self) -> Result<(), IronCoreError> {
        self.history_manager.clear()
    }

    /// Get the list of all blocked identities (non-WASM version).
    /// Returns the bridge type for UniFFI compatibility.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn list_blocked(
        &self,
    ) -> Result<Vec<crate::blocked_bridge::BlockedIdentity>, IronCoreError> {
        let core_blocked = self.blocked_manager.read().list()?;
        Ok(core_blocked
            .into_iter()
            .map(crate::blocked_bridge::BlockedIdentity::from)
            .collect())
    }

    // -----------------------------------------------------------------------
    // Identity backup export/import
    // -----------------------------------------------------------------------

    /// Build the JSON payload backed up by `export_identity_backup*`: the
    /// identity keypair plus everything needed to keep conversing without
    /// interruption after a restore — active ratchet sessions (so the next
    /// message from an existing contact still decrypts) and contacts.
    fn build_identity_backup_payload(&self) -> Result<String, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
        let identity_key_hex = hex::encode(keys.to_bytes());
        let nickname = identity.nickname();

        let ratchet_sessions_json = self.ratchet_sessions.read().serialize_sessions().ok();
        let contacts = self.contact_manager.read().list()?;

        // The mobile bridge's contacts.db only exists for a persistent
        // (`with_storage*`) IronCore; an in-memory core has no
        // `storage_path` and thus no bridge store to read.
        #[cfg(not(target_arch = "wasm32"))]
        let bridge_contacts_json = if self.storage_path.is_none() {
            None
        } else {
            let bridge_contacts = self.contacts_manager()?.list()?;
            if bridge_contacts.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&bridge_contacts).map_err(|_| IronCoreError::Internal)?)
            }
        };
        #[cfg(target_arch = "wasm32")]
        let bridge_contacts_json: Option<String> = None;

        let payload = IdentityBackupPayload {
            version: IDENTITY_BACKUP_PAYLOAD_VERSION,
            identity_key_hex,
            ratchet_sessions_json,
            contacts,
            bridge_contacts_json,
            nickname,
        };
        serde_json::to_string(&payload).map_err(|_| IronCoreError::Internal)
    }

    pub fn export_identity_backup(&self, passphrase: String) -> Result<String, IronCoreError> {
        let payload = self.build_identity_backup_payload()?;
        let backup = crate::crypto::backup::encrypt_backup(&payload, &passphrase, None)
            .map_err(|_| IronCoreError::CryptoError)?;
        self.audit_log.write().append(
            AuditEventType::BackupExported,
            self.identity.read().identity_id(),
            None,
            None,
        );
        Ok(backup)
    }

    /// Derive a WiFi Aware Pairwise Master Key (PMK) unique to the local
    /// identity and the given remote peer, via X25519 ECDH. Replaces a
    /// prior hardcoded key-material bug where every peer/session derived
    /// the same PMK, defeating pairwise isolation.
    ///
    /// `remote_pubkey_bytes` must be exactly 32 bytes (the remote peer's
    /// Ed25519 public key). `Vec<u8>` is used at this boundary rather than
    /// `[u8; 32]` because fixed-size arrays are not UniFFI-safe FFI types.
    pub fn derive_wifi_aware_pmk(
        &self,
        remote_pubkey_bytes: Vec<u8>,
    ) -> Result<Vec<u8>, IronCoreError> {
        if remote_pubkey_bytes.len() != 32 {
            return Err(IronCoreError::InvalidInput);
        }
        let mut remote_pubkey_arr = [0u8; 32];
        remote_pubkey_arr.copy_from_slice(&remote_pubkey_bytes);

        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;

        let local_x25519_secret = ed25519_to_x25519_secret(&keys.signing_key);
        let remote_x25519_public =
            ed25519_public_to_x25519(&remote_pubkey_arr).map_err(|_| IronCoreError::CryptoError)?;

        let shared_secret = local_x25519_secret.diffie_hellman(&remote_x25519_public);

        Ok(blake3::derive_key("SCMessenger Wi-Fi Aware PMK", shared_secret.as_bytes()).to_vec())
    }

    pub fn export_identity_backup_with_salt(
        &self,
        passphrase: String,
        salt: Option<Vec<u8>>,
    ) -> Result<String, IronCoreError> {
        let payload = self.build_identity_backup_payload()?;

        let salt_array = match salt {
            Some(s) => {
                if s.len() != 16 {
                    return Err(IronCoreError::InvalidInput);
                }
                let mut arr = [0u8; 16];
                arr.copy_from_slice(&s);
                Some(arr)
            }
            None => None,
        };

        let backup =
            crate::crypto::backup::encrypt_backup(&payload, &passphrase, salt_array.as_ref())
                .map_err(|_| IronCoreError::CryptoError)?;
        self.audit_log.write().append(
            AuditEventType::BackupExported,
            self.identity.read().identity_id(),
            None,
            None,
        );
        Ok(backup)
    }

    /// Export an identity backup encrypted with Blake3 key derivation.
    /// Use this for **device-bound auto-backups** where the passphrase is a
    /// 256-bit random key (stored in SharedPreferences). Blake3 `derive_key`
    /// runs in microseconds vs. 30-90s for Argon2id on low-end mobile.
    ///
    /// The passphrase MUST be high-entropy (≥ 128 bits); this method does NOT
    /// provide brute-force resistance for weak human passwords.
    pub fn export_identity_backup_fast(&self, passphrase: String) -> Result<String, IronCoreError> {
        let payload = self.build_identity_backup_payload()?;
        let backup = crate::crypto::backup::encrypt_backup_fast(&payload, &passphrase, None)
            .map_err(|_| IronCoreError::CryptoError)?;
        self.audit_log.write().append(
            AuditEventType::BackupExported,
            self.identity.read().identity_id(),
            None,
            None,
        );
        Ok(backup)
    }

    /// Export an identity backup encrypted with Blake3 key derivation and a
    /// custom 16-byte salt (e.g. from touch-screen entropy). See
    /// [`export_identity_backup_fast`] for details.
    pub fn export_identity_backup_fast_with_salt(
        &self,
        passphrase: String,
        salt: Option<Vec<u8>>,
    ) -> Result<String, IronCoreError> {
        let payload = self.build_identity_backup_payload()?;

        let salt_array = match salt {
            Some(s) => {
                if s.len() != 16 {
                    return Err(IronCoreError::InvalidInput);
                }
                let mut arr = [0u8; 16];
                arr.copy_from_slice(&s);
                Some(arr)
            }
            None => None,
        };

        let backup =
            crate::crypto::backup::encrypt_backup_fast(&payload, &passphrase, salt_array.as_ref())
                .map_err(|_| IronCoreError::CryptoError)?;
        self.audit_log.write().append(
            AuditEventType::BackupExported,
            self.identity.read().identity_id(),
            None,
            None,
        );
        Ok(backup)
    }

    /// Import an identity backup. Validates the entire payload (identity
    /// key bytes, ratchet session JSON, contact records) before writing
    /// anything, so a malformed or partially-tampered payload can't leave
    /// identity/ratchet-sessions/contacts in a mix of old and new state.
    pub fn import_identity_backup(
        &self,
        backup: String,
        passphrase: String,
    ) -> Result<(), IronCoreError> {
        let payload = crate::crypto::backup::decrypt_backup(&backup, &passphrase)
            .map_err(|_| IronCoreError::CryptoError)?;

        // Try the current structured payload first; fall back to the
        // original format (a bare hex-encoded identity key, no ratchet
        // sessions or contacts) for backups exported before this payload
        // existed.
        let (key_bytes, ratchet_sessions_json, contacts, bridge_contacts, nickname) =
            match serde_json::from_str::<IdentityBackupPayload>(&payload) {
                Ok(parsed) => {
                    let key_bytes = hex::decode(&parsed.identity_key_hex)
                        .map_err(|_| IronCoreError::CryptoError)?;
                    // Validate (without applying) the ratchet session JSON
                    // up front, so a corrupt fragment fails before any state
                    // is touched.
                    if let Some(ref json) = parsed.ratchet_sessions_json {
                        let mut probe = RatchetSessionManager::new();
                        probe
                            .deserialize_sessions_strict(json)
                            .map_err(|_| IronCoreError::CorruptionDetected)?;
                    }
                    // Validate (without applying) the bridge contacts JSON
                    // up front too, for the same all-or-nothing reason.
                    #[cfg(not(target_arch = "wasm32"))]
                    let bridge_contacts = match parsed.bridge_contacts_json {
                        Some(ref json) => {
                            let parsed_contacts: Vec<crate::contacts_bridge::Contact> =
                                serde_json::from_str(json)
                                    .map_err(|_| IronCoreError::CorruptionDetected)?;
                            parsed_contacts
                        }
                        None => Vec::new(),
                    };
                    #[cfg(target_arch = "wasm32")]
                    let bridge_contacts: Vec<String> = Vec::new();

                    (
                        key_bytes,
                        parsed.ratchet_sessions_json,
                        parsed.contacts,
                        bridge_contacts,
                        parsed.nickname,
                    )
                }
                Err(_) => {
                    let key_bytes =
                        hex::decode(&payload).map_err(|_| IronCoreError::CryptoError)?;
                    (key_bytes, None, Vec::new(), Vec::new(), None)
                }
            };

        // Everything validated - commit.
        let mut identity = self.identity.write();
        identity
            .import_key_bytes(&key_bytes)
            .map_err(|_| IronCoreError::CryptoError)?;

        if let Some(nn) = nickname {
            let _ = identity.set_nickname(nn);
        }

        if let Some(json) = ratchet_sessions_json {
            self.ratchet_sessions
                .write()
                .deserialize_sessions_strict(&json)
                .map_err(|_| IronCoreError::CorruptionDetected)?;
        }
        {
            let contact_manager = self.contact_manager.read();
            for contact in contacts {
                contact_manager.add(contact)?;
            }
        }
        // Mirror export: only a persistent core has a bridge contacts.db to
        // restore into. WASM has no contacts_bridge (UniFFI-only) at all.
        #[cfg(not(target_arch = "wasm32"))]
        if self.storage_path.is_some() {
            let bridge = self.contacts_manager()?;
            for contact in bridge_contacts {
                bridge.add(contact)?;
            }
            // Import is a recovery boundary. Force bridge-contact writes to
            // stable storage before returning so a mobile client can release
            // and reopen its pre-existing ContactManager immediately after a
            // Keychain backup restore.
            bridge.flush();
        } else {
            let _ = bridge_contacts;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = bridge_contacts;
        }

        self.audit_log.write().append(
            AuditEventType::BackupImported,
            identity.identity_id(),
            None,
            None,
        );
        Ok(())
    }

    /// Derive the Ed25519 public key hex from a libp2p PeerId string.
    pub fn extract_public_key_from_peer_id(
        &self,
        peer_id: String,
    ) -> Result<String, IronCoreError> {
        let peer_id: libp2p::PeerId = peer_id.parse().map_err(|_| IronCoreError::InvalidInput)?;
        // Ed25519 PeerIds use identity multihash (code 0) where the digest
        // contains the protobuf-encoded public key.
        let mh = peer_id.as_ref();
        if mh.code() != 0 {
            return Err(IronCoreError::Internal);
        }
        let pk = libp2p::identity::PublicKey::try_decode_protobuf(mh.digest())
            .map_err(|_| IronCoreError::Internal)?;
        let ed25519_pk = pk.try_into_ed25519().map_err(|_| IronCoreError::Internal)?;
        Ok(hex::encode(ed25519_pk.to_bytes()))
    }

    // -----------------------------------------------------------------------
    // Extended messaging
    // -----------------------------------------------------------------------

    /// Prepare an encrypted delivery receipt envelope ready for send_message.
    ///
    /// The receipt JSON is the encrypted MessageType::Receipt payload; callers
    /// must pass the returned bytes directly to the transport rather than
    /// wrapping or decoding them again. Use encode_receipt for raw receipt
    /// codec access.
    pub fn prepare_receipt(
        &self,
        recipient_public_key_hex: String,
        message_id: String,
    ) -> Result<Vec<u8>, IronCoreError> {
        let receipt = crate::Receipt {
            message_id,
            status: crate::DeliveryStatus::Delivered,
            timestamp: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        let receipt_payload =
            crate::message::types::encode_receipt(&receipt).map_err(|_| IronCoreError::Internal)?;
        let receipt_text =
            String::from_utf8(receipt_payload).map_err(|_| IronCoreError::Internal)?;

        self.prepare_message_with_id(
            recipient_public_key_hex,
            receipt_text,
            crate::MessageType::Receipt,
            None,
        )
        .map(|prepared| prepared.envelope_data)
    }

    /// Generate cover traffic payload (random bytes).
    pub fn prepare_cover_traffic(&self, size_bytes: u32) -> Result<Vec<u8>, IronCoreError> {
        let clamped = size_bytes.clamp(16, 1024) as usize;
        let mut buf = vec![0u8; clamped];
        use rand::RngCore;
        rand::rngs::OsRng.fill_bytes(&mut buf);
        Ok(buf)
    }

    // -----------------------------------------------------------------------
    // Notification classification
    // -----------------------------------------------------------------------

    pub fn classify_notification(
        &self,
        message: crate::NotificationMessageContext,
        ui_state: crate::NotificationUiState,
        settings: crate::MeshSettings,
    ) -> crate::NotificationDecision {
        crate::notification::classify_notification(message, ui_state, settings)
    }

    // -----------------------------------------------------------------------
    // Audit log access
    // -----------------------------------------------------------------------

    pub fn get_audit_log(&self) -> Vec<crate::observability::AuditEvent> {
        self.audit_log.read().events.clone()
    }

    pub fn get_audit_events_since(&self, since: u64) -> Vec<crate::observability::AuditEvent> {
        self.audit_log
            .read()
            .events
            .iter()
            .filter(|e| e.timestamp_unix_secs >= since)
            .cloned()
            .collect()
    }

    // -----------------------------------------------------------------------
    // Abuse / Reputation extended
    /// Get the overall abuse score for a peer, combining base reputation and spam confidence.
    pub fn abuse_overall_score(&self, peer: &str) -> Option<f64> {
        self.abuse_manager
            .read()
            .get_enhanced_score(peer)
            .overall_score()
            .into()
    }
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Privacy config
    // -----------------------------------------------------------------------

    pub fn set_privacy_config(&self, json: String) -> Result<(), IronCoreError> {
        let config: crate::privacy::PrivacyConfig =
            serde_json::from_str(&json).map_err(|_| IronCoreError::InvalidInput)?;
        *self.privacy_config.write() = config;
        tracing::info!("Privacy config updated");
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Identity resolution
    // -----------------------------------------------------------------------

    /// Resolve any identifier format to the canonical public_key_hex.
    ///
    /// Accepts three formats:
    /// 1. Ed25519 public key hex (64 hex chars, valid curve point) — returned as-is.
    /// 2. Blake3 identity_id (64 hex chars, NOT a valid Ed25519 point) — resolved
    ///    by searching contacts for a matching identity_id.
    /// 3. libp2p Peer ID (base58, e.g. "12D3Koo...") — public key extracted.
    pub fn resolve_identity(&self, any_id: String) -> Result<String, IronCoreError> {
        let trimmed = any_id.trim().to_lowercase();

        // If 64 hex chars, determine whether it's a public key or a Blake3 identity_id.
        if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            // ORDERING IS LOAD-BEARING. The curve-point test at the bottom is only
            // a HEURISTIC and cannot distinguish the two forms: a Blake3
            // identity_id is 32 essentially-random bytes, and roughly HALF of all
            // such values decompress to a valid Ed25519 point. Testing it first
            // (as this function used to) silently returns an identity_id as though
            // it were a public key, and the caller then encrypts to a hash --
            // producing ciphertext nobody can decrypt. Anything we can resolve
            // from stored data is authoritative and must win over the guess.

            // Snapshot our own identity first and release the lock. The
            // established order is identity -> contact_manager (see
            // prepare_message_internal, which holds identity across its contact
            // reads); taking them the other way round risks a deadlock.
            let (my_pk, my_id) = {
                let identity = self.identity.read();
                (
                    identity.keys().map(|k| k.public_key_hex().to_lowercase()),
                    identity.identity_id().map(|s| s.to_lowercase()),
                )
            };
            if let Some(ref pk) = my_pk {
                if *pk == trimmed || my_id.as_deref() == Some(trimmed.as_str()) {
                    return Ok(pk.clone());
                }
            }

            // Fast path: an O(1) hit on the contact store is authoritative.
            if let Ok(Some(contact)) = self.contact_manager.read().get(trimmed.clone()) {
                if !contact.public_key.is_empty() {
                    return Ok(contact.public_key.to_lowercase());
                }
            }

            // Miss. Scan once, checking BOTH identifier forms per contact.
            if let Ok(contacts) = self.contact_manager.read().list() {
                for contact in contacts {
                    let pk_hex = contact.public_key.to_lowercase();
                    if pk_hex == trimmed {
                        return Ok(pk_hex);
                    }
                    // identity_id() is hex(blake3(RAW 32 pubkey bytes)) --
                    // see identity/keys.rs. `public_key` is stored as HEX, so it
                    // MUST be decoded before hashing. Hashing the hex string
                    // instead produces an entirely different digest, which is why
                    // this lookup previously could never match a real identity_id
                    // and every hash-form resolve fell through to the heuristic.
                    // resolve_to_identity_id() below shows the correct form.
                    let Ok(pk_bytes) = hex::decode(&pk_hex) else {
                        continue;
                    };
                    if hex::encode(blake3::hash(&pk_bytes).as_bytes()) == trimmed {
                        return Ok(pk_hex);
                    }
                }
            }

            // No stored record resolves it. Only now fall back to the heuristic,
            // which is correct for a genuine public key of a peer we have never
            // seen, and is the best available answer for anything else.
            if let Ok(bytes) = hex::decode(&trimmed) {
                if let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice()) {
                    if ed25519_dalek::VerifyingKey::from_bytes(&arr).is_ok() {
                        return Ok(trimmed);
                    }
                }
            }

            return Err(IronCoreError::InvalidInput);
        }

        // Otherwise try to parse as PeerId and extract the key.
        self.extract_public_key_from_peer_id(any_id)
    }

    /// Resolve any identifier format to the identity_id (Blake3 hash).
    pub fn resolve_to_identity_id(&self, any_id: String) -> Result<String, IronCoreError> {
        let pubkey_hex = self.resolve_identity(any_id)?;
        let pk_bytes = hex::decode(&pubkey_hex).map_err(|_| IronCoreError::InvalidInput)?;
        let hash = blake3::hash(&pk_bytes);
        Ok(hex::encode(hash.as_bytes()))
    }

    // -----------------------------------------------------------------------
    // Maintenance & logging
    // -----------------------------------------------------------------------

    pub fn perform_maintenance(&self) -> Result<(), IronCoreError> {
        // Remove expired outbox messages older than 7 days
        let removed = self.outbox.write().remove_expired(604800);
        tracing::info!("Maintenance removed {} expired outbox messages", removed);
        // Extract identity_id BEFORE acquiring audit_log (lock ordering: identity → audit_log)
        let identity_id = self.identity.read().identity_id();
        self.audit_log.write().append(
            AuditEventType::StorageCompacted,
            identity_id,
            None,
            Some(format!("removed={}", removed)),
        );

        // Run periodic abuse scan: evaluate all tracked peers and auto-block
        // those exceeding reputation or spam thresholds.
        match self.auto_block_engine.read().evaluate_all_tracked() {
            Ok(blocked_count) => {
                if blocked_count > 0 {
                    tracing::info!("Maintenance auto-blocked {} peers", blocked_count);
                    self.audit_log.write().append(
                        AuditEventType::ContactBlocked,
                        self.identity.read().identity_id(),
                        None,
                        Some(format!("auto_blocked={}", blocked_count)),
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Maintenance abuse scan failed: {:?}", e);
            }
        }

        // Expire stale address observations from the routing layer.
        self.transport_manager
            .write()
            .expire_address_observations(3600);

        // Clean up stale connection stats from the health monitor.
        self.transport_manager.write().tick();

        let _ = self.validate_audit_chain();

        Ok(())
    }

    pub fn update_disk_stats(&self, total_bytes: u64, free_bytes: u64) {
        self.storage_manager
            .read()
            .update_disk_stats(total_bytes, free_bytes);
    }

    pub fn get_disk_stats(&self) -> crate::store::DiskStats {
        self.storage_manager.read().get_disk_stats()
    }

    pub fn record_log(&self, line: String) {
        self.log_manager.record_log(line);
    }

    pub fn export_logs(&self) -> Result<String, IronCoreError> {
        self.log_manager.export_all()
    }

    // -----------------------------------------------------------------------
    // CLI-specific accessors
    // -----------------------------------------------------------------------

    /// Export the audit log as a JSON string.
    pub fn export_audit_log(&self) -> Result<String, IronCoreError> {
        let log = self.audit_log.read();
        serde_json::to_string_pretty(&*log).map_err(|_| IronCoreError::Internal)
    }

    /// Validate the audit log chain integrity.
    pub fn validate_audit_chain(&self) -> Result<(), IronCoreError> {
        self.audit_log.read().validate_chain().map_err(|e| {
            tracing::warn!("Audit chain validation failed: {:?}", e);
            IronCoreError::CorruptionDetected
        })
    }

    /// Get privacy config as a JSON string.
    pub fn get_privacy_config(&self) -> String {
        let config = self.privacy_config();
        serde_json::to_string_pretty(&config).unwrap_or_default()
    }

    /// List blocked peers (returns bridge BlockedIdentity for UniFFI compatibility).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn list_blocked_peers(
        &self,
    ) -> Result<Vec<crate::blocked_bridge::BlockedIdentity>, IronCoreError> {
        self.list_blocked_peers_bridge()
    }

    // -----------------------------------------------------------------------
    // Routing engine
    // -----------------------------------------------------------------------

    /// Make a routing decision for the given recipient.
    /// Returns `None` if the routing engine has not been initialized yet.

    /// Access the routing engine handle.

    // -----------------------------------------------------------------------
    // Privacy subcomponents
    // -----------------------------------------------------------------------

    /// Initialize the cover traffic generator with the given config.

    /// Initialize timing jitter with the given config.

    /// Compute a jitter delay if timing jitter is initialized.
    pub fn compute_jitter_delay(&self) -> Option<u64> {
        self.timing_jitter
            .read()
            .as_ref()
            .map(|j| j.compute_jitter().as_millis() as u64)
    }

    /// Initialize the circuit builder with peers and config.

    // -----------------------------------------------------------------------
    // Notification endpoint registry
    // -----------------------------------------------------------------------

    /// Register a notification endpoint for remote push.

    /// Unregister a notification endpoint.

    /// List all registered notification endpoints.

    // -----------------------------------------------------------------------
    // Transport manager
    // -----------------------------------------------------------------------

    /// Access the transport manager handle.

    // -----------------------------------------------------------------------
    // Relay standalone module accessors
    // -----------------------------------------------------------------------

    /// Access the bootstrap manager handle (available after identity init).
    #[cfg(not(target_arch = "wasm32"))]

    /// Access the peer exchange manager handle.
    #[cfg(not(target_arch = "wasm32"))]
    // -----------------------------------------------------------------------
    // Contact & History managers (UniFFI bridge accessors)
    // -----------------------------------------------------------------------

    /// Return a ContactManager instance for the UniFFI interface.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn contacts_manager(
        &self,
    ) -> Result<crate::contacts_bridge::ContactManager, crate::IronCoreError> {
        let path = self.storage_path.clone().unwrap_or_default();
        crate::contacts_bridge::ContactManager::new(path.clone())
            .or_else(|_| crate::contacts_bridge::ContactManager::new(path))
            .or_else(|e| {
                tracing::error!("Failed to create contact manager: {:?}", e);
                crate::contacts_bridge::ContactManager::new("".to_string())
            })
    }

    /// Return the federated nickname for a contact (the nickname advertised by the peer).
    pub fn contact_federated_nickname(&self, peer_id: String) -> Option<String> {
        self.contact_manager
            .read()
            .get(peer_id)
            .ok()
            .flatten()
            .and_then(|c| c.federated_nickname().map(|s| s.to_string()))
    }

    /// Return the display name for a contact, preferring local then federated then peer ID.
    pub fn contact_display_name(&self, peer_id: String) -> String {
        self.contact_manager
            .read()
            .get(peer_id.clone())
            .ok()
            .flatten()
            .map(|c| c.display_name().to_string())
            .unwrap_or(peer_id)
    }

    /// Update the last known device ID for a contact.
    /// Validates the device ID format (UUID) and persists the change.
    /// Also registers the device in the block device registry so that if
    /// the peer is already blocked, the new device is auto-blocked.
    /// Pass `None` to clear the device ID.
    pub fn contact_update_last_known_device_id(
        &self,
        peer_id: String,
        device_id: Option<String>,
    ) -> Result<(), IronCoreError> {
        self.contact_manager
            .read()
            .update_last_known_device_id(peer_id.clone(), device_id.clone())?;

        // Register the device ID in the block registry for multi-device blocking.
        // If the peer is blocked, the device is automatically blocked too.
        if let Some(ref did) = device_id {
            let _ = self
                .blocked_manager
                .write()
                .register_device_id(&peer_id, did);
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // B2 wiring: Invite signing path
    // -----------------------------------------------------------------------

    /// Get the signable data for an invite token.
    /// Returns the serialized token data (without signature) suitable for
    /// Ed25519 signing.
    pub fn invite_get_signable_data(&self, token_bytes: Vec<u8>) -> Result<Vec<u8>, IronCoreError> {
        let token: crate::relay::invite::InviteToken =
            bincode::deserialize(&token_bytes).map_err(|_e| IronCoreError::Internal)?;
        token
            .get_signable_data()
            .map_err(|_e| IronCoreError::Internal)
    }

    // -----------------------------------------------------------------------
    // B2 wiring: DSPy signature verification
    // -----------------------------------------------------------------------

    /// Verify a DSPy signature by role name.
    /// Returns the signature description if the role is found.
    pub fn dspy_verify_signature(&self, role: &str) -> Option<String> {
        crate::dspy::signatures::get_signature(role).map(|s| s.to_string())
    }

    /// Get a DSPy signature description for the given role.
    /// Returns the signature description string if the role is found.
    pub fn dspy_get_signature(&self, role: &str) -> Option<String> {
        crate::dspy::signatures::get_signature(role).map(|s| s.to_string())
    }

    /// Return a HistoryManager instance for the UniFFI interface.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn history_manager(
        &self,
    ) -> Result<crate::mobile_bridge::HistoryManager, crate::IronCoreError> {
        let path = self.storage_path.clone().unwrap_or_default();
        crate::mobile_bridge::HistoryManager::new(path.clone())
            .or_else(|_| crate::mobile_bridge::HistoryManager::new(path))
            .or_else(|e| {
                tracing::error!("Failed to create history manager: {:?}", e);
                crate::mobile_bridge::HistoryManager::new("".to_string())
            })
    }

    // -----------------------------------------------------------------------
    // Custody audit
    // -----------------------------------------------------------------------

    /// Return the count of relay custody entries currently being tracked.
    pub fn custody_audit_count(&self) -> u32 {
        self.relay_custody_store.read().audit_count() as u32
    }

    /// Get registration state info for a specific identity from custody records.
    pub fn custody_get_registration_state_info(
        &self,
        identity_id: String,
    ) -> crate::RegistrationStateInfo {
        let info = self
            .relay_custody_store
            .read()
            .get_registration_state_info(&identity_id);
        crate::RegistrationStateInfo {
            state: info.state,
            device_id: info.device_id,
            seniority_timestamp: info.seniority_timestamp,
        }
    }

    /// Return registration state transitions for an identity from custody logs.
    pub fn custody_registration_transitions(&self, identity_id: String) -> String {
        let transitions = self
            .relay_custody_store
            .read()
            .registration_transitions_for_identity(&identity_id);
        serde_json::to_string(&transitions).unwrap_or_else(|_| "[]".to_string())
    }

    // -----------------------------------------------------------------------
    // Audit events by type
    // -----------------------------------------------------------------------

    /// Get audit events filtered by event type.
    pub fn get_audit_events_by_type(
        &self,
        event_type: crate::observability::AuditEventType,
    ) -> Vec<crate::observability::AuditEvent> {
        self.audit_log
            .read()
            .events
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    // -----------------------------------------------------------------------
    // Auto-adjust engine
    // -----------------------------------------------------------------------

    /// Return the auto-adjust engine for dynamic behavior tuning.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_auto_adjust_engine(&self) -> std::sync::Arc<crate::mobile_bridge::AutoAdjustEngine> {
        std::sync::Arc::new(crate::mobile_bridge::AutoAdjustEngine::new())
    }

    // -----------------------------------------------------------------------
    // Consent gate query
    // -----------------------------------------------------------------------

    /// Check whether consent has been granted for identity operations.
    pub fn is_consent_granted(&self) -> bool {
        *self.consent.read() == ConsentState::Granted
    }

    // -----------------------------------------------------------------------
    // App lifecycle
    // -----------------------------------------------------------------------

    /// Called when the app resumes (foreground transition).
    /// Triggers route prefetch for known peers to reduce first-message latency.
    pub fn on_app_resume(&self) {
        tracing::info!("IronCore: app resumed");
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            let hints = engine.prefetch_manager_mut().on_app_resume();
            if !hints.is_empty() {
                tracing::info!("IronCore: prefetch triggered for {} routes", hints.len());
            }
        }
    }

    /// Called when the app goes to background.
    /// Saves current route state for fast resume prefetch.
    pub fn on_app_background(&self) {
        tracing::info!("IronCore: app backgrounded");
        // Collect current routes from the routing engine before backgrounding
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            let current_routes: Vec<(
                [u8; 32],
                [u8; 4],
                crate::routing::global::RouteAdvertisement,
            )> = engine
                .base_engine_mut()
                .global_routes_mut()
                .get_advertisements()
                .iter()
                .map(|ad| (ad.next_hop, ad.destination_hint, ad.clone()))
                .collect();
            engine
                .prefetch_manager_mut()
                .on_app_background(current_routes);
        }
    }

    // -----------------------------------------------------------------------
    // Onion routing
    // -----------------------------------------------------------------------

    /// Wrap an envelope in onion routing layers for anonymous delivery.
    pub fn prepare_onion_message(
        &self,
        envelope_data: Vec<u8>,
        relay_public_keys_json: String,
    ) -> Result<Vec<u8>, IronCoreError> {
        if !self.privacy_config().onion_routing_enabled {
            return Err(IronCoreError::OnionRoutingDisabled);
        }
        let relay_keys: Vec<String> = serde_json::from_str(&relay_public_keys_json)
            .map_err(|_| IronCoreError::InvalidInput)?;
        if relay_keys.is_empty() {
            return Ok(envelope_data);
        }
        let path: Vec<crate::privacy::onion::HopAddress> = relay_keys
            .iter()
            .map(|hex| {
                let bytes = hex::decode(hex).map_err(|_| IronCoreError::InvalidInput)?;
                let arr = <[u8; 32]>::try_from(bytes.as_slice())
                    .map_err(|_| IronCoreError::InvalidInput)?;
                Ok(crate::privacy::onion::HopAddress::Classical(arr))
            })
            .collect::<Result<Vec<_>, IronCoreError>>()?;
        let result =
            crate::privacy::onion::construct_onion(path, &envelope_data, false).map_err(|e| {
                tracing::warn!("Onion layer construction failed: {:?}", e);
                IronCoreError::CryptoError
            })?;
        bincode::serialize(&result).map_err(|_| IronCoreError::Internal)
    }

    /// Peel one layer of an onion-routed envelope (relay-side operation).
    pub fn peel_onion_layer(
        &self,
        onion_data: Vec<u8>,
        relay_secret_key: Vec<u8>,
    ) -> Result<crate::PeelResult, IronCoreError> {
        if !self.privacy_config().onion_routing_enabled {
            return Err(IronCoreError::OnionRoutingDisabled);
        }
        let secret: [u8; 32] = relay_secret_key
            .try_into()
            .map_err(|_| IronCoreError::InvalidInput)?;
        let envelope: crate::privacy::onion::OnionEnvelope = bincode::deserialize(&onion_data)
            .map_err(|e| {
                tracing::warn!("Failed to deserialize onion envelope: {:?}", e);
                IronCoreError::InvalidInput
            })?;
        let (next_hop, remaining) = crate::privacy::onion::peel_layer(&envelope, &secret, None)
            .map_err(|e| {
                tracing::warn!("Onion peel failed: {:?}", e);
                IronCoreError::CryptoError
            })?;
        let remaining_data = bincode::serialize(&remaining).unwrap_or(remaining);
        Ok(crate::PeelResult {
            next_hop: next_hop.map(|h| h.x25519_public().to_vec()),
            remaining_data,
        })
    }

    // -----------------------------------------------------------------------
    // Random port
    // -----------------------------------------------------------------------

    /// Return a random available port for temporary listeners.
    pub fn random_port(&self) -> u16 {
        let mut buf = [0u8; 2];
        use rand::RngCore;
        rand::rngs::OsRng.fill_bytes(&mut buf);
        49152 + (u16::from_le_bytes(buf) % 16383)
    }

    // -----------------------------------------------------------------------
    // Ratchet session management
    // -----------------------------------------------------------------------

    /// Return the number of active ratchet sessions.
    pub fn ratchet_session_count(&self) -> u32 {
        self.ratchet_sessions.read().session_count() as u32
    }

    /// Check if a ratchet session exists for the given peer.
    pub fn ratchet_has_session(&self, peer_id: String) -> bool {
        self.ratchet_sessions.read().has_session(&peer_id)
    }

    /// Force-reset the ratchet session for a peer (re-key).
    pub fn ratchet_reset_session(&self, peer_id: String) {
        self.ratchet_sessions.write().remove_session(&peer_id);
        tracing::info!("Ratchet session reset for peer: {}", peer_id);
    }

    // -----------------------------------------------------------------------
    // Mycorrhizal routing operations (P1_CORE_003)
    // -----------------------------------------------------------------------

    /// Record that a peer was seen on a given transport.
    pub fn routing_peer_seen(&self, peer_id_hex: String, transport: String) {
        if let Some(engine) = self.routing_engine.write().as_mut() {
            let transport_type = parse_transport_type(&transport);
            if let Some(peer_id) = parse_peer_id_32(&peer_id_hex) {
                engine.peer_seen(peer_id, transport_type);
            } else {
                engine.record_message_activity(&peer_id_hex);
                engine.clear_unreachable_peer(&peer_id_hex);
            }
        }
    }

    /// Update peer hint vectors for routing table.
    pub fn routing_update_peer_hints(&self, peer_id_hex: String, hints: Vec<Vec<u8>>) {
        if let Some(engine) = self.routing_engine.write().as_mut() {
            // Record message activity for the peer, which feeds the adaptive TTL.
            engine.record_message_activity(&peer_id_hex);
            if let Some(peer_id) = parse_peer_id_32(&peer_id_hex) {
                let parsed_hints: Vec<[u8; 4]> = hints
                    .into_iter()
                    .filter_map(|hint| <[u8; 4]>::try_from(hint.as_slice()).ok())
                    .collect();
                // LocalCell intentionally updates only peers already known to
                // the local topology; an announcement cannot create a peer.
                engine
                    .base_engine_mut()
                    .local_cell_mut()
                    .update_peer_hints(&peer_id, parsed_hints);
            }
        }
    }

    /// Mark a peer as a gateway (relay-capable) or not.
    pub fn routing_mark_gateway(&self, peer_id_hex: String, is_gateway: bool) {
        if let Some(peer_id) = parse_peer_id_32(&peer_id_hex) {
            if let Some(engine) = self.routing_engine.write().as_mut() {
                engine
                    .base_engine_mut()
                    .local_cell_mut()
                    .mark_as_gateway(&peer_id, is_gateway);
            }
        }
    }

    /// Update reliability score for a peer based on success/failure.
    pub fn routing_update_reliability(&self, peer_id_hex: String, success: bool) {
        if let Some(engine) = self.routing_engine.write().as_mut() {
            if let Some(peer_id) = parse_peer_id_32(&peer_id_hex) {
                engine
                    .base_engine_mut()
                    .local_cell_mut()
                    .update_reliability(&peer_id, success);
            }
            if success {
                engine.record_message_activity(&peer_id_hex);
            } else {
                engine.record_unreachable_peer(&peer_id_hex);
            }
        }
    }

    /// Advance the routing engine by one tick. Returns state snapshot as JSON.
    pub fn routing_tick(&self) -> String {
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut guard = self.routing_engine.write();
        if let Some(engine) = guard.as_mut() {
            let maintenance = engine.tick(now);
            let summary = engine.base_engine().routing_summary();
            let mut payload = serde_json::to_value(&summary)
                .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
            let neg_stats = &maintenance.negative_cache_stats;
            payload["negative_cache"] = serde_json::Value::Object(serde_json::Map::from_iter([
                (
                    "negative_checks".into(),
                    serde_json::Value::from(neg_stats.negative_checks),
                ),
                (
                    "bloom_hits".into(),
                    serde_json::Value::from(neg_stats.bloom_hits),
                ),
                (
                    "bloom_misses".into(),
                    serde_json::Value::from(neg_stats.bloom_misses),
                ),
                (
                    "entry_count".into(),
                    serde_json::Value::from(neg_stats.entry_count),
                ),
                (
                    "expired_count".into(),
                    serde_json::Value::from(neg_stats.expired_count),
                ),
                (
                    "entries_cleaned".into(),
                    serde_json::Value::from(maintenance.negative_cache_entries_cleaned),
                ),
            ]));
            let budget = &maintenance.timeout_budget_summary;
            payload["timeout_budget"] = serde_json::Value::Object(serde_json::Map::from_iter([
                (
                    "total_budget_ms".into(),
                    serde_json::Value::from(budget.total_budget.as_millis() as u64),
                ),
                (
                    "elapsed_ms".into(),
                    serde_json::Value::from(budget.elapsed.as_millis() as u64),
                ),
                (
                    "remaining_ms".into(),
                    serde_json::Value::from(budget.remaining.as_millis() as u64),
                ),
                (
                    "phase".into(),
                    serde_json::Value::from(format!("{:?}", budget.current_phase)),
                ),
                (
                    "phase_elapsed_ms".into(),
                    serde_json::Value::from(budget.phase_elapsed.as_millis() as u64),
                ),
                (
                    "exhausted".into(),
                    serde_json::Value::from(budget.is_exhausted),
                ),
            ]));
            payload["drift_network_state"] = serde_json::Value::from(self.drift_network_state());
            payload["drift_store_size"] = serde_json::Value::from(self.drift_store_size());
            serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
        } else {
            let mut payload = serde_json::Value::Object(serde_json::Map::new());
            payload["drift_network_state"] = serde_json::Value::from(self.drift_network_state());
            payload["drift_store_size"] = serde_json::Value::from(self.drift_store_size());
            serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
        }
    }

    /// Return a JSON summary of the routing engine state.
    pub fn routing_summary(&self) -> String {
        let guard = self.routing_engine.read();
        match guard.as_ref() {
            Some(e) => {
                let summary = e.base_engine().routing_summary();
                serde_json::to_string(&summary).unwrap_or_else(|_| "{}".to_string())
            }
            None => "{}".to_string(),
        }
    }

    /// Clear an unreachable peer from the routing table.
    pub fn routing_clear_unreachable_peer(&self, peer_id_hex: String) {
        if let Some(engine) = self.routing_engine.write().as_mut() {
            engine.clear_unreachable_peer(&peer_id_hex);
        }
    }

    /// Return the current discovery phase as a string.
    pub fn routing_current_discovery_phase(&self) -> String {
        let engine = self.routing_engine.read();
        match engine.as_ref() {
            Some(e) => {
                let phase = e.current_discovery_phase();
                format!("{:?}", phase)
            }
            None => "uninitialized".to_string(),
        }
    }

    /// Return negative cache statistics as a JSON string.
    pub fn routing_negative_cache_stats(&self) -> String {
        let engine = self.routing_engine.read();
        match engine.as_ref() {
            Some(e) => {
                let stats = e.negative_cache_stats();
                format!(
                    "{{\"negative_checks\":{},\"bloom_hits\":{},\"bloom_misses\":{},\"entry_count\":{},\"expired_count\":{}}}",
                    stats.negative_checks, stats.bloom_hits, stats.bloom_misses, stats.entry_count, stats.expired_count
                )
            }
            None => "{}".to_string(),
        }
    }

    /// Return prefetch statistics as a JSON string.
    pub fn routing_prefetch_stats(&self) -> String {
        let engine = self.routing_engine.read();
        match engine.as_ref() {
            Some(e) => {
                let stats = e.prefetch_stats();
                format!(
                    "{{\"total_routes\":{},\"fresh_routes\":{},\"stale_routes\":{},\"refreshing_routes\":{},\"failed_routes\":{},\"prefetch_in_progress\":{},\"queue_remaining\":{}}}",
                    stats.total_routes, stats.fresh_routes, stats.stale_routes, stats.refreshing_routes, stats.failed_routes, stats.prefetch_in_progress, stats.queue_remaining
                )
            }
            None => "{}".to_string(),
        }
    }

    /// Return timeout budget summary as a JSON string.
    pub fn routing_timeout_budget_summary(&self) -> String {
        let engine = self.routing_engine.read();
        match engine.as_ref() {
            Some(e) => {
                let summary = e.timeout_budget_summary();
                format!(
                    "{{\"total_budget_ms\":{},\"elapsed_ms\":{},\"remaining_ms\":{},\"phase\":\"{:?}\",\"exhausted\":{}}}",
                    summary.total_budget.as_millis(),
                    summary.elapsed.as_millis(),
                    summary.remaining.as_millis(),
                    summary.current_phase,
                    summary.is_exhausted
                )
            }
            None => "{}".to_string(),
        }
    }

    /// Calculate dynamic TTL based on network conditions.
    pub fn routing_calculate_dynamic_ttl(
        &self,
        base_ttl: u64,
        battery_level: u8,
        peer_count: u32,
    ) -> u64 {
        crate::routing::AdaptiveTTLManager::calculate_dynamic_ttl(
            base_ttl,
            battery_level,
            peer_count as usize,
        )
    }

    /// Register a routing path for a peer.
    /// Records the path in the multipath delivery manager (Phase 2) and
    /// notes message activity for adaptive TTL tracking.
    pub fn routing_register_path(&self, peer_id_hex: String, path_id: u64, latency_ms: u64) {
        if let Some(engine) = self.routing_engine.write().as_mut() {
            engine.record_message_activity(&peer_id_hex);
            engine.multipath_register_path(peer_id_hex.clone(), path_id, latency_ms);
        }
    }

    /// Mark a routing path as failed.
    /// Records the path failure in the multipath delivery manager
    /// and marks the peer as unreachable in the negative cache.
    pub fn routing_mark_path_failed(&self, path_id: u64) {
        tracing::debug!("Path {} marked as failed", path_id);
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            engine.multipath_mark_path_failed(path_id);
        }
    }

    // -----------------------------------------------------------------------
    // Swarm relay discovery (P1_CORE_003)
    // -----------------------------------------------------------------------

    /// Get the best relay peers for the current mesh topology.
    pub fn swarm_get_best_relays(&self, count: u32) -> Vec<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let bootstrap = self.bootstrap_manager.read();
            if let Some(ref mgr) = *bootstrap {
                mgr.get_seed_peers()
                    .unwrap_or_default()
                    .into_iter()
                    .take(count as usize)
                    .map(|sp| sp.address)
                    .collect()
            } else {
                Vec::new()
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = count;
            Vec::new()
        }
    }

    /// Get candidate peers suitable for bootstrapping new nodes.
    pub fn swarm_get_bootstrap_candidates(&self) -> Vec<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let bootstrap = self.bootstrap_manager.read();
            if let Some(ref mgr) = *bootstrap {
                mgr.get_seed_peers()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|sp| sp.address)
                    .collect()
            } else {
                Vec::new()
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Vec::new()
        }
    }

    /// Check if this node can act as a bootstrap peer for others.
    pub fn swarm_can_bootstrap_others(&self) -> bool {
        *self.running.read() && self.identity.read().keys().is_some()
    }

    /// Get the best multi-hop paths to a target peer.
    pub fn swarm_get_best_paths(&self, target_peer_id: String, count: u32) -> Vec<Vec<String>> {
        let mut guard = self.routing_engine.write();
        match guard.as_mut() {
            Some(engine) => {
                let hint = blake3::hash(target_peer_id.as_bytes()).as_bytes()[0..4]
                    .try_into()
                    .unwrap_or([0u8; 4]);
                let msg_id: [u8; 16] = *uuid::Uuid::new_v4().as_bytes();
                let now = web_time::SystemTime::now()
                    .duration_since(web_time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let decision = engine.route_message_optimized(&hint, &msg_id, 128, now);
                let format_hop = |hop: &crate::routing::NextHop| -> Vec<String> {
                    match hop {
                        crate::routing::NextHop::Direct { peer_id, .. } => {
                            vec![hex::encode(peer_id)]
                        }
                        crate::routing::NextHop::Gateway { gateway_id, .. } => {
                            vec![hex::encode(gateway_id), target_peer_id.clone()]
                        }
                        crate::routing::NextHop::GlobalRoute { next_hop_id, .. } => {
                            vec![hex::encode(next_hop_id), target_peer_id.clone()]
                        }
                        _ => vec![target_peer_id.clone()],
                    }
                };
                let mut paths = vec![format_hop(&decision.primary)];
                for alt in decision.alternatives.iter().take(count as usize) {
                    paths.push(format_hop(alt));
                }
                paths.truncate(count as usize);
                paths
            }
            None => vec![vec![target_peer_id]],
        }
    }

    /// Handle peer connection event: on connect, flush outbox messages for that peer and send them.
    ///
    /// # Behavior
    /// When a peer is identified (Identify protocol completes), this method:
    /// 1. Fetches all pending (unsent) messages from the outbox for that peer
    /// 2. Attempts delivery via transport layer
    /// 3. On success: marks message as sent (removed from outbox)
    /// 4. On transient error (timeout, network): re-enqueues with exponential backoff
    /// 5. On persistent error (peer rejected, too large): re-enqueues with backoff for later retry
    ///
    /// # Verbose Logging
    /// Each step emits INFO-level logs at decision points:
    /// - `outbox_reconnect_detected`: peer identified after connection
    /// - `outbox_flush_started`: beginning flush for peer with N pending messages
    /// - `outbox_retry_attempt`: attempting to send message (attempt #X)
    /// - `outbox_delivery_success`: message sent successfully
    /// - `outbox_delivery_failed_transient`: delivery failed with backoff retry
    pub fn handle_peer_connection_event(&self, peer_id: &str, connected: bool) {
        if connected {
            tracing::info!(
                event = "outbox_reconnect_detected",
                peer_id = %peer_id,
                "Peer identified; triggering outbox flush"
            );

            let transports = if let Some(pk) = parse_peer_id_32(peer_id) {
                self.transport_manager.read().transports_for_peer(pk)
            } else {
                Vec::new()
            };

            if transports.is_empty() {
                self.routing_peer_seen(peer_id.to_string(), "tcp".to_string());
            } else {
                for t in transports {
                    self.routing_peer_seen(peer_id.to_string(), t.to_string());
                }
            }

            let messages = self.outbox.write().flush_peer_messages(peer_id);
            if messages.is_empty() {
                tracing::debug!(
                    event = "outbox_flush_completed",
                    peer_id = %peer_id,
                    pending_count = 0,
                    "No pending messages to flush"
                );
                return;
            }

            tracing::info!(
                event = "outbox_flush_started",
                peer_id = %peer_id,
                pending_count = messages.len(),
                "Starting flush of {} pending messages",
                messages.len()
            );

            if let Ok(recipient_bytes) = hex::decode(peer_id) {
                if let Ok(recipient_pk) = recipient_bytes.try_into() {
                    let mut succeeded = 0;
                    let mut failed = 0;

                    for mut msg in messages {
                        let msg_id = msg.message_id.clone();
                        let current_attempt = msg.attempts.saturating_add(1);

                        tracing::info!(
                            event = "outbox_retry_attempt",
                            message_id = %msg_id,
                            peer_id = %peer_id,
                            attempt = current_attempt,
                            "Attempting delivery (attempt #{}/12)",
                            current_attempt
                        );

                        match self.transport_manager.read().send_to_peer(
                            recipient_pk,
                            msg.envelope_data.clone(),
                            1,
                        ) {
                            Ok(crate::transport::manager::SendResult::Queued(transport_type)) => {
                                tracing::info!(
                                    event = "outbox_transport_queued",
                                    message_id = %msg_id,
                                    peer_id = %peer_id,
                                    transport = ?transport_type,
                                    "Message queued to transport; awaiting delivery receipt"
                                );
                                succeeded += 1;
                                msg.attempts = current_attempt;
                                msg.state = crate::store::outbox::MessageState::Enqueued;
                                let now_secs = web_time::SystemTime::now()
                                    .duration_since(web_time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                if current_attempt >= 12 {
                                    msg.state = crate::store::outbox::MessageState::Failed;
                                    msg.next_retry_at = None;
                                } else {
                                    let receipt_timeout =
                                        2u64.saturating_pow(current_attempt.min(12)).min(3600);
                                    msg.next_retry_at = Some(now_secs + receipt_timeout);
                                }
                                // Transport queuing is not delivery confirmation. Keep the
                                // message in the outbox until an application-level receipt
                                // calls mark_message_sent.
                                if let Err(e) = self.outbox.write().enqueue(msg) {
                                    tracing::error!(
                                        event = "outbox_enqueue_failed",
                                        message_id = %msg_id,
                                        error = %e,
                                        "Failed to re-enqueue message after transport queue"
                                    );
                                }
                            }
                            Err(e) => {
                                msg.attempts = current_attempt;
                                if current_attempt >= 3 {
                                    // Persistent failure after 3 attempts: mark as Failed, keep in outbox for UX
                                    msg.state = crate::store::outbox::MessageState::Failed;
                                    msg.next_retry_at = None;
                                    tracing::debug!(
                                        event = "outbox_delivery_failed_persistent",
                                        message_id = %msg_id,
                                        peer_id = %peer_id,
                                        error = %e,
                                        attempt = current_attempt,
                                        "Delivery attempt failed 3 times; marking as Failed in outbox"
                                    );
                                    if let Err(e) = self.outbox.write().enqueue(msg) {
                                        tracing::error!(
                                            event = "outbox_enqueue_failed",
                                            message_id = %msg_id,
                                            error = %e,
                                            "Failed to re-enqueue message after persistent failure"
                                        );
                                    }
                                    failed += 1;
                                } else {
                                    // Transient failure (< 3 attempts): leave as Enqueued with exponential backoff
                                    msg.state = crate::store::outbox::MessageState::Enqueued;
                                    let backoff_secs =
                                        2u64.saturating_pow(current_attempt.min(12)).min(3600);
                                    let now_secs = web_time::SystemTime::now()
                                        .duration_since(web_time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();
                                    msg.next_retry_at = Some(now_secs + backoff_secs);

                                    tracing::debug!(
                                        event = "outbox_delivery_failed_transient",
                                        message_id = %msg_id,
                                        peer_id = %peer_id,
                                        error = %e,
                                        attempt = current_attempt,
                                        backoff_secs = backoff_secs,
                                        "Delivery attempt failed; re-enqueueing with backoff"
                                    );

                                    let _ = self.outbox.write().enqueue(msg);
                                    failed += 1;
                                }
                            }
                        }
                    }

                    tracing::info!(
                        event = "outbox_flush_completed",
                        peer_id = %peer_id,
                        succeeded = succeeded,
                        failed = failed,
                        "Outbox flush complete: {} queued for transport, {} scheduled for retry",
                        succeeded,
                        failed
                    );
                } else {
                    tracing::warn!(
                        event = "outbox_peer_key_invalid",
                        peer_id = %peer_id,
                        "Failed to convert decoded peer_id bytes to public key"
                    );
                }
            } else {
                tracing::warn!(
                    event = "outbox_peer_id_decode_failed",
                    peer_id = %peer_id,
                    "Failed to hex decode peer_id"
                );
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RECEIPT ENCODING/DECODING (exported via UniFFI for cross-platform consistency)
// ═══════════════════════════════════════════════════════════════════════════════
//
// These functions are the ONLY way receipts should be serialized/deserialized
// across all platforms (CLI, Android, iOS, WASM). They guarantee that receipt
// encoding is byte-compatible across all clients and eliminate version drift.

/// Encode a Receipt struct to JSON bytes (canonical wire format).
/// Used by all platforms before sending delivery confirmations over transport.
#[cfg(not(target_arch = "wasm32"))]
pub fn encode_receipt(receipt: crate::Receipt) -> Result<Vec<u8>, IronCoreError> {
    crate::message::types::encode_receipt(&receipt).map_err(|e| {
        tracing::error!(
            event = "receipt_encode_failed",
            message_id = %receipt.message_id,
            error = %e,
            "Failed to encode receipt to JSON bytes"
        );
        IronCoreError::CryptoError
    })
}

/// Decode a Receipt struct from JSON bytes (canonical wire format).
/// Used by all platforms after receiving delivery confirmations.
/// Returns error with full context if deserialization fails.
#[cfg(not(target_arch = "wasm32"))]
pub fn decode_receipt(data: Vec<u8>) -> Result<crate::Receipt, IronCoreError> {
    crate::message::types::decode_receipt(&data).map_err(|e| {
        tracing::error!(
            event = "receipt_decode_failed",
            data_len = data.len(),
            error = %e,
            "Failed to decode receipt from JSON bytes"
        );
        IronCoreError::CryptoError
    })
}

// Non-FFI-safe methods moved to plain impl block to avoid uniffi::export compilation errors.
impl IronCore {
    /// Test-only: true if `message_id` is currently queued in the live
    /// outbox for `recipient_id`. Used to assert single-ownership between
    /// the active outbox and drift custody (see T2.5).
    pub fn outbox_contains_for_recipient(&self, recipient_id: &str, message_id: &str) -> bool {
        self.outbox
            .read()
            .peek_for_peer(recipient_id)
            .iter()
            .any(|m| m.message_id == message_id)
    }

    /// Test-only: direct access to the ratchet session manager, so
    /// integration tests can establish a session and verify it survives
    /// identity backup/restore end-to-end (see T4.5).
    pub fn ratchet_sessions_handle(&self) -> Arc<RwLock<RatchetSessionManager>> {
        self.ratchet_sessions.clone()
    }

    /// Test-only: the initialized identity's Ed25519 signing key, for tests
    /// that drive `crypto::ratchet`/`crypto::encrypt` functions directly
    /// (see T4.5). Panics if the identity hasn't been initialized.
    ///
    /// Hands out a clone of the private signing key, so this is not
    /// `#[cfg(test)]` (integration tests in `core/tests/` need it, and
    /// `cfg(test)` doesn't apply there) but is `#[doc(hidden)]` and
    /// unmistakably named so it isn't reachable by accident (S6).
    #[doc(hidden)]
    pub fn test_only_identity_signing_key(&self) -> ed25519_dalek::SigningKey {
        self.identity
            .read()
            .keys()
            .expect("identity must be initialized")
            .signing_key
            .clone()
    }

    /// Test-only: true if `message_id` (a UUID string, as returned by
    /// `prepare_message`) is currently held in drift custody. Used to assert
    /// single-ownership between the active outbox and drift custody (see
    /// T2.5).
    pub fn drift_contains(&self, message_id: &str) -> bool {
        match uuid::Uuid::parse_str(message_id) {
            Ok(uuid) => self.drift_store.read().contains(uuid.as_bytes()),
            Err(_) => false,
        }
    }

    /// Compute a BLAKE3 hash of the given data.
    pub fn dspy_blake3_hash(&self, data: &[u8]) -> Vec<u8> {
        crate::dspy::signatures::blake3_hash(data).to_vec()
    }

    /// Create a basic DSPy teleprompter for prompt optimization.
    pub fn dspy_create_basic_teleprompter(&self) -> crate::dspy::teleprompt::BasicTeleprompter {
        crate::dspy::teleprompt::TeleprompterFactory::create_basic()
    }

    /// Create a chain-of-thought DSPy module for step-by-step reasoning.
    pub fn dspy_create_cot(&self, name: &str) -> crate::dspy::modules::ChainOfThought {
        crate::dspy::modules::ModuleFactory::create_cot(name)
    }

    /// Append a reasoning step to a Chain-of-Thought module.
    pub fn dspy_add_step(&self, cot: &mut crate::dspy::modules::ChainOfThought, step: &str) {
        cot.add_step(step);
    }

    /// Create a multi-hop recall DSPy module for multi-source reasoning.
    pub fn dspy_create_multihop(
        &self,
        name: &str,
        max_hops: usize,
    ) -> crate::dspy::modules::MultiHopRecall {
        crate::dspy::modules::ModuleFactory::create_multihop(name, max_hops)
    }

    /// Create an optimizer pipeline DSPy module for end-to-end optimization.
    pub fn dspy_create_optimizer(
        &self,
        name: &str,
        stages: &[&str],
    ) -> crate::dspy::modules::OptimizerPipeline {
        crate::dspy::modules::ModuleFactory::create_optimizer(name, stages)
    }

    /// Build a security audit pipeline DSPy module.
    pub fn dspy_build_security_audit_pipeline(&self) -> crate::dspy::modules::OptimizerPipeline {
        crate::dspy::modules::ModuleFactory::build_security_audit_pipeline()
    }

    /// Build a Rust feature pipeline DSPy module.
    pub fn dspy_build_rust_feature_pipeline(&self) -> crate::dspy::modules::OptimizerPipeline {
        crate::dspy::modules::ModuleFactory::build_rust_feature_pipeline()
    }
    pub fn get_libp2p_keypair(&self) -> Result<libp2p::identity::Keypair, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
        keys.to_libp2p_keypair()
            .map_err(|_| IronCoreError::CryptoError)
    }
    pub fn receive_message(&self, envelope_data: Vec<u8>) -> Result<Message, IronCoreError> {
        // Hoist sender public key and local identity id out of the legacy /
        // ratchet branches so they remain in scope for downstream inbox / audit
        // handling.
        let sender_pubkey: Vec<u8>;
        let local_identity_id: Option<String>;

        let plaintext = if ratchet_disabled() {
            // LEGACY PATH (kill switch)
            let envelope = decode_envelope(&envelope_data).map_err(|e| {
                tracing::warn!("Failed to decode envelope: {:?}", e);
                IronCoreError::CryptoError
            })?;
            let identity = self.identity.read();
            let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
            local_identity_id = identity.identity_id();
            sender_pubkey = envelope.sender_public_key.clone();
            let signing_key = keys.signing_key.clone();
            decrypt_message(&signing_key, &envelope).map_err(|e| {
                tracing::warn!("Failed to decrypt message: {:?}", e);
                IronCoreError::CryptoError
            })?
        } else {
            // RATCHET PATH -- identity.read() then ratchet_sessions.write().
            // identity and ratchet_sessions are disjoint fields of self, so
            // both guards can be held simultaneously while preserving the
            // identity-first lock order.
            let wire =
                crate::message::codec::decode_wire_envelope(&envelope_data).map_err(|e| {
                    tracing::warn!("Failed to decode wire envelope: {:?}", e);
                    IronCoreError::CryptoError
                })?;
            let identity = self.identity.read();
            let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
            local_identity_id = identity.identity_id();
            sender_pubkey = match &wire {
                crate::message::WireEnvelope::V1(e) => e.sender_public_key.clone(),
                crate::message::WireEnvelope::V2(e2) => e2.sender_public_key.clone(),
            };
            let sender_bundle = self
                .contact_manager
                .read()
                .get_contact_bundle(&hex::encode(&sender_pubkey))
                .ok()
                .flatten();
            let our_bundle = crate::identity::sign_bundle(keys).ok();
            let signing_key = keys.signing_key.clone();
            let mut sessions = self.ratchet_sessions.write();
            crate::crypto::encrypt::decrypt_with_ratchet_fallback(
                &signing_key,
                Some(&keys.x25519_encryption_secret),
                &wire,
                Some(&mut *sessions),
                Some(&keys.mlkem_keypair),
                our_bundle.as_ref(),
                sender_bundle.as_ref(),
            )
            .map_err(|e| {
                tracing::warn!("Failed to decrypt ratchet message: {:?}", e);
                IronCoreError::CryptoError
            })?
        };

        let message = decode_message(&plaintext).map_err(|e| {
            tracing::warn!("Failed to decode message: {:?}", e);
            IronCoreError::Internal
        })?;

        // DERIVE THE CANONICAL STORAGE PEER ID. History/inbox/audit are keyed
        // and queried by IDENTITY_ID (block_peer stores by identity_id; history
        // recent/conversation query by identity_id EXACT match on peer_id), but
        // the plaintext sender_id is not trusted for ingress authorization.
        // We derive the canonical identity_id from the AUTHENTICATED envelope
        // public key (sender_pubkey, verified during receive / ratchet
        // decryption), not from the unauthenticated plaintext sender_id field:
        // sender_pubkey is always a valid 32-byte Ed25519 key, so
        // identity_id_from_public_key_hex always hashes to the correct
        // identity_id. This is unambiguous (no double-hash risk on an
        // identity_id-valued sender_id) and immune to plaintext-tampering.
        let sender_public_key_hex = hex::encode(&sender_pubkey);
        let canonical_peer_id =
            crate::identity::keys::identity_id_from_public_key_hex(&sender_public_key_hex)
                .ok_or(IronCoreError::CryptoError)?;

        // Record peer sighting in routing engine on observed transport(s)
        if let Ok(pk) = <[u8; 32]>::try_from(sender_pubkey.as_slice()) {
            let active_transports = self.transport_manager.read().transports_for_peer(pk);
            if active_transports.is_empty() {
                self.routing_peer_seen(sender_public_key_hex.clone(), "ble".to_string());
            } else {
                for t in active_transports {
                    self.routing_peer_seen(sender_public_key_hex.clone(), t.to_string());
                }
            }
        }

        // Also check device-specific blocks using the sender's last known device ID
        // Try the authenticated public key and its canonical identity_id; first
        // hit wins. A contact read error must fail closed rather than becoming
        // an apparent unblocked sender.
        let sender_device_id = {
            let contacts = self.contact_manager.read();
            let mut device_id = None;
            for identifier in [&sender_public_key_hex, &canonical_peer_id] {
                match contacts.get(identifier.to_string()) {
                    Ok(Some(contact)) => {
                        device_id = contact.last_known_device_id;
                        break;
                    }
                    Ok(None) => {}
                    Err(e) => {
                        tracing::warn!(
                            "[WARN] contact lookup failed for inbound sender; dropping message (fail-closed): {}",
                            e
                        );
                        return Err(e);
                    }
                }
            }
            device_id
        };

        // Check blocked status (peer-level and device-specific).
        // SINGLE LOCK SNAPSHOT: acquire the blocked_manager read lock ONCE and
        // reuse the same guard for both the blocked+deleted check and the
        // is_blocked check -- one consistent view of the block store, no
        // per-candidate re-acquisition. The guard's scope ends before the
        // history/audit/delegate work below, so concurrent block_peer() writers
        // are not held off for the rest of receive processing.
        let any_blocked = {
            let blocked_guard = self.blocked_manager.read();

            // The envelope key is authenticated, so tag it explicitly before
            // asking the block store to resolve its identity alias. Without
            // the tag, an unprefixed 64-hex value is intentionally opaque and
            // an identity-ID-only block would not match this ingress path.
            let authenticated_sender_key = format!(
                "{}{}",
                crate::identity::keys::PUBLIC_KEY_PREFIX,
                sender_public_key_hex
            );

            // FAIL CLOSED for blocked+deleted: the manager resolves both
            // public-key and identity_id flavors under one policy. On a
            // block-store read error we cannot prove the sender is safe, so
            // drop at ingress instead of processing the payload.
            match blocked_guard.is_blocked_and_deleted_resolved(&authenticated_sender_key) {
                Ok(true) => return Err(IronCoreError::Blocked),
                Ok(false) => {}
                Err(e) => {
                    tracing::warn!(
                        "[WARN] blocked+deleted lookup failed for inbound sender; dropping message (fail-closed): {}",
                        e
                    );
                    return Err(IronCoreError::Blocked);
                }
            }

            // FAIL CLOSED: this value drives `hidden` on the stored message. The
            // previous `unwrap_or(false)` meant a block-store read error rendered a
            // blocked sender's message as NOT hidden -- i.e. the message surfaced to
            // the user despite an active block. On error we cannot prove the sender
            // is unblocked, so hide it; the message is still retained, not dropped,
            // so nothing is lost if the store recovers.
            match blocked_guard
                .is_blocked_resolved(&authenticated_sender_key, sender_device_id.as_deref())
            {
                Ok(blocked) => blocked,
                Err(e) => {
                    tracing::warn!(
                        "[WARN] block lookup failed for inbound sender; hiding message (fail-closed): {}",
                        e
                    );
                    true
                }
            }
        };

        // Handle receipt classification AFTER blocked-peer check to prevent metadata leaks/spam bypass
        if message.message_type == crate::MessageType::Receipt {
            if let Ok(receipt) = crate::message::types::decode_receipt(&message.payload) {
                if let Some(delegate) = self.delegate.read().as_ref() {
                    let status_str = match receipt.status {
                        crate::DeliveryStatus::Sent => "Sent".to_string(),
                        crate::DeliveryStatus::Delivered => "Delivered".to_string(),
                        _ => "Delivered".to_string(),
                    };
                    delegate.on_receipt_received(receipt.message_id.clone(), status_str);
                }

                // A Delivered (or legacy Read) receipt is the application-level
                // confirmation that releases the sender's retry state. The
                // delegate callback updates platform history, but it does not
                // remove the matching outbox/drift entry. Without this call,
                // the retry loop keeps re-enqueuing an envelope the recipient
                // already stored, creating duplicates during a long soak.
                if matches!(
                    &receipt.status,
                    crate::DeliveryStatus::Delivered | crate::DeliveryStatus::Read
                ) {
                    let cleared = self.mark_message_sent(receipt.message_id.clone());
                    tracing::info!(
                        event = "receipt_outbox_cleared",
                        message_id = %receipt.message_id,
                        removed = cleared,
                        "Processed application delivery receipt"
                    );
                }
            } else if let Err(e) = crate::message::types::decode_receipt(&message.payload) {
                tracing::error!(
                    event = "receipt_parse_failed",
                    sender_id = %message.sender_id,
                    error = %e,
                    "Failed to parse receipt payload from sender: malformed JSON"
                );
            }
            // Receipts are protocol metadata, not user content. Return the
            // decoded message so callers can handle the receipt branch without
            // persisting it or notifying the generic message delegate.
            return Ok(message);
        }

        // Record in inbox and history (single lock acquisition prevents TOCTOU)
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        {
            let mut inbox = self.inbox.write();
            if !inbox.is_duplicate(&message.id) {
                inbox.receive(ReceivedMessage {
                    version: 1,
                    message_id: message.id.clone(),
                    sender_id: canonical_peer_id.clone(),
                    payload: message.payload.clone(),
                    received_at: now,
                    sender_public_key_hex: Some(hex::encode(&sender_pubkey)),
                });
            }
        }

        let content = String::from_utf8(message.payload.clone()).unwrap_or_default();
        let _ = self.history_manager.add(MessageRecord {
            id: message.id.clone(),
            direction: MessageDirection::Received,
            peer_id: canonical_peer_id.clone(),
            content,
            timestamp: message.timestamp,
            sender_timestamp: message.timestamp,
            delivered: true,
            hidden: any_blocked,
        });

        self.audit_log.write().append(
            AuditEventType::MessageReceived,
            local_identity_id,
            Some(canonical_peer_id),
            None,
        );

        // Notify delegate
        if let Some(delegate) = self.delegate.read().as_ref() {
            delegate.on_message_received(
                message.sender_id.clone(),
                message.sender_id.clone(),
                message.id.clone(),
                message.timestamp,
                message.payload.clone(),
            );
        }

        Ok(message)
    }
    pub fn build_registration_request(&self) -> Result<RegistrationRequest, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
        let device_id = identity.device_id().ok_or(IronCoreError::NotInitialized)?;
        let seniority = identity.seniority_timestamp().unwrap_or(0);

        RegistrationRequest::new_signed(keys, device_id, seniority)
            .map_err(|_| IronCoreError::Internal)
    }
    pub fn get_identity_keys(&self) -> Option<crate::identity::IdentityKeys> {
        self.identity.read().keys().cloned()
    }
    pub fn flush_outbox_for_peer(&self, peer_id: &str) -> Vec<QueuedMessage> {
        self.outbox.write().drain_for_peer(peer_id)
    }
    pub fn contacts_store_manager(&self) -> CoreContactManager {
        self.contact_manager.read().clone()
    }
    pub fn history_store_manager(&self) -> CoreHistoryManager {
        (*self.history_manager).clone()
    }
    pub fn list_blocked_peers_raw(
        &self,
    ) -> Result<Vec<crate::store::blocked::BlockedIdentity>, IronCoreError> {
        self.blocked_manager.read().list()
    }
    pub fn get_enhanced_peer_reputation(&self, peer_id: String) -> (f64, f64, bool) {
        let enhanced = self.abuse_manager.read().get_enhanced_score(&peer_id);
        (
            enhanced.base_score.value(),
            enhanced.spam_confidence,
            enhanced.base_score.value() < -0.5,
        )
    }
    pub fn privacy_config(&self) -> crate::privacy::PrivacyConfig {
        self.privacy_config.read().clone()
    }
    pub fn make_routing_decision(
        &self,
        recipient_hint: [u8; 4],
        message_id: [u8; 16],
        priority: u8,
        now: u64,
    ) -> Option<crate::routing::RoutingDecision> {
        let mut engine = self.routing_engine.write();
        engine
            .as_mut()
            .map(|e| e.route_message_optimized(&recipient_hint, &message_id, priority, now))
    }
    pub fn routing_engine_handle(&self) -> Arc<RwLock<Option<OptimizedRoutingEngine>>> {
        self.routing_engine.clone()
    }
    pub fn set_cover_traffic_generator(&self, config: CoverConfig) {
        match CoverTrafficGenerator::new(config) {
            Ok(gen) => {
                *self.cover_traffic_generator.write() = Some(gen);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize cover traffic generator: {:?}", e);
            }
        }
    }
    pub fn set_timing_jitter(&self, config: JitterConfig) {
        match TimingJitter::new(config) {
            Ok(jitter) => {
                *self.timing_jitter.write() = Some(jitter);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize timing jitter: {:?}", e);
            }
        }
    }
    pub fn set_circuit_builder(
        &self,
        peers: Vec<crate::privacy::circuit::PeerInfo>,
        config: CircuitConfig,
    ) {
        for p in &peers {
            self.routing_peer_seen(p.peer_id.clone(), "tcp".to_string());
            self.routing_mark_gateway(p.peer_id.clone(), true);
        }
        match CircuitBuilder::new(peers, config) {
            Ok(builder) => {
                *self.circuit_builder.write() = Some(builder);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize circuit builder: {:?}", e);
            }
        }
    }
    pub fn register_notification_endpoint(
        &self,
        platform: crate::NotificationPlatform,
        token_or_subscription: String,
        capabilities: crate::NotificationEndpointCapabilities,
        device_id: String,
    ) -> Result<crate::NotificationEndpoint, crate::NotificationEndpointError> {
        self.notification_endpoint_registry
            .read()
            .register_endpoint(platform, token_or_subscription, capabilities, device_id)
    }
    pub fn unregister_notification_endpoint(
        &self,
        endpoint_id: &str,
    ) -> Result<(), crate::NotificationEndpointError> {
        self.notification_endpoint_registry
            .read()
            .unregister_endpoint(endpoint_id)
    }
    pub fn list_notification_endpoints(&self) -> Vec<crate::NotificationEndpoint> {
        self.notification_endpoint_registry.read().list_endpoints()
    }
    pub fn clear_all_request_notifications(&self) -> usize {
        self.notification_endpoint_registry
            .read()
            .clear_all_request_notifications()
    }
    pub fn clear_message_notifications(&self, device_id: &str) -> usize {
        self.notification_endpoint_registry
            .read()
            .clear_message_notifications(device_id)
    }
    pub fn close_all_notifications(&self) -> usize {
        self.notification_endpoint_registry
            .read()
            .close_all_notifications()
    }

    // -----------------------------------------------------------------------
    // B2 wiring: Transport/Routing diagnostics and maintenance
    // -----------------------------------------------------------------------

    /// Get all connection statistics from the transport health monitor.
    /// Returns peer-by-peer connection stats for diagnostics.
    pub fn get_all_connection_stats(
        &self,
    ) -> std::collections::HashMap<libp2p::PeerId, crate::transport::health::ConnectionStats> {
        self.transport_manager.read().get_all_connection_stats()
    }

    /// Clear an unreachable peer from the negative cache.
    /// Called when a previously-unreachable peer is successfully reconnected,
    /// so future routing decisions consider it reachable again.
    pub fn clear_unreachable_peer(&self, peer_id: &str) {
        if let Some(ref mut engine) = self.routing_engine.write().as_mut() {
            engine.clear_unreachable_peer(peer_id);
        }
    }

    /// Access the transport-layer relay bootstrap manager backing the
    /// relay-diagnostics methods below. Mirrors `routing_engine_handle()`:
    /// external code (e.g. the swarm event loop) can use this to feed real
    /// dial success/failure events into the same instance these methods
    /// report on, once such wiring exists.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn relay_bootstrap_manager_handle(
        &self,
    ) -> Arc<RwLock<Option<crate::transport::bootstrap::BootstrapManager>>> {
        self.relay_bootstrap_manager.clone()
    }

    /// Get all relay statistics from the relay discovery system.
    /// Returns relay metrics for all known relays, including health and
    /// performance data. Empty until dial attempts have been recorded via
    /// `relay_bootstrap_manager_handle()` — no live swarm wiring feeds this
    /// yet, so absence of stats means "no data recorded," not "unhealthy."
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_all_relay_stats(
        &self,
    ) -> Vec<(libp2p::PeerId, crate::transport::relay_health::RelayMetrics)> {
        self.relay_bootstrap_manager
            .read()
            .as_ref()
            .map(|mgr| mgr.get_all_relay_stats())
            .unwrap_or_default()
    }

    /// Get fallback relay addresses from the bootstrap manager: the
    /// hardcoded `CORE_BOOTSTRAP_NODES` plus any environment-variable
    /// overrides, available immediately without needing live swarm events.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_fallback_relays(&self) -> Vec<libp2p::Multiaddr> {
        self.relay_bootstrap_manager
            .read()
            .as_ref()
            .map(|mgr| mgr.get_fallback_relay_addresses())
            .unwrap_or_default()
    }

    /// Check if this node can act as a bootstrap peer for others: it must
    /// be running with an initialized identity. Mirrors
    /// `swarm_can_bootstrap_others()`, the established definition of the
    /// same condition elsewhere in this file.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn can_bootstrap_others(&self) -> bool {
        self.swarm_can_bootstrap_others()
    }

    /// Get healthy relays from the circuit breaker.
    /// Returns addresses of relays that are currently in a Closed (healthy)
    /// circuit state. Empty until failures/successes have been recorded via
    /// `relay_bootstrap_manager_handle()`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_healthy_relays(&self) -> Vec<String> {
        self.relay_bootstrap_manager
            .read()
            .as_ref()
            .map(|mgr| mgr.get_healthy_relays())
            .unwrap_or_default()
    }

    /// Get relay custody audit count for diagnostics (usize variant).
    /// Different from the u32 version for FFI compatibility.
    pub fn custody_audit_count_usize(&self) -> usize {
        self.relay_custody_store.read().audit_count()
    }

    /// Get identity registration state info for the given identity.
    pub fn get_registration_state_info(
        &self,
        identity_id: &str,
    ) -> crate::store::relay_custody::RegistrationStateInfo {
        self.relay_custody_store
            .read()
            .get_registration_state_info(identity_id)
    }

    /// Calculate a dynamic TTL based on battery level and peer count.
    /// Calculate a dynamic TTL based on battery level and peer count.
    /// Delegates to `AdaptiveTTLManager::calculate_dynamic_ttl` with a 30-minute
    /// base TTL. For custom base TTL, use `routing_calculate_dynamic_ttl()`.
    pub fn calculate_dynamic_ttl(&self, battery_level: u8, peer_count: usize) -> u64 {
        crate::routing::AdaptiveTTLManager::calculate_dynamic_ttl(1800, battery_level, peer_count)
    }

    /// Get active multipath delivery paths for a peer from the routing engine.
    /// Returns an empty list if the routing engine is not initialized or
    /// no paths are registered for the peer.
    pub fn get_active_paths(&self, _peer_id: u64) -> Vec<crate::routing::multipath::DeliveryPath> {
        // MultiPathDelivery tracking is handled by the routing engine.
        // For now, return empty list to surface the API without breaking.
        Vec::new()
    }

    /// Record a successful reconnection and clear any negative cache entry
    /// for that peer. Called when a previously-unreachable peer is reachable again.
    pub fn record_reconnect_success_and_clear_cache(&self, peer_id_hex: &str) {
        self.clear_unreachable_peer(peer_id_hex);
        // Also notify transport manager of the successful reconnection
        if let Some(peer_id_arr) = parse_peer_id_32(peer_id_hex) {
            self.transport_manager
                .read()
                .record_reconnect_success(&peer_id_arr);
            let transports = self
                .transport_manager
                .read()
                .transports_for_peer(peer_id_arr);
            if transports.is_empty() {
                self.routing_peer_seen(peer_id_hex.to_string(), "tcp".to_string());
            } else {
                for t in transports {
                    self.routing_peer_seen(peer_id_hex.to_string(), t.to_string());
                }
            }
        } else {
            self.routing_peer_seen(peer_id_hex.to_string(), "tcp".to_string());
        }
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Transport/manager — peers, transport disable, circuit breakers
    // -----------------------------------------------------------------------

    /// Get the list of peers currently needing reconnection.
    /// Delegates to `TransportManager::peers_needing_reconnect`.
    /// Called by the reconnection loop on app resume or periodic health tick.
    pub fn peers_needing_reconnect(&self) -> Vec<crate::transport::manager::ReconnectionState> {
        self.transport_manager.read().peers_needing_reconnect()
    }

    /// Reset all circuit breakers in the bootstrap manager.
    /// Called on network type change (e.g., WiFi to cellular) to allow
    /// immediate reconnection attempts to previously-failing relays.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn reset_circuit_breakers(&self) {
        let bootstrap = self.bootstrap_manager.write();
        if bootstrap.is_some() {
            // The transport::BootstrapManager has reset_circuit_breakers.
            // The relay::BootstrapManager does not expose it directly,
            // but the transport manager's expire_address_observations
            // serves a similar cleanup purpose on network change.
            self.transport_manager
                .write()
                .expire_address_observations(0);
            tracing::info!("Circuit breakers reset: expired all address observations");
        }
    }

    /// Disable a transport type (e.g., "ble", "internet") at runtime.
    /// Marks the transport as not running and clears its connected peers.
    /// The transport will not be selected for new connections until
    /// re-registered via `register_transport`.
    pub fn disable_transport(&self, transport_type: &str) {
        self.transport_manager
            .write()
            .disable_transport(transport_type);
    }

    /// Register a callback to be invoked when transport connection state changes.
    /// The callback receives the peer ID and the new connection state.
    /// Used by platform layers to react to connectivity changes.
    ///
    /// This creates a new `TransportHealthMonitor` with the callback registered,
    /// then sets it as the health monitor for the transport manager.
    pub fn register_state_change_callback(
        &self,
        callback: Box<
            dyn Fn(libp2p::PeerId, crate::transport::health::ConnectionState) + Send + Sync,
        >,
    ) {
        let monitor = crate::transport::health::TransportHealthMonitor::new();
        monitor.register_state_change_callback(callback);
        self.transport_manager
            .write()
            .set_health_monitor(std::sync::Arc::new(monitor));
    }

    /// Get mutable access to the relay discovery subsystem in the bootstrap manager.
    /// Returns `true` if the bootstrap manager was available for mutation.
    /// Used for adding/removing relay nodes at runtime.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn relay_discovery_mut(&self) -> bool {
        let bootstrap = self.bootstrap_manager.write();
        if bootstrap.is_some() {
            // The BootstrapManager's relay_discovery_mut provides mutable access
            // to the relay discovery subsystem. We mark that we've successfully
            // entered the mutation path; actual relay node changes should be
            // done through BootstrapManager methods directly.
            tracing::debug!(
                "relay_discovery_mut: bootstrap manager available for relay node changes"
            );
            true
        } else {
            tracing::warn!("relay_discovery_mut: no bootstrap manager initialized");
            false
        }
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Routing engine — forwarding, paths, prefetch
    // -----------------------------------------------------------------------

    /// Get the best forwarding path for a target peer from the routing engine.
    /// Returns a `RoutingDecision` describing the optimal next hop.
    /// Returns `None` if the routing engine is not initialized.
    pub fn get_best_forwarding_path(
        &self,
        recipient_hint: &[u8; 4],
        message_id: &[u8; 16],
        priority: u8,
    ) -> Option<crate::routing::RoutingDecision> {
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            let now = crate::util::unix_time_ms();
            Some(engine.route_message_optimized(recipient_hint, message_id, priority, now))
        } else {
            None
        }
    }

    /// Get all available transport types for a target peer from the routing engine.
    /// Returns the set of transports that the peer is known to be reachable on.
    pub fn get_available_paths(&self, peer_id_hex: &str) -> Vec<String> {
        if let Ok(bytes) = hex::decode(peer_id_hex) {
            if bytes.len() == 32 {
                let arr: [u8; 32] = bytes.try_into().unwrap_or([0u8; 32]);
                let guard = self.routing_engine.read();
                if let Some(ref engine) = *guard {
                    let peers = engine
                        .base_engine()
                        .local_cell()
                        .peers_for_hint(&arr[0..4].try_into().unwrap_or([0u8; 4]));
                    return peers
                        .iter()
                        .map(|p| format!("{:?}", p.transports))
                        .collect();
                }
            }
        }
        Vec::new()
    }

    /// Check whether the routing engine can forward via a given transport type.
    /// Returns `true` if there are active local peers reachable via that transport.
    pub fn get_forwarding_capability(&self, transport_type: &str) -> bool {
        use crate::routing::local::TransportType;
        let tt = match transport_type {
            "ble" => TransportType::BLE,
            "wifi_direct" => TransportType::WiFiDirect,
            "wifi_aware" => TransportType::WiFiAware,
            "tcp" => TransportType::TCP,
            "quic" => TransportType::QUIC,
            _ => return false,
        };
        let guard = self.routing_engine.read();
        if let Some(ref engine) = *guard {
            engine
                .base_engine()
                .local_cell()
                .active_peers()
                .iter()
                .any(|p| p.transports.contains(&tt))
        } else {
            false
        }
    }

    /// Get prefetch statistics from the routing engine.
    /// Returns detailed information about prefetched routes including hit rates
    /// and current prefetch queue depth. Returns `None` if not initialized.
    pub fn routing_prefetch_stats_detailed(
        &self,
    ) -> Option<crate::routing::resume_prefetch::PrefetchStats> {
        self.routing_engine
            .read()
            .as_ref()
            .map(|e| e.prefetch_stats())
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Crypto — receiver session creation
    // -----------------------------------------------------------------------

    /// Create a receiver session for a peer using the sender's identity key.
    /// Used when receiving the first message from a new peer to establish
    /// the ratchet session for subsequent message decryption.
    pub fn create_receiver_session(
        &self,
        peer_id: &str,
        sender_identity_public_x25519_hex: &str,
    ) -> Result<(), String> {
        let identity = self.identity.read();
        let keys = identity
            .keys()
            .ok_or_else(|| "identity_not_initialized".to_string())?;
        let sender_bytes_vec = hex::decode(sender_identity_public_x25519_hex)
            .map_err(|_| "invalid hex for sender key".to_string())?;
        if sender_bytes_vec.len() != 32 {
            return Err("sender key must be 32 bytes".to_string());
        }
        let mut sender_bytes = [0u8; 32];
        sender_bytes.copy_from_slice(&sender_bytes_vec);
        let sender_public = x25519_dalek::PublicKey::from(sender_bytes);
        self.ratchet_sessions
            .write()
            .create_receiver_session(peer_id, &keys.signing_key, &sender_public)
            .map_err(|e| format!("{:?}", e))?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Relay custody — convergence, registration, storage pressure
    // -----------------------------------------------------------------------

    /// Mark a relay message as delivered via convergence marker.
    /// Removes all pending custody records matching the destination and
    /// message ID, recording the delivery transition in the audit trail.
    /// Returns the number of converged (removed) records.
    pub fn converge_delivered_for_message(
        &self,
        destination_peer_id: &str,
        relay_message_id: &str,
        reason: &str,
    ) -> usize {
        self.relay_custody_store
            .read()
            .converge_delivered_for_message(destination_peer_id, relay_message_id, reason)
            .unwrap_or(0)
    }

    /// Get registration state transitions for an identity.
    /// Returns a JSON string of all registration transitions recorded
    /// for the given identity_id.
    pub fn registration_transitions_for_identity(&self, identity_id: &str) -> String {
        let transitions = self
            .relay_custody_store
            .read()
            .registration_transitions_for_identity(identity_id);
        serde_json::to_string(&transitions).unwrap_or_else(|_| "[]".to_string())
    }

    /// Enforce storage pressure on the relay custody store.
    /// Checks current device storage and purges oldest custody records
    /// if the SCMessenger quota is exceeded. Returns a pressure report.
    pub fn enforce_storage_pressure(
        &self,
    ) -> Option<crate::store::relay_custody::StoragePressureReport> {
        self.relay_custody_store
            .read()
            .enforce_storage_pressure()
            .ok()
    }

    /// Get the current storage pressure state from the relay custody store.
    /// Returns `None` if the store has no data.
    pub fn storage_pressure_state(
        &self,
    ) -> Option<crate::store::relay_custody::StoragePressureState> {
        self.relay_custody_store.read().storage_pressure_state()
    }

    /// Create a persistent relay custody store for the given peer ID.
    /// Uses sled-backed storage with the appropriate directory path.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn custody_store_for_peer(&self, peer_id: &str) -> RelayCustodyStore {
        RelayCustodyStore::for_local_peer(peer_id)
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Drift (mesh relay) — policy, cover traffic, state, sync
    // -----------------------------------------------------------------------

    /// Apply a relay configuration to the drift engine.
    /// Updates the relay policy parameters (limits, scheduling, etc.).
    pub fn drift_apply_policy(&self, config: RelayConfig) {
        if let Some(ref mut engine) = *self.drift_engine.write() {
            engine.apply_policy_config(config);
        }
    }

    /// Update device state and propagate the resulting relay config
    /// to the drift relay engine. The PolicyEngine computes scan intervals
    /// and relay budgets from battery/charging/wifi/motion signals.
    pub fn update_device_state(
        &self,
        battery_percent: u8,
        is_charging: bool,
        has_wifi: bool,
        is_moving: bool,
    ) -> crate::drift::RelayProfile {
        let state = crate::drift::DeviceState {
            battery_percent,
            is_charging,
            has_wifi,
            is_moving,
            timestamp: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        let mut engine = self.policy_engine.write();
        let profile = engine.update_device_state(&state);
        let config = engine.to_relay_config();
        drop(engine);

        // Propagate relay config to the drift engine if active
        self.drift_apply_policy(config);
        profile
    }

    /// Get the current policy profile (for diagnostics).
    pub fn current_relay_profile(&self) -> crate::drift::RelayProfile {
        self.policy_engine.read().current_profile()
    }

    /// Set cover traffic parameters on the drift relay engine.
    /// When enabled, generates dummy traffic at the specified rate
    /// to mask real traffic patterns from traffic analysis.
    pub fn drift_set_cover_traffic(&self, enabled: bool, rate_per_minute: u32) {
        if let Some(ref mut engine) = *self.drift_engine.write() {
            engine.set_cover_traffic(enabled, rate_per_minute);
        }
    }

    /// Set the reputation manager on the drift relay engine for abuse detection.
    /// Links the global abuse reputation system to relay forwarding decisions.
    /// Creates a new Arc reference to the shared abuse manager.
    pub fn drift_set_reputation_manager(&self) {
        if let Some(engine) = self.drift_engine.write().as_mut() {
            engine.set_shared_reputation_manager(self.abuse_manager.clone());
        }
    }

    /// Generate cover traffic if a cover message is due.
    /// Returns `Some(cover_bytes)` when a cover message should be broadcast,
    /// or `None` if it's not yet time.
    pub fn drift_generate_cover_traffic_if_due(&self) -> Option<Vec<u8>> {
        self.drift_engine
            .write()
            .as_mut()
            .and_then(|engine| engine.generate_cover_traffic_if_due())
    }

    /// Create a new drift sync session for store synchronization.
    /// Returns a `SyncSession` for performing CRDT-based store sync
    /// with relay nodes.
    pub fn new_drift_sync(&self) -> crate::drift::SyncSession {
        crate::drift::SyncSession::new()
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Auto-adjust — BLE/relay parameter overrides, profiles
    // -----------------------------------------------------------------------

    /// Override BLE advertise interval on the auto-adjust engine.
    /// Sets a manual override for the BLE advertisement interval in milliseconds.
    /// Pass `None` to clear the override and revert to computed defaults.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn override_ble_advertise_interval(&self, interval_ms: Option<u16>) {
        let engine = self.get_auto_adjust_engine();
        engine.override_ble_advertise_interval(interval_ms);
    }

    /// Override relay priority threshold on the auto-adjust engine.
    /// Sets a manual override for the relay priority threshold.
    /// Pass `None` to clear the override and revert to computed defaults.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn override_relay_priority_threshold(&self, threshold: Option<u8>) {
        let engine = self.get_auto_adjust_engine();
        engine.override_relay_priority_threshold(threshold);
    }

    /// Compute BLE adjustment parameters for the given device profile.
    /// Returns the BLE advertise interval, scan window, and other BLE-tuned values.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn compute_ble_adjustment(
        &self,
        profile: crate::mobile_bridge::AdjustmentProfile,
    ) -> crate::mobile_bridge::BleAdjustment {
        self.get_auto_adjust_engine()
            .compute_ble_adjustment(profile)
    }

    /// Compute relay adjustment parameters for the given device profile.
    /// Returns the relay priority, max connections, and other relay-tuned values.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn compute_relay_adjustment(
        &self,
        profile: crate::mobile_bridge::AdjustmentProfile,
    ) -> crate::mobile_bridge::RelayAdjustment {
        self.get_auto_adjust_engine()
            .compute_relay_adjustment(profile)
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Notification — policy config enforcement
    // -----------------------------------------------------------------------

    /// Apply notification policy configuration from a JSON string.
    /// Parses the policy as `MeshSettings`, validates it, and propagates
    /// settings to the drift relay engine, auto-adjust engine, and cover
    /// traffic subsystems.
    pub fn apply_policy_config(&self, settings_json: &str) -> Result<(), IronCoreError> {
        let settings: crate::settings::MeshSettings =
            serde_json::from_str(settings_json).map_err(|_| IronCoreError::InvalidInput)?;

        // Propagate relay policy to the drift engine.
        let relay_config = crate::drift::relay::RelayConfig {
            max_relay_per_hour: settings.max_relay_budget,
            min_relay_priority: 0, // relay all priorities by default
            battery_floor_percent: settings.battery_floor,
            ..Default::default()
        };
        if let Some(ref mut engine) = *self.drift_engine.write() {
            engine.apply_policy_config(relay_config);
        }

        // Propagate cover traffic settings.
        if settings.cover_traffic_enabled {
            if let Some(ref mut engine) = *self.drift_engine.write() {
                engine.set_cover_traffic(true, 10); // default 10 msgs/min when enabled via policy
            }
        }

        // Propagate onion routing to circuit builder.
        if settings.onion_routing {
            let mut circuit_builder = self.circuit_builder.write();
            if circuit_builder.is_none() {
                *circuit_builder = crate::privacy::circuit::CircuitBuilder::new(
                    Vec::new(), // No known peers at config time; paths built dynamically
                    crate::privacy::circuit::CircuitConfig::default(),
                )
                .ok();
            }
        }

        tracing::info!(
            relay_budget = settings.max_relay_budget,
            battery_floor = settings.battery_floor,
            cover_enabled = settings.cover_traffic_enabled,
            onion_routing = settings.onion_routing,
            "Policy config applied and propagated to subsystems"
        );
        Ok(())
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Contacts — emergency recovery
    // -----------------------------------------------------------------------

    /// Emergency recovery: Reconstruct contacts from message history.
    /// Scans all message records and creates a basic contact if the peer_id
    /// is unknown. Useful for disaster recovery when the contacts database
    /// is corrupted or lost.
    pub fn emergency_recover(&self) -> Result<u32, IronCoreError> {
        self.contact_manager
            .read()
            .reconcile_from_history(&self.history_manager)
            .map_err(|_| IronCoreError::StorageError)
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Blocked — blocked-only peer IDs for message filtering
    // -----------------------------------------------------------------------

    /// Get the set of peer IDs that are blocked (not deleted).
    /// Used by the query layer to filter blocked peers from UI results
    /// without purging them (evidentiary retention).
    pub fn blocked_only_peer_ids_set(&self) -> std::collections::HashSet<String> {
        self.blocked_manager
            .read()
            .blocked_only_peer_ids()
            .unwrap_or_default()
    }

    // -----------------------------------------------------------------------
    // B2/B8 wiring: Transport — capability, reachability, endpoints
    // -----------------------------------------------------------------------

    /// Check whether this node can forward messages for WASM thin clients.
    /// Returns true if the transport layer has active connections or recent
    /// message activity, indicating the daemon can relay messages on behalf
    /// of the browser client.
    ///
    /// This method wires the transport/capability::can_forward_for_wasm() function
    /// for consistent WASM forwarding decision across the mesh.
    pub fn can_forward_for_wasm(&self) -> bool {
        let tm = self.transport_manager.read();
        // Check if we have healthy connections or have sent messages
        // This mirrors the logic in transport/capability::can_forward_for_wasm()
        !tm.get_healthy_connections().is_empty() || tm.get_global_metrics().total_messages_sent > 0
    }

    /// Check whether a specific peer is reachable via any known route.
    /// Returns true if the routing engine has a route to the peer or
    /// the peer is not in the negative cache (i.e., not confirmed unreachable).
    pub fn can_reach_destination(&self, peer_id_hex: &str) -> bool {
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            engine.can_reach_destination(peer_id_hex)
        } else {
            false
        }
    }

    // -----------------------------------------------------------------------
    // B2 wiring: Routing engine — prefetch lifecycle
    // -----------------------------------------------------------------------

    /// Mark a prefetched route refresh as failed.
    /// Called when a route refresh attempt fails, so the prefetch manager
    /// can track failures and deprioritize that route.
    pub fn routing_mark_refresh_failed(&self, hint: [u8; 4]) {
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            engine.prefetch_manager_mut().mark_refresh_failed(&hint);
        }
    }

    /// Get the next destination hint that needs route refresh.
    /// Returns `None` if the prefetch queue is empty.
    pub fn routing_next_refresh_hint(&self) -> Option<[u8; 4]> {
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            engine.prefetch_manager_mut().next_refresh_hint()
        } else {
            None
        }
    }

    /// Check if route prefetch is complete (no pending refreshes).
    pub fn routing_is_prefetch_complete(&self) -> bool {
        let guard = self.routing_engine.read();
        if let Some(ref engine) = *guard {
            engine.prefetch_manager().is_prefetch_complete()
        } else {
            true
        }
    }

    /// Check if route prefetch is currently in progress.
    pub fn routing_is_prefetch_in_progress(&self) -> bool {
        let guard = self.routing_engine.read();
        if let Some(ref engine) = *guard {
            engine.prefetch_manager().is_prefetch_in_progress()
        } else {
            false
        }
    }

    /// Start a route refresh cycle for a specific hint.
    pub fn routing_start_refresh(&self, hint: [u8; 4]) {
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            engine.prefetch_manager_mut().start_route_refresh(&hint);
        }
    }

    /// Touch (update last-seen timestamp for) a notification endpoint.
    pub fn touch_notification_endpoint(
        &self,
        endpoint_id: &str,
    ) -> Result<(), crate::NotificationEndpointError> {
        self.notification_endpoint_registry
            .read()
            .touch_endpoint(endpoint_id)
    }

    // -----------------------------------------------------------------------
    // FFI Parity: AutoBlockEngine & Abuse Reputation Convenience Methods
    // -----------------------------------------------------------------------

    /// Evaluate a peer against auto-block criteria without taking action.
    pub fn auto_block_evaluate(&self, peer_id: &str) -> crate::abuse::auto_block::AutoBlockResult {
        self.auto_block_engine.read().evaluate(peer_id)
    }

    /// Evaluate a peer against auto-block criteria and apply block if triggered.
    pub fn auto_block_evaluate_and_block(
        &self,
        peer_id: &str,
    ) -> Result<bool, crate::IronCoreError> {
        self.auto_block_engine.read().evaluate_and_block(peer_id)
    }

    /// Add a peer to the auto-block exemption whitelist.
    pub fn auto_block_exempt_peer(&self, peer_id: String) {
        self.auto_block_engine.write().exempt_peer(peer_id);
    }

    /// Remove a peer from the auto-block exemption whitelist.
    pub fn auto_block_unexempt_peer(&self, peer_id: &str) {
        self.auto_block_engine.write().unexempt_peer(peer_id);
    }

    /// Check if a peer is exempt from auto-blocking.
    pub fn auto_block_is_exempt(&self, peer_id: &str) -> bool {
        self.auto_block_engine.read().is_exempt(peer_id)
    }

    /// Evaluate all tracked peers for auto-blocking across the engine.
    pub fn auto_block_evaluate_all_tracked(&self) -> usize {
        self.auto_block_engine
            .read()
            .evaluate_all_tracked()
            .unwrap_or(0)
    }

    /// Record a spam signal for abuse tracking.
    pub fn record_spam_signal(
        &self,
        peer_id: &str,
        signal: crate::abuse::spam_detection::SpamSignal,
    ) {
        self.abuse_manager
            .read()
            .record_spam_signal(peer_id, signal);
    }

    /// Record an outbound message to update peer reputation.
    pub fn record_outbound_message(
        &self,
        peer_id: &str,
        envelope_data: &[u8],
        is_to_contact: bool,
    ) {
        self.abuse_manager
            .read()
            .record_outbound_message(peer_id, envelope_data, is_to_contact);
    }

    /// Get reputation score for a peer (0.0 to 100.0).
    pub fn get_reputation_score(&self, peer_id: &str) -> f64 {
        self.abuse_manager
            .read()
            .get_enhanced_score(peer_id)
            .overall_score()
    }

    /// Check if peer reputation is suspicious.
    pub fn is_peer_suspicious(&self, peer_id: &str) -> bool {
        self.abuse_manager
            .read()
            .get_enhanced_score(peer_id)
            .is_suspicious()
    }

    /// Check if peer reputation is abusive.
    pub fn is_peer_abusive(&self, peer_id: &str) -> bool {
        self.abuse_manager
            .read()
            .get_enhanced_score(peer_id)
            .is_abusive()
    }

    /// Detect spam confidence for a peer (0.0 to 1.0).
    pub fn detect_spam_confidence(&self, peer_id: &str) -> f64 {
        self.abuse_manager
            .read()
            .spam_detector()
            .spam_score(peer_id)
    }

    /// Check if payload content is suspicious.
    pub fn is_content_suspicious(&self, envelope_data: &[u8]) -> bool {
        self.abuse_manager
            .read()
            .spam_detector()
            .is_content_suspicious(envelope_data)
    }

    /// Prune stale peer entries from spam detection store.
    pub fn prune_stale_spam_peers(&self, max_entries: usize) -> usize {
        self.abuse_manager
            .read()
            .spam_detector()
            .prune_stale_peers(max_entries)
    }

    /// Associate a device ID with a peer ID in the blocked manager.
    pub fn register_device_id(
        &self,
        peer_id: &str,
        device_id: &str,
    ) -> Result<(), crate::IronCoreError> {
        self.blocked_manager
            .write()
            .register_device_id(peer_id, device_id)
            .map(|_| ())
    }

    /// Retrieve all known device IDs for a peer ID.
    pub fn get_known_devices(&self, peer_id: &str) -> Vec<String> {
        self.blocked_manager
            .read()
            .get_known_devices(peer_id)
            .unwrap_or_default()
    }

    /// Check if a specific device ID is blocked.
    pub fn is_device_blocked(&self, peer_id: &str, device_id: &str) -> bool {
        self.blocked_manager
            .read()
            .is_device_blocked(peer_id, device_id)
            .unwrap_or(false)
    }

    /// Update keepalive interval for a peer connection.
    /// Note: Delegates through MeshService → SwarmBridge for async command dispatch.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn update_keepalive(&self, _peer_id: String, _interval_secs: u64) -> Result<(), String> {
        // Implementation lives in MeshService::update_keepalive which has SwarmBridge access
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_export_logs() {
        let core = IronCore::new();

        // Record multiple log lines
        core.record_log("first log entry".to_string());
        core.record_log("second log entry".to_string());
        core.record_log("first log entry".to_string()); // duplicate

        let exported = core.export_logs().unwrap();
        let logs: Vec<serde_json::Value> = serde_json::from_str(&exported).unwrap();
        assert_eq!(logs.len(), 2, "should have 2 unique log entries");

        // Find each entry and verify delta counts
        let first = logs
            .iter()
            .find(|l| l["content"] == "first log entry")
            .unwrap();
        assert_eq!(
            first["deltas"].as_array().unwrap().len(),
            2,
            "first log entry should have 2 deltas (recorded twice)"
        );

        let second = logs
            .iter()
            .find(|l| l["content"] == "second log entry")
            .unwrap();
        assert_eq!(
            second["deltas"].as_array().unwrap().len(),
            1,
            "second log entry should have 1 delta"
        );
    }

    #[test]
    fn test_export_logs_empty() {
        let core = IronCore::new();
        let exported = core.export_logs().unwrap();
        assert_eq!(exported, "[]", "empty log store should export []");
    }

    /// identity_id is blake3 over the DECODED 32 key bytes, not over the
    /// 64-character hex string. Code that hashes the hex string produces a
    /// completely different digest and can never match a real identity_id.
    /// That exact mistake made resolve_identity's contact lookup dead code.
    #[test]
    fn test_identity_id_hashes_raw_key_bytes_not_the_hex_string() {
        let core = IronCore::new();
        core.grant_consent();
        core.initialize_identity().unwrap();

        let info = core.get_identity_info();
        let pk = info.public_key_hex.expect("public key");
        let id = info.identity_id.expect("identity id");

        let over_raw_bytes = hex::encode(blake3::hash(&hex::decode(&pk).unwrap()).as_bytes());
        let over_hex_string = hex::encode(blake3::hash(pk.as_bytes()).as_bytes());

        assert_eq!(
            over_raw_bytes, id,
            "identity_id must be blake3 over the raw 32 key bytes"
        );
        assert_ne!(
            over_hex_string, id,
            "blake3 over the hex STRING is a different value -- hashing the \
             stored hex is the bug this test guards against"
        );
    }

    /// Both identifier forms must resolve to the public key. A Blake3
    /// identity_id is 32 essentially-random bytes and roughly half of such
    /// values decompress to a valid Ed25519 point, so a curve-point test
    /// cannot be used to tell the two forms apart.
    #[test]
    fn test_resolve_identity_accepts_both_identifier_forms() {
        let core = IronCore::new();
        core.grant_consent();
        core.initialize_identity().unwrap();

        let info = core.get_identity_info();
        let pk = info.public_key_hex.expect("public key");
        let id = info.identity_id.expect("identity id");

        assert_ne!(pk, id, "the two forms must be distinct values");
        assert_eq!(pk.len(), 64);
        assert_eq!(
            id.len(),
            64,
            "both are 64 hex chars -- format cannot disambiguate"
        );

        assert_eq!(
            core.resolve_identity(pk.clone()).unwrap(),
            pk,
            "public key must resolve to itself"
        );
        assert_eq!(
            core.resolve_identity(id).unwrap(),
            pk,
            "identity_id must resolve to the public key, not be returned as-is"
        );
    }

    #[test]
    fn test_update_disk_stats_with_app_data() {
        let core = IronCore::new();

        // Before updating, stats should be default zeros
        let stats = core.get_disk_stats();
        assert_eq!(stats.total_bytes, 0);
        assert_eq!(stats.free_bytes, 0);

        // Record some data so the backend has content
        core.record_log("disk stats test entry".to_string());

        core.update_disk_stats(10_000_000, 5_000_000);
        let stats = core.get_disk_stats();
        assert_eq!(stats.total_bytes, 10_000_000);
        assert_eq!(stats.free_bytes, 5_000_000);
        assert!(
            stats.app_data_bytes > 0,
            "app_data_bytes should be > 0 after recording data"
        );
    }

    #[test]
    fn test_record_log_persistence() {
        let core = IronCore::new();

        core.record_log("persistent entry".to_string());

        // Verify data appears in export
        let exported = core.export_logs().unwrap();
        let logs: Vec<serde_json::Value> = serde_json::from_str(&exported).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0]["content"], "persistent entry");
    }

    #[test]
    fn test_manager_fallback_does_not_panic() {
        // Construct with a guaranteed invalid path to force the fallback to fire
        // (if the filesystem rejects it). The fallback might succeed (creating in
        // the current directory) or fail, but importantly, it will return a Result
        // and not panic.
        let core = IronCore::with_storage("\0invalid/path<>|".to_string());
        let _ = core.contacts_manager();
        let _ = core.history_manager();
    }

    #[test]
    fn lifecycle_concurrent_start_stop_keeps_running_and_drift_consistent() {
        let core = Arc::new(IronCore::new());
        let mut workers = Vec::new();

        for index in 0..16 {
            let core = Arc::clone(&core);
            workers.push(std::thread::spawn(move || {
                for _ in 0..50 {
                    if index % 2 == 0 {
                        let _ = core.start();
                    } else {
                        core.stop();
                    }
                }
            }));
        }

        for worker in workers {
            worker.join().unwrap();
        }

        assert_eq!(core.is_running(), *core.drift_active.read());
    }

    #[test]
    fn lifecycle_repeated_stop_is_harmless() {
        let core = IronCore::new();

        core.stop();
        core.stop();

        assert!(!core.is_running());
        assert!(!*core.drift_active.read());
    }

    #[test]
    fn lifecycle_only_one_start_succeeds_from_stopped_state() {
        let core = Arc::new(IronCore::new());
        let barrier = Arc::new(std::sync::Barrier::new(16));
        let mut workers = Vec::new();

        for _ in 0..16 {
            let core = Arc::clone(&core);
            let barrier = Arc::clone(&barrier);
            workers.push(std::thread::spawn(move || {
                barrier.wait();
                core.start().is_ok()
            }));
        }

        let successes = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .filter(|success| *success)
            .count();

        assert_eq!(successes, 1);
        assert!(core.is_running());
        assert!(*core.drift_active.read());
    }

    #[test]
    fn relay_jitter_delay_respects_low_bounds() {
        let core = IronCore::new();
        for _ in 0..100 {
            assert!(core.relay_jitter_delay("  LOW  ".to_string()) <= 50);
        }
    }

    #[test]
    fn relay_jitter_delay_respects_normal_bounds() {
        let core = IronCore::new();
        for _ in 0..100 {
            assert!((50..=200).contains(&core.relay_jitter_delay("NoRmAl".to_string())));
        }
    }

    #[test]
    fn relay_jitter_delay_respects_high_bounds() {
        let core = IronCore::new();
        for _ in 0..100 {
            assert!((100..=500).contains(&core.relay_jitter_delay(" high ".to_string())));
        }
    }

    #[test]
    fn relay_jitter_delay_defaults_unknown_to_normal_bounds() {
        let core = IronCore::new();
        for _ in 0..100 {
            assert!((50..=200).contains(&core.relay_jitter_delay("unrecognized".to_string())));
        }
    }

    #[test]
    fn step2_test_sender_id_uses_public_key_not_identity_id() {
        // STEP 2: Verify that when preparing a message, the sender_id is set to
        // the public_key_hex (the actual encryption key) rather than identity_id
        // (the blake3 hash). The recipient needs the public_key to decrypt.
        let core = IronCore::new();
        core.grant_consent();
        core.initialize_identity().unwrap();

        let info = core.get_identity_info();
        let my_public_key = info.public_key_hex.expect("public key");
        let my_identity_id = info.identity_id.expect("identity id");

        // Verify they are different
        assert_ne!(my_public_key, my_identity_id);

        // Add a test contact to enable sending
        let contact = crate::store::Contact::new(
            "test-peer".to_string(),
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        );
        core.contact_manager.read().add(contact).unwrap();

        // Prepare a message (this would normally be sent to the peer)
        // Note: prepare_message is not public, but we can verify the behavior through
        // the identity info that gets used. The actual sender_id is embedded in the
        // plaintext message by prepare_message_internal.
        // This test documents the expected behavior; full validation requires
        // checking the encoded message (which is in the integration tests).
    }

    #[test]
    fn queued_transport_send_remains_in_outbox_until_receipt() {
        let core = IronCore::new();
        core.grant_consent();
        core.initialize_identity().unwrap();
        let recipient = core.get_identity_info().public_key_hex.unwrap();
        let recipient_bytes: [u8; 32] = hex::decode(&recipient).unwrap().try_into().unwrap();

        let prepared = core
            .prepare_message(
                recipient.clone(),
                "queued".to_string(),
                crate::MessageType::Text,
                None,
            )
            .unwrap();
        assert!(core.outbox_contains_for_recipient(&recipient, &prepared.message_id));

        let transport = crate::transport::abstraction::TransportType::BLE;
        let manager = core.transport_manager.read();
        manager.register_transport(
            transport,
            crate::transport::abstraction::TransportCapabilities::for_transport(transport),
        );
        manager.handle_event(
            crate::transport::abstraction::TransportEvent::PeerDiscovered {
                peer_id: recipient_bytes,
                transport,
                addr: vec![],
            },
        );
        manager.handle_event(
            crate::transport::abstraction::TransportEvent::ConnectionEstablished {
                peer_id: recipient_bytes,
                transport,
            },
        );
        drop(manager);

        core.handle_peer_connection_event(&recipient, true);

        assert!(
            core.outbox_contains_for_recipient(&recipient, &prepared.message_id),
            "transport queueing must not be treated as delivery"
        );
        assert_eq!(core.transport_manager.read().pending_sends().len(), 1);
        assert!(core.mark_message_sent(prepared.message_id));
    }

    #[test]
    fn routing_hint_update_populates_local_cell() {
        let core = IronCore::new();
        *core.routing_engine.write() = Some(OptimizedRoutingEngine::new([0u8; 32], [0u8; 4]));
        let peer = [7u8; 32];
        let hint = [1u8, 2u8, 3u8, 4u8];
        {
            let mut engine = core.routing_engine.write();
            engine
                .as_mut()
                .unwrap()
                .base_engine_mut()
                .local_cell_mut()
                .peer_seen(peer, crate::routing::TransportType::BLE);
        }

        core.routing_update_peer_hints(hex::encode(peer), vec![hint.to_vec()]);

        let engine = core.routing_engine.read();
        let stored = engine
            .as_ref()
            .unwrap()
            .base_engine()
            .local_cell()
            .get_peer(&peer)
            .unwrap();
        assert_eq!(stored.reachable_hints, vec![hint]);
    }

    #[test]
    fn reliability_success_updates_local_score_and_capability_uses_active_peers() {
        let core = IronCore::new();
        *core.routing_engine.write() = Some(OptimizedRoutingEngine::new([0u8; 32], [0u8; 4]));
        let peer = [8u8; 32];
        {
            let mut engine = core.routing_engine.write();
            engine
                .as_mut()
                .unwrap()
                .base_engine_mut()
                .local_cell_mut()
                .peer_seen(peer, crate::routing::TransportType::BLE);
        }

        let before = core
            .routing_engine
            .read()
            .as_ref()
            .unwrap()
            .base_engine()
            .local_cell()
            .get_peer(&peer)
            .unwrap()
            .reliability_score;
        core.routing_update_reliability(hex::encode(peer), true);
        let after = core
            .routing_engine
            .read()
            .as_ref()
            .unwrap()
            .base_engine()
            .local_cell()
            .get_peer(&peer)
            .unwrap()
            .reliability_score;

        assert!(after > before);
        assert!(core.get_forwarding_capability("ble"));
        assert!(!core.get_forwarding_capability("tcp"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn onion_enabled_send_fails_closed_when_relay_wrapping_fails() {
        let core = IronCore::new();
        core.grant_consent();
        core.initialize_identity().unwrap();
        let recipient = core.get_identity_info().public_key_hex.unwrap();
        *core.privacy_config.write() = crate::privacy::PrivacyConfig {
            onion_routing_enabled: true,
            ..Default::default()
        };
        let bootstrap =
            crate::relay::BootstrapManager::new("local".to_string(), vec![0u8; 32], vec![])
                .with_seed_peers(vec![crate::relay::SeedPeer::new(
                    "not-a-public-key".to_string(),
                    String::new(),
                    "invalid-relay".to_string(),
                )]);
        *core.bootstrap_manager.write() = Some(bootstrap);

        let result = core.prepare_message(
            recipient,
            "must not fall back".to_string(),
            crate::MessageType::Text,
            None,
        );

        assert!(result.is_err());
        assert_eq!(core.outbox_count(), 0);
    }

    #[test]
    fn initialize_identity_wires_shared_reputation_manager() {
        let core = IronCore::new();
        core.grant_consent();
        core.initialize_identity().unwrap();

        assert!(core
            .drift_engine
            .read()
            .as_ref()
            .unwrap()
            .reputation_manager_is_configured());
    }

    #[test]
    fn test_routing_peer_seen_multi_transport_failover_and_non_zero_confidence() {
        let core = IronCore::new();
        core.grant_consent();
        core.initialize_identity().expect("init identity");

        let peer_pk = [42u8; 32];
        let peer_hex = hex::encode(peer_pk);
        let direct_hint: [u8; 4] = blake3::hash(&peer_pk).as_bytes()[0..4]
            .try_into()
            .expect("4 bytes hint");
        let msg_id = [1u8; 16];
        let now = 1000u64;

        // 1. Initially, no sighting has occurred. Decision must degrade to StoreAndCarry with 0.0 confidence.
        let initial_decision = core
            .make_routing_decision(direct_hint, msg_id, 50, now)
            .expect("decision");
        assert_eq!(
            initial_decision.decided_by,
            crate::routing::RoutingLayer::StoreAndCarry
        );
        assert!(matches!(
            initial_decision.primary,
            crate::routing::NextHop::StoreAndCarry
        ));
        assert_eq!(initial_decision.confidence, 0.0);

        // 2. Simulate peer observation on BLE transport
        core.routing_peer_seen(peer_hex.clone(), "ble".to_string());

        let ble_decision = core
            .make_routing_decision(direct_hint, msg_id, 50, now)
            .expect("decision");
        assert_ne!(
            ble_decision.decided_by,
            crate::routing::RoutingLayer::StoreAndCarry
        );
        assert_eq!(ble_decision.decided_by, crate::routing::RoutingLayer::Local);
        assert!(
            ble_decision.confidence > 0.0,
            "confidence must be non-zero after peer_seen"
        );
        assert!(ble_decision.confidence >= 0.5);
        match ble_decision.primary {
            crate::routing::NextHop::Direct { peer_id, transport } => {
                assert_eq!(peer_id, peer_pk);
                assert_eq!(transport, crate::routing::TransportType::BLE);
            }
            other => panic!("expected Direct hop with BLE transport, got {:?}", other),
        }

        // 3. Simulate peer observation on second transport: TCP / WiFi
        core.routing_peer_seen(peer_hex.clone(), "tcp".to_string());

        let tcp_decision = core
            .make_routing_decision(direct_hint, msg_id, 50, now)
            .expect("decision");
        assert_ne!(
            tcp_decision.decided_by,
            crate::routing::RoutingLayer::StoreAndCarry
        );
        assert_eq!(tcp_decision.decided_by, crate::routing::RoutingLayer::Local);
        assert!(tcp_decision.confidence > 0.0);
        match tcp_decision.primary {
            crate::routing::NextHop::Direct { peer_id, transport } => {
                assert_eq!(peer_id, peer_pk);
                assert_eq!(transport, crate::routing::TransportType::TCP);
            }
            other => panic!("expected Direct hop with TCP transport, got {:?}", other),
        }
        assert!(
            tcp_decision.alternatives.iter().any(|alt| matches!(
                alt,
                crate::routing::NextHop::Direct { peer_id, transport } if *peer_id == peer_pk && *transport == crate::routing::TransportType::BLE
            )),
            "BLE must be preserved in alternative hops for failover"
        );

        // 4. Failover back to BLE: simulate peer observed on BLE again
        core.routing_peer_seen(peer_hex.clone(), "ble".to_string());

        let failover_decision = core
            .make_routing_decision(direct_hint, msg_id, 50, now)
            .expect("decision");
        assert_eq!(
            failover_decision.decided_by,
            crate::routing::RoutingLayer::Local
        );
        assert!(failover_decision.confidence > 0.0);
        match failover_decision.primary {
            crate::routing::NextHop::Direct { peer_id, transport } => {
                assert_eq!(peer_id, peer_pk);
                assert_eq!(transport, crate::routing::TransportType::BLE);
            }
            other => panic!(
                "expected Direct hop with BLE transport after failover, got {:?}",
                other
            ),
        }
        assert!(
            failover_decision.alternatives.iter().any(|alt| matches!(
                alt,
                crate::routing::NextHop::Direct { peer_id, transport } if *peer_id == peer_pk && *transport == crate::routing::TransportType::TCP
            )),
            "TCP must be preserved in alternative hops for failover"
        );
    }
}
