// scmessenger-cli — Complete Desktop CLI
//
// Cross-platform (macOS, Linux, Windows) command-line interface for SCMessenger.

#![allow(dead_code, unused)]

mod api;
mod ble_daemon;
pub mod ble_ids;
mod ble_mesh;
mod bootstrap;
mod config;
mod ledger;
mod server;
mod transport_api;
mod transport_bridge;

#[cfg(target_os = "windows")]
mod ble_windows;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use libp2p::{Multiaddr, PeerId};
use scmessenger_core::message::{decode_envelope, MessageType};
use scmessenger_core::store::{Contact, ContactManager, MessageDirection, Outbox, QueuedMessage};
use scmessenger_core::transport::abstraction::TransportType;
use scmessenger_core::transport::{self, SwarmEvent, SwarmHandle};
use scmessenger_core::wasm_support::rpc::{
    notif_delivery_status, notif_message_received, notif_peer_discovered, rpc_error, rpc_result,
    ClientIntent, DeliveryStatusParams, MeshTopologyUpdateParams, MessageReceivedParams,
    PeerDiscoveredParams,
};
use scmessenger_core::IronCore;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Convert a Path to a string, returning an error if the path contains invalid UTF-8.
/// This is safer than using .unwrap() which would panic on non-UTF-8 paths.
fn path_to_string(path: &std::path::Path) -> Result<String> {
    path.to_str()
        .ok_or_else(|| anyhow::anyhow!("Path contains invalid UTF-8: {}", path.display()))
        .map(|s| s.to_string())
}

/// Try to replace the port in a multiaddr with a new port.
/// This is used as a fallback mechanism when the stored port is stale.
fn try_replace_port(addr: &Multiaddr, new_port: u16) -> Option<Multiaddr> {
    let addr_str = addr.to_string();

    // Parse the multiaddr and replace the TCP port
    // Format: /ip4/X.X.X.X/tcp/PORT or /ip6/X:X:X:X/tcp/PORT
    let parts: Vec<&str> = addr_str.split('/').collect();

    let mut new_parts: Vec<String> = Vec::new();
    let mut replaced = false;

    for (i, part) in parts.iter().enumerate() {
        if *part == "tcp" && i + 1 < parts.len() {
            // This is a TCP port - try to replace it
            if parts[i + 1].parse::<u16>().is_ok() {
                new_parts.push(part.to_string());
                new_parts.push(new_port.to_string());
                replaced = true;
                // Skip the next part (original port)
                continue;
            }
        }
        new_parts.push(part.to_string());
    }

    if replaced {
        // Reconstruct the multiaddr, removing empty parts and joining with /
        let result: String = new_parts
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join("/");
        result.parse().ok()
    } else {
        None
    }
}

fn load_or_create_headless_network_keypair(
    storage_path: &std::path::Path,
    core: &IronCore,
) -> Result<libp2p::identity::Keypair> {
    std::fs::create_dir_all(storage_path).context("Failed to create relay storage directory")?;
    let key_path = storage_path.join("relay_network_key.pb");

    if key_path.exists() {
        let bytes = std::fs::read(&key_path).context("Failed to read relay network key file")?;
        match libp2p::identity::Keypair::from_protobuf_encoding(&bytes) {
            Ok(keypair) => return Ok(keypair),
            Err(err) => {
                tracing::warn!(
                    "Relay network key decode failed ({}); rotating key file: {}",
                    err,
                    key_path.display()
                );
            }
        }
    }

    // Key file absent or corrupt — try migrating from IronCore identity to
    // preserve the relay PeerId across upgrades.
    if let Ok(keypair) = core.get_libp2p_keypair() {
        tracing::info!("Migrating relay network key from existing IronCore identity");
        if let Ok(encoded) = keypair.to_protobuf_encoding() {
            if let Err(e) = std::fs::write(&key_path, &encoded) {
                tracing::warn!("Failed to persist migrated relay key: {}", e);
            } else {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ =
                        std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600));
                }
            }
        }
        return Ok(keypair);
    }

    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let encoded = keypair
        .to_protobuf_encoding()
        .context("Failed to encode relay network key")?;
    std::fs::write(&key_path, &encoded).context("Failed to persist relay network key")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600));
    }

    Ok(keypair)
}

const VERSION_INFO: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("SCM_GIT_HASH"),
    " ",
    env!("SCM_BUILD_TIME"),
    ")"
);

#[derive(Parser)]
#[command(name = "scm")]
#[command(about = "SCMessenger — Sovereign Encrypted Messaging", long_about = None)]
#[command(version = VERSION_INFO)]
struct Cli {
    /// Bind address for an optional HTTP health-check server (e.g. 0.0.0.0:8080).
    /// Intended for cloud relay deployments (AWS/Alibaba) that need external
    /// health monitoring; off by default.
    #[arg(long, value_name = "ADDR", global = true)]
    http_bind: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[allow(clippy::disallowed_methods)] // serde_json::json! expands to unwrap() calls internally
async fn spawn_http_health_server(bind_addr: String) {
    let addr: std::net::SocketAddr = match bind_addr.parse() {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!("Invalid --http-bind address '{}': {}", bind_addr, e);
            return;
        }
    };
    let app = axum::Router::new().route(
        "/health",
        axum::routing::get(|| async { axum::Json(serde_json::json!({"status": "healthy"})) }),
    );
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("Failed to bind HTTP health server on {}: {}", addr, e);
            return;
        }
    };
    tracing::info!("HTTP health server listening on {}", addr);
    if let Err(e) = axum::serve(listener, app).await {
        tracing::warn!("HTTP health server error: {}", e);
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize new identity
    Init {
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Show identity information
    Identity {
        #[command(subcommand)]
        action: Option<IdentityAction>,
    },
    /// Manage contacts
    Contact {
        #[command(subcommand)]
        action: ContactAction,
    },
    /// Configure settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// View message history
    History {
        #[arg(short, long)]
        peer: Option<String>,
        #[arg(short, long)]
        search: Option<String>,
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// Start P2P messaging node
    Start {
        #[arg(short, long)]
        port: Option<u16>,
        /// Send one bounded acknowledgement for each unique incoming text message.
        /// Test-harness capability: without it a CLI node can receive but never
        /// respond, so it can only ever demonstrate one direction of a pair.
        /// Also enabled by setting SCM_AUTO_REPLY=1.
        #[arg(long)]
        auto_reply: bool,
    },
    /// Run headless relay node (no interactive console)
    Relay {
        /// P2P listen multiaddr (default: /ip4/0.0.0.0/tcp/0)
        #[arg(short, long, default_value = "/ip4/0.0.0.0/tcp/0")]
        listen: String,
        /// HTTP status/landing page port (default: 9000)
        #[arg(long, default_value = "9000")]
        http_port: u16,
        /// Node name for logging/status (default: auto from peer ID)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Send a message (offline mode)
    Send { recipient: String, message: String },
    /// Show network status
    Status,
    /// Mark an outbox message as delivered/sent
    MarkSent { message_id: String },
    /// Clear all local history records
    HistoryClear {
        /// Required confirmation flag
        #[arg(long)]
        yes: bool,
    },
    /// Keep only the newest N history messages
    HistoryEnforceRetention { max_messages: u32 },
    /// Remove history older than a unix timestamp (seconds)
    HistoryPruneBefore { before_timestamp: u64 },
    /// Stop the running node
    Stop,
    /// Manage peer blocking
    Block {
        #[command(subcommand)]
        action: BlockAction,
    },
    /// Get a single history message by ID
    HistoryGet { id: String },
    /// Show history statistics
    HistoryStats,
    /// Show total history message count
    HistoryCount,
    /// Mark a history message as delivered
    HistoryMarkDelivered { id: String },
    /// Clear conversation history with a specific peer
    HistoryClearConversation { peer: String },
    /// Remove an entire conversation (alias for clear-conversation)
    HistoryRemoveConversation { peer: String },
    /// Delete a single history message by ID
    HistoryDelete { id: String },
    /// Run self-tests
    Test,
    /// Manage audit log
    Audit {
        #[command(subcommand)]
        action: AuditAction,
    },
    /// Swarm management commands
    Swarm {
        #[command(subcommand)]
        action: SwarmAction,
    },
    /// Manage local network discovery (BLE, mDNS, WiFi-Aware)
    Discovery {
        #[command(subcommand)]
        action: DiscoveryAction,
    },
    /// Host APK for remote QR code scan-to-install on the local network
    ShareApk {
        /// Optional path to APK (default: search local build outputs)
        #[arg(short, long)]
        apk_path: Option<String>,
        /// Duration in minutes to host the server
        #[arg(short, long, default_value = "15")]
        duration_mins: u64,
        /// HTTP port to listen on (falls back to an ephemeral port)
        #[arg(short, long)]
        port: Option<u16>,
    },
}

#[derive(Subcommand)]
enum AuditAction {
    /// Export the entire audit log as JSON
    Export {
        /// Optional output file path
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Verify the cryptographic integrity of the audit log
    Verify,
    /// Show a summary of audit log statistics
    Stats,
}

#[derive(Subcommand)]
enum BlockAction {
    /// Block a peer
    Add {
        peer_id: String,
        #[arg(short, long)]
        device_id: Option<String>,
        #[arg(short, long)]
        reason: Option<String>,
    },
    /// Unblock a peer
    Remove {
        peer_id: String,
        #[arg(short, long)]
        device_id: Option<String>,
    },
    /// Block a peer AND delete all their stored messages (cascade purge)
    Delete {
        peer_id: String,
        #[arg(short, long)]
        device_id: Option<String>,
        #[arg(short, long)]
        reason: Option<String>,
    },
    /// List all blocked peers
    List,
    /// Check if a peer is blocked
    Check {
        peer_id: String,
        #[arg(short, long)]
        device_id: Option<String>,
    },
    /// Show total blocked count
    Count,
}

#[derive(Subcommand)]
enum IdentityAction {
    Show,
    Export {
        /// Passphrase to encrypt the backup
        #[arg(short, long)]
        passphrase: String,
        /// Optional output file path for backup payload
        #[arg(short, long)]
        output: Option<String>,
    },
    Import {
        /// Passphrase to decrypt the backup
        #[arg(short, long)]
        passphrase: String,
        /// Backup payload string
        #[arg(long, conflicts_with = "input")]
        backup: Option<String>,
        /// Read backup payload from file
        #[arg(short = 'i', long)]
        input: Option<String>,
    },
    SetName {
        name: String,
    },
    /// Show the local device ID (WS13)
    DeviceId,
    /// Show the seniority timestamp (WS13)
    Seniority,
    /// Show registration state for an identity
    RegistrationState {
        identity_id: String,
    },
    /// Sign arbitrary data with the local identity key
    SignData {
        /// Hex-encoded data to sign
        data_hex: String,
    },
    /// Verify a signature against data and a public key
    VerifySignature {
        /// Hex-encoded data
        data_hex: String,
        /// Hex-encoded signature
        signature_hex: String,
        /// Hex-encoded Ed25519 public key
        public_key_hex: String,
    },
}

#[derive(Subcommand)]
enum ContactAction {
    Add {
        peer_id: String,
        public_key: String,
        #[arg(short, long)]
        name: Option<String>,
    },
    List,
    Show {
        contact: String,
    },
    Remove {
        contact: String,
    },
    Search {
        query: String,
    },
    SetLocalNickname {
        contact: String,
        nickname: Option<String>,
        #[arg(long)]
        clear: bool,
    },
    /// Set the federated (broadcast) nickname for a contact
    SetNickname {
        contact: String,
        nickname: Option<String>,
        #[arg(long)]
        clear: bool,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    Set {
        key: String,
        value: String,
    },
    Get {
        key: String,
    },
    List,
    /// Manage privacy-enhancing features (onion routing, padding, etc.)
    Privacy {
        /// Enable/disable message padding
        #[arg(short, long)]
        padding: Option<bool>,
        /// Enable/disable onion routing
        #[arg(short, long)]
        onion: Option<bool>,
        /// Enable/disable cover traffic
        #[arg(short, long)]
        cover: Option<bool>,
        /// Enable/disable timing obfuscation (jitter)
        #[arg(short, long)]
        timing: Option<bool>,
    },
}

#[derive(Subcommand)]
enum SwarmAction {
    /// Show swarm connection statistics
    Stats,
}

#[derive(Subcommand)]
enum DiscoveryAction {
    /// Show current discovery transport status
    Status,
    /// Trigger an active discovery scan
    Scan,
    /// List peers discovered via local transports
    Peers,
}

/// Current Unix timestamp in seconds.
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// TTL for relay-circuit health. libp2p relay client renews reservations
/// periodically (new OutboundCircuitEstablished events refresh the timestamp);
/// a 10-minute TTL bounds the stale-healthy window when the relay dies silently.
const RELAY_HEALTHY_TTL_SECS: u64 = 600;

/// Pure helper for testing: is a relay established-at timestamp healthy?
fn relay_healthy_from_ts(established_at: u64, now: u64) -> bool {
    established_at != 0 && now.saturating_sub(established_at) < RELAY_HEALTHY_TTL_SECS
}

/// Keep relay fallback eligible even when the ledger knows the circuit's
/// target PeerId. A `DialKey::Peer` is intentionally suppressed while a relay
/// is healthy; circuit paths must instead retain their circuit classification
/// because they are the fallback path being selected.
fn scheduler_dial_key(addr_str: &str, peer_id: Option<PeerId>) -> ledger::DialKey {
    if addr_str.contains("/p2p-circuit") {
        ledger::DialKey::Addr(ledger::strip_peer_id(addr_str))
    } else {
        ledger::DialKey::for_target(addr_str, peer_id)
    }
}

/// Fire-and-forget outbound dial scheduler.
///
/// Enforces per-peer backoff and limits concurrent outbound dials to unknown
/// peers. Once the relay path reports healthy, direct dials to unknown peers
/// are suppressed in favor of relay circuits.
#[derive(Clone)]
struct DialScheduler {
    ledger: Arc<tokio::sync::Mutex<ledger::ConnectionLedger>>,
    swarm: transport::SwarmHandle,
    unknown_dial_sem: Arc<tokio::sync::Semaphore>,
    relay_established_at: Arc<std::sync::atomic::AtomicU64>,
}

impl DialScheduler {
    fn new(
        ledger: Arc<tokio::sync::Mutex<ledger::ConnectionLedger>>,
        swarm: transport::SwarmHandle,
    ) -> Self {
        Self {
            ledger,
            swarm,
            unknown_dial_sem: Arc::new(tokio::sync::Semaphore::new(3)),
            relay_established_at: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    fn set_relay_healthy(&self, healthy: bool) {
        let ts = if healthy { now_secs() } else { 0 };
        self.relay_established_at
            .store(ts, std::sync::atomic::Ordering::Relaxed);
    }

    fn relay_is_healthy(&self) -> bool {
        let ts = self
            .relay_established_at
            .load(std::sync::atomic::Ordering::Relaxed);
        relay_healthy_from_ts(ts, now_secs())
    }

    fn dial(&self, addr_str: String, peer_id_opt: Option<PeerId>) {
        let ledger = Arc::clone(&self.ledger);
        let swarm = self.swarm.clone();
        let sem = Arc::clone(&self.unknown_dial_sem);
        let scheduler = self.clone();

        tokio::spawn(async move {
            let key = scheduler_dial_key(&addr_str, peer_id_opt);

            // Optimistic unknown-class check without holding the ledger lock.
            let is_unknown = {
                let l = ledger.lock().await;
                match l.dial_state(&key) {
                    Some(state) => !state.is_known_good,
                    None => true,
                }
            };

            let permit: Option<tokio::sync::OwnedSemaphorePermit> = if is_unknown {
                match sem.try_acquire_owned() {
                    Ok(p) => Some(p),
                    Err(_) => {
                        tracing::debug!(
                            "Dial to {} skipped: max concurrent unknown dials reached",
                            addr_str
                        );
                        return;
                    }
                }
            } else {
                None
            };

            let now = now_secs();
            let allowed = {
                let mut l = ledger.lock().await;
                l.try_begin_dial(key.clone(), now, scheduler.relay_is_healthy())
            };
            if !allowed {
                drop(permit);
                return;
            }

            // Preserve the complete address, especially the target component
            // after `/p2p-circuit`. The core needs the relay hop and explicit
            // target identity to apply known-peer dial conditions without
            // turning a relay path into an address-only/Always dial.
            let addr = match addr_str.parse::<Multiaddr>() {
                Ok(a) => a,
                Err(e) => {
                    tracing::error!("Invalid multiaddr: {} - {}", addr_str, e);
                    let mut l = ledger.lock().await;
                    l.complete_dial(&key, false, now, None);
                    drop(permit);
                    return;
                }
            };

            tokio::spawn(async move {
                let result = match peer_id_opt {
                    Some(peer_id) => swarm.dial_peer(peer_id, addr).await,
                    None => swarm.dial(addr).await,
                };
                let now2 = now_secs();
                let mut l = ledger.lock().await;
                match result {
                    Ok(_) => {
                        // Credit the connection to the Peer slot so the
                        // per-peer concurrent-connection cap in the ledger
                        // sees it. `record_disconnect` (fired per
                        // SwarmEvent::PeerDisconnected) is the matching
                        // release. For Addr keys the learned id is not
                        // known; the ledger treats those as transient.
                        let learned = match &key {
                            ledger::DialKey::Peer(pid) => Some(*pid),
                            ledger::DialKey::Addr(_) => None,
                        };
                        l.complete_dial(&key, true, now2, learned);
                    }
                    Err(_) => {
                        l.record_failure(&addr_str);
                        l.complete_dial(&key, false, now2, None);
                    }
                }
                // OwnedSemaphorePermit drops here, releasing the concurrent-dial slot.
            });
        });
    }
}

#[cfg(test)]
mod dial_scheduler_tests {
    use super::*;

    #[test]
    fn test_relay_healthy_from_ts_never_established() {
        assert!(!relay_healthy_from_ts(0, 1_000_000));
    }

    #[test]
    fn test_relay_healthy_from_ts_recent() {
        assert!(relay_healthy_from_ts(1_000_000, 1_000_100));
    }

    #[test]
    fn test_relay_healthy_from_ts_stale() {
        assert!(!relay_healthy_from_ts(1_000_000, 1_000_601));
    }

    #[test]
    fn circuit_scheduler_key_retains_relay_fallback_classification() {
        let target = PeerId::random();
        let addr = format!(
            "/ip4/192.0.2.1/tcp/443/p2p/{}/p2p-circuit/p2p/{}",
            PeerId::random(),
            target
        );

        assert!(matches!(
            scheduler_dial_key(&addr, Some(target)),
            ledger::DialKey::Addr(key) if key.contains("/p2p-circuit")
        ));
    }

    #[test]
    fn auto_reply_is_limited_to_one_ack_per_message_id() {
        let mut seen_ids = HashSet::new();
        let mut seen_order = VecDeque::new();

        assert!(should_send_auto_reply(
            "message-1",
            "stop the spam",
            &mut seen_ids,
            &mut seen_order
        ));
        assert!(!should_send_auto_reply(
            "message-1",
            "stop the spam",
            &mut seen_ids,
            &mut seen_order
        ));
        assert!(should_send_auto_reply(
            "message-2",
            "a different message",
            &mut seen_ids,
            &mut seen_order
        ));
    }

    #[test]
    fn auto_reply_never_answers_machine_messages() {
        let mut seen_ids = HashSet::new();
        let mut seen_order = VecDeque::new();

        assert!(!should_send_auto_reply(
            "machine-message",
            AUTO_REPLY_ACK,
            &mut seen_ids,
            &mut seen_order
        ));
        assert!(seen_ids.is_empty());
    }

    /// SELF-CERTIFYING KEY BINDING: an identity envelope without a usable
    /// public key must create a placeholder contact with an EMPTY key (plus a
    /// notes annotation), never peer_id-as-public_key.
    #[test]
    fn envelope_learning_without_key_stores_placeholder_not_peer_id() {
        let dir = tempfile::tempdir().unwrap();
        let core = IronCore::with_storage(path_to_string(&dir.path().join("storage")).unwrap());
        let contacts = core.contacts_store_manager();

        let unknown_peer = "12D3KooWEfZ2fJ8AcGvVfEUi2wFQPo6z8kZVr5TsgP7JQF2B9kS1";
        let decoded = scmessenger_core::message::identity_envelope::DecodedIdentityEnvelope {
            nickname: Some("Alice".to_string()),
            public_key: Some("zz-not-hex".to_string()),
            ..Default::default()
        };

        learn_sender_identity_from_envelope(&contacts, unknown_peer, &decoded);

        let stored = contacts
            .get(unknown_peer.to_string())
            .unwrap()
            .expect("contact");
        assert_eq!(stored.nickname.as_deref(), Some("Alice"));
        assert!(
            stored.public_key.is_empty(),
            "placeholder key must be empty, not poisoned"
        );
        assert_ne!(stored.public_key, unknown_peer);
        assert!(stored
            .notes
            .as_deref()
            .unwrap_or_default()
            .contains("awaiting verified key"));
    }

    /// A signed envelope carrying a valid Ed25519 key backfills the contact
    /// (signed-source corroboration); the binding must survive restarts.
    #[test]
    fn envelope_learning_with_valid_key_backfills_contact() {
        let dir = tempfile::tempdir().unwrap();
        let core = IronCore::with_storage(path_to_string(&dir.path().join("storage")).unwrap());
        let contacts = core.contacts_store_manager();

        let mut seed = [0u8; 32];
        seed[..10].copy_from_slice(b"cli-env-kp");
        let signing = libp2p::identity::ed25519::SecretKey::try_from_bytes(&mut seed).unwrap();
        let kp = libp2p::identity::ed25519::Keypair::from(signing);
        let key_hex = hex::encode(kp.public().to_bytes());
        let peer_id = libp2p::identity::PublicKey::from(kp.public())
            .to_peer_id()
            .to_string();

        // Pre-existing placeholder record (legacy poison shape) gets repaired.
        contacts
            .add(Contact::new(peer_id.clone(), peer_id.clone()))
            .unwrap();

        let decoded = scmessenger_core::message::identity_envelope::DecodedIdentityEnvelope {
            nickname: Some("Bob".to_string()),
            public_key: Some(key_hex.clone()),
            ..Default::default()
        };
        learn_sender_identity_from_envelope(&contacts, &peer_id, &decoded);

        let stored = contacts.get(peer_id.clone()).unwrap().expect("contact");
        assert_eq!(stored.public_key.to_lowercase(), key_hex);
        assert_ne!(stored.public_key.to_lowercase(), peer_id);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Determine data directory early for logging
    let data_dir = config::Config::data_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let log_dir = data_dir.join("logs");
    std::fs::create_dir_all(&log_dir).ok();

    // 2. Setup file logging (rolling hourly, keeping last 24h)
    let file_appender = tracing_appender::rolling::hourly(&log_dir, "scm.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 3. Initialize tracing with both stdout (fmt) and file (appender)
    use tracing_subscriber::prelude::*;
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking),
        )
        .init();

    tracing::info!(
        "SCMessenger CLI starting up... (CLI Version: {})",
        VERSION_INFO
    );
    tracing::info!(
        "Core Provenance: {}",
        scmessenger_core::get_build_provenance()
    );
    tracing::info!("Log directory: {}", log_dir.display());

    // 4. Prune old logs (keep last 7 days)
    if let Err(e) = prune_logs(&log_dir, 7) {
        tracing::warn!("Failed to prune old logs: {}", e);
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => cmd_init(name).await,
        Commands::Identity { action } => cmd_identity(action).await,
        Commands::Contact { action } => cmd_contact(action).await,
        Commands::Config { action } => cmd_config(action).await,
        Commands::History {
            peer,
            search,
            limit,
        } => cmd_history(peer, search, limit).await,
        Commands::Start { port, auto_reply } => cmd_start(port, cli.http_bind, auto_reply).await,
        Commands::Relay {
            listen,
            http_port,
            name,
        } => cmd_relay(listen, http_port, name, cli.http_bind).await,
        Commands::Stop => cmd_stop().await,
        Commands::Send { recipient, message } => cmd_send_offline(recipient, message).await,
        Commands::Status => cmd_status().await,
        Commands::MarkSent { message_id } => cmd_mark_sent(message_id).await,
        Commands::HistoryClear { yes } => cmd_history_clear(yes).await,
        Commands::HistoryEnforceRetention { max_messages } => {
            cmd_history_enforce_retention(max_messages).await
        }
        Commands::HistoryPruneBefore { before_timestamp } => {
            cmd_history_prune_before(before_timestamp).await
        }
        Commands::Block { action } => cmd_block(action).await,
        Commands::HistoryGet { id } => cmd_history_get(id).await,
        Commands::HistoryStats => cmd_history_stats().await,
        Commands::HistoryCount => cmd_history_count().await,
        Commands::HistoryMarkDelivered { id } => cmd_history_mark_delivered(id).await,
        Commands::HistoryClearConversation { peer }
        | Commands::HistoryRemoveConversation { peer } => {
            cmd_history_clear_conversation(peer).await
        }
        Commands::HistoryDelete { id } => cmd_history_delete(id).await,
        Commands::Test => cmd_test().await,
        Commands::Audit { action } => cmd_audit(action).await,
        Commands::Swarm { action } => cmd_swarm(action).await,
        Commands::Discovery { action } => cmd_discovery(action).await,
        Commands::ShareApk {
            apk_path,
            duration_mins,
            port,
        } => cmd_share_apk(apk_path, duration_mins, port).await,
    }
}

/// Host an APK for a bounded local-network download session.
///
/// The URL intentionally contains no guessed bootstrap address. A missing
/// bootstrap hint is recoverable; advertising a stale or fabricated address
/// causes a silent join failure.
async fn cmd_share_apk(
    apk_path: Option<String>,
    duration_mins: u64,
    requested_port: Option<u16>,
) -> Result<()> {
    let target_file = resolve_apk_path(apk_path)?;
    let file_size = target_file.metadata()?.len();

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, requested_port.unwrap_or(8080)))
        .or_else(|_| TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0)))
        .context("Failed to bind TCP listener for APK server")?;
    let bound_port = listener.local_addr()?.port();
    let local_ip = get_local_ipv4().unwrap_or(Ipv4Addr::LOCALHOST);
    let download_url = format!("http://{}:{}/scmessenger.apk", local_ip, bound_port);

    println!("============================================================");
    println!(" [OK] SCMessenger Node Ephemeral APK Host Active");
    println!("============================================================");
    println!(" APK Source:    {}", target_file.display());
    println!(" File Size:     {} bytes", file_size);
    println!(" Download URL:  {}", download_url);
    println!(" Bootstrap:     not advertised (provide a verified hint separately)");
    println!(" Duration:      {} minutes", duration_mins);
    println!("============================================================");
    println!(" Scan QR or enter the URL on the receiving phone:");
    render_terminal_url_box(&download_url);
    println!(" Press Ctrl+C to stop sharing manually.");
    println!("============================================================");

    let is_running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&is_running);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(duration_mins * 60)).await;
        running_clone.store(false, Ordering::SeqCst);
        println!("\\n[INFO] Share timeframe expired. Shutting down HTTP host.");
    });

    listener.set_nonblocking(true)?;
    while is_running.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((mut stream, addr)) => {
                println!("[INFO] Client connected from {}", addr);
                if let Err(error) = serve_apk_stream(&mut stream, &target_file, file_size) {
                    eprintln!("[WARNING] APK download failed: {}", error);
                }
            }
            Err(ref error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(error) => {
                eprintln!("[WARNING] Error accepting connection: {}", error);
                break;
            }
        }
    }

    println!("[OK] APK host server stopped cleanly.");
    Ok(())
}

