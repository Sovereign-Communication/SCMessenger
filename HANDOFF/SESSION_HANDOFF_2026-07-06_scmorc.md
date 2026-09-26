# /scmorc Session Handoff -- 2026-07-06

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Halted reason:** Claude API quota hit ~4% remaining (HARDLOCK tier per the
quota governor in `.claude/commands/scmorc.md`). This session stopped
dispatching and committed all verified work. Resume next session once the
5-hour window resets.

## What landed this session (12 commits, `2e9eab05`..`13c47c24`)

1. **P1-CLI-TRANSPORT-NEGOTIATION-FALSE-POSITIVE** (`e9a6763c`) -- root-caused
   the long-standing "Failed to negotiate transport protocol(s)" CLI warning
   as a false positive (Android's own `SubnetProbe` LAN-discovery port-scan
   being misread as a failed peer negotiation). Log-level fix, audited CLEAR.
   Ticket moved to `done/`.
2. **P1-ANDROID-LISTENER-EVENT-LOGGING** (`48f79f0e`) -- added
   `SwarmEvent::ListenerError`/`ListenerClosed` logging to
   `core/src/transport/swarm.rs` (previously silently dropped), needed to
   diagnose the real remaining blocker below. Audited CLEAR (one LOW
   informational finding, deferred as non-blocking follow-up).
3. **Android listener-reachability static analysis** (`c6aeb043`) --
   dispatched to gemini-3.1-pro via `agy`, audited by Opus. Manifest/Kotlin
   config confirmed correct (no permission/foreground-service-type gap).
   Opus caught two real errors in the first draft's live-retest checklist
   and found two better leads the first pass missed entirely. See "Next
   priority" below -- this is the most important open thread.
4. **P2-IOS-Silent-TryQuestion-Swallows-Actions** (`7a43b6fe`) -- fixed two
   silent `try?` swallows (JoinMeshView, MainTabView) via agy/gemini-3.5-flash.
5. **P2-IOS-ContactManagerFix-TryBang-FFI-Crash-Risk** (`5d213546`) --
   documented (not code-changed) that `count()`/`flush()`'s `try!` is safe;
   verified genuinely infallible Rust signatures via agy/gemini-3.5-flash.
6. **P1-GEMINI-FLASH-021/022/023** (`6477dac8`, `3461f19c`, `0fd5767d`) --
   three mechanical panic fixes (CLI identity_id `.expect()`, WASM
   notification-permission `.unwrap()`, core history retention corrupt-record
   `.expect()`) via agy/gemini-3.5-flash. All build/test-verified clean.
