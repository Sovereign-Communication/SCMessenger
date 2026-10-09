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

/// Per-path open gate (#413 review F1). Serialises concurrent opens of the
/// SAME path only, so the lock-contention retry (up to ~5 s) is never run while
/// holding the global registry mutex: opens of other paths and every registry
/// lookup stay unblocked.
fn contact_open_gate(path: &std::path::Path) -> Arc<Mutex<()>> {
    static GATES: OnceLock<Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>> = OnceLock::new();
    GATES
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .entry(path.to_path_buf())
        .or_default()
        .clone()
}

/// Short registry lookup; the lock is released before returning.
fn registered_contact_database(path: &std::path::Path) -> Option<SharedContactDatabase> {
    contact_database_registry()
        .lock()
        .get(path)
        .and_then(Weak::upgrade)
}

#[uniffi::export]
impl ContactManager {
    /// Create or open contact database at the given path
    #[uniffi::constructor]
    pub fn new(storage_path: String) -> Result<Self, crate::IronCoreError> {
        let path = PathBuf::from(storage_path).join("contacts.db");

        if let Some(existing) = registered_contact_database(&path) {
            return Ok(Self { db: existing });
        }

        // #413 review F1: the open below can sleep up to ~5 s in the lock
        // retry. The global registry mutex must NOT be held across it, so we
        // serialise on a per-path gate instead, then re-check the registry
        // (another caller may have opened this path while we waited).
        let gate = contact_open_gate(&path);
        let _open_guard = gate.lock();
        if let Some(existing) = registered_contact_database(&path) {
            return Ok(Self { db: existing });
        }

        // MESSAGE-STORE-LOCK-001 (2026-09-21): a just-stopped MeshService can
        // still hold this store's sled lock, because its UniFFI wrapper is
        // released by the GC/cleaner rather than by `stop()` -- and a
        // stop -> Start on the Pixel reaches exactly here. Open through the
        // shared, teardown-sized retry so a transient holder is not reported
        // as a broken store; a real holder still fails loud.
        let (db, open_attempt) = crate::store::backend::open_with_lock_retry(|| {
            sled::Config::default()
                .path(&path)
                .mode(sled::Mode::LowSpace)
                .use_compression(false)
                .open()
        })
        .map_err(|(attempts, err)| {
            tracing::error!(
                "ContactManager::new: sled failed to open {:?} after {} attempts: {}",
                path,
                attempts,
                err
            );
            crate::IronCoreError::StorageError
        })?;
        if open_attempt > 1 {
            tracing::warn!(
                "ContactManager::new: opened {:?} on attempt {} (previous holder still releasing)",
                path,
                open_attempt
            );
        }

        let db: SharedContactDatabase = Arc::new(Mutex::new(db));
        // Short lock: replaces any expired weak entry left by a manager that
        // was released after an app lifecycle transition.
        contact_database_registry()
            .lock()
            .insert(path, Arc::downgrade(&db));

        Ok(Self { db })
    }

    /// Add a contact to the database
    // UNIFICATION: verbose logging for nickname save — localNickname is user-defined (e.g. ChristyLove), nickname is federated
    pub fn add(&self, contact: Contact) -> Result<(), crate::IronCoreError> {
        tracing::info!(
            event = "contacts_bridge_add",
            peer_id = %contact.peer_id,
            nickname = ?contact.nickname,
            local_nickname = ?contact.local_nickname,
            public_key_prefix = %contact.public_key.chars().take(8).collect::<String>(),
            "UNIFICATION saving contact nickname (bridge)"
        );
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
    // UNIFICATION verbose logging for nickname load
    pub fn get(&self, peer_id: String) -> Result<Option<Contact>, crate::IronCoreError> {
        let db = self.db.lock();
        if let Some(data) = db
            .get(peer_id.as_bytes())
            .map_err(|_| crate::IronCoreError::StorageError)?
        {
            let contact: Contact = serde_json::from_slice(&data)
                .context("Failed to deserialize contact")
                .map_err(|_| crate::IronCoreError::Internal)?;
            tracing::debug!(
                event = "contacts_bridge_get",
                peer_id = %peer_id,
                nickname = ?contact.nickname,
                local_nickname = ?contact.local_nickname,
                "UNIFICATION loaded contact nickname"
            );
            return Ok(Some(contact));
        }
        // Fallback: the row may be filed under a different spelling of the
        // same identity (libp2p peer id vs public-key hex vs identity id).
        Ok(Self::scan_for_identifier(&db, &peer_id)?.map(|(_, contact)| contact))
    }

    /// Remove a contact by peer ID, public key, or identity ID
    pub fn remove(&self, peer_id: String) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock();
        db.remove(peer_id.as_bytes())
            .map_err(|_| crate::IronCoreError::StorageError)?;
        if let Some((key, _)) = Self::scan_for_identifier(&db, &peer_id)? {
            db.remove(key)
                .map_err(|_| crate::IronCoreError::StorageError)?;
        }
        Ok(())
    }

