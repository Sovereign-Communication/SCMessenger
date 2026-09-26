# BATCH: Rust Group G  miscellaneous core wiring (2 tasks)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

# AGENT: rust-coder
# MODEL: glm-5.1:cloud
# FALLBACK: qwen3-coder-next:cloud
# TARGET FILES: core/src/abuse/reputation.rs, core/src/dspy/signatures.rs

1. **overall_score**  Wire into abuse reputation scoring aggregation and routing negative cache decisions.
2. **get_signature**  Wire into crypto signature verification and identity authentication flows (already in Group B but duplicated here for separate dspy/signatures.rs targeting).

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.

# REPO_MAP Context for Task: BATCH_RUST_GROUPG_MISC_CORE

**Target function: `BATCH_RUST_GROUPG_MISC_CORE`**

## core/src/abuse/auto_block.rs (1 chunks, 332 lines)
Function `BATCH_RUST_GROUPG_MISC_CORE` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/abuse/auto_block.rs: Defines 7 types: AutoBlockConfig, Default, AutoBlockReason, AutoBlockAuditEntry, AutoBlockResult; 19 functions; 11 imports

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
| `default` | 35 | new |
| `new` | 97 | new |
| `evaluate` | 112 | evaluate |
| `evaluate_and_block` | 161 | evaluate, new |
| `exempt_peer` | 195 | evaluate_and_block |
| `unexempt_peer` | 200 | evaluate_and_block, now |
| `is_exempt` | 205 | evaluate_and_block, now |
| `audit_log` | 214 | default, new, evaluate_and_block, now |
| `update_config` | 219 | default, new, evaluate_and_block, now |
| `config` | 224 | default, new, evaluate_and_block, now |
| `evaluate_all_tracked` | 230 | default, new, evaluate_and_block, now |
| `current_epoch_secs` | 243 | default, new, now |
| `make_engine` | 258 | default, new |
| `test_default_config` | 273 | default |
| `test_exempt_peer_not_blocked` | 281 | default |
| `test_unexempt_peer` | 290 | default |
| `test_audit_log_records_block` | 298 | default |
| `test_disabled_auto_block` | 316 | default |
| `test_neutral_peer_not_blocked` | 326 |  |

