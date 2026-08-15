# Phase 3 Kickoff: Observability - Metrics, Tracing, Health Checks

## SCMessenger Rust Transformation

---

## Context

You are continuing the SCMessenger Rust Transformation project. **Phase 1 (Async Hygiene) and Phase 2 (Protocol Hardening) are complete**. You are now starting **Phase 3: Observability - Metrics, Tracing, Health Checks**.

### Project Overview

- **Project**: SCMessenger Rust Transformation
- **Spec Location**: `docs/specs/scmessenger-rust-transformation/`
- **Current Phase**: Phase 3 (Observability)
- **Previous Phases**: 
  - Phase 1 (Async Hygiene) - [OK] COMPLETE
  - Phase 2 (Protocol Hardening) - [OK] COMPLETE

### Phase 2 Completion Status

[OK] **Schema versioning added**: All sync messages include version field  
[OK] **Cryptographic peer proofs**: blake3-based proofs prevent spoofing  
[OK] **Rate limiting implemented**: SyncRateLimiter prevents sync flooding  
[OK] **All tests passing**: 870+ tests pass, zero regressions  
[OK] **Code formatted**: cargo fmt passes  

---

## Phase 3 Objective

**Add comprehensive observability to the mesh network for production monitoring and debugging.**

### Goals

1. Add structured metrics collection for sync operations
2. Implement distributed tracing for message flow
3. Add health check endpoints for monitoring
4. Create diagnostic tools for troubleshooting

---

## Phase 3 Tasks

### Task 3.1: Add Metrics Collection for Sync Operations (~150 LoC)

**Objective**: Collect and expose metrics for sync protocol operations

**Current State** (verify first):
```rust
// core/src/drift/sync.rs
// No metrics collection currently
```

**Target Pattern**:
```rust
// core/src/drift/metrics.rs (new file)
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Metrics for sync protocol operations
#[derive(Debug, Clone)]
pub struct SyncMetrics {
    /// Total sync offers sent
    pub offers_sent: Arc<AtomicU64>,
    /// Total sync offers received
    pub offers_received: Arc<AtomicU64>,
    /// Total sync responses sent
    pub responses_sent: Arc<AtomicU64>,
    /// Total sync responses received
    pub responses_received: Arc<AtomicU64>,
    /// Total sync completions sent
    pub completions_sent: Arc<AtomicU64>,
    /// Total sync completions received
    pub completions_received: Arc<AtomicU64>,
    /// Total messages synced (sent to peers)
    pub messages_synced_out: Arc<AtomicU64>,
    /// Total messages synced (received from peers)
    pub messages_synced_in: Arc<AtomicU64>,
    /// Total sync failures
    pub sync_failures: Arc<AtomicU64>,
    /// Total version mismatches
    pub version_mismatches: Arc<AtomicU64>,
    /// Total rate limit denials
    pub rate_limit_denials: Arc<AtomicU64>,
}

impl SyncMetrics {
    pub fn new() -> Self {
        Self {
            offers_sent: Arc::new(AtomicU64::new(0)),
            offers_received: Arc::new(AtomicU64::new(0)),
            responses_sent: Arc::new(AtomicU64::new(0)),
            responses_received: Arc::new(AtomicU64::new(0)),
            completions_sent: Arc::new(AtomicU64::new(0)),
            completions_received: Arc::new(AtomicU64::new(0)),
            messages_synced_out: Arc::new(AtomicU64::new(0)),
            messages_synced_in: Arc::new(AtomicU64::new(0)),
            sync_failures: Arc::new(AtomicU64::new(0)),
            version_mismatches: Arc::new(AtomicU64::new(0)),
            rate_limit_denials: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get a snapshot of all metrics
    pub fn snapshot(&self) -> SyncMetricsSnapshot {
        SyncMetricsSnapshot {
            offers_sent: self.offers_sent.load(Ordering::Relaxed),
            offers_received: self.offers_received.load(Ordering::Relaxed),
            responses_sent: self.responses_sent.load(Ordering::Relaxed),
            responses_received: self.responses_received.load(Ordering::Relaxed),
            completions_sent: self.completions_sent.load(Ordering::Relaxed),
            completions_received: self.completions_received.load(Ordering::Relaxed),
            messages_synced_out: self.messages_synced_out.load(Ordering::Relaxed),
            messages_synced_in: self.messages_synced_in.load(Ordering::Relaxed),
            sync_failures: self.sync_failures.load(Ordering::Relaxed),
            version_mismatches: self.version_mismatches.load(Ordering::Relaxed),
            rate_limit_denials: self.rate_limit_denials.load(Ordering::Relaxed),
        }
    }

    /// Reset all metrics to zero
    pub fn reset(&self) {
        self.offers_sent.store(0, Ordering::Relaxed);
        self.offers_received.store(0, Ordering::Relaxed);
        self.responses_sent.store(0, Ordering::Relaxed);
        self.responses_received.store(0, Ordering::Relaxed);
        self.completions_sent.store(0, Ordering::Relaxed);
        self.completions_received.store(0, Ordering::Relaxed);
        self.messages_synced_out.store(0, Ordering::Relaxed);
        self.messages_synced_in.store(0, Ordering::Relaxed);
        self.sync_failures.store(0, Ordering::Relaxed);
        self.version_mismatches.store(0, Ordering::Relaxed);
        self.rate_limit_denials.store(0, Ordering::Relaxed);
    }
}

/// Snapshot of sync metrics at a point in time
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncMetricsSnapshot {
    pub offers_sent: u64,
    pub offers_received: u64,
    pub responses_sent: u64,
    pub responses_received: u64,
    pub completions_sent: u64,
    pub completions_received: u64,
    pub messages_synced_out: u64,
    pub messages_synced_in: u64,
    pub sync_failures: u64,
    pub version_mismatches: u64,
    pub rate_limit_denials: u64,
}

// Update SyncSession to track metrics
impl SyncSession {
    pub fn initiate(&mut self, store: &MeshStore, metrics: &SyncMetrics) -> Result<SyncMessage, DriftError> {
        // ... existing code ...
        metrics.offers_sent.fetch_add(1, Ordering::Relaxed);
        Ok(offer)
    }

    pub fn respond(&mut self, store: &MeshStore, offer: &SyncMessage, metrics: &SyncMetrics) 
        -> Result<(SyncMessage, Vec<StoredEnvelope>), DriftError> {
        metrics.offers_received.fetch_add(1, Ordering::Relaxed);
        // ... existing code ...
        metrics.responses_sent.fetch_add(1, Ordering::Relaxed);
        metrics.messages_synced_out.fetch_add(missing_envelopes.len() as u64, Ordering::Relaxed);
        Ok((response, envelopes))
    }
}
```

**Actions**:
1. Create `core/src/drift/metrics.rs`
2. Implement `SyncMetrics` struct with atomic counters
3. Implement `SyncMetricsSnapshot` for point-in-time snapshots
4. Add metrics parameter to `SyncSession` methods
5. Increment appropriate counters in sync operations
6. Add unit tests for metrics collection
7. Export metrics types from drift module
8. Run `cargo test` in core/

**Verification**:
- [ ] cargo test passes
- [ ] Metrics are incremented correctly
- [ ] Snapshot captures current state
- [ ] Reset clears all counters

---

### Task 3.2: Implement Distributed Tracing (~150 LoC)

**Objective**: Add trace IDs to track message flow through the mesh

**Target Pattern**:
```rust
// core/src/drift/tracing.rs (new file)
use uuid::Uuid;
use std::collections::HashMap;
use std::time::Instant;

/// Trace context for distributed tracing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TraceContext {
    /// Unique trace ID for this operation
    pub trace_id: String,
    /// Parent span ID (if this is a child span)
    pub parent_span_id: Option<String>,
    /// Current span ID
    pub span_id: String,
    /// Timestamp when trace started
    pub started_at: u64,
}

impl TraceContext {
    /// Create a new root trace context
    pub fn new_root() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            span_id: Uuid::new_v4().to_string(),
            started_at: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Create a child span from this context
    pub fn child_span(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            parent_span_id: Some(self.span_id.clone()),
            span_id: Uuid::new_v4().to_string(),
            started_at: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Span for tracking operation duration
#[derive(Debug)]
pub struct Span {
    pub context: TraceContext,
    pub operation: String,
    pub started_at: Instant,
    pub attributes: HashMap<String, String>,
}

impl Span {
    pub fn new(operation: String, context: TraceContext) -> Self {
        Self {
            context,
            operation,
            started_at: Instant::now(),
            attributes: HashMap::new(),
        }
    }

    pub fn add_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
    }

    pub fn duration_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }

    pub fn finish(self) -> SpanRecord {
        SpanRecord {
            trace_id: self.context.trace_id,
            span_id: self.context.span_id,
            parent_span_id: self.context.parent_span_id,
            operation: self.operation,
            duration_ms: self.duration_ms(),
            attributes: self.attributes,
        }
    }
}

/// Completed span record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpanRecord {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation: String,
    pub duration_ms: u64,
    pub attributes: HashMap<String, String>,
}

// Add trace context to SyncMessage
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncMessage {
    SyncOffer {
        iblt_data: Vec<u8>,
        message_count: u32,
        sketch_capacity: u32,
        peer_proof: String,
        timestamp: u64,
        #[serde(default)]
        trace_context: Option<TraceContext>,
    },
    // ... other variants with trace_context
}
```

