// core/src/crypto/session_manager.rs
use super::ratchet::{Chain, RatchetKey, RatchetSession};
use crate::store::backend::StorageBackend;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};
use zeroize::Zeroize;

/// Manages ratchet sessions for multiple peer conversations.
pub struct RatchetSessionManager {
    sessions: HashMap<String, RatchetSession>,
    backend: Option<Arc<dyn StorageBackend>>,
}

impl Default for RatchetSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RatchetSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            backend: None,
        }
    }

    /// Create a manager backed by persistent storage.
    pub fn with_backend(backend: Arc<dyn StorageBackend>) -> Self {
        Self {
            sessions: HashMap::new(),
            backend: Some(backend),
        }
    }

    /// Save all sessions to the persistent backend.
    pub fn save(&self) -> Result<()> {
        if let Some(backend) = &self.backend {
            let mut json = self.serialize_sessions()?;
            backend
                .put(b"ratchet_sessions_v1", json.as_bytes())
                .map_err(|e| anyhow::anyhow!("Failed to save ratchet sessions: {}", e))?;
            // Zeroize the JSON string containing all session secrets
            use zeroize::Zeroize;
            json.zeroize();
        }
        Ok(())
    }

    /// Load sessions from the persistent backend.
    pub fn load(&mut self) -> Result<()> {
        if let Some(backend) = &self.backend {
            if let Some(bytes) = backend
                .get(b"ratchet_sessions_v1")
                .map_err(|e| anyhow::anyhow!("Failed to load ratchet sessions: {}", e))?
            {
                let json = String::from_utf8(bytes)
                    .map_err(|e| anyhow::anyhow!("Invalid ratchet session encoding: {}", e))?;
                self.deserialize_sessions(&json)?;
            }
        }
        Ok(())
    }

    /// Get or create a ratchet session for a peer (as sender).
    pub fn get_or_create_session(
        &mut self,
        peer_id: &str,
        our_signing_key: &ed25519_dalek::SigningKey,
        their_identity_public_x25519: &X25519PublicKey,
    ) -> Result<&mut RatchetSession> {
        match self.sessions.entry(peer_id.to_string()) {
            std::collections::hash_map::Entry::Occupied(e) => Ok(e.into_mut()),
            std::collections::hash_map::Entry::Vacant(e) => {
                let session =
                    RatchetSession::init_as_sender(our_signing_key, their_identity_public_x25519)?;
                Ok(e.insert(session))
            }
        }
    }

    /// Create a receiver session with the sender's identity key.
    pub fn create_receiver_session(
        &mut self,
        peer_id: &str,
        our_signing_key: &ed25519_dalek::SigningKey,
        sender_identity_public_x25519: &X25519PublicKey,
    ) -> Result<&mut RatchetSession> {
        let session =
            RatchetSession::init_as_receiver(our_signing_key, sender_identity_public_x25519)?;
        match self.sessions.entry(peer_id.to_string()) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                e.insert(session);
                Ok(e.into_mut())
            }
            std::collections::hash_map::Entry::Vacant(e) => Ok(e.insert(session)),
        }
    }

    /// Get or create a hybrid ratchet session for a peer (as sender).
    pub fn get_or_create_session_hybrid(
        &mut self,
        peer_id: &str,
        our_signing_key: &ed25519_dalek::SigningKey,
        our_x25519_secret: &x25519_dalek::StaticSecret,
        our_bundle: &crate::identity::PublicKeyBundle,
        their_bundle: &crate::identity::PublicKeyBundle,
    ) -> Result<&mut RatchetSession> {
        match self.sessions.entry(peer_id.to_string()) {
            std::collections::hash_map::Entry::Occupied(e) => Ok(e.into_mut()),
            std::collections::hash_map::Entry::Vacant(e) => {
                let (suite, hash) = crate::crypto::negotiation::negotiate_suite(
                    &our_bundle.supported_suites,
                    &their_bundle.supported_suites,
                    &our_bundle.ed25519_public,
                    &their_bundle.ed25519_public,
                )?;

                let session = if suite == 0x03 {
                    // Current hybrid derivation: static-static DH sender
                    // authentication folded into the root key ("iron-core
                    // session-root v3 2026-08").
                    RatchetSession::init_as_sender_hybrid(our_x25519_secret, their_bundle, hash)?
                } else if suite == 0x02 {
                    // ORIGINAL hybrid derivation, preserved byte-for-byte for
                    // a peer that only ever advertises suite 0x02 ("iron-core
                    // session-root v2 2026-07", no sender-authentication DH
                    // term). Never merge this into the 0x03 branch above --
                    // see crypto::negotiation::HYBRID_SUITE_IDS.
                    RatchetSession::init_as_sender_hybrid_suite02(
                        our_signing_key,
                        their_bundle,
                        hash,
                    )?
                } else {
                    let their_x25519 = crate::crypto::encrypt::ed25519_public_to_x25519(
                        &their_bundle.ed25519_public,
                    )?;
                    let mut s = RatchetSession::init_as_sender(our_signing_key, &their_x25519)?;
                    s.negotiated_suite = Some(suite);
                    s.transcript_hash = Some(hash);
                    s
                };

                Ok(e.insert(session))
            }
        }
    }

    /// Build a receiver hybrid session WITHOUT touching the session map.
    ///
    /// Split out of `create_receiver_session_hybrid` so a caller can attempt a
    /// decrypt against the candidate first and commit it only if that decrypt
    /// succeeds. Writing before proving the session works lets a replayed
    /// envelope destroy a healthy session: the rebuild unconditionally
    /// overwrites the live entry, whether or not the rebuilt session can
    /// decrypt anything.
    #[allow(clippy::too_many_arguments)]
    pub fn build_receiver_session_hybrid(
        our_signing_key: &ed25519_dalek::SigningKey,
        our_x25519_secret: &x25519_dalek::StaticSecret,
        our_mlkem_keypair: &crate::crypto::pq::MlKem768KeyPair,
        our_bundle: &crate::identity::PublicKeyBundle,
        their_bundle: &crate::identity::PublicKeyBundle,
        hct_opt: Option<&crate::crypto::pq::hybrid::HybridCiphertext>,
    ) -> Result<&mut RatchetSession> {
        let (suite, hash) = crate::crypto::negotiation::negotiate_suite(
            &their_bundle.supported_suites, // Initiator's suites
            &our_bundle.supported_suites,   // Responder's suites
            &their_bundle.ed25519_public,   // Initiator's ed25519
            &our_bundle.ed25519_public,     // Responder's ed25519
        )?;

        let sender_x25519 = x25519_dalek::PublicKey::from(their_bundle.x25519_public);
        let session = if suite == 0x03 {
            // Current hybrid derivation -- see get_or_create_session_hybrid.
            if let Some(hct) = hct_opt {
                RatchetSession::init_as_receiver_hybrid(
                    our_signing_key,
                    our_x25519_secret,
                    our_mlkem_keypair,
                    their_bundle,
                    hct,
                    hash,
                )?
            } else {
                anyhow::bail!("Suite 0x03 requires hybrid ciphertext for receiver initialization");
            }
        } else if suite == 0x02 {
            // ORIGINAL hybrid derivation, preserved for a peer that only ever
            // advertises suite 0x02 -- see get_or_create_session_hybrid.
            if let Some(hct) = hct_opt {
                RatchetSession::init_as_receiver_hybrid_suite02(
                    our_signing_key,
                    our_x25519_secret,
                    our_mlkem_keypair,
                    their_bundle,
                    hct,
                    hash,
                )?
            } else {
                anyhow::bail!("Suite 0x02 requires hybrid ciphertext for receiver initialization");
            }
        } else {
            let mut s = RatchetSession::init_as_receiver(our_signing_key, &sender_x25519)?;
            s.negotiated_suite = Some(suite);
            s.transcript_hash = Some(hash);
            s
        };

        Ok(session)
    }

    /// Create (or replace) a receiver hybrid session and store it immediately.
    ///
    /// Prefer `build_receiver_session_hybrid` + `insert_session` on any path
    /// where an already-stored session must survive a failed rebuild.
    #[allow(clippy::too_many_arguments)]
    pub fn create_receiver_session_hybrid(
        &mut self,
        peer_id: &str,
        our_signing_key: &ed25519_dalek::SigningKey,
        our_x25519_secret: &x25519_dalek::StaticSecret,
        our_mlkem_keypair: &crate::crypto::pq::MlKem768KeyPair,
        our_bundle: &crate::identity::PublicKeyBundle,
        their_bundle: &crate::identity::PublicKeyBundle,
        hct_opt: Option<&crate::crypto::pq::hybrid::HybridCiphertext>,
    ) -> Result<&mut RatchetSession> {
        let session = Self::build_receiver_session_hybrid(
            our_signing_key,
            our_x25519_secret,
            our_mlkem_keypair,
            our_bundle,
            their_bundle,
            hct_opt,
        )?;
        Ok(self.insert_session(peer_id, session))
    }

    /// Insert (or replace) a session for a peer and return a mutable borrow.
    pub fn insert_session(
        &mut self,
        peer_id: &str,
        session: RatchetSession,
    ) -> &mut RatchetSession {
        match self.sessions.entry(peer_id.to_string()) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                e.insert(session);
                e.into_mut()
            }
            std::collections::hash_map::Entry::Vacant(e) => e.insert(session),
        }
    }

    /// Get an existing session for a peer (returns None if no session exists).
    pub fn get_session(&self, peer_id: &str) -> Option<&RatchetSession> {
        self.sessions.get(peer_id)
    }

    /// Get a mutable session for a peer.
    pub fn get_session_mut(&mut self, peer_id: &str) -> Option<&mut RatchetSession> {
        self.sessions.get_mut(peer_id)
    }

    /// Remove a session (e.g., on peer disconnect or session timeout).
    pub fn remove_session(&mut self, peer_id: &str) {
        self.sessions.remove(peer_id);
    }

    /// Number of active sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Check if a ratchet session exists for a peer.
    pub fn has_session(&self, peer_id: &str) -> bool {
        self.sessions.contains_key(peer_id)
    }

    /// Serialize all sessions to JSON for persistence across app restarts.
    pub fn serialize_sessions(&self) -> Result<String> {
        let serializable: HashMap<String, SerializableRatchetSession> = self
            .sessions
            .iter()
            .map(|(k, v)| (k.clone(), SerializableRatchetSession::from_session(v)))
            .collect();
        serde_json::to_string(&serializable)
            .map_err(|e| anyhow::anyhow!("Failed to serialize ratchet sessions: {}", e))
    }

    /// Deserialize sessions from JSON and merge into the manager.
    /// Existing sessions for the same peer_id are NOT overwritten.
    /// Per-entry conversion failures are silently skipped; this is meant
    /// for best-effort startup loads (`load()`). Use
    /// `deserialize_sessions_strict` for contexts (e.g. backup import)
    /// that must fail on any corrupted entry instead of dropping it.
    pub fn deserialize_sessions(&mut self, json: &str) -> Result<()> {
        let map: HashMap<String, SerializableRatchetSession> = serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize ratchet sessions: {}", e))?;

        for (peer_id, serializable) in map {
            if self.sessions.contains_key(&peer_id) {
                continue; // Don't overwrite existing in-memory sessions
            }
            if let Ok(session) = serializable.into_session() {
                self.sessions.insert(peer_id, session);
            }
        }
        Ok(())
    }

    /// Like `deserialize_sessions`, but aborts on the first per-entry
    /// conversion failure instead of skipping it. Use where a corrupted
    /// entry must fail the whole operation, e.g. validating and restoring
    /// a backup.
    pub fn deserialize_sessions_strict(&mut self, json: &str) -> Result<()> {
        let map: HashMap<String, SerializableRatchetSession> = serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize ratchet sessions: {}", e))?;

        let mut decoded = Vec::new();

        for (peer_id, serializable) in map {
            if self.sessions.contains_key(&peer_id) {
                continue; // Don't overwrite existing in-memory sessions
            }
            let session = serializable.into_session()?;
            decoded.push((peer_id, session));
        }

        for (peer_id, session) in decoded {
            self.sessions.insert(peer_id, session);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Session serialization (for persistence across app restarts)
// ---------------------------------------------------------------------------

/// Serializable representation of a `RatchetSession`.
///
/// This is the wire/disk format. Secret keys are stored as hex strings.
/// When loading, the caller must ensure the storage is encrypted at rest.
#[derive(Serialize, Deserialize)]
pub struct SerializableRatchetSession {
    /// Our DH ratchet secret key (hex-encoded X25519 static secret bytes)
    pub our_dh_secret_hex: String,
    /// Our DH ratchet public key (hex-encoded)
    pub our_dh_public_hex: String,
    /// Their DH ratchet public key (hex-encoded), if known
    pub their_dh_public_hex: Option<String>,
    /// Root key (hex-encoded)
    pub root_key_hex: String,
    /// Sending chain key (hex-encoded) and index, if initialized
    pub sending_chain: Option<ChainState>,
    /// Receiving chain key (hex-encoded) and index, if initialized
    pub receiving_chain: Option<ChainState>,
    /// DH ratchet step count
    pub dh_step_count: u32,
    /// Whether session is fully initialized
    pub initialized: bool,
    /// Whether identity secret is still held (for first DH step)
    pub has_identity_secret: bool,
    /// Identity secret (hex-encoded), only present before first DH ratchet
    pub identity_secret_hex: Option<String>,
    /// The negotiated cryptographic suite (from PQC-04)
    pub negotiated_suite: Option<u8>,
    /// The transcript hash binding the session to the negotiation (hex-encoded, from PQC-04)
    pub transcript_hash_hex: Option<String>,
    #[serde(default)]
    pub peer_confirmed: bool,
    pub bootstrap_hct_ephemeral_hex: Option<String>,
    pub bootstrap_hct_mlkem_hex: Option<String>,
    /// Skipped ratchet message keys, as (their_dh_hex, message_index, key_hex).
    /// `#[serde(default)]` so sessions persisted before this field existed
    /// still deserialize (with no skipped keys, matching the old behavior).
    #[serde(default)]
    pub skipped_keys: Vec<(String, u32, String)>,
}

/// Serializable chain state.
#[derive(Serialize, Deserialize)]
pub struct ChainState {
    pub chain_key_hex: String,
    pub index: u32,
}

impl SerializableRatchetSession {
    /// Create a serializable snapshot from a live session.
    pub fn from_session(session: &RatchetSession) -> Self {
        Self {
            our_dh_secret_hex: hex::encode(session.our_dh_secret_bytes()),
            our_dh_public_hex: hex::encode(session.our_public_key()),
            their_dh_public_hex: session.their_public_key().map(hex::encode),
            root_key_hex: hex::encode(session.root_key_bytes()),
            sending_chain: session
                .sending_chain_state()
                .map(|(key, index)| ChainState {
                    chain_key_hex: hex::encode(key),
                    index,
                }),
            receiving_chain: session
                .receiving_chain_state()
                .map(|(key, index)| ChainState {
                    chain_key_hex: hex::encode(key),
                    index,
                }),
            dh_step_count: session.dh_step_count(),
            initialized: session.is_initialized(),
            has_identity_secret: session.has_identity_secret(),
            identity_secret_hex: session.identity_secret_bytes().map(hex::encode),
            negotiated_suite: session.negotiated_suite,
            transcript_hash_hex: session.transcript_hash.map(hex::encode),
            peer_confirmed: session.peer_confirmed,
            bootstrap_hct_ephemeral_hex: session
                .bootstrap_hct
                .as_ref()
                .map(|h| hex::encode(h.x25519_ephemeral_public)),
            bootstrap_hct_mlkem_hex: session
                .bootstrap_hct
                .as_ref()
                .map(|h| hex::encode(&h.mlkem_ciphertext)),
            skipped_keys: session
                .skipped_keys_snapshot()
                .into_iter()
                .map(|(dh, idx, key)| (hex::encode(dh), idx, hex::encode(key)))
                .collect(),
        }
    }

    /// Reconstruct a live session from a serialized snapshot.
    pub fn into_session(self) -> Result<RatchetSession> {
        let mut our_dh_secret_bytes = hex::decode(&self.our_dh_secret_hex)
            .map_err(|e| anyhow::anyhow!("Invalid our_dh_secret_hex: {}", e))?;
        if our_dh_secret_bytes.len() != 32 {
            bail!("our_dh_secret must be 32 bytes");
        }
        let mut secret_arr = [0u8; 32];
        secret_arr.copy_from_slice(&our_dh_secret_bytes);
        our_dh_secret_bytes.zeroize();
        let our_dh_secret = X25519StaticSecret::from(secret_arr);
        secret_arr.zeroize();

        let our_dh_public_bytes = hex::decode(&self.our_dh_public_hex)
            .map_err(|e| anyhow::anyhow!("Invalid our_dh_public_hex: {}", e))?;
        if our_dh_public_bytes.len() != 32 {
            bail!("our_dh_public must be 32 bytes");
        }
        let mut pub_arr = [0u8; 32];
        pub_arr.copy_from_slice(&our_dh_public_bytes);
        let our_dh_public = X25519PublicKey::from(pub_arr);

        let their_dh_public = match &self.their_dh_public_hex {
            Some(hex_str) => {
                let bytes = hex::decode(hex_str)
                    .map_err(|e| anyhow::anyhow!("Invalid their_dh_public_hex: {}", e))?;
                if bytes.len() != 32 {
                    bail!("their_dh_public must be 32 bytes");
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Some(X25519PublicKey::from(arr))
            }
            None => None,
        };

        let mut root_key_bytes = hex::decode(&self.root_key_hex)
            .map_err(|e| anyhow::anyhow!("Invalid root_key_hex: {}", e))?;
        if root_key_bytes.len() != 32 {
            bail!("root_key must be 32 bytes");
        }
        let mut rk_arr = [0u8; 32];
        rk_arr.copy_from_slice(&root_key_bytes);
        root_key_bytes.zeroize();
        let root_key = RatchetKey::from_bytes(rk_arr);

        let sending_chain = self
            .sending_chain
            .map(|cs| -> anyhow::Result<Chain> {
                let mut ck = [0u8; 32];
                hex::decode_to_slice(&cs.chain_key_hex, &mut ck)
                    .map_err(|e| anyhow::anyhow!("Corrupt sending chain_key_hex in DB: {}", e))?;
                Ok(Chain::new_with_index(RatchetKey::from_bytes(ck), cs.index))
            })
            .transpose()?;

        let receiving_chain = self
            .receiving_chain
            .map(|cs| -> anyhow::Result<Chain> {
                let mut ck = [0u8; 32];
                hex::decode_to_slice(&cs.chain_key_hex, &mut ck)
                    .map_err(|e| anyhow::anyhow!("Corrupt receiving chain_key_hex in DB: {}", e))?;
                Ok(Chain::new_with_index(RatchetKey::from_bytes(ck), cs.index))
            })
            .transpose()?;

        let our_identity_secret = if let Some(ref hex_str) = self.identity_secret_hex {
            let mut bytes = hex::decode(hex_str)
                .map_err(|e| anyhow::anyhow!("Invalid identity_secret_hex: {}", e))?;
            if bytes.len() != 32 {
                bail!("identity_secret must be 32 bytes");
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            bytes.zeroize();
            let secret = X25519StaticSecret::from(arr);
            arr.zeroize();
            Some(secret)
        } else {
            None
        };

        let transcript_hash = if let Some(ref hex_str) = self.transcript_hash_hex {
            let bytes = hex::decode(hex_str)
                .map_err(|e| anyhow::anyhow!("Invalid transcript_hash_hex: {}", e))?;
            if bytes.len() != 32 {
                bail!("transcript_hash must be 32 bytes");
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            Some(arr)
        } else {
            None
        };

        let bootstrap_hct = if let (Some(e_hex), Some(m_hex)) = (
            self.bootstrap_hct_ephemeral_hex,
            self.bootstrap_hct_mlkem_hex,
        ) {
            let e_bytes = hex::decode(e_hex).map_err(|e| anyhow::anyhow!("bad eph: {}", e))?;
            let m_bytes = hex::decode(m_hex).map_err(|e| anyhow::anyhow!("bad mlkem: {}", e))?;
            if e_bytes.len() != 32 || m_bytes.len() != 1088 {
                anyhow::bail!("bad hct lengths");
            }
            let mut e_arr = [0u8; 32];
            e_arr.copy_from_slice(&e_bytes);
            Some(crate::crypto::pq::hybrid::HybridCiphertext {
                x25519_ephemeral_public: e_arr,
                mlkem_ciphertext: m_bytes,
            })
        } else {
            None
        };

        let mut skipped_keys = HashMap::with_capacity(self.skipped_keys.len());
        for (dh_hex, idx, key_hex) in &self.skipped_keys {
            let dh_bytes = hex::decode(dh_hex)
                .map_err(|e| anyhow::anyhow!("Invalid skipped_keys dh hex: {}", e))?;
            let key_bytes = hex::decode(key_hex)
                .map_err(|e| anyhow::anyhow!("Invalid skipped_keys key hex: {}", e))?;
            if dh_bytes.len() != 32 || key_bytes.len() != 32 {
                bail!("skipped_keys entries must be 32 bytes each");
            }
            let mut dh_arr = [0u8; 32];
            dh_arr.copy_from_slice(&dh_bytes);
            let mut key_arr = [0u8; 32];
            key_arr.copy_from_slice(&key_bytes);
            skipped_keys.insert((dh_arr, *idx), RatchetKey::from_bytes(key_arr));
        }

        Ok(RatchetSession::reconstruct(
            our_dh_secret,
            our_dh_public,
            their_dh_public,
            root_key,
            sending_chain,
            receiving_chain,
            self.dh_step_count,
            self.initialized,
            our_identity_secret,
            self.negotiated_suite,
            transcript_hash,
            self.peer_confirmed,
            bootstrap_hct,
            None, // pq_our_keypair
            None, // pq_prev_keypair
            None, // pq_their_encaps_key
            None, // pq_pending_ct
            None, // pq_pending_sent
            None, // pq_pending_recv
            None, // pq_last_mixed_fp
            0,    // transition_counter
            skipped_keys,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::backend::MemoryStorage;
    use ed25519_dalek::SigningKey;
    use rand::RngCore;

    fn generate_signing_key() -> SigningKey {
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        SigningKey::from_bytes(&bytes)
    }

    #[test]
    fn test_manager_persistence_roundtrip() {
        let backend = Arc::new(MemoryStorage::new());
        let mut manager = RatchetSessionManager::with_backend(backend.clone());

        let our_key = generate_signing_key();
        let their_pub = X25519PublicKey::from([1u8; 32]);
        let peer_id = "peer-1";

        // Create a session
        manager
            .get_or_create_session(peer_id, &our_key, &their_pub)
            .unwrap();
        assert_eq!(manager.session_count(), 1);

        // Save
        manager.save().unwrap();

        // Create a new manager with same backend
        let mut manager2 = RatchetSessionManager::with_backend(backend);
        assert_eq!(manager2.session_count(), 0);

        // Load
        manager2.load().unwrap();
        assert_eq!(manager2.session_count(), 1);
        assert!(manager2.get_session(peer_id).is_some());
    }

    #[test]
    fn test_skipped_keys_survive_persistence_roundtrip() {
        // Regression test for PQC_RATCHET_SKIPPED_KEYS_NOT_PERSISTED: a skipped
        // message key held at save time must still be present after reload,
        // otherwise a legitimately-delayed message becomes undecryptable.
        let our_key = generate_signing_key();
        let their_pub = X25519PublicKey::from([2u8; 32]);
        let mut manager = RatchetSessionManager::new();
        manager
            .get_or_create_session("peer-skip", &our_key, &their_pub)
            .unwrap();
        let session = manager.get_session("peer-skip").unwrap();
        assert!(
            session.skipped_keys_snapshot().is_empty(),
            "freshly-created session should have no skipped keys yet"
        );

        let mut serializable = SerializableRatchetSession::from_session(session);
        assert!(serializable.skipped_keys.is_empty());

        // Inject a fake skipped key entry, simulating one held at save time.
        let fake_dh = hex::encode([7u8; 32]);
        let fake_key = hex::encode([9u8; 32]);
        serializable
            .skipped_keys
            .push((fake_dh.clone(), 42, fake_key.clone()));

        let reconstructed = serializable.into_session().unwrap();
        let snapshot = reconstructed.skipped_keys_snapshot();
        assert_eq!(
            snapshot.len(),
            1,
            "the injected skipped key must survive into_session()"
        );
        let (dh, idx, key) = snapshot[0];
        assert_eq!(hex::encode(dh), fake_dh);
        assert_eq!(idx, 42);
        assert_eq!(hex::encode(key), fake_key);
    }

    #[test]
    fn test_deserialize_sessions_strict_rejects_corrupted_entry() {
        let mut manager = RatchetSessionManager::new();
        let our_key = generate_signing_key();
        let their_pub = X25519PublicKey::from([1u8; 32]);

        manager
            .get_or_create_session("peer-good", &our_key, &their_pub)
            .unwrap();
        let mut good_serializable: HashMap<String, SerializableRatchetSession> = manager
            .sessions
            .iter()
            .map(|(k, v)| (k.clone(), SerializableRatchetSession::from_session(v)))
            .collect();

        // Corrupt the one entry's hex field so `into_session()` fails.
        let mut corrupted = good_serializable.remove("peer-good").unwrap();
        corrupted.our_dh_secret_hex = "not-hex".to_string();
        let mut map = HashMap::new();
        map.insert("peer-corrupted".to_string(), corrupted);
        let json = serde_json::to_string(&map).unwrap();

        // Lenient path: silently skips the corrupted entry.
        let mut lenient = RatchetSessionManager::new();
        lenient.deserialize_sessions(&json).unwrap();
        assert_eq!(lenient.session_count(), 0);

        // Strict path: fails instead of dropping the entry.
        let mut strict = RatchetSessionManager::new();
        assert!(strict.deserialize_sessions_strict(&json).is_err());
        assert_eq!(strict.session_count(), 0);
    }
}
