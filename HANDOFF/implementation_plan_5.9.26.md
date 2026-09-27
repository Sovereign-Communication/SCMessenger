# Resolve Phase 2 Routing & Core Compilation Errors

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Problem Statement

`cargo check -p scmessenger-core --all-features --all-targets` fails with **5 compilation errors** across 2 files. These errors are gated behind `#[cfg(feature = "phase2_apis")]` and only manifest when compiling with `--all-features`. The standard build (`cargo check -p scmessenger-core`) passes cleanly.

### Error Inventory

| # | Error Code | File | Line | Root Cause |
|---|-----------|------|------|------------|
| 1 | E0425 | `iron_core.rs` | 1832 | `path_id` referenced but parameter named `_path_id` |
| 2 | E0425 | `iron_core.rs` | 1832 | `latency_ms` referenced but parameter named `_latency_ms` |
| 3 | E0308 | `optimized_engine.rs` | 122 | `p.peer_id.to_be_bytes()` yields `[u8; 8]`, expected `[u8; 32]` |
| 4 | E0308 | `optimized_engine.rs` | 130 | Same as #3 on primary path |
| 5 | E0599 | `optimized_engine.rs` | 385 | `MultiPathDelivery` has no method `prune_below` |

Additionally, `optimized_engine.rs:113` contains a **latent panic**: `recipient_hint[..8]` slices a `[u8; 4]` — this would panic at runtime even if it compiled.

---

## Proposed Changes

### File 1: [iron_core.rs](file:///c:/Users/kanal/Documents/Github/SCMessenger/core/src/iron_core.rs)

> Fixes errors #1 and #2.

#### What's wrong

Line 1828 declares parameters with leading underscores (`_path_id`, `_latency_ms`) to suppress unused-variable warnings when `phase2_apis` is disabled. But line 1832 (inside a `#[cfg(feature = "phase2_apis")]` block) references them without underscores:

```rust
// L1828 — current
pub fn routing_register_path(&self, peer_id_hex: String, _path_id: u64, _latency_ms: u64) {
    // ...
    #[cfg(feature = "phase2_apis")]
    engine.multipath_register_path(peer_id_hex.clone(), path_id, latency_ms);
    //                                                   ^^^^^^^  ^^^^^^^^^^
    //                                                   E0425    E0425
}
```

#### Exact change

**Location:** Line 1828

**Before:**
```rust
    pub fn routing_register_path(&self, peer_id_hex: String, _path_id: u64, _latency_ms: u64) {
```

**After:**
```rust
    pub fn routing_register_path(&self, peer_id_hex: String, path_id: u64, latency_ms: u64) {
```

> [!NOTE]
> Removing underscores will trigger `unused_variable` warnings when `phase2_apis` is disabled. We suppress these inline with `let _ = path_id;` guards in the non-feature path, OR we use conditional compilation on the parameters themselves. The cleanest approach: keep the names without underscores and add `#[cfg(not(feature = "phase2_apis"))]` `let _ =` bindings:

**Full replacement block (L1828–L1834):**

```rust
    pub fn routing_register_path(&self, peer_id_hex: String, path_id: u64, latency_ms: u64) {
        if let Some(engine) = self.routing_engine.write().as_mut() {
            engine.record_message_activity(&peer_id_hex);
            #[cfg(feature = "phase2_apis")]
            engine.multipath_register_path(peer_id_hex.clone(), path_id, latency_ms);
        }
        #[cfg(not(feature = "phase2_apis"))]
        {
            let _ = (path_id, latency_ms);
        }
    }
```

**~5 LOC change.**

---

### File 2: [multipath.rs](file:///c:/Users/kanal/Documents/Github/SCMessenger/core/src/routing/multipath.rs)

> Fixes error #5 (missing `prune_below` method) and errors #3/#4 (type mismatch from `u64` peer IDs).

#### What's wrong

