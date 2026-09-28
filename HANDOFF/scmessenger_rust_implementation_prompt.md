<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

MASTER IMPLEMENTATION PROMPT: SCMESSENGER RUST TRANSFORMATION
===============================================================


Stance: Zero-regression, gate-verified incremental transformation
Source of Truth: https://github.com/Sovereign-Communication/SCMessenger/tree/main


CONTEXT & CONSTRAINTS
=====================

Before modifying ANY file, you MUST:

1. Load and parse the target file from the repository — do not hallucinate APIs, field names, or module structure
2. Verify dependencies in the relevant Cargo.toml — version constraints determine available APIs
3. Preserve ALL existing functionality — every public function, every impl block, every use statement must survive unless explicitly marked for removal in this prompt
4. Gate every phase — no proceeding to Phase N+1 until Phase N passes its verification checklist

Critical crate versions to verify before writing code:
- hyper — currently 0.14 (migration to 1.0/axum required in Phase 1)
- tokio — verify minor version for tokio::sync::RwLock availability
- libp2p — verify for noise protocol, gossipsub, mdns APIs
- sled — verify transaction API (sled::transaction stabilized in 0.34+)
- uniffi — verify for generate_scaffolding signature
- bincode — verify for Options trait availability (1.3+ vs 2.0)
- serde — verify for #[serde(default)] and container attributes


PHASE 0 — SAFETY: ELIMINATE PANIC VECTORS & ESTABLISH ERROR HIERARCHY
=====================================================================

Objective: Zero .unwrap() or .expect() in production code paths. Structured error types in core.


Step 0.1: Audit & Catalog All Panic Sites
-----------------------------------------

Action: Search every .rs file across all crates for:
- .unwrap()
- .expect(...)
- panic!(...)
- unreachable!() (verify — may be legitimate)

Files to audit:
- core/build.rs
- core/src/**/*.rs
- cli/src/**/*.rs
- wasm/src/**/*.rs

Output: Generate a markdown table:

| File | Line | Expression | Context | Proposed Fix |
|------|------|------------|---------|--------------|
| core/build.rs | 12 | .unwrap() | UniFFI scaffolding | Propagate to std::process::exit(1) |

Gate 0.1: Table reviewed and approved by human operator.


Step 0.2: Create core/src/error.rs — The thiserror Hierarchy
-------------------------------------------------------------

Prerequisites: Verify thiserror is in core/Cargo.toml. If missing or wrong version, update first.

Requirements:

```rust
//! SCMessenger core error types.
//! 
//! All library errors are structured and typed. Applications may use anyhow
//! for ergonomics, but scmessenger_core never erases its own errors.

use thiserror::Error;

/// Top-level error for mesh operations.
#[derive(Error, Debug)]
pub enum MeshError {
    #[error("sync protocol version mismatch: got {got}, expected {expected}")]
    VersionMismatch { got: u16, expected: u16 },

    #[error("transport layer failure: {0}")]
    Transport(#[from] TransportError),

    #[error("relay denied: peer {peer_id} is in state {state:?}")]
    RelayDenied { peer_id: PeerId, state: NetworkState },

    #[error("storage quota exceeded: {used} / {max} bytes")]
    StorageQuota { used: usize, max: usize },

    #[error("serialization failure: {0}")]
    Serialization(#[from] SerializationError),

    #[error("peer authentication failed: {0}")]
    Auth(String),

    #[error("IBLT decode failure: {reason}")]
    IbltDecode { reason: String },
}

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("noise handshake failed: {0}")]
    NoiseHandshake(String),

    #[error("connection reset by peer: {peer_id}")]
    ConnectionReset { peer_id: PeerId },
}

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("bincode encode failed: {0}")]
    Encode(#[from] bincode::Error), // VERIFY: bincode 1.x uses Box<bincode::ErrorKind> or bincode::Error

    #[error("schema version {version} not supported")]
    UnsupportedVersion { version: u16 },
}
```

