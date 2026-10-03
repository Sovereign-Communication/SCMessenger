# P0_BUILD_002: Integration Test Repair

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P0 (Build Blocking)
**Platform:** Core/Rust
**Status:** Open
**Source:** REMAINING_WORK_TRACKING.md - Build Status section

## Problem Description
Core integration tests fail with rlib format errors and rustc crash on `test_address_observation.rs`. Tests reference unimplemented Phase 2+ APIs (ReputationTracker, MultiPathDelivery, etc.), blocking comprehensive testing.

## Impact
- `cargo test --workspace` fails completely
- Prevents CI pipeline implementation
- Hides potential regressions
- Blocks comprehensive test coverage

## Specific Failures
1. **rlib format errors** - Compatibility issues with test artifacts
2. **rustc crash** on `test_address_observation.rs` - Compiler crashes
3. **Unimplemented API references** - Tests expect Phase 2+ features not yet wired
4. **Integration test dependencies** - Missing test infrastructure

## Root Cause
Integration tests were written for future Phase 2+ features but never updated when implementation priorities shifted. The tests reference:
- `ReputationTracker` full functionality (partially implemented but not fully wired)
- `MultiPathDelivery` complete API surface
- Advanced mesh routing features not yet activated

## Implementation Required
1. Fix rlib format compatibility issues
2. Resolve rustc crash in `test_address_observation.rs`
3. Mock or stub unimplemented Phase 2+ APIs for testing
4. Create test infrastructure for current Phase 1 functionality
5. Ensure all existing functionality has proper test coverage
6. Separate integration tests from unit tests

## Key Files
- `core/tests/integration/` - All integration test files
- `test_address_observation.rs` - Specific crashing test
- `core/src/lib.rs` - Test configuration and setup
- Build system configuration files
- Test mock/stub infrastructure

## Expected Outcome
- `cargo test --workspace` passes completely
- All existing functionality properly tested
- No crashes or rlib format errors
- Foundation for CI pipeline established
- Comprehensive test coverage for Phase 1 features

## Verification
- `cargo test --workspace` returns exit code 0
- All integration tests pass
- No compiler crashes
- Test coverage report shows >80% coverage