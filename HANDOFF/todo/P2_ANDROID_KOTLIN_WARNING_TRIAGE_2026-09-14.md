# P2 — Android Kotlin warning triage (80 warnings, CI run 34907771392)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Filed: 2026-09-14 by the Freebuff lane, from the Android Debug APK compile log.
Source: `w:` warnings enumerated in full from the CI build output (no
truncation; full list preserved in the run log linked from PR #288).

## Disposition classes

### Class A — fix before 0.4.0 tag (correctness risk, not style)

- `SubnetProbe.kt:371:22` — "Unsafe use of a nullable receiver of type
  String?" inside `narrowToScanSubnet` (`ip.hostAddress.split(".")` chain on
  a platform getter that is platform-typed). Real NPE risk on a code path
  that feeds subnet scanning. Fix is a null-guard or non-null assertion with
  provenance comment; 5-line change.
- `MeshRepository.kt:3500:73` and `:4517/:4596/:4625/:4813/:6935` cluster —
  "Unnecessary safe call on a non-null receiver of type String": these are
  harmless AT THIS INSTANT but encode false nullability beliefs. Fix in the
  P1 curve-unification pass (PR #289 touches the same viewmodel family) so
  the same files are not churned twice.

### Class B — deferred to 0.5.0 (hygiene; zero runtime effect)

- All `Variable 'x' is never used` / `Parameter 'p' is never used` warnings
  (~15 sites): dead locals and params. One mechanical PR, `cargo`-style
  rename-to-`_` or removal, Kotlin Linting gate is the checker.
- All `Elvis operator always returns the left operand` and `Unnecessary
  non-null assertion (!!))` and `Condition always true` (~20 sites): type
  system noise from platform-type leakage; fold into the same 0.5.0
  mechanical PR.
- `Name shadowed: canonicalId` (MeshRepository.kt:1964) and
  `traceMessageId` (7882): rename in the 0.5.0 PR (shadowing is a
  future-bug factory but is not a current defect).

### Class C — deferred to 0.5.0/1.0.0 (deprecated platform APIs; API-level
upgrade work, each needs the modern replacement verified on-device)

- BLE: `BleGattClient.kt` (603-867: `onCharacteristicRead/Changed` deprecated
  signatures, `writeCharacteristic(Characteristic)`), `BleGattServer.kt`
  (209-559) — the replacement API (`onCharacteristicChanged(gatt, char,
  value)`) changes callback threading; needs the BLE rig, not a blind edit.
- NSD/mDNS: `MdnsServiceDiscovery.kt` (240/293 `host` getter, 740
  `resolveService`) — modern path is `NsdManager.discoverServices` with the
  executor overload; gate mapping touches PF-11, coordinate with the
  self-ratchet regression tests.
- Network: `NetworkDetector.kt`/`SubnetProbe.kt` `allNetworks` (deprecated)
  — replacement is `ConnectivityManager.networkCallback` snapshots.
- `WifiDirectTransport.kt:294-296` (`getParcelableExtra`, `NetworkInfo`) and
  `MeshForegroundService.kt:539` (`icon: Int` deprecated), plus Compose
  `HelpOutline` auto-mirror (AddContactScreen.kt:717).
- `PerformanceMonitor.kt:159` unused `anrEvent` parameter belongs to Class B.

## Tracking rule

Re-run the warning census at each tag candidate; Class A must be zero, Class
B/C counts may shrink but must be re-enumerated in the release ticket.