fn resolve_apk_path(provided: Option<String>) -> Result<PathBuf> {
    if let Some(path) = provided {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            return Ok(path_buf);
        }
    }

    for candidate in [
        PathBuf::from("android/app/build/outputs/apk/debug/app-debug.apk"),
        PathBuf::from("android/app/build/outputs/apk/release/app-release.apk"),
        PathBuf::from("target/release/app-release.apk"),
    ] {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    anyhow::bail!("No APK file found. Build the APK first or pass --apk-path <path>")
}

fn get_local_ipv4() -> Option<Ipv4Addr> {
    let socket = std::net::UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.connect((Ipv4Addr::new(8, 8, 8, 8), 80)).ok()?;
    match socket.local_addr().ok()?.ip() {
        std::net::IpAddr::V4(ip) if !ip.is_loopback() => Some(ip),
        _ => None,
    }
}

/// Return concrete local transport addresses for self-dial filtering.
///
/// libp2p commonly reports a wildcard listener (`0.0.0.0`) even though peers
/// learn the concrete LAN address through Identify.  Comparing the wildcard
/// against that learned address does not prevent a self-dial, so expand the
/// wildcard using the active local interface and include any consensus
/// external addresses the swarm has observed.
async fn get_local_transport_addresses(swarm: &SwarmHandle) -> Vec<String> {
    let bound = swarm.get_bound_addresses().await.unwrap_or_default();
    let external = swarm.get_external_addresses().await.unwrap_or_default();
    let local_ipv4 = get_local_ipv4();
    let mut addresses: Vec<String> = Vec::new();

    for address in bound {
        let value = address.to_string();
        if !addresses.contains(&value) {
            addresses.push(value.clone());
        }
        if let Some(ip) = local_ipv4 {
            let concrete = value.replacen("/ip4/0.0.0.0/", &format!("/ip4/{ip}/"), 1);
            if concrete != value && !addresses.contains(&concrete) {
                addresses.push(concrete);
            }
        }
    }

    for socket in external {
        if socket.ip().is_unspecified() || socket.port() == 0 {
            continue;
        }
        let protocol = match socket.ip() {
            std::net::IpAddr::V4(ip) => format!("/ip4/{ip}"),
            std::net::IpAddr::V6(ip) => format!("/ip6/{ip}"),
        };
        let value = format!("{protocol}/tcp/{}", socket.port());
        if !addresses.contains(&value) {
            addresses.push(value);
        }
    }

    addresses
}

fn serve_apk_stream(stream: &mut TcpStream, file_path: &PathBuf, file_size: u64) -> Result<()> {
    let mut request = [0u8; 1024];
    let _ = stream.read(&mut request);
    let header = format!(
        "HTTP/1.1 200 OK\\r\\n\\
         Content-Type: application/vnd.android.package-archive\\r\\n\\
         Content-Length: {}\\r\\n\\
         Content-Disposition: attachment; filename=\\\"scmessenger-v0.4.0.apk\\\"\\r\\n\\
         Connection: close\\r\\n\\r\\n",
        file_size
    );
    stream.write_all(header.as_bytes())?;

    let mut file = File::open(file_path)?;
    let mut chunk = [0u8; 8192];
    loop {
        let read = file.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        stream.write_all(&chunk[..read])?;
    }
    stream.flush()?;
    println!("[OK] Served full APK download successfully.");
    Ok(())
}

fn render_terminal_url_box(url: &str) {
    let border = "-".repeat(url.len() + 6);
    println!("+{}", border);
    println!("|  {}  |", url);
    println!("+{}", border);
}

async fn cmd_stop() -> Result<()> {
    if !api::is_api_available().await {
        println!("{}", "No SCMessenger node is running.".yellow());
        return Ok(());
    }

    print!("Stopping SCMessenger node... ");
    match api::stop_node_via_api().await {
        Ok(_) => println!("{}", "Done.".green()),
        Err(e) => println!("{} {}", "Error:".red(), e),
    }
    Ok(())
}

async fn cmd_init(name: Option<String>) -> Result<()> {
    println!("{}", "Initializing SCMessenger...".bold());
    println!();

    let config = config::Config::load()?;
    println!("  {} Configuration", "[OK]".green());

    let data_dir = config::Config::data_dir()?;
    println!(
        "  {} Data directory: {}",
        "[OK]".green(),
        data_dir.display()
    );

    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    let info = core.get_identity_info();
    let already_initialized = info.initialized;

    core.grant_consent();
    core.initialize_identity()
        .context("Failed to initialize identity")?;

    // Set nickname if provided
    if let Some(nickname) = name {
        core.set_nickname(nickname)
            .context("Failed to set nickname")?;
        println!("  {} Nickname set", "[OK]".green());
    }

    if already_initialized {
        println!("  {} Existing identity loaded", "[OK]".green());
    } else {
        println!("  {} Identity created", "[OK]".green());
    }
    println!();

    print_full_identity(&core, &config)?;

    println!();
    println!("{}", "Next steps:".bold());
    println!(
        "  • Add contacts: {}",
        "scm contact add <peer-id> <public-key> --name <nickname>".bright_green()
    );
    println!("  • Start node:   {}", "scm start".bright_green());

    Ok(())
}

async fn cmd_identity(action: Option<IdentityAction>) -> Result<()> {
    let config = config::Config::load()?;

    // If running with default data directory and API is available (node is running),
    // prefer querying the live node's API to avoid sled lock contention.
    let is_custom_data_dir = std::env::var("SCMESSENGER_DATA_DIR").is_ok();
    if !is_custom_data_dir
        && api::is_api_available().await
        && matches!(action, None | Some(IdentityAction::Show))
    {
        match api::get_identity_via_api().await {
            Ok(info) => {
                print_api_identity(&info, &config)?;
                return Ok(());
            }
            Err(e) => {
                tracing::warn!("Failed to query identity from running node API: {}", e);
            }
        }
    }

    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);

    let info = core.get_identity_info();
    if !info.initialized {
        let relay_key_path = storage_path.join("relay_network_key.pb");
        if relay_key_path.exists() {
            if let Ok(bytes) = std::fs::read(&relay_key_path) {
                if let Ok(keypair) = libp2p::identity::Keypair::from_protobuf_encoding(&bytes) {
                    let local_peer_id = keypair.public().to_peer_id();
                    println!("{}", "Identity Information (Headless Relay)".bold());
                    println!(
                        "  Peer ID (Network):      {}",
                        local_peer_id.to_string().bright_cyan()
                    );
                    println!(
                        "  Public Key:             {}",
                        "(headless/identity-agnostic)".bright_yellow()
                    );
                    return Ok(());
                }
            }
        }

        println!(
            "{} No identity found for data directory: {}",
            "[FAIL]".red(),
            data_dir.display()
        );
        println!(
            "Run '{}' to initialize a new identity.",
            "scm init".bright_green()
        );
        return Ok(());
    }

    core.grant_consent();
    core.initialize_identity()
        .context("Failed to load identity")?;

    match action {
        None | Some(IdentityAction::Show) => {
            print_full_identity(&core, &config)?;
        }
        Some(IdentityAction::SetName { name }) => {
            core.set_nickname(name.clone())
                .context("Failed to set nickname")?;
            println!(
                "{} Nickname updated to: {}",
                "[OK]".green(),
                name.bright_cyan()
            );
        }
        Some(IdentityAction::Export { passphrase, output }) => {
            let backup = core
                .export_identity_backup(passphrase)
                .context("Failed to export identity backup")?;
            let info = core.get_identity_info();

            println!("{}", "Export Identity Backup".bold());
            println!();
            println!(
                "{}",
                "[WARN]  WARNING: backup payload contains private key material."
                    .bright_red()
                    .bold()
            );
            println!("Identity ID: {}", info.identity_id.unwrap_or_default());
            println!("Public Key:  {}", info.public_key_hex.unwrap_or_default());
            println!("Payload size: {} bytes", backup.len());
            println!();

            if let Some(path) = output {
                std::fs::write(&path, &backup)
                    .with_context(|| format!("Failed to write backup file: {}", path))?;
                println!(
                    "{} Backup written to {}",
                    "[OK]".green(),
                    path.bright_cyan()
                );
            } else {
                println!("{}", "Backup payload:".bold());
                println!("{}", backup);
            }
        }
        Some(IdentityAction::Import {
            passphrase,
            backup,
            input,
        }) => {
            let payload = if let Some(path) = input {
                std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read backup file: {}", path))?
            } else if let Some(raw) = backup {
                raw
            } else {
                anyhow::bail!("Provide --backup <payload> or --input <file>");
            };

            core.import_identity_backup(payload, passphrase)
                .context("Failed to import identity backup")?;
            let info = core.get_identity_info();
            println!("{}", "[OK] Identity backup imported".green());
            println!(
                "  Identity ID: {}",
                info.identity_id.unwrap_or_default().bright_cyan()
            );
            println!(
                "  Public Key:  {}",
                info.public_key_hex.unwrap_or_default().bright_yellow()
            );
        }
        Some(IdentityAction::DeviceId) => match core.get_device_id() {
            Some(id) => println!("Device ID: {}", id.bright_cyan()),
            None => println!(
                "{}",
                "No device ID available (identity not initialized?)".dimmed()
            ),
        },
        Some(IdentityAction::Seniority) => match core.get_seniority_timestamp() {
            Some(ts) => println!("Seniority Timestamp: {} ({})", ts, format_timestamp(ts)),
            None => println!("{}", "No seniority timestamp available".dimmed()),
        },
        Some(IdentityAction::RegistrationState { identity_id }) => {
            let state = core.get_registration_state(identity_id.clone());
            println!("{}", "Registration State".bold());
            println!("  Identity:   {}", identity_id.bright_cyan());
            println!("  State:      {}", state.state);
            if let Some(device_id) = state.device_id {
                println!("  Device ID:  {}", device_id);
            }
            if let Some(ts) = state.seniority_timestamp {
                println!("  Seniority:  {} ({})", ts, format_timestamp(ts));
            }
        }
        Some(IdentityAction::SignData { data_hex }) => {
            let data = hex::decode(&data_hex).context("Invalid hex data")?;
            let result = core.sign_data(data).context("Failed to sign data")?;
            println!("{}", "Signature Result".bold());
            println!(
                "  Signature:  {}",
                hex::encode(&result.signature).bright_yellow()
            );
            println!("  Public Key: {}", result.public_key_hex.bright_cyan());
        }
        Some(IdentityAction::VerifySignature {
            data_hex,
            signature_hex,
            public_key_hex,
        }) => {
            let data = hex::decode(&data_hex).context("Invalid hex data")?;
            let signature = hex::decode(&signature_hex).context("Invalid hex signature")?;
            let valid = core
                .verify_signature(data, signature, public_key_hex)
                .context("Failed to verify signature")?;
            if valid {
                println!("{} Signature is valid", "[OK]".green());
            } else {
                println!("{} Signature is INVALID", "[FAIL]".red());
            }
        }
    }

    Ok(())
}

