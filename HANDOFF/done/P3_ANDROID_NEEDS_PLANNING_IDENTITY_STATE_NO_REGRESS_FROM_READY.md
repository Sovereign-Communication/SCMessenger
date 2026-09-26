# [NEEDS PLANNING] Android  IdentityCreationCoordinator Can Never Leave IdentityState.Ready Once Reached

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Context

Fresh sweep of `android/app/src/main/java/com/scmessenger/android/`
(2026-07-04), following up on the most recent commit touching Android code
(`514434e0`, "android/core: fix identity load lag and state machine races",
2026-07-03). Not covered by T8-T13 (`docs/release-readiness-2026-07-02.md`
7)  those predate this commit.

That commit added a "don't regress from Ready" guard to
`IdentityCreationCoordinator.kt` (~lines 48-56) to fix a real bug: the
`identityInfo` StateFlow's initial `null` emission at construction time was
overriding a valid cached `Ready` state, causing UI flicker
(Settings/Identity screens briefly showing "Create Identity" before
snapping to the real state). The fix:

```kotlin
meshRepository.identityInfo.collect { info ->
    val initialized = info?.initialized == true
    if (initialized) {
        _identityState.value = IdentityState.Ready
    } else if (_identityState.value != IdentityState.Ready) {
        // Don't regress from Ready  the initial null emission from
        // StateFlow construction must not override a valid cached state.
        determineInitialState()
    }
}
```

## The latent gap

This guard is correct for its stated purpose (suppressing a spurious
transient `null` at startup) but, read literally, it means: **once
`_identityState` reaches `Ready`, no future `identityInfo` emission of
`initialized == false` can ever move it out of `Ready` again**  the
`else if` branch is permanently skipped from that point on.

Checked whether this is reachable today: grepped the whole Android source
tree for any delete-identity / wipe-identity / factory-reset /
reset-identity call path (`deleteIdentity`, `wipeIdentity`,
`resetIdentity`, `clearIdentity`, `factoryReset`  all absent).
**There is currently no UI flow that would ever cause `identityInfo` to
transition from initialized back to uninitialized after first reaching
Ready**, so this is not a live bug today  it's a latent trap.

The trap: if/when a "delete identity" or "reinstall/reset" feature is
added (this is a reasonable v1.0.0-era feature  most secure messengers
have one), and it works by clearing core state such that
`meshRepository.identityInfo` emits `initialized = false` afterward, the
Settings/Identity screens (which read `identityState` via
`SettingsViewModel`/`IdentityViewModel`, both of which now also seed their
own `_identityInfo` from `meshRepository.identityInfo.value` per the same
commit) will keep showing "Ready" / the old identity fields indefinitely,
because this coordinator's state machine has no path back to `None`.

## Question that needs an answer before implementing

Is a delete/wipe/reset-identity feature planned for v1.0.0 or shortly
after? Two very different fixes depending on the answer:

- **If yes / soon**: that feature's implementation must explicitly call
  something like `identityCreationCoordinator.determineInitialState()` (or
  a new explicit `notifyIdentityCleared()` method) immediately after the
  clear operation completes, rather than relying on the `identityInfo`
  collector to notice  because the collector structurally cannot notice,
  per the guard above. This should be scoped as part of that feature's
  spec, not bolted on separately.
- **If no such feature is planned soon**: this task can stay open as a
  documented risk (add a code comment at the guard site noting the
  invariant this depends on: "no code path currently clears
  `identityInfo` after initialization; if one is added, it must call
  `determineInitialState()` explicitly") rather than be implemented now,
  to avoid speculative state-machine complexity for a feature that
  doesn't exist yet.

Do not guess which of these applies  it depends on product roadmap, not
code structure. Whoever picks this up should check `REMAINING_WORK_TRACKING.md`
and any v1.0.0 milestone doc for a planned delete-identity feature before
choosing a direction.

## Files

- `android/app/src/main/java/com/scmessenger/android/data/IdentityCreationCoordinator.kt`
  (init block, ~lines 44-59)
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`
  and `IdentityViewModel.kt` (both now seed initial state from
  `meshRepository.identityInfo.value` per the same commit  same
  no-regression assumption applies transitively)

## Verification (once direction is chosen)

If direction 1 (wire in an explicit clear notification): add a unit/instrumented
test simulating identity creation  Ready  simulated clear  assert
`identityState` returns to `None`. Run:
```bash
cd android
./gradlew assembleDebug -x lint --quiet
```

If direction 2 (document only): confirm the comment is added and no other
change is needed; no build-affecting verification required beyond a normal
`assembleDebug`.
