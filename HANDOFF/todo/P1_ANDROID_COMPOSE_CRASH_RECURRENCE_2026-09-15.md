# P1 - Android: Compose crash class recurs on device (2 FATALs in 52 minutes)

Status: Open
Priority: P1 (blocks the "no crashes" v0.4.0 claim; operator reported "still
hits a crash" on the mesh screen and this is the concrete artifact)
Filed: 2026-09-16T04:2xZ by the Freebuff lane, passive three-node audit
Device: Google Pixel 6a, Android 17 (API 37), wireless adb
Build on device: versionName 0.4.0, versionCode 15
  Git 09f52799a051aab3afb86a25731ada7a4d382911, ref 288/merge
  Build time 2026-09-14T23:16:00Z
  Installed lastUpdateTime=2026-09-14 13:56:34 (device local)

## Evidence (all commands run this session, passive reads only)

On-device crash reports written by the app's own crash handler:

    adb shell run-as com.scmessenger.android ls -la files/ | grep crash
    -rw------- 3517 2026-09-15 16:43  crash_1789526624102.log
    -rw------- 8088 2026-09-15 17:35  crash_1789529711204.log

Both headers carry `ComposePrefetchCrash: true`.

Crash A - 2026-09-16T02:43:44Z (device-local 09-15 16:43), during
recomposition/insert on the UI frame clock:

    java.lang.ArrayIndexOutOfBoundsException:
        src.length=16 srcPos=4 dst.length=16 dstPos=5 length=-1
        at java.lang.System.arraycopy(Native Method)
        at kotlin.collections.ArraysKt___ArraysJvmKt.copyInto
        at androidx.compose.runtime.collection.MutableVector.add
        at androidx.compose.ui.node.MutableVectorWithMutationTracking.add
        at androidx.compose.ui.node.LayoutNode.insertAt$ui_release
        at androidx.compose.ui.node.UiApplier.insertBottomUp
        at androidx.compose.runtime.changelist.Operation$PostInsertNodeFixup.execute
        at androidx.compose.runtime.CompositionImpl.applyChanges
        at androidx.compose.ui.platform.AndroidUiDispatcher...(Choreographer frame)

Crash B - 2026-09-16T03:35:11Z (device-local 09-15 17:35), at activity teardown:

    java.lang.RuntimeException: Unable to destroy activity
        {com.scmessenger.android/com.scmessenger.android.ui.MainActivity}
      Caused by: java.lang.ArrayIndexOutOfBoundsException: length=5120; index=-1446
        at androidx.compose.runtime.SlotTableKt.dataAnchor(SlotTable.kt:4003)
        at androidx.compose.runtime.SlotWriter.moveSlotGapTo
        at androidx.compose.runtime.SlotWriter.removeSlots
        at androidx.compose.runtime.CompositionImpl.dispose
        at androidx.compose.ui.platform.WrappedComposition.dispose

Corresponding logcat crash buffer in the same hour:

    adb logcat -d -b crash -t 300
    09-15 17:35:11.200 E AndroidRuntime: FATAL EXCEPTION: main
    09-15 17:35:11.200 E AndroidRuntime: Process: com.scmessenger.android, PID: 25449
    ... SlotTableKt.dataAnchor(SlotTable.kt:4003) ... length=5120; index=-1446

The process actually died: `adb shell pidof` now returns 26289, and
`ps -o ELAPSED` puts that PID's start ~17:37 local - the 17:35 FATAL ended
PID 25449.

## Why the existing guard did not save it

`ComposePrefetchCrash: true` in the report header shows the app's classifier
DID recognise the crash class and wrote the report - yet both FATALs
propagated. Both are thrown on the main thread inside framework dispatch
(Choreographer frame callback; ActivityThread lifecycle dispatch for
onActivityPreDestroyed), i.e. outside the composition-scoped guard. A guard
that records a crash but lets the process die is not a fix; the crash class
is still OPEN on device.

## Why this pass's committed fixes cannot be credited yet

The device APK was installed 2026-09-14 13:56:34, before every commit of this
work stream (de54001a polish pack, 0448013d notification sentinel, 765dd70a
Identicon memoization - all dated 2026-09-15). So nothing in this stream is
exercised on the handset. Independently, none of those three changes targets
the signatures above: memoizing `Identicon` byte arrays per peerId does not
touch `SlotTable`/`MutableVector` slot-gap arithmetic or LayoutNode insertion,
and the notification sentinel is a different subsystem.

## Root-cause determination (2026-09-16, code + history, not soak)

Class. Both signatures are the same class of failure: the recomposition
change list / slot table disagreeing with the node tree, at composition
teardown. Each artifact was read in full this session (tmp/crashA.log,
tmp/crashB.log, pulled from the device before the package was reinstalled).

Crash A mechanism (`src.length=16 srcPos=4 dst.length=16 dstPos=5 length=-1`).
`MutableVector.add(index, element)` copies `content.copyInto(content,
index + 1, index, size)` -> length `size - index`. The args fix `index = 4`
and `size = 3`: the applier was told to insert the node at child index 4 into
a parent that held 3 children. That is a STALE INSERT INDEX, resolved by
`Operation$PostInsertNodeFixup` after the same change list had already removed
children from that parent; nothing in Kotlin sets that index. The tail of the
stack is `AndroidUiFrameClock.withFrameNanos ... AndroidUiDispatcher` dispatched
from `Choreographer`, running `Recomposer$runRecomposeAndApplyChanges` - and the
throwable carries
`Suppressed: DiagnosticCoroutineContextException: [..., StandaloneCoroutine{Cancelling}, AndroidUiDispatcher]`,
i.e. the frame callback applied a pending change list inside a recomposition
coroutine that was ALREADY CANCELLING. That is composition teardown
interleaved with change application, not steady-state recomposition of list
content.

Crash B mechanism (`length=5120; index=-1446`). `SlotTableKt.dataAnchor`
returned a negative anchor during `SlotWriter.moveSlotGapTo/removeSlots` from
`CompositionImpl.dispose`. The dispose chain is the root `WrappedComposition`
-> a node that owns a subcomposition (`LayoutNodeSubcompositionsState
.onRelease`) -> a second, nested one -> the innermost slot table read. A
negative anchor index is corrupt bookkeeping inside that innermost
subcomposition's table; no app frame appears at any level.

Why this is not the mesh/topology UI layer.
- Neither artifact contains a single application frame: inner frames are
  Compose's own applier/slot-table machinery, outer frames are
  Choreographer/ActivityThread teardown.
- `grep -rnE "LazyColumn|BoxWithConstraints|HorizontalPager|Subcompose"` over
  `ui/dashboard/` and `ui/screens/DashboardScreen.kt` returns only COMMENTS:
  the mesh screens are `Column(verticalScroll)` + `Card`/`Row`/`Canvas` and
  own no subcomposition, so they cannot be the disposing node in the crash-B
  chain; they also cannot be LazyLayout prefetch, which crash A does not show.
- Their keyed lists cannot collide keys: `DashboardViewModel.kt:541` already
  does `filter { it.peerId.isNotBlank() }.distinctBy { it.peerId.trim() }`
  before `_peers.value` is set, and `sortPeersForUnifiedView` feeds from it.
- The subcomposition levels the crash-B chain walks come from the framework
  and shell, verified against the resolved artifacts in the local Gradle
  cache, not from memory: `material3-android/1.3.1` `ScaffoldKt.class`
  references `SubcomposeLayout` (every screen Scaffold is a subcomposition),
  and `navigation-compose/2.7.5` `ComposeNavigator$Destination.class`
  references `AnimatedContent` (NavHost renders destinations through it). This
  app nests Scaffolds - app shell (`MeshApp.kt`) plus each screen's own -
  which is exactly two nested subcomposition levels below the root.

Does the mechanism still exist on the tip? Yes, and it never lived in the mesh
layer. `git diff --stat b39bfd2d ee93394c -- .../android/ui/` reports the whole
UI tree changed in exactly TWO files (`Identicon.kt` +22/-9,
`TopologyScreen.kt` +18/-1); `MeshApplication.kt` (the guard) is byte-identical
between the crashing build (b39bfd2d content, `288/merge` = this branch) and the
tip. Nothing on this branch touches applier index resolution, slot-gap
arithmetic, the recomposer cancellation path, or the nested-Scaffold/
NavHost-graph subcomposition surface. Crash A and B (02:43:44Z / 03:35:11Z on
09-16) also both PREDATE this stream's Android fixes (de54001a at 04:02:49Z).
Consequence: no mesh/topology-layer edit can be credited with fixing either
signature, so none was made as a fix. See "Change made" below for the one
narrow change that was made, and its honest label.

## Guard finding (report only - the guard is outside the mesh/topology layer)

`MeshApplication.kt` `installGlobalCrashHandler` classifies via
`isComposePrefetchCrash` and, on a match, returns without chaining to the
previous handler, with the comment "process kept alive for recovery". That
claim is not supported by the artifacts:
- Crash B was classified (`ComposePrefetchCrash: true`, crash file
  `crash_1789529711204.log` written 17:35:11.204 local) yet the crash buffer
  holds `09-15 17:35:11.200 E/AndroidRuntime(25449): FATAL EXCEPTION: main /
  Process: com.scmessenger.android, PID: 25449` - the framework log lands 4 ms
  BEFORE the classifier's own file, and the process did die (PID later 26289).
- A destroy-time failure thrown out of `ActivityThread.performDestroyActivity`
  is not survivable by returning early from an uncaught-exception handler: the
  failed transaction leaves the activity thread broken.
- The classifier's terms are bare substrings - `"LayoutNode"`, `"SlotTable"`,
  `"MutableVector"` - which appear in nearly every Compose-internal stack, so
  it classifies far more than the intended family.
- The claim "The Compose runtime will recover on next recomposition" matches
  neither artifact.
Deliberately NOT changed here (it is `MeshApplication.kt`, not the mesh/
topology UI layer): this is the user's scope call.

## Change made on the branch (CHURN-001) and its honest label

`DashboardScreen.kt` and `PeerListScreen.kt` no longer wrap peer rows in
`key(peer.peerId)`. Both are plain `Column`s, and neither `PeerItem` nor
`PeerCard` holds per-item remembered state (the only `remember`s in either
file are list-level), so the key bought nothing - while every online/offline
re-sort (`sortPeersForUnifiedView`) forced a dispose+insert of each position
whose peer changed place. Those remove+insert operations are exactly the
change-list shape crash A misfired on. Positional composition emits content
updates only, so a re-sort now queues no structural operations.
Label: churn reduction, NOT a demonstrated fix of the runtime race. The
runtime race is still open (see Scope decision). Verified: `:app:
compileDebugKotlin` BUILD SUCCESSFUL, `:app:testDebugUnitTest` BUILD
SUCCESSFUL (44 suites / 334 tests / 0 failures / 3 skipped), APK installed in
place (`firstInstallTime` preserved 2026-09-15 21:04:34, identity
`f83ab163...` / `12D3KooWD776...` / nickname `LucasFix1` intact, mesh traffic
flowing: `[OK] Message delivered successfully to
12D3KooWD6vZQ...` at 11:18:34Z).

## Soak (passive, started 2026-09-16T11:00:02Z)

`tmp/soak/watch.sh` polls logcat + the app's crash-file dir every 60 s into
`tmp/soak/soak.log`. Baseline at start: `logcat_matches=3`, all three being the
historical 09-15 17:35 `dataAnchor` lines already in the crash buffer - any
INCREASE on that number is a new event. The historical latency was hours into a
session, so a short window cannot clear the class; the window is stated with
the result whenever it is read.

## Scope decision required (not mine to make)

1. Compose BOM upgrade (currently `compose-bom:2024.12.01`,
   `navigation-compose:2.7.5`, `activity-compose:1.8.0`) - the only lever that
   can fix a runtime-level applier/slot-table race, and it needs its own
   verification pass.
2. Reduce the nested-subcomposition surface the crash-B chain walks: drop the
   per-screen `Scaffold`s in favour of the app shell's, stop rebuilding the
   NavHost graph on every `hasIdentity` change (`MeshApp.kt` builds the graph
   inside `if (hasIdentity)`, and already carries debounce workarounds), and
   treat the `showOnboarding` tree swap as a teardown event.
3. Guard contract: either stop claiming recovery for teardown-time failures, or
   install a deliberate one-shot recovery path and accept its risks.

## Work items

1. DONE (this pass): both signatures read in full and matched to the runtime
   mechanism above; no app frames, no mesh/topology-layer defect.
2. DONE (this pass): mesh/topology composables audited - no lazy layout, no
   subcomposition, no duplicate/blank keys; churn removed (CHURN-001).
3. OPEN, operator scope call: which of the three options in "Scope decision".
4. OPEN: the crash-absence window must be read from `tmp/soak/soak.log` and
   reported with its length; a short window does not close a class whose
   historical latency is hours.
