# TELEGRAM — P0_024 shipped, P0_025 fix committed, retest BLOCKED by phone offline

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-06-06 01:00 PT
**Phase 1+2 status:** Both P0 fixes committed locally; no push (your gate).

## Phase 1 (P0_024) — DONE
- Worktree pointer repaired (broken `.git` worktree renamed aside; `git worktree add` recreated cleanly)
- 3 modifications + 2 untracked test files (BleScannerTest, MeshRepositoryHistoryTest) restored from /tmp/p0-024-backup
- **Commit 7c362c63 on `fix/p0-android-024-identity`**: re-entrancy guard on `createIdentity()`, OnboardingScreen `&& !isCreating` defense, BleScanner `clearPeerCache()`, plus the 2 gemini regression tests

## Phase 2 (P0_025) — FIX COMMITTED, RETEST BLOCKED
- **Commit e84f4fc3 on `fix/p0-android-025-mdns-listener-collision`**: per-call `ResolveListener` tracked in `ConcurrentHashMap<String, ResolveListener>`. +38/-15 LoC, all in `MdnsServiceDiscovery.kt`. No more "listener already in use" crash possible.
- Build green: 291MB APK at `E:\SCMessenger-build-p0-025\android\app\build\outputs\apk\debug\app-debug.apk`. Used NDK 26b override.
- **Live mDNS retest BLOCKED**: Pixel 6a is offline. adb sees no devices; mDNS `_adb-tls-connect._tcp` returns empty; ARP table has no Pixel 6a. Phone is WiFi-asleep or USB-unplugged (or both). Windows CLI relay from prior session is also gone (svchost is now on 9001/9002).
- I have NOT pushed. The fix is on the local branch `fix/p0-android-025-mdns-listener-collision` ready for your review.

## To unblock the retest (when you can)
1. Plug Pixel 6a into USB (and/or wake WiFi)
2. Tap "Allow" on the RSA fingerprint prompt
3. `adb -s adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp install -r E:\SCMessenger-build-p0-025\android\app\build\outputs\apk\debug\app-debug.apk`
4. Start the Windows CLI: `E:\SCMessenger-Github-Repo\SCMessenger\target\debug\scmessenger-cli.exe relay --listen /ip4/0.0.0.0/tcp/9101 --http-port 9102`
5. Launch the app; confirm: no FATAL EXCEPTION, Windows CLI's `discovery peers` shows Android peer-id within 60s

## What's next (PHASE 3)
- Committing the integration branch changes now: `HANDOFF/CLAUDE_CODE_PROTOCOL.md` + 4 file changes (build.gradle, BleScanner.kt, ContactsViewModelTest.kt x2) on `integration/v0.2.2-pre-android-push-2026-06-05`. Will not push.

## Question (only if you have time)
Should I:
- A) Wait for you to retest P0_025 in person before doing anything else
- B) Proceed to PHASE 3 + PHASE 4 (worker pool warmup) while you retest
- C) Other

Reply via `HANDOFF/REPLY_*.md` whenever you're ready. I'm idling in the HANDOFF/ monitor.
