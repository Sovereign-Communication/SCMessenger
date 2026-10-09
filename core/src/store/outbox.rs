// Outbox — queue messages for peers that may be offline
//
// Messages are stored locally and retried when the peer comes online.
// This is the foundation for store-and-forward delivery.

use crate::store::backend::StorageBackend;
use crate::store::storage::StorageManager;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct LegacyQueuedMessage {
    pub message_id: String,
    pub recipient_id: String,
    pub envelope_data: Vec<u8>,
    pub queued_at: u64,
    pub attempts: u32,
    pub next_retry_at: Option<u64>,
    #[serde(default = "default_false")]
    pub in_custody: bool,
    #[serde(default = "default_zero")]
    pub custody_established_at: u64,
    #[serde(default = "default_enqueued")]
    pub state: MessageState,
}

fn deserialize_queued_message(data: &[u8]) -> Result<QueuedMessage, bincode::Error> {
    if data.is_empty() {
        return Err(Box::new(bincode::ErrorKind::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "empty",
        ))));
    }
    // If the first byte is a known version, it's the versioned format (same
    // layout for both versions; see `QUEUED_VERSION_FIRE_AND_FORGET`).
    // Legacy format starts with a String length. Since string lengths are usually 36 (for UUIDs),
    // their first byte is 36, not 1 or 2.
    if data[0] == QUEUED_VERSION_STANDARD || data[0] == QUEUED_VERSION_FIRE_AND_FORGET {
        bincode::deserialize(data)
    } else {
        let legacy: LegacyQueuedMessage = bincode::deserialize(data)?;
        Ok(QueuedMessage {
            version: 1,
            message_id: legacy.message_id,
            recipient_id: legacy.recipient_id,
            envelope_data: legacy.envelope_data,
            queued_at: legacy.queued_at,
            attempts: legacy.attempts,
            next_retry_at: legacy.next_retry_at,
            in_custody: legacy.in_custody,
            custody_established_at: legacy.custody_established_at,
            state: legacy.state,
        })
    }
}

/// `QueuedMessage::version` of an ordinary message: re-dispatched until an
/// application-level delivery receipt clears it.
pub const QUEUED_VERSION_STANDARD: u8 = 1;

/// `QueuedMessage::version` of a delivery receipt. Same wire layout as
/// `QUEUED_VERSION_STANDARD`; the version byte doubles as the kind marker so it
/// persists with the entry and survives restarts without a schema change.
/// Nothing ever acknowledges a receipt, so waiting for one re-sent the same
/// envelope every grace window forever. A receipt is dispatched best-effort:
/// cleared once the transport accepts it, and dropped once the message it
/// refers to has aged out of protocol retention (`MESSAGE_RETENTION_SECS`).
/// There is no attempt-count cap: attempts are driven by reconnect events.
pub const QUEUED_VERSION_FIRE_AND_FORGET: u8 = 2;

/// Protocol message retention: the lifetime of a message (the drift envelope
/// default TTL, and the outbox expiry applied by maintenance). A receipt does
/// not carry its target's TTL, so this is the bound on how long the target
/// message can still matter to the sender.
pub const MESSAGE_RETENTION_SECS: u64 = 7 * 24 * 60 * 60;

/// Maximum messages queued per peer
const MAX_QUEUE_PER_PEER: usize = 1000;

/// Maximum total messages across all peers
const MAX_TOTAL_QUEUED: usize = 10_000;

/// Maximum delivery attempts before automatic removal
const MAX_DELIVERY_ATTEMPTS: u32 = 12;

const QUEUE_PREFIX: &[u8] = b"outbox_";
const RECEIPT_AUTH_PREFIX: &[u8] = b"receipt_auth_";
const RECEIPT_AUTHORIZATION_TTL_SECS: u64 = 7 * 24 * 60 * 60;

fn current_unix_secs() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Clone, Serialize, Deserialize)]
struct ReceiptAuthorization {
    recipient_public_key: [u8; 32],
    expires_at: u64,
}

fn receipt_authorization_key(message_id: &str) -> Vec<u8> {
    [RECEIPT_AUTH_PREFIX, message_id.as_bytes()].concat()
}

/// Resolve a recipient identifier to the 32-byte key an authorization binds to.
///
/// Canonicalized first, so every spelling of one peer -- Ed25519 hex from the
/// transport, a base58 PeerId from a contact row, the queue key `enqueue` wrote
/// -- resolves to the SAME key. A base58 PeerId would otherwise fail to decode
/// and silently drop the authorization, leaving the sender's retry state
/// unreleasable. A value that canonicalizes to no key material at all
/// (placeholder, truncated, non-hex garbage) still returns `None`: fail closed.
fn decode_recipient_key(recipient_id: &str) -> Option<[u8; 32]> {
    let bytes = hex::decode(canonical_peer_key(recipient_id)).ok()?;
    bytes.try_into().ok()
}

fn queued_recipient_matches(queued: &QueuedMessage, message_id: &str, key: &[u8]) -> bool {
    queued.message_id == message_id
        && hex::decode(&queued.recipient_id).is_ok_and(|queued_key| queued_key.as_slice() == key)
}

fn decode_receipt_authorization(value: &[u8], now: u64) -> Option<ReceiptAuthorization> {
    let authorization: ReceiptAuthorization = bincode::deserialize(value).ok()?;
    (authorization.expires_at >= now).then_some(authorization)
}

fn remember_memory_receipt_authorization(
    authorizations: &mut HashMap<String, ReceiptAuthorization>,
    message_id: &str,
    recipient_public_key: [u8; 32],
    now: u64,
) -> Result<(), String> {
    authorizations.retain(|_, authorization| authorization.expires_at >= now);
    if authorizations
        .get(message_id)
        .is_some_and(|existing| existing.recipient_public_key != recipient_public_key)
    {
        return Err("message ID is already bound to another recipient".to_string());
    }
    authorizations.insert(
        message_id.to_string(),
        ReceiptAuthorization {
            recipient_public_key,
            expires_at: now.saturating_add(RECEIPT_AUTHORIZATION_TTL_SECS),
        },
    );
    Ok(())
}

fn prune_expired_receipt_authorizations(db: &dyn StorageBackend, now: u64) {
    if let Ok(entries) = db.scan_prefix(RECEIPT_AUTH_PREFIX) {
        for (key, value) in entries {
            if bincode::deserialize::<ReceiptAuthorization>(&value)
                .is_ok_and(|authorization| authorization.expires_at < now)
            {
                let _ = db.remove(&key);
            }
        }
    }
}

fn remember_persistent_receipt_authorization(
    db: &dyn StorageBackend,
    message_id: &str,
    recipient_public_key: [u8; 32],
    now: u64,
) -> Result<(), String> {
    let key = receipt_authorization_key(message_id);
    let existing = db.get(&key)?;
    if let Some(value) = &existing {
        let authorization: ReceiptAuthorization = bincode::deserialize(value)
            .map_err(|error| format!("invalid receipt authorization record: {error}"))?;
        if authorization.expires_at >= now
            && authorization.recipient_public_key != recipient_public_key
        {
            return Err("message ID is already bound to another recipient".to_string());
        }
    }
    let value = bincode::serialize(&ReceiptAuthorization {
        recipient_public_key,
        expires_at: now.saturating_add(RECEIPT_AUTHORIZATION_TTL_SECS),
    })
    .map_err(|error| error.to_string())?;
    db.put(&key, &value)?;
    db.flush()
}

/// Canonical queue key for a peer, so a queue written under one representation
/// of a peer is still drained when the connection event arrives under another.
///
/// The tree genuinely disagrees with itself about this. The transport drains via
/// `handle_peer_connection_event_with_egress(pk_hex, ..)` (`transport::swarm`),
/// i.e. Ed25519 public-key hex, while contact rows and CLI argument paths can
/// still carry a base58 libp2p PeerId -- the same identity in two spellings.
/// Exact-string lookups bridge neither, and the cost is on record in-tree:
/// `cli/src/main.rs::peer_id_from_contact_identifier` documents that parsing the
/// hex as base58 made `send` report "Invalid peer ID in contact" for a message it
/// had already enqueued, and the queue-key form of the same mismatch is the
/// "keyed under one spelling, drained with another, stranded forever" finding.
///
/// Delegates to `store::ledger_entry::canonical_ledger_peer_id`, the tree's only
/// other implementation of this mapping, so the outbox and the ledger agree by
/// construction instead of by coincidence.
///
/// Deliberately NOT an identity guess: this maps spellings of THE SAME key
/// material and nothing else. Two different keys stay different, and any value
/// whose key cannot be recovered (a hashed PeerId, an identity_id, a placeholder
/// like "peer_a") is returned unchanged.
fn canonical_peer_key(recipient_id: &str) -> String {
    let trimmed = recipient_id.trim();
    crate::store::ledger_entry::canonical_ledger_peer_id(trimmed, None)
        .unwrap_or_else(|| trimmed.to_string())
}

