# TASK: PQC-07 — Post-quantum ratchet steps (hybrid forward secrecy + post-compromise security)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Read `PQC_00_MASTER_PLAN.md` first. Depends on: PQC-06. Wave 3. Min tier: Sonnet ONLY — do not assign to Haiku. ADVERSARIAL REVIEW MANDATORY + release-gatekeeper before merge. This is the highest-risk task in the workstream.

## Why

After PQC-06, only the session ROOT is hybrid; ongoing forward secrecy still comes solely from X25519 DH ratchet steps (audit F2: a recorded transcript falls to Shor per-step). This task injects fresh ML-KEM secrets into the root chain during the conversation, so compromise windows are bounded by BOTH assumptions. Model: Signal's Triple Ratchet/SPQR, deliberately simplified (see "Deviation" below).

## Design (fixed)

State added to `RatchetSession` (suite 0x02 only):

```
pq_our_keypair:      MlKem768KeyPair          // rotates every PQ step we initiate
pq_their_encaps_key: Option<Vec<u8>>          // latest advertised by peer (starts: their bundle key)
pq_pending_ct:       Option<Vec<u8>>          // ct we must keep sending until peer acks (lossy mesh)
```

Piggyback on the EXISTING DH ratchet step (the point in `ratchet.rs` where a received new `ratchet_dh_public` triggers `root_key` advance — locate with `rg -n "dh_ratchet|root" core/src/crypto/ratchet.rs`):

- **When we perform a sending DH step:** also (a) `(ct, ss_pq) = encapsulate(pq_their_encaps_key)` [plain ML-KEM from PQC-01 — the classical half of this step is the DH output itself], (b) generate fresh `pq_our_keypair`, (c) root update becomes `root' = derive_key(ROOT_KDF_CONTEXT_V2, root || dh_output || ss_pq)` instead of `root || dh_output`, (d) outgoing envelopes carry `pq_kem_ciphertext = ct` and `pq_encaps_key = new encaps key` until acked.
- **When we receive a DH step envelope carrying pq fields:** decapsulate with OUR current `pq_our_keypair` — CAREFUL: the ct was made for the keypair we advertised LAST step; retain exactly one previous keypair (`pq_prev_keypair`) to absorb one-step-behind arrivals, mirroring how the DH ratchet tolerates one outstanding step; update `pq_their_encaps_key` from the envelope; apply the same root formula.
- **Ack semantics:** receiving any envelope from the peer whose root reflects our last PQ step (i.e., it decrypts under the new chains) clears `pq_pending_ct`.
- If a suite-0x02 envelope performs a DH step WITHOUT pq fields while `pq_their_encaps_key` is Some: reject the step (protocol violation — a PQ-stripping attempt), fail decryption, log audit event.
- New context string `ROOT_KDF_CONTEXT_V2 = "iron-core root-chain v2 hybrid 2026-07"`. Suite 0x01 sessions keep the v1 context and old formula untouched.

## Deviation from SPQR (documented, approved in audit)

SPQR chunk/erasure-codes encapsulation keys across many messages to keep per-message overhead tiny. We instead attach full PQ material (1184+1088 B) ONLY on DH-step envelopes (a small fraction of traffic). Acceptable because drift framing already fragments large payloads for BLE (verify: `rg -n "fragment|mtu|chunk" core/src/drift --type rust -i`). If review finds BLE-path envelopes with PQ fields failing to deliver, STOP and escalate — do not invent chunking ad hoc.

## Tests (all required)

1. Extend `integration_pq_session.rs`: long interleaved conversation (200+ messages, alternating direction every 1-30 messages, forcing many DH+PQ steps) — all decrypt.
2. Out-of-order and dropped envelopes across a PQ step boundary (skipped-key logic must still work; one-step-behind ct absorbed by `pq_prev_keypair`).
3. PQ-strip rejection test: hand-craft a DH-step envelope without pq fields on a 0x02 session -> decryption fails + audit event.
4. Transcript decryption resistance sanity: serialize a whole session transcript; assert plaintext recovery requires the session state (i.e., a fresh session from the same identity bundles CANNOT decrypt mid-conversation envelopes — this pins forward secrecy behavior).
5. Persistence: save/load mid-conversation across a pending (un-acked) PQ step; conversation continues.
6. Proptest: random step/drop/reorder schedules never panic and never desync (both sides end able to exchange).

## Definition of Done

- [ ] Standard gates PASS.
- [ ] All tests above green; `integration_e2e`, `integration_retry_lifecycle`, `integration_offline_partition_matrix` still green (ratchet interacts with retry/offline paths).
- [ ] Adversarial review verdict + release-gatekeeper verdict recorded here.
- [ ] File moved to HANDOFF/done/ + committed.

## Do NOT

- Change v1 (suite 0x01) ratchet behavior in any observable way.
- Batch this task with PQC-08 in one commit — land and review separately.
- Deviate from the root formula or contexts without human sign-off (PQC-00 rule 8).


# REPO_MAP Context for Task: PQC_07_PQ_RATCHET

**Target function: `PQC_07_PQ_RATCHET`**

## core/src/routing/adaptive_ttl.rs (1 chunks, 250 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/adaptive_ttl.rs: Defines 6 types: ActivityHistory, Default, ActivityHistory, AdaptiveTTLManager, AdaptiveTTLManager; 21 functions; 3 imports

### Structs/Classes
- ActivityHistory
- AdaptiveTTLManager
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 28 | new, from_secs, calculate_ttl, now |
| `new` | 35 | now, from_secs, calculate_ttl |
| `record_message` | 44 | now, new, calculate_ttl |
| `calculate_ttl` | 50 | new, from_secs, calculate_ttl |
| `decay` | 64 | new, from_secs, calculate_ttl |
| `new` | 87 | new, from_secs |
| `with_defaults` | 97 | new, from_secs |
| `calculate_ttl` | 106 |  |
| `record_activity` | 118 | with_defaults |
| `get_activity` | 125 | from_secs, with_defaults |
| `cleanup` | 130 | from_secs, with_defaults |
| `len` | 138 | from_secs, with_defaults |
| `is_empty` | 143 | from_secs, with_defaults |
| `calculate_dynamic_ttl` | 150 | from_secs, with_defaults |
| `default` | 163 | from_secs, with_defaults |
| `test_adaptive_ttl_creation` | 173 | from_secs, from_millis, new, with_defaults |
| `test_inactive_peer_ttl` | 180 | sleep, with_defaults, new, from_millis, from_secs |
| `test_active_peer_ttl` | 187 | sleep, with_defaults, new, from_millis, from_secs |
| `test_moderate_peer_ttl` | 200 | sleep, with_defaults, new, from_nanos, from_millis, from_secs |
| `test_activity_decay` | 213 | sleep, with_defaults, new, from_nanos, from_millis, from_secs |
| `test_cleanup_old_entries` | 236 | from_nanos, with_defaults |

### Imports
- `use std::collections::HashMap`
- `use super::*`
- `use web_time::{Duration, Instant}`
---

## core/src/abuse/auto_block.rs (1 chunks, 336 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/abuse/auto_block.rs: Defines 7 types: AutoBlockConfig, Default, AutoBlockReason, AutoBlockAuditEntry, AutoBlockResult; 19 functions; 10 imports

### Structs/Classes
- AutoBlockAuditEntry
- AutoBlockConfig
- AutoBlockEngine
- AutoBlockReason
- AutoBlockResult
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 34 | new |
| `new` | 96 | new |
| `evaluate` | 111 |  |
| `evaluate_and_block` | 163 | new, evaluate |
| `exempt_peer` | 197 | evaluate_and_block |
| `unexempt_peer` | 202 | now, evaluate_and_block |
| `is_exempt` | 207 | now, evaluate_and_block |
| `audit_log` | 216 | now, evaluate_and_block, new, default |
| `update_config` | 221 | now, evaluate_and_block, new, default |
| `config` | 226 | now, evaluate_and_block, new, default |
| `evaluate_all_tracked` | 232 | now, evaluate_and_block, new, default |
| `current_epoch_secs` | 245 | now, default, new |
| `make_engine` | 260 | new, default |
| `test_default_config` | 275 | default |
| `test_exempt_peer_not_blocked` | 283 | default |
| `test_unexempt_peer` | 292 | default |
| `test_audit_log_records_block` | 300 | default |
| `test_disabled_auto_block` | 318 | default |
| `test_neutral_peer_not_blocked` | 330 |  |

### Imports
- `use crate::abuse::reputation::EnhancedAbuseReputationManager`
- `use crate::abuse::spam_detection::{SpamDetectionConfig, SpamDetectionEngine}`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::blocked::BlockedManager`
- `use crate::store::contacts::ContactManager`
- `use parking_lot::RwLock`
- `use serde::{Deserialize, Serialize}`
- `use std::sync::Arc`
- `use std::time::SystemTime`
- `use super::*`
---

## core/src/crypto/backup.rs (1 chunks, 494 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/crypto/backup.rs: 22 functions; 13 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `derive_key_argon2id` | 66 | derive_key, with_capacity, new, new_from_slice, from_slice |
| `derive_key_blake3` | 88 | derive_key, with_capacity, new_from_slice, from_slice |
| `derive_key_pbkdf2` | 100 | new_from_slice, from_slice |
| `try_decrypt` | 109 | new_from_slice, from_slice |
| `encrypt_backup` | 130 |  |
| `encrypt_backup_fast` | 153 | new_from_slice, with_capacity, from_slice |
| `encrypt_backup_inner` | 163 | new_from_slice, encode, with_capacity, from_slice |
| `decrypt_backup` | 226 | decode, from_utf8 |
| `test_encrypt_decrypt_roundtrip` | 300 | encode |
| `test_custom_salt_encrypt_decrypt` | 311 | encode |
| `test_decrypt_wrong_passphrase_fails` | 323 | decode, encode |
| `test_decrypt_invalid_hex_fails` | 334 | decode, encode |
| `test_decrypt_truncated_data_fails` | 340 | decode, encode |
| `test_different_passphrases_produce_different_ciphertexts` | 348 | decode |
| `test_user_exports_use_argon2id_format_tag` | 360 | decode |
| `test_fast_backups_use_blake3_format_tag` | 368 | decode, encode |
| `test_fast_encrypt_decrypt_roundtrip` | 376 | encode |
| `test_fast_custom_salt_roundtrip` | 388 | new_from_slice, encode, from_slice |
| `test_kdf_is_memory_hard` | 409 | new, new_from_slice, encode, from_slice |
| `test_legacy_pbkdf2_with_salt_format_still_decrypts` | 423 | new, new_from_slice, encode, from_slice, hash |
| `test_oldest_blake3_salt_format_still_decrypts` | 453 | new, decode, new_from_slice, encode, from_slice, hash |
| `test_tampered_blob_fails_with_corruption_detected` | 480 | decode, encode |

### Imports
- `use argon2::{Algorithm, Argon2, Params, Version}`
- `use crate::IronCoreError`
- `use pbkdf2::pbkdf2_hmac`
- `use rand::RngCore`
- `use rand::rngs::OsRng`
- `use sha2::Sha256`
- `use super::*`
- `use zeroize::Zeroize`
---

## core/src/blocked_bridge.rs (1 chunks, 161 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/blocked_bridge.rs: Defines 6 types: BlockedIdentity, From, From, BlockedManager, Default; 17 functions; 2 imports

### Structs/Classes
- BlockedIdentity
- BlockedManager
- Default
- From

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `from` | 25 | new |
| `from` | 38 | new |
| `default` | 56 | new, from |
| `new` | 62 | new, from |
| `block` | 71 | from |
| `unblock` | 76 | from |
| `is_blocked` | 81 | from |
| `get` | 90 | new, from |
| `list` | 102 | new |
| `count` | 108 | new |
| `register_device_id` | 114 | new |
| `get_known_devices` | 123 | new |
| `is_device_blocked` | 128 | new |
| `blocked_identity_new` | 138 | new |
| `blocked_identity_with_device_id` | 141 |  |
| `blocked_identity_with_reason` | 150 |  |
| `blocked_identity_with_notes` | 156 |  |

### Imports
- `use crate::IronCoreError`
- `use std::sync::Arc`
---

## core/src/relay/bootstrap.rs (1 chunks, 470 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/bootstrap.rs: Defines 8 types: BootstrapError, BootstrapMethod, SeedPeer, SeedPeer, InvitePayload; 39 functions; 4 imports

### Structs/Classes
- BootstrapError
- BootstrapManager
- BootstrapMethod
- InvitePayload
- SeedPeer

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 50 | now |
| `new` | 78 | to_string, now, SerializationError, deserialize, serialize |
| `with_group_key` | 95 | to_string, now, SerializationError, deserialize, from_str, serialize |
| `with_expiry` | 101 | to_string, now, SerializationError, deserialize, from_str, serialize |
| `is_valid` | 107 | to_string, now, new, SerializationError, deserialize, from_str, serialize |
| `to_bytes` | 117 | to_string, new, SerializationError, deserialize, from_str, serialize |
| `from_bytes` | 122 | to_string, new, SerializationError, deserialize, from_str |
| `to_json` | 127 | from_str, new, to_string, SerializationError |
| `from_json` | 132 | from_str, new, SerializationError |
| `new` | 151 | new, generate_invite |
| `with_seed_peers` | 161 | new, from_bytes, generate_invite, accept_invite |
| `get_seed_peers` | 167 | new, from_bytes, generate_invite, accept_invite |
| `generate_invite` | 175 | new, from_bytes, generate_invite, accept_invite |
| `generate_qr_data` | 184 | from_bytes, generate_invite, accept_invite |
| `accept_invite` | 190 | from_bytes, new, accept_invite |
| `parse_qr_data` | 203 | from_bytes, new, accept_invite |
| `get_peer_addresses` | 209 | new |
| `set_addresses` | 220 | new |
| `get_addresses` | 225 | new |
| `test_seed_peer` | 237 | new |
| `test_bootstrap_manager` | 245 | new |
| `test_seed_peer_creation` | 255 | new, from_bytes |
| `test_invite_payload_creation` | 262 | new, from_json, from_bytes |
| `test_invite_payload_with_group_key` | 275 | from_bytes, sleep, new, from_millis, from_json |
| `test_invite_payload_serialization` | 288 | from_bytes, sleep, new, from_millis, from_json |
| `test_invite_payload_json_serialization` | 303 | new, from_json, from_millis, sleep |
| `test_invite_payload_expiry` | 317 | new, sleep, from_millis |
| `test_bootstrap_manager_creation` | 330 | new |
| `test_bootstrap_manager_with_seed_peers` | 337 | new |
| `test_get_seed_peers` | 350 | from_bytes |
| `test_get_seed_peers_empty` | 360 | from_bytes, new |
| `test_generate_invite` | 368 | from_bytes, new |
| `test_generate_qr_data` | 380 | from_bytes, sleep, from_millis, new |
| `test_parse_qr_data` | 394 | new, sleep, from_millis |
| `test_accept_invite_valid` | 407 | new, sleep, from_millis |
| `test_accept_invite_expired` | 420 | new, sleep, from_millis |
| `test_set_addresses` | 436 | new |
| `test_get_peer_addresses` | 446 | new |
| `test_bootstrap_method_enum` | 459 |  |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/transport/bootstrap.rs (1 chunks, 646 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/bootstrap.rs: Defines 6 types: BootstrapConfig, Default, BootstrapState, BootstrapNode, BootstrapManager; 37 functions; 10 imports

### Structs/Classes
- BootstrapConfig
- BootstrapManager
- BootstrapNode
- BootstrapState
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 56 | from_secs, default |
| `new` | 114 | new, default |
| `with_defaults` | 152 | new, default |
| `state` | 157 |  |
| `connected_count` | 162 |  |
| `total_nodes` | 167 | next_connectable_node |
| `relay_discovery` | 172 | next_connectable_node, record_failure |
| `relay_discovery_mut` | 177 | next_connectable_node, backoff_for_node, record_failure |
| `add_bootstrap_node` | 182 | next_connectable_node, backoff_for_node, record_failure |
| `bootstrap` | 205 | sleep, backoff_for_node, random, next_connectable_node, record_success, record_attempt, record_failure |
| `record_success` | 337 | now, from_secs_f64 |
| `record_failure` | 346 | now, new, from_secs_f64 |
| `record_attempt` | 353 | now, new, from_secs_f64 |
| `backoff_for_node` | 362 | new, from_secs_f64 |
| `next_connectable_node` | 376 | new |
| `discover_fallback_nodes` | 388 | Wss, new, Ws |
| `is_websocket_address` | 428 | Wss, Ws, from_multiaddr |
| `try_websocket_connection` | 439 | from_multiaddr |
| `circuit_breaker` | 465 | new, var |
| `get_healthy_relays` | 472 | new, var |
| `get_all_relay_stats` | 478 | new, var |
| `get_fallback_relay_addresses` | 488 | new, var |
| `reset_circuit_breakers` | 493 | new, var |
| `resolve_env_bootstrap_nodes` | 499 | new, var |
| `discover_dns_bootstrap` | 513 | new, from_secs, default, with_defaults |
| `discover_local_peers` | 520 | new, from_secs, default, with_defaults |
| `discover_websocket_bootstrap` | 534 | new, from_secs, default, with_defaults |
| `discover_hardcoded_backup_relays` | 542 | new, from_secs, default, with_defaults |
| `test_bootstrap_config_defaults` | 551 | with_defaults, new, now, default, from_secs |
| `test_bootstrap_manager_creation` | 562 | with_defaults, new, now, default, from_secs |
| `test_bootstrap_manager_add_node` | 570 | with_defaults, new, now, default, from_secs |
| `test_bootstrap_manager_no_duplicate` | 579 | with_defaults, new, now, default, from_secs |
| `test_exponential_backoff` | 589 | new, from_secs, default, now |
| `test_env_bootstrap_override` | 608 |  |
| `test_dns_discovery` | 617 |  |
| `test_local_discovery` | 626 |  |
| `test_websocket_discovery` | 633 |  |

### Imports
- `use crate::transport::circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager}`
- `use crate::transport::internet::{InternetRelay, InternetTransportError}`
- `use crate::transport::relay_health::{RelayDiscovery, RelayFallback, RelayMetrics}`
- `use crate::transport::swarm::SwarmHandle`
- `use crate::transport::websocket::{diagnose_websocket_error, WebSocketTransport}`
- `use libp2p::{Multiaddr, PeerId}`
- `use std::collections::VecDeque`
- `use super::*`
- `use tracing::{debug, info, warn}`
- `use web_time::{Duration, SystemTime}`
---

## core/src/privacy/circuit.rs (1 chunks, 529 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/privacy/circuit.rs: Defines 11 types: CircuitId, CircuitId, CircuitError, PeerInfo, CircuitPath; 29 functions; 8 imports

### Structs/Classes
- CircuitBuilder
- CircuitConfig
- CircuitError
- CircuitId
- CircuitPath
- Default
- PeerInfo

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `random` | 16 | new, thread_rng, from_le_bytes |
| `public_keys` | 63 | new |
| `hop_count` | 88 |  |
| `default` | 107 | select_hop_count |
| `validate` | 119 | select_random_hops, select_hop_count, select_diverse_hops, random |
| `new` | 143 | select_random_hops, thread_rng, random, select_diverse_hops, select_hop_count |
| `build_circuit` | 149 | select_random_hops, thread_rng, random, new, select_diverse_hops, select_hop_count |
| `select_hop_count` | 172 | new, thread_rng |
| `select_diverse_hops` | 179 | new |
| `select_random_hops` | 244 |  |
| `peers` | 275 | random, serialize, deserialize |
| `update_peer_reliability` | 280 | default, random, serialize, deserialize |
| `create_test_peers` | 299 | default, random, serialize, deserialize |
| `test_circuit_id_random` | 312 | default, random, serialize, deserialize |
| `test_circuit_id_serialization` | 319 | default, random, serialize, deserialize |
| `test_circuit_config_default` | 327 | random, default |
| `test_circuit_config_invalid_min_hops` | 335 | random |
| `test_circuit_config_invalid_order` | 346 | random |
| `test_circuit_path_public_keys` | 357 | random, default, new |
| `test_circuit_path_public_keys_invalid_length` | 374 | random, default, new |
| `test_circuit_path_hop_count` | 388 | random, default, new |
| `test_circuit_builder_new` | 402 | new, default |
| `test_circuit_builder_invalid_config` | 410 | new, default |
| `test_circuit_builder_insufficient_peers` | 423 | new, default |
| `test_circuit_builder_build_circuit` | 445 | new, default |
| `test_circuit_builder_diverse_paths` | 464 | new, default |
| `test_circuit_builder_update_reliability` | 493 | new, random, deserialize, default, serialize |
| `test_circuit_builder_update_reliability_nonexistent` | 503 | new, random, deserialize, default, serialize |
| `test_circuit_path_serialization` | 513 | random, serialize, deserialize |

### Imports
- `use rand::Rng`
- `use rand::RngCore`
- `use rand::seq::SliceRandom`
- `use rand::thread_rng`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use super::*`
- `use thiserror::Error`
---

