# TASK: PQC-04 — Suite advertisement and cryptographically authenticated downgrade

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Read `PQC_00_MASTER_PLAN.md` first. Depends on: PQC-02, PQC-03. Wave 1. Min tier: Sonnet.

## Why

A hybrid suite is worthless if an active attacker can strip the PQ offer and force both sides down to 0x01 silently (audit F3; Signal binds negotiation into the transcript for exactly this reason). This task adds: (a) advertising supported suites, (b) a deterministic negotiation rule, (c) transcript binding so a stripped offer breaks decryption, (d) a strict mode.

## Design (fixed)

1. **Advertisement.** `PublicKeyBundle` (PQC-03) gains `supported_suites: Vec<u8>` INSIDE the signed bytes (bump bundle format tag to 0x02; keep decoding tag 0x01 bundles, treating them as `supported_suites = [0x01]`). A peer that has a valid bundle with 0x02 listed is "v2-capable".
2. **Negotiation rule (deterministic, no round trip).** `negotiated_suite = max(intersection(our_suites, their_suites))`. Two v2 peers therefore ALWAYS land on 0x02. No intersection = refuse to send (error, not fallback).
3. **Transcript binding.** Define `transcript_hash = blake3::derive_key("iron-core suite-transcript v1", our_suites || 0xFF || their_suites || 0xFF || negotiated_suite || our_ed25519_pub || their_ed25519_pub)` computed by the INITIATOR with "our/their" in initiator/responder order. This 32-byte value:
   - goes into `EnvelopeV2.transcript_hash` on session-establishing envelopes, AND
   - is mixed into the session root KDF in PQC-06 (PQC-06 consumes it; you only need to compute, carry, and verify it here + store it on the session struct).
   The responder recomputes it from its own view of both bundles and REJECTS session establishment on mismatch. Result: an attacker who altered either advertised suite list produces a mismatched transcript and the session never opens.
4. **Strict mode.** Add `require_pq: bool` (default false) to `core/src/settings.rs`. When true: sending with negotiated suite < 0x02 returns an error surfaced to the caller; log an audit event either way (`rg -n "audit" core/src --type rust -l` to find the audit log manager).

## Steps

1. Bundle field + tag bump + compatibility decode (PQC-03 structures).
2. Negotiation function + transcript computation in a new `core/src/crypto/negotiation.rs` with exhaustive unit tests: empty intersection, singleton, unknown future suite ids (must be carried opaquely, not panic), determinism initiator/responder symmetry test.
3. Store negotiated suite + transcript on the ratchet session struct (field only; PQC-06 uses it).
4. Downgrade-detection integration test: craft a tampered bundle (0x02 removed, signature left stale) -> bundle verification fails; craft a re-signed-by-attacker bundle -> ed25519_public mismatch vs contact identity -> rejected; craft mismatched transcript -> session establishment rejected.

## Definition of Done

- [ ] Standard gates PASS.
- [ ] New negotiation tests + downgrade tests pass: `cargo test -p scmessenger-core negotiation`
- [ ] File moved to HANDOFF/done/ + committed.

## Do NOT

- Implement any KEM/ratchet logic here (PQC-05/06/07).
- Make `require_pq = true` the default (that flip is a human decision at rollout time).
