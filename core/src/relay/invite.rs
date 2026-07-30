//! Invite System — cryptographic invites with web-of-trust tracking
//!
//! # Invites carry a seed ledger — routing only, no identity
//!
//! Since v0.4.0 an [`InviteToken`] carries a [`SeedLedgerEntry`] snapshot of the
//! inviter's ledger, including the inviter's own dialable address, so a fresh
//! invitee starts with a warm view of the mesh instead of an empty one. This is
//! the whole cold-start story: seed delivery is invite/QR only, there is no DNS
//! seed and no shipped node list.
//!
//! **A seed entry is a bare multiaddr and nothing else.** No `peer_id`, no
//! public key, no nickname, no topics, no success/failure counters, no
//! `last_seen`. An invite hands over *where to knock*, not *who lives there*:
//! every one of those fields is identity or behavioural metadata about a third
//! party who never consented to appearing in someone else's invite. The invitee
//! dials the bare address, completes the Noise handshake and learns the peer's
//! identity from Identify at connect time, exactly as it would for an mDNS- or
//! gossip-learned peer; `LedgerManager::annotate_identity` is the path for
//! attaching identity after the fact.
//!
//! This is safe because transport peer identity is not what secures messages.
//! Confidentiality is per-contact X25519 / XChaCha20-Poly1305 established out of
//! band from public keys, so connecting to an unintended node at a given address
//! leaks nothing and decrypts nothing. Omitting `peer_id` does forgo dial-time
//! identity pinning, which is an availability consideration rather than a
//! confidentiality one.
//!
//! # Residual privacy note (reduced, not eliminated)
//!
//! An invite still discloses the inviter's IP address and up to
//! [`MAX_SEED_LEDGER_ENTRIES`] node IPs to whoever holds it, and QR codes get
//! photographed, forwarded and posted publicly. Treat an invite as though its
//! address list were public. This is unavoidable if the invite is to be useful
//! at all, and it is now bare routing data with no identities attached. Invite
//! UI copy must say so plainly; see also `docs/BOOTSTRAP.md`.
//!
//! The seed ledger is covered by [`InviteToken::get_signable_data`]. That is not
//! optional: if it were outside the signature, anyone who intercepted or relayed
//! an invite could inject attacker-controlled multiaddrs that a fresh node dials
//! on first launch.
//!
//! # Verification
//!
//! [`InviteToken::verify`] is the only real validity gate: Ed25519 over
//! [`InviteToken::get_signable_data`] with `inviter_public_key`, plus ML-DSA-65
//! over the same bytes when the token carries a post-quantum key and signature.
//! The signed bytes are domain separated with [`INVITE_SIGNING_DOMAIN`].
//! [`InviteToken::is_valid`] is a boolean wrapper over it.
//!
//! No code may treat any field of a token as authenticated — `seed_ledger`
//! above all — before `verify` has returned `Ok`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use web_time::{SystemTime, UNIX_EPOCH};

pub use crate::store::ledger_entry::{SeedLedgerEntry, MAX_SEED_LEDGER_ENTRIES};

/// Practical payload ceiling for a QR code in byte mode (version 40, ECC level
/// L). Higher error-correction levels are smaller still, so this is an upper
/// bound, not a target.
pub const QR_BYTE_BUDGET: usize = 2953;

/// Format tag on the encoded invite payload. Bumping this is how a future
/// encoding change stays distinguishable from the current one.
const QR_PAYLOAD_PREFIX: &str = "SCI1:";

/// Domain-separation tag prefixed to the bytes an invite signature covers.
///
/// Without it the signed blob is a bare bincode struct, so a signature produced
/// over some other bincode-encoded structure with a coincidentally compatible
/// layout could be replayed as an invite signature (and vice versa). The tag
/// binds every invite signature to the invite context and to this version of
/// the signed-byte layout.
///
/// WIRE BREAK: this changes the signed byte string on top of the seed-ledger
/// change documented on [`InviteToken::get_signable_data`]. Signatures minted
/// by any earlier build do not verify here and vice versa. Invites are short
/// lived (30 day default expiry), so this is an accepted pre-1.0 break rather
/// than a migration; bump the version suffix on any future layout change.
pub const INVITE_SIGNING_DOMAIN: &[u8] = b"SCM-INVITE-v1";

/// Encoded length of an ML-DSA-65 public key, per FIPS 204.
///
/// Mirrors the length `crate::crypto::pq::mldsa::verify` enforces. Checked here
/// only so a wrong-sized key reports as [`InviteError::MalformedKey`] rather
/// than being indistinguishable from a genuine verification failure.
const MLDSA65_PUBLIC_KEY_LEN: usize = 1952;

/// Invite system errors
#[derive(Debug, Error)]
pub enum InviteError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Signature verification failed")]
    VerificationFailed,
    #[error("Token expired")]
    TokenExpired,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Invalid inviter")]
    InvalidInviter,
    #[error("Encoded invite is {0} bytes, over the {1} byte QR budget")]
    PayloadTooLarge(usize, usize),
    #[error("Malformed invite payload: {0}")]
    MalformedPayload(String),
    /// The token carries no Ed25519 signature at all.
    #[error("Invite carries no signature")]
    MissingSignature,
    /// Signature bytes are present but structurally wrong (bad length).
    #[error("Malformed signature: {0}")]
    MalformedSignature(String),
    /// Key material is the wrong length or not a valid point.
    #[error("Malformed key: {0}")]
    MalformedKey(String),
    /// Policy demanded a post-quantum signature and the token has none.
    #[error("Post-quantum signature required but not present")]
    PqSignatureRequired,
    /// The ML-DSA-65 signature is present but does not verify.
    #[error("Post-quantum signature verification failed")]
    PqVerificationFailed,
}