## core/src/relay/client.rs (1 chunks, 1042 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/client.rs: Defines 9 types: TransportType, RelayClientConfig, Default, ConnectionState, RelayConnection; 41 functions; 15 imports

### Structs/Classes
- ConnectionState
- Default
- RelayClient
- RelayClientConfig
- RelayClientError
- RelayConnection
- TransportType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 56 | new, from_secs |
| `new` | 102 | now |
| `with_transport` | 114 | now |
| `set_state` | 126 | now |
| `is_connected` | 139 | new, full_relay |
| `new` | 184 | connect_tcp, new, connect_quic, full_relay |
| `set_capabilities` | 198 | connect_quic, connect_tcp, connect_websocket |
| `create_handshake` | 203 | connect_quic, connect_tcp, connect_websocket |
| `connect` | 214 | connect_quic, connect_tcp, connect_websocket |
| `connect_websocket` | 263 | ConnectionFailed, IoTimeout, with_transport, connect_async, into_client_request, create_handshake |
| `connect_websocket` | 362 | ConnectionFailed, IoTimeout, with_transport, new, complete_handshake, clone, send_and_receive_raw, create_handshake, connect |
| `connect_tcp` | 374 | ConnectionFailed, IoTimeout, with_transport, new, complete_handshake, clone, send_and_receive_raw, create_handshake, connect |
| `connect_quic` | 406 | ConnectionFailed, IoTimeout, with_transport, new, client, try_with_platform_verifier |
| `complete_handshake` | 512 | ConnectionFailed, HandshakeFailed |
| `connect_quic` | 544 | ConnectionFailed, MessageError, send_and_receive_raw |
| `push_envelopes` | 554 | ConnectionFailed, MessageError, send_and_receive_raw |
| `pull_envelopes` | 597 | new, ConnectionFailed, MessageError, send_and_receive_raw |
| `active_connections` | 648 | MessageError |
| `add_connection` | 658 | min, from_millis, MessageError |
| `remove_connection` | 663 | min, from_millis, MessageError |
| `send_ping` | 670 | min, from_millis, MessageError, SerializationError |
| `get_relay_addresses` | 697 | min, ConnectionFailed, MessageError, IoTimeout, SerializationError, from_millis |
| `backoff_duration` | 702 | min, from_bytes, ConnectionFailed, MessageError, IoTimeout, SerializationError, from_millis |
| `send_and_receive_raw` | 707 | from_bytes, ConnectionFailed, MessageError, IoTimeout, SerializationError |
| `test_client` | 760 | from_bytes, mobile, new, default, bind, spawn |
| `test_relay_client_creation` | 766 | from_bytes, mobile, full_relay, bind, spawn |
| `test_create_handshake` | 773 | from_bytes, mobile, full_relay, bind, spawn |
| `test_set_capabilities` | 792 | from_bytes, mobile, new, full_relay, bind, spawn |
| `test_connect_to_relay` | 801 | from_bytes, new, full_relay, bind, spawn |
| `test_relay_connection_creation` | 832 | new, full_relay |
| `test_relay_connection_state_transitions` | 840 | new, full_relay |
| `test_complete_handshake_success` | 858 | new, full_relay |
| `test_complete_handshake_version_mismatch` | 875 | new, full_relay |
| `test_push_envelopes_not_connected` | 890 | new |
| `test_pull_envelopes_not_connected` | 902 | new, spawn, bind |
| `test_active_connections` | 914 | from_bytes, new, full_relay, bind, spawn |
| `test_remove_connection` | 931 | from_bytes, new, full_relay, bind, spawn |
| `test_push_pull_and_ping_over_network` | 945 | bind, from_bytes, spawn, full_relay |
| `test_send_ping_not_connected` | 999 | new, default |
| `test_backoff_duration` | 1011 | new, default |
| `test_get_relay_addresses` | 1028 | new, default |

### Imports
- `use futures::{SinkExt, StreamExt}`
- `use quinn`
- `use std::collections::HashMap`
- `use std::net::SocketAddr`
- `use std::sync::Arc`
- `use super::*`
- `use super::protocol::{RelayCapability, RelayMessage, PROTOCOL_VERSION}`
- `use thiserror::Error`
- `use tokio::io::{AsyncReadExt, AsyncWriteExt}`
- `use tokio::net::TcpStream`
- `use tokio::sync::{Mutex, RwLock}`
- `use tokio::time::timeout`
- `use tokio_tungstenite::tungstenite::client::IntoClientRequest`
- `use tokio_tungstenite::tungstenite::protocol::Message`
- `use web_time::{Duration, SystemTime, UNIX_EPOCH}`
---

## core/src/message/codec.rs (1 chunks, 745 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/message/codec.rs: 16 functions; 9 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `validate_payload_size` | 22 | serialize, deserialize |
| `encode_message` | 34 | serialize, deserialize |
| `decode_message` | 51 | serialize, deserialize |
| `encode_envelope` | 73 | from_bytes, serialize, deserialize |
| `decode_envelope` | 100 | from_bytes, deserialize |
| `encode_drift_envelope` | 127 | now |
| `test_message_roundtrip` | 192 | text |
| `test_reject_oversized_payload` | 202 | thread_rng, text |
| `test_payload_boundary_accepts_8191_and_8192` | 212 | thread_rng, text |
| `test_payload_boundary_rejects_8193` | 223 | thread_rng, text |
| `test_reject_oversized_decode` | 231 | from_bytes, thread_rng |
| `test_envelope_roundtrip` | 238 | from_bytes, thread_rng |
| `test_encode_envelope_produces_drift_format` | 288 |  |
| `test_decode_envelope_drift_format` | 307 | serialize |
| `test_decode_envelope_bincode_fallback` | 338 | serialize |
| `test_envelope_compression_threshold` | 362 |  |

### Imports
- `use anyhow::{bail, Result}`
- `use crate::drift::DRIFT_VERSION`
- `use crate::drift::DriftEnvelope`
- `use crate::drift::envelope::COMPRESSION_THRESHOLD`
- `use crate::drift::{DriftEnvelope, EnvelopeType}`
- `use crate::message::types::Message`
- `use rand::RngCore`
- `use super::*`
- `use super::types::{Envelope, Message}`
---

## core/src/drift/compress.rs (1 chunks, 106 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/compress.rs: 10 functions; 2 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `compress` | 8 | compress_prepend_size, decompress_size_prepended, DecompressionFailed |
| `decompress` | 15 | decompress_size_prepended, DecompressionFailed |
| `test_compress_decompress_roundtrip` | 25 |  |
| `test_compress_empty_data` | 34 |  |
| `test_compress_single_byte` | 43 |  |
| `test_compress_large_data` | 52 |  |
| `test_decompress_invalid_data` | 62 |  |
| `test_compress_repetitive_data` | 68 |  |
| `test_compress_random_data` | 81 |  |
| `test_compress_text_data` | 95 |  |

### Imports
- `use super::*`
- `use super::DriftError`
---

## core/src/contacts_bridge.rs (1 chunks, 448 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/contacts_bridge.rs: Defines 4 types: Contact, Contact, ContactManager, ContactManager; 27 functions; 8 imports

### Structs/Classes
- Contact
- ContactManager

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 33 | new |
| `tombstone` | 47 | new, default, from |
| `with_nickname` | 62 | new, default, from, to_vec |
| `display_name` | 67 | new, default, from, to_vec |
| `federated_nickname` | 74 | new, default, from, to_vec |
| `new` | 90 | new, from, to_vec, default, from_slice |
| `add` | 106 | new, from_slice, to_vec |
| `get` | 121 | new, from_slice |
| `remove` | 137 | new, from_slice |
| `list` | 145 | new, from_slice |
| `search` | 162 | new, get, add, from_slice |
| `set_nickname` | 193 | get, add |
| `set_local_nickname` | 210 | get, add |
| `update_last_seen` | 227 | new, get, add |
| `update_device_id` | 239 | new, get, add |
| `reconcile_from_history` | 255 | new, get, add |
| `merge_remote_contacts` | 282 | get, add |
| `mark_verified` | 312 | get, add |
| `unverify` | 321 | new, get, add |
| `count` | 330 | new, get, now, add |
| `flush` | 334 | new, get, now, add |
| `verify_integrity` | 342 | new, get, now, add |
| `emergency_recover` | 356 | new, now, get, tempdir, add |
| `current_timestamp` | 373 | now, tempdir, new |
| `test_contact_creation` | 386 | new, tempdir |
| `test_contact_manager` | 395 | new, tempdir |
| `test_contact_persistence_across_manager_restart` | 427 | new, tempdir |

### Imports
- `use anyhow::{Context, Result}`
- `use crate::mobile_bridge::HistoryManager`
- `use parking_lot::Mutex`
- `use serde::{Deserialize, Serialize}`
- `use sled::Db`
- `use std::path::PathBuf`
- `use std::sync::Arc`
- `use super::*`
---

## core/src/privacy/cover.rs (1 chunks, 527 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/privacy/cover.rs: Defines 10 types: CoverTrafficError, CoverConfig, Default, CoverConfig, CoverMessage; 41 functions; 5 imports

### Structs/Classes
- CoverConfig
- CoverMessage
- CoverTrafficError
- CoverTrafficGenerator
- CoverTrafficScheduler
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 31 | InvalidConfig |
| `validate` | 42 | InvalidConfig |
| `message_interval_ms` | 62 | InvalidConfig |
| `is_cover_traffic` | 89 | thread_rng, InvalidConfig, generate_cover_message |
| `new` | 101 | thread_rng, InvalidConfig, generate_cover_message |
| `generate_cover_message` | 107 | thread_rng, InvalidConfig, generate_cover_message |
| `generate_batch` | 137 | new, from_millis, now, generate_cover_message |
| `config` | 142 | new, from_millis, now |
| `new` | 155 | new, from_millis, now |
| `should_generate_cover_traffic` | 166 | new, from_millis, now |
| `generate_and_update` | 181 | new, from_millis, default, now |
| `reset_timer` | 189 | from_millis, default |
| `next_generation_time` | 194 | from_millis, default |
| `config` | 204 | default |
| `is_cover_traffic` | 213 | default |
| `test_cover_config_default` | 224 | default |
| `test_cover_config_validate` | 232 |  |
| `test_cover_config_validate_zero_rate_enabled` | 242 |  |
| `test_cover_config_validate_zero_rate_disabled` | 252 |  |
| `test_cover_config_validate_zero_message_size` | 262 |  |
| `test_cover_config_validate_excessive_message_size` | 272 | new, default |
| `test_cover_config_message_interval` | 282 | new, default |
| `test_cover_config_message_interval_low_rate` | 292 | new, default |
| `test_cover_message_creation` | 302 | new, default |
| `test_cover_traffic_generator_new` | 316 | new, default |
| `test_cover_traffic_generator_invalid_config` | 323 | new |
| `test_cover_traffic_generator_disabled` | 334 | new |
| `test_generate_cover_message` | 346 | new |
| `test_generate_cover_message_uniqueness` | 362 | new, default |
| `test_generate_batch` | 379 | new, default |
| `test_cover_traffic_scheduler_new` | 396 | new, default |
| `test_cover_traffic_scheduler_should_generate_initially` | 403 | new, default |
| `test_cover_traffic_scheduler_disabled` | 415 | new, default |
| `test_cover_traffic_scheduler_generate_and_update` | 426 | new, default |
| `test_cover_traffic_scheduler_reset_timer` | 442 | new, default, serialize, deserialize |
| `test_cover_traffic_scheduler_next_generation_time` | 452 | new, serialize, deserialize |
| `test_cover_traffic_scheduler_next_generation_time_disabled` | 464 | new, serialize, deserialize |
| `test_is_cover_traffic` | 476 | new, serialize, deserialize |
| `test_cover_config_serialization` | 482 | new, serialize, deserialize |
| `test_cover_message_serialization` | 498 | new, serialize, deserialize |
| `test_cover_traffic_various_sizes` | 515 | new |

