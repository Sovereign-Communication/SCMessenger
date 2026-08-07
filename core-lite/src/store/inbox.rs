// Inbox — receive and deduplicate incoming messages

use crate::store::backend::StorageBackend;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

const MAX_SEEN_IDS: usize = 50_000;
const SEEN_IDS_KEY: &[u8] = b"inbox_seen_ids";
const MESSAGES_PREFIX: &[u8] = b"inbox_msg_";

/// A received message record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivedMessage {
    /// Version byte for serialization format
    #[serde(default = "default_version")]
    pub version: u8,
    /// Message ID
    pub message_id: String,
    /// Sender's identity ID
    pub sender_id: String,
    /// Decrypted payload bytes
    pub payload: Vec<u8>,
    /// When this was received (unix timestamp)
    pub received_at: u64,
    /// Sender's Ed25519 public key (hex-encoded)
    #[serde(default)]
    pub sender_public_key_hex: Option<String>,
}

fn default_version() -> u8 {
    1
}

/// Storage backend for inbox
enum InboxBackend {
    Memory {
        seen_ids: FxHashSet<[u8; 32]>,
        seen_order: Vec<[u8; 32]>,
        messages: HashMap<String, Vec<ReceivedMessage>>,
        total: usize,
    },
    Persistent(Arc<dyn StorageBackend>),
}

/// Inbound message deduplication and storage
pub struct Inbox {
    backend: InboxBackend,
}

impl Inbox {
    /// Create a new in-memory inbox
    pub fn new() -> Self {
        Self {
            backend: InboxBackend::Memory {
                seen_ids: FxHashSet::default(),
                seen_order: Vec::new(),
                messages: HashMap::new(),
                total: 0,
            },
        }
    }

    /// Create a persistent inbox
    pub fn persistent(backend: Arc<dyn StorageBackend>) -> Self {
        Self {
            backend: InboxBackend::Persistent(backend),
        }
    }

    /// Check if a message ID has already been seen (duplicate)
    pub fn is_duplicate(&self, message_id: &str) -> bool {
        let hash = *blake3::hash(message_id.as_bytes()).as_bytes();
        match &self.backend {
            InboxBackend::Memory { seen_ids, .. } => seen_ids.contains(&hash),
            InboxBackend::Persistent(db) => {
                if let Ok(Some(bytes)) = db.get(SEEN_IDS_KEY) {
                    if let Ok(seen_ids) = bincode::deserialize::<FxHashSet<[u8; 32]>>(&bytes) {
                        return seen_ids.contains(&hash);
                    }
                }
                false
            }
        }
    }

    /// Mark a message ID as seen
    pub fn mark_seen(&mut self, message_id: &str) -> Result<(), String> {
        let hash = *blake3::hash(message_id.as_bytes()).as_bytes();
        match &mut self.backend {
            InboxBackend::Memory { seen_ids, seen_order, .. } => {
                if seen_ids.len() >= MAX_SEEN_IDS {
                    // Remove oldest
                    if let Some(oldest) = seen_order.first() {
                        seen_ids.remove(oldest);
                    }
                    seen_order.remove(0);
                }
                seen_ids.insert(hash);
                seen_order.push(hash);
                Ok(())
            }
            InboxBackend::Persistent(db) => {
                let mut seen_ids: FxHashSet<[u8; 32]> = db
                    .get(SEEN_IDS_KEY)?
                    .and_then(|b| bincode::deserialize(&b).ok())
                    .unwrap_or_default();
                seen_ids.insert(hash);
                let data = bincode::serialize(&seen_ids).map_err(|e| e.to_string())?;
                db.put(SEEN_IDS_KEY, &data)?;
                Ok(())
            }
        }
    }

    /// Store a received message
    pub fn store(&mut self, msg: ReceivedMessage) -> Result<(), String> {
        let sender_id = msg.sender_id.clone();
        match &mut self.backend {
            InboxBackend::Memory { messages, total, .. } => {
                messages.entry(sender_id).or_default().push(msg);
                *total += 1;
                Ok(())
            }
            InboxBackend::Persistent(db) => {
                let key = format!(
                    "{}{}",
                    String::from_utf8_lossy(MESSAGES_PREFIX),
                    msg.message_id
                );
                let data = bincode::serialize(&msg).map_err(|e| e.to_string())?;
                db.put(key.as_bytes(), &data)?;
                Ok(())
            }
        }
    }

    /// Get messages for a peer
    pub fn get_messages(&self, sender_id: &str) -> Vec<ReceivedMessage> {
        match &self.backend {
            InboxBackend::Memory { messages, .. } => {
                messages.get(sender_id).cloned().unwrap_or_default()
            }
            InboxBackend::Persistent(db) => {
                let prefix = format!("{}{}", String::from_utf8_lossy(MESSAGES_PREFIX), sender_id);
                db.scan_prefix(prefix.as_bytes())
                    .ok()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|(_, v)| bincode::deserialize(&v).ok())
                    .collect()
            }
        }
    }

    /// Get all messages
    pub fn all_messages(&self) -> Vec<ReceivedMessage> {
        match &self.backend {
            InboxBackend::Memory { messages, .. } => {
                messages.values().flat_map(|v| v.iter().cloned()).collect()
            }
            InboxBackend::Persistent(db) => {
                db.scan_prefix(MESSAGES_PREFIX)
                    .ok()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|(_, v)| bincode::deserialize(&v).ok())
                    .collect()
            }
        }
    }

    /// Remove messages for a peer
    pub fn remove_peer(&mut self, sender_id: &str) -> Result<(), String> {
        match &mut self.backend {
            InboxBackend::Memory { messages, total, .. } => {
                if let Some(msgs) = messages.remove(sender_id) {
                    *total = total.saturating_sub(msgs.len());
                }
                Ok(())
            }
            InboxBackend::Persistent(db) => {
                let prefix = format!("{}{}", String::from_utf8_lossy(MESSAGES_PREFIX), sender_id);
                for (key, _) in db.scan_prefix(prefix.as_bytes())? {
                    db.remove(&key)?;
                }
                Ok(())
            }
        }
    }

    /// Clear all messages
    pub fn clear(&mut self) -> Result<(), String> {
        match &mut self.backend {
            InboxBackend::Memory { messages, total, .. } => {
                messages.clear();
                *total = 0;
                Ok(())
            }
            InboxBackend::Persistent(db) => {
                for (key, _) in db.scan_prefix(MESSAGES_PREFIX)? {
                    db.remove(&key)?;
                }
                db.remove(SEEN_IDS_KEY)?;
                Ok(())
            }
        }
    }

    /// Get total message count
    pub fn len(&self) -> usize {
        match &self.backend {
            InboxBackend::Memory { total, .. } => *total,
            InboxBackend::Persistent(db) => db.count_prefix(MESSAGES_PREFIX).unwrap_or(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for Inbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_inbox() {
        let mut inbox = Inbox::new();
        let msg = ReceivedMessage {
            version: 1,
            message_id: "msg-1".to_string(),
            sender_id: "peer-1".to_string(),
            payload: b"hello".to_vec(),
            received_at: 1000,
            sender_public_key_hex: Some("abc".to_string()),
        };
        assert!(!inbox.is_duplicate("msg-1"));
        inbox.mark_seen("msg-1").unwrap();
        assert!(inbox.is_duplicate("msg-1"));
        inbox.store(msg.clone()).unwrap();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox.get_messages("peer-1").len(), 1);
    }
}