7. **CORE-SWEEP-02** (`a8b5a114`) -- `IronCore::contacts_manager()`/
   `history_manager()` no longer panic on double-fallback failure (now return
   `Result`). Implemented by gemini-3.1-pro via agy; **the orchestrator had
   to apply a 2-line surgical fixup** (two internal call sites in
   `iron_core.rs` that agy's cross-directory grep missed) before it compiled
   -- worth remembering that a headless worker's "zero external callers"
   claim doesn't cover same-file internal callers.
8. **P0-ANDROID-ANR-BatteryReceiver** (`ab4998c4`) -- fixed a real, reproduced
   ANR (BroadcastReceiver doing synchronous FFI on the main thread) via a
   native sonnet worker. Compile-verified only -- see the test-infrastructure
   discovery below, unrelated to this fix's correctness.
9. Two escalation/hygiene tickets filed: `P2_CORE_Bootstrap_Test_Depends_On_Unset_Env_Var.md`
   and `P1_ANDROID_Unit_Tests_Force_Disabled_Since_2026-06-06.md` (see below).
10. `scripts/disk_hygiene.sh` added (read-only report, not wired into the
    session-start hook -- that would need explicit operator sign-off since it
    changes agent startup behavior).
11. `.claude/commands/scmorc.md` updated with two new hard-constraint
    safeguards (see "Process changes" below).

## MOST IMPORTANT DISCOVERY: Android unit tests have been a no-op since 2026-06-06

`android/app/build.gradle:147-169` (commit `23174061`, a month before this
session) force-disables all `Test`-type Gradle tasks and zeroes the
`test`/`androidTest` sourceSets, with test dependencies commented out in the
same commit. **Every "RoleNavigationPolicyTest passes" report since then --
including this repo's own documented mandatory pre-merge rule in
`.claude/rules/android.md` -- has been a structurally-guaranteed false
positive** (`NO-SOURCE`/`SKIPPED`/`BUILD SUCCESSFUL` regardless of test
content). Reproduced with `--no-configuration-cache --rerun-tasks` to rule
out a caching artifact -- it's real, deliberate config.

Filed as `HANDOFF/todo/P1_ANDROID_Unit_Tests_Force_Disabled_Since_2026-06-06.md`,
explicitly flagged as an **escalation** (architecture-direction decision, not
a mechanical fix) per CLAUDE.md's escalation rule. **This needs the
operator's decision** on whether to investigate-and-restore or
formally-accept-and-redocument. Next session should surface this decision
point again if not yet addressed -- do not silently re-enable it.

## Next priority: Android listener-reachability (real remaining Phase 1 blocker)

`HANDOFF/todo/P1_ANDROID_Inbound_Libp2p_Listener_Not_Externally_Reachable.md`
is now the actual gating bug for Android<->Windows messaging (the negotiation
bug that looked like the blocker was a false positive -- see above). The
corrected, Opus-audited live-retest checklist is in that ticket. **Top lead
to check first:** `AndroidPlatformBridge.onEnteringBackground()` ->
`meshRepository.pauseMeshService()` -> `meshService.pause()`
(`MeshRepository.kt:3394`) may be tearing down the inbound listener when the
phone's screen is off/backgrounded -- retest with the app foregrounded and
screen on as the very first control, before chasing any OS-level Doze/
firewall theory. Second lead: an unawaited async race in
`initializeAndStartSwarm()` (`MeshRepository.kt:2163-2199`) between
`listen_on()` binding and the service being marked `RUNNING`. This needs live
device access (adb + the Pixel 6a) -- not delegable to a headless worker.

## Process changes made this session (now in `.claude/commands/scmorc.md`)

1. **HOST BUILD SERIALIZATION**: a Gradle target that looks Rust-free
   (`:app:compileDebugKotlin`) silently pulled in a full `cargo ndk` cross-compile
   as an upstream task dependency, running concurrently with an orchestrator-run
   `cargo test` -- caught before any corruption, but confirmed the risk is
   real. New default: workers implement code only, never run
   `cargo build/check/test`/`./gradlew` themselves -- the orchestrator is now
   the single writer for all build verification. Before backgrounding any
   build command, check `tasklist` for `cargo.exe`/`java.exe` first.
2. **ORCHESTRATOR PROCESS OWNERSHIP**: maintain a live inventory of every
   in-flight process (Claude workers, `agy.exe`/Gemini dispatches, builds),
   reconciled against actual `tasklist`/`Get-CimInstance Win32_Process` state,
   not just what you remember dispatching.

## agy / Gemini dispatch pattern (new capability this session, not previously documented)

`C:\Users\SCM\AppData\Local\agy\bin\agy.exe` runs Gemini models headlessly:
```bash
agy.exe -p "<prompt>" --model "Gemini 3.5 Flash (High)" --dangerously-skip-permissions --print-timeout 30m
```
Available models (`agy.exe models`): Gemini 3.5 Flash (Low/Medium/High),
Gemini 3.1 Pro (Low/High), Claude Sonnet 4.6, Claude Opus 4.6, GPT-OSS 120B.
No scoped `--allowedTools` equivalent exists for agy -- operator explicitly
approved `--dangerously-skip-permissions` for well-scoped, non-crypto/transport
tasks this session. Default print-timeout (5m) was too short for a real
task including its own build/test attempt; fixed by (a) using 30m timeouts
and (b) instructing agy workers to never build/test themselves (matches the
HOST BUILD SERIALIZATION change above). Single-slot concurrency for agy per
explicit operator instruction (one dispatch at a time, wait for completion).
Route: mechanical/single-file fixes -> Gemini 3.5 Flash High; tasks needing
real design judgment (blast-radius analysis, architecture questions) ->
Gemini 3.1 Pro High, optionally with a quick Opus audit pass afterward for
higher-stakes findings (used successfully for the listener-reachability
analysis) -- this pattern saves Claude quota effectively and should be the
default going forward whenever Claude quota is constrained.

## Backlog state

`HANDOFF/todo/` still has: PQC_02-14, TASK_KMP_* (all four), P1-11/12/13/14/17
(adaptive port selection + WiFi Direct, queued behind the listener-reachability
fix per the lane rules), P1_ANDROID_LAN_Discovery_Not_Feeding_Bootstrap_Peer_Count,
P1_ANDROID_mDNS_Self_Loopback_Discovery, P1_ANDROID_QR_Code_Identity_Export_Extremely_Slow,
P2_ANDROID_BLE_MAC_Rotation_Breaks_Session_Continuity, ANDROID_SWEEP_01 (hardcoded
strings, overlaps with P2_ANDROID_HARDCODED_STRINGS_CONTACTS_SETTINGS.md --
worth de-duplicating before dispatching either), CORE_SWEEP_04, and the two
new escalation tickets above. `REMAINING_WORK_TRACKING.md` was not updated
this session -- worth a quick pass next session to reflect the false-positive
resolution and the new blocker.

## Disk state

Freed ~50G this session (`cargo clean` + `gradlew clean` + an orphaned Gradle
JDK tarball): 32G -> 81G free before the Android native rebuild consumed some
of it back. Run `bash scripts/disk_hygiene.sh` at the start of next session to
check current state.
