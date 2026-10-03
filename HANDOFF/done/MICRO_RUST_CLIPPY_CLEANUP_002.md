# MODEL: gemini-3-flash-preview:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

# BUDGET: 600
# token_budget: 6000

# MICRO_RUST_CLIPPY_CLEANUP_002

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder
**Budget:** 600s (MICRO tier)
**Phase:** v0.2.1 polish
**Source:** `.clippy.toml` disallowed-methods config blocks `unwrap` in `serde_json::json!` macro expansions

---

## Verified Gap

`cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments` reports 46 errors. ALL are `unwrap` calls originating from the `serde_json::json!` macro expansion. The `.clippy.toml` explicitly disallows `std::result::Result::unwrap` and `std::option::Option::unwrap`.

**Files with remaining errors (verified):**

| File | Error Count | Root Cause |
|------|-------------|------------|
| `cli/src/server.rs` | ~18 | New P0_JSONRPC dispatch handlers use `json!({"key": value})` in RPC responses |
| `cli/src/api.rs` | ~2 | Existing code uses `json!` macro |
| `wasm/src/lib.rs` | ~2 | Existing code uses `json!` macro |

**Example problematic pattern (server.rs):**
```rust
rpc_result(id, json!({"backup": backup}))
```
The `json!` macro internally calls `.unwrap()` which triggers the clippy lint.

## Scope

Replace ALL `serde_json::json!(...)` invocations in the affected files with explicit `serde_json::Value` construction using `serde_json::Map` or `serde_json::json!` alternatives that don't use unwrap.

**Approach:**
For simple objects like `json!({"backup": backup})`, replace with:
```rust
let mut map = serde_json::Map::new();
map.insert("backup".to_string(), serde_json::Value::String(backup));
serde_json::Value::Object(map)
```

Or for mixed types, build a `serde_json::Value` via `serde_json::to_value(...)`.

**Files to edit:**
- `cli/src/server.rs` [EDIT]  search for `json!(` and replace with explicit Value construction
- `cli/src/api.rs` [EDIT]  search for `json!(` and replace
- `wasm/src/lib.rs` [EDIT]  search for `json!(` and replace

## Build Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
cargo test --workspace --no-run
```

## Acceptance Gates

1. `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments` passes with zero warnings/errors
2. `cargo test --workspace --no-run` passes

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
