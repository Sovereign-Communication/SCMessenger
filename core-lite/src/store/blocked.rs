// Blocked identities and device management

use crate::store::backend::StorageBackend;
use crate::error::MeshError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

/// Storage key prefix for blocked identity entries
const BLOCKED_PREFIX: &str = "blocked:";
/// Storage key prefix for device registry entries (peer -> known device IDs)
const DEVICE_REGISTRY_PREFIX: &str = "blocked_devs:";

/// A blocked identity entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedIdentity {
    /// The peer ID (identity hash) being blocked
    pub peer_id: String,
    /// Optional device ID for granular blocking.
    /// When present, only this device of the peer is blocked.
    /// When absent, all devices of the peer are blocked.
    pub device_id: Option<String>,
    /// When this identity was blocked
    pub blocked_at: u64,
    /// Optional reason for blocking
    pub reason: Option<String>,
    /// Notes about this block
    pub notes: Option<String>,
    /// When true, the contact has been both blocked AND deleted.
    #[serde(default)]
    pub is_deleted: bool,
}

impl BlockedIdentity {
    /// Create a new blocked identity
    pub fn new(peer_id: String) -> Self {
        Self {
            peer_id,
            device_id: None,
            blocked_at: current_timestamp(),
            reason: None,
            notes: None,
            is_deleted: false,
        }
    }

    pub fn with_device_id(mut self, device_id: String) -> Self {
        self.device_id = Some(device_id);
        self
    }

    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    fn storage_key(&self) -> String {
        match &self.device_id {
            Some(device_id) => format!("{}{}:{}", BLOCKED_PREFIX, self.peer_id, device_id),
            None => format!("{}{}", BLOCKED_PREFIX, self.peer_id),
        }
    }
}

fn current_timestamp() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Manager for blocked identities with device-ID pairing support.
#[derive(Clone)]
pub struct BlockedManager {
    backend: Arc<dyn StorageBackend>,
}

impl BlockedManager {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    /// Block a peer ID, also blocking all known device IDs for that peer.
    pub fn block(&self, blocked: BlockedIdentity) -> Result<(), MeshError> {
        let key = blocked.storage_key();
        let value = serde_json::to_vec(&blocked).map_err(|_| MeshError::Serialization(SerializationError::Json(serde_json::Error::custom("serialize failed"))))?;
        self.backend.put(key.as_bytes(), &value)
            .map_err(|_| MeshError::Storage("put failed".to_string()))?;

        // Peer-level block: also block every known device for this peer
        if blocked.device_id.is_none() {
            let devices = self.get_known_devices(&blocked.peer_id)?;
            for device_id in &devices {
                let device_blocked = BlockedIdentity {
                    peer_id: blocked.peer_id.clone(),
                    device_id: Some(device_id.clone()),
                    blocked_at: blocked.blocked_at,
                    reason: blocked.reason.clone(),
                    notes: blocked.notes.clone(),
                    is_deleted: blocked.is_deleted,
                };
                let dkey = device_blocked.storage_key();
                let dvalue = serde_json::to_vec(&device_blocked).map_err(|_| MeshError::Serialization(SerializationError::Json(serde_json::Error::custom("serialize failed"))))?;
                self.backend.put(dkey.as_bytes(), &dvalue)
                    .map_err(|_| MeshError::Storage("put failed".to_string()))?;
            }
        }
        Ok(())
    }

    /// Unblock a peer (and all its devices)
    pub fn unblock(&self, peer_id: &str, device_id: Option<&str>) -> Result<(), MeshError> {
        if let Some(dev_id) = device_id {
            let key = format!("{}{}:{}", BLOCKED_PREFIX, peer_id, dev_id);
            self.backend.remove(key.as_bytes())
                .map_err(|_| MeshError::Storage("remove failed".to_string()))?;
        } else {
            // Remove peer-level block
            let key = format!("{}{}", BLOCKED_PREFIX, peer_id);
            self.backend.remove(key.as_bytes())
                .map_err(|_| MeshError::Storage("remove failed".to_string()))?;
            
            // Remove all device blocks for this peer
            let prefix = format!("{}{}:", BLOCKED_PREFIX, peer_id);
            for (key, _) in self.backend.scan_prefix(prefix.as_bytes())
                .map_err(|_| MeshError::Storage("scan failed".to_string()))? {
                self.backend.remove(&key)
                    .map_err(|_| MeshError::Storage("remove failed".to_string()))?;
            }
        }
        Ok(())
    }

