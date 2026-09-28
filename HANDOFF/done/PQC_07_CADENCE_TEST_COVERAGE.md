# TASK: Add test coverage for the PQ ratchet cadence (message-100 trigger)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO. Follow-up to `PQC_07_WIRE_RATCHET_STEP` (landed, committed) --
the wiring itself is real and existing tests still pass, but nothing
exercises the new cadence path: `encrypt_message_ratcheted` in
`core/src/crypto/encrypt.rs` now calls `session.perform_pq_ratchet_step()`
every 100 messages after `peer_confirmed`, and
`decrypt_message_ratcheted_v2` calls `session.handle_incoming_pq_fields()`
whenever `envelope.pq_kem_ciphertext` is present post-confirmation. No test
sends anywhere near 100 messages, so this path has zero coverage.

## What to add (in core/tests/integration_pq_session.rs)

A new test, e.g. `test_pq_ratchet_cadence_refreshes_shared_secret`:

1. Establish a confirmed hybrid (suite 0x02) session between two parties
   (mirror the existing `test_pq_session_full_send_receive` setup, but
   drive it to `peer_confirmed = true` on both sides first via one
   round-trip message exchange).
2. Send 100+ messages one-directionally (or bidirectionally, whichever is
   simpler given the existing session-manager API) so the cadence trigger
   (`current_message_number % 100 == 0`) actually fires at least once.
3. Assert the envelope at the triggering message number has
   `pq_kem_ciphertext.is_some()` and `pq_encaps_key.is_some()` (whereas
   envelopes at other message numbers in between should have both `None`).
4. Assert the receiver successfully decrypts the triggering message (proves
   `handle_incoming_pq_fields` + the subsequent `session.decrypt()` still
   work together).
5. Capture the session's root key (via whatever accessor exists, e.g.
   `root_key_bytes()`) immediately before and after the cadence message,
   and assert it changed -- proving the fresh PQ shared secret actually
   entered the KDF, not just that the ciphertext was transmitted.

## Do NOT

- Do not change the cadence constant (100) or any production logic in
  `encrypt.rs`/`ratchet.rs` -- test-only addition.
- Do not weaken or remove the 5 existing tests in this file.

## Gate

```
cargo test -p scmessenger-core --test integration_pq_session -j 2
```
All tests (existing 5 + new one) must pass.

## Output format (MANDATORY)

Return the FULL updated contents of `core/tests/integration_pq_session.rs`
in one fenced code block with `// core/tests/integration_pq_session.rs` as
the first line inside the block.
