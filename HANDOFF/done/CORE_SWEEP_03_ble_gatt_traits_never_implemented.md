# TASK: CORE-SWEEP-03 [NEEDS PLANNING] — `GattServer`/`GattClient` traits in `core/src/transport/ble/gatt.rs` have zero implementations anywhere

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Context

Found during a comprehensive gap sweep of `core/src/` (2026-07-04). This
overlaps with (re-verifies and updates) a finding in
`HANDOFF/ACTIVE_LEDGER.md` (dated 2026-05-13), which listed `ble/gatt.rs`
`on_read`/`on_write` as "Stub-Only / Test-Only (no production call paths
found)". Per this task's instructions to re-verify against current source
rather than trust the old date: **this is still true today.**

`core/src/transport/ble/gatt.rs` defines:

```rust
pub trait GattServer: Send + Sync {
    fn on_write(&mut self, characteristic: GattCharacteristic, data: &[u8]) -> Result<(), GattError>;
    fn on_read(&self, characteristic: GattCharacteristic) -> Result<Vec<u8>, GattError>;
    fn notify(&mut self, characteristic: GattCharacteristic, data: &[u8]) -> Result<(), GattError>;
    fn is_enabled(&self) -> bool;
}

pub trait GattClient: Send + Sync {
    fn write(&mut self, characteristic: GattCharacteristic, data: &[u8]) -> Result<(), GattError>;
    // ...
}
```

A repo-wide grep for `impl.*GattServer` / `impl.*GattClient` finds **zero
implementations** anywhere in `core/`, `cli/`, `desktop_bridge/`,
`android/`, or `iOS/`. Meanwhile, the Android app has its own, completely
separate, working BLE GATT implementation directly in Kotlin:
`android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt`
and `BleGattClient.kt`, using Android's native
`BluetoothGattServerCallback` — it does not call into or implement this
Rust trait at all.

This means the Rust `GattServer`/`GattClient` trait abstraction in
`core/src/transport/ble/gatt.rs` is currently pure dead API surface: no
platform (Android, desktop via `desktop_bridge/`, iOS) has ever wired a
real implementation through it. The BLE queue/reassembly helper types in
the same module (`GattWriteRequest` queue, etc.) DO have real logic and
tests, but the trait itself that's supposed to let platform code plug into
it has no plugs.

## Why this needs planning, not a direct fix

This is an **architectural question, not a bug fix**:

1. Is this Rust-side `GattServer`/`GattClient` abstraction still the
   intended cross-platform design (e.g. meant to eventually be used by
   `desktop_bridge/`'s own BLE support, or a future iOS BLE integration),
   in which case it should stay and get a real implementation wired in?
2. Or has the actual architecture already diverged to "each platform
   implements its own native BLE GATT stack and only talks to the Rust core
   via higher-level transport events" (which is what Android currently
   does) — in which case this trait is vestigial from an earlier design and
   should be removed (or explicitly marked as a documented-but-unused
   reference interface) rather than eventually implemented?

Given `.claude/rules/security.md`'s Adversarial Review Protocol explicitly
lists `core/src/transport/` (BLE, relay, QUIC paths) as requiring
mandatory `crypto-security-auditor` review before merge, and given BLE
GATT is a network-facing attack surface (untrusted peer devices write to
GATT characteristics), **whichever direction is chosen — implement or
remove — the change must go through the crypto-security-auditor subagent
before being considered mergeable.** Do not skip this step even if the fix
looks purely mechanical (e.g. "just delete the unused trait") — deleting
security-relevant abstraction can itself be a decision with security
implications if something was relying on its absence being noticed later.

**What's uncertain (do not guess):**
- Whether desktop Linux/Windows BLE (via `desktop_bridge/`) is expected to
  use this trait per `HANDOFF/DESKTOP_BRIDGE_WIRING_SPEC.md` — check that
  spec and the concurrent subagent's actual `desktop_bridge/src/ble.rs`
  changes (already landed) for whether they reference `GattServer`/
  `GattClient` or implement BLE a different way.
- Whether removing this trait would break any planned-but-not-yet-landed
  work referenced elsewhere in `HANDOFF/` (search `HANDOFF/todo/` and
  `HANDOFF/IN_PROGRESS/` for `GattServer`, `GattClient`, `gatt.rs` before
  deciding).
- Whether `docs/CURRENT_STATE.md` or `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  describe an intended future use for this trait that would argue for
  keeping and eventually implementing it rather than deleting it.

## Acceptance Criteria (once the direction is decided by a human or a planning pass)

- If implementing: at least one real platform implementation of both
  traits exists and is exercised by an integration test; mandatory
  crypto-security-auditor review completed and findings addressed.
- If removing/deprecating: the trait and any now-unreachable supporting
  code is removed or clearly marked with a doc comment explaining it's
  superseded by platform-native BLE GATT (Android's `BleGattServer.kt`
  pattern), and the `HANDOFF/ACTIVE_LEDGER.md` / dead-code-triage records
  are reconciled to reflect the decision so a future sweep doesn't re-flag
  the same thing.

## Files Likely Involved

- `core/src/transport/ble/gatt.rs`
- `desktop_bridge/src/ble.rs` (check current state post-wiring-spec)
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt`, `BleGattClient.kt` (reference only, not to be modified without separate Android-scoped review)

## Verification Commands (once a direction is chosen)

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test -p scmessenger-core transport::ble
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
```

Then invoke the `crypto-security-auditor` subagent per
`.claude/rules/security.md` before merge, regardless of which direction is
taken.