**Actions**:
1. Create `core/src/drift/tracing.rs`
2. Implement `TraceContext` for distributed tracing
3. Implement `Span` for operation tracking
4. Add `trace_context` field to `SyncMessage` variants
5. Propagate trace context through sync operations
6. Add unit tests for tracing
7. Export tracing types from drift module
8. Run `cargo test` in core/

**Verification**:
- [ ] cargo test passes
- [ ] Trace IDs propagate through operations
- [ ] Child spans reference parent spans
- [ ] Span duration is captured correctly

---

### Task 3.3: Add Health Check Endpoints (~100 LoC)

**Objective**: Provide health check API for monitoring systems

**Target Pattern**:
```rust
// core/src/drift/health.rs (new file)
use serde::{Deserialize, Serialize};

/// Health status of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Component is healthy
    Healthy,
    /// Component is degraded but functional
    Degraded,
    /// Component is unhealthy
    Unhealthy,
}

/// Health check result for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Component name
    pub component: String,
    /// Health status
    pub status: HealthStatus,
    /// Optional message with details
    pub message: Option<String>,
    /// Timestamp of check
    pub checked_at: u64,
}

/// Overall system health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Overall status (worst of all components)
    pub status: HealthStatus,
    /// Individual component health checks
    pub components: Vec<HealthCheck>,
    /// Timestamp of health check
    pub checked_at: u64,
}

impl SystemHealth {
    pub fn new() -> Self {
        Self {
            status: HealthStatus::Healthy,
            components: Vec::new(),
            checked_at: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn add_check(&mut self, check: HealthCheck) {
        // Update overall status to worst component status
        if check.status == HealthStatus::Unhealthy {
            self.status = HealthStatus::Unhealthy;
        } else if check.status == HealthStatus::Degraded && self.status == HealthStatus::Healthy {
            self.status = HealthStatus::Degraded;
        }
        self.components.push(check);
    }

    /// Check sync protocol health
    pub fn check_sync_health(metrics: &SyncMetrics, rate_limiter: &SyncRateLimiter) -> HealthCheck {
        let snapshot = metrics.snapshot();
        
        // Check for high failure rate
        let total_ops = snapshot.offers_sent + snapshot.responses_sent + snapshot.completions_sent;
        let failure_rate = if total_ops > 0 {
            (snapshot.sync_failures as f64) / (total_ops as f64)
        } else {
            0.0
        };

        let status = if failure_rate > 0.5 {
            HealthStatus::Unhealthy
        } else if failure_rate > 0.2 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let message = if failure_rate > 0.2 {
            Some(format!("High sync failure rate: {:.1}%", failure_rate * 100.0))
        } else {
            None
        };

        HealthCheck {
            component: "sync_protocol".to_string(),
            status,
            message,
            checked_at: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Check mesh store health
    pub fn check_store_health(store: &MeshStore) -> HealthCheck {
        let message_count = store.len();
        let max_capacity = 10_000; // From store.rs MAX_MESSAGES

        let status = if message_count >= max_capacity {
            HealthStatus::Unhealthy
        } else if message_count as f64 > (max_capacity as f64 * 0.8) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let message = if message_count >= max_capacity {
            Some(format!("Store at capacity: {}/{}", message_count, max_capacity))
        } else if message_count as f64 > (max_capacity as f64 * 0.8) {
            Some(format!("Store nearly full: {}/{}", message_count, max_capacity))
        } else {
            None
        };

        HealthCheck {
            component: "mesh_store".to_string(),
            status,
            message,
            checked_at: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}
```

**Actions**:
1. Create `core/src/drift/health.rs`
2. Implement `HealthStatus` enum
3. Implement `HealthCheck` struct
4. Implement `SystemHealth` aggregator
5. Add health check methods for sync and store
6. Add unit tests for health checks
7. Export health types from drift module
8. Run `cargo test` in core/

