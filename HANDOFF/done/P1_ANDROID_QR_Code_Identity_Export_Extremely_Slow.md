# TASK: P1-ANDROID-QR-SLOW  "Show Identity QR" takes up to ~45 seconds to load

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Tier:** [SONNET]
**Gates:** Kotlin/Android only, no `crypto-security-auditor` gate (confirmed below: this path does
no cryptographic key derivation at all, unlike the sibling ticket it's easy to conflate this with).

## Source

Live operator report, 2026-07-05: "my QR code isn't loading.. or it's loading REALLY slowly..."
 confirmed loaded after approximately 45 seconds. Operator asked whether this is a regression of
`HANDOFF/IN_PROGRESS/IN_PROGRESS_P0_ANDROID_IDENTITY_GENERATION_ARGON2_OPT_LEVEL_FIX.md`
(the Argon2id `opt-level=3` fix from 2026-07-02).

## Is this a regression of the Argon2id fix?  No, confirmed by reading the actual call path.

`IdentityViewModel.getQrCodeData()` (`IdentityViewModel.kt:252-268`) calls
`meshRepository.getIdentityExportString(minimalForQr = true)`
(`MeshRepository.kt:8774-8813`)  this is the **public identity card** export (peer ID, public
key, listeners), explicitly documented at `MeshRepository.kt:3193-3194` as "no encryption." It
never calls `exportIdentityBackup*` / `derive_key_argon2id` at all. The Argon2id fix's target
function is a completely different code path (encrypted backup export, used by identity
creation/settings backup, not QR display). **This is a separate, real bug, not a recurrence.**

## Problem (exact, verified by reading the current call chain)

`getIdentityExportString(minimalForQr: Boolean)` does real, wasteful, redundant work even in
"minimal" mode:

1. **Unconditional full-address computation that's thrown away.** Lines 8776-8779 compute
   `getListeningAddresses()`, `getExternalAddresses()`, and `getPreferredRelay()`
   **unconditionally**, before the `if (minimalForQr)` branch at line 8799 even runs. When
   `minimalForQr == true` (the QR screen's case), none of `listeners`/`externalAddresses`/`relay`
   end up in the final payload  only `localIp` (from the separate, cheap
   `getLocalIpAddress()`) is used. `getExternalAddresses()` and `getListeningAddresses()`
   (`MeshRepository.kt:8717-8729`) both call into `swarmBridge` (a Rust FFI round-trip)  if
   external-address observation involves any wait on NAT reflection/identify events completing,
   this is pure wasted latency on the QR hot path for data that gets discarded.

2. **Redundant, uncached identity re-fetch.** `getIdentityExportString()` calls the plain
   `getIdentityInfo()` (`MeshRepository.kt:4131-4146`, NOT the cached
   `getIdentityInfoNonBlocking()` variant) even though `IdentityViewModel.getQrCodeData()`
   (`IdentityViewModel.kt:259-260`) already called `getIdentityInfoNonBlocking()` moments earlier
   specifically to confirm the identity is initialized before proceeding. This uncached
   `getIdentityInfo()` unconditionally calls `ensureServiceInitializedFireAndForget()` +
   `runCatching { ensureLocalIdentityFederation() }` (which itself does another
   `core.getIdentityInfo()` FFI call, a possible `restoreIdentityFromBackup()`, and
   `core.grantConsent()`) + a **nested `kotlinx.coroutines.runBlocking { ... }`** for nickname
   caching (`MeshRepository.kt:4143-4146`). All of this duplicates work the ViewModel already did
   via the fast cached path, and `runBlocking` inside an IO-dispatched coroutine is exactly the
   kind of thing that can silently balloon under load or contention.

3. **Possible coupling to MeshService cold-start.** This codebase documents elsewhere
   (`MeshRepository.kt:4964-4972`, `ensureServiceInitialized()`) that MeshService cold-start
   ("sled DB init, manager setup, data migration") is expected to potentially take **up to 60
   seconds**, with a dedicated progress-callback mechanism built specifically to narrate that
   wait to the user. If the QR screen is opened while the service is still mid-cold-start, any of
   the FFI calls in the chain above could be waiting on pieces of that sequence  this would
   explain an duration in the tens-of-seconds range far better than any single fast JSON-building
   operation should ever take.

None of the three items above individually is proven to be *the* 45-second cause  they are
verified-real, wasteful/redundant code paths on the hot path; item 3 is the most likely single
explanation for the specific magnitude reported, but needs device timing to confirm (see
Verification below).

## Root Cause

`getIdentityExportString()` was written to serve both the full identity card (used elsewhere) and
the "minimal" QR variant from a single function, but the `minimalForQr` short-circuit was only
applied to *payload construction*, not to the *expensive computation feeding it*  the function
still does all the work the full variant needs, discards most of it, and additionally re-fetches
identity info through the slow path instead of accepting an already-known-initialized identity
from the caller.

## Blast Radius

`android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (`getIdentityExportString`,
`getIdentityInfo`, `ensureLocalIdentityFederation`),
`android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt`
(`getQrCodeData`). No Rust core changes expected  this is pure Kotlin call-path waste.

## Related (do not conflate)

- `HANDOFF/todo/P2_ANDROID_IDENTITY_QR_PRERENDER.md`  addresses QR generation blocking the Main
  thread and jank on `IdentityScreen.kt` (assumed magnitude: 200-2000ms). Current code already
  wraps `getQrCodeData()` in `withContext(Dispatchers.IO)` (`IdentityViewModel.kt:253`), so that
  ticket's core threading complaint may already be substantially addressed  but its assumed
  *magnitude* (2s worst case) is far smaller than the 45s now reported, confirming this is a
  distinct, more severe problem than what that ticket scoped. Land both; this ticket's fix reduces
  the actual work done, that ticket's (if still needed) improves perceived UX around whatever
  latency remains.
- `HANDOFF/IN_PROGRESS/IN_PROGRESS_P0_ANDROID_IDENTITY_GENERATION_ARGON2_OPT_LEVEL_FIX.md` 
  confirmed unrelated (see "Is this a regression" section above), but that ticket's own unresolved
  "Open question" (a user report of continued slowness after a fresh reinstall) may actually have
  been *this* bug all along, misattributed to Argon2id at the time since both manifest as "identity
  screen is slow." Worth closing that ticket's open question with a pointer here once this lands.

## Scope / What to do

1. Add a fast path in `getIdentityExportString(minimalForQr: Boolean)`: when `minimalForQr` is
   true, skip `getListeningAddresses()`/`getExternalAddresses()`/`getPreferredRelay()` entirely 
   only compute `getLocalIpAddress()` and the identity fields actually used in the minimal payload.
2. Have `getIdentityExportString()` accept an already-fetched `IdentityInfo` (or call
   `getIdentityInfoNonBlocking()` instead of the slow `getIdentityInfo()`) when called from the QR
   path, since the caller (`IdentityViewModel.getQrCodeData()`) already confirmed initialization
   via the fast cached path moments before.
3. Instrument (temporarily, or via existing Timber timing patterns already used elsewhere in this
   file) each step of the QR generation path to get real device timing data pinpointing exactly
   which sub-call accounts for the ~45s  do not guess further without this before considering the
   fix complete; the three factors above are verified-real but not yet individually measured.

## Files to Touch

- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
  (`getIdentityExportString`, possibly a new lightweight identity-info accessor for this path)
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt`
  (`getQrCodeData`, if it should pass the already-fetched `IdentityInfo` through)

## Verification Commands

```bash
cd android && ./gradlew assembleDebug -x lint --quiet
```

Manual (device, required  this is fundamentally a timing bug): fresh app launch, wait for
MeshService to reach RUNNING (confirm via existing status UI/logcat), then open Identity screen /
QR display and time until the QR renders. Repeat once more immediately after (warm path) to see if
the delay is cold-start-specific or reproducible every time. Add Timber timing breadcrumbs around
each of `getListeningAddresses`/`getExternalAddresses`/`getIdentityInfo` calls in this path for the
first repro run to get real numbers instead of guessing.

## Do NOT

- Do NOT assume this is the Argon2id opt-level issue recurring  confirmed unrelated function.
- Do NOT touch `exportIdentityBackup`/`derive_key_argon2id` as part of this fix  different code
  path, out of scope.
- Do NOT remove `getListeningAddresses()`/`getExternalAddresses()` from the **non-minimal**
  `getIdentityExportString()` branch  those are legitimately needed there (full identity export
  used elsewhere, e.g. contact sharing beyond QR).
