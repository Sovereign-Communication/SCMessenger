# P3_IOS_SimulateBackgroundProcessing_Missing_Maintenance_Steps

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P3
**Platform:** iOS
**Status:** TODO
**Source:** native sweep 2026-07-04  follow-up gap on T17 in
`docs/release-readiness-2026-07-02.md`, which is otherwise CONFIRMED FIXED in
this sweep (re-verified against current source, see below). This is a residual
sub-issue T17's own fix didn't fully close, not a re-report of T17 itself.

## Status of T17 as originally filed (for context  do not re-fix this part)

T17(a)  tests sleeping instead of awaiting the spawned `Task`  is FIXED:
`iOS/SCMessengerTests/MeshBackgroundServiceTests.swift` now does
`let task = backgroundService.simulateBackgroundRefresh(); await task.value`
instead of `Task.sleep`.

T17(b) as originally filed  `simulateBackgroundRefresh()` skipping
`quickPeerDiscovery()`  is FIXED: `MeshBackgroundService.swift:217-229` now
calls `quickPeerDiscovery()`:

```swift
// MeshBackgroundService.swift:217-229
func simulateBackgroundRefresh() -> Task<Void, Never> {
    logger.debug("Simulating background refresh")
    return Task {
        do {
            try await meshRepository.syncPendingMessages()
            meshRepository.updateStats()
            try await meshRepository.quickPeerDiscovery()
            logger.debug("Simulated background refresh completed")
        } catch {
            logger.error("Simulated background refresh failed: \(error.localizedDescription)")
        }
    }
}
```

## The residual gap (new finding, not in the original T17 report)

The REAL (non-simulated) background processing handler, a few lines above in
the same file, runs three maintenance steps in sequence:

```swift
// MeshBackgroundService.swift:187-197 (the real handler)
try await self.meshRepository.performBulkSync()
try await self.meshRepository.cleanupOldMessages()
try await self.meshRepository.updatePeerLedger()
```

But its test-only simulated counterpart, `simulateBackgroundProcessing()`
(lines 231-243), only calls the first of the three:

```swift
// MeshBackgroundService.swift:231-243
func simulateBackgroundProcessing() -> Task<Void, Never> {
    logger.debug("Simulating background processing")
    return Task {
        do {
            try await meshRepository.performBulkSync()
            logger.debug("Simulated background processing completed")
        } catch {
            logger.error("Simulated background processing failed: \(error.localizedDescription)")
        }
    }
}
```

`cleanupOldMessages()` and `updatePeerLedger()` are silently skipped in the
simulation. This is the exact same class of test/production drift T17(b)
already fixed for `simulateBackgroundRefresh()` vs. its real handler  it just
wasn't applied to `simulateBackgroundProcessing()`'s sibling real handler in
the same pass. Any regression in `cleanupOldMessages()` or `updatePeerLedger()`
(e.g. one of them silently failing or being removed) would NOT be caught by
whatever test exercises `simulateBackgroundProcessing()`, since the simulation
never calls them.

## Fix Plan

Add the two missing calls to `simulateBackgroundProcessing()`, matching the
real handler's sequence and error handling:

```swift
func simulateBackgroundProcessing() -> Task<Void, Never> {
    logger.debug("Simulating background processing")
    return Task {
        do {
            try await meshRepository.performBulkSync()
            try await meshRepository.cleanupOldMessages()
            try await meshRepository.updatePeerLedger()
            logger.debug("Simulated background processing completed")
        } catch {
            logger.error("Simulated background processing failed: \(error.localizedDescription)")
        }
    }
}
```

Note: the real handler also runs `runMaintenanceCycle(budgetMs:)` on the Rust
core afterward (line 200)  decide whether the simulation should call that too
for full parity, or whether it's intentionally excluded (it may have side
effects or timing behavior unsuitable for a fast test loop). If excluded
intentionally, add a one-line comment explaining why, so the next audit
doesn't re-flag it as a gap.

## Files to Touch

- `iOS/SCMessenger/SCMessenger/Services/MeshBackgroundService.swift` [EDIT] 
  `simulateBackgroundProcessing()`, lines 231-243

## Verification

No Xcode available in this sweep. A build-capable session should run whatever
test currently exercises `simulateBackgroundProcessing()` (likely in
`iOS/SCMessengerTests/MeshBackgroundServiceTests.swift`, alongside the
`simulateBackgroundRefresh` test fixed under T17a) and confirm it still passes
after adding the two calls, and ideally add assertions that
`cleanupOldMessages`/`updatePeerLedger` were actually invoked (e.g. via a
mock/spy on `meshRepository` if the test harness supports one).

## Acceptance Criteria

- `simulateBackgroundProcessing()` calls `cleanupOldMessages()` and
  `updatePeerLedger()` in addition to `performBulkSync()`, matching the real
  handler's maintenance sequence (or an explicit, commented decision to
  exclude one/both, plus the `runMaintenanceCycle` question above resolved
  either way).
- Existing background-service tests still pass.
