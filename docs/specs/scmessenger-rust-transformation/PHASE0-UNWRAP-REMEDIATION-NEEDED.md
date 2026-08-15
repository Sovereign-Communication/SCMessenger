# Phase 0 Unwrap Remediation - Scope Document

## Status: NOT STARTED (Marked Complete But Not Executed)

Phase 0 was marked as complete in the task list, but the core objective—eliminating `.unwrap()` calls—was never executed.

## Current State

### Clippy Configuration

The project has `.clippy.toml` configured to disallow unwrap:

```toml
disallowed-methods = [
    { path = "std::option::Option::unwrap", reason = "Use ? or expect() with context instead" },
    { path = "std::result::Result::unwrap", reason = "Use ? or expect() with context instead" },
]
```

### Current Violations

**Total: ~917 unwrap() calls across 100+ files**

**Breakdown:**
- ~779 `Option::unwrap()` calls
- ~138 `Result::unwrap()` calls

**Top Offenders:**
| File | Warnings |
|------|----------|
| `core/tests/integration_*.rs` | 49 |
| `core/src/transport/swarm.rs` | 46 |
| `core/src/store/relay_custody.rs` | 40 |
| `core/src/identity/mod.rs` | 39 |
| `core/src/mobile_bridge.rs` | 39 |
| `core/src/store/outbox.rs` | 35 |
| `wasm/src/daemon_bridge.rs` | 31 |
| `core/src/identity/storage.rs` | 28 |
| `cli/src/server.rs` | 27 |
| `core/src/drift/sketch.rs` | 26 |
| `core/src/drift/relay.rs` | 26 |
| `core/src/relay/client.rs` | 26 |
| `core/src/transport/health.rs` | 25 |
| `core/src/drift/frame.rs` | 22 |
| `wasm/src/lib.rs` | 22 |
| ...and 85+ more files | 500+ |

## Why This Matters

`.unwrap()` calls cause panics when they encounter `None` or `Err` values. In production:

1. **Crashes**: Panics terminate the thread/process
2. **Poor UX**: No graceful error messages
3. **Debugging**: Stack traces instead of structured errors
4. **Recovery**: No opportunity to handle errors gracefully

## Scope of Work

### Phase 0.1: Error Type Design

**Create comprehensive error hierarchy:**

```rust
// core/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum MeshError {
    #[error("Version mismatch: expected {expected}, got {received}")]
    VersionMismatch { expected: u32, received: u32 },
    
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    
    #[error("Relay denied: {reason}")]
    RelayDenied { reason: String },
    
    #[error("Storage quota exceeded")]
    StorageQuota,
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),
    
    #[error("Authentication failed: {0}")]
    Auth(String),
    
    #[error("IBLT decode failed: {0}")]
    IbltDecode(String),
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Noise handshake failed: {0}")]
    NoiseHandshake(String),
    
    #[error("Connection reset")]
    ConnectionReset,
    
    #[error("Timeout")]
    Timeout,
}

#[derive(Debug, thiserror::Error)]
pub enum SerializationError {
    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}
```

### Phase 0.2: Systematic Unwrap Removal

**For each file with unwrap() calls:**

1. **Identify context**: What operation is being unwrapped?
2. **Choose error type**: Which error variant applies?
3. **Update signature**: Change return type to `Result<T, MeshError>`
4. **Replace unwrap**: Use `?` operator or `map_err()`
5. **Update callers**: Propagate errors up the call chain
6. **Add tests**: Verify error paths work correctly

**Example transformation:**

```rust
// BEFORE
pub fn process_message(data: &[u8]) -> Message {
    let envelope = bincode::deserialize(data).unwrap();
    let decrypted = decrypt(&envelope).unwrap();
    Message::parse(&decrypted).unwrap()
}

// AFTER
pub fn process_message(data: &[u8]) -> Result<Message, MeshError> {
    let envelope = bincode::deserialize(data)
        .map_err(SerializationError::from)?;
    let decrypted = decrypt(&envelope)?;
    Message::parse(&decrypted)
        .map_err(|e| MeshError::InvalidState(e.to_string()))
}
```

### Phase 0.3: Test Code Strategy

**Test code has different requirements:**

1. **Integration tests**: Can use `.unwrap()` with good error messages
2. **Unit tests**: Should test error paths explicitly
3. **Helper functions**: Should return `Result` and use `?`

**Approach:**
- Allow `.unwrap()` in test assertions (expected to succeed)
- Use `.expect("descriptive message")` for test setup
- Test error paths explicitly with `assert!(result.is_err())`

### Phase 0.4: WASM Considerations

WASM has special constraints:
- Panics are harder to debug
- Error messages must cross FFI boundary
- Consider `wasm-bindgen` error types

### Phase 0.5: Verification

**Acceptance criteria:**
- [ ] Zero `.unwrap()` in production code (core/src/, cli/src/, wasm/src/)
- [ ] All error types documented
- [ ] Error paths tested
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] All existing tests still pass
- [ ] New error path tests added

## Execution Strategy

### Option A: Big Bang (Not Recommended)
- Fix all 917 unwraps in one phase
- High risk of introducing bugs
- Difficult to review
- Long feedback cycle

### Option B: Incremental by Module (Recommended)
- Fix one module at a time
- Verify tests pass after each module
- Easier to review
- Lower risk

**Suggested order:**
1. `core/src/error.rs` - Create error types
2. `core/src/drift/` - Sync protocol (already partially done)
3. `core/src/crypto/` - Cryptographic operations
4. `core/src/store/` - Storage layer
5. `core/src/transport/` - Network transport
6. `core/src/identity/` - Identity management
7. `cli/src/` - CLI application
8. `wasm/src/` - WASM bindings
9. Test files - Clean up test code

### Option C: Defer to Future Phase
- Document as technical debt
- Create tracking issue
- Address in dedicated "Production Hardening" phase
- Focus current phases on feature completion

## Recommendation

**Defer to dedicated Phase 0 execution** after current transformation phases complete.

**Rationale:**
1. Current phases (1-5) focus on architectural improvements
2. Unwrap removal is orthogonal to those goals
3. Mixing concerns increases complexity and risk
4. Better to do unwrap removal as focused, systematic effort
5. Can be done by different team member in parallel

**Proposed approach:**
1. Complete Phases 1-5 as planned
2. Create dedicated "Phase 0 Remediation" effort
3. Allocate dedicated time for systematic unwrap removal
4. Use incremental module-by-module approach
5. Maintain separate branch for review

## Tracking

**Issue:** Phase 0 unwrap removal not executed  
**Impact:** ~917 clippy warnings, potential production panics  
**Priority:** High (production stability)  
**Effort:** Large (multiple days of focused work)  
**Status:** Documented, awaiting scheduling  

## References

- `.clippy.toml` - Disallowed methods configuration
- `docs/specs/scmessenger-rust-transformation/tasks.md` - Phase 0 tasks (incomplete)
- `PHASE2-COMPLETION-SUMMARY.md` - Current clippy status
