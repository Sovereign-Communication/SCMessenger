// Message history persistence and retrieval

use crate::store::backend::StorageBackend;
use crate::error::MeshError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

fn current_timestamp() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

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
    #[serde(default)]
    pub hidden: bool,
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

impl HistoryManager {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    /// P0_SECURITY_005: Expose the storage backend for audit log persistence.
    pub fn backend(&self) -> Arc<dyn StorageBackend> {
        self.backend.clone()
    }

    pub fn add(&self, record: MessageRecord) -> Result<(), MeshError> {
        let key = format!("msg_{}", record.id);
        let value = serde_json::to_vec(&record).map_err(|_| MeshError::Serialization(SerializationError::Json(serde_json::Error::custom("serialize failed"))))?;
        self.backend.put(key.as_bytes(), &value)
            .map_err(|_| MeshError::Storage("put failed".to_string()))?;
        Ok(())
    }

    pub fn get(&self, id: String) -> Result<Option<MessageRecord>, MeshError> {
        let key = format!("msg_{}", id);
        if let Some(data) = self
            .backend
            .get(key.as_bytes())
            .map_err(|_| MeshError::Storage("get failed".to_string()))?
        {
            let record: MessageRecord =
                serde_json::from_slice(&data).map_err(|_| MeshError::Serialization(SerializationError::Json(serde_json::Error::custom("deserialize failed"))))?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    pub fn recent(
        &self,
        peer_filter: Option<String>,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, MeshError> {
        self.recent_internal(peer_filter, limit, false)
    }

    fn recent_internal(
        &self,
        peer_filter: Option<String>,
        limit: u32,
        include_hidden: bool,
    ) -> Result<Vec<MessageRecord>, MeshError> {
        let mut records = Vec::new();
        for (_, value) in self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| MeshError::Storage("scan failed".to_string()))?
        {
            if let Ok(mut record) = serde_json::from_slice::<MessageRecord>(&value) {
                if let Some(ref peer) = peer_filter {
                    if &record.peer_id != peer {
                        continue;
                    }
                }
                if !include_hidden && record.hidden {
                    continue;
                }
                records.push(record);
            }
        }
        // Sort by timestamp descending
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        records.truncate(limit as usize);
        Ok(records)
    }

    pub fn clear(&self) -> Result<(), MeshError> {
        for (key, _) in self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| MeshError::Storage("scan failed".to_string()))?
        {
            self.backend.remove(&key)
                .map_err(|_| MeshError::Storage("remove failed".to_string()))?;
        }
        Ok(())
    }

    pub fn clear_conversation(&self, peer_id: &str) -> Result<(), MeshError> {
        let mut keys_to_remove = Vec::new();
        for (key, value) in self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| MeshError::Storage("scan failed".to_string()))?
        {
            if let Ok(record) = serde_json::from_slice::<MessageRecord>(&value) {
                if record.peer_id == peer_id {
                    keys_to_remove.push(key);
                }
            }
        }
        for key in keys_to_remove {
            self.backend.remove(&key)
                .map_err(|_| MeshError::Storage("remove failed".to_string()))?;
        }
        Ok(())
    }

    pub fn stats(&self) -> Result<HistoryStats, MeshError> {
        let mut stats = HistoryStats::default();
        for (_, value) in self
            .backend
            .scan_prefix(b"msg_")
            .map_err(|_| MeshError::Storage("scan failed".to_string()))?
        {
            if let Ok(record) = serde_json::from_slice::<MessageRecord>(&value) {
                if record.hidden {
                    continue;
                }
                stats.total_messages += 1;
                match record.direction {
                    MessageDirection::Sent => stats.sent_count += 1,
                    MessageDirection::Received => stats.received_count += 1,
                }
                if !record.delivered {
                    stats.undelivered_count += 1;
                }
            }
        }
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::backend::MemoryStorage;

    #[test]
    fn test_history_manager() {
        let backend = Arc::new(MemoryStorage::new());
        let mgr = HistoryManager::new(backend);
        
        let msg = MessageRecord::new_sent("peer-1".to_string(), "Hello".to_string());
        mgr.add(msg.clone()).unwrap();
        
        let loaded = mgr.get(msg.id.clone()).unwrap().unwrap();
        assert_eq!(loaded.content, "Hello");
        assert_eq!(loaded.direction, MessageDirection::Sent);
        
        let recent = mgr.recent(None, 10).unwrap();
        assert_eq!(recent.len(), 1);
    }
}