1. `DeliveryPath.peer_id` is `u64`. The routing engine's `NextHop::GlobalRoute` expects `next_hop_id: [u8; 32]` (aliased as `PeerId`). Code at `optimized_engine.rs:122` calls `p.peer_id.to_be_bytes()` which yields `[u8; 8]`, not the required `[u8; 32]`.

2. `MultiPathDelivery` has no `prune_below` method, but `optimized_engine.rs:385` calls `self.multipath.prune_below(threshold)`.

3. Lookup uses `HashMap<u64, ...>` but the caller only has a `[u8; 4]` hint — causing the broken `recipient_hint[..8]` slice at `optimized_engine.rs:113`.

#### Exact change — complete file rewrite

**Before:** 80-line stub with `peer_id: u64`, `HashMap<u64, Vec<DeliveryPath>>`, no `prune_below`.

**After:** Full implementation (~120 LOC) with:

```rust
//! Multi-path delivery (Phase 2 API)
//!
//! Tracks multiple delivery paths per recipient hint, scores them by
//! success rate and latency, and prunes underperforming routes.

use std::collections::HashMap;

/// Represents a delivery path for multi-path message routing
#[derive(Debug, Clone)]
pub struct DeliveryPath {
    /// Unique path identifier
    pub path_id: u64,
    /// Target peer ID (32-byte Ed25519 public key hash)
    pub peer_id: [u8; 32],
    /// Latency estimate in milliseconds (moving average)
    pub estimated_latency_ms: u64,
    /// Whether this path is currently active
    pub active: bool,
    /// Performance score (0.0–100.0), higher is better
    pub score: f64,
    /// Total delivery attempts through this path
    pub attempt_count: u64,
    /// Successful deliveries through this path
    pub success_count: u64,
}

impl DeliveryPath {
    /// Recalculate the performance score based on current statistics.
    ///
    /// Formula:
    ///   success_ratio = success_count / max(attempt_count, 1)
    ///   latency_penalty = min(estimated_latency_ms, 2000) / 100.0
    ///   score = clamp((success_ratio * 80.0) + 20.0 - latency_penalty, 0.0, 100.0)
    ///
    /// New paths with 0 attempts start at 75.0 (neutral-positive).
    pub fn recalculate_score(&mut self) {
        if self.attempt_count == 0 {
            self.score = 75.0;
            return;
        }
        let success_ratio = self.success_count as f64 / self.attempt_count.max(1) as f64;
        let latency_penalty = (self.estimated_latency_ms.min(2000) as f64) / 100.0;
        self.score = ((success_ratio * 80.0) + 20.0 - latency_penalty).clamp(0.0, 100.0);
    }

    /// Record a successful delivery, updating the moving average latency.
    pub fn record_success(&mut self, latency_ms: u64) {
        self.attempt_count += 1;
        self.success_count += 1;
        // Moving average: new_avg = (old_avg + sample) / 2
        self.estimated_latency_ms = (self.estimated_latency_ms + latency_ms) / 2;
        self.recalculate_score();
    }

    /// Record a failed delivery attempt.
    pub fn record_failure(&mut self) {
        self.attempt_count += 1;
        self.recalculate_score();
    }
}

/// Manages multi-path message delivery across redundant routes.
///
/// Paths are indexed by recipient hint (`[u8; 4]`) for O(1) lookup
/// from the routing engine's decision path.
#[derive(Debug, Clone)]
pub struct MultiPathDelivery {
    paths: HashMap<[u8; 4], Vec<DeliveryPath>>,
    max_paths_per_hint: usize,
}

impl Default for MultiPathDelivery {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiPathDelivery {
    /// Create a new multi-path delivery manager
    pub fn new() -> Self {
        Self {
            paths: HashMap::new(),
            max_paths_per_hint: 3,
        }
    }

    /// Register a delivery path for a recipient hint.
    ///
    /// If a path with the same `path_id` already exists under this hint,
    /// it is replaced. Otherwise the path is appended if under the limit.
    pub fn register_path(&mut self, hint: [u8; 4], path: DeliveryPath) {
        let paths = self.paths.entry(hint).or_default();
        // Update existing path with same path_id
        if let Some(existing) = paths.iter_mut().find(|p| p.path_id == path.path_id) {
            *existing = path;
            return;
        }
        if paths.len() < self.max_paths_per_hint {
            paths.push(path);
        }
    }

    /// Get all active paths for a recipient hint, sorted by score descending.
    pub fn active_paths(&self, hint: &[u8; 4]) -> Vec<&DeliveryPath> {
        let mut result: Vec<&DeliveryPath> = self
            .paths
            .get(hint)
            .map(|paths| paths.iter().filter(|p| p.active).collect())
            .unwrap_or_default();
        result.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    /// Mark a path as inactive (failed).
    pub fn mark_path_failed(&mut self, path_id: u64) {
        for paths in self.paths.values_mut() {
            if let Some(path) = paths.iter_mut().find(|p| p.path_id == path_id) {
                path.active = false;
                path.record_failure();
                break;
            }
        }
    }

    /// Prune paths whose score falls below the given threshold.
    ///
    /// Paths below threshold are deactivated (`active = false`) but
    /// retained in storage so their history is preserved for scoring.
    pub fn prune_below(&mut self, threshold: f64) {
        for paths in self.paths.values_mut() {
            for path in paths.iter_mut() {
                if path.score < threshold {
                    path.active = false;
                }
            }
        }
    }

    /// Get the number of recipient hints with registered paths
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Check if the delivery manager is empty
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}
```

