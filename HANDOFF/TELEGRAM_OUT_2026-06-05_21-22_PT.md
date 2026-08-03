# FOR LUCAS (via Telegram 6014795323) — Decision needed on P0_ANDROID_025

**From:** Overseer Claude session (PID-context: this Claude Code session on E:\, run 21:22 PT)
**To:** Lucas
**Route:** Hermes gateway watches `HANDOFF/` and forwards to Telegram DM 6014795323. Reply from Telegram is dropped back into this folder.
**Date:** 2026-06-05 21:22 PT
**Mode:** Local-only. Zero cloud API used. Quota unchanged (5h=50%, 7d=86.6%).

---

## Status snapshot

| Item | State |
|---|---|
| 3 P0/P1 fixes (P0_024 re-entrancy, P0_024 defense-in-depth, P1_022 cache) | **Uncommitted** in `E:\SCMessenger-build-p0-024\` (branch `fix/p0-android-024-identity`). 24 LoC across 3 files. |
| APK | **Built** (291 MB), **installed** on Pixel 6a (versionCode=7, versionName=0.2.1, firstInstall 21:16:45). |
| Build env verified | NDK r26b at `E:\build-tools\android-sdk\ndk\26.1.10909125` with `-Pandroid.ndkVersion=26.1.10909125` override. Gradle 8.13, JDK 17.0.14, cargo-ndk 4.1.2. |
| Windows SCMessenger CLI | Tested locally. Peer ID `12D3KooW<redacted>`, listening `/ip4/x.x.x.x/tcp/9101`, mDNS enabled, swarm topics subscribed. Working correctly. |
| Hermes | Active, Overseer framework warm. Worker pool cold (no active tasks). |
| Quota | UNTOUCHED this turn. No cloud dispatches. |

---

## New P0 discovered during end-to-end test

**`P0_ANDROID_025 — mDNS "listener already in use" crash on Android<->Windows discovery`** — filed at `HANDOFF/todo/P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH.md`.

**What happened:** Installed the new APK on your Pixel 6a, started the Windows CLI relay on port 9101, the Android app saw the CLI's mDNS broadcast, tried to resolve the service, and crashed with `java.lang.IllegalArgumentException: listener already in use` at `MdnsServiceDiscovery.kt:476`. The crash kills the `ConnectivityThread` which is fatal to the foreground service — full app death.

**Root cause:** `getResolveListener()` returns a singleton. Each `onServiceFound` callback calls `nsdManager.resolveService(info, ..., getResolveListener())`. Second call throws.

**Important:** This bug pre-exists my 3 fixes. It was latent because v0.2.3 (the previous install on your phone) had never actually discovered a peer — the new test with the Windows CLI finally exercised the path. **My 3 fixes are correct and ship-clean independently.** P0_025 is a separate bug, needs a separate worktree, needs a separate commit.

**Fix scope:** ~15 LoC, all in `MdnsServiceDiscovery.kt`. Either per-service listener (cheap) or pending-resolve set with onComplete cleanup (canonical). No spec work, no architecture changes.

---

## What I need from you — single decision

**Do I create a second worktree and fix P0_025 now (locally, no API), or do we ship the 3 fixes first and queue P0_025 for the next dispatch?**

| Option | Time | Risk | Outcome |
|---|---|---|---|
| **A. Ship 3 fixes now, defer P0_025** | You commit `fix/p0-android-024-identity`, push, install, retest onboarding. P0_025 stays in `HANDOFF/todo/` and gets dispatched as a normal handoff next session. | Onboarding regresses if any peer is on the LAN at install time (your mDNS is now active). | Hotfix ships tonight, P0_025 lands in next batch. |
| **B. Fix P0_025 in the same window, then ship together** | I create `E:\SCMessenger-build-p0-025\` worktree off origin/main, write 15 LoC fix, build, install, retest end-to-end. ~25-30 min. Single commit covers both. | More code in one commit, but they're independent fixes in different files. | Both ship together, no half-state on your phone. |
| **C. Defer everything, wait for next session** | Nothing changes. Build and worktree sit uncommitted. | You have v0.2.3 with the identity bug on your phone. | P0_024 stays unfixed. |

Reply with **A**, **B**, or **C**. If you don't reply, I sit idle and monitor this folder.

---

## Self-persistence plan (per your last message)

I will:
1. Set a folder-monitor on `HANDOFF/` that wakes me when a new file appears (your Telegram reply will land here).
2. Idle (not exit) until the monitor fires or you say something in this chat.
3. When the monitor fires, I read the file, parse the answer, and execute.
4. If the file is ambiguous, I ask one crisp follow-up here in this same Claude chat, NOT in Telegram — that path is for blocking decisions, not clarifying questions.

If you want me to do something other than the 3 options, write a free-form `REPLY_*.md` into `HANDOFF/` and I'll parse it.

---

## Quota posture reaffirmed

This turn used **zero** Ollama Cloud / Anthropic API calls. All work was local (cargo, gradle, adb, scmessenger-cli.exe, kotlinc in-process fallback). Quota is at the same level as when you messaged me. Future work will remain local-only per your "pivot NOW to only local models" directive, unless you explicitly say otherwise.

---

*Overseer Claude — `HANDOFF/STATE/2026-06-05_ANDROID_P0_024_P1_022_BUILD_VERIFIED.md` has the full build + test log for reference. The worktree state is preserved exactly as you left it in the orchestration index. This file is the Telegram handoff; do not commit it to the repo.*
