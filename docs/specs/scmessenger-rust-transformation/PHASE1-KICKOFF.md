# Phase 1 Kickoff: Async Hygiene - Lock Standardization & Runtime Upgrade
## SCMessenger Rust Transformation

---

## Context

You are continuing the SCMessenger Rust Transformation project. **Phase 0 (Safety) is complete**. You are now starting **Phase 1: Async Hygiene - Lock Standardization & Runtime Upgrade**.

### Project Overview
- **Project**: SCMessenger Rust Transformation
- **Spec Location**: `docs/specs/scmessenger-rust-transformation/`
- **Current Phase**: Phase 1 (Async Hygiene)
- **Previous Phase**: Phase 0 (Safety) - [OK] COMPLETE

### Phase 0 Completion Status
[OK] **Rust updated**: 1.75.0 → 1.95.0  
[OK] **Production panic vectors eliminated**: 40+ unwraps/expects replaced  
[OK] **Error hierarchy established**: MeshError, TransportError, SerializationError  
[OK] **Helper functions created**: `path_to_string()`, `to_js_value_safe()`  
[OK] **All tests passing**: 908 tests (860 core + 44 cli + 4 mobile)  
[OK] **Build verified**: `cargo check --workspace` passes  

---

## Phase 1 Objective

**Eliminate blocking locks in async contexts and migrate from Hyper 0.14 to Axum 0.7.**

### Goals
1. Catalog all lock usage across crates
2. Replace WASM locks (parking_lot::RwLock) with RefCell (single-threaded)
3. Standardize CLI async locks (use tokio::sync::Mutex/RwLock)
4. Migrate HTTP API from Hyper 0.14 to Axum 0.7
5. Verify Hyper 0.14 completely removed from dependency tree

---

## Phase 1 Tasks

### Task 1.1: Catalog All Lock Usage [PENDING]
**Objective**: Identify all synchronization primitives and categorize by context

**Actions**:
1. Search for `Mutex` in core/src/, cli/src/, wasm/src/
2. Search for `RwLock` in core/src/, cli/src/, wasm/src/
3. Search for `parking_lot` in core/src/, cli/src/, wasm/src/
4. Create categorization table:

| File | Type | Current | Context | Should Be |
|------|------|---------|---------|-----------|
| cli/src/server.rs | tokio::sync::Mutex | [OK] | Async handler | Keep |
| wasm/src/mesh.rs | parking_lot::RwLock | [FAIL] | WASM single-threaded | RefCell |
| cli/src/api.rs | std::sync::Arc | — | Shared state | Keep, verify Send |

**Verification**: Table complete and reviewed

---

### Task 1.2: Replace WASM Locks with RefCell [PENDING]
**Objective**: Replace thread-safe locks with single-threaded RefCell in WASM

**Current Pattern** (verify first):
```rust
use parking_lot::RwLock;
use std::sync::Arc;

pub struct MeshNode {
    state: Arc<RwLock<MeshState>>,
}
```

**Target Pattern**:
```rust
use std::cell::RefCell;
use std::rc::Rc;

/// WASM is single-threaded — Rc<RefCell<T>> is correct and zero-overhead.
pub struct MeshNode {
    state: Rc<RefCell<MeshState>>,
}

impl MeshNode {
    pub fn with_state<F, R>(&self, f: F) -> R 
    where F: FnOnce(&MeshState) -> R 
    {
        f(&*self.state.borrow())
    }
    
    pub fn with_state_mut<F, R>(&self, f: F) -> R 
    where F: FnOnce(&mut MeshState) -> R 
    {
        f(&mut *self.state.borrow_mut())
    }
}
```

**Actions**:
1. Read wasm/src/mesh.rs (or equivalent) to verify structure
2. Replace Arc<RwLock<T>> with Rc<RefCell<T>>
3. Implement with_state() and with_state_mut() helper methods
4. Update all lock usage sites to use helper methods
5. Remove parking_lot from wasm/Cargo.toml
6. Run `wasm-pack build` to verify compilation

**Critical**: If MeshNode is used across await points, RefCell::borrow_mut() will panic. Verify usage pattern first.

**Verification**: 
- [ ] wasm-pack build succeeds
- [ ] parking_lot removed from wasm/Cargo.toml
- [ ] No runtime panics in browser tests

---

### Task 1.3: Standardize CLI Async Locks [PENDING]
**Objective**: Ensure all locks in async contexts use tokio::sync

**Rule**: Any lock held across `.await` MUST be tokio::sync::Mutex or tokio::sync::RwLock