**Verification**:
- [ ] cargo test passes
- [ ] Health checks detect unhealthy states
- [ ] Overall status reflects worst component
- [ ] Health checks are serializable

---

### Task 3.4: Phase 3 Verification Gate (~10 LoC)

**Objective**: Verify all Phase 3 changes maintain zero regression

**Actions**:
1. Run `cargo check --workspace`
2. Run `cargo test --workspace`
3. Run `cargo clippy --workspace`
4. Run `cargo fmt --check`
5. Verify metrics are collected correctly
6. Verify trace context propagates
7. Verify health checks work
8. Run smoke test

**Verification**:
- [ ] cargo check passes
- [ ] cargo test passes
- [ ] cargo clippy passes
- [ ] cargo fmt --check passes
- [ ] Metrics collection functional
- [ ] Tracing functional
- [ ] Health checks functional
- [ ] Smoke test passes

---

## Critical Files to Review

### Before Starting

1. **core/src/drift/sync.rs** - Current sync protocol (will add metrics/tracing)
2. **core/src/drift/store.rs** - Mesh store (will add health checks)
3. **core/src/drift/mod.rs** - Module exports (will add new modules)

### Implementation Prompt Reference

- **HANDOFF/scmessenger_rust_implementation_prompt.md** - Detailed Phase 3 instructions

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

### Observability Best Practices

- Metrics should use atomic operations (thread-safe)
- Trace IDs should be UUIDs for uniqueness
- Health checks should be fast (<100ms)
- All observability data should be serializable

---

## Success Criteria

Phase 3 is complete when:

- [OK] Metrics collection added to sync operations
- [OK] Distributed tracing implemented
- [OK] Health check endpoints created
- [OK] All tests pass (cargo test)
- [OK] All crates compile (cargo check)
- [OK] Clippy passes
- [OK] Code formatted (cargo fmt)
- [OK] Metrics are collected correctly
- [OK] Trace context propagates
- [OK] Health checks work

---

## Getting Started

### Step 1: Verify Phase 2 Completion

```bash
cargo check --workspace
cargo test --lib -p scmessenger-core -- drift::sync drift::rate_limit
```

### Step 2: Start Task 3.1 (Metrics Collection)

```bash
# Create metrics module
touch core/src/drift/metrics.rs

# Check current sync protocol
cat core/src/drift/sync.rs
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
5. Observability overhead impacts performance significantly

---

## Quick Reference

### Current State (Post-Phase 2)

- **Rust version**: 1.95.0
- **Build status**: [OK] Passing
- **Test status**: [OK] 870+ tests passing
- **Schema versioning**: [OK] Implemented
- **Peer proofs**: [OK] Implemented
- **Rate limiting**: [OK] Implemented

### Phase 3 Focus

- **Metrics**: Atomic counters for sync operations
- **Tracing**: Distributed trace context propagation
- **Health checks**: Component health monitoring
- **Zero regression**: All tests must pass

---

## Estimated Work

**Total LoC:** ~400 LoC

- Task 3.1: ~150 LoC (metrics collection)
- Task 3.2: ~150 LoC (distributed tracing)
- Task 3.3: ~100 LoC (health checks)
- Task 3.4: ~10 LoC (verification)

---

## Prompt to Use in New Window

```
I'm continuing the SCMessenger Rust Transformation project. Phase 1 (Async Hygiene) and Phase 2 (Protocol Hardening) are complete. Please execute Phase 3: Observability - Metrics, Tracing, Health Checks.

**Context:**
- Spec location: docs/specs/scmessenger-rust-transformation/
- Phase 1 status: [OK] COMPLETE (locks optimized, HTTP API modernized)
- Phase 2 status: [OK] COMPLETE (schema versioning, peer proofs, rate limiting)
- Current Rust version: 1.95.0
- Build status: All tests passing (871 passed, 0 failed)

**Phase 3 Objectives:**
1. Add metrics collection for sync operations
2. Implement distributed tracing for message flow
3. Add health check endpoints for monitoring
4. Verify zero regression

**Instructions:**
- Read docs/specs/scmessenger-rust-transformation/PHASE3-KICKOFF.md for detailed instructions
- Follow tasks in docs/specs/scmessenger-rust-transformation/tasks.md (Phase 3 section)
- Reference HANDOFF/scmessenger_rust_implementation_prompt.md for implementation details
- Update task status as you progress
- Maintain zero-regression (all tests must pass)

Please start with Task 3.1: Add Metrics Collection for Sync Operations.
```

---

**Phase 3 Status: [PENDING] READY TO START**