**~130 LOC change (full file replacement).**

---

### File 3: [optimized_engine.rs](file:///c:/Users/kanal/Documents/Github/SCMessenger/core/src/routing/optimized_engine.rs)

> Fixes errors #3, #4, and the latent panic at line 113.

There are **three separate change sites** in this file:

#### Change 3A: Multipath lookup block (L110–L139)

**Root cause:** Line 113 tries `recipient_hint[..8]` on a `[u8; 4]` (panics). Lines 122 and 130 call `.to_be_bytes()` on a `u64` yielding `[u8; 8]` but `NextHop::GlobalRoute.next_hop_id` requires `[u8; 32]`.

**Before (L110–L139):**
```rust
        #[cfg(feature = "phase2_apis")]
        {
            let peer_id_int =
                u64::from_be_bytes(recipient_hint[..8].try_into().unwrap_or([0u8; 8]));
            let active = self.multipath.active_paths(peer_id_int);
            if let Some(primary_path) = active.first() {
                if primary_path.active {
                    let alternatives: Vec<NextHop> = active
                        .iter()
                        .skip(1)
                        .filter(|p| p.active)
                        .map(|p| NextHop::GlobalRoute {
                            next_hop_id: p.peer_id.to_be_bytes(),
                            total_hops: 1,
                        })
                        .collect();
                    return RoutingDecision {
                        message_id: *message_id,
                        recipient_hint: *recipient_hint,
                        primary: NextHop::GlobalRoute {
                            next_hop_id: primary_path.peer_id.to_be_bytes(),
                            total_hops: 1,
                        },
                        alternatives,
                        decided_by: RoutingLayer::Global,
                        confidence: 0.9,
                    };
                }
            }
        }
```

**After:**
```rust
        #[cfg(feature = "phase2_apis")]
        {
            let active = self.multipath.active_paths(recipient_hint);
            if let Some(primary_path) = active.first() {
                if primary_path.active {
                    let alternatives: Vec<NextHop> = active
                        .iter()
                        .skip(1)
                        .filter(|p| p.active)
                        .map(|p| NextHop::GlobalRoute {
                            next_hop_id: p.peer_id,
                            total_hops: 1,
                        })
                        .collect();
                    return RoutingDecision {
                        message_id: *message_id,
                        recipient_hint: *recipient_hint,
                        primary: NextHop::GlobalRoute {
                            next_hop_id: primary_path.peer_id,
                            total_hops: 1,
                        },
                        alternatives,
                        decided_by: RoutingLayer::Global,
                        confidence: 0.9,
                    };
                }
            }
        }
```

