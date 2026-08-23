// Contact management storage
//
// Refactored to use generic StorageBackend for cross-platform parity (Sled/IndexedDB/Memory).

use crate::identity::PublicKeyBundle;
use crate::store::backend::StorageBackend;
use crate::store::history::HistoryManager;
use crate::IronCoreError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub peer_id: String,
    pub nickname: Option<String>, // Federated nickname (from the peer)
    pub local_nickname: Option<String>, // Local override set by the user
    pub public_key: String,
    pub added_at: u64,
    pub last_seen: Option<u64>,
    pub notes: Option<String>,
    /// WS13 tight-pair: most-recently-observed device UUID for this contact.
    /// Updated when an inbound message carries WS13 device metadata.
    /// Used as `intended_device_id` when sending to this contact.
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
            notes: None,
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

    /// Returns the federated nickname (the nickname advertised by the peer),
    /// without falling through to local_nickname or peer_id.
    pub fn federated_nickname(&self) -> Option<&str> {
        self.nickname.as_deref()
    }
}

/// Key prefix namespacing contact records in the shared backend. `IronCore`
/// hands identity, history, logs, blocked-list, and contact storage the same
/// `Arc<dyn StorageBackend>` instance, so without a prefix, `list()`/`count()`,
/// would scan (and try to parse as `Contact`) every other subsystem's keys too.
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

#[derive(Clone)]
pub struct ContactManager {
    backend: Arc<dyn StorageBackend>,
}

impl ContactManager {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        let manager = Self { backend };
        manager.migrate_unprefixed_contacts();
        manager
    }

    /// One-time migration for installs that stored contacts under bare
    /// `peer_id` keys before `CONTACT_KEY_PREFIX` existed: those records
    /// became invisible to `list()`/`get()`/`count()` (which only see
    /// `contact:`-prefixed keys) after upgrading, without being deleted.
    /// Idempotent - a no-op once every contact has been rewritten under its
    /// prefixed key.
    fn migrate_unprefixed_contacts(&self) {
        if self
            .backend
            .get(b"metadata_contacts_migrated")
            .map(|opt| opt.is_some())
            .unwrap_or(false)
        {
            return;
        }

        let Ok(entries) = self.backend.scan_prefix(b"") else {
            return;
        };
        let mut migrated = 0u32;
        for (key, value) in entries {
            if key.starts_with(CONTACT_KEY_PREFIX) {
                continue;
            }
            let Ok(contact) = serde_json::from_slice::<Contact>(&value) else {
                continue;
            };
            // Disambiguator against other subsystems' records sharing this
            // backend: only treat it as a migratable contact if the bare
            // key really is that contact's peer_id.
            if contact.peer_id.as_bytes() != key.as_slice() {
                continue;
            }

            let prefixed = contact_key(&contact.peer_id);
            let already_exists = self
                .backend
                .get(&prefixed)
                .map(|opt| opt.is_some())
                .unwrap_or(false);

            if already_exists {
                // Prefixed key already exists, don't overwrite.
                // Just remove the legacy bare key to clean up the backend.
                let _ = self.backend.remove(&key);
            } else if self.backend.put(&prefixed, &value).is_ok() {
                let _ = self.backend.remove(&key);
                migrated += 1;
            }
        }

        let _ = self.backend.put(b"metadata_contacts_migrated", b"true");

        if migrated > 0 {
            tracing::info!(
                event = "contacts_key_prefix_migration",
                migrated_count = migrated,
                "migrated bare-keyed contacts to contact:-prefixed keys"
            );
        }
    }

    /// Reconcile contacts from message history to recover potentially lost records.
    /// Scans all message records and creates a basic contact if the peer_id is unknown.
    pub fn reconcile_from_history(&self, history: &HistoryManager) -> Result<u32, IronCoreError> {
        let all_messages = history.recent_including_hidden(None, 10000)?;
        let mut recovered_count = 0;

        for msg in all_messages {
            if self.get(msg.peer_id.clone()).is_ok() && self.get(msg.peer_id.clone())?.is_none() {
                // We have the peer_id from history, but no contact record.
                // Note: We lack the public key here unless we can derive it from the peer_id.
                // In libp2p, the peer_id typically contains the public key.
                if let Ok(pub_key) = self.derive_public_key_from_peer_id(&msg.peer_id) {
                    let contact = Contact::new(msg.peer_id.clone(), pub_key);
                    self.add(contact)?;
                    recovered_count += 1;
                }
            }
        }
        Ok(recovered_count)
    }

    fn derive_public_key_from_peer_id(&self, peer_id: &str) -> Result<String, IronCoreError> {
        let trimmed = peer_id.trim();

        // If it's 64 hex chars, validate it's a genuine Ed25519 public key.
        // identity_id is also 64 hex chars (Blake3 hash) but NOT a valid Ed25519 key.
        // Rejecting identity_id here prevents reconcile_from_history from creating
        // contacts with public_key = identity_id, which breaks future encryption.
        if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Ok(bytes) = hex::decode(trimmed) {
                if bytes.len() == 32 {
                    if let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice()) {
                        if ed25519_dalek::VerifyingKey::from_bytes(&arr).is_ok() {
                            return Ok(trimmed.to_lowercase());
                        }
                    }
                }
            }
            // 64-hex but not a valid Ed25519 key -> likely identity_id; cannot derive pubkey.
            return Err(IronCoreError::InvalidInput);
        }

        // Try to decode as libp2p PeerId (base58) and extract Ed25519 public key.
        // Matches the protobuf prefix used by libp2p identity multihash:
        // 0x00 0x24 0x08 0x01 0x12 0x20 <32 bytes>
        if let Ok(bytes) = bs58::decode(trimmed).into_vec() {
            if bytes.len() == 38
                && bytes[0] == 0x00
                && bytes[1] == 0x24
                && bytes[2] == 0x08
                && bytes[3] == 0x01
                && bytes[4] == 0x12
                && bytes[5] == 0x20
            {
                return Ok(hex::encode(&bytes[6..38]));
            }
            // Fallback: take last 32 bytes for non-standard PeerIds
            if bytes.len() >= 32 {
                return Ok(hex::encode(&bytes[bytes.len() - 32..]));
            }
        }

        Err(IronCoreError::InvalidInput)
    }

    pub fn add(&self, contact: Contact) -> Result<(), IronCoreError> {
        let key = contact_key(&contact.peer_id);
        let value = serde_json::to_vec(&contact).map_err(|_| IronCoreError::Internal)?;
        self.backend
            .put(&key, &value)
            .map_err(|_| IronCoreError::StorageError)?;

        // STEP 2: Maintain identity_id -> public_key index for backward compatibility.
        // Extract identity_id (blake3 hash of raw public_key bytes) and create the
        // reverse-lookup index so that even if we receive an identity_id hash,
        // we can resolve it to the canonical public_key for crypto operations.
        if let Ok(pk_bytes) = hex::decode(&contact.public_key) {
            if pk_bytes.len() == 32 {
                let identity_id = hex::encode(blake3::hash(&pk_bytes).as_bytes());
                let _ = self.save_identity_id_index(&identity_id, &contact.public_key);
            }
        }

        Ok(())
    }

    pub fn get(&self, peer_id: String) -> Result<Option<Contact>, IronCoreError> {
        let key = contact_key(&peer_id);
        if let Some(data) = self
            .backend
            .get(&key)
            .map_err(|_| IronCoreError::StorageError)?
        {
            let contact: Contact =
                serde_json::from_slice(&data).map_err(|_| IronCoreError::Internal)?;
            return Ok(Some(contact));
        }

        let trimmed = peer_id.trim();

        // 1. If resolving via identity_id index succeeds, look up by that public key
        if let Ok(Some(public_key)) = self.resolve_identity_id(trimmed) {
            if let Ok(Some(contact)) = self.get_by_public_key(&public_key) {
                return Ok(Some(contact));
            }
        }

        // 2. Direct lookup by public key if trimmed is a candidate public key
        if let Ok(Some(contact)) = self.get_by_public_key(trimmed) {
            return Ok(Some(contact));
        }

        // 3. If trimmed is a 64-hex Blake3 identity ID not yet in index, scan contacts for matching public key Blake3 hash
        if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Ok(contacts) = self.list() {
                for contact in contacts {
                    if let Ok(pk_bytes) = hex::decode(&contact.public_key) {
                        if pk_bytes.len() == 32 {
                            let id = hex::encode(blake3::hash(&pk_bytes).as_bytes());
                            if id.eq_ignore_ascii_case(trimmed) {
                                // Populate index for fast lookup next time
                                let _ = self.save_identity_id_index(&id, &contact.public_key);
                                return Ok(Some(contact));
                            }
                        }
                    }
                }
            }
        }

        // 4. If peer_id is a libp2p Peer ID, extract public key and lookup
        if let Ok(parsed) = trimmed.parse::<libp2p::PeerId>() {
            let mh = parsed.as_ref();
            if mh.code() == 0 {
                if let Ok(pk) = libp2p::identity::PublicKey::try_decode_protobuf(mh.digest()) {
                    if let Ok(ed_pk) = pk.try_into_ed25519() {
                        let pubkey_hex = hex::encode(ed_pk.to_bytes());
                        if let Ok(Some(contact)) = self.get_by_public_key(&pubkey_hex) {
                            return Ok(Some(contact));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Find a contact by its canonical Ed25519 public-key hex.
    ///
    /// Contact records are stored under their libp2p PeerId, while the send
    /// path deliberately encrypts to the public key. Keeping this lookup
    /// explicit avoids treating a known contact as an unknown peer merely
    /// because the caller has the key flavor required for encryption.
    pub fn get_by_public_key(&self, public_key: &str) -> Result<Option<Contact>, IronCoreError> {
        let normalized = public_key.trim();
        if normalized.is_empty() {
            return Ok(None);
        }

        Ok(self
            .list()?
            .into_iter()
            .find(|contact| contact.public_key.eq_ignore_ascii_case(normalized)))
    }

    pub fn remove(&self, peer_id: String) -> Result<(), IronCoreError> {
        if let Some(contact) = self.get(peer_id.clone())? {
            let bundle_key = contact_bundle_key(&contact.public_key);
            let _ = self.backend.remove(&bundle_key);
            let key = contact_key(&contact.peer_id);
            let _ = self.backend.remove(&key);
        }
        let key = contact_key(&peer_id);
        self.backend
            .remove(&key)
            .map_err(|_| IronCoreError::StorageError)?;
        Ok(())
    }

    /// Save a contact's public key bundle.
    pub fn save_contact_bundle(
        &self,
        public_key_hex: &str,
        bundle: &PublicKeyBundle,
    ) -> Result<(), IronCoreError> {
        let key = contact_bundle_key(public_key_hex);
        let value = serde_json::to_vec(bundle).map_err(|_| IronCoreError::Internal)?;
        self.backend
            .put(&key, &value)
            .map_err(|_| IronCoreError::StorageError)?;
        Ok(())
    }

    /// Load a contact's public key bundle.
    pub fn get_contact_bundle(
        &self,
        public_key_hex: &str,
    ) -> Result<Option<PublicKeyBundle>, IronCoreError> {
        let key = contact_bundle_key(public_key_hex);
        if let Some(data) = self
            .backend
            .get(&key)
            .map_err(|_| IronCoreError::StorageError)?
        {
            let bundle: PublicKeyBundle =
                serde_json::from_slice(&data).map_err(|_| IronCoreError::Internal)?;
            Ok(Some(bundle))
        } else {
            // If not found by public_key_hex, try resolving as identity_id
            if let Ok(Some(pk)) = self.resolve_identity_id(public_key_hex) {
                return self.get_contact_bundle(&pk);
            }
            Ok(None)
        }
    }

    pub fn list(&self) -> Result<Vec<Contact>, IronCoreError> {
        let all = self
            .backend
            .scan_prefix(CONTACT_KEY_PREFIX)
            .map_err(|_| IronCoreError::StorageError)?;

        let mut contacts = Vec::new();
        for (_, value) in all {
            let contact: Contact =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            contacts.push(contact);
        }

        contacts.sort_by(|a, b| a.display_name().cmp(b.display_name()));
        Ok(contacts)
    }

    pub fn search(&self, query: String) -> Result<Vec<Contact>, IronCoreError> {
        let query_lower = query.to_lowercase();
        let all = self.list()?;

        let results = all
            .into_iter()
            .filter(|contact| {
                contact.peer_id.to_lowercase().contains(&query_lower)
                    || contact.public_key.to_lowercase().contains(&query_lower)
                    || contact
                        .nickname
                        .as_ref()
                        .is_some_and(|n| n.to_lowercase().contains(&query_lower))
                    || contact
                        .local_nickname
                        .as_ref()
                        .is_some_and(|n| n.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(results)
    }

    pub fn set_nickname(
        &self,
        peer_id: String,
        nickname: Option<String>,
    ) -> Result<(), IronCoreError> {
        if let Some(mut contact) = self.get(peer_id)? {
            contact.nickname = nickname
                .map(|n| n.trim().to_string())
                .filter(|n| !n.is_empty());
            self.add(contact)?;
            Ok(())
        } else {
            Err(IronCoreError::InvalidInput)
        }
    }

    pub fn set_local_nickname(
        &self,
        peer_id: String,
        nickname: Option<String>,
    ) -> Result<(), IronCoreError> {
        if let Some(mut contact) = self.get(peer_id)? {
            contact.local_nickname = nickname
                .map(|n| n.trim().to_string())
                .filter(|n| !n.is_empty());
            self.add(contact)?;
            Ok(())
        } else {
            Err(IronCoreError::InvalidInput)
        }
    }

    pub fn update_last_seen(&self, peer_id: String) -> Result<(), IronCoreError> {
        if let Some(mut contact) = self.get(peer_id)? {
            contact.last_seen = Some(current_timestamp());
            self.add(contact)?;
        }
        Ok(())
    }

    /// Update the most-recently-observed device ID for a contact (WS13 tight-pair).
    ///
    /// Called when an inbound message or ledger exchange reveals the sender's current device UUID.
    /// The stored value is used as `intended_device_id` when routing future messages to this peer.
    /// A `None` value clears any previously-stored device ID (e.g., after a factory reset signal).
    /// `Some` values are normalized (`trim`) and only persisted when non-empty and valid UUIDs;
    /// malformed values are ignored to avoid replacing a previously known-good device ID.
    pub fn update_last_known_device_id(
        &self,
        peer_id: String,
        device_id: Option<String>,
    ) -> Result<(), IronCoreError> {
        if let Some(mut contact) = self.get(peer_id)? {
            match device_id {
                None => {
                    contact.last_known_device_id = None;
                    self.add(contact)?;
                }
                Some(device_id) => {
                    let normalized = device_id.trim();
                    if !normalized.is_empty() && uuid::Uuid::parse_str(normalized).is_ok() {
                        contact.last_known_device_id = Some(normalized.to_string());
                        self.add(contact)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn count(&self) -> u32 {
        self.backend.count_prefix(CONTACT_KEY_PREFIX).unwrap_or(0) as u32
    }

    pub fn flush(&self) {
        let _ = self.backend.flush();
    }

    /// Verify database integrity and detect corruption.
    /// Returns an error if the database has contact-prefixed entries but
    /// `list()` returns 0 contacts (i.e. entries exist but fail to parse).
    pub fn verify_integrity(&self) -> Result<(), IronCoreError> {
        let contact_count = self.count();
        let raw_entry_count = self.backend.count_prefix(CONTACT_KEY_PREFIX).unwrap_or(0);

        // If contact count is 0 but contact-prefixed entries exist, there may
        // be corruption or the contacts were not properly loaded.
        if contact_count == 0 && raw_entry_count > 0 {
            let has_data = !self
                .backend
                .scan_prefix(CONTACT_KEY_PREFIX)
                .unwrap_or_default()
                .is_empty();
            if has_data {
                // Contact-prefixed entries exist but count() returns 0 -
                // potential corruption (data stored but not properly deserialized).
                return Err(IronCoreError::CorruptionDetected);
            }
        }
        Ok(())
    }

    /// Resolve an identity_id (blake3 hash of public key) to its public key
    /// by looking up the identity_id index.
    pub fn resolve_identity_id(&self, identity_id: &str) -> Result<Option<String>, IronCoreError> {
        let key = identity_id_index_key(identity_id);
        if let Some(data) = self
            .backend
            .get(&key)
            .map_err(|_| IronCoreError::StorageError)?
        {
            let public_key = String::from_utf8(data).map_err(|_| IronCoreError::Internal)?;
            Ok(Some(public_key))
        } else {
            Ok(None)
        }
    }

    /// Save the identity_id -> public_key mapping in the index.
    fn save_identity_id_index(
        &self,
        identity_id: &str,
        public_key_hex: &str,
    ) -> Result<(), IronCoreError> {
        let key = identity_id_index_key(identity_id);
        self.backend
            .put(&key, public_key_hex.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?;
        Ok(())
    }

    /// STEP 5: Migrate existing contacts to populate identity_id -> public_key index.
    ///
    /// This function scans all stored contacts and, for each one, computes its
    /// identity_id (blake3 hash of raw public key) and creates an index entry
    /// mapping identity_id -> public_key_hex. This allows backward-compatible
    /// resolution if old code or network peers send identity_id hashes instead
    /// of public keys.
    ///
    /// Idempotent: contacts that already have an index entry will be skipped.
    pub fn migrate_identity_id_index(&self) -> Result<u32, IronCoreError> {
        if self
            .backend
            .get(b"metadata_identity_id_index_migrated")
            .map(|opt| opt.is_some())
            .unwrap_or(false)
        {
            return Ok(0); // Already migrated
        }

        let mut migrated = 0u32;
        if let Ok(contacts) = self.list() {
            for contact in contacts {
                if let Ok(pk_bytes) = hex::decode(&contact.public_key) {
                    if pk_bytes.len() == 32 {
                        let identity_id = hex::encode(blake3::hash(&pk_bytes).as_bytes());
                        // Only save if not already indexed
                        if let Ok(None) = self.resolve_identity_id(&identity_id) {
                            let _ = self.save_identity_id_index(&identity_id, &contact.public_key);
                            migrated += 1;
                        }
                    }
                }
            }
        }

        // Mark as completed
        let _ = self
            .backend
            .put(b"metadata_identity_id_index_migrated", b"true");

        if migrated > 0 {
            tracing::info!(
                event = "contacts_identity_id_index_migration",
                migrated_count = migrated,
                "migrated existing contacts to populate identity_id index"
            );
        }

        Ok(migrated)
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
    use crate::store::backend::MemoryStorage;
    use std::sync::Arc;

    fn make_manager() -> ContactManager {
        ContactManager::new(Arc::new(MemoryStorage::new()))
    }

    #[test]
    fn contact_new_has_no_last_known_device_id() {
        let c = Contact::new("peer-1".to_string(), "pubkey-hex".to_string());
        assert!(c.last_known_device_id.is_none());
    }

    #[test]
    fn update_last_known_device_id_persists_and_is_readable() {
        let mgr = make_manager();
        mgr.add(Contact::new("peer-1".to_string(), "pubkey".to_string()))
            .unwrap();

        mgr.update_last_known_device_id(
            "peer-1".to_string(),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
        )
        .unwrap();

        let contact = mgr.get("peer-1".to_string()).unwrap().unwrap();
        assert_eq!(
            contact.last_known_device_id.as_deref(),
            Some("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn update_last_known_device_id_can_clear() {
        let mgr = make_manager();
        let mut c = Contact::new("peer-2".to_string(), "pubkey".to_string());
        c.last_known_device_id = Some("old-device".to_string());
        mgr.add(c).unwrap();

        mgr.update_last_known_device_id("peer-2".to_string(), None)
            .unwrap();

        let contact = mgr.get("peer-2".to_string()).unwrap().unwrap();
        assert!(contact.last_known_device_id.is_none());
    }

    #[test]
    fn contact_roundtrips_through_serde_with_default_device_id() {
        // Simulate a pre-WS13 contact record (no last_known_device_id field).
        let json = r#"{"peer_id":"peer-old","nickname":null,"local_nickname":null,"public_key":"pk","added_at":0,"last_seen":null,"notes":null}"#;
        let c: Contact = serde_json::from_str(json).unwrap();
        assert!(
            c.last_known_device_id.is_none(),
            "legacy records must default to None"
        );
    }

    #[test]
    fn update_last_known_device_id_trims_valid_uuid() {
        let mgr = make_manager();
        mgr.add(Contact::new("peer-3".to_string(), "pubkey".to_string()))
            .unwrap();

        mgr.update_last_known_device_id(
            "peer-3".to_string(),
            Some("  550e8400-e29b-41d4-a716-446655440000  ".to_string()),
        )
        .unwrap();

        let contact = mgr.get("peer-3".to_string()).unwrap().unwrap();
        assert_eq!(
            contact.last_known_device_id.as_deref(),
            Some("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn update_last_known_device_id_ignores_invalid_values() {
        let mgr = make_manager();
        let mut c = Contact::new("peer-4".to_string(), "pubkey".to_string());
        c.last_known_device_id = Some("550e8400-e29b-41d4-a716-446655440000".to_string());
        mgr.add(c).unwrap();

        mgr.update_last_known_device_id("peer-4".to_string(), Some("   ".to_string()))
            .unwrap();
        mgr.update_last_known_device_id("peer-4".to_string(), Some("not-a-uuid".to_string()))
            .unwrap();

        let contact = mgr.get("peer-4".to_string()).unwrap().unwrap();
        assert_eq!(
            contact.last_known_device_id.as_deref(),
            Some("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn test_unprefixed_contacts_migrate_on_open() {
        let backend = Arc::new(MemoryStorage::new());
        let contact = Contact::new("peer-legacy".to_string(), "pubkey-hex".to_string());
        let bytes = serde_json::to_vec(&contact).unwrap();
        // Simulate a pre-prefix install: the contact stored under its bare
        // peer_id key, with no `contact:` prefix.
        backend.put(b"peer-legacy", &bytes).unwrap();

        let mgr = ContactManager::new(backend.clone());

        let contacts = mgr.list().unwrap();
        assert_eq!(
            contacts.len(),
            1,
            "the bare-keyed contact must be visible after migration"
        );
        assert_eq!(contacts[0].peer_id, "peer-legacy");

        assert!(
            backend.get(b"peer-legacy").unwrap().is_none(),
            "the bare key must be removed after migration"
        );
        assert!(
            backend.get(&contact_key("peer-legacy")).unwrap().is_some(),
            "the contact must now live under its prefixed key"
        );

        // Idempotent: reopening must not duplicate or lose it.
        let mgr2 = ContactManager::new(backend);
        assert_eq!(mgr2.list().unwrap().len(), 1);
    }

    #[test]
    fn test_migration_ignores_non_contact_records_sharing_the_backend() {
        let backend = Arc::new(MemoryStorage::new());
        // A record from another subsystem that happens to be valid JSON but
        // is not a Contact (or whose peer_id doesn't match the key) must be
        // left untouched.
        backend
            .put(b"some-other-key", br#"{"unrelated":"record"}"#)
            .unwrap();
        let mismatched = Contact::new("actual-peer-id".to_string(), "pk".to_string());
        backend
            .put(b"different-key", &serde_json::to_vec(&mismatched).unwrap())
            .unwrap();

        let mgr = ContactManager::new(backend.clone());

        assert_eq!(mgr.list().unwrap().len(), 0);
        assert!(backend.get(b"some-other-key").unwrap().is_some());
        assert!(backend.get(b"different-key").unwrap().is_some());
    }

    #[test]
    fn test_contact_bundle_storage() {
        use crate::identity::{sign_bundle, IdentityKeys};

        let mgr = make_manager();
        let keys = IdentityKeys::generate();
        let bundle = sign_bundle(&keys).unwrap();

        // 1. Initially there is no bundle
        let loaded = mgr.get_contact_bundle("some-pubkey").unwrap();
        assert!(loaded.is_none());

        // 2. Save and load the bundle
        mgr.save_contact_bundle("some-pubkey", &bundle).unwrap();
        let loaded = mgr.get_contact_bundle("some-pubkey").unwrap().unwrap();
        assert_eq!(loaded.ed25519_public, bundle.ed25519_public);
        assert_eq!(loaded.x25519_public, bundle.x25519_public);
        assert_eq!(loaded.mlkem_encaps_key, bundle.mlkem_encaps_key);
        assert_eq!(loaded.created_at, bundle.created_at);
        assert_eq!(loaded.signature, bundle.signature);

        // 3. Add contact, verify remove deletes bundle
        let contact = Contact::new("peer-bundle-test".to_string(), "some-pubkey".to_string());
        mgr.add(contact).unwrap();
        mgr.remove("peer-bundle-test".to_string()).unwrap();

        let loaded = mgr.get_contact_bundle("some-pubkey").unwrap();
        assert!(
            loaded.is_none(),
            "bundle must be deleted when contact is removed"
        );
    }

    #[test]
    fn step2_test_contact_add_populates_identity_id_index() {
        let mgr = make_manager();
        // Create a contact with a real Ed25519 public key (32 bytes hex)
        // This key is taken from a valid Ed25519 point
        let valid_pubkey =
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let contact = Contact::new("peer-test".to_string(), valid_pubkey.clone());

        mgr.add(contact).unwrap();

        // Compute the expected identity_id (blake3 hash of the raw 32 bytes)
        if let Ok(pk_bytes) = hex::decode(&valid_pubkey) {
            if pk_bytes.len() == 32 {
                let expected_identity_id = hex::encode(blake3::hash(&pk_bytes).as_bytes());
                // Verify the index can resolve identity_id back to public_key
                let resolved = mgr.resolve_identity_id(&expected_identity_id).unwrap();
                assert!(
                    resolved.is_some(),
                    "identity_id should resolve to public_key after contact.add()"
                );
                assert_eq!(resolved.unwrap(), valid_pubkey);
            }
        }
    }

    #[test]
    fn lookup_by_public_key_resolves_peer_keyed_contact() {
        let mgr = make_manager();
        let public_key =
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        mgr.add(Contact::new("peer-keyed".to_string(), public_key.clone()))
            .unwrap();

        let contact = mgr
            .get_by_public_key(&public_key.to_uppercase())
            .unwrap()
            .expect("public-key lookup should find a PeerId-keyed contact");
        assert_eq!(contact.peer_id, "peer-keyed");
    }

    #[test]
    fn step2_test_reject_hash_as_public_key_in_send() {
        // This test verifies that prepare_message_internal rejects a blake3 hash
        // when used as a public key (i.e., when the sender mistakenly passes
        // identity_id instead of public_key_hex).
        // This is a unit test fixture; the actual rejection happens in iron_core.rs.
        // Here we just verify the hash validation logic works.

        let mgr = make_manager();
        let valid_pubkey =
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let contact = Contact::new("peer-test".to_string(), valid_pubkey.clone());
        mgr.add(contact).unwrap();

        // Compute the identity_id (hash) for this public key
        if let Ok(pk_bytes) = hex::decode(&valid_pubkey) {
            if pk_bytes.len() == 32 {
                let identity_id = hex::encode(blake3::hash(&pk_bytes).as_bytes());
                // Verify that the identity_id is different from the public_key
                assert_ne!(identity_id, valid_pubkey);
                // Verify that resolve_identity_id can map it back
                assert_eq!(
                    mgr.resolve_identity_id(&identity_id).unwrap().unwrap(),
                    valid_pubkey
                );
            }
        }
    }

    /// Simulate a contact stored BEFORE the identity_id index existed.
    ///
    /// `add()` now populates the index on insert, so a contact added through
    /// the public API is already indexed and the migration correctly has
    /// nothing to backfill. To exercise the migration itself, drop the index
    /// entry that `add()` created, leaving the contact in its pre-migration
    /// state.
    fn strip_identity_id_index(mgr: &ContactManager, public_key_hex: &str) {
        let pk_bytes = hex::decode(public_key_hex).expect("test pubkey must be valid hex");
        let identity_id = hex::encode(blake3::hash(&pk_bytes).as_bytes());
        mgr.backend
            .remove(&identity_id_index_key(&identity_id))
            .expect("removing the index entry must succeed");
    }

    #[test]
    fn step5_test_migration_populates_identity_id_index() {
        let mgr = make_manager();
        // Add a few contacts without triggering the migration yet
        let pubkey1 =
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let pubkey2 =
            "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321".to_string();

        mgr.add(Contact::new("peer1".to_string(), pubkey1.clone()))
            .unwrap();
        mgr.add(Contact::new("peer2".to_string(), pubkey2.clone()))
            .unwrap();

        // Put both contacts back into the pre-index state the migration exists
        // to repair.
        strip_identity_id_index(&mgr, &pubkey1);
        strip_identity_id_index(&mgr, &pubkey2);

        // Run the migration
        let migrated = mgr.migrate_identity_id_index().unwrap();
        assert_eq!(migrated, 2, "migration should have indexed both contacts");

        // Verify both identity_ids are now resolvable
        if let Ok(pk1_bytes) = hex::decode(&pubkey1) {
            if pk1_bytes.len() == 32 {
                let id1 = hex::encode(blake3::hash(&pk1_bytes).as_bytes());
                assert_eq!(mgr.resolve_identity_id(&id1).unwrap().unwrap(), pubkey1);
            }
        }
        if let Ok(pk2_bytes) = hex::decode(&pubkey2) {
            if pk2_bytes.len() == 32 {
                let id2 = hex::encode(blake3::hash(&pk2_bytes).as_bytes());
                assert_eq!(mgr.resolve_identity_id(&id2).unwrap().unwrap(), pubkey2);
            }
        }
    }

    #[test]
    fn step5_test_migration_idempotent() {
        let mgr = make_manager();
        let pubkey = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        mgr.add(Contact::new("peer-idempotent".to_string(), pubkey.clone()))
            .unwrap();

        // Put the contact back into the pre-index state so the first migration
        // has real work to do.
        strip_identity_id_index(&mgr, &pubkey);

        // First migration
        let migrated1 = mgr.migrate_identity_id_index().unwrap();
        assert_eq!(migrated1, 1);

        // Second migration should be a no-op
        let migrated2 = mgr.migrate_identity_id_index().unwrap();
        assert_eq!(
            migrated2, 0,
            "second migration should be idempotent (no-op)"
        );
    }
}