fn print_full_identity(core: &IronCore, config: &config::Config) -> Result<()> {
    let info = core.get_identity_info();
    // Derive PeerId from identity
    let network_keypair = core
        .get_libp2p_keypair()
        .context("Failed to get network keypair")?;
    let local_peer_id = network_keypair.public().to_peer_id();

    println!("{}", "Identity Information".bold());
    println!(
        "  ID:                     {}",
        info.identity_id
            .as_deref()
            .unwrap_or("(pending)")
            .bright_cyan()
    );
    println!(
        "  Peer ID (Network):      {}",
        local_peer_id.to_string().bright_cyan()
    );
    println!(
        "  Nickname:               {}",
        info.nickname
            .as_deref()
            .unwrap_or("(not set)")
            .bright_cyan()
    );
    println!(
        "  Public Key:             {}",
        info.public_key_hex
            .as_deref()
            .unwrap_or("(none)")
            .bright_yellow()
    );
    println!();

    println!("{}", "Direct Connection Info".bold());
    let ws_port = if config.listen_port == 0 {
        9000
    } else {
        config.listen_port
    };
    let p2p_port = ws_port + 1;

    // Show P2P listening address
    println!(
        "  P2P Listener:           {}",
        format!("/ip4/0.0.0.0/tcp/{}", p2p_port).green()
    );

    // Show examples for LAN/Localhost
    println!("  Local Address:          /ip4/127.0.0.1/tcp/{}", p2p_port);

    // Attempt to show LAN IP if possible (simple heuristic or just mention it)
    println!(
        "  LAN Address:            /ip4/<YOUR_LAN_IP>/tcp/{}",
        p2p_port
    );

    println!();

    Ok(())
}

fn print_api_identity(info: &serde_json::Value, config: &config::Config) -> Result<()> {
    let identity_id = info["identity_id"].as_str().unwrap_or("(pending)");
    let peer_id = info["libp2p_peer_id"].as_str().unwrap_or("(unknown)");
    let nickname = info["nickname"].as_str().unwrap_or("(not set)");
    let public_key = info["public_key_hex"]
        .as_str()
        .unwrap_or("(headless/identity-agnostic)");

    println!("{}", "Identity Information (from running node)".bold());
    println!("  ID:                     {}", identity_id.bright_cyan());
    println!("  Peer ID (Network):      {}", peer_id.bright_cyan());
    println!("  Nickname:               {}", nickname.bright_cyan());
    println!("  Public Key:             {}", public_key.bright_yellow());
    println!();

    println!("{}", "Direct Connection Info".bold());
    let ws_port = if config.listen_port == 0 {
        9000
    } else {
        config.listen_port
    };
    let p2p_port = ws_port + 1;

    println!(
        "  P2P Listener:           {}",
        format!("/ip4/0.0.0.0/tcp/{}", p2p_port).green()
    );
    println!("  Local Address:          /ip4/127.0.0.1/tcp/{}", p2p_port);
    println!(
        "  LAN Address:            /ip4/<YOUR_LAN_IP>/tcp/{}",
        p2p_port
    );

    println!();

    Ok(())
}

