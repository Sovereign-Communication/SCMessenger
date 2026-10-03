# Wiring Audit — Dead-Function Scan Findings

**Scope:** advisory findings from `scripts/check_wiring.py` after the dead-function scan was widened to all of `android/app/src/main`.
**Base commit:** `8e672c4e` (merge of #436).
**Open findings:** 32 (28 `C1_ZERO_CALLERS`, 4 `C4_TRANSITIVE_DEAD`).
**Gate impact:** none. Every item below is `WARN` severity. The blocking pass reports zero findings, unchanged from before this work.

Reproduce with:

```
python scripts/check_wiring.py                 # human readable
python scripts/check_wiring.py --json          # machine readable
```

---

## Why this list exists

Before this change the gate collected plain functions **only** under `/utils/`. Every
non-composable function in `service/`, `data/`, `transport/`, `ui/screens/` and
`ui/viewmodels/` was invisible to the zero-caller check. That is how
`MeshForegroundService.decideCommand` sat on the live seam with zero production
callers while ~25 tests exercised it and `check_wiring.py` reported
`[OK] All components, composables, routes, and utilities are correctly wired.`

Widen the collection alone and the function is *still* invisible: a second exemption
skips any nested member whose enclosing class is reachable, and `decideCommand` is
nested in `MeshForegroundService`, a live Service. The advisory pass therefore also
ignores the live-container exemption. Both were verified, not assumed (see Calibration).

## Calibration

`decideCommand` is the calibration entry. It must fire where it was dead and stay
silent where it is live:

| Revision | `decideCommand` state | Expected | Result |
|---|---|---|---|
| `e0150763` (pre-#436) | dead, zero production callers | flagged | **flagged** — `MeshForegroundService.kt:908`, `C1_ZERO_CALLERS` |
| `8e672c4e` (current main) | live, called by `registerLifecycleCommand` | not flagged | **not flagged** |

Both are reproduced by running the script with `--root` pointed at each revision.

The blocking pass was diffed old-script vs new-script on both revisions: **identical**,
zero findings each way. The widening cannot change what the gate blocks on.

## Two false positives found and fixed while building this

1. **Local functions reported as top-level.** `DiagnosticsScreen.refreshLogs` was
   reported unreferenced despite three call sites in its own file. The `fun_depth`
   heuristic never engages for a signature spanning several lines. Replaced with a
   brace-depth test for the function branch.
2. **Companion members misread as locals.** Once brace depth was used,
   `decideCommand` became invisible again, because `companion object` was never
   modelled as a scope. Added.

Neither bug affected the blocking pass. Both are covered by
`scripts/test_check_wiring.py`.

## Known false-positive classes to expect while triaging

- **Framework / DI invoked.** Providers reached by Hilt or Koin wiring, `@EntryPoint`
  accessors (`MeshSyncWorkerEntryPoint.getMeshRepository`), and interface methods a
  framework calls reflectively.
- **Called only from Kotlin string templates or reflection.** References are matched
  by bare identifier token over comment-stripped source.
- **Public API surface kept on purpose.** `MeshRepository` exposes several accessors
  that may be used by tests or tooling rather than app code.

The last class is why this is `WARN` and not blocking: several of the
`MeshRepository` entries are plausibly deliberate API surface, and a gate that
guesses will be ignored.

## Findings

| # | Kind | Location | Symbol | Chain |
|---|---|---|---|---|
| 1 | `C1_ZERO_CALLERS` | `data/GhostIdentityGate.kt:63` | `GhostIdentityGate.shouldRenderAsNode` | |
| 2 | `C1_ZERO_CALLERS` | `data/IdentityCreationCoordinator.kt:89` | `IdentityCreationCoordinator.isBackupAvailable` | |
| 3 | `C1_ZERO_CALLERS` | `data/MeshRepository.kt:1595` | `MeshRepository.testLedgerRelayConnectivity` | |
| 4 | `C4_TRANSITIVE_DEAD` | `data/MeshRepository.kt:4248` | `MeshRepository.getMeshService` | -> getMeshRepository |
| 5 | `C1_ZERO_CALLERS` | `data/MeshRepository.kt:5111` | `MeshRepository.signData` | |
| 6 | `C1_ZERO_CALLERS` | `data/MeshRepository.kt:5121` | `MeshRepository.verifySignature` | |
| 7 | `C1_ZERO_CALLERS` | `data/MeshRepository.kt:5135` | `MeshRepository.getDeviceId` | |
| 8 | `C1_ZERO_CALLERS` | `data/MeshRepository.kt:5139` | `MeshRepository.getSeniorityTimestamp` | |
| 9 | `C1_ZERO_CALLERS` | `data/MeshRepository.kt:5143` | `MeshRepository.getRegistrationState` | |
| 10 | `C1_ZERO_CALLERS` | `data/MeshRepository.kt:5151` | `MeshRepository.exportLogs` | |
| 11 | `C1_ZERO_CALLERS` | `data/MeshRepository.kt:6629` | `MeshRepository.recordConnection` | |
| 12 | `C1_ZERO_CALLERS` | `data/MeshRepository.kt:7302` | `MeshRepository.getLedgerSummary` | |
| 13 | `C1_ZERO_CALLERS` | `data/MeshRepository.kt:7324` | `MeshRepository.getServiceStateName` | |
| 14 | `C1_ZERO_CALLERS` | `data/MeshRepository.kt:7328` | `MeshRepository.getDiscoveredPeerCount` | |
| 15 | `C1_ZERO_CALLERS` | `data/MeshRepository.kt:7332` | `MeshRepository.getPendingOutboxCount` | |
| 16 | `C1_ZERO_CALLERS` | `data/MeshRepository.kt:7359` | `MeshRepository.getMissingRuntimePermissions` | |
| 17 | `C4_TRANSITIVE_DEAD` | `data/MeshRepository.kt:7813` | `MeshRepository.unsubscribeTopic` | -> unsubscribe |
| 18 | `C1_ZERO_CALLERS` | `data/TopicManager.kt:87` | `TopicManager.unsubscribe` | |
| 19 | `C1_ZERO_CALLERS` | `service/AnrWatchdog.kt:38` | `OnAnrDetected.onAnr` | |
| 20 | `C1_ZERO_CALLERS` | `service/AnrWatchdog.kt:263` | `AnrWatchdog.getTotalAnrEvents` | |
| 21 | `C4_TRANSITIVE_DEAD` | `service/AnrWatchdog.kt:265` | `AnrWatchdog.isMainThreadResponsive` | -> onAnr |
| 22 | `C1_ZERO_CALLERS` | `service/MeshSyncWorker.kt:25` | `MeshSyncWorkerEntryPoint.getMeshRepository` | |
| 23 | `C1_ZERO_CALLERS` | `service/PerformanceMonitor.kt:205` | `AnrEvent.toJson` | |
| 24 | `C4_TRANSITIVE_DEAD` | `transport/SmartTransportRouter.kt:174` | `SmartTransportRouter.getPreferredTransport` | -> TransportAttempt |
| 25 | `C1_ZERO_CALLERS` | `transport/TransportManager.kt:327` | `TransportManager.attemptEscalation` | |
| 26 | `C1_ZERO_CALLERS` | `transport/WifiAwareTransport.kt:47` | `WifiAwareTransport.encodePortTlv` | |
| 27 | `C1_ZERO_CALLERS` | `transport/WifiAwareTransport.kt:51` | `WifiAwareTransport.decodePortTlv` | |
| 28 | `C1_ZERO_CALLERS` | `transport/ble/BleScanner.kt:606` | `BleScanner.onTransportPause` | |
| 29 | `C1_ZERO_CALLERS` | `ui/viewmodels/ChatViewModel.kt:390` | `ChatViewModel.getRetryDelayForAttempt` | |
| 30 | `C1_ZERO_CALLERS` | `ui/viewmodels/MainViewModel.kt:214` | `MainViewModel.clearIdentityError` | |
| 31 | `C1_ZERO_CALLERS` | `ui/viewmodels/MainViewModel.kt:416` | `MainViewModel.consumeRequestsInboxNav` | |
| 32 | `C1_ZERO_CALLERS` | `ui/viewmodels/SettingsViewModel.kt:626` | `SettingsViewModel.resetSettingsToDefault` | |

## How to triage

For each row, decide which of three it is:

1. **Genuinely dead** — delete it, and delete whatever only it calls.
2. **Live but invisible to static analysis** — wire it explicitly, or record an
   allowlist entry with the reason.
3. **Deliberate API surface** — allowlist with the reason.

Then promote the scan:

```
python scripts/check_wiring.py --method-findings=block
```

and make the gate job pass. Promoting before this list is triaged is how a gate gets
ignored, so the default stays `warn` until the backlog is at zero.
