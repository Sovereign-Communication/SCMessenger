# TASK: D-05 Unwrap/Panic Hardening — RE-DISPATCH (Scope-Locked)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: REDISPATCH-READY (Fusion Lite verdict incorporated)
Model: Deepseek-v4-pro (upgraded for domain reasoning)
Budget: $0.10 (increased, per farm-sim criticality)
Tier: CODER-THINK hybrid

## Critical Scope Enforcement

Fusion Lite Panel (unanimous) ruled:
- Receipt FFI encoding/decoding is OUT OF SCOPE (belongs in A-04/A-05)
- Root cause of scope creep: Task confusion due to overlapping "receipt" terminology
- Solution: Provide explicit file list + IN/OUT OF SCOPE sections

### IN SCOPE — MUST FIX:

1. mobile_bridge.rs (FFI boundary: ALL exported methods must return Result)
   - Lines ~3000-3100 (tokio runtime init)
   - Any unwrap() / expect() / panic!() in FFI-facing functions
   - FFI boundary violations must propagate errors to caller, never panic

2. iron_core.rs (startup & initialization)
   - Lines ~1-500 (init paths, key loading)
   - Replace startup panics with error returns
   - Initialization failures should not crash, should return Err

3. core/src/crypto/ (all files)
   - Replace panics with checked logic OR documented invariant panics
   - If panic is truly unreachable/catastrophic, add `// SAFETY:` comment explaining why
   - Use `debug_assert!` for logic bugs, not `panic!()` for recoverable errors

4. core/src/store/ (all files)
   - Database/deserialization errors must return Result
   - Handle missing data gracefully (log + default or return Err)
   - No panics on data format issues

### OUT OF SCOPE — DO NOT TOUCH:

- [NO] Do NOT add new FFI struct definitions (e.g., Receipt dictionary)
- [NO] Do NOT modify api.udl (UDL is for A-04/A-05)
- [NO] Do NOT create new test files for data round-tripping
- [NO] Do NOT implement platform-specific logic (A-04/A-05 territory)
- [NO] Do NOT add new wrapper functions like encode_receipt_from_components

## Error Handling Patterns Required

### FFI Boundary Pattern:
```rust
// BEFORE (FORBIDDEN):
pub fn ffi_exported_fn() {
    let session = get_session().unwrap(); // FFI panic
    ...
}

// AFTER (REQUIRED):
pub fn ffi_exported_fn() -> Result<SomeType, String> {
    let session = get_session()
        .ok_or_else(|| "Session not found".to_string())?;
    ...
    Ok(result)
}
```

### Startup Path Pattern:
```rust
// BEFORE:
fn init_keys() {
    let keys = load_keys().expect("keys must exist"); // Panic on init failure
}

// AFTER:
fn init_keys() -> Result<Keys> {
    let keys = load_keys()
        .map_err(|e| anyhow::anyhow!("Key initialization failed: {}", e))?;
    Ok(keys)
}
```

### Crypto Invariant Pattern:
```rust
// BEFORE:
let secret = decrypt().unwrap(); // Unclear if recoverable

// AFTER (Option 1 - Recoverable):
let secret = decrypt()
    .map_err(|e| {
        tracing::warn!("Decryption failed (recoverable): {}", e);
        // Continue with fallback
        DEFAULT_SECRET
    })?;

// AFTER (Option 2 - Unreachable Invariant):
// SAFETY: This code is reached only after key_validation() call (line 123).
// If decrypt fails here, it's a logic bug, not a runtime error.
let secret = decrypt()
    .expect("decrypt must succeed after validation");
```

### Storage Pattern:
```rust
// BEFORE:
let blob = store.get("key").unwrap(); // Panic on missing data

// AFTER:
let blob = store.get("key")
    .map_err(|e| anyhow::anyhow!("Store access failed: {}", e))?
    .ok_or_else(|| anyhow::anyhow!("Key not found in store"))?;
```

## Verification Steps

1. Compile Gate:
   ```bash
   cargo check --workspace
   cargo test --workspace --no-run
   ```