CRITICAL VERIFICATION:
- Check actual bincode version in core/Cargo.toml
- If bincode 1.x: Box<bincode::ErrorKind> — use #[from] Box<bincode::ErrorKind>
- If bincode 2.x: API changed significantly — research before implementing
- PeerId and NetworkState must be imported from their actual modules

Gate 0.2: 
- [ ] cargo check passes in core/
- [ ] No type errors on #[from] derives
- [ ] All referenced types (PeerId, NetworkState) resolve


Step 0.3: Replace unwrap() in core/build.rs
-------------------------------------------

Current (verify before changing):
```rust
// core/build.rs — LOAD AND VERIFY EXACT CONTENT
fn main() {
    uniffi::generate_scaffolding("src/api.udl").unwrap();
}
```

Target:
```rust
use std::process;

fn main() {
    println!("cargo:rerun-if-changed=src/api.udl");
    
    if let Err(e) = uniffi::generate_scaffolding("src/api.udl") {
        eprintln!("error: UniFFI scaffolding failed for src/api.udl");
        eprintln!("  {e}");
        process::exit(1);
    }
}
```

Gate 0.3: cargo build in core/ succeeds. No change to generated bindings behavior.


Step 0.4: Replace unwrap() in wasm/src/storage.rs
-------------------------------------------------

Prerequisites: Load actual wasm/src/storage.rs. Identify every .unwrap() and its context.

Common patterns to fix:

| Pattern | Replacement |
|---------|-------------|
| lock().unwrap() on std::sync | Use parking_lot (no unwrap) or std::sync::Mutex + ? if fallible |
| Arc::try_unwrap(x).unwrap() | Return Err or handle Arc retention gracefully |
| parse::<T>().unwrap() | Use parse::<T>().map_err(...)? |

Gate 0.4: 
- [ ] wasm-pack build succeeds for wasm/ target
- [ ] No unwrap() remains in wasm/src/**/*.rs


Step 0.5: Replace unwrap() in cli/src/api.rs
--------------------------------------------

Prerequisites: Load cli/src/api.rs. This is Hyper 0.14 code — verify exact handler signatures.

Pattern: Hyper 0.14 handlers often use .unwrap() on Body extraction or header parsing.

Target: Convert to match or ? with hyper::Response::builder() error responses.

Gate 0.5:
- [ ] cargo check in cli/ passes
- [ ] All handlers return Result<Response<Body>, Infallible> or equivalent
- [ ] No unwrap() in cli/src/api.rs


Step 0.6: Verify Zero unwrap() Remaining
----------------------------------------

Action: Run grep -rn '\.unwrap()\|\.expect(' core/src/ cli/src/ wasm/src/ — output must be empty.

Exceptions allowed (document each):
- #[cfg(test)] modules
- build.rs after Step 0.3
- Explicit SAFETY: comments with invariant justification

Gate 0.6: Grep returns zero matches in production code.


PHASE 1 — ASYNC HYGIENE: LOCK STANDARDIZATION & RUNTIME UPGRADE
===============================================================

Objective: Eliminate blocking locks in async contexts. Migrate Hyper 0.14 → Axum 0.7.


Step 1.1: Catalog All Lock Usage
--------------------------------

Action: Search for all synchronization primitives:

grep -rn 'Mutex\|RwLock\|parking_lot' core/src/ cli/src/ wasm/src/

Categorize each:

| File | Type | Current | Context | Should Be |
|------|------|---------|---------|-----------|
| cli/src/server.rs | tokio::sync::Mutex |  | Async handler | Keep |
| wasm/src/mesh.rs | parking_lot::RwLock |  | WASM single-threaded | RefCell or Cell |
| cli/src/api.rs | std::sync::Arc | — | Shared state | Keep, but verify Send |

Gate 1.1: Table complete and reviewed.


Step 1.2: WASM Lock Replacement — parking_lot::RwLock → RefCell
---------------------------------------------------------------

File: wasm/src/mesh.rs (verify exact module name and structure)

Prerequisites: Confirm wasm32-unknown-unknown is single-threaded (no Send/Sync needed).

Current (verify):
```rust
use parking_lot::RwLock;
use std::sync::Arc;

pub struct MeshNode {
    state: Arc<RwLock<MeshState>>,
}
```

