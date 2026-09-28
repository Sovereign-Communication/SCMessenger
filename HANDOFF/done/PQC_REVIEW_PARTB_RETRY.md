# TASK: Adversarial security review of PQ ratchet + encrypt fallback (retry)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

CRITICAL INSTRUCTION, read this first: you are reviewing code, NOT writing
or reproducing it. If your response contains the source code of any file
listed below (a Rust `fn`, `struct`, `impl` block, etc. copied or
reconstructed from the files), you have FAILED this task regardless of
anything else in your response. Your entire output must be prose analysis
in the verdict format at the bottom -- no code blocks of any kind.

You are a hostile cryptographic security auditor reviewing a hybrid
post-quantum upgrade (X25519 + ML-KEM-768) to an existing double-ratchet
messenger (SCMessenger). The files provided below are
`core/src/crypto/ratchet.rs`, `core/src/crypto/encrypt.rs`, and
`core/tests/integration_pq_session.rs`. Read them carefully, then answer
the 8 questions below using file:line citations from the ACTUAL content
provided -- do not rely on any prior/training knowledge of what similar
code "usually" looks like.

Probe SPECIFICALLY for, with file:line evidence:
1. Is the ML-KEM shared secret (`pq_ss` or equivalent) actually threaded
   into root_key_ratchet_v2 on EVERY DH ratchet step for suite 0x02
   sessions, or can it silently be dropped/None on some code path while the
   session still claims suite 0x02? Trace `handle_dh_ratchet`,
   `perform_pq_ratchet_step`, `handle_incoming_pq_fields` precisely.
2. KEM misuse: encaps/decaps argument order, ciphertext length validation
   (ML-KEM-768 ciphertext should be 1088 bytes), implicit-rejection handling.
3. Secret hygiene: are ML-KEM shared secrets and private keys zeroized on
   drop? Cloned into long-lived structs unnecessarily? Logged anywhere?
4. Downgrade: can two suite-0x02-capable peers be forced to suite 0x01
   (is negotiation transcript-bound and authenticated)?
5. Ratchet regressions: out-of-order message keys, skipped-key storage
   bounds (MAX_SKIP_KEYS), replay windows -- did PQ additions break these?
6. Serialization/reconstruct() round-trip: can stale or attacker-controlled
   stored state desync sessions?
7. Error-path/timing: distinguishable failures between decaps failure and
   MAC failure; early returns leaking which layer rejected.
8. Test honesty: do the tests in integration_pq_session.rs assert real
   security properties (both directions, tampered ciphertext rejected,
   downgrade rejected, PQ shared secret actually affects derived keys) or
   only happy paths? List missing test cases.

Output format (verdict only, no code):
VERDICT: PASS / PASS-WITH-FINDINGS / FAIL
FINDINGS: each with severity CRITICAL/HIGH/MED/LOW, file:line, description,
exploit scenario, suggested fix (fix described in words, NOT as a code diff).
MISSING TESTS: bullet list.