**Key differences:**
- Delete broken `u64::from_be_bytes(recipient_hint[..8]...)` — pass `recipient_hint` directly
- Replace `p.peer_id.to_be_bytes()` → `p.peer_id` (now `[u8; 32]`, matching `PeerId` directly)
- Replace `primary_path.peer_id.to_be_bytes()` → `primary_path.peer_id`

#### Change 3B: `active_paths` shim methods (L304–L315)

**Before:**
```rust
    #[cfg(feature = "phase2_apis")]
    pub fn active_paths(&self, peer_id: u64) -> Vec<&super::multipath::DeliveryPath> {
        self.multipath.active_paths(peer_id)
    }

    #[cfg(not(feature = "phase2_apis"))]
    pub fn active_paths(&self, _peer_id: u64) -> Vec<()> {
        Vec::new()
    }
```

**After:**
```rust
    /// Get active multipath delivery paths for a recipient hint.
    /// Returns an empty list when Phase 2 APIs are not enabled.
    #[cfg(feature = "phase2_apis")]
    pub fn active_paths(&self, hint: &[u8; 4]) -> Vec<&super::multipath::DeliveryPath> {
        self.multipath.active_paths(hint)
    }

    /// Get active multipath delivery paths for a recipient hint (stub when Phase 2 not enabled).
    #[cfg(not(feature = "phase2_apis"))]
    pub fn active_paths(&self, _hint: &[u8; 4]) -> Vec<()> {
        Vec::new()
    }
```

#### Change 3C: `multipath_register_path` (L407–L423)

**Before:**
```rust
    #[cfg(feature = "phase2_apis")]
    pub fn multipath_register_path(&mut self, peer_id_hex: String, path_id: u64, latency_ms: u64) {
        use super::multipath::DeliveryPath;
        let peer_id_hash = {
            let bytes = hex::decode(&peer_id_hex).unwrap_or_default();
            let arr: [u8; 8] = bytes[..8].try_into().unwrap_or([0u8; 8]);
            u64::from_le_bytes(arr)
        };
        let path = DeliveryPath {
            path_id,
            peer_id: peer_id_hash,
            estimated_latency_ms: latency_ms,
            active: true,
        };
        self.multipath.register_path(peer_id_hash, path);
    }
```

**After:**
```rust
    /// Register a delivery path in the multipath delivery manager (Phase 2).
    #[cfg(feature = "phase2_apis")]
    pub fn multipath_register_path(&mut self, peer_id_hex: String, path_id: u64, latency_ms: u64) {
        use super::multipath::DeliveryPath;
        let bytes = hex::decode(&peer_id_hex).unwrap_or_default();
        let peer_id: [u8; 32] = if bytes.len() >= 32 {
            bytes[..32].try_into().unwrap_or([0u8; 32])
        } else {
            let mut arr = [0u8; 32];
            arr[..bytes.len()].copy_from_slice(&bytes);
            arr
        };
        // Derive the 4-byte recipient hint from the peer ID (first 4 bytes of blake3 hash)
        let hint: [u8; 4] = blake3::hash(&peer_id).as_bytes()[0..4]
            .try_into()
            .unwrap_or([0u8; 4]);
        let path = DeliveryPath {
            path_id,
            peer_id,
            estimated_latency_ms: latency_ms,
            active: true,
            score: 75.0,
            attempt_count: 0,
            success_count: 0,
        };
        self.multipath.register_path(hint, path);
    }
```

**Key differences:**
- Decode full 32-byte peer ID (with safe fallback for shorter inputs)
- Derive `[u8; 4]` hint via `blake3::hash` (matches how hints are derived everywhere else in the codebase, e.g. `swarm.rs:1784`)
- Populate the new `score`, `attempt_count`, `success_count` fields

