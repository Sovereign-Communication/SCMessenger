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

1. **blake3_hash**  core/src/dspy/signatures.rs  Wire into DSPy signature verification path
2. **can_forward_for_wasm**  core/src/routing/  Wire into WASM forwarding decision
3. **can_reach_destination**  core/src/routing/  Wire into routing reachability check
4. **create_basic**  core/src/routing/  Wire into default route creation
5. **create_cot**  core/src/routing/  Wire into chain-of-thought route creation
6. **create_multihop**  core/src/routing/  Wire into multipath route builder
7. **create_optimizer**  core/src/routing/  Wire into routing optimization init
8. **evaluate_all_tracked**  core/src/routing/  Wire into routing evaluation loop
9. **isAtMaxDelay**  core/src/routing/  Wire into retry delay check
10. **list_endpoints**  core/src/routing/  Wire into endpoint enumeration
11. **mark_path_failed**  core/src/routing/  Wire into path failure handler
12. **mark_refresh_failed**  core/src/routing/  Wire into refresh failure path
13. **negative_cache_stats**  core/src/routing/  Wire into routing diagnostics
14. **next_refresh_hint**  core/src/routing/  Wire into refresh scheduler
15. **prune_below**  core/src/routing/  Wire into path pruning
16. **read_with_timeout**  core/src/routing/ or store  Wire into timeout read path
17. **refresh_delegate_routes**  core/src/routing/  Wire into delegate refresh
18. **register_endpoint**  core/src/routing/  Wire into endpoint registration
19. **register_path**  core/src/routing/  Wire into path registration
20. **resolveDeliveryState**  core/src/store/ or drift  Wire into message delivery resolution
21. **run_optimization**  core/src/routing/  Wire into optimization trigger
22. **send_message_status**  core/src/drift/  Wire into message status reporting
23. **should_advance**  core/src/routing/  Wire into ratchet advancement check
24. **start_refresh**  core/src/routing/  Wire into refresh initiation
25. **timeout_budget_summary**  core/src/routing/  Wire into TTL budget diagnostics
26. **touch_endpoint**  core/src/routing/  Wire into endpoint liveness update
27. **unregister_endpoint**  core/src/routing/  Wire into endpoint removal
28. **update_keepalive**  core/src/routing/  Wire into keepalive update path
29. **transport_type_to_routing_transport**  core/src/routing/  Wire into transport type mapping

## Tasks  Group B: Identity & Contact Wiring

30. **contact_new_has_no_last_known_device_id**  core/src/store/contacts  Wire into new contact creation validation
31. **contact_roundtrips_through_serde_with_default_device_id**  core/src/store/contacts  Wire into serde roundtrip test path
32. **federated_nickname**  core/src/identity/  Wire into federated identity display
33. **get_signable_data**  core/src/identity/  Wire into identity signing path
34. **get_signature**  core/src/identity/  Wire into signature retrieval
35. **update_last_known_device_id_can_clear**  core/src/store/contacts  Wire into device ID clearing
36. **update_last_known_device_id_ignores_invalid_values**  core/src/store/contacts  Wire into validation path
37. **update_last_known_device_id_persists_and_is_readable**  core/src/store/contacts  Wire into persistence path
38. **update_last_known_device_id_trims_valid_uuid**  core/src/store/contacts  Wire into UUID trimming
39. **annotate_identity**  core/src/identity/  Wire into identity annotation/display
40. **initialize_identity_from_daemon**  core/src/wasm_support/  Wire into WASM identity init

## Tasks  Group C: Crypto & Protocol Validation Wiring

41. **encrypt_xchacha20**  core/src/crypto/  Wire into message encryption path
42. **chain_ratchet_produces_distinct_keys**  core/src/crypto/  Wire into ratchet test harness
43. **derive_key_always_32_bytes**  core/src/crypto/  Wire into key derivation validation
44. **ed25519_conversion_produces_32_bytes**  core/src/crypto/  Wire into key conversion validation
45. **force_ratchet**  core/src/crypto/  Wire into ratchet force-advance path
46. **nonce_length_invariant**  core/src/crypto/  Wire into nonce validation
47. **ratchet_has_session**  core/src/crypto/  Wire into session check path
48. **ratchet_reset_session**  core/src/crypto/  Wire into session reset
49. **ratchet_session_count**  core/src/crypto/  Wire into session diagnostics
50. **decode_rejects_short_buffer**  core/src/drift/  Wire into drift decode validation
51. **proptest_different_ciphertexts_same_plaintext**  Wire into proptest harness
52. **proptest_encrypt_decrypt_roundtrip**  Wire into proptest harness
53. **proptest_envelope_field_lengths**  Wire into proptest harness
54. **proptest_ratchet_forward_secrecy**  Wire into proptest harness
55. **proptest_ratchet_roundtrip**  Wire into proptest harness
56. **proptest_sign_verify_roundtrip**  Wire into proptest harness
57. **proptest_wrong_key_fails**  Wire into proptest harness

## Tasks  Group D: Relay, Storage, Abuse & Privacy Wiring

58. **relay_discovery_mut**  core/src/relay/  Wire into relay discovery mutation
59. **relay_jitter_delay**  core/src/relay/  Wire into relay timing
60. **relay_request_carries_ws13_metadata_when_set**  core/src/relay/  Wire into WS1.3 relay request
61. **relay_request_missing_ws13_fields_deserialize_with_defaults**  core/src/relay/  Wire into relay deserialization
62. **peer_rate_limit_multiplier**  core/src/abuse/  Wire into rate limit calculation
63. **peer_spam_score**  core/src/abuse/  Wire into spam scoring
64. **cheap_heuristics_reject_invalid_payload_shapes**  core/src/abuse/  Wire into abuse detection
65. **checkAndRecordMessage**  core/src/abuse/  Wire into message check pipeline
66. **storage_pressure_emergency_mode_rejects_non_critical_and_recovers**  core/src/store/  Wire into storage pressure handler
67. **storage_pressure_purge_prioritizes_non_identity_then_identity**  core/src/store/  Wire into storage purge
68. **storage_pressure_purge_records_audit_transition_before_delete**  core/src/store/  Wire into audit trail
69. **storage_pressure_quota_bands_follow_locked_policy**  core/src/store/  Wire into quota enforcement
70. **storage_pressure_state_uses_synthetic_snapshot_when_probe_unavailable**  core/src/store/  Wire into storage probe fallback
71. **token_bucket_refills_after_elapsed_time**  core/src/abuse/  Wire into rate limiter
72. **validate_audit_chain**  core/src/store/  Wire into audit validation
73. **validate_settings**  core/src/store/  Wire into settings validation
74. **custody_audit_persists_across_restart**  core/src/relay/  Wire into custody persistence test
75. **custody_deduplicates_same_destination_and_message_id**  core/src/relay/  Wire into custody dedup
76. **custody_transitions_are_recorded**  core/src/relay/  Wire into custody state tracking

## Tasks  Group E: WASM/CLI Bridge Wiring

77. **get_history_via_api**  core/src/wasm_support/  Wire into JSON-RPC history endpoint
78. **get_identity_from_daemon**  core/src/wasm_support/  Wire into WASM identity bridge
79. **is_prefetch_complete**  core/src/wasm_support/  Wire into WASM prefetch check
80. **is_prefetch_in_progress**  core/src/wasm_support/  Wire into WASM prefetch status
81. **detect_browser**  core/src/wasm_support/  Wire into WASM browser detection
82. **get_browser_options**  core/src/wasm_support/  Wire into WASM browser config
83. **get_daemon_socket_url**  core/src/wasm_support/  Wire into WASM daemon connection
84. **set_daemon_socket_url**  core/src/wasm_support/  Wire into WASM daemon URL config
85. **jsonrpc_get_identity**  core/src/wasm_support/  Wire into JSON-RPC identity endpoint
86. **jsonrpc_send_message_roundtrip**  core/src/wasm_support/  Wire into JSON-RPC send
87. **drift_activate**  core/src/drift/  Wire into drift protocol activation
88. **drift_deactivate**  core/src/drift/  Wire into drift protocol deactivation
89. **drift_network_state**  core/src/drift/  Wire into drift network reporting
90. **drift_store_size**  core/src/drift/  Wire into drift store diagnostics
91. **get_iron_core_mode**  core/src/wasm_support/  Wire into mode query
92. **set_iron_core_mode**  core/src/wasm_support/  Wire into mode config
93. **get_swarm_bridge**  core/src/wasm_support/  Wire into swarm bridge access
94. **parse_response**  core/src/wasm_support/  Wire into JSON-RPC response parsing
95. **unknown_method_error**  core/src/wasm_support/  Wire into JSON-RPC error handling

## Execution Strategy

Work through groups A  B  C  D  E in order. After each group, run `cargo check --workspace` to verify compilation. Fix any errors before moving to the next group.

When all groups are complete and compilation passes, move ALL completed task files from `HANDOFF/todo/` to `HANDOFF/done/`.

# REPO_MAP Context for Task: BATCH_CORE_RUST_WIRING_C4