    /// List all contacts, sorted by display name
    // UNIFICATION verbose logging for nickname list
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
        tracing::debug!(
            event = "contacts_bridge_list",
            count = contacts.len(),
            nicknames = ?contacts.iter().map(|c| (c.peer_id.chars().take(8).collect::<String>(), c.nickname.clone(), c.local_nickname.clone())).collect::<Vec<_>>(),
            "UNIFICATION listed contacts with nicknames (bridge)"
        );
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
    // UNIFICATION: nickname is federated (peer's self-reported name), not user-defined
    pub fn set_nickname(
        &self,
        peer_id: String,
        nickname: Option<String>,
    ) -> Result<(), crate::IronCoreError> {
        tracing::info!(
            event = "contacts_bridge_set_nickname",
            peer_id = %peer_id,
            nickname = ?nickname,
            "UNIFICATION set federated nickname"
        );
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
    // UNIFICATION: local_nickname is user-defined (e.g. ChristyLove), never overwritten by ledger sync
    pub fn set_local_nickname(
        &self,
        peer_id: String,
        nickname: Option<String>,
    ) -> Result<(), crate::IronCoreError> {
        tracing::info!(
            event = "contacts_bridge_set_local_nickname",
            peer_id = %peer_id,
            local_nickname = ?nickname,
            "UNIFICATION set user-defined local nickname"
        );
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
                // SELF-CERTIFYING KEY BINDING: only bind a public key when it
                // re-derives from the peer id itself (identity multihash).
                // Never store the peer_id AS the public key -- an unverified
                // placeholder poisons identity resolution and receipt
                // encryption. Without a derivable key, keep a clearly-marked
                // placeholder record (empty key + notes annotation).
                let contact = placeholder_or_derived_contact(&msg.peer_id);
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
                // SELF-CERTIFYING KEY BINDING: see reconcile_from_history.
                // The peer_id is never stored AS the public key; without a
                // derivable key the record keeps an empty key plus a notes
                // annotation so later verified material can backfill it.
                let contact = placeholder_or_derived_contact(&msg.peer_id);
                self.add(contact)?;
                recovered += 1;
            }
        }
        Ok(recovered)
    }
}

// Internal helpers: kept out of the `#[uniffi::export]` block (associated
// functions without `self` are not exportable).
impl ContactManager {
    /// Does `contact` answer to `identifier` (peer id or public key, case
    /// insensitive, or the identity id derived from its public key)?
    fn contact_answers_to(contact: &Contact, identifier: &str) -> bool {
        if identifier.is_empty() {
            return false;
        }
        // A libp2p peer id is base58 and therefore case-SENSITIVE: two distinct
        // peer ids can differ only by case, so it must match exactly (a
        // case-insensitive match could remove the wrong contact). Only the hex
        // spellings (public key, identity id) are case-folded, and only when the
        // identifier is itself hex-shaped.
        if contact.peer_id == identifier {
            return true;
        }
        let hex_shaped = identifier.bytes().all(|b| b.is_ascii_hexdigit());
        hex_shaped
            && (contact.peer_id.eq_ignore_ascii_case(identifier)
                || contact.public_key.eq_ignore_ascii_case(identifier)
                || crate::identity::identity_id_from_public_key_hex(&contact.public_key)
                    .is_some_and(|id| id.eq_ignore_ascii_case(identifier)))
    }

