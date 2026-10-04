<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

\# [NEEDS PLANNING] Android  SmartTransportRouter.attemptDelivery Accepts and Discards Real Data

## Context

Fresh sweep of `android/app/src/main/java/com/scmessenger/android/` (2026-07-04),
independent of the T8-T13 items in `docs/release-readiness-2026-07-02.md` 7.
Not a duplicate of any tracked item.

`transport/SmartTransportRouter.kt`, fn `attemptDelivery` (~line 296), has
four parameters marked `@Suppress("UNUSED_PARAMETER")`:

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

The caller, `MeshRepository.kt` (~line 5896), passes real data for all four:
the actual encrypted payload (`envelopeData = encryptedData`), the listener
multiaddr list, a trace message ID, and an attempt-context string used
throughout the surrounding code for `logDeliveryAttempt(...)` calls (dozens
of call sites in the same file, e.g. lines 5873, 5885, 5919, 5929...).

Verified this is **not currently a functional bug**: the `tryWifi`/`tryBle`/
`tryTcpMdns`/`tryCore` lambdas passed into `attemptDelivery` are closures
defined in `MeshRepository.kt` that capture `traceMessageId`,
`attemptContext`, and `encryptedData` from their own enclosing scope
directly  they don't rely on `attemptDelivery` re-passing these values back
to them. So today, nothing is silently broken by these being unused inside
`attemptDelivery` itself.

## Why this needs a decision, not just a fix

Two legitimate directions, and picking wrong either wastes work or leaves a
correctness trap for the next person who touches this function:

1. **Delete the four unused parameters** from `attemptDelivery`'s signature
   and simplify the `MeshRepository.kt` call site. This is safe today given
   the closure-capture behavior confirmed above, but it means
   `SmartTransportRouter`  which already tracks `TransportHealthMonitor`
   stats, dedup cache, and per-transport success rates internally  has no
   way to correlate its *own* internal logging/dedup entries
   (`getDedupStats(messageId)` at line ~275, `messageDedupCache`) with the
   `traceMessageId` used by `MeshRepository`'s `logDeliveryAttempt` calls,
   because it never receives that ID. Whether that correlation is wanted
   is a product/debugging-tooling decision, not a code-mechanics one.

2. **Wire the four parameters in for real**  use `envelopeData` if
   `SmartTransportRouter` should independently verify payload size/validity
   before racing transports, use `traceMessageId`/`attemptContext` in the
   router's own `Timber` logging so its log lines correlate with
   `MeshRepository`'s `logDeliveryAttempt` trail (search both files for the
   same trace ID), and use `listeners` if the router should attempt a
   listener-based fallback dial when the primary peer IDs are all null
   (currently `listeners` is accepted but the router has no listener-dial
   logic at all  worth checking if that's a known gap or intentional).

## Question that needs an answer before implementing

Does `SmartTransportRouter`'s own internal state (dedup cache, health
monitor) need to be correlatable with `MeshRepository`'s
`logDeliveryAttempt` trace IDs for debugging delivery issues in the field?
If yes  direction 2 (wire in `traceMessageId`/`attemptContext` for logging
at minimum). If the router is meant to stay a "dumb" transport-racing
utility with all attribution logic living in `MeshRepository`  direction 1
(delete the dead parameters, keep `attemptDelivery`'s signature minimal).

Do not guess at this  it changes the shape of the function signature and
touches the one call site in `MeshRepository.kt`, so getting it wrong means
two changes instead of one.

## Files

- `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt`
  (fn `attemptDelivery`, ~line 296)
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
  (call site, ~line 5896)

## Verification (once direction is chosen)

```bash
cd android
./gradlew assembleDebug -x lint --quiet
```
If parameters are wired in for logging, manually trace one delivery attempt
in logcat and confirm the same trace ID appears in both
`SmartTransportRouter`'s and `MeshRepository`'s log lines.
