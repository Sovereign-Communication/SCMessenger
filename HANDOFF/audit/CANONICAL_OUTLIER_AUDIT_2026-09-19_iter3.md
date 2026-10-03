# Canonical Outlier Audit -- iteration 3 (DIM-C: wiring / reachability)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Task: HANDOFF/freebuff/queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
Date: 2026-09-19
Branch: glm/canonical-outlier-audit
HEAD: 1acb63531aaa1e7c22071bb7430c72b5a9753210
Model: Buffy (Freebuff)
Mode: REPORT-ONLY

## Command and exit code (correct $? capture, no pipe before it)

```
python scripts/check_wiring.py > tmp/canonical_audit_2026-09-19/check_wiring.txt 2>&1; rc=$?
RC=0
```

## FULL checker output (6 lines, nothing elided)

```
=== SCMessenger Wiring & Reachability Gate ===
Exclusions:
  [INFO] Excluded test sources (android/app/src/test, android/app/src/androidTest).
  [INFO] Excluded @Preview annotated composables (tooling preview only).

[OK] All components, composables, routes, and utilities are correctly wired.
```

## What the gate actually checks (read this session, scripts/check_wiring.py)

- Header doc lines 4-19: C1 components; "C2. Nav route reachability: Parses
  Screen route definitions and NavHost composable(...) registrations. Reports
  any route that is defined and/or navigated to but has no composable(...)
  registration in the NavHost (navigation lands nowhere), and registered routes
  that are never navigated to."
- `parse_manifest` (:149-176) parses `activity`, `service`, `receiver`,
  `provider` tags and compares against source declarations.

## Independent spot-checks (not trusting the green alone)

1. Manifest service/receiver entries vs source files:
   `grep -n -A2 -E "<service|<receiver" android/app/src/main/AndroidManifest.xml`:
   - :104-105 `.service.MeshForegroundService`
   - :111-112 `.service.MeshVpnService`
   - :122-123 `.service.BootReceiver`
   - :134-135 `.notification.NotificationActionReceiver`
   - :147-148 `.utils.ShareReceiver`
   Source service dir (`ls android/.../service/`) contains exactly the named
   service/receiver classes plus non-component helpers (AndroidPlatformBridge,
   AnrWatchdog, MeshEventBus, MeshSyncWorker, PerformanceMonitor,
   ServiceHealthMonitor) -- no class in the component-shaped set is missing
   from the manifest.
2. The ebf5411b incident classes from AGENTS.md rule 16 (defined-but-not-
   registered routes, services without manifest entries) are exactly what C2
   and parse_manifest check; the 2026-09-19 result is green on both.

## Findings

- 0 new findings this dimension.

## Scope note (honest limit, not a finding)

The wiring gate covers the Android compose/manifest surface and the repo's own
route/utility inventory. It does NOT prove CLI or iOS reachability (e.g. iOS
code restored without call sites would not be caught here). No iOS reachability
check was run this session; iOS wiring remains UNVERIFIED in this audit.

## Not done / UNVERIFIED

- iOS/macOS reachability: UNVERIFIED (above).
- Checker exclusions mean test-source wiring is unverified by design (fine).

## Next iteration aim

DIM-D: canonical doc vs code / doc vs doc; docs_sync_check.sh exit code.
