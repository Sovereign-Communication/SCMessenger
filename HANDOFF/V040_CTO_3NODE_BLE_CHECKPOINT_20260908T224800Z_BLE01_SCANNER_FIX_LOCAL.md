# V040 CTO three-node BLE checkpoint — BLE01_SCANNER_FIX_LOCAL

## Metadata

- Stage: `BLE01_SCANNER_FIX_LOCAL` (code-fix stage executed LOCALLY on operator ruling "no dispatch - do all work locally", 2026-09-08)
- UTC timestamp: `2026-09-08T22:48Z`
- Operator/session: Freebuff `/cto` continuation; Windows/AWS owner lane
- Branch: `cto/t2-disk-ruling-2026-08-31` @ base `ba474a7a` (branch confirmed via `git branch --show-current`)
- Coordination: CEO active per `HANDOFF/CEO_STATE.md` (last audit 2026-09-08 ~21:00Z); this checkpoint follows the CEO audit cycle
- Operator ruling recorded: **"no dispatch - do all work locally."** The earlier
  dispatch attempt (BLE-01 via `scripts/delegate_task.py` into an isolated
  worktree) was aborted after one round; its round-1 output is preserved as
  evidence at `tmp/cto/BLE01_DISPATCH_EVIDENCE/` and the worktree was removed.
  The controller edited application source directly under this explicit
  operator override of the controller-lane boundary.

## Defect 1: BLE scanner duty-cycle restart stranding — FIXED (local, gated)

Root cause (verified in source before edit, Rule 13): `BleScanner.stopScanningInternal()`
stopped the platform scanner but never cleared `isScanning`, while the
duty-cycle runnable in `startDutyCycle()` requires `isScanning == false` before
calling `startScanningInternal()`. After the first duty-cycle window, scanning
was permanently stranded — matching live logcat evidence
("BLE scan window ended" then `peersDiscovered=0` forever, capture
`tmp/cto/V040_ADB_SCMESSENGER_FOCUSED_20260908T205239Z/`).

Change (2 files, +36/-1 net, verified via `git diff --stat`):

1. `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt`
   (`stopScanningInternal()`): set `isScanning = false` on the success path,
   the catch path, and the null-scanner early-return path.
2. `android/app/src/test/java/com/scmessenger/android/transport/ble/BleScannerTest.kt`
   (+ `import org.junit.Assert.assertFalse`): new regression test
   `dutyCycleStop_clearsIsScanningFlag` — reflectively sets `isScanning`,
   invokes `stopScanningInternal()` reflectively, asserts the flag is false.

Verdict: **PASS** (unit gate). Authoritative gate on the Windows host, under
`scripts/build_lock.py` (holder `cto-ble01-fix`):

```
cd android && .\gradlew.bat :app:testDebugUnitTest --tests com.scmessenger.android.transport.ble.BleScannerTest --console=plain
BUILD SUCCESSFUL in 45s
BleScannerTest > dutyCycleStop_clearsIsScanningFlag PASSED   (new)
BleScannerTest > pruneOldPeers_removesStaleEntries PASSED
BleScannerTest > onTransportPause_clearsCache PASSED
BleScannerTest > clearPeerCache_preservesCounterStats PASSED
BleScannerTest > clearPeerCache_isIdempotentOnEmptyCache PASSED
BleScannerTest > clearPeerCache_removesAllDiscoveredPeers PASSED
BleScannerTest > getDiscoveryStats_reportsZeroAfterClear PASSED
```

Gate log: `tmp/cto/BLE01_GATE_FINAL_20260908T224500Z.log` (exit 0).
Earlier run at `tmp/cto/BLE01_GATE_20260908T124100Z.log` (exit 0, pre-EOL-fix
bytes — superseded by the FINAL run on the exact on-disk content).

File SHA256 (post-fix):

- `BleScanner.kt` = `32b68d7d34c58c08e2901e374fb296272ee97beb49798495deccd95c9f121961`
- `BleScannerTest.kt` = `f8899bb150c58dab77aa28a83e3d128ef1a8a530120d3a5246671a8a7916da37`

## Defect 2: relay store-and-forward (cell) — NOT a code defect on this branch

Re-derivation this session, from source read in this session:

- The previously documented "custody split-brain" is **already fixed** on this
  branch: `core/src/transport/swarm.rs:3019-3041` constructs
  `RelayCustodyStore::for_service_storage(...)` and swaps it into `IronCore`
  (`*core.relay_custody_store.write() = relay_custody_store.clone()`); every
  field is an `Arc`/`Copy`, so the swap shares state rather than duplicating.
