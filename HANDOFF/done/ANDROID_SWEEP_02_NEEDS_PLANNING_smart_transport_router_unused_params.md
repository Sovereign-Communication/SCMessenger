# TASK [NEEDS PLANNING]: Android  Unused Trace/Context Params in SmartTransportRouter.attemptDelivery and Conversation Row Composables

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Context

Independent sweep of `android/app/src/main/java/com/scmessenger/android/`
(2026-07-04), looking for gaps not already covered by
`docs/release-readiness-2026-07-02.md` (T8-T13) or the existing HANDOFF
backlog. This is distinct from T12 (WifiAwareTransport loopback proxy
defects)  same general area (transport/delivery) but a different file and a
different kind of problem.

`SmartTransportRouter.attemptDelivery()`
(`android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt:296-310`)
takes four parameters that are marked `@Suppress("UNUSED_PARAMETER")` and
never read inside the function body:

```kotlin
suspend fun attemptDelivery(
    peerId: String,
    @Suppress("UNUSED_PARAMETER") envelopeData: ByteArray,
    wifiPeerId: String?,
    blePeerId: String?,
    tcpMdnsPeerId: String?,
    routePeerCandidates: List<String>,
    @Suppress("UNUSED_PARAMETER") listeners: List<String>,
    @Suppress("UNUSED_PARAMETER") traceMessageId: String?,
    @Suppress("UNUSED_PARAMETER") attemptContext: String?,
    tryWifi: suspend (String) -> Boolean,
    tryBle: suspend (String) -> Boolean,
    tryTcpMdns: suspend (String) -> Boolean,
    tryCore: suspend (String) -> Boolean
): TransportDeliveryResult
```

The single call site
(`android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:5896-`)
passes real values for all four: `envelopeData = encryptedData`, plus
(need to confirm at the call site) real `listeners`, `traceMessageId`, and
`attemptContext` values. The function accepts this data but silently drops it
on the floor  the actual transport attempts happen via the `tryWifi`/
`tryBle`/`tryTcpMdns`/`tryCore` closures passed in from the caller, which
presumably already capture what they need from `MeshRepository`'s scope.

A structurally similar pattern exists in `ConversationsScreen.kt:271-272`
(`ConversationRow` composable, name unconfirmed  read the surrounding
function signature to get the exact name): `deliveryState` and
`onNavigateToChat` parameters are also `@Suppress("UNUSED_PARAMETER")` with
default values, suggesting they were added for a feature (per-message
delivery-state display, tap-to-navigate) that was never wired into the
composable body.

## Why this needs planning, not a direct fix

Three different explanations are consistent with what's in the code, and
picking the wrong one either deletes a parameter another change is mid-way
through wiring up, or leaves genuinely dead plumbing in place indefinitely:

1. **Genuinely dead plumbing**  these params were added in anticipation of
   a feature (structured trace logging keyed on `traceMessageId`/
   `attemptContext`; a `deliveryState` badge in the conversation row) that
   was descoped or deferred, and nobody went back to remove the now-unused
   parameters and their `@Suppress` annotations. Correct fix: remove the
   parameters, update the call site(s), drop the suppressions.
2. **Incomplete instrumentation**  `traceMessageId` and `attemptContext`
   look purpose-built for delivery-path tracing/diagnostics (compare to the
   `Timber.i`-heavy tracing already present elsewhere in
   `MeshRepository.kt`'s transport code). If so, the correct fix is to
   actually thread them into the `Timber` log lines inside
   `attemptDelivery()` (e.g. `Timber.d("[$traceMessageId] attempt via $type
   ($attemptContext)")`) rather than deleting them  deleting would remove
   diagnostic capability the caller clearly intended to provide.
3. **`deliveryState`/`onNavigateToChat` in `ConversationsScreen.kt`
   specifically may be intentional API surface**  default-valued unused
   parameters on a composable can be a deliberate "reserved for caller" API
   shape (e.g. future per-row tap-to-open-chat behavor) rather than a bug.
   Needs a decision on whether `ConversationsScreen`'s conversation list rows
   are supposed to navigate to the chat on tap already (if they do via some
   other code path, `onNavigateToChat` is confirmed dead and should go; if
   tapping a conversation currently does nothing, this may be a missing-
   wiring bug, not dead code).

## What needs to be answered before implementation

- Is there a tracing/observability requirement (existing design doc, HANDOFF
  task, or product decision) that `traceMessageId`/`attemptContext` were
  meant to serve? Check `HANDOFF/done/` for any prior task that introduced
  these parameters  the commit that added them will explain intent.
- Does tapping a row in the conversations list currently navigate to the
  chat screen via a different mechanism (e.g. an `onClick` on the outer
  `Row`/`Card` rather than via `onNavigateToChat`)? Read the full body of the
  composable at `ConversationsScreen.kt` around line 271 (not just the
  signature) to check.
- If the answer to both is "no design intent found, no other navigation path
  exists"  this is a bug (dead navigation callback) and should be
  re-classified as a straightforward wiring fix rather than a cleanup.

## Acceptance Criteria (once the above is resolved)
- [ ] A decision is recorded (in this file or a follow-up) on whether each
      unused parameter is: (a) removed as dead code, or (b) wired up to its
      intended behavior.
- [ ] Whichever path is chosen, `@Suppress("UNUSED_PARAMETER")` no longer
      appears on the resolved parameters.
- [ ] `./gradlew assembleDebug -x lint --quiet` succeeds.

## Files Involved
- `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt` (lines ~296-310)
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (call site, ~line 5896)
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt` (lines ~271-272 and the composable's full body)

## Verification (once implemented)
```bash
cd android
./gradlew assembleDebug -x lint --quiet
grep -rn "UNUSED_PARAMETER" app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt \
  app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt
```
