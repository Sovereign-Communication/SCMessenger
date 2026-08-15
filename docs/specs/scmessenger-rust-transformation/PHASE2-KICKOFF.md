# Phase 2 Kickoff: Protocol Hardening - Sync Auth, Versioning, Rate Limits
## SCMessenger Rust Transformation

---

## Context

You are continuing the SCMessenger Rust Transformation project. **Phase 1 (Async Hygiene) is complete**. You are now starting **Phase 2: Protocol Hardening - Sync Auth, Versioning, Rate Limits**.

### Project Overview
- **Project**: SCMessenger Rust Transformation
- **Spec Location**: `docs/specs/scmessenger-rust-transformation/`
- **Current Phase**: Phase 2 (Protocol Hardening)
- **Previous Phase**: Phase 1 (Async Hygiene) - [OK] COMPLETE

### Phase 1 Completion Status
[OK] **WASM locks optimized**: Arc<Mutex> → Rc<RefCell> (~700 LoC)  
[OK] **CLI async locks verified**: tokio::sync::Mutex usage correct  
[OK] **HTTP API modernized**: Hyper 0.14 → Axum 0.7 (~400 LoC)  
[OK] **All tests passing**: cargo check/clippy/fmt all pass  
[OK] **Zero regressions**: All 13 API endpoints functional  

---

## Phase 2 Objective

**Add cryptographic authentication, schema versioning, and rate limiting to the sync protocol.**

### Goals
1. Add schema versioning to network messages
2. Implement cryptographic peer proofs for sync messages
3. Add rate limiting to prevent sync flooding
4. Verify all protocol changes maintain backward compatibility

---

## Phase 2 Tasks

### Task 2.1: Add Schema Versioning to Network Messages (~100 LoC)

**Objective**: Add version field to SyncMessage to enable protocol evolution

**Current Pattern** (verify first):
```rust
// core/src/drift/sync.rs
#[derive(Serialize, Deserialize)]
pub enum SyncMessage {
    SyncOffer { iblt_data: Vec<u8> },
    SyncResponse { missing_ids: Vec<String> },
    // ...
}
```

**Target Pattern**:
```rust
// core/src/drift/sync.rs
pub const SYNC_SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct VersionedSyncMessage {
    pub schema_version: u32,
    pub payload: SyncMessage,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    SyncOffer { 
        iblt_data: Vec<u8>,
        peer_proof: String,      // Added in Task 2.2
        timestamp: u64,
    },
    SyncResponse { 
        missing_ids: Vec<String> 
    },
    // ...
}

impl VersionedSyncMessage {
    pub fn new(payload: SyncMessage) -> Self {
        Self {
            schema_version: SYNC_SCHEMA_VERSION,
            payload,
        }
    }
    
    pub fn validate(&self) -> Result<(), MeshError> {
        if self.schema_version != SYNC_SCHEMA_VERSION {
            return Err(MeshError::VersionMismatch {
                expected: SYNC_SCHEMA_VERSION,
                received: self.schema_version,
            });
        }
        Ok(())
    }
}
```

**Actions**:
1. Read `core/src/drift/sync.rs` to verify current SyncMessage structure
2. Add `SYNC_SCHEMA_VERSION` constant (value: 1)
3. Create `VersionedSyncMessage` struct with schema_version and payload fields
4. Update `SyncMessage` enum with `#[serde(tag = "type")]` attribute
5. Add `peer_proof` and `timestamp` fields to `SyncOffer` variant (for Task 2.2)
6. Implement `VersionedSyncMessage::new()` method
7. Implement `VersionedSyncMessage::validate()` method
8. Update all SyncMessage creation sites to use VersionedSyncMessage
9. Update all SyncMessage deserialization sites to validate version
10. Add unit tests for version mismatch handling
11. Run `cargo test` in core/

**Verification**:
- [ ] cargo test passes
- [ ] Version mismatch produces MeshError::VersionMismatch
- [ ] All sync messages include schema_version field

---

### Task 2.2: Add Cryptographic Peer Proofs (~100 LoC)

**Objective**: Add cryptographic proofs to sync offers to prevent spoofing

**Current Pattern** (after Task 2.1):
```rust
SyncMessage::SyncOffer { 
    iblt_data: vec![...],
    peer_proof: String::new(),  // Empty for now
    timestamp: 0,
}
```

