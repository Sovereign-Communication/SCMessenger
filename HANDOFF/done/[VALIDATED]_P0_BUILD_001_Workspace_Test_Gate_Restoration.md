# MODEL: glm-5.1:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 1800
# token_budget: 18000

# P0_BUILD_001_Workspace_Test_Gate_Restoration

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder (A1, A2, A3) + worker (A4)
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 P0 gate restoration
**Source:** HANDOFF/ACTIVE_LEDGER.md (2026-05-13 sweep) + planfromclaudeforhermes 2 Phase A
**Depends on:** P0_SETUP_001 (Ollama models + Hermes config must be live)
**Blocks:** All P0_SECURITY_* and P1_CORE_* tasks

---

## Verified Gap

`cargo test --workspace --no-run` fails with 10 compile errors that look like ICEs but are actually stale integration-test imports. Per `HANDOFF/ACTIVE_LEDGER.md`:

> "Root cause: ICEs cascade from `integration_registration_protocol.rs` importing symbols (`IdentityKeys`, `DeregistrationRequest`, `RegistrationRequest`) that don't exist in the public API or have been renamed/removed."

Also: 1 unused import warning in `wasm/src/transport.rs:17`. No CI gate for clippy.

## Scope  4 sub-tasks, ~80 LoC total

### A1: Fix integration_registration_protocol.rs (LOC: ~30)

**File:** `core/tests/integration_registration_protocol.rs`

Open the file and find imports of:
- `IdentityKeys`  renamed/removed; replace with current public-API symbol (likely `identity::Keypair` or `identity::PublicKey`)
- `DeregistrationRequest`  replaced; check `relay_custody.rs` for current message types
- `RegistrationRequest`  same; check `relay_custody.rs:488` `get_registration_state_info` for current surface
- `SwarmEvent2`  replaced; check `swarm.rs` for current event enum name
- `start_swarm`  replaced; check `iron_core.rs:2566` for current initialization entry

**Verification:** `cargo check --tests -p scmessenger-core` shows 0 errors for this file.

### A2: Resolve cascade errors in 9 dependent test files (LOC: ~40)

Files with cascading errors that should auto-resolve when A1 lands:
- `core/tests/integration_ironcore_roundtrip.rs`
- `core/tests/integration_contact_block.rs`
- `core/tests/integration_e2e.rs`
- `core/tests/test_mesh_routing.rs`
- `core/tests/test_address_observation.rs` (independent "1 prior error" per ledger)
- `core/tests/property_tests.rs`
- `cli/tests/integration.rs`
- `wasm/src/lib.rs` (lib test)
- `core/examples/nat_reflection_demo.rs`

For each: run `cargo check --tests` and address remaining errors. Some may be independent; document any that require further work.

**Verification:** `cargo test --workspace --no-run` shows 0 errors.

### A3: Remove unused import (LOC: ~1)

**File:** `wasm/src/transport.rs:17`

Remove `use std::sync::Arc;` if no other usage in file. Verify with `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown`.

### A4: Add clippy gate to CI (LOC: ~20)

**File:** `.github/workflows/ci.yml` (create if missing, else edit)

Add a job that runs:
```yaml
- name: Cargo clippy strict
  run: cargo clippy --workspace --lib --bins --examples -- -D warnings -A clippy::empty_line_after_doc_comments
```

## File Targets

- `core/tests/integration_registration_protocol.rs` [EDIT]
- `core/tests/integration_ironcore_roundtrip.rs` [EDIT, likely cascade-only]
- `core/tests/integration_contact_block.rs` [EDIT, likely cascade-only]
- `core/tests/integration_e2e.rs` [EDIT, likely cascade-only]
- `core/tests/test_mesh_routing.rs` [EDIT, likely cascade-only]
- `core/tests/test_address_observation.rs` [EDIT, may have independent fix]
- `core/tests/property_tests.rs` [EDIT, likely cascade-only]
- `cli/tests/integration.rs` [EDIT, likely cascade-only]
- `wasm/src/lib.rs` [EDIT, likely cascade-only]
- `core/examples/nat_reflection_demo.rs` [EDIT, likely cascade-only]
- `wasm/src/transport.rs` [EDIT, remove unused import]
- `.github/workflows/ci.yml` [NEW or EDIT]

## Build Verification Commands

```bash
# After A1-A3
cargo check --workspace 2>&1 | tee /e/build-tools/logs/check-$(date +%Y%m%d).log
cargo test --workspace --no-run 2>&1 | tee /e/build-tools/logs/test-norun-$(date +%Y%m%d).log

# After A4
cargo clippy --workspace --lib --bins --examples -- -D warnings -A clippy::empty_line_after_doc_comments 2>&1 | tee /e/build-tools/logs/clippy-$(date +%Y%m%d).log
```

## Acceptance Gates

1. `cargo check --workspace`  0 errors, 0 warnings
2. `cargo test --workspace --no-run`  0 errors
3. `cargo clippy --workspace --lib --bins --examples -- -D warnings -A clippy::empty_line_after_doc_comments`  clean
4. CI workflow file exists at `.github/workflows/ci.yml` with the clippy job
5. Commit: `build: v0.2.1 Phase A gate restored  test imports fixed, clippy in CI`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST_CORE] [REQUIRES: GLM-5.1] [DEPENDS_ON: P0_SETUP_001] [BLOCKS: P0_SECURITY_*, P1_CORE_*]
