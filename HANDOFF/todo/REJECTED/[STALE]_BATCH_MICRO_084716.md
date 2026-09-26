# MODEL: gemini-3-flash-preview:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 180
# BATCHED_TASKS: 3
# STRIPPED_CONTEXT: true

# BATCH_INSTRUCTIONS:
# You are processing 3 MICRO tasks sequentially.
# For each TASK section below:
#   1. Apply the code change described
#   2. Move the original task file from todo/ to done/ (file name shown in each section)
#   3. Proceed to the next TASK section
# Do NOT run ./gradlew :app:assembleDebug for individual MICRO changes.
# Only run the build ONCE after all tasks are done, if you have time.
# If you run out of budget, stop cleanly -- remaining tasks stay in todo/.
# STRIPPED CONTEXT: You do NOT need to read CLAUDE.md in full.
# Relevant rules only: android.md (minSdk 26, compileSdk 35, Hilt, Compose).
---
## TASK: [VALIDATED]_MICRO_ANDROID_NOTIFICATION_STRINGS_001.md
# MICRO_ANDROID_NOTIFICATION_STRINGS_001

**Status:** VERIFIED REMAINING WORK
**Agent:** worker / triage-router
**Budget:** 300s (MICRO tier)
**Phase:** WS14.3 follow-up
**Source:** Static analysis of NotificationHelper.kt

---

## Verified Gap

`NotificationHelper.kt` contains hardcoded user-facing strings that violate android.md rules ("No hardcoded strings in UI â€” all user-facing text in `strings.xml`").

**Verified Code State:**
- `NotificationHelper.kt:330` â€” `.setLabel("Reply")` hardcoded
- `NotificationHelper.kt:335` â€” `"Reply"` hardcoded (action title)
- `NotificationHelper.kt:355` â€” `"Mark Read"` hardcoded (action title)
- `NotificationHelper.kt:371` â€” `"Mute"` hardcoded (action title)
- `NotificationHelper.kt:195` â€” `.setContentTitle("Mesh Network Active")` hardcoded (already in strings.xml as `mesh_service_notification_title` but NOT used)
- `NotificationHelper.kt:509` â€” `.setContentTitle("Peer Discovered")` hardcoded
- `NotificationHelper.kt:510` â€” `.setContentText("$peerId via $transport")` hardcoded format string

**Note:** The Requests Inbox strings were already added by the WS14.3 agent (see strings.xml lines 26-33).

## Scope

1. Add missing strings to `android/app/src/main/res/values/strings.xml`:
   - `notification_action_reply`
   - `notification_action_mark_read`
   - `notification_action_mute`
   - `notification_peer_discovered_title`
   - `notification_peer_discovered_format` (format string with placeholders)
2. Update `NotificationHelper.kt` to reference `R.string.*` for all hardcoded strings above
3. Verify `NotificationHelper.kt:195` uses existing `R.string.mesh_service_notification_title`

## File Targets

- `android/app/src/main/res/values/strings.xml` [EDIT]
- `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt` [EDIT]

## Build Verification Commands

```bash
cd android
./gradlew :app:compileDebugKotlin -q
```

## Acceptance Gates

1. `./gradlew :app:compileDebugKotlin` passes
2. Zero hardcoded user-facing strings remain in `NotificationHelper.kt` (grep `"[A-Z][a-z]` for title-case strings)
3. All notification action labels use `R.string.*` references

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.


---
## TASK: [VALIDATED]_MICRO_RUST_CLIPPY_CLEANUP_002.md
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
- `cli/src/server.rs` [EDIT] â€” search for `json!(` and replace with explicit Value construction
- `cli/src/api.rs` [EDIT] â€” search for `json!(` and replace
- `wasm/src/lib.rs` [EDIT] â€” search for `json!(` and replace

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


---
## TASK: [VALIDATED]_MICRO_RUST_RELAY_ONION_ENABLE_001.md
# MICRO_RUST_RELAY_ONION_ENABLE_001

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 test completeness
**Source:** `core/tests/integration_relay_onion.rs.disabled`

---

## Verified Gap

An integration test for relay onion routing exists but is disabled (`.disabled` suffix), meaning it is never compiled or run by CI.

**Verified Code State:**
- `core/tests/integration_relay_onion.rs.disabled` exists but is excluded from test compilation
- The test likely has compile errors or was disabled during a previous refactoring
- Relay onion routing is part of the `privacy/` module and should have integration test coverage

## Scope

1. Rename `core/tests/integration_relay_onion.rs.disabled` to `core/tests/integration_relay_onion.rs`
2. Run `cargo test --test integration_relay_onion -p scmessenger-core` to identify compile errors
3. Fix all compile errors (likely API mismatches from transport or privacy module changes)
4. If the test requires significant rework (>300s), document blockers and re-disable with TODO comment

## File Targets

- `core/tests/integration_relay_onion.rs.disabled` -> `core/tests/integration_relay_onion.rs` [RENAME]
- `core/tests/integration_relay_onion.rs` [EDIT to fix compile errors]

## Build Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo test --test integration_relay_onion -p scmessenger-core --no-run
cargo test --test integration_relay_onion -p scmessenger-core
```

## Acceptance Gates

1. File renamed from `.disabled` to `.rs`
2. `cargo test --test integration_relay_onion -p scmessenger-core --no-run` compiles
3. Test runs and passes (or passes after reasonable fixes)
4. If test requires major rework: document in file comment and re-disable with explanation

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.



