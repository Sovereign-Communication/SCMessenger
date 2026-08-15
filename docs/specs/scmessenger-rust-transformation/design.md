# Design Document: SCMessenger Rust Transformation

## Overview

This design document outlines the technical approach for transforming the SCMessenger Rust codebase through a phased, gate-verified process. The transformation addresses critical safety, architectural, performance, and observability issues while maintaining zero regression.

### Design Principles

1. **Zero-Regression**: All existing functionality must be preserved throughout transformation
2. **Gate-Verified Progress**: Each phase must pass verification before proceeding to next phase
3. **Incremental Transformation**: Changes are applied in small, verifiable steps
4. **Evidence-Based Design**: Load and verify actual code before making changes
5. **Type-Safe Error Handling**: Replace panic vectors with structured error types

### Architecture Context

SCMessenger consists of three main crates:
- **scmessenger_core**: Library crate with mesh networking, sync protocol, storage
- **scmessenger_cli**: Binary crate with desktop daemon and HTTP API
- **scmessenger_wasm**: WASM crate for browser integration

## Phase 0: Safety - Eliminate Panic Vectors & Establish Error Hierarchy

### Objective
Remove all `.unwrap()`, `.expect()`, and `panic!()` calls from production code and establish a structured error hierarchy using `thiserror`.

### Component Design

#### Error Type Hierarchy

```rust
// core/src/error.rs

/// Top-level error for mesh operations
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
    
    #[error("rate limited: peer {peer_id}")]
    RateLimited { peer_id: PeerId },
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
    Encode(#[from] Box<bincode::ErrorKind>), // For bincode 1.x
    
    #[error("schema version {version} not supported")]
    UnsupportedVersion { version: u16 },
}
```

#### Build Script Error Handling

```rust
// core/build.rs
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

### Implementation Strategy

1. **Audit Phase**: Search all `.rs` files for panic vectors
2. **Error Type Creation**: Create `core/src/error.rs` with complete hierarchy
3. **Systematic Replacement**: Replace unwrap/expect with `?` operator or explicit error handling
4. **Verification**: Run `grep` to confirm zero panic vectors remain

### Verification Gates

- [ ] `cargo check` passes in all crates
- [ ] No type errors on `#[from]` derives
- [ ] All referenced types (PeerId, NetworkState) resolve
- [ ] `grep -rn '\.unwrap()\|\.expect('` returns zero matches in production code

## Phase 1: Async Hygiene - Lock Standardization & Runtime Upgrade

### Objective
Eliminate blocking locks in async contexts and migrate from Hyper 0.14 to Axum 0.7.

### Component Design

#### Lock Usage Strategy

| Context | Lock Type | Rationale |
|---------|-----------|-----------|
| Async functions (CLI) | `tokio::sync::Mutex` / `tokio::sync::RwLock` | Yields to executor, prevents thread blocking |
| Sync functions | `parking_lot::Mutex` / `parking_lot::RwLock` | Fast, no poisoning |
| WASM (single-threaded) | `RefCell` / `Cell` | Zero overhead, no Send/Sync needed |

#### WASM Lock Replacement