async fn cmd_contact(action: ContactAction) -> Result<()> {
    match action {
        ContactAction::Add {
            peer_id,
            public_key,
            name,
        } => {
            // Validate public key format before adding
            scmessenger_core::crypto::validate_ed25519_public_key(&public_key)
                .context("Invalid public key")?;

            // Resolve the peer_id argument to a canonical public_key_hex.
            // Accepts three formats:
            //   1. libp2p Peer ID (e.g. "12D3Koo...")
            //   2. Ed25519 public key hex (64 hex chars, valid curve point)
            //   3. Blake3 identity_id (64 hex chars, resolved via contact lookup)
            let data_dir = config::Config::data_dir()?;
            let storage_path = data_dir.join("storage");
            let core = IronCore::with_storage(path_to_string(&storage_path)?);
            let contacts = core.contacts_store_manager();

            let resolved_pk = if looks_like_libp2p_peer_id(&peer_id) {
                // libp2p Peer ID — derive public key and verify match
                let canonical = core
                    .extract_public_key_from_peer_id(peer_id.clone())
                    .context("Failed to derive public key from Peer ID")?;
                if canonical.to_lowercase() != public_key.to_lowercase() {
                    eprintln!(
                        "{} The provided public key does not match the Peer ID.",
                        "[WARN] Error:".red()
                    );
                    eprintln!(
                        "  Peer ID {} resolves to public key: {}",
                        peer_id.dimmed(),
                        canonical.yellow()
                    );
                    eprintln!("  You provided public key: {}", public_key.dimmed());
                    return Ok(());
                }
                canonical
            } else if looks_like_ed25519_pk(&peer_id) {
                // Direct Ed25519 public key — verify it matches the --public-key arg
                if peer_id.to_lowercase() != public_key.to_lowercase() {
                    eprintln!(
                        "{} The peer-id argument and public-key differ.",
                        "[WARN] Error:".red()
                    );
                    eprintln!("  Use either the Peer ID (12D3Koo...) or supply matching keys.");
                    return Ok(());
                }
                peer_id.to_lowercase()
            } else if looks_like_blake3_id(&peer_id) {
                // Blake3 identity_id — resolve via contacts or local identity
                match core.resolve_identity(peer_id.clone()) {
                    Ok(pk) => {
                        if pk.to_lowercase() != public_key.to_lowercase() {
                            eprintln!(
                                "{} Identity ID resolves to a different public key.",
                                "[WARN] Error:".red()
                            );
                            eprintln!("  Identity resolves to: {}", pk.yellow());
                            eprintln!("  You provided public key: {}", public_key.dimmed());
                            return Ok(());
                        }
                        pk
                    }
                    Err(_) => {
                        eprintln!(
                            "{} Could not resolve identity ID '{}'. No matching contact found.",
                            "[WARN] Error:".red(),
                            peer_id
                        );
                        eprintln!("  Identity IDs can only be resolved if the contact already exists in your address book.");
                        eprintln!("  Use the Peer ID (12D3Koo...) or public key hex instead.");
                        return Ok(());
                    }
                }
            } else {
                eprintln!(
                    "{} '{}' is not a recognized ID format.",
                    "[WARN] Error:".red(),
                    peer_id
                );
                eprintln!("  Accepted formats:");
                eprintln!("    - libp2p Peer ID (e.g. 12D3Koo...)");
                eprintln!("    - Ed25519 public key hex (64 hex chars)");
                eprintln!("    - Blake3 identity ID (64 hex chars, must match existing contact)");
                return Ok(());
            };

            // Try to use API if a node is running
            if api::is_api_available().await {
                let _ = api::add_contact_via_api(&peer_id, &public_key, name.clone())
                    .await
                    .context("Failed to add contact via API");
                println!("{} Contact added:", "[OK]".green());
                if let Some(nickname) = &name {
                    println!("  Name: {}", nickname.bright_cyan());
                }
                println!("  Canonical ID: {}", resolved_pk.yellow());
                return Ok(());
            }

            let mut contact = Contact::new(resolved_pk.clone(), public_key);
            if let Some(nickname) = name.clone() {
                contact.nickname = Some(nickname);
            }

            contacts
                .add(contact)
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;

            println!("{} Contact added:", "[OK]".green());
            if let Some(nickname) = name {
                println!("  Name: {}", nickname.bright_cyan());
            }
            println!("  Canonical ID: {}", resolved_pk.yellow());
        }

        _ => {
            // For other contact operations, use direct database access
            let data_dir = config::Config::data_dir()?;
            let storage_path = data_dir.join("storage");
            let core = IronCore::with_storage(path_to_string(&storage_path)?);
            let contacts = core.contacts_store_manager();

            match action {
                ContactAction::List => {
                    let list = contacts.list().unwrap_or_default();

                    if list.is_empty() {
                        println!("{}", "No contacts yet.".dimmed());
                    } else {
                        println!("{} ({} total)", "Contacts".bold(), list.len());
                        println!();

                        for contact in list {
                            let display = contact
                                .nickname
                                .clone()
                                .unwrap_or_else(|| contact.peer_id.clone());
                            println!("  {} {}", "•".bright_green(), display.bright_cyan());
                            println!("    Peer ID: {}", contact.peer_id.dimmed());
                        }
                    }
                }

                ContactAction::Show { contact: query } => {
                    let contact = find_contact(&contacts, &query)?;

                    let display = contact
                        .nickname
                        .clone()
                        .unwrap_or_else(|| contact.peer_id.clone());
                    println!("{}", "Contact Details".bold());
                    println!("  Name:       {}", display.bright_cyan());
                    println!("  Peer ID:    {}", contact.peer_id);
                    println!("  Public Key: {}", contact.public_key.bright_yellow());
                    println!("  Added:      {}", format_timestamp(contact.added_at));
                    // Wire set_notes display for contact notes
                    if let Some(ref notes) = contact.notes {
                        println!("  Notes:      {}", notes);
                    }
                }

                ContactAction::Remove { contact: query } => {
                    let contact = find_contact(&contacts, &query)?;
                    let name = contact
                        .nickname
                        .clone()
                        .unwrap_or_else(|| contact.peer_id.clone());

                    contacts
                        .remove(contact.peer_id)
                        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
                    println!("{} Removed contact: {}", "[OK]".green(), name.bright_cyan());
                }

                ContactAction::Search { query } => {
                    let results = contacts.search(query).unwrap_or_default();

                    if results.is_empty() {
                        println!("{}", "No matching contacts.".dimmed());
                    } else {
                        println!("{} ({} matches)", "Search Results".bold(), results.len());
                        println!();

                        for contact in results {
                            let display = contact
                                .nickname
                                .clone()
                                .unwrap_or_else(|| contact.peer_id.clone());
                            println!("  {} {}", "•".bright_green(), display.bright_cyan());
                            println!("    {}", contact.peer_id.dimmed());
                        }
                    }
                }

                ContactAction::SetLocalNickname {
                    contact: query,
                    nickname,
                    clear,
                } => {
                    if clear && nickname.is_some() {
                        anyhow::bail!("Use either <nickname> or --clear, not both");
                    }

                    let contact = find_contact(&contacts, &query)?;
                    let local = if clear { None } else { nickname };
                    contacts
                        .set_local_nickname(contact.peer_id.clone(), local.clone())
                        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

                    match local {
                        Some(name) => {
                            println!(
                                "{} Local nickname set for {} -> {}",
                                "[OK]".green(),
                                contact.peer_id.dimmed(),
                                name.bright_cyan()
                            );
                        }
                        None => {
                            println!(
                                "{} Local nickname cleared for {}",
                                "[OK]".green(),
                                contact.peer_id.dimmed()
                            );
                        }
                    }
                }

                ContactAction::Add { .. } => unreachable!(),
                ContactAction::SetNickname {
                    contact: query,
                    nickname,
                    clear,
                } => {
                    if clear && nickname.is_some() {
                        anyhow::bail!("Use either <nickname> or --clear, not both");
                    }

                    let contact = find_contact(&contacts, &query)?;
                    let nick = if clear { None } else { nickname };
                    contacts
                        .set_nickname(contact.peer_id.clone(), nick.clone())
                        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

                    match nick {
                        Some(name) => {
                            println!(
                                "{} Federated nickname set for {} -> {}",
                                "[OK]".green(),
                                contact.peer_id.dimmed(),
                                name.bright_cyan()
                            );
                        }
                        None => {
                            println!(
                                "{} Federated nickname cleared for {}",
                                "[OK]".green(),
                                contact.peer_id.dimmed()
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn cmd_config(action: ConfigAction) -> Result<()> {
    let mut config = config::Config::load()?;

    match action {
        ConfigAction::Set { key, value } => {
            config.set(&key, &value)?;
            println!("{} Set {} = {}", "[OK]".green(), key.bright_cyan(), value);
        }

        ConfigAction::Get { key } => {
            if let Some(value) = config.get(&key) {
                println!("{} = {}", key.bright_cyan(), value);
            } else {
                anyhow::bail!("Unknown config key: {}", key);
            }
        }

        ConfigAction::List => {
            println!("{}", "Configuration".bold());
            println!();

            for (key, value) in config.list() {
                println!("  {:<20} {}", key.bright_cyan(), value);
            }
        }

        ConfigAction::Privacy {
            padding,
            onion,
            cover,
            timing,
        } => {
            let data_dir = config::Config::data_dir()?;
            let storage_path = data_dir.join("storage");
            let core = IronCore::with_storage(path_to_string(&storage_path)?);

            let mut p: scmessenger_core::privacy::PrivacyConfig =
                serde_json::from_str(&core.get_privacy_config())?;

            if padding.is_none() && onion.is_none() && cover.is_none() && timing.is_none() {
                // Just show current config if no flags provided
                println!("{}", "Privacy Configuration".bold());
                println!(
                    "  Message Padding:   {}",
                    if p.message_padding_enabled {
                        "ON".green()
                    } else {
                        "OFF".red()
                    }
                );
                println!(
                    "  Onion Routing:      {}",
                    if p.onion_routing_enabled {
                        "ON".green()
                    } else {
                        "OFF".red()
                    }
                );
                println!(
                    "  Cover Traffic:      {}",
                    if p.cover_traffic_enabled {
                        "ON".green()
                    } else {
                        "OFF".red()
                    }
                );
                println!(
                    "  Timing Obfuscation: {}",
                    if p.timing_obfuscation_enabled {
                        "ON".green()
                    } else {
                        "OFF".red()
                    }
                );
                return Ok(());
            }

            if let Some(v) = padding {
                p.message_padding_enabled = v;
            }
            if let Some(v) = onion {
                p.onion_routing_enabled = v;
            }
            if let Some(v) = cover {
                p.cover_traffic_enabled = v;
            }
            if let Some(v) = timing {
                p.timing_obfuscation_enabled = v;
            }

            core.set_privacy_config(serde_json::to_string(&p)?)?;
            println!("{} Privacy configuration updated.", "[OK]".green());
        }
    }

    Ok(())
}

async fn cmd_history(
    peer_filter: Option<String>,
    search_query: Option<String>,
    limit: usize,
) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    let history = core.history_store_manager();

    let messages = if let Some(query) = search_query {
        history
            .search(query, limit as u32)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
    } else if let Some(peer) = peer_filter {
        let contacts = core.contacts_store_manager();
        let peer_id = if let Ok(contact) = find_contact(&contacts, &peer) {
            contact.peer_id
        } else {
            peer
        };

        history
            .conversation(peer_id, limit as u32)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
    } else {
        history
            .recent(None, limit as u32)
            .map_err(|e| anyhow::anyhow!("{:?}", e))?
    };

    if messages.is_empty() {
        println!("{}", "No messages found.".dimmed());
        return Ok(());
    }

    println!("{} ({} messages)", "Message History".bold(), messages.len());
    println!();

    for msg in messages {
        let direction = match msg.direction {
            MessageDirection::Sent => "→".bright_green(),
            MessageDirection::Received => "←".bright_blue(),
        };

        let time = format_timestamp(msg.timestamp).dimmed();
        let peer = msg.peer_id;

        println!("{} {} [{}]", direction, peer.bright_cyan(), time);
        println!("   {}", msg.content);
        println!();
    }

    Ok(())
}

/// True if every port in `ports` can be bound on both loopback and
/// all-interfaces (the same check `cmd_start` needs before it hands the
/// ports to the real listeners).
fn port_pair_available(ports: &[u16]) -> bool {
    ports.iter().all(|&p| {
        let addrs = [
            std::net::SocketAddr::from(([127, 0, 0, 1], p)),
            std::net::SocketAddr::from(([0, 0, 0, 0], p)),
        ];
        addrs
            .iter()
            .all(|addr| match std::net::TcpListener::bind(addr) {
                Ok(listener) => {
                    // Drop each probe before checking the next bind address.
                    // Keeping the first listener alive makes the subsequent
                    // 0.0.0.0 probe conflict with our own test socket and makes
                    // every otherwise-free port look occupied.
                    drop(listener);
                    true
                }
                Err(_) => false,
            })
    })
}

/// Scan forward from `start` for the next `(port, port + 1)` pair that's free,
/// to suggest as a `--port` fallback when the requested port stays occupied.
fn find_free_port_pair(start: u16) -> Option<u16> {
    (1u16..=20)
        .filter_map(|i| start.checked_add(i * 2))
        .find(|&candidate| {
            candidate
                .checked_add(1)
                .is_some_and(|next| port_pair_available(&[candidate, next]))
        })
}

/// Marks a message as machine-generated so responder nodes do not answer each
/// other in an unbounded loop.
const AUTO_REPLY_PREFIX: &str = "[auto-reply] ";
const AUTO_REPLY_ACK: &str =
    "[auto-reply] Thank you. Your message was received by this CLI; no further reply will be sent.";
const AUTO_REPLY_SEEN_CAPACITY: usize = 4096;

/// Permit at most one machine acknowledgement for a logical incoming message.
///
/// A transport can deliver the same envelope more than once while a connection
/// retries or converges. The previous implementation treated each delivery as
/// a new message, which amplified one incoming message into an auto-reply storm.
fn should_send_auto_reply(
    message_id: &str,
    incoming: &str,
    seen_ids: &mut HashSet<String>,
    seen_order: &mut VecDeque<String>,
) -> bool {
    if incoming.starts_with(AUTO_REPLY_PREFIX) || !seen_ids.insert(message_id.to_string()) {
        return false;
    }

    seen_order.push_back(message_id.to_string());
    if seen_order.len() > AUTO_REPLY_SEEN_CAPACITY {
        if let Some(expired_id) = seen_order.pop_front() {
            seen_ids.remove(&expired_id);
        }
    }
    true
}

async fn cmd_start(port: Option<u16>, http_bind: Option<String>, auto_reply: bool) -> Result<()> {
    // Env fallback so a node already under a process supervisor can be flipped
    // into responder mode without changing its argv.
    let auto_reply = auto_reply
        || matches!(
            std::env::var("SCM_AUTO_REPLY").as_deref(),
            Ok("1") | Ok("true")
        );
    if auto_reply {
        println!(
            "{} Bounded auto-reply ENABLED: one acknowledgement per unique text message",
            "[INFO]".yellow()
        );
    }
    let config = config::Config::load()?;
    let ws_port = port.unwrap_or({
        if config.listen_port == 0 {
            9000 // Default to 9000 if config has random port
        } else {
            config.listen_port
        }
    });

    // 1. Check if SCMessenger is already running via Control API
    if api::is_api_available().await {
        println!("{}", "SCMessenger is already running!".yellow());
        println!(
            "Run {} to stop the existing node first.",
            "scm stop".bright_green()
        );
        return Ok(());
    }

    // 2. Check if ports are occupied by something else (v4, v6, and localhost).
    // A port can be transiently held by a process that's still exiting (e.g. a
    // previous `scm start` shutting down after Ctrl+C), so retry with backoff
    // before giving up — and offer the next free port pair as a fallback.
    let p2p_port = ws_port + 1;
    let check_ports = [ws_port, p2p_port];
    const BIND_RETRIES: u32 = 5;

    for p in check_ports {
        let mut bound = false;
        for attempt in 0..BIND_RETRIES {
            bound = port_pair_available(&[p]);
            if bound {
                break;
            }
            if attempt + 1 < BIND_RETRIES {
                // Exponential backoff: 200ms, 400ms, 800ms, 1600ms
                let delay_ms = 200u64 << attempt;
                tracing::warn!(
                    "Port {} still in use (attempt {}/{}), retrying in {}ms — \
                     a previous node may still be shutting down",
                    p,
                    attempt + 1,
                    BIND_RETRIES,
                    delay_ms
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            }
        }

        if !bound {
            println!("{} Port {} is already in use.", "Error:".red(), p);
            match find_free_port_pair(ws_port) {
                Some(alt) => println!(
                    "Try {} to use a free port instead, or run {} to stop a stale process.",
                    format!("scm start --port {}", alt).bright_green(),
                    "scm stop".bright_green()
                ),
                None => println!(
                    "Try running {} or checking for other processes on this port.",
                    "scm stop".bright_green()
                ),
            }
            return Ok(());
        }
    }

    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    core.grant_consent();
    core.initialize_identity()
        .context("Failed to load identity")?;

    let info = core.get_identity_info();

    let contacts = core.contacts_store_manager();
    let history = core.history_store_manager();
    let _ = history.enforce_retention(10000); // Limit history to 10k messages

    // ── Outbox — persistent store-and-forward queue ──────────────────────
    let outbox = Outbox::open_default(&data_dir).map_err(|e| anyhow::anyhow!(e))?;

    // ── Connection Ledger — dial state over the single core store ───────
    // The core store is loaded by IronCore::with_storage above; this facade
    // attaches to the same storage-path entry state. A legacy peers.json is
    // migrated once at the bootstrap dial sweep (where own addresses known).
    let connection_ledger = ledger::ConnectionLedger::new(
        scmessenger_core::store::LedgerManager::new(path_to_string(&storage_path)?),
    );

    // Subscribe to any topics discovered in the ledger from past sessions
    let known_topics = connection_ledger.all_known_topics();

    println!("{}", "SCMessenger — Starting...".bold());
    println!();
    println!(
        "Identity: {}",
        info.identity_id
            .clone()
            .unwrap_or_else(|| "(pending)".to_string())
            .bright_cyan()
    );
    println!(
        "Public Key: {}",
        info.public_key_hex
            .as_deref()
            .unwrap_or("(not initialized)")
    );
    println!("Landing Page:  http://127.0.0.1:{}", ws_port);
    println!("WebSocket:     ws://127.0.0.1:{}/ws", ws_port);
    println!("P2P Listener:  /ip4/0.0.0.0/tcp/{}", p2p_port);
    println!("WASM Bridge:   /ip4/0.0.0.0/tcp/{}/ws", p2p_port + 1);
    println!(" {}", connection_ledger.summary());
    println!();

    // Wrap core in Arc early so WebContext and later tasks can share it.
    let core = Arc::new(core);

    // Use identity keypair for network (unified ID)
    let network_keypair = core
        .get_libp2p_keypair()
        .context("Failed to get network keypair from identity")?;
    let local_peer_id = network_keypair.public().to_peer_id();

    // NOTE: PeerId is now derived from identity keys. Existing installations that
    // had a separate network_keypair.dat will see their PeerId change. This is
    // intentional to unify identity and network IDs, but may require updating
    // peer expectations/ledgers on migration.

    println!("{} Peer ID: {}", "[OK]".green(), local_peer_id);
    println!();

    // Create shared state BEFORE server start so landing page has access
    let peers: Arc<tokio::sync::Mutex<HashMap<libp2p::PeerId, Option<String>>>> =
        Arc::new(tokio::sync::Mutex::new(HashMap::new()));
    let ledger = Arc::new(tokio::sync::Mutex::new(connection_ledger));

    // Create transport bridge
    let transport_bridge = Arc::new(tokio::sync::Mutex::new(
        transport_bridge::TransportBridge::new(),
    ));

    // Merge config bootstrap nodes with environment / default bootstrap nodes
    let merged_bootstrap = bootstrap::merge_bootstrap_nodes(config.bootstrap_nodes.clone());

    // Build web context for landing page + public APIs
    let web_ctx = Arc::new(server::WebContext {
        node_peer_id: local_peer_id.to_string(),
        node_public_key: info.public_key_hex.clone().unwrap_or_default(),
        bootstrap_nodes: merged_bootstrap.clone(),
        ledger: ledger.clone(),
        peers: peers.clone(),
        start_time: std::time::Instant::now(),
        transport_bridge: transport_bridge.clone(),
        ui_port: ws_port,
        core: Some(Arc::clone(&core)),
    });

    // Start WebSocket + HTTP Server (serves landing page at /)
    let (ui_broadcast, mut ui_cmd_rx) = server::start(ws_port, web_ctx.clone()).await?;

    let listen_addr: libp2p::Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", p2p_port).parse()?;
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(256);

    // Build discovery config from CLI config
    let discovery_config =
        scmessenger_core::transport::DiscoveryConfig::new(if config.enable_mdns {
            scmessenger_core::transport::DiscoveryMode::Open
        } else {
            scmessenger_core::transport::DiscoveryMode::Manual
        });

    // Parse bootstrap node multiaddrs from merged list (relay also uses bootstrap nodes)
    let relay_bootstrap: Vec<libp2p::Multiaddr> = merged_bootstrap
        .iter()
        .filter_map(|addr| addr.parse().ok())
        .collect();

    let multiport_config = scmessenger_core::transport::multiport::MultiPortConfig {
        preferred_port: Some(p2p_port),
        ..Default::default()
    };

    // CORE HANDLE IS LOAD-BEARING NOW (choke-point refactor 2026-07-26). This
    // used to be `None`, which was survivable while the CLI built its own
    // ledger-exchange payload with `to_shared_entries()`. That door is gone:
    // `SwarmCommand::ShareLedger` builds the payload from
    // `IronCore::ledger_manager` through `exchange_response_entries`, so without
    // a core handle this node would offer an empty ledger to every peer. Wiring
    // it also activates `record_connection` on outbound `ConnectionEstablished`
    // (review F11), which is what populates that ledger in the first place.
    let swarm_handle = transport::start_swarm_with_config(
        network_keypair,
        Some(listen_addr),
        event_tx,
        Some(multiport_config),
        relay_bootstrap,
        None,
        Some(Arc::downgrade(&core)),
        false,
        Some(discovery_config),
        transport::default_routing_engine_handle(),
    )
    .await?;

    // ── WebSocket P2P Bridge for WASM ────────────────────────────────────
    // Redundant explicit bind removed; handled by MultiPortConfig.

    println!("{} Network started", "[OK]".green());

    // btleplug's CoreBluetooth adapter starts a worker for each adapters()
    // call. The central ingress below owns that lifecycle on macOS; a
    // concurrent diagnostic probe can trigger a btleplug completion panic.
    // Keep the probe on Windows/Linux, where the existing startup behavior is
    // safe and remains useful for adapter diagnostics.
    if config.enable_ble && !cfg!(target_os = "macos") {
        tokio::spawn(async move {
            ble_daemon::probe_and_log().await;
        });
    }

    // Subscribe to any topics from the ledger
    for topic in known_topics {
        let _ = swarm_handle.subscribe_topic(topic).await;
    }

    // Subscribe to default topics
    for topic in [scmessenger_core::TOPIC_LOBBY, scmessenger_core::TOPIC_MESH] {
        let _ = swarm_handle.subscribe_topic(topic.to_string()).await;
    }

    println!();
    println!("{}", "Commands:".bold());
    println!("  {} <contact> <message>", "send".bright_green());
    println!("  {}                      ", "contacts".bright_green());
    println!("  {}                       ", "peers".bright_green());
    println!("  {}                      ", "status".bright_green());
    println!("  {}                        ", "quit".bright_green());
    println!();

    // Note: core was wrapped in Arc above before WebContext creation;
    // peers and ledger Arc<Mutex> were created above before server::start
    // so the landing page and API endpoints have access to them.

    if config.enable_ble {
        let core_ble = Arc::clone(&core);
        let ui_ble = ui_broadcast.clone();
        tokio::spawn(async move {
            ble_mesh::run_ble_central_ingress(core_ble, ui_ble).await;
        });

        let core_ble_adv = Arc::clone(&core);
        let ui_ble_adv = ui_broadcast.clone();
        tokio::spawn(async move {
            ble_mesh::run_ble_peripheral_advertising(core_ble_adv, ui_ble_adv).await;
        });
    }

    // ── Dial known peers from persistent ledger ──────────────────────────
    // Dial any peers from the persistent ledger that pass backoff.
    let dial_scheduler = Arc::new(DialScheduler::new(ledger.clone(), swarm_handle.clone()));
    {
        println!();
        println!(
            "{} Aggressive Discovery — dialing known peers...",
            "".yellow()
        );
        let scheduler = Arc::clone(&dial_scheduler);
        let ledger_clone = ledger.clone();
        let swarm_clone = swarm_handle.clone();
        let data_dir_migrate = data_dir.clone();

        tokio::spawn(async move {
            let my_addrs = get_local_transport_addresses(&swarm_clone).await;
            let addrs = {
                let l = ledger_clone.lock().await;
                match l.run_legacy_migration(
                    &data_dir_migrate,
                    Some(&local_peer_id.to_string()),
                    &my_addrs,
                ) {
                    Ok(report) if report.offered > 0 => {
                        println!(
                            "[INFO] Migrated {} of {} legacy peers.json entries into the core ledger ({} rejected, archived={})",
                            report.imported, report.offered, report.rejected, report.archived
                        );
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("[WARNING] legacy peers.json migration failed: {:#}", e);
                    }
                }
                l.dialable_addresses(Some(&local_peer_id.to_string()), &my_addrs)
            };

            // Dial all known addresses (bootstrap + discovered)
            for (i, (multiaddr_str, peer_id_opt)) in addrs.iter().enumerate() {
                let label =
                    ledger::extract_ip_port(multiaddr_str).unwrap_or_else(|| multiaddr_str.clone());
                println!("  {}.  Dialing {} (promiscuous)...", i + 1, label);

                let peer_id = peer_id_opt.as_ref().and_then(|s| s.parse::<PeerId>().ok());
                scheduler.dial(multiaddr_str.clone(), peer_id);

                // Brief pause between dials to avoid overwhelming
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        });
    }

    // Broadcast status loop for WebSocket UI
    let ui_broadcast_clone = ui_broadcast.clone();
    let peers_clone_status = peers.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let count = peers_clone_status.lock().await.len();
            // Don't crash if no subscribers
            let _ = ui_broadcast_clone.send(server::UiOutbound::Legacy(
                server::UiEvent::NetworkStatus {
                    status: "online".to_string(),
                    peer_count: count,
                },
            ));
        }
    });

    // P0_TRANSPORT_001: Periodic address refresh - before dialing from ledger,
    // send an Identify probe to refresh peer addresses. This ensures we have
    // current listen addresses even if peers restarted with new ports.
    let swarm_refresh_clone = swarm_handle.clone();
    let ledger_refresh_clone = ledger.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;

            let my_addrs = get_local_transport_addresses(&swarm_refresh_clone).await;
            let addrs = {
                let l = ledger_refresh_clone.lock().await;
                l.dialable_addresses(Some(&local_peer_id.to_string()), &my_addrs)
            };

            // For each peer, try to refresh their address via Identify
            // This asks the peer: "What is your current listening address?"
            for (_multiaddr_str, peer_id_opt) in &addrs {
                if let Some(ref peer_id_str) = peer_id_opt {
                    if let Ok(peer_id) = peer_id_str.parse::<PeerId>() {
                        // Attempt to request address reflection
                        // If this succeeds, we'll get the current address
                        // If it fails (no connection), we'll still dial with current address
                        let _ = swarm_refresh_clone
                            .request_address_reflection(peer_id)
                            .await;
                    }
                }
            }
        }
    });

    // EXTERNAL-ADDRESS AWARENESS: on daemon start and every 10 minutes,
    // determine this node's externally observed address(es) (the same source
    // the GET /api/external-address endpoint reports), log the refresh, and
    // register them in the local route tables (hint store + ledger) so peers
    // can dial us and our outbound identity envelopes always carry fresh
    // external reachables.
    {
        let ext_swarm = swarm_handle.clone();
        let ext_local_peer = local_peer_id;
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(600));
            ticker.tick().await; // first tick is immediate (daemon start)
            loop {
                ticker.tick().await;
                match ext_swarm.get_external_addresses().await {
                    Ok(sockets) => {
                        let mut addrs: Vec<String> = Vec::new();
                        for socket in sockets {
                            if socket.ip().is_unspecified() || socket.port() == 0 {
                                continue;
                            }
                            let host = match socket.ip() {
                                std::net::IpAddr::V4(ip) => format!("/ip4/{ip}"),
                                std::net::IpAddr::V6(ip) => format!("/ip6/{ip}"),
                            };
                            let value = format!("{host}/tcp/{}", socket.port());
                            if !addrs.contains(&value) {
                                addrs.push(value);
                            }
                        }
                        if addrs.is_empty() {
                            tracing::debug!(
                                "EXTERNAL_ADDRESS_REFRESH: no externally observed addresses yet for {}",
                                ext_local_peer
                            );
                            continue;
                        }
                        tracing::info!(
                            "EXTERNAL_ADDRESS_REFRESH peer={} addrs={:?}",
                            ext_local_peer,
                            addrs
                        );
                        scmessenger_core::transport::hint_store::annotate_peer_hints(
                            ext_local_peer,
                            &addrs,
                        );
                    }
                    Err(e) => {
                        tracing::debug!("EXTERNAL_ADDRESS_REFRESH failed: {}", e);
                    }
                }
            }
        });
    }

    // Start control API server
    let api_ctx = api::ApiContext {
        core: core.clone(),
        swarm_handle: Arc::new(swarm_handle.clone()),
    };

    let http_bind_api = http_bind.clone();
    tokio::spawn(async move {
        if let Err(e) = api::start_api_server(api_ctx, http_bind_api).await {
            tracing::error!("API server error: {}", e);
        }
    });

    println!(
        "{} Control API: {}",
        "[OK]".green(),
        format!("http://127.0.0.1:{}", api::API_PORT).dimmed()
    );

    let core_rx = core.clone();
    let contacts_rx = contacts.clone();
    let history_rx = history.clone();
    let peers_rx = peers.clone();
    let ledger_rx = ledger.clone();
    let outbox_rx = outbox.clone();
    let scheduler_rx = Arc::clone(&dial_scheduler);

    // Stdin handling
    // Ctrl+C handler for graceful shutdown
    let ctrl_c_swarm = swarm_handle.clone();

    // Duplicate network deliveries must not create duplicate machine replies.
    // This cache is intentionally process-local and bounded; delivery retries
    // of an already-seen message remain covered for the lifetime of the node.
    let mut auto_reply_seen_ids = HashSet::new();
    let mut auto_reply_seen_order = VecDeque::new();

    // Swarm liveness watchdog.
    //
    // The swarm runs in its OWN task. When it died in 5-node run 1 the process
    // survived: the HTTP control API kept accepting connections while the mesh
    // was gone. That zombie then blocked its own restart, because the "is it
    // already running?" check is a bare TCP connect against that same port and
    // cannot tell a healthy node from a corpse still holding the socket.
    //
    // SwarmTaskLivenessGuard already flips this flag on unwind; nothing on the
    // CLI side was reading it. Fail fast and loudly instead of lying about being
    // up -- a node that exits gets restarted, a zombie silently drops traffic
    // and looks like a messaging bug.
    let watchdog_swarm = swarm_handle.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(10));
        ticker.tick().await; // first tick is immediate
        loop {
            ticker.tick().await;
            if !watchdog_swarm.is_event_loop_alive() {
                tracing::error!(
                    "swarm_event_loop_died: the mesh is down but the process is still up; exiting so this node does not linger as a zombie"
                );
                eprintln!(
                    "{} Swarm event loop died -- exiting rather than running without a mesh.",
                    "[FAIL]".red()
                );
                std::process::exit(1);
            }
        }
    });

    let stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut stdin_lines = tokio::io::AsyncBufReadExt::lines(stdin);

    loop {
        tokio::select! {
                    // 0. Ctrl+C — graceful shutdown
                    _ = tokio::signal::ctrl_c() => {
                        println!("\nCaught Ctrl+C, shutting down gracefully...");
                        let _ = ctrl_c_swarm.shutdown().await;
                        // The core store persists its own mutations; nothing
                        // to flush on shutdown.
                        let _ledger_guard = ledger.lock().await;
                        break;
                    }

                    // 1. Swarm Events (Network -> App -> UI)
                    Some(event) = event_rx.recv() => {
                        match event {
                            SwarmEvent::PeerDiscovered(peer_id) => {
                                 let mut p = peers_rx.lock().await;
                                 if let std::collections::hash_map::Entry::Vacant(e) = p.entry(peer_id) {
                                     e.insert(None);
                                     println!("\n{} Peer: {}", "[OK]".green(), peer_id);
                                     print!("> ");
                                     let _ = std::io::Write::flush(&mut std::io::stdout());
                                     let _ = contacts_rx.update_last_seen(peer_id.to_string());

                                     // Try to get public key from existing contact, if available
                                     let (public_key, identity) = contacts_rx.get(peer_id.to_string())
                                         .ok().flatten()
                                         .map(|c| (Some(c.public_key), Some(c.peer_id.clone())))
                                         .unwrap_or((None, None));

                                     let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::PeerDiscovered {
                                         peer_id: peer_id.to_string(),
                                         transport: "tcp".to_string(),
                                         public_key: public_key.unwrap_or_default(),
                                         identity: identity.unwrap_or_default(),
                                     }));
                                     let n = notif_peer_discovered(PeerDiscoveredParams {
                                         peer_id: peer_id.to_string(),
                                         transport: "tcp".to_string(),
                                         public_key: contacts_rx
                                             .get(peer_id.to_string())
                                             .ok()
                                             .flatten()
                                             .map(|c| c.public_key),
                                     });
                                     if let Ok(v) = serde_json::to_value(&n) {
                                         let _ = ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                                     }

                                     // Register peer with transport bridge using default capabilities
                                     let capabilities = vec![TransportType::Internet, TransportType::Local];
                                     let capabilities_clone = capabilities.clone();
                                     let mut bridge = transport_bridge.lock().await;
                                     bridge.register_peer(peer_id, capabilities);
                                     let can_reach = bridge.can_reach_destination(&peer_id);
                                     tracing::info!("Registered transport capabilities for {}: {:?}, reachable={}", peer_id, capabilities_clone, can_reach);

                                     // AUTO LEDGER EXCHANGE: Share our known peers with the new
                                     // connection. The payload is built inside the swarm from
                                     // `LedgerManager::exchange_response_entries`
                                     if let Err(e) = swarm_handle.share_ledger(peer_id).await {
                                         tracing::warn!("Failed to share ledger with {}: {}", peer_id, e);
                                     }

                                     // OUTBOX FLUSH: Deliver any queued messages for this peer now
                                     // that they are online.
                                     if !can_reach {
                                         tracing::warn!("No compatible transport path to {}; deferring outbox flush", peer_id);
                                     } else {
                                         flush_outbox_for_peer(&outbox_rx, &swarm_handle, peer_id).await;
                                     }
                                 }
                            }
                            SwarmEvent::PeerDisconnected(peer_id) => {
                                peers_rx.lock().await.remove(&peer_id);

                                // Record disconnect in ledger (useful for backoff tracking)
                                // We find the entry by PeerID and record failure
                                let mut l = ledger_rx.lock().await;
                                if let Some(multiaddr) = l.find_peer_multiaddr(&peer_id.to_string())
                                {
                                    l.record_failure(&multiaddr);
                                }
                                // Release the per-peer concurrent-connection
                                // slot (P0 cap). Core emits one
                                // PeerDisconnected per dropped connection, so
                                // this balances the saturating_add in
                                // complete_dial.
                                l.record_disconnect(peer_id);
                            }

                            SwarmEvent::RelayCircuitEstablished => {
                                scheduler_rx.set_relay_healthy(true);
                            }
                            SwarmEvent::RelayCircuitBroken => {
                                scheduler_rx.set_relay_healthy(false);
                            }

                            // LEDGER EXCHANGE: Received peer list from a connected peer
                            SwarmEvent::LedgerReceived { from_peer, entries } => {
                                let mut l = ledger_rx.lock().await;
                                let new_count = l.merge_shared_entries(&entries);

                                if new_count > 0 {
                                    println!(
                                        "\n{}  Learned {} new peers from {}",
                                        "[OK]".green(),
                                        new_count,
                                        from_peer
                                    );
                                    print!("> ");
                                    let _ = std::io::Write::flush(&mut std::io::stdout());

                                    // The core store persists its own updates;
                                    // nothing to flush here.

                                    // Dial newly discovered peers
                                    let new_entries = ledger::prioritize_dial_candidates(
                                        entries
                                        .iter()
                                        .map(|e| {
                                            (e.multiaddr.clone(), e.last_peer_id.clone())
                                        })
                                        .collect(),
                                    );
                                    drop(l); // release lock before dialing

                                    // Graceful-AF dial policy: know our own addresses
                                    // before promiscuously dialing whatever the ledger
                                    // handed us
                                    let my_addrs = get_local_transport_addresses(&swarm_handle).await;

                                    for (addr_str, peer_id_opt) in new_entries {
                                        let local_peer_id_string = local_peer_id.to_string();
                                        if peer_id_opt.as_deref() == Some(local_peer_id_string.as_str())
                                            || ledger::contains_peer_id_component(
                                                &addr_str,
                                                &local_peer_id_string,
                                            )
                                        {
                                            tracing::debug!(
                                                "Skipping self-targeted ledger address: {}",
                                                addr_str
                                            );
                                            continue;
                                        }
                                        // Skip non-routable addresses (loopback,
                                        // link-local, site-local) a peer may
                                        // advertise -- dialing them fails forever
                                        // and storms the request_response handler.
                                        if !ledger::is_dialable_multiaddr(
                                            &addr_str,
                                            ledger::NetworkMode::Local,
                                            ledger::DnsPolicy::Reject,
                                        ) {
                                            continue;
                                        }
                                        if !ledger::is_dialable_for_this_node(
                                            &addr_str,
                                            ledger::NetworkMode::Local,
                                            &my_addrs,
                                        ) {
                                            continue;
                                        }
                                        let peer_id = peer_id_opt
                                            .as_ref()
                                            .and_then(|s| s.parse::<PeerId>().ok());
                                        // Fire-and-forget: scheduler spawns the actual dial
                                        // so this shared event loop never blocks on a
                                        // connection/timeout.
                                        scheduler_rx.dial(addr_str, peer_id);
                                    }
                                }
                            }

                            // IDENTIFY: Peer identity confirmed — update ledger.
                            SwarmEvent::PeerIdentified { peer_id, listen_addrs, .. } => {
                                {
                                    let mut l = ledger_rx.lock().await;
                                    let advertised: Vec<String> =
                                        listen_addrs.iter().map(|a| a.to_string()).collect();
                                    l.record_identified_peer(&peer_id.to_string(), &advertised);
                                }
                                if let Err(e) = swarm_handle.share_ledger(peer_id).await {
                                    tracing::warn!("Failed to share ledger with identified peer {}: {}", peer_id, e);
                                }
                                flush_outbox_for_peer(&outbox_rx, &swarm_handle, peer_id).await;
                            }

                            // GOSSIPSUB: New topic discovered
                            SwarmEvent::TopicDiscovered { peer_id, topic } => {
                                tracing::info!("Topic discovered from {}: {}", peer_id, topic);
                                // Record the topic in the ledger for this peer
                                let mut l = ledger_rx.lock().await;
                                if let Some(multiaddr) = l.find_peer_multiaddr(&peer_id.to_string())
                                {
                                    l.record_topic(&multiaddr, &topic);
                                }
                            }

                            SwarmEvent::MessageReceived { peer_id, envelope_data } => {
                                // Extract sender's Ed25519 public key from the envelope before decryption.
                                // We need it to encrypt the delivery receipt back to them.
                                let sender_public_key_hex = decode_envelope(&envelope_data)
                                    .ok()
                                    .map(|env| hex::encode(&env.sender_public_key));

                                if let Ok(msg) = core_rx.receive_message(envelope_data) {
                                    match msg.message_type {
                                        MessageType::OnionRelay => {
                                            // Forward onion-routed packet to next hop
                                            let next_hop_hex = msg.recipient_id.clone();
                                            let payload = msg.payload.clone();

                                            if let Ok(next_hop_bytes) = hex::decode(&next_hop_hex) {
                                                // Convert Ed25519 PK to libp2p PeerId
                                                if let Ok(keypair) = libp2p::identity::ed25519::Keypair::try_from_bytes(&mut next_hop_bytes[..32].to_vec()) {
                                                    let next_peer_id = libp2p::PeerId::from_public_key(&keypair.public().into());

                                                    tracing::info!("Relaying onion packet from {} to next hop {}", peer_id, next_peer_id);
                                                    let swarm_clone = swarm_handle.clone();
                                                    tokio::spawn(async move {
                                                        if let Err(e) = swarm_clone.send_message(next_peer_id, payload, None, None).await {
                                                            tracing::warn!("Failed to relay onion packet to {}: {}", next_peer_id, e);
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                        MessageType::Text => {
                                            // Identity-envelope parity with Android/iOS:
                                            // inbound chat text may be wrapped in a
                                            // scm.message.identity.v1 envelope. Decode it
                                            // for display/history and learn the sender's
                                            // nickname + route hints.
                                            let raw_text =
                                                msg.text_content().unwrap_or_else(|| "<binary>".into());
                                            let decoded_envelope = scmessenger_core::message::identity_envelope::parse_identity_envelope(&raw_text);
                                            if let Some(decoded) = decoded_envelope.as_ref() {
                                                learn_sender_identity_from_envelope(
                                                    &contacts_rx,
                                                    &peer_id.to_string(),
                                                    decoded,
                                                );
                                            }
                                            let text = decoded_envelope
                                                .as_ref()
                                                .map(|d| d.text.clone())
                                                .unwrap_or(raw_text);
                                            let sender_name = contacts_rx.get(peer_id.to_string())
                                                .ok().flatten()
                                                .map(|c| c.display_name().to_string())
                                                .unwrap_or_else(|| peer_id.to_string());

                                            println!("\n{} {}: {}", "←".bright_blue(), sender_name.bright_cyan(), text);
                                            print!("> ");
                                            let _ = std::io::Write::flush(&mut std::io::stdout());


                                            let ts = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_secs();
                                            let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::MessageReceived {
                                                from: peer_id.to_string(),
                                                content: text.clone(),
                                                timestamp: ts,
                                                message_id: msg.id.clone(),
                                            }));
                                            let mn = notif_message_received(MessageReceivedParams {
                                                from: peer_id.to_string(),
                                                content: text,
                                                timestamp: ts,
                                                message_id: msg.id.clone(),
                                            });
                                            if let Ok(v) = serde_json::to_value(&mn) {
                                                let _ = ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                                            }

                                            // Send delivery receipt back to the sender -- but ONLY
                                            // for a genuine inbound user message. UNIFICATION_V3 D2:
                                            // suppress acking scm.message.identity.v1 SYNC/CONFIG
                                            // metadata (kind != "text", e.g. identity_sync /
                                            // history_sync) that the sender emits send-and-forget
                                            // with no outbox record, and a message that looped back
                                            // to its own origin (sender == local node). NOTE: every
                                            // outbound CHAT message is also wrapped in an identity
                                            // envelope with kind == "text", so only kinds other than
                                            // "text" are treated as metadata -- acking those produced
                                            // the receiver's "[RECEIPT-RX] IGNORING ...
                                            // direction=missing" stampede.
                                            let is_identity_metadata = decoded_envelope
                                                .as_ref()
                                                .map(|d| d.kind.as_str() != "text")
                                                .unwrap_or(false);
                                            let local_pk = core_rx.get_identity_info().public_key_hex;
                                            let is_self_loop = local_pk
                                                .as_deref()
                                                .map(|pk| sender_public_key_hex.as_deref() == Some(pk))
                                                .unwrap_or(false);
                                            if !is_identity_metadata && !is_self_loop {
                                                if let Some(ref pk_hex) = sender_public_key_hex {
                                                    match core_rx.prepare_receipt(pk_hex.clone(), msg.id.clone()) {
                                                        Ok(ack_bytes) => {
                                                            tracing::debug!("Sending delivery ACK for {} to {}", msg.id, peer_id);
                                                            if let Err(e) = swarm_handle.send_message(peer_id, ack_bytes, None, None).await {
                                                                tracing::debug!("Failed to send delivery ACK to {}: {}", peer_id, e);
                                                            }
                                                        }
                                                        Err(e) => {
                                                            tracing::debug!("Failed to prepare delivery ACK: {}", e);
                                                        }
                                                    }
                                                }
                                            }

                                            // Optional machine acknowledgement. Keep it
                                            // short, do not echo user content, and send
                                            // at most once per logical message ID. The
                                            // receipt above already provides delivery
                                            // evidence; this acknowledgement exists only
                                            // for the explicit CLI test-harness mode.
                                            if auto_reply {
                                                let incoming = decoded_envelope
                                                    .as_ref()
                                                    .map(|d| d.text.clone())
                                                    .unwrap_or_else(|| {
                                                        msg.text_content().unwrap_or_default()
                                                    });
                                                if !should_send_auto_reply(
                                                    &msg.id,
                                                    &incoming,
                                                    &mut auto_reply_seen_ids,
                                                    &mut auto_reply_seen_order,
                                                ) {
                                                    tracing::debug!(
                                                        "auto_reply_suppressed_duplicate_or_machine_message in_reply_to={} from={}",
                                                        msg.id,
                                                        peer_id
                                                    );
                                                } else if let Some(ref pk_hex) =
                                                    sender_public_key_hex
                                                {
                                                    match core_rx.prepare_message_with_id(
                                                        pk_hex.clone(),
                                                        AUTO_REPLY_ACK.to_string(),
                                                        scmessenger_core::MessageType::Text,
                                                        None,
                                                    ) {
                                                        Ok(prep) => {
                                                            match swarm_handle
                                                                .send_message(peer_id, prep.envelope_data, None, None)
                                                                .await
                                                            {
                                                                Ok(_) => tracing::info!(
                                                                    "auto_reply_ack_queued in_reply_to={} to={}",
                                                                    msg.id,
                                                                    peer_id
                                                                ),
                                                                Err(e) => tracing::warn!(
                                                                    "auto_reply_ack_queue_failed in_reply_to={} to={}: {}",
                                                                    msg.id,
                                                                    peer_id,
                                                                    e
                                                                ),
                                                            }
                                                        }
                                                        // This is the branch that would have
                                                        // caught the run-1 identity defect on
                                                        // the CLI side, so log it loudly.
                                                        Err(e) => tracing::error!(
                                                            "auto_reply_ack_prepare_failed in_reply_to={} to={}: {}",
                                                            msg.id,
                                                            peer_id,
                                                            e
                                                        ),
                                                    }
                                                }
                                            }
                                        }
                                        MessageType::Receipt => {
                                            // Received a delivery receipt — the remote peer confirmed delivery.
                                            if let Ok(receipt) = scmessenger_core::decode_receipt(msg.payload.clone()) {
                                                let short_id = receipt.message_id.get(..8).unwrap_or(&receipt.message_id);
                                                println!("\n{} Delivered: {}", "[OK][OK]".green(), short_id);
                                                print!("> ");
                                                let _ = std::io::Write::flush(&mut std::io::stdout());
                                                tracing::debug!("Delivery ACK received from {}: msg_id={}", peer_id, receipt.message_id);

                                                // Mark the message as delivered in history
                                                if let Err(e) = history_rx.mark_delivered(receipt.message_id.clone()) {
                                                    tracing::warn!("Failed to mark message {} as delivered: {}", receipt.message_id, e);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            SwarmEvent::ListeningOn(addr) => {
                                println!("{} Listening on {}", "[OK]".green(), addr);
                            }
                            _ => {}
                        }
                    }



                    // 2. UI Commands (UI -> App -> Network)
                    Some(cmd) = ui_cmd_rx.recv() => {
                        match cmd {
                            server::UiCommand::IdentityShow => {
                                let i = core_rx.get_identity_info();
                                let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::IdentityInfo {
                                    peer_id: i.identity_id.unwrap_or_default(),
                                    public_key: i.public_key_hex.unwrap_or_default(),
                                    nickname: i.nickname,
                                    libp2p_peer_id: i.libp2p_peer_id,
                                }));
                            }
                            server::UiCommand::IdentityExport => {
                                let i = core_rx.get_identity_info();
                                let data_dir = config::Config::data_dir().unwrap_or_default();
                                let storage_path = data_dir.join("storage");

                                let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::IdentityExportData {
                                    identity_id: i.identity_id.unwrap_or_default(),
                                    public_key: i.public_key_hex.unwrap_or_default(),
                                    private_key: "Keys are stored securely in the data directory.".to_string(),
                                    storage_path: storage_path.display().to_string(),
                                }));
                            }
                            server::UiCommand::ContactList => {
                                if let Ok(list) = contacts_rx.list() {
                                    let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::ContactList { contacts: list.into_iter().map(|c| serde_json::to_value(c).unwrap_or_default()).collect() }));
                                }
                            }
                            server::UiCommand::HistoryList { peer_id, limit } => {
                                let l = limit.unwrap_or(50);
                                if let Ok(messages) = history_rx.conversation(peer_id.clone(), l) {
                                    let history_messages = messages.into_iter().map(|m| {
                                        crate::api::HistoryMessage {
                                            id: m.id,
                                            peer_id: m.peer_id,
                                            content: m.content,
                                            direction: match m.direction {
                                                MessageDirection::Sent => "sent".to_string(),
                                                MessageDirection::Received => "received".to_string(),
                                            },
                                            timestamp: m.timestamp,
                                            delivered: m.delivered,
                                        }
                                    }).collect::<Vec<_>>();
                                    let history_messages: Vec<serde_json::Value> = history_messages.into_iter().map(|m| serde_json::to_value(m).unwrap_or_default()).collect();
                                    let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::HistoryList {
                                        peer_id,
                                        messages: history_messages
                                    }));
                                }
                            }
                            server::UiCommand::Status => {
                                let count = peers_rx.lock().await.len();
                                let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::NetworkStatus {
                                    status: "online".to_string(),
                                    peer_count: count
                                }));
                            }
                            server::UiCommand::Send { recipient, message, id } => {
                                // Resolve recipient to PeerID and PublicKey
                                let peer_id_res = recipient.parse::<libp2p::PeerId>();
                                let contact_res = contacts_rx.get(recipient.clone());

                                let target_peer = if let Ok(pid) = peer_id_res {
                                    Some(pid)
                                } else if let Ok(Some(contact)) = contact_res {
                                    contact.peer_id.parse().ok()
                                } else {
                                    None
                                };

                                if let Some(target) = target_peer {
                                     // Try to find public key
                                     let pk_opt = if let Ok(Some(c)) = contacts_rx.get(target.to_string()) {
                                         Some(c.public_key)
                                     } else { None };

                                     if let Some(pk) = pk_opt {
                                         // prepare_message_with_id automatically saves outgoing history
                                         // Identity-envelope parity: wrap chat text so the
                                         // receiver learns our nickname + route hints.
                                         let wire_message = crate::api::build_identity_wrapped_text(&core_rx, &swarm_handle, &message).await;
                                          if let Ok(prep) = core_rx.prepare_message_with_id(pk.clone(), wire_message, scmessenger_core::MessageType::Text, None) {
                                               let ble_ok = ble_mesh::send_ble_message(&target.to_string(), &prep.envelope_data).await.is_ok();
                                               let swarm_ok = swarm_handle.send_message(target, prep.envelope_data, None, None).await.is_ok();
                                               if swarm_ok {
                                                   // True transport ACK (R2): release the outbox entry without a
                                                   // Delivered receipt. BLE gatt is fire-and-forget, so only the
                                                   // swarm path marks sent.
                                                   core_rx.mark_message_sent(prep.message_id.clone());
                                               }
                                               let sent = ble_ok || swarm_ok;

                                               if sent {
                                                  let mid = id.clone().unwrap_or_default();
                                                  let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::MessageStatus {
                                                      message_id: mid.clone(),
                                                      status: "sent".to_string()
                                                  }));
                                                  let dn = notif_delivery_status(DeliveryStatusParams {
                                                      message_id: mid,
                                                      status: "sent".to_string(),
                                                  });
                                                  if let Ok(v) = serde_json::to_value(&dn) {
                                                      let _ = ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                                                  }
                                              }
                                         }
                                     }
                                }
                            }
                            server::UiCommand::ContactAdd { peer_id, name, public_key } => {
                                // Assuming public key is provided or we can fetch it? MVP assumes provided.
                                if let Some(pk) = public_key {
                                    // Validate public key before adding
                                    if let Err(e) = scmessenger_core::crypto::validate_ed25519_public_key(&pk) {
                                        tracing::warn!("Failed to add contact {}: invalid public key - {}", peer_id, e);
                                        let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::Error {
                                            message: format!("Invalid public key: {}", e)
                                        }));
                                        continue;
                                    }

                                    let contact = Contact::new(peer_id.clone(), pk)
                                        .with_nickname(name.unwrap_or(peer_id));
                                    let _ = contacts_rx.add(contact);
                                    if let Ok(list) = contacts_rx.list() {
                                        let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::ContactList { contacts: list.into_iter().map(|c| serde_json::to_value(c).unwrap_or_default()).collect() }));
                                    }
                                }
                            }
                            server::UiCommand::ContactRemove { contact } => {
                                 // remove by peer_id (assuming contact arg is peer_id for now, or resolving nickname)
                                 // contacts.remove takes peer_id string
                                 if contacts_rx.remove(contact).is_ok() {
                                     if let Ok(list) = contacts_rx.list() {
                                         let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::ContactList { contacts: list.into_iter().map(|c| serde_json::to_value(c).unwrap_or_default()).collect() }));
                                     }
                                 }
                            }
                            server::UiCommand::ConfigGet { key } => {
                                if let Ok(cfg) = config::Config::load() {
                                    let value = cfg.get(&key);
                                    let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::ConfigValue {
                                        key: key.clone(),
                                        value: value.unwrap_or_default(),
                                    }));
                                }
                            }
                            server::UiCommand::ConfigList => {
                                if let Ok(cfg) = config::Config::load() {
                                    let config_data = cfg.list();
                                    let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::ConfigData {
                                        config: serde_json::to_value(&config_data).unwrap_or_default(),
                                    }));
                                }
                            }
                            server::UiCommand::ConfigSet { key, value } => {
                                if let Ok(mut cfg) = config::Config::load() {
                                    if cfg.set(&key, &value).is_ok() {
                                        // Config updated
                                    }
                                }
                            }
                            server::UiCommand::ConfigBootstrapAdd { multiaddr } => {
                                if let Ok(mut cfg) = config::Config::load() {
                                    let _ = cfg.add_bootstrap_node(multiaddr.clone());
                                }
                            }
                            server::UiCommand::ConfigBootstrapRemove { multiaddr } => {
                                if let Ok(mut cfg) = config::Config::load() {
                                    let _ = cfg.remove_bootstrap_node(&multiaddr);
                                }
                            }
                            server::UiCommand::FactoryReset => {
                                println!("{} Factory Reset initiated from UI...", "[WARN]".yellow());
                                // Attempt to clean data dir. This is aggressive.
                                if let Ok(data_dir) = config::Config::data_dir() {
                                     // On unix we can delete even if open? Sometimes.
                                     // Best effort: Log and Exit
                                     println!("Process will exit to clear data.");
                                     let _ = std::fs::remove_dir_all(&data_dir);
                                }
                                std::process::exit(0);
                            }
                            server::UiCommand::Restart => {
                                println!("Restart requested from UI - shutting down...");
                                std::process::exit(0);
                            }
                            server::UiCommand::DaemonRpc { id, intent } => {
                                let intent: ClientIntent = serde_json::from_str(&intent)
                                    .unwrap_or(ClientIntent::GetIdentity {});
                                let push = |result: serde_json::Value| {
                                    let resp = rpc_result(Some(serde_json::Value::String(id.clone())), result);
                                    if let Ok(v) = serde_json::to_value(&resp) {
                                        let _ = ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                                    }
                                };
                                let push_err = |code: i32, msg: String| {
                                    let resp = rpc_error(
                                        Some(serde_json::Value::String(id.clone())),
                                        scmessenger_core::wasm_support::rpc::JsonRpcErrorBody {
                                            code,
                                            message: msg,
                                            data: None,
                                        },
                                    );
                                    if let Ok(v) = serde_json::to_value(&resp) {
                                        let _ = ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                                    }
                                };
                                match intent {
                                    ClientIntent::GetIdentity {} => {
                                        let i = core_rx.get_identity_info();
                                        let mut m = serde_json::Map::new();
                                        m.insert("peer_id".to_string(), i.identity_id.into());
                                        m.insert("public_key_hex".to_string(), i.public_key_hex.into());
                                        m.insert("libp2p_peer_id".to_string(), i.libp2p_peer_id.into());
                                        m.insert("initialized".to_string(), i.initialized.into());
                                        m.insert("nickname".to_string(), i.nickname.into());
                                        push(serde_json::Value::Object(m));
                                    }
                                    ClientIntent::ScanPeers {} => {
                                        let peers: Vec<String> = peers_rx
                                            .lock()
                                            .await
                                            .keys()
                                            .map(|p| p.to_string())
                                            .collect();
                                        let mut m = serde_json::Map::new();
                                        m.insert("peers".to_string(), peers.into());
                                        push(serde_json::Value::Object(m));
                                    }
                                    ClientIntent::GetTopology {} => {
                                        let peer_count = peers_rx.lock().await.len();
                                        let (known_peers, bootstrap_nodes) = {
                                            let l = ledger_rx.lock().await;
                                            let known = l.entry_count_with_known_topics();
                                            (known, web_ctx.bootstrap_nodes.clone())
                                        };
                                        let topo = MeshTopologyUpdateParams {
                                            peer_count,
                                            known_peers,
                                            bootstrap_nodes,
                                        };
                                        if let Ok(v) = serde_json::to_value(&topo) {
                                            push(v);
                                        }
                                    }
                                    ClientIntent::SendMessage {
                                        recipient,
                                        message,
                                        id: msg_id,
                                    } => {
                                        let peer_id_res = recipient.parse::<libp2p::PeerId>();
                                        let contact_res = contacts_rx.get(recipient.clone());
                                        let target_peer = if let Ok(pid) = peer_id_res {
                                            Some(pid)
                                        } else if let Ok(Some(contact)) = contact_res {
                                            contact.peer_id.parse().ok()
                                        } else {
                                            None
                                        };
                                        let Some(target) = target_peer else {
                                            push_err(-32001, "Recipient not found".into());
                                            continue;
                                        };
                                        let pk_opt = if let Ok(Some(c)) = contacts_rx.get(target.to_string()) {
                                            Some(c.public_key)
                                        } else {
                                            None
                                        };
                                        let Some(pk) = pk_opt else {
                                            push_err(-32002, "No public key for recipient".into());
                                            continue;
                                        };
                                        let wire_message = crate::api::build_identity_wrapped_text(&core_rx, &swarm_handle, &message).await;
        match core_rx.prepare_message_with_id(pk.clone(), wire_message, scmessenger_core::MessageType::Text, None) {
                                            Ok(prep) => {
                                                if swarm_handle
                                                    .send_message(target, prep.envelope_data, None, None)
                                                    .await
                                                    .is_ok()
                                                {
                                                    // True transport ACK (R2): release the outbox entry.
                                                    core_rx.mark_message_sent(prep.message_id.clone());
                                                    let mid = msg_id.clone().unwrap_or_default();
                                                    let mut m = serde_json::Map::new();
                                                    m.insert("status".to_string(), "sent".into());
                                                    m.insert("message_id".to_string(), mid.clone().into());
                                                    push(serde_json::Value::Object(m));
                                                    let _ = ui_broadcast.send(server::UiOutbound::Legacy(
                                                        server::UiEvent::MessageStatus {
                                                            message_id: mid.clone(),
                                                            status: "sent".to_string(),
                                                        },
                                                    ));
                                                    let dn = notif_delivery_status(DeliveryStatusParams {
                                                        message_id: mid,
                                                        status: "sent".to_string(),
                                                    });
                                                    if let Ok(v) = serde_json::to_value(&dn) {
                                                        let _ =
                                                            ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                                                    }
                                                } else {
                                                    push_err(-32003, "Swarm send failed".into());
                                                }
                                            }
                                            Err(e) => {
                                                push_err(-32004, format!("Prepare message: {}", e));
                                            }
                                        }
                                    }
                                    // New intents handled via WebSocket server.
                                    _ => {
                                        push_err(-32601, "Not supported in daemon context".into());
                                    }
                                }
                            }
                        }
                    }

                    // 3. Stdin (User -> App)
                    Ok(Some(line)) = stdin_lines.next_line() => {
                        let line = line.trim();
                        if line.is_empty() {
                             print!("> ");
                             let _ = std::io::Write::flush(&mut std::io::stdout());
                             continue;
                        }
                        if line == "quit" || line == "exit" {
                            println!("Shutting down...");
                            let _ = swarm_handle.shutdown().await;
                            {
                                let _ledger_guard = ledger.lock().await;
                            }
                            break;
                        }
                        // (Implement simple CLI commands if needed, mirroring old logic)
                        if line == "status" {
                             let c = peers_rx.lock().await.len();
                             println!("Peers: {}", c);
                        }
                        if line == "peers" {
                             let p = peers_rx.lock().await;
                             for k in p.keys() { println!("  {}", k); }
                        }
                        if line == "contacts" {
                            if let Ok(l) = contacts_rx.list() {
                                for c in l { println!("  {}", c.display_name()); }
                            }
                        }

                        print!("> ");
                        let _ = std::io::Write::flush(&mut std::io::stdout());
                    }
                }
    }

    Ok(())
}

