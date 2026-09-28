# P2_IOS_ContactManagerFix_TryBang_FFI_Crash_Risk

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P2
**Platform:** iOS
**Status:** TODO
**Source:** native sweep 2026-07-04 (independent re-verification pass, not covered by
`docs/release-readiness-2026-07-02.md` T14-T17 or any existing `HANDOFF/todo/*IOS*` file)

## Problem

`iOS/SCMessenger/SCMessenger/ContactManagerFix.swift` is a hand-written workaround
file (per its own header comment: "Fixed ContactManager implementation to work
around UniFFI generation issues") that extends the UniFFI-generated `ContactManager`
type with method implementations. Two of those implementations use `try!` on a
Rust FFI call, instead of the `try`/`rustCallWithError` pattern every other method
in the same extension uses:

```swift
// ContactManagerFix.swift:79-83
public func count() -> UInt32 {
    return try! rustCall() {
        uniffi_scmessenger_core_fn_method_contactmanager_count(self.uniffiClonePointer(), $0)
    }
}

// ContactManagerFix.swift:85-89
public func flush() {
    try! rustCall() {
        uniffi_scmessenger_core_fn_method_contactmanager_flush(self.uniffiClonePointer(), $0)
    }
}
```

Every other method on this same `extension ContactManager` (`add`, `get`, `list`,
`remove`, `search`, `setLocalNickname`, `setNickname`, `updateDeviceId`,
`updateLastSeen`) uses `rustCallWithError(FfiConverterTypeIronCoreError.lift())`
and propagates errors via Swift `throws`. `count()`/`flush()` use the plain
(non-error-lifting) `rustCall()` helper wrapped in `try!`, which crashes the whole
app if the underlying Rust call ever panics or returns an unexpected FFI status
(UniFFI's `rustCall()`  as opposed to `rustCallWithError()`  is documented
upstream as "panics on any Rust-side error," which is the correct choice only when
the corresponding Rust function is `#[uniffi::export]`ed as truly infallible).

Whether `count()`/`flush()` on `core/src/store/contacts.rs` `ContactManager` are
in fact infallible in the Rust source has NOT been confirmed in this sweep  that
is the open question this task should resolve before or during the fix.

## Why this matters

`ContactManagerFixed`/the extended `ContactManager` is the concrete type used
throughout the iOS app for all contact storage (`typealias ContactManager =
ContactManagerFixed` at the bottom of the file forces every callsite in the app
onto this type). `count()` is a plausible hot path (e.g. contact-list badge
counts, empty-state checks) and `flush()` is plausible on app-background/backup
paths. If the underlying sled-backed store ever hits an I/O error (disk full,
permission error, corrupted file) on either of these two specific calls, the
`try!` converts that into an unconditional app crash, whereas the exact same
class of failure on `add`/`get`/`list`/etc. degrades gracefully into a catchable
Swift `Error`.

## Fix Plan

1. Check `core/src/store/contacts.rs` (or wherever `ContactManager::count`/
   `ContactManager::flush` are defined and `#[uniffi::export]`ed) to determine
   whether the UniFFI codegen actually generates a throwing FFI signature for
   these two methods (i.e. whether `rustCallWithError` is even available/correct
   here) or whether they are genuinely declared as non-throwing in the `.udl`/
   proc-macro attributes.
2. If the Rust functions are non-throwing by design: this `try!` is technically
   safe under normal conditions, but still crashes on host-level FFI failures
   (e.g. lock poisoning inside `Arc<RwLock<...>>` if the Rust side ever panics
   internally). Consider whether that's acceptable, and if not, wrap in a
   Swift-side `do`/`catch` around the UniFFI panic-hook mechanism per project
   convention (check how other truly-infallible-but-defensive UniFFI calls are
   handled elsewhere in the iOS codebase, if any exist).
3. If the Rust functions ARE fallible (return `Result<...>` in Rust) but were
   miswired to the non-throwing `rustCall()` helper in this hand-written
   workaround file: switch both to `rustCallWithError(FfiConverterTypeIronCoreError.lift())`
   and update the Swift signatures to `throws` (this changes the
   `ContactManagerProtocol` conformance signature for `count()`/`flush()` 
   check `ContactManagerProtocol`'s declaration and all callers of
   `.count()`/`.flush()` across the iOS app to update them to `try`/`try?`
   appropriately).

## Files to Touch

- `iOS/SCMessenger/SCMessenger/ContactManagerFix.swift` [EDIT]  lines 79-89
- Possibly `core/src/store/contacts.rs` or wherever `ContactManager::count`/
  `flush` are defined, to confirm/adjust the Rust-side signature (read-only
  investigation first  do not change Rust API surface without confirming this
  is the right layer to fix)
- Any Swift callers of `.count()`/`.flush()` on `ContactManager` if the
  signature changes to `throws` (grep `iOS/` for `.count()` and `.flush()` on
  a `ContactManager`/`ContactManagerFixed` receiver)

## Verification

No Xcode/toolchain available in this sweep  this is a spec only. A
build-capable session should:
1. Confirm the Rust-side fallibility of `count`/`flush` first (read-only).
2. Apply whichever fix path applies from the plan above.
3. Build the iOS target and run existing contact-related unit/UI tests.

## Acceptance Criteria

- `count()`/`flush()` no longer crash the app on a genuine Rust-side error
  path, OR a clear, reviewed justification is documented (in a comment) for
  why `try!` is correct here specifically (e.g. "Rust API contractually
  cannot fail; verified against `core/src/store/contacts.rs` line N").
- Behavior of `count()`/`flush()` is consistent with the error-handling
  convention used by every other method in the same `extension ContactManager`
  block, unless a specific reason for the exception is documented.