- Both production `start_swarm_with_config` call sites
  (`cli/src/main.rs:2109`, `cli/src/main.rs:3495`) pass `None` as
  `storage_path`, so `for_service_storage(None, ...)` falls back to
  `for_local_peer()` — a persistent sled store under the custody base dir, not
  memory. Persistent custody survives restarts.
- Live observation 2026-09-08T22:46Z: Windows node healthy on
  `http://127.0.0.1:9876` (identity `985a25f9...`, diagnostics JSON returned,
  `custody_audit_count=5073`), AWS node healthy
  (`http://18.234.62.247:9876/health` -> `{"status":"healthy"}`). This
  **proves custody state exists and is audited; it does NOT prove an offline
  destination was later delivered** (diagnostics are state, not delivery).

What actually blocks relay delivery today (evidence on file):

1. **T14 (P0, queued):** the node advertises an ephemeral NAT source port as
   its external address, so peers dial a port where nothing listens
   (`HANDOFF/freebuff/README.md` T14 row; flagged as P0 ahead of T13 rework,
   Rule-8 gate). This is the highest-value code fix for the relay path.
2. **Android runtime config, not code:** fresh logcat shows Pixel failing to
   dial the Windows LAN peer `192.168.0.222:9001/9002` (IO error) and
   `no proven ledger relay candidates; network=WIFI, cellular=false`.
3. **Windows firewall hypothesis (UNVERIFIED):** inbound 9001/9002 from the
   LAN may be blocked on the Windows host. Requires an operator-approved
   firewall probe — NOT tested this session.

Verdict: **BLOCKED for a fix claim** (no code change made for relay this
session; the correct gate is T14 + a live offline-custody delivery proof).
Store-and-forward is **not** claimed fixed.

## Scope, reachability, and rules compliance

- Rule 16 (wire-it-or-it-is-dead) check on the BLE fix: the fixed method is
  called from the duty-cycle runnable (BleScanner.kt:388 `stopScanningInternal()`
  call site verified in source this session), which is started by
  `startDutyCycle()` whenever `scanWindowMs < scanIntervalMs`; the scanner is
  started by the mesh service. The call chain is reachable; the fix lands on a
  live path.
- No file outside the two listed files was modified by this session's fix.
- Shared-checkout rule 11: `git status --porcelain` limited to the two target
  files (verified after the gate; no other tracked file touched).
- Rule 1 (no emoji): held in code, comments, and this checkpoint.
- Rule 8 (adversarial review): **not triggered** — the change touches
  `android/` only, outside `core/src/{crypto,transport,routing,privacy}`.
  The session's source reads of `core/src/transport/swarm.rs` and
  `core/src/store/relay_custody.rs` were read-only; no review is owed on
  reads.
- Operator rulings honored: "no dispatch - do all work locally" (applied);
  Android remains operator-driven ("I drive android") — the Pixel was NOT
  touched this session.

## Deployment state (evidence-gated, not assumed)

- The running Windows node binary and the installed Android APK still carry
  the **pre-fix** behavior; the fix exists only in working-tree source and
  will reach runtime only after rebuild + redeploy. Not claimed deployed.
- The installed APK's BLE behavior remains governed by the pre-fix binary
  until a new APK is built and installed (operator drives Android).

## Next steps (ordered)

1. Rebuild the Windows CLI + rebuild/reinstall the Android APK with this fix,
   then rerun the Phase-2/3 evidence gates from the tracked package
   (`HANDOFF/V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md`).
2. Queue T14 (ephemeral-port P0) as the relay-path code fix, with its Rule-8
   gate — it is the highest-value store-and-forward change.
3. Operator-approved Windows firewall probe for inbound 9001/9002 (UNVERIFIED
   hypothesis for the Pixel LAN-dial failure).
4. Live three-node re-verification (BLE peer discovery + offline-custody
   delivery) before any completion claim.

## Explicit verdicts

- BLE scanner code fix: **PASS** (unit gate, Windows host, build-locked).
- BLE end-to-end on a device: **UNVERIFIED** (requires rebuild + operator
  Android steps).
- Store-and-forward (cell) code: **BLOCKED for a fix claim** — no defect
  fixed this session; relay blockers are T14 (queued P0) + runtime config;
  live delivery proof outstanding.
- Store-and-forward live delivery: **UNVERIFIED** (custody state exists on
  both nodes; delivery to an offline destination not yet proven).
- Dispatch flow: **ABORTED per operator ruling; evidence preserved** at
  `tmp/cto/BLE01_DISPATCH_EVIDENCE/` (round-1 worker output lacked the
  metadata footer and the assertFalse import — documented there).
- Overall three-node V040: **BLOCKED** (unchanged; Phase 2/3/4 gates remain
  open, Android service control stays with the operator).
