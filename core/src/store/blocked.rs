// Blocked identities and device management
//
// Implements identity blocking with device-ID pairing for multi-device blocking.
// When a peer identity is blocked, all known device IDs for that peer are also
// blocked. New device IDs discovered for a blocked peer are auto-blocked.

use crate::identity::keys::identity_id_from_public_key_hex;
use crate::store::backend::StorageBackend;
use crate::IronCoreError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

/// Storage key prefix for blocked identity entries
const BLOCKED_PREFIX: &str = "blocked:";
/// Storage key prefix for device registry entries (peer -> known device IDs)
const DEVICE_REGISTRY_PREFIX: &str = "blocked_devs:";

/// Explicit identifier provenance. Raw 64-hex strings are intentionally not
/// classified by curve-point validity: a Blake3 identity ID can happen to be
/// a valid Ed25519 point, and guessing in that case double-hashes identities.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentifierFlavor {
    #[default]
    Legacy,
    PublicKey,
    IdentityId,
}

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
    ///
    /// Blocked-only (`is_deleted = false`): messages are still received and
    /// persisted for evidentiary purposes, but filtered out of all UI queries.
    ///
    /// Blocked + Deleted (`is_deleted = true`): all existing stored messages
    /// are purged and any future incoming network payloads are dropped at the
    /// ingress layer without being persisted.
    #[serde(default)]
    pub is_deleted: bool,
    /// How the caller identified this entry. Older rows deserialize as Legacy.
    #[serde(default)]
    pub identifier_flavor: IdentifierFlavor,
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
            identifier_flavor: IdentifierFlavor::Legacy,
        }
    }

    /// Create a full relay capability (for WASM compatibility)
    /// This is a stub implementation for WASM targets where relay functionality
    /// is not available but the type is used for compatibility.
    pub fn full_relay() -> Self {
        Self::new("relay-stub".to_string())
    }

    /// Block a specific device of this identity
    pub fn with_device_id(mut self, device_id: String) -> Self {
        self.device_id = Some(device_id);
        self
    }

    /// Add a reason for blocking
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    /// Get a storage key for this block
    fn storage_key(&self) -> String {
        match &self.device_id {
            Some(device_id) => format!("{}{}:{}", BLOCKED_PREFIX, self.peer_id, device_id),
            None => format!("{}{}", BLOCKED_PREFIX, self.peer_id),
        }
    }
}

/// Manager for blocked identities with device-ID pairing support.
///
/// When a peer identity is blocked, all known device IDs for that peer are
/// also blocked via device-specific entries. New device IDs registered for a
/// blocked peer are automatically blocked.
#[derive(Clone)]
pub struct BlockedManager {
    backend: Arc<dyn StorageBackend>,
}

impl BlockedManager {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    /// Return the identifier flavors that can refer to the same peer.
    ///
    /// A valid Ed25519 public key has a derived identity_id flavor. An
    /// identity_id cannot be reversed into a public key, so identity_id input
    /// yields only itself. An inline Ed25519 libp2p PeerId is expanded to its
    /// public-key and identity_id aliases so transport ingress enforces blocks
    /// created by the UI under either canonical identifier.
    pub fn resolved_identifiers(&self, peer_id: &str) -> Vec<String> {
        let (raw_peer_id, flavor) = split_identifier_flavor(peer_id);
        let mut identifiers = vec![raw_peer_id.to_string()];
        if flavor != IdentifierFlavor::Legacy {
            push_unique(&mut identifiers, peer_id.to_string());
        }

        match flavor {
            IdentifierFlavor::PublicKey => {
                if let Some(derived_id) = identity_id_from_public_key_hex(raw_peer_id) {
                    push_unique(&mut identifiers, derived_id);
                }
            }
            IdentifierFlavor::Legacy => {
                if let Some(public_key) = public_key_hex_from_peer_id(raw_peer_id) {
                    // Current rows use the unprefixed raw value after
                    // split_identifier_flavor(); the prefixed alias keeps
                    // pre-canonicalization rows enforceable until migration.
                    push_unique(&mut identifiers, public_key.clone());
                    push_unique(&mut identifiers, format!("pk:{public_key}"));
                    if let Some(derived_id) = identity_id_from_public_key_hex(&public_key) {
                        push_unique(&mut identifiers, derived_id.clone());
                        push_unique(&mut identifiers, format!("id:{derived_id}"));
                    }
                }
            }
            IdentifierFlavor::IdentityId => {}
        }
        identifiers
    }

    fn write_block_entry(&self, blocked: &BlockedIdentity) -> Result<(), IronCoreError> {
        let key = blocked.storage_key();
        let value = serde_json::to_vec(blocked).map_err(|_| IronCoreError::Internal)?;
        self.backend
            .put(key.as_bytes(), &value)
            .map_err(|_| IronCoreError::StorageError)
    }

    fn remove_block_entry(
        &self,
        peer_id: &str,
        device_id: Option<&str>,
    ) -> Result<(), IronCoreError> {
        let key = match device_id {
            Some(device_id) => format!("{}{}:{}", BLOCKED_PREFIX, peer_id, device_id),
            None => format!("{}{}", BLOCKED_PREFIX, peer_id),
        };
        self.backend
            .remove(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)
    }