**~40 LOC change across 3 sites.**

---

### File 4: [nat_reflection_demo.rs](file:///c:/Users/kanal/Documents/Github/SCMessenger/core/examples/nat_reflection_demo.rs)

> Not part of the 5 errors (examples need `--all-targets` to compile). Fixes signature mismatch.

#### What's wrong

`start_swarm` takes 6 parameters (the 6th is `discovery_config: Option<DiscoveryConfig>`). The example only passes 5.

#### Exact change — 3 call sites

**Line 45 — Before:**
```rust
    let swarm1 = start_swarm(keypair1, None, event_tx1, None, false).await?;
```
**After:**
```rust
    let swarm1 = start_swarm(keypair1, None, event_tx1, None, false, None).await?;
```

**Line 48 — Before:**
```rust
    let swarm2 = start_swarm(keypair2, None, event_tx2, None, false).await?;
```
**After:**
```rust
    let swarm2 = start_swarm(keypair2, None, event_tx2, None, false, None).await?;
```

**Line 51 — Before:**
```rust
    let swarm3 = start_swarm(keypair3, None, event_tx3, None, false).await?;
```
**After:**
```rust
    let swarm3 = start_swarm(keypair3, None, event_tx3, None, false, None).await?;
```

**~3 LOC change.**

---

## Type System Trace

This section documents the chain of type dependencies to prove correctness:

```
NextHop::GlobalRoute { next_hop_id: PeerId, total_hops: u8 }
                                     ^^^^^^
                                     PeerId = [u8; 32]  (defined in routing/local.rs)

DeliveryPath.peer_id: [u8; 32]   ← NEW (was u64)
    → Used in optimized_engine.rs L122: next_hop_id: p.peer_id   [u8; 32] = [u8; 32]
    → Used in optimized_engine.rs L130: next_hop_id: primary_path.peer_id  

MultiPathDelivery.paths: HashMap<[u8; 4], Vec<DeliveryPath>>  ← NEW (was HashMap<u64, ...>)
    → Keyed by recipient hint [u8; 4]
    → Lookup in route_message_optimized: self.multipath.active_paths(recipient_hint)
    → recipient_hint is &[u8; 4]  

active_paths(&self, hint: &[u8; 4]) → Vec<&DeliveryPath>  ← NEW signature
    → Called from optimized_engine L114 with recipient_hint: &[u8; 4]  
    → Called from optimized_engine L308 shim  

register_path(&mut self, hint: [u8; 4], path: DeliveryPath)  ← NEW signature
    → Called from multipath_register_path with derived hint  

prune_below(&mut self, threshold: f64)  ← NEW method
    → Called from optimized_engine L385  
```

---

## Execution Order

These changes must be applied in this exact sequence to avoid intermediate compilation failures:

1. **multipath.rs** — Rewrite first (provides new types and methods)
2. **optimized_engine.rs** — Update consumers of multipath API (3 sites)
3. **iron_core.rs** — Fix parameter naming
4. **nat_reflection_demo.rs** — Fix example call sites

---

## Verification Plan

### Step 1: Targeted check (fast, ~30s)
```powershell
$env:CARGO_INCREMENTAL=0
cargo check -p scmessenger-core --all-features
```
Expected: `Finished` with 0 errors.

### Step 2: Full surface area check (~60s)
```powershell
$env:CARGO_INCREMENTAL=0
cargo check -p scmessenger-core --all-features --all-targets
```
Expected: `Finished` with 0 errors. This covers lib, tests, examples, and benches.

### Step 3: Standard build regression check (~30s)
```powershell
$env:CARGO_INCREMENTAL=0
cargo check -p scmessenger-core
```
Expected: `Finished` with 0 errors. Confirms no regression for the default feature set.

### Step 4: Workspace check (~60s)
```powershell
$env:CARGO_INCREMENTAL=0
cargo check --workspace
```
Expected: `Finished` — confirms cli, mobile, and wasm crates still compile.