**Anti-pattern to find**:
```rust
// WRONG — blocks executor thread
let guard = std::sync::Mutex::lock(&state).unwrap();
let result = some_async_fn().await; // Other tasks stall!
drop(guard);
```

**Correct pattern**:
```rust
// CORRECT — yields to executor
let guard = state.lock().await;
let result = some_async_fn().await;
drop(guard);
```

**Actions**:
1. Read cli/src/server.rs and cli/src/api.rs
2. Identify all std::sync::Mutex in async functions
3. Replace std::sync::Mutex with tokio::sync::Mutex in async contexts
4. Replace parking_lot::Mutex with tokio::sync::Mutex in async contexts
5. Update lock acquisition to use `.lock().await`
6. Run `cargo clippy -- -W clippy::await_holding_lock` in cli/

**Verification**:
- [ ] cargo clippy -- -W clippy::await_holding_lock passes
- [ ] No std::sync::Mutex or parking_lot::Mutex in async fn bodies

---

### Task 1.4: Migrate from Hyper 0.14 to Axum 0.7 [PENDING]
**Objective**: Replace Hyper 0.14 with Axum 0.7 for type-safe routing

**Current Pattern** (verify first):
```rust
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};

let make_svc = make_service_fn(|_conn| async {
    Ok::<_, Infallible>(service_fn(router))
});

let server = Server::bind(&addr).serve(make_svc);
```

**Target Pattern**:
```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};

#[derive(Clone)]
pub struct ApiState {
    pub mesh: Arc<Mutex<MeshHandle>>,
    pub identity: Arc<Identity>,
}

pub fn create_app(state: ApiState) -> Router {
    Router::new()
        .route("/send", post(send_message))
        .route("/identity", get(get_identity))
        .route("/peers", get(list_peers))
        .route("/messages/:peer_id", get(get_messages))
        .layer(CorsLayer::new()
            .allow_origin("http://localhost:9000".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([header::CONTENT_TYPE]))
        .with_state(state)
}

async fn send_message(
    State(state): State<ApiState>,
    Json(payload): Json<SendRequest>,
) -> impl IntoResponse {
    let mesh = state.mesh.lock().await;
    match mesh.send(payload.peer_id, payload.content).await {
        Ok(id) => (StatusCode::OK, Json(json!({ "message_id": id }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))),
    }
}
```

**Actions**:
1. Read cli/src/api.rs to verify current Hyper usage
2. Read cli/src/server.rs to verify server setup
3. Update cli/Cargo.toml:
   - Add: `axum = "0.7"`, `tower = "0.4"`, `tower-http = { version = "0.5", features = ["cors", "trace"] }`
   - Remove: `hyper = "0.14"`, `hyper-tls` (if present)
4. Create ApiState struct with mesh and identity fields
5. Implement create_app() function with Router and routes
6. Rewrite send_message handler using Axum extractors
7. Rewrite get_identity handler using Axum extractors
8. Rewrite list_peers handler using Axum extractors
9. Rewrite get_messages handler using Axum extractors
10. Add CORS middleware using tower-http
11. Update server.rs to use tokio::net::TcpListener and axum::serve
12. Run `cargo check` in cli/
13. Run `cargo build` in cli/
14. Test all API endpoints

**Critical**: Axum 0.7 uses `axum::serve(listener, app)` not `axum::Server`. Verify exact API.

