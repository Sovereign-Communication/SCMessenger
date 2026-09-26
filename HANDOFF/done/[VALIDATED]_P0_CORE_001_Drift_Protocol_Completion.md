# MODEL: glm-5.1:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 3600

# P0_CORE_001: Drift Protocol Completion

**Priority:** P0 (Critical Performance)
**Platform:** Core/Rust
**Status:** COMPLETE
**Source:** Completed task audit - Drift Protocol not wired to production

## Problem Description
The Drift Protocol (8 files, all unit-tested) was COMPLETELY DORMANT and not wired to any production path. Current implementation used legacy bincode format instead of optimized Drift format.

## Missing Integration Points - ALL RESOLVED
1. [DONE] **Message send/receive path wrapping** - Now using DriftEnvelope serialization with bincode fallback
2. [DONE] **SyncSession triggering** - Already active on PeerDiscovered events (verified)
3. [DONE] **PolicyEngine integration** - Wired to IronCore with `update_device_state()` method
4. [DONE] **Drift compression** - LZ4 compression active via COMPRESSION_THRESHOLD in prepare_message path
5. [DONE] **DriftFrame transport wrapping** - All send/receive paths wrap in DriftFrame::Data

## Implementation Completed

### 1. Message Codec (`core/src/message/codec.rs`)
- `encode_envelope()` now produces DriftEnvelope binary format (primary path)
- `decode_envelope()` tries DriftEnvelope first, falls back to bincode for backward compatibility
- New function `encode_drift_envelope()` handles legacy-to-Drift conversion
- Compression threshold (256 bytes) automatically activates LZ4 for large payloads

### 2. Prepare Message Path (`core/src/iron_core.rs`)
- `prepare_message_internal()` now uses `DriftEnvelope::from_legacy_envelope()` with proper signing
- Ed25519 signature covers all envelope fields
- Recipient hint derived from blake3 hash for efficient routing
- TTL and hop count properly initialized

### 3. Transport Layer (`core/src/transport/swarm.rs`)
- `wrap_in_drift_frame()` helper wraps all outgoing data in DriftFrame::Data with CRC32
- Receive path unwraps DriftFrame via `DriftFrame::from_bytes()` with legacy fallback
- All send points (direct, relay, peer broadcast, WASM) use DriftFrame wrapping
- Removed unused `get_drift_frame_type()` function

### 4. PolicyEngine Wiring (`core/src/iron_core.rs`)
- Added `policy_engine: Arc<RwLock<PolicyEngine>>` field to IronCore struct
- `update_device_state(battery, charging, wifi, moving)` method computes profile and propagates config
- `get_policy_relay_config()` and `current_relay_profile()` for diagnostics
- Policy changes automatically propagated to RelayEngine via `drift_apply_policy()`

### 5. SyncSession (Already Active)
- Verified: SyncSession::new() is inserted on every PeerDiscovered event
- No additional wiring needed

## Performance Impact
- Fixed 187-byte envelope overhead (vs variable bincode)
- LZ4 compression active for payloads > 256 bytes
- CRC32 integrity verification on transport frames
- Adaptive relay policies based on battery/charging/wifi/motion state

## Verification Results
- `cargo check -p scmessenger-core` - PASS
- `cargo test -p scmessenger-core --lib -- message::codec` - 10/10 PASS
- `cargo test -p scmessenger-core --lib -- drift` - 152/153 PASS (1 pre-existing proptest failure unrelated to changes)
- `cargo test -p scmessenger-core --lib -- drift::envelope` - 19/19 PASS
- `cargo test -p scmessenger-core --lib -- drift::policy` - All PASS

## Key Files Modified
- `core/src/message/codec.rs` - Drift encoding integration with bincode fallback
- `core/src/iron_core.rs` - PolicyEngine wiring, prepare_message Drift path
- `core/src/transport/swarm.rs` - DriftFrame wrapping/unwrapping, PolicyEngine imports

## Cross-Platform Consistency
- All paths (Desktop, Android, WASM) use the same codec
- Transport wrapping applies to all send points including WASM
- Bincode fallback ensures backward compatibility with older nodes