/// Learn sender identity from an inbound `scm.message.identity.v1` envelope:
/// upsert the sender's nickname into contacts (fill blanks / replace synthetic
/// placeholders, never a user-set local nickname — mirrors Android's
/// `selectAuthoritativeNickname`) and backfill the public key for unknown
/// peers so replies can be encrypted.
fn learn_sender_identity_from_envelope(
    contacts: &ContactManager,
    peer_id: &str,
    decoded: &scmessenger_core::message::identity_envelope::DecodedIdentityEnvelope,
) {
    // Route hints first: envelope-sourced reachables feed the hint store so
    // outbound sends can dial them when local candidates are stale.
    if let Ok(sender_peer) = peer_id.parse::<libp2p::PeerId>() {
        let mut hints = decoded.connection_hints.clone();
        for extra in decoded
            .external_addresses
            .iter()
            .chain(decoded.listeners.iter())
        {
            hints.push(extra.clone());
        }
        scmessenger_core::transport::hint_store::annotate_peer_hints(sender_peer, &hints);
        if !hints.is_empty() {
            tracing::debug!(
                "Learned {} route hint(s) from identity envelope of {}",
                hints.len(),
                peer_id
            );
        }
    }

    let valid_key = decoded
        .public_key
        .as_deref()
        .filter(|pk| pk.len() == 64 && pk.chars().all(|c| c.is_ascii_hexdigit()))
        .map(|pk| pk.to_lowercase());

    match contacts.get(peer_id.to_string()) {
        Ok(Some(mut contact)) => {
            let authoritative =
                scmessenger_core::message::identity_envelope::select_authoritative_nickname(
                    decoded.nickname.as_deref(),
                    contact.nickname.as_deref(),
                );
            if authoritative.as_deref() != contact.nickname.as_deref() {
                let changed = authoritative.clone();
                if let Err(e) = contacts.set_nickname(peer_id.to_string(), changed) {
                    tracing::debug!(
                        "Failed to learn nickname from identity envelope for {}: {:?}",
                        peer_id,
                        e
                    );
                } else if let Some(ref name) = authoritative {
                    tracing::info!("Learned nickname '{}' for {}", name, peer_id);
                }
            }
            // Backfill a missing/placeholder public key from the envelope.
            let key_is_placeholder = contact.public_key.is_empty()
                || contact.public_key.eq_ignore_ascii_case(&contact.peer_id);
            if key_is_placeholder {
                if let Some(pk) = valid_key {
                    contact.public_key = pk;
                    let _ = contacts.add(contact);
                }
            }
        }
        _ => {
            // Unknown sender: create a minimal contact so the nickname and
            // encryption key survive restarts. The envelope key is signed
            // material so binding it is corroborated; without one, keep an
            // empty-key placeholder (never peer_id-as-public_key -- that
            // poisons identity resolution and receipt encryption).
            if let Some(name) = decoded.nickname.as_deref() {
                let has_verified_key = valid_key.is_some();
                let mut contact = Contact::new(peer_id.to_string(), valid_key.unwrap_or_default())
                    .with_nickname(name.to_string());
                if !has_verified_key {
                    contact.notes = Some(
                        "public_key unavailable: awaiting verified key from signed source"
                            .to_string(),
                    );
                }
                contact.local_nickname = None;
                if let Err(e) = contacts.add(contact) {
                    tracing::debug!(
                        "Failed to upsert contact {} from identity envelope: {:?}",
                        peer_id,
                        e
                    );
                } else {
                    tracing::info!(
                        "Learned new contact '{}' ({}) from identity envelope",
                        name,
                        peer_id
                    );
                }
            }
        }
    }
}

