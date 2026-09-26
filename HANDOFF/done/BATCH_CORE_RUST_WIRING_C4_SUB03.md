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


## Sub-batch 3 of 7

1. **contact_roundtrips_through_serde_with_default_device_id**  core/src/store/contacts  Wire into serde roundtrip test path
2. **federated_nickname**  core/src/identity/  Wire into federated identity display
3. **get_signable_data**  core/src/identity/  Wire into identity signing path
4. **get_signature**  core/src/identity/  Wire into signature retrieval
5. **update_last_known_device_id_can_clear**  core/src/store/contacts  Wire into device ID clearing
6. **update_last_known_device_id_ignores_invalid_values**  core/src/store/contacts  Wire into validation path
7. **update_last_known_device_id_persists_and_is_readable**  core/src/store/contacts  Wire into persistence path
8. **update_last_known_device_id_trims_valid_uuid**  core/src/store/contacts  Wire into UUID trimming
9. **annotate_identity**  core/src/identity/  Wire into identity annotation/display
10. **initialize_identity_from_daemon**  core/src/wasm_support/  Wire into WASM identity init

## Tasks  Group C: Crypto & Protocol Validation Wiring

11. **encrypt_xchacha20**  core/src/crypto/  Wire into message encryption path
12. **chain_ratchet_produces_distinct_keys**  core/src/crypto/  Wire into ratchet test harness
13. **derive_key_always_32_bytes**  core/src/crypto/  Wire into key derivation validation
14. **ed25519_conversion_produces_32_bytes**  core/src/crypto/  Wire into key conversion validation
15. **force_ratchet**  core/src/crypto/  Wire into ratchet force-advance path