    fn scan_for_identifier(
        db: &Db,
        identifier: &str,
    ) -> Result<Option<(sled::IVec, Contact)>, crate::IronCoreError> {
        let trimmed = identifier.trim();
        for item in db.iter() {
            let (key, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            if let Ok(contact) = serde_json::from_slice::<Contact>(&value) {
                if Self::contact_answers_to(&contact, trimmed) {
                    return Ok(Some((key, contact)));
                }
            }
        }
        Ok(None)
    }
}

fn current_timestamp() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Notes annotation marking a contact whose public key could not be derived
/// from its peer id. The empty `public_key` + this marker is the placeholder
/// contract: later verified material (signed envelope, user add) backfills
/// the key; the peer_id itself must never be stored as the key.
///
/// Single owner: `store::contacts::PLACEHOLDER_KEY_NOTE`. Both recovery paths
/// produce the same record shape, so they must produce the same marker.
use crate::store::contacts::PLACEHOLDER_KEY_NOTE;

/// Build a recovery contact for a history-known peer without an existing
/// record. Binds a public key ONLY when it self-certifies (re-derives the
/// peer id); otherwise stores a clearly-marked placeholder with an empty key.
fn placeholder_or_derived_contact(peer_id: &str) -> Contact {
    match crate::store::ledger_entry::public_key_hex_from_libp2p_peer_id(peer_id) {
        Some(key) => Contact::new(peer_id.to_string(), key),
        None => {
            let mut contact = Contact::new(peer_id.to_string(), String::new());
            contact.notes = Some(PLACEHOLDER_KEY_NOTE.to_string());
            tracing::warn!(
                "Contact recovery: peer ..{} has no self-certifying key binding -- \
                 storing placeholder record without a public key",
                peer_id
            );
            contact
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::self_certifying_keypair;

    /// #413 review F1: while one caller is sleeping in the lock-contention
    /// retry for path A, the global registry mutex must stay free and an open
    /// of an unrelated path B must not wait behind A's retry.
    #[test]
    fn registry_lock_is_not_held_across_open_retry() {
        let dir_a = tempfile::tempdir().unwrap();
        let dir_b = tempfile::tempdir().unwrap();
        let a = dir_a.path().to_str().unwrap().to_string();
        let b = dir_b.path().to_str().unwrap().to_string();

        // An outside holder keeps A's sled lock, forcing the retry path.
        let held = sled::open(dir_a.path().join("contacts.db")).unwrap();
        let waiter = std::thread::spawn(move || ContactManager::new(a).map(|_| ()));
        std::thread::sleep(std::time::Duration::from_millis(400));

        let started = std::time::Instant::now();
        assert!(
            contact_database_registry().try_lock().is_some(),
            "registry mutex must be free while a retry is sleeping"
        );
        ContactManager::new(b).expect("unrelated path opens promptly");
        assert!(
            started.elapsed() < std::time::Duration::from_secs(2),
            "unrelated open waited behind the retry: {:?}",
            started.elapsed()
        );

        drop(held);
        waiter
            .join()
            .expect("waiter thread")
            .expect("retry succeeds once the holder releases");
    }

    /// Two callers on the same path share one Db (the registry contract is
    /// preserved by the per-path gate + re-check).
    #[test]
    fn concurrent_opens_of_one_path_share_one_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let threads: Vec<_> = (0..4)
            .map(|_| {
                let p = path.clone();
                std::thread::spawn(move || ContactManager::new(p).expect("open"))
            })
            .collect();
        let managers: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
        for m in &managers[1..] {
            assert!(Arc::ptr_eq(&managers[0].db, &m.db));
        }
    }

    #[test]
    fn test_contact_creation() {
        let contact = Contact::new("12D3KooTest".to_string(), "abcd1234".to_string())
            .with_nickname("Alice".to_string());

        assert_eq!(contact.display_name(), "Alice");
        assert_eq!(contact.peer_id, "12D3KooTest");
    }

    #[test]
    fn contact_manager_resolves_and_removes_by_any_identity_spelling(
    ) -> Result<(), crate::IronCoreError> {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage_path = temp_dir.path().to_str().unwrap_or_default().to_string();
        let manager = ContactManager::new(storage_path)?;

        let (peer_id, key_hex) = self_certifying_keypair(b"scm-idv2-bridge");
        let identity_id = crate::identity::identity_id_from_public_key_hex(&key_hex).unwrap();
        manager
            .add(Contact::new(peer_id.clone(), key_hex.clone()).with_nickname("A".to_string()))?;

        assert!(manager.get(peer_id.clone())?.is_some());
        assert!(manager.get(key_hex.clone())?.is_some());
        assert!(manager.get(key_hex.to_uppercase())?.is_some());
        assert!(manager.get(identity_id.clone())?.is_some());
        assert!(manager.get("unrelated".to_string())?.is_none());
        assert!(manager.get(String::new())?.is_none());

        manager.remove(identity_id)?;
        assert!(manager.get(peer_id)?.is_none());
        assert!(manager.get(key_hex)?.is_none());
        Ok(())
    }

    #[test]
    fn contact_manager_rejects_non_key_identifiers_and_keeps_peer_id_case_exact(
    ) -> Result<(), crate::IronCoreError> {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage_path = temp_dir.path().to_str().unwrap_or_default().to_string();
        let manager = ContactManager::new(storage_path)?;

        let (peer_a, key_a) = self_certifying_keypair(b"scm-idv2-neg-a");
        let (peer_b, key_b) = self_certifying_keypair(b"scm-idv2-neg-b");
        manager.add(Contact::new(peer_a.clone(), key_a.clone()))?;
        manager.add(Contact::new(peer_b.clone(), key_b.clone()))?;

        // Non-key identifiers never resolve and never delete anything.
        for junk in ["not-a-key", "12D3KooWnotakey", "zz", " "] {
            assert!(manager.get(junk.to_string())?.is_none(), "{junk}");
            manager.remove(junk.to_string())?;
        }
        assert_eq!(manager.list()?.len(), 2);

        // A base58 peer id is case-sensitive: a case-flipped spelling must not
        // resolve to (or remove) the contact.
        let flipped: String = peer_a
            .chars()
            .map(|c| {
                if c.is_ascii_lowercase() {
                    c.to_ascii_uppercase()
                } else {
                    c.to_ascii_lowercase()
                }
            })
            .collect();
        assert!(manager.get(flipped.clone())?.is_none());
        manager.remove(flipped)?;
        assert!(manager.get(peer_a.clone())?.is_some());

        // The hex spelling of a different contact does not touch the first.
        manager.remove(key_b.to_uppercase())?;
        assert!(manager.get(peer_b)?.is_none());
        assert!(manager.get(peer_a)?.is_some());
        assert_eq!(manager.list()?.len(), 1);
        Ok(())
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
    fn recovery_binds_only_self_certifying_keys() {
        let (peer_id, key_hex) = self_certifying_keypair(b"scm-recover-test");

        // Self-certifying: the derived contact carries the real key.
        let derived = placeholder_or_derived_contact(&peer_id);
        assert_eq!(derived.peer_id, peer_id);
        assert_eq!(derived.public_key, key_hex);
        assert!(crate::store::ledger_entry::is_self_certifying_binding(
            &derived.peer_id,
            &derived.public_key
        ));

        // Non-derivable id: placeholder record, never peer_id-as-key.
        for bad in [
            "not-a-peer-id",
            "12D3KooWEfZ2fJ8AcGvVfEUi2wFQPo6z8kZVr5TsgP7JQF2B9kS1",
        ] {
            let placeholder = placeholder_or_derived_contact(bad);
            assert_eq!(placeholder.peer_id, bad);
            assert!(
                placeholder.public_key.is_empty(),
                "placeholder must not carry a fabricated key"
            );
            assert_ne!(placeholder.public_key, bad);
            assert_eq!(placeholder.notes.as_deref(), Some(PLACEHOLDER_KEY_NOTE));
        }
    }

    #[test]
    fn reconcile_from_history_never_poisons_public_key() -> Result<(), crate::IronCoreError> {
        use crate::mobile_bridge::HistoryManager;

        let temp_dir = tempfile::tempdir().unwrap();
        let storage_path = temp_dir.path().to_str().unwrap_or_default().to_string();

        let manager = ContactManager::new(storage_path.clone())?;
        let history = HistoryManager::new(storage_path)?;

        let (self_certifying_id, key_hex) = self_certifying_keypair(b"scm-recover-test");
        for peer in [
            self_certifying_id.clone(),
            "12D3KooWEfZ2fJ8AcGvVfEUi2wFQPo6z8kZVr5TsgP7JQF2B9kS1".to_string(),
        ] {
            history
                .add(crate::mobile_bridge::MessageRecord {
                    id: format!("msg-{}", &peer[..12]),
                    direction: crate::mobile_bridge::MessageDirection::Received,
                    peer_id: peer,
                    content: "hello".to_string(),
                    timestamp: 1,
                    sender_timestamp: 1,
                    delivered: false,
                    status: crate::mobile_bridge::MessageStatus::default(),
                    hidden: false,
                    stored_at_millis: 0,
                })
                .map_err(|_| crate::IronCoreError::StorageError)?;
        }

        manager.reconcile_from_history(&history)?;

        let good = manager.get(self_certifying_id)?.expect("derived contact");
        assert_eq!(good.public_key, key_hex);

        let placeholder = manager
            .get("12D3KooWEfZ2fJ8AcGvVfEUi2wFQPo6z8kZVr5TsgP7JQF2B9kS1".to_string())?
            .expect("placeholder contact");
        assert!(placeholder.public_key.is_empty());
        assert_ne!(
            placeholder.public_key,
            "12D3KooWEfZ2fJ8AcGvVfEUi2wFQPo6z8kZVr5TsgP7JQF2B9kS1"
        );
        Ok(())
    }
}