**Target Pattern**:
```rust
// core/src/drift/sync.rs
impl MeshState {
    pub fn generate_proof(&self) -> String {
        // Deterministic hash of mesh state
        let mut hasher = blake3::Hasher::new();
        
        // Hash peer ID
        hasher.update(self.peer_id.as_bytes());
        
        // Hash message IDs in sorted order
        let mut ids: Vec<_> = self.messages.keys().collect();
        ids.sort();
        for id in ids {
            hasher.update(id.as_bytes());
        }
        
        hex::encode(hasher.finalize().as_bytes())
    }
    
    pub fn verify_proof(&self, proof: &str) -> bool {
        self.generate_proof() == proof
    }
}

// When creating sync offer:
let proof = mesh_state.generate_proof();
let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();

SyncMessage::SyncOffer {
    iblt_data,
    peer_proof: proof,
    timestamp,
}

// When receiving sync offer:
if !mesh_state.verify_proof(&peer_proof) {
    return Err(MeshError::Auth("Invalid peer proof".to_string()));
}
```

**Actions**:
1. Add `blake3` dependency to core/Cargo.toml if not present
2. Read `core/src/drift/sync.rs` to find MeshState definition
3. Implement `MeshState::generate_proof()` method with deterministic hashing
4. Implement `MeshState::verify_proof()` method
5. Update sync offer creation to include peer_proof
6. Update sync offer handling to validate peer_proof
7. Add unit tests for proof generation determinism
8. Add unit tests for proof validation
9. Run `cargo test` in core/

**Verification**:
- [ ] cargo test passes
- [ ] Proof generation is deterministic (same state → same proof)
- [ ] Invalid proofs are rejected
- [ ] Valid proofs are accepted

---

### Task 2.3: Implement Rate Limiting for Sync Initiations (~100 LoC)

**Objective**: Prevent sync flooding by rate limiting sync requests per peer

**Target Pattern**:
```rust
// core/src/drift/rate_limit.rs (new file)
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct SyncRateLimiter {
    limits: HashMap<String, Vec<Instant>>,
    window: Duration,
    max_per_window: usize,
}

impl SyncRateLimiter {
    pub fn new(window: Duration, max_per_window: usize) -> Self {
        Self {
            limits: HashMap::new(),
            window,
            max_per_window,
        }
    }
    
    pub fn allow_sync(&mut self, peer_id: &str) -> bool {
        let now = Instant::now();
        
        // Get or create entry for peer
        let timestamps = self.limits.entry(peer_id.to_string()).or_insert_with(Vec::new);
        
        // Remove expired timestamps
        timestamps.retain(|&t| now.duration_since(t) < self.window);
        
        // Check if under limit
        if timestamps.len() >= self.max_per_window {
            return false;
        }
        
        // Record this sync
        timestamps.push(now);
        true
    }
    
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.limits.retain(|_, timestamps| {
            timestamps.retain(|&t| now.duration_since(t) < self.window);
            !timestamps.is_empty()
        });
    }
}

// In sync protocol handler:
if !rate_limiter.allow_sync(&peer_id) {
    return Err(MeshError::RateLimited {
        peer_id: peer_id.to_string(),
    });
}
```

**Actions**:
1. Create `core/src/drift/rate_limit.rs`
2. Implement `SyncRateLimiter` struct with limits, window, max_per_window fields
3. Implement `SyncRateLimiter::new()` constructor
4. Implement `SyncRateLimiter::allow_sync()` method
5. Implement `SyncRateLimiter::cleanup_expired()` method
6. Add `RateLimited` variant to MeshError enum
7. Integrate rate limiter into sync protocol handler
8. Add unit tests for rate limiting behavior
9. Add unit tests for cleanup preventing unbounded growth
10. Run `cargo test` in core/

**Verification**:
- [ ] cargo test passes
- [ ] Sync requests under limit are allowed
- [ ] Sync requests over limit are denied
- [ ] Expired entries are cleaned up
- [ ] Memory usage is bounded

---

### Task 2.4: Phase 2 Verification Gate (~10 LoC)

**Objective**: Verify all Phase 2 changes maintain zero regression

**Actions**:
1. Run `cargo check --workspace`
2. Run `cargo test --workspace`
3. Run `cargo clippy --workspace`
4. Run `cargo fmt --check`
5. Verify version mismatch produces correct error
6. Verify proof validation works correctly
7. Verify rate limiting denies excessive requests
8. Run smoke test

**Verification**:
- [ ] cargo check passes
- [ ] cargo test passes
- [ ] cargo clippy passes
- [ ] cargo fmt --check passes
- [ ] Version mismatch error correct
- [ ] Proof validation functional
- [ ] Rate limiting functional
- [ ] Smoke test passes

---

## Critical Files to Review

### Before Starting
1. **core/src/drift/sync.rs** - Current sync protocol implementation
2. **core/src/error.rs** - Error types (add VersionMismatch, RateLimited)
3. **core/Cargo.toml** - Check for blake3 dependency

### Implementation Prompt Reference
- **HANDOFF/scmessenger_rust_implementation_prompt.md** - Detailed Phase 2 instructions

---

## Important Constraints

### Zero-Regression Requirement
- All existing functionality must be preserved
- All tests must continue to pass
- No public APIs should be deleted

### Verification Before Changes
- Always read actual code before making changes
- Verify dependency versions in Cargo.toml
- Check exact struct/enum definitions

### Protocol Compatibility
- Schema versioning must be backward compatible
- Peer proofs must not break existing sync
- Rate limiting must not affect normal operation

---

## Success Criteria

Phase 2 is complete when:
- [OK] Schema versioning added to all sync messages
- [OK] Cryptographic peer proofs implemented
- [OK] Rate limiting prevents sync flooding
- [OK] All tests pass (cargo test)
- [OK] All crates compile (cargo check)
- [OK] Clippy passes
- [OK] Code formatted (cargo fmt)
- [OK] Version mismatch handled correctly
- [OK] Proof validation works
- [OK] Rate limiting works

---

## Getting Started

### Step 1: Verify Phase 1 Completion
```bash
cargo check --workspace
cargo test --lib -p scmessenger-core
```

### Step 2: Start Task 2.1 (Schema Versioning)
```bash
# Read current sync protocol
cat core/src/drift/sync.rs

# Check for existing version handling
rg "version" core/src/drift/
```

### Step 3: Update Task Status
Use the task tracking system to mark tasks as in_progress/completed:
- Task file: `docs/specs/scmessenger-rust-transformation/tasks.md`
- Update status as you complete each task

---

## Emergency Halt Conditions

Stop and request human review if:
1. Compilation fails and fix not obvious within 15 minutes
2. Public API changes require updates in >2 files
3. Dependency conflict arises
4. Test coverage drops below pre-transformation levels
5. Protocol changes break backward compatibility

---

## Quick Reference

### Current State (Post-Phase 1)
- **Rust version**: 1.95.0
- **Build status**: [OK] Passing
- **Test status**: [OK] All tests passing
- **HTTP API**: [OK] Axum 0.7
- **WASM locks**: [OK] Rc<RefCell>
- **CLI locks**: [OK] tokio::sync::Mutex

### Phase 2 Focus
- **Schema versioning**: Enable protocol evolution
- **Peer proofs**: Prevent sync spoofing
- **Rate limiting**: Prevent sync flooding
- **Zero regression**: All tests must pass

---

## Estimated Work

**Total LoC:** ~300 LoC
- Task 2.1: ~100 LoC (schema versioning)
- Task 2.2: ~100 LoC (peer proofs)
- Task 2.3: ~100 LoC (rate limiting)
- Task 2.4: ~10 LoC (verification)

**Estimated Time:** 2-3 hours

---

## Prompt to Use in New Window

```
I'm continuing the SCMessenger Rust Transformation project. Phase 1 (Async Hygiene) is complete. 

Please execute Phase 2: Protocol Hardening - Sync Auth, Versioning, Rate Limits.

**Context:**
- Spec location: docs/specs/scmessenger-rust-transformation/
- Phase 1 status: [OK] COMPLETE (locks optimized, HTTP API modernized)
- Current Rust version: 1.95.0
- Build status: All tests passing

**Phase 2 Objectives:**
1. Add schema versioning to sync messages
2. Implement cryptographic peer proofs
3. Add rate limiting for sync initiations
4. Verify zero regression

**Instructions:**
- Read docs/specs/scmessenger-rust-transformation/PHASE2-KICKOFF.md for detailed instructions
- Follow tasks in docs/specs/scmessenger-rust-transformation/tasks.md (Phase 2 section)
- Reference HANDOFF/scmessenger_rust_implementation_prompt.md for implementation details
- Update task status as you progress
- Maintain zero-regression (all tests must pass)

Please start with Task 2.1: Add Schema Versioning to Network Messages.
```

---

**Phase 2 Status: [PENDING] READY TO START**
