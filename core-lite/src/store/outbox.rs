// Outbox — queue messages for peers that may be offline

use crate::store::backend::StorageBackend;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

const MAX_QUEUE_PER_PEER: usize = 1000;
const MAX_TOTAL_QUEUED: usize = 10_000;
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
}

impl Outbox {
    /// Create a new in-memory outbox
    pub fn new() -> Self {
        Self {
            backend: OutboxBackend::Memory {
                queues: HashMap::new(),
                total: 0,
            },
        }
    }

    /// Create a new persistent outbox
    pub fn persistent(backend: Arc<dyn StorageBackend>) -> Self {
        Self {
            backend: OutboxBackend::Persistent(backend),
        }
    }

    /// Enqueue a message for delivery
    pub fn enqueue(&mut self, msg: QueuedMessage) -> Result<(), String> {
        match &mut self.backend {
            OutboxBackend::Memory { queues, total } => {
                if *total >= MAX_TOTAL_QUEUED {
                    return Err("Outbox full".to_string());
                }
                let queue = queues.entry(msg.recipient_id.clone()).or_default();
                if queue.len() >= MAX_QUEUE_PER_PEER {
                    return Err("Per-peer queue full".to_string());
                }
                queue.push_back(msg);
                *total += 1;
                Ok(())
            }
            OutboxBackend::Persistent(db) => {
                let key = format!("{}{}", String::from_utf8_lossy(QUEUE_PREFIX), msg.message_id);
                let data = bincode::serialize(&msg).map_err(|e| e.to_string())?;
                db.put(key.as_bytes(), &data)?;
                Ok(())
            }
        }
    }

    /// Remove a message from the queue
    pub fn remove(&mut self, message_id: &str) -> bool {
        match &mut self.backend {
            OutboxBackend::Memory { queues, total } => {
                for queue in queues.values_mut() {
                    if let Some(pos) = queue.iter().position(|m| m.message_id == message_id) {
                        queue.remove(pos);
                        *total = total.saturating_sub(1);
                        return true;
                    }
                }
                false
            }
            OutboxBackend::Persistent(db) => {
                let key = format!("{}{}", String::from_utf8_lossy(QUEUE_PREFIX), message_id);
                db.remove(key.as_bytes()).is_ok()
            }
        }
    }

    /// Get a message by ID
    pub fn get(&self, message_id: &str) -> Option<QueuedMessage> {
        match &self.backend {
            OutboxBackend::Memory { queues, .. } => {
                for queue in queues.values() {
                    for msg in queue {
                        if msg.message_id == message_id {
                            return Some(msg.clone());
                        }
                    }
                }
                None
            }
            OutboxBackend::Persistent(db) => {
                let key = format!("{}{}", String::from_utf8_lossy(QUEUE_PREFIX), message_id);
                db.get(key.as_bytes())
                    .ok()
                    .flatten()
                    .and_then(|data| bincode::deserialize(&data).ok())
            }
        }
    }

    /// Get all messages for a peer
    pub fn get_peer_messages(&self, recipient_id: &str) -> Vec<QueuedMessage> {
        match &self.backend {
            OutboxBackend::Memory { queues, .. } => {
                queues
                    .get(recipient_id)
                    .map(|q| q.iter().cloned().collect())
                    .unwrap_or_default()
            }
            OutboxBackend::Persistent(db) => {
                let prefix = format!("{}{}", String::from_utf8_lossy(QUEUE_PREFIX), recipient_id);
                db.scan_prefix(prefix.as_bytes())
                    .ok()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|(_, v)| bincode::deserialize(&v).ok())
                    .collect()
            }
        }
    }

    /// Get all queued messages
    pub fn all_messages(&self) -> Vec<QueuedMessage> {
        match &self.backend {
            OutboxBackend::Memory { queues, .. } => {
                queues.values().flat_map(|q| q.iter().cloned()).collect()
            }
            OutboxBackend::Persistent(db) => {
                db.scan_prefix(QUEUE_PREFIX)
                    .ok()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|(_, v)| bincode::deserialize(&v).ok())
                    .collect()
            }
        }
    }

    /// Mark message as sent
    pub fn mark_sent(&mut self, message_id: &str) -> bool {
        if let Some(mut msg) = self.get(message_id) {
            msg.state = MessageState::Sent;
            self.remove(message_id);
            self.enqueue(msg).is_ok()
        } else {
            false
        }
    }

    /// Increment attempt count and schedule next retry
    pub fn record_attempt(&mut self, message_id: &str, next_retry_at: u64) -> bool {
        if let Some(mut msg) = self.get(message_id) {
            msg.attempts += 1;
            msg.next_retry_at = Some(next_retry_at);
            if msg.attempts >= MAX_DELIVERY_ATTEMPTS {
                msg.state = MessageState::Failed;
            }
            self.remove(message_id);
            self.enqueue(msg).is_ok()
        } else {
            false
        }
    }

    /// Get messages ready for retry
    pub fn get_retryable(&self, now: u64) -> Vec<QueuedMessage> {
        self.all_messages()
            .into_iter()
            .filter(|m| {
                m.state == MessageState::Enqueued
                    && m.attempts < MAX_DELIVERY_ATTEMPTS
                    && m.next_retry_at.map_or(true, |t| t <= now)
            })
            .collect()
    }

    /// Get total queued count
    pub fn len(&self) -> usize {
        match &self.backend {
            OutboxBackend::Memory { total, .. } => *total,
            OutboxBackend::Persistent(db) => {
                db.count_prefix(QUEUE_PREFIX).unwrap_or(0)
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for Outbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_outbox() {
        let mut outbox = Outbox::new();
        let msg = QueuedMessage {
            version: 1,
            message_id: "msg-1".to_string(),
            recipient_id: "peer-1".to_string(),
            envelope_data: vec![1, 2, 3],
            queued_at: 1000,
            attempts: 0,
            next_retry_at: None,
            in_custody: false,
            custody_established_at: 0,
            state: MessageState::Enqueued,
        };
        outbox.enqueue(msg.clone()).unwrap();
        assert_eq!(outbox.len(), 1);
        assert!(outbox.get("msg-1").is_some());
        outbox.remove("msg-1");
        assert_eq!(outbox.len(), 0);
    }
}