    fn is_blocked_exact(
        &self,
        peer_id: &str,
        device_id: Option<&str>,
    ) -> Result<bool, IronCoreError> {
        if let Some(device_id) = device_id {
            let key = format!("{}{}:{}", BLOCKED_PREFIX, peer_id, device_id);
            if self
                .backend
                .get(key.as_bytes())
                .map_err(|_| IronCoreError::StorageError)?
                .is_some()
            {
                return Ok(true);
            }
        }

        let key = format!("{}{}", BLOCKED_PREFIX, peer_id);
        Ok(self
            .backend
            .get(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?
            .is_some())
    }

    fn get_exact(
        &self,
        peer_id: &str,
        device_id: Option<&str>,
    ) -> Result<Option<BlockedIdentity>, IronCoreError> {
        let key = match device_id {
            Some(device_id) => format!("{}{}:{}", BLOCKED_PREFIX, peer_id, device_id),
            None => format!("{}{}", BLOCKED_PREFIX, peer_id),
        };

        if let Some(data) = self
            .backend
            .get(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?
        {
            let blocked: BlockedIdentity =
                serde_json::from_slice(&data).map_err(|_| IronCoreError::Internal)?;
            Ok(Some(blocked))
        } else {
            Ok(None)
        }
    }

    fn is_blocked_and_deleted_exact(&self, peer_id: &str) -> Result<bool, IronCoreError> {
        let key = format!("{}{}", BLOCKED_PREFIX, peer_id);
        if let Some(data) = self
            .backend
            .get(key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?
        {
            let blocked: BlockedIdentity =
                serde_json::from_slice(&data).map_err(|_| IronCoreError::Internal)?;
            Ok(blocked.is_deleted)
        } else {
            Ok(false)
        }
    }

    // -----------------------------------------------------------------------
    // Core blocking operations
    // -----------------------------------------------------------------------

    /// Block a peer ID, also blocking all known device IDs for that peer.
    ///
    /// If the `BlockedIdentity` has no `device_id`, this creates a peer-level
    /// block AND individual device-specific blocks for every device ID in the
    /// device registry for this peer. If a `device_id` is set, only that
    /// specific device is blocked.
    pub fn block(&self, blocked: BlockedIdentity) -> Result<(), IronCoreError> {
        let (raw_peer_id, flavor) = split_identifier_flavor(&blocked.peer_id);
        let blocked = BlockedIdentity {
            peer_id: raw_peer_id.to_string(),
            identifier_flavor: flavor,
            ..blocked
        };

        // Resolve the canonical alias before the first write. For a public-key
        // request, persist the identity_id row first so an interrupted
        // dual-flavor write fails closed: ingress can still match the
        // canonical block even if the legacy/public-key mirror is the write
        // that fails. There is no transactional batch primitive in the
        // StorageBackend, so ordering is the fail-safe we can guarantee here.
        let derived_id = if flavor == IdentifierFlavor::PublicKey {
            Some(
                identity_id_from_public_key_hex(&blocked.peer_id)
                    .ok_or(IronCoreError::InvalidInput)?,
            )
        } else {
            None
        };

        let devices = if blocked.device_id.is_none() {
            self.get_known_devices(&blocked.peer_id)?
        } else {
            Vec::new()
        };

        if let Some(derived_id) = derived_id.as_ref() {
            let canonical_blocked = BlockedIdentity {
                peer_id: derived_id.clone(),
                device_id: blocked.device_id.clone(),
                blocked_at: blocked.blocked_at,
                reason: blocked.reason.clone(),
                notes: blocked.notes.clone(),
                is_deleted: blocked.is_deleted,
                identifier_flavor: IdentifierFlavor::IdentityId,
            };
            self.write_block_entry(&canonical_blocked)?;
        }

        self.write_block_entry(&blocked)?;

        // Peer-level block: also block every known device for this peer
        if blocked.device_id.is_none() {
            for device_id in &devices {
                let device_blocked = BlockedIdentity {
                    peer_id: blocked.peer_id.clone(),
                    device_id: Some(device_id.clone()),
                    blocked_at: blocked.blocked_at,
                    reason: blocked.reason.clone(),
                    notes: blocked.notes.clone(),
                    is_deleted: blocked.is_deleted,
                    identifier_flavor: flavor,
                };
                self.write_block_entry(&device_blocked)?;
            }

            if let Some(derived_id) = derived_id.as_ref() {
                // Mirror every physical block entry under the alternate
                // identifier flavor. This also covers known devices.
                for device_id in &devices {
                    let alias_dev = BlockedIdentity {
                        peer_id: derived_id.clone(),
                        device_id: Some(device_id.clone()),
                        blocked_at: blocked.blocked_at,
                        reason: blocked.reason.clone(),
                        notes: blocked.notes.clone(),
                        is_deleted: blocked.is_deleted,
                        identifier_flavor: IdentifierFlavor::IdentityId,
                    };
                    self.write_block_entry(&alias_dev)?;
                }
            }
        }

        Ok(())
    }

    /// Block a peer AND mark them as deleted (cascade purge variant).
    ///
    /// This sets `is_deleted = true` on the block record so that the ingress
    /// layer knows to drop all future payloads without persisting them, and so
    /// that the caller can trigger a cascade purge of existing stored messages.
    /// Also blocks all known device IDs for the peer.
    pub fn block_and_delete(
        &self,
        peer_id: String,
        reason: Option<String>,
    ) -> Result<(), IronCoreError> {
        let mut blocked = BlockedIdentity::new(peer_id);
        blocked.is_deleted = true;
        if let Some(r) = reason {
            blocked.reason = Some(r);
        }
        self.block(blocked)
    }

    /// Unblock a peer ID.
    ///
    /// If `device_id` is `None`, removes both the peer-level block AND all
    /// device-specific blocks for that peer, and clears the device registry.
    /// If `device_id` is `Some(...)`, only removes that specific device block.
    pub fn unblock(&self, peer_id: String, device_id: Option<String>) -> Result<(), IronCoreError> {
        let identifiers = self.identifiers_for_unblock(&peer_id)?;

        match device_id {
            Some(did) => {
                for identifier in &identifiers {
                    self.remove_block_entry(identifier, Some(&did))?;
                    self.unregister_device_from_registry(identifier, &did)?;
                }
            }
            None => {
                let mut devices = HashSet::new();
                for identifier in &identifiers {
                    devices.extend(self.get_known_devices(identifier)?);
                }

                for identifier in &identifiers {
                    self.remove_block_entry(identifier, None)?;
                    for did in &devices {
                        self.remove_block_entry(identifier, Some(did))?;
                    }
                    let reg_key = format!("{}{}", DEVICE_REGISTRY_PREFIX, identifier);
                    self.backend
                        .remove(reg_key.as_bytes())
                        .map_err(|_| IronCoreError::StorageError)?;
                }
            }
        }
        Ok(())
    }

    /// Resolve both identifier flavors for an unblock request.
    ///
    /// The forward public-key -> identity_id mapping is derivable, but the
    /// reverse mapping is only available when a public-key block or registry
    /// row is still present. Legacy identity_id-only rows remain identity-only.
    fn identifiers_for_unblock(&self, peer_id: &str) -> Result<Vec<String>, IronCoreError> {
        let mut identifiers = self.resolved_identifiers(peer_id);
        let (raw_peer_id, _) = split_identifier_flavor(peer_id);

        let blocked_entries = self
            .backend
            .scan_prefix(BLOCKED_PREFIX.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?;
        for (_, value) in blocked_entries {
            let Ok(blocked) = serde_json::from_slice::<BlockedIdentity>(&value) else {
                tracing::warn!("Skipping malformed blocked row while resolving unblock");
                continue;
            };
            if blocked.identifier_flavor == IdentifierFlavor::PublicKey
                && identity_id_from_public_key_hex(&blocked.peer_id).as_deref() == Some(raw_peer_id)
                && !identifiers
                    .iter()
                    .any(|identifier| identifier == &blocked.peer_id)
            {
                identifiers.push(blocked.peer_id);
            }
        }

        let registries = self
            .backend
            .scan_prefix(DEVICE_REGISTRY_PREFIX.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?;
        for (key, _) in registries {
            let Some(public_key) = key
                .strip_prefix(DEVICE_REGISTRY_PREFIX.as_bytes())
                .and_then(|key| std::str::from_utf8(key).ok())
            else {
                continue;
            };
            if identity_id_from_public_key_hex(public_key).as_deref() == Some(raw_peer_id)
                && !identifiers
                    .iter()
                    .any(|identifier| identifier == public_key)
            {
                identifiers.push(public_key.to_string());
            }
        }

        Ok(identifiers)
    }

    /// Check if a peer ID is blocked.
    ///
    /// If `device_id` is provided, checks for a device-specific block first,
    /// then falls back to the peer-level block. A peer-level block covers all
    /// devices of that peer.
    pub fn is_blocked(
        &self,
        peer_id: &str,
        device_id: Option<&str>,
    ) -> Result<bool, IronCoreError> {
        self.is_blocked_resolved(peer_id, device_id)
    }

    /// Check a peer across all identifier flavors known to this manager.
    ///
    /// This is the single block-resolution policy used by ingress and bridge
    /// callers. It closes the mixed-fleet gap where a public-key block must
    /// match an identity_id-valued inbound sender, and also handles legacy
    /// identity_id-only rows.
    pub fn is_blocked_resolved(
        &self,
        peer_id: &str,
        device_id: Option<&str>,
    ) -> Result<bool, IronCoreError> {
        for identifier in self.resolved_identifiers(peer_id) {
            if self.is_blocked_exact(&identifier, device_id)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Get blocked identity details
    pub fn get(
        &self,
        peer_id: &str,
        device_id: Option<&str>,
    ) -> Result<Option<BlockedIdentity>, IronCoreError> {
        for identifier in self.resolved_identifiers(peer_id) {
            if let Some(blocked) = self.get_exact(&identifier, device_id)? {
                return Ok(Some(blocked));
            }
        }
        Ok(None)
    }

    /// List all blocked identities (peer-level and device-specific).
    ///
    /// When dual-flavor block entries exist (a public-key entry and its derived
    /// identity_id alias), they are deduplicated so only one entry surfaces per
    /// logical block. The canonical identity_id flavor is preferred.
    pub fn list(&self) -> Result<Vec<BlockedIdentity>, IronCoreError> {
        let all = self
            .backend
            .scan_prefix(BLOCKED_PREFIX.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?;

        let mut blocked_list = Vec::new();
        for (_, value) in all {
            let blocked: BlockedIdentity =
                serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
            blocked_list.push(blocked);
        }

        // Deduplicate dual-flavor entries: for every peer_id that is a valid
        // public key, compute its derived identity_id. If an entry keyed by
        // the derived identity_id also exists, drop the public-key-flavored
        // entry in favor of the canonical identity_id one.
        let identity_ids: HashSet<String> =
            blocked_list.iter().map(|b| b.peer_id.clone()).collect();

        blocked_list.retain(|b| {
            if b.identifier_flavor == IdentifierFlavor::PublicKey {
                if let Some(derived) = identity_id_from_public_key_hex(&b.peer_id) {
                    if derived != b.peer_id && identity_ids.contains(&derived) {
                        return false; // Duplicate -- identity_id entry wins
                    }
                }
            }
            true
        });

        blocked_list.sort_by_key(|b| std::cmp::Reverse(b.blocked_at));
        Ok(blocked_list)
    }

    /// Get count of blocked identities
    pub fn count(&self) -> Result<usize, IronCoreError> {
        Ok(self.list()?.len())
    }

    /// Check if a peer is both blocked AND deleted (cascade purge state).
    ///
    /// Checks the peer-level block. Returns `true` only when `is_deleted = true`
    /// on the peer-level block record.
    pub fn is_blocked_and_deleted(&self, peer_id: &str) -> Result<bool, IronCoreError> {
        self.is_blocked_and_deleted_resolved(peer_id)
    }

    /// Check deleted state across all identifier flavors.
    pub fn is_blocked_and_deleted_resolved(&self, peer_id: &str) -> Result<bool, IronCoreError> {
        for identifier in self.resolved_identifiers(peer_id) {
            if self.is_blocked_and_deleted_exact(&identifier)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Return the peer IDs of all blocked-only (not deleted) identities.
    ///
    /// Used by the query layer to filter messages from blocked peers out of UI
    /// results without purging them (evidentiary retention). Includes both
    /// peer-level and device-specific blocked entries.
    pub fn blocked_only_peer_ids(
        &self,
    ) -> Result<std::collections::HashSet<String>, IronCoreError> {
        let list = self.list()?;
        Ok(list
            .into_iter()
            .filter(|b| !b.is_deleted)
            .map(|b| b.peer_id)
            .collect())
    }

    // -----------------------------------------------------------------------
    // Device-ID pairing and registry
    // -----------------------------------------------------------------------

    /// Register a device ID as belonging to a peer identity.
    ///
    /// If the peer is currently blocked (either peer-level or device-specific),
    /// the newly registered device is automatically blocked as well.
    /// This enables multi-device blocking: when a new device is discovered for
    /// an already-blocked peer, that device is immediately blocked too.
    pub fn register_device_id(
        &self,
        peer_id: &str,
        device_id: &str,
    ) -> Result<bool, IronCoreError> {
        let (raw_peer_id, flavor) = split_identifier_flavor(peer_id);
        let mut devices = self.get_known_devices(raw_peer_id)?;
        if devices.contains(&device_id.to_string()) {
            return Ok(false); // Already registered
        }
        devices.push(device_id.to_string());
        let reg_key = format!("{}{}", DEVICE_REGISTRY_PREFIX, raw_peer_id);
        let encoded = serde_json::to_vec(&devices).map_err(|_| IronCoreError::Internal)?;
        self.backend
            .put(reg_key.as_bytes(), &encoded)
            .map_err(|_| IronCoreError::StorageError)?;

        // If the peer is blocked, auto-block this device
        if self.is_blocked_resolved(raw_peer_id, None)? {
            let peer_block = self.get(raw_peer_id, None)?;
            let blocked = BlockedIdentity {
                peer_id: raw_peer_id.to_string(),
                device_id: Some(device_id.to_string()),
                blocked_at: peer_block
                    .as_ref()
                    .map(|b| b.blocked_at)
                    .unwrap_or_else(current_timestamp),
                reason: peer_block.as_ref().and_then(|b| b.reason.clone()),
                notes: None,
                is_deleted: peer_block.as_ref().map(|b| b.is_deleted).unwrap_or(false),
                identifier_flavor: flavor,
            };
            self.block(blocked)?;
        }

        Ok(true) // Newly registered
    }

    /// Get all known device IDs registered for a peer.
    pub fn get_known_devices(&self, peer_id: &str) -> Result<Vec<String>, IronCoreError> {
        let (raw_peer_id, _) = split_identifier_flavor(peer_id);
        let reg_key = format!("{}{}", DEVICE_REGISTRY_PREFIX, raw_peer_id);
        if let Some(data) = self
            .backend
            .get(reg_key.as_bytes())
            .map_err(|_| IronCoreError::StorageError)?
        {
            let devices: Vec<String> =
                serde_json::from_slice(&data).map_err(|_| IronCoreError::Internal)?;
            Ok(devices)
        } else {
            Ok(Vec::new())
        }
    }

    /// Remove a device ID from the registry for a peer.
    fn unregister_device_from_registry(
        &self,
        peer_id: &str,
        device_id: &str,
    ) -> Result<(), IronCoreError> {
        let (raw_peer_id, _) = split_identifier_flavor(peer_id);
        let mut devices = self.get_known_devices(raw_peer_id)?;
        devices.retain(|d| d != device_id);
        let reg_key = format!("{}{}", DEVICE_REGISTRY_PREFIX, raw_peer_id);
        if devices.is_empty() {
            self.backend
                .remove(reg_key.as_bytes())
                .map_err(|_| IronCoreError::StorageError)?;
        } else {
            let encoded = serde_json::to_vec(&devices).map_err(|_| IronCoreError::Internal)?;
            self.backend
                .put(reg_key.as_bytes(), &encoded)
                .map_err(|_| IronCoreError::StorageError)?;
        }
        Ok(())
    }

    /// Register multiple device IDs for a peer at once.
    /// Returns the number of newly registered device IDs.
    pub fn register_device_ids(
        &self,
        peer_id: &str,
        device_ids: &[String],
    ) -> Result<usize, IronCoreError> {
        let mut registered = 0;
        for device_id in device_ids {
            if self.register_device_id(peer_id, device_id)? {
                registered += 1;
            }
        }
        Ok(registered)
    }

    /// Check if a specific device of a peer is blocked.
    /// This checks both the device-specific block and the peer-level block.
    pub fn is_device_blocked(&self, peer_id: &str, device_id: &str) -> Result<bool, IronCoreError> {
        self.is_blocked(peer_id, Some(device_id))
    }
}

fn current_timestamp() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn split_identifier_flavor(peer_id: &str) -> (&str, IdentifierFlavor) {
    if let Some(raw) = peer_id.strip_prefix(crate::identity::keys::PUBLIC_KEY_PREFIX) {
        (raw, IdentifierFlavor::PublicKey)
    } else if let Some(raw) = peer_id.strip_prefix(crate::identity::keys::IDENTITY_ID_PREFIX) {
        (raw, IdentifierFlavor::IdentityId)
    } else {
        (peer_id, IdentifierFlavor::Legacy)
    }
}

fn push_unique(identifiers: &mut Vec<String>, identifier: String) {
    if !identifiers.iter().any(|existing| existing == &identifier) {
        identifiers.push(identifier);
    }
}

/// Extract an inline Ed25519 public key from a libp2p PeerId. Hashed PeerIds
/// intentionally return `None`: their public key is not recoverable from the
/// PeerId and must be resolved from an authenticated contact record instead.
fn public_key_hex_from_peer_id(peer_id: &str) -> Option<String> {
    let peer_id = peer_id.parse::<libp2p::PeerId>().ok()?;
    let multihash = peer_id.as_ref();
    if multihash.code() != 0 {
        return None;
    }
    let public_key = libp2p::identity::PublicKey::try_decode_protobuf(multihash.digest()).ok()?;
    let ed25519 = public_key.try_into_ed25519().ok()?;
    Some(hex::encode(ed25519.to_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::backend::MemoryStorage;

    #[test]
    fn test_block_unblock() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "12D3KooWTest123";

        // Initially not blocked
        assert!(!manager.is_blocked(peer_id, None).unwrap());

        // Block the peer
        let blocked = BlockedIdentity::new(peer_id.to_string()).with_reason("Spam".to_string());
        manager.block(blocked).unwrap();

        // Now blocked
        assert!(manager.is_blocked(peer_id, None).unwrap());

        // Unblock
        manager.unblock(peer_id.to_string(), None).unwrap();
        assert!(!manager.is_blocked(peer_id, None).unwrap());
    }

    #[test]
    fn test_device_specific_block() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "12D3KooWTest123";
        let device_id = "device-abc-123";

        // Block specific device
        let blocked =
            BlockedIdentity::new(peer_id.to_string()).with_device_id(device_id.to_string());
        manager.block(blocked).unwrap();

        // Device-specific check
        assert!(manager.is_blocked(peer_id, Some(device_id)).unwrap());
        // Peer without device not blocked
        assert!(!manager.is_blocked(peer_id, None).unwrap());
    }

    #[test]
    fn test_list_blocked() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        manager
            .block(BlockedIdentity::new("peer1".to_string()))
            .unwrap();
        manager
            .block(BlockedIdentity::new("peer2".to_string()))
            .unwrap();

        let list = manager.list().unwrap();
        assert!(list.len() >= 2);
    }

    // -----------------------------------------------------------------------
    // Multi-device blocking tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_block_peer_auto_blocks_registered_devices() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-multi-dev";
        let device_a = "device-a";
        let device_b = "device-b";

        // Register devices before blocking
        manager.register_device_id(peer_id, device_a).unwrap();
        manager.register_device_id(peer_id, device_b).unwrap();

        // Block the peer (no device_id = peer-level block)
        manager
            .block(BlockedIdentity::new(peer_id.to_string()))
            .unwrap();

        // Peer-level block
        assert!(manager.is_blocked(peer_id, None).unwrap());
        // Device-specific blocks auto-created
        assert!(manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(manager.is_blocked(peer_id, Some(device_b)).unwrap());
    }

    #[test]
    fn test_register_device_auto_blocks_if_peer_blocked() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-auto-block";
        let device_first = "device-first";
        let device_later = "device-later";

        // Block the peer first
        manager
            .block(BlockedIdentity::new(peer_id.to_string()))
            .unwrap();

        // Register first device (should auto-block)
        manager.register_device_id(peer_id, device_first).unwrap();
        assert!(manager.is_blocked(peer_id, Some(device_first)).unwrap());

        // Register another device later (should also auto-block)
        manager.register_device_id(peer_id, device_later).unwrap();
        assert!(manager.is_blocked(peer_id, Some(device_later)).unwrap());
    }

    #[test]
    fn test_register_device_no_auto_block_if_peer_not_blocked() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-not-blocked";
        let device_id = "device-x";

        // Register device without blocking the peer
        manager.register_device_id(peer_id, device_id).unwrap();

        // Device is in registry but NOT blocked
        assert!(!manager.is_blocked(peer_id, Some(device_id)).unwrap());
        assert!(!manager.is_blocked(peer_id, None).unwrap());
    }

    #[test]
    fn test_register_device_idempotent() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-idem";
        let device_id = "device-idem";

        let first = manager.register_device_id(peer_id, device_id).unwrap();
        let second = manager.register_device_id(peer_id, device_id).unwrap();

        assert!(first); // Newly registered
        assert!(!second); // Already registered
    }

    #[test]
    fn test_unblock_peer_removes_all_device_blocks() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-unblock-all";
        let device_a = "device-a";
        let device_b = "device-b";

        // Register devices and block peer
        manager.register_device_id(peer_id, device_a).unwrap();
        manager.register_device_id(peer_id, device_b).unwrap();
        manager
            .block(BlockedIdentity::new(peer_id.to_string()))
            .unwrap();

        // Verify blocked
        assert!(manager.is_blocked(peer_id, None).unwrap());
        assert!(manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(manager.is_blocked(peer_id, Some(device_b)).unwrap());

        // Unblock the peer (no device_id = full unblock)
        manager.unblock(peer_id.to_string(), None).unwrap();

        // Everything unblocked
        assert!(!manager.is_blocked(peer_id, None).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(device_b)).unwrap());

        // Device registry cleared
        assert!(manager.get_known_devices(peer_id).unwrap().is_empty());
    }

    #[test]
    fn test_unblock_single_device_when_no_peer_block() {
        // When only a device-specific block exists (no peer-level block),
        // unblocking that device should make the device unblocked.
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-dev-only-unblock";
        let device_a = "device-a";
        let device_b = "device-b";

        // Register devices
        manager.register_device_id(peer_id, device_a).unwrap();
        manager.register_device_id(peer_id, device_b).unwrap();

        // Block only device_a (not the whole peer)
        manager
            .block(BlockedIdentity::new(peer_id.to_string()).with_device_id(device_a.to_string()))
            .unwrap();

        // device_a is blocked, device_b and peer are NOT blocked
        assert!(manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(device_b)).unwrap());
        assert!(!manager.is_blocked(peer_id, None).unwrap());

        // Unblock device_a
        manager
            .unblock(peer_id.to_string(), Some(device_a.to_string()))
            .unwrap();

        // Now device_a is not blocked either
        assert!(!manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(!manager.is_blocked(peer_id, None).unwrap());

        // device_b was never blocked
        assert!(!manager.is_blocked(peer_id, Some(device_b)).unwrap());
    }

    #[test]
    fn test_peer_level_block_overrides_device_unblock() {
        // When a peer-level block exists, unblocking a specific device
        // does NOT make the device accessible because the peer-level block
        // covers all devices. To selectively unblock a device, the caller
        // must first unblock the peer entirely, then block individual devices.
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-full-block";
        let device_a = "device-a";
        let device_b = "device-b";

        manager.register_device_id(peer_id, device_a).unwrap();
        manager.register_device_id(peer_id, device_b).unwrap();
        manager
            .block(BlockedIdentity::new(peer_id.to_string()))
            .unwrap();

        // Attempt to unblock just device_a while peer-level block exists
        manager
            .unblock(peer_id.to_string(), Some(device_a.to_string()))
            .unwrap();

        // Peer-level block still active: all devices remain blocked
        assert!(manager.is_blocked(peer_id, None).unwrap());
        assert!(manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(manager.is_blocked(peer_id, Some(device_b)).unwrap());
    }

    #[test]
    fn test_block_and_delete_propagates_to_devices() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-del";
        let device_id = "device-del";

        manager.register_device_id(peer_id, device_id).unwrap();
        manager
            .block_and_delete(peer_id.to_string(), Some("harassment".to_string()))
            .unwrap();

        assert!(manager.is_blocked_and_deleted(peer_id).unwrap());
        assert!(manager.is_blocked(peer_id, Some(device_id)).unwrap());
    }

    #[test]
    fn test_blocked_only_peer_ids_includes_devices() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        manager
            .block(BlockedIdentity::new("peer-1".to_string()))
            .unwrap();
        manager
            .block(BlockedIdentity::new("peer-2".to_string()))
            .unwrap();

        let peer_ids = manager.blocked_only_peer_ids().unwrap();
        assert!(peer_ids.contains("peer-1"));
        assert!(peer_ids.contains("peer-2"));
    }

    #[test]
    fn test_multi_device_blocking_scenario() {
        // Simulate a real scenario: user blocks a contact who has 3 devices
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "12D3KooWMultiDevice";
        let phone = "device-phone";
        let laptop = "device-laptop";
        let tablet = "device-tablet";

        // Step 1: Register known devices for the contact
        manager.register_device_id(peer_id, phone).unwrap();
        manager.register_device_id(peer_id, laptop).unwrap();
        // tablet not registered yet (will be discovered later)

        // Step 2: User blocks the contact
        manager
            .block(BlockedIdentity::new(peer_id.to_string()).with_reason("spam".to_string()))
            .unwrap();

        // All known devices are blocked
        assert!(manager.is_blocked(peer_id, None).unwrap());
        assert!(manager.is_blocked(peer_id, Some(phone)).unwrap());
        assert!(manager.is_blocked(peer_id, Some(laptop)).unwrap());

        // Step 3: New device discovered later - auto-blocked
        manager.register_device_id(peer_id, tablet).unwrap();
        assert!(manager.is_blocked(peer_id, Some(tablet)).unwrap());

        // Step 4: User unblocks the contact - all devices cleared
        manager.unblock(peer_id.to_string(), None).unwrap();
        assert!(!manager.is_blocked(peer_id, None).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(phone)).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(laptop)).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(tablet)).unwrap());
    }

    #[test]
    fn test_device_specific_block_does_not_block_other_devices() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-dev-only";
        let device_a = "device-a";
        let device_b = "device-b";

        manager.register_device_id(peer_id, device_a).unwrap();
        manager.register_device_id(peer_id, device_b).unwrap();

        // Block only device_a (not the whole peer)
        manager
            .block(BlockedIdentity::new(peer_id.to_string()).with_device_id(device_a.to_string()))
            .unwrap();

        // device_a is blocked, device_b and peer are NOT blocked
        assert!(manager.is_blocked(peer_id, Some(device_a)).unwrap());
        assert!(!manager.is_blocked(peer_id, Some(device_b)).unwrap());
        assert!(!manager.is_blocked(peer_id, None).unwrap());
    }

