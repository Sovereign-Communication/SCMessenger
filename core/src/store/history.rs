// Message history persistence and retrieval
//
// Refactored to use generic StorageBackend for cross-platform parity (Sled/IndexedDB/Memory).

use crate::store::backend::StorageBackend;
use crate::IronCoreError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageDirection {
    Sent,
    Received,
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
    /// When `true` the message is from a blocked-only peer and is retained for
    /// evidentiary purposes but must be filtered out of all UI-facing queries.
    /// The flag is cleared when the peer is unblocked.
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

impl MessageRecord {
    pub fn new_sent(peer_id: String, content: String) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let ts = current_timestamp();
        Self {
            id,
            direction: MessageDirection::Sent,
            peer_id,
            content,
            timestamp: ts,
            sender_timestamp: ts,
            delivered: false,
            hidden: false,
        }
    }

    pub fn new_received(peer_id: String, content: String, sender_timestamp: u64) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            direction: MessageDirection::Received,
            peer_id,
            content,
            timestamp: current_timestamp(),
            sender_timestamp,
            delivered: true,
            hidden: false,
        }
    }
}

fn current_timestamp() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone, Default)]
pub struct HistoryStats {
    pub total_messages: u32,
    pub sent_count: u32,
    pub received_count: u32,
    pub undelivered_count: u32,
}

#[derive(Clone)]
pub struct HistoryManager {
    backend: Arc<dyn StorageBackend>,
}

/// D4 coalescing: history records are stored under the canonical identity_id
/// (blake3 of the sender public key) while contacts and thread lookups are
/// often keyed by the public_key_hex flavor on a different surface (e.g. a
/// contact's stored `peer_id` is its public key). Match a record's peer
/// against EITHER the queried value OR, when the query value is a public key,
/// that key's derived identity_id, so a single identity's conversation is
/// reachable from both forms instead of being silently split into two threads.
/// `identity_id_from_public_key_hex` returns `None` for non-64-hex inputs
/// (libp2p ids, already-identity_id values that are not valid curve points), in
/// which case only the exact-match arm applies.
fn history_peer_matches(filter: &str, record_peer: &str) -> bool {
    if record_peer.eq_ignore_ascii_case(filter) {
        return true;
    }
    if let Some(identity_id) = crate::identity::keys::identity_id_from_public_key_hex(filter) {
        if record_peer.eq_ignore_ascii_case(&identity_id) {
            return true;
        }
    }
    false
}

impl HistoryManager {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    /// P0_SECURITY_005: Expose the storage backend for audit log persistence.
    pub fn backend(&self) -> Arc<dyn StorageBackend> {
        self.backend.clone()
    }

    pub fn add(&self, record: MessageRecord) -> Result<(), IronCoreError> {
        let key = format!("msg_{}", record.id);
        let value = serde_json::to_vec(&record).map_err(|_| IronCoreError::Internal)?;
        self.backend
            .put(key.as_bytes(), &value)
            .map_err(|_| IronCoreError::StorageError)?;
        Ok(())
    }

