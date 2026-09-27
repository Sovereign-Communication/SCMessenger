# TASK: PQC-05/06/07 Adversarial Review Checkpoint [AUDIT-GATE][BLOCKING]

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: TODO -- BLOCKS all PQC-09+ work (master-plan rule: auditor pass after
PQC-05 before waves 2+ stack up; no verdict exists in HANDOFF/review/).
Tier: [THINKING] read-only review. Zero-Anthropic route sanctioned by
.claude/rules/security.md precedent (P1-16 passed adversarial audit from
qwen3-235b-a22b-thinking).

## Dispatch (read-only -- NO --apply)

```bash
python scripts/delegate_task.py \
  --task HANDOFF/todo/PQC_REVIEW_CHECKPOINT_05_06_07.md \
  --provider qwen --tier thinking \
  --files core/src/crypto/pq/mod.rs core/src/crypto/session_manager.rs \
          core/src/crypto/ratchet.rs core/src/crypto/negotiation.rs \
          core/tests/integration_pq_session.rs
```

If the context overflows, split into two dispatches: (A) pq/mod.rs +
session_manager.rs + negotiation.rs, (B) ratchet.rs + encrypt.rs + tests.
The orchestrator then copies the response from tmp/ into
`HANDOFF/review/PQC_05_06_07_ADVERSARIAL_REVIEW.md`, sanity-checks findings
against source, and files follow-up tasks for anything HIGH/CRITICAL.

## Review instructions (for the reviewing model)

You are a hostile cryptographic security auditor. The code under review
implements a hybrid post-quantum upgrade to an existing X25519 +
XChaCha20-Poly1305 double-ratchet messenger:

- PQC-05: ML-KEM-768 primitives module (`crypto/pq/`, libcrux-ml-kem).
- PQC-06: hybrid session establishment -- X25519 ECDH combined with
  ML-KEM-768 encapsulation feeding the session root key (suite 0x02),
  with legacy suite 0x01 (X25519-only) still negotiable.
- PQC-07: PQ ratchet steps -- periodic re-encapsulation mixed into
  `root_key_ratchet_v2` alongside DH ratchet output.

Probe SPECIFICALLY for, with file:line evidence:
1. KEM misuse: encaps/decaps argument confusion, ciphertext/key length
   validation, implicit-rejection handling of malformed ciphertexts.
2. Secret hygiene: are ML-KEM shared secrets and private keys zeroized?
   Cloned into long-lived structs? Logged or Debug-printed anywhere?
3. Hybrid KDF: is the PQ shared secret ACTUALLY mixed into the root key
   derivation with domain separation, or can a code path silently drop it
   (e.g. `pq_ss = None` fallthrough) while claiming suite 0x02?
4. Downgrade: can an attacker force suite 0x01 on two 0x02-capable peers
   (negotiation not transcript-bound / not authenticated)?
5. Ratchet regressions: did the PQ additions break existing guarantees --
   out-of-order message keys, skipped-key storage bounds, replay windows?
6. Serialization: reconstruct()/persistence round-trip of the new PQ fields
   (session_manager.rs) -- can stale or attacker-controlled stored state
   desynchronize or duplicate key material?
7. Error paths and timing: distinguishable failures between decaps failure
   and MAC failure; any early returns leaking which layer rejected.
8. Test honesty: do the integration tests assert real security properties
   (both directions, tampered ciphertext rejected, downgrade rejected) or
   only happy paths? List missing test cases.

Output format: a markdown verdict with sections VERDICT (PASS /
PASS-WITH-FINDINGS / FAIL), FINDINGS (each: severity CRITICAL/HIGH/MED/LOW,
file:line, description, exploit scenario, suggested fix), and MISSING TESTS.
Do NOT return code files. No emoji.

## Acceptance

- Verdict file exists at HANDOFF/review/PQC_05_06_07_ADVERSARIAL_REVIEW.md.
- Every CRITICAL/HIGH finding has a follow-up task file in HANDOFF/todo/.
- _QUEUE.md updated; PQC-09+ unfreezes only on PASS or after fixes land.
