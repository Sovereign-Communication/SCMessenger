# CRITICAL: Ratchet + PQ subsystem is unreachable from IronCore's production message path

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

> **UPDATE 2026-08-24 (PR #221):** the `SCM_RATCHET_DISABLE` kill switch described
> below has been REMOVED. It bypassed the ingress signature verification added by
> PR #221 entirely, so setting it reopened the original P0 message-forgery hole
> byte-for-byte. No fleet ever set this variable (verified: no script, CI job,
> Dockerfile, Kotlin, Swift, or config referenced it, and the last public release
> predates this subsystem). Do not reintroduce it; there is no legacy-format
> fleet to support. `ratchet_disabled()` no longer exists in `core/src/iron_core.rs`.

Status: DONE 2026-07-17 -- implemented (dispatch A: DriftEnvelope PQ ext + decode order; dispatch B: IronCore send/receive wiring + kill switch), build-verified (cargo check + 1140 unit tests + 17 integration tests across 4 suites), Fusion adversarial review UNANIMOUS PASS (round 2, tmp/fusion-e00-adversarial-verdict-r2.md). Kill switch = env SCM_RATCHET_DISABLE (REMOVED 2026-08-24, see update above). Pre-existing from_bytes offset+=4 bug fixed (exposed by PQ ext). Audit logging enabled on legacy fallback.
Filed: 2026-07-17 (native audit session, code-truth verification pass)
Tier: MAX design review, then CODER implementation waves
Review: crypto-security-auditor MANDATORY = Fusion adversarial panel, unanimous (docs/ORCHESTRATION.md Section 10)

## OPERATOR APPROVAL (2026-07-17)

Operator approved wiring the production path through the session manager.
Decisions taken: (1) wire send + receive through the fallback wrappers;
(2) legacy fallback preserved for no-session/no-bundle peers (mixed fleet);
(3) kill switch = env var `SCM_RATCHET_DISABLE` (all-platform, zero plumbing) --
REMOVED 2026-08-24, see update banner at top of this file;
(4) E-00 first, then E-01 family lands in a live path.

## PRE-FLIGHT ANALYSIS FINDINGS (qwen THINK, tmp/e00_analysis_task_response.md -- pending Fusion unanimous judgement)

1. Send: `DriftEnvelope::from_legacy_envelope` reads Envelope FIELDS (non-opaque;
   encrypt.rs:172-235 region); NO V2 path exists. V1 arm -> from_legacy_envelope;
   V2 arm -> NEW conversion fn (DriftEnvelope lacks pq_kem/pq_encaps/transcript/suite
   fields -- must carry V2 as tagged opaque ciphertext bytes or extend the struct;
   decide in implementation packet, judge via Fusion).
2. Receive: decode order MUST be Drift-binary (first byte 0x01) -> bincode V1 ->
   tagged V2 (0x02). WIRE_TAG_V2=0x02 (codec.rs:220), DRIFT_VERSION=0x01
   (drift/envelope.rs:35), no collision. decode_wire_envelope alone CANNOT parse
   Drift-binary V1 -- falling back to legacy decode_envelope is mandatory.
3. Self bundle: `identity::sign_bundle(&keys)` (keys.rs:255-296). CACHE it --
   per-send Ed25519+ML-DSA signing is wasteful.
4. Peer bundle: `ContactStore::get_contact_bundle(recipient_id)` (contacts.rs:135-152);
   recipient_id IS hex(ed25519 public); identity_id()==hex(blake3(ed25519_public))
   matches receive-side derivation. Sessions auto-form on send (get_or_create) and
   on receive (create_receiver_session*) -- no separate handshake dispatch needed.
5. Locks: iron_core.rs:1315 takes identity.write BEFORE ratchet_sessions.read --
   keep that order at the new call sites (identity.read -> ratchet_sessions.write)
   to avoid inversion.

## NEXT COMMANDS (orchestrator, in order)

1. Fusion judgement of the analysis (Section 10):
   `python scripts/fusion_lite.py --prompt-file tmp/fusion-e00-analysis-judge.md --panel "qwen/qwen3-235b-a22b-2507,deepseek/deepseek-chat-v3.1,meta-llama/llama-3.3-70b-instruct" --judge "qwen/qwen3-235b-a22b-2507" --max-tokens 1000 --max-cost 0.05 --out tmp/fusion-e00-analysis-verdict.md`
   (judge prompt = analysis + "correct and sufficient to implement safely? unanimous PASS required")
2. If unanimous: write implementation packet (V2-Drift decision from finding 1)
   -> dispatch qwen CODER `--mode diff --apply --verify "cargo check --workspace"`.
3. cargo test -p scmessenger-core --test integration_pq_session and integration_e2e.
4. Fusion adversarial panel on the applied diff (unanimous) -> commit, move ticket.

## Finding

The entire Double Ratchet + PQ-hybrid subsystem, although implemented and
tested in isolation, is dead code with respect to real message traffic.

- `IronCore::prepare_message_internal` (core/src/iron_core.rs:636-668), reached
  by every CLI/FFI send path (`prepare_message_with_id`/`prepare_message`, called
  from cli/src/api_axum.rs:52, cli/src/api.rs:503, cli/src/main.rs:2040/2231/2853/2936),
  calls the bare legacy `crypto::encrypt_message` (core/src/crypto/encrypt.rs:123)
  directly.
- `IronCore::receive_message` (core/src/iron_core.rs:2714-2726), called from
  cli/src/main.rs:1866/2740, cli/src/ble_mesh.rs:75, and
  core/src/mobile_bridge.rs:786/1277 (Android/iOS), calls the bare legacy
  `crypto::decrypt_message` directly.
- The ratchet-aware paths `encrypt_with_ratchet_fallback` /
  `decrypt_with_ratchet_fallback` (encrypt.rs:503-566, 581-660) -- the only
  functions that consult `RatchetSessionManager` -- are never called outside
  encrypt.rs itself and core/tests/integration_pq_session.rs.

Consequence: every real message sent by the app today has ZERO forward secrecy
and ZERO PQ protection, regardless of the correctness of the ratchet code.

## Contradiction with prior audit

`HANDOFF/done/PQC_08_LEGACY_PATH_RETIREMENT.md` (2026-07-11, "verified")
asserts "No direct encrypt_message( calls exist anywhere in cli/, wasm/,
mobile/, or desktop_bridge/". That inventory missed core/src/iron_core.rs
itself; the bare calls at iron_core.rs:667 and :2723 date to commit 5ae934aa2
(2026-07-02), nine days before that audit.

## Relationship to open tickets

This supersedes/reframes the E-01 family scope:
- PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY (E-01): still real
  (ratchet.rs:580-588 hardcodes pq_ss=None in both branches), but fixing it
  changes nothing for users until THIS ticket lands.
- PQC_07_WIRE_RATCHET_STEP (E-04): its narrow claim (cadence call inside
  encrypt_message_ratcheted) is TRUE and landed 2026-07-11; the wiring gap is
  one level higher, at IronCore.
- PQC_07_FORCE_RATCHET_SAME_DEFECT (E-02): still real (ratchet.rs:676).

## Required operator decisions before dispatch

1. Wire `prepare_message_internal`/`receive_message` through the session
   manager (encrypt_with_ratchet_fallback / decrypt_with_ratchet_fallback)?
   This is the architecture-direction change the standing rules say must be
   operator-approved.
2. Migration/compat: fallback behavior for peers without established ratchet
   sessions (the _fallback functions already model this -- verify semantics).
3. Sequencing vs E-01b (PQ mixing design): recommend wiring first
   (E-00), then E-01c lands inside an actually-live path.

## Acceptance (once approved)

- prepare_message_internal and receive_message route through the ratchet
  session manager; bare legacy encrypt_message/decrypt_message calls remain
  only inside the fallback implementation.
- integration_e2e + integration_pq_session green; new integration test proves
  a CLI-to-CLI message round-trip advances the ratchet state.
- Adversarial review PASS (crypto-security-auditor tier).
