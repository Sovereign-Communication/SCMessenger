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

## Work items

1. Reproduce with a symbolicated debug build (the APK is a release/minified
   build; Compose frames are unminified here but app frames would need the
   mapping) while holding the mesh screen and rotating/tearing down, to find
   which composable's node insertion corrupts the slot table.
2. Audit the mesh/topology composables for slot-table-corrupting patterns:
   unstable keys in `LazyColumn`/`key()`, composition of a changing number of
   children inside a non-`key`ed loop, and `SubcomposeLayout` disposal during
   recomposition. `TopologyScreen` builds a variable-length node ring inside
   `Canvas`/`Column`; `DashboardScreen`/`PeerListScreen` filter + `distinctBy`
   the peer list every emission.
3. Decide the guard's contract: if the app intends to survive this crash
   class, the guard must be installed at the process/ActivityThread level
   (or the offending composition must be isolated so its failure cannot take
   the activity down); today it only writes a report.
4. Re-verify on hardware after the fix: >= 1 hour on the mesh screen with
   network churn, asserting `files/crash_*.log` count does not grow and
   logcat shows no `SlotTableKt`/`MutableVector` frames.
