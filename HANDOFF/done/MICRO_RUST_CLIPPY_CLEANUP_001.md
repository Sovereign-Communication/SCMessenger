# MICRO_RUST_CLIPPY_CLEANUP_001

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder / triage-router
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 polish
**Source:** `cargo clippy --workspace` warnings

---

## Verified Gap

`cargo clippy --workspace` generates 74 errors across the workspace. The `.clippy.toml` disallows `Option::unwrap` and `Result::unwrap`, which triggers in macro expansions (e.g., `serde_json::json!`) and direct calls.

**Verified Code State (74 errors):**

### Category 1: unwrap in serde_json::json! macro expansions
- `core/src/iron_core.rs`  multiple lines around 1882-1905 (json! macro internal unwrap)

### Category 2: redundant closure / unnecessary lazy evaluations
- `core/src/iron_core.rs:1142`  redundant closure
- `core/src/iron_core.rs:1353`  unnecessary closure for Option::None
- `core/src/routing/global.rs:207`
- `core/src/routing/local.rs:207,230,342,345`
- `core/src/routing/neighborhood.rs:324`
- `core/src/store/backend.rs:43,49,53,59,71,83`
- `core/src/store/dedup.rs:83,99`
- `core/src/transport/observation.rs:52,99,172`
- `core/src/transport/mesh_routing.rs:141,207,332,354,604`
- `core/src/transport/peer_broadcast.rs:50,74,101`

### Category 3: Original task items (still pending)
- `core/tests/integration_offline_partition_matrix.rs`  unused imports
- `core/src/transport/swarm.rs:4929`  unused import `CustodyCompatMode` in test module
- `core/tests/integration_mycorrhizal_routing.rs:507`  useless comparison `stats_after.negative_checks >= 0`

## Scope

1. Fix all redundant closures (replace with direct function references where applicable)
2. Fix unnecessary lazy evaluations (replace `map_or_else(|| x, f)` with `map_or(x, f)` or similar)
3. Fix unwrap in `json!` macro usage  replace `json!({...})` with `serde_json::Value::from(...)` or build JSON values explicitly to avoid macro-internal unwrap
4. Remove unused imports from original task files
5. Fix useless comparison in integration_mycorrhizal_routing.rs

## File Targets

- `core/src/iron_core.rs` [EDIT]
- `core/src/routing/global.rs` [EDIT]
- `core/src/routing/local.rs` [EDIT]
- `core/src/routing/neighborhood.rs` [EDIT]
- `core/src/store/backend.rs` [EDIT]
- `core/src/store/dedup.rs` [EDIT]
- `core/src/transport/observation.rs` [EDIT]
- `core/src/transport/mesh_routing.rs` [EDIT]
- `core/src/transport/peer_broadcast.rs` [EDIT]
- `core/tests/integration_offline_partition_matrix.rs` [EDIT]
- `core/tests/integration_mycorrhizal_routing.rs` [EDIT]
- `core/src/transport/swarm.rs` [EDIT]

## Build Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
cargo test --workspace --no-run
```

## Acceptance Gates

1. `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments` passes with zero warnings/errors
2. `cargo test --workspace --no-run` still passes

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
