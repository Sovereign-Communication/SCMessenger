# E-00 IMPLEMENTATION PACKET -- wire ratchet/PQ subsystem into IronCore production path

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

> **REMOVED 2026-08-24 (PR #221):** every `SCM_RATCHET_DISABLE` / `ratchet_disabled()`
> reference below (the kill switch this packet specified) has been deleted from
> `core/src/iron_core.rs`. It bypassed the ingress signature verification added by
> PR #221 and reopened the original P0 message-forgery hole; nothing in the repo
> ever set the variable. This packet is kept as the historical implementation
> record for the rest of the E-00 design (session wiring, lock order, Drift PQ
> extension) -- do not treat the kill-switch sections as current guidance, and do
> not reintroduce it.

STATUS: APPROVED 2026-07-17 (operator). Pre-flight THINK analysis + Fusion
adversarial panel UNANIMOUS PASS (3 rounds; verdict file
tmp/fusion-e00-analysis-verdict-r3.md). This packet is cleared for CODER
implementation. Kill switch = env SCM_RATCHET_DISABLE (REMOVED 2026-08-24, see banner above).

YOU ARE A CODER WORKER. Produce unified-diff file blocks (the orchestrator
runs you in --mode diff). Do NOT run cargo/gradlew -- the orchestrator owns
all build verification. Do NOT commit. Implement exactly the design below;
it is Fusion-approved and operator-approved. Do NOT improvise on the design.
If you hit an ambiguity not covered here, implement the most conservative
option that preserves the no-message-loss and no-silent-PQ-downgrade
invariants and note it in your report.

NO EMOJI. Use [OK]/[INFO]/[WARNING].

## DIFF DISCIPLINE (CRITICAL -- round 1 failed by violating this)

Produce MINIMAL SURGICAL unified-diff hunks. The orchestrator applies your
diff with `git apply`. A hunk that re-emits unchanged existing code as added
lines WILL NOT APPLY and produces a vacuous failure.
- Include ONLY the lines that change, plus the minimum surrounding context
  (3 lines) for `git apply` to locate the hunk. NEVER re-emit an entire
  unchanged function as an addition.
- Do NOT duplicate existing methods. The file core/src/drift/envelope.rs
  ALREADY has a `fn sign(&self, signing_key) -> [u8;64]` at line ~462-502
  and `from_legacy_envelope` already calls `drift_env.sign(signing_key)` at
  line ~456. You must EXTEND the existing `sign()` (add PQ-extension hashing
  after its ratchet-extension block at line ~494), NOT add a second `sign()`.
  `from_v2_envelope` must follow the SAME pattern as `from_legacy_envelope`:
  build the struct with `signature: [0u8;64]`, call `drift_env.sign(signing_key)`,
  set the signature, return. Do not re-implement signing inline.
- Cover ALL files in this dispatch (envelope.rs, relay.rs, codec.rs,
  iron_core.rs, and the new test file). Do not stop after the first file.
- Every `DriftEnvelope {` construction site in the codebase MUST get the 4
  new PQ fields initialized. The known sites are:
    core/src/drift/envelope.rs:319  (from_bytes return)
    core/src/drift/envelope.rs:436  (from_legacy_envelope)
    core/src/drift/envelope.rs:510  (test make_test_envelope)
    core/src/message/codec.rs:164, :454, :504  (3 sites)
    core/src/drift/relay.rs:486  (relay construction)
  All non-V2 sites set the 4 fields = None. If cargo check reports any other
  missing-field site, fix it too (you have write access to all --files).
- Confirm current line numbers by reading the embedded file context before
  writing hunks; line numbers above are from 2026-07-17 and may have shifted.

## GOAL

Route IronCore::prepare_message_internal AND IronCore::receive_message through
the ratchet-aware fallback wrappers (encrypt_with_ratchet_fallback /
decrypt_with_ratchet_fallback) instead of the bare legacy
crypto::encrypt_message / decrypt_message. Preserve legacy fallback for peers
with no session/bundle (mixed fleet). Add an env-var kill switch
SCM_RATCHET_DISABLE that forces the legacy path. Add a Drift binary V2 PQ
extension so V2 (PQ-hybrid) envelopes round-trip through the Drift wire format
without losing PQ material. Fix decode_wire_envelope to try Drift-binary first.

After this lands, every real CLI/FFI message has forward secrecy + PQ
protection when a ratchet session exists, and legacy behavior otherwise.

## INVARIANTS (never violate)

1. No message loss tolerated in the mixed fleet (old + new builds).
2. PQ material (pq_kem_ciphertext / pq_encaps_key / transcript_hash) MUST
   round-trip intact through Drift to a PQ-capable receiver -- never silently
   dropped (that is a silent PQ downgrade).
3. Legacy fallback remains for peers with no session / no bundle.
4. Lock order: acquire identity lock (read or write) BEFORE ratchet_sessions
   lock (read or write) whenever both are needed in one scope. Never the
   reverse (proven-safe order; all 3 existing dual-lock sites obey it).
5. Kill switch SCM_RATCHET_DISABLE (env) forces the EXACT current legacy path
   for both send and receive -- zero behavior change when set.

## EXACT CHANGES PER FILE

### FILE 1: core/src/drift/envelope.rs

1a. Extend struct DriftEnvelope (envelope.rs:38-80) with 4 new Option fields
    AFTER ratchet_message_number:
      pub suite: Option<u8>,
      pub pq_kem_ciphertext: Option<Vec<u8>>,
      pub pq_encaps_key: Option<Vec<u8>>,
      pub transcript_hash: Option<Vec<u8>>,
    All default None for V1 (no behavior change for the V1 path).

1b. Update every DriftEnvelope literal in the codebase to initialize these 4
    fields (the struct has no ..Default; find all `DriftEnvelope {` construction
    sites via grep and add the 4 fields = None unless it is the V2 conversion).

1c. to_bytes (envelope.rs:141-192): AFTER the existing ratchet extension block
    (envelope.rs:180-189), append a PQ extension block:
      if self.suite.is_some() || self.pq_kem_ciphertext.is_some()
         || self.pq_encaps_key.is_some() || self.transcript_hash.is_some() {
          buf.push(0x01); // pq_flag = present
          buf.push(self.suite.unwrap_or(0x02));
          append_blob(&mut buf, &self.pq_kem_ciphertext);   // 4-byte LE len + bytes (len 0 if None)
          append_blob(&mut buf, &self.pq_encaps_key);
          append_blob(&mut buf, &self.transcript_hash);
      } else {
          buf.push(0x00); // pq_flag = absent
      }
    where append_blob pushes (len as u32).to_le_bytes() then the bytes (empty
    vec if the Option is None). NOTE: always emit the pq_flag byte (0x00 or
    0x01) so the parser can deterministically detect the extension presence.
    Keep MAX_CIPHERTEXT check on ciphertext only (PQ blobs are trailing
    metadata, not ciphertext; do not let them trip the 65535 ciphertext cap).

1d. from_bytes (envelope.rs:200-337): AFTER the existing ratchet extension read
    (envelope.rs:290-317), add a PQ extension read. At the point where the
    ratchet ext finishes (the `offset` after ratchet data), if
    offset < data.len(): read pq_flag = data[offset]; offset += 1.
      if pq_flag == 0x01: read suite = data[offset]; offset += 1;
        then read three 4-byte-LE-length-prefixed blobs into
        pq_kem_ciphertext, pq_encaps_key, transcript_hash (empty vec -> None,
        non-empty -> Some(vec)). Validate lengths against remaining buffer
        (BufferTooShort on overrun). 
      else (pq_flag == 0x00 or any other): suite/pq_*/transcript = None.
    If offset == data.len() after the ratchet ext (no trailing bytes): the 4
    PQ fields = None (this is the old-envelope case). Update the final
    DriftEnvelope { ... } return (envelope.rs:319-336) to include the 4 new
    fields. Update FIXED_OVERHEAD only if the mandatory prefix grew -- it does
    NOT (the pq_flag is trailing-optional, NOT part of the 187 fixed bytes);
    leave FIXED_OVERHEAD = 187.

1e. Add a new constructor:
      pub fn from_v2_envelope(
          v2: crate::message::EnvelopeV2,
          message_id: String,
          recipient_public_key: [u8; 32],
          signing_key: &ed25519_dalek::SigningKey,
      ) -> Result<Self, DriftError>
    Mirror from_legacy_envelope (envelope.rs:388+) for header/routing/crypto
    field setup (uuid parse, recipient_hint via hint_from_public_key,
    created_at, ttl 7d, sender_public_key from v2.sender_public_key (validate
    32B), ephemeral_public_key from v2.ephemeral_public_key (validate 32B),
    nonce from v2.nonce (validate 24B), ciphertext = v2.ciphertext). Then set:
      ratchet_dh_public = v2.ratchet_dh_public.map(|v| v.try_into().unwrap_or([0u8;32])) (validate 32B, error if wrong len),
      ratchet_message_number = v2.ratchet_message_number,
      suite = Some(v2.suite),
      pq_kem_ciphertext = v2.pq_kem_ciphertext,
      pq_encaps_key = v2.pq_encaps_key,
      transcript_hash = v2.transcript_hash.
    Compute the signature via the EXISTING `self.sign(signing_key)` method
    (envelope.rs:462) -- do NOT re-implement signing. After you EXTEND sign()
    (see 1g below) to hash the PQ extension bytes, from_v2_envelope simply
    builds the struct with `signature: [0u8;64]`, calls
    `drift_env.sign(signing_key)`, assigns the result, and returns -- exactly
    like from_legacy_envelope does at envelope.rs:455-457. Set envelope_type =
    EncryptedMessage, compressed = (v2.ciphertext.len() > COMPRESSION_THRESHOLD),
    hop_count = 0, priority = 128 (match from_legacy_envelope).

1f. Add a reverse mapper for the receive path:
      pub fn to_wire_envelope(&self) -> crate::message::WireEnvelope
    If self.suite.is_some() || self.pq_kem_ciphertext.is_some() ||
       self.pq_encaps_key.is_some() || self.transcript_hash.is_some():
      return WireEnvelope::V2(EnvelopeV2 {
        suite: self.suite.unwrap_or(0x02),
        sender_public_key: self.sender_public_key.to_vec(),
        ephemeral_public_key: self.ephemeral_public_key.to_vec(),
        nonce: self.nonce.to_vec(),
        ciphertext: self.ciphertext.clone(),
        ratchet_dh_public: self.ratchet_dh_public.map(|k| k.to_vec()),
        ratchet_message_number: self.ratchet_message_number,
        pq_kem_ciphertext: self.pq_kem_ciphertext.clone(),
        pq_encaps_key: self.pq_encaps_key.clone(),
        transcript_hash: self.transcript_hash.clone(),
      });
    else: return WireEnvelope::V1(self.to_legacy_envelope()) (envelope.rs:371).
    Keep to_legacy_envelope as-is (it is still used for the V1 arm).

1g. EXTEND the existing `fn sign(&self, signing_key: &ed25519_dalek::SigningKey)
    -> [u8; 64]` (envelope.rs:462-502) -- do NOT duplicate it. After its
    ratchet-extension hashing block (envelope.rs:488-495, which hashes a 0x01
    or 0x00 flag + ratchet bytes), add a PQ-extension hashing block that
    mirrors to_bytes (1c) EXACTLY so the signature covers the PQ material:
      if self.suite.is_some() || self.pq_kem_ciphertext.is_some()
         || self.pq_encaps_key.is_some() || self.transcript_hash.is_some() {
          hasher.update(&[0x01]); // pq_flag = present
          hasher.update(&[self.suite.unwrap_or(0x02)]);
          // hash each blob as u32 LE len + bytes (empty -> len 0, no bytes)
          hash_blob(&mut hasher, &self.pq_kem_ciphertext);
          hash_blob(&mut hasher, &self.pq_encaps_key);
          hash_blob(&mut hasher, &self.transcript_hash);
      } else {
          hasher.update(&[0x00]); // pq_flag = absent
      }
    where hash_blob hashes (len as u32).to_le_bytes() then the bytes. This
    MUST match to_bytes byte-for-byte so a signed V2 Drift envelope verifies.
    The existing V1 path (no PQ fields) hashes 0x00 and is unchanged.

### FILE 2: core/src/message/codec.rs

2a. decode_wire_envelope (codec.rs:213-280): change the dispatch order to try
    Drift-binary FIRST. At the top of the function (after the size checks), add:
      if !buf.is_empty() && buf[0] == crate::drift::DRIFT_VERSION /* 0x01 */ {
          if let Ok(drift_env) = crate::drift::DriftEnvelope::from_bytes(buf) {
              return Ok(drift_env.to_wire_envelope());
          }
          // fall through on Drift parse failure
      }
    Then keep the existing WIRE_TAG_V2 (0x02) check (codec.rs:221) and the V1
    bincode fallback (codec.rs:264). This makes the order: Drift-binary (0x01)
    -> V2-tag (0x02) -> V1-bincode. Drift-binary and V2-tag cannot collide
    (0x01 vs 0x02) and a legitimate V1 bincode Envelope always leads with 0x20
    (32-byte Vec<u8> length prefix), so there is no ambiguity. This fixes the
    GAP-2 wiring gap: the post-wiring receive path (which uses
    decode_wire_envelope) can now decode the Drift-binary V1 envelopes the
    current send path emits. Make sure DRIFT_VERSION is imported (codec.rs
    already references it at codec.rs:13,113,165 per the gap analysis).

2b. The 3 `DriftEnvelope {` construction sites in codec.rs (lines ~164, ~454,
    ~504 -- likely in encode_envelope and tests) MUST have the 4 new PQ fields
    added = None. These are mechanical field additions; produce minimal hunks.

### FILE 2b: core/src/drift/relay.rs

2c. The `DriftEnvelope {` construction site at relay.rs:486 MUST have the 4
    new PQ fields added = None (relay constructs a DriftEnvelope for cover
    traffic / relay purposes; it is never V2). Minimal hunk.

### FILE 3: core/src/iron_core.rs

3a. Add a kill-switch helper (module-level or as an associated fn). Use a
    OnceLock<AtomicBool> read from env SCM_RATCHET_DISABLE on first access:
      fn ratchet_disabled() -> bool {
          static FLAG: std::sync::OnceLock<std::sync::atomic::AtomicBool> =
              std::sync::OnceLock::new();
          let f = FLAG.get_or_init(|| {
              std::sync::atomic::AtomicBool::new(
                  std::env::var("SCM_RATCHET_DISABLE").map(|v| !v.is_empty() && v != "0" && v.to_lowercase() != "false").unwrap_or(false)
              )
          });
          f.load(std::sync::atomic::Ordering::Relaxed)
      }
    All-platform, zero plumbing (works on FFI mobile + CLI + WASM env where
    available; WASM env access is best-effort and defaults to enabled).

3b. prepare_message_internal (iron_core.rs:636-694): replace the body of the
    encryption + drift-wrap section (iron_core.rs:667-679). New logic:
      let mut envelope_data = if ratchet_disabled() {
          // LEGACY PATH (verbatim current behavior)
          let envelope = encrypt_message(&keys.signing_key, &recipient_pk, &message_bytes)
              .map_err(|_| IronCoreError::CryptoError)?;
          let drift_env = crate::drift::DriftEnvelope::from_legacy_envelope(
              envelope, message_id.clone(), recipient_pk, &keys.signing_key,
          ).map_err(|_| IronCoreError::Internal)?;
          drift_env.to_bytes().map_err(|_| IronCoreError::Internal)?
      } else {
          // RATCHET PATH: hold identity.read() then ratchet_sessions.write()
          // (identity is already held as `identity` guard from line 643).
          // Build our self bundle (cache it -- see 3d).
          let our_bundle = <self-bundle-cache lookup, see 3d>;
          // Fetch recipient bundle (recipient_id is hex(ed25519 public)).
          let recipient_bundle = self.contact_manager.read()
              .get_contact_bundle(recipient_id).ok().flatten();
          let peer_id = recipient_id.to_string(); // recipient_id IS the peer id used by session mgr (hex ed25519 pub)
          let mut sessions = self.ratchet_sessions.write();
          let mut audit = self.audit_log.write();
          let wire = crate::crypto::encrypt::encrypt_with_ratchet_fallback(
              &keys.signing_key,
              recipient_bundle.as_ref(),
              &recipient_pk,
              &message_bytes,
              Some(&mut *sessions),
              &peer_id,
              our_bundle.as_ref(),
              false, // require_pq = false (mixed fleet; legacy fallback allowed)
              Some(&mut *audit),
          ).map_err(|_| IronCoreError::CryptoError)?;
          // Map WireEnvelope -> DriftEnvelope, then to_bytes.
          let drift_env = match wire {
              crate::message::WireEnvelope::V1(env) =>
                  crate::drift::DriftEnvelope::from_legacy_envelope(
                      env, message_id.clone(), recipient_pk, &keys.signing_key)
                      .map_err(|_| IronCoreError::Internal)?,
              crate::message::WireEnvelope::V2(env2) =>
                  crate::drift::DriftEnvelope::from_v2_envelope(
                      env2, message_id.clone(), recipient_pk, &keys.signing_key)
                      .map_err(|_| IronCoreError::Internal)?,
          };
          drift_env.to_bytes().map_err(|_| IronCoreError::Internal)?
      };
    IMPORTANT lock order: `identity` read guard is held from line 643 already;
    acquire ratchet_sessions.write() AFTER (inside this block). Do NOT drop
    identity first. audit_log.write() is acquired last (it has no inversion
    hazard with identity/ratchet_sessions -- confirm by checking no site takes
    audit_log then identity; if unsure, drop the audit_log borrow before any
    other lock and pass None for audit_log instead to avoid risk). Keep the
    existing onion-routing block (iron_core.rs:681-692) and everything after
    unchanged -- it operates on `envelope_data`.
    peer_id note: the session manager keys sessions by peer_id = hex of the
    peer's ed25519 public (encrypt.rs:594-595 derives blake3(sender_public_key)
    on receive; on send, encrypt_with_ratchet_fallback uses the peer_id YOU
    pass). Use recipient_id as the peer_id for send-side consistency with how
    prepare_message_internal identifies the recipient. If the session manager's
    receive-side keying (blake3 hash) differs from recipient_id (plain hex),
    document this mismatch in a comment -- it is the EXISTING behavior of the
    wrapper and is out of scope to change here; the wrapper already reconciles
    it internally via get_or_create.

3c. receive_message (iron_core.rs:2714-2731): replace the decode+decrypt
    section (iron_core.rs:2715-2726). New logic:
      let plaintext = if ratchet_disabled() {
          // LEGACY PATH (verbatim current behavior)
          let envelope = decode_envelope(&envelope_data).map_err(|e| {
              tracing::warn!("Failed to decode envelope: {:?}", e);
              IronCoreError::CryptoError
          })?;
          let identity = self.identity.read();
          let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
          // NOTE: must drop `identity` guard before acquiring ratchet_sessions
          // in the else branch to keep lock order clean; in the legacy branch
          // there is no ratchet_sessions access so this is fine.
          crate::crypto::decrypt_message(&keys.signing_key, &envelope).map_err(|e| {
              tracing::warn!("Failed to decrypt message: {:?}", e);
              IronCoreError::CryptoError
          })?
      } else {
          // RATCHET PATH
          let wire = crate::message::codec::decode_wire_envelope(&envelope_data).map_err(|e| {
              tracing::warn!("Failed to decode wire envelope: {:?}", e);
              IronCoreError::CryptoError
          })?;
          let identity = self.identity.read();
          let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
          // For V2 receive we need the recipient x25519 secret + mlkem keypair
          // + our bundle + sender bundle. Derive them from keys/contacts here.
          let our_bundle = <self-bundle-cache, see 3d>;
          // Sender bundle: derive peer_id from wire.sender_public_key via
          // blake3 hash (matches encrypt.rs:594-595 / 620-621), then look up
          // the contact bundle for that peer.
          let (sender_pubkey, peer_id) = match &wire {
              crate::message::WireEnvelope::V1(e) => (e.sender_public_key.clone(),
                  hex::encode(blake3::hash(&e.sender_public_key).as_bytes())),
              crate::message::WireEnvelope::V2(e2) => (e2.sender_public_key.clone(),
                  hex::encode(blake3::hash(&e2.sender_public_key).as_bytes())),
          };
          let sender_bundle = self.contact_manager.read()
              .get_contact_bundle(&hex::encode(&sender_pubkey)).ok().flatten();
          // recipient x25519 secret + mlkem keypair from IdentityKeys
          let recipient_x25519_secret = Some(&keys.x25519_encryption_secret);
          let our_mlkem_keypair = Some(&keys.mlkem_keypair);
          // Drop identity guard BEFORE acquiring ratchet_sessions.write() to
          // preserve identity-first lock order WITHOUT holding both as nested
          // write guards unnecessarily -- actually identity.read() ->
          // ratchet_sessions.write() IS the proven-safe order (GAP 3), so
          // holding identity.read() while taking ratchet_sessions.write() is
          // correct. Keep identity guard in scope here.
          let mut sessions = self.ratchet_sessions.write();
          crate::crypto::encrypt::decrypt_with_ratchet_fallback(
              &keys.signing_key,
              recipient_x25519_secret,
              &wire,
              Some(&mut *sessions),
              our_mlkem_keypair,
              our_bundle.as_ref(),
              sender_bundle.as_ref(),
          ).map_err(|e| {
              tracing::warn!("Failed to decrypt ratchet message: {:?}", e);
              IronCoreError::CryptoError
          })?
      };
    Then continue with decode_message(&plaintext) (iron_core.rs:2728) and the
    rest unchanged. Verify the field names: IdentityKeys has
    `x25519_encryption_secret` and `mlkem_keypair` (read keys.rs to confirm
    exact field names; the gap analysis confirmed sign_bundle uses
    keys.x25519_encryption_secret at keys.rs:325 and keys.mlkem_keypair at
    keys.rs:326). get_contact_bundle takes hex(ed25519_public) (contacts.rs:266)
    -- sender_pubkey is the ed25519 public so hex::encode(&sender_pubkey) is the
    right arg (NOT the blake3 hash -- blake3 hash is the session-manager peer_id;
    the CONTACT bundle is keyed by plain ed25519 hex). Use the correct key per
    accessor.

3d. Self-bundle cache: add a field to IronCore or a OnceLock-based cache so
    identity::sign_bundle(&keys) is called ONCE per IdentityKeys, not per send.
    Simplest: add `our_bundle_cache: Arc<RwLock<HashMap<String, PublicKeyBundle>>>`
    keyed by identity_id(), OR a single `Arc<RwLock<Option<PublicKeyBundle>>>`
    populated on first send. Populate via crate::identity::sign_bundle(keys) on
    first miss. read it for both prepare_message_internal (3b) and
    receive_message (3c). If sign_bundle fails, proceed with our_bundle = None
    (forces V1 fallback -- safe, no message loss). Document the cache.

### FILE 4 (NEW): core/tests/integration_e00_ratchet_wiring.rs

Add a new integration test proving the wiring works end-to-end:
  - Two IronCore instances (alice, bob) with separate sled temp backends.
  - Exchange public key bundles (alice publishes hers, bob publishes his;
    insert via contact_manager so get_contact_bundle returns them).
  - alice.prepare_message_internal to bob -> bob.receive_message -> assert
    plaintext matches and message_number advanced (inspect ratchet session
    state via ratchet_sessions, or send a SECOND message and assert it also
    decrypts -- proving the ratchet advanced rather than re-using a static
    key).
  - Legacy fallback test: a recipient with NO published bundle -> send still
    succeeds and receiver decrypts via legacy path.
  - Kill-switch test: set SCM_RATCHET_DISABLE=1 (use std::env::set_var in a
    #[serial] test or a separate process -- env vars are process-global so
    guard with a serialization or test in isolation) -> assert the legacy path
    is used (a ratchet session is NOT created for a new peer).
  - V2 round-trip test: if ML-KEM keypair generation is available in tests,
    publish bundles with suite 0x02, send, and assert the receiver gets a V2
    envelope with pq_kem_ciphertext populated and decrypts successfully.
Use the existing integration_pq_session.rs (core/tests/) as a template for
IronCore setup + temp sled backend. Keep the test deterministic and fast.

## VERIFY (the orchestrator runs this; you do NOT run builds)

The orchestrator will run: cargo check --workspace
Then: cargo test -p scmessenger-core --test integration_pq_session
      cargo test -p scmessenger-core --test integration_e2e
      cargo test -p scmessenger-core --test integration_e00_ratchet_wiring

## REPORT FORMAT

RESULT: DONE|BLOCKED|FAILED
FILES: <paths touched>
NOTES: <decisions made, any invariant-preserving conservative choice, anything
       the verifier must know -- max 8 lines>

## DO NOT

- Do NOT run cargo, gradlew, or any build.
- Do NOT commit or push.
- Do NOT move HANDOFF files.
- Do NOT add emoji.
- Do NOT invent line numbers -- read the files to confirm current line numbers
  before writing diffs (line numbers above are from 2026-07-17 and may have
  shifted).
- Do NOT change the on-wire Drift version byte (keep DRIFT_VERSION = 0x01); the
  PQ extension is a trailing optional block, not a version bump.
- Do NOT remove the legacy fallback paths -- they are mandatory for the mixed
  fleet.