    pub fn get(&self, id: String) -> Result<Option<MessageRecord>, IronCoreError> {
        let key = format!("msg_{}", id);
        if let Some(data) = self
            .backend
            .get(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?
        {
            let record: MessageRecord =
                serde_json::from_slice(&data).map_err(|_| IronCoreError::Internal)?;
            Ok(Some(record.adjust_legacy_timestamps()))
        } else {
            Ok(None)
        }
    }

    pub fn recent(
        &self,
        peer_filter: Option<String>,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, IronCoreError> {
        self.recent_internal(peer_filter, limit, false)
    }

    /// Like `recent()` but also returns messages that are hidden due to the
    /// sender being blocked.  Used by administrative / evidentiary access paths.
    pub fn recent_including_hidden(
        &self,
        peer_filter: Option<String>,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, IronCoreError> {
        self.recent_internal(peer_filter, limit, true)
    }

    fn recent_internal(
        &self,
        peer_filter: Option<String>,
        limit: u32,
        include_hidden: bool,
    ) -> Result<Vec<MessageRecord>, IronCoreError> {
        let mut records = Vec::new();
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        for (_, value) in all {
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            let record = record.adjust_legacy_timestamps();

            // Evidentiary retention: skip hidden messages in normal queries.
            if record.hidden && !include_hidden {
                continue;
            }

            if let Some(ref peer) = peer_filter {
                if history_peer_matches(peer, &record.peer_id) {
                    records.push(record);
                }
            } else {
                records.push(record);
            }
        }

        // Sort by timestamp descending
        records.sort_by_key(|b| std::cmp::Reverse(b.timestamp));

        if records.len() > limit as usize {
            records.truncate(limit as usize);
        }

        Ok(records)
    }

    pub fn conversation(
        &self,
        peer_id: String,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, IronCoreError> {
        self.recent(Some(peer_id), limit)
    }

    /// Unhide all stored messages for a given peer (called on unblock).
    pub fn unhide_messages_for_peer(&self, peer_id: &str) -> Result<u32, IronCoreError> {
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        let mut count = 0u32;
        for (_, value) in all {
            let mut record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            if record.hidden && record.peer_id.eq_ignore_ascii_case(peer_id) {
                record.hidden = false;
                let key = format!("msg_{}", record.id);
                let updated = serde_json::to_vec(&record).map_err(|_| IronCoreError::Internal)?;
                self.backend
                    .put(key.as_bytes(), &updated)
                    .map_err(|_| IronCoreError::StorageError)?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Hide all stored messages for a given peer (called on block).
    pub fn hide_messages_for_peer(&self, peer_id: &str) -> Result<u32, IronCoreError> {
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        let mut count = 0u32;
        for (_, value) in all {
            let mut record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            if !record.hidden && record.peer_id.eq_ignore_ascii_case(peer_id) {
                record.hidden = true;
                let key = format!("msg_{}", record.id);
                let updated = serde_json::to_vec(&record).map_err(|_| IronCoreError::Internal)?;
                self.backend
                    .put(key.as_bytes(), &updated)
                    .map_err(|_| IronCoreError::StorageError)?;
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn search(&self, query: String, limit: u32) -> Result<Vec<MessageRecord>, IronCoreError> {
        let query_lower = query.to_lowercase();
        let mut records = Vec::new();
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        for (_, value) in all {
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            let record = record.adjust_legacy_timestamps();
            // Evidentiary retention: skip hidden messages in search results.
            if record.hidden {
                continue;
            }
            if record.content.to_lowercase().contains(&query_lower) {
                records.push(record);
            }
        }

        // Return newest matches first and cap at the requested limit.
        records.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        if records.len() > limit as usize {
            records.truncate(limit as usize);
        }
        Ok(records)
    }

    pub fn remove_conversation(&self, peer_id: String) -> Result<(), IronCoreError> {
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        for (key, value) in all {
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            let record = record.adjust_legacy_timestamps();

            if record.peer_id.eq_ignore_ascii_case(&peer_id) {
                self.backend
                    .remove(&key)
                    .map_err(|_| IronCoreError::StorageError)?;
            }
        }

        Ok(())
    }

    pub fn mark_delivered(&self, id: String) -> Result<(), IronCoreError> {
        tracing::info!("Attempting to mark message {} as delivered", id);
        if let Some(mut record) = self.get(id.clone())? {
            record.delivered = true;
            self.add(record)?;
            tracing::info!("Successfully marked message {} as delivered", id);
        } else {
            tracing::warn!(
                "Message {} not found in history, could not mark as delivered",
                id
            );
        }
        Ok(())
    }

    pub fn clear(&self) -> Result<(), IronCoreError> {
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        for (key, _) in all {
            self.backend
                .remove(&key)
                .map_err(|_| IronCoreError::StorageError)?;
        }
        Ok(())
    }

    pub fn delete(&self, id: String) -> Result<(), IronCoreError> {
        let key = format!("msg_{}", id);
        self.backend
            .remove(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?;
        Ok(())
    }

    pub fn stats(&self) -> Result<HistoryStats, IronCoreError> {
        let mut stats = HistoryStats::default();
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        for (_, value) in all {
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            let record = record.adjust_legacy_timestamps();
            stats.total_messages += 1;
            match record.direction {
                MessageDirection::Sent => {
                    stats.sent_count += 1;
                    if !record.delivered {
                        stats.undelivered_count += 1;
                    }
                }
                MessageDirection::Received => stats.received_count += 1,
            }
        }
        Ok(stats)
    }

    pub fn count(&self) -> u32 {
        self.backend.count_prefix(b"msg_").unwrap_or(0) as u32
    }

    /// Enforce the message history retention policy by pruning old messages
    /// when the total count exceeds `max_messages`.
    ///
    /// Malformed or corrupt records that fail to deserialize are skipped
    /// and logged with a warning, rather than causing a panic or returning an error.
    pub fn enforce_retention(&self, max_messages: u32) -> Result<u32, IronCoreError> {
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        if all.len() <= max_messages as usize {
            return Ok(0);
        }

        // Parse and sort by timestamp ascending (oldest first)
        let mut records: Vec<(Vec<u8>, MessageRecord)> = all
            .into_iter()
            .filter_map(|(k, v)| match serde_json::from_slice::<MessageRecord>(&v) {
                Ok(rec) => Some((k, rec)),
                Err(e) => {
                    let key_str = String::from_utf8_lossy(&k);
                    tracing::warn!(
                        key = %key_str,
                        error = %e,
                        "skipping corrupt history record during retention sweep"
                    );
                    None
                }
            })
            .collect();

        records.sort_by_key(|a| a.1.timestamp);

        let to_remove = records.len().saturating_sub(max_messages as usize);
        for (key, _) in records.iter().take(to_remove) {
            self.backend
                .remove(key)
                .map_err(|_| IronCoreError::StorageError)?;
        }

        Ok(to_remove as u32)
    }

    pub fn prune_before(&self, before_timestamp: u64) -> Result<u32, IronCoreError> {
        let all = self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| IronCoreError::StorageError)?;

        let mut removed = 0u32;
        for (key, value) in all {
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            if record.timestamp < before_timestamp {
                self.backend
                    .remove(&key)
                    .map_err(|_| IronCoreError::StorageError)?;
                removed += 1;
            }
        }

        Ok(removed)
    }

    pub fn flush(&self) {
        let _ = self.backend.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::backend::MemoryStorage;

    #[test]
    fn test_case_insensitive_peer_id_matching() {
        let backend = Arc::new(MemoryStorage::new());
        let history = HistoryManager::new(backend);

        // Add message with lowercase peer ID
        let record1 = MessageRecord {
            id: "msg1".to_string(),
            peer_id: "12d3koowtest123".to_string(),
            direction: MessageDirection::Sent,
            content: "test message 1".to_string(),
            timestamp: 1000,
            sender_timestamp: 1000,
            delivered: false,
            hidden: false,
        };
        history.add(record1.clone()).unwrap();

        // Add message with mixed case peer ID
        let record2 = MessageRecord {
            id: "msg2".to_string(),
            peer_id: "12D3KooWTEST123".to_string(),
            direction: MessageDirection::Received,
            content: "test message 2".to_string(),
            timestamp: 2000,
            sender_timestamp: 2000,
            delivered: true,
            hidden: false,
        };
        history.add(record2.clone()).unwrap();

        // Query with uppercase - should find both
        let results = history
            .conversation("12D3KOOWTEST123".to_string(), 10)
            .unwrap();
        assert_eq!(results.len(), 2, "Should find messages regardless of case");

        // Query with lowercase - should find both
        let results = history
            .conversation("12d3koowtest123".to_string(), 10)
            .unwrap();
        assert_eq!(results.len(), 2, "Should find messages regardless of case");

        // Query with original mixed case - should find both
        let results = history
            .conversation("12D3KooWTEST123".to_string(), 10)
            .unwrap();
        assert_eq!(results.len(), 2, "Should find messages regardless of case");
    }

    #[test]
    fn test_remove_conversation_case_insensitive() {
        let backend = Arc::new(MemoryStorage::new());
        let history = HistoryManager::new(backend);

        // Add messages with different case variants
        let record1 = MessageRecord {
            id: "msg1".to_string(),
            peer_id: "lowercase123".to_string(),
            direction: MessageDirection::Sent,
            content: "test 1".to_string(),
            timestamp: 1000,
            sender_timestamp: 1000,
            delivered: false,
            hidden: false,
        };
        history.add(record1).unwrap();

        let record2 = MessageRecord {
            id: "msg2".to_string(),
            peer_id: "LOWERCASE123".to_string(),
            direction: MessageDirection::Sent,
            content: "test 2".to_string(),
            timestamp: 2000,
            sender_timestamp: 2000,
            delivered: false,
            hidden: false,
        };
        history.add(record2).unwrap();

        // Remove with mixed case
        history
            .remove_conversation("LowerCase123".to_string())
            .unwrap();

        // Verify both were removed
        let results = history
            .conversation("lowercase123".to_string(), 10)
            .unwrap();
        assert_eq!(
            results.len(),
            0,
            "All messages should be removed regardless of case"
        );
    }

    #[test]
    fn test_enforce_retention_handles_corrupt_records() {
        let backend = Arc::new(MemoryStorage::new());
        let history = HistoryManager::new(backend.clone());

        // Seed 4 valid messages with distinct timestamps
        for i in 1..=4 {
            let record = MessageRecord {
                id: format!("msg{}", i),
                peer_id: "peer_a".to_string(),
                direction: MessageDirection::Sent,
                content: format!("message {}", i),
                timestamp: 1000 + i * 100, // 1100, 1200, 1300, 1400
                sender_timestamp: 1000 + i * 100,
                delivered: true,
                hidden: false,
            };
            history.add(record).unwrap();
        }

        // Seed a corrupt record that fails deserialization
        let corrupt_key = b"msg_corrupt_id";
        let corrupt_val = b"not a valid json byte blob";
        backend.put(corrupt_key, corrupt_val).unwrap();

        // Check total raw count is 5
        assert_eq!(backend.count_prefix(b"msg_").unwrap(), 5);

        // Call enforce_retention down to max_messages = 2
        // Should return Ok(2) since there are 4 valid messages and we prune to 2
        let pruned = history.enforce_retention(2).unwrap();
        assert_eq!(pruned, 2);

        // Verify valid messages msg1 and msg2 were pruned, while msg3 and msg4 remain
        assert!(history.get("msg1".to_string()).unwrap().is_none());
        assert!(history.get("msg2".to_string()).unwrap().is_none());
        assert!(history.get("msg3".to_string()).unwrap().is_some());
        assert!(history.get("msg4".to_string()).unwrap().is_some());

        // Verify the corrupt record is skipped (meaning it was not removed)
        assert!(backend.get(corrupt_key).unwrap().is_some());
    }
}