/// Build the seed ledger for an invite: the inviter's own address first and
/// never evicted, then the ranked peers, deduped on the `/p2p/`-stripped
/// multiaddr and capped at [`MAX_SEED_LEDGER_ENTRIES`].
///
/// `ranked_peers` is expected to come from
/// `LedgerManager::export_seed_entries`, i.e. already ordered by
/// `get_preferred_relays()` ranking. Any `/p2p/` component is stripped here as
/// well as at import: a peer id must not ride along inside the multiaddr string
/// after we deliberately removed the dedicated field.
pub fn build_seed_ledger(
    inviter_entry: SeedLedgerEntry,
    ranked_peers: Vec<SeedLedgerEntry>,
) -> Vec<SeedLedgerEntry> {
    fn stripped(entry: &SeedLedgerEntry) -> SeedLedgerEntry {
        SeedLedgerEntry {
            multiaddr: match entry.multiaddr.find("/p2p/") {
                Some(idx) => entry.multiaddr[..idx].to_string(),
                None => entry.multiaddr.clone(),
            },
        }
    }

    let first = stripped(&inviter_entry);
    let mut seen = vec![first.multiaddr.clone()];
    let mut out = vec![first];

    for peer in ranked_peers {
        if out.len() >= MAX_SEED_LEDGER_ENTRIES {
            break;
        }
        let peer = stripped(&peer);
        if seen.contains(&peer.multiaddr) {
            continue;
        }
        seen.push(peer.multiaddr.clone());
        out.push(peer);
    }

    out
}

/// Cryptographic invite token signed by an inviter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteToken {
    /// ID of the inviter (who sent the invite)
    pub inviter_id: String,
    /// Inviter's Ed25519 public key (for verification)
    pub inviter_public_key: Vec<u8>,
    /// ID of the invitee (who can use this invite)
    pub invitee_id: String,
    /// Unix timestamp when token was created
    pub created_at: u64,
    /// Unix timestamp when token expires
    pub expires_at: u64,
    /// Ed25519 signature over the token data
    pub signature: Vec<u8>,
    /// Optional metadata/purpose
    pub metadata: Option<String>,
    /// ML-DSA-65 public key (v2 tokens)
    #[serde(default)]
    pub pq_public_key: Option<Vec<u8>>,
    /// ML-DSA-65 signature (v2 tokens)
    #[serde(default)]
    pub pq_signature: Option<Vec<u8>>,
    /// Snapshot of the inviter's ledger, including the inviter's own dialable
    /// address, so a fresh invitee starts with a warm view of the mesh.
    ///
    /// Covered by [`Self::get_signable_data`] — see the module-level security
    /// note. `#[serde(default)]` keeps self-describing (JSON) v1/v2 tokens
    /// deserializable; the bincode path is handled explicitly in
    /// [`Self::from_bytes`], since bincode is not self-describing and
    /// `serde(default)` alone does nothing for it.
    #[serde(default)]
    pub seed_ledger: Vec<SeedLedgerEntry>,
}

/// The pre-v0.4.0 wire shape of [`InviteToken`], i.e. without `seed_ledger`.
///
/// Only used as a bincode fallback in [`InviteToken::from_bytes`]. bincode is
/// not self-describing, so an old token's bytes end exactly where the new
/// struct expects the seed-ledger length prefix and fail with EOF; decoding
/// them requires a struct with the old field list.
#[derive(Deserialize)]
struct LegacyInviteToken {
    inviter_id: String,
    inviter_public_key: Vec<u8>,
    invitee_id: String,
    created_at: u64,
    expires_at: u64,
    signature: Vec<u8>,
    metadata: Option<String>,
    #[serde(default)]
    pq_public_key: Option<Vec<u8>>,
    #[serde(default)]
    pq_signature: Option<Vec<u8>>,
}

