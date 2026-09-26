# P0_SECURITY_001: Bounded Retention Enforcement

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P0 (Critical Security)
**Platform:** Core/Rust
**Status:** Completed  
**Source:** REMAINING_WORK_TRACKING.md - PHIL-005

## Problem Description
sled database grows without bound, no compaction or retention enforcement. This creates unlimited storage consumption and potential privacy risks.

## Security Impact
- Storage exhaustion attacks possible
- Historical data remains indefinitely 
- No message lifecycle management
- Violates data minimization principles

## Implementation Required
1. Implement retention policies in `core/src/store/history.rs`
2. Add automatic compaction logic
3. Create configurable retention periods
4. Add enforcement during outbox/inbox operations

## Key Files
- `core/src/store/history.rs` - Retention enforcement
- `core/src/store/mod.rs` - Storage configuration
- Configuration system for retention periods

## Expected Outcome
- Configurable message retention (e.g., 30/90/365 days)
- Automatic cleanup of expired messages
- Storage size bounding and compaction
- Compliance with data minimization