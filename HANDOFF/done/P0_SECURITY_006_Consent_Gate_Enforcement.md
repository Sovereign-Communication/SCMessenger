# P0_SECURITY_006: Consent Gate Enforcement

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P0 (Security/Compliance)
**Platform:** Core/Rust
**Status:** Partially Implemented
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md - PHIL-004

## Problem Description
First-run consent gate is implemented at UI level (Android, iOS, WASM, CLI) but NOT enforced at Rust core API level. `initialize_identity()` can be called without consent verification.

## Security Impact
- Consent bypass possible at API level
- Regulatory compliance risk
- Privacy boundary violation
- UI consent becomes advisory only

## Implementation Required
1. Add consent verification to `core/src/api.rs` `initialize_identity()`
2. Create consent state management in core
3. Add platform consent verification hooks
4. Ensure API-level enforcement

## Key Files
- `core/src/api.rs` - `initialize_identity()` function
- `core/src/identity/consent.rs` (new) - Consent state management
- Platform-specific consent verification

## Expected Outcome
- API-level consent enforcement
- Regulatory compliance ensured
- Privacy boundaries respected
- UI and core consent alignment