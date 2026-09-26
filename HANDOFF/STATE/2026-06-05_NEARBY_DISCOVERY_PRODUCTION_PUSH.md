# STATE: Nearby Discovery → Production-Code Push

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-06-05
**Owner:** Lucas Ballek (Overseer)
**Goal:** Bring Android codebase to "perfect / production-quality" so Lucas can manually upload to Google Play (Play Console already configured, just needs the code).
**Quota context:** Weekly ~50% used, 2 days remaining. Session resets in ~15 min from dispatch. Dispatching 3 subagents in parallel; their work continues after my session ends.

---

## What "perfect" means here (per Lucas's directive)

Code, not Play Console. The Play Console is his side. My side is:
- All known P0/P1/P2 Android tickets closed
- Nearby Discovery UI fully integrated and functional
- Production hardening (ProGuard, network config, FGS types, crash handler, signing config)
- Clean `assembleDebug` + `assembleRelease` builds
- No debug logging leaking in release
- Strings externalized
- Tests for the new code paths
- Honest documentation of what's still blocked (no faking)

**Out of scope** (Lucas's side, not mine):
- Play Console data safety form, privacy policy, content rating
- Real keystore provision (I added a placeholder, he swaps in his)
- Crashlytics / analytics / crash reporting service
- 14,000-device compatibility testing
- Localization beyond English

---

## Repository state at dispatch

**Branch:** main
**Last commit:** `87d1ef61 fix(android): FAB reappear + TCP subnet probe for LAN discovery`
**Uncommitted change:** `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` — added `onTransportPause()` (clean, well-commented, will be committed as part of this push).
**APK on disk:** `android/app/build/outputs/apk/debug/app-debug.apk` (291MB, built Jun 4 15:37). Includes commit `87d1ef61` and the SubnetProbe integration.

---

## Subagent workstreams (dispatched in parallel)

### Subagent A: Nearby Discovery UI Integration
**Goal:** `AddContactScreen.NearbyDiscoveryTab()` is currently a placeholder. Wire it to `viewModel.nearbyPeers`.

**Files owned (strict):**
- `android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt`
- `android/app/src/main/res/values/strings.xml` (AddContact-related keys only)

**Subtasks:**
1. Read `ContactsViewModel.nearbyPeers` as state
2. Empty state: CircularProgressIndicator + "Searching for nearby peers on this network..."
3. Empty after scan: "No peers found" + "Rescan" button
4. Permission missing state: rationale card + "Grant permissions" button → system settings
5. Populated state: LazyColumn of cards. Each card: IdenticonFromPeerId, nickname or peerId.take(16), transport icon (BLE/WiFi/mDNS/TCP), "Add" button.
6. "Add" button: calls `viewModel.promoteNearbyPeerToContact(peerId)` (if missing, add it to ContactsViewModel — explain in HANDOFF)
7. "Rescan" button at top: triggers fresh discovery (likely calls `meshRepository.replayDiscoveredPeerEvents()` or a new `meshRepository.refreshDiscovery()`)
8. All new strings → strings.xml with semantic keys (`add_contact_nearby_*`)

**Build gate:** `cd android && ./gradlew :app:assembleDebug -x lint --quiet` must succeed.

**Commit message:** `feat(android): wire NearbyDiscoveryTab to nearbyPeers StateFlow`
**HANDOFF section:** "Subagent A" below.

---

### Subagent B: Production Hardening
**Goal:** Release-readiness code (NOT Play Console side). All work Lucas can swap a real keystore into and upload.

**Files owned (strict):**
- `android/app/build.gradle` (add release signingConfig + buildTypes.release)
- `android/app/src/main/AndroidManifest.xml` (networkSecurityConfig, fullBackupContent, dataExtractionRules, foregroundServiceType)
- `android/app/src/main/res/xml/network_security_config.xml` (NEW)
- `android/app/src/main/res/xml/backup_rules.xml` (NEW)
- `android/app/src/main/res/xml/data_extraction_rules.xml` (NEW)
- `android/app/proguard-rules.pro` (NEW)
- `android/app/src/main/java/com/scmessenger/android/MeshApplication.kt` (UncaughtExceptionHandler + release-mode Timber tree)
- `android/app/src/main/java/com/scmessenger/android/service/MeshService.kt` (foregroundServiceType + ensure startForeground within 5s for Android 14+)
- `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt` (request POST_NOTIFICATIONS on Android 13+; rationale dialog if denied twice)

**Subtasks:**
1. **ProGuard/R8 rules:** Keep `uniffi.api.*`, Hilt-generated, Timber, Compose-related (defaults OK), Rust FFI signatures. Verify with `./gradlew :app:minifyReleaseWithR8` if possible (will fail without signing — that's expected).
2. **Network security config:** Block cleartext by default. Allow cleartext only to `127.0.0.1` (local daemon) and `10.0.2.2` (emulator host).
3. **Backup rules:** Disable auto-backup of identity keys (security). Allow prefs + database.
4. **Data extraction rules (Android 12+):** Match backup rules.
5. **Release signing config:** Placeholder. Real keystore path/alias/passwords via `~/.gradle/gradle.properties` or `ANDROID_KEYSTORE_PATH` env var. DO NOT include any actual keys. Print a loud warning at build time if keystore is missing.
6. **Global UncaughtExceptionHandler** in MeshApplication:
   - Log to internal files dir (not logcat, not external)
   - Stop MeshService gracefully
   - Re-throw to default handler
7. **Release Timber tree:** `if (BuildConfig.DEBUG) Timber.plant(Timber.DebugTree())` — no debug tree in release.
8. **Foreground service type:** Check `MeshService.startForeground()` — add `ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC` (or `CONNECTED_DEVICE` if BLE-heavy) for Android 14+ compliance. Call `startForeground()` within 5s of `startService()`.
9. **POST_NOTIFICATIONS permission:** Request via `ActivityResultContracts.RequestPermission()` in MainActivity on Android 13+. Show rationale dialog if denied twice.
10. **Build types in build.gradle:**
    - `debug`: debuggable, debug tree, dev URL endpoints
    - `release`: minifyEnabled, shrinkResources, ProGuard files, release signing (or debug fallback with warning), no debug tree

**Build gate:** `./gradlew :app:assembleDebug -x lint --quiet` must succeed. `./gradlew :app:assembleRelease -x lint --quiet` should produce a release APK even with placeholder keystore (use debug keystore as fallback, print warning).

**Commit message:** `feat(android): production hardening for release-readiness`
**HANDOFF section:** "Subagent B" below.

---

### Subagent C: Close 8 Open Android Tickets
**Goal:** Implement fixes for all open Android tickets in HANDOFF/todo/.

**Tickets (all under `HANDOFF/todo/`):**
- `[VALIDATED]_P0_ANDROID_022_Relay_Peer_Contacts_Filter.md` — verify `MeshRepository.upsertFederatedContact` filter is complete; add to other paths if missing.
- `[VALIDATED]_P1_ANDROID_022_BLE_Stale_Cache_Cleanup.md` — the uncommitted `BleScanner.onTransportPause()` is the fix; wire it into TransportManager lifecycle (e.g., on app background).
- `[VALIDATED]_P1_ANDROID_023_History_Persistence_Regression_Test.md` — add unit tests verifying message history persists across "app restart" (clear in-memory state, re-read from DB, assert messages still present).
- `[VALIDATED]_P1_ANDROID_AUDIT_LOG_VIEWER_001.md` — new `AuditLogScreen` + `AuditLogViewModel`. Read from audit log table, paginated list, basic filter by event type.
- `[VALIDATED]_P1_ANDROID_MESSAGE_SEARCH_UI_001.md` — search bar on Chats/Contacts screen. Filters by message content, sender, peer ID.
- `P2_ANDROID_IDENTITY_QR_PRERENDER.md` — pre-render the identity QR in a `LaunchedEffect` so it appears instantly when navigating to the screen.
- `P2_ANDROID_IDENTITY_SCROLL_FIX.md` — wrap IdentityScreen content in a `verticalScroll(rememberScrollState())` Column.
- `P1_ANDROID_CRASH_TRIAGE.md` — add `Timber.e(e, "...")` to the UncaughtExceptionHandler (overlaps with Subagent B — coordinate; Subagent B owns the handler, Subagent C documents the repro procedure in the HANDOFF).

**Files owned (strict):**
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` (verify uncommitted change is committed)
- `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt` (call onTransportPause on lifecycle event)
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (relay filter verification + any missing filter sites)
- `android/app/src/main/java/com/scmessenger/android/ui/screens/IdentityScreen.kt` (QR pre-render + scroll fix)
- `android/app/src/main/java/com/scmessenger/android/ui/audit/AuditLogScreen.kt` (NEW)
- `android/app/src/main/java/com/scmessenger/android/ui/audit/AuditLogViewModel.kt` (NEW)
- `android/app/src/main/java/com/scmessenger/android/ui/search/MessageSearchScreen.kt` (NEW) or integrate into existing chats/contacts screen
- `android/app/src/test/java/com/scmessenger/android/transport/ble/BleScannerTest.kt` (extend with onTransportPause test)
- `android/app/src/test/java/com/scmessenger/android/data/HistoryPersistenceTest.kt` (NEW)
- `android/app/src/main/res/values/strings.xml` (audit/search/identity-related keys only — DO NOT collide with Subagent A's `add_contact_nearby_*` keys)

**Subtasks:** One per ticket. Each ticket → fix → test → build → commit. Or one combined commit if small.

**Build gate:** `./gradlew :app:assembleDebug -x lint --quiet` AND `./gradlew :app:testDebugUnitTest --quiet` must succeed.

**HANDOFF section:** "Subagent C" below. Move each ticket file from `HANDOFF/todo/` to `HANDOFF/done/` after completion.

**Commit messages:** One per ticket, e.g. `feat(android): P0_ANDROID_022 verify relay peer filter`, `feat(android): P1_ANDROID_022 wire BleScanner.onTransportPause`, etc.

---

## Integration phase (next session, after my Ollama session resets)

**Steps:**
1. `git log --oneline -20` to see all subagent commits.
2. Resolve any merge conflicts (likely on `strings.xml` between A and C, and on `MeshApplication.kt` between B and C).
3. Re-run final build: `./gradlew :app:assembleDebug -x lint --quiet`
4. Set up Android emulator:
   - `cd /mnt/e/Android/sdk/cmdline-tools/latest/bin && ./sdkmanager --list_installed` to verify AVD system image
   - `avdmanager list avd` to see existing AVDs
   - If no AVD: `echo "no" | avdmanager create avd -n scmessenger_test -k "system-images;android-35;google_apis;x86_64" -d pixel_6`
   - `emulator -avd scmessenger_test -no-window -no-audio -no-boot-anim -gpu swiftshader_indirect &`
   - Wait for boot: `adb wait-for-device shell 'while [[ -z $(getprop sys.boot_completed) ]]; do sleep 1; done'`
5. Install: `adb install -r android/app/build/outputs/apk/debug/app-debug.apk`
6. Launch: `adb shell am start -n com.scmessenger.android/.ui.MainActivity`
7. Capture logcat for 60s: `adb logcat -d -t 60 > tmp/session_logs/emulator_smoke_$(date +%Y%m%d_%H%M).log`
8. Verify no FATAL EXCEPTION, no ANR, mesh service starts, identity loads.
9. **Then, ONLY IF all the above passes**, push to Lucas's Pixel 6a: `adb -s <pixel_serial> install -r android/app/build/outputs/apk/debug/app-debug.apk`
10. Final git commit: `chore: production-readiness push complete`
11. Update this HANDOFF file with the final results.

---

## Emulator location (verified earlier)

- ADB: `/mnt/e/Android/sdk/platform-tools/adb.exe` (in WSL PATH)
- SDK: `/mnt/e/Android/sdk/`
- Android Studio: `/mnt/e/Android/android-studio/`
- System images: should be at `/mnt/e/Android/sdk/system-images/`
- AVDs: `~/.android/avd/` (Linux) or `/mnt/e/Android/.android/avd/` (Windows)

---

## Constraints / non-negotiables

- **No fake progress.** If a build fails, document it. If a ticket can't be closed without user repro (e.g. P1_ANDROID_CRASH_TRIAGE), say so.
- **No silent test skips.** All new tests must actually pass.
- **No scope creep into Play Console side.** That's Lucas's job.
- **No `cargo` work in this phase** — Rust core is stable for v0.2.1 alpha. If a Rust change is needed, flag it in HANDOFF, don't do it.
- **All commits must include files-modified + test/build status per CLAUDE.md mandatory rules.**

---

## Subagent results (filled in by subagents)

### Subagent A — Nearby Discovery UI Integration
**Status:** Code complete, **build NOT verified** (gradle wrapper bootstrap blocked in WSL).
**Files modified (uncommitted):**
- `android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt` (404→689 lines)
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ContactsViewModel.kt` (672→767 lines)
- `android/app/src/main/res/values/strings.xml` (433→458 lines, +20 keys)
- `android/app/src/test/java/com/scmessenger/android/ui/viewmodels/ContactsViewModelTest.kt` (NEW, 162 lines, 4 tests)
**New public methods on ContactsViewModel:**
- `promoteNearbyPeerToContact(peer: NearbyPeer): Boolean` (line 704)
- `refreshDiscovery()` (line 731)
**Build status:** NOT VERIFIED (gradle wrapper `Could not find or load main class org.gradle.wrapper.GradleWrapperMain` from WSL).
**Test status:** NOT VERIFIED.
**Honest caveats:** The `isRefreshing` state in the new `NearbyDiscoveryTab` doesn't actually flip on rescan — minor polish. The spinner-after-rescan is a missing UX piece.
**Ticket move:** Did NOT move `HANDOFF/todo/P1_ANDROID_LAN_DISCOVERY_REPAIR.md` to done/ — leaves the call to the Overseer after a real build run.

### Subagent B — Production Hardening
**Status:** All 9 subtasks done, **build NOT verified** (pre-existing NDK path bug, not in ownership scope).
**Files modified (uncommitted):**
- `android/app/build.gradle` (signing config + buildTypes.release hardening)
- `android/app/src/main/AndroidManifest.xml` (added `android:networkSecurityConfig`)
- `android/app/src/main/java/com/scmessenger/android/MeshApplication.kt` (crash handler + release tree)
- `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt` (POST_NOTIFICATIONS request)
- `android/app/src/main/res/values/strings.xml` (+4 keys: notification_permission_*)
- `android/app/src/main/res/xml/backup_rules.xml` (added identity_keys.db exclusion)
- `android/app/src/main/res/xml/data_extraction_rules.xml` (added identity_keys.db exclusion)
**Files created:**
- `android/app/proguard-rules.pro` (REPLACED — was 29 lines, now 130+)
- `android/app/src/main/res/xml/network_security_config.xml` (NEW, 33 lines)
**Build status:** NOT RUN. The pre-existing `buildRustAndroid` Gradle task at `app/build.gradle:272` shells out to `cmd /c cargo ndk …` reading NDK path from `local.properties`'s `sdk.dir = E:\Android\sdk` (no `ndk/` subdir there). Actual NDK at `E:\build-tools\android-sdk\ndk\26.1.10909125`. **This blocks all builds including the pre-existing `assembleDebug`.**
**Compliance:** ANDROID_14_FGS (YES — `MeshForegroundService` was already compliant), POST_NOTIFICATIONS (YES), UNCAUGHT_EXCEPTION_HANDLER (YES), PROGUARD_RULES_TESTED (NO).
**Honest caveats:** Did NOT commit intentionally so the Overseer can do a single integration commit.

### Subagent C — Close 8 Open Android Tickets
**Status:** **1 of 8 tickets partially complete. 7 untouched. Out of tool budget.**
**Files modified (uncommitted):**
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (+12 lines around line 3398, `addContact` relay filter)
**Tickets closed:** 0
**Tickets partial:** 1 — `[VALIDATED]_P0_ANDROID_022_Relay_Peer_Contacts_Filter.md`
**Tickets skipped:** 7 — P1_ANDROID_022 BLE stale cache, P1_ANDROID_023 history tests, P1_ANDROID_AUDIT_LOG_VIEWER, P1_ANDROID_MESSAGE_SEARCH_UI, P2_ANDROID_IDENTITY_QR_PRERENDER, P2_ANDROID_IDENTITY_SCROLL_FIX, P1_ANDROID_CRASH_TRIAGE
**Build status:** NOT RUN.
**Test status:** NOT RUN.

---

## ️ BLOCKING ISSUE FOR NEXT SESSION

**The Android Gradle build is currently broken** at `app/build.gradle:272` due to an NDK path mismatch. The script reads `local.properties`'s `sdk.dir = E:\Android\sdk` and tries to find `${sdkDir}/ndk/${android.ndkVersion}` — that path does not exist. The actual NDK is at `E:\build-tools\android-sdk\ndk\26.1.10909125`.

**Both `assembleDebug` AND `assembleRelease` fail at this step.** This is a pre-existing issue, not caused by today's subagent work.

**Next session MUST:**
1. **First** — fix the NDK path. Options:
   a. Edit `local.properties` to set `ndk.dir=E:\build-tools\android-sdk\ndk\26.1.10909125` (note: gradle's `ndk.dir` may need `sdk.dir` to also be updated)
   b. Edit `app/build.gradle:272` to read `ANDROID_NDK_HOME` env var as a fallback
   c. Symlink: `ln -s /mnt/e/build-tools/android-sdk/ndk /mnt/e/Android/sdk/ndk` (cheapest, no code change)
2. **Verify** `./gradlew :app:assembleDebug -x lint --quiet` passes on **clean main** (commit eafa9a1b, BEFORE the subagent edits are applied)
3. **Then** re-apply the uncommitted subagent edits and iterate on any compile errors
4. **Then** continue with the 7 untouched Subagent C tickets

**DO NOT push to Lucas's phone until the build passes locally.**

---

## Final integration results (filled in by next session)
_TBD_