/// Every spelling this store could have written for one peer's queue.
///
/// All of them are derivable from the requested value, so resolution never needs
/// to scan the queue: a stored key either canonicalizes to this peer -- and is
/// therefore one of these spellings -- or it belongs to a different peer.
fn queue_key_candidates(recipient_id: &str) -> Vec<String> {
    let canonical = canonical_peer_key(recipient_id);
    let mut candidates = vec![
        // Canonical first: this is where `enqueue` writes, so the common case
        // costs one probe.
        canonical.clone(),
        recipient_id.trim().to_string(),
        canonical.to_uppercase(),
    ];
    // A queue written before canonicalization may sit under the base58 PeerId.
    if let Some(peer_id) = crate::store::ledger_entry::peer_id_from_public_key_hex(&canonical) {
        candidates.push(peer_id);
    }
    let mut spellings: Vec<String> = Vec::new();
    for candidate in candidates {
        if !candidate.is_empty() && !spellings.contains(&candidate) {
            spellings.push(candidate);
        }
    }
    spellings
}

/// Message state for tracking lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageState {
    /// Message is queued and ready to send
    Enqueued,
    /// Message has been sent successfully
    Sent,
    /// Message failed permanently and won't be retried
    Failed,
}

/// A queued outbound message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    /// Version byte for serialization format
    #[serde(default = "default_version")]
    pub version: u8,
    /// Unique message ID
    pub message_id: String,
    /// Target peer's identity ID
    pub recipient_id: String,
    /// Serialized envelope bytes
    pub envelope_data: Vec<u8>,
    /// When this was queued (unix timestamp)
    pub queued_at: u64,
    /// Number of delivery attempts
    pub attempts: u32,
    /// Next retry time (unix timestamp)
    pub next_retry_at: Option<u64>,
    /// Whether this message is currently in relay custody
    #[serde(default = "default_false")]
    pub in_custody: bool,
    /// Timestamp when custody was established
    #[serde(default = "default_zero")]
    pub custody_established_at: u64,
    /// Current state of the message
    #[serde(default = "default_enqueued")]
    pub state: MessageState,
}

impl QueuedMessage {
    /// True for an entry that no peer will ever acknowledge (a delivery
    /// receipt), so it must not wait for an acknowledgement to be cleared.
    pub fn is_fire_and_forget(&self) -> bool {
        self.version == QUEUED_VERSION_FIRE_AND_FORGET
    }

    /// True once a fire-and-forget entry (a delivery receipt) is no longer
    /// useful: the message it confirms has outlived protocol retention, so the
    /// original sender can no longer be waiting on it. The receipt is queued
    /// when the target arrives, so `queued_at` bounds the target's age from
    /// below; this is never earlier than the target's own expiry. Ordinary
    /// messages never expire here.
    pub fn receipt_expired(&self, now: u64) -> bool {
        self.is_fire_and_forget() && now.saturating_sub(self.queued_at) >= MESSAGE_RETENTION_SECS
    }
}

fn default_version() -> u8 {
    1
}

fn default_false() -> bool {
    false
}

fn default_zero() -> u64 {
    0
}

fn default_enqueued() -> MessageState {
    MessageState::Enqueued
}

/// Storage backend for outbox
enum OutboxBackend {
    Memory {
        queues: HashMap<String, VecDeque<QueuedMessage>>,
        total: usize,
        receipt_authorizations: HashMap<String, ReceiptAuthorization>,
    },
    Persistent(Arc<dyn StorageBackend>),
}

/// Outbound message queue with automatic retention enforcement
pub struct Outbox {
    backend: OutboxBackend,
    storage_manager: Option<Arc<StorageManager>>,
}

impl Outbox {
    /// Create a new in-memory outbox
    pub fn new() -> Self {
        Self {
            backend: OutboxBackend::Memory {
                queues: HashMap::new(),
                total: 0,
                receipt_authorizations: HashMap::new(),
            },
            storage_manager: None,
        }
    }

    /// Create a persistent outbox with an arbitrary backend and storage manager
    pub fn persistent_with_storage(
        backend: Arc<dyn StorageBackend>,
        storage_manager: Arc<StorageManager>,
    ) -> Self {
        Self {
            backend: OutboxBackend::Persistent(backend),
            storage_manager: Some(storage_manager),
        }
    }

    /// Create a persistent outbox with an arbitrary backend
    pub fn persistent(backend: Arc<dyn StorageBackend>) -> Self {
        Self {
            backend: OutboxBackend::Persistent(backend),
            storage_manager: None,
        }
    }

