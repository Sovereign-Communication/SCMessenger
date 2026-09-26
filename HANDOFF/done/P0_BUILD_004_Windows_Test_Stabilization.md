# P0_BUILD_004: Windows Integration Test Stabilization

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Status:  P0 BLOCKER - Windows Build System Unstable
**Source:** Subagent feedback on rlib metadata / "can't find crate" errors

## Problem Statement
Windows-specific build issues preventing integration test compilation:
- rlib metadata errors in libp2p swarm tests
- "can't find crate" errors for integration tests
- test_address_observation.rs, integration_e2e.rs, integration_relay_custody.rs failing
- CARGO_INCREMENTAL=0 required but file lock issues

## Immediate Actions Required

### 1. Windows Build Environment Fix (~100 LoC)
**Files:** `.cargo/config.toml`, `scripts/windows_build.sh`
- Set `CARGO_INCREMENTAL=0` in environment
- Configure cargo for Windows-specific optimizations
- Handle file lock contention issues

### 2. Integration Test Isolation (~200 LoC)
**Files:** `core/tests/integration_*.rs`, `Cargo.toml`
- Add `#[cfg(feature = "integration")]` feature gates
- Create mock/stub implementations for swarm-dependent tests
- Separate integration tests from unit tests

### 3. Windows-Specific Workarounds (~150 LoC)
**Files:** `build.rs`, platform-specific configuration
- Handle Windows paging file issues
- Alternative compilation strategies for Windows
- Graceful fallback when incremental compilation fails

### 4. Test Infrastructure Repair (~150 LoC)
**Files:** Test utilities and mock libraries
- Fix rlib metadata generation
- Ensure crate discovery works on Windows
- Cross-platform test compatibility

## Total Estimate: ~600 LoC

## Success Criteria
1.  `cargo test --workspace` completes without rlib errors on Windows
2.  All integration tests compile and run
3.  No more "can't find crate" errors  
4.  Windows build environment stable and reproducible
5.  Cross-platform test compatibility achieved

## Special Instructions
- Use `CARGO_INCREMENTAL=0` environment variable
- May require `cargo clean` between builds
- Focus on Windows-specific file locking and paging issues
- Priority: Fix test_address_observation.rs first

## Priority: URGENT
Blocking all Windows development and integration testing.