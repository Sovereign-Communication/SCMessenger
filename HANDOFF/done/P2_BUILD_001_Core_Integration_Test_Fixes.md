# P2_BUILD_001: Core Integration Test Fixes

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P2 (Build Infrastructure)
**Platform:** Core/Rust
**Status:** Open
**Source:** REMAINING_WORK_TRACKING.md - Build Status

## Problem Description
Core integration tests reference unimplemented Phase 2+ APIs (ReputationTracker, MultiPathDelivery, etc.), have rlib format errors, and rustc crashes on `test_address_observation.rs`. CLI binary tests were fixed via `test = false` but core integration tests remain broken.

## Impact
- Blocks comprehensive test coverage
- Prevents CI pipeline implementation
- Hides potential regressions in core functionality
- Limits development velocity

## Issues Identified
1. **Unimplemented API references**: Tests call Phase 2+ APIs not yet implemented
2. **RLIB format errors**: Compilation issues with library format
3. **Rustc crashes**: Specific test files causing compiler crashes
4. **Test organization**: Integration tests need proper structure

## Implementation Required
1. Fix unimplemented API references in integration tests
2. Resolve rlib format compilation errors
3. Fix rustc crashes in specific test files
4. Reorganize test structure for better maintainability
5. Re-enable comprehensive test suite

## Key Files
- `core/tests/integration/` - Various integration test files
- `test_address_observation.rs` - Specific crashing test
- Cargo.toml configurations
- Build system configurations

## Expected Outcome
- All integration tests pass without crashes
- Comprehensive test coverage restored
- CI pipeline ready for implementation
- Development velocity improved