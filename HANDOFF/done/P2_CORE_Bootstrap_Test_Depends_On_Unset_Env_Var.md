# P2_CORE_Bootstrap_Test_Depends_On_Unset_Env_Var

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P2
**Platform:** Core (Rust, test hygiene)
**Status:** TODO
**Source:** native /scmorc session 2026-07-06, discovered incidentally while
verifying unrelated tasks (`cargo test -p scmessenger-core --lib`)

## Problem

`core/src/transport/bootstrap.rs`'s `test_bootstrap_manager_creation` asserts
`mgr.total_nodes() > 0` for a `BootstrapManager::with_defaults()` instance.
This is **not hermetic**: `total_nodes()` depends on
`CORE_BOOTSTRAP_NODES` (a hardcoded const, currently `&[]` -- intentionally
empty per "sovereign mode," no centralized bootstrap domain, per an adjacent
code comment) chained with `resolve_env_bootstrap_nodes()`, which reads the
`SC_BOOTSTRAP_NODES` environment variable at line ~500. With both empty (the
const is empty by design, and `SC_BOOTSTRAP_NODES` is unset in a fresh shell),
`total_nodes()` is legitimately `0`, and the test fails:

```
thread 'transport::bootstrap::tests::test_bootstrap_manager_creation' panicked at
core\src\transport\bootstrap.rs:565:9:
assertion failed: mgr.total_nodes() > 0
```

This means the test's pass/fail outcome depends on whatever happened to be in
the calling shell's environment when it was last verified, not on the code
under test -- a classic non-hermetic test bug. It is unrelated to any
Rust changes landed in this session (verified: none of the session's diffs
touch `transport/bootstrap.rs`).

## Fix Plan

Either:
- (a) Seed the test with an explicit non-empty node list (construct a
  `BootstrapConfig`/inject nodes directly rather than relying on
  `with_defaults()`'s environment-dependent resolution), so the test verifies
  actual `BootstrapManager` construction logic without depending on shell
  state, or
- (b) If the intent was specifically to verify the "sovereign mode, zero
  centralized defaults" behavior, invert the assertion to
  `assert_eq!(mgr.total_nodes(), 0)` when `SC_BOOTSTRAP_NODES` is confirmed
  unset (guard the test or make it explicitly set/clear the env var itself
  for hermeticity, e.g. `std::env::remove_var`/`set_var` scoped to the test
  -- note Rust env var mutation in tests needs care around parallel test
  execution races; consider `serial_test` or an explicit non-env-dependent
  constructor path instead).

Prefer (a) unless investigation of the test's original intent (git blame /
surrounding test module) shows (b) was the actual goal.

## Files to Touch

- `core/src/transport/bootstrap.rs` (test module only)

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo test -p scmessenger-core --lib transport::bootstrap
```
Run twice: once with `SC_BOOTSTRAP_NODES` unset, once with it set to a sample
value, confirming the test passes identically both times (true hermeticity).

## Acceptance Criteria

- `test_bootstrap_manager_creation` passes regardless of whether
  `SC_BOOTSTRAP_NODES` is set in the calling environment.
- No change to `BootstrapManager`'s actual runtime behavior -- test-only fix.

---
**FIXED 2026-07-07 (orchestrator, option a):** test_bootstrap_manager_creation now seeds an explicit node via add_bootstrap_node before asserting total_nodes() > 0, no longer depends on SC_BOOTSTRAP_NODES shell state. Gate: cargo test -p scmessenger-core --lib bootstrap:: - 27/27 PASS.