### Imports
- `use crate::abuse::reputation::EnhancedAbuseReputationManager`
- `use crate::abuse::spam_detection::{SpamDetectionConfig, SpamDetectionEngine}`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::blocked::BlockedManager`
- `use crate::store::contacts::ContactManager`
- `use crate::transport::reputation::ReputationScore`
- `use parking_lot::RwLock`
- `use serde::{Deserialize, Serialize}`
- `use std::sync::Arc`
- `use std::time::SystemTime`
- `use super::*`
---

## core/src/transport/reputation.rs (1 chunks, 619 lines)
Function `BATCH_RUST_GROUPG_MISC_CORE` not found in REPO_MAP chunks. Full file listing below.

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
| `new` | 103 | neutral, calculate_score, now |
| `record_signal` | 122 | new, calculate_score, now |
| `calculate_score` | 146 | new |
| `rate_limit_multiplier` | 165 | from_secs, new |
| `new` | 195 | from_secs, new, now |
| `with_backend` | 206 | from_secs, new, now |
| `load_from_storage` | 218 | to_vec, now |
| `persist_peer` | 257 | new, to_vec |
| `remove_peer_from_storage` | 274 | new |
| `apply_decay` | 286 | persist_peer, new |
| `flush_to_storage` | 335 | persist_peer, remove_peer_from_storage, new |
| `record_signal` | 351 | persist_peer, remove_peer_from_storage, new |
| `get_score` | 378 | remove_peer_from_storage, new, now |
| `rate_limit_multiplier` | 387 | remove_peer_from_storage, new, now |
| `all_reputations` | 396 | remove_peer_from_storage, new, now |
| `prune_stale` | 406 | remove_peer_from_storage, new, now, neutral |
| `len` | 429 | neutral, new, now |
| `is_empty` | 434 | neutral, new, now |
| `current_epoch_secs` | 438 | neutral, new, now |
| `test_neutral_score` | 451 | new, neutral |
| `test_successful_delivery_increases_score` | 460 | new |
| `test_rate_limiting_decreases_score` | 470 | from_secs, new |
| `test_rate_limit_multiplier` | 480 | from_secs, new |
| `test_reputation_manager_eviction` | 495 | from_secs, new, with_backend |
| `test_prune_stale` | 508 | from_secs, new, with_backend |
| `test_mixed_signals` | 518 | new, with_backend |
| `test_persistence_roundtrip` | 532 | from_utf8_lossy, new, with_backend |
| `test_persistence_eviction_cleans_storage` | 558 | from_utf8_lossy, new, with_backend |
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

## core/src/abuse/spam_detection.rs (1 chunks, 498 lines)
Function `BATCH_RUST_GROUPG_MISC_CORE` not found in REPO_MAP chunks. Full file listing below.

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
| `detect_spam` | 157 | from_secs, new, now |
| `spam_score` | 268 | detect_spam |
| `record_spam_signal` | 279 | content_fingerprint |
| `record_outbound_message` | 304 | content_fingerprint, from_secs, now |
| `is_content_suspicious` | 350 | default, new_heuristics_only, new |
| `prune_stale_peers` | 355 | default, new_heuristics_only, new |
| `make_engine` | 389 | default, content_fingerprint, new_heuristics_only, new |
| `make_heuristics_only_engine` | 396 | default, content_fingerprint, new_heuristics_only |
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

## core/src/dspy/signatures.rs (1 chunks, 222 lines)
Function `BATCH_RUST_GROUPG_MISC_CORE` not found in REPO_MAP chunks. Full file listing below.

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
| `new` | 57 | from, new |
| `new` | 75 | from, new |
| `generate_keypair` | 91 | from, new, hash |
| `encrypt_xchacha20` | 110 | new, hash |
| `blake3_hash` | 134 | new, hash |
| `signature_fingerprint` | 139 | new, to_string, from_str |
| `blake3_hash` | 147 | new, to_string, from_str |
| `get_signature` | 167 | new, to_string, from_str |
| `test_architect_signature_serialization` | 179 | new, to_string, from_str |
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

## core/src/abuse/reputation.rs (1 chunks, 274 lines)
Function `BATCH_RUST_GROUPG_MISC_CORE` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/abuse/reputation.rs: Defines 4 types: EnhancedAbuseReputationManager, EnhancedAbuseReputationManager, EnhancedReputationScore, EnhancedReputationScore; 24 functions; 9 imports

### Structs/Classes
- EnhancedAbuseReputationManager
- EnhancedReputationScore

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 20 | new, with_backend |
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

## core/src/dspy/modules.rs (1 chunks, 317 lines)
Function `BATCH_RUST_GROUPG_MISC_CORE` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/dspy/modules.rs: Defines 26 types: DSPyModule, Input, Output, ModuleMetadata, ModuleMetadata; 29 functions; 2 imports

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
| `execute` | 20 | ExecutionError, ValidationError, OptimizerError, blake3_hash, new |
| `validate_input` | 23 | ExecutionError, ValidationError, OptimizerError, blake3_hash, new |
| `get_metadata` | 26 | ExecutionError, ValidationError, OptimizerError, blake3_hash, new |
| `fingerprint` | 38 | ExecutionError, ValidationError, OptimizerError, blake3_hash, new |
| `fmt` | 66 | ValidationError, ExecutionError, OptimizerError |
| `new` | 88 |  |
| `add_step` | 98 |  |
| `execute` | 107 |  |
| `validate_input` | 113 | ValidationError, recall |
| `get_metadata` | 117 | ValidationError, recall |
| `new` | 134 | ValidationError, recall |
| `recall` | 144 | ValidationError, recall |
| `execute` | 155 | ValidationError, recall |
| `validate_input` | 162 |  |
| `get_metadata` | 166 |  |
| `new` | 183 | new |
| `run_optimization` | 193 | new |
| `execute` | 204 | new |
| `validate_input` | 209 | new |
| `get_metadata` | 213 | new |
| `create_cot` | 227 | new |
| `create_multihop` | 230 | new |
| `create_optimizer` | 234 | new |
| `build_rust_feature_pipeline` | 240 | build_rust_feature_pipeline, new |
| `build_security_audit_pipeline` | 255 | build_rust_feature_pipeline, new |
| `test_chain_of_thought_module` | 274 | build_rust_feature_pipeline, new |
| `test_multihop_recall` | 281 | build_rust_feature_pipeline, new |
| `test_rust_feature_pipeline` | 288 | build_rust_feature_pipeline |
| `test_module_metadata_fingerprint` | 294 |  |

### Imports
- `use crate::dspy::signatures`
- `use super::*`
---