/// Shared outbox flush logic used across CLI event loops (`cmd_relay` and `cmd_start`).
/// Drains queued messages for a given peer and attempts immediate delivery over the swarm.
async fn flush_outbox_for_peer(
    outbox: &Arc<tokio::sync::Mutex<Outbox>>,
    swarm_handle: &SwarmHandle,
    peer_id: PeerId,
) {
    let queued = {
        let mut ob = outbox.lock().await;
        ob.drain_for_peer(&peer_id.to_string())
    };

    if !queued.is_empty() {
        tracing::info!(
            "Flushing {} queued message(s) to peer {}",
            queued.len(),
            peer_id
        );
    }

    for msg in queued {
        let msg_id = msg.message_id.clone();
        match swarm_handle
            .send_message(peer_id, msg.envelope_data.clone(), None, None)
            .await
        {
            Ok(()) => {
                tracing::info!("Flushed queued message {} to {}", msg_id, peer_id);
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to flush queued message {} to {}: {} — re-enqueuing",
                    msg_id,
                    peer_id,
                    e
                );
                let mut ob = outbox.lock().await;
                if let Err(eq_err) = ob.enqueue(msg) {
                    tracing::error!("Failed to re-enqueue message {}: {}", msg_id, eq_err);
                }
            }
        }
    }
}