    /// Open or create the default persistent outbox for the given data directory.
    /// Returns Arc<tokio::sync::Mutex<Self>> matching CLI usage pattern.
    ///
    /// This is the single source of truth for outbox initialization across the CLI.
    /// It creates a persistent outbox using Sled storage, falling back to in-memory
    /// if the persistent storage fails to initialize.
    ///
    /// # Arguments
    /// * `data_dir` - Path to the application's data directory
    ///
    /// # Returns
    /// * `Ok(Arc<tokio::sync::Mutex<Self>>)` on success
    /// * `Err(String)` on failure to initialize either storage option
    ///
    /// CLI/desktop-only: uses SledStorage (filesystem-path-backed), which is
    /// itself gated to `cfg(not(target_arch = "wasm32"))` -- WASM has no
    /// traditional filesystem path to open and uses rexie/IndexedDB instead.
    /// Only called from cli/src/main.rs; nothing in wasm/ references this.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open_default(
        data_dir: &std::path::Path,
    ) -> std::result::Result<Arc<tokio::sync::Mutex<Self>>, String> {
        let outbox_path = data_dir.join("outbox");

        // Try to create persistent outbox first
        match crate::store::backend::SledStorage::new(
            outbox_path
                .to_str()
                .ok_or_else(|| "Invalid UTF-8 path".to_string())?,
        ) {
            Ok(backend) => {
                let outbox = Self::persistent(Arc::new(backend));
                Ok(Arc::new(tokio::sync::Mutex::new(outbox)))
            }
            Err(e) => {
                tracing::warn!("Failed to open persistent outbox: {}", e);
                tracing::info!("Falling back to in-memory outbox");
                let outbox = Self::new();
                Ok(Arc::new(tokio::sync::Mutex::new(outbox)))
            }
        }
    }

    /// Trigger maintenance to enforce retention policies after outbox operations.
    /// This automatically prunes expired messages and enforces configured limits.
    /// If storage_manager is not available (None), this is a no-op.
    fn trigger_maintenance(&self) {
        if let Some(storage_mgr) = &self.storage_manager {
            // Trigger maintenance - this will enforce retention policies
            let _ = storage_mgr.perform_maintenance();
        }
    }

    /// Queue a message for delivery
    pub fn enqueue(&mut self, msg: QueuedMessage) -> std::result::Result<(), String> {
        // Structured tracing: packet lifecycle span for message correlation
        let _span = tracing::info_span!(
            "packet_lifecycle",
            message_id = %msg.message_id,
            recipient = %msg.recipient_id
        )
        .entered();

        tracing::info!(
            event = "outbox_enqueue",
            message_id = %msg.message_id,
            recipient_id = %msg.recipient_id,
            queued_at = msg.queued_at,
            attempts = msg.attempts,
            payload_size = msg.envelope_data.len()
        );

        // One queue per peer, whichever spelling the caller holds -- and the
        // same spelling `resolve_queue_key` will look under on reconnect.
        let queue_key = canonical_peer_key(&msg.recipient_id);

        match &mut self.backend {
            OutboxBackend::Memory {
                queues,
                total,
                receipt_authorizations,
            } => {
                if *total >= MAX_TOTAL_QUEUED {
                    return Err(format!("Outbox full ({} messages)", MAX_TOTAL_QUEUED));
                }

                let queue = queues.entry(queue_key.clone()).or_default();

                if queue.len() >= MAX_QUEUE_PER_PEER {
                    return Err(format!(
                        "Queue full for peer {} ({} messages)",
                        msg.recipient_id, MAX_QUEUE_PER_PEER
                    ));
                }
                if let Some(recipient_key) = decode_recipient_key(&queue_key) {
                    remember_memory_receipt_authorization(
                        receipt_authorizations,
                        &msg.message_id,
                        recipient_key,
                        current_unix_secs(),
                    )?;
                }
                queue.push_back(msg);
                *total += 1;
                // Trigger maintenance on memory outbox
                // Note: This is a best-effort call and any errors are silently ignored
                self.trigger_maintenance();
                Ok(())
            }
            OutboxBackend::Persistent(db) => {
                let storage_key = format!(
                    "{}{}_{}",
                    String::from_utf8_lossy(QUEUE_PREFIX),
                    queue_key,
                    msg.message_id
                );
                let previous_queue = db
                    .get(storage_key.as_bytes())
                    .map_err(|error| error.to_string())?;
                let queue_exists = previous_queue.is_some();
                let current_total = db
                    .count_prefix(QUEUE_PREFIX)
                    .map_err(|error| error.to_string())?;
                if current_total >= MAX_TOTAL_QUEUED && !queue_exists {
                    return Err(format!("Outbox full ({} messages)", MAX_TOTAL_QUEUED));
                }

                // Check per-peer limit
                let peer_prefix =
                    format!("{}{}_", String::from_utf8_lossy(QUEUE_PREFIX), queue_key);
                let peer_count = db
                    .count_prefix(peer_prefix.as_bytes())
                    .map_err(|error| format!("failed counting queued peer messages: {error}"))?;
                if peer_count >= MAX_QUEUE_PER_PEER && !queue_exists {
                    return Err(format!(
                        "Queue full for peer {} ({} messages)",
                        msg.recipient_id, MAX_QUEUE_PER_PEER
                    ));
                }

                // Store message
                let bytes = bincode::serialize(&msg).map_err(|error| error.to_string())?;
                if let Some(recipient_key) = decode_recipient_key(&queue_key) {
                    let auth_key = receipt_authorization_key(&msg.message_id);
                    let previous = db.get(&auth_key)?;
                    remember_persistent_receipt_authorization(
                        db.as_ref(),
                        &msg.message_id,
                        recipient_key,
                        current_unix_secs(),
                    )?;
                    if let Err(error) = db
                        .put(storage_key.as_bytes(), &bytes)
                        .and_then(|_| db.flush())
                    {
                        match previous_queue {
                            Some(value) => db.put(storage_key.as_bytes(), &value)?,
                            None => db.remove(storage_key.as_bytes())?,
                        }
                        match previous {
                            Some(value) => db.put(&auth_key, &value)?,
                            None => db.remove(&auth_key)?,
                        }
                        db.flush()?;
                        return Err(error);
                    }
                } else {
                    db.put(storage_key.as_bytes(), &bytes)?;
                    db.flush()?;
                }
                // Trigger maintenance on persistent outbox
                self.trigger_maintenance();
                Ok(())
            }
        }
    }

    /// Get all queued messages for a peer (without removing them)
    pub fn peek_for_peer(&self, recipient_id: &str) -> Vec<QueuedMessage> {
        let resolved_key = self.resolve_queue_key(recipient_id);
        let recipient_id: &str = &resolved_key;
        match &self.backend {
            OutboxBackend::Memory { queues, .. } => queues
                .get(recipient_id)
                .map(|q| q.iter().cloned().collect())
                .unwrap_or_default(),
            OutboxBackend::Persistent(db) => {
                let prefix_str =
                    format!("{}{}_", String::from_utf8_lossy(QUEUE_PREFIX), recipient_id);
                if let Ok(results) = db.scan_prefix(prefix_str.as_bytes()) {
                    results
                        .into_iter()
                        .filter_map(|(_, value)| deserialize_queued_message(&value).ok())
                        .collect()
                } else {
                    Vec::new()
                }
            }
        }
    }

    /// Get all pending messages (messages with state == Enqueued)
    pub fn pending(&self) -> Vec<QueuedMessage> {
        match &self.backend {
            OutboxBackend::Memory { queues, .. } => {
                let mut all_pending = Vec::new();
                for queue in queues.values() {
                    for msg in queue.iter() {
                        if msg.state == MessageState::Enqueued {
                            all_pending.push(msg.clone());
                        }
                    }
                }
                all_pending
            }
            OutboxBackend::Persistent(db) => {
                let mut all_pending = Vec::new();
                if let Ok(results) = db.scan_prefix(QUEUE_PREFIX) {
                    for (_, value) in results {
                        if let Ok(msg) = deserialize_queued_message(&value) {
                            if msg.state == MessageState::Enqueued {
                                all_pending.push(msg);
                            }
                        }
                    }
                }
                all_pending
            }
        }
    }

    /// Bind a message ID to its intended recipient before it can be sent.
    pub(crate) fn authorize_recipient(
        &mut self,
        message_id: &str,
        recipient_id: &str,
    ) -> Result<(), String> {
        let recipient_key = decode_recipient_key(recipient_id)
            .ok_or_else(|| "recipient key must be 32-byte hex".to_string())?;
        let now = current_unix_secs();
        match &mut self.backend {
            OutboxBackend::Memory {
                receipt_authorizations,
                ..
            } => remember_memory_receipt_authorization(
                receipt_authorizations,
                message_id,
                recipient_key,
                now,
            ),
            OutboxBackend::Persistent(db) => remember_persistent_receipt_authorization(
                db.as_ref(),
                message_id,
                recipient_key,
                now,
            ),
        }
    }

    /// Check whether a retained authorization belongs to the authenticated recipient.
    pub(crate) fn contains_for_recipient_key(
        &self,
        message_id: &str,
        recipient_public_key: &[u8],
    ) -> bool {
        let Ok(recipient_key) = <[u8; 32]>::try_from(recipient_public_key) else {
            return false;
        };
        let now = current_unix_secs();
        let matches_authorization = |authorization: &ReceiptAuthorization| {
            authorization.expires_at >= now && authorization.recipient_public_key == recipient_key
        };

        match &self.backend {
            OutboxBackend::Memory {
                queues,
                receipt_authorizations,
                ..
            } => match receipt_authorizations.get(message_id) {
                Some(authorization) => matches_authorization(authorization),
                None => queues.iter().any(|(recipient_id, queue)| {
                    decode_recipient_key(recipient_id) == Some(recipient_key)
                        && queue.iter().any(|queued| queued.message_id == message_id)
                }),
            },
            OutboxBackend::Persistent(db) => {
                let auth_key = receipt_authorization_key(message_id);
                match db.get(&auth_key) {
                    Ok(Some(value)) => {
                        decode_receipt_authorization(&value, now).is_some_and(|authorization| {
                            authorization.recipient_public_key == recipient_key
                        })
                    }
                    Ok(None) => db.scan_prefix(QUEUE_PREFIX).is_ok_and(|entries| {
                        entries.into_iter().any(|(_, value)| {
                            deserialize_queued_message(&value).is_ok_and(|queued| {
                                queued_recipient_matches(&queued, message_id, recipient_public_key)
                            })
                        })
                    }),
                    Err(_) => false,
                }
            }
        }
    }

    /// Remove the matching retry entry and consume its retained authorization.
    pub(crate) fn remove_for_recipient_key(
        &mut self,
        message_id: &str,
        recipient_public_key: &[u8],
    ) -> bool {
        let Ok(recipient_key) = <[u8; 32]>::try_from(recipient_public_key) else {
            return false;
        };
        let now = current_unix_secs();

        match &mut self.backend {
            OutboxBackend::Memory {
                queues,
                total,
                receipt_authorizations,
            } => {
                let matching_queue = queues.iter().find_map(|(queued_recipient, queue)| {
                    if decode_recipient_key(queued_recipient) == Some(recipient_key) {
                        queue
                            .iter()
                            .position(|queued| queued.message_id == message_id)
                            .map(|position| (queued_recipient.clone(), position))
                    } else {
                        None
                    }
                });
                let authorization = receipt_authorizations.get(message_id);
                let authorized = authorization.is_some_and(|authorization| {
                    authorization.expires_at >= now
                        && authorization.recipient_public_key == recipient_key
                }) || (authorization.is_none() && matching_queue.is_some());
                if !authorized {
                    return false;
                }
                if let Some((queued_recipient, position)) = matching_queue {
                    if let Some(queue) = queues.get_mut(&queued_recipient) {
                        queue.remove(position);
                        *total -= 1;
                        if queue.is_empty() {
                            queues.remove(&queued_recipient);
                        }
                    }
                }
                receipt_authorizations.remove(message_id);
                true
            }
            OutboxBackend::Persistent(db) => {
                let auth_key = receipt_authorization_key(message_id);
                let authorization = match db.get(&auth_key) {
                    Ok(value) => value,
                    Err(_) => return false,
                };
                let matching_queue = match db.scan_prefix(QUEUE_PREFIX) {
                    Ok(entries) => entries.into_iter().find_map(|(key, value)| {
                        deserialize_queued_message(&value)
                            .ok()
                            .filter(|queued| {
                                queued_recipient_matches(queued, message_id, recipient_public_key)
                            })
                            .map(|_| (key, value))
                    }),
                    Err(_) => return false,
                };
                let authorized = match authorization.as_ref() {
                    Some(value) => {
                        decode_receipt_authorization(value, now).is_some_and(|authorization| {
                            authorization.recipient_public_key == recipient_key
                        })
                    }
                    None => matching_queue.is_some(),
                };
                if !authorized {
                    return false;
                }
                if let Some((queue_key, _)) = &matching_queue {
                    if db.remove(queue_key).is_err() {
                        return false;
                    }
                }
                if authorization.is_some() && db.remove(&auth_key).is_err() {
                    if let Some((queue_key, value)) = &matching_queue {
                        let _ = db.put(queue_key, value);
                    }
                    let _ = db.flush();
                    return false;
                }
                if db.flush().is_err() {
                    if let Some((queue_key, value)) = &matching_queue {
                        let _ = db.put(queue_key, value);
                    }
                    if let Some(value) = &authorization {
                        let _ = db.put(&auth_key, value);
                    }
                    let _ = db.flush();
                    return false;
                }
                true
            }
        }
    }

    /// Remove a specific message by ID after transport delivery. This removes
    /// retry state only; retained recipient authorization remains until an
    /// authenticated application receipt consumes it or its TTL expires.
    pub fn remove(&mut self, message_id: &str) -> bool {
        let result = match &mut self.backend {
            OutboxBackend::Memory {
                queues,
                total,
                receipt_authorizations,
            } => {
                for queue in queues.values_mut() {
                    if let Some(index) = queue
                        .iter()
                        .position(|queued| queued.message_id == message_id)
                    {
                        if let Some(recipient_key) =
                            decode_recipient_key(&queue[index].recipient_id)
                        {
                            if remember_memory_receipt_authorization(
                                receipt_authorizations,
                                message_id,
                                recipient_key,
                                current_unix_secs(),
                            )
                            .is_err()
                            {
                                return false;
                            }
                        }
                        queue.remove(index);
                        *total -= 1;
                        return true;
                    }
                }
                false
            }
            OutboxBackend::Persistent(db) => {
                let Ok(entries) = db.scan_prefix(QUEUE_PREFIX) else {
                    return false;
                };
                for (key, value) in entries {
                    let Ok(queued) = deserialize_queued_message(&value) else {
                        continue;
                    };
                    if queued.message_id != message_id {
                        continue;
                    }
                    if let Some(recipient_key) = decode_recipient_key(&queued.recipient_id) {
                        let auth_key = receipt_authorization_key(message_id);
                        let Ok(previous_authorization) = db.get(&auth_key) else {
                            return false;
                        };
                        if remember_persistent_receipt_authorization(
                            db.as_ref(),
                            message_id,
                            recipient_key,
                            current_unix_secs(),
                        )
                        .is_err()
                        {
                            return false;
                        }
                        if db.remove(&key).and_then(|_| db.flush()).is_err() {
                            let _ = db.put(&key, &value);
                            match previous_authorization {
                                Some(value) => {
                                    let _ = db.put(&auth_key, &value);
                                }
                                None => {
                                    let _ = db.remove(&auth_key);
                                }
                            }
                            let _ = db.flush();
                            return false;
                        }
                        return true;
                    }
                    return db.remove(&key).and_then(|_| db.flush()).is_ok();
                }
                false
            }
        };

        if result {
            tracing::info!(
                event = "outbox_dequeue",
                message_id = %message_id,
                reason = "transport_acknowledged"
            );
        }

        result
    }

    /// Drain all messages for a peer (for batch delivery).
    ///
    /// Drains every spelling of this peer's queue in one pass (#395):
    /// a queue split across the canonical key and a pre-canonicalization
    /// spelling is fully emptied by a single call.
    pub fn drain_for_peer(&mut self, recipient_id: &str) -> Vec<QueuedMessage> {
        // Resolve before borrowing the backend.
        let keys = self.queue_keys_with_messages(recipient_id);
        match &mut self.backend {
            OutboxBackend::Memory { queues, total, .. } => {
                let mut drained = Vec::new();
                for key in &keys {
                    if let Some(queue) = queues.remove(key.as_str()) {
                        let count = queue.len();
                        *total -= count;
                        // Filter out messages that are in custody and should not be delivered locally
                        for msg in queue.into_iter() {
                            if !msg.in_custody {
                                drained.push(msg);
                            }
                        }
                    }
                }
                drained
            }
            OutboxBackend::Persistent(db) => {
                let mut messages = Vec::new();
                let mut keys_to_remove = Vec::new();

                for key in &keys {
                    let prefix_str = format!("{}{}_", String::from_utf8_lossy(QUEUE_PREFIX), key);
                    if let Ok(results) = db.scan_prefix(prefix_str.as_bytes()) {
                        for (key, value) in results {
                            if let Ok(msg) = deserialize_queued_message(&value) {
                                if !msg.in_custody {
                                    messages.push(msg);
                                    keys_to_remove.push(key);
                                }
                            }
                        }
                    }
                }

                for key in keys_to_remove {
                    let _ = db.remove(&key);
                }
                let _ = db.flush();

                messages
            }
        }
    }

    /// Restore a message removed by `flush_peer_messages` after a delivery
    /// attempt could not be durably re-enqueued. This is intentionally
    /// crate-private and bypasses only queue-capacity checks: callers may use it
    /// only for an item they just drained, so the outbox never silently loses
    /// ownership when another producer fills the queue in between.
    pub(crate) fn restore_drained(
        &mut self,
        msg: QueuedMessage,
    ) -> std::result::Result<(), String> {
        let message_id = msg.message_id.clone();
        // Keyed like `enqueue`, so a restored entry is drainable under either
        // spelling rather than creating a second queue for the same peer.
        let queue_key = canonical_peer_key(&msg.recipient_id);
        match &mut self.backend {
            OutboxBackend::Memory { queues, total, .. } => {
                let queue = queues.entry(queue_key.clone()).or_default();
                if let Some(existing) = queue.iter_mut().find(|m| m.message_id == message_id) {
                    *existing = msg;
                } else {
                    queue.push_back(msg);
                    *total += 1;
                }
                Ok(())
            }
            OutboxBackend::Persistent(db) => {
                let key = format!(
                    "{}{}_{}",
                    String::from_utf8_lossy(QUEUE_PREFIX),
                    queue_key,
                    msg.message_id
                );
                let bytes = bincode::serialize(&msg).map_err(|error| error.to_string())?;
                db.put(key.as_bytes(), &bytes)?;
                db.flush()
            }
        }
    }

    /// Make a queued message eligible for the next reconnect flush.
    ///
    /// A transport request can fail after the reconnect flush has already
    /// restored the entry with its grace timer. Clearing that timer avoids
    /// waiting for the grace window when the request-response layer has already
    /// proved that the attempt failed. Custody entries are left alone because
    /// custody owns their retry lifecycle.
    pub(crate) fn retry_now(&mut self, message_id: &str) -> bool {
        match &mut self.backend {
            OutboxBackend::Memory { queues, .. } => {
                for queue in queues.values_mut() {
                    if let Some(msg) = queue.iter_mut().find(|m| m.message_id == message_id) {
                        if msg.in_custody {
                            return false;
                        }
                        msg.state = MessageState::Enqueued;
                        msg.next_retry_at = None;
                        return true;
                    }
                }
            }
            OutboxBackend::Persistent(db) => {
                if let Ok(results) = db.scan_prefix(QUEUE_PREFIX) {
                    for (key, value) in results {
                        if let Ok(mut msg) = deserialize_queued_message(&value) {
                            if msg.message_id == message_id {
                                if msg.in_custody {
                                    return false;
                                }
                                msg.state = MessageState::Enqueued;
                                msg.next_retry_at = None;
                                let Ok(bytes) = bincode::serialize(&msg) else {
                                    return false;
                                };
                                if db.put(&key, &bytes).is_err() || db.flush().is_err() {
                                    return false;
                                }
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// The key this peer's queue is actually stored under.
    ///
    /// Probes the derivable spellings and returns the first that holds messages,
    /// falling back to the canonical spelling -- where `enqueue` writes, so a
    /// message queued now is drainable under any spelling later.
    fn resolve_queue_key(&self, requested: &str) -> String {
        for candidate in queue_key_candidates(requested) {
            if self.key_holds_messages(&candidate) {
                return candidate;
            }
        }
        canonical_peer_key(requested)
    }

    fn key_holds_messages(&self, key: &str) -> bool {
        match &self.backend {
            OutboxBackend::Memory { queues, .. } => {
                queues.get(key).map(|q| !q.is_empty()).unwrap_or(false)
            }
            OutboxBackend::Persistent(db) => {
                let prefix = format!("{}{}_", String::from_utf8_lossy(QUEUE_PREFIX), key);
                db.count_prefix(prefix.as_bytes()).unwrap_or(0) > 0
            }
        }
    }

    /// Every spelling of this peer's queue that currently holds messages,
    /// canonical first. The drain paths use this so one pass empties all
    /// spellings (#395); `resolve_queue_key` stays the single-key resolver
    /// for read-only peeks.
    fn queue_keys_with_messages(&self, requested: &str) -> Vec<String> {
        queue_key_candidates(requested)
            .into_iter()
            .filter(|candidate| self.key_holds_messages(candidate))
            .collect()
    }

    /// Flush peer messages that are due for delivery.
    ///
    /// Drains every spelling of this peer's queue in one pass (#395):
    /// a queue split across the canonical key and a pre-canonicalization
    /// spelling is fully emptied by a single call.
    pub fn flush_peer_messages(&mut self, recipient_id: &str) -> Vec<QueuedMessage> {
        // Resolve before borrowing the backend, so the per-key drain below
        // can take &mut self.
        let keys = self.queue_keys_with_messages(recipient_id);
        let mut drained = Vec::new();
        for key in &keys {
            drained.extend(self.flush_queue_key(key));
        }
        drained
    }

    /// Drain due messages from a single resolved queue key.
    fn flush_queue_key(&mut self, queue_key: &str) -> Vec<QueuedMessage> {
        match &mut self.backend {
            OutboxBackend::Memory { queues, total, .. } => {
                let now_ms = web_time::SystemTime::now()
                    .duration_since(web_time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let now_secs = now_ms / 1000;
                let is_due = |next_retry: Option<u64>| -> bool {
                    match next_retry {
                        None => true,
                        Some(val) => {
                            if val > 10_000_000_000 {
                                val <= now_ms
                            } else {
                                val <= now_secs
                            }
                        }
                    }
                };

                if let Some(queue) = queues.get_mut(queue_key) {
                    let mut drained = Vec::new();
                    let mut remaining = VecDeque::new();
                    for msg in queue.drain(..) {
                        // Skip messages that are in custody or not Enqueued - they should not be delivered locally
                        if msg.in_custody || msg.state != MessageState::Enqueued {
                            remaining.push_back(msg);
                            continue;
                        }
                        if is_due(msg.next_retry_at) {
                            drained.push(msg);
                        } else {
                            remaining.push_back(msg);
                        }
                    }
                    *total -= drained.len();
                    *queue = remaining;
                    if queue.is_empty() {
                        queues.remove(queue_key);
                    }
                    drained
                } else {
                    Vec::new()
                }
            }
            OutboxBackend::Persistent(db) => {
                let now_ms = web_time::SystemTime::now()
                    .duration_since(web_time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let now_secs = now_ms / 1000;
                let is_due = |next_retry: Option<u64>| -> bool {
                    match next_retry {
                        None => true,
                        Some(val) => {
                            if val > 10_000_000_000 {
                                val <= now_ms
                            } else {
                                val <= now_secs
                            }
                        }
                    }
                };

                let prefix_str = format!("{}{}_", String::from_utf8_lossy(QUEUE_PREFIX), queue_key);
                let mut messages = Vec::new();
                let mut keys_to_remove = Vec::new();

                if let Ok(results) = db.scan_prefix(prefix_str.as_bytes()) {
                    for (key, value) in results {
                        if let Ok(msg) = deserialize_queued_message(&value) {
                            // Skip messages that are in custody or not Enqueued
                            if msg.in_custody || msg.state != MessageState::Enqueued {
                                continue;
                            }
                            if is_due(msg.next_retry_at) {
                                messages.push(msg);
                                keys_to_remove.push(key);
                            }
                        }
                    }
                }

                for key in keys_to_remove {
                    let _ = db.remove(&key);
                }
                let _ = db.flush();

                messages
            }
        }
    }

    /// Increment attempt count for a message.
    /// Returns true if the message should be removed (max attempts exceeded).
    /// Returns false if the message is in custody and should not be removed.
    pub fn record_attempt(&mut self, message_id: &str) -> bool {
        match &mut self.backend {
            OutboxBackend::Memory { queues, .. } => {
                for queue in queues.values_mut() {
                    if let Some(msg) = queue.iter_mut().find(|m| m.message_id == message_id) {
                        // If message is in custody, suppress local retries
                        if msg.in_custody {
                            return false;
                        }
                        msg.attempts = msg.attempts.saturating_add(1);
                        return msg.attempts >= MAX_DELIVERY_ATTEMPTS;
                    }
                }
            }
            OutboxBackend::Persistent(db) => {
                if let Ok(results) = db.scan_prefix(QUEUE_PREFIX) {
                    for (key, value) in results {
                        if let Ok(mut msg) = deserialize_queued_message(&value) {
                            if msg.message_id == message_id {
                                // If message is in custody, suppress local retries
                                if msg.in_custody {
                                    return false;
                                }
                                msg.attempts = msg.attempts.saturating_add(1);
                                let exceeded = msg.attempts >= MAX_DELIVERY_ATTEMPTS;
                                if let Ok(bytes) = bincode::serialize(&msg) {
                                    let _ = db.put(&key, &bytes);
                                    let _ = db.flush();
                                }
                                return exceeded;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Mark a message as being in relay custody
    pub fn mark_in_custody(&mut self, message_id: &str, custody_established_at: u64) -> bool {
        match &mut self.backend {
            OutboxBackend::Memory { queues, .. } => {
                for queue in queues.values_mut() {
                    if let Some(msg) = queue.iter_mut().find(|m| m.message_id == message_id) {
                        msg.in_custody = true;
                        msg.custody_established_at = custody_established_at;
                        return true;
                    }
                }
            }
            OutboxBackend::Persistent(db) => {
                if let Ok(results) = db.scan_prefix(QUEUE_PREFIX) {
                    for (key, value) in results {
                        if let Ok(mut msg) = deserialize_queued_message(&value) {
                            if msg.message_id == message_id {
                                msg.in_custody = true;
                                msg.custody_established_at = custody_established_at;
                                if let Ok(bytes) = bincode::serialize(&msg) {
                                    let _ = db.put(&key, &bytes);
                                    let _ = db.flush();
                                }
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Mark a message as no longer in relay custody
    pub fn mark_not_in_custody(&mut self, message_id: &str) -> bool {
        match &mut self.backend {
            OutboxBackend::Memory { queues, .. } => {
                for queue in queues.values_mut() {
                    if let Some(msg) = queue.iter_mut().find(|m| m.message_id == message_id) {
                        msg.in_custody = false;
                        msg.custody_established_at = 0;
                        return true;
                    }
                }
            }
            OutboxBackend::Persistent(db) => {
                if let Ok(results) = db.scan_prefix(QUEUE_PREFIX) {
                    for (key, value) in results {
                        if let Ok(mut msg) = deserialize_queued_message(&value) {
                            if msg.message_id == message_id {
                                msg.in_custody = false;
                                msg.custody_established_at = 0;
                                if let Ok(bytes) = bincode::serialize(&msg) {
                                    let _ = db.put(&key, &bytes);
                                    let _ = db.flush();
                                }
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if a message is in relay custody
    pub fn is_in_custody(&self, message_id: &str) -> bool {
        match &self.backend {
            OutboxBackend::Memory { queues, .. } => {
                for queue in queues.values() {
                    if let Some(msg) = queue.iter().find(|m| m.message_id == message_id) {
                        return msg.in_custody;
                    }
                }
                false
            }
            OutboxBackend::Persistent(db) => {
                if let Ok(results) = db.scan_prefix(QUEUE_PREFIX) {
                    for (_, value) in results {
                        if let Ok(msg) = deserialize_queued_message(&value) {
                            if msg.message_id == message_id {
                                return msg.in_custody;
                            }
                        }
                    }
                }
                false
            }
        }
    }

    /// Total queued messages
    pub fn total_count(&self) -> usize {
        match &self.backend {
            OutboxBackend::Memory { total, .. } => *total,
            OutboxBackend::Persistent(db) => db.count_prefix(QUEUE_PREFIX).unwrap_or(0),
        }
    }

    /// Number of peers with queued messages
    pub fn peer_count(&self) -> usize {
        match &self.backend {
            OutboxBackend::Memory { queues, .. } => queues.len(),
            OutboxBackend::Persistent(db) => {
                use std::collections::HashSet;
                let mut peers: HashSet<String> = HashSet::new();
                if let Ok(results) = db.scan_prefix(QUEUE_PREFIX) {
                    for (_, value) in results {
                        if let Ok(msg) = deserialize_queued_message(&value) {
                            peers.insert(msg.recipient_id);
                        }
                    }
                }
                peers.len()
            }
        }
    }

    /// Remove expired messages (older than max_age_secs)
    pub fn remove_expired(&mut self, max_age_secs: u64) -> usize {
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        match &mut self.backend {
            OutboxBackend::Memory {
                queues,
                total,
                receipt_authorizations,
            } => {
                let mut removed_ids = Vec::new();
                let mut removed = 0;

                for queue in queues.values_mut() {
                    let before = queue.len();
                    queue.retain(|msg| {
                        let keep = now.saturating_sub(msg.queued_at) < max_age_secs;
                        if !keep {
                            removed_ids.push(msg.message_id.clone());
                        }
                        keep
                    });
                    removed += before - queue.len();
                }

                *total -= removed;
                for message_id in removed_ids {
                    receipt_authorizations.remove(&message_id);
                }
                receipt_authorizations.retain(|_, authorization| authorization.expires_at >= now);

                // Clean up empty queues
                queues.retain(|_, q| !q.is_empty());

                removed
            }
            OutboxBackend::Persistent(db) => {
                let mut keys_to_remove = Vec::new();
                let mut message_ids_to_remove = Vec::new();

                if let Ok(results) = db.scan_prefix(QUEUE_PREFIX) {
                    for (key, value) in results {
                        if let Ok(msg) = deserialize_queued_message(&value) {
                            if now.saturating_sub(msg.queued_at) >= max_age_secs {
                                keys_to_remove.push(key);
                                message_ids_to_remove.push(msg.message_id);
                            }
                        }
                    }
                }

                let removed = keys_to_remove.len();
                for key in keys_to_remove {
                    let _ = db.remove(&key);
                }
                for message_id in message_ids_to_remove {
                    let _ = db.remove(&receipt_authorization_key(&message_id));
                }
                prune_expired_receipt_authorizations(db.as_ref(), now);
                let _ = db.flush();

                removed
            }
        }
    }

    /// Update message state to Sent
    pub fn mark_sent(&mut self, message_id: &str) -> bool {
        match &mut self.backend {
            OutboxBackend::Memory { queues, .. } => {
                for queue in queues.values_mut() {
                    if let Some(msg) = queue.iter_mut().find(|m| m.message_id == message_id) {
                        msg.state = MessageState::Sent;
                        return true;
                    }
                }
            }
            OutboxBackend::Persistent(db) => {
                if let Ok(results) = db.scan_prefix(QUEUE_PREFIX) {
                    for (key, value) in results {
                        if let Ok(mut msg) = deserialize_queued_message(&value) {
                            if msg.message_id == message_id {
                                msg.state = MessageState::Sent;
                                if let Ok(bytes) = bincode::serialize(&msg) {
                                    let _ = db.put(&key, &bytes);
                                    let _ = db.flush();
                                }
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Update message state to Failed
    pub fn mark_failed(&mut self, message_id: &str) -> bool {
        match &mut self.backend {
            OutboxBackend::Memory { queues, .. } => {
                for queue in queues.values_mut() {
                    if let Some(msg) = queue.iter_mut().find(|m| m.message_id == message_id) {
                        msg.state = MessageState::Failed;
                        return true;
                    }
                }
            }
            OutboxBackend::Persistent(db) => {
                if let Ok(results) = db.scan_prefix(QUEUE_PREFIX) {
                    for (key, value) in results {
                        if let Ok(mut msg) = deserialize_queued_message(&value) {
                            if msg.message_id == message_id {
                                msg.state = MessageState::Failed;
                                if let Ok(bytes) = bincode::serialize(&msg) {
                                    let _ = db.put(&key, &bytes);
                                    let _ = db.flush();
                                }
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
}

impl Default for Outbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Retry configuration for message delivery.
///
/// This is the ONLY place retry policy is defined. All platforms
/// (CLI, Android, iOS, WASM) use this struct. Changes to backoff
/// strategy apply everywhere automatically.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (including initial attempt).
    pub max_retries: u32,
    /// Initial delay in milliseconds before first retry.
    pub initial_delay_ms: u64,
    /// Backoff multiplier (2 = exponential, 1 = fixed).
    pub backoff_factor: u32,
    /// Whether to suppress retries when message is in relay custody
    #[serde(default = "default_true")]
    pub suppress_on_custody: bool,
}

fn default_true() -> bool {
    true
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,        // CLI baseline
            initial_delay_ms: 100, // CLI baseline
            backoff_factor: 2,     // exponential: 100ms, 200ms, 400ms
            suppress_on_custody: true,
        }
    }
}

impl RetryPolicy {
    /// Compute the delay before the given attempt (1-indexed).
    ///
    /// Returns None if attempt exceeds max_retries (delivery should be abandoned).
    pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        if attempt > self.max_retries {
            return None;
        }
        if attempt == 1 {
            // No delay for initial attempt
            return Some(Duration::from_millis(0));
        }
        // exponential: delay = initial * (backoff ^ (attempt - 2))
        // attempt 2: delay = initial * 1 = 100ms
        // attempt 3: delay = initial * 2 = 200ms
        // attempt 4: delay = initial * 4 = 400ms
        let power = attempt - 2;
        let multiplier = (self.backoff_factor as u64).saturating_pow(power);
        let delay_ms = self.initial_delay_ms.saturating_mul(multiplier);
        Some(Duration::from_millis(delay_ms))
    }

    /// Whether another retry is possible.
    pub fn can_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }

    /// Whether to suppress retries when message is in relay custody
    pub fn suppress_on_custody(&self) -> bool {
        self.suppress_on_custody
    }
}

#[cfg(test)]
mod retry_tests {
    use super::*;

    #[test]
    fn test_default_retry_delays() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.delay_for_attempt(1), Some(Duration::from_millis(0)));
        assert_eq!(
            policy.delay_for_attempt(2),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            policy.delay_for_attempt(3),
            Some(Duration::from_millis(200))
        );
        assert!(policy.delay_for_attempt(4).is_none()); // exceeds max_retries
    }

    #[test]
    fn test_can_retry() {
        let policy = RetryPolicy::default();
        assert!(policy.can_retry(1));
        assert!(policy.can_retry(2));
        assert!(!policy.can_retry(3)); // 3 is the max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(id: &str, recipient: &str) -> QueuedMessage {
        QueuedMessage {
            version: 1,
            message_id: id.to_string(),
            recipient_id: recipient.to_string(),
            envelope_data: vec![1, 2, 3],
            queued_at: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            attempts: 0,
            next_retry_at: None,
            in_custody: false,
            custody_established_at: 0,
            state: MessageState::Enqueued,
        }
    }

    #[test]
    fn fire_and_forget_version_roundtrips_through_persistent_backend() {
        let backend: Arc<dyn StorageBackend> =
            Arc::new(crate::store::backend::MemoryStorage::new());
        let mut outbox = Outbox::persistent(backend);
        let mut receipt = make_msg("rcpt-ff", "ab".repeat(32).as_str());
        receipt.version = QUEUED_VERSION_FIRE_AND_FORGET;
        outbox.enqueue(receipt).unwrap();
        outbox
            .enqueue(make_msg("plain", "ab".repeat(32).as_str()))
            .unwrap();
        let drained = outbox.flush_peer_messages(&"ab".repeat(32));
        assert_eq!(drained.len(), 2);
        let ff = drained.iter().find(|m| m.message_id == "rcpt-ff").unwrap();
        let plain = drained.iter().find(|m| m.message_id == "plain").unwrap();
        assert!(ff.is_fire_and_forget());
        assert!(!plain.is_fire_and_forget());
    }

    #[test]
    fn receipt_expiry_follows_retention_not_attempts() {
        let mut receipt = make_msg("rcpt", "peer_a");
        receipt.version = QUEUED_VERSION_FIRE_AND_FORGET;
        receipt.queued_at = 1_000;
        receipt.attempts = u32::MAX;
        assert!(!receipt.receipt_expired(1_000 + MESSAGE_RETENTION_SECS - 1));
        assert!(receipt.receipt_expired(1_000 + MESSAGE_RETENTION_SECS));
        let mut plain = make_msg("plain", "peer_a");
        plain.queued_at = 1_000;
        assert!(!plain.receipt_expired(u64::MAX));
    }

    #[test]
    fn test_flush_peer_messages() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg3", "peer_b")).unwrap();

        let flushed = outbox.flush_peer_messages("peer_a");
        assert_eq!(flushed.len(), 2);
        assert_eq!(outbox.total_count(), 1);
        assert_eq!(outbox.peek_for_peer("peer_a").len(), 0);
    }

    // --- peer-key spelling (regression) --------------------------------------
    //
    // Real ids from the 3-node rig, so the fixtures are not invented: BASE58_PEER
    // is a node's libp2p PeerId as it appears in its log, HEX_PEER is the
    // Ed25519 public-key hex the transport passes as `pk_hex`, and
    // OTHER_HEX_PEER is the cloud node's key hex -- a different node, which the
    // phone's outbox saw reconnects for while its own queue was keyed to the
    // Windows node.
    const BASE58_PEER: &str = "12D3KooWD776DQdWh6iHV8Qcnpj9jvTXhpJAgSsFPbMCaRtQpFmn";
    const HEX_PEER: &str = "30dce2bb779b4f1419f6d7d9e91b3ae201aed9e3b181aef674a9496f340a0645";
    const OTHER_HEX_PEER: &str = "69805e175cdc59b244f36001303e0b2e735f729557d2069a02688c0e69764a7c";

    #[test]
    fn canonical_peer_key_bridges_base58_and_hex_for_the_same_key() {
        assert_eq!(canonical_peer_key(BASE58_PEER), HEX_PEER);
        assert_eq!(canonical_peer_key(HEX_PEER), HEX_PEER);
        assert_eq!(canonical_peer_key(&HEX_PEER.to_uppercase()), HEX_PEER);
        // Two different keys stay different: this is not an identity guess.
        assert_ne!(
            canonical_peer_key(OTHER_HEX_PEER),
            canonical_peer_key(HEX_PEER)
        );
        // A value whose key is not recoverable passes through untouched.
        assert_eq!(canonical_peer_key("peer_a"), "peer_a");
    }

    #[test]
    fn flush_drains_a_queue_keyed_under_the_other_spelling() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", BASE58_PEER)).unwrap();
        assert_eq!(outbox.total_count(), 1);

        // The reconnect arrives as public-key hex.
        let flushed = outbox.flush_peer_messages(HEX_PEER);
        assert_eq!(flushed.len(), 1, "queued under base58, drained as hex");
        assert_eq!(flushed[0].message_id, "msg1");
        assert_eq!(outbox.total_count(), 0);
    }

    #[test]
    fn legacy_queue_keyed_before_canonicalization_still_drains() {
        // Simulate a queue persisted by an older build, which wrote the raw key.
        let mut outbox = Outbox::new();
        if let OutboxBackend::Memory { queues, total, .. } = &mut outbox.backend {
            queues.insert(
                BASE58_PEER.to_string(),
                VecDeque::from(vec![make_msg("legacy", BASE58_PEER)]),
            );
            *total += 1;
        } else {
            panic!("unit tests run on the memory backend");
        }

        let flushed = outbox.flush_peer_messages(HEX_PEER);
        assert_eq!(
            flushed.len(),
            1,
            "the pre-canonicalization entry must still go out"
        );
    }

    /// #395: a queue split across the canonical key and a pre-canonicalization
    /// spelling is fully emptied by a SINGLE flush -- no second pass needed.
    #[test]
    fn flush_drains_all_spellings_in_one_pass() {
        let mut outbox = Outbox::new();
        // One message under the canonical spelling (where `enqueue` writes).
        outbox.enqueue(make_msg("msg-canonical", HEX_PEER)).unwrap();
        // One message stranded under the legacy base58 spelling.
        if let OutboxBackend::Memory { queues, total, .. } = &mut outbox.backend {
            queues.insert(
                BASE58_PEER.to_string(),
                VecDeque::from(vec![make_msg("msg-legacy", BASE58_PEER)]),
            );
            *total += 1;
        } else {
            panic!("unit tests run on the memory backend");
        }
        assert_eq!(outbox.total_count(), 2);

        let flushed = outbox.flush_peer_messages(HEX_PEER);
        assert_eq!(
            flushed.len(),
            2,
            "one flush must drain both spellings, got: {:?}",
            flushed.iter().map(|m| &m.message_id).collect::<Vec<_>>()
        );
        assert_eq!(outbox.total_count(), 0, "no spelling may retain messages");
    }

    /// #395: same one-pass guarantee for the batch `drain_for_peer` path.
    #[test]
    fn drain_for_peer_drains_all_spellings_in_one_pass() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg-canonical", HEX_PEER)).unwrap();
        if let OutboxBackend::Memory { queues, total, .. } = &mut outbox.backend {
            queues.insert(
                BASE58_PEER.to_string(),
                VecDeque::from(vec![make_msg("msg-legacy", BASE58_PEER)]),
            );
            *total += 1;
        } else {
            panic!("unit tests run on the memory backend");
        }

        let drained = outbox.drain_for_peer(HEX_PEER);
        assert_eq!(
            drained.len(),
            2,
            "one drain must empty both spellings, got: {:?}",
            drained.iter().map(|m| &m.message_id).collect::<Vec<_>>()
        );
        assert_eq!(outbox.total_count(), 0, "no spelling may retain messages");
    }

    #[test]
    fn flush_does_not_cross_two_different_peers() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", HEX_PEER)).unwrap();
        outbox.enqueue(make_msg("msg2", OTHER_HEX_PEER)).unwrap();

        let flushed = outbox.flush_peer_messages(OTHER_HEX_PEER);
        assert_eq!(flushed.len(), 1);
        assert_eq!(flushed[0].message_id, "msg2");
        assert_eq!(
            outbox.peek_for_peer(HEX_PEER).len(),
            1,
            "the other peer's queue must be untouched"
        );
    }

    #[test]
    fn test_enqueue_and_peek() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg3", "peer_b")).unwrap();

        assert_eq!(outbox.total_count(), 3);
        assert_eq!(outbox.peer_count(), 2);
        assert_eq!(outbox.peek_for_peer("peer_a").len(), 2);
        assert_eq!(outbox.peek_for_peer("peer_b").len(), 1);
        assert_eq!(outbox.peek_for_peer("peer_c").len(), 0);
    }

    #[test]
    fn test_remove() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();

        assert!(outbox.remove("msg1"));
        assert_eq!(outbox.total_count(), 1);
        assert!(!outbox.remove("msg1")); // Already removed
    }

    #[test]
    fn receipt_authorization_contract_across_backends() {
        let recipient = [7u8; 32];
        let other = [9u8; 32];
        let recipient_id = hex::encode(recipient);
        let backends = [
            Outbox::new(),
            Outbox::persistent(Arc::new(crate::store::backend::MemoryStorage::new())),
        ];

        for mut outbox in backends {
            outbox.enqueue(make_msg("queued", &recipient_id)).unwrap();
            for (sender, accepted) in [(other, false), (recipient, true)] {
                assert_eq!(outbox.remove_for_recipient_key("queued", &sender), accepted);
            }
            assert_eq!(outbox.total_count(), 0);

            outbox.enqueue(make_msg("acked", &recipient_id)).unwrap();
            assert!(outbox.remove("acked"), "transport ACK removes retry state");
            assert_eq!(outbox.total_count(), 0);
            assert!(outbox.contains_for_recipient_key("acked", &recipient));
            assert!(!outbox.remove_for_recipient_key("acked", &other));
            assert!(outbox.remove_for_recipient_key("acked", &recipient));
            assert!(!outbox.contains_for_recipient_key("acked", &recipient));
        }
    }

    #[test]
    fn legacy_authorization_survives_ack_and_restart() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("outbox").to_str().unwrap().to_string();
        let recipient = [7u8; 32];
        let recipient_id = hex::encode(recipient);
        {
            let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
            let queue_key = format!("outbox_{recipient_id}_restart");
            backend
                .put(
                    queue_key.as_bytes(),
                    &bincode::serialize(&LegacyQueuedMessage {
                        message_id: "restart".to_string(),
                        recipient_id: recipient_id.clone(),
                        envelope_data: vec![1, 2, 3],
                        queued_at: current_unix_secs(),
                        attempts: 0,
                        next_retry_at: None,
                        in_custody: false,
                        custody_established_at: 0,
                        state: MessageState::Enqueued,
                    })
                    .unwrap(),
                )
                .unwrap();
            let mut outbox = Outbox::persistent(backend);
            assert!(outbox.remove("restart"));
        }
        let mut outbox = Outbox::persistent(Arc::new(
            crate::store::backend::SledStorage::new(&path).unwrap(),
        ));
        assert!(outbox.contains_for_recipient_key("restart", &recipient));
        assert!(outbox.remove_for_recipient_key("restart", &recipient));
        assert!(!outbox.contains_for_recipient_key("restart", &recipient));
    }

    /// A row written by a build that predates `receipt_auth_*` carries no
    /// authorization record, so `remove_for_recipient_key` falls back to the
    /// queue row itself. That fallback is the one path that could have
    /// reintroduced the original bug -- an unauthenticated peer clearing retry
    /// state for a message it was never sent -- so it is pinned directly here
    /// rather than only via the ACK-migration path above.
    #[test]
    fn legacy_row_without_authorization_still_rejects_the_wrong_recipient() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("outbox").to_str().unwrap().to_string();
        let recipient = [7u8; 32];
        let recipient_id = hex::encode(recipient);
        let stranger = [9u8; 32];

        let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
        // Legacy key form: no `receipt_auth_*` sibling exists for this message.
        backend
            .put(
                format!("outbox_{recipient_id}_orphan").as_bytes(),
                &bincode::serialize(&LegacyQueuedMessage {
                    message_id: "orphan".to_string(),
                    recipient_id: recipient_id.clone(),
                    envelope_data: vec![1, 2, 3],
                    queued_at: current_unix_secs(),
                    attempts: 0,
                    next_retry_at: None,
                    in_custody: false,
                    custody_established_at: 0,
                    state: MessageState::Enqueued,
                })
                .unwrap(),
            )
            .unwrap();

        let mut outbox = Outbox::persistent(backend);

        // The stranger must be refused on both the read path and the clearing
        // path, and must leave the row intact for the real recipient.
        assert!(!outbox.contains_for_recipient_key("orphan", &stranger));
        assert!(!outbox.remove_for_recipient_key("orphan", &stranger));

        // The intended recipient is still served by the fallback.
        assert!(outbox.contains_for_recipient_key("orphan", &recipient));
        assert!(outbox.remove_for_recipient_key("orphan", &recipient));
        assert!(!outbox.contains_for_recipient_key("orphan", &recipient));
    }

    #[test]
    fn expired_authorization_is_rejected() {
        let recipient = [11u8; 32];
        let authorization = ReceiptAuthorization {
            recipient_public_key: recipient,
            expires_at: current_unix_secs().saturating_sub(1),
        };
        let backend = crate::store::backend::MemoryStorage::new();
        backend
            .put(
                &receipt_authorization_key("expired"),
                &bincode::serialize(&authorization).unwrap(),
            )
            .unwrap();
        let outbox = Outbox::persistent(Arc::new(backend));
        assert!(!outbox.contains_for_recipient_key("expired", &recipient));
    }

    #[test]
    fn test_drain_for_peer() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg3", "peer_b")).unwrap();

        let drained = outbox.drain_for_peer("peer_a");
        assert_eq!(drained.len(), 2);
        assert_eq!(outbox.total_count(), 1);
        assert_eq!(outbox.peek_for_peer("peer_a").len(), 0);
    }

    #[test]
    fn test_record_attempt() {
        let mut outbox = Outbox::new();
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();

        outbox.record_attempt("msg1");
        outbox.record_attempt("msg1");

        let msgs = outbox.peek_for_peer("peer_a");
        assert_eq!(msgs[0].attempts, 2);
    }

    #[test]
    fn test_remove_expired() {
        let mut outbox = Outbox::new();

        let mut old_msg = make_msg("old", "peer_a");
        old_msg.queued_at = 0; // epoch = very old
        outbox.enqueue(old_msg).unwrap();

        let fresh_msg = make_msg("fresh", "peer_a");
        outbox.enqueue(fresh_msg).unwrap();

        let removed = outbox.remove_expired(3600); // 1 hour max age
        assert_eq!(removed, 1);
        assert_eq!(outbox.total_count(), 1);
    }

    #[test]
    fn test_restore_drained_bypasses_capacity_for_recovery() {
        let mut outbox = Outbox::new();
        let drained = make_msg("drained", "peer_a");
        outbox.enqueue(drained.clone()).unwrap();
        assert_eq!(outbox.drain_for_peer("peer_a").len(), 1);

        for index in 0..MAX_QUEUE_PER_PEER {
            outbox
                .enqueue(make_msg(&format!("existing-{index}"), "peer_a"))
                .unwrap();
        }
        assert_eq!(outbox.total_count(), MAX_QUEUE_PER_PEER);

        outbox.restore_drained(drained).unwrap();
        assert_eq!(outbox.total_count(), MAX_QUEUE_PER_PEER + 1);
        assert!(outbox
            .peek_for_peer("peer_a")
            .iter()
            .any(|msg| msg.message_id == "drained"));
    }

    #[test]
    fn test_retry_now_clears_deferred_timer() {
        let mut outbox = Outbox::new();
        let mut msg = make_msg("retry-now", "peer_a");
        msg.next_retry_at = Some(u64::MAX);
        outbox.enqueue(msg).unwrap();

        assert!(outbox.retry_now("retry-now"));
        let restored = outbox.peek_for_peer("peer_a");
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].state, MessageState::Enqueued);
        assert_eq!(restored[0].next_retry_at, None);
    }

    #[test]
    fn test_persistent_outbox() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir
            .path()
            .join("outbox_store")
            .to_str()
            .unwrap()
            .to_string();

        let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
        let mut outbox = Outbox::persistent(backend);

        // Enqueue messages
        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg3", "peer_b")).unwrap();

        assert_eq!(outbox.total_count(), 3);
        assert_eq!(outbox.peer_count(), 2);

        // Peek messages
        let peer_a_msgs = outbox.peek_for_peer("peer_a");
        assert_eq!(peer_a_msgs.len(), 2);

        // Remove a message
        assert!(outbox.remove("msg1"));
        assert_eq!(outbox.total_count(), 2);
    }

    #[test]
    fn test_persistent_outbox_survives_restart() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir
            .path()
            .join("outbox_store")
            .to_str()
            .unwrap()
            .to_string();

        // First instance: enqueue messages
        {
            let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
            let mut outbox = Outbox::persistent(backend);
            outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
            outbox.enqueue(make_msg("msg2", "peer_b")).unwrap();
        }

        // Second instance: messages should still be there
        {
            let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
            let outbox = Outbox::persistent(backend);
            assert_eq!(outbox.total_count(), 2);
            assert_eq!(outbox.peer_count(), 2);

            let peer_a_msgs = outbox.peek_for_peer("peer_a");
            assert_eq!(peer_a_msgs.len(), 1);
            assert_eq!(peer_a_msgs[0].message_id, "msg1");
        }
    }

    #[test]
    fn test_persistent_outbox_drain() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir
            .path()
            .join("outbox_store")
            .to_str()
            .unwrap()
            .to_string();

        let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
        let mut outbox = Outbox::persistent(backend);

        outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg2", "peer_a")).unwrap();
        outbox.enqueue(make_msg("msg3", "peer_b")).unwrap();

        let drained = outbox.drain_for_peer("peer_a");
        assert_eq!(drained.len(), 2);
        assert_eq!(outbox.total_count(), 1);
        assert_eq!(outbox.peek_for_peer("peer_a").len(), 0);
    }

    #[test]
    fn test_persistent_attempts_survive_restart() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir
            .path()
            .join("outbox_store")
            .to_str()
            .unwrap()
            .to_string();

        {
            let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
            let mut outbox = Outbox::persistent(backend);
            outbox.enqueue(make_msg("msg1", "peer_a")).unwrap();
            outbox.record_attempt("msg1");
            outbox.record_attempt("msg1");
            // Explicit drop (rather than relying on end-of-scope order) makes
            // the close-then-reopen race this test exercises obvious: the
            // second block below reopens the same sled path, which can race
            // the OS releasing the file lock this `Outbox`'s `SledStorage`
            // holds. `SledStorage::new`'s bounded lock-contention retry is
            // what actually makes that reopen deterministic; this drop just
            // documents which handle is being released.
            drop(outbox);
        }

        {
            let backend = Arc::new(crate::store::backend::SledStorage::new(&path).unwrap());
            let outbox = Outbox::persistent(backend);
            let msgs = outbox.peek_for_peer("peer_a");
            assert_eq!(msgs.len(), 1);
            assert_eq!(msgs[0].attempts, 2);
        }
    }

    #[test]
    fn test_record_attempt_never_drops_message() {
        let mut outbox = Outbox::new();
        let mut msg = make_msg("msg1", "peer_a");
        msg.attempts = u32::MAX - 1;
        outbox.enqueue(msg).unwrap();

        outbox.record_attempt("msg1");
        outbox.record_attempt("msg1");
        outbox.record_attempt("msg1");

        let msgs = outbox.peek_for_peer("peer_a");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].attempts, u32::MAX);
    }
}
