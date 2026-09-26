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


## Sub-batch 5 of 7

1. **relay_request_missing_ws13_fields_deserialize_with_defaults**  core/src/relay/  Wire into relay deserialization
2. **peer_rate_limit_multiplier**  core/src/abuse/  Wire into rate limit calculation
3. **peer_spam_score**  core/src/abuse/  Wire into spam scoring
4. **cheap_heuristics_reject_invalid_payload_shapes**  core/src/abuse/  Wire into abuse detection
5. **checkAndRecordMessage**  core/src/abuse/  Wire into message check pipeline
6. **storage_pressure_emergency_mode_rejects_non_critical_and_recovers**  core/src/store/  Wire into storage pressure handler
7. **storage_pressure_purge_prioritizes_non_identity_then_identity**  core/src/store/  Wire into storage purge
8. **storage_pressure_purge_records_audit_transition_before_delete**  core/src/store/  Wire into audit trail
9. **storage_pressure_quota_bands_follow_locked_policy**  core/src/store/  Wire into quota enforcement
10. **storage_pressure_state_uses_synthetic_snapshot_when_probe_unavailable**  core/src/store/  Wire into storage probe fallback
11. **token_bucket_refills_after_elapsed_time**  core/src/abuse/  Wire into rate limiter
12. **validate_audit_chain**  core/src/store/  Wire into audit validation
13. **validate_settings**  core/src/store/  Wire into settings validation
14. **custody_audit_persists_across_restart**  core/src/relay/  Wire into custody persistence test
15. **custody_deduplicates_same_destination_and_message_id**  core/src/relay/  Wire into custody dedup