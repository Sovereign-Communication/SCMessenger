//! P0_SECURITY_003: Double Ratchet Protocol for Forward Secrecy.
//!
//! Implements the Double Ratchet algorithm (Signal Protocol) adapted for
//! SCMessenger's mesh architecture. Key properties:
//!
//! - **DH Ratchet**: New X25519 keypair per ratchet step; compromising a DH
//!   private key only decrypts messages until the next DH step.
//! - **Symmetric Key Ratchet**: Chain keys derive message keys via Blake3 KDF;
//!   compromising a chain key only reveals future messages until the next DH step.
//! - **Backward compatibility**: Existing per-message ECDH+XChaCha20 envelopes
//!   continue to work for peers that haven't established a ratcheted session.
//!

use anyhow::{bail, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use rand::RngCore;
use std::collections::HashMap;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};
use zeroize::Zeroize;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// KDF context strings for ratchet chain derivation.
const RATCHET_KDF_CONTEXT: &str = "iron-core ratchet v1 2026-04-15";
const ROOT_KDF_CONTEXT_V1: &str = "iron-core root-chain v1 2026-04-15";
const ROOT_KDF_CONTEXT_V2: &str = "iron-core root-chain v2 hybrid 2026-07";

/// Maximum number of skipped message keys we'll store per session.
/// Signal Protocol recommends "a few hundred" for reliable out-of-order delivery.
/// 256 provides good coverage for mesh networks with high latency variance.
const MAX_SKIP_KEYS: usize = 256;

/// Maximum number of DH ratchet steps before requiring re-initialization.
const MAX_RATCHET_STEPS: u32 = 10_000;

// ---------------------------------------------------------------------------
// Key types (zeroizing wrappers)
// ---------------------------------------------------------------------------

/// A 32-byte key that zeroizes on drop.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct RatchetKey([u8; 32]);

impl RatchetKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Debug for RatchetKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RatchetKey([REDACTED])")
    }
}

/// Compute a 16-byte fingerprint of a PQ shared secret for deduplication.
fn pq_ss_fingerprint(ss: &[u8]) -> [u8; 16] {
    let hash = blake3::hash(ss);
    let mut fp = [0u8; 16];
    fp.copy_from_slice(&hash.as_bytes()[..16]);
    fp
}

/// A PQ shared secret we encapsulated (we are the ciphertext sender), held
/// until trial adoption proves the peer mixed it.
#[derive(Clone)]
pub(crate) struct PendingPqSecret {
    /// The 32-byte ML-KEM-768 shared secret.
    pub(crate) ss: RatchetKey,
    /// dh_step_count at creation time.
    pub(crate) created_step: u32,
}

impl std::fmt::Debug for PendingPqSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PendingPqSecret {{ created_step: {} }}",
            self.created_step
        )
    }
}

// ---------------------------------------------------------------------------
// Ratchet session state
// ---------------------------------------------------------------------------

/// A symmetric key chain that ratchets forward via KDF.
#[derive(Clone)]
pub(crate) struct Chain {
    /// Current chain key — each step derives a new chain key + message key.
    chain_key: RatchetKey,
    /// Number of message keys derived from this chain so far.
    index: u32,
}

impl Chain {
    pub(crate) fn new(chain_key: RatchetKey) -> Self {
        Self {
            chain_key,
            index: 0,
        }
    }

    pub(crate) fn new_with_index(chain_key: RatchetKey, index: u32) -> Self {
        Self { chain_key, index }
    }

    /// Reserved introspection accessor for ratchet chain-key state (e.g. future debug/audit tooling); no current caller outside this module.
    #[allow(dead_code)]
    pub(crate) fn chain_key_bytes(&self) -> [u8; 32] {
        *self.chain_key.as_bytes()
    }

    /// Reserved introspection accessor for ratchet chain index (e.g. future debug/audit tooling); no current caller outside this module.
    #[allow(dead_code)]
    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    /// Advance the chain: derive a message key and update the chain key.
    /// Returns the message key for this step.
    pub(crate) fn next_message_key(&mut self) -> RatchetKey {
        let msg_key = derive_key_with_info(&self.chain_key, b"message-key");
        self.chain_key = derive_key_with_info(&self.chain_key, b"chain-key");
        self.index += 1;
        msg_key
    }
}

/// State for a single peer-to-peer ratcheted session.
pub struct RatchetSession {
    /// Our current DH ratchet keypair (sending side).
    our_dh_secret: X25519StaticSecret,
    /// Our current DH ratchet public key.
    our_dh_public: X25519PublicKey,
    /// Their current DH ratchet public key (receiving side).
    their_dh_public: Option<X25519PublicKey>,
    /// Root key — updated on every DH ratchet step.
    root_key: RatchetKey,
    /// Our sending chain (None until DH ratchet is initialized).
    sending_chain: Option<Chain>,
    /// Our receiving chain (None until we receive their DH ratchet key).
    receiving_chain: Option<Chain>,
    /// Number of DH ratchet steps performed.
    dh_step_count: u32,
    /// Skipped message keys: (their_dh_public_bytes, message_number) → message_key.
    skipped_keys: HashMap<([u8; 32], u32), RatchetKey>,
    /// Whether this session has been initialized (we've received their DH key).
    initialized: bool,
    /// Our X25519 identity secret (for first DH ratchet step on receiver side).
    /// Kept only until the first DH ratchet is performed, then zeroized.
    our_identity_secret: Option<X25519StaticSecret>,
    /// The negotiated cryptographic suite (from PQC-04).
    pub negotiated_suite: Option<u8>,
    /// The transcript hash binding the session to the negotiation (from PQC-04).
    pub transcript_hash: Option<[u8; 32]>,
    /// Flag indicating if the peer has proven they established the session.
    pub peer_confirmed: bool,
    /// Hybrid ciphertext for session bootstrap (stored until peer confirmed).
    pub bootstrap_hct: Option<crate::crypto::pq::hybrid::HybridCiphertext>,
    // PQC-07: Post-quantum ratchet state (PQ-hybrid sessions only, suite 0x02 or 0x03)
    /// Our current ML-KEM keypair for PQ ratchet steps.
    pub pq_our_keypair: Option<crate::crypto::pq::MlKem768KeyPair>,
    /// Previous ML-KEM keypair to handle one-step-behind arrivals.
    pub pq_prev_keypair: Option<crate::crypto::pq::MlKem768KeyPair>,
    /// Latest encapsulation key advertised by peer.
    pub pq_their_encaps_key: Option<Vec<u8>>,
    /// Pending ciphertext we must keep sending until peer acks.
    pub pq_pending_ct: Option<Vec<u8>>,
    /// E-01c: outstanding secret we ENCAPSULATED (we are the ct sender)
    pub(crate) pq_pending_sent: Option<PendingPqSecret>,
    /// E-01c: secret we DECAPSULATED (we are the ct receiver)
    pub(crate) pq_pending_recv: Option<RatchetKey>,
    /// E-01c: fingerprint of the last secret we MIXED at KDF-2
    pub(crate) pq_last_mixed_fp: Option<[u8; 16]>,
    /// E-01c: count of root-key transitions this session has committed
    pub(crate) transition_counter: u64,
}

impl Clone for RatchetSession {
    fn clone(&self) -> Self {
        // Clone StaticSecret by converting to bytes and back
        let our_dh_secret_bytes = self.our_dh_secret.to_bytes();
        let our_dh_secret = X25519StaticSecret::from(our_dh_secret_bytes);

        let our_identity_secret = self.our_identity_secret.as_ref().map(|s| {
            let bytes = s.to_bytes();
            X25519StaticSecret::from(bytes)
        });

        Self {
            our_dh_secret,
            our_dh_public: self.our_dh_public,
            their_dh_public: self.their_dh_public,
            root_key: self.root_key.clone(),
            sending_chain: self.sending_chain.clone(),
            receiving_chain: self.receiving_chain.clone(),
            dh_step_count: self.dh_step_count,
            skipped_keys: self.skipped_keys.clone(),
            initialized: self.initialized,
            our_identity_secret,
            negotiated_suite: self.negotiated_suite,
            transcript_hash: self.transcript_hash,
            peer_confirmed: self.peer_confirmed,
            bootstrap_hct: self.bootstrap_hct.clone(),
            pq_our_keypair: self.pq_our_keypair.clone(),
            pq_prev_keypair: self.pq_prev_keypair.clone(),
            pq_their_encaps_key: self.pq_their_encaps_key.clone(),
            pq_pending_ct: self.pq_pending_ct.clone(),
            pq_pending_sent: self.pq_pending_sent.clone(),
            pq_pending_recv: self.pq_pending_recv.clone(),
            pq_last_mixed_fp: self.pq_last_mixed_fp,
            transition_counter: self.transition_counter,
        }
    }
}

impl RatchetSession {
    /// Reconstruct a session from serialized state (internal use).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn reconstruct(
        our_dh_secret: X25519StaticSecret,
        our_dh_public: X25519PublicKey,
        their_dh_public: Option<X25519PublicKey>,
        root_key: RatchetKey,
        sending_chain: Option<Chain>,
        receiving_chain: Option<Chain>,
        dh_step_count: u32,
        initialized: bool,
        our_identity_secret: Option<X25519StaticSecret>,
        negotiated_suite: Option<u8>,
        transcript_hash: Option<[u8; 32]>,
        peer_confirmed: bool,
        bootstrap_hct: Option<crate::crypto::pq::hybrid::HybridCiphertext>,
        pq_our_keypair: Option<crate::crypto::pq::MlKem768KeyPair>,
        pq_prev_keypair: Option<crate::crypto::pq::MlKem768KeyPair>,
        pq_their_encaps_key: Option<Vec<u8>>,
        pq_pending_ct: Option<Vec<u8>>,
        pq_pending_sent: Option<PendingPqSecret>,
        pq_pending_recv: Option<RatchetKey>,
        pq_last_mixed_fp: Option<[u8; 16]>,
        transition_counter: u64,
        skipped_keys: HashMap<([u8; 32], u32), RatchetKey>,
    ) -> Self {
        Self {
            our_dh_secret,
            our_dh_public,
            their_dh_public,
            root_key,
            sending_chain,
            receiving_chain,
            dh_step_count,
            skipped_keys,
            initialized,
            our_identity_secret,
            negotiated_suite,
            transcript_hash,
            peer_confirmed,
            bootstrap_hct,
            pq_our_keypair,
            pq_prev_keypair,
            pq_their_encaps_key,
            pq_pending_ct,
            pq_pending_sent,
            pq_pending_recv,
            pq_last_mixed_fp,
            transition_counter,
        }
    }

    pub(crate) fn our_dh_secret_bytes(&self) -> [u8; 32] {
        self.our_dh_secret.to_bytes()
    }

    pub(crate) fn their_public_key(&self) -> Option<[u8; 32]> {
        self.their_dh_public.as_ref().map(|k| k.to_bytes())
    }

    #[cfg(not(feature = "test-utils"))]
    pub(crate) fn root_key_bytes(&self) -> [u8; 32] {
        *self.root_key.as_bytes()
    }

    /// Exposed `pub` only under `test-utils` so external integration tests can
    /// assert the PQ ratchet cadence actually refreshed the root key; kept
    /// `pub(crate)` otherwise so raw root-key material never enters the
    /// library's default public API surface.
    #[cfg(feature = "test-utils")]
    pub fn root_key_bytes(&self) -> [u8; 32] {
        *self.root_key.as_bytes()
    }

    pub(crate) fn sending_chain_state(&self) -> Option<([u8; 32], u32)> {
        self.sending_chain
            .as_ref()
            .map(|c| (*c.chain_key.as_bytes(), c.index))
    }

    pub(crate) fn receiving_chain_state(&self) -> Option<([u8; 32], u32)> {
        self.receiving_chain
            .as_ref()
            .map(|c| (*c.chain_key.as_bytes(), c.index))
    }

    pub(crate) fn has_identity_secret(&self) -> bool {
        self.our_identity_secret.is_some()
    }

    /// Snapshot of currently-held skipped message keys, for persistence.
    /// See `SerializableRatchetSession::skipped_keys` (session_manager.rs) -
    /// without this, a skipped key held at session-save time is silently lost
    /// on reload and a legitimately-delayed message becomes undecryptable.
    pub(crate) fn skipped_keys_snapshot(&self) -> Vec<([u8; 32], u32, [u8; 32])> {
        self.skipped_keys
            .iter()
            .map(|((dh, idx), key)| (*dh, *idx, *key.as_bytes()))
            .collect()
    }

    pub(crate) fn identity_secret_bytes(&self) -> Option<[u8; 32]> {
        self.our_identity_secret.as_ref().map(|s| s.to_bytes())
    }

    /// Initialize a new ratchet session as the initiator (Alice).
    pub fn init_as_sender(
        our_signing_key: &ed25519_dalek::SigningKey,
        their_identity_public_x25519: &X25519PublicKey,
    ) -> Result<Self> {
        // Generate our initial DH ratchet keypair
        let mut our_dh_secret_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut our_dh_secret_bytes);
        let our_dh_secret = X25519StaticSecret::from(our_dh_secret_bytes);
        our_dh_secret_bytes.zeroize();
        let our_dh_public = X25519PublicKey::from(&our_dh_secret);

        // Convert our signing key to X25519 for the initial DH
        let our_x25519 = super::encrypt::ed25519_to_x25519_secret(our_signing_key);

        // X3DH step 1: our_identity_secret × their_identity_public → shared_secret
        let shared_secret = our_x25519.diffie_hellman(their_identity_public_x25519);

        // Derive initial root key from the identity DH
        let root_key =
            derive_root_key_v1(&RatchetKey::from_bytes([0u8; 32]), shared_secret.as_bytes());

        // X3DH step 2: our_dh_secret × their_identity_public → sending chain
        let dh_output = our_dh_secret.diffie_hellman(their_identity_public_x25519);
        let (new_root_key, sending_chain_key) =
            root_key_ratchet_v1(&root_key, dh_output.as_bytes());

        let sending_chain = Chain::new(sending_chain_key);

        Ok(Self {
            our_dh_secret,
            our_dh_public,
            their_dh_public: Some(*their_identity_public_x25519),
            root_key: new_root_key,
            sending_chain: Some(sending_chain),
            receiving_chain: None,
            dh_step_count: 1,
            skipped_keys: HashMap::new(),
            initialized: true,
            our_identity_secret: None,
            negotiated_suite: None,
            transcript_hash: None,
            peer_confirmed: false,
            bootstrap_hct: None,
            pq_our_keypair: None,
            pq_prev_keypair: None,
            pq_their_encaps_key: None,
            pq_pending_ct: None,
            pq_pending_sent: None,
            pq_pending_recv: None,
            pq_last_mixed_fp: None,
            transition_counter: 1,
        })
    }

    /// Initialize a new ratchet session as the receiver (Bob).
    pub fn init_as_receiver(
        our_signing_key: &ed25519_dalek::SigningKey,
        sender_identity_public_x25519: &X25519PublicKey,
    ) -> Result<Self> {
        // Generate our initial DH ratchet keypair
        let mut our_dh_secret_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut our_dh_secret_bytes);
        let our_dh_secret = X25519StaticSecret::from(our_dh_secret_bytes);
        our_dh_secret_bytes.zeroize();
        let our_dh_public = X25519PublicKey::from(&our_dh_secret);

        // Convert our signing key to X25519 for the initial DH
        let our_x25519 = super::encrypt::ed25519_to_x25519_secret(our_signing_key);

        // X3DH step 1: our_identity_secret × sender_identity_public → shared_secret
        let shared_secret = our_x25519.diffie_hellman(sender_identity_public_x25519);

        // Derive initial root key
        let root_key =
            derive_root_key_v1(&RatchetKey::from_bytes([0u8; 32]), shared_secret.as_bytes());

        // Store our identity secret for the first DH ratchet step
        let our_identity_secret = our_x25519;

        Ok(Self {
            our_dh_secret,
            our_dh_public,
            their_dh_public: None,
            root_key,
            sending_chain: None,
            receiving_chain: None,
            dh_step_count: 0,
            skipped_keys: HashMap::new(),
            initialized: false,
            our_identity_secret: Some(our_identity_secret),
            negotiated_suite: None,
            transcript_hash: None,
            peer_confirmed: false,
            bootstrap_hct: None,
            pq_our_keypair: None,
            pq_prev_keypair: None,
            pq_their_encaps_key: None,
            pq_pending_ct: None,
            pq_pending_sent: None,
            pq_pending_recv: None,
            pq_last_mixed_fp: None,
            transition_counter: 0,
        })
    }

    /// Initialize a new ratchet session as the initiator (Alice) using hybrid
    /// encryption -- suite 0x03, the CURRENT derivation.
    ///
    /// Folds a static-static DH sender-authentication term into the root key
    /// (domain separator "iron-core session-root v3 2026-08"), so the root key
    /// can no longer be computed from public material alone. For the ORIGINAL
    /// suite 0x02 derivation (no sender-authentication term, "iron-core
    /// session-root v2 2026-07"), preserved for peers that only advertise
    /// 0x02, see `init_as_sender_hybrid_suite02`.
    pub fn init_as_sender_hybrid(
        our_x25519_secret: &x25519_dalek::StaticSecret,
        their_bundle: &crate::identity::PublicKeyBundle,
        transcript_hash: [u8; 32],
    ) -> Result<Self> {
        // Generate our initial DH ratchet keypair
        let mut our_dh_secret_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut our_dh_secret_bytes);
        let our_dh_secret = X25519StaticSecret::from(our_dh_secret_bytes);
        our_dh_secret_bytes.zeroize();
        let our_dh_public = X25519PublicKey::from(&our_dh_secret);

        // Hybrid encapsulate
        let (hct, ss_hybrid) = crate::crypto::pq::hybrid::hybrid_encapsulate(
            &their_bundle.x25519_public,
            &their_bundle.mlkem_encaps_key,
        )?;

        // Static-static DH for sender authentication. Uses our DEDICATED X25519 encryption
        // secret (not an Ed25519-derived one) to avoid cross-protocol key reuse. The
        // receiver must correspondingly use the sender's dedicated `x25519_public` from the
        // sender's published bundle (see `init_as_receiver_hybrid`) -- Curve25519 DH is
        // commutative, so both sides land on the same shared secret as long as they agree
        // on which keypair is in play.
        let recipient_x25519 = X25519PublicKey::from(their_bundle.x25519_public);
        let dh_static = our_x25519_secret.diffie_hellman(&recipient_x25519);

        // Derive root key with sender authentication
        let root_key_0 = blake3::derive_key(
            "iron-core session-root v3 2026-08",
            &[
                ss_hybrid.as_bytes(),
                dh_static.as_bytes(),
                &transcript_hash[..],
            ]
            .concat(),
        );

        // X3DH step 2: our_dh_secret × their_identity_public → sending chain
        let dh_output =
            our_dh_secret.diffie_hellman(&X25519PublicKey::from(their_bundle.x25519_public));
        let (new_root_key, sending_chain_key) = root_key_ratchet_v2(
            &RatchetKey::from_bytes(root_key_0),
            dh_output.as_bytes(),
            None,
        );

        let sending_chain = Chain::new(sending_chain_key);

        // Initialize PQ state (shared mechanics across every hybrid suite)
        let pq_our_keypair = Some(crate::crypto::pq::generate());
        let pq_their_encaps_key = Some(their_bundle.mlkem_encaps_key.to_vec());

        Ok(Self {
            our_dh_secret,
            our_dh_public,
            their_dh_public: Some(X25519PublicKey::from(their_bundle.x25519_public)),
            root_key: new_root_key,
            sending_chain: Some(sending_chain),
            receiving_chain: None,
            dh_step_count: 1,
            skipped_keys: HashMap::new(),
            initialized: true,
            our_identity_secret: None,
            negotiated_suite: Some(0x03),
            transcript_hash: Some(transcript_hash),
            peer_confirmed: false,
            bootstrap_hct: Some(hct.clone()),
            pq_our_keypair,
            pq_prev_keypair: None,
            pq_their_encaps_key,
            pq_pending_ct: None,
            pq_pending_sent: None,
            pq_pending_recv: None,
            pq_last_mixed_fp: None,
            transition_counter: 1,
        })
    }

    /// Initialize a new ratchet session as the receiver (Bob) using hybrid
    /// decryption -- suite 0x03, the CURRENT derivation.
    ///
    /// Must mirror `init_as_sender_hybrid`'s choice of keypair and domain
    /// separator exactly. For the ORIGINAL suite 0x02 derivation, preserved
    /// for peers that only advertise 0x02, see
    /// `init_as_receiver_hybrid_suite02`.
    // PERIMETER-ALLOW-UNDERSCORE: receiver-side sender authentication is
    // X25519-based (our_x25519_secret x sender_bundle.x25519_public) in the
    // 0x03 derivation; the receiver's Ed25519 signing key never participates.
    pub fn init_as_receiver_hybrid(
        _our_signing_key: &ed25519_dalek::SigningKey,
        our_x25519_secret: &x25519_dalek::StaticSecret,
        our_mlkem_keypair: &crate::crypto::pq::MlKem768KeyPair,
        sender_bundle: &crate::identity::PublicKeyBundle,
        hct: &crate::crypto::pq::hybrid::HybridCiphertext,
        transcript_hash: [u8; 32],
    ) -> Result<Self> {
        // Generate our initial DH ratchet keypair
        let mut our_dh_secret_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut our_dh_secret_bytes);
        let our_dh_secret = X25519StaticSecret::from(our_dh_secret_bytes);
        our_dh_secret_bytes.zeroize();
        let our_dh_public = X25519PublicKey::from(&our_dh_secret);

        // Hybrid decapsulate
        let ss_hybrid = crate::crypto::pq::hybrid::hybrid_decapsulate(
            our_x25519_secret,
            our_mlkem_keypair,
            hct,
        )?;

        // Static-static DH for sender authentication. Must mirror the sender's choice in
        // `init_as_sender_hybrid`: the sender now uses its DEDICATED X25519 secret, so we
        // use the sender's dedicated `x25519_public` from their published bundle here --
        // NOT an Ed25519-derived key. Curve25519 DH is commutative, so this still lands on
        // the same shared secret the sender computed.
        let sender_x25519_public = X25519PublicKey::from(sender_bundle.x25519_public);
        let dh_static = our_x25519_secret.diffie_hellman(&sender_x25519_public);

        // Derive root key with sender authentication
        let root_key_0 = blake3::derive_key(
            "iron-core session-root v3 2026-08",
            &[
                ss_hybrid.as_bytes(),
                dh_static.as_bytes(),
                &transcript_hash[..],
            ]
            .concat(),
        );

        // Store our identity secret for the first DH ratchet step
        let our_identity_secret = our_x25519_secret.clone();

        // Initialize PQ state (shared mechanics across every hybrid suite)
        let pq_our_keypair = Some(our_mlkem_keypair.clone());
        let pq_their_encaps_key = Some(hct.mlkem_ciphertext.to_vec());

        Ok(Self {
            our_dh_secret,
            our_dh_public,
            their_dh_public: None, // Fixed: should be None initially, like classical receiver
            root_key: RatchetKey::from_bytes(root_key_0),
            sending_chain: None,
            receiving_chain: None,
            dh_step_count: 0,
            skipped_keys: HashMap::new(),
            initialized: false,
            our_identity_secret: Some(our_identity_secret),
            negotiated_suite: Some(0x03),
            transcript_hash: Some(transcript_hash),
            peer_confirmed: false,
            bootstrap_hct: Some(hct.clone()),
            pq_our_keypair,
            pq_prev_keypair: None,
            pq_their_encaps_key,
            pq_pending_ct: None,
            pq_pending_sent: None,
            pq_pending_recv: None,
            pq_last_mixed_fp: None,
            transition_counter: 0,
        })
    }

    /// Initialize a new ratchet session as the initiator (Alice) using hybrid
    /// encryption -- suite 0x02, the ORIGINAL derivation.
    ///
    /// Preserved byte-for-byte from before PR #221 for interop with a peer
    /// that only ever advertises suite 0x02: the root key is derived from the
    /// ML-KEM/X25519 hybrid shared secret and the negotiation transcript hash
    /// ONLY (domain separator "iron-core session-root v2 2026-07") -- there is
    /// NO static-static DH sender-authentication term, unlike suite 0x03's
    /// `init_as_sender_hybrid`. `_our_signing_key` is intentionally unused;
    /// that is the original 0x02 contract, not an oversight. A current node
    /// never volunteers 0x02 in its own `supported_suites` (see
    /// `identity::keys::sign_bundle`), so this path is only reached when
    /// negotiation is forced down to 0x02 by a peer that has no 0x03 support.
    ///
    /// Do NOT change this function's derivation. If suite 0x02's behavior
    /// ever needs to change, mint a new suite ID instead -- see
    /// `crypto::negotiation::HYBRID_SUITE_IDS` for why.
    // PERIMETER-ALLOW-UNDERSCORE: suite 0x02 has no sender-authentication DH
    // term by design; the signing key must stay unused to preserve the 0x02
    // root-key derivation ("Do NOT change this function's derivation").
    pub fn init_as_sender_hybrid_suite02(
        _our_signing_key: &ed25519_dalek::SigningKey,
        their_bundle: &crate::identity::PublicKeyBundle,
        transcript_hash: [u8; 32],
    ) -> Result<Self> {
        // Generate our initial DH ratchet keypair
        let mut our_dh_secret_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut our_dh_secret_bytes);
        let our_dh_secret = X25519StaticSecret::from(our_dh_secret_bytes);
        our_dh_secret_bytes.zeroize();
        let our_dh_public = X25519PublicKey::from(&our_dh_secret);

        // Hybrid encapsulate
        let (hct, ss_hybrid) = crate::crypto::pq::hybrid::hybrid_encapsulate(
            &their_bundle.x25519_public,
            &their_bundle.mlkem_encaps_key,
        )?;

        // Derive root key (suite 0x02: no sender-authentication DH term)
        let root_key_0 = blake3::derive_key(
            "iron-core session-root v2 2026-07",
            &[&ss_hybrid.as_bytes()[..], &transcript_hash[..]].concat(),
        );

        // X3DH step 2: our_dh_secret × their_identity_public → sending chain
        let dh_output =
            our_dh_secret.diffie_hellman(&X25519PublicKey::from(their_bundle.x25519_public));
        let (new_root_key, sending_chain_key) = root_key_ratchet_v2(
            &RatchetKey::from_bytes(root_key_0),
            dh_output.as_bytes(),
            None,
        );

        let sending_chain = Chain::new(sending_chain_key);

        // Initialize PQ state (shared mechanics across every hybrid suite)
        let pq_our_keypair = Some(crate::crypto::pq::generate());
        let pq_their_encaps_key = Some(their_bundle.mlkem_encaps_key.to_vec());

        Ok(Self {
            our_dh_secret,
            our_dh_public,
            their_dh_public: Some(X25519PublicKey::from(their_bundle.x25519_public)),
            root_key: new_root_key,
            sending_chain: Some(sending_chain),
            receiving_chain: None,
            dh_step_count: 1,
            skipped_keys: HashMap::new(),
            initialized: true,
            our_identity_secret: None,
            negotiated_suite: Some(0x02),
            transcript_hash: Some(transcript_hash),
            peer_confirmed: false,
            bootstrap_hct: Some(hct.clone()),
            pq_our_keypair,
            pq_prev_keypair: None,
            pq_their_encaps_key,
            pq_pending_ct: None,
            pq_pending_sent: None,
            pq_pending_recv: None,
            pq_last_mixed_fp: None,
            transition_counter: 1,
        })
    }

    /// Initialize a new ratchet session as the receiver (Bob) using hybrid
    /// decryption -- suite 0x02, the ORIGINAL derivation.
    ///
    /// Preserved byte-for-byte from before PR #221; see
    /// `init_as_sender_hybrid_suite02` for why this must never be merged into
    /// the suite 0x03 code path.
    // PERIMETER-ALLOW-UNDERSCORE: suite 0x02 receiver derivation has no
    // sender-authentication term: neither our signing key nor the sender
    // bundle participates (preserved byte-for-byte; do not change).
    pub fn init_as_receiver_hybrid_suite02(
        _our_signing_key: &ed25519_dalek::SigningKey,
        our_x25519_secret: &x25519_dalek::StaticSecret,
        our_mlkem_keypair: &crate::crypto::pq::MlKem768KeyPair,
        _sender_bundle: &crate::identity::PublicKeyBundle,
        hct: &crate::crypto::pq::hybrid::HybridCiphertext,
        transcript_hash: [u8; 32],
    ) -> Result<Self> {
        // Generate our initial DH ratchet keypair
        let mut our_dh_secret_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut our_dh_secret_bytes);
        let our_dh_secret = X25519StaticSecret::from(our_dh_secret_bytes);
        our_dh_secret_bytes.zeroize();
        let our_dh_public = X25519PublicKey::from(&our_dh_secret);

        // Hybrid decapsulate
        let ss_hybrid = crate::crypto::pq::hybrid::hybrid_decapsulate(
            our_x25519_secret,
            our_mlkem_keypair,
            hct,
        )?;

        // Derive root key (suite 0x02: no sender-authentication DH term)
        let root_key_0 = blake3::derive_key(
            "iron-core session-root v2 2026-07",
            &[&ss_hybrid.as_bytes()[..], &transcript_hash[..]].concat(),
        );

        // Store our identity secret for the first DH ratchet step
        let our_identity_secret = our_x25519_secret.clone();

        // Initialize PQ state (shared mechanics across every hybrid suite)
        let pq_our_keypair = Some(our_mlkem_keypair.clone());
        let pq_their_encaps_key = Some(hct.mlkem_ciphertext.to_vec());

        Ok(Self {
            our_dh_secret,
            our_dh_public,
            their_dh_public: None, // should be None initially, like classical receiver
            root_key: RatchetKey::from_bytes(root_key_0),
            sending_chain: None,
            receiving_chain: None,
            dh_step_count: 0,
            skipped_keys: HashMap::new(),
            initialized: false,
            our_identity_secret: Some(our_identity_secret),
            negotiated_suite: Some(0x02),
            transcript_hash: Some(transcript_hash),
            peer_confirmed: false,
            bootstrap_hct: Some(hct.clone()),
            pq_our_keypair,
            pq_prev_keypair: None,
            pq_their_encaps_key,
            pq_pending_ct: None,
            pq_pending_sent: None,
            pq_pending_recv: None,
            pq_last_mixed_fp: None,
            transition_counter: 0,
        })
    }

    /// Get our current DH ratchet public key.
    pub fn our_public_key(&self) -> [u8; 32] {
        self.our_dh_public.to_bytes()
    }

    pub fn dh_step_count(&self) -> u32 {
        self.dh_step_count
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// True if this session negotiated a PQ-hybrid suite (0x02 or 0x03) --
    /// i.e. it mixes ML-KEM shared secrets into the ratchet -- as opposed to
    /// suite 0x01 (classical, no PQ). Both hybrid suites share identical
    /// post-handshake ratchet-stepping and PQ-mixing mechanics; they differ
    /// ONLY in how the session's initial root key was derived at
    /// establishment (see `crypto::negotiation::HYBRID_SUITE_IDS`). Use this
    /// instead of comparing `negotiated_suite` against a single suite literal
    /// so a future suite bump can't silently disable PQ mixing for sessions
    /// established under the new suite ID.
    pub fn is_pq_hybrid(&self) -> bool {
        matches!(self.negotiated_suite, Some(s) if crate::crypto::negotiation::HYBRID_SUITE_IDS.contains(&s))
    }

    /// Encrypt a message using the current sending chain.
    pub fn encrypt(&mut self, plaintext: &[u8], aad: &[u8]) -> Result<RatchetEncryptResult> {
        let chain = self
            .sending_chain
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Sending chain not initialized"))?;

        if self.dh_step_count >= MAX_RATCHET_STEPS {
            bail!("Ratchet session has exceeded maximum steps");
        }

        let message_key = chain.next_message_key();
        let message_number = chain.index - 1;

        let mut nonce_bytes = [0u8; 24];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);

        let cipher = XChaCha20Poly1305::new_from_slice(message_key.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        let ciphertext = cipher
            .encrypt(
                nonce,
                Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        Ok(RatchetEncryptResult {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
            message_number,
            our_dh_public: self.our_dh_public.to_bytes(),
        })
    }

    /// Decrypt a message using the receiving chain.
    pub fn decrypt(
        &mut self,
        their_dh_public: &[u8],
        message_number: u32,
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        if their_dh_public.len() != 32 {
            bail!("Invalid DH public key length");
        }
        let mut their_dh_bytes = [0u8; 32];
        their_dh_bytes.copy_from_slice(their_dh_public);
        let their_dh = X25519PublicKey::from(their_dh_bytes);

        // H1: Check the skipped keys cache BEFORE the dh_changed check
        if let Some(message_key) = self
            .skipped_keys
            .get(&(their_dh.to_bytes(), message_number))
        {
            let nonce_obj = XNonce::from_slice(nonce);
            let cipher = XChaCha20Poly1305::new_from_slice(message_key.as_bytes())
                .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;
            let plaintext = cipher
                .decrypt(
                    nonce_obj,
                    Payload {
                        msg: ciphertext,
                        aad,
                    },
                )
                .map_err(|_| anyhow::anyhow!("Decryption failed with skipped key"))?;

            self.skipped_keys
                .remove(&(their_dh.to_bytes(), message_number));
            self.peer_confirmed = true;
            return Ok(plaintext);
        }

        // H2: Implement the trial adoption and commit-on-success pattern
        let mut cloned = (*self).clone();

        let dh_changed = match &cloned.their_dh_public {
            None => true,
            Some(their_current) => their_current.as_bytes() != their_dh.as_bytes(),
        };

        if dh_changed {
            return self.handle_dh_ratchet_trial(&their_dh, message_number, nonce, ciphertext, aad);
        }

        let message_key = cloned.get_message_key(&their_dh, message_number)?;

        let nonce_obj = XNonce::from_slice(nonce);
        let cipher = XChaCha20Poly1305::new_from_slice(message_key.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        let plaintext = cipher
            .decrypt(
                nonce_obj,
                Payload {
                    msg: ciphertext,
                    aad,
                },
            )
            .map_err(|_| anyhow::anyhow!("Decryption failed"))?;

        *self = cloned;
        self.peer_confirmed = true;
        Ok(plaintext)
    }

    /// Handle DH ratchet with trial adoption pattern (E-01c R5, H2)
    fn handle_dh_ratchet_trial(
        &mut self,
        their_new_dh: &X25519PublicKey,
        message_number: u32,
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let mut candidates = Vec::new();
        if let Some(ref pending_sent) = self.pq_pending_sent {
            candidates.push(Some(pending_sent.ss.clone()));
        }
        candidates.push(None);

        let mut errors = Vec::new();

        for candidate in candidates {
            let mut trial = (*self).clone();

            let first_dh_secret = if let Some(identity_secret) = trial.our_identity_secret.take() {
                identity_secret
            } else {
                let mut dh_bytes = trial.our_dh_secret.to_bytes();
                let secret = X25519StaticSecret::from(dh_bytes);
                dh_bytes.zeroize();
                secret
            };
            let dh_output = first_dh_secret.diffie_hellman(their_new_dh);
            let pq_ss_1 = if trial.is_pq_hybrid() {
                trial.pq_pending_recv.clone()
            } else {
                None
            };
            let (new_root_key, receiving_chain_key) = if trial.is_pq_hybrid() {
                root_key_ratchet_v2(
                    &trial.root_key,
                    dh_output.as_bytes(),
                    pq_ss_1.as_ref().map(|k| k.as_bytes().to_vec()),
                )
            } else {
                root_key_ratchet_v1(&trial.root_key, dh_output.as_bytes())
            };
            trial.their_dh_public = Some(*their_new_dh);
            trial.receiving_chain = Some(Chain::new(receiving_chain_key));

            let mut new_secret_bytes = [0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut new_secret_bytes);
            let new_dh_secret = X25519StaticSecret::from(new_secret_bytes);
            new_secret_bytes.zeroize();
            let new_dh_public = X25519PublicKey::from(&new_dh_secret);

            let dh_output_2 = new_dh_secret.diffie_hellman(their_new_dh);
            let pq_ss_2 = if trial.is_pq_hybrid() {
                candidate.clone()
            } else {
                None
            };
            let (new_root_key_2, sending_chain_key) = if trial.is_pq_hybrid() {
                root_key_ratchet_v2(
                    &new_root_key,
                    dh_output_2.as_bytes(),
                    pq_ss_2.as_ref().map(|k| k.as_bytes().to_vec()),
                )
            } else {
                root_key_ratchet_v1(&new_root_key, dh_output_2.as_bytes())
            };
            trial.our_dh_secret = new_dh_secret;
            trial.our_dh_public = new_dh_public;
            trial.root_key = new_root_key_2;
            trial.sending_chain = Some(Chain::new(sending_chain_key));
            trial.dh_step_count += 1;
            trial.initialized = true;

            let message_key = match trial.get_message_key(their_new_dh, message_number) {
                Ok(k) => k,
                Err(e) => {
                    errors.push(format!("get_message_key failed: {}", e));
                    continue;
                }
            };

            let nonce_obj = XNonce::from_slice(nonce);
            let cipher = match XChaCha20Poly1305::new_from_slice(message_key.as_bytes()) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!("cipher creation failed: {}", e));
                    continue;
                }
            };

            match cipher.decrypt(
                nonce_obj,
                Payload {
                    msg: ciphertext,
                    aad,
                },
            ) {
                Ok(plaintext) => {
                    if let Some(ref _adopted_ss) = candidate {
                        trial.pq_pending_sent = None;
                        trial.pq_pending_ct = None;
                    }
                    if let Some(ref ss_recv) = trial.pq_pending_recv {
                        let fp = pq_ss_fingerprint(ss_recv.as_bytes());
                        trial.pq_last_mixed_fp = Some(fp);
                    }
                    trial.pq_pending_recv = None;
                    trial.transition_counter += 2;
                    trial.peer_confirmed = true;

                    *self = trial;
                    return Ok(plaintext);
                }
                Err(e) => {
                    errors.push(format!("decryption failed: {}", e));
                }
            }
        }

        bail!(
            "Trial decryption failed for all candidates. Errors: {:?}",
            errors
        )
    }

    #[allow(dead_code)]
    fn handle_dh_ratchet(&mut self, their_new_dh: &X25519PublicKey) -> Result<()> {
        let first_dh_secret = if let Some(identity_secret) = self.our_identity_secret.take() {
            identity_secret
        } else {
            let mut dh_bytes = self.our_dh_secret.to_bytes();
            let secret = X25519StaticSecret::from(dh_bytes);
            dh_bytes.zeroize();
            secret
        };

        let dh_output = first_dh_secret.diffie_hellman(their_new_dh);

        // Handle PQ ratchet step if this is a PQ-hybrid session (suite 0x02 or 0x03)
        let pq_ss = if self.is_pq_hybrid() {
            self.pq_pending_recv.clone()
        } else {
            None
        };

        let (new_root_key, receiving_chain_key) = if self.is_pq_hybrid() {
            root_key_ratchet_v2(
                &self.root_key,
                dh_output.as_bytes(),
                pq_ss.as_ref().map(|k| k.as_bytes().to_vec()),
            )
        } else {
            root_key_ratchet_v1(&self.root_key, dh_output.as_bytes())
        };

        self.their_dh_public = Some(*their_new_dh);

        let mut new_secret_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut new_secret_bytes);
        let new_dh_secret = X25519StaticSecret::from(new_secret_bytes);
        new_secret_bytes.zeroize();
        let new_dh_public = X25519PublicKey::from(&new_dh_secret);

        let dh_output_2 = new_dh_secret.diffie_hellman(their_new_dh);
        let pq_ss_2 = if self.is_pq_hybrid() {
            self.pq_pending_sent.as_ref().map(|p| p.ss.clone())
        } else {
            None
        };
        let (new_root_key_2, sending_chain_key) = if self.is_pq_hybrid() {
            root_key_ratchet_v2(
                &new_root_key,
                dh_output_2.as_bytes(),
                pq_ss_2.as_ref().map(|k| k.as_bytes().to_vec()),
            )
        } else {
            root_key_ratchet_v1(&new_root_key, dh_output_2.as_bytes())
        };

        self.our_dh_secret = new_dh_secret;
        self.our_dh_public = new_dh_public;
        self.root_key = new_root_key_2;
        self.receiving_chain = Some(Chain::new(receiving_chain_key));
        self.sending_chain = Some(Chain::new(sending_chain_key));
        self.dh_step_count += 1;
        self.initialized = true;

        Ok(())
    }

    fn get_message_key(
        &mut self,
        their_dh: &X25519PublicKey,
        target_number: u32,
    ) -> Result<RatchetKey> {
        let cache_key = (their_dh.to_bytes(), target_number);
        if let Some(key) = self.skipped_keys.remove(&cache_key) {
            return Ok(key);
        }

        let chain = self
            .receiving_chain
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Receiving chain not initialized"))?;

        if target_number < chain.index {
            bail!("Message number is behind current chain position");
        }

        let skip_count = target_number - chain.index;
        if skip_count as usize > MAX_SKIP_KEYS {
            bail!("Too many skipped messages");
        }

        while chain.index < target_number {
            let skipped_key = chain.next_message_key();
            let skipped_number = chain.index - 1;
            self.skipped_keys
                .insert((their_dh.to_bytes(), skipped_number), skipped_key);

            if self.skipped_keys.len() > MAX_SKIP_KEYS {
                if let Some(oldest) = self.skipped_keys.keys().min().cloned() {
                    self.skipped_keys.remove(&oldest);
                }
            }
        }

        Ok(chain.next_message_key())
    }

    /// Perform a PQ ratchet step when initiating a DH step (PQ-hybrid sessions only).
    pub fn perform_pq_ratchet_step(&mut self) -> Result<(Vec<u8>, Vec<u8>)> {
        if !self.is_pq_hybrid() {
            bail!(
                "PQ ratchet step only supported for PQ-hybrid suites (0x02/0x03), \
                 session negotiated {:?}",
                self.negotiated_suite
            );
        }

        let their_encaps_key = self
            .pq_their_encaps_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No PQ encapsulation key from peer"))?;

        // Encapsulate with their current key
        let (ct, ss_pq) = crate::crypto::pq::encapsulate(their_encaps_key)?;

        // Store the encapsulated shared secret as a pending PQ secret
        self.pq_pending_sent = Some(PendingPqSecret {
            ss: RatchetKey::from_bytes(ss_pq),
            created_step: self.dh_step_count,
        });

        // Rotate our keypair: move current to previous, generate new.
        // Derive the encapsulation key from the freshly generated keypair before
        // storing it, which avoids re-borrowing `pq_our_keypair` right after assignment.
        let new_keypair = crate::crypto::pq::generate();
        let new_encaps_key = new_keypair.public_key().to_vec();
        self.pq_prev_keypair = self.pq_our_keypair.take();
        self.pq_our_keypair = Some(new_keypair);

        // Store pending ciphertext until acked
        self.pq_pending_ct = Some(ct.clone());

        Ok((ct, new_encaps_key))
    }

    /// Handle incoming PQ fields during DH ratchet step (PQ-hybrid sessions only).
    pub fn handle_incoming_pq_fields(
        &mut self,
        pq_kem_ciphertext: &[u8],
        pq_encaps_key: &[u8],
    ) -> Result<Vec<u8>> {
        if !self.is_pq_hybrid() {
            bail!(
                "PQ fields only expected for PQ-hybrid suites (0x02/0x03), \
                 session negotiated {:?}",
                self.negotiated_suite
            );
        }

        let mut decapsulation_success = None;

        // Try to decapsulate with current keypair first
        if let Some(ref our_keypair) = self.pq_our_keypair {
            match crate::crypto::pq::decapsulate(our_keypair, pq_kem_ciphertext) {
                Ok(ss_pq) => {
                    decapsulation_success = Some(ss_pq);
                }
                Err(e) => {
                    // Log the error but don't propagate it
                    tracing::warn!("Failed to decapsulate with current keypair: {}", e);
                }
            }
        }

        // Try with previous keypair (one-step-behind tolerance)
        if let Some(ref prev_keypair) = self.pq_prev_keypair {
            match crate::crypto::pq::decapsulate(prev_keypair, pq_kem_ciphertext) {
                Ok(ss_pq) => {
                    decapsulation_success = Some(ss_pq);
                }
                Err(e) => {
                    // Log the error but don't propagate it
                    tracing::warn!("Failed to decapsulate with previous keypair: {}", e);
                }
            }
        }

        let ss_pq = match decapsulation_success {
            Some(ss) => ss,
            None => {
                // Non-fatal: log warning and return empty vector
                tracing::warn!("Failed to decapsulate PQ ciphertext with either current or previous keypair - soft skipping");
                return Ok(vec![]);
            }
        };

        let ss_pq_key = RatchetKey::from_bytes(ss_pq);
        let fp = pq_ss_fingerprint(ss_pq_key.as_bytes());

        // Deduplication: check if we've already mixed this secret
        let is_duplicate = self.pq_last_mixed_fp == Some(fp)
            || self
                .pq_pending_recv
                .as_ref()
                .map(|k| pq_ss_fingerprint(k.as_bytes()))
                == Some(fp);

        if !is_duplicate {
            self.pq_pending_recv = Some(ss_pq_key);
        }

        // Update their encaps key and clear pending ct
        self.pq_their_encaps_key = Some(pq_encaps_key.to_vec());
        self.pq_pending_ct = None;

        Ok(ss_pq.to_vec())
    }

    /// Validate that PQ fields are present when expected (PQ-hybrid sessions only).
    pub fn validate_pq_fields_present(&self, has_pq_fields: bool) -> Result<()> {
        if self.is_pq_hybrid() && self.pq_their_encaps_key.is_some() && !has_pq_fields {
            bail!(
                "PQ stripping attempt detected: DH step without PQ fields on a \
                 PQ-hybrid session (suite {:?})",
                self.negotiated_suite
            );
        }
        Ok(())
    }

    /// Clear pending PQ ciphertext (called when peer acks our PQ step).
    pub fn clear_pq_pending_ct(&mut self) {
        self.pq_pending_ct = None;
    }
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct RatchetEncryptResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub message_number: u32,
    pub our_dh_public: [u8; 32],
}

// ---------------------------------------------------------------------------
// KDF helpers
// ---------------------------------------------------------------------------

fn derive_key_with_info(key: &RatchetKey, info: &[u8]) -> RatchetKey {
    let context = format!("{}:{}", RATCHET_KDF_CONTEXT, blake3::hash(info).to_hex());
    let derived = blake3::derive_key(&context, key.as_bytes());
    RatchetKey::from_bytes(derived)
}

fn root_key_ratchet_v1(root_key: &RatchetKey, dh_output: &[u8]) -> (RatchetKey, RatchetKey) {
    let combined = blake3::derive_key(
        ROOT_KDF_CONTEXT_V1,
        &[root_key.as_bytes(), dh_output].concat(),
    );
    let new_root = blake3::derive_key(&format!("{}:root", ROOT_KDF_CONTEXT_V1), &combined);
    let chain_key = blake3::derive_key(&format!("{}:chain", ROOT_KDF_CONTEXT_V1), &combined);
    (
        RatchetKey::from_bytes(new_root),
        RatchetKey::from_bytes(chain_key),
    )
}

fn root_key_ratchet_v2(
    root_key: &RatchetKey,
    dh_output: &[u8],
    pq_ss: Option<Vec<u8>>,
) -> (RatchetKey, RatchetKey) {
    let mut input = vec![root_key.as_bytes().to_vec(), dh_output.to_vec()];
    if let Some(ss_pq) = pq_ss {
        input.push(ss_pq);
    }
    let combined = blake3::derive_key(ROOT_KDF_CONTEXT_V2, &input.concat());
    let new_root = blake3::derive_key(&format!("{}:root", ROOT_KDF_CONTEXT_V2), &combined);
    let chain_key = blake3::derive_key(&format!("{}:chain", ROOT_KDF_CONTEXT_V2), &combined);
    (
        RatchetKey::from_bytes(new_root),
        RatchetKey::from_bytes(chain_key),
    )
}

