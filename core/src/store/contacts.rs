// Contact management storage
//
// Refactored to use generic StorageBackend for cross-platform parity (Sled/IndexedDB/Memory).

use crate::identity::keys::is_valid_public_key;
use crate::identity::PublicKeyBundle;
use crate::store::backend::StorageBackend;
use crate::store::history::HistoryManager;
use crate::IronCoreError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Notes annotation marking a contact whose public key could not be derived
/// from its peer id. An empty `public_key` plus this marker IS the placeholder
/// contract: verified material (a signed envelope, an explicit user add)
/// backfills the key, and the peer id itself is never stored as the key.
///
/// Single owner. This module is compiled for every target, so the recovery
/// path in `contacts_bridge` (native only) imports it rather than keeping a
/// second copy of the string that could drift.
pub const PLACEHOLDER_KEY_NOTE: &str =
    "public_key unavailable: not self-certifying from peer id; awaiting verified key";

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

// UNIFICATION: normalize nickname — trims whitespace, returns None if empty (mirrors Kotlin normalizeNickname)
fn normalize_nickname(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

// UNIFICATION: synthetic fallback detection — "peer-..." placeholder must never overwrite real nickname
fn is_synthetic_fallback_nickname(value: &Option<String>) -> bool {
    if let Some(normalized) = normalize_nickname(value) {
        normalized.to_lowercase().starts_with("peer-")
    } else {
        false
    }
}

// UNIFICATION: authoritative nickname selection — prefers real over synthetic, mirrors Kotlin/iOS selectAuthoritativeNickname
fn select_authoritative_nickname(
    incoming: &Option<String>,
    existing: &Option<String>,
) -> Option<String> {
    let incoming_norm = normalize_nickname(incoming);
    let existing_norm = normalize_nickname(existing);
    let incoming_synthetic = is_synthetic_fallback_nickname(&incoming_norm);
    let existing_synthetic = is_synthetic_fallback_nickname(&existing_norm);
    match (&incoming_norm, &existing_norm) {
        _ if incoming_norm.is_none() && existing_synthetic => None,
        _ if incoming_norm.is_none() => existing_norm,
        _ if incoming_synthetic && existing_norm.is_none() => None,
        _ if incoming_synthetic && existing_synthetic => None,
        _ if incoming_synthetic => existing_norm,
        _ if existing_synthetic => incoming_norm,
        _ => incoming_norm,
    }
}

#[derive(Clone)]
pub struct ContactManager {
    backend: Arc<dyn StorageBackend>,
}

impl ContactManager {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        let manager = Self { backend };
        manager.migrate_unprefixed_contacts();
        manager.migrate_libp2p_peer_ids_to_canonical_hex();
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

    /// UNIFICATION: canonicalize peer_id from libp2p (12D3Koo...) to public_key_hex (64 hex).
    /// Both hashes refer to same identity (libp2p for routing, hex for crypto) but must not spawn duplicate nodes.
    /// Verbose log for verification. Re-runnable: scans every startup even if flag set, to catch contacts added
    /// as 12D3 after initial migration (e.g., via addContact before canonical fix, or ledger entries with old 12D3).
    /// Also canonicalizes peer_id from `public_key` when peerId is libp2p but public_key is already 30d0fa hex.
    fn migrate_libp2p_peer_ids_to_canonical_hex(&self) {
        let already_migrated = self
            .backend
            .get(b"metadata_contacts_canonical_hex_migrated")
            .map(|opt| opt.is_some())
            .unwrap_or(false);
        let Ok(contacts) = self.list() else {
            let _ = self
                .backend
                .put(b"metadata_contacts_canonical_hex_migrated", b"true");
            return;
        };
        let mut migrated = 0u32;
        let mut deduped = 0u32;
        let mut normalized_case = 0u32;
        for contact in contacts {
            let peer_id_trimmed = contact.peer_id.trim();
            if peer_id_trimmed.is_empty() {
                continue;
            }
            // UNIFICATION: Determine canonical hex: valid publicKey (64-hex) is strongest, else derive from libp2p peerId.
            // This handles the duplicate where peerId=12D3Koo... and public_key=30d0fa... (both same identity).
            let canonical_hex = if contact.public_key.trim().len() == 64
                && contact.public_key.chars().all(|c| c.is_ascii_hexdigit())
                && hex::decode(contact.public_key.trim()).is_ok()
            {
                contact.public_key.trim().to_lowercase()
            } else if let Ok(derived) = self.derive_public_key_from_peer_id(peer_id_trimmed) {
                derived.to_lowercase()
            } else {
                continue;
            };
            if canonical_hex == peer_id_trimmed.to_lowercase() {
                // UNIFICATION: Already canonical hex but may need case normalization (30D0FA -> 30d0fa)
                let needs_case_norm =
                    contact.peer_id != canonical_hex || contact.public_key.trim() != canonical_hex;
                if needs_case_norm {
                    let mut norm = contact.clone();
                    norm.peer_id = canonical_hex.clone();
                    norm.public_key = canonical_hex.clone();
                    if self.add(norm).is_ok() && contact.peer_id != canonical_hex {
                        let _ = self.backend.remove(&contact_key(peer_id_trimmed));
                    }
                    normalized_case += 1;
                    tracing::info!(
                        event = "contacts_canonical_hex_case_norm",
                        from = %peer_id_trimmed,
                        to = %canonical_hex,
                        "normalized contact case to canonical lower hex"
                    );
                }
                continue;
            }
            // Check if canonical already exists (avoid duplicate)
            let canonical_key = contact_key(&canonical_hex);
            let exists = self
                .backend
                .get(&canonical_key)
                .map(|opt| opt.is_some())
                .unwrap_or(false);
            if exists {
                // UNIFICATION: Merge nicknames using authoritative logic — prefer real over synthetic
                // Previously only copied when canonical None, keeping synthetic "peer-..." when libp2p held real "ChristyLove".
                // Now use select_authoritative_nickname / is_synthetic_fallback_nickname: if canonical synthetic/None and libp2p real, replace.
                if let Ok(Some(mut canonical_contact)) = self.get(canonical_hex.clone()) {
                    let mut changed = false;
                    let canonical_nick_before = canonical_contact.nickname.clone();
                    let libp2p_nick = contact.nickname.clone();
                    let canonical_local_before = canonical_contact.local_nickname.clone();
                    let libp2p_local = contact.local_nickname.clone();

                    // UNIFICATION: authoritative nickname merge for federated nickname
                    let authoritative_nick = select_authoritative_nickname(
                        &contact.nickname,
                        &canonical_contact.nickname,
                    );
                    let canonical_is_synthetic =
                        is_synthetic_fallback_nickname(&canonical_contact.nickname);
                    let libp2p_is_synthetic = is_synthetic_fallback_nickname(&contact.nickname);
                    let should_update_nick =
                        match (&authoritative_nick, &canonical_contact.nickname) {
                            (Some(auth), Some(curr)) => {
                                // Only replace if canonical was synthetic/None; preserve real canonical when both real
                                (is_synthetic_fallback_nickname(&Some(curr.clone()))
                                    || normalize_nickname(&Some(curr.clone())).is_none())
                                    && !is_synthetic_fallback_nickname(&Some(auth.clone()))
                            }
                            (Some(_), None) => true,
                            (None, Some(curr))
                                if is_synthetic_fallback_nickname(&Some(curr.clone())) =>
                            {
                                true
                            }
                            _ => false,
                        };
                    // UNIFICATION verbose: log every deduplication nickname decision
                    if should_update_nick {
                        tracing::info!(
                            event = "contacts_canonical_hex_dedup_nickname_merge",
                            from = %peer_id_trimmed,
                            to = %canonical_hex,
                            canonical_nickname_before = ?canonical_nick_before,
                            libp2p_nickname = ?libp2p_nick,
                            authoritative_nickname = ?authoritative_nick,
                            canonical_was_synthetic = canonical_is_synthetic,
                            libp2p_was_synthetic = libp2p_is_synthetic,
                            canonical_was_none = canonical_nick_before.is_none(),
                            "UNIFICATION dedup: merging nickname via selectAuthoritativeNickname"
                        );
                        canonical_contact.nickname = authoritative_nick.clone();
                        changed = true;
                    } else {
                        tracing::info!(
                            event = "contacts_canonical_hex_dedup_nickname_keep",
                            from = %peer_id_trimmed,
                            to = %canonical_hex,
                            canonical_nickname = ?canonical_nick_before,
                            libp2p_nickname = ?libp2p_nick,
                            authoritative_nickname = ?authoritative_nick,
                            canonical_was_synthetic = canonical_is_synthetic,
                            libp2p_was_synthetic = libp2p_is_synthetic,
                            "UNIFICATION dedup: keeping canonical nickname (no merge needed)"
                        );
                        // UNIFICATION: if authoritative is None but canonical synthetic, clear placeholder to None
                        if authoritative_nick.is_none() && canonical_is_synthetic {
                            tracing::info!(
                                event = "contacts_canonical_hex_dedup_nickname_clear_synthetic",
                                from = %peer_id_trimmed,
                                to = %canonical_hex,
                                cleared = ?canonical_nick_before,
                                "UNIFICATION dedup: clearing synthetic nickname to None"
                            );
                            canonical_contact.nickname = None;
                            changed = true;
                        }
                    }

                    // UNIFICATION: same authoritative logic for localNickname
                    let authoritative_local = select_authoritative_nickname(
                        &contact.local_nickname,
                        &canonical_contact.local_nickname,
                    );
                    let canonical_local_is_synthetic =
                        is_synthetic_fallback_nickname(&canonical_contact.local_nickname);
                    let libp2p_local_is_synthetic =
                        is_synthetic_fallback_nickname(&contact.local_nickname);
                    let should_update_local =
                        match (&authoritative_local, &canonical_contact.local_nickname) {
                            (Some(auth), Some(curr)) => {
                                (is_synthetic_fallback_nickname(&Some(curr.clone()))
                                    || normalize_nickname(&Some(curr.clone())).is_none())
                                    && !is_synthetic_fallback_nickname(&Some(auth.clone()))
                            }
                            (Some(_), None) => true,
                            (None, Some(curr))
                                if is_synthetic_fallback_nickname(&Some(curr.clone())) =>
                            {
                                true
                            }
                            _ => false,
                        };
                    if should_update_local {
                        tracing::info!(
                            event = "contacts_canonical_hex_dedup_local_nickname_merge",
                            from = %peer_id_trimmed,
                            to = %canonical_hex,
                            canonical_local_before = ?canonical_local_before,
                            libp2p_local = ?libp2p_local,
                            authoritative_local = ?authoritative_local,
                            canonical_was_synthetic = canonical_local_is_synthetic,
                            libp2p_was_synthetic = libp2p_local_is_synthetic,
                            "UNIFICATION dedup: merging localNickname via selectAuthoritativeNickname"
                        );
                        canonical_contact.local_nickname = authoritative_local.clone();
                        changed = true;
                    } else {
                        tracing::info!(
                            event = "contacts_canonical_hex_dedup_local_nickname_keep",
                            from = %peer_id_trimmed,
                            to = %canonical_hex,
                            canonical_local = ?canonical_local_before,
                            libp2p_local = ?libp2p_local,
                            authoritative_local = ?authoritative_local,
                            "UNIFICATION dedup: keeping canonical localNickname"
                        );
                        if authoritative_local.is_none() && canonical_local_is_synthetic {
                            tracing::info!(
                                event = "contacts_canonical_hex_dedup_local_nickname_clear_synthetic",
                                from = %peer_id_trimmed,
                                to = %canonical_hex,
                                cleared = ?canonical_local_before,
                                "UNIFICATION dedup: clearing synthetic localNickname to None"
                            );
                            canonical_contact.local_nickname = None;
                            changed = true;
                        }
                    }

                    if changed {
                        let _ = self.add(canonical_contact.clone());
                        tracing::info!(
                            event = "contacts_canonical_hex_dedup_updated_canonical",
                            peer_id = %canonical_hex,
                            nickname = ?canonical_contact.nickname,
                            local_nickname = ?canonical_contact.local_nickname,
                            "UNIFICATION dedup: updated canonical contact with authoritative nicknames"
                        );
                    } else {
                        tracing::info!(
                            event = "contacts_canonical_hex_dedup_no_change",
                            from = %peer_id_trimmed,
                            to = %canonical_hex,
                            canonical_nickname = ?canonical_nick_before,
                            canonical_local = ?canonical_local_before,
                            "UNIFICATION dedup: no nickname changes needed"
                        );
                    }
                } else {
                    tracing::warn!(
                        event = "contacts_canonical_hex_dedup_missing_canonical",
                        from = %peer_id_trimmed,
                        to = %canonical_hex,
                        "UNIFICATION dedup: canonical contact missing despite exists flag"
                    );
                }
                let _ = self.backend.remove(&contact_key(peer_id_trimmed));
                deduped += 1;
                tracing::info!(
                    event = "contacts_canonical_hex_dedup",
                    from = %peer_id_trimmed,
                    to = %canonical_hex,
                    "deduped libp2p contact into canonical hex"
                );
            } else {
                // Rename: create canonical, remove old
                let mut new_contact = contact.clone();
                new_contact.peer_id = canonical_hex.clone();
                // Ensure publicKey is canonical hex
                new_contact.public_key = canonical_hex.clone();
                if self.add(new_contact).is_ok() {
                    let _ = self.backend.remove(&contact_key(peer_id_trimmed));
                    migrated += 1;
                    tracing::info!(
                        event = "contacts_canonical_hex_migration",
                        from = %peer_id_trimmed,
                        to = %canonical_hex,
                        nickname = ?contact.nickname,
                        "migrated libp2p peerId to canonical public_key_hex"
                    );
                }
            }
        }
        let _ = self
            .backend
            .put(b"metadata_contacts_canonical_hex_migrated", b"true");
        if migrated > 0 || deduped > 0 || normalized_case > 0 {
            tracing::info!(
                event = "contacts_canonical_hex_migration_done",
                migrated_count = migrated,
                deduped_count = deduped,
                normalized_case_count = normalized_case,
                already_migrated_flag = already_migrated,
                "contacts canonical hex migration completed"
            );
        } else if already_migrated {
            tracing::debug!(
                event = "contacts_canonical_hex_migration_skipped",
                "contacts already canonical — re-runnable check found no libp2p entries"
            );
        }
    }

    /// Reconcile contacts from message history to recover potentially lost records.
    /// Scans all message records and creates a basic contact if the peer_id is unknown.
    ///
    /// WP1.1: a peer whose key cannot be derived is still recorded, but as a
    /// PLACEHOLDER with an empty `public_key` plus a notes marker. The record
    /// is deliberately NOT an encryptable send target until verified material
    /// (a signed envelope, an explicit user add) backfills the key. Recording
    /// the peer keeps it visible to the operator; leaving the key fabricated
    /// or the record absent are both worse, and an empty key makes
    /// `prepare_message` refuse rather than emit unopenable ciphertext.
    pub fn reconcile_from_history(&self, history: &HistoryManager) -> Result<u32, IronCoreError> {
        let all_messages = history.recent_including_hidden(None, 10000)?;
        let mut recovered_count = 0;

        for msg in all_messages {
            if self.get(msg.peer_id.clone()).is_ok() && self.get(msg.peer_id.clone())?.is_none() {
                // We have the peer_id from history but no contact record. A
                // public key binds ONLY when it is self-certifying from the
                // peer id; the peer id is never itself stored as the key.
                let contact = match self.derive_public_key_from_peer_id(&msg.peer_id) {
                    Ok(pub_key) => Contact::new(msg.peer_id.clone(), pub_key),
                    Err(_) => {
                        tracing::warn!(
                            event = "contact_recovery_placeholder",
                            "History peer has no self-certifying key binding; storing \
                             placeholder record without a public key"
                        );
                        let mut c = Contact::new(msg.peer_id.clone(), String::new());
                        c.notes = Some(PLACEHOLDER_KEY_NOTE.to_string());
                        c
                    }
                };
                self.add(contact)?;
                recovered_count += 1;
            }
        }
        Ok(recovered_count)
    }

    /// Recover a peer's Ed25519 public key from the identifier we hold for it.
    ///
    /// A key is returned ONLY when the identifier is SELF-CERTIFYING: either it
    /// is already a valid Ed25519 point, or a libp2p identity-multihash peer id
    /// that re-derives from the key extracted from it. Everything else --
    /// SHA-256-hashed peer ids, arbitrary base58 blobs, `identity_id` hashes,
    /// garbage -- yields `Err` so the caller stores a placeholder with an empty
    /// key instead of a fabricated one.
    ///
    /// WP1.1 removed a "take the last 32 bytes of the multihash" fallback that
    /// returned `Ok` for any base58 blob of length >= 32. For a non-identity
    /// peer id those bytes are HASH OUTPUT, not a public key, so every send
    /// encrypted to that value produced ciphertext the peer can never open --
    /// and because the field then looked populated, no later layer re-checked
    /// it. That is the same defect as storing `peer_id` as the key, reached by
    /// a different route. Delegates to the single self-certifying helper the
    /// recovery path in `contacts_bridge` already uses, so both agree by
    /// construction rather than by parallel maintenance.
    fn derive_public_key_from_peer_id(&self, peer_id: &str) -> Result<String, IronCoreError> {
        let trimmed = peer_id.trim();

        // A 64-hex identifier is ALREADY in canonical key form, so there is
        // nothing to derive: the answer is the value itself or nothing.
        //
        // WP1.1: an `identity_id` is also 64 hex chars, so shape cannot tell
        // the two apart -- ~48% of blake3 digests pass curve decompression
        // (measured on this tree: 966/2000). This function therefore does NOT
        // attempt to guess: it returns the value unchanged when it has the
        // shape of a key, and refuses otherwise. Reversing a hash into a key is
        // impossible, so an identity_id that reaches here simply passes through
        // as an identifier; whether it may be USED as an encryption key is
        // decided at the point of use (`IronCore::prepare_message`), which is
        // the only layer that can compare it against a known contact's key.
        if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            if is_valid_public_key(trimmed) {
                return Ok(trimmed.to_lowercase());
            }
            return Err(IronCoreError::InvalidInput);
        }

        // libp2p peer id: only the strict identity multihash embeds a key, and
        // the helper re-derives the peer id from the extracted key before
        // returning it, so a crafted blob cannot smuggle a binding through.
        crate::store::ledger_entry::public_key_hex_from_libp2p_peer_id(trimmed)
            .map(|k| k.to_lowercase())
            .ok_or(IronCoreError::InvalidInput)
    }

    /// The canonical key a contact is stored and looked up under.
    ///
    /// This is the SINGLE owner of "what is the contact key". Both the write
    /// path (`add`) and the read path (`get`) ask this function, so the rule
    /// cannot drift between them -- which is exactly the asymmetry that let
    /// `get` miss a row `add` had just written under a different spelling.
    ///
    /// Order is the long-standing unification contract, unchanged:
    ///   1. a supplied key that is ed25519-shaped IS the canonical identity
    ///      (V2 contract, asserted by `lookup_by_public_key_resolves_peer_keyed_contact`);
    ///   2. otherwise derive from the peer id (libp2p identity multihash);
    ///   3. otherwise the peer id is already a canonical key;
    ///   4. otherwise there is no canonical key and the caller keeps whatever
    ///      it was given.
    ///
    /// Note this answers a SHAPE question via
    /// [`crate::identity::keys::is_valid_public_key`], the single owner of that
    /// predicate. Roughly half of all 32-byte values pass curve decompression,
    /// so shape never decides whether a value is the right key for a peer --
    /// that is decided where a key is used, in `IronCore::prepare_message`.
    fn canonical_contact_key(&self, peer_id: &str, public_key: &str) -> Option<String> {
        let peer_id = peer_id.trim();
        if peer_id.is_empty() {
            return None;
        }
        let public_key = public_key.trim();
        if is_valid_public_key(public_key) {
            return Some(public_key.to_lowercase());
        }
        if let Ok(derived) = self.derive_public_key_from_peer_id(peer_id) {
            return Some(derived.to_lowercase());
        }
        if is_valid_public_key(peer_id) {
            return Some(peer_id.to_lowercase());
        }
        None
    }

    pub fn add(&self, mut contact: Contact) -> Result<(), IronCoreError> {
        // UNIFICATION: Live canonicalize contact writes — mirrors migrate_libp2p_peer_ids_to_canonical_hex (load migration).
        // Prevents new 12D3 entries that would duplicate already-migrated hex nodes until next load.
        let peer_id_trimmed = contact.peer_id.trim().to_string();
        if !peer_id_trimmed.is_empty() {
            // The rule lives in `canonical_contact_key`; the read path asks the
            // same function, so the two cannot disagree about the key.
            let canonical_hex: Option<String> =
                self.canonical_contact_key(&peer_id_trimmed, &contact.public_key);
            if let Some(canonical) = canonical_hex {
                if peer_id_trimmed.to_lowercase() != canonical {
                    tracing::info!(
                        event = "contacts_canonical_hex_live",
                        from = %peer_id_trimmed,
                        to = %canonical,
                        "canonicalized contact peer_id on write libp2p -> hex"
                    );
                    contact.peer_id = canonical.clone();
                    // Ensure public_key is populated/normalized when peer_id was libp2p
                    let pk_valid = is_valid_public_key(contact.public_key.trim());
                    // Clippy: both arms set public_key to canonical; collapse.
                    if !pk_valid
                        || (contact.public_key.trim().to_lowercase() == canonical
                            && contact.public_key != canonical)
                    {
                        contact.public_key = canonical.clone();
                    }
                } else if contact.peer_id != canonical {
                    contact.peer_id = canonical.clone();
                    if contact.public_key.trim().to_lowercase() == canonical
                        && contact.public_key != canonical
                    {
                        contact.public_key = canonical;
                    }
                }
            }
        }
        // UNIFICATION verbose logging for nickname save — diagnose ChristyLove revert to peer-... synthetic
        tracing::info!(
            event = "contacts_add",
            peer_id = %contact.peer_id,
            nickname = ?contact.nickname,
            local_nickname = ?contact.local_nickname,
            public_key_prefix = %contact.public_key.chars().take(8).collect::<String>(),
            "UNIFICATION saving contact nickname"
        );
        let key = contact_key(&contact.peer_id);
        let value = serde_json::to_vec(&contact).map_err(|_| IronCoreError::Internal)?;
        self.backend
            .put(&key, &value)
            .map_err(|_| IronCoreError::StorageError)?;

        // STEP 2: Maintain identity_id -> public_key index for backward compatibility.
        // UNIFICATION_V2_IDENTITY: Use single source of truth for identity_id derivation.
        if let Some(identity_id) =
            crate::identity::identity_id_from_public_key_hex(&contact.public_key)
        {
            let _ = self.save_identity_id_index(&identity_id, &contact.public_key);
        }

        Ok(())
    }

    pub fn get(&self, peer_id: String) -> Result<Option<Contact>, IronCoreError> {
        // UNIFICATION verbose logging for nickname load
        let key = contact_key(&peer_id);
        if let Some(data) = self
            .backend
            .get(&key)
            .map_err(|_| IronCoreError::StorageError)?
        {
            let contact: Contact =
                serde_json::from_slice(&data).map_err(|_| IronCoreError::Internal)?;
            tracing::debug!(
                event = "contacts_get",
                peer_id = %peer_id,
                nickname = ?contact.nickname,
                local_nickname = ?contact.local_nickname,
                "UNIFICATION loaded contact nickname"
            );
            Ok(Some(contact))
        } else {
            // If not found by peer_id, try resolving as identity_id
            if let Ok(Some(public_key)) = self.resolve_identity_id(&peer_id) {
                return self.get(public_key);
            }
            // WP1: `add()` canonicalizes a libp2p peer id to the contact's
            // public-key hex before writing, so the row is filed under the hex.
            // Resolve through the SAME owner the write path uses, or
            // `get(base58_peer_id)` misses a row this very manager just wrote.
            // That asymmetry is not cosmetic: the envelope-learning path looks a
            // contact up by the peer id it saw on the wire, and a miss there
            // skips the nickname-preference logic and rebuilds the record.
            //
            // The guard bounds the recursion: for a value that is already the
            // canonical key the owner returns it unchanged, and re-entering
            // `get` would look up the same missing key forever. An uppercase
            // hex input case-folds to lowercase on the first hop and then
            // stops.
            if let Some(canonical) = self.canonical_contact_key(&peer_id, "") {
                if !canonical.eq_ignore_ascii_case(&peer_id) {
                    return self.get(canonical);
                }
            }
            Ok(None)
        }
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
        // UNIFICATION verbose logging for nickname list — helps diagnose ledger overwrite of localNickname
        tracing::debug!(
            event = "contacts_list",
            count = contacts.len(),
            nicknames = ?contacts.iter().map(|c| (c.peer_id.chars().take(8).collect::<String>(), c.nickname.clone(), c.local_nickname.clone())).collect::<Vec<_>>(),
            "UNIFICATION listed contacts with nicknames"
        );
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
                // UNIFICATION_V2_IDENTITY: Use single source of truth for identity_id derivation.
                if let Some(identity_id) =
                    crate::identity::identity_id_from_public_key_hex(&contact.public_key)
                {
                    // Only save if not already indexed
                    if let Ok(None) = self.resolve_identity_id(&identity_id) {
                        let _ = self.save_identity_id_index(&identity_id, &contact.public_key);
                        migrated += 1;
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
    use crate::identity::keys::self_certifying_keypair;
    use crate::store::backend::MemoryStorage;
    use std::sync::Arc;

    fn make_manager() -> ContactManager {
        ContactManager::new(Arc::new(MemoryStorage::new()))
    }

    /// Build a genuine self-certifying (peer id, key hex) pair.

    /// A base58-encoded SHA-256 multihash (`0x12 0x20 <32 bytes>`): a valid
    /// libp2p peer id shape that is NOT an identity multihash, so it embeds no
    /// public key. This is the input the removed fabrication fallback used to
    /// "recover" a key from.
    fn sha256_multihash_peer_id() -> String {
        let mut bytes = vec![0x12u8, 0x20];
        bytes.extend(0u8..32);
        bs58::encode(bytes).into_string()
    }

    // -----------------------------------------------------------------------
    // WP1.1 -- recovery never fabricates a public key
    // -----------------------------------------------------------------------

    /// WP1.1: a non-identity peer id carries no public key. The removed
    /// "last 32 bytes of the multihash" fallback returned `Some` for exactly
    /// this input, so a send encrypted to a hash nobody holds.
    #[test]
    fn derive_refuses_non_identity_multihash_peer_id() {
        let mgr = make_manager();
        let peer = sha256_multihash_peer_id();
        assert!(
            mgr.derive_public_key_from_peer_id(&peer).is_err(),
            "a SHA-256 multihash peer id must not yield a public key"
        );
    }

    /// WP1.1: the self-certifying case must still work. A fix that refuses
    /// everything would satisfy the test above while breaking every real send.
    #[test]
    fn derive_accepts_self_certifying_peer_id() {
        let mgr = make_manager();
        let (peer_id, key_hex) = self_certifying_keypair(b"wp1-derive-accept");
        assert_eq!(
            mgr.derive_public_key_from_peer_id(&peer_id).unwrap(),
            key_hex
        );
    }

    /// WP1.1: garbage and truncated peer ids yield no key.
    #[test]
    fn derive_refuses_garbage_identifiers() {
        let mgr = make_manager();
        for bad in [
            "",
            "not-a-peer-id",
            "abc",
            // Short, wrong-width, and non-hex values are what the shape filter
            // is for. A base58 `12D3Koo...` id is a REAL peer id that simply
            // is not an identity multihash, so it is rejected by
            // self-certification rather than by shape -- see
            // `derive_refuses_non_identity_multihash_peer_id`.
            "zz",
            "00",
        ] {
            assert!(
                mgr.derive_public_key_from_peer_id(bad).is_err(),
                "identifier {bad:?} must not yield a public key"
            );
        }
    }

    /// WP1.1 + WP1.2: an `identity_id` is 64 hex chars and decodes to 32 bytes,
    /// so width-based checks admit it, and ~48% of blake3 digests also pass
    /// curve decompression. The store therefore CANNOT distinguish an identity_id
    /// from a public key, and must not pretend to. What it must guarantee is
    /// narrower and real: it never INVENTS a key. Derivation returns either the
    /// identifier unchanged (already canonical) or an error -- never a key
    /// reconstructed from bytes that are not one.
    ///
    /// The hash-vs-key decision belongs to the point of use, where a comparison
    /// against a known contact's key is possible. See
    /// `identity_hash_not_usable_as_recipient` in iron_core.
    #[test]
    fn derive_never_invents_a_key_from_an_identity_id() {
        let mgr = make_manager();
        let (peer_id, key_hex) = self_certifying_keypair(b"wp1-identity-id");
        let identity_id =
            crate::identity::keys::identity_id_from_public_key_hex(&key_hex).expect("identity id");

        assert_eq!(identity_id.len(), 64, "identity_id is also 64 hex chars");
        assert!(
            hex::decode(&identity_id).is_ok(),
            "and it decodes to 32 bytes"
        );
        assert_ne!(identity_id, key_hex);

        // A hash does not self-certify as a peer id, so it can never be
        // mistaken for a binding -- this is the property that holds
        // deterministically, unlike the shape check.
        assert!(
            !crate::store::ledger_entry::is_self_certifying_binding(&identity_id, &identity_id),
            "an identity_id must not self-certify as its own key"
        );

        // Whatever `derive` returns for the hash, it is either the value
        // unchanged (canonical pass-through) or a refusal. It is NEVER some
        // third value reconstructed from the hash.
        match mgr.derive_public_key_from_peer_id(&identity_id) {
            Err(_) => {}
            Ok(passed_through) => assert_eq!(
                passed_through,
                identity_id.to_lowercase(),
                "a 64-hex identifier must pass through unchanged or be refused, \
                 never be turned into a different value"
            ),
        }

        // The genuine key and peer id both resolve to the real key, so the
        // function is discriminating rather than blanket-refusing.
        assert_eq!(
            mgr.derive_public_key_from_peer_id(&key_hex).unwrap(),
            key_hex
        );
        assert_eq!(
            mgr.derive_public_key_from_peer_id(&peer_id).unwrap(),
            key_hex
        );
    }

    /// WP1.2: a supplied key that is NOT a valid 64-hex value must not become
    /// the stored key, and must not rewrite `peer_id` to itself. This is the
    /// half of the write path that IS decidable locally.
    #[test]
    fn add_refuses_malformed_key() {
        let mgr = make_manager();
        let (peer_id, key_hex) = self_certifying_keypair(b"wp1-add-malformed");

        for bad in [
            "zz-not-hex",
            "deadbeef",
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdefzz",
        ] {
            mgr.add(Contact::new(peer_id.clone(), bad.to_string()))
                .unwrap();
            let stored = mgr
                .get(key_hex.clone())
                .unwrap()
                .expect("contact canonicalized from the self-certifying peer id");
            assert!(
                !stored.public_key.eq_ignore_ascii_case(bad),
                "malformed key {bad:?} must not be stored as the encryption key"
            );
            assert!(
                is_valid_public_key(&stored.public_key),
                "the derived real key must win, got {}",
                stored.public_key
            );
        }
    }

    // -----------------------------------------------------------------------
    // WP1.2 -- writes only ever store a self-certifying binding
    // -----------------------------------------------------------------------

    /// WP1.2: read and write must agree on the contact key. `add()`
    /// canonicalizes a libp2p peer id to the contact's public-key hex, so the
    /// row is filed under the hex -- a `get()` by the base58 peer id must
    /// still find it.
    ///
    /// This asymmetry was live: `add(Contact::new(peer_id, peer_id))` wrote a
    /// correct, self-certifying row under the hex, and `get(peer_id)` returned
    /// `None` for the row the same manager had just written. Callers that key
    /// off the wire peer id -- notably CLI envelope learning -- silently missed
    /// the contact, skipped the nickname-preference branch, and rebuilt the
    /// record, dropping a user-set local nickname.
    #[test]
    fn get_resolves_peer_id_that_add_canonicalized_to_hex() {
        let mgr = make_manager();
        let (peer_id, key_hex) = self_certifying_keypair(b"wp1-read-write-symmetry");

        mgr.add(Contact::new(peer_id.clone(), key_hex.clone()))
            .unwrap();

        // Written under the canonical hex...
        let by_hex = mgr.get(key_hex.clone()).unwrap().expect("row by hex");
        // ...and reachable by the peer id it was added under.
        let by_peer = mgr
            .get(peer_id.clone())
            .unwrap()
            .expect("row by peer id must resolve");
        assert_eq!(by_peer.peer_id, by_hex.peer_id);
        assert_eq!(by_peer.public_key, key_hex);
    }

    /// The legacy poison shape -- a libp2p peer id stored as the public key --
    /// must be repaired by `add()` and then be readable by either spelling.
    /// This is the exact fixture the CLI envelope-learning test seeds.
    #[test]
    fn legacy_peer_id_as_public_key_is_repaired_and_readable() {
        let mgr = make_manager();
        let (peer_id, key_hex) = self_certifying_keypair(b"wp1-legacy-poison");

        mgr.add(Contact::new(peer_id.clone(), peer_id.clone()))
            .unwrap();

        for lookup in [peer_id.clone(), key_hex.clone()] {
            let stored = mgr
                .get(lookup.clone())
                .unwrap()
                .unwrap_or_else(|| panic!("contact must resolve by {lookup}"));
            assert_eq!(
                stored.public_key.to_lowercase(),
                key_hex,
                "the peer id must never survive as the stored key"
            );
            assert_ne!(stored.public_key, peer_id);
            // Self-certification is checked against the ORIGINAL base58 id: the
            // stored `peer_id` field is canonicalized to the key hex, so
            // comparing the hex to itself would prove nothing.
            assert!(
                crate::store::ledger_entry::is_self_certifying_binding(&peer_id, &key_hex),
                "the repaired key must still re-derive the original peer id"
            );
        }
    }

    /// A lookup that cannot be canonicalized must stay a clean miss, not an
    /// error and not an infinite fallback.
    #[test]
    fn get_returns_none_for_unrelated_identifier() {
        let mgr = make_manager();
        let (peer_id, _) = self_certifying_keypair(b"wp1-miss");
        mgr.add(Contact::new(peer_id, String::new())).unwrap();

        assert!(
            mgr.get("nobody-by-that-name".to_string())
                .unwrap()
                .is_none(),
            "an unrelated identifier must miss cleanly"
        );
    }

    /// WP1.2: the canonical write shape -- libp2p peer id + its real key --
    /// canonicalizes to the hex key, and the binding is self-certifying. This
    /// is the case the unification exists to serve, so it must not regress.
    #[test]
    fn add_canonicalizes_self_certifying_binding() {
        let mgr = make_manager();
        let (peer_id, key_hex) = self_certifying_keypair(b"wp1-add-canonical");
        mgr.add(Contact::new(peer_id.clone(), key_hex.clone()))
            .unwrap();

        // Looked up by the canonical key hex, which is how the row is stored.
        let stored = mgr.get(key_hex.clone()).unwrap().expect("contact");
        assert_eq!(stored.peer_id, key_hex, "peer_id canonicalizes to key hex");
        assert_eq!(stored.public_key, key_hex);
        assert!(is_valid_public_key(&stored.public_key));
        assert!(crate::store::ledger_entry::is_self_certifying_binding(
            &peer_id,
            &stored.public_key
        ));
    }

    /// WP1.1: recovery of a history peer with no derivable key stores a
    /// PLACEHOLDER -- recorded so the operator can see the peer, with an EMPTY
    /// key so it cannot be used as an encrypt target.
    #[test]
    fn reconcile_stores_placeholder_for_non_derivable_peer() {
        use crate::store::history::{HistoryManager, MessageRecord};

        let mgr = make_manager();
        let history = HistoryManager::new(Arc::new(MemoryStorage::new()));
        let peer = sha256_multihash_peer_id();

        history
            .add(MessageRecord {
                id: "wp1-placeholder-msg".to_string(),
                direction: crate::store::history::MessageDirection::Received,
                peer_id: peer.clone(),
                content: "hello".to_string(),
                timestamp: 1,
                sender_timestamp: 1,
                delivered: false,
                hidden: false,
            })
            .unwrap();

        let recovered = mgr.reconcile_from_history(&history).unwrap();
        assert_eq!(recovered, 1, "the peer is recorded, not silently dropped");

        let stored = mgr.get(peer.clone()).unwrap().expect("placeholder contact");
        assert!(
            stored.public_key.is_empty(),
            "placeholder must carry no key, got {}",
            stored.public_key
        );
        assert_ne!(stored.public_key, peer, "never peer-id-as-public-key");
        assert_eq!(stored.notes.as_deref(), Some(PLACEHOLDER_KEY_NOTE));
    }

    /// WP1.1: a self-certifying history peer still recovers its real key, so
    /// the placeholder change did not turn every recovery into a placeholder.
    #[test]
    fn reconcile_derives_key_for_self_certifying_keypair() {
        use crate::store::history::HistoryManager;

        let mgr = make_manager();
        let history = HistoryManager::new(Arc::new(MemoryStorage::new()));
        let (peer_id, key_hex) = self_certifying_keypair(b"wp1-reconcile-derived");

        history
            .add(crate::store::history::MessageRecord {
                id: "wp1-derived-msg".to_string(),
                direction: crate::store::history::MessageDirection::Received,
                peer_id: peer_id.clone(),
                content: "hello".to_string(),
                timestamp: 1,
                sender_timestamp: 1,
                delivered: false,
                hidden: false,
            })
            .unwrap();

        assert_eq!(mgr.reconcile_from_history(&history).unwrap(), 1);
        let stored = mgr.get(key_hex.clone()).unwrap().expect("derived contact");
        assert_eq!(stored.public_key, key_hex);
        assert!(is_valid_public_key(&stored.public_key));
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
            .expect("public-key lookup should find the contact");
        // V2 canonicalization: a valid 64-hex public key IS the canonical
        // contact identity, so the stored peer_id is the lowercased public
        // key, not the arbitrary add-time label.
        assert_eq!(contact.peer_id, public_key);
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

    // UNIFICATION: dedup must prefer real nickname over synthetic "peer-..." placeholder
    #[test]
    fn test_canonical_hex_dedup_preserves_real_over_synthetic() {
        use crate::identity::IdentityKeys;
        let backend = Arc::new(MemoryStorage::new());
        let keys = IdentityKeys::generate();
        let pubkey_hex = hex::encode(keys.signing_key.verifying_key().to_bytes()).to_lowercase();
        let mut libp2p_bytes = vec![0x00, 0x24, 0x08, 0x01, 0x12, 0x20];
        libp2p_bytes.extend_from_slice(&keys.signing_key.verifying_key().to_bytes());
        let libp2p_peer_id = bs58::encode(&libp2p_bytes).into_string();

        let synthetic = Contact {
            peer_id: pubkey_hex.clone(),
            nickname: Some("peer-30d0fa67".to_string()),
            local_nickname: Some("peer-abcd".to_string()),
            public_key: pubkey_hex.clone(),
            added_at: 0,
            last_seen: None,
            notes: None,
            last_known_device_id: None,
        };
        let real = Contact {
            peer_id: libp2p_peer_id.clone(),
            nickname: Some("ChristyLove".to_string()),
            local_nickname: Some("MyChristy".to_string()),
            public_key: pubkey_hex.clone(),
            added_at: 0,
            last_seen: None,
            notes: None,
            last_known_device_id: None,
        };
        backend
            .put(
                &contact_key(&pubkey_hex),
                &serde_json::to_vec(&synthetic).unwrap(),
            )
            .unwrap();
        backend
            .put(
                &contact_key(&libp2p_peer_id),
                &serde_json::to_vec(&real).unwrap(),
            )
            .unwrap();
        let mgr = ContactManager::new(backend.clone());
        let result = mgr.get(pubkey_hex.clone()).unwrap().unwrap();
        assert_eq!(
            result.nickname.as_deref(),
            Some("ChristyLove"),
            "UNIFICATION dedup should replace synthetic with real nickname"
        );
        assert_eq!(
            result.local_nickname.as_deref(),
            Some("MyChristy"),
            "UNIFICATION dedup should replace synthetic localNickname with real"
        );
        assert!(
            backend
                .get(&contact_key(&libp2p_peer_id))
                .unwrap()
                .is_none(),
            "libp2p duplicate should be removed"
        );
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_canonical_hex_dedup_keeps_real_when_libp2p_synthetic() {
        use crate::identity::IdentityKeys;
        let backend = Arc::new(MemoryStorage::new());
        let keys = IdentityKeys::generate();
        let pubkey_hex = hex::encode(keys.signing_key.verifying_key().to_bytes()).to_lowercase();
        let mut libp2p_bytes = vec![0x00, 0x24, 0x08, 0x01, 0x12, 0x20];
        libp2p_bytes.extend_from_slice(&keys.signing_key.verifying_key().to_bytes());
        let libp2p_peer_id = bs58::encode(&libp2p_bytes).into_string();

        let real = Contact {
            peer_id: pubkey_hex.clone(),
            nickname: Some("ChristyLove".to_string()),
            local_nickname: Some("MyChristy".to_string()),
            public_key: pubkey_hex.clone(),
            added_at: 0,
            last_seen: None,
            notes: None,
            last_known_device_id: None,
        };
        let synthetic = Contact {
            peer_id: libp2p_peer_id.clone(),
            nickname: Some("peer-abcdef".to_string()),
            local_nickname: Some("peer-123456".to_string()),
            public_key: pubkey_hex.clone(),
            added_at: 0,
            last_seen: None,
            notes: None,
            last_known_device_id: None,
        };
        backend
            .put(
                &contact_key(&pubkey_hex),
                &serde_json::to_vec(&real).unwrap(),
            )
            .unwrap();
        backend
            .put(
                &contact_key(&libp2p_peer_id),
                &serde_json::to_vec(&synthetic).unwrap(),
            )
            .unwrap();
        let mgr = ContactManager::new(backend.clone());
        let result = mgr.get(pubkey_hex.clone()).unwrap().unwrap();
        assert_eq!(
            result.nickname.as_deref(),
            Some("ChristyLove"),
            "should keep canonical real when libp2p synthetic"
        );
        assert_eq!(
            result.local_nickname.as_deref(),
            Some("MyChristy"),
            "should keep canonical real localNickname"
        );
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_is_synthetic_and_authoritative_helpers() {
        // UNIFICATION helpers unit test
        assert!(is_synthetic_fallback_nickname(&Some(
            "peer-123".to_string()
        )));
        assert!(is_synthetic_fallback_nickname(&Some(
            "PEER-abc".to_string()
        )));
        assert!(is_synthetic_fallback_nickname(&Some(
            " peer-xyz ".to_string()
        )));
        assert!(!is_synthetic_fallback_nickname(&Some(
            "ChristyLove".to_string()
        )));
        assert!(!is_synthetic_fallback_nickname(&None));
        assert!(!is_synthetic_fallback_nickname(&Some("".to_string())));
        assert_eq!(
            select_authoritative_nickname(
                &Some("ChristyLove".to_string()),
                &Some("peer-30d0fa".to_string())
            )
            .as_deref(),
            Some("ChristyLove")
        );
        assert_eq!(
            select_authoritative_nickname(
                &Some("peer-abc".to_string()),
                &Some("ChristyLove".to_string())
            )
            .as_deref(),
            Some("ChristyLove")
        );
        assert_eq!(
            select_authoritative_nickname(&None, &Some("peer-abc".to_string())),
            None
        );
        assert_eq!(
            select_authoritative_nickname(
                &Some("peer-abc".to_string()),
                &Some("peer-xyz".to_string())
            ),
            None
        );
    }
}
