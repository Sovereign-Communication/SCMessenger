<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-11

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** `HANDOFF/STATE/PLAN_VERIFICATION_2026-06-11.md` 1 (Wi-Fi Aware gap)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (Android logging + Kotlin test)
**Rationale:** Per verification, `core/src/transport/wifi_aware.rs:add_discovered_peer` has no external callers. Stub-only. The Android side has Wi-Fi Aware publish/subscribe but doesn't call into the core stub. Add a one-way wired integration: when the Android `WifiAwareManager` discovers a peer, log + count it (visibility into the dormant path). ~30 LoC Kotlin. Trivial for Flash.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 6000

# P1_GEMINI_FLASH_015  Android Wi-Fi Aware Discovery Hook + Stub-Call Counter

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  Wi-Fi Aware visibility
**Source:** `HANDOFF/STATE/PLAN_VERIFICATION_2026-06-11.md` 1 (wifi_aware stub gap)
**Depends on:** none

---

## Verified Gap

`core/src/transport/wifi_aware.rs` has `add_discovered_peer` with **no external callers** (per ACTIVE_LEDGER 2026-05-13). The Android side has Wi-Fi Aware publish/subscribe in `WifiAwareManager.kt` but doesn't notify the core when a peer is discovered. The capability is dormant on both sides.

This ticket doesn't fully wire wifi_aware  that requires UniFFI work (separate scope). It adds an Android-side counter + log so we can SEE discovery events happening, which is the prerequisite for later full wiring.

## Scope (~30 LoC, 1 file)

In `android/app/src/main/java/com/scmessenger/android/transport/wifi/WifiAwareManager.kt`:

Add a `WifiAwareDiscoveryCounter`:
- Atomic counter of discovered peers (since process start)
- `onServiceDiscovered(peerId: String, serviceName: String)` callback
- Log at INFO level: `Wi-Fi Aware discovery: peer=$peerId svc=$serviceName (total=$count)`
- Expose `getDiscoveryCount(): Int` for diagnostics API

Wire into the existing `subscribe()` flow: call `counter.onServiceDiscovered(...)` when a peer is found.

## File Targets

- `android/app/src/main/java/com/scmessenger/android/transport/wifi/WifiAwareManager.kt` [EDIT  counter + callback, ~30 LoC]

## Build Verification

```bash
cd android
./gradlew :app:compileDebugKotlin --quiet
./gradlew :app:assembleDebug -x lint --quiet
# Unit test (if time):
./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.transport.wifi.WifiAwareCounterTest"
```

## Acceptance Gates

1. APK builds
2. `getDiscoveryCount()` returns 0 on cold start
3. Mock test: simulate 3 discoveries  counter returns 3
4. Logcat shows `Wi-Fi Aware discovery: peer=...` lines

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: KOTLIN] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 15]
