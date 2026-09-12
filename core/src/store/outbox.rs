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
    // If the first byte is 1, it's the new versioned format.
    // Legacy format starts with a String length. Since string lengths are usually 36 (for UUIDs),
    // their first byte is 36, not 1.
    if data[0] == 1 {
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

/// Maximum messages queued per peer
const MAX_QUEUE_PER_PEER: usize = 1000;

/// Maximum total messages across all peers
const MAX_TOTAL_QUEUED: usize = 10_000;

/// Maximum delivery attempts before automatic removal
const MAX_DELIVERY_ATTEMPTS: u32 = 12;

const QUEUE_PREFIX: &[u8] = b"outbox_";

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

        match &mut self.backend {
            OutboxBackend::Memory { queues, total } => {
                if *total >= MAX_TOTAL_QUEUED {
                    return Err(format!("Outbox full ({} messages)", MAX_TOTAL_QUEUED));
                }

                let queue = queues.entry(msg.recipient_id.clone()).or_default();

                if queue.len() >= MAX_QUEUE_PER_PEER {
                    return Err(format!(
                        "Queue full for peer {} ({} messages)",
                        msg.recipient_id, MAX_QUEUE_PER_PEER
                    ));
                }

                queue.push_back(msg);
                *total += 1;
                // Trigger maintenance on memory outbox
                // Note: This is a best-effort call and any errors are silently ignored
                self.trigger_maintenance();
                Ok(())
            }
            OutboxBackend::Persistent(db) => {
                // Check total limit
                let current_total = db.count_prefix(QUEUE_PREFIX).unwrap_or(0);
                if current_total >= MAX_TOTAL_QUEUED {
                    return Err(format!("Outbox full ({} messages)", MAX_TOTAL_QUEUED));
                }

                // Check per-peer limit
                let peer_prefix = format!(
                    "{}{}_",
                    String::from_utf8_lossy(QUEUE_PREFIX),
                    msg.recipient_id
                );
                let peer_count = db.count_prefix(peer_prefix.as_bytes()).unwrap_or(0);
                if peer_count >= MAX_QUEUE_PER_PEER {
                    return Err(format!(
                        "Queue full for peer {} ({} messages)",
                        msg.recipient_id, MAX_QUEUE_PER_PEER
                    ));
                }

                // Store message
                let key_str = format!(
                    "{}{}_{}",
                    String::from_utf8_lossy(QUEUE_PREFIX),
                    msg.recipient_id,
                    msg.message_id
                );
                if let Ok(bytes) = bincode::serialize(&msg) {
                    db.put(key_str.as_bytes(), &bytes)?;
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

    /// Remove a specific message by ID (after successful delivery)
    pub fn remove(&mut self, message_id: &str) -> bool {
        let result = match &mut self.backend {
            OutboxBackend::Memory { queues, total } => {
                for queue in queues.values_mut() {
                    if let Some(pos) = queue.iter().position(|m| m.message_id == message_id) {
                        queue.remove(pos);
                        *total -= 1;
                        return true;
                    }
                }
                false
            }
            OutboxBackend::Persistent(db) => {
                // Find and remove the message
                if let Ok(results) = db.scan_prefix(QUEUE_PREFIX) {
                    for (key, value) in results {
                        if let Ok(msg) = deserialize_queued_message(&value) {
                            if msg.message_id == message_id {
                                let _ = db.remove(&key);
                                let _ = db.flush();
                                return true;
                            }
                        }
                    }
                }
                false
            }
        };

        if result {
            tracing::info!(
                event = "outbox_dequeue",
                message_id = %message_id,
                reason = "delivery_confirmed"
            );
        }

        result
    }

    /// Drain all messages for a peer (for batch delivery)
    pub fn drain_for_peer(&mut self, recipient_id: &str) -> Vec<QueuedMessage> {
        match &mut self.backend {
            OutboxBackend::Memory { queues, total } => {
                let mut drained = Vec::new();
                if let Some(queue) = queues.remove(recipient_id) {
                    let count = queue.len();
                    *total -= count;
                    // Filter out messages that are in custody and should not be delivered locally
                    for msg in queue.into_iter() {
                        if !msg.in_custody {
                            drained.push(msg);
                        }
                    }
                }
                drained
            }
            OutboxBackend::Persistent(db) => {
                let prefix_str =
                    format!("{}{}_", String::from_utf8_lossy(QUEUE_PREFIX), recipient_id);
                let mut messages = Vec::new();
                let mut keys_to_remove = Vec::new();

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
        match &mut self.backend {
            OutboxBackend::Memory { queues, total } => {
                let queue = queues.entry(msg.recipient_id.clone()).or_default();
                if let Some(existing) = queue.iter_mut().find(|m| m.message_id == message_id) {
                    *existing = msg;
                } else {
                    queue.push_back(msg);
                    *total += 1;
                }
                Ok(())
            }
            OutboxBackend::Persistent(db) => {
                let key_str = format!(
                    "{}{}_{}",
                    String::from_utf8_lossy(QUEUE_PREFIX),
                    msg.recipient_id,
                    msg.message_id
                );
                let bytes = bincode::serialize(&msg).map_err(|e| e.to_string())?;
                db.put(key_str.as_bytes(), &bytes)?;
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

    /// Flush peer messages that are due for delivery
    pub fn flush_peer_messages(&mut self, recipient_id: &str) -> Vec<QueuedMessage> {
        match &mut self.backend {
            OutboxBackend::Memory { queues, total } => {
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

                if let Some(queue) = queues.get_mut(recipient_id) {
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
                        queues.remove(recipient_id);
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

                let prefix_str =
                    format!("{}{}_", String::from_utf8_lossy(QUEUE_PREFIX), recipient_id);
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
            OutboxBackend::Memory { queues, total } => {
                let mut removed = 0;

                for queue in queues.values_mut() {
                    let before = queue.len();
                    queue.retain(|msg| now.saturating_sub(msg.queued_at) < max_age_secs);
                    removed += before - queue.len();
                }

                *total -= removed;

                // Clean up empty queues
                queues.retain(|_, q| !q.is_empty());

                removed
            }
            OutboxBackend::Persistent(db) => {
                let mut keys_to_remove = Vec::new();

                if let Ok(results) = db.scan_prefix(QUEUE_PREFIX) {
                    for (key, value) in results {
                        if let Ok(msg) = deserialize_queued_message(&value) {
                            if now.saturating_sub(msg.queued_at) >= max_age_secs {
                                keys_to_remove.push(key);
                            }
                        }
                    }
                }

                let removed = keys_to_remove.len();
                for key in keys_to_remove {
                    let _ = db.remove(&key);
                }
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