impl InviteToken {
    /// Create a new unsigned invite token
    pub fn new(inviter_id: String, inviter_public_key: Vec<u8>, invitee_id: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            inviter_id,
            inviter_public_key,
            invitee_id,
            created_at: now,
            expires_at: now + 30 * 24 * 3600, // 30 days default
            signature: Vec::new(),
            metadata: None,
            pq_public_key: None,
            pq_signature: None,
            seed_ledger: Vec::new(),
        }
    }

    /// Set custom expiry duration
    pub fn with_expiry(mut self, duration_secs: u64) -> Self {
        self.expires_at = self.created_at + duration_secs;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set signature
    pub fn with_signature(mut self, signature: Vec<u8>) -> Self {
        self.signature = signature;
        self
    }

    /// Set PQ signatures
    pub fn with_pq_signature(mut self, pq_pubkey: Vec<u8>, pq_sig: Vec<u8>) -> Self {
        self.pq_public_key = Some(pq_pubkey);
        self.pq_signature = Some(pq_sig);
        self
    }

    /// Attach a seed ledger, truncated to [`MAX_SEED_LEDGER_ENTRIES`].
    ///
    /// Truncation keeps the head of the list, so a ledger built by
    /// [`build_seed_ledger`] can never lose the inviter's own address.
    /// Must be called BEFORE signing: the seed ledger is signed data.
    pub fn with_seed_ledger(mut self, mut entries: Vec<SeedLedgerEntry>) -> Self {
        entries.truncate(MAX_SEED_LEDGER_ENTRIES);
        self.seed_ledger = entries;
        self
    }

    /// Cryptographically verify the token: expiry, Ed25519, and ML-DSA-65 when
    /// the token carries a post-quantum key and signature.
    ///
    /// This is the only real validity gate. `Ok(())` means the inviter named by
    /// `inviter_public_key` actually signed every field covered by
    /// [`Self::get_signable_data`] — including `seed_ledger`, whose addresses a
    /// fresh invitee dials on first launch.
    ///
    /// A token that has a post-quantum key but no post-quantum signature is
    /// accepted here (Ed25519 alone). Use [`Self::verify_with_policy`] with
    /// `require_pq = true` to refuse that downgrade.
    pub fn verify(&self) -> Result<(), InviteError> {
        self.verify_with_policy(false)
    }

    /// [`Self::verify`], plus a policy check that the token carries a
    /// post-quantum signature when `require_pq` is set.
    pub fn verify_with_policy(&self, require_pq: bool) -> Result<(), InviteError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now >= self.expires_at {
            return Err(InviteError::TokenExpired);
        }

        if self.signature.is_empty() {
            return Err(InviteError::MissingSignature);
        }

        // Both algorithms sign exactly these bytes.
        let signed = self.get_signable_data()?;

        self.verify_ed25519(&signed)?;
        self.verify_pq(&signed, require_pq)
    }

    /// Ed25519 verification of [`Self::signature`] over `signed` using
    /// [`Self::inviter_public_key`].
    fn verify_ed25519(&self, signed: &[u8]) -> Result<(), InviteError> {
        use ed25519_dalek::Verifier;

        let key_bytes: &[u8; 32] = self.inviter_public_key.as_slice().try_into().map_err(|_| {
            InviteError::MalformedKey(format!(
                "inviter_public_key is {} bytes, expected 32",
                self.inviter_public_key.len()
            ))
        })?;
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(key_bytes)
            .map_err(|e| InviteError::MalformedKey(format!("inviter_public_key: {}", e)))?;

        let sig_bytes: &[u8; 64] = self.signature.as_slice().try_into().map_err(|_| {
            InviteError::MalformedSignature(format!(
                "Ed25519 signature is {} bytes, expected 64",
                self.signature.len()
            ))
        })?;
        let signature = ed25519_dalek::Signature::from_bytes(sig_bytes);

        verifying_key
            .verify(signed, &signature)
            .map_err(|_| InviteError::VerificationFailed)
    }

    /// ML-DSA-65 verification of [`Self::pq_signature`] over `signed` using
    /// [`Self::pq_public_key`], and the `require_pq` downgrade policy.
    ///
    /// Note `pq_public_key` is covered by the Ed25519 signature but
    /// `pq_signature` is not (it cannot be, it signs the same bytes), so a
    /// post-quantum signature with no accompanying key is unverifiable and is
    /// rejected rather than silently skipped.
    fn verify_pq(&self, signed: &[u8], require_pq: bool) -> Result<(), InviteError> {
        match (self.pq_public_key.as_deref(), self.pq_signature.as_deref()) {
            (Some(pq_key), Some(pq_sig)) => {
                if pq_key.len() != MLDSA65_PUBLIC_KEY_LEN {
                    return Err(InviteError::MalformedKey(format!(
                        "pq_public_key is {} bytes, expected {}",
                        pq_key.len(),
                        MLDSA65_PUBLIC_KEY_LEN
                    )));
                }
                crate::crypto::pq::mldsa::verify(pq_key, signed, pq_sig)
                    .map_err(|_| InviteError::PqVerificationFailed)
            }
            (None, Some(_)) => Err(InviteError::MalformedKey(
                "pq_signature present without pq_public_key".to_string(),
            )),
            (_, None) => {
                if require_pq {
                    return Err(InviteError::PqSignatureRequired);
                }
                tracing::warn!(
                    inviter_id = %self.inviter_id,
                    invitee_id = %self.invitee_id,
                    "AUDIT: accepted legacy single-signature (non-PQ) invite"
                );
                Ok(())
            }
        }
    }

    /// Boolean wrapper over [`Self::verify_with_policy`], for call sites that
    /// only branch on validity. Prefer `verify_with_policy` where the reason
    /// matters: this discards it to a `debug` log.
    pub fn is_valid(&self, require_pq: bool) -> bool {
        match self.verify_with_policy(require_pq) {
            Ok(()) => true,
            Err(e) => {
                tracing::debug!(
                    inviter_id = %self.inviter_id,
                    error = %e,
                    "rejected invite token"
                );
                false
            }
        }
    }

    /// Get data to be signed (everything except signature)
    ///
    /// SECURITY: `seed_ledger` is included here on purpose. It carries dialable
    /// addresses that a fresh invitee will contact on first launch, so it must
    /// be authenticated by the inviter's signature. Removing it from this
    /// struct turns any invite relay into an address-injection oracle.
    ///
    /// NOTE: adding the field changed the signed byte string, so signatures
    /// produced by pre-v0.4.0 builds no longer verify against v0.4.0 tokens and
    /// vice versa. Invites are short lived (30 day default expiry), so this is
    /// an accepted pre-1.0 break rather than a migration.
    ///
    /// SECURITY: the returned bytes are prefixed with [`INVITE_SIGNING_DOMAIN`]
    /// so an invite signature is bound to the invite context and cannot be
    /// replayed as a signature over another bincode structure. That prefix is a
    /// second, independent wire break on top of the one described above.
    pub fn get_signable_data(&self) -> Result<Vec<u8>, InviteError> {
        let temp = Self {
            inviter_id: self.inviter_id.clone(),
            inviter_public_key: self.inviter_public_key.clone(),
            invitee_id: self.invitee_id.clone(),
            created_at: self.created_at,
            expires_at: self.expires_at,
            signature: Vec::new(),
            metadata: self.metadata.clone(),
            pq_public_key: self.pq_public_key.clone(),
            pq_signature: None,
            seed_ledger: self.seed_ledger.clone(),
        };

        let body = bincode::serialize(&temp)
            .map_err(|e| InviteError::SerializationError(e.to_string()))?;

        let mut signable = Vec::with_capacity(INVITE_SIGNING_DOMAIN.len() + body.len());
        signable.extend_from_slice(INVITE_SIGNING_DOMAIN);
        signable.extend_from_slice(&body);
        Ok(signable)
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, InviteError> {
        bincode::serialize(self).map_err(|e| InviteError::SerializationError(e.to_string()))
    }

    /// Deserialize from bytes.
    ///
    /// Tries the current wire shape first, then falls back to the pre-v0.4.0
    /// shape so tokens minted by older builds still parse (with an empty seed
    /// ledger). The order matters: bincode tolerates trailing bytes, so a
    /// legacy-first attempt would silently accept a current token and drop its
    /// seed ledger, whereas a current-first attempt cleanly fails with EOF on
    /// legacy bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, InviteError> {
        match bincode::deserialize::<Self>(bytes) {
            Ok(token) => Ok(token),
            Err(current_err) => match bincode::deserialize::<LegacyInviteToken>(bytes) {
                Ok(legacy) => Ok(Self {
                    inviter_id: legacy.inviter_id,
                    inviter_public_key: legacy.inviter_public_key,
                    invitee_id: legacy.invitee_id,
                    created_at: legacy.created_at,
                    expires_at: legacy.expires_at,
                    signature: legacy.signature,
                    metadata: legacy.metadata,
                    pq_public_key: legacy.pq_public_key,
                    pq_signature: legacy.pq_signature,
                    seed_ledger: Vec::new(),
                }),
                Err(_) => Err(InviteError::SerializationError(current_err.to_string())),
            },
        }
    }

    /// Encode the token as a QR-ready ASCII payload: `SCI1:` + base64(bincode).
    ///
    /// No compression. A [`SeedLedgerEntry`] is a bare multiaddr (~25-30 bytes),
    /// so a full 16-entry token lands well inside the QR budget on its own and
    /// a compressor would only add a decompression-bomb surface for nothing.
    ///
    /// Returns [`InviteError::PayloadTooLarge`] rather than emitting something
    /// that cannot be scanned. If a caller hits that, the fix is fewer seed
    /// entries — never a silent truncation, which could drop the inviter's own
    /// address and leave the invitee with no way back to the inviter.
    pub fn to_qr_payload(&self) -> Result<String, InviteError> {
        use base64::Engine as _;

        let raw = self.to_bytes()?;
        let encoded = format!(
            "{}{}",
            QR_PAYLOAD_PREFIX,
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&raw)
        );

        if encoded.len() > QR_BYTE_BUDGET {
            return Err(InviteError::PayloadTooLarge(encoded.len(), QR_BYTE_BUDGET));
        }

        Ok(encoded)
    }

    /// Decode a payload produced by [`Self::to_qr_payload`].
    pub fn from_qr_payload(payload: &str) -> Result<Self, InviteError> {
        use base64::Engine as _;

        let body = payload
            .strip_prefix(QR_PAYLOAD_PREFIX)
            .ok_or_else(|| InviteError::MalformedPayload("missing SCI1 prefix".to_string()))?;

        // Bound the work before decoding: a scanner should never hand us more
        // than a QR code can hold.
        if body.len() > QR_BYTE_BUDGET {
            return Err(InviteError::PayloadTooLarge(body.len(), QR_BYTE_BUDGET));
        }

        let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(body)
            .map_err(|e| InviteError::MalformedPayload(e.to_string()))?;

        Self::from_bytes(&raw)
    }
}