/// Headless relay/bootstrap node — runs the full mesh functionality without
/// interactive console. Designed for server, Docker, and GCP deployment.
///
/// Capabilities:
/// - Uses persisted headless network identity (no persisted user profile init)
/// - Starts libp2p swarm listening on configurable multiaddr
/// - Operates as a relay node: forwards all mesh traffic
/// - Subscribes to lobby + mesh gossipsub topics
/// - Serves HTTP landing page and REST API for status
/// - Periodically re-dials bootstrap peers for mesh continuity
/// - Runs forever (no stdin, no quit command)
async fn cmd_relay(
    listen_addr: String,
    http_port: u16,
    node_name: Option<String>,
    http_bind: Option<String>,
) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = Arc::new(IronCore::with_storage(path_to_string(&storage_path)?));
    // Load existing identity (if any) so the relay can migrate its network key
    // from the IronCore identity, preserving the PeerId on first upgrade.
    let _ = core.initialize_identity();
    let network_keypair = load_or_create_headless_network_keypair(&storage_path, &core)?;
    let local_peer_id = network_keypair.public().to_peer_id();
    let display_name = node_name
        .clone()
        .unwrap_or_else(|| format!("relay-{}", &local_peer_id.to_string()[..8]));

    // Sync nickname to IronCore identity if provided
    if let Some(ref name) = node_name {
        if let Err(e) = core.set_nickname(name.clone()) {
            tracing::warn!("Failed to sync relay nickname to identity: {}", e);
        }
    }

    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════╗".bright_cyan()
    );
    println!(
        "{}",
        "║        SCMessenger Relay/Bootstrap Node (headless)       ║".bright_cyan()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════╝".bright_cyan()
    );
    println!();
    println!("  Node Name:    {}", display_name.bright_green());
    println!(
        "  Peer ID:      {}",
        local_peer_id.to_string().bright_cyan()
    );
    println!("=== OWN_IDENTITY: {} ===", local_peer_id);
    println!(
        "  Public Key:   {}",
        "(headless/identity-agnostic)".bright_yellow()
    );
    println!("  P2P Listen:   {}", listen_addr.green());
    println!("  HTTP Status:  http://0.0.0.0:{}", http_port);
    println!("  WS Bridge:    ws://0.0.0.0:9002 (libp2p-ws)");
    println!(
        "  Discovery:    http://localhost:{}/api/network-info",
        http_port
    );
    println!();

    // Load config for bootstrap nodes
    let config = config::Config::load()?;
    let all_bootstrap = bootstrap::merge_bootstrap_nodes(config.bootstrap_nodes.clone());
    println!(
        "  Bootstrap:    {} node(s)",
        all_bootstrap.len().to_string().bright_cyan()
    );
    for (i, node) in all_bootstrap.iter().enumerate() {
        println!("    {}. {}", i + 1, node.dimmed());
    }
    println!();

    // Connection ledger — dial state over the single core store
    let mut connection_ledger = ledger::ConnectionLedger::new(
        scmessenger_core::store::LedgerManager::new(path_to_string(&storage_path)?),
    );
    let known_topics = connection_ledger.all_known_topics();
    for node in &all_bootstrap {
        connection_ledger.add_bootstrap(node, Some(&local_peer_id.to_string()));
    }
    let ledger = Arc::new(tokio::sync::Mutex::new(connection_ledger));

    // Peers map
    let peers: Arc<tokio::sync::Mutex<HashMap<libp2p::PeerId, Option<String>>>> =
        Arc::new(tokio::sync::Mutex::new(HashMap::new()));

    // Create transport bridge
    let transport_bridge = Arc::new(tokio::sync::Mutex::new(
        transport_bridge::TransportBridge::new(),
    ));

    // Web context for landing page + API
    let web_ctx = Arc::new(server::WebContext {
        node_peer_id: local_peer_id.to_string(),
        node_public_key: String::new(),
        bootstrap_nodes: all_bootstrap.clone(),
        ledger: ledger.clone(),
        peers: peers.clone(),
        start_time: std::time::Instant::now(),
        transport_bridge: transport_bridge.clone(),
        ui_port: http_port,
        core: Some(Arc::clone(&core)),
    });

    // Start HTTP server (landing page + WebSocket)
    let (ui_broadcast, _ui_cmd_rx) = server::start(http_port, web_ctx.clone()).await?;
    println!(
        "{} HTTP server started on port {}",
        "[OK]".green(),
        http_port
    );

    // Start swarm
    let listen_multiaddr: libp2p::Multiaddr =
        listen_addr.parse().context("Invalid listen multiaddr")?;
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(256);

    let discovery_config =
        scmessenger_core::transport::DiscoveryConfig::new(if config.enable_mdns {
            scmessenger_core::transport::DiscoveryMode::Open
        } else {
            scmessenger_core::transport::DiscoveryMode::Manual
        });

    // Parse bootstrap node multiaddrs from config
    let bootstrap_multiaddrs: Vec<libp2p::Multiaddr> = all_bootstrap
        .iter()
        .filter_map(|addr| addr.parse().ok())
        .collect();
    if !bootstrap_multiaddrs.is_empty() {
        println!(
            " Auto-dialing {} bootstrap node(s)",
            bootstrap_multiaddrs.len()
        );
    }

    let listen_port = listen_multiaddr.iter().find_map(|p| match p {
        libp2p::multiaddr::Protocol::Tcp(port) => Some(port),
        _ => None,
    });
    let multiport_config = scmessenger_core::transport::multiport::MultiPortConfig {
        preferred_port: listen_port,
        ..Default::default()
    };

    // Core handle: see the note on the same call in `cmd_start`. The relay's
    // ledger-exchange payload now comes from `IronCore::ledger_manager`.
    let swarm_handle = transport::start_swarm_with_config(
        network_keypair,
        Some(listen_multiaddr.clone()),
        event_tx,
        Some(multiport_config),
        bootstrap_multiaddrs,
        None,
        Some(Arc::downgrade(&core)),
        true,
        Some(discovery_config),
        transport::default_routing_engine_handle(),
    )
    .await?;
    println!("{} P2P swarm started on {}", "[OK]".green(), listen_addr);

    // Subscribe to topics
    for topic in known_topics {
        let _ = swarm_handle.subscribe_topic(topic).await;
    }
    // Subscribe to default topics (hardcoded - matches bootstrap.rs)
    for topic in [scmessenger_core::TOPIC_LOBBY, scmessenger_core::TOPIC_MESH] {
        let _ = swarm_handle.subscribe_topic(topic.to_string()).await;
    }
    println!("{} Subscribed to mesh topics", "[OK]".green());

    // Contacts + History (for relay message handling)
    let contacts = core.contacts_store_manager();
    let _history = core.history_store_manager();

    // Outbox
    let outbox = Outbox::open_default(&data_dir).map_err(|e| anyhow::anyhow!(e))?;

    // Control API — core is already Arc<IronCore>
    let core_arc = Arc::clone(&core);
    let api_ctx = api::ApiContext {
        core: core_arc.clone(),
        swarm_handle: Arc::new(swarm_handle.clone()),
    };
    let http_bind_api = http_bind.clone();
    tokio::spawn(async move {
        if let Err(e) = api::start_api_server(api_ctx, http_bind_api).await {
            tracing::error!("API server error: {}", e);
        }
    });
    println!(
        "{} Control API: {}",
        "[OK]".green(),
        format!("http://127.0.0.1:{}", api::API_PORT).dimmed()
    );

    // The BLE central task is the sole CoreBluetooth manager owner on macOS;
    // do not start a second probe worker in the headless startup path.
    if config.enable_ble && !cfg!(target_os = "macos") {
        tokio::spawn(async move {
            ble_daemon::probe_and_log().await;
        });
        let core_ble = Arc::clone(&core_arc);
        let ui_ble = ui_broadcast.clone();
        tokio::spawn(async move {
            ble_mesh::run_ble_central_ingress(core_ble, ui_ble).await;
        });

        let core_ble_adv = Arc::clone(&core_arc);
        let ui_ble_adv = ui_broadcast.clone();
        tokio::spawn(async move {
            ble_mesh::run_ble_peripheral_advertising(core_ble_adv, ui_ble_adv).await;
        });
    }

    // ── Initial bootstrap dial ──────────────────────────────────────────
    let relay_scheduler = Arc::new(DialScheduler::new(ledger.clone(), swarm_handle.clone()));
    {
        let scheduler = Arc::clone(&relay_scheduler);
        let ledger_clone = ledger.clone();
        let swarm_clone = swarm_handle.clone();
        let data_dir_migrate = data_dir.clone();
        tokio::spawn(async move {
            let my_addrs = get_local_transport_addresses(&swarm_clone).await;
            let addrs = {
                let l = ledger_clone.lock().await;
                match l.run_legacy_migration(
                    &data_dir_migrate,
                    Some(&local_peer_id.to_string()),
                    &my_addrs,
                ) {
                    Ok(report) if report.offered > 0 => {
                        println!(
                            "[INFO] Migrated {} of {} legacy peers.json entries into the core ledger ({} rejected, archived={})",
                            report.imported, report.offered, report.rejected, report.archived
                        );
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("[WARNING] legacy peers.json migration failed: {:#}", e);
                    }
                }
                l.dialable_addresses(Some(&local_peer_id.to_string()), &my_addrs)
            };
            for (i, (multiaddr_str, peer_id_opt)) in addrs.iter().enumerate() {
                let label =
                    ledger::extract_ip_port(multiaddr_str).unwrap_or_else(|| multiaddr_str.clone());
                println!("  {}.  Dialing {} ...", i + 1, label);

                let peer_id = peer_id_opt.as_ref().and_then(|s| s.parse::<PeerId>().ok());
                scheduler.dial(multiaddr_str.clone(), peer_id);

                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        });
    }

    // ── Periodic bootstrap re-dial (every 120 seconds) ──────────────────
    {
        let scheduler = Arc::clone(&relay_scheduler);
        let ledger_clone = ledger.clone();
        let swarm_clone = swarm_handle.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;
                let my_addrs = get_local_transport_addresses(&swarm_clone).await;
                let addrs = {
                    let l = ledger_clone.lock().await;
                    l.dialable_addresses(Some(&local_peer_id.to_string()), &my_addrs)
                };
                for (multiaddr_str, peer_id_opt) in &addrs {
                    let peer_id = peer_id_opt.as_ref().and_then(|s| s.parse::<PeerId>().ok());
                    scheduler.dial(multiaddr_str.clone(), peer_id);
                }
                tracing::info!("Periodic re-dial: {} addresses attempted", addrs.len());
            }
        });
    }

    // ── Status broadcast (every 10 seconds) ─────────────────────────────
    let ui_broadcast_clone = ui_broadcast.clone();
    let peers_status = peers.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            let count = peers_status.lock().await.len();
            let _ = ui_broadcast_clone.send(server::UiOutbound::Legacy(
                server::UiEvent::NetworkStatus {
                    status: "online".to_string(),
                    peer_count: count,
                },
            ));
        }
    });

    println!();
    println!("{}", "Relay node is running. Press Ctrl+C to stop.".bold());
    println!();

    // ── Main event loop (headless — no stdin) ───────────────────────────
    let contacts_rx = contacts.clone();
    let ledger_rx = ledger.clone();
    let outbox_rx = outbox.clone();
    let scheduler_rx = Arc::clone(&relay_scheduler);

    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                match event {
                    SwarmEvent::PeerDiscovered(peer_id) => {
                        let mut p = peers.lock().await;
                        if let std::collections::hash_map::Entry::Vacant(e) = p.entry(peer_id) {
                            e.insert(None);
                            tracing::info!("Peer discovered: {}", peer_id);
                            let _ = contacts_rx.update_last_seen(peer_id.to_string());

                            let (public_key, identity) = contacts_rx.get(peer_id.to_string())
                                .ok().flatten()
                                .map(|c| (Some(c.public_key), Some(c.peer_id.clone())))
                                .unwrap_or((None, None));

                            let _ = ui_broadcast.send(server::UiOutbound::Legacy(server::UiEvent::PeerDiscovered {
                                peer_id: peer_id.to_string(),
                                transport: "tcp".to_string(),
                                public_key: public_key.unwrap_or_default(),
                                identity: identity.unwrap_or_default(),
                            }));
                            let n = notif_peer_discovered(PeerDiscoveredParams {
                                peer_id: peer_id.to_string(),
                                transport: "tcp".to_string(),
                                public_key: contacts_rx
                                    .get(peer_id.to_string())
                                    .ok()
                                    .flatten()
                                    .map(|c| c.public_key),
                            });
                            if let Ok(v) = serde_json::to_value(&n) {
                                let _ = ui_broadcast.send(server::UiOutbound::JsonRpc(v));
                            }

                            // Register peer with transport bridge using default capabilities
                            let capabilities = vec![TransportType::Internet, TransportType::Local];
                            let mut bridge = transport_bridge.lock().await;
                            bridge.register_peer(peer_id, capabilities);
                            let can_reach = bridge.can_reach_destination(&peer_id);

                            // Share ledger with new peer. Payload built inside the swarm
                            // from `exchange_response_entries`.
                            if let Err(e) = swarm_handle.share_ledger(peer_id).await {
                                tracing::warn!("Failed to share ledger with {}: {}", peer_id, e);
                            }

                            // Flush outbox for this peer (only if transport-reachable)
                            if !can_reach {
                                tracing::warn!("No compatible transport path to {}; deferring outbox flush", peer_id);
                            } else {
                                flush_outbox_for_peer(&outbox_rx, &swarm_handle, peer_id).await;
                            }
                        }
                    }
                    SwarmEvent::PeerDisconnected(peer_id) => {
                        peers.lock().await.remove(&peer_id);
                        let mut l = ledger_rx.lock().await;
                        if let Some(multiaddr) = l.find_peer_multiaddr(&peer_id.to_string()) {
                            l.record_failure(&multiaddr);
                        }
                        // Release the per-peer concurrent-connection slot (P0
                        // cap) -- same wiring as cmd_start's handler.
                        l.record_disconnect(peer_id);
                        tracing::info!("Peer disconnected: {}", peer_id);
                    }
                    SwarmEvent::RelayCircuitEstablished => {
                        scheduler_rx.set_relay_healthy(true);
                    }
                    SwarmEvent::RelayCircuitBroken => {
                        scheduler_rx.set_relay_healthy(false);
                    }
                    SwarmEvent::LedgerReceived { from_peer, entries } => {
                        let mut l = ledger_rx.lock().await;
                        let new_count = l.merge_shared_entries(&entries);
                        if new_count > 0 {
                            tracing::info!("Learned {} new peers from {}", new_count, from_peer);
                            // The core store persists its own updates.
                            let new_entries: Vec<(String, Option<String>)> = entries
                                .iter()
                                .map(|e| {
                                    (e.multiaddr.clone(), e.last_peer_id.clone())
                                })
                                .collect();
                            drop(l);

                            // Graceful-AF dial policy: know our own addresses before
                            // promiscuously dialing whatever the ledger handed us
                            let my_addrs = get_local_transport_addresses(&swarm_handle).await;

                            for (addr_str, peer_id_opt) in new_entries {
                                let local_peer_id_string = local_peer_id.to_string();
                                if peer_id_opt.as_deref() == Some(local_peer_id_string.as_str())
                                    || ledger::contains_peer_id_component(
                                        &addr_str,
                                        &local_peer_id_string,
                                    )
                                {
                                    tracing::debug!(
                                        "Skipping self-targeted ledger address: {}",
                                        addr_str
                                    );
                                    continue;
                                }
                                // Skip non-routable addresses (loopback, link-local,
                                // site-local) a peer may advertise
                                if !ledger::is_dialable_multiaddr(&addr_str, ledger::NetworkMode::Local, ledger::DnsPolicy::Reject) {
                                    continue;
                                }
                                if !ledger::is_dialable_for_this_node(&addr_str, ledger::NetworkMode::Local, &my_addrs) {
                                    continue;
                                }
                                let peer_id = peer_id_opt
                                    .as_ref()
                                    .and_then(|s| s.parse::<PeerId>().ok());
                                // Fire-and-forget: scheduler spawns the actual dial
                                // so this shared event loop never blocks on a
                                // connection/timeout.
                                scheduler_rx.dial(addr_str, peer_id);
                            }
                        }
                    }
                    SwarmEvent::PeerIdentified { peer_id, listen_addrs, .. } => {
                        let mut l = ledger_rx.lock().await;
                        let advertised: Vec<String> =
                            listen_addrs.iter().map(|a| a.to_string()).collect();
                        l.record_identified_peer(&peer_id.to_string(), &advertised);
                        if let Err(e) = swarm_handle.share_ledger(peer_id).await {
                            tracing::warn!("Failed to share ledger with identified peer {}: {}", peer_id, e);
                        }
                        flush_outbox_for_peer(&outbox_rx, &swarm_handle, peer_id).await;
                    }
                    SwarmEvent::TopicDiscovered { peer_id, topic } => {
                        tracing::info!("Topic discovered from {}: {}", peer_id, topic);
                        let mut l = ledger_rx.lock().await;
                        if let Some(multiaddr) = l.find_peer_multiaddr(&peer_id.to_string()) {
                            l.record_topic(&multiaddr, &topic);
                        }
                    }
                    SwarmEvent::MessageReceived { peer_id, envelope_data } => {
                        // In relay mode, we automatically peel and forward onion layers
                        if let Ok(msg) = core_arc.receive_message(envelope_data.clone()) {
                            if msg.message_type == scmessenger_core::MessageType::OnionRelay {
                                let next_hop_hex = msg.recipient_id.clone();
                                let payload = msg.payload.clone();

                                if let Ok(next_hop_bytes) = hex::decode(&next_hop_hex) {
                                    if let Ok(libp2p_kp) = libp2p::identity::ed25519::Keypair::try_from_bytes(&mut next_hop_bytes[..32].to_vec()) {
                                        let next_peer_id = libp2p::PeerId::from_public_key(&libp2p::identity::PublicKey::from(libp2p_kp.public()));

                                        tracing::info!("Relay node: forwarding onion packet to {}", next_peer_id);
                                        let swarm_clone = swarm_handle.clone();
                                        tokio::spawn(async move {
                                            let _ = swarm_clone.send_message(next_peer_id, payload, None, None).await;
                                        });
                                    }
                                }
                            }
                        }

                        // Also log standard envelopes for debugging
                        if let Ok(env) = decode_envelope(&envelope_data) {
                            let sender_key = hex::encode(&env.sender_public_key);
                            tracing::debug!(
                                "Relayed envelope from {} sender={} bytes={}",
                                peer_id,
                                &sender_key[..sender_key.len().min(12)],
                                envelope_data.len()
                            );
                        }
                    }
                    SwarmEvent::ListeningOn(addr) => {
                        tracing::info!("Listening on {}", addr);
                    }
                    _ => {}
                }
            }

            // Ctrl+C shutdown
            _ = tokio::signal::ctrl_c() => {
                println!("\nShutting down relay node...");
                let _ = swarm_handle.shutdown().await;
                // The core store persists its own mutations; nothing to flush.
                let _ledger_guard = ledger.lock().await;
                break;
            }
        }
    }

    println!("{} Relay node stopped.", "[OK]".green());
    Ok(())
}

async fn cmd_send_offline(recipient: String, message: String) -> Result<()> {
    // Try to use API if a node is running
    if api::is_api_available().await {
        api::send_message_via_api(&recipient, &message)
            .await
            .context("Failed to send message via API")?;
        println!("{} Message sent via running node", "[OK]".green());
        return Ok(());
    }

    // P0_TRANSPORT_001: Fallback to native IronCore send when API unavailable
    // Start a temporary swarm to send the message directly (not just queue)
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = Arc::new(IronCore::with_storage(path_to_string(&storage_path)?));
    let info = core.get_identity_info();
    if !info.initialized {
        anyhow::bail!(
            "No identity found for data directory: {}. Run 'scm init' first.",
            data_dir.display()
        );
    }
    core.grant_consent();
    core.initialize_identity()
        .context("Failed to load identity")?;

    let contacts = core.contacts_store_manager();

    let contact = find_contact(&contacts, &recipient).context("Contact not found")?;

    // Get the network keypair from the core
    let network_keypair = core
        .get_libp2p_keypair()
        .context("Failed to get network keypair")?;

    // Build discovery config
    let discovery_config = scmessenger_core::transport::DiscoveryConfig::new(
        scmessenger_core::transport::DiscoveryMode::Manual,
    );

    // Build swarm for immediate send
    println!(
        "{} Starting temporary swarm for immediate send...",
        "".yellow()
    );
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(16);
    let core_for_events = Arc::clone(&core);
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            if let SwarmEvent::MessageReceived {
                peer_id,
                envelope_data,
            } = event
            {
                match core_for_events.receive_message(envelope_data) {
                    Ok(message) => tracing::debug!(
                        "Temporary CLI swarm processed {:?} from {}",
                        message.message_type,
                        peer_id
                    ),
                    Err(error) => tracing::warn!(
                        "Temporary CLI swarm failed to process message from {}: {}",
                        peer_id,
                        error
                    ),
                }
            }
        }
    });
    let routing_handle = scmessenger_core::transport::default_routing_engine_handle();

    let swarm_handle = match scmessenger_core::transport::start_swarm(
        network_keypair,
        None, // Let swarm auto-select port
        event_tx,
        Some(Arc::downgrade(&core)),
        true, // headless mode for CLI send
        Some(discovery_config),
        routing_handle,
    )
    .await
    {
        Ok(swarm) => swarm,
        Err(e) => {
            // If swarm startup fails, fall back to queuing
            tracing::warn!("Failed to start swarm: {}, falling back to queue", e);
            return queue_message_for_later_delivery(&data_dir, &contact, &message).await;
        }
    };

    // Prepare the message envelope
    // Identity-envelope parity: wrap chat text so the receiver learns our
    // nickname + route hints.
    let wire_message =
        crate::api::build_identity_wrapped_text(&core, &swarm_handle, &message).await;
    let (envelope_bytes, prepared_message_id) = core
        .prepare_message(
            contact.public_key.clone(),
            wire_message,
            scmessenger_core::MessageType::Text,
            None,
        )
        .map(|pm| (pm.envelope_data, pm.message_id))
        .context("Failed to encrypt message")?;

    println!(
        "{} Message encrypted: {} bytes",
        "[OK]".green(),
        envelope_bytes.len()
    );

    // Send the message via the swarm
    let recipient_peer_id = contact
        .peer_id
        .parse::<libp2p::PeerId>()
        .context("Invalid peer ID in contact: {}")?;
    println!(
        "{} Sending message to {}...",
        "[OK]".green(),
        recipient_peer_id
    );

    let retry_policy = scmessenger_core::RetryPolicy::default();
    let mut attempts = 0;
    let mut last_error: Option<String> = None;

    loop {
        attempts += 1;
        match swarm_handle
            .send_message(recipient_peer_id, envelope_bytes.clone(), None, None)
            .await
        {
            Ok(_) => {
                // True transport ACK (R2): release the outbox entry without a
                // Delivered receipt. send_message() awaits the actual delivery,
                // not a buffer enqueue.
                core.mark_message_sent(prepared_message_id.clone());
                println!(
                    "{} Message sent successfully to {} (attempt {}/{})",
                    "[OK]".green(),
                    recipient_peer_id,
                    attempts,
                    retry_policy.max_retries
                );
                return Ok(());
            }
            Err(e) => {
                last_error = Some(format!("{}", e));
                tracing::warn!(
                    "Send attempt {}/{} failed: {}",
                    attempts,
                    retry_policy.max_retries,
                    e
                );

                if !retry_policy.can_retry(attempts) {
                    break;
                }

                if let Some(delay) = retry_policy.delay_for_attempt(attempts + 1) {
                    if delay > tokio::time::Duration::ZERO {
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
    }

    // All retries failed - fall back to queuing
    println!(
        "{} All send attempts failed ({}), falling back to queue",
        "[WARN]".yellow(),
        last_error.unwrap_or("unknown error".to_string())
    );
    queue_message_for_later_delivery(&data_dir, &contact, &message).await
}

/// Queue a message in the outbox for later delivery.
/// Used when the swarm send fails or the API is unavailable.
async fn queue_message_for_later_delivery(
    data_dir: &std::path::Path,
    contact: &Contact,
    message: &str,
) -> Result<()> {
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    let info = core.get_identity_info();
    if !info.initialized {
        anyhow::bail!(
            "No identity found for data directory: {}. Run 'scm init' first.",
            data_dir.display()
        );
    }
    core.grant_consent();
    core.initialize_identity()
        .context("Failed to initialize identity for queued send")?;

    // NOTE (UNIFICATION_V3 D1 fix): the envelope's wire message id MUST be
    // reused as the outbox entry key. A Delivered receipt returned by the
    // recipient carries the wire id, and `Outbox::remove` matches strictly on
    // `message_id`. Mismatching the two (previously a fresh UUID here) meant
    // the receipt never cleared the outbox entry -> infinite retry storm.
    let prepared = core.prepare_message(
        contact.public_key.clone(),
        message.to_string(),
        scmessenger_core::MessageType::Text,
        None,
    )?;
    let envelope_bytes = prepared.envelope_data;
    let wire_message_id = prepared.message_id;

    match Outbox::open_default(data_dir) {
        Ok(outbox_arc) => {
            let mut outbox = outbox_arc.lock().await;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let queued_msg = QueuedMessage {
                version: 1,
                message_id: wire_message_id,
                recipient_id: contact.peer_id.clone(),
                envelope_data: envelope_bytes,
                queued_at: now,
                attempts: 0,
                next_retry_at: None,
                in_custody: false,
                custody_established_at: 0,
                state: scmessenger_core::store::outbox::MessageState::Enqueued,
            };
            match outbox.enqueue(queued_msg) {
                Ok(()) => {
                    println!(
                        "{} Message queued for {} — will be delivered when peer comes online",
                        "[OK]".green(),
                        contact.display_name().bright_cyan(),
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to enqueue message for {}: {}", contact.peer_id, e);
                    println!("{} Could not queue message: {}", "[WARN]".yellow(), e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Could not open outbox for queuing: {}", e);
            println!(
                "{} Message encrypted but could not be queued (outbox unavailable: {})",
                "[WARN]".yellow(),
                e
            );
        }
    }

    Ok(())
}

async fn cmd_discovery(action: DiscoveryAction) -> Result<()> {
    if !api::is_api_available().await {
        println!(
            "{}",
            "No SCMessenger node is running. Discovery commands require a running node.".yellow()
        );
        return Ok(());
    }

    match action {
        DiscoveryAction::Status => {
            let status = api::get_discovery_status().await?;
            println!("{}", "Local Discovery Status".bold());
            println!(
                "  mDNS:       {}",
                if status.mdns_enabled {
                    "ENABLED".green()
                } else {
                    "DISABLED".red()
                }
            );
            println!(
                "  BLE:        {}",
                if status.ble_enabled {
                    "ENABLED".green()
                } else {
                    "DISABLED".red()
                }
            );
            println!(
                "  WiFi-Aware: {}",
                if status.wifi_aware_enabled {
                    "ENABLED".green()
                } else {
                    "DISABLED".red()
                }
            );
        }
        DiscoveryAction::Scan => {
            print!("Triggering discovery scan... ");
            api::trigger_discovery_scan().await?;
            println!("{}", "Done.".green());
        }
        DiscoveryAction::Peers => {
            let peers = api::get_discovery_peers().await?;
            println!("{}", "Locally Discovered Peers".bold());
            if peers.is_empty() {
                println!("  {}", "No peers discovered via local transports.".dimmed());
            } else {
                for peer in peers {
                    println!(
                        "  • {} ({})",
                        peer.peer_id.bright_cyan(),
                        peer.transport.bright_yellow()
                    );
                    if let Some(name) = peer.nickname {
                        println!("    Name: {}", name);
                    }
                }
            }
        }
    }
    Ok(())
}

async fn cmd_status() -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);

    let contacts = core.contacts_store_manager();
    let history = core.history_store_manager();
    let stats = history.stats().map_err(|e| anyhow::anyhow!("{:?}", e))?;

    println!("{}", "SCMessenger Status".bold());
    println!();

    println!("Contacts: {}", contacts.list().unwrap_or_default().len());
    println!(
        "Messages: {} (sent: {}, received: {})",
        stats.total_messages, stats.sent_count, stats.received_count
    );

    // BLE status check (task_wire_is_ble_available)
    println!(
        "BLE: {}",
        if ble_daemon::is_ble_available().await {
            "Available".green()
        } else {
            "Unavailable".yellow()
        }
    );

    if api::is_api_available().await {
        println!();
        println!("{}", "Runtime Network Surface".bold());

        match api::get_peers_via_api().await {
            Ok(peers) => {
                println!("Peers: {}", peers.len());
                for peer in peers {
                    let rep_color = if peer.reputation > 80.0 {
                        "green"
                    } else if peer.reputation < 30.0 {
                        "red"
                    } else {
                        "yellow"
                    };
                    println!(
                        "  - {} (reputation: {})",
                        peer.peer_id.dimmed(),
                        format!("{:.1}", peer.reputation).color(rep_color)
                    );
                }
            }
            Err(e) => println!("Peers: {} ({})", "unavailable".yellow(), e),
        }

        match api::get_listeners_via_api().await {
            Ok(listeners) => println!("Listeners: {}", listeners.len()),
            Err(e) => println!("Listeners: {} ({})", "unavailable".yellow(), e),
        }

        match api::get_external_address_via_api().await {
            Ok(addrs) => {
                if addrs.is_empty() {
                    println!("External Addresses: {}", "(none)".dimmed());
                } else {
                    println!("External Addresses:");
                    for addr in addrs {
                        println!("  - {}", addr.dimmed());
                    }
                }
            }
            Err(e) => println!("External Addresses: {} ({})", "unavailable".yellow(), e),
        }

        match api::get_connection_path_state_via_api().await {
            Ok(state) => println!("Connection Path State: {}", state.bright_cyan()),
            Err(e) => println!("Connection Path State: {} ({})", "unavailable".yellow(), e),
        }

        match api::get_drift_state_via_api().await {
            Ok(status) => {
                let state_color = if status.state == "Active" {
                    status.state.bright_green()
                } else {
                    status.state.yellow()
                };
                println!(
                    "Drift Protocol:        {} (store: {} msgs)",
                    state_color, status.store_size
                );
            }
            Err(e) => println!("Drift Protocol:        {} ({})", "unavailable".yellow(), e),
        }

        match api::export_diagnostics_via_api().await {
            Ok(diag) => {
                println!("Diagnostics JSON bytes: {}", diag.len());
            }
            Err(e) => println!("Diagnostics: {} ({})", "unavailable".yellow(), e),
        }
    }

    Ok(())
}

async fn cmd_mark_sent(message_id: String) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    let removed = core.mark_message_sent(message_id.clone());
    if removed {
        println!(
            "{} Marked message as sent: {}",
            "[OK]".green(),
            message_id.bright_cyan()
        );
    } else {
        println!(
            "{} Message ID not found in outbox: {}",
            "[WARN]".yellow(),
            message_id.dimmed()
        );
    }
    Ok(())
}

async fn cmd_history_clear(yes: bool) -> Result<()> {
    if !yes {
        anyhow::bail!("Refusing destructive clear without --yes");
    }
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    let history = core.history_store_manager();
    history.clear().map_err(|e| anyhow::anyhow!("{:?}", e))?;
    println!("{} Cleared all message history", "[OK]".green());
    Ok(())
}

async fn cmd_history_enforce_retention(max_messages: u32) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    let history = core.history_store_manager();
    let pruned = history
        .enforce_retention(max_messages)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    println!(
        "{} Retention enforced (max={}): pruned {}",
        "[OK]".green(),
        max_messages,
        pruned
    );
    Ok(())
}

async fn cmd_history_prune_before(before_timestamp: u64) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    let history = core.history_store_manager();
    let pruned = history
        .prune_before(before_timestamp)
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    println!(
        "{} Pruned {} message(s) older than {}",
        "[OK]".green(),
        pruned,
        before_timestamp
    );
    Ok(())
}

async fn cmd_block(action: BlockAction) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    core.initialize_identity()
        .context("Failed to load identity")?;

    match action {
        BlockAction::Add {
            peer_id,
            device_id,
            reason,
        } => {
            core.block_peer(peer_id.clone(), device_id.clone(), reason.clone())
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("{} Blocked peer: {}", "[OK]".green(), peer_id.bright_cyan());
            if let Some(device_id) = device_id {
                println!("  Device ID: {}", device_id.dimmed());
            }
            if let Some(r) = reason {
                println!("  Reason: {}", r.dimmed());
            }
        }
        BlockAction::Remove { peer_id, device_id } => {
            core.unblock_peer(peer_id.clone(), device_id.clone())
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            println!(
                "{} Unblocked peer: {}",
                "[OK]".green(),
                peer_id.bright_cyan()
            );
            if let Some(device_id) = device_id {
                println!("  Device ID: {}", device_id.dimmed());
            }
        }
        BlockAction::Delete {
            peer_id,
            device_id,
            reason,
        } => {
            core.block_and_delete_peer(peer_id.clone(), device_id.clone(), reason.clone())
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            println!(
                "{} Blocked and deleted peer: {} (messages purged)",
                "[OK]".green(),
                peer_id.bright_cyan()
            );
            if let Some(device_id) = device_id {
                println!("  Device ID: {}", device_id.dimmed());
            }
            if let Some(r) = reason {
                println!("  Reason: {}", r.dimmed());
            }
        }
        BlockAction::List => {
            let list = core
                .list_blocked_peers()
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if list.is_empty() {
                println!("{}", "No blocked peers.".dimmed());
            } else {
                println!("{} ({} total)", "Blocked Peers".bold(), list.len());
                println!();
                for item in list {
                    let status = if item.is_deleted {
                        "blocked+deleted".bright_red()
                    } else {
                        "blocked".yellow()
                    };
                    println!(
                        "  {} {} ({})",
                        "•".bright_red(),
                        item.peer_id.bright_cyan(),
                        status
                    );
                    println!(
                        "    Blocked at: {}",
                        format_timestamp(item.blocked_at).dimmed()
                    );
                    if let Some(ref reason) = item.reason {
                        println!("    Reason: {}", reason.dimmed());
                    }
                }
            }
        }
        BlockAction::Check { peer_id, device_id } => {
            let blocked = core
                .is_peer_blocked(peer_id.clone(), device_id.clone())
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            if blocked {
                println!("{} {} is blocked", "[FAIL]".red(), peer_id.bright_cyan());
                if let Some(device_id) = device_id {
                    println!("  Device ID: {}", device_id.dimmed());
                }
            } else {
                println!(
                    "{} {} is NOT blocked",
                    "[OK]".green(),
                    peer_id.bright_cyan()
                );
                if let Some(device_id) = device_id {
                    println!("  Device ID: {}", device_id.dimmed());
                }
            }
        }
        BlockAction::Count => {
            let count = core.blocked_count().map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("Blocked peers: {}", count);
        }
    }

    Ok(())
}

async fn cmd_history_get(id: String) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    let history = core.history_store_manager();

    match history.get(id.clone()) {
        Ok(Some(msg)) => {
            let direction = match msg.direction {
                MessageDirection::Sent => "→ Sent",
                MessageDirection::Received => "← Received",
            };
            println!("{}", "Message Details".bold());
            println!("  ID:        {}", msg.id.bright_cyan());
            println!("  Direction: {}", direction);
            println!("  Peer:      {}", msg.peer_id);
            println!("  Time:      {}", format_timestamp(msg.timestamp));
            println!("  Delivered: {}", msg.delivered);
            println!("  Content:   {}", msg.content);
        }
        Ok(None) => {
            println!("{} Message not found: {}", "[WARN]".yellow(), id.dimmed());
        }
        Err(e) => {
            anyhow::bail!("Failed to retrieve message: {:?}", e);
        }
    }

    Ok(())
}