### Imports
- `use rand::RngCore`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{Duration, SystemTime}`
---

## core/src/relay/delegate_prewarm.rs (1 chunks, 426 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/delegate_prewarm.rs: Defines 10 types: DelegateInfo, DelegateInfo, WarmConnection, WarmConnection, DelegatePrewarmManager; 26 functions; 5 imports

### Structs/Classes
- Default
- DelegateInfo
- DelegatePrewarmConfig
- DelegatePrewarmManager
- DelegatePrewarmStats
- WarmConnection
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 39 | now |
| `update` | 52 | now |
| `has_capacity` | 59 | now |
| `score` | 64 | now |
| `new` | 88 | now |
| `update_keepalive` | 100 | now |
| `mark_registered` | 105 | from_secs |
| `is_stale` | 110 | from_secs |
| `default` | 148 | Vacant, new, select_best_delegates, now, default, from_secs |
| `new` | 162 | Vacant, new, select_best_delegates, now, default |
| `with_defaults` | 173 | new, select_best_delegates, default, Vacant |
| `add_delegate` | 178 | new, select_best_delegates, Vacant |
| `prewarm_for_background` | 185 | new, select_best_delegates, Vacant, now |
| `refresh_delegate_routes` | 215 | new, default, now |
| `select_best_delegates` | 237 | default |
| `tick` | 256 | now, default |
| `stats` | 289 | now |
| `active_connection_count` | 303 | new, from, generate_ed25519 |
| `registered_delegate_count` | 308 | new, from, generate_ed25519 |
| `fmt` | 330 | new, from, generate_ed25519, with_defaults |
| `create_test_delegate` | 345 | new, from, generate_ed25519, with_defaults |
| `test_delegate_creation` | 359 | new, with_defaults, now |
| `test_delegate_selection` | 366 | new, with_defaults, now |
| `test_prewarm_for_background` | 380 | new, with_defaults, now |
| `test_tick_maintenance` | 398 | new, with_defaults, now |
| `test_stats` | 415 | with_defaults |

### Imports
- `use libp2p::PeerId`
- `use libp2p::identity::Keypair`
- `use std::collections::{HashMap, VecDeque}`
- `use super::*`
- `use web_time::{Duration, Instant}`
---

## core/src/crypto/encrypt.rs (1 chunks, 926 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/crypto/encrypt.rs: 30 functions; 9 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `ed25519_to_x25519_secret` | 37 | decode, from, from_slice |
| `validate_ed25519_public_key` | 61 | decode, from, derive_key, from_slice |
| `ed25519_public_to_x25519` | 95 | random_from_rng, derive_key, from, from_slice |
| `derive_key` | 110 | derive_key, from, new_from_slice, random_from_rng, from_slice |
| `encrypt_message` | 123 | random_from_rng, from, new_from_slice, from_slice |
| `decrypt_message` | 183 | from, new_from_slice, from_slice |
| `decrypt_message_ratcheted` | 245 |  |
| `encrypt_message_ratcheted` | 288 |  |
| `encrypt_with_ratchet_fallback` | 320 | encode |
| `decrypt_with_ratchet_fallback` | 351 | serialize, encode |
| `is_ratcheted_envelope` | 371 | serialize |
| `sign_envelope` | 386 | from_bytes, serialize |
| `verify_envelope` | 415 | from_bytes, serialize |
| `generate_keypair` | 453 | from_bytes, encode |
| `test_validate_ed25519_public_key_valid` | 463 | encode |
| `test_validate_ed25519_public_key_invalid_hex` | 473 |  |
| `test_validate_ed25519_public_key_wrong_length` | 484 |  |
| `test_encrypt_decrypt_roundtrip` | 496 |  |
| `test_wrong_recipient_fails` | 510 |  |
| `test_tampered_ciphertext_fails` | 525 |  |
| `test_different_messages_different_ciphertext` | 543 |  |
| `test_sender_public_key_in_envelope` | 558 |  |
| `test_empty_plaintext` | 572 |  |
| `test_large_plaintext` | 584 |  |
| `test_invalid_envelope_nonce` | 597 |  |
| `test_aad_binding_prevents_sender_spoofing` | 613 |  |
| `test_sign_and_verify_envelope` | 634 |  |
| `test_tampered_envelope_fails_verification` | 651 |  |
| `test_forged_signature_fails_verification` | 674 |  |
| `test_relay_can_verify_without_decrypting` | 695 |  |

### Imports
- `use anyhow::{bail, Result}`
- `use curve25519_dalek::edwards::CompressedEdwardsY`
- `use ed25519_dalek::SigningKey`
- `use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey}`
- `use rand::RngCore`
- `use super::*`
- `use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret}`
- `use zeroize::Zeroize`
---

## core/src/drift/envelope.rs (1 chunks, 788 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/envelope.rs: Defines 5 types: DriftEnvelope, EnvelopeType, EnvelopeType, DriftEnvelope, DriftEnvelope; 30 functions; 6 imports

### Structs/Classes
- DriftEnvelope
- EnvelopeType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `from_u8` | 110 | with_capacity, compress, InvalidEnvelopeType, CiphertextTooLarge |
| `as_u8` | 125 | compress, with_capacity, CiphertextTooLarge |
| `to_bytes` | 141 | compress, with_capacity, CiphertextTooLarge |
| `from_bytes` | 200 | from_u8, from_le_bytes, InvalidVersion |
| `hint_from_public_key` | 340 | now, hash |
| `increment_hop` | 350 | IoError, now, hint_from_public_key, parse_str |
| `is_expired` | 357 | IoError, now, hint_from_public_key, parse_str |
| `to_legacy_envelope` | 371 | IoError, now, hint_from_public_key, parse_str |
| `from_legacy_envelope` | 388 | IoError, now, hint_from_public_key, parse_str |
| `sign` | 463 | new |
| `make_test_envelope` | 508 | from_u8 |
| `test_envelope_type_conversion` | 531 | from_bytes, from_u8 |
| `test_envelope_serialize_deserialize` | 561 | from_bytes |
| `test_envelope_empty_ciphertext` | 575 | from_bytes, CiphertextTooLarge, hint_from_public_key |
| `test_envelope_large_ciphertext` | 587 | from_bytes, CiphertextTooLarge, hint_from_public_key |
| `test_envelope_max_ciphertext` | 600 | from_bytes, CiphertextTooLarge, hint_from_public_key |
| `test_envelope_ciphertext_too_large` | 610 | CiphertextTooLarge, hint_from_public_key |
| `test_hint_from_public_key_deterministic` | 619 | hint_from_public_key |
| `test_hint_from_public_key_different_keys` | 628 | from_bytes, hint_from_public_key |
| `test_increment_hop` | 639 | from_bytes |
| `test_is_expired_never_expires` | 652 | from_bytes, InvalidVersion |
| `test_is_expired_in_future` | 660 | from_bytes, InvalidEnvelopeType, InvalidVersion |
| `test_is_expired_in_past` | 668 | from_bytes, InvalidEnvelopeType, InvalidVersion |
| `test_buffer_too_short` | 676 | from_bytes, InvalidEnvelopeType, InvalidVersion |
| `test_invalid_version` | 690 | from_bytes, InvalidEnvelopeType, InvalidVersion |
| `test_invalid_envelope_type` | 699 | from_bytes, InvalidEnvelopeType |
| `test_ciphertext_length_exceeds_buffer` | 709 | from_bytes |
| `test_little_endian_timestamps` | 724 | from_bytes |
| `test_compressed_envelope_roundtrip` | 744 | from_bytes |
| `test_compression_flag_preserved_on_roundtrip` | 774 | from_bytes |

### Imports
- `use crate::message::Envelope`
- `use ed25519_dalek::Signer`
- `use super::*`
- `use super::{DriftError, DRIFT_VERSION}`
- `use uuid::Uuid`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/message/ephemeral.rs (1 chunks, 64 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/message/ephemeral.rs: Defines 1 types: TtlConfig; 3 functions; 2 imports

### Structs/Classes
- TtlConfig

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `is_expired` | 22 | now |
| `test_not_expired` | 36 | now |
| `test_expired` | 50 | now |

### Imports
- `use std::time::{SystemTime, UNIX_EPOCH}`
- `use super::*`
---

## core/src/relay/findmy.rs (1 chunks, 463 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/findmy.rs: Defines 9 types: FindMyConfig, Default, FindMyConfig, WakeUpPayload, WakeUpPayload; 31 functions; 4 imports

### Structs/Classes
- Default
- FindMyBeaconManager
- FindMyConfig
- FindMyError
- WakeUpPayload

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 23 |  |
| `new` | 34 |  |
| `with_key` | 43 |  |
| `with_interval` | 49 |  |
| `new` | 68 | new |
| `encode_wakeup` | 101 | new |
| `decode_wakeup` | 142 | new |
| `is_our_wakeup` | 178 |  |
| `new` | 198 |  |
| `should_broadcast` | 206 |  |
| `generate_beacon` | 220 | new, default |
| `process_beacon` | 238 | new, default |
| `is_our_beacon` | 249 | new, default |
| `default` | 261 | new, default |
| `test_key` | 273 | new |
| `test_wakeup_payload_creation` | 279 | new |
| `test_encode_decode_roundtrip` | 287 | new |
| `test_encode_decode_message_flag_false` | 301 | new |
| `test_encode_produces_22_bytes` | 312 | new |
| `test_is_our_wakeup_match` | 321 | new |
| `test_is_our_wakeup_no_match` | 333 | new |
| `test_is_our_wakeup_invalid_length` | 346 | new |
| `test_beacon_manager_creation` | 357 | new |
| `test_beacon_manager_should_broadcast` | 366 | new |
| `test_beacon_manager_generate_beacon` | 378 | new, default |
| `test_beacon_manager_process_beacon` | 390 | new, default |
| `test_beacon_manager_is_our_beacon` | 409 | new, default |
| `test_find_my_config_default` | 426 | new, default |
| `test_find_my_config_builder` | 434 | new |
| `test_different_keys_produce_different_output` | 444 | new |
| `test_beacon_missing_key_error` | 456 | new |

### Imports
- `use blake3::Hasher`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
---

## core/src/drift/frame.rs (1 chunks, 423 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/frame.rs: Defines 4 types: DriftFrame, FrameType, FrameType, DriftFrame; 20 functions; 4 imports

### Structs/Classes
- DriftFrame
- FrameType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `from_u8` | 47 | InvalidFrameType, with_capacity, new |
| `as_u8` | 59 | new, with_capacity |
| `to_bytes` | 72 | new, with_capacity, from_le_bytes |
| `from_bytes` | 111 | new, from_u8, from_le_bytes |
| `read_with_timeout` | 183 | IoError, from_bytes, with_capacity, from_le_bytes |
| `make_test_frame` | 229 | from_bytes, from_u8 |
| `test_frame_type_conversion` | 238 | from_bytes, from_u8 |
| `test_frame_serialize_deserialize` | 251 | from_bytes |
| `test_frame_empty_payload` | 263 | from_bytes |
| `test_frame_large_payload` | 275 | from_bytes |
| `test_frame_crc32_validation` | 287 | from_bytes |
| `test_frame_crc32_tamper_type` | 299 | from_bytes |
| `test_frame_crc32_tamper_crc` | 311 | from_bytes |
| `test_frame_buffer_too_short` | 325 | from_bytes |
| `test_frame_length_mismatch` | 339 | from_bytes |
| `test_frame_invalid_type` | 349 | from_bytes, from_le_bytes |
| `test_frame_all_types` | 362 | from_bytes, from_le_bytes |
| `test_frame_length_calculation` | 384 | from_bytes, from_le_bytes |
| `test_frame_crc32_deterministic` | 403 | from_bytes |
| `test_frame_multiple_roundtrips` | 412 | from_bytes |

### Imports
- `use crc32fast::Hasher`
- `use super::*`
- `use super::DriftError`
- `use tokio::time::timeout`
---

## core/src/bin/gen_swift.rs (1 chunks, 171 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/bin/gen_swift.rs: 2 functions; 3 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `main` | 2 | new, var, create_dir_all, from |
| `main` | 171 |  |

### Imports
- `use camino::Utf8PathBuf`
- `use std::fs`
- `use uniffi_bindgen::bindings::{generate, GenerateOptions, TargetLanguage}`
---

## core/src/relay/invite.rs (1 chunks, 498 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/invite.rs: Defines 8 types: InviteError, InviteToken, InviteToken, InviteChain, InviteChain; 47 functions; 5 imports

### Structs/Classes
- Default
- InviteChain
- InviteError
- InviteSystem
- InviteToken

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 44 | now, new |
| `with_expiry` | 62 | now, new, SerializationError, deserialize, serialize |
| `with_metadata` | 68 | now, new, SerializationError, deserialize, serialize |
| `with_signature` | 74 | now, new, SerializationError, deserialize, serialize |
| `is_valid` | 80 | now, new, SerializationError, deserialize, serialize |
| `get_signable_data` | 90 | new, now, SerializationError, deserialize, serialize |
| `to_bytes` | 105 | new, now, SerializationError, deserialize, serialize |
| `from_bytes` | 110 | new, now, SerializationError, deserialize |
| `new` | 124 | new, get_inviter, now |
| `record_invite` | 131 | now, get_inviter, get_trust_chain |
| `get_inviter` | 142 | get_inviter, get_trust_chain |
| `get_invitees` | 152 | get_inviter, get_trust_chain |
| `get_trust_chain` | 166 | new, get_inviter, get_trust_chain |
| `distance_from_root` | 179 | new, get_trust_chain |
| `invite_count` | 184 | new |
| `clear` | 189 | new |
| `get_direct_invitations` | 194 | new |
| `default` | 209 | new |
| `new` | 226 | new |
| `create_invite_token` | 235 | new |
| `record_invitation` | 240 | new |
| `get_trust_chain` | 247 | new |
| `get_invitees` | 252 | new |
| `get_inviter` | 257 | new |
| `is_direct_connection` | 262 | new |
| `get_connected_peers` | 270 | new |
| `test_token` | 288 | new, sleep, from_bytes, from_millis |
| `test_invite_token_creation` | 294 | from_bytes, sleep, from_millis, new |
| `test_invite_token_with_expiry` | 302 | from_bytes, sleep, from_millis, new |
| `test_invite_token_with_metadata` | 308 | from_bytes, sleep, from_millis, new |
| `test_invite_token_validity` | 314 | from_bytes, sleep, from_millis, new |
| `test_invite_token_expiry_check` | 323 | from_bytes, sleep, from_millis, new |
| `test_invite_token_serialization` | 331 | from_bytes, new |
| `test_invite_chain_creation` | 342 | new |
| `test_record_invite` | 348 | new |
| `test_get_invitees` | 357 | new |
| `test_get_trust_chain` | 373 | new |
| `test_distance_from_root` | 385 | new |
| `test_get_direct_invitations` | 402 | new |
| `test_invite_system_creation` | 416 | new |
| `test_create_invite_token` | 422 | new |
| `test_record_invitation` | 431 | new |
| `test_get_inviter` | 440 | new |
| `test_is_direct_connection` | 455 | new |
| `test_get_connected_peers` | 464 | new |
| `test_get_trust_chain_via_system` | 476 | new |
| `test_chain_clear` | 490 | new |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/iron_core.rs (1 chunks, 3823 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/iron_core.rs: Defines 7 types: CoreDelegate, ConsentState, IronCore, IdentityBackupPayload, Default; 232 functions; 27 imports

### Structs/Classes
- ConsentState
- CoreDelegate
- Default
- IdentityBackupPayload
- IronCore

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `on_peer_discovered` | 86 |  |
| `on_peer_disconnected` | 87 |  |
| `on_peer_identified` | 88 |  |
| `on_message_received` | 89 |  |
| `on_receipt_received` | 97 |  |
| `default` | 253 | persistent, new_heuristics_only, new, default, build_security_audit_pipeline |
| `new` | 262 | temp_dir, persistent, new_heuristics_only, new, default, build_security_audit_pipeline |
| `with_storage` | 341 | persistent, new_heuristics_only, new, default, build_security_audit_pipeline, with_backend |
| `with_storage_and_logs` | 428 | persistent, new_heuristics_only, new, default, build_security_audit_pipeline, with_backend |
| `start` | 513 | drift_activate, drift_deactivate, default, new |
| `stop` | 525 | new, drift_deactivate, default, hash |
| `grant_consent` | 532 | new, default, hash |
| `initialize_identity` | 539 | new, default, hash |
| `identity_id` | 590 | decode, new_v4 |
| `device_id` | 595 | now, decode, new_v4 |
| `public_key_hex` | 600 | now, decode, new_v4, encode_message |
| `set_delegate` | 607 | from_legacy_envelope, now, decode, encode_message, new_v4 |
| `prepare_message_internal` | 618 | privacy_config, prepare_onion_message, to_string, from_legacy_envelope, now, decode, swarm_get_best_relays, encode_message, new_v4 |
| `prepare_message` | 742 | prepare_message_internal, parse_str |
| `prepare_message_with_id` | 753 | from_utf8_lossy, prepare_message_internal, prepare_message, parse_str |
| `mark_message_sent` | 766 | from_utf8_lossy, mark_message_sent, prepare_message, parse_str |
| `send_message_status` | 788 | from_utf8_lossy, prepare_message, mark_message_sent |
| `is_peer_blocked` | 814 |  |
| `blocked_only_peer_ids` | 827 |  |
| `register_blocked_device` | 836 |  |
| `get_blocked_peer_devices` | 848 |  |
| `get_peer_reputation` | 853 |  |
| `peer_spam_score` | 858 |  |
| `peer_rate_limit_multiplier` | 866 |  |
| `sign_data` | 871 |  |
| `get_registration_state` | 882 |  |
| `notify_peer_discovered` | 900 |  |
| `notify_peer_disconnected` | 916 |  |
| `record_abuse_signal` | 923 | now |
| `drift_activate` | 952 | now |
| `drift_deactivate` | 961 | now |
| `run_maintenance_cycle` | 971 | now |
| `drift_network_state` | 991 | now |
| `drift_store_size` | 1000 | now |
| `relay_jitter_delay` | 1005 | now |
| `is_running` | 1025 |  |
| `get_identity_info` | 1033 |  |
| `get_device_id` | 1057 |  |
| `get_libp2p_peer_id` | 1064 | decode, verify |
| `get_seniority_timestamp` | 1070 | decode, verify |
| `set_nickname` | 1076 | decode, verify |
| `verify_signature` | 1104 | decode, verify |
| `outbox_count` | 1119 |  |
| `inbox_count` | 1123 |  |
| `drain_received_messages` | 1131 | new |
| `peek_received_messages` | 1139 | new |
| `block_peer` | 1150 | new |
| `unblock_peer` | 1198 |  |
| `block_and_delete_peer` | 1210 |  |
| `list_blocked_peers_bridge` | 1245 | encode |
| `blocked_count` | 1254 | contacts_manager, encode |
| `clear_history` | 1260 | to_string, contacts_manager, encode |
| `list_blocked` | 1267 | to_string, contacts_manager, encode |
| `build_identity_backup_payload` | 1285 | to_string, encrypt_backup, build_identity_backup_payload, encode, contacts_manager |
| `export_identity_backup` | 1321 | encrypt_backup, build_identity_backup_payload |
| `export_identity_backup_with_salt` | 1334 | encrypt_backup, encrypt_backup_fast, build_identity_backup_payload |
| `export_identity_backup_fast` | 1373 | encrypt_backup_fast, build_identity_backup_payload |
| `export_identity_backup_fast_with_salt` | 1389 | encrypt_backup_fast, decode, decrypt_backup, build_identity_backup_payload |
| `import_identity_backup` | 1424 | decode, new, decrypt_backup, from_str |
| `extract_public_key_from_peer_id` | 1528 | try_decode_protobuf, to_vec, encode, now |
| `prepare_receipt` | 1550 | now, classify_notification, to_vec |
| `prepare_cover_traffic` | 1567 | classify_notification |
| `classify_notification` | 1578 | from_str, classify_notification |
| `get_audit_log` | 1591 | from_str |
| `get_audit_events_since` | 1595 | from_str |
| `abuse_overall_score` | 1609 | from_str, from_bytes, decode |
| `set_privacy_config` | 1621 | from_bytes, decode, encode, from_str, hash |
| `resolve_identity` | 1641 | from_bytes, decode, encode, extract_public_key_from_peer_id, hash |
| `resolve_to_identity_id` | 1691 | resolve_identity, decode, hash, encode |
| `perform_maintenance` | 1701 | validate_audit_chain |
| `update_disk_stats` | 1746 | privacy_config, list_blocked_peers_bridge, to_string_pretty |
| `get_disk_stats` | 1752 | privacy_config, list_blocked_peers_bridge, to_string_pretty |
| `record_log` | 1756 | privacy_config, list_blocked_peers_bridge, to_string_pretty |
| `export_logs` | 1760 | privacy_config, list_blocked_peers_bridge, to_string_pretty |
| `export_audit_log` | 1770 | privacy_config, list_blocked_peers_bridge, to_string_pretty |
| `validate_audit_chain` | 1776 | privacy_config, list_blocked_peers_bridge, to_string_pretty |
| `get_privacy_config` | 1784 | privacy_config, list_blocked_peers_bridge, to_string_pretty |
| `list_blocked_peers` | 1791 | list_blocked_peers_bridge |
| `compute_jitter_delay` | 1815 | new |
| `contacts_manager` | 1855 | new |
| `contact_federated_nickname` | 1867 |  |
| `contact_display_name` | 1877 | deserialize |
| `contact_update_last_known_device_id` | 1892 | get_signature, deserialize |
| `invite_get_signable_data` | 1919 | new, get_signature, deserialize |
| `dspy_verify_signature` | 1933 | new, get_signature |
| `dspy_get_signature` | 1939 | new, to_string, get_signature |
| `history_manager` | 1945 | new, to_string |
| `custody_audit_count` | 1961 | to_string |
| `custody_get_registration_state_info` | 1966 | new, to_string |
| `custody_registration_transitions` | 1982 | new, to_string |
| `get_audit_events_by_type` | 1995 | new |
| `get_auto_adjust_engine` | 2014 | new |
| `is_consent_granted` | 2023 |  |
| `on_app_resume` | 2033 | from_str |
| `on_app_background` | 2046 | from_str, decode, construct_onion, serialize |
| `prepare_onion_message` | 2073 | construct_onion, decode, deserialize, peel_layer, from_str, serialize |
| `peel_onion_layer` | 2099 | from_le_bytes, peel_layer, serialize, deserialize |
| `random_port` | 2129 | from_le_bytes |
| `ratchet_session_count` | 2141 | decode |
| `ratchet_has_session` | 2146 | decode |
| `ratchet_reset_session` | 2151 | new, decode, routing_peer_seen |
| `routing_peer_seen` | 2161 | new, decode, routing_peer_seen, now |
| `routing_update_peer_hints` | 2168 | Object, from_iter, new, now, decode, to_value, routing_peer_seen |
| `routing_mark_gateway` | 2182 | Object, from_iter, new, now, decode, from, to_value, routing_peer_seen |
| `routing_update_reliability` | 2197 | Object, from_iter, new, now, from, to_value, routing_peer_seen |
| `routing_tick` | 2206 | Object, from_iter, now, new, from, to_value |
| `routing_summary` | 2283 | to_string |
| `routing_clear_unreachable_peer` | 2295 |  |
| `routing_current_discovery_phase` | 2302 |  |
| `routing_negative_cache_stats` | 2314 |  |
| `routing_prefetch_stats` | 2329 | calculate_dynamic_ttl |
| `routing_timeout_budget_summary` | 2344 | calculate_dynamic_ttl |
| `routing_calculate_dynamic_ttl` | 2363 | calculate_dynamic_ttl |
| `routing_register_path` | 2379 | new |
| `routing_mark_path_failed` | 2389 | new |
| `swarm_get_best_relays` | 2402 | new |
| `swarm_get_bootstrap_candidates` | 2425 | new, now, encode, hash, new_v4 |
| `swarm_can_bootstrap_others` | 2446 | now, hash, new_v4, encode |
| `swarm_get_best_paths` | 2451 | now, hash, new_v4, encode |
| `outbox_contains_for_recipient` | 2495 | blake3_hash, parse_str |
| `ratchet_sessions_handle` | 2506 | create_basic, create_cot, blake3_hash, parse_str |
| `test_only_identity_signing_key` | 2519 | create_multihop, blake3_hash, create_basic, create_cot, parse_str |
| `drift_contains` | 2532 | create_multihop, blake3_hash, create_basic, create_cot, parse_str, create_optimizer, build_security_audit_pipeline |
| `dspy_blake3_hash` | 2540 | build_rust_feature_pipeline, create_multihop, blake3_hash, create_basic, create_optimizer, create_cot, build_security_audit_pipeline |
| `dspy_create_basic_teleprompter` | 2545 | build_rust_feature_pipeline, create_multihop, create_basic, create_optimizer, create_cot, build_security_audit_pipeline |
| `dspy_create_cot` | 2550 | build_rust_feature_pipeline, create_multihop, create_optimizer, create_cot, build_security_audit_pipeline |
| `dspy_add_step` | 2555 | build_security_audit_pipeline, build_rust_feature_pipeline, create_multihop, create_optimizer |
| `dspy_create_multihop` | 2560 | build_security_audit_pipeline, build_rust_feature_pipeline, create_multihop, create_optimizer |
| `dspy_create_optimizer` | 2569 | build_rust_feature_pipeline, build_security_audit_pipeline, create_optimizer |
| `dspy_build_security_audit_pipeline` | 2578 | build_rust_feature_pipeline, build_security_audit_pipeline |
| `dspy_build_rust_feature_pipeline` | 2583 | build_rust_feature_pipeline |
| `list_blocked_wasm` | 2589 |  |
| `list_blocked_peers_wasm` | 2598 |  |
| `get_libp2p_keypair` | 2603 |  |
| `receive_message` | 2610 | now |
| `build_registration_request` | 2704 | new_signed |
| `get_identity_keys` | 2713 | new |
| `flush_outbox_for_peer` | 2716 | new |
| `contacts_store_manager` | 2719 | new |
| `history_store_manager` | 2722 | new |
| `list_blocked_peers_raw` | 2725 | new |
| `get_enhanced_peer_reputation` | 2730 | new |
| `privacy_config` | 2738 | new |
| `make_routing_decision` | 2741 | new |
| `routing_engine_handle` | 2753 | new |
| `set_cover_traffic_generator` | 2756 | new |
| `set_timing_jitter` | 2766 | new |
| `set_circuit_builder` | 2776 | new |
| `register_notification_endpoint` | 2790 |  |
| `unregister_notification_endpoint` | 2801 |  |
| `list_notification_endpoints` | 2809 |  |
| `clear_all_request_notifications` | 2812 |  |
| `clear_message_notifications` | 2817 |  |
| `close_all_notifications` | 2822 |  |
| `transport_manager_handle` | 2827 |  |
| `get_healthy_connections` | 2832 |  |
| `expire_address_observations` | 2838 |  |
| `bootstrap_manager_handle` | 2844 |  |
| `peer_exchange_manager_handle` | 2848 |  |
| `get_unhealthy_connections` | 2858 |  |
| `get_all_connection_stats` | 2864 |  |
| `cleanup_stale_connections` | 2873 |  |
| `current_discovery_phase` | 2881 |  |
| `clear_unreachable_peer` | 2891 |  |
| `get_peer_activity` | 2900 |  |
| `relay_bootstrap_manager_handle` | 2916 | swarm_can_bootstrap_others |
| `get_all_relay_stats` | 2928 | swarm_can_bootstrap_others |
| `get_fallback_relays` | 2942 | swarm_can_bootstrap_others |
| `can_bootstrap_others` | 2955 | swarm_can_bootstrap_others, calculate_dynamic_ttl, get_hole_punch_status |
| `get_healthy_relays` | 2964 | calculate_dynamic_ttl, get_hole_punch_status |
| `custody_audit_count_usize` | 2974 | new, clear_unreachable_peer, calculate_dynamic_ttl, get_hole_punch_status |
| `get_registration_state_info` | 2979 | clear_unreachable_peer, get_hole_punch_status, new, decode, calculate_dynamic_ttl |
| `calculate_dynamic_ttl` | 2992 | clear_unreachable_peer, get_hole_punch_status, new, decode, calculate_dynamic_ttl |
| `get_hole_punch_status` | 3001 | new, decode, clear_unreachable_peer |
| `get_active_paths` | 3015 | new, decode, clear_unreachable_peer |
| `record_reconnect_success_and_clear_cache` | 3023 | decode, clear_unreachable_peer |
| `peers_needing_reconnect` | 3044 | new, default |
| `reset_circuit_breakers` | 3052 | new, default |
| `disable_transport` | 3070 | new, default |
| `start_hole_punch` | 3080 | new, default |
| `register_state_change_callback` | 3105 | new, now |
| `relay_discovery_mut` | 3122 | now, decode |
| `get_best_forwarding_path` | 3146 | now, decode, new |
| `get_available_paths` | 3166 | new, decode |
| `get_forwarding_capability` | 3188 |  |
| `routing_prefetch_stats_detailed` | 3215 | decode, from |
| `force_ratchet` | 3232 | decode, from |
| `create_receiver_session` | 3244 | decode, from |
| `converge_delivered_for_message` | 3276 | to_string, for_local_peer |
| `registration_transitions_for_identity` | 3291 | to_string, for_local_peer |
| `enforce_storage_pressure` | 3302 | for_local_peer |
| `storage_pressure_state` | 3313 | now, for_local_peer |
| `custody_store_for_peer` | 3322 | now, drift_apply_policy, for_local_peer |
| `drift_apply_policy` | 3332 | now, drift_apply_policy |
| `update_device_state` | 3341 | now, drift_apply_policy |
| `get_policy_relay_config` | 3369 |  |
| `current_relay_profile` | 3374 | new |
| `drift_set_cover_traffic` | 3381 | new |
| `drift_set_reputation_manager` | 3390 | new, get_auto_adjust_engine |
| `drift_generate_cover_traffic_if_due` | 3409 | new, get_auto_adjust_engine |
| `new_drift_sync` | 3419 | new, get_auto_adjust_engine |
| `override_ble_advertise_interval` | 3431 | from_str, get_auto_adjust_engine |
| `override_relay_priority_threshold` | 3440 | from_str, default, get_auto_adjust_engine |
| `compute_ble_adjustment` | 3448 | from_str, default, get_auto_adjust_engine |
| `compute_relay_adjustment` | 3459 | from_str, new, default, get_auto_adjust_engine |
| `apply_policy_config` | 3475 | from_str, new, default |
| `emergency_recover` | 3527 | can_forward_for_wasm |
| `blocked_only_peer_ids_set` | 3541 | can_forward_for_wasm |
| `can_forward_for_wasm` | 3559 | to_string, can_forward_for_wasm |
| `can_reach_destination` | 3569 | to_string |
| `routing_refresh_delegate_routes` | 3584 | to_string |
| `routing_run_optimization` | 3593 | to_string |
| `routing_evaluate_all_tracked` | 3605 |  |
| `routing_prune_below` | 3615 |  |
| `routing_should_advance` | 3624 |  |
| `prefetch_start_refresh` | 3638 |  |
| `routing_mark_refresh_failed` | 3648 |  |
| `routing_next_refresh_hint` | 3657 |  |
| `routing_is_prefetch_complete` | 3667 |  |
| `routing_is_prefetch_in_progress` | 3677 | from_str, new |
| `routing_start_refresh` | 3687 | from_str, new |
| `touch_notification_endpoint` | 3695 | from_str, new |
| `update_keepalive` | 3707 | from_str, new |
| `test_record_and_export_logs` | 3718 | from_str, new |
| `test_export_logs_empty` | 3753 | from_str, new |
| `test_update_disk_stats_with_app_data` | 3760 | from_str, new |
| `test_record_log_persistence` | 3782 | from_str, new |

### Imports
- `use crate::IronCoreError`
- `use crate::abuse::EnhancedAbuseReputationManager`
- `use crate::abuse::auto_block::{AutoBlockConfig, AutoBlockEngine}`
- `use crate::abuse::spam_detection::{SpamDetectionConfig, SpamDetectionEngine}`
- `use crate::crypto::{decrypt_message, encrypt_message, session_manager::RatchetSessionManager}`
- `use crate::drift::{MeshStore, NetworkState, RelayConfig, RelayEngine}`
- `use crate::identity::IdentityManager`
- `use crate::message::{decode_envelope, decode_message, Message}`
- `use crate::notification::NotificationEndpointRegistry`
- `use crate::observability::{AuditEventType, AuditLog as AuditLogType}`
- `use crate::relay::{BootstrapManager, PeerExchangeManager}`
- `use crate::routing::local::TransportType`
- `use crate::routing::optimized_engine::OptimizedRoutingEngine`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::backend::SledStorage`
- `use crate::store::blocked::BlockedManager as CoreBlockedManager`
- `use crate::store::logs::LogManager`
- `use crate::transport::behaviour::RegistrationRequest`
- `use crate::transport::manager::TransportManager`
- `use crate::transport::nat::NatTraversal`
- `use crate::transport::reputation::AbuseSignal`
- `use parking_lot::RwLock`
- `use rand::RngCore`
- `use std::sync::Arc`
- `use std::time::{SystemTime, UNIX_EPOCH}`
- `use super::*`
---

