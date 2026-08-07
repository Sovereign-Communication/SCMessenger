// Contact management storage

use crate::store::backend::StorageBackend;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const CONTACT_KEY_PREFIX: &[u8] = b"contact:";
const CONTACT_BUNDLE_KEY_PREFIX: &[u8] = b"contact_bundle:";
const IDENTITY_ID_INDEX_PREFIX: &[u8] = b"identity_id_idx:";

fn contact_key(peer_id: &str) -> Vec<u8> {
    [CONTACT_KEY_PREFIX, peer_id.as_bytes()].concat()
}

fn contact_bundle_key(public_key_hex: &str) -> Vec<u8> {
    [CONTACT_BUNDLE_KEY_PREFIX, public_key_hex.as_bytes()].concat()
}

fn identity_id_index_key(identity_id: &str) -> Vec<u8> {
    [IDENTITY_ID_INDEX_PREFIX, identity_id.as_bytes()].concat()
}

fn current_timestamp() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub peer_id: String,
    pub nickname: Option<String>,
    pub local_nickname: Option<String>,
    pub public_key: String,
    pub added_at: u64,
    pub last_seen: Option<u64>,
    #[serde(default)]
    pub last_known_device_id: Option<String>,
}

impl Contact {
    pub fn new(peer_id: String, public_key: String) -> Self {
        Self {
            peer_id,
            nickname: None,
            local_nickname: None,
            public_key,
            added_at: current_timestamp(),
            last_seen: None,
            last_known_device_id: None,
        }
    }

    pub fn with_nickname(mut self, nickname: String) -> Self {
        self.nickname = Some(nickname);
        self
    }

    pub fn display_name(&self) -> &str {
        if let Some(ref local) = self.local_nickname {
            return local;
        }
        self.nickname.as_deref().unwrap_or(&self.peer_id)
    }

    pub fn federated_nickname(&self) -> Option<&str> {
        self.nickname.as_deref()
    }
}

/// Contact bundle for ratchet session resumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactBundle {
    pub peer_id: String,
    pub public_key_hex: String,
    pub identity_id: String,
    pub x25519_public: Option<Vec<u8>>,
    pub mlkem_public: Option<Vec<u8>>,
    pub mldsa_public: Option<Vec<u8>>,
    pub supported_suites: Vec<u8>,
    pub created_at: u64,
}

/// Contact manager for persistent storage
#[derive(Clone)]
pub struct ContactManager {
    backend: Arc<dyn StorageBackend>,
}

impl ContactManager {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    /// Add or update a contact
    pub fn add(&self, contact: Contact) -> Result<(), String> {
        let key = contact_key(&contact.peer_id);
        let data = serde_json::to_vec(&contact).map_err(|e| e.to_string())?;
        self.backend.put(&key, &data)?;
        
        // Index by identity_id (blake3 hash of public_key)
        let identity_id = crate::identity::identity_id_from_public_key_hex(&contact.public_key)
            .ok_or("Invalid public key for identity_id")?;
        let idx_key = identity_id_index_key(&identity_id);
        self.backend.put(&idx_key, contact.peer_id.as_bytes())?;
        
        // Index by public_key
        let bundle_key = contact_bundle_key(&contact.public_key);
        self.backend.put(&bundle_key, contact.peer_id.as_bytes())?;
        
        Ok(())
    }

    /// Get a contact by peer_id
    pub fn get(&self, peer_id: &str) -> Result<Option<Contact>, String> {
        let key = contact_key(peer_id);
        self.backend.get(&key)?
            .map(|data| serde_json::from_slice(&data).map_err(|e| e.to_string()))
            .transpose()
    }

    /// Get a contact by public_key
    pub fn get_by_public_key(&self, public_key: &str) -> Result<Option<Contact>, String> {
        let key = contact_bundle_key(public_key);
        if let Some(peer_id_bytes) = self.backend.get(&key)? {
            let peer_id = String::from_utf8(peer_id_bytes).map_err(|e| e.to_string())?;
            self.get(&peer_id)
        } else {
            Ok(None)
        }
    }

    /// Get a contact by identity_id
    pub fn get_by_identity_id(&self, identity_id: &str) -> Result<Option<Contact>, String> {
        let key = identity_id_index_key(identity_id);
        if let Some(peer_id_bytes) = self.backend.get(&key)? {
            let peer_id = String::from_utf8(peer_id_bytes).map_err(|e| e.to_string())?;
            self.get(&peer_id)
        } else {
            Ok(None)
        }
    }

    /// List all contacts
    pub fn list(&self) -> Result<Vec<Contact>, String> {
        let mut contacts = Vec::new();
        for (_, value) in self.backend.scan_prefix(CONTACT_KEY_PREFIX)? {
            if let Ok(contact) = serde_json::from_slice::<Contact>(&value) {
                contacts.push(contact);
            }
        }
        Ok(contacts)
    }

    /// Remove a contact
    pub fn remove(&self, peer_id: &str) -> Result<(), String> {
        if let Some(contact) = self.get(peer_id)? {
            let key = contact_key(peer_id);
            self.backend.remove(&key)?;
            
            // Remove indexes
            let identity_id = crate::identity::identity_id_from_public_key_hex(&contact.public_key)
                .ok_or("Invalid public key")?;
            let idx_key = identity_id_index_key(&identity_id);
            self.backend.remove(&idx_key)?;
            
            let bundle_key = contact_bundle_key(&contact.public_key);
            self.backend.remove(&bundle_key)?;
        }
        Ok(())
    }

    /// Get contact count
    pub fn count(&self) -> Result<usize, String> {
        self.backend.count_prefix(CONTACT_KEY_PREFIX)
    }

    /// Migrate identity_id index (for backward compatibility)
    pub fn migrate_identity_id_index(&self) -> Result<(), String> {
        for contact in self.list()? {
            let identity_id = crate::identity::identity_id_from_public_key_hex(&contact.public_key)
                .ok_or("Invalid public key")?;
            let idx_key = identity_id_index_key(&identity_id);
            if self.backend.get(&idx_key)?.is_none() {
                self.backend.put(&idx_key, contact.peer_id.as_bytes())?;
            }
        }
        Ok(())
    }

    /// Get contact bundle (for ratchet)
    pub fn get_contact_bundle(&self, public_key: &str) -> Result<Option<ContactBundle>, String> {
        // This is a simplified version - in full core it has more fields
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::backend::MemoryStorage;

    #[test]
    fn test_contact_manager() {
        let backend = Arc::new(MemoryStorage::new());
        let mgr = ContactManager::new(backend);
        
        let contact = Contact::new("peer-1".to_string(), "a".repeat(64));
        mgr.add(contact.clone()).unwrap();
        
        let loaded = mgr.get("peer-1").unwrap().unwrap();
        assert_eq!(loaded.peer_id, "peer-1");
        assert_eq!(loaded.public_key, contact.public_key);
    }
}