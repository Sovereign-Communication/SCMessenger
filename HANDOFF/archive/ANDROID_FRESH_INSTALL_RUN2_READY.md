# Android Fresh Install for 5-Node Run 2

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** READY TO EXECUTE
**Priority:** CRITICAL — must be fresh when testing begins
**Date:** 2026-08-04
**Device:** Pixel 6a
**Method:** Wireless ADB (enabled)

---

## Current State

- APK built: `android/app/build/outputs/apk/debug/app-debug.apk` (48M, Aug 2)
- ADB: Wireless ADB enabled and ready
- Identity canonicalization: PR #136 in CI (awaiting merge)

---

## Execute When Ready (Anytime Now)

### 1. Fresh Install

```bash
# Ensure wireless ADB connected
adb devices  # should show Pixel 6a

# Uninstall old app
adb uninstall com.scmessenger.android 2>/dev/null || true

# Install fresh APK
adb install -r android/app/build/outputs/apk/debug/app-debug.apk

# Verify installed
adb shell pm list packages | grep scmessenger
```

### 2. Start Mesh + Capture Baseline

```bash
# Clear old logs
adb logcat -c

# Launch app
adb shell am start -n com.scmessenger.android/.MainActivity

# Capture logcat (let run for 60 seconds)
adb logcat > /tmp/android_fresh_baseline.log &
LOGCAT_PID=$!
sleep 60
kill $LOGCAT_PID 2>/dev/null || true

# Check for Mesh startup
echo "=== MESH STARTUP VERIFICATION ==="
grep -i "mesh\|swarm\|identity\|initialized" /tmp/android_fresh_baseline.log | head -20

# Check for errors/panics
echo "=== ERROR CHECK ==="
grep -i "panic\|fatal\|error" /tmp/android_fresh_baseline.log | grep -v "E/tag" || echo "No critical errors"
```

### 3. Verify Identity

```bash
# Check identity is initialized (not NotInitialized)
adb logcat | grep -i "identity" | head -5

# Check for BLE advertising active
adb logcat | grep -i "ble\|advertis" | head -5

# Check any transport listener bound
adb logcat | grep -i "listener\|bind\|listening" | head -5
```

### 4. Accept Criteria

- [OK] App installed fresh, old version uninstalled
- [OK] Mesh starts without panics
- [OK] Identity initialized (public_key + identity_id present)
- [OK] At least one transport listener active (BLE, TCP, mDNS, relay)
- [OK] No "wrong key" or "crypto" errors in logcat
- [OK] Baseline logs saved for correlation

---

## After Installation

Once confirmed working:
- [CHECK] Note exact app version (check Settings → About)
- [CHECK] Capture device info: `adb shell getprop | grep -E "ro.build|ro.product"`
- [CHECK] Store baseline logs: `/tmp/android_fresh_baseline.log`
- [CHECK] Ready for 5-node run 2 test matrix

---

## If Mesh Does NOT Start

Check:
1. `adb logcat | grep -i "exception\|error"` — exact stacktrace
2. `adb shell ls -la /data/data/com.scmessenger.android/` — app data directory exists?
3. `adb shell pm grant com.scmessenger.android android.permission.ACCESS_FINE_LOCATION` — permissions granted?
4. Recent changes: PR #133, #134, #135 (identity)

---

## Wireless ADB Prep (If Not Already Done)

```bash
# Enable wireless ADB on device (via USB first)
adb tcpip 5555

# Connect to device IP (find via Settings → Developer Options → IP)
adb connect <device-ip>:5555

# Verify connection
adb devices
```

---

## Timeline Integration

- **T-0**: APK ready ([DONE] Aug 2)
- **T+0**: PR #136 CI passes
- **T+15min**: PR merged to main
- **T+20min**: Docker image published
- **T+25min**: AWS node updated
- **T+30min**: ANDROID FRESH INSTALL (now)
- **T+40min**: iOS/macOS confirmed (GPT handoff)
- **T+45min**: All 5 nodes fresh + healthy
- **T+50min**: 5-NODE RUN 2 START

---

## Acceptance

When this completes: deliver `HANDOFF/gpt/ANDROID_FRESH_INSTALL_RUN2_2026-08-04.md`

Contents:
- Device: Pixel 6a
- APK version: [from about screen]
- Identity status: initialized (pk:[...], id:[...])
- Transports active: [which ones listening]
- Baseline logs: [attached]
- Status: READY FOR RUN 2