## core/src/identity/keys.rs (1 chunks, 660 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/identity/keys.rs: Defines 4 types: KeyPair, KeyPair, IdentityKeys, IdentityKeys; 25 functions; 6 imports

### Structs/Classes
- IdentityKeys
- KeyPair

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `generate` | 15 | from_bytes, hash, encode |
| `verifying_key` | 25 | from_bytes, hash, encode |
| `generate` | 38 | from_bytes, hash, encode |
| `public_key_hex` | 48 | from_bytes, hash, encode |
| `identity_id` | 53 | from_bytes, to_bytes, hash, encode |
| `sign` | 60 | from_bytes, to_bytes, try_from_bytes |
| `verify` | 66 | from_bytes, to_bytes, try_from_bytes |
| `to_bytes` | 83 | from_bytes, to_libp2p_keypair, try_from_bytes, from, to_bytes |
| `from_bytes` | 88 | from_bytes, to_libp2p_keypair, try_from_bytes, from, to_bytes |
| `to_libp2p_peer_id` | 111 | to_libp2p_keypair, try_from_bytes, decode, from |
| `to_libp2p_keypair` | 127 | from_be_bytes, with_capacity, new, try_from_bytes, decode, from |
| `safety_number` | 148 | from_be_bytes, with_capacity, new, decode, generate |
| `test_key_generation` | 193 | from_bytes, verify, generate |
| `test_signing` | 203 | from_bytes, verify, generate |
| `test_verification` | 212 | from_bytes, verify, generate |
| `test_serialization` | 228 | from_bytes, generate |
| `test_libp2p_peer_id_derivation` | 240 | generate |
| `test_identity_hash_differs_from_public_key` | 262 | generate |
| `test_peer_id_roundtrip_deterministic` | 278 | try_decode_protobuf, generate, encode |
| `test_peer_id_unique_per_keypair` | 290 | try_decode_protobuf, generate, encode |
| `test_public_key_to_peer_id_to_public_key_roundtrip` | 303 | try_decode_protobuf, from_bytes, decode, encode, generate |
| `test_identity_id_is_not_valid_ed25519_point` | 331 | from_bytes, decode, generate |
| `test_safety_number_is_order_independent_and_deterministic` | 357 | generate |
| `test_safety_number_differs_for_different_key_pairs` | 378 | generate |
| `test_safety_number_rejects_malformed_keys` | 390 |  |

### Imports
- `use anyhow::Result`
- `use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey}`
- `use rand::RngCore`
- `use super::*`
- `use zeroize::Zeroize`
---

## cli/src/lib.rs (1 chunks, 19 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/lib.rs: structural extraction

---

## core/src/mobile_bridge.rs (1 chunks, 4283 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/mobile_bridge.rs: Defines 49 types: MeshServiceConfig, ServiceState, ConnectionPathState, MotionState, ProximityTransport; 242 functions; 14 imports

### Structs/Classes
- AdjustmentProfile
- AutoAdjustEngine
- BehaviorAdjustment
- BleAdjustment
- ConnectionPathState
- Default
- DeviceProfile
- DeviceState
- HistoryManager
- HistoryStats
- LedgerEntry
- LedgerManager
- MeshService
- MeshServiceConfig
- MeshServiceCoreDelegate
- MeshSettingsManager
- MessageDirection
- MessageRecord
- MessageStatus
- MockBridgeHandle
- MockPlatformBridge
- MotionState
- NetworkType
- PlatformBridge
- PlatformWifiAwareBridge
- ProximityTransport
- RelayAdjustment
- ServiceState
- ServiceStats
- SwarmBridge
- WifiAwareCallback
- WifiAwarePlatformBridge
- crate
- fmt

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `max_payload_size` | 63 |  |
| `fmt` | 74 |  |
| `from_profile` | 119 | recommended_behavior |
| `new` | 193 | new, default |
| `with_storage` | 220 | new, default |
| `with_storage_and_logs` | 247 | new, default, with_storage_and_logs |
| `start` | 275 | new, downgrade, with_storage, with_storage_and_logs |
| `set_delegate` | 425 |  |
| `stop` | 428 | get_swarm_bridge |
| `pause` | 456 | default, get_swarm_bridge |
| `resume` | 463 | default, get_swarm_bridge |
| `get_state` | 470 | default, get_swarm_bridge |
| `get_stats` | 474 | default, get_swarm_bridge |
| `reset_stats` | 481 | default |
| `set_platform_bridge` | 486 | get_stats |
| `update_keepalive` | 492 | get_stats |
| `get_nat_status` | 512 | Object, from_iter, get_state, get_nat_status, get_stats, from, get_connection_path_state, to_value |
| `get_connection_path_state` | 515 | Object, from_iter, get_state, get_nat_status, get_stats, from, get_connection_path_state, to_value |
| `export_diagnostics` | 531 | Object, from_iter, get_state, get_nat_status, get_stats, from, get_connection_path_state, to_value |
| `start_swarm` | 607 | resolve_swarm_keypair_and_mode |
| `get_swarm_bridge` | 980 | from_profile |
| `update_device_state` | 984 | from_profile |
| `recommended_behavior` | 1119 | get_core |
| `get_device_state` | 1127 | get_core |
| `set_relay_budget` | 1130 | get_core |
| `get_auto_adjust_engine` | 1152 | get_core |
| `on_peer_discovered` | 1155 | get_core |
| `on_peer_disconnected` | 1165 | get_core |
| `on_data_received` | 1172 | get_core |
| `on_battery_changed` | 1228 | on_proximity_data_received, resume, update_device_state, pause |
| `on_network_changed` | 1239 | on_proximity_data_received, resume, update_device_state, pause |
| `on_motion_changed` | 1250 | resume, on_data_received, update_device_state, pause, on_proximity_data_received |
| `on_entering_background` | 1260 | resume, get_core, on_data_received, pause, on_proximity_data_received |
| `on_entering_foreground` | 1265 | on_data_received, on_proximity_data_received, resume, get_core |
| `on_ble_data_received` | 1270 | on_data_received, on_proximity_data_received, get_core |
| `on_proximity_data_received` | 1274 | on_data_received, get_core |
| `get_core` | 1299 | derive_key, get_core |
| `run_maintenance_cycle` | 1304 | derive_key, get_core |
| `on_wifi_aware_peer_discovered` | 1311 | derive_key |
| `on_wifi_aware_data_path_confirmed` | 1339 |  |
| `on_wifi_direct_peer_discovered` | 1350 |  |
| `on_wifi_direct_connection_info` | 1373 |  |
| `export_identity_backup` | 1403 |  |
| `export_identity_backup_with_salt` | 1415 |  |
| `export_identity_backup_fast` | 1431 |  |
| `export_identity_backup_fast_with_salt` | 1444 |  |
| `import_identity_backup` | 1456 | new, get_core |
| `prepare_onion_message` | 1475 | new, get_core |
| `peel_onion_layer` | 1488 | new, get_core |
| `random_port` | 1500 | new, get_core |
| `ratchet_session_count` | 1509 | new, get_core |
| `ratchet_has_session` | 1518 | new, get_core |
| `ratchet_reset_session` | 1527 | new, get_core |
| `routing_tick` | 1534 | new, dispatch_proximity_packet, get_core |
| `is_running` | 1543 | new, dispatch_proximity_packet, get_core |
| `get_all_connection_stats` | 1549 | new, dispatch_proximity_packet, get_core |
| `dispatch_ble_packet` | 1580 | dispatch_proximity_packet |
| `dispatch_proximity_packet` | 1585 |  |
| `compute_behavior` | 1613 |  |
| `resolve_swarm_keypair_and_mode` | 1656 | generate_ed25519, create_dir_all, load_or_create_headless_network_keypair, from, from_protobuf_encoding, read |
| `load_or_create_headless_network_keypair` | 1676 | from_mode, generate_ed25519, create_dir_all, from, from_protobuf_encoding, write, read, set_permissions |
| `on_peer_discovered` | 1727 |  |
| `on_peer_disconnected` | 1735 |  |
| `on_peer_identified` | 1744 |  |
| `on_message_received` | 1757 |  |
| `on_receipt_received` | 1778 |  |
| `on_battery_changed` | 1790 |  |
| `on_network_changed` | 1791 |  |
| `on_motion_changed` | 1792 |  |
| `on_ble_data_received` | 1793 |  |
| `on_entering_background` | 1794 |  |
| `on_entering_foreground` | 1795 | new |
| `send_ble_packet` | 1796 | new |
| `on_proximity_data_received` | 1797 | new |
| `send_proximity_packet` | 1803 | new |
| `wifi_aware_publish` | 1804 | new |
| `wifi_aware_subscribe` | 1805 | new |
| `wifi_aware_create_data_path` | 1806 | new |
| `wifi_aware_stop` | 1807 | new |
| `wifi_direct_discover_peers` | 1808 | new |
| `wifi_direct_stop_discovery` | 1809 | new |
| `wifi_direct_connect` | 1810 | new |
| `wifi_direct_create_group` | 1811 | new |
| `wifi_direct_remove_group` | 1812 | new |
| `on_service_discovered` | 1816 | new |
| `on_data_path_confirmed` | 1817 | new |
| `new_platform_ref` | 1840 | new |
| `with_platform` | 1850 | new, with_platform |
| `handle_service_discovered` | 1857 | new, with_platform |
| `handle_data_path_confirmed` | 1866 | PlatformError, new, with_platform |
| `get_discovered_peer` | 1890 | PlatformError, with_platform |
| `is_available` | 1898 | PlatformError, with_platform |
| `publish_service` | 1901 | PlatformError |
| `subscribe_to_services` | 1918 | PlatformError, DataPathFailed, channel |
| `unpublish_service` | 1933 | from_secs, DataPathFailed, timeout, channel |
| `unsubscribe_from_services` | 1940 | from_secs, DataPathFailed, timeout, channel |
| `create_data_path` | 1947 | from_secs, DataPathFailed, timeout, channel |
| `close_data_path` | 1981 |  |
| `set_on_service_discovered` | 1985 |  |
| `set_on_message_received` | 1989 |  |
| `set_on_data_path_confirmed` | 1991 |  |
| `default` | 2043 | new |
| `new` | 2051 | new |
| `compute_profile` | 2057 |  |
| `compute_ble_adjustment` | 2072 |  |
| `compute_relay_adjustment` | 2088 |  |
| `override_ble_scan_interval` | 2104 | from |
| `override_ble_advertise_interval` | 2108 | from |
| `override_relay_max_per_hour` | 2119 | from_str, read_to_string, default, from |
| `override_relay_priority_threshold` | 2123 | validate, from, create_dir_all, read_to_string, default, from_str |
| `clear_overrides` | 2132 | to_string_pretty, validate, from, create_dir_all, read_to_string, default, write, from_str |
| `new` | 2151 | to_string_pretty, validate, from, create_dir_all, read_to_string, default, write, from_str |
| `load` | 2156 | to_string_pretty, validate, create_dir_all, read_to_string, default, write, from_str |
| `save` | 2169 | to_string_pretty, validate, create_dir_all, default, write |
| `validate` | 2183 | default |
| `default_settings` | 2210 | default |
| `adjust_legacy_timestamps` | 2252 | new, default, from, to_vec |
| `new` | 2276 | recent_internal, new, from, to_vec, default, from_slice |
| `add` | 2289 | new, from_slice, recent_internal, to_vec |
| `get` | 2298 | new, recent_internal, from_slice |
| `recent` | 2312 | new, recent_internal, from_slice |
| `recent_including_hidden` | 2323 | new, recent_internal, from_slice |
| `recent_internal` | 2330 | new, recent, from_slice |
| `conversation` | 2369 | new, recent, from_slice |
| `remove_conversation` | 2377 | new, from_slice |
| `search` | 2400 | new, from_slice |
| `unhide_messages_for_peer` | 2434 | new, to_vec, get, from_slice |
| `hide_messages_for_peer` | 2459 | add, new, get, to_vec, from_slice |
| `mark_delivered` | 2482 | add, new, get, default, from_slice |
| `clear` | 2490 | new, default, from_slice |
| `clear_conversation` | 2496 | new, default, from_slice |
| `stats` | 2519 | default, with_capacity, from_slice |
| `count` | 2542 | new, with_capacity, Reverse, from_slice |
| `flush` | 2547 | new, with_capacity, Reverse, from_slice |
| `enforce_retention` | 2557 | new, with_capacity, Reverse, from_slice |
| `prune_before` | 2590 | new, from_slice |
| `delete` | 2611 | from_str, new, from, read_to_string |
| `new` | 2645 | to_string_pretty, new, create_dir_all, from, save_with_entries, read_to_string, write, from_str |
| `load` | 2651 | to_string_pretty, new, create_dir_all, save_with_entries, read_to_string, write, from_str |
| `save_with_entries` | 2663 | to_string_pretty, new, create_dir_all, write, save_with_entries |
| `save` | 2675 | save_with_entries, new |
| `record_connection` | 2680 | save_with_entries, new |
| `record_failure` | 2701 | save_with_entries |
| `annotate_identity` | 2709 | save_with_entries, new |
| `dialable_addresses` | 2761 | Reverse |
| `get_preferred_relays` | 2770 | Reverse |
| `all_known_topics` | 2783 | new |
| `summary` | 2791 | new |
| `default` | 2826 | new, new_multi_thread, new_current_thread |
| `get_global_runtime` | 2834 | new, new_multi_thread, new_current_thread |
| `new` | 2879 | from_str, new, dispatch_ble_packet, get_runtime_handle |
| `send_message` | 2893 | from_str, dispatch_ble_packet, get_runtime_handle |
| `send_message_status` | 2933 | from_str, dispatch_ble_packet, get_runtime_handle |
| `send_to_all_peers` | 2973 | from_str, dispatch_ble_packet, get_runtime_handle |
| `dial` | 3013 | from_str, new, get_runtime_handle |
| `get_peers` | 3048 | new, get_runtime_handle |
| `get_listeners` | 3062 | new, get_runtime_handle |
| `get_external_addresses` | 3076 | new, get_runtime_handle |
| `get_topics` | 3090 | new, get_runtime_handle |
| `subscribe_topic` | 3103 | get_runtime_handle |
| `unsubscribe_topic` | 3114 | dispatch_proximity_packet, get_runtime_handle |
| `publish_topic` | 3126 | dispatch_proximity_packet, get_runtime_handle |
| `shutdown` | 3138 | dispatch_proximity_packet, get_runtime_handle |
| `set_handle` | 3151 | dispatch_proximity_packet, new |
| `get_runtime_handle` | 3156 | dispatch_proximity_packet, new |
| `dispatch_ble_packet` | 3163 | dispatch_proximity_packet, new |
| `dispatch_proximity_packet` | 3168 | new, decode |
| `set_dispatch_ble_fn` | 3186 | new, decode |
| `set_dispatch_proximity_fn` | 3192 | new, decode |
| `get_escalation_engine` | 3202 | new, decode |
| `recommended_transport` | 3214 | decode |
| `update_peer_transports` | 3231 | now, safety_number, decode |
| `safety_number` | 3266 | now, safety_number, encode |
| `current_timestamp` | 3269 | now, encode |
| `make_state` | 3285 | new, with_storage, encode |
| `test_safety_number_returns_empty_string_on_malformed_keys` | 3303 | new, with_storage, encode |
| `test_safety_number_is_order_independent_for_valid_keys` | 3308 | new, with_storage, encode |
| `test_fresh_install_without_identity_resolves_headless_mode_with_persisted_key` | 3320 | new, new_platform_ref, with_storage, channel |
| `test_handle_data_path_confirmed_resolves_ipv4` | 3368 | new, new_platform_ref, channel |
| `test_handle_data_path_confirmed_resolves_ipv6_link_local` | 3386 | new, new_platform_ref, with_storage, channel |
| `test_handle_data_path_confirmed_ignores_unparseable_ip_without_panicking` | 3408 | new, new_platform_ref, with_storage, channel |
| `test_identity_creation_upgrades_resolved_mode_from_headless_to_full` | 3429 | new, with_storage |
| `test_connection_path_state_disconnected_by_default` | 3466 | new, compute_behavior |
| `test_compute_behavior_minimal_mode` | 3486 | compute_behavior |
| `test_compute_behavior_low_battery` | 3500 | compute_behavior, from_profile |
| `test_compute_behavior_stationary_good_battery` | 3514 | new, compute_behavior, from_profile |
| `test_compute_behavior_charging_always_full` | 3523 | new, compute_behavior, from_profile |
| `test_compute_behavior_normal_operation` | 3531 | new, compute_behavior, from_profile |
| `test_device_state_from_profile` | 3540 | new, from_profile |
| `test_update_device_state_stores_state` | 3557 | new |
| `test_update_device_state_transitions` | 3585 | new |
| `test_connection_path_state_disconnected_without_peers` | 3631 | from_str, new |
| `test_export_diagnostics_contains_state_fields` | 3643 | from_str, new, default |
| `test_get_swarm_bridge_initialization` | 3657 | new, default |
| `test_history_manager_persists_across_restart` | 3669 | new, default |
| `test_history_manager_recent_sorts_by_timestamp_not_key_order` | 3702 | new, default |
| `test_ledger_preferred_relays` | 3762 | new, sleep, default, from_millis |
| `test_mesh_settings_default` | 3789 | from_str, new, to_string, default |
| `message_status_monotone_progress` | 3806 | from_str, to_string, default, send_proximity_packet |
| `message_status_serialization_roundtrip` | 3829 | from_str, to_string, send_proximity_packet |
| `on_battery_changed` | 3849 | send_proximity_packet |
| `on_network_changed` | 3850 | send_proximity_packet |
| `on_motion_changed` | 3851 | send_proximity_packet |
| `on_ble_data_received` | 3852 | send_proximity_packet |
| `on_entering_background` | 3853 | send_proximity_packet |
| `on_entering_foreground` | 3854 | send_proximity_packet |
| `send_ble_packet` | 3855 | send_proximity_packet |
| `on_proximity_data_received` | 3858 |  |
| `send_proximity_packet` | 3865 |  |
| `wifi_aware_publish` | 3873 | new, default |
| `wifi_aware_subscribe` | 3876 | new, default |
| `wifi_aware_create_data_path` | 3879 | new, default |
| `wifi_aware_stop` | 3882 | new, default |
| `wifi_direct_discover_peers` | 3883 | new, default |
| `wifi_direct_stop_discovery` | 3886 | new, default |
| `wifi_direct_connect` | 3887 | new, default |
| `wifi_direct_create_group` | 3890 | new, default |
| `wifi_direct_remove_group` | 3893 | new, default |
| `test_mesh_service_config` | 3895 | new, default |
| `all_proximity_transports` | 3902 | new, default |
| `proximity_oversize_payload_rejected_per_transport_outbound` | 3916 | new, default |
| `proximity_oversize_payload_rejected_per_transport_inbound` | 3941 | new, default |
| `proximity_payload_at_exact_limit_is_forwarded` | 3961 | new, default |
| `proximity_round_trip_via_mock_bridge_ble_and_wifi_aware` | 3985 | new, default |
| `on_battery_changed` | 4028 |  |
| `on_network_changed` | 4031 |  |
| `on_motion_changed` | 4034 |  |
| `on_ble_data_received` | 4037 |  |
| `on_entering_background` | 4040 |  |
| `on_entering_foreground` | 4043 |  |
| `send_ble_packet` | 4046 |  |
| `on_proximity_data_received` | 4049 |  |
| `send_proximity_packet` | 4057 |  |
| `wifi_aware_publish` | 4065 |  |
| `wifi_aware_subscribe` | 4068 |  |
| `wifi_aware_create_data_path` | 4071 |  |
| `wifi_aware_stop` | 4074 |  |
| `wifi_direct_discover_peers` | 4077 |  |
| `wifi_direct_stop_discovery` | 4080 |  |
| `wifi_direct_connect` | 4083 |  |
| `wifi_direct_create_group` | 4086 |  |
| `wifi_direct_remove_group` | 4089 |  |

