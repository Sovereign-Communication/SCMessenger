# TASK: CORE-SWEEP-02  `IronCore::contacts_manager()`/`history_manager()` panic if the empty-path fallback also fails

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Context

Found during a comprehensive gap sweep of `core/src/` (2026-07-04), scoped to
find anything NOT already covered by the PQC workstream, the 39-item
dead-code triage, or the T1-T7/S4-S7 release-readiness fixes.

`core/src/iron_core.rs`, two UniFFI bridge accessors used by mobile
(Android/iOS) clients:

```rust
// ~line 1855
pub fn contacts_manager(&self) -> crate::contacts_bridge::ContactManager {
    let path = self.storage_path.clone().unwrap_or_default();
    crate::contacts_bridge::ContactManager::new(path.clone())
        .or_else(|_| crate::contacts_bridge::ContactManager::new(path))
        .unwrap_or_else(|e| {
            tracing::error!("Failed to create contact manager: {:?}", e);
            crate::contacts_bridge::ContactManager::new("".to_string())
                .expect("ContactManager fallback also failed")
        })
}

// ~line 1945, identical shape
pub fn history_manager(&self) -> crate::mobile_bridge::HistoryManager {
    let path = self.storage_path.clone().unwrap_or_default();
    crate::mobile_bridge::HistoryManager::new(path.clone())
        .or_else(|_| crate::mobile_bridge::HistoryManager::new(path))
        .unwrap_or_else(|e| {
            tracing::error!("Failed to create history manager: {:?}", e);
            crate::mobile_bridge::HistoryManager::new("".to_string())
                .expect("HistoryManager fallback also failed")
        })
}
```

Both functions try the real storage path, retry the same path once, and if
that still fails, fall back to constructing the manager against an empty
(`""`) path as a last resort  and if THAT also fails, they `.expect()`,
panicking the whole process. This is reachable on real (non-programmer-error)
conditions: a genuinely unwritable/permission-denied filesystem, out of
disk space, a sandboxed mobile environment where even the fallback
default path can't be opened, or a concurrent sled lock held by another
process instance. On mobile this means a storage failure  which a user
could plausibly hit (full disk, revoked storage permission mid-session,
restrictive OS sandbox)  crashes the entire app instead of surfacing a
recoverable error to the Kotlin/Swift caller.

These are plain `IronCore`/bridge-layer accessors, not `crypto/`,
`transport/`, `routing/`, or `privacy/`  the mandatory
crypto-security-auditor adversarial review does not apply, but this
function is on the UniFFI mobile bridge boundary, so any signature change
must be checked against Android/iOS callers (`android/`, `iOS/` Kotlin/Swift
sources) before landing.

## Acceptance Criteria

- Neither function panics if all three construction attempts
  (real path, retry, empty-path fallback) fail.
- Decide and implement one of:
  - (a) Change the return type to `Result<ContactManager, IronCoreError>` /
    `Result<HistoryManager, IronCoreError>` and propagate the error to
    callers  check the UniFFI `.udl` interface definitions
    (`core/src/api.udl` or wherever these are exposed) and all Kotlin/Swift
    call sites first, since this is a breaking signature change across the
    UniFFI boundary (mobile apps would need updated bindings + call-site
    handling).
  - (b) Keep the non-`Result` signature (avoiding an API-breaking change)
    but make the innermost fallback infallible instead of `.expect()`-able
     e.g. construct an in-memory-only/no-op manager variant that can never
    fail to construct (if `ContactManager`/`HistoryManager` support an
    in-memory backend already, or if adding one is small; check
    `contacts_bridge.rs` and `mobile_bridge.rs` for an existing in-memory
    test-only variant that could be promoted to a real fallback).
  - Pick (b) unless investigation shows (a) is trivial and low-blast-radius
    (e.g. these two functions already have very few call sites). State
    which was chosen and why in the PR/commit notes.
- Add a test that forces all three construction attempts to fail (e.g. by
  pointing `storage_path` at a location that can't be created, if the test
  harness allows it cross-platform; if not fully forceable in a unit test,
  at minimum add a test asserting the empty-path fallback branch itself
  doesn't panic when constructed directly) and assert no panic occurs.

## Implementation Plan

1. Grep `contacts_manager()` and `history_manager()` call sites across
   `core/`, `cli/`, `android/`, `iOS/`, `wasm/` to size the blast radius of
   an option-(a) signature change.
2. Check `core/src/contacts_bridge.rs` and `core/src/mobile_bridge.rs` for
   whether `ContactManager::new` / `HistoryManager::new` already support (or
   could cheaply support) a genuinely infallible in-memory mode.
3. Implement the chosen approach; update UniFFI bindings
   (`cargo run -p scmessenger-core --features gen-bindings --bin gen_kotlin`
   and `gen_swift`) if the signature changed, and update any Kotlin/Swift
   call sites accordingly.
4. Add the test described above.

## Files to Touch

- `core/src/iron_core.rs`
- `core/src/contacts_bridge.rs` and/or `core/src/mobile_bridge.rs` (if adding an infallible fallback mode)
- Possibly `core/src/api.udl`, generated bindings, and Android/iOS call sites (only if option (a) is chosen)

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo test -p scmessenger-core iron_core
cargo build --workspace
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
```

If UniFFI signatures changed, also run:
```bash
cargo run -p scmessenger-core --features gen-bindings --bin gen_kotlin
cargo run -p scmessenger-core --features gen-bindings --bin gen_swift
cd android && ./gradlew assembleDebug -x lint --quiet
```
and confirm iOS build via Xcode workspace if changed there too.