    #[test]
    fn test_get_known_devices() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-devices";
        assert!(manager.get_known_devices(peer_id).unwrap().is_empty());

        manager.register_device_id(peer_id, "d1").unwrap();
        manager.register_device_id(peer_id, "d2").unwrap();

        let devices = manager.get_known_devices(peer_id).unwrap();
        assert_eq!(devices.len(), 2);
        assert!(devices.contains(&"d1".to_string()));
        assert!(devices.contains(&"d2".to_string()));
    }

    #[test]
    fn test_is_device_blocked() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let peer_id = "peer-dib";
        manager.register_device_id(peer_id, "d1").unwrap();
        manager
            .block(BlockedIdentity::new(peer_id.to_string()))
            .unwrap();

        assert!(manager.is_device_blocked(peer_id, "d1").unwrap());
        // Unknown device also blocked by peer-level block
        assert!(manager.is_device_blocked(peer_id, "unknown-dev").unwrap());
    }

    // -----------------------------------------------------------------------
    // Dual-flavor block storage tests (T1 mixed-fleet block bypass)
    // -----------------------------------------------------------------------

    /// Helper: generate a real Ed25519 key pair and return (public_key_hex, identity_id)
    fn generate_test_keypair() -> (String, String) {
        use crate::identity::keys::{identity_id_from_public_key_hex, KeyPair};
        let kp = KeyPair::generate();
        let pk_hex = hex::encode(kp.verifying_key().as_bytes());
        let id = identity_id_from_public_key_hex(&pk_hex)
            .expect("valid keypair must derive identity_id");
        (pk_hex, id)
    }

    #[test]
    fn block_under_public_key_matches_inbound_identity_id() {
        // The P1 regression: block by public key, then is_blocked(derived_identity_id) == true.
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let (pk_hex, derived_id) = generate_test_keypair();

        // Block under the public key
        manager
            .block(BlockedIdentity::new(format!("pk:{pk_hex}")).with_reason("spam".to_string()))
            .unwrap();

        // The derived identity_id must also be blocked (dual-flavor write)
        assert!(
            manager.is_blocked(&derived_id, None).unwrap(),
            "block under public key must also block the derived identity_id"
        );
        // Original public key is also blocked
        assert!(manager.is_blocked(&pk_hex, None).unwrap());
    }

    #[test]
    fn block_under_identity_id_matches_inbound_public_key() {
        // Block by identity_id (not a valid curve point), verify is_blocked works
        // for the identity_id itself. The public key direction is already covered
        // by the ingress candidate derivation, so no alias is written.
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let (_pk_hex, derived_id) = generate_test_keypair();

        // Block under the identity_id directly
        manager
            .block(BlockedIdentity::new(derived_id.clone()))
            .unwrap();

        // identity_id is blocked
        assert!(manager.is_blocked(&derived_id, None).unwrap());
        // The public key is NOT stored (no reverse alias possible), but the
        // ingress gate derives identity_id from inbound pk and will match.
    }

    #[test]
    fn inline_peer_id_matches_ui_public_key_block() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);
        let keypair = crate::identity::keys::KeyPair::generate();
        let public_key = hex::encode(keypair.verifying_key().as_bytes());
        let peer_id = libp2p::identity::PublicKey::from(
            libp2p::identity::ed25519::PublicKey::try_from_bytes(
                &keypair.verifying_key().to_bytes(),
            )
            .unwrap(),
        )
        .to_peer_id()
        .to_string();

        manager
            .block(BlockedIdentity::new(format!("pk:{public_key}")))
            .unwrap();

        assert!(
            manager.is_blocked(&peer_id, None).unwrap(),
            "transport PeerId must resolve to the UI's public-key block"
        );
    }

    #[test]
    fn legacy_prefixed_peer_id_rows_remain_enforceable() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend.clone());
        let keypair = crate::identity::keys::KeyPair::generate();
        let public_key = hex::encode(keypair.verifying_key().as_bytes());
        let peer_id = libp2p::identity::PublicKey::from(
            libp2p::identity::ed25519::PublicKey::try_from_bytes(
                &keypair.verifying_key().to_bytes(),
            )
            .unwrap(),
        )
        .to_peer_id()
        .to_string();
        let legacy = BlockedIdentity::new(format!("pk:{public_key}"));
        let value = serde_json::to_vec(&legacy).unwrap();

        // Simulate a pre-canonicalization row whose stored value retained the
        // prefix instead of the normalized raw key.
        backend
            .put(legacy.storage_key().as_bytes(), &value)
            .unwrap();

        assert!(manager.is_blocked(&peer_id, None).unwrap());
    }

    #[test]
    fn unblock_removes_both_flavor_entries() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let (pk_hex, derived_id) = generate_test_keypair();

        // Block under public key (creates dual entries)
        manager
            .block(BlockedIdentity::new(format!("pk:{pk_hex}")))
            .unwrap();
        assert!(manager.is_blocked(&pk_hex, None).unwrap());
        assert!(manager.is_blocked(&derived_id, None).unwrap());

        // Unblock using the public key
        manager.unblock(format!("pk:{pk_hex}"), None).unwrap();

        // Both flavors must be gone
        assert!(
            !manager.is_blocked(&pk_hex, None).unwrap(),
            "public key entry must be removed after unblock"
        );
        assert!(
            !manager.is_blocked(&derived_id, None).unwrap(),
            "derived identity_id entry must be removed after unblock"
        );
    }

    #[test]
    fn unblock_canonical_identity_id_removes_public_key_devices_and_registry() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend.clone());

        let (pk_hex, identity_id) = generate_test_keypair();
        let device_a = "device-a";
        let device_b = "device-b";

        manager.register_device_id(&pk_hex, device_a).unwrap();
        manager.register_device_id(&pk_hex, device_b).unwrap();
        manager
            .block(BlockedIdentity::new(format!("pk:{pk_hex}")))
            .unwrap();

        let listed = manager.list().unwrap();
        assert!(!listed.is_empty());
        assert!(listed.iter().all(|blocked| blocked.peer_id == identity_id));

        manager.unblock(identity_id.clone(), None).unwrap();

        assert!(!manager.is_blocked(&pk_hex, None).unwrap());
        assert!(!manager.is_blocked(&identity_id, None).unwrap());
        assert!(!manager.is_blocked(&pk_hex, Some(device_a)).unwrap());
        assert!(!manager.is_blocked(&pk_hex, Some(device_b)).unwrap());
        assert!(manager.get_known_devices(&pk_hex).unwrap().is_empty());
        assert!(manager.get_known_devices(&identity_id).unwrap().is_empty());
        assert!(backend
            .scan_prefix(BLOCKED_PREFIX.as_bytes())
            .unwrap()
            .is_empty());
        assert!(backend
            .scan_prefix(DEVICE_REGISTRY_PREFIX.as_bytes())
            .unwrap()
            .is_empty());
    }

    #[test]
    fn list_blocked_peers_dedupes_both_flavors() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);

        let (pk_hex, derived_id) = generate_test_keypair();

        // Block under public key (writes two peer-level entries)
        manager
            .block(BlockedIdentity::new(format!("pk:{pk_hex}")).with_reason("test".to_string()))
            .unwrap();

        // Verify both entries exist in the raw store
        assert!(manager.is_blocked(&pk_hex, None).unwrap());
        assert!(manager.is_blocked(&derived_id, None).unwrap());

        // list() should dedupe: only the canonical identity_id entry surfaces
        let list = manager.list().unwrap();
        let peer_level: Vec<_> = list.iter().filter(|b| b.device_id.is_none()).collect();
        assert_eq!(
            peer_level.len(),
            1,
            "dual-flavor block should surface as a single entry in list()"
        );
        assert_eq!(
            peer_level[0].peer_id, derived_id,
            "the surfaced entry should use the canonical identity_id"
        );
    }

    #[test]
    fn block_identity_id_writes_no_pk_alias() {
        // Blocking an identity_id (not a valid Ed25519 curve point) should
        // write exactly one peer-level entry and no reverse alias.
        // Use a deterministic non-curve-point identity_id: "7f" * 32 is
        // 64 hex chars but NOT a valid Ed25519 point (probed against
        // ed25519-dalek). This avoids the ~50% flake where a random
        // blake3 hash happens to be a valid curve point.
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);
        let derived_id = "7f".repeat(32);

        // Block the identity_id directly
        manager
            .block(BlockedIdentity::new(derived_id.clone()))
            .unwrap();

        // list() should have exactly one peer-level entry
        let list = manager.list().unwrap();
        let peer_level: Vec<_> = list.iter().filter(|b| b.device_id.is_none()).collect();
        assert_eq!(
            peer_level.len(),
            1,
            "blocking an identity_id must write exactly one entry (no reverse alias)"
        );
        assert_eq!(peer_level[0].peer_id, derived_id);

        // Verify: identity_id_from_public_key_hex returns None for an identity_id
        assert!(
            crate::identity::keys::identity_id_from_public_key_hex(&derived_id).is_none(),
            "an identity_id is not a valid Ed25519 curve point; derivation must return None"
        );
    }

    #[test]
    fn curve_valid_identity_id_is_not_double_hashed() {
        let backend = Arc::new(MemoryStorage::new());
        let manager = BlockedManager::new(backend);
        let (identity_id, double_hash) = (0..10_000)
            .find_map(|_| {
                let (_pk, id) = generate_test_keypair();
                identity_id_from_public_key_hex(&id).map(|double_hash| (id, double_hash))
            })
            .expect("test should find a curve-valid identity id");

        manager
            .block(BlockedIdentity::new(identity_id.clone()))
            .unwrap();

        assert_eq!(
            manager.resolved_identifiers(&identity_id),
            vec![identity_id.clone()]
        );
        assert!(manager.is_blocked(&identity_id, None).unwrap());
        assert!(!manager.is_blocked(&double_hash, None).unwrap());
        assert_eq!(manager.list().unwrap().len(), 1);
    }
}