### Imports
- `use async_trait::async_trait`
- `use crate::settings::MeshSettings`
- `use crate::transport::SwarmHandle`
- `use crate::transport::wifi_direct::{PlatformWifiDirectBridge, WifiDirectTransport}`
- `use libp2p::{Multiaddr, PeerId}`
- `use parking_lot::{Mutex, RwLock}`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::{HashMap, HashSet}`
- `use std::fmt`
- `use std::os::unix::fs::PermissionsExt`
- `use std::str::FromStr`
- `use std::sync::Arc`
- `use super::*`
- `use tempfile::tempdir`
---

## AgentSwarmCline/scmessenger_swarm/observability.rs (1 chunks, 86 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
AgentSwarmCline/scmessenger_swarm/observability.rs: Defines 6 types: RelayTracePayload, fmt, RelayTracePayload, RelayTraceError, fmt; 3 functions; 1 imports

### Structs/Classes
- RelayTraceError
- RelayTracePayload
- fmt
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `fmt` | 30 |  |
| `validate` | 51 |  |
| `fmt` | 78 |  |

### Imports
- `use std::fmt`
---

## core/src/privacy/onion.rs (1 chunks, 508 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/privacy/onion.rs: Defines 4 types: OnionError, OnionLayer, OnionHeader, OnionEnvelope; 17 functions; 6 imports

### Structs/Classes
- OnionEnvelope
- OnionError
- OnionHeader
- OnionLayer

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `construct_onion` | 93 | thread_rng, new, TooManyHops, from, random_from_rng, from_slice |
| `peel_layer` | 235 | new, from, from_slice |
| `derive_layer_key` | 300 | derive_key, thread_rng, from, random_from_rng, from_slice |
| `derive_nonce` | 308 | random_from_rng, derive_key, thread_rng, from |
| `test_construct_onion_single_hop` | 322 | new, random_from_rng, thread_rng, from |
| `test_construct_onion_multiple_hops` | 337 | new, random_from_rng, thread_rng, from |
| `test_construct_onion_max_hops` | 361 | new, random_from_rng, thread_rng, from |
| `test_construct_onion_too_many_hops` | 375 | thread_rng, new, from, deserialize, random_from_rng, serialize |
| `test_construct_onion_empty_path` | 389 | thread_rng, from, deserialize, random_from_rng, serialize |
| `test_peel_layer_single_hop` | 395 | thread_rng, from, deserialize, random_from_rng, serialize |
| `test_onion_layer_serialization` | 414 | serialize, deserialize |
| `test_onion_envelope_serialization` | 433 | thread_rng, from, deserialize, random_from_rng, serialize |
| `test_key_derivation_deterministic` | 454 | random_from_rng, thread_rng, from |
| `test_key_derivation_different_secrets` | 463 | TooManyHops, random_from_rng, thread_rng, from |
| `test_onion_ephemeral_keys_unique` | 474 | TooManyHops, random_from_rng, thread_rng, from |
| `test_construct_onion_payload_preserved` | 489 | TooManyHops, random_from_rng, thread_rng, from |
| `test_onion_error_display` | 501 | TooManyHops |

### Imports
- `use chacha20poly1305::XChaCha20Poly1305`
- `use chacha20poly1305::aead::{Aead, KeyInit, Payload}`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
- `use x25519_dalek::{EphemeralSecret, PublicKey}`
---

## core/src/privacy/padding.rs (1 chunks, 320 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/privacy/padding.rs: Defines 2 types: PaddingScheme, PaddingError; 25 functions; 4 imports

### Structs/Classes
- PaddingError
- PaddingScheme

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `pad_message` | 46 | with_capacity, MessageTooLarge |
| `unpad_message` | 74 | InvalidConfig |
| `pad_to_next_standard_size` | 98 | thread_rng, InvalidConfig, MessageTooLarge, Random, Fixed |
| `apply_padding_scheme` | 121 | thread_rng, InvalidConfig, MessageTooLarge, Random, Fixed |
| `test_pad_message_exact_size` | 154 |  |
| `test_pad_message_minimum_size` | 166 |  |
| `test_pad_message_too_large` | 174 |  |
| `test_unpad_message_basic` | 181 |  |
| `test_unpad_message_no_padding` | 189 |  |
| `test_unpad_message_invalid_padding` | 196 |  |
| `test_unpad_message_invalid_trailing_bytes` | 204 | Fixed |
| `test_pad_to_next_standard_size_exact` | 212 | Fixed |
| `test_pad_to_next_standard_size_round_up` | 219 | Random, Fixed |
| `test_pad_to_next_standard_size_small` | 226 | Random, Fixed |
| `test_pad_to_next_standard_size_too_large` | 233 | Random, Fixed |
| `test_apply_padding_none` | 240 | Random, Fixed |
| `test_apply_padding_fixed` | 247 | Random, Fixed |
| `test_apply_padding_power_of_two` | 254 | Random |
| `test_apply_padding_random` | 261 | Random |
| `test_apply_padding_random_invalid` | 268 | Random |
| `test_round_trip_padding` | 275 |  |
| `test_padding_multiple_sizes` | 283 |  |
| `test_padding_empty_message` | 295 |  |
| `test_padding_with_embedded_marker` | 305 |  |
| `test_standard_sizes_constant` | 314 |  |

### Imports
- `use rand::Rng`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
---

## core/src/relay/peer_exchange.rs (1 chunks, 489 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/peer_exchange.rs: Defines 6 types: PeerExchangeError, RelayPeerInfo, RelayPeerInfo, PeerExchangeManager, PeerExchangeManager; 42 functions; 5 imports

### Structs/Classes
- Default
- PeerExchangeError
- PeerExchangeManager
- RelayPeerInfo

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 34 | now |
| `mark_seen` | 50 | now |
| `record_success` | 59 | new |
| `record_failure` | 64 | new |
| `to_message` | 69 | new |
| `from_message` | 80 | new |
| `new` | 103 | new |
| `with_config` | 112 | new, add_peer |
| `add_peer` | 121 | now, add_peer |
| `get_peer` | 138 | now, add_peer |
| `get_all_peers` | 143 | now, add_peer |
| `get_peers_by_reliability` | 148 | now, add_peer |
| `merge_peer_list` | 159 | now, add_peer, merge_peer_list |
| `prune_stale` | 166 | now, get_peers_by_reliability, merge_peer_list |
| `record_success` | 177 | new, get_peers_by_reliability, merge_peer_list |
| `record_failure` | 185 | new, get_peers_by_reliability, merge_peer_list |
| `peer_count` | 192 | new, get_peers_by_reliability, merge_peer_list, full_relay |
| `has_peer` | 197 | new, get_peers_by_reliability, merge_peer_list, full_relay |
| `clear` | 202 | new, get_peers_by_reliability, merge_peer_list, full_relay |
| `exchange_peers` | 207 | sleep, new, get_peers_by_reliability, full_relay, from_millis, merge_peer_list |
| `default` | 220 | new, sleep, from_millis, full_relay |
| `test_peer` | 232 | new, sleep, from_millis, full_relay |
| `test_relay_peer_info_creation` | 242 | sleep, from_millis |
| `test_mark_seen` | 250 | sleep, from_millis |
| `test_record_success` | 261 | new, from_message |
| `test_record_failure` | 271 | new, from_message |
| `test_score_bounds` | 281 | new, from_message |
| `test_peer_message_conversion` | 298 | new, from_message, with_config |
| `test_peer_exchange_manager_creation` | 310 | new, with_config |
| `test_add_peer` | 317 | new, with_config |
| `test_add_peer_duplicate` | 327 | new, with_config |
| `test_add_peer_capacity` | 341 | new, with_config |
| `test_get_peer` | 352 | new |
| `test_get_all_peers` | 364 | new |
| `test_get_peers_by_reliability` | 376 | new |
| `test_merge_peer_list` | 399 | new, sleep, with_config, from_millis |
| `test_record_success_2` | 411 | new, sleep, with_config, from_millis |
| `test_record_failure_2` | 423 | new, sleep, with_config, from_millis |
| `test_prune_stale` | 435 | from_millis, sleep, with_config, new |
| `test_clear` | 448 | new |
| `test_exchange_peers` | 461 | new |
| `test_exchange_peers_truncation` | 474 | new |

### Imports
- `use std::collections::HashMap`
- `use super::*`
- `use super::protocol::{RelayCapability, RelayPeerInfoMessage}`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/drift/policy.rs (1 chunks, 578 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/policy.rs: Defines 6 types: DeviceState, RelayProfile, PolicyError, PolicyEngine, PolicyEngine; 49 functions; 3 imports

### Structs/Classes
- Default
- DeviceState
- PolicyEngine
- PolicyError
- RelayProfile

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 51 | compute_profile |
| `update_device_state` | 61 | compute_profile, relay_budget_per_hour |
| `scan_interval_ms` | 67 | relay_budget_per_hour |
| `relay_budget_per_hour` | 82 | relay_budget_per_hour |
| `to_relay_config` | 97 | relay_budget_per_hour |
| `set_scan_interval_override` | 118 |  |
| `set_relay_budget_override` | 123 | new |
| `set_battery_floor_override` | 132 | new |
| `should_reduce` | 137 | new, now |
| `current_profile` | 145 | new, now |
| `compute_profile` | 150 | new, now |
| `default` | 171 | new, now |
| `make_device_state` | 179 | now, new |
| `test_profile_maximum` | 194 | new |
| `test_profile_high` | 203 | new |
| `test_profile_standard_high_battery` | 212 | new |
| `test_profile_standard_at_boundary` | 221 | new |
| `test_profile_reduced_mid_battery` | 230 | new |
| `test_profile_reduced_at_boundary_high` | 239 | new |
| `test_profile_reduced_at_boundary_low` | 248 | new |
| `test_profile_minimal_low_battery` | 257 | new |
| `test_profile_minimal_critical` | 266 | new |
| `test_scan_interval_maximum` | 275 | new |
| `test_scan_interval_high` | 284 | new |
| `test_scan_interval_standard` | 293 | new |
| `test_scan_interval_reduced` | 302 | new |
| `test_scan_interval_minimal` | 311 | new |
| `test_scan_interval_override` | 320 | new |
| `test_relay_budget_maximum` | 335 | new |
| `test_relay_budget_high` | 344 | new |
| `test_relay_budget_standard` | 353 | new |
| `test_relay_budget_reduced` | 362 | new |
| `test_relay_budget_minimal` | 371 | new |
| `test_relay_budget_override` | 380 | new |
| `test_coupling_relay_budget_cannot_be_zero` | 398 | new |
| `test_battery_floor_maximum` | 410 | new |
| `test_battery_floor_minimal` | 420 | new |
| `test_battery_floor_override` | 430 | new |
| `test_to_relay_config_maximum` | 446 | new |
| `test_to_relay_config_minimal` | 460 | new, default |
| `test_should_reduce_false_for_standard` | 471 | new, default |
| `test_should_reduce_false_for_maximum` | 480 | new, default |
| `test_should_reduce_true_for_reduced` | 489 | new, default |
| `test_should_reduce_true_for_minimal` | 498 | new, default |
| `test_default_policy_engine` | 507 | new, default |
| `test_current_profile_getter` | 513 | new |
| `test_profile_transitions` | 522 | new |
| `test_relay_config_has_correct_defaults` | 552 | new |
| `test_override_combinations` | 562 | new |

### Imports
- `use super::*`
- `use super::relay::RelayConfig`
- `use thiserror::Error`
---

## core/src/relay/protocol.rs (1 chunks, 379 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/protocol.rs: Defines 7 types: RelayCapability, RelayCapability, Default, RelayMessage, RelayPeerInfoMessage; 19 functions; 3 imports

### Structs/Classes
- Default
- RelayCapability
- RelayMessage
- RelayPeerInfoMessage
- RelayProtocolError

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `full_relay` | 21 | full_relay |
| `mobile` | 31 | full_relay |
| `is_relay` | 41 | full_relay |
| `is_store` | 46 | full_relay |
| `default` | 52 | full_relay |
| `to_bytes` | 166 | DeserializationError, full_relay, SerializationError, deserialize, serialize |
| `from_bytes` | 171 | mobile, DeserializationError, full_relay, deserialize |
| `message_type` | 177 | mobile, full_relay |
| `test_capability_full_relay` | 206 | mobile, from_bytes, full_relay |
| `test_capability_mobile` | 217 | mobile, from_bytes, full_relay |
| `test_relay_message_handshake_serialization` | 228 | from_bytes, full_relay |
| `test_relay_message_store_request_serialization` | 242 | from_bytes, full_relay |
| `test_relay_message_pull_request_serialization` | 260 | from_bytes, full_relay |
| `test_relay_message_peer_exchange_serialization` | 282 | from_bytes, full_relay |
| `test_relay_message_disconnect_serialization` | 306 | from_bytes |
| `test_relay_message_ping_pong` | 323 | from_bytes |
| `test_relay_message_store_ack` | 338 | from_bytes |
| `test_relay_message_pull_response` | 357 | from_bytes |
| `test_invalid_deserialization` | 374 | from_bytes |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
---

## core/src/drift/relay.rs (1 chunks, 810 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/relay.rs: Defines 11 types: NetworkState, RelayConfig, Default, RelayDecision, DropReason; 37 functions; 9 imports

### Structs/Classes
- Default
- DropReason
- From
- MaintenanceReport
- NetworkState
- RelayConfig
- RelayDecision
- RelayEngine
- RelayError
- SecurityAuditError

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 44 |  |
| `new` | 126 | now, hint_from_public_key, new, new_with_audit |
| `new_with_audit` | 131 | now, hint_from_public_key, new |
| `set_network_state` | 157 | new, with_capacity |
| `network_state` | 162 | new, with_capacity |
| `set_cover_traffic` | 170 | new, from_bytes, with_capacity |
| `set_reputation_manager` | 184 | from_bytes, with_capacity |
| `generate_cover_traffic_if_due` | 196 | from_bytes, with_capacity |
| `process_incoming` | 212 | from_bytes |
| `prepare_outgoing` | 344 | SerializationFailed |
| `messages_for_sync` | 356 | now |
| `messages_for_recipient` | 362 | now |
| `maintenance` | 367 | now |
| `apply_policy_config` | 384 | now, NotConfigured |
| `store` | 389 | now, NotConfigured |
| `store_mut` | 394 | now, NotConfigured |
| `check_rate_limit` | 399 | now, NotConfigured |
| `run_security_audit` | 416 | NotConfigured, Failed |
| `set_security_audit_pipeline` | 439 | now, Failed |
| `security_audit_pipeline` | 445 | now, Failed |
| `from` | 464 | now, new, hint_from_public_key, default, Failed |
| `make_test_envelope` | 473 | now, default, hint_from_public_key, new |
| `test_deliver_local` | 507 | new, default, hint_from_public_key |
| `test_store_and_relay` | 525 | new, default |
| `test_duplicate_detection` | 546 | new, default |
| `test_expired_message_dropped` | 574 | new, default |
| `test_max_hops_exceeded` | 593 | new, default |
| `test_low_priority_dropped` | 612 | new, default |
| `test_network_dormant_drop` | 635 | new, default |
| `test_rate_limiting` | 654 | new, default |
| `test_coupling_cannot_send_when_dormant` | 686 | new, default |
| `test_coupling_can_send_when_active` | 698 | new, default |
| `test_messages_for_sync` | 710 | new, default |
| `test_messages_for_recipient` | 726 | new, default, now |
| `test_maintenance_removes_expired` | 754 | new, default, now |
| `test_network_state_toggle` | 788 | new, default |
| `test_relay_config_default` | 802 | default |

### Imports
- `use crate::dspy::modules::{DSPyModule, OptimizerPipeline}`
- `use crate::privacy::cover::{CoverConfig, CoverTrafficScheduler}`
- `use std::sync::Arc`
- `use super::*`
- `use super::DriftError`
- `use super::envelope::DriftEnvelope`
- `use super::store::{MeshStore, MessageId, StoredEnvelope}`
- `use super::super::envelope::EnvelopeType`
- `use thiserror::Error`
---

## core/src/abuse/reputation.rs (1 chunks, 274 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/abuse/reputation.rs: Defines 4 types: EnhancedAbuseReputationManager, EnhancedAbuseReputationManager, EnhancedReputationScore, EnhancedReputationScore; 24 functions; 9 imports

### Structs/Classes
- EnhancedAbuseReputationManager
- EnhancedReputationScore

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 20 | with_backend, new |
| `with_backend` | 29 | with_backend |
| `apply_decay` | 41 |  |
| `flush_to_storage` | 46 |  |
| `record_signal` | 52 |  |
| `record_spam_signal` | 71 |  |
| `record_outbound_message` | 101 |  |
| `get_enhanced_score` | 112 |  |
| `get_score` | 124 |  |
| `rate_limit_multiplier` | 129 |  |
| `all_enhanced_scores` | 134 |  |
| `spam_detector` | 153 | new |
| `base_manager` | 158 | new, default |
| `overall_score` | 177 | new, default |
| `is_suspicious` | 183 | new, default |
| `is_abusive` | 188 | new, default |
| `make_manager` | 201 | new, default |
| `test_neutral_peer_has_neutral_score` | 212 |  |
| `test_positive_signals_increase_score` | 219 |  |
| `test_negative_signals_decrease_score` | 229 |  |
| `test_enhanced_score_combines_base_and_spam` | 239 |  |
| `test_spam_signal_recording` | 248 |  |
| `test_all_enhanced_scores` | 257 |  |
| `test_outbound_message_tracking` | 266 |  |

### Imports
- `use crate::abuse::spam_detection::SpamDetectionConfig`
- `use crate::abuse::spam_detection::{SpamDetectionEngine, SpamSignal}`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::blocked::BlockedManager`
- `use crate::store::contacts::ContactManager`
- `use crate::transport::reputation::{AbuseReputationManager, AbuseSignal, ReputationScore}`
- `use std::sync::Arc`
- `use super::*`
---

