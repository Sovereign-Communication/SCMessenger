# P1_CORE_004: Outbox Flush Completion

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P1 (Core Functionality)
**Platform:** Mobile (Android/iOS) + WASM
**Status:** Open
**Source:** REMAINING_WORK_TRACKING.md - Item #10

## Problem Description
Outbox flush on PeerDiscovered is incomplete - mobile and WASM paths missing. Messages may not be automatically sent when peers are discovered.

## Impact
- Reduced message delivery reliability
- Manual intervention required for message sending
- Inefficient use of peer discovery events
- Poor user experience with delayed messages

## Current State
Outbox flush logic exists but is not fully integrated across all platforms:
- Core Rust implementation: Partial
- Android: Incomplete integration
- iOS: Missing paths
- WASM: Not implemented

## Implementation Required
1. Complete outbox flush integration in `core/src/`
2. Add mobile platform integration (Android `MeshRepository.kt`, iOS `MeshRepository.swift`)
3. Implement WASM outbox flush support
4. Add proper error handling and retry logic
5. Ensure cross-platform consistency

## Key Files
- `core/src/transport/outbox.rs` - Outbox management
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` - Android integration
- `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift` - iOS integration
- `wasm/src/lib.rs` - WASM support
- Peer discovery event handlers across all platforms

## Expected Outcome
- Automatic message sending on peer discovery
- Complete cross-platform coverage
- Reliable outbox flushing
- Improved message delivery rates