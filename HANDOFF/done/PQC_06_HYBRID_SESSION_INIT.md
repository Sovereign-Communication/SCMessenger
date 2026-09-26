# TASK: PQC-06 — Hybrid ratchet session establishment (suite 0x02)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Read `PQC_00_MASTER_PLAN.md` first. Depends on: PQC-02, PQC-03, PQC-04, PQC-05. Wave 2. Min tier: Sonnet. ADVERSARIAL REVIEW MANDATORY.

## Scope

Replace the classical-only session bootstrap with a hybrid one when suite 0x02 is negotiated. Anchors: `core/src/crypto/session_manager.rs` (`get_or_create_session`, `RatchetSession::init_as_sender`) and the root-key initialization inside `core/src/crypto/ratchet.rs`. The mesh is asynchronous — there is no interactive handshake; the initiator must be able to derive the session and send in one shot, exactly like today. ML-KEM supports this: encapsulation to the recipient's STATIC bundle key (from PQC-03) needs no round trip.

## Design (fixed)

Initiator (suite 0x02, has recipient's verified `PublicKeyBundle`):

```
(hct, ss_hybrid) = hybrid_encapsulate(bundle.x25519_public, bundle.mlkem_encaps_key)   // PQC-05
root_key_0 = blake3::derive_key("iron-core session-root v2 2026-07",
                                ss_hybrid || transcript_hash /* PQC-04, 32B */)
```

- `root_key_0` seeds the existing Double Ratchet root exactly where `init_as_sender` derives it today; the DH ratchet then runs unchanged on top (PQ ratchet steps come in PQC-07).
- The first outgoing envelope(s) of the session are `EnvelopeV2` with `suite = 0x02`, `pq_kem_ciphertext = Some(hct.mlkem_ciphertext)`, `ephemeral_public_key = hct.x25519_ephemeral_public`, `transcript_hash = Some(...)`. Repeat these fields on every envelope until the peer's first reply proves session establishment (mesh delivery is lossy/unordered; the responder must be able to bootstrap from ANY ONE of them).
- Responder: on receiving an EnvelopeV2 with `pq_kem_ciphertext` and no existing session: verify sender bundle known + transcript match (PQC-04), `hybrid_decapsulate` with its own identity bundle secrets, derive the same `root_key_0`, init as receiver, decrypt.
- Suite 0x01 peers: existing classical path, untouched.

## Steps

1. Extend `RatchetSession` state: `suite: u8`, `transcript_hash: [u8; 32]`, plus whatever PQC-04 already added. Persisted session state is JSON (`serialize_sessions`, key `ratchet_sessions_v1`): new fields get `#[serde(default)]`; bump storage key to `ratchet_sessions_v2` with one-time migration that copies v1 sessions in as `suite = 0x01` (keep reading v1 key forever; write only v2).
2. `init_as_sender_hybrid(...)` / `init_as_receiver_hybrid(...)` beside the existing initializers.
3. Wire into `encrypt_with_ratchet_fallback` / `decrypt_with_ratchet_fallback` (`core/src/crypto/encrypt.rs`): negotiation (PQC-04) decides which initializer runs. The bootstrap-fields-repeat logic lives at the session manager level (a flag `peer_confirmed` cleared once any inbound ratcheted envelope from the peer decrypts).
4. New integration test `core/tests/integration_pq_session.rs` (register in `core/Cargo.toml` like the others):
   - two identities with v2 bundles -> full send/receive both directions, out-of-order first messages (deliver message 3 first), decrypt OK;
   - lost-first-envelope: drop initiator's first envelope, deliver the second -> responder still bootstraps;
   - transcript mismatch -> establishment refused;
   - v2 initiator to v1-only peer -> classical session, envelopes are v1, no pq fields;
   - `require_pq = true` + v1-only peer -> send returns error.
5. Persistence test: establish hybrid session, save, reload (`RatchetSessionManager::load`), continue conversation.

## Definition of Done

- [ ] Standard gates PASS.
- [ ] `cargo test -p scmessenger-core --test integration_pq_session` green; `--test integration_e2e` and `--test integration_ironcore_roundtrip` still green.
- [ ] Adversarial review verdict recorded here.
- [ ] File moved to HANDOFF/done/ + committed.

## Do NOT

- Delete or bypass the classical initializers (v1 peers depend on them).
- Introduce a synchronous handshake round trip.
- Start PQ ratchet stepping (PQC-07). In this task, PQ material enters ONLY at session init.