## core/src/transport/reputation.rs (1 chunks, 619 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/reputation.rs: Defines 8 types: ReputationScore, ReputationScore, std, AbuseSignal, PeerAbuseStats; 37 functions; 6 imports

### Structs/Classes
- AbuseReputationManager
- AbuseSignal
- PeerAbuseStats
- ReputationScore
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 26 |  |
| `neutral` | 30 |  |
| `value` | 34 |  |
| `is_trusted` | 38 |  |
| `is_suspicious` | 42 |  |
| `is_abusive` | 46 |  |
| `fmt` | 53 |  |
| `new` | 103 | now, neutral, calculate_score |
| `record_signal` | 122 | now, new, calculate_score |
| `calculate_score` | 146 | new |
| `rate_limit_multiplier` | 165 | new, from_secs |
| `new` | 195 | new, from_secs, now |
| `with_backend` | 206 | new, from_secs, now |
| `load_from_storage` | 218 | now, to_vec |
| `persist_peer` | 257 | new, to_vec |
| `remove_peer_from_storage` | 274 | new |
| `apply_decay` | 286 | persist_peer, new |
| `flush_to_storage` | 335 | persist_peer, new, remove_peer_from_storage |
| `record_signal` | 351 | persist_peer, new, remove_peer_from_storage |
| `get_score` | 378 | now, new, remove_peer_from_storage |
| `rate_limit_multiplier` | 387 | now, new, remove_peer_from_storage |
| `all_reputations` | 396 | now, new, remove_peer_from_storage |
| `prune_stale` | 406 | now, neutral, new, remove_peer_from_storage |
| `len` | 429 | now, new, neutral |
| `is_empty` | 434 | now, new, neutral |
| `current_epoch_secs` | 438 | now, new, neutral |
| `test_neutral_score` | 451 | new, neutral |
| `test_successful_delivery_increases_score` | 460 | new |
| `test_rate_limiting_decreases_score` | 470 | new, from_secs |
| `test_rate_limit_multiplier` | 480 | new, from_secs |
| `test_reputation_manager_eviction` | 495 | with_backend, new, from_secs |
| `test_prune_stale` | 508 | with_backend, new, from_secs |
| `test_mixed_signals` | 518 | with_backend, new |
| `test_persistence_roundtrip` | 532 | with_backend, new, from_utf8_lossy |
| `test_persistence_eviction_cleans_storage` | 558 | with_backend, new, from_utf8_lossy |
| `test_decay_moves_toward_neutral` | 580 | new |
| `test_epoch_secs_recorded` | 608 | new |

### Imports
- `use crate::store::backend::StorageBackend`
- `use parking_lot::RwLock`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH}`
- `use super::*`
---

## cli/src/server.rs (1 chunks, 1721 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/server.rs: Defines 8 types: UiEvent, UiOutbound, UiCommand, WebContext, Clone; 5 functions; 10 imports

### Structs/Classes
- Clone
- UiCommand
- UiEvent
- UiOutbound
- WebContext
- WsSender
- WsSenderList
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `clone` | 132 | clone, new |
| `fmt` | 148 | html, new, end |
| `start` | 169 | dir, html, new, ws, path, end |
| `handle_ws_connection` | 250 | JsonRpc, to_string, Lagged, text, Legacy, spawn |
| `handle_jsonrpc_request` | 339 | from_str, new, Object |

### Imports
- `use futures::StreamExt`
- `use futures_util::SinkExt`
- `use libp2p::PeerId`
- `use serde::{Deserialize, Serialize}`
- `use serde_json::{Map, Value}`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use std::time::Instant`
- `use tokio::sync::{broadcast, mpsc, Mutex}`
- `use warp::Filter`
---

## core/src/crypto/session_manager.rs (1 chunks, 544 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/crypto/session_manager.rs: Defines 6 types: RatchetSessionManager, Default, RatchetSessionManager, SerializableRatchetSession, ChainState; 20 functions; 13 imports

### Structs/Classes
- ChainState
- Default
- RatchetSessionManager
- SerializableRatchetSession

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 17 | new, from_utf8, deserialize_sessions, serialize_sessions |
| `new` | 23 | new, from_utf8, deserialize_sessions, serialize_sessions |
| `with_backend` | 31 | init_as_sender, from_utf8, new, deserialize_sessions, serialize_sessions |
| `save` | 39 | init_as_sender, deserialize_sessions, from_utf8, serialize_sessions |
| `load` | 53 | init_as_sender, deserialize_sessions, init_as_receiver, from_utf8 |
| `get_or_create_session` | 68 | init_as_sender, init_as_receiver |
| `create_receiver_session` | 87 | from_session, init_as_receiver, to_string |
| `get_session` | 104 | from_session, to_string, from_str |
| `get_session_mut` | 109 | from_session, to_string, from_str |
| `remove_session` | 114 | from_session, to_string, from_str |
| `session_count` | 119 | from_session, new, to_string, from_str |
| `has_session` | 124 | from_session, new, to_string, from_str |
| `serialize_sessions` | 129 | from_session, new, to_string, from_str |
| `deserialize_sessions` | 145 | from_str, new |
| `deserialize_sessions_strict` | 164 | from_str, new |
| `from_session` | 226 | decode, from, encode |
| `into_session` | 252 | from_bytes, decode_to_slice, decode, from, new_with_index |
| `generate_signing_key` | 345 | from_bytes, new, from, from_session, with_backend |
| `test_manager_persistence_roundtrip` | 353 | to_string, new, from, from_session, with_backend |
| `test_deserialize_sessions_strict_rejects_corrupted_entry` | 381 | from_session, new, to_string, from |

### Imports
- `use anyhow::{bail, Result}`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::backend::StorageBackend`
- `use ed25519_dalek::SigningKey`
- `use rand::RngCore`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use super::*`
- `use super::ratchet::{Chain, RatchetKey, RatchetSession}`
- `use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret}`
- `use zeroize::Zeroize`
---

## core/src/abuse/spam_detection.rs (1 chunks, 498 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/abuse/spam_detection.rs: Defines 8 types: SpamDetectionConfig, Default, SpamDetectionResult, PeerMessageTrack, PeerMessageTrack; 23 functions; 9 imports

### Structs/Classes
- Default
- PeerMessageTrack
- SpamDetectionConfig
- SpamDetectionEngine
- SpamDetectionResult
- SpamSignal

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 43 | new |
| `new` | 91 | new |
| `new` | 119 | new |
| `new_heuristics_only` | 135 | new |
| `content_fingerprint` | 147 | new |
| `detect_spam` | 157 | new, from_secs, now |
| `spam_score` | 268 | detect_spam |
| `record_spam_signal` | 279 | content_fingerprint |
| `record_outbound_message` | 304 | now, from_secs, content_fingerprint |
| `is_content_suspicious` | 350 | new_heuristics_only, new, default |
| `prune_stale_peers` | 355 | new_heuristics_only, new, default |
| `make_engine` | 389 | new_heuristics_only, new, default, content_fingerprint |
| `make_heuristics_only_engine` | 396 | new_heuristics_only, default, content_fingerprint |
| `test_default_config` | 402 | default, content_fingerprint |
| `test_no_contacts_is_not_spam` | 409 | content_fingerprint |
| `test_heuristics_only_no_contacts_is_not_spam` | 417 | content_fingerprint |
| `test_content_fingerprint_deterministic` | 425 | content_fingerprint |
| `test_record_spam_signal_accumulates` | 437 |  |
| `test_record_outbound_message` | 446 |  |
| `test_record_outbound_cold_contact` | 456 |  |
| `test_content_suspicious_length` | 465 |  |
| `test_prune_stale_peers` | 474 |  |
| `test_heuristics_only_flooding_detection` | 487 |  |

### Imports
- `use crate::store::backend::MemoryStorage`
- `use crate::store::blocked::BlockedManager`
- `use crate::store::contacts::ContactManager`
- `use parking_lot::RwLock`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use std::time::Instant`
- `use super::*`
---

## core/src/drift/store.rs (1 chunks, 821 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/store.rs: Defines 6 types: MessageId, StoredEnvelope, StoredEnvelope, MeshStore, MeshStore; 39 functions; 2 imports

### Structs/Classes
- Default
- MeshStore
- MessageId
- StoredEnvelope

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `priority_score` | 45 | now, new |
| `new` | 88 | new, evict_if_over_budget |
| `with_capacity` | 96 | new, evict_if_over_budget |
| `insert` | 107 | evict_if_over_budget |
| `merge` | 128 | evict_if_over_budget |
| `get` | 138 |  |
| `contains` | 143 | now |
| `message_ids` | 148 | now |
| `messages_for_recipient` | 153 | now |
| `by_priority` | 161 | now |
| `len` | 172 | now, message_ids, new, encode |
| `is_empty` | 177 | now, message_ids, new, encode |
| `remove` | 182 | now, message_ids, new, encode |
| `remove_expired` | 189 | now, message_ids, new, encode |
| `generate_proof` | 211 | new, message_ids, serialize, encode |
| `evict_if_over_budget` | 226 | insert, serialize |
| `save` | 247 | new, insert, serialize |
| `load` | 259 | new, insert |
| `default` | 287 | new, now |
| `make_envelope` | 295 | now, new |
| `test_insert_single_message` | 319 | new |
| `test_insert_duplicate_is_idempotent` | 330 | new |
| `test_get_message` | 345 | new |
| `test_merge_non_overlapping_stores` | 358 | new |
| `test_merge_overlapping_stores` | 379 | new |
| `test_merge_commutativity` | 401 | new |
| `test_merge_idempotency` | 432 | new, with_capacity, now |
| `test_eviction_on_over_capacity` | 454 | now, with_capacity |
| `test_priority_score_newer_higher` | 487 | now |
| `test_priority_score_fewer_hops_higher` | 521 | now |
| `test_priority_score_explicit_priority` | 553 | now, new |
| `test_remove_expired_messages` | 585 | new, now |
| `test_messages_for_recipient` | 644 | new, now |
| `test_by_priority_ordering` | 671 | new, now |
| `test_message_ids` | 723 | new, with_capacity, now |
| `test_empty_store` | 743 | new, with_capacity, now |
| `test_custom_capacity` | 750 | now, with_capacity |
| `test_merge_with_eviction` | 756 | now, with_capacity |
| `test_insert_after_eviction_preserves_crdt` | 788 | with_capacity |

### Imports
- `use std::collections::HashMap`
- `use super::*`
---

## core/src/drift/sync.rs (1 chunks, 711 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/sync.rs: Defines 8 types: VersionedSyncMessage, VersionedSyncMessage, SyncMessage, SyncMessage, SyncState; 32 functions; 8 imports

### Structs/Classes
- Default
- SyncMessage
- SyncSession
- SyncState
- VersionedSyncMessage

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 30 | DecompressionFailed, serialize, deserialize |
| `validate` | 38 | DecompressionFailed, serialize, deserialize |
| `to_bytes` | 49 | DecompressionFailed, serialize, deserialize |
| `from_bytes` | 54 | DecompressionFailed, serialize, deserialize |
| `to_bytes` | 94 | DecompressionFailed, serialize, deserialize |
| `from_bytes` | 99 | DecompressionFailed, deserialize |
| `frame_type` | 104 | new |
| `new` | 147 | new, DecompressionFailed |
| `state` | 159 | new, DecompressionFailed, now |
| `peer_missing_ids` | 164 | new, DecompressionFailed, now |
| `our_missing_ids` | 169 | new, DecompressionFailed, now |
| `initiate` | 174 | new, DecompressionFailed, now |
| `respond` | 217 | with_cells, from_bytes, DecompressionFailed |
| `complete` | 293 | from_bytes, DecompressionFailed, serialize |
| `default` | 368 | new |
| `merge_envelopes` | 374 | new |
| `make_test_envelope` | 388 | new |
| `make_test_id` | 401 | new |
| `test_sync_session_creation` | 407 | new |
| `test_sync_full_workflow_identical_stores` | 413 | new |
| `test_sync_full_workflow_disjoint_stores` | 441 | new |
| `test_sync_overlapping_stores` | 470 | new, from_bytes |
| `test_sync_message_serialization` | 504 | new, from_bytes |
| `test_sync_message_frame_types` | 534 | new |
| `test_merge_envelopes_into_store` | 559 | new |
| `test_merge_envelopes_idempotent` | 574 | new |
| `test_sync_session_state_transitions` | 591 | new, with_capacity |
| `test_sync_initiate_wrong_state_fails` | 603 | new, with_capacity |
| `test_sync_empty_stores` | 615 | new, with_capacity |
| `test_sync_large_symmetric_difference` | 633 | new, with_capacity |
| `test_sync_response_with_envelopes` | 659 | new |
| `test_proptest_sync_reconciles_arbitrary_sets` | 687 | new |

### Imports
- `use bincode`
- `use crate::drift::DriftError`
- `use crate::drift::StoredEnvelope`
- `use crate::error::MeshError`
- `use super::*`
- `use super::frame::FrameType`
- `use super::sketch::IBLT`
- `use super::store::{MeshStore, MessageId, StoredEnvelope}`
---

## core/src/privacy/timing.rs (1 chunks, 397 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/privacy/timing.rs: Defines 11 types: JitterDistribution, JitterConfig, Default, JitterConfig, MessagePriority; 27 functions; 4 imports

### Structs/Classes
- Default
- JitterConfig
- JitterDistribution
- JitterError
- MessagePriority
- RelayTimingPolicy
- TimingJitter

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 30 | InvalidConfig |
| `validate` | 41 | InvalidConfig |
| `jitter_config` | 69 |  |
| `default` | 102 | new |
| `new` | 124 | new, from_millis, thread_rng |
| `with_priority` | 130 | new, from_millis, thread_rng |
| `compute_jitter` | 135 | from_millis, default, thread_rng |
| `config` | 140 | from_millis, default, thread_rng |
| `compute_jitter` | 155 | from_millis, default, thread_rng |
| `test_jitter_config_default` | 181 | default |
| `test_jitter_config_validate_valid` | 189 |  |
| `test_jitter_config_validate_equal` | 199 | new, default |
| `test_jitter_config_validate_invalid_order` | 209 | new, default |
| `test_jitter_config_validate_zero_max` | 219 | new, default, with_priority |
| `test_message_priority_jitter_config` | 229 | new, default, with_priority |
| `test_timing_jitter_new` | 244 | new, default, with_priority |
| `test_timing_jitter_new_invalid_config` | 251 | new, with_priority |
| `test_timing_jitter_with_priority` | 262 | with_priority, from_millis |
| `test_compute_jitter_uniform` | 269 | from_millis |
| `test_compute_jitter_exponential` | 284 | from_millis, default |
| `test_compute_jitter_equal_bounds` | 299 | new, from_millis, default, serialize |
| `test_compute_jitter_zero_min` | 311 | new, default, serialize, deserialize |
| `test_relay_timing_policy_default` | 326 | new, default, serialize, deserialize |
| `test_timing_jitter_config_access` | 334 | new, serialize, deserialize |
| `test_jitter_distribution_serialization` | 347 | serialize, deserialize |
| `test_jitter_config_serialization` | 360 | serialize, deserialize |
| `test_exponential_distribution_bias` | 373 |  |

### Imports
- `use rand::Rng`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use web_time::Duration`
---

## core/src/message/types.rs (1 chunks, 277 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/message/types.rs: Defines 8 types: MessageType, DeliveryStatus, Message, Receipt, Envelope; 10 functions; 2 imports

### Structs/Classes
- DeliveryStatus
- Envelope
- Message
- MessageType
- Receipt
- SignedEnvelope

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `text` | 110 | now, serialize, from_utf8, new_v4 |
| `receipt` | 125 | now, serialize, from_utf8, new_v4 |
| `text_content` | 140 | now, from_utf8, text |
| `is_recent` | 149 | now, delivered, text |
| `delivered` | 160 | receipt, now, delivered, text |
| `test_create_text_message` | 177 | text, delivered, deserialize, receipt, serialize |
| `test_create_receipt` | 193 | text, delivered, deserialize, receipt, serialize |
| `test_receipt_message` | 200 | text, delivered, deserialize, receipt, serialize |
| `test_message_recency` | 209 | deserialize, serialize, text |
| `test_message_serialization` | 219 | deserialize, serialize, text |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
---

## core/src/wasm_support/mod.rs (1 chunks, 6 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/wasm_support/mod.rs: structural extraction

---

## wasm/src/lib.rs (1 chunks, 2363 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
wasm/src/lib.rs: Defines 30 types: DiscoveryMode, MeshSettings, Default, From, MeshSettingsManager; 145 functions; 9 imports

