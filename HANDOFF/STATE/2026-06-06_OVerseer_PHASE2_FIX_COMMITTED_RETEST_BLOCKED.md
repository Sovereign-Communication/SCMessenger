# 2026-06-06 — Overseer: PHASE 2 fixed and committed; live retest BLOCKED

**Status:** P0_ANDROID_025 fix committed on `fix/p0-android-025-mdns-listener-collision`; build green; live mDNS retest on Pixel 6a blocked by phone being offline
**Date:** 2026-06-06 01:00 PT
**Commit:** `e84f4fc3` on `fix/p0-android-025-mdns-listener-collision` (off `origin/main` dd109707)
**Worktree:** `E:\SCMessenger-build-p0-025\`

---

## What happened in PHASE 2

1. **Worktree created** off `origin/main` at `E:\SCMessenger-build-p0-025\`, branch `fix/p0-android-025-mdns-listener-collision`.
2. **Copied `android/local.properties`** from main repo (the new worktree didn't have it).
3. **JDK 17 path correction**: first build attempt failed because `JAVA_HOME=E:\build-tools\jdk17\jdk-17.0.0.14` doesn't exist; actual path is `E:\build-tools\jdk17\jdk-17.0.14`. Fixed.
4. **Build succeeded** with NDK 26b override `-Pandroid.ndkVersion=26.1.10909125`. APK at `app/build/outputs/apk/debug/app-debug.apk`, 291MB. Exit 0. CXX1104 warnings about ndkVersion mismatch (30 vs 26) are non-fatal — `local.properties` `ndk.dir` takes precedence and points to 26b.
5. **Fix applied** to `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt`:
   - Removed `getResolveListener()` singleton.
   - Added `inFlightResolves: ConcurrentHashMap<String, NsdManager.ResolveListener>` to track in-flight resolves by service name.
   - Added `newResolveListener(serviceName: String): NsdManager.ResolveListener` factory that creates a per-call listener and self-removes from the in-flight set on either terminal callback.
   - Updated `resolveService()` to construct a fresh listener per call and register it in the in-flight set.
   - Updated `stop()` to clear `inFlightResolves` for tidiness.
   - Net: +38/-15 LoC. All comments carry P0_ANDROID_025 markers.
6. **Committed** as `e84f4fc3` on `fix/p0-android-025-mdns-listener-collision`. **Not pushed.**
7. **Live retest BLOCKED**: Pixel 6a is offline. adb shows no devices; mDNS discovery of `_adb-tls-connect._tcp` returns empty; ARP table on x.x.x.x/24 has no Pixel 6a MAC. Phone is either WiFi-asleep, USB-unplugged, or both. Windows CLI relay from prior session also no longer running (PID 5072 is `svchost` owning 9001/9002 — the prior `scmessenger-cli.exe` is gone).

## Acceptance gates status (from HANDOFF/todo/P0_ANDROID_025)

- [x] Build verification: `./gradlew :app:assembleDebug` succeeds (NDK 26b override, 291MB APK).
- [ ] App survives `onServiceFound` callbacks for at least 2 distinct mDNS peers without crashing. **BLOCKED** by phone offline.
- [ ] App survives the same peer re-broadcasting (the common case on Android 14+). **BLOCKED**.
- [ ] No FATAL EXCEPTION in logcat during a 5-min Android<->Windows LAN session. **BLOCKED**.
- [ ] The Windows CLI's `discovery peers` output shows the Android phone's peer-id within 60 sec. **BLOCKED** (also needs CLI relay to be restarted).
- [ ] Hand off a post-mortem to `HANDOFF/STATE/`. This file IS the post-mortem (with the retest gates marked blocked).

## What Lucas must do to unblock retest

1. Plug the Pixel 6a into USB (and/or wake it from WiFi-asleep).
2. Allow USB debugging (tap "Allow" on the RSA fingerprint prompt).
3. From `E:\SCMessenger-build-p0-025\android`: `adb -s adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp install -r app/build/outputs/apk/debug/app-debug.apk`
4. From `E:\SCMessenger-Github-Repo\SCMessenger\target\debug\`: start CLI relay: `scmessenger-cli.exe relay --listen /ip4/0.0.0.0/tcp/9101 --http-port 9102`
5. Launch the app and confirm: no FATAL EXCEPTION, the Windows CLI's `discovery peers` output shows the Android phone's peer-id within 60 sec.

The Hermes Telegram gateway has been notified; awaiting a reply.

## What remains

- [ ] PHASE 3: commit `HANDOFF/CLAUDE_CODE_PROTOCOL.md` + 4 integration-branch changes (build.gradle, BleScanner.kt, ContactsViewModelTest.kt x2) as a separate commit on `integration/v0.2.2-pre-android-push-2026-06-05`.
- [ ] PHASE 4 (non-blocking): dispatch `[META]_ORCHESTRATOR_WORKER_POOL_WARMUP.md` worker pool ticket.
- [ ] Do NOT push. Do NOT exit. Idle in folder-monitor on `HANDOFF/`.
