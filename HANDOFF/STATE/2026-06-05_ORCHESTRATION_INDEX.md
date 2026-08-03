# ORCHESTRATION INDEX — Android/Windows/Ubuntu Production-Ready Push
**Date:** 2026-06-05 13:25 PT
**Author:** Hermes Overseer session
**Audience:** Claude Code (next session) + all subagents dispatched from this index

---

## Role & Protocol

All Claude Code sessions on this workspace must read `HANDOFF/CLAUDE_CODE_PROTOCOL.md` first. It is the Overseer role anchor. The orchestration workflow is `HANDOFF/todo/` → swarm dispatch → `HANDOFF/done/` via `git mv`. No new frameworks.

---

## TL;DR

1. **Android v0.2.3 is shipped and running on the user's Pixel 6a** (versionCode 7, installed 14:05 PT). v0.2.3 includes the UniFFI binding race fix from this session.
2. ** P0 NEW: Identity-generation regression reported on v0.2.3 onboarding** (14:20 PT, 2026-06-05). Logcat shows 8–10 `initializeIdentity` calls in 1 second — multi-component contention. Ticket: `HANDOFF/todo/P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md`. **PROMOTE TO BATCH 1 — this is now highest priority.**
3. **7 Android tickets are the next delegation batch** (Subagent C ran out of tool budget before finishing them). See Batch 1.
4. **2 P0 CLI tickets are blockers for cross-OS triangulation** (P0_CLI_023 key collision, P0_CLI_027 Drift dormancy). See Batch 2.
4. **Build chain is now reproducible from WSL** — full env below.
5. **Dynamic-port migration is researched and ready to execute** — Phase 0 first.

---

## DONE (do not redo)

### Android v0.2.2 — commit aaf603f9, on Pixel 6a
**Verified in logcat at 13:18 PT:**
- PID 7643, app alive
- 5 runtime permissions granted (BLE_SCAN, BLE_ADVERTISE, BLE_CONNECT, NEARBY_WIFI_DEVICES, POST_NOTIFICATIONS)
- BLE GATT server started, mDNS advertiser on port 9001, 3 pubsub topics subscribed
- No FATAL EXCEPTION, no ANR in this launch

**Code shipped in this push:**

| Layer | Files | What it does |
|---|---|---|
| Nearby Discovery UI | `AddContactScreen.kt` (404→689) | `NearbyDiscoveryTab()` reads `viewModel.nearbyPeers`; 3 states; per-card Add button; Rescan |
| Nearby Discovery VM | `ContactsViewModel.kt` (672→767) | Added `promoteNearbyPeerToContact()` + `refreshDiscovery()` |
| Nearby Discovery tests | `ContactsViewModelTest.kt` NEW 162 lines | 4 test cases |
| Production hardening | `build.gradle` (signing) | env-driven release signing, debug fallback + warning |
| Production hardening | `proguard-rules.pro` 130+ lines | uniffi, Hilt, Timber, Compose, JNA keeps |
| Production hardening | `network_security_config.xml` NEW | cleartext blocked except 127.0.0.1, 10.0.2.2, localhost |
| Production hardening | `backup_rules.xml` + `data_extraction_rules.xml` | exclude identity_keys.db |
| Production hardening | `AndroidManifest.xml` | networkSecurityConfig attribute |
| Production hardening | `MeshApplication.kt` | UncaughtExceptionHandler, release Timber tree |
| Production hardening | `MainActivity.kt` | POST_NOTIFICATIONS runtime request with rationale |
| Production hardening | `strings.xml` | 20 add_contact_nearby_* + 4 notification_permission_* keys |
| Relay filter | `MeshRepository.kt` | `addContact()` filters relay peers (P0_ANDROID_022) |

**Env fixes (not features):**
- `scripts/OllamaQuotaScraper.ps1` baseDir → `E:\SCMessenger-Github-Repo\SCMessenger`
- `android/gradle.properties` for WSL JDK 17 toolchain
- Linux NDK r26b at `/home/scemessenger/android-sdk/ndk/26.1.10909125/`
- WSL source at `/home/scemessenger/scmessenger-build/`
- APK: `tmp/APP-v0.2.2-debug.apk` (291MB)

### Dynamic-port research
`HANDOFF/research/2026-06-05_DYNAMIC_PORT_DISCOVERY_RESEARCH.md` (439 lines)
- 4-phase plan: config → ephemeral → negotiation → liveness
- 3 code sketches (ephemeral bind, UDP self-NAT, relay hole-punch)
- All citations verified

---

## NEXT (delegation queue)

### Batch 1: 7 remaining Android tickets (HIGHEST PRIORITY)

In `HANDOFF/todo/`. Split into 2 subagents:

**Subagent D (small, related to existing files):**
1. `[VALIDATED]_P1_ANDROID_022_BLE_Stale_Cache_Cleanup.md` — wire `BleScanner.onTransportPause()` to app lifecycle
2. `[VALIDATED]_P1_ANDROID_023_History_Persistence_Regression_Test.md` — new test file
3. `P2_ANDROID_IDENTITY_QR_PRERENDER.md` — pre-render QR in LaunchedEffect
4. `P2_ANDROID_IDENTITY_SCROLL_FIX.md` — wrap content in verticalScroll

**Subagent E (new screens):**
5. `[VALIDATED]_P1_ANDROID_AUDIT_LOG_VIEWER_001.md` — AuditLogScreen + ViewModel + nav route
6. `[VALIDATED]_P1_ANDROID_MESSAGE_SEARCH_UI_001.md` — search bar in Chats/Contacts
7. `P1_ANDROID_CRASH_TRIAGE.md` — CrashReportScreen + diagnostics export

**Each MUST:** build successfully, move ticket to `done/`, report status.

### Batch 2: 2 P0 CLI tickets (cross-OS blockers)

8. `[VALIDATED]_P0_CLI_023_ContactManager_Shared_Backend_Key_Collision.md` — no unique constraint
9. `[VALIDATED]_P0_CLI_027_Drift_Protocol_Still_Dormant_At_0_2_1.md` — biggest v0.2.1 integrity question

**For #9, investigate first:** `grep -r "drift::" core/src/`, check DriftFrame/DriftEnvelope/SyncSession call sites from swarm.rs send path. If imported but never called, then yes dormant.

**Dispatch as single subagent F.**

### Batch 3: 10 P1 CLI tickets (cross-OS infra)

In `HANDOFF/todo/`:
- P1_CLI_024 (mDNS TXT), 025 (identify spam), 026 (external addr), 028 (port stale), 029 (binary lock), 030 (transport hardcode), 031 (BLE daemon), 032 (control API GET /contacts), 033 (Windows E2E smoke)
- P1_WASM_003 (CLI local authority E2E)

Split into 3 subagents by domain (G/H/I).

**Cross-OS triangulation gate (after Batch 3):**
1. Windows host: `cli relay --listen 9101 --http-port 9002`
2. WSL Ubuntu: `cli relay --listen 9103 --http-port 9004`
3. Android discovers both via mDNS, sends message to each
4. Verify circuit: Android → Windows CLI → WSL CLI → back to Android
5. Capture logs, attach to this HANDOFF file

### Batch 4: Dynamic-port migration Phase 0 (config plumbing)

Write `HANDOFF/todo/[VALIDATED]_P0_DISCOVERY_PORT_RANGE_CONFIG.md` from the research doc's Section 5. Then dispatch as subagent.

Smallest possible diff. Fully backwards compatible. Unblocks all subsequent phases.

### Batch 5: P0 release paperwork (skip if user doesn't ask)

- `[VALIDATED]_P0_RELEASE_001_v0.2.1_Complete_Notes.md` → write `RELEASE_NOTES_v0.2.1.md`
- `[VALIDATED]_P0_DOC_002_Promotion_Roadmap_v0.3.md` → write v0.3 roadmap

### Batch 6: iOS / KMP (SKIP — user has not asked for iOS)

`P1_IOS_001/002/003`, `TASK_KMP_*` — defer.

---

## Build Environment (CRITICAL)