Target:
```rust
use std::cell::RefCell;
use std::rc::Rc;

/// WASM is single-threaded — Rc<RefCell<T>> is correct and zero-overhead.
pub struct MeshNode {
    state: Rc<RefCell<MeshState>>,
}

impl MeshNode {
    pub fn with_state<F, R>(&self, f: F) -> R 
    where 
        F: FnOnce(&MeshState) -> R,
    {
        f(&*self.state.borrow())
    }
    
    pub fn with_state_mut<F, R>(&self, f: F) -> R 
    where 
        F: FnOnce(&mut MeshState) -> R,
    {
        f(&mut *self.state.borrow_mut())
    }
}
```

CRITICAL: If MeshNode is used across await points in WASM (e.g., in async JS bindings), RefCell::borrow_mut() across await will panic. Verify actual usage pattern.

Gate 1.2:
- [ ] wasm-pack build succeeds
- [ ] parking_lot removed from wasm/Cargo.toml dependencies
- [ ] No runtime panics in browser tests


Step 1.3: CLI Async Lock Audit — tokio::sync Standardization
------------------------------------------------------------

File: cli/src/server.rs, cli/src/api.rs, any other async modules.

Rule: Any lock held across .await MUST be tokio::sync::Mutex or tokio::sync::RwLock.

Anti-pattern to find and fix:
```rust
// WRONG — blocks executor thread
let guard = std::sync::Mutex::lock(&state).unwrap();
let result = some_async_fn().await; // Other tasks on this thread stall!
drop(guard);
```

Target:
```rust
// CORRECT — yields to executor
let guard = state.lock().await;
let result = some_async_fn().await; // Lock held, but executor can run other tasks
drop(guard);
```

Gate 1.3:
- [ ] cargo clippy -- -W clippy::await_holding_lock passes (or manual audit)
- [ ] No std::sync::Mutex or parking_lot::Mutex in async fn bodies


Step 1.4: Hyper 0.14 → Axum 0.7 Migration
-----------------------------------------

Files: cli/src/api.rs, cli/src/server.rs, cli/Cargo.toml

Prerequisites: Verify current Hyper usage. Common Hyper 0.14 patterns:

```rust
// Current (verify exact code)
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};

let make_svc = make_service_fn(|_conn| async {
    Ok::<_, Infallible>(service_fn(router))
});

let server = Server::bind(&addr).serve(make_svc);
```

Step 1.4a: Update cli/Cargo.toml

```toml
[dependencies]
# REMOVE: hyper = "0.14"
# REMOVE: hyper-tls (if present)
# ADD:
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tokio = { version = "1", features = ["full"] } # Verify existing features
```

Step 1.4b: Rewrite cli/src/api.rs

```rust
//! HTTP API for SCMessenger CLI daemon.
//! 
//! Replaces Hyper 0.14 with Axum 0.7 for type-safe routing and middleware.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

/// Shared application state.
#[derive(Clone)]
pub struct ApiState {
    // VERIFY: Match existing state fields exactly
    pub mesh: Arc<Mutex<MeshHandle>>,
    pub identity: Arc<Identity>,
}

/// Build the Axum router with all routes and middleware.
pub fn create_app(state: ApiState) -> Router {
    Router::new()
        .route("/send", post(send_message))
        .route("/identity", get(get_identity))
        .route("/peers", get(list_peers))
        .route("/messages/:peer_id", get(get_messages))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:9000".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE]),
        )
        .with_state(state)
}

async fn send_message(
    State(state): State<ApiState>,
    Json(payload): Json<SendRequest>,
) -> impl IntoResponse {
    info!(peer = %payload.peer_id, "sending message");
    
    let mesh = state.mesh.lock().await;
    match mesh.send(payload.peer_id, payload.content).await {
        Ok(id) => (StatusCode::OK, Json(json!({ "message_id": id }))),
        Err(e) => {
            warn!(error = %e, "send failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        }
    }
}

// ... other handlers with identical error handling pattern
```

Step 1.4c: Rewrite cli/src/server.rs