2. Grep Verification:
   ```bash
   grep -rn "\.unwrap()\|\.expect(" core/src/ --include="*.rs" \
     | grep -v "test\|SAFETY\|// "
   ```
   Result should be EMPTY in target files (mobile_bridge, iron_core, crypto/*, store/*)

3. Diff Review:
   - All changes must be error-handling transformations
   - No new data structures, no new test files
   - No UDL changes

## Success Criteria

- [PASS] Diff applies cleanly via `--mode diff --apply --verify "cargo check --workspace"`
- [PASS] Zero new `.unwrap()` or `.expect()` in target files post-fix
- [PASS] All new `panic!()` have `// SAFETY:` comments explaining invariant
- [PASS] No new function definitions or UDL modifications
- [PASS] No test files for receipt encoding/decoding
- [PASS] `cargo test --workspace --no-run` passes (compile gate)
- [PASS] All existing tests still pass (no regressions)

## Handoff

Move this file to `HANDOFF/done/D-05_UNWRAP_PANIC_HARDENING_REDISPATCH.md` ON COMPLETION via `mv` command.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `IN_PROGRESS/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Diff Mode

Expected output format:
```
--- a/core/src/mobile_bridge.rs
+++ b/core/src/mobile_bridge.rs
@@ -XXXX,Y +XXXX,Z @@
-    .unwrap()
+    .ok_or_else(|| anyhow::anyhow!("..."))?
```

Apply via: `git apply --recount`

## Execution

Route to Deepseek-v4-pro with `--mode diff --apply --verify "cargo check --workspace"`.
Budget: $0.10. No override on model selection.

---

# IMPLEMENTATION NOTES / SAFETY-DECISION AUDIT TRAIL

Status: CODE COMPLETE (scope-clean slice) — COMPILE GATE UNVERIFIED. Left in
IN_PROGRESS deliberately; not moved to done/ (see "Open items" below).

## Method

Surveyed all `unwrap()` / `expect(` / `panic!` in the four target zones and
classified each as either inside a `#[cfg(test)]` module (out of scope; the
verification grep filters test code) or a genuine production-path site. Only
production-path sites were touched. Each was then triaged into one of three
buckets:

- RECOVERABLE  -> return `Result` / `Err` (caller can handle it).
- INVARIANT (removable) -> refactor to non-panicking logic that upholds the
  invariant by construction (entry API, local binding, `match`).
- INVARIANT (irreducible) -> keep the panic, document with `// SAFETY:` (the
  function cannot return an error, or a borrow-checker constraint forces a
  re-fetch that can never fail).

## Per-site decisions

| Zone / file | Site(s) | Decision | Rationale |
|---|---|---|---|
| `crypto/negotiation.rs` | `max().unwrap()` (~L26) | RECOVERABLE -> `match ... None => return Err(CryptoError)` | Empty suite intersection is a real negotiation failure reachable from peer input; the prior `is_empty()` guard + `unwrap()` was redundant. Folded into one `match`. |
| `crypto/ratchet.rs` | `pq_our_keypair.as_ref().unwrap()` (~L901) | INVARIANT (removed) | Derive `new_encaps_key` from a local `new_keypair` before moving it into `self.pq_our_keypair`; removes the re-borrow-after-assign entirely. Behavior identical. |
| `crypto/session_manager.rs` | 4x insert-then-`get_mut().expect("session just inserted")` | INVARIANT (removed) | Rewrote as `HashMap::entry(key)` with `Occupied`/`Vacant` arms returning `into_mut()` / `insert()`. `Vacant` arm carries the fallible init (`?`) so a failed init inserts nothing (same as before). No panic path remains. |
| `store/backend.rs` | 14x lock guards + 1 oneshot | RECOVERABLE -> `map_err(...)?` | Backend uses `std::sync::RwLock` (NOT parking_lot -- the old `"parking_lot RwLock never poisons"` message was factually wrong; std locks DO poison). Every method already returns `Result<_, String>`, so a poisoned lock now propagates `Err("storage lock poisoned: ...")`. WASM `new_sync` returns `Err` on a dropped oneshot instead of panicking. |
| `store/dedup.rs` | 2x `get(id).expect("entry just ...")` | INVARIANT (irreducible, documented) | `record_received` returns `&DedupStats` (no `Result` channel). The immutable re-fetch exists solely because the `&mut` from `get_mut` cannot be returned across the branch, and the just-inserted key cannot be absent. Added `// SAFETY:` docs. Note: the classic conditional-return-of-borrow refactor does NOT compile on stable (non-Polonius), which is why the re-fetch pattern is retained. |
| `mobile_bridge.rs` | 2x tokio runtime-init `expect` (~L3065/3077) | INVARIANT (irreducible, documented) | `get_global_runtime()` is an internal helper feeding the `#[uniffi::export]` constructor and returns `tokio::runtime::Handle`, not `Result`. If both multi-thread and current-thread runtimes fail to build, the process cannot function -- a documented catastrophic panic. Converting to `Result` would require an api.udl / FFI-signature change, which is OUT OF SCOPE. `// SAFETY:` docs already present (from the prior attempt). |
| `iron_core.rs` | (none in L1-500) | NO CHANGE | Zero production unwrap/expect/panic in the scoped startup range. The lone `.expect("identity must be initialized")` at ~L2701 is inside a `#[doc(hidden)] test_only_identity_signing_key()` helper -- out of the scoped range and effectively test-only. Left untouched. |
| `crypto/proptest_harness.rs` | 15x | NO CHANGE | Whole module is `#[cfg(test)] mod proptest_harness;` -- compiles only under test. Out of scope; converting property-test `unwrap()`s to `Result` would defeat the test assertions. |

## Policy applied to "documented invariant panic" vs "recoverable"

A site was treated as RECOVERABLE only when the enclosing function already
exposes an error channel (`Result`) AND the failure is reachable from data /
peer input / resource exhaustion the caller could plausibly handle. A site was
kept as a documented INVARIANT panic only when (a) the function has no error
channel and changing it is out of scope, or (b) the "failure" is logically
unreachable and only present to satisfy the borrow checker. Every retained
panic has a `// SAFETY:` comment naming the invariant and where it is
established.

## Verification performed

- Grep classification of all four zones: every remaining non-test
  `unwrap`/`expect` in the target files is now either a documented `// SAFETY:`
  invariant (`dedup.rs` x2, `mobile_bridge.rs` x2) or inside a `#[cfg(test)]`
  module. No undocumented production panics remain in scope.
- Static review of each edit for type / borrow / syntax correctness
  (entry-API return types, `?` error-type coercion in `Result<_, String>` and
  `anyhow::Result` contexts, temporary-lifetime of lock guards in `for` heads).

## Open items (why this is still IN_PROGRESS)

1. COMPILE GATE NOT RUN. The executing environment had no Rust toolchain, so
   `cargo check --workspace` and `cargo test --workspace --no-run` were not
   executed. Run locally before promoting to done/:
   ```
   export CARGO_INCREMENTAL=0
   cargo check --workspace
   cargo test --workspace --no-run
   cargo build -p scmessenger-wasm --target wasm32-unknown-unknown
   ```
2. PRE-EXISTING OUT-OF-SCOPE CHANGES REMAIN IN THE WORKING TREE (left in place
   by owner decision): `core/src/api.udl` (new `Receipt` dictionary +
   `encode_receipt_from_components` / `decode_receipt_to_components`
   declarations) and the matching wrappers in `core/src/lib.rs`. These are
   exactly the receipt scope-creep this redispatch was created to exclude, and
   `lib.rs` references `message::Receipt` / `message::types::{encode,decode}_receipt`
   which may not yet exist (possible compile blocker unrelated to the hardening
   slice). They must be reverted, stashed, or routed to A-04/A-05 before the
   D-05 change set can satisfy this task's "no UDL / no new wrappers" criteria.
3. Also present from the prior attempt and considered in-scope/benign hardening
   (kept): `crypto/encrypt.rs` (`unwrap`/`expect` -> `ok_or_else(...)?`),
   `identity/keys.rs` (`unwrap` -> documented `expect`; note `identity/` is just
   outside the four named zones). Re-verify these compile as part of item 1.