    /// Check if a peer (or specific device) is blocked
    pub fn is_blocked(&self, peer_id: &str, device_id: Option<&str>) -> Result<bool, MeshError> {
        // Check peer-level block
        let key = format!("{}{}", BLOCKED_PREFIX, peer_id);
        if self.backend.get(key.as_bytes())
            .map_err(|_| MeshError::Storage("get failed".to_string()))?
            .is_some() {
            return Ok(true);
        }

        // Check device-level block
        if let Some(dev_id) = device_id {
            let key = format!("{}{}:{}", BLOCKED_PREFIX, peer_id, dev_id);
            if self.backend.get(key.as_bytes())
                .map_err(|_| MeshError::Storage("get failed".to_string()))?
                .is_some() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Register a device ID for a peer in the device registry
    pub fn register_device_id(&self, peer_id: &str, device_id: &str) -> Result<(), MeshError> {
        let key = format!("{}{}", DEVICE_REGISTRY_PREFIX, peer_id);
        let mut devices: HashSet<String> = self
            .backend
            .get(key.as_bytes())
            .map_err(|_| MeshError::Storage("get failed".to_string()))?
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        
        devices.insert(device_id.to_string());
        
        let value = serde_json::to_vec(&devices).map_err(|_| MeshError::Serialization(SerializationError::Json(serde_json::Error::custom("serialize failed"))))?;
        self.backend.put(key.as_bytes(), &value)
            .map_err(|_| MeshError::Storage("put failed".to_string()))?;
        Ok(())
    }

    /// Get all known device IDs for a blocked peer
    pub fn get_known_devices(&self, peer_id: &str) -> Result<Vec<String>, MeshError> {
        let key = format!("{}{}", DEVICE_REGISTRY_PREFIX, peer_id);
        Ok(self
            .backend
            .get(key.as_bytes())
            .map_err(|_| MeshError::Storage("get failed".to_string()))?
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
            .into_iter()
            .collect())
    }

    /// Get all blocked identities (peer-level only)
    pub fn list(&self) -> Result<Vec<BlockedIdentity>, MeshError> {
        let mut blocked = Vec::new();
        let prefix = format!("{}", BLOCKED_PREFIX);
        for (_, value) in self.backend.scan_prefix(prefix.as_bytes())
            .map_err(|_| MeshError::Storage("scan failed".to_string()))? {
            if let Ok(b) = serde_json::from_slice::<BlockedIdentity>(&value) {
                // Only include peer-level blocks (no device_id)
                if b.device_id.is_none() {
                    blocked.push(b);
                }
            }
        }
        Ok(blocked)
    }

    /// Get peer IDs that are blocked-only (not deleted)
    pub fn blocked_only_peer_ids(&self) -> Result<Vec<String>, MeshError> {
        let mut ids = Vec::new();
        for b in self.list()? {
            if !b.is_deleted {
                ids.push(b.peer_id);
            }
        }
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::backend::MemoryStorage;

    #[test]
    fn test_blocked_manager() {
        let backend = Arc::new(MemoryStorage::new());
        let mgr = BlockedManager::new(backend);
        
        let blocked = BlockedIdentity::new("peer-1".to_string());
        mgr.block(blocked).unwrap();
        
        assert!(mgr.is_blocked("peer-1", None).unwrap());
        assert!(!mgr.is_blocked("peer-2", None).unwrap());
        
        mgr.unblock("peer-1", None).unwrap();
        assert!(!mgr.is_blocked("peer-1", None).unwrap());
    }
}