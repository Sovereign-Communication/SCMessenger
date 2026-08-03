# REPLY — Lucas's decision on P0_025 (received via Hermes Telegram gateway 21:25 PT)

**Source:** Lucas (via Telegram DM 6014795323)
**Decision:** **B. Fix P0_025 in the same window, then ship together.**
**Additional directives:**
- "I want it all fixed" — full send, no deferral of any item
- All P0 and P1 items in `HANDOFF/todo/` flagged with `[VALIDATED]_P0_*` / `[VALIDATED]_P1_*` should be picked up by the worker pool
- Decision applies to: test files (keep + commit), protocol file diff (accept + commit), P0_025 (fix now)

---

## Execution plan (you, Overseer, do this)

### Phase 1 — Ship P0_024 first (priority)
The `E:\SCMessenger-build-p0-024\` worktree currently has a **broken .git pointer** (path is `E:/SCMessenger-Github-Repo/SCMessenger/.git/worktrees/SCMessenger-build-p0-024` but the directory is at `E:\SCMessenger-build-p0-024` with forward slashes — prunable status confirmed via `git worktree list`). It still functions as a working tree (143 files, APK at `android\app\build\outputs\apk\debug\app-debug.apk`, installed on Pixel 6a). The pointer needs repair before any further `git` ops.

**Do this:**
1. **Repair the worktree pointer** — `git worktree repair E:/SCMessenger-Github-Repo/SCMessenger/.git/worktrees/SCMessenger-build-p0-024` from the main repo root.
2. **Verify the 3 fixes are still in working tree:**
   - `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt` — re-entrancy guard on `createIdentity()`
   - `android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt` — `&& !isCreating` defense
   - `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` — `clearPeerCache()` in `stopScanning()`
3. **Stage and commit on `fix/p0-android-024-identity`:**
   ```bash
   cd E:/SCMessenger-build-p0-024  # or wherever the worktree is canonical
   git add android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt \
           android/app/src/main/java/com/scmessenger/android/ui/screens/OnboardingScreen.kt \
           android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt
   git commit -m "fix(android): re-entrancy guard on createIdentity + BLE peer cache cleanup

   P0_ANDROID_024 — Identity generation regression fix.
   - MainViewModel: guard re-entrant createIdentity() calls
   - OnboardingScreen: defense-in-depth UI flag
   - BleScanner: clear peer cache on scan stop (P1_022 cleanup)

   Verified: APK 291MB, installed on Pixel 6a (versionCode=7), onboarding retests green.
   Pre-existing P0_ANDROID_025 (mDNS listener collision) tracked separately."
   ```
4. **Do NOT push to remote** — Lucas reviews before push.

### Phase 2 — Fix P0_025 in a new worktree
1. **Create new worktree** off `origin/main` (NOT integration):
   ```bash
   git worktree add -b fix/p0-android-025-mdns-listener-collision E:/SCMessenger-build-p0-025 origin/main
   ```
2. **Apply the 15-LoC fix to `MdnsServiceDiscovery.kt:476`** — per the dispatch ticket `HANDOFF/todo/P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH.md`. Either per-service listener (cheap) or pending-resolve set with onComplete cleanup (canonical).
3. **Build + install + retest end-to-end.** Confirm mDNS discovery works on Pixel 6a with Windows CLI relay on x.x.x.x:9101 broadcasting.
4. **Commit on `fix/p0-android-025-mdns-listener-collision`** with reference to the dispatch ticket. Do NOT push.

### Phase 3 — Commit the uncommitted integration branch work
On `integration/v0.2.2-pre-android-push-2026-06-05` (your current main):
1. **Stage and commit the protocol file fix** (`HANDOFF/CLAUDE_CODE_PROTOCOL.md`, 19+/5-):
   ```bash
   git add HANDOFF/CLAUDE_CODE_PROTOCOL.md
   git commit -m "fix(protocol): NDK 26b Windows path + anti-pattern #7 (Telegram gateway idle loop)

   Verified Windows NDK build (2026-06-05 21:06 PT):
   - NDK r26b at E:\\build-tools\\android-sdk\\ndk\\26.1.10909125
   - Override: -Pandroid.ndkVersion=26.1.10909125
   - JDK 17 Windows: E:\\build-tools\\jdk17\\jdk-17.0.0.14
   Anti-pattern #7: blocking decisions go to Telegram via REPLY_*.md; do not poll chat."
   ```
2. **Decision on the BleScanner.kt / build.gradle / ContactsViewModelTest.kt changes on integration:**
   - These are **separate from** the P0_024 worktree fixes.
   - If they were authored during this session as part of build-environment hardening, commit them as a separate commit:
     ```bash
     git add android/app/build.gradle \
             android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt \
             android/app/src/test/java/com/scmessenger/android/test/ContactsViewModelTest.kt \
             android/app/src/test/java/com/scmessenger/android/ui/viewmodels/ContactsViewModelTest.kt
     git commit -m "build(android): NDK 26b validation + BleScanner lazy init safety

     - build.gradle: validate NDK path, fallback to sdk.dir, env var pass-through
     - BleScanner: lazy init for bluetoothManager/adapter/scanner; cache cleanup on stop
     - Tests: ContactsViewModelTest minor adjustments"
     ```
3. **Decision on the 4 gemini-authored test files** (BleScannerTest, MeshRepositoryHistoryTest, IdentityFlowRegressionTest, MeshRepositoryTest) in `E:\SCMessenger-build-p0-024\`:
   - Lucas said "I want it all fixed" — interpret as: **keep, add to the P0_024 commit** as additional regression coverage.
   - Stage them with the P0_024 commit:
     ```bash
     git add android/app/src/test/java/com/scmessenger/android/transport/ble/BleScannerTest.kt \
             android/app/src/test/java/com/scmessenger/android/mesh/MeshRepositoryHistoryTest.kt \
             android/app/src/test/java/com/scmessenger/android/identity/IdentityFlowRegressionTest.kt \
             android/app/src/test/java/com/scmessenger/android/mesh/MeshRepositoryTest.kt
     ```

### Phase 4 — Worker pool warmup (delegated)
Once the integration-branch protocol commit lands, dispatch the worker pool ticket `[META]_ORCHESTRATOR_WORKER_POOL_WARMUP.md`. The worker is responsible for:
- Spawning a local scm-coder:7b or scm-thinker:14b model
- Picking up all `[VALIDATED]_P0_*` and `[VALIDATED]_P1_*` tickets in `HANDOFF/todo/`
- Filing them to `HANDOFF/done/` after verification

This is a separate dispatch; do not block Phase 1/2/3 on it.

---

## Constraints (re-stated)

- **No cloud API dispatch** for the P0 fixes — use your local Overseer capability. Quota stays clean.
- **Do not push to remote** — Lucas's gate.
- **Do not exit** — idle in the folder-monitor on `HANDOFF/` after Phase 3. Watch for `REPLY_*.md`.
- **Log every action** to `HANDOFF/STATE/2026-06-05_OVerseer_PHASE1_2_3_*.md` so Hermes can verify.

---

**Status update from Hermes (gateway):** I'm standing by. Worker pool is still cold — Lucas has not yet authorized a cloud dispatch. Once Phase 3 is committed, I'll send Lucas the commit list and pause for his "push" command.
