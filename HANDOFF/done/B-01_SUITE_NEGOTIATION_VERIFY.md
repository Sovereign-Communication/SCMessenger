# TASK: B-01 Suite Negotiation Verify

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: DISPATCH-READY (unblocked by E-01c)
Model: Qwen-plus (verification task)
Tier: CODER
Estimate: 150 LOC review + verification

## Objective

Verify current suite negotiation implementation state against PQC-04 specification. Ensure X25519+ML-KEM hybrid negotiation is correct and wired end-to-end.

## Current State

- PQC-04 (Suite Negotiation) was marked DONE 2026-07-10
- E-01c (receiver-anchored mixing) now DONE
- B-wave (PQC-09..14) is now UNBLOCKED

## Implementation

### 1. Read Suite Negotiation Spec
- Check `core/src/crypto/negotiation.rs` - suite selection logic
- Verify X25519 + ML-KEM-768 hybrid negotiation flows
- Check suite tag definitions (0x01=legacy, 0x02=hybrid)

### 2. Verify Against PQC-04 Definition of Done
- Suite negotiation advertisement in identity bundles
- Peer bundle verification + suite selection logic
- Fallback to X25519-only if peer doesn't support hybrid
- No suite mismatches that cause message decryption failure

### 3. Trace End-to-End
- send path: prepare_message_internal -> negotiation choice -> encrypt_message_ratcheted
- receive path: receive_message -> suite detection -> decrypt_message_ratcheted_v2
- Confirm both paths use negotiated suite, not hardcoded

### 4. Test Coverage
- Check existing tests in `core/tests/` for suite negotiation scenarios
- Verify tests cover: (1) both peers support hybrid, (2) peer X25519-only, (3) suite mismatch handling

### 5. Identify Any Gaps
- If suite negotiation is incomplete, list specific gaps and estimate LOC needed
- If complete and wired, confirm via cargo test pass

## Success Criteria

- [PASS] Suite negotiation implemented per PQC-04 spec
- [PASS] Both send and receive paths use negotiated suite
- [PASS] Fallback to X25519 works for legacy peers
- [PASS] All suite negotiation tests pass
- [PASS] No hardcoded suite assumptions (0x01/0x02 used everywhere)
- [PASS] `cargo test --workspace --no-run` green

## Output

Report findings:
1. Suite negotiation state: COMPLETE / INCOMPLETE / NEEDS CLARIFICATION
2. End-to-end wiring: VERIFIED / GAPS FOUND (list specific gaps)
3. Test coverage: ADEQUATE / NEEDS TESTS (recommend test scenarios)
4. Estimated follow-up work: 0 LOC (done) / N LOC (if gaps found)

If gaps found: estimate LOC and create follow-up task (B-01-IMPL).

Move this file to `HANDOFF/done/B-01_SUITE_NEGOTIATION_VERIFY.md` when done (execute mv command).

CRITICAL: You are forbidden from considering a task complete until you execute the mv or Rename-Item command to move this file from IN_PROGRESS/ to done/. If you do not move the file, the Orchestrator assumes you failed.
