# AND-SS-001 — Android mesh Stop/Start broken: stop-flood pile-up + start cancellation leaves mesh dead

Status: Todo (filed 2026-09-23, Buffy / Freebuff lane, from the 3-node audit)
Priority: P0 — the operator cannot reliably stop or start the mesh from the UI;
both directions can strand the service in the wrong state.
Found by: 2026-09-23 three-node log audit (this session); Pixel evidence in
`tmp/pixel-logcat-20260921.txt` (device log pulled 2026-09-21).
Related but distinct: STOP-RACE-001 (2026-09-11, first-STOP-restarts fix —
latch only; does not cover flood or cancellation) and
`HANDOFF/audit/RCA_STOP_RACE_AND_CELL_STORED_2026-09-11.md`.

## Symptom, with evidence obtained by running commands

### A. Stop flood: 6 STOPs in 2 seconds, 6 sequential full teardowns

`tmp/pixel-logcat-20260921.txt` lines ~910940+ (2026-09-20 23:26:44–46Z):

```
MeshServiceViewModel$stopService: Mesh service stop requested    (x6)
MeshForegroundService$stopMeshService: Stopping mesh service     (x6, distinct threads)
```

Each `MeshServiceViewModel.stopService()` sends a fresh ACTION_STOP intent;
`decideCommand` always returns `Stop` and `stopMeshService()` re-runs the full
teardown (BLE/WiFi/mDNS stop, TransportManager cleanup, `runBlocking { swarm
shutdown }`, `meshService.stop()` Rust FFI) concurrently per delivery. Every
delivery runs on its own `serviceScope.launch` coroutine, so teardowns race
each other; `meshService` can be nulled by one while another is mid-`stop()`.

### B. Start dies with JobCancellationException; mesh stays stopped

Same log, 2026-09-21 05:52 window (process restart after the flood):

```
MeshRepository: Mesh service stopped
MeshForegroundService$startMeshService: Repository did not reach RUNNING state; aborting foreground service start
MeshForegroundService$startMeshService: kotlinx.coroutines.JobCancellationException: Job was cancelled; job=SupervisorJobImpl{Cancelling}
```

Two concurrent ACTION_START deliveries both ran `startMeshService()`; a stop
in flight cancelled the shared `serviceScope`, killing the start coroutines
between `meshRepository.startMeshService(config)` (already completed) and the
state re-check. The service then aborts and stops itself while the repository
core is actually running or half-started. The user's Start tap produced no
running mesh.

## Root causes (code anchors, current main)

1. `MeshForegroundService.stopMeshService()` (MeshForegroundService.kt:409)
   has no in-flight guard: repeated STOP deliveries each run the full teardown
   concurrently.
2. `decideCommand` honors every STOP unconditionally (by design, STOP-RACE-001
   R4-M1) — correct for the latch, but nothing coalesces duplicate teardowns.
3. Start and stop both run on the shared `serviceScope` (SupervisorJob), so a
   stop-path cancellation kills in-flight start coroutines and vice versa;
   `startMeshService()` re-checks state under no lock, so "already running"
   bail-out does not survive a racing stop.
4. `MeshServiceViewModel.stopService()` does not debounce; nothing prevents
   N intents from N taps.

## Acceptance criteria

1. Repeat STOP taps produce exactly one teardown; subsequent STOP deliveries
   are no-ops (unit-testable via `decideCommand` + a teardown guard flag).
2. A START delivered while teardown is in flight either queues cleanly behind
   it or is refused with the latch respected — never a half-started core.
3. Start path cannot be killed by a stop-path cancellation (no shared-job
   coupling between start and stop coroutines; state re-check happens under
   the lifecycle lock).
4. Regression tests: Kotlin unit test pins (a) single teardown under stop
   flood, (b) start-not-cancelled by racing stop.
5. Live: on the Pixel, STOP then START within 10 s ends with
   `ServiceState.RUNNING` and `service_start_requested` -> `Mesh service
   started successfully` in logcat exactly once per tap.

## Gates

Android/Kotlin only — no Rule-8 (no core/ change in this ticket). UI wiring:
run `python scripts/check_wiring.py` per rule 16 if nav/service wiring is
touched.

## References

- `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`
  (onStartCommand, startMeshService, stopMeshService, decideCommand)
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
  startMeshService (line ~1638) / stopMeshService (line ~4275) /
  `ensureServiceInitializedDeferred` (line ~6346)
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MeshServiceViewModel.kt`
  (startService/stopService/toggleService)
- Pixel evidence: `tmp/pixel-logcat-20260921.txt` 2026-09-20 23:26 flood +
  2026-09-21 05:52 cancellation window