```rust
use axum::Server; // VERIFY: axum 0.7 uses tokio::net::TcpListener pattern
use tokio::net::TcpListener;

pub async fn run_server(addr: SocketAddr, state: ApiState) -> Result<(), std::io::Error> {
    let app = create_app(state);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("SCMessenger daemon listening on {}", addr);
    
    axum::serve(listener, app).await
}
```

CRITICAL VERIFICATION: Axum 0.7 changed the server API from axum::Server to axum::serve(listener, app). Verify exact Axum 0.7.x API before writing.

Gate 1.4:
- [ ] cargo check in cli/ passes
- [ ] cargo build in cli/ passes
- [ ] All original routes functional (/send, /identity, /peers, /messages/*)
- [ ] CORS headers present on responses
- [ ] No Hyper 0.14 types remain in cli/src/


Step 1.5: Remove Hyper 0.14 from Dependency Tree
------------------------------------------------

Action: cargo tree | grep hyper — should show only Hyper 1.x (pulled by Axum), not 0.14.

Gate 1.5: No hyper 0.14 in Cargo.lock.


PHASE 2 — PROTOCOL HARDENING: SYNC AUTH, VERSIONING, RATE LIMITS
================================================================

Objective: Mesh sync protocol is resistant to DoS, replay, and version skew.


Step 2.1: Add Schema Versioning to All Network Messages
-------------------------------------------------------

File: core/src/drift/sync.rs (verify exact location)

Current (verify):
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncMessage {
    SyncOffer { iblt_data: Vec<u8>, ... }
    SyncResponse { envelopes: Vec<DriftEnvelope>, ... }
}
```

Target:
```rust
/// Current protocol version. Bump on breaking changes.
pub const SYNC_SCHEMA_VERSION: u16 = 1;

/// Wrapper providing forward-compatible versioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedSyncMessage {
    pub schema_version: u16,
    #[serde(flatten)]
    pub payload: SyncMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SyncMessage {
    #[serde(rename = "offer")]
    SyncOffer { iblt_data: Vec<u8>, peer_proof: [u8; 32], timestamp: u64 },
    
    #[serde(rename = "response")]
    SyncResponse { envelopes: Vec<DriftEnvelope> },
    
    #[serde(rename = "ack")]
    SyncAck { received_count: usize },
}

impl VersionedSyncMessage {
    pub fn new(payload: SyncMessage) -> Self {
        Self { schema_version: SYNC_SCHEMA_VERSION, payload }
    }
    
    pub fn validate(self) -> Result<SyncMessage, MeshError> {
        if self.schema_version != SYNC_SCHEMA_VERSION {
            return Err(MeshError::VersionMismatch {
                got: self.schema_version,
                expected: SYNC_SCHEMA_VERSION,
            });
        }
        Ok(self.payload)
    }
}
```

Gate 2.1:
- [ ] cargo test in core/ passes
- [ ] Bincode serialization round-trips correctly (test with old + new format)
- [ ] Version mismatch produces MeshError::VersionMismatch


Step 2.2: Add Cryptographic Peer Proofs to Sync Offers
------------------------------------------------------

File: core/src/drift/sync.rs

Requirements: Each SyncOffer includes a blake3 hash proving knowledge of current mesh state, preventing blind DoS diff computations.

```rust
use blake3::Hasher;

impl MeshState {
    /// Generate a proof of current state for sync handshake.
    pub fn generate_proof(&self) -> [u8; 32] {
        let mut hasher = Hasher::new();
        // VERIFY: Actual state digest strategy — may need ordered iteration
        for msg in self.messages.iter() {
            hasher.update(&msg.id);
            hasher.update(&msg.timestamp.to_le_bytes());
        }
        hasher.finalize().into()
    }
}
```

Gate 2.2:
- [ ] Proof is deterministic for identical state
- [ ] Proof changes when state changes
- [ ] Remote peer validates proof before computing IBLT diff


Step 2.3: Rate-Limit Sync Initiations Per Peer
----------------------------------------------

File: core/src/drift/relay.rs or new core/src/drift/rate_limit.rs

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct SyncRateLimiter {
    limits: Mutex<HashMap<PeerId, Vec<Instant>>>,
    window: Duration,
    max_per_window: usize,
}

impl SyncRateLimiter {
    pub async fn allow_sync(&self, peer: &PeerId) -> bool {
        let mut limits = self.limits.lock().await;
        let now = Instant::now();
        let window_start = now - self.window;
        
        let entries = limits.entry(peer.clone()).or_default();
        entries.retain(|&t| t > window_start);
        
        if entries.len() >= self.max_per_window {
            return false;
        }
        
        entries.push(now);
        true
    }
}
```

Gate 2.3:
- [ ] Peer exceeding limit is denied with MeshError::RateLimited
- [ ] Rate limit state doesn't grow unbounded (old entries purged)


PHASE 3 — PERFORMANCE: MEMORY BOUNDS, SERIALIZATION, WASM SIZE
==============================================================


Step 3.1: Implement Bounded Storage with EvictionPolicy
-------------------------------------------------------

File: wasm/src/storage.rs

Current (verify): VecDeque<StoredMessage> with optional max_size.

Target:
```rust
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    OldestFirst,
    UnknownSendersFirst,
    LargestFirst,
}

pub struct BoundedStorage {
    messages: VecDeque<StoredMessage>,
    capacity: usize,
    policy: EvictionPolicy,
    current_size: usize, // Track byte-weighted size if needed
}

impl BoundedStorage {
    pub fn with_capacity(capacity: usize, policy: EvictionPolicy) -> Self {
        Self {
            messages: VecDeque::with_capacity(capacity),
            capacity,
            policy,
            current_size: 0,
        }
    }
    
    pub fn push(&mut self, msg: StoredMessage) {
        if self.messages.len() >= self.capacity {
            self.evict_one();
        }
        self.current_size += msg.serialized_size();
        self.messages.push_back(msg);
    }
    
    fn evict_one(&mut self) {
        let idx = match self.policy {
            EvictionPolicy::OldestFirst => 0,
            EvictionPolicy::UnknownSendersFirst => {
                self.messages.iter()
                    .position(|m| !m.verified)
                    .unwrap_or(0)
            }
            EvictionPolicy::LargestFirst => {
                let (idx, _) = self.messages.iter()
                    .enumerate()
                    .max_by_key(|(_, m)| m.serialized_size())
                    .unwrap_or((0, &self.messages[0]));
                idx
            }
        };
        
        if let Some(removed) = self.messages.remove(idx) {
            self.current_size -= removed.serialized_size();
        }
    }
}
```

Gate 3.1:
- [ ] wasm-pack test passes (if tests exist)
- [ ] Storage never exceeds capacity entries
- [ ] current_size tracking is accurate


Step 3.2: Optimize Inbox Deduplication with FxHashSet<[u8; 32]>
---------------------------------------------------------------

File: core/src/store/inbox.rs

Current (verify): HashSet<String> or similar.

Target:
```rust
use rustc_hash::FxHashSet;

pub struct Inbox {
    seen_ids: FxHashSet<[u8; 32]>,
    max_size: usize,
}

impl Inbox {
    pub fn is_duplicate(&self, message_id: &str) -> bool {
        let hash = blake3::hash(message_id.as_bytes());
        self.seen_ids.contains(hash.as_bytes())
    }
    
    pub fn mark_seen(&mut self, message_id: &str) {
        let hash = blake3::hash(message_id.as_bytes());
        if self.seen_ids.len() >= self.max_size {
            // FIFO eviction or clear half
            self.seen_ids.clear(); // Simple: periodic reset
        }
        self.seen_ids.insert(*hash.as_bytes());
    }
}
```

Gate 3.2:
- [ ] Deduplication still correct (no false negatives)
- [ ] Memory usage reduced vs HashSet<String> (measure if possible)


Step 3.3: Add WASM Size Optimization Profile
--------------------------------------------

File: wasm/Cargo.toml

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"

# VERIFY: wee_alloc compatibility with current allocator usage
[dependencies]
# wee_alloc = "0.4" # Optional — measure first, add if >50KB savings
```

File: wasm/build.sh or package.json script

```bash
#!/bin/bash
set -e
wasm-pack build --release --target web
wasm-opt -Oz pkg/scmessenger_wasm_bg.wasm -o pkg/scmessenger_wasm_bg.wasm
```

Gate 3.3:
- [ ] wasm-pack build --release succeeds
- [ ] .wasm file size measured and recorded
- [ ] wasm-opt reduces size further (or document why not)


PHASE 4 — OBSERVABILITY: LOGGING, METRICS, TESTING
===================================================


Step 4.1: Integrate tracing Across All Crates
---------------------------------------------

Files: All Cargo.toml, all src/main.rs / src/lib.rs

Add to each Cargo.toml:
```toml
[dependencies]
tracing = "0.1"
```

Binaries only:
```toml
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

Update cli/src/main.rs:
```rust
fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    // ... existing main
}
```

Instrument key functions (verify exact signatures):
```rust
#[tracing::instrument(skip(envelope), fields(msg_id = %envelope.id))]
pub async fn relay_envelope(&self, envelope: DriftEnvelope) -> Result<(), RelayError> {
    tracing::info!("attempting relay");
    // ...
}
```

Gate 4.1:
- [ ] RUST_LOG=info cargo run in cli/ produces structured output
- [ ] No println! or eprintln! remains in production code


Step 4.2: Add Property-Based Tests for IBLT Sync
------------------------------------------------

File: core/src/drift/sync.rs (in #[cfg(test)] module)

```rust
#[cfg(test)]
mod proptests {
    use proptest::prelude::*;
    use super::*;
    
    proptest! {
        #[test]
        fn sync_reconciles_arbitrary_sets(
            local in prop::collection::vec(any::<MessageId>(), 0..500),
            remote in prop::collection::vec(any::<MessageId>(), 0..500),
        ) {
            let mut local_store = MeshStore::from_iter(local.clone());
            let mut remote_store = MeshStore::from_iter(remote.clone());
            
            let offer = local_store.create_offer();
            let response = remote_store.respond_to_offer(&offer);
            local_store.apply_response(&response);
            
            let expected: HashSet<_> = local.into_iter().chain(remote).collect();
            prop_assert_eq!(local_store.all_messages(), &expected);
            prop_assert_eq!(remote_store.all_messages(), &expected);
        }
    }
}
```

Gate 4.2:
- [ ] cargo test passes
- [ ] proptest runs and doesn't regress


PHASE 5 — POLISH: DOCUMENTATION, EXAMPLES, CI
=============================================


Step 5.1: Doc Comments on All Public APIs
-----------------------------------------

Rule: Every pub fn, pub struct, pub enum, pub trait gets /// docs.

Gate 5.1: cargo doc in core/ produces zero warnings.


Step 5.2: Add mdbook Architecture Guide
---------------------------------------

File: docs/architecture.md (new)

```markdown
# SCMessenger Architecture

## Mesh Topology
## Sync Protocol
## Security Model
## WASM Bridge
```

Gate 5.2: Book builds with mdbook build.


GLOBAL NON-REGRESSION CHECKLIST
===============================

Before declaring ANY phase complete:

- [ ] cargo check passes in all crates (core, cli, wasm)
- [ ] cargo test passes in all crates
- [ ] cargo clippy -- -D warnings passes (or justify each warning)
- [ ] cargo fmt --check passes
- [ ] No deleted public APIs without deprecation cycle
- [ ] wasm-pack build succeeds (if WASM changed)
- [ ] Binary runs and basic smoke test passes (send/receive message)


EMERGENCY HALT CONDITIONS
=========================

Stop immediately and request human review if:

1. Compilation fails and fix is not obvious within 15 minutes
2. Public API changes require updates in >2 files (suggests broader refactor needed)
3. Dependency conflict arises (e.g., Axum 0.7 requires Tokio 1.37+, but project pins 1.28)
4. Test coverage drops below pre-transformation levels
5. WASM size increases by >20% after optimization phase


Execute Phase 0 sequentially. Await gate approval before proceeding to Phase 1.

---
Created using www.jenova.ai