**Verification**:
- [ ] cargo check in cli/ passes
- [ ] cargo build in cli/ passes
- [ ] All original routes functional (/send, /identity, /peers, /messages/*)
- [ ] CORS headers present on responses
- [ ] No Hyper 0.14 types remain in cli/src/

---

### Task 1.5: Verify Hyper 0.14 Removal [PENDING]
**Objective**: Confirm Hyper 0.14 completely removed from dependency tree

**Actions**:
1. Run `cargo tree | grep hyper` in cli/
2. Verify only Hyper 1.x present (pulled by Axum)
3. Verify no hyper 0.14 in Cargo.lock

**Verification**:
- [ ] No hyper 0.14 in cargo tree output
- [ ] No hyper 0.14 in Cargo.lock

---

### Task 1.6: Phase 1 Verification Gate [PENDING]
**Objective**: Verify all Phase 1 changes maintain zero regression

**Actions**:
1. Run `cargo check` in all crates
2. Run `cargo test` in all crates
3. Run `cargo clippy -- -D warnings` in all crates (or without -D if test warnings)
4. Run `cargo fmt --check` in all crates
5. Verify all API endpoints functional
6. Verify CORS headers present
7. Run smoke test

**Verification**:
- [ ] cargo check passes
- [ ] cargo test passes
- [ ] cargo clippy passes
- [ ] cargo fmt --check passes
- [ ] All API endpoints work
- [ ] CORS headers present
- [ ] Smoke test passes

---

## Critical Files to Review

### Before Starting
1. **cli/Cargo.toml** - Check current Hyper version
2. **wasm/Cargo.toml** - Check for parking_lot dependency
3. **cli/src/api.rs** - Current HTTP API implementation
4. **cli/src/server.rs** - Current server setup
5. **wasm/src/mesh.rs** - Current WASM lock usage (if exists)

### Implementation Prompt Reference
- **HANDOFF/scmessenger_rust_implementation_prompt.md** - Detailed Phase 1 instructions

---

## Important Constraints

### Zero-Regression Requirement
- All existing functionality must be preserved
- All tests must continue to pass
- No public APIs should be deleted

### Verification Before Changes
- Always read actual code before making changes
- Verify dependency versions in Cargo.toml
- Check exact API signatures (Axum 0.7 vs 0.6 differs)

### Lock Replacement Rules
1. **Async contexts**: Use tokio::sync::Mutex/RwLock
2. **Sync contexts**: Use parking_lot or std::sync
3. **WASM (single-threaded)**: Use RefCell/Cell
4. **Never hold std::sync::Mutex across await**: Blocks executor

---

## Success Criteria

Phase 1 is complete when:
- [OK] All locks categorized and documented
- [OK] WASM uses RefCell instead of parking_lot::RwLock
- [OK] CLI async code uses tokio::sync locks
- [OK] HTTP API migrated to Axum 0.7
- [OK] Hyper 0.14 completely removed
- [OK] All tests pass (cargo test)
- [OK] All crates compile (cargo check)
- [OK] Clippy passes
- [OK] Code formatted (cargo fmt)
- [OK] API endpoints functional
- [OK] CORS working

---

## Getting Started

### Step 1: Verify Phase 0 Completion
```bash
cargo check --workspace
cargo test --lib -p scmessenger-core -p scmessenger-cli -p scmessenger-mobile
```

### Step 2: Start Task 1.1 (Catalog Locks)
```bash
# Search for Mutex usage
rg "Mutex" core/src/ cli/src/ wasm/src/

# Search for RwLock usage
rg "RwLock" core/src/ cli/src/ wasm/src/

# Search for parking_lot usage
rg "parking_lot" core/src/ cli/src/ wasm/src/
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
3. Dependency conflict arises (e.g., Axum 0.7 requires Tokio 1.37+)
4. Test coverage drops below pre-transformation levels
5. WASM size increases by >20%

---

## Quick Reference

### Current State (Post-Phase 0)
- **Rust version**: 1.95.0
- **Build status**: [OK] Passing
- **Test status**: [OK] 908 tests passing
- **Panic vectors**: [OK] Eliminated from production code
- **Error hierarchy**: [OK] Established

### Phase 1 Focus
- **Lock hygiene**: Async-aware locks in async contexts
- **WASM optimization**: Single-threaded RefCell instead of locks
- **HTTP framework**: Modern Axum 0.7 instead of legacy Hyper 0.14
- **Dependency cleanup**: Remove Hyper 0.14 completely

---

## Prompt to Use in New Window

```
I'm continuing the SCMessenger Rust Transformation project. Phase 0 (Safety) is complete. 

Please execute Phase 1: Async Hygiene - Lock Standardization & Runtime Upgrade.

**Context:**
- Spec location: docs/specs/scmessenger-rust-transformation/
- Phase 0 status: [OK] COMPLETE (panic vectors eliminated, error hierarchy established)
- Current Rust version: 1.95.0
- Build status: All tests passing (908 tests)

**Phase 1 Objectives:**
1. Catalog all lock usage
2. Replace WASM locks with RefCell
3. Standardize CLI async locks (tokio::sync)
4. Migrate HTTP API from Hyper 0.14 to Axum 0.7
5. Verify Hyper 0.14 removal

**Instructions:**
- Read docs/specs/scmessenger-rust-transformation/PHASE1-KICKOFF.md for detailed instructions
- Follow tasks in docs/specs/scmessenger-rust-transformation/tasks.md (Phase 1 section)
- Reference HANDOFF/scmessenger_rust_implementation_prompt.md for implementation details
- Update task status as you progress
- Maintain zero-regression (all tests must pass)

Please start with Task 1.1: Catalog All Lock Usage.
```

---

**Phase 1 Status: [PENDING] READY TO START**
