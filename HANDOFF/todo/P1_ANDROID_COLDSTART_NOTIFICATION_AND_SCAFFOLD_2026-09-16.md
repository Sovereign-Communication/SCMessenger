# P1: Android Cold-Start Notification Buffering & Scaffold Layout De-Nesting (AND-01 & AND-02)

**Status:** OPEN  
**Priority:** P1 (v0.4.0 Release Blocker)  
**Target Branch:** `feat/v040-multi-transport-store-forward`  
**Components:** `android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt`, `android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt`, `PeerListScreen.kt`, `TopologyScreen.kt`, `MeshApplication.kt`  
**Reference Audit:** `HANDOFF/audit/SHADOW_AUDIT_V040_V050_ADVERSARIAL_REVIEW_2026-09-16.md`

## Problem Description
1. **Cold-Start Notification Loss (AND-01)**: In `NotificationHelper.kt:101`, `notificationsEnabled` is initialized to `null`. During cold boot, reading preferences from DataStore takes 50-300ms. Inbound messages arriving during this window are permanently dropped as `suppressed_settings` without buffering or replay.
2. **Nested Scaffold Subcomposition Crash (AND-02)**: In `MeshApp.kt` and child screens (`PeerListScreen.kt`, `TopologyScreen.kt`), nesting child `Scaffold` composables inside `MeshNavHost` creates two layers of `SubcomposeLayout`. Disposing during navigation causes negative slot table index corruption (`SlotTableKt.dataAnchor`). In `MeshApplication.kt`, the exception handler swallows this crash, leaving the main thread dead and the app in an unrecoverable zombie ANR state.
3. **SubnetProbe Port 9001 Omission (AND-03)**: `RAW_TCP_PORTS` omits `9001`, causing the node to dial HTTP ports on LAN hosts rather than libp2p swarm listeners.

## Acceptance Criteria
1. Buffer inbound notifications during cold start in an in-memory queue until DataStore hydration completes.
2. Hoist the single authoritative `Scaffold` to `MeshApp.kt`; replace nested `Scaffold` in child screens with `Column(Modifier.fillMaxSize())`.
3. Remove the exception-swallowing handler in `MeshApplication.kt`.
4. Add `9001` as the primary port in `SubnetProbe.RAW_TCP_PORTS`.
