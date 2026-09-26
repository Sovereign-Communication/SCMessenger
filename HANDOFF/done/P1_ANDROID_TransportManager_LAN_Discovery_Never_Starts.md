# TASK: P1-ANDROID-TRANSPORTMANAGER-LAN-SILENT  Android's LAN discovery (mDNS + TCP subnet probe) never engages, even on a fresh install with default settings

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Tier:** [SONNET] [DEVICE]
**Gates:** Kotlin/Android only, does not touch `core/src/crypto|transport|routing|privacy` (Rust side)  no `crypto-security-auditor` gate. Standard Android pre-merge checklist applies (`.claude/rules/android.md`).

## Source

Discovered live during a 2026-07-05 native `/scm` tandem debug session working
`HANDOFF/todo/P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`
(see that ticket's "Progress (2026-07-05)" section for the full session log). This
ticket is the split-out root-cause investigation for a more foundational bug that
session surfaced.

## Problem (exact, verified)

On a completely fresh Android install (clean uninstall + `./gradlew clean
:app:installDebug`, default settings, `internetEnabled` defaults `true`) tandem-
tested against a freshly-rebuilt Windows CLI on the same LAN, the phone showed
"no nearby peers" for the full ~8 minute test window and the CLI logged zero
connection activity from the phone at all.

Root cause traced to the Android side: `MeshRepository.kt:2162` (inside a
`repoScope.launch { ... }` init block, `:2140-2169`) calls
`transportManager?.startAll(enableMdns = settings.internetEnabled)` after
constructing BLE/WiFi/Swarm transports. `TransportManager.startAll()`
(`TransportManager.kt:103-138`) is what starts BOTH of Android's LAN-discovery
mechanisms:
1. `MdnsServiceDiscovery` (NsdManager `_p2p._udp` registration + discovery).
2. `SubnetProbe` (active TCP connect-scan of local subnets on ports 9001/9002 
   the documented mDNS workaround, since multicast doesn't cross routers/VLANs).

Both mechanisms are correctly implemented and correctly wired as constructor
calls + `.start()` invocations inside `startAll()`. But across the app's **entire
logcat buffer since install** (22,089 lines, covering the whole session), there is
**zero** occurrence of:
- `"SubnetProbe"` (any log line from that class at all, including its own
  `start()` log `"SubnetProbe starting (interval=...)"`)
- `"All transports started"` (the line `startAll()` logs immediately after
  starting both mDNS and the probe  neither the mDNS-enabled nor mDNS-disabled
  variant of this line appears, meaning `startAll()` itself likely never even
  reached that point, or was never called)
- `"mDNS service registered"` (confirms `MdnsServiceDiscovery.registerService()`'s
  NSD call never completed even its own registration)
- `"TransportManager startAll failed"` (the warning the surrounding try/catch at
  `MeshRepository.kt:2160-2165` would log on a thrown exception  its absence
  rules out an exception path)

Meanwhile `MeshRepository` itself was demonstrably alive and logging throughout
(e.g. periodic `"Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)"`), so this
isn't a total process crash  it's specifically this init block's mDNS/probe
branch that never fired, or fired with the wrong receiver.

## Root Cause (leading hypothesis, not yet confirmed  this ticket's job)

`transportManager` is `@Volatile private var transportManager: TransportManager? =
null` (`MeshRepository.kt:323`), assigned at `MeshRepository.kt:862`. The call at
`:2162` uses a nullable safe-call (`transportManager?.startAll(...)`)  if
`transportManager` is still `null` when this coroutine runs (e.g. an
initialization-order race between the assignment at `:862` and this
`repoScope.launch` block, which may run concurrently or before `:862` completes on
some code paths), the call silently no-ops: no exception, no log, exactly matching
what was observed. This needs to be confirmed (not assumed) by:
1. Adding a temporary diagnostic log immediately before `:2162` printing whether
   `transportManager` is null at that exact point, OR
2. Reading the actual call graph/ordering between wherever `:862`'s assignment
   happens and wherever the `:2140` init block is triggered from  are they
   guaranteed sequential, or can `:2140`'s block run on a different
   coroutine/thread before `:862` completes?

Alternative hypothesis to rule out: the `:2140-2169` init block might simply not be
on the code path actually exercised when the app starts fresh (e.g. it might be
gated behind a condition  identity restore vs fresh creation, or a specific
service-start trigger  that a normal fresh-install-then-launch flow doesn't hit).
Trace the caller(s) of the function containing this block before concluding it's
purely a null-race.

## Blast Radius

`android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (the
init/coroutine sequencing around `:2140-2169`, and wherever `transportManager` is
assigned at `:862`), `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt`
(no code change expected here  it's correctly implemented; useful only for
confirming call contracts). This blocks the "EASY mode" same-LAN cell of the
Phase 1 exit matrix (`HANDOFF/plans/P1-15_transport_matrix_audit.md`  that
matrix's mDNS/LAN row will need re-marking once this is understood; it currently
reads "wired both sides" based on the 2026-07-04 report, which predates this
finding).

## Files to Touch

- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
  (`:862` assignment site, `:2140-2169` init block  read the actual sequencing
  first, do not guess a fix before confirming the null-race or alternate-path
  hypothesis)

## Verification Commands

```bash
cd android && ./gradlew assembleDebug -x lint --quiet
./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.RoleNavigationPolicyTest"
```

Manual (device, required  this is fundamentally a live-init-sequencing bug):
`adb logcat -c` (clear buffer), fresh install + cold launch, then within seconds
confirm `"All transports started (including mDNS LAN discovery + TCP subnet
probe)"`, `"mDNS service registered: ..."`, and `"SubnetProbe starting
(interval=..."` all appear. Then confirm `SubnetProbe` actually finds an open
Windows CLI port on the same LAN (`"SubnetProbe: open port ... -> ..."`) within
one 30s sweep interval, and that the CLI's swarm eventually attempts a dial
(reaching the negotiation stage that `P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`
investigates next).

## Do NOT

- Do NOT touch `TransportManager.kt`, `MdnsServiceDiscovery.kt`, or `SubnetProbe.kt`
  themselves unless the investigation shows the bug is actually inside one of
  them  all three read as correctly implemented; the leading hypothesis is a
  caller-side sequencing bug in `MeshRepository.kt`.
- Do NOT assume the null-race hypothesis without confirming it (add a diagnostic
  log or trace the actual coroutine/call ordering)  the alternate hypothesis
  (this init block isn't reached on the exercised path at all) is equally
  plausible from the evidence gathered so far.
- Do NOT mark `P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`'s
  negotiation-failure investigation as re-attempted until this ticket confirms
  LAN discovery actually engages  retesting before this lands will just
  reproduce the same silent-discovery stall.
