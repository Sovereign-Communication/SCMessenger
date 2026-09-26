# BATCH: Core Rust + WASM + CLI Wiring (C4)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

You are a worker implementing wiring tasks. Each task requires you to:
1. Find the target function
2. Identify where it should be called
3. Wire it into the production call path
4. Verify compilation with `cargo check --workspace`
5. Move the task file from HANDOFF/todo/ to HANDOFF/done/

CRITICAL: You are forbidden from considering a task 'complete' until you execute the mv or Rename-Item command to move the task markdown file from todo/ (or IN_PROGRESS/) to done/. If you do not move the file, the Orchestrator assumes you failed.

## Build Verification
After wiring, run: `cargo check --workspace`

## Tasks  Group A: Core Infrastructure Wiring (routing, relay, transport)


## Sub-batch 4 of 7

1. **nonce_length_invariant**  core/src/crypto/  Wire into nonce validation
2. **ratchet_has_session**  core/src/crypto/  Wire into session check path
3. **ratchet_reset_session**  core/src/crypto/  Wire into session reset
4. **ratchet_session_count**  core/src/crypto/  Wire into session diagnostics
5. **decode_rejects_short_buffer**  core/src/drift/  Wire into drift decode validation
6. **proptest_different_ciphertexts_same_plaintext**  Wire into proptest harness
7. **proptest_encrypt_decrypt_roundtrip**  Wire into proptest harness
8. **proptest_envelope_field_lengths**  Wire into proptest harness
9. **proptest_ratchet_forward_secrecy**  Wire into proptest harness
10. **proptest_ratchet_roundtrip**  Wire into proptest harness
11. **proptest_sign_verify_roundtrip**  Wire into proptest harness
12. **proptest_wrong_key_fails**  Wire into proptest harness

## Tasks  Group D: Relay, Storage, Abuse & Privacy Wiring

13. **relay_discovery_mut**  core/src/relay/  Wire into relay discovery mutation
14. **relay_jitter_delay**  core/src/relay/  Wire into relay timing
15. **relay_request_carries_ws13_metadata_when_set**  core/src/relay/  Wire into WS1.3 relay request