# TASK: U2 — Topic Constants Centralization

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: READY FOR QWE DELEGATION
Owner: Qwen (mechanical refactoring)
Scope: Unification U2 (prerequisite for F0 delivery-truth fixes)

## Objective

Define `TOPIC_LOBBY` and `TOPIC_MESH` constants once in `core/src/lib.rs`, replace all hardcoded instances across CLI/Android/iOS/WASM.

## Current State

Grep results show hardcoded topic strings in 3+ places:
- `cli/src/` — topic strings hardcoded in startup/routing
- `core/src/` — topic strings referenced but not centralized
- `android/` — Kotlin side references hardcoded strings

## Requirements

1. **Define constants in core:**
   ```rust
   pub const TOPIC_LOBBY: &str = "scm.lobby";
   pub const TOPIC_MESH: &str = "scm.mesh";
   ```
   Location: `core/src/lib.rs` (after `pub mod retry_policy`)

2. **Export via UniFFI:**
   - Add to `api.udl`: string constants (if FFI-exposed)
   - Verify Android/iOS can import via generated bindings

3. **Replace all hardcoded instances:**
   - Grep: `"scm\\.lobby"` + `"scm\\.mesh"` across repo
   - Replace with `TOPIC_LOBBY` / `TOPIC_MESH`
   - Update imports in CLI, Android, iOS

4. **Verification:**
   - `cargo build --workspace` (compile gate)
   - `cargo clippy --workspace` (no unused imports)
   - Grep: zero remaining hardcoded topic strings outside comments/tests

## Acceptance Criteria

- [ ] Constants defined in core/src/lib.rs
- [ ] All hardcoded instances replaced
- [ ] Compile gate passes
- [ ] No clippy warnings
- [ ] Grep confirms zero hardcoded topic strings
- [ ] Commit: `unification: U2 topic constants centralized`

## Estimated Time

15-20 minutes (grep + replace + verify)

## Blocking/Blocked

**Blocked by:** None
**Blocks:** U3 + F0 delivery-truth work (low priority)
