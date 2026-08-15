# Continue Phase 1: Async Hygiene - Lock Standardization & Runtime Upgrade

## Current Status

**Phase 1 Progress:** 2/6 tasks complete

- [OK] Task 1.1: Catalog All Lock Usage (COMPLETE)
- [OK] Task 1.2: Replace WASM Locks with RefCell (COMPLETE - ~700 LoC)
- [PENDING] Task 1.3: Standardize CLI Async Locks (VERIFY ONLY - already correct)
- [PENDING] Task 1.4: Migrate Hyper 0.14 → Axum 0.7 (~400 LoC)
- [PENDING] Task 1.5: Verify Hyper 0.14 Removal (VERIFY ONLY)
- [PENDING] Task 1.6: Phase 1 Verification Gate (VERIFY ONLY)

---

## Prompt for Next Session

```
Continue Phase 1 of the SCMessenger Rust Transformation.

**Context:**
- Spec: docs/specs/scmessenger-rust-transformation/
- Phase 1: Async Hygiene - Lock Standardization & Runtime Upgrade
- Tasks 1.1 and 1.2 are COMPLETE
- Current Rust version: 1.95.0
- Build status: cargo check --workspace passes

**Completed Work:**
- Task 1.1: Cataloged all lock usage (phase1-lock-catalog.md)
- Task 1.2: Replaced all WASM locks with Rc<RefCell> (~700 LoC across 6 files)
  - wasm/src/lib.rs, connection_state.rs, transport.rs, worker.rs, storage.rs, mesh.rs
  - All Arc<Mutex/RwLock> → Rc<RefCell>
  - All .write()/.read() → .borrow_mut()/.borrow()
  - Compilation verified: cargo check -p scmessenger-wasm passes

**Next Tasks:**

**Task 1.3: Standardize CLI Async Locks** (VERIFY ONLY - ~10 LoC)
- CLI already uses tokio::sync::Mutex correctly
- Action: Verify with `cargo clippy -- -W clippy::await_holding_lock` in cli/
- Mark task complete if no warnings

**Task 1.4: Migrate Hyper 0.14 → Axum 0.7** (~400 LoC)
1. Read cli/Cargo.toml to verify current Hyper version
2. Read cli/src/api.rs to understand current HTTP API structure
3. Read cli/src/server.rs to understand server setup
4. Update cli/Cargo.toml:
   - Remove: hyper = "0.14", hyper-tls (if present)
   - Add: axum = "0.7", tower = "0.4", tower-http = { version = "0.5", features = ["cors", "trace"] }
5. Rewrite cli/src/api.rs:
   - Create ApiState struct
   - Create create_app() function with Router
   - Rewrite handlers using Axum extractors (State, Json, Path)
   - Add CORS middleware
6. Rewrite cli/src/server.rs:
   - Use tokio::net::TcpListener
   - Use axum::serve(listener, app)
7. Verify: cargo check in cli/, cargo build in cli/

**Task 1.5: Verify Hyper 0.14 Removal** (~5 LoC)
- Run: cargo tree | grep hyper in cli/
- Verify: Only Hyper 1.x present (pulled by Axum)
- Verify: No hyper 0.14 in Cargo.lock

**Task 1.6: Phase 1 Verification Gate** (~10 LoC)
- cargo check --workspace
- cargo test --workspace
- cargo clippy --workspace
- cargo fmt --check --workspace
- Verify all API endpoints functional
- Mark Phase 1 complete

**Reference Files:**
- docs/specs/scmessenger-rust-transformation/tasks.md (Phase 1 section)
- docs/specs/scmessenger-rust-transformation/PHASE1-KICKOFF.md (detailed instructions)
- docs/specs/scmessenger-rust-transformation/phase1-lock-catalog.md (lock analysis)
- docs/specs/scmessenger-rust-transformation/PHASE1-TASK1.2-COMPLETE.md (Task 1.2 summary)
- HANDOFF/scmessenger_rust_implementation_prompt.md (Phase 1 implementation details)

**Important:**
- Maintain zero-regression requirement
- All tests must pass
- Follow implementation prompt patterns exactly
- Update task status as you progress
- Use LoC estimates, not time estimates

Start with Task 1.3 (verify CLI async locks).
```

---

## Quick Reference

### Files to Read for Task 1.4
1. cli/Cargo.toml - Check Hyper version
2. cli/src/api.rs - Current HTTP API (~200 LoC)
3. cli/src/server.rs - Current server setup (~100 LoC)

### Expected Changes for Task 1.4
- cli/Cargo.toml: ~10 LoC (dependency updates)
- cli/src/api.rs: ~200 LoC (complete rewrite with Axum)
- cli/src/server.rs: ~50 LoC (update to use Axum serve)
- **Total:** ~260 LoC changes

### Axum 0.7 Pattern (from implementation prompt)
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
    // Handler implementation
}
```

### Verification Commands
```bash
# Task 1.3
cargo clippy -p scmessenger-cli -- -W clippy::await_holding_lock

# Task 1.4
cargo check -p scmessenger-cli
cargo build -p scmessenger-cli

# Task 1.5
cargo tree -p scmessenger-cli | grep hyper

# Task 1.6
cargo check --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt --check --workspace
```

---

## Phase 1 Remaining Work

**Estimated LoC:**
- Task 1.3: ~10 LoC (verification)
- Task 1.4: ~260 LoC (Hyper → Axum migration)
- Task 1.5: ~5 LoC (verification)
- Task 1.6: ~10 LoC (verification)
- **Total:** ~285 LoC

**After Phase 1:**
Proceed to Phase 2: Protocol Hardening - Sync Auth, Versioning, Rate Limits