```bash
# Prereqs (already installed):
# - Linux NDK r26b at /home/scemessenger/android-sdk/ndk/26.1.10909125/
# - JDK 17 Temurin at /home/scemessenger/.local/jdk/jdk-17.0.12-7
# - Android SDK at /mnt/e/Android/sdk (symlinks added for .exe → no-suffix)
# - Source at /home/scemessenger/scmessenger-build/ (NOT /mnt/e/ — 9P I/O errors)

# NDK wrapper stubs MUST be replaced (the NDK ships 3-200 byte files that fail
# when invoked via absolute path):
# - clang → /home/scemessenger/android-sdk/ndk/26.1.10909125/toolchains/llvm/prebuilt/linux-x86_64/bin/clang-17
# - clang++ → clang-17 --driver-mode=g++
# - ld.lld → lld -flavor gnu

# Build:
cd /home/scemessenger/scmessenger-build/android
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/home/scemessenger/.cargo-target
export GRADLE_USER_HOME=/home/scemessenger/.gradle
export ANDROID_HOME=/mnt/e/Android/sdk
export ANDROID_NDK_HOME=/home/scemessenger/android-sdk/ndk/26.1.10909125
export PATH="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"
export JAVA_HOME=/home/scemessenger/.local/jdk/jdk-17.0.12-7
export PATH="$JAVA_HOME/bin:$PATH"

# CRITICAL: build.gradle copies Rust .so from ../../target/${target}/ but
# CARGO_TARGET_DIR overrides. Either symlink or copy manually:
for abi in arm64-v8a armeabi-v7a x86_64; do
  case $abi in
    arm64-v8a) triple=aarch64-linux-android ;;
    armeabi-v7a) triple=armv7-linux-androideabi ;;
    x86_64) triple=x86_64-linux-android ;;
  esac
  cp /home/scemessenger/.cargo-target/$triple/debug/libscmessenger_mobile.so \
     /home/scemessenger/scmessenger-build/core/target/android-libs/$abi/
done

# UniFFI gen_kotlin needs host debug:
cp /home/scemessenger/.cargo-target/debug/libscmessenger_mobile.so \
   /home/scemessenger/scmessenger-build/target/debug/
cp /home/scemessenger/.cargo-target/debug/gen_kotlin \
   /home/scemessenger/scmessenger-build/target/debug/

# Then:
./gradlew :app:assembleDebug -x lint --no-daemon
# → android/app/build/outputs/apk/debug/app-debug.apk (~291MB debug)
```

**Install:**
```bash
/mnt/e/Android/sdk/platform-tools/adb.exe -s adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp install -r \
  /home/scemessenger/scmessenger-build/android/app/build/outputs/apk/debug/app-debug.apk
```

**Verify:**
```bash
adb.exe -s adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp shell am start -n com.scmessenger.android/.ui.MainActivity
adb.exe -s adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp shell pidof com.scmessenger.android
adb.exe -s adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp logcat -d 2>&1 | grep -E "Mesh:|BleGatt|MdnsAdvertiser" | tail -20
```

---

## Quota (fresh 2026-06-05 13:18 PT)

```
Session: 20.8% used, resets in ~37 min → still Tier 1
Weekly:  58% used, resets in 2 days
Status:  ok
```

Heavy-lift tier active. Full budget for the next 30 min.

---

## Constraints (non-negotiable for subagents)

1. **No fake PASS reports.** If build fails, document it.
2. **No test skips.** New tests must actually pass.
3. **No scope creep.** Each subagent owns a specific file list.
4. **Build before commit.** Android subagents MUST run `./gradlew :app:assembleDebug -x lint --quiet` successfully.
5. **Move ticket on commit.** `git mv` from `todo/` to `done/`. Breaking this breaks orchestration.
6. **No Rust changes** unless explicitly tasked.
7. **Use the Linux NDK path.** Windows NDK at `/mnt/e/build-tools/...` is .exe only.

---

## Open questions for the user (escalate, don't guess)

1. **Real keystore** — Subagent B added env-driven signing. User needs to provide `SCMESSENGER_KEYSTORE_PATH` before Play Store upload.
2. **Cross-OS triangulation** — User said "test against Windows and Ubuntu" but hasn't started the Windows CLI daemon yet. WSL→Windows x.x.x.x:9101 reachable at <1ms (per memory).
3. **7 remaining Subagent C tickets** — auto-dispatch or reprioritize?
4. **P0 CLI Drift dormancy** — investigate first or just fix?

---

## File map

```
HANDOFF/
├── todo/                                # 45 open tickets (CLAUDE CODE QUEUE)
├── done/                                # 250+ closed tickets
├── STATE/                               # Long-running state notes
│   ├── 2026-06-05_NEARBY_DISCOVERY_PRODUCTION_PUSH.md
│   ├── 2026-06-05_ORCHESTRATION_INDEX.md  # THIS FILE
│   └── agent1-4_complete.md, phase1_complete.md
└── research/
    └── 2026-06-05_DYNAMIC_PORT_DISCOVERY_RESEARCH.md

core/src/transport/                       # Rust CLI/Core changes
android/app/src/main/java/com/scmessenger/android/
  ├── transport/                         # BleScanner, TransportManager, SubnetProbe
  ├── ui/contacts/AddContactScreen.kt    # Nearby Discovery
  ├── ui/audit/                          # Audit log viewer (NEW)
  ├── ui/diagnostics/                    # Crash report (NEW)
  └── data/MeshRepository.kt             # Contact store, audit log API
```

---

*Co-located per the agent state-machine pattern. Last updated 2026-06-05 13:25 PT.*