```rust
// wasm/src/mesh.rs
use std::cell::RefCell;
use std::rc::Rc;

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

#### Axum Migration Architecture

```rust
// cli/src/api.rs

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
    let mesh = state.mesh.lock().await;
    match mesh.send(payload.peer_id, payload.content).await {
        Ok(id) => (StatusCode::OK, Json(json!({ "message_id": id }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
```

### Dependency Updates

```toml
# cli/Cargo.toml
[dependencies]
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tokio = { version = "1", features = ["full"] }
```

### Verification Gates

- [ ] `cargo clippy -- -W clippy::await_holding_lock` passes
- [ ] No `std::sync::Mutex` in async function bodies
- [ ] `wasm-pack build` succeeds
- [ ] All API endpoints functional after Axum migration
- [ ] No Hyper 0.14 in `cargo tree` output

## Phase 2: Protocol Hardening - Sync Auth, Versioning, Rate Limits

### Objective
Make mesh sync protocol resistant to DoS, replay attacks, and version skew.

### Component Design

#### Schema Versioning

```rust
// core/src/drift/sync.rs

pub const SYNC_SCHEMA_VERSION: u16 = 1;

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
    SyncOffer { 
        iblt_data: Vec<u8>, 
        peer_proof: [u8; 32], 
        timestamp: u64 
    },
    
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

#### Cryptographic Peer Proofs

```rust
// core/src/drift/sync.rs

use blake3::Hasher;

impl MeshState {
    pub fn generate_proof(&self) -> [u8; 32] {
        let mut hasher = Hasher::new();
        
        // Sort for deterministic hashing
        let mut messages: Vec<_> = self.messages.iter().collect();
        messages.sort_by_key(|m| &m.id);
        
        for msg in messages {
            hasher.update(msg.id.as_bytes());
            hasher.update(&msg.timestamp.to_le_bytes());
        }
        
        hasher.finalize().into()
    }
    
    pub fn verify_proof(&self, proof: &[u8; 32]) -> bool {
        &self.generate_proof() == proof
    }
}
```

#### Rate Limiter

```rust
// core/src/drift/rate_limit.rs

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct SyncRateLimiter {
    limits: Mutex<HashMap<PeerId, Vec<Instant>>>,
    window: Duration,
    max_per_window: usize,
}

impl SyncRateLimiter {
    pub fn new(window: Duration, max_per_window: usize) -> Self {
        Self {
            limits: Mutex::new(HashMap::new()),
            window,
            max_per_window,
        }
    }
    
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
    
    pub async fn cleanup_expired(&self) {
        let mut limits = self.limits.lock().await;
        let now = Instant::now();
        let window_start = now - self.window;
        
        limits.retain(|_, entries| {
            entries.retain(|&t| t > window_start);
            !entries.is_empty()
        });
    }
}
```

### Verification Gates

- [ ] Version mismatch produces `MeshError::VersionMismatch`
- [ ] Proof is deterministic for identical state
- [ ] Rate limiter denies excessive requests
- [ ] Rate limiter state doesn't grow unbounded

## Phase 3: Performance - Memory Bounds, Serialization, WASM Size

### Objective
Optimize memory usage, serialization efficiency, and WASM bundle size.

### Component Design

#### Bounded Storage with Eviction

```rust
// wasm/src/storage.rs

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
    current_size: usize,
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
                self.messages.iter()
                    .enumerate()
                    .max_by_key(|(_, m)| m.serialized_size())
                    .map(|(idx, _)| idx)
                    .unwrap_or(0)
            }
        };
        
        if let Some(removed) = self.messages.remove(idx) {
            self.current_size -= removed.serialized_size();
        }
    }
}
```

#### Optimized Deduplication

```rust
// core/src/store/inbox.rs

use rustc_hash::FxHashSet;
use blake3;

pub struct Inbox {
    seen_ids: FxHashSet<[u8; 32]>,
    max_size: usize,
}

impl Inbox {
    pub fn new(max_size: usize) -> Self {
        Self {
            seen_ids: FxHashSet::default(),
            max_size,
        }
    }
    
    pub fn is_duplicate(&self, message_id: &str) -> bool {
        let hash = blake3::hash(message_id.as_bytes());
        self.seen_ids.contains(hash.as_bytes())
    }
    
    pub fn mark_seen(&mut self, message_id: &str) {
        let hash = blake3::hash(message_id.as_bytes());
        
        if self.seen_ids.len() >= self.max_size {
            self.seen_ids.clear();
        }
        
        self.seen_ids.insert(*hash.as_bytes());
    }
}
```

#### WASM Size Optimization

```toml
# wasm/Cargo.toml

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
```

```bash
# wasm/build.sh
#!/bin/bash
set -e

echo "Building WASM with size optimizations..."
wasm-pack build --release --target web

echo "Running wasm-opt..."
if command -v wasm-opt &> /dev/null; then
    wasm-opt -Oz pkg/scmessenger_wasm_bg.wasm -o pkg/scmessenger_wasm_bg.wasm
    echo "WASM optimization complete"
else
    echo "Warning: wasm-opt not found. Install binaryen for size optimization."
fi

echo "Final bundle size:"
ls -lh pkg/scmessenger_wasm_bg.wasm
```

### Verification Gates

- [ ] Storage never exceeds capacity
- [ ] Deduplication has no false negatives
- [ ] WASM bundle size measured and optimized
- [ ] `wasm-pack build --release` succeeds

## Phase 4: Observability - Logging, Metrics, Testing

### Objective
Integrate structured logging and property-based testing.

### Component Design

#### Tracing Integration

```rust
// cli/src/main.rs

use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_thread_ids(true)
        .init();
    
    tracing::info!("SCMessenger CLI starting");
    
    // ... rest of main
}
```

```rust
// core/src/drift/relay.rs

#[tracing::instrument(skip(envelope), fields(msg_id = %envelope.id))]
pub async fn relay_envelope(&self, envelope: DriftEnvelope) -> Result<(), MeshError> {
    tracing::info!("attempting relay");
    
    // ... relay logic
    
    tracing::info!("relay successful");
    Ok(())
}
```

#### Property-Based Tests

```rust
// core/src/drift/sync.rs

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
        
        #[test]
        fn proof_is_deterministic(messages in prop::collection::vec(any::<Message>(), 0..100)) {
            let state1 = MeshState::from_iter(messages.clone());
            let state2 = MeshState::from_iter(messages);
            
            prop_assert_eq!(state1.generate_proof(), state2.generate_proof());
        }
    }
}
```

### Verification Gates

- [ ] `RUST_LOG=info cargo run` produces structured output
- [ ] No `println!` or `eprintln!` in production code
- [ ] Property tests pass with 100+ cases
- [ ] `cargo test` passes

## Phase 5: Polish - Documentation, Examples, CI

### Objective
Complete documentation and establish CI pipeline.

### Component Design

#### API Documentation

All public APIs must have doc comments:

```rust
/// Manages bounded message storage with configurable eviction policies.
///
/// # Examples
///
/// ```
/// use scmessenger_wasm::BoundedStorage;
///
/// let mut storage = BoundedStorage::with_capacity(100, EvictionPolicy::OldestFirst);
/// storage.push(message);
/// ```
pub struct BoundedStorage {
    // ...
}
```

#### Architecture Documentation

```markdown
# docs/architecture.md

## Mesh Topology

SCMessenger uses a peer-to-peer mesh topology where each node maintains
direct connections to discovered peers...

## Sync Protocol

The IBLT (Invertible Bloom Lookup Table) sync protocol enables efficient
set reconciliation between peers...

## Security Model

- Noise protocol for transport encryption
- Ed25519 for identity and message signing
- Blake3 for state proofs and deduplication

## WASM Bridge

The WASM crate exposes a JavaScript API via UniFFI bindings...
```

### Verification Gates

- [ ] `cargo doc` produces zero warnings
- [ ] All public APIs documented
- [ ] `mdbook build` succeeds
- [ ] Architecture diagrams included

## Global Verification Checklist

Before declaring ANY phase complete:

- [ ] `cargo check` passes in all crates
- [ ] `cargo test` passes in all crates
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] No deleted public APIs without deprecation
- [ ] `wasm-pack build` succeeds (if WASM changed)
- [ ] Smoke test passes (send/receive message)

## Emergency Halt Conditions

Stop and request human review if:

1. Compilation fails and fix not obvious within 15 minutes
2. Public API changes require updates in >2 files
3. Dependency conflict arises
4. Test coverage drops below pre-transformation levels
5. WASM size increases by >20% after optimization

## Implementation Order

1. Phase 0: Safety (foundation for all other work)
2. Phase 1: Async Hygiene (enables correct concurrent behavior)
3. Phase 2: Protocol Hardening (security and reliability)
4. Phase 3: Performance (optimization)
5. Phase 4: Observability (debugging and monitoring)
6. Phase 5: Polish (documentation and CI)

Each phase builds on the previous, and all phases maintain zero regression.