### Structs/Classes
- Default
- DiscoveryMode
- From
- IronCore
- IronCoreMode
- MeshSettings
- MeshSettingsManager
- WasmContactManager
- WasmHistoryManager
- WasmHistoryStats
- WasmIdentityInfo
- WasmMeshSettings
- WasmMessage
- WasmNotificationDecision
- WasmNotificationMessageContext
- WasmNotificationUiState
- WasmPreparedMessage
- WasmRegistrationStateInfo
- WasmSignatureResult

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 51 |  |
| `from` | 77 | from_str, read_to_string, from |
| `new` | 114 | to_string_pretty, validate, from, create_dir_all, read_to_string, default, write, from_str |
| `load` | 117 | to_string_pretty, validate, from, create_dir_all, read_to_string, default, write, from_str |
| `save` | 134 | to_string_pretty, new, validate, create_dir_all, from, write, set_as_global_default, set_once |
| `validate` | 153 | set_as_global_default, new, set_once |
| `init_logging` | 176 | set_as_global_default, new, set_once |
| `new` | 219 | new, default, with_storage |
| `with_storage` | 250 | new, default, with_storage |
| `with_storage` | 281 | new, default |
| `with_storage_async` | 312 | new, default, from, from_value |
| `start` | 333 | from_value, from |
| `stop` | 339 | from_value, from |
| `is_running` | 345 | from_value, from |
| `get_identity_info` | 350 | from_value, from |
| `set_iron_core_mode` | 359 | from_value |
| `get_iron_core_mode` | 368 |  |
| `set_daemon_socket_url` | 376 |  |
| `get_daemon_socket_url` | 382 |  |
| `initialize_identity` | 394 |  |
| `initialize_identity_from_daemon` | 421 |  |
| `get_identity_from_daemon` | 456 | to_value, from |
| `sign_data` | 479 | from |
| `verify_signature` | 487 |  |
| `prepare_message` | 504 |  |
| `receive_message` | 540 | clone |
| `outbox_count` | 557 | clone |
| `flush_outbox_for_peer` | 564 | clone |
| `inbox_count` | 569 | clone |
| `start_swarm` | 577 | clone |
| `stop_swarm` | 591 |  |
| `send_prepared_envelope` | 604 | to_value |
| `get_peers` | 629 | to_value |
| `get_external_addresses` | 648 | to_value |
| `get_connection_path_state` | 667 | new, is_running, inbox_count, get_connection_path_state, outbox_count |
| `export_diagnostics` | 693 | Object, new, now, is_running, inbox_count, get_connection_path_state, outbox_count |
| `subscribe_topic` | 728 |  |
| `unsubscribe_topic` | 746 |  |
| `publish_topic` | 764 |  |
| `dial` | 784 | new |
| `send_to_all_peers` | 804 | new |
| `get_listeners` | 844 | to_value |
| `get_nat_status` | 866 | to_value |
| `get_drift_state` | 871 | from_str, from_value, to_value, to_string |
| `get_drift_store_size` | 876 | from_str, from_value, to_value, to_string |
| `get_audit_log` | 881 | from_str, from_value, to_value, to_string |
| `get_audit_events_since` | 887 | from_str, from_value, to_value, to_string |
| `get_peer_reputation` | 893 | from_str, from_value, to_value, to_string |
| `get_enhanced_peer_reputation` | 898 | from_str, from_value, to_value, to_string |
| `get_overall_score` | 906 | from_str, from_value, to_value, to_string |
| `get_privacy_config` | 911 | from_str, from_value, to_value, to_string |
| `set_privacy_config` | 917 | from_str, from_value, to_string, clone |
| `validate_settings` | 930 | from_value, clone |
| `start_receive_loop` | 956 | clone, to_value, from, new |
| `drain_received_messages` | 982 | new, to_value, from_value, from |
| `get_settings` | 1000 | from_value, default, from |
| `update_settings` | 1008 | from_value, to_value, default, from |
| `get_default_settings` | 1033 | from_value, to_value, default, from |
| `classify_notification` | 1045 | from_value, to_value, from |
| `unregister_notification_endpoint` | 1064 |  |
| `touch_notification_endpoint` | 1072 |  |
| `set_nickname` | 1082 |  |
| `export_identity_backup` | 1090 |  |
| `import_identity_backup` | 1099 |  |
| `extract_public_key_from_peer_id` | 1111 |  |
| `prepare_message_with_id` | 1122 |  |
| `prepare_receipt` | 1147 |  |
| `prepare_cover_traffic` | 1162 |  |
| `mark_message_sent` | 1171 |  |
| `get_contact_manager` | 1176 |  |
| `get_history_manager` | 1183 |  |
| `contact_federated_nickname` | 1192 |  |
| `get_invite_signable_data` | 1200 |  |
| `resolve_identity` | 1211 |  |
| `resolve_to_identity_id` | 1219 |  |
| `block_peer` | 1229 | set, new, from_str |
| `unblock_peer` | 1237 | set, new, from_str |
| `block_and_delete_peer` | 1246 | set, new, from_f64, from_str |
| `is_peer_blocked` | 1258 | set, new, from_f64, from_str |
| `list_blocked_peers` | 1266 | set, from_bool, from_f64, new, from_str |
| `blocked_count` | 1319 |  |
| `get_device_id` | 1329 |  |
| `get_seniority_timestamp` | 1335 |  |
| `get_registration_state` | 1341 | set, new, from |
| `blake3_hash` | 1355 | set, new, from |
| `get_signature` | 1363 | set, new, from |
| `perform_maintenance` | 1371 | set, new, from |
| `update_disk_stats` | 1379 | set, new, from |
| `get_disk_stats` | 1385 | set, new, from |
| `record_log` | 1411 | from_value |
| `export_logs` | 1417 | from_value |
| `notify_peer_discovered` | 1427 | from_value, new |
| `notify_peer_disconnected` | 1433 | from_value, new |
| `add` | 1446 | from_value, new |
| `get` | 1455 | new |
| `remove` | 1464 | new |
| `list` | 1471 | new |
| `count` | 1484 | new |
| `set_local_nickname` | 1489 | new |
| `search` | 1501 | new |
| `set_nickname` | 1515 |  |
| `update_last_seen` | 1523 | from_value |
| `update_device_id` | 1531 | from_value |
| `flush` | 1543 | from_value, new |
| `add` | 1564 | from_value, new |
| `recent` | 1574 | new |
| `conversation` | 1591 | new |
| `clear` | 1604 |  |
| `stats` | 1611 | new |
| `count` | 1626 | new |
| `enforce_retention` | 1631 | new |
| `prune_before` | 1638 | new |
| `get` | 1646 | new |
| `search` | 1656 | from_str, new |
| `mark_delivered` | 1670 | from_str, new, to_value |
| `clear_conversation` | 1679 | from_str, new, to_value, from_value |
| `delete` | 1687 | from_str, new, to_value, from_value |
| `flush` | 1695 | from_str, new, to_value, from_value |
| `js_value_from_str` | 1699 | from_str, new, to_value, from_value |
| `to_js_value_safe` | 1714 | new, to_value, from_value |
| `parse_bootstrap_addrs` | 1717 | new, from_value |
| `relay_url_to_multiaddr` | 1726 |  |
| `ensure_mesh_participation_enabled` | 1779 | start_swarm_with_config, downgrade, default_routing_engine_handle, channel |
| `start_swarm_runtime` | 1788 | default_routing_engine_handle, start_swarm_with_config, downgrade, clone, channel, spawn_local |
| `resolve_swarm_keypair_and_mode` | 1904 | generate_ed25519 |
| `from` | 1936 |  |
| `from` | 1957 |  |
| `from` | 2016 |  |
| `from` | 2046 |  |
| `from` | 2091 |  |
| `from` | 2115 | new |
| `from` | 2135 | new, with_storage |
| `test_wasm_core_creation` | 2163 | new, with_storage |
| `test_wasm_identity` | 2172 | new, with_storage |
| `test_desktop_identity_flow_exposes_metadata_after_init` | 2181 | with_storage |
| `test_relay_url_to_multiaddr_ws_defaults` | 2214 | new |
| `test_relay_url_to_multiaddr_wss_defaults` | 2220 | new, with_storage |
| `test_relay_url_to_multiaddr_ipv4_port` | 2226 | new, with_storage |
| `test_relay_url_to_multiaddr_rejects_http` | 2232 | new, with_storage |
| `test_desktop_role_resolution_defaults_to_relay_only_without_identity` | 2238 | new, with_storage |
| `test_desktop_relay_only_flow_blocks_outbound_message_prepare` | 2256 | new, with_storage |
| `test_desktop_contacts_and_messaging_interaction_flow` | 2266 | new, with_storage |
| `test_desktop_mesh_dashboard_stats_update_with_message_flow` | 2307 | temp_dir, new, now, create_dir_all, with_storage, id |
| `test_notification_manager_creation` | 2344 | temp_dir, new, now, create_dir_all, id |
| `temp_storage_path` | 2348 | now, temp_dir, create_dir_all, id |

### Imports
- `use anyhow::Error`
- `use libp2p::{Multiaddr, PeerId}`
- `use scmessenger_core::store::{Contact, MessageDirection, MessageRecord}`
- `use serde_json::{Map, Value}`
- `use std::cell::RefCell`
- `use std::rc::Rc`
- `use super::*`
- `use wasm_bindgen::prelude::*`
- `use wasm_bindgen_test::*`
---

## cli/src/bootstrap.rs (1 chunks, 208 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/bootstrap.rs: 10 functions; 2 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default_bootstrap_nodes` | 36 | strip_peer_id, var |
| `promiscuous_bootstrap_addrs` | 83 | strip_peer_id, new |
| `parse_bootstrap_addr` | 95 | strip_peer_id, new |
| `merge_bootstrap_nodes` | 106 | strip_peer_id, new |
| `default_topics` | 130 |  |
| `test_default_bootstrap_nodes` | 139 |  |
| `test_promiscuous_addrs_strip_peer_id` | 153 |  |
| `test_parse_bootstrap_addr` | 170 |  |
| `test_merge_deduplicates_by_ip` | 186 |  |
| `test_default_topics` | 203 |  |

### Imports
- `use crate::ledger`
- `use super::*`
---

## mobile/src/lib.rs (1 chunks, 78 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
mobile/src/lib.rs: 4 functions; 2 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `test_mobile_bindings_lifecycle` | 12 | new |
| `test_mobile_identity` | 24 | new |
| `test_mobile_messaging` | 41 | new |
| `test_swarm_bridge_creation` | 66 | new |

### Imports
- `use scmessenger_core::IronCore`
- `use scmessenger_core::SwarmBridge`
---

## core/src/notification.rs (1 chunks, 660 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/notification.rs: Defines 14 types: NotificationKind, NotificationMessageContext, NotificationUiState, NotificationDecision, NotificationDecision; 29 functions; 6 imports

### Structs/Classes
- Default
- NotificationDecision
- NotificationEndpoint
- NotificationEndpointCapabilities
- NotificationEndpointError
- NotificationEndpointRegistry
- NotificationKind
- NotificationMessageContext
- NotificationPlatform
- NotificationUiState
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `suppressed` | 45 | suppressed |
| `allow` | 60 | suppressed |
| `classify_notification` | 77 | suppressed |
| `normalize_required` | 198 |  |
| `normalize_optional` | 202 |  |
| `ids_match` | 211 |  |
| `as_str` | 229 |  |
| `fmt` | 271 | new, StorageError |
| `new` | 294 | new |
| `register_endpoint` | 302 |  |
| `unregister_endpoint` | 340 |  |
| `list_endpoints` | 349 |  |
| `touch_endpoint` | 355 |  |
| `clear_all_request_notifications` | 372 |  |
| `clear_message_notifications` | 393 | new, now |
| `close_all_notifications` | 413 | new, default, now |
| `default` | 422 | new, default, now |
| `now_ms` | 426 | now, default |
| `base_message` | 437 | default |
| `base_ui_state` | 452 | default |
| `unknown_sender_defaults_to_direct_message_request` | 461 | default |
| `known_contact_defaults_to_direct_message` | 471 | default |
| `explicit_request_overrides_known_contact_inference` | 480 | default |
| `disabled_notifications_suppress_delivery` | 490 | default |
| `duplicates_are_suppressed` | 505 | new, default |
| `foreground_direct_messages_follow_foreground_toggle` | 518 | new, default |
| `clear_all_request_notifications_removes_request_only_endpoints` | 545 | new |
| `clear_message_notifications_clears_by_device_id` | 592 | new |
| `close_all_notifications_clears_entire_registry` | 631 | new |

### Imports
- `use crate::MeshSettings`
- `use parking_lot::Mutex`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use super::*`
---

## core/src/dspy/modules.rs (1 chunks, 332 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/dspy/modules.rs: Defines 26 types: DSPyModule, Input, Output, ModuleMetadata, ModuleMetadata; 30 functions; 2 imports

### Structs/Classes
- ChainOfThought
- DSPyError
- DSPyModule
- Input
- ModuleComplexity
- ModuleFactory
- ModuleMetadata
- MultiHopRecall
- OptimizerPipeline
- Output
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `execute` | 20 | ValidationError, ExecutionError, blake3_hash, new, OptimizerError |
| `validate_input` | 23 | ValidationError, ExecutionError, blake3_hash, new, OptimizerError |
| `get_metadata` | 26 | ValidationError, ExecutionError, blake3_hash, new, OptimizerError |
| `fingerprint` | 38 | ValidationError, ExecutionError, blake3_hash, new, OptimizerError |
| `fmt` | 66 | ValidationError, OptimizerError, ExecutionError |
| `new` | 88 |  |
| `add_step` | 98 | new |
| `execute` | 107 | new |
| `validate_input` | 113 | new |
| `get_metadata` | 117 | new |
| `new` | 138 | ValidationError, new, recall |
| `add_step` | 152 | ValidationError, recall |
| `recall` | 155 | ValidationError, recall |
| `execute` | 166 | ValidationError, recall |
| `validate_input` | 175 |  |
| `get_metadata` | 179 |  |
| `new` | 198 | new |
| `run_optimization` | 208 | new |
| `execute` | 219 | new |
| `validate_input` | 224 | new |
| `get_metadata` | 228 | new |
| `create_cot` | 242 | new |
| `create_multihop` | 245 | new |
| `create_optimizer` | 249 | new |
| `build_rust_feature_pipeline` | 255 | new, build_rust_feature_pipeline |
| `build_security_audit_pipeline` | 270 | new, build_rust_feature_pipeline |
| `test_chain_of_thought_module` | 289 | new, build_rust_feature_pipeline |
| `test_multihop_recall` | 296 | new, build_rust_feature_pipeline |
| `test_rust_feature_pipeline` | 303 | build_rust_feature_pipeline |
| `test_module_metadata_fingerprint` | 309 |  |

### Imports
- `use crate::dspy::signatures`
- `use super::*`
---

## core/src/dspy/signatures.rs (1 chunks, 222 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/dspy/signatures.rs: Defines 8 types: ArchitectSignature, ArchitectSignature, CoderSignature, CoderSignature, VerifierSignature; 15 functions; 8 imports

### Structs/Classes
- ArchitectSignature
- AuditorSignature
- CoderSignature
- VerifierSignature

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 21 |  |
| `new` | 39 |  |
| `new` | 57 | new, from |
| `new` | 75 | new, from |
| `generate_keypair` | 91 | new, from, hash |
| `encrypt_xchacha20` | 110 | new, hash |
| `blake3_hash` | 134 | new, hash |
| `signature_fingerprint` | 139 | from_str, new, to_string |
| `blake3_hash` | 147 | from_str, new, to_string |
| `get_signature` | 167 | from_str, new, to_string |
| `test_architect_signature_serialization` | 179 | from_str, new, to_string |
| `test_golden_examples_exist` | 194 |  |
| `test_blake3_hash_deterministic` | 201 |  |
| `test_blake3_hash_different_inputs` | 210 |  |
| `test_signature_fingerprint_format` | 217 |  |

### Imports
- `use blake3::hash`
- `use chacha20::ChaCha20`
- `use chacha20::cipher::{KeyIVInit, StreamCipher}`
- `use poly1305::Poly1305`
- `use ring::eddsa::KeyPair`
- `use ring::rand::SecureRandom`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
---

## core/src/abuse/mod.rs (1 chunks, 21 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/abuse/mod.rs: structural extraction

---

## core/src/bin/gen_kotlin.rs (1 chunks, 128 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/bin/gen_kotlin.rs: 2 functions; 3 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `main` | 2 | from, new, create_dir_all, var |
| `main` | 128 |  |

### Imports
- `use camino::Utf8PathBuf`
- `use std::fs`
- `use uniffi_bindgen::bindings::{generate, GenerateOptions, TargetLanguage}`
---

## core/src/crypto/kani_proofs.rs (1 chunks, 154 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/crypto/kani_proofs.rs: 8 functions; 2 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `ed25519_conversion_produces_32_bytes` | 80 | from_slice, from_bytes, derive_key, any, hash |
| `derive_key_always_32_bytes` | 89 | from_slice, from_bytes, derive_key, any, hash |
| `nonce_length_invariant` | 96 | hash, from_slice, from, from_bytes, any, derive_key |
| `chain_ratchet_produces_distinct_keys` | 103 | from, from_bytes, derive_key, any, hash |
| `ratchet_key_length_invariant` | 126 | any, from, from_bytes |
| `ed25519_signature_length_invariant` | 133 | any, from, from_bytes |
| `x25519_public_key_length_invariant` | 140 | any, from, from_bytes |
| `ed25519_verifying_key_length_invariant` | 147 | any, from_bytes |

### Imports
- `use crate::crypto::RatchetKey`
- `use crate::crypto::encrypt::{ed25519_to_x25519_secret, KDF_CONTEXT}`
---

## core/src/crypto/mod.rs (1 chunks, 26 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/crypto/mod.rs: structural extraction

---

## core/src/crypto/pq/mod.rs (1 chunks, 154 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/crypto/pq/mod.rs: Defines 3 types: MlKem768PrivateKey, MlKem768KeyPair, MlKem768KeyPair; 8 functions; 7 imports

### Structs/Classes
- MlKem768KeyPair
- MlKem768PrivateKey

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `public_key` | 23 | generate_key_pair, encapsulate, from |
| `private_key` | 28 | generate_key_pair, encapsulate, from |
| `generate` | 34 | generate_key_pair, encapsulate, from |
| `encapsulate` | 52 | decapsulate, from, encapsulate |
| `decapsulate` | 72 | decapsulate, from |
| `test_roundtrip` | 95 |  |
| `test_wrong_lengths` | 109 |  |
| `test_tampered_ciphertext` | 122 |  |

### Imports
- `use anyhow::{anyhow, Result}`
- `use libcrux_ml_kem::mlkem768`
- `use libcrux_ml_kem::{MlKemCiphertext, MlKemPrivateKey, MlKemPublicKey}`
- `use rand::RngCore`
- `use rand::rngs::OsRng`
- `use super::*`
- `use zeroize::Zeroize`
---

## core/src/crypto/proptest_harness.rs (1 chunks, 198 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/crypto/proptest_harness.rs: 13 functions; 6 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `generate_keypair` | 22 | from_bytes |
| `arb_plaintext` | 30 |  |
| `arb_nonempty_plaintext` | 34 |  |
| `proptest_encrypt_decrypt_roundtrip` | 41 |  |
| `proptest_different_ciphertexts_same_plaintext` | 57 |  |
| `proptest_wrong_key_fails` | 75 | from |
| `proptest_envelope_field_lengths` | 90 | from, init_as_sender, init_as_receiver |
| `proptest_sign_verify_roundtrip` | 104 | from, init_as_sender, init_as_receiver |
| `proptest_ratchet_roundtrip` | 118 | from, init_as_sender, init_as_receiver |
| `proptest_ratchet_forward_secrecy` | 152 | from, from_bytes, from_utf8_lossy, derive_key, new, init_as_sender |
| `chain_ratchet_produces_distinct_keys` | 173 | from_utf8_lossy, derive_key, new, from_bytes |
| `derive_key_always_32_bytes` | 184 | from_utf8_lossy, derive_key |
| `ed25519_conversion_produces_32_bytes` | 193 |  |

### Imports
- `use crate::crypto::ratchet::{Chain, RatchetKey}`
- `use ed25519_dalek::SigningKey`
- `use proptest::prelude::*`
- `use rand::RngCore`
- `use x25519_dalek::PublicKey as X25519PublicKey`
- `use zeroize::Zeroize`
---

## core/src/crypto/ratchet.rs (1 chunks, 774 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/crypto/ratchet.rs: Defines 7 types: RatchetKey, RatchetKey, std, Chain, RatchetSession; 22 functions; 7 imports

### Structs/Classes
- Chain
- RatchetEncryptResult
- RatchetKey
- RatchetSession
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `from_bytes` | 50 |  |
| `as_bytes` | 53 |  |
| `fmt` | 60 |  |
| `init_as_sender` | 198 | from, ed25519_to_x25519_secret, new, from_bytes |
| `init_as_receiver` | 240 | from, ed25519_to_x25519_secret, new, from_bytes |
| `our_public_key` | 279 | from_slice, new_from_slice |
| `dh_step_count` | 282 | from_slice, new_from_slice |
| `is_initialized` | 286 | from_slice, new_from_slice |
| `encrypt` | 292 | from_slice, new_from_slice |
| `decrypt` | 331 | new_from_slice, from_slice, from, handle_dh_ratchet, get_message_key |
| `handle_dh_ratchet` | 371 | from, new |
| `get_message_key` | 408 | from |
| `force_ratchet` | 448 | from, from_bytes, derive_key, hash, new |
| `derive_key_with_info` | 487 | from, ed25519_to_x25519_secret, from_bytes, derive_key, hash |
| `root_key_ratchet` | 493 | from, ed25519_to_x25519_secret, from_bytes, derive_key, new |
| `derive_root_key` | 503 | from, ed25519_to_x25519_secret, from_bytes, derive_key, new |
| `generate_keypair` | 516 | from, ed25519_to_x25519_secret, from_bytes, new, init_as_sender |
| `signing_key_to_x25519_public` | 524 | from, ed25519_to_x25519_secret, from_bytes, new, init_as_sender |
| `test_ratchet_key_zeroizes` | 531 | init_as_sender, new, from_bytes, init_as_receiver |
| `test_chain_ratchet_advances` | 539 | init_as_sender, new, from_bytes, init_as_receiver |
| `test_init_as_sender_and_encrypt` | 554 | init_as_sender, init_as_receiver |
| `test_init_as_receiver_then_decrypt` | 570 | init_as_sender, init_as_receiver |

### Imports
- `use anyhow::{bail, Result}`
- `use ed25519_dalek::SigningKey`
- `use rand::RngCore`
- `use std::collections::HashMap`
- `use super::*`
- `use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret}`
- `use zeroize::Zeroize`
---

## core/src/drift/mod.rs (1 chunks, 96 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/mod.rs: Defines 1 types: DriftError; 2 functions; 2 imports

### Structs/Classes
- DriftError

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `read_frame_with_timeout` | 35 | read_with_timeout |
| `test_drift_version_constant` | 93 |  |

### Imports
- `use super::*`
- `use thiserror::Error`
---

## core/src/drift/rate_limit.rs (1 chunks, 261 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/rate_limit.rs: Defines 2 types: SyncRateLimiter, SyncRateLimiter; 16 functions; 4 imports

### Structs/Classes
- SyncRateLimiter

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 38 | now, from_secs, new |
| `allow_sync` | 64 | now, from_secs, new |
| `cleanup_expired` | 87 | now, from_secs, new |
| `tracked_peer_count` | 96 | from_secs, new |
| `peer_sync_count` | 101 | from_secs, new |
| `test_rate_limiter_creation` | 112 | from_secs, new, from_millis |
| `test_allow_sync_under_limit` | 118 | from_secs, new, from_millis |
| `test_deny_sync_over_limit` | 130 | from_secs, sleep, new, from_millis |
| `test_different_peers_independent` | 144 | from_secs, sleep, new, from_millis |
| `test_window_expiry` | 160 | from_secs, sleep, new, from_millis |
| `test_cleanup_expired` | 176 | from_secs, sleep, new, from_millis |
| `test_cleanup_preserves_active` | 194 | from_secs, sleep, new, from_millis |
| `test_sliding_window` | 206 | from_secs, sleep, new, from_millis |
| `test_peer_sync_count` | 227 | from_secs, new |
| `test_zero_limit` | 241 | from_secs, new |
| `test_large_limit` | 250 | from_secs, new |

### Imports
- `use std::collections::HashMap`
- `use std::thread`
- `use std::time::{Duration, Instant}`
- `use super::*`
---

## core/src/drift/sketch.rs (1 chunks, 564 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/drift/sketch.rs: Defines 4 types: IBLTCell, IBLTCell, IBLT, IBLT; 32 functions; 3 imports

### Structs/Classes
- IBLT
- IBLTCell

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 36 | new, xor_in, from_le_bytes |
| `is_pure` | 45 | new, xor_in, from_le_bytes |
| `is_empty` | 50 | new, xor_in, from_le_bytes |
| `xor_in` | 55 | new, xor_in, from_le_bytes |
| `xor_out` | 64 | hash_to_cell, new, xor_in, from_le_bytes |
| `new` | 78 | hash_to_cell, new, from_le_bytes |
| `with_cells` | 85 | hash_to_cell, with_cells, new, from_le_bytes |
| `hash_to_cell` | 95 | hash_to_cell, with_cells, new, from_le_bytes |
| `insert` | 108 | new, with_cells, hash_to_cell |
| `remove` | 117 | new, DecompressionFailed, with_cells, hash_to_cell |
| `subtract` | 127 | DecompressionFailed, with_cells, new |
| `decode` | 150 | DecompressionFailed, new, from_le_bytes |
| `to_bytes` | 223 | from_le_bytes, with_capacity |
| `from_bytes` | 244 | with_capacity, from_le_bytes |
| `serialized_size` | 289 | new |
| `cell_count` | 294 | new |
| `make_test_id` | 302 | new |
| `test_iblt_insert_decode_single_element` | 308 | new |
| `test_iblt_symmetric_difference` | 324 | new |
| `test_iblt_empty_sets` | 349 | new, from_bytes |
| `test_iblt_identical_sets` | 361 | new, from_bytes |
| `test_iblt_serialization_roundtrip` | 378 | new, from_bytes |
| `test_iblt_asymmetric_differences` | 402 | new |
| `test_iblt_size_calculation` | 426 | new |
| `test_iblt_single_difference` | 433 | new |
| `test_iblt_multiple_differences` | 453 | new |
| `test_iblt_remove_operation` | 484 | new |
| `test_iblt_oversized_differences_fail_decode` | 498 | new, from_bytes |
| `test_iblt_commutativity_with_swap` | 516 | new, from_bytes |
| `test_iblt_from_bytes_invalid_length` | 542 | new, from_bytes |
| `test_iblt_from_bytes_empty` | 551 | new, from_bytes |
| `test_iblt_subtract_mismatched_cells` | 557 | new |

### Imports
- `use blake3`
- `use crate::drift::{DriftError, MessageId}`
- `use super::*`
---

## core/src/dspy/mod.rs (1 chunks, 21 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/dspy/mod.rs: structural extraction

---

## core/src/dspy/teleprompt.rs (1 chunks, 320 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/dspy/teleprompt.rs: Defines 11 types: Teleprompter, TeleprompterError, std, std, OptimizationStats; 26 functions; 3 imports

### Structs/Classes
- BasicTeleprompter
- Default
- OptimizationStats
- Teleprompter
- TeleprompterError
- TeleprompterFactory
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `compile` | 16 | OptimizationError, CompilationError, ValidationError |
| `optimize` | 19 | OptimizationError, CompilationError, new, ValidationError |
| `get_stats` | 22 | OptimizationError, CompilationError, new, ValidationError |
| `reset` | 25 | ValidationError, OptimizationError, CompilationError, new, default |
| `fmt` | 36 | ValidationError, OptimizationError, CompilationError, new, default |
| `default` | 67 | build_architect_prompt, ValidationError, build_verifier_prompt, build_coder_prompt, build_auditor_prompt, new, default |
| `new` | 73 | build_architect_prompt, ValidationError, build_verifier_prompt, build_coder_prompt, build_auditor_prompt, new, default |
| `add_golden_examples` | 82 | build_architect_prompt, ValidationError, build_verifier_prompt, build_coder_prompt, build_auditor_prompt |
| `golden_examples_fingerprint` | 89 | build_architect_prompt, ValidationError, build_verifier_prompt, build_coder_prompt, build_auditor_prompt |
| `compile_for_signature` | 100 | build_architect_prompt, ValidationError, build_verifier_prompt, build_coder_prompt, build_auditor_prompt |
| `build_architect_prompt` | 123 |  |
| `build_coder_prompt` | 141 |  |
| `build_verifier_prompt` | 159 | compile_for_signature |
| `build_auditor_prompt` | 179 | compile_for_signature, default |
| `compile` | 203 | compile_for_signature, new, default |
| `optimize` | 210 | new, default |
| `get_stats` | 217 | new, ValidationError, default |
| `reset` | 221 | build_rust_feature_pipeline, new, ValidationError, default |
| `create_basic` | 237 | new, build_rust_feature_pipeline, ValidationError |
| `create_for_scenario` | 240 | new, build_rust_feature_pipeline, ValidationError |
| `build_optimization_pipeline` | 270 | create_for_scenario, new, build_rust_feature_pipeline |
| `test_basic_teleprompter_compilation` | 280 | create_for_scenario, new |
| `test_signature_type_validation` | 289 | create_for_scenario, new |
| `test_optimization_stats` | 298 | create_for_scenario, new |
| `test_scenario_based_teleprompter` | 305 | new, create_for_scenario |
| `test_golden_examples_fingerprint` | 311 | new |

### Imports
- `use crate::dspy::modules::{ModuleFactory, OptimizerPipeline}`
- `use crate::dspy::signatures::blake3_hash`
- `use super::*`
---

## core/src/error.rs (1 chunks, 334 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/error.rs: Defines 6 types: MeshError, TransportError, SerializationError, MeshResult, TransportResult; 5 functions; 4 imports

### Structs/Classes
- MeshError
- MeshResult
- SerializationError
- SerializationResult
- TransportError
- TransportResult

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `test_mesh_error_display` | 296 | NoiseHandshake, new |
| `test_transport_error_from_io` | 308 | NoiseHandshake, new |
| `test_serialization_error_from_json` | 315 | NoiseHandshake |
| `test_mesh_error_from_transport` | 322 | NoiseHandshake |
| `test_mesh_error_from_serialization` | 329 |  |

### Imports
- `use crate::drift::relay::NetworkState`
- `use crate::routing::local::PeerId`
- `use super::*`
- `use thiserror::Error`
---

## core/src/identity/mod.rs (1 chunks, 423 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/identity/mod.rs: Defines 3 types: IdentityManager, IdentityManager, Default; 32 functions; 7 imports

### Structs/Classes
- Default
- IdentityManager

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 23 | memory, persistent, ensure_device_metadata |
| `with_backend` | 33 | ensure_device_metadata, persistent |
| `hydrate_from_store` | 48 | hydrate_from_store, ensure_device_metadata, generate |
| `ensure_device_metadata` | 74 | hydrate_from_store, ensure_device_metadata, generate |
| `initialize` | 83 | hydrate_from_store, ensure_device_metadata, generate, verify |
| `keys` | 102 | verify |
| `public_key_hex` | 107 | verify |
| `identity_id` | 112 | from_bytes, verify |
| `sign` | 117 | ensure_device_metadata, from_bytes, verify |
| `verify` | 125 | ensure_device_metadata, from_bytes, verify |
| `set_nickname` | 130 | ensure_device_metadata, from_bytes |
| `nickname` | 137 | ensure_device_metadata, from_bytes |
| `device_id` | 142 | ensure_device_metadata, from_bytes |
| `seniority_timestamp` | 149 | ensure_device_metadata, from_bytes |
| `export_key_bytes` | 156 | ensure_device_metadata, from_bytes |
| `import_key_bytes` | 161 | ensure_device_metadata, new, from_bytes |
| `get_dspy_signature` | 177 | new |
| `signature_fingerprint` | 189 | new, parse_str |
| `blake3_hash` | 201 | new, parse_str |
| `default` | 207 | new, parse_str |
| `test_identity_manager_creation` | 217 | new, parse_str |
| `test_identity_initialization` | 223 | new, parse_str |
| `test_identity_signing` | 237 | new, with_backend |
| `test_identity_verification` | 248 | new, with_backend |
| `test_identity_persistence` | 269 | new, with_backend |
| `test_identity_import_export_roundtrip` | 305 | new, with_backend |
| `test_with_path_hydrates_existing_identity_without_initialize` | 323 | new, with_backend |
| `test_get_dspy_signature` | 350 | new |
| `test_get_dspy_signature_content` | 364 | new |
| `test_signature_fingerprint` | 376 | new |
| `test_blake3_hash` | 389 | new |
| `test_identity_signature_integration` | 405 | new |

### Imports
- `use anyhow::Result`
- `use crate::dspy::signatures::{blake3_hash, get_signature, signature_fingerprint}`
- `use crate::store::backend::StorageBackend`
- `use std::sync::Arc`
- `use super::*`
- `use tempfile::tempdir`
---

## core/src/identity/store.rs (1 chunks, 431 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/identity/store.rs: Defines 4 types: DeviceMetadata, DeviceMetadata, IdentityStore, IdentityStore; 18 functions; 11 imports

### Structs/Classes
- DeviceMetadata
- IdentityStore

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `generate` | 22 | now, new_v4, Persistent |
| `memory` | 41 | Persistent |
| `persistent` | 46 | Persistent |
| `save_keys` | 51 | Persistent |
| `save_nickname` | 70 | Persistent, from_bytes |
| `save_device_metadata` | 92 | load_device_id, Persistent, from_bytes, parse_str, load_seniority_timestamp |
| `load_keys` | 111 | load_device_id, Persistent, from_bytes, parse_str, load_seniority_timestamp |
| `load_device_metadata` | 129 | load_device_id, Persistent, save_device_metadata, generate, load_device_metadata, parse_str, load_seniority_timestamp |
| `load_or_create_device_metadata` | 162 | Persistent, from_utf8, save_device_metadata, generate, load_device_metadata |
| `load_nickname` | 173 | from_utf8, Persistent |
| `load_device_id` | 202 | from_utf8, Persistent |
| `load_seniority_timestamp` | 216 | from_utf8, Persistent, memory |
| `clear` | 238 | Persistent, persistent, generate, memory, new |
| `test_memory_store` | 265 | persistent, memory, generate, new |
| `test_persistent_store` | 277 | persistent, generate, new |
| `test_store_clear` | 297 | persistent, sleep, from_millis, generate, new |
| `test_store_persistence_across_instances` | 313 | persistent, sleep, from_millis, generate, new |
| `test_device_metadata_persistence_across_instances` | 352 | persistent, sleep, from_millis, parse_str, new |

### Imports
- `use anyhow::Result`
- `use crate::store::backend::StorageBackend`
- `use std::sync::Arc`
- `use std::thread`
- `use super::*`
- `use super::IdentityKeys`
- `use tempfile::tempdir`
- `use uuid::Uuid`
- `use web_time::Duration`
- `use web_time::{SystemTime, UNIX_EPOCH}`
- `use zeroize::Zeroize`
---

## core/src/lib.rs (1 chunks, 146 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/lib.rs: Defines 7 types: IronCoreError, IdentityInfo, SignatureResult, PreparedMessage, PeelResult

### Structs/Classes
- IdentityInfo
- IronCoreError
- MessageRequest
- PeelResult
- PreparedMessage
- RegistrationStateInfo
- SignatureResult

---

## core/src/message/mod.rs (1 chunks, 15 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/message/mod.rs: structural extraction

---

## core/src/notification_defaults.rs (1 chunks, 27 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/notification_defaults.rs: 7 functions

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `notifications_enabled` | 1 |  |
| `notify_dm_enabled` | 4 |  |
| `notify_dm_request_enabled` | 8 |  |
| `notify_dm_in_foreground` | 12 |  |
| `notify_dm_request_in_foreground` | 16 |  |
| `sound_enabled` | 20 |  |
| `badge_enabled` | 24 |  |

---

## core/src/observability.rs (1 chunks, 506 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/observability.rs: Defines 9 types: AuditEventType, fmt, AuditEvent, fmt, AuditEvent; 17 functions; 7 imports

### Structs/Classes
- AuditEvent
- AuditEventType
- AuditLog
- AuditLogError
- Default
- fmt

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `fmt` | 50 |  |
| `fmt` | 89 | now, hash, new_v4, to_string |
| `new` | 100 | hash, to_string, new_v4, now, new |
| `chain_hash` | 125 | hash, new, to_string |
| `default` | 146 | from_utf8_lossy, new, to_string |
| `new` | 153 | from_utf8_lossy, new, to_string |
| `persist` | 165 | from_utf8_lossy, new, to_string |
| `load` | 174 | from_utf8_lossy, new |
| `prune_before` | 193 |  |
| `append` | 239 | new |
| `validate_chain` | 261 |  |
| `test_event_creation` | 321 | now, new, parse_str |
| `test_chain_hash_determinism` | 351 | new |
| `test_valid_chain` | 376 | new |
| `test_tampered_chain_detection` | 401 | new |
| `test_empty_log_validation` | 443 | new |
| `test_multi_event_chaining` | 452 | new |

### Imports
- `use crate::store::backend::StorageBackend`
- `use serde::{Deserialize, Serialize}`
- `use std::fmt`
- `use std::sync::Arc`
- `use super::*`
- `use thiserror::Error`
- `use uuid::Uuid`
---

## core/src/privacy/mod.rs (1 chunks, 69 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/privacy/mod.rs: Defines 3 types: PrivacyConfig, Default, PrivacyConfig; 2 functions; 1 imports

### Structs/Classes
- Default
- PrivacyConfig

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 45 | default |
| `full` | 59 | default |

### Imports
- `use serde::{Deserialize, Serialize}`
---

## core/src/relay/mod.rs (1 chunks, 37 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/mod.rs: structural extraction

---

## core/src/relay/server.rs (1 chunks, 553 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/server.rs: Defines 10 types: RelayServerConfig, Default, StoredEnvelope, ConnectionState, RelayServerStats; 29 functions; 8 imports

### Structs/Classes
- ConnectionState
- Default
- RelayPeerSession
- RelayServer
- RelayServerConfig
- RelayServerError
- RelayServerStats
- StoredEnvelope

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 25 |  |
| `new` | 115 | with_config, new, default |
| `with_config` | 120 | new |
| `register_peer` | 135 | now |
| `complete_handshake` | 166 | now |
| `store_for_peer` | 178 | now |
| `get_stored_for` | 215 | now, new |
| `remove_peer` | 244 | now |
| `cleanup_expired` | 253 | now, full_relay |
| `get_stats` | 275 | full_relay, new |
| `is_peer_connected` | 280 | full_relay, new |
| `stored_count_for_peer` | 288 | full_relay, new |
| `create_handshake_ack` | 294 | full_relay, new |
| `add_bytes_relayed` | 303 | with_config, full_relay, new, default |
| `default` | 310 | with_config, full_relay, new, default |
| `test_server` | 322 | with_config, full_relay, new, default |
| `test_server_creation` | 328 | with_config, full_relay, default |
| `test_register_peer` | 336 | with_config, full_relay, default |
| `test_connection_limit` | 348 | with_config, full_relay, default |
| `test_store_and_retrieve` | 372 | with_config, default |
| `test_storage_limit_per_peer` | 397 | sleep, from_millis, with_config, full_relay, default |
| `test_cleanup_expired` | 415 | sleep, from_millis, with_config, full_relay, default |
| `test_remove_peer` | 439 | full_relay |
| `test_complete_handshake` | 457 | full_relay |
| `test_is_peer_connected` | 475 | full_relay |
| `test_stored_count_for_peer` | 490 |  |
| `test_create_handshake_ack` | 503 | now |
| `test_stats_tracking` | 522 | now |
| `test_retrieve_with_timestamp_filter` | 535 | now |

### Imports
- `use parking_lot::RwLock`
- `use std::collections::{HashMap, VecDeque}`
- `use std::net::SocketAddr`
- `use std::sync::Arc`
- `use super::*`
- `use super::protocol::{RelayCapability, RelayMessage, PROTOCOL_VERSION}`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/routing/mod.rs (1 chunks, 43 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/routing/mod.rs: structural extraction

---

## core/src/store/mod.rs (1 chunks, 31 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/store/mod.rs: structural extraction

---

## core/src/transport/ble/mod.rs (1 chunks, 38 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/ble/mod.rs: structural extraction

---

## core/src/transport/mod.rs (1 chunks, 82 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/mod.rs: structural extraction

---

## core/src/transport/routing/mod.rs (1 chunks, 6 lines)
Function `PQC_07_PQ_RATCHET` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/routing/mod.rs: structural extraction

---
