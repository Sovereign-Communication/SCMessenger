# AND-LIFECYCLE -- unbounded waits on lifecycle and send paths (class audit)

Status: OPEN -- filed 2026-09-21 from the operator-reported stop regression RCA
Priority: P1 -- latent; the live report was STOP-TEARDOWN-TIMEOUT-001, which is fixed separately
Lane: Freebuff
Authority: operator request 2026-09-21 ("RCA why regressions aren't being squashed, but keep resurfacing")
Scope: `android/` only. No `core/` Rust, so no Rule-8 path in this ticket.
Target: the class fix for the reported instance is on branch `freebuff/android-stop-teardown-timeout` (PR #351); this ticket is the remainder.

## Premise (verified 2026-09-21, on that branch)

STOP-TEARDOWN-TIMEOUT-001 was one instance of a class: **a wait with no bound, on a
path that must complete**. Measured on the Pixel: one Stop tap logged
`stop requested` + `Stopping mesh service`, and the service record, its foreground
notification and the RUNNING state all survived, because
`MeshRepository.stopMeshService()` awaited `runBlocking { swarmBridge?.shutdown() }`
inside `synchronized(serviceLifecycleLock)`. Seven earlier stop fixes (R3, R4, R6-2,
R7-1, R8-1, STOP-RACE-001, HANG-LOCK-001) all shipped in the installed APK
(`userStoppedForSession` and `decideCommand` are present in its dex) and none of them
bounded that await or checked that a stop completes.

A class gate now covers the `runBlocking` shape:
`scripts/check_unbounded_runblocking.py` (self-tested, run from the Android Wiring
Gate job) fails CI when a `runBlocking` in `android/app/src/main/java` has no
`withTimeout`/`withTimeoutOrNull` in its own body. It is green over 105 Kotlin
sources on that branch.

## Sweep -- every other unbounded-wait shape in the Android main sources

| Site | Shape | Verdict (evidence) |
|---|---|---|
| `MeshRepository.kt:11522` `deferred.await()` | coroutine await, bootstrap race | **bounded** -- inside `withTimeoutOrNull(3_000L)` at `MeshRepository.kt:11497` |
| `BleGattClient.kt:447` `initiationSignal.await()` | coroutine await, BLE write init | **bounded** -- `withTimeoutOrNull(WRITE_INIT_TIMEOUT_MS) { ... }` on the same line |
| `SmartTransportRouter.kt:432` `winner.await()` in `attemptDelivery` | race that resolves only once EVERY candidate has finished | **UNBOUNDED**. The `withTimeoutOrNull(PREFERRED_TRANSPORT_TIMEOUT_MS)` at `:379` is 500 ms and encloses only the preferred-transport attempt; the parallel race opens at `:401` with no bound, and the call site `MeshRepository.kt:8146` has no enclosing bound either |
| `WifiAwareTransport.kt:511-512` `peerToLocal.join(); localToPeer.join()` | coroutine joins over two blocking socket pumps | **different class, lower severity** -- no monitor is held. `pump()` loops `while (!closed)` on a blocking `read()`, and `close()` only fires from `invokeOnCompletion`, so a mutually-silent socket pair leaves both joins waiting indefinitely |

## Do

1. `SmartTransportRouter.attemptDelivery`: bound the parallel race (wrap the
   `coroutineScope { ... }` block at `:401-436` in `withTimeoutOrNull` with a named
   constant) and let a timeout resolve to the existing no-route failure, so a send
   cannot hang behind one silent transport. Keep the winner semantics unchanged:
   first success wins, `null` only after all candidates finish.
2. `WifiAwareTransport.acceptAndPump`: give the join pair a bound or close the bridge
   on a deadline, so a silent socket pair cannot pin a scope indefinitely.
3. Extend the class gate to `join()`/`await()` that have no enclosing `withTimeout`.
   This needs enclosing-scope analysis, not a line-local grep -- approximating it
   would produce false positives and a gate people learn to ignore. If it cannot be
   done soundly, say so here and keep the two sites above fixed by hand.
4. Do not re-open the `runBlocking` gate; that shape is covered.

## Acceptance

- [ ] `SmartTransportRouter.attemptDelivery` bounded; race semantics unchanged; a
      test shows a timeout resolves to the no-route failure rather than hanging
- [ ] `WifiAwareTransport.acceptAndPump` join pair bounded
- [ ] `python scripts/check_unbounded_runblocking.py` exit 0, self-tests exit 0
- [ ] Mechanical gates green (`rules_check.py`, `check_wiring.py`); Rule-8 not
      applicable (no `core/src/{crypto,transport,routing,privacy}` path)
- [ ] A device pass: a send to an unreachable peer returns an error/failure state
      rather than leaving the message stuck in `delivering`
- [ ] PR evidence: commands and their literal output

## Review gate

No Rule-8 path if the change stays in `android/`. If a fix is ever pushed into
`core/src/transport`, this ticket acquires the Rule-8 gate and the WP2 precedent
applies: `UNVERIFIED-JEV` is not done.

## Rules

No emojis. Evidence contract. Worktree. No self-merge. Do not mark an acceptance row
met to move a score.
