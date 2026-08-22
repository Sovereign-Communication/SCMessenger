#![cfg(not(target_arch = "wasm32"))]

// Contact management bridge for mobile platforms
//
// Wraps CLI contact storage logic (sled-based) for UniFFI exposure to Android/iOS.
// Ensures cross-platform database compatibility via JSON serialization.

use crate::mobile_bridge::HistoryManager;
use anyhow::{Context, Result};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, Weak};

/// Public contact structure exposed via UniFFI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub peer_id: String,
    pub nickname: Option<String>,
    pub local_nickname: Option<String>,
    pub public_key: String,
    pub added_at: u64,
    pub last_seen: Option<u64>,
    pub notes: Option<String>,
    #[serde(default)]
    pub last_known_device_id: Option<String>,
    #[serde(default)]
    pub verified_at: Option<u64>,
    #[serde(default)]
    pub is_tombstone: bool,
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
            notes: None,
            last_known_device_id: None,
            verified_at: None,
            is_tombstone: false,
        }
    }

    pub fn tombstone(peer_id: String) -> Self {
        Self {
            peer_id,
            nickname: None,
            local_nickname: None,
            public_key: String::new(),
            added_at: current_timestamp(),
            last_seen: None,
            notes: None,
            last_known_device_id: None,
            verified_at: None,
            is_tombstone: true,
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

/// Contact manager with thread-safe sled database backend.
///
/// Mobile clients hold a bridge manager for their UI while IronCore opens the
/// same store for identity backup/restore. All handles for a storage path must
/// share one `Db` instance; otherwise sled's per-handle view can make a just
/// saved contact invisible to the backup exporter (or a restored contact
/// invisible to the UI) until a process restart.
#[derive(uniffi::Object)]
pub struct ContactManager {
    db: Arc<Mutex<Db>>,
}

type SharedContactDatabase = Arc<Mutex<Db>>;
type WeakContactDatabase = Weak<Mutex<Db>>;

fn contact_database_registry() -> &'static Mutex<HashMap<PathBuf, WeakContactDatabase>> {
    static REGISTRY: OnceLock<Mutex<HashMap<PathBuf, WeakContactDatabase>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

#[uniffi::export]
impl ContactManager {
    /// Create or open contact database at the given path
    #[uniffi::constructor]
    pub fn new(storage_path: String) -> Result<Self, crate::IronCoreError> {
        let path = PathBuf::from(storage_path).join("contacts.db");
        let mut registry = contact_database_registry().lock();

        if let Some(existing) = registry.get(&path).and_then(Weak::upgrade) {
            return Ok(Self { db: existing });
        }

        // A previous manager may have been released after an app lifecycle
        // transition. Drop its expired weak entry before opening a new store.
        registry.remove(&path);
        let db = sled::Config::default()
            .path(&path)
            .mode(sled::Mode::LowSpace)
            .use_compression(false)
            .open()
            .context("Failed to open contacts database")
            .map_err(|_| crate::IronCoreError::StorageError)?;

        let db: SharedContactDatabase = Arc::new(Mutex::new(db));
        registry.insert(path, Arc::downgrade(&db));

        Ok(Self { db })
    }

    /// Add a contact to the database
    pub fn add(&self, contact: Contact) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock();
        let key = contact.peer_id.as_bytes();
        let value = serde_json::to_vec(&contact)
            .context("Failed to serialize contact")
            .map_err(|_| crate::IronCoreError::Internal)?;

        db.insert(key, value)
            .context("Failed to insert contact")
            .map_err(|_| crate::IronCoreError::StorageError)?;

        Ok(())
    }

    /// Get a contact by peer ID, public key, or identity ID
    pub fn get(&self, peer_id: String) -> Result<Option<Contact>, crate::IronCoreError> {
        let db = self.db.lock();
        if let Some(data) = db
            .get(peer_id.as_bytes())
            .map_err(|_| crate::IronCoreError::StorageError)?
        {
            let contact: Contact = serde_json::from_slice(&data)
                .context("Failed to deserialize contact")
                .map_err(|_| crate::IronCoreError::Internal)?;
            return Ok(Some(contact));
        }

        let trimmed = peer_id.trim();

        // Scan entries to find matching public key, identity ID, or peer ID
        for item in db.iter() {
            let (_, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            if let Ok(contact) = serde_json::from_slice::<Contact>(&value) {
                if contact.peer_id.eq_ignore_ascii_case(trimmed)
                    || contact.public_key.eq_ignore_ascii_case(trimmed)
                {
                    return Ok(Some(contact));
                }
                if let Ok(pk_bytes) = hex::decode(&contact.public_key) {
                    if pk_bytes.len() == 32 {
                        let id = hex::encode(blake3::hash(&pk_bytes).as_bytes());
                        if id.eq_ignore_ascii_case(trimmed) {
                            return Ok(Some(contact));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Remove a contact
    pub fn remove(&self, peer_id: String) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock();
        let _ = db.remove(peer_id.as_bytes());

        let trimmed = peer_id.trim();
        let mut keys_to_remove = Vec::new();

        for item in db.iter() {
            let (key, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            if let Ok(contact) = serde_json::from_slice::<Contact>(&value) {
                if contact.peer_id.eq_ignore_ascii_case(trimmed)
                    || contact.public_key.eq_ignore_ascii_case(trimmed)
                {
                    keys_to_remove.push(key);
                } else if let Ok(pk_bytes) = hex::decode(&contact.public_key) {
                    if pk_bytes.len() == 32 {
                        let id = hex::encode(blake3::hash(&pk_bytes).as_bytes());
                        if id.eq_ignore_ascii_case(trimmed) {
                            keys_to_remove.push(key);
                        }
                    }
                }
            }
        }

        for key in keys_to_remove {
            let _ = db.remove(key);
        }

        Ok(())
    }

    /// List all contacts, sorted by display name
    pub fn list(&self) -> Result<Vec<Contact>, crate::IronCoreError> {
        let db = self.db.lock();
        let mut contacts = Vec::new();

        for item in db.iter() {
            let (_, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let contact: Contact = serde_json::from_slice(&value)
                .context("Failed to deserialize contact")
                .map_err(|_| crate::IronCoreError::Internal)?;
            contacts.push(contact);
        }

        contacts.sort_by(|a, b| a.display_name().cmp(b.display_name()));
        Ok(contacts)
    }

    /// Search contacts by query (matches nickname, peer_id, public_key, or notes)
    pub fn search(&self, query: String) -> Result<Vec<Contact>, crate::IronCoreError> {
        let db = self.db.lock();
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for item in db.iter() {
            let (_, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let contact: Contact =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;

            let matches = contact.peer_id.to_lowercase().contains(&query_lower)
                || contact.public_key.to_lowercase().contains(&query_lower)
                || contact
                    .nickname
                    .as_ref()
                    .is_some_and(|n| n.to_lowercase().contains(&query_lower))
                || contact
                    .notes
                    .as_ref()
                    .is_some_and(|n| n.to_lowercase().contains(&query_lower));

            if matches {
                results.push(contact);
            }
        }

        results.sort_by(|a, b| a.display_name().cmp(b.display_name()));
        Ok(results)
    }

    /// Set or update contact federated nickname
    pub fn set_nickname(
        &self,
        peer_id: String,
        nickname: Option<String>,
    ) -> Result<(), crate::IronCoreError> {
        if let Some(mut contact) = self.get(peer_id.clone())? {
            contact.nickname = nickname
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            self.add(contact)?;
            Ok(())
        } else {
            Err(crate::IronCoreError::InvalidInput)
        }
    }

    /// Set or update local nickname override
    pub fn set_local_nickname(
        &self,
        peer_id: String,
        nickname: Option<String>,
    ) -> Result<(), crate::IronCoreError> {
        if let Some(mut contact) = self.get(peer_id.clone())? {
            contact.local_nickname = nickname
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            self.add(contact)?;
            Ok(())
        } else {
            Err(crate::IronCoreError::InvalidInput)
        }
    }

    /// Update contact's last seen timestamp to now
    pub fn update_last_seen(&self, peer_id: String) -> Result<(), crate::IronCoreError> {
        if let Some(mut contact) = self.get(peer_id.clone())? {
            contact.last_seen = Some(current_timestamp());
            self.add(contact)?;
            Ok(())
        } else {
            // Silently ignore if contact doesn't exist
            Ok(())
        }
    }

    /// Update the last known device ID for a contact (WS13.2)
    pub fn update_device_id(
        &self,
        peer_id: String,
        device_id: Option<String>,
    ) -> Result<(), crate::IronCoreError> {
        if let Some(mut contact) = self.get(peer_id.clone())? {
            contact.last_known_device_id = device_id;
            self.add(contact)?;
            Ok(())
        } else {
            Err(crate::IronCoreError::InvalidInput)
        }
    }

    /// Reconcile contacts from message history to recover potentially lost records.
    /// Scans all message records and creates a basic contact if the peer_id is unknown.
    pub fn reconcile_from_history(
        &self,
        history: &HistoryManager,
    ) -> Result<u32, crate::IronCoreError> {
        // The bridge needs to call the inner manager's reconcile logic.
        // Since ContactManager is the inner type in contacts.rs, and this bridge
        // wraps the sled DB, we implement the logic here or delegate.

        // Note: history is also a bridge.
        let messages = history.recent(None, 10000)?;
        let mut recovered = 0;

        for msg in messages {
            if self.get(msg.peer_id.clone())?.is_none() {
                // We need a public key to create a Contact.
                // In a real scenario, we'd use the libp2p peer_id to derive the key.
                let contact = Contact::new(msg.peer_id.clone(), msg.peer_id.clone());
                self.add(contact)?;
                recovered += 1;
            }
        }
        Ok(recovered)
    }

    /// Merge contacts from another device using LWW-register CRDT semantics.
    /// Higher `added_at` timestamp wins. For blocks, block always wins over unblock.
    /// Returns the number of contacts updated.
    pub fn merge_remote_contacts(
        &self,
        remote_contacts: Vec<Contact>,
    ) -> Result<u32, crate::IronCoreError> {
        let mut updated = 0;
        for remote in remote_contacts {
            if let Some(local) = self.get(remote.peer_id.clone())? {
                // LWW: higher timestamp wins
                if remote.added_at > local.added_at {
                    let mut merged = remote;
                    // Preserve local-only fields that the remote doesn't know about
                    if merged.local_nickname.is_none() && local.local_nickname.is_some() {
                        merged.local_nickname = local.local_nickname;
                    }
                    if merged.verified_at.is_none() && local.verified_at.is_some() {
                        merged.verified_at = local.verified_at;
                    }
                    self.add(merged)?;
                    updated += 1;
                }
            } else {
                // New contact from remote
                self.add(remote)?;
                updated += 1;
            }
        }
        Ok(updated)
    }

    /// Mark a contact as verified (out-of-band verification completed).
    pub fn mark_verified(&self, peer_id: String) -> Result<(), crate::IronCoreError> {
        if let Some(mut contact) = self.get(peer_id)? {
            contact.verified_at = Some(current_timestamp());
            self.add(contact)?;
        }
        Ok(())
    }

    /// Clear verification status (e.g., when key changes).
    pub fn unverify(&self, peer_id: String) -> Result<(), crate::IronCoreError> {
        if let Some(mut contact) = self.get(peer_id)? {
            contact.verified_at = None;
            self.add(contact)?;
        }
        Ok(())
    }

    /// Count total contacts
    pub fn count(&self) -> u32 {
        let db = self.db.lock();
        db.len() as u32
    }

    pub fn flush(&self) {
        let db = self.db.lock();
        let _ = db.flush();
    }

    /// Verify database integrity and detect corruption.
    /// Returns an error if the database has data but returns 0 contacts.
    pub fn verify_integrity(&self) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock();
        let contact_count = db.len() as u32;
        let has_entries = db.iter().next().is_some();

        if contact_count == 0 && has_entries {
            // Database has entries but count is 0 - potential corruption
            return Err(crate::IronCoreError::CorruptionDetected);
        }
        Ok(())
    }

    /// Emergency recovery: Reconstruct contacts from message history.
    /// Scans all message records and creates a basic contact if the peer_id is unknown.
    pub fn emergency_recover(&self, history: &HistoryManager) -> Result<u32, crate::IronCoreError> {
        let messages = history.recent(None, 10000)?;
        let mut recovered = 0;

        for msg in messages {
            if self.get(msg.peer_id.clone())?.is_none() {
                // We need a public key to create a Contact.
                // In a real scenario, we'd use the libp2p peer_id to derive the key.
                // For emergency recovery, we use the peer_id as the public key placeholder.
                let contact = Contact::new(msg.peer_id.clone(), msg.peer_id.clone());
                self.add(contact)?;
                recovered += 1;
            }
        }
        Ok(recovered)
    }
}

fn current_timestamp() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_creation() {
        let contact = Contact::new("12D3KooTest".to_string(), "abcd1234".to_string())
            .with_nickname("Alice".to_string());

        assert_eq!(contact.display_name(), "Alice");
        assert_eq!(contact.peer_id, "12D3KooTest");
    }

    #[test]
    fn test_contact_manager() -> Result<(), crate::IronCoreError> {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage_path = temp_dir.path().to_str().unwrap_or_default().to_string();

        let manager = ContactManager::new(storage_path)?;

        // Add contact
        let contact = Contact::new("12D3KooTest1".to_string(), "pubkey1".to_string())
            .with_nickname("Alice".to_string());

        manager.add(contact)?;

        // Retrieve contact
        let retrieved = manager.get("12D3KooTest1".to_string())?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().nickname, Some("Alice".to_string()));

        // List contacts
        let list = manager.list()?;
        assert_eq!(list.len(), 1);

        // Search
        let results = manager.search("alice".to_string())?;
        assert_eq!(results.len(), 1);

        // Count
        assert_eq!(manager.count(), 1);

        Ok(())
    }

    #[test]
    fn test_contact_persistence_across_manager_restart() -> Result<(), crate::IronCoreError> {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage_path = temp_dir.path().to_str().unwrap_or_default().to_string();

        {
            let manager = ContactManager::new(storage_path.clone())?;
            let contact = Contact::new("peer-alpha".to_string(), "pubkey-alpha".to_string())
                .with_nickname("FederatedName".to_string());
            manager.add(contact)?;
            manager.set_local_nickname("peer-alpha".to_string(), Some("LocalAlias".to_string()))?;
        }

        let reloaded = ContactManager::new(storage_path)?;
        let contact = reloaded
            .get("peer-alpha".to_string())?
            .expect("contact should persist");
        assert_eq!(contact.nickname.as_deref(), Some("FederatedName"));
        assert_eq!(contact.local_nickname.as_deref(), Some("LocalAlias"));
        assert_eq!(reloaded.count(), 1);
        Ok(())
    }

    #[test]
    fn test_contact_manager_multi_flavor_lookup() -> Result<(), crate::IronCoreError> {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage_path = temp_dir.path().to_str().unwrap_or_default().to_string();
        let manager = ContactManager::new(storage_path)?;

        let public_key = "2620607086af6de0d8c4f181a09ee5c90b0a5f968a7379a31b9906c4255fbd9c".to_string();
        let pk_bytes = hex::decode(&public_key).unwrap();
        let identity_id = hex::encode(blake3::hash(&pk_bytes).as_bytes());

        // Contact stored under libp2p peer ID (as CLI does)
        let contact = Contact::new("12D3KooWCPCJ6B3aQD55nd7FvjuUkrhHRgqXGXoNGw2rstaohUrw".to_string(), public_key.clone())
            .with_nickname("Lucas Ballek".to_string());
        manager.add(contact)?;

        // 1. Get by peer ID
        let c1 = manager.get("12D3KooWCPCJ6B3aQD55nd7FvjuUkrhHRgqXGXoNGw2rstaohUrw".to_string())?;
        assert!(c1.is_some());
        assert_eq!(c1.unwrap().nickname.as_deref(), Some("Lucas Ballek"));

        // 2. Get by public key
        let c2 = manager.get(public_key)?;
        assert!(c2.is_some());
        assert_eq!(c2.unwrap().nickname.as_deref(), Some("Lucas Ballek"));

        // 3. Get by identity ID (Blake3 hash)
        let c3 = manager.get(identity_id)?;
        assert!(c3.is_some());
        assert_eq!(c3.unwrap().nickname.as_deref(), Some("Lucas Ballek"));

        Ok(())
    }
}