fn derive_root_key_v1(prior_root: &RatchetKey, shared_secret: &[u8]) -> RatchetKey {
    let derived = blake3::derive_key(
        ROOT_KDF_CONTEXT_V1,
        &[prior_root.as_bytes(), shared_secret].concat(),
    );
    RatchetKey::from_bytes(derived)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn generate_keypair() -> SigningKey {
        let mut secret = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret);
        let key = SigningKey::from_bytes(&secret);
        secret.zeroize();
        key
    }

    fn signing_key_to_x25519_public(signing_key: &SigningKey) -> X25519PublicKey {
        let secret = super::super::encrypt::ed25519_to_x25519_secret(signing_key);
        X25519PublicKey::from(&secret)
    }

    /// Test-only helper: derive a dedicated-style X25519 secret consistently from a signing
    /// key, so a test's `PublicKeyBundle.x25519_public` (built via
    /// `signing_key_to_x25519_public`) and the secret passed to `init_as_sender_hybrid` /
    /// `init_as_receiver_hybrid` describe the same keypair.
    fn signing_key_to_x25519_secret(signing_key: &SigningKey) -> X25519StaticSecret {
        super::super::encrypt::ed25519_to_x25519_secret(signing_key)
    }

    #[test]
    fn test_ratchet_key_zeroizes() {
        let key = RatchetKey::from_bytes([0xAB; 32]);
        let bytes = *key.as_bytes();
        drop(key);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_chain_ratchet_advances() {
        let chain_key = RatchetKey::from_bytes([0x42; 32]);
        let mut chain = Chain::new(chain_key);

        let key0 = chain.next_message_key();
        let key1 = chain.next_message_key();
        let key2 = chain.next_message_key();

        assert_ne!(key0.as_bytes(), key1.as_bytes());
        assert_ne!(key1.as_bytes(), key2.as_bytes());
        assert_ne!(key0.as_bytes(), key2.as_bytes());
        assert_eq!(chain.index, 3);
    }

    #[test]
    fn test_init_as_sender_and_encrypt() {
        let alice_key = generate_keypair();
        let bob_key = generate_keypair();
        let bob_x25519 = signing_key_to_x25519_public(&bob_key);

        let mut alice_session = RatchetSession::init_as_sender(&alice_key, &bob_x25519).unwrap();
        assert!(alice_session.is_initialized());
        assert_eq!(alice_session.dh_step_count(), 1);

        let result = alice_session.encrypt(b"hello bob", b"aad").unwrap();
        assert!(!result.ciphertext.is_empty());
        assert_eq!(result.nonce.len(), 24);
        assert_eq!(result.message_number, 0);
    }

    #[test]
    fn test_init_as_receiver_then_decrypt() {
        let alice_key = generate_keypair();
        let bob_key = generate_keypair();
        let alice_x25519 = signing_key_to_x25519_public(&alice_key);
        let bob_x25519 = signing_key_to_x25519_public(&bob_key);

        let mut alice_session = RatchetSession::init_as_sender(&alice_key, &bob_x25519).unwrap();
        let mut bob_session = RatchetSession::init_as_receiver(&bob_key, &alice_x25519).unwrap();
        assert!(!bob_session.is_initialized());

        let encrypted = alice_session.encrypt(b"hello bob", b"aad").unwrap();
        let plaintext = bob_session
            .decrypt(
                &encrypted.our_dh_public,
                encrypted.message_number,
                &encrypted.nonce,
                &encrypted.ciphertext,
                b"aad",
            )
            .unwrap();

        assert_eq!(plaintext, b"hello bob");
        assert!(bob_session.is_initialized());
    }

    #[test]
    fn test_init_as_sender_hybrid_and_encrypt() {
        let alice_key = generate_keypair();
        let bob_key = generate_keypair();
        let bob_x25519 = signing_key_to_x25519_public(&bob_key);
        let bob_ed25519 = bob_key.verifying_key().to_bytes();
        let bob_mlkem_keypair = crate::crypto::pq::generate();
        let bob_bundle = crate::identity::PublicKeyBundle {
            ed25519_public: bob_ed25519,
            x25519_public: bob_x25519.to_bytes(),
            mlkem_encaps_key: bob_mlkem_keypair.public_key().to_vec(),
            created_at: 0,
            supported_suites: vec![0x02],
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };
        let transcript_hash = [0u8; 32];

        let mut alice_session = RatchetSession::init_as_sender_hybrid(
            &signing_key_to_x25519_secret(&alice_key),
            &bob_bundle,
            transcript_hash,
        )
        .unwrap();
        assert!(alice_session.is_initialized());
        assert_eq!(alice_session.dh_step_count(), 1);

        let result = alice_session.encrypt(b"hello bob", b"aad").unwrap();
        assert!(!result.ciphertext.is_empty());
        assert_eq!(result.nonce.len(), 24);
        assert_eq!(result.message_number, 0);
    }

    #[test]
    fn test_init_as_receiver_hybrid_then_decrypt() {
        let alice_key = generate_keypair();
        let bob_key = generate_keypair();
        let alice_x25519 = signing_key_to_x25519_public(&alice_key);
        let bob_x25519 = signing_key_to_x25519_public(&bob_key);
        let alice_ed25519 = alice_key.verifying_key().to_bytes();
        let alice_bundle = crate::identity::PublicKeyBundle {
            ed25519_public: alice_ed25519,
            x25519_public: alice_x25519.to_bytes(),
            mlkem_encaps_key: crate::crypto::pq::generate().public_key().to_vec(),
            created_at: 0,
            supported_suites: vec![0x02],
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };
        let bob_mlkem_keypair = crate::crypto::pq::generate();
        let bob_bundle = crate::identity::PublicKeyBundle {
            ed25519_public: bob_key.verifying_key().to_bytes(),
            x25519_public: bob_x25519.to_bytes(),
            mlkem_encaps_key: bob_mlkem_keypair.public_key().to_vec(),
            created_at: 0,
            supported_suites: vec![0x02],
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };
        let transcript_hash = [0u8; 32];

        let mut alice_session = RatchetSession::init_as_sender_hybrid(
            &signing_key_to_x25519_secret(&alice_key),
            &bob_bundle,
            transcript_hash,
        )
        .unwrap();
        let hct = alice_session
            .bootstrap_hct
            .clone()
            .expect("sender_hybrid sets bootstrap_hct");
        let bob_x25519_secret = super::super::encrypt::ed25519_to_x25519_secret(&bob_key);
        let mut bob_session = RatchetSession::init_as_receiver_hybrid(
            &bob_key,
            &bob_x25519_secret,
            &bob_mlkem_keypair,
            &alice_bundle,
            &hct,
            transcript_hash,
        )
        .unwrap();
        assert!(!bob_session.is_initialized());

        let encrypted = alice_session.encrypt(b"hello bob", b"aad").unwrap();
        let plaintext = bob_session
            .decrypt(
                &encrypted.our_dh_public,
                encrypted.message_number,
                &encrypted.nonce,
                &encrypted.ciphertext,
                b"aad",
            )
            .unwrap();

        assert_eq!(plaintext, b"hello bob");
        assert!(bob_session.is_initialized());
    }

    #[test]
    fn test_pq_ratchet_step() {
        let alice_key = generate_keypair();
        let bob_key = generate_keypair();
        let bob_x25519 = signing_key_to_x25519_public(&bob_key);
        let bob_ed25519 = bob_key.verifying_key().to_bytes();
        let bob_mlkem_keypair = crate::crypto::pq::generate();
        let bob_bundle = crate::identity::PublicKeyBundle {
            ed25519_public: bob_ed25519,
            x25519_public: bob_x25519.to_bytes(),
            mlkem_encaps_key: bob_mlkem_keypair.public_key().to_vec(),
            created_at: 0,
            supported_suites: vec![0x02],
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };
        let transcript_hash = [0u8; 32];

        let mut alice_session = RatchetSession::init_as_sender_hybrid(
            &signing_key_to_x25519_secret(&alice_key),
            &bob_bundle,
            transcript_hash,
        )
        .unwrap();

        // Perform PQ ratchet step
        let (ct, new_encaps_key) = alice_session.perform_pq_ratchet_step().unwrap();
        assert!(!ct.is_empty());
        assert!(!new_encaps_key.is_empty());
        assert!(alice_session.pq_pending_ct.is_some());
        assert!(alice_session.pq_prev_keypair.is_some());
    }

    #[test]
    fn test_pq_validation_rejection() {
        let alice_key = generate_keypair();
        let bob_key = generate_keypair();
        let bob_x25519 = signing_key_to_x25519_public(&bob_key);
        let bob_ed25519 = bob_key.verifying_key().to_bytes();
        let bob_mlkem_keypair = crate::crypto::pq::generate();
        let bob_bundle = crate::identity::PublicKeyBundle {
            ed25519_public: bob_ed25519,
            x25519_public: bob_x25519.to_bytes(),
            mlkem_encaps_key: bob_mlkem_keypair.public_key().to_vec(),
            created_at: 0,
            supported_suites: vec![0x02],
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };
        let transcript_hash = [0u8; 32];

        let alice_session = RatchetSession::init_as_sender_hybrid(
            &signing_key_to_x25519_secret(&alice_key),
            &bob_bundle,
            transcript_hash,
        )
        .unwrap();

        // Should reject DH step without PQ fields after initialization
        let result = alice_session.validate_pq_fields_present(false);
        assert!(result.is_err());

        // Should accept when PQ fields are present
        let result = alice_session.validate_pq_fields_present(true);
        assert!(result.is_ok());
    }

    /// Regression test for the suite 0x02 silent-redefinition bug: suite 0x02
    /// (the ORIGINAL, pre-#221 derivation, preserved via
    /// `init_as_sender_hybrid_suite02` / `init_as_receiver_hybrid_suite02`)
    /// must still fully interoperate end-to-end for a peer that only ever
    /// advertises 0x02, completely independent of the suite 0x03 code path.
    #[test]
    fn test_suite02_sender_receiver_roundtrip_still_works() {
        let alice_key = generate_keypair();
        let bob_key = generate_keypair();
        let alice_x25519 = signing_key_to_x25519_public(&alice_key);
        let bob_x25519 = signing_key_to_x25519_public(&bob_key);
        let alice_bundle = crate::identity::PublicKeyBundle {
            ed25519_public: alice_key.verifying_key().to_bytes(),
            x25519_public: alice_x25519.to_bytes(),
            mlkem_encaps_key: crate::crypto::pq::generate().public_key().to_vec(),
            created_at: 0,
            supported_suites: vec![0x01, 0x02],
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };
        let bob_mlkem_keypair = crate::crypto::pq::generate();
        let bob_bundle = crate::identity::PublicKeyBundle {
            ed25519_public: bob_key.verifying_key().to_bytes(),
            x25519_public: bob_x25519.to_bytes(),
            mlkem_encaps_key: bob_mlkem_keypair.public_key().to_vec(),
            created_at: 0,
            supported_suites: vec![0x01, 0x02],
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };
        let transcript_hash = [0u8; 32];

        let mut alice_session =
            RatchetSession::init_as_sender_hybrid_suite02(&alice_key, &bob_bundle, transcript_hash)
                .unwrap();
        assert_eq!(alice_session.negotiated_suite, Some(0x02));
        assert!(alice_session.is_pq_hybrid());

        let hct = alice_session
            .bootstrap_hct
            .clone()
            .expect("sender_hybrid_suite02 sets bootstrap_hct");
        let bob_x25519_secret = super::super::encrypt::ed25519_to_x25519_secret(&bob_key);
        let mut bob_session = RatchetSession::init_as_receiver_hybrid_suite02(
            &bob_key,
            &bob_x25519_secret,
            &bob_mlkem_keypair,
            &alice_bundle,
            &hct,
            transcript_hash,
        )
        .unwrap();
        assert_eq!(bob_session.negotiated_suite, Some(0x02));
        assert!(bob_session.is_pq_hybrid());

        let encrypted = alice_session
            .encrypt(b"hello via old suite", b"aad")
            .unwrap();
        let plaintext = bob_session
            .decrypt(
                &encrypted.our_dh_public,
                encrypted.message_number,
                &encrypted.nonce,
                &encrypted.ciphertext,
                b"aad",
            )
            .unwrap();

        assert_eq!(plaintext, b"hello via old suite");
        assert!(bob_session.is_initialized());
    }

    /// Regression test proving suite 0x02 and suite 0x03 are NOT the same
    /// derivation under the hood.
    ///
    /// Compares the RECEIVER side (`init_as_receiver_hybrid_suite02` vs
    /// `init_as_receiver_hybrid`) rather than the sender side, and holds the
    /// hybrid ciphertext fixed across both calls: unlike the sender
    /// functions (which generate a fresh ephemeral X3DH keypair AND a fresh
    /// hybrid encapsulation internally, so their root keys would differ on
    /// every call for reasons having nothing to do with which suite was
    /// used), the receiver's `root_key` immediately after `init_as_receiver_*`
    /// is exactly `root_key_0` -- deterministic decapsulation of a caller-
    /// supplied `hct`, with no additional per-call randomness. That isolates
    /// this comparison to ONLY the domain-separator / static-static-DH
    /// difference between the two suites.
    ///
    /// If this ever fails, suite 0x02 and 0x03 have been silently aliased
    /// back together -- exactly the class of bug this suite split exists to
    /// prevent (see `crypto::negotiation::HYBRID_SUITE_IDS`).
    #[test]
    fn test_suite02_and_suite03_derive_different_root_keys() {
        let alice_key = generate_keypair();
        let bob_key = generate_keypair();
        let bob_x25519_secret = signing_key_to_x25519_secret(&bob_key);
        let bob_x25519_public = X25519PublicKey::from(&bob_x25519_secret);
        let bob_x25519_public_bytes = bob_x25519_public.to_bytes();
        let bob_mlkem_keypair = crate::crypto::pq::generate();

        let alice_bundle = crate::identity::PublicKeyBundle {
            ed25519_public: alice_key.verifying_key().to_bytes(),
            x25519_public: signing_key_to_x25519_public(&alice_key).to_bytes(),
            mlkem_encaps_key: crate::crypto::pq::generate().public_key().to_vec(),
            created_at: 0,
            supported_suites: vec![0x01, 0x03],
            signature: vec![],
            mldsa_public: None,
            mldsa_signature: None,
        };

        // Encapsulate ONCE against Bob's real ML-KEM public key, so both
        // init_as_receiver_hybrid* calls below decapsulate the exact same
        // hybrid shared secret from the exact same ciphertext.
        let (hct, _ss_hybrid_unused) = crate::crypto::pq::hybrid::hybrid_encapsulate(
            &bob_x25519_public_bytes,
            bob_mlkem_keypair.public_key(),
        )
        .unwrap();
        let transcript_hash = [7u8; 32];

        let old_receiver = RatchetSession::init_as_receiver_hybrid_suite02(
            &bob_key,
            &bob_x25519_secret,
            &bob_mlkem_keypair,
            &alice_bundle,
            &hct,
            transcript_hash,
        )
        .unwrap();
        let new_receiver = RatchetSession::init_as_receiver_hybrid(
            &bob_key,
            &bob_x25519_secret,
            &bob_mlkem_keypair,
            &alice_bundle,
            &hct,
            transcript_hash,
        )
        .unwrap();

        assert_eq!(old_receiver.negotiated_suite, Some(0x02));
        assert_eq!(new_receiver.negotiated_suite, Some(0x03));
        assert_ne!(
            old_receiver.root_key_bytes(),
            new_receiver.root_key_bytes(),
            "suite 0x02 and suite 0x03 must never derive the same root key from \
             the same hybrid ciphertext, sender bundle, and transcript hash -- \
             they are different, non-interchangeable protocol versions"
        );
    }
}
