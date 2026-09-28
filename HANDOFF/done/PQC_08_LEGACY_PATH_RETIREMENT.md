# TASK: PQC-08 — Retire legacy static-ECDH sending between v2 peers

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Read `PQC_00_MASTER_PLAN.md` first. Depends on: PQC-07. Wave 4. Min tier: Haiku.

## Why

Audit finding F1: `encrypt_message` (`core/src/crypto/encrypt.rs`) does static-ephemeral ECDH against the recipient's identity-derived key — the single worst HNDL surface (no forward secrecy classically, full retro-decrypt quantumly). Once PQC-06/07 exist, there is no reason two capable peers ever use it.

## Rules to implement

In `encrypt_with_ratchet_fallback` (and any other caller of `encrypt_message` — inventory with `rg -n "encrypt_message\(" core/src --type rust`):

1. Peer has verified v2 bundle (0x02 negotiable) -> hybrid ratchet session MUST be used or established; falling through to `encrypt_message` for such a peer is now an ERROR path, not a fallback.
2. Peer is v1-only -> classical DH-ratchet session is REQUIRED where possible; bare `encrypt_message` remains ONLY for the first message to a peer with no session and no ratchet capability (current behavior), and each such send logs an audit event `legacy_static_ecdh_send`.
3. `require_pq = true` (PQC-04): case 2 becomes an error.
4. ALL decrypt paths stay untouched and tested (old stored messages must decrypt forever).

## Steps

1. Implement gating exactly as above; keep the decision logic in ONE function with unit tests per branch (v2 peer / v1 peer with ratchet / v1 peer bare / strict mode).
2. Audit events: reuse the existing audit log manager (locate via `rg -n "audit" core/src/iron_core.rs`).
3. Update any integration tests that asserted bare-ECDH sends between capable peers (expect ratcheted/hybrid instead).
4. Grep the CLI and wasm crates for direct `encrypt_message` calls and route them through the fallback function if any exist (list findings here).

## Definition of Done

- [x] Standard gates PASS (verified 2026-07-11: full lib suite 1130/0,
      workspace compile gate green, CLI rebuild green).
- [x] `cargo test -p scmessenger-core --test integration_e2e --test integration_pq_session --features test-utils --test integration_ironcore_roundtrip -j 2`
      green (note: `integration_pq_session` requires `--features test-utils`
      as of the PQC_07_CADENCE_TEST_COVERAGE work - without it, cargo
      silently skips that target due to its `required-features` gate rather
      than erroring, so the flag must be explicit to actually exercise it).
- [x] Branch-coverage unit tests for the gating function green - already
      present, 8 tests in `core/src/crypto/encrypt.rs` covering v2-peer/
      v1-peer x with-session/no-session x require_pq true/false, plus the
      suite-negotiation edge cases (v2 peer bundle without suite 0x02).
- [x] Call-site inventory written into this file (below).
- [x] File moved to HANDOFF/done/.

## Call-site inventory (2026-07-11)

`rg -n "encrypt_message\(" core/src cli/src wasm/src mobile/src desktop_bridge/src --type rust`
found exactly ONE production call site outside `encrypt_message`'s own
definition and test modules: `core/src/crypto/encrypt.rs`'s
`encrypt_with_ratchet_fallback`, in the `false` branch of
`should_use_ratcheted_encryption`'s match - i.e. exactly the intentional,
gated legacy-fallback path this task specifies (rule #2: "bare
`encrypt_message` remains ONLY for the first message to a peer with no
session and no ratchet capability"). No direct `encrypt_message(` calls
exist anywhere in `cli/`, `wasm/`, `mobile/`, or `desktop_bridge/` - all
platform crates already route through `encrypt_with_ratchet_fallback`. No
call-site migration needed; the gating landed correctly on the first pass.

## Do NOT

- Delete `encrypt_message`/`decrypt_message` or the Ed25519->X25519 conversion helpers.
- Change default `require_pq` to true.