async fn cmd_history_stats() -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    let history = core.history_store_manager();
    let stats = history.stats().map_err(|e| anyhow::anyhow!("{:?}", e))?;

    println!("{}", "History Statistics".bold());
    println!("  Total:       {}", stats.total_messages);
    println!("  Sent:        {}", stats.sent_count);
    println!("  Received:    {}", stats.received_count);
    println!("  Undelivered: {}", stats.undelivered_count);

    // Count messages per peer (wired from count_with_peer pattern)
    let contacts_mgr = core.contacts_store_manager();
    if let Ok(contact_list) = contacts_mgr.list() {
        for contact in contact_list.iter().take(5) {
            let peer_id = &contact.peer_id;
            let count = history
                .conversation(peer_id.clone(), u32::MAX)
                .unwrap_or_default()
                .len() as u64;
            let display = contact.nickname.as_deref().unwrap_or(peer_id);
            println!("  {} messages: {}", count, display);
        }
    }

    Ok(())
}

async fn cmd_history_count() -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    let history = core.history_store_manager();
    println!("History count: {}", history.count());
    Ok(())
}

async fn cmd_history_mark_delivered(id: String) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    let history = core.history_store_manager();
    history
        .mark_delivered(id.clone())
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    println!(
        "{} Marked message as delivered: {}",
        "[OK]".green(),
        id.bright_cyan()
    );
    Ok(())
}

async fn cmd_history_clear_conversation(peer: String) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    let history = core.history_store_manager();

    // Try to resolve peer name to peer_id via contacts
    let contacts = core.contacts_store_manager();
    let peer_id = if let Ok(contact) = find_contact(&contacts, &peer) {
        contact.peer_id
    } else {
        peer.clone()
    };

    history
        .remove_conversation(peer_id.clone())
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    println!(
        "{} Cleared conversation with {}",
        "[OK]".green(),
        peer_id.bright_cyan()
    );
    Ok(())
}

async fn cmd_history_delete(id: String) -> Result<()> {
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);
    let history = core.history_store_manager();
    history
        .delete(id.clone())
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    println!("{} Deleted message: {}", "[OK]".green(), id.bright_cyan());
    Ok(())
}

async fn cmd_test() -> Result<()> {
    println!("{}", "Running self-tests...".bold());
    println!();

    let alice = IronCore::new();
    let bob = IronCore::new();

    alice.initialize_identity()?;
    bob.initialize_identity()?;

    let _alice_info = alice.get_identity_info();
    let bob_info = bob.get_identity_info();

    println!("{} Identity generation", "[OK]".green());

    let envelope = alice.prepare_message(
        bob_info
            .public_key_hex
            .clone()
            .expect("Bob's public key should be available"),
        "Test message".to_string(),
        scmessenger_core::MessageType::Text,
        None,
    )?;

    println!(
        "{} Message encryption ({} bytes)",
        "[OK]".green(),
        envelope.envelope_data.len()
    );

    let msg = bob.receive_message(envelope.envelope_data)?;
    assert_eq!(
        msg.text_content()
            .expect("Message text should be available"),
        "Test message"
    );

    println!("{} Message decryption", "[OK]".green());

    let eve = IronCore::new();
    eve.initialize_identity()?;

    let envelope = alice.prepare_message(
        bob_info
            .public_key_hex
            .clone()
            .expect("Bob's public key should be available"),
        "Secret".to_string(),
        scmessenger_core::MessageType::Text,
        None,
    )?;

    assert!(eve.receive_message(envelope.envelope_data).is_err());
    println!("{} Encryption security", "[OK]".green());

    println!();
    println!("{}", "All tests passed!".green().bold());

    Ok(())
}

/// Returns true if `s` is exactly 64 hex characters — the shape of a
/// Blake3 identity_id (32-byte hash → 64 hex chars).  A user who copies their
/// `scm identity` "ID" field will get this format.
/// NOTE: This also matches valid Ed25519 public keys (also 64 hex chars).
/// Use looks_like_ed25519_pk() to distinguish.
fn looks_like_blake3_id(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c: char| c.is_ascii_hexdigit())
}

/// Returns true if `s` is a valid Ed25519 public key (64 hex chars that decode
/// to a valid curve point).  Distinguishes public keys from Blake3 identity IDs,
/// which are also 64 hex chars but are NOT valid Ed25519 points.
fn looks_like_ed25519_pk(s: &str) -> bool {
    if s.len() != 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }
    if let Ok(bytes) = hex::decode(s) {
        if bytes.len() == 32 {
            if let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice()) {
                // Use libp2p's ed25519 crate instead of ed25519_dalek
                return libp2p::identity::ed25519::PublicKey::try_from_bytes(&arr).is_ok();
            }
        }
    }
    false
}

/// Returns true if `s` can be parsed as a valid libp2p PeerId
/// (base58-encoded multihash, e.g. "12D3Koo...").
fn looks_like_libp2p_peer_id(s: &str) -> bool {
    s.parse::<libp2p::PeerId>().is_ok()
}

fn find_contact(manager: &ContactManager, query: &str) -> Result<Contact> {
    let list = manager.list().unwrap_or_default();
    let query_lower = query.to_lowercase();

    list.into_iter()
        .find(|c| {
            c.peer_id == query
                || c.nickname.as_ref().map(|n| n.to_lowercase()).as_deref() == Some(&query_lower)
                || c.public_key == query
        })
        .ok_or_else(|| anyhow::anyhow!("Contact not found: {}", query))
}

fn format_timestamp(timestamp: u64) -> String {
    use chrono::{DateTime, Local, Utc};

    let dt = DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_else(Utc::now);
    let local: DateTime<Local> = dt.into();

    local.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn prune_logs(log_dir: &std::path::Path, max_days: u64) -> Result<()> {
    if !log_dir.exists() {
        return Ok(());
    }

    let now = std::time::SystemTime::now();
    let max_age = std::time::Duration::from_secs(max_days * 24 * 60 * 60);

    for entry in std::fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(age) = now.duration_since(modified) {
                        if age > max_age {
                            tracing::info!("Pruning old log file: {}", path.display());
                            let _ = std::fs::remove_file(path);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

async fn cmd_audit(action: AuditAction) -> Result<()> {
    let _config = config::Config::load()?;
    let data_dir = config::Config::data_dir()?;
    let storage_path = data_dir.join("storage");
    let core = IronCore::with_storage(path_to_string(&storage_path)?);

    match action {
        AuditAction::Export { output } => {
            let json = core
                .export_audit_log()
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
            if let Some(path) = output {
                std::fs::write(&path, json)?;
                println!("{} Audit log exported to {}", "[OK]".green(), path);
            } else {
                println!("{}", json);
            }
        }
        AuditAction::Verify => match core.validate_audit_chain() {
            Ok(_) => println!("{} Audit chain integrity verified: OK", "[OK]".green()),
            Err(e) => println!("{} Audit chain validation failed: {:?}", "[FAIL]".red(), e),
        },
        AuditAction::Stats => {
            let events = core.get_audit_events_since(0);
            println!("{}", "Audit Log Statistics".bold());
            println!("  Total Events:   {}", events.len());
            if let Some(first) = events.first() {
                println!(
                    "  First Event:    {}",
                    format_timestamp(first.timestamp_unix_secs)
                );
            }
            if let Some(last) = events.last() {
                println!(
                    "  Last Event:     {}",
                    format_timestamp(last.timestamp_unix_secs)
                );
            }
        }
    }
    Ok(())
}

async fn cmd_swarm(action: SwarmAction) -> Result<()> {
    match action {
        SwarmAction::Stats => cmd_swarm_stats().await,
    }
}

async fn cmd_swarm_stats() -> Result<()> {
    println!("{}", "SCMessenger Swarm Connection Stats".bold());
    println!();

    if api::is_api_available().await {
        match api::get_swarm_stats_via_api().await {
            Ok(stats) => {
                if stats.is_empty() {
                    println!(
                        "{}",
                        "No active peer connections in the swarm stats.".yellow()
                    );
                    println!();
                    println!("  Start the mesh node with: {}", "scm relay".dimmed());
                    println!(
                        "  Or start the messaging node with: {}",
                        "scm start".dimmed()
                    );
                } else {
                    println!(
                        "{:<52} {:<12} {:<10} {:<14} {:<20}",
                        "Peer ID", "State", "Latency", "Sent/Failed", "Bytes Sent/Recv"
                    );
                    println!(
                        "{:-<52} {:-<12} {:-<10} {:-<14} {:-<20}",
                        "", "", "", "", ""
                    );

                    for peer in &stats {
                        let state_colored = match peer.state.as_str() {
                            "Connected" => peer.state.green(),
                            "Connecting" => peer.state.yellow(),
                            "Failed" => peer.state.red(),
                            _ => peer.state.normal(),
                        };
                        let latency = format!("{}ms", peer.avg_latency_ms);
                        let sent_failed =
                            format!("{}/{}", peer.messages_sent, peer.message_failures);
                        let bytes_sent_recv =
                            format!("{}/{}", peer.bytes_sent, peer.bytes_received);

                        println!(
                            "{:<52} {:<12} {:<10} {:<14} {:<20}",
                            peer.peer_id.dimmed(),
                            state_colored,
                            latency,
                            sent_failed,
                            bytes_sent_recv,
                        );
                    }

                    println!();
                    println!(
                        "{} {} peer(s) in the swarm stats.",
                        "ℹ".dimmed(),
                        stats.len()
                    );
                }
            }
            Err(e) => {
                println!("{} Failed to fetch swarm stats: {}", "[FAIL]".red(), e);
            }
        }

        if let Ok(listeners) = api::get_listeners_via_api().await {
            println!("  Listeners: {}", listeners.len());
        }

        if let Ok(addrs) = api::get_external_address_via_api().await {
            if !addrs.is_empty() {
                println!("  External addresses:");
                for addr in &addrs {
                    println!("    - {}", addr.dimmed());
                }
            }
        }
    } else {
        let _config = config::Config::load()?;
        let data_dir = config::Config::data_dir()?;
        let storage_path = data_dir.join("storage");
        let core = IronCore::with_storage(path_to_string(&storage_path)?);

        let stats = core.get_all_connection_stats();

        if stats.is_empty() {
            println!(
                "{}",
                "No active peer connections in the swarm stats.".yellow()
            );
            println!();
            println!("  Start the mesh node with: {}", "scm relay".dimmed());
            println!(
                "  Or start the messaging node with: {}",
                "scm start".dimmed()
            );
        } else {
            println!(
                "{:<52} {:<12} {:<10} {:<14} {:<20}",
                "Peer ID", "State", "Latency", "Sent/Failed", "Bytes Sent/Recv"
            );
            println!(
                "{:-<52} {:-<12} {:-<10} {:-<14} {:-<20}",
                "", "", "", "", ""
            );

            for (peer_id, stat) in &stats {
                let state_str = match stat.state {
                    scmessenger_core::transport::health::ConnectionState::Connecting => {
                        "Connecting"
                    }
                    scmessenger_core::transport::health::ConnectionState::Connected => "Connected",
                    scmessenger_core::transport::health::ConnectionState::Disconnecting => {
                        "Disconnecting"
                    }
                    scmessenger_core::transport::health::ConnectionState::Disconnected => {
                        "Disconnected"
                    }
                    scmessenger_core::transport::health::ConnectionState::Failed => "Failed",
                };
                let state_colored = match state_str {
                    "Connected" => state_str.green(),
                    "Connecting" => state_str.yellow(),
                    "Failed" => state_str.red(),
                    _ => state_str.normal(),
                };
                let latency = format!("{}ms", stat.avg_latency_ms);
                let sent_failed = format!("{}/{}", stat.messages_sent, stat.message_failures);
                let bytes_sent_recv = format!("{}/{}", stat.bytes_sent, stat.bytes_received);

                println!(
                    "{:<52} {:<12} {:<10} {:<14} {:<20}",
                    peer_id.to_string().dimmed(),
                    state_colored,
                    latency,
                    sent_failed,
                    bytes_sent_recv,
                );
            }

            println!();
            println!(
                "{} {} peer(s) in the swarm stats.",
                "ℹ".dimmed(),
                stats.len()
            );
        }
    }

    Ok(())
}