/// Tracks the web-of-trust chain (who invited whom)
#[derive(Debug, Clone)]
pub struct InviteChain {
    /// Invite ID -> (inviter_id, invitee_id, timestamp)
    invites: HashMap<String, (String, String, u64)>,
}

impl InviteChain {
    /// Create a new invite chain tracker
    pub fn new() -> Self {
        Self {
            invites: HashMap::new(),
        }
    }

    /// Record an invite relationship
    pub fn record_invite(&mut self, invite_id: String, inviter_id: String, invitee_id: String) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.invites
            .insert(invite_id, (inviter_id, invitee_id, now));
    }

    /// Get who invited a specific person
    pub fn get_inviter(&self, invitee_id: &str) -> Option<String> {
        for (inviter_id, invited_id, _) in self.invites.values() {
            if invited_id == invitee_id {
                return Some(inviter_id.clone());
            }
        }
        None
    }

    /// Get all people invited by a specific inviter
    pub fn get_invitees(&self, inviter_id: &str) -> Vec<String> {
        self.invites
            .values()
            .filter_map(|(iid, invitee_id, _)| {
                if iid == inviter_id {
                    Some(invitee_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Build the trust chain from a root node
    pub fn get_trust_chain(&self, node_id: &str) -> Vec<String> {
        let mut chain = vec![node_id.to_string()];

        let mut current = node_id.to_string();
        while let Some(inviter) = self.get_inviter(&current) {
            chain.push(inviter.clone());
            current = inviter;
        }

        chain
    }

    /// Get number of degrees of separation from root
    pub fn distance_from_root(&self, node_id: &str) -> u32 {
        (self.get_trust_chain(node_id).len() as u32).saturating_sub(1)
    }

    /// Get total number of invites tracked
    pub fn invite_count(&self) -> usize {
        self.invites.len()
    }

    /// Clear all invite records
    pub fn clear(&mut self) {
        self.invites.clear();
    }

    /// Get direct invitations from a person (not recursive)
    pub fn get_direct_invitations(&self, person_id: &str) -> Vec<(String, u64)> {
        self.invites
            .values()
            .filter_map(|(inviter_id, invitee_id, timestamp)| {
                if inviter_id == person_id {
                    Some((invitee_id.clone(), *timestamp))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for InviteChain {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level invite system
pub struct InviteSystem {
    /// Our node ID
    our_id: String,
    /// Our Ed25519 public key
    our_public_key: Vec<u8>,
    /// Tracking invite chain (web of trust)
    chain: InviteChain,
}

impl InviteSystem {
    /// Create a new invite system
    pub fn new(node_id: String, public_key: Vec<u8>) -> Self {
        Self {
            our_id: node_id,
            our_public_key: public_key,
            chain: InviteChain::new(),
        }
    }

    /// Create an invite token for another peer
    pub fn create_invite_token(&self, invitee_id: String) -> InviteToken {
        InviteToken::new(self.our_id.clone(), self.our_public_key.clone(), invitee_id)
    }

    /// Record that we invited someone
    pub fn record_invitation(&mut self, invitee_id: String) {
        let invite_id = format!("{}_{}", self.our_id, invitee_id);
        self.chain
            .record_invite(invite_id, self.our_id.clone(), invitee_id);
    }

    /// Get the trust chain for a peer
    pub fn get_trust_chain(&self, peer_id: &str) -> Vec<String> {
        self.chain.get_trust_chain(peer_id)
    }

    /// Get our direct invitees
    pub fn get_invitees(&self) -> Vec<String> {
        self.chain.get_invitees(&self.our_id)
    }

    /// Get who invited us
    pub fn get_inviter(&self) -> Option<String> {
        self.chain.get_inviter(&self.our_id)
    }

    /// Check if we're directly connected to a peer in the trust graph
    pub fn is_direct_connection(&self, peer_id: &str) -> bool {
        self.chain
            .get_invitees(&self.our_id)
            .contains(&peer_id.to_string())
            || self.chain.get_inviter(&self.our_id).as_deref() == Some(peer_id)
    }

    /// Get all connected peers in our trust network
    pub fn get_connected_peers(&self) -> Vec<String> {
        let mut peers = self.chain.get_invitees(&self.our_id);

        if let Some(inviter) = self.chain.get_inviter(&self.our_id) {
            peers.push(inviter);
        }

        peers
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_token() -> InviteToken {
        InviteToken::new("alice".to_string(), vec![1, 2, 3, 4, 5], "bob".to_string())
    }

    fn signing_key() -> ed25519_dalek::SigningKey {
        ed25519_dalek::SigningKey::from_bytes(&[7u8; 32])
    }

    /// A second, unrelated inviter key.
    fn other_signing_key() -> ed25519_dalek::SigningKey {
        ed25519_dalek::SigningKey::from_bytes(&[9u8; 32])
    }

    /// Sign a token over its signable data, exactly as a platform client does.
    ///
    /// `inviter_public_key` is set to the signer's key first: it is part of the
    /// signed bytes, so signing with a key the token does not name would only
    /// ever produce a token that cannot verify.
    fn sign_token(mut token: InviteToken) -> InviteToken {
        use ed25519_dalek::Signer;
        let key = signing_key();
        token.inviter_public_key = key.verifying_key().to_bytes().to_vec();
        let data = token.get_signable_data().expect("signable data");
        token.signature = key.sign(&data).to_bytes().to_vec();
        token
    }

    /// Sign a token with both Ed25519 and a freshly generated ML-DSA-65 key,
    /// over the same bytes.
    fn sign_token_pq(token: InviteToken) -> InviteToken {
        use ed25519_dalek::Signer;

        let pq = crate::crypto::pq::mldsa::generate_keypair();
        let key = signing_key();

        // The PQ public key is inside the signed bytes, so it must be attached
        // before the Ed25519 signature is computed.
        let mut token = token;
        token.inviter_public_key = key.verifying_key().to_bytes().to_vec();
        token.pq_public_key = Some(pq.verifying_key().to_vec());
        token.pq_signature = None;

        let data = token.get_signable_data().expect("signable data");
        token.signature = key.sign(&data).to_bytes().to_vec();
        token.pq_signature = Some(crate::crypto::pq::mldsa::sign(&pq, &data).expect("mldsa sign"));
        token
    }

    #[test]
    fn test_invite_token_creation() {
        let token = test_token();
        assert_eq!(token.inviter_id, "alice");
        assert_eq!(token.invitee_id, "bob");
        assert!(token.expires_at > token.created_at);
    }

    #[test]
    fn test_invite_token_with_expiry() {
        let token = test_token().with_expiry(3600);
        assert_eq!(token.expires_at - token.created_at, 3600);
    }

    #[test]
    fn test_invite_token_with_metadata() {
        let token = test_token().with_metadata("group-1".to_string());
        assert_eq!(token.metadata, Some("group-1".to_string()));
    }

    #[test]
    fn test_invite_token_validity() {
        let token = test_token();
        // No signature yet.
        assert!(matches!(token.verify(), Err(InviteError::MissingSignature)));
        assert!(!token.is_valid(false));

        let signed = sign_token(test_token());
        assert!(signed.verify().is_ok());
        assert!(signed.is_valid(false));
    }

    #[test]
    fn test_invite_token_expiry_check() {
        let token = sign_token(test_token().with_expiry(0));

        std::thread::sleep(web_time::Duration::from_millis(10));
        assert!(matches!(token.verify(), Err(InviteError::TokenExpired)));
        assert!(!token.is_valid(false));
    }

    /// THE F1 FORGERY, verbatim from the adversarial review: an attacker-built
    /// token with junk in both signature fields used to return `is_valid(true)`.
    #[test]
    fn test_invite_token_forged_signature_is_rejected() {
        let mut forged = test_token().with_seed_ledger(vec![seed("/ip4/6.6.6.6/tcp/9001")]);
        forged.inviter_public_key = signing_key().verifying_key().to_bytes().to_vec();
        forged.signature = vec![0x00];
        forged.pq_signature = Some(vec![0x01]);
        forged.pq_public_key = Some(vec![0x02]);

        assert!(!forged.is_valid(true), "forged invite must not validate");
        assert!(!forged.is_valid(false), "forged invite must not validate");
        assert!(matches!(
            forged.verify(),
            Err(InviteError::MalformedSignature(_))
        ));
    }

    /// A signature of the right shape but the wrong contents: this is the case
    /// a length check alone would let through.
    #[test]
    fn test_invite_token_zeroed_full_length_signature_is_rejected() {
        let mut forged = test_token();
        forged.inviter_public_key = signing_key().verifying_key().to_bytes().to_vec();
        forged.signature = vec![0x00; 64];

        assert!(matches!(
            forged.verify(),
            Err(InviteError::VerificationFailed)
        ));
    }

    #[test]
    fn test_invite_token_wrong_public_key_is_rejected() {
        // Signed by `signing_key()`, but the token names a different inviter.
        let mut token = sign_token(test_token());
        token.inviter_public_key = other_signing_key().verifying_key().to_bytes().to_vec();

        assert!(matches!(
            token.verify(),
            Err(InviteError::VerificationFailed)
        ));
        assert!(!token.is_valid(false));
    }

    #[test]
    fn test_invite_token_malformed_public_key_is_rejected() {
        // The default `test_token()` key is 5 bytes, not an Ed25519 key.
        let mut token = test_token();
        token.signature = vec![0x00; 64];

        assert!(matches!(token.verify(), Err(InviteError::MalformedKey(_))));
    }

    /// Domain separation: a signature over the bare bincode body, without the
    /// [`INVITE_SIGNING_DOMAIN`] prefix, must not verify.
    #[test]
    fn test_invite_token_requires_domain_separation() {
        use ed25519_dalek::Signer;

        let key = signing_key();
        let mut token = test_token();
        token.inviter_public_key = key.verifying_key().to_bytes().to_vec();

        let domained = token.get_signable_data().expect("signable data");
        assert!(
            domained.starts_with(INVITE_SIGNING_DOMAIN),
            "signable data must carry the domain-separation prefix"
        );

        let undomained = &domained[INVITE_SIGNING_DOMAIN.len()..];
        token.signature = key.sign(undomained).to_bytes().to_vec();

        assert!(
            matches!(token.verify(), Err(InviteError::VerificationFailed)),
            "a signature over the undomained body must not verify"
        );
    }

    #[test]
    fn test_invite_token_v1_compatibility() {
        // Ed25519-only invite: valid, but only under a policy that permits the
        // non-PQ downgrade.
        let token = sign_token(test_token());
        assert!(token.verify().is_ok());
        assert!(token.is_valid(false));
    }

    #[test]
    fn test_invite_token_require_pq_rejects_v1() {
        let token = sign_token(test_token());
        assert!(matches!(
            token.verify_with_policy(true),
            Err(InviteError::PqSignatureRequired)
        ));
        assert!(!token.is_valid(true));
    }

    #[test]
    fn test_invite_token_v2_dual_sig_verification() {
        let token = sign_token_pq(test_token());

        assert!(token.verify_with_policy(true).is_ok());
        assert!(token.is_valid(true));
        assert!(token.is_valid(false));
    }

    #[test]
    fn test_invite_token_tampered_pq_signature() {
        let mut token = sign_token_pq(test_token());
        let sig = token.pq_signature.as_mut().expect("pq signature");
        sig[0] ^= 1;

        assert!(matches!(
            token.verify(),
            Err(InviteError::PqVerificationFailed)
        ));
        assert!(!token.is_valid(true));
        assert!(!token.is_valid(false));
    }

    #[test]
    fn test_invite_token_garbage_pq_signature() {
        let mut token = sign_token_pq(test_token());
        token.pq_signature = Some(vec![0x01]);

        assert!(matches!(
            token.verify(),
            Err(InviteError::PqVerificationFailed)
        ));
        assert!(!token.is_valid(true));
    }

    /// A PQ signature with no PQ key is unverifiable, so it is rejected rather
    /// than skipped: skipping would let an attacker append a fake `pq_signature`
    /// to a legitimate Ed25519-only invite and satisfy a `require_pq` policy.
    #[test]
    fn test_invite_token_pq_signature_without_key_is_rejected() {
        let mut token = sign_token(test_token());
        token.pq_signature = Some(vec![0x01; 3309]);

        assert!(matches!(token.verify(), Err(InviteError::MalformedKey(_))));
        assert!(!token.is_valid(true));
        assert!(!token.is_valid(false));
    }

    /// Every field covered by `get_signable_data` must break verification when
    /// mutated after signing.
    #[test]
    fn test_invite_token_mutating_any_signed_field_breaks_verification() {
        let base = sign_token(
            test_token()
                .with_metadata("group-1".to_string())
                .with_seed_ledger(vec![seed("/ip4/198.51.100.7/tcp/9001")]),
        );
        assert!(base.verify().is_ok());

        let mutations: Vec<(&str, Box<dyn Fn(&mut InviteToken)>)> = vec![
            (
                "inviter_id",
                Box::new(|t: &mut InviteToken| t.inviter_id = "mallory".to_string()),
            ),
            (
                "invitee_id",
                Box::new(|t: &mut InviteToken| t.invitee_id = "mallory".to_string()),
            ),
            (
                "created_at",
                Box::new(|t: &mut InviteToken| t.created_at -= 1),
            ),
            (
                "expires_at",
                Box::new(|t: &mut InviteToken| t.expires_at += 1),
            ),
            (
                "metadata",
                Box::new(|t: &mut InviteToken| t.metadata = Some("group-2".to_string())),
            ),
            (
                "pq_public_key",
                Box::new(|t: &mut InviteToken| t.pq_public_key = Some(vec![0u8; 1952])),
            ),
        ];

        for (field, mutate) in mutations {
            let mut mutated = base.clone();
            mutate(&mut mutated);
            assert!(
                mutated.verify().is_err(),
                "mutating {} must break verification",
                field
            );
        }
    }

    // ------------------------------------------------------------------
    // Seed ledger (v0.4.0 item 1)
    // ------------------------------------------------------------------

    fn seed(addr: &str) -> SeedLedgerEntry {
        SeedLedgerEntry {
            multiaddr: addr.to_string(),
        }
    }

    /// A plausible worst case: 16 distinct public IPv4 addresses.
    fn full_seed_ledger() -> Vec<SeedLedgerEntry> {
        (0..MAX_SEED_LEDGER_ENTRIES)
            .map(|i| seed(&format!("/ip4/203.0.{}.{}/tcp/{}", i, 200 - i, 9000 + i)))
            .collect()
    }

    /// A token with realistically sized identifiers, for size assertions.
    fn qr_sized_token() -> InviteToken {
        let mut token = test_token();
        token.inviter_public_key = vec![0xAB; 32];
        token.inviter_id = "12D3KooWinviterAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string();
        token.invitee_id = "12D3KooWinviteeBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB".to_string();
        token
    }

    #[test]
    fn test_seed_ledger_is_covered_by_signature() {
        // REGRESSION GUARD: if seed_ledger ever falls outside
        // get_signable_data(), anyone who intercepts an invite can inject
        // attacker-controlled multiaddrs that a fresh node dials on launch.
        let token =
            sign_token(test_token().with_seed_ledger(vec![seed("/ip4/198.51.100.7/tcp/9001")]));
        assert!(token.verify().is_ok(), "honest token must verify");

        // 1. Append an attacker address.
        let mut injected = token.clone();
        injected.seed_ledger.push(seed("/ip4/6.6.6.6/tcp/9001"));
        assert!(
            injected.verify().is_err(),
            "appending a seed_ledger entry must break the signature"
        );

        // 2. Rewrite an existing address in place.
        let mut rewritten = token.clone();
        rewritten.seed_ledger[0].multiaddr = "/ip4/6.6.6.6/tcp/9001".to_string();
        assert!(
            rewritten.verify().is_err(),
            "mutating a seed_ledger multiaddr must break the signature"
        );

        // 3. Change the port only -- a one-character difference must still fail.
        let mut reported = token.clone();
        reported.seed_ledger[0].multiaddr = "/ip4/198.51.100.7/tcp/9002".to_string();
        assert!(
            reported.verify().is_err(),
            "mutating a seed_ledger port must break the signature"
        );

        // 4. Strip the whole list.
        let mut stripped = token.clone();
        stripped.seed_ledger.clear();
        assert!(
            stripped.verify().is_err(),
            "removing the seed_ledger must break the signature"
        );
    }

    // Property: no mutation of the seed ledger survives verification, for any
    // ledger shape and any single-entry edit. The hand-written cases above
    // cover the four shapes we expect; this covers the ones we did not think
    // of.
    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(64))]

        #[test]
        fn prop_seed_ledger_mutation_always_breaks_verification(
            addrs in proptest::collection::vec(
                "/ip4/[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}/tcp/[0-9]{1,5}",
                1..8usize,
            ),
            index in 0usize..8,
            suffix in "[a-z0-9]{1,8}",
        ) {
            let entries: Vec<SeedLedgerEntry> = addrs.iter().map(|a| seed(a)).collect();
            let token = sign_token(test_token().with_seed_ledger(entries));
            proptest::prop_assert!(token.verify().is_ok(), "honest token must verify");

            let i = index % token.seed_ledger.len();

            // Append.
            let mut appended = token.clone();
            appended.seed_ledger.push(seed(&format!("/ip4/6.6.6.6/tcp/1{}", suffix)));
            proptest::prop_assert!(appended.verify().is_err());

            // Edit one entry in place.
            let mut edited = token.clone();
            edited.seed_ledger[i].multiaddr.push_str(&suffix);
            proptest::prop_assert!(edited.verify().is_err());

            // Remove one entry.
            let mut removed = token.clone();
            removed.seed_ledger.remove(i);
            proptest::prop_assert!(removed.verify().is_err());
        }
    }

    #[test]
    fn test_build_seed_ledger_keeps_inviter_first_and_caps() {
        let inviter = seed("/ip4/198.51.100.1/tcp/9001");
        let peers: Vec<SeedLedgerEntry> = (0..40)
            .map(|i| seed(&format!("/ip4/203.0.113.{}/tcp/9001", i)))
            .collect();

        let built = build_seed_ledger(inviter.clone(), peers);
        assert_eq!(built.len(), MAX_SEED_LEDGER_ENTRIES);
        assert_eq!(built[0], inviter, "inviter must be first and never evicted");
    }

    #[test]
    fn test_build_seed_ledger_strips_peer_ids_and_dedupes() {
        let inviter = seed("/ip4/198.51.100.1/tcp/9001/p2p/12D3KooWinviter");
        let peers = vec![
            // Same address as the inviter, without the suffix: must dedupe.
            seed("/ip4/198.51.100.1/tcp/9001"),
            seed("/ip4/203.0.113.5/tcp/9001/p2p/12D3KooWother"),
        ];

        let built = build_seed_ledger(inviter, peers);
        assert_eq!(built.len(), 2);
        // The /p2p/ component must not survive on either entry: a peer id
        // smuggled into the multiaddr string is still a peer id.
        assert_eq!(built[0].multiaddr, "/ip4/198.51.100.1/tcp/9001");
        assert_eq!(built[1].multiaddr, "/ip4/203.0.113.5/tcp/9001");
    }

    #[test]
    fn test_with_seed_ledger_truncates_without_dropping_inviter() {
        let mut entries = vec![seed("/ip4/198.51.100.1/tcp/9001")];
        entries.extend(full_seed_ledger());
        let token = test_token().with_seed_ledger(entries);

        assert_eq!(token.seed_ledger.len(), MAX_SEED_LEDGER_ENTRIES);
        assert_eq!(token.seed_ledger[0].multiaddr, "/ip4/198.51.100.1/tcp/9001");
    }

    #[test]
    fn test_seed_ledger_full_invite_fits_qr_budget() {
        // Ed25519-only ("v1") token: 32-byte public key + 64-byte signature.
        let token = sign_token(qr_sized_token().with_seed_ledger(full_seed_ledger()));

        let payload = token.to_qr_payload().expect("full seed ledger must encode");
        assert_eq!(token.seed_ledger.len(), MAX_SEED_LEDGER_ENTRIES);
        assert!(
            payload.len() <= QR_BYTE_BUDGET,
            "encoded invite is {} bytes, over the {} byte QR budget",
            payload.len(),
            QR_BYTE_BUDGET
        );

        let decoded = InviteToken::from_qr_payload(&payload).expect("round trip");
        assert_eq!(decoded.seed_ledger, token.seed_ledger);
        assert!(
            decoded.verify().is_ok(),
            "round trip must preserve signature"
        );
    }

    /// LEAK REGRESSION GUARD (operator directive 2026-07-25): the seed ledger
    /// is routing data only. Asserted against the SERIALISED bytes rather than
    /// the struct shape, so re-adding an identity field to `SeedLedgerEntry`
    /// fails here even if every other test still compiles.
    #[test]
    fn test_encoded_invite_leaks_no_third_party_identity() {
        // A REAL peer id, not a hand-written lookalike. The previous fixture did
        // not parse as a multihash, so `strip_peer_id_component` returned the
        // address unchanged and the whole export path was exercised on a string
        // that no `Multiaddr` gate could ever accept -- which is also how the
        // NEW-7 filter surfaced it.
        let peer_id = libp2p::PeerId::random().to_string();
        let peer_id: &str = &peer_id;
        const PUBLIC_KEY: &str = "b7f3c1a95e2d40886af1c0d3e9b25714aa0c6f8d1e3b5972c4a8d60f1e2b3c4d";
        const NICKNAME: &str = "carol-thinkpad";

        // A ledger whose peers have identity attached locally. None of it may
        // reach the wire, even though the exporter has access to all of it.
        let dir = tempfile::tempdir().expect("tempdir");
        let ledger = crate::store::ledger_entry::LedgerManager::new(
            dir.path().to_string_lossy().to_string(),
        );
        ledger.record_connection(
            format!("/ip4/203.0.113.9/tcp/9001/p2p/{}", peer_id),
            peer_id.to_string(),
        );
        ledger.annotate_identity(
            format!("/ip4/203.0.113.9/tcp/9001/p2p/{}", peer_id),
            peer_id.to_string(),
            Some(PUBLIC_KEY.to_string()),
            Some(NICKNAME.to_string()),
        );

        let seed_ledger = build_seed_ledger(
            seed("/ip4/198.51.100.1/tcp/9001"),
            ledger.export_seed_entries(MAX_SEED_LEDGER_ENTRIES as u32),
        );
        assert_eq!(
            seed_ledger.len(),
            2,
            "the ledger peer should have been exported as a bare address"
        );

        let token = sign_token(qr_sized_token().with_seed_ledger(seed_ledger));
        let raw = token.to_bytes().expect("serialize");
        let payload = token.to_qr_payload().expect("encode");
        let raw_text = String::from_utf8_lossy(&raw).to_string();

        for needle in [peer_id, PUBLIC_KEY, NICKNAME, "/p2p/"] {
            assert!(
                !raw_text.contains(needle),
                "serialised invite leaked third-party identity: {}",
                needle
            );
            assert!(
                !payload.contains(needle),
                "encoded invite leaked third-party identity: {}",
                needle
            );
        }

        // And the address itself did make it through, so the assertion above
        // is not passing vacuously.
        assert!(raw_text.contains("/ip4/203.0.113.9/tcp/9001"));
    }

    #[test]
    fn test_qr_payload_rejects_malformed_input() {
        assert!(matches!(
            InviteToken::from_qr_payload("not-an-invite"),
            Err(InviteError::MalformedPayload(_))
        ));
        assert!(matches!(
            InviteToken::from_qr_payload("SCI1:!!!not-base64!!!"),
            Err(InviteError::MalformedPayload(_))
        ));
        // Well-formed base64, but far too short to be a token.
        assert!(matches!(
            InviteToken::from_qr_payload("SCI1:AAAA"),
            Err(InviteError::SerializationError(_))
        ));
        // Over the QR budget before we decode anything.
        let oversized = format!("SCI1:{}", "A".repeat(QR_BYTE_BUDGET + 1));
        assert!(matches!(
            InviteToken::from_qr_payload(&oversized),
            Err(InviteError::PayloadTooLarge(_, _))
        ));
    }

    #[test]
    fn test_legacy_token_bytes_still_deserialize() {
        // A pre-v0.4.0 token has no seed_ledger field on the wire. bincode is
        // not self-describing, so this exercises the explicit fallback path.
        #[derive(Serialize)]
        struct LegacyWire {
            inviter_id: String,
            inviter_public_key: Vec<u8>,
            invitee_id: String,
            created_at: u64,
            expires_at: u64,
            signature: Vec<u8>,
            metadata: Option<String>,
            pq_public_key: Option<Vec<u8>>,
            pq_signature: Option<Vec<u8>>,
        }

        let legacy = LegacyWire {
            inviter_id: "alice".to_string(),
            inviter_public_key: vec![1, 2, 3, 4, 5],
            invitee_id: "bob".to_string(),
            created_at: 1_700_000_000,
            expires_at: 1_700_086_400,
            signature: vec![9; 64],
            metadata: Some("group-1".to_string()),
            pq_public_key: None,
            pq_signature: None,
        };
        let bytes = bincode::serialize(&legacy).expect("serialize legacy");

        let restored = InviteToken::from_bytes(&bytes).expect("legacy token must deserialize");
        assert_eq!(restored.inviter_id, "alice");
        assert_eq!(restored.invitee_id, "bob");
        assert_eq!(restored.metadata, Some("group-1".to_string()));
        assert!(restored.seed_ledger.is_empty());
    }

    #[test]
    fn test_current_token_bytes_round_trip_with_seed_ledger() {
        let token = test_token().with_seed_ledger(vec![seed("/ip4/198.51.100.7/tcp/9001")]);
        let bytes = token.to_bytes().expect("serialize");
        let restored = InviteToken::from_bytes(&bytes).expect("deserialize");
        assert_eq!(restored.seed_ledger, token.seed_ledger);
    }

    #[test]
    fn test_invite_token_serialization() {
        let token = test_token().with_signature(vec![1, 2, 3]);
        let bytes = token.to_bytes().expect("Failed to serialize");
        let restored = InviteToken::from_bytes(&bytes).expect("Failed to deserialize");

        assert_eq!(token.inviter_id, restored.inviter_id);
        assert_eq!(token.invitee_id, restored.invitee_id);
        assert_eq!(token.signature, restored.signature);
    }

    #[test]
    fn test_invite_chain_creation() {
        let chain = InviteChain::new();
        assert_eq!(chain.invite_count(), 0);
    }

    #[test]
    fn test_record_invite() {
        let mut chain = InviteChain::new();
        chain.record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());

        assert_eq!(chain.invite_count(), 1);
        assert_eq!(chain.get_inviter("bob"), Some("alice".to_string()));
    }

    #[test]
    fn test_get_invitees() {
        let mut chain = InviteChain::new();
        chain.record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());
        chain.record_invite(
            "inv2".to_string(),
            "alice".to_string(),
            "charlie".to_string(),
        );

        let invitees = chain.get_invitees("alice");
        assert_eq!(invitees.len(), 2);
        assert!(invitees.contains(&"bob".to_string()));
        assert!(invitees.contains(&"charlie".to_string()));
    }

    #[test]
    fn test_get_trust_chain() {
        let mut chain = InviteChain::new();
        chain.record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());
        chain.record_invite("inv2".to_string(), "bob".to_string(), "charlie".to_string());

        let trust_chain = chain.get_trust_chain("charlie");
        assert_eq!(trust_chain[0], "charlie");
        assert_eq!(trust_chain[1], "bob");
        assert_eq!(trust_chain[2], "alice");
    }

    #[test]
    fn test_distance_from_root() {
        let mut chain = InviteChain::new();
        chain.record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());
        chain.record_invite("inv2".to_string(), "bob".to_string(), "charlie".to_string());
        chain.record_invite(
            "inv3".to_string(),
            "charlie".to_string(),
            "diana".to_string(),
        );

        assert_eq!(chain.distance_from_root("alice"), 0);
        assert_eq!(chain.distance_from_root("bob"), 1);
        assert_eq!(chain.distance_from_root("charlie"), 2);
        assert_eq!(chain.distance_from_root("diana"), 3);
    }

    #[test]
    fn test_get_direct_invitations() {
        let mut chain = InviteChain::new();
        chain.record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());
        chain.record_invite(
            "inv2".to_string(),
            "alice".to_string(),
            "charlie".to_string(),
        );

        let direct = chain.get_direct_invitations("alice");
        assert_eq!(direct.len(), 2);
    }

    #[test]
    fn test_invite_system_creation() {
        let system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);
        assert_eq!(system.our_id, "alice");
    }

    #[test]
    fn test_create_invite_token() {
        let system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);
        let token = system.create_invite_token("bob".to_string());

        assert_eq!(token.inviter_id, "alice");
        assert_eq!(token.invitee_id, "bob");
    }

    #[test]
    fn test_record_invitation() {
        let mut system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);
        system.record_invitation("bob".to_string());

        let invitees = system.get_invitees();
        assert!(invitees.contains(&"bob".to_string()));
    }

    #[test]
    fn test_get_inviter() {
        let _system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);

        // Simulate being invited by alice
        let mut other_system = InviteSystem::new("bob".to_string(), vec![4, 5, 6]);
        other_system.chain.record_invite(
            "inv1".to_string(),
            "alice".to_string(),
            "bob".to_string(),
        );

        assert_eq!(other_system.get_inviter(), Some("alice".to_string()));
    }

    #[test]
    fn test_is_direct_connection() {
        let mut system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);
        system.record_invitation("bob".to_string());

        assert!(system.is_direct_connection("bob"));
        assert!(!system.is_direct_connection("charlie"));
    }

    #[test]
    fn test_get_connected_peers() {
        let mut system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);
        system.record_invitation("bob".to_string());
        system.record_invitation("charlie".to_string());

        let peers = system.get_connected_peers();
        assert_eq!(peers.len(), 2);
        assert!(peers.contains(&"bob".to_string()));
        assert!(peers.contains(&"charlie".to_string()));
    }

    #[test]
    fn test_get_trust_chain_via_system() {
        let mut system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);
        system
            .chain
            .record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());
        system
            .chain
            .record_invite("inv2".to_string(), "bob".to_string(), "charlie".to_string());

        let trust_chain = system.get_trust_chain("charlie");
        assert_eq!(trust_chain.len(), 3);
    }

    #[test]
    fn test_chain_clear() {
        let mut chain = InviteChain::new();
        chain.record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());
        assert_eq!(chain.invite_count(), 1);

        chain.clear();
        assert_eq!(chain.invite_count(), 0);
    }
}
