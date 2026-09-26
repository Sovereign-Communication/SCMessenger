# Overseer: Unblock, Test, Build, Verify, Unify  Single Sweep Handoff

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**From:** Hermes (Telegram orchestrator)
**To:** Claude Code (Overseer)  pick up and execute
**Date:** 2026-06-08 (Monday)
**Session:** Continuation of `IN_PROGRESS/IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07.md`
**Authority:** Full GO from Lucas  "I want it all fixed"
**Status:** NEW  claim, execute, verify, commit

---

## 0. Mission

Four outcomes, all in one sweep, in this order:

1. **Unblock** every blocked item in the kanban + HANDOFF state machines
2. **Unify** the kanban  HANDOFF/todo/ state machines so the two views agree
3. **Build** the full workspace end-to-end (Rust workspace + Android + WASM as available)
4. **Verify** with cargo test + gradle test + a real CLI smoke test
5. **Commit** locally (no push  Lucas's gate still holds)

**Stop conditions** (escalate to Lucas, do not auto-retry):
- Crypto/transport/concurrency changes that need a deliberate design call
- Multi-crate build failure (3+ crates broken simultaneously)
- Free disk on C: < 2 GB
- Any need to revert a previous commit

**Quota state at dispatch:** 5h=33.8%, 7d=6.0%  TIER 2 (EXECUTE). 120 min to 5h reset. 3 slots free (per Lucas: "Agy does not count as a slot, and local llm doesn't count as a slot").

---

## 1. The State of the World (Audit Before Action)

The user's "all the blocked (35)" reads from the **autonomous triage bucket** produced by `scmessenger-triage.py` (slot 2 bridge). There are **two parallel state machines** that disagree:

### Machine A  Hermes Kanban Board (`hermes kanban list`)
- **8 blocked** tasks, all `(unassigned)`, all aging 11+ days:
  - `t_570a8f4e` Setup: Android build toolchain  child of nothing
  - `t_2fcfa616` Build Rust core for Android  child of `t_570a8f4e`
  - `t_3118d436` Implement missing UI screens
  - `t_93efd990` BLE Transport
  - `t_0b4a8488` WiFi Direct Transport
  - `t_f3314103` Notifications + FCM
  - `t_d211204b` Build APK
  - `t_73f0ffdc` Test on device
- **0 ready, 0 in_progress, 1 done** (`t_b521e8a9` "Audit repo: real gaps vs claims")
- User guidance on the board: *"Get all the pieces assembled and keep going until we have a complete android app."*

### Machine B  HANDOFF/todo/ (file-based state, 51 files)
- **40 `[VALIDATED]_*.md` tickets** (autonomous triage scans these)
  - [WARN] **35 blocked** (per `scmessenger-triage.py`  `HANDOFF/todo/` scan on 2026-06-07 22:43 PT)
  - [FAIL] **5 missing-deliverable** (ticket exists but no code change seen)
  - [OK] 0 ready, [OK] 0 already-done
- **6 non-`[VALIDATED]_` tickets** (`P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md`, `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH.md`, `P1_ANDROID_CRASH_TRIAGE.md`, `P1_ANDROID_LAN_DISCOVERY_REPAIR.md`, `P2_ANDROID_IDENTITY_QR_PRERENDER.md`, `P2_ANDROID_IDENTITY_SCROLL_FIX.md`)
- **1 `REJECTED/` subdirectory** with 5 stale drafts
- **1 `IN_PROGRESS/`** with the Agy handoff ticket (unresolved from 2026-06-07 20:24 PT)

### Discrepancy
- The kanban board is 8 tasks, the file-based backlog is 51 tickets, the triager reports 35 blocked. **None of these are linked.** The triager doesn't know about the kanban board. The kanban board doesn't know about the file tickets. Lucas is correct that we need to unify them.

---

## 2. Dispatch Plan (read this before creating any tasks)

### Phase 1  Unify state (no code, no builds, ~5 min)

**Goal:** Make the two machines reference the same reality.

1. **Inventory both machines** into one source of truth. Output: `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md` with three tables:
   - All kanban tasks (id, status, title, last_update)
   - All HANDOFF/todo/ files (filename, triage bucket, status, deps)
   - Cross-reference: which kanban task covers which HANDOFF file (if any)

2. **Add triage_bucket frontmatter to each `[VALIDATED]_*.md`** so the triager and the kanban are forced to agree next time. Use a single bash pass:
   ```bash
   cd /mnt/e/SCMessenger-Github-Repo/SCMessenger
   for f in HANDOFF/todo/[VALIDATED]_*.md; do
     # parse the existing bucket from the most recent triage scan
     # insert `triage_bucket: <blocked|ready|missing-deliverable|done>` after the first H1
   done
   ```
   Actually  the triager is read-only and rewrites nothing. Don't fight it. Instead, **add a one-line `triage_status:` and `kanban_id:` frontmatter to each ticket** so the next audit can cross-reference in one pass.

3. **Reconcile the 8 kanban-blocked tasks** against the 40 `[VALIDATED]_` tickets:
   - `t_570a8f4e` Setup: Android build toolchain  supersedes nothing; status is "tools installed, E: drive SDK, WSL NDK 26b" per `STATE/2026-06-05_ANDROID_P0_024_P1_022_BUILD_VERIFIED.md`. **Should be `done`.**
   - `t_2fcfa616` Build Rust core for Android  P0_ANDROID_024 supersession candidates exist. **Should be unblocked or closed-as-superseded.**
   - `t_3118d436` Implement missing UI screens  maps to `P1_ANDROID_022_BLE_Stale_Cache_Cleanup`, `P2_ANDROID_IDENTITY_QR_PRERENDER`, `P2_ANDROID_IDENTITY_SCROLL_FIX`, plus Findings A/B from the Agy handoff. **Decompose into 3-4 sub-cards or close-as-superseded.**
   - `t_93efd990` BLE Transport  maps to `P1_ANDROID_022_BLE_Stale_Cache_Cleanup.md` (already shipped per `d4a9214d` build commit). **Should be `done` or close-as-superseded.**
   - `t_0b4a8488` WiFi Direct Transport  no direct ticket, but `P1_ANDROID_LAN_DISCOVERY_REPAIR.md` may cover. **Reconcile or create new ticket.**
   - `t_f3314103` Notifications + FCM  no direct ticket. **Create `HANDOFF/todo/[VALIDATED]_P2_ANDROID_Notifications_FCM.md` stub or close.**
   - `t_d211204b` Build APK  superseded by the existing `assembleDebug` green state. **Should be `done`.**
   - `t_73f0ffdc` Test on device  mapped to multiple `[VALIDATED]_P1_*_TEST_*.md` and `P1_CLI_033_Comprehensive_Windows_E2E_Smoke_Test_Harness.md`. **Decompose or close-as-superseded.**

   For each of the 8: take the most-defensible action (mark `done` if shipped, `cancelled` with reason if superseded, or unblock-and-dispatch if real remaining work). Document every action in the unified backlog file.

4. **Move `IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07.md`**  it's been idle since 2026-06-07 20:24 PT (12+ hours). Either mark it done-with-rollforward or move its 6 bugs into a fresh `IN_PROGRESS/IN_PROGRESS_*_agy_bugs_sweep_2026-06-08.md` and claim it. **Do not delete it; the rollup is valuable.**

### Phase 2  Bulk unblock the 35 triage-blocked tickets

**Goal:** Every [VALIDATED]_ ticket has a real status, not a synthetic "blocked."

Read the triager's output (see Section 6 source pointers) and reclassify each of the 35:

- **Already shipped (commit on `integration/v0.2.2-pre-android-push-2026-06-05` or `fix/p0-android-*`)**  mark `triage_bucket: done`, prepend the commit SHA, move to `HANDOFF/done/`.
- **Real remaining work, independent**  unblock (kanban `unblock` if linked, else just remove the synthetic block marker from the frontmatter).
- **Real remaining work, blocked on a dependency**  leave blocked, write the parent reference in the frontmatter, surface as a "Phase 3 wave" in the unified backlog.
- **Missing deliverable (the 5 from triager)**  either (a) find the code/dispatch that proves it's done, or (b) author the dispatch contract as a new file, or (c) cancel with reason.

For each ticket, prepend a one-line `## Triage Decision  2026-06-08` block with: status (shipped/blocked/cancelled/superseded), commit SHA if shipped, parent ticket if blocked, decision rationale.

Use a single delegated worker pass for the mechanical reclassification  see Section 5 dispatch.

### Phase 3  Test & build & verify

**Goal:** Green build + green tests + a working CLI binary.

Run in this order, halt on first failure:

```bash
# Pre-flight
cd /mnt/e/SCMessenger-Github-Repo/SCMessenger
export CARGO_INCREMENTAL=0
df -h /mnt/e /mnt/c  # C: must have > 2GB free

# 1. Rust workspace check (fastest signal)
cargo check --workspace 2>&1 | tee tmp/build_logs/cargo_check_$(date +%s).log

# 2. Cargo test (the full suite)
cargo test --workspace 2>&1 | tee tmp/build_logs/cargo_test_$(date +%s).log

# 3. Android build (the big one  ~10-15 min)
cd android
./gradlew assembleDebug -x lint --quiet 2>&1 | tee ../tmp/build_logs/gradle_assemble_$(date +%s).log

# 4. Android unit tests
./gradlew :app:testDebugUnitTest 2>&1 | tee ../tmp/build_logs/gradle_test_$(date +%s).log

# 5. CLI smoke test (start daemon, hit endpoints, stop)
cd ..
./target/release/scmessenger-cli.exe init --data-dir /tmp/scm-smoke-$$
./target/release/scmessenger-cli.exe start --data-dir /tmp/scm-smoke-$$ --listen 9200 --http-port 9201 &
DAEMON_PID=$!
sleep 3
curl -s http://127.0.0.1:9201/api/status
curl -s http://127.0.0.1:9201/api/diagnostics
kill $DAEMON_PID
```

**Self-verify each artifact** (do not trust subagent prose):
- `target/debug/scmessenger-cli.exe` exists and `os.path.getsize() > 1_000_000`
- `android/app/build/outputs/apk/debug/app-debug.apk` exists and `os.path.getsize() > 1_000_000`
- `target/debug/deps/*.so` count > 0 for `libscmessenger_core` (if Linux dev)
- No `[Command interrupted]` or `error: linking` in logs

**Halt conditions** (report and stop, do not fix silently):
- `cargo check` fails with linker errors  check `patch/if-watch-full/` and `patch/if-watch/` (Android stub)
- `cargo check` fails with `kani-proofs` feature conflict  `cargo check --workspace --no-default-features`
- `assembleDebug` fails on NDK path  verify `/home/scemessenger/android-sdk/ndk/26.1.10909125` exists (WSL path) AND `E:\Android\sdk\ndk\26.1.10909125` (Windows path)
- `assembleDebug` fails on cargo-ndk missing  `cargo install cargo-ndk`
- Gradle wrapper permission denied  `chmod +x android/gradlew`

### Phase 4  Verify-gate per ticket

After Phase 3 is green, for every ticket that claims "code change shipped" in the last 6 hours (see Section 4 audit scope), run a focused verify:

1. `git log --since="6 hours ago" --name-only --pretty=format:"%h %s"`  enumerate changed files
2. For each `[VALIDATED]_` ticket that references those files: confirm the code is on disk via `read_file` and the commit SHA matches
3. For each ticket that claims a test exists: confirm the test is in the test suite and `cargo test <test_name>` passes
4. For each ticket that claims an APK was built: `os.path.exists('android/app/build/outputs/apk/debug/app-debug.apk')`

### Phase 5  Commit and roll up

```bash
# 1. Stage ONLY the files this sweep touched
cd /mnt/e/SCMessenger-Github-Repo/SCMessenger
git status -s  # review carefully

# 2. Explicit adds (no `git add -A`  the dirty tree is 4+ files from prior sessions)
git add HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md
git add HANDOFF/IN_PROGRESS/  # the new handoff file and any moved ones
git add HANDOFF/todo/ HANDOFF/done/  # only files this sweep actually reclassified
# DO NOT add: android/ (build artifacts), tmp/, target/, *.log, *.pid

# 3. Commit with a comprehensive message
git commit -m "swarm: unified backlog + 35-ticket triage sweep + green build verify (2026-06-08)

- Phase 1: HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md (single source of truth for kanban + HANDOFF/todo/)
- Phase 2: reclassified 35 triage-blocked tickets (X shipped, Y real-remaining, Z cancelled)
- Phase 3: cargo check --workspace + cargo test --workspace + gradle assembleDebug + smoke test
- Phase 4: per-ticket verify of last 6h commits
- Phase 5: this commit (no push per Lucas's gate)

Refs: IN_PROGRESS_task_agy_android_stability_complete_handoff_2026-06-07
Quota: 5h=33.8% 7d=6.0% (TIER 2 EXECUTE)
Build: [PASS|FAIL with details]"

# 4. Write the roll-up
cat > HANDOFF/STATE/2026-06-08_SWEEP_RESULTS.md <<EOF
# 2026-06-08 Sweep Results

**Status:** [GREEN|YELLOW|RED]
**Build:** cargo check=[PASS|FAIL] cargo test=[PASS|FAIL] assembleDebug=[PASS|FAIL]
**Tickets reclassified:** 35 / 35
**Commits this sweep:** <SHA>
**Outstanding blockers:** [list any remaining]
**Next session should:** [specific actionable next step]
EOF
```

---

## 3. Files This Sweep Will Touch

```
HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md          [NEW  single source of truth]
HANDOFF/STATE/2026-06-08_SWEEP_RESULTS.md            [NEW  sweep rollup]
HANDOFF/STATE/2026-06-08_LIVE_VERIFY_PASS.md         [NEW  Phase 6 pass report, or .FAILURE.md]
HANDOFF/IN_PROGRESS/IN_PROGRESS_*_sweep_2026-06-08.md [NEW  the active claim, replaces agy handoff if needed]
HANDOFF/todo/[VALIDATED]_*.md ( 42 files)          [EDIT  add triage decision block to each]
                                                  [+2 new: settings-identity-entropy + windows-wsl-cli-e2e]
HANDOFF/done/                                        [MOVE  shipped tickets land here]
HANDOFF/REJECTED/                                    [CLEAN  drop the 5 stale drafts]
android/, core/, cli/, wasm/, mobile/                 [BUILD ONLY  no source edits unless Phase 3 finds a real bug]
scripts/verify_windows_wsl_cli_e2e.sh                [NEW  Phase 6 orchestration driver]
```

### 3.1 New tickets to fold into the unified backlog

These two were added 2026-06-08 from Lucas's follow-up DMs. They are independent of Phases 15 but should be unblocked and dispatched as part of this sweep:

- **`[VALIDATED]_P1_ANDROID_Identity_Generation_From_Settings_Missing_Entropy_And_Hangs_30s.md`** (15.6 KB)
  Bug: Settings  Identity path skips the finger-move EntropyCanvas entirely and silently hangs ~30s on "Generate Identity" with no progress feedback. Fix: extract a shared `IdentityCreationFlow` composable used by both Onboarding and Settings, add a re-entrancy guard to `IdentityViewModel.createIdentity`, pass the touch-entropy salt through. ~80 LoC across 3 files.
- **`[VALIDATED]_P1_VERIFY_Windows_WSL_CLI_Discovery_Messaging_E2E.md`** (11.0 KB)
  Bug: After the build is green, the Windows + WSL CLI pair has never been round-trip-verified end-to-end (no message delivery, no mutual discovery timing). Fix: author `scripts/verify_windows_wsl_cli_e2e.sh` that starts both nodes on the same LAN, watches `/api/peers`, exchanges a message in each direction, calls `verify_receipt_convergence.sh` on the logs, and writes a JSON summary + pass/fail report. GATED on Phase 3 build green.

**Sequence:** Phase 1 (UNIFY)  Phase 2 (UNBLOCK)  Phase 3 (TEST & BUILD)  Phase 4 (VERIFY code-level)  Phase 5 (COMMIT)  **Phase 6 (LIVE VERIFY)  windowswsl cli pair + receipt convergence  Phase 7 (FINAL COMMIT + REPORT)**.

Phase 6 is the new addition from Lucas's "live running the windows and Ubuntu (WSL) SCMessenger CLI apps" directive.

**Hard rule: no source edits unless a build/test failure is unambiguous and the fix is mechanical (typo, missing import, path typo). For any non-trivial code change, stop and escalate.**

---

## 4. Audit Scope: Last 6 Hours of Work

Per Lucas: "check of any other Claude code session ran for the empty slot for work product.. audit all work done in the last 6 hours."

```bash
# Enumerate commits in the last 6 hours across all branches and worktrees
cd /mnt/e/SCMessenger-Github-Repo/SCMessenger
git log --all --since="6 hours ago" --pretty=format:"%h %ai %an %s" 2>&1
git log --all --since="6 hours ago" --name-only --pretty=format:"===%h %s===" 2>&1
```

Worktrees known to be live:
- `E:/SCMessenger-Github-Repo/SCMessenger` (this one, `d4a9214d` integration/v0.2.2-pre-android-push-2026-06-05)
- `E:/SCMessenger-build-p0-024` (`7c362c63` fix/p0-android-024-identity)
- `E:/SCMessenger-build-p0-025` (`e84f4fc3` fix/p0-android-025-mdns-listener-collision)
- `E:/SCMessenger-build-p0-024.stale2026-06-06/` (preserved, broken  leave alone)

For each worktree, `git log --since="6 hours ago"` and reconcile with the HANDOFF/IN_PROGRESS/ ticket (the Agy handoff). Anything in worktrees that isn't reflected in HANDOFF needs to be surfaced in the unified backlog.

---

### Phase 6  Live network verification (gated on Phase 3 green)

**Goal:** Confirm the build actually works on the wire.

1. Pre-flight: both `target/release/scmessenger-cli` (WSL/Linux) and `target/release/scmessenger-cli.exe` (Windows) must exist. If either is missing, run the platform-appropriate build (cargo from WSL, `build_desktop.ps1` from Windows).
2. Run `bash scripts/verify_windows_wsl_cli_e2e.sh` (the new script the worker authors). This starts both nodes on the LAN, exchanges a test message in each direction, and writes a JSON summary.
3. The script must call `scripts/verify_receipt_convergence.sh` on the captured logs and report `failed_message_ids: 0`.
4. On pass: write `HANDOFF/STATE/2026-06-08_LIVE_VERIFY_PASS.md` with the JSON summary + latencies.
5. On fail: write `HANDOFF/STATE/2026-06-08_LIVE_VERIFY_FAILURE.md` with the failure axis + the last 50 lines of each node's stdout + cross-refs to the existing ticket that matches the failure mode (e.g. `P1_CLI_026 External_Address_Omits_LAN_Interface` for the `os error 10061` pattern from Agy's task-389.log). Do **not** silently fix. Stop and escalate.

### Phase 7  Final commit + report

- Commit the Phase 6 script + report (separate commit from Phase 5's sweep commit, so the build is green before the network test runs).
- Update the unified backlog with Phase 6 outcome.
- DM Lucas the final sweep summary (see Section 9).

## 5. Worker Dispatch Hints

If you (Overseer Claude) decide to delegate portions of this to workers:

| Phase | Worker profile | Toolsets | Why |
|---|---|---|---|
| Phase 1 inventory | `default` (any) | `file` | Pure read + write a markdown table |
| Phase 1 reconciliation | `implementer` | `file` + `terminal` (git only) | `kanban unblock`, `git mv`, frontmatter edits |
| Phase 2 bulk reclassify | `default` | `file` | Mechanical edit; no code changes |
| Phase 3 cargo check + test | `implementer` | `terminal` + `file` | Standard build verify |
| Phase 3 assembleDebug | `android-builder` | `terminal` | NDK + Gradle expertise |
| Phase 3 smoke test | `qa-tester` | `terminal` | Knows the CLI subcommands |
| Phase 4 per-ticket verify | `verifier` | `file` + `terminal` (read-only git) | Self-Report Gate enforcement |
| Phase 5 commit | `default` (you) | `terminal` | Overseer does the commit |

**Use cloud models** for Phase 3  TIER 2 EXECUTE allows it, and the build will be too long for the local `scm-coder:7b` to hold state across a 15-minute assembleDebug. Per the capability matrix:
- Rust work  `glm-5.1:cloud` (fallback `qwen3-coder-next:cloud`)
- Android work  `qwen3-coder-next:cloud` (fallback `glm-5.1:cloud`)
- Smoke test  `qwen3-coder:480b:cloud` (fallback `qwen3.5:397b:cloud`)

**Self-Report Gate (per kanban-orchestrator pitfalls):** Every worker that says "I built X" must return the absolute path + size + SHA256 of the artifact. Overseer verifies with `os.path.exists() + os.path.getsize() + sha256sum` before accepting.

**Blank-Response Gate:** If a worker returns a plan with zero tool calls, dispatch is broken  re-dispatch to a cloud model. Local models under 14B are unreliable for multi-step mechanical work.

---

## 6. Source Pointers (where the data lives)

For the worker that picks up Phase 1 (unification):

- **Kanban state:** `PYTHONPATH=/usr/local/lib/hermes-agent/venv/lib/python3.11/site-packages:/usr/local/lib/hermes-agent python3 -m hermes_cli.main kanban list` and `kanban show <id>` for each
- **HANDOFF inventory:** `ls HANDOFF/todo/[VALIDATED]_*.md` (40 files)
- **Triage output:** latest entry in HANDOFF/STATE/ from `scmessenger-triage.py` (read HANDOFF/REPLY_*.md or the auto-triage stdout)
- **Triage script:** `scripts/triage.sh` (log-dir analyzer  not the same as `scmessenger-triage.py` which is a slot 2 bridge)
- **Build env:** see `HANDOFF/CLAUDE_CODE_PROTOCOL.md` for the build commands + E: drive SDK path + WSL NDK path

For the worker that picks up Phase 3 (build):

- **Build env (June 5 2026):** E: drive. SDK `/mnt/e/Android/sdk`. WSL-native NDK `/home/scemessenger/android-sdk/ndk/26.1.10909125` (Linux r26b). JDK `/home/scemessenger/.local/jdk/jdk-17.0.12+7` (PLUS). Build copy on ext4 `/home/scemessenger/scmessenger-build/` to bypass 9P-bridge.
- **Run `export CARGO_INCREMENTAL=0` before any cargo command** (Windows-specific gotcha to prevent `.rlib` corruption).
- **CLI test isolation:** `XDG_DATA_HOME=/tmp/path` redirects data dir. For 2nd-instance, use `relay --listen X --http-port Y`. Config key is `bootstrap_node_add` (singular, _add suffix), NOT `bootstrap_nodes`.
- **Pixel 6a status:** OFFLINE as of 2026-06-06 (PHASE 2 retest). Live mDNS retest blocked on hardware reattach. Do not block this sweep on it.

For the worker that picks up Phase 4 (verify):

- **Subagent Self-Report Gate:** verify every claim with `os.path.exists()` + `os.path.getsize()`. The 2026-05-27 hallucination event: a `llama3.2:3b` subagent claimed `.so` files at specific sizes; manual verification found zero `.so` files in the target tree.
- **HANDOFF/STATE/2026-06-05_ANDROID_P0_024_P1_022_BUILD_VERIFIED.md** is the last known "build was green" snapshot  use it as the baseline to compare against.

---

## 7. Hard Constraints (no exceptions)

1. **No push to remote.** Local commits only. Lucas reviews and pushes.
2. **No source code edits** unless Phase 3 build/test failure is unambiguous and the fix is mechanical. Non-trivial code change  stop and escalate.
3. **No `git add -A`**  the working tree has 4+ dirty files from prior sessions. Use explicit `git add <path>...`.
4. **No `/tmp`, `/var/tmp`, `/dev/shm`**  use `tmp/` inside the repo.
5. **No `.py` files in repo root**  put scripts in `scripts/`.
6. **No C: drive installs**  Android SDK, NDK, build outputs go on E:.
7. **No new orchestration framework**  use the existing `.claude/orchestrator_manager.sh pool launch` workflow.
8. **No false "doesn't exist" claims** in any persistent file. If you assert a skill/command/path doesn't exist, broad-search first (`ls ~/.hermes/skills/`, `ls .claude/commands/`, `find . -name X`).
9. **No MarkdownV2 `</3` or unescaped `_`/`*` in Telegram DMs.** Use periods or em-dashes in sign-offs.
10. **No silent retry loops.** If a build halts, report and stop. Lucas's directive: "Engine to stop if any issue immediately."

---

## 8. Acceptance Criteria (what "done" looks like)

This sweep is **done** when **all** of the following are true:

- [ ] `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md` exists with the 3-table cross-reference
- [ ] Every `[VALIDATED]_*.md` in `HANDOFF/todo/` has a `## Triage Decision  2026-06-08` block (or has been moved to `done/` with the decision in the move commit)
- [ ] All 8 kanban-blocked tasks have a final status (`done` / `cancelled` with reason / unblocked-and-dispatched)
- [ ] `cargo check --workspace` exits 0
- [ ] `cargo test --workspace` exits 0 (or, if pre-existing failures exist, they are documented and reproduced)
- [ ] `cd android && ./gradlew assembleDebug -x lint --quiet` produces `app/build/outputs/apk/debug/app-debug.apk`  1 MB
- [ ] `./gradlew :app:testDebugUnitTest` exits 0
- [ ] A CLI smoke test started, served a `/api/status` 200, and was killed cleanly
- [ ] `HANDOFF/STATE/2026-06-08_SWEEP_RESULTS.md` exists with build status + commit SHA + next-action
- [ ] `scripts/verify_windows_wsl_cli_e2e.sh` exists and runs cleanly on the WindowsWSL CLI pair
- [ ] `HANDOFF/STATE/2026-06-08_LIVE_VERIFY_PASS.md` OR `2026-06-08_LIVE_VERIFY_FAILURE.md` exists with the JSON summary
- [ ] Local commit created (not pushed), with the comprehensive message in Section 2 Phase 5
- [ ] Second local commit with the live verification script + report (Phase 7)
- [ ] DM to Lucas on Telegram: "Sweep done. Build: [PASS/FAIL]. Tickets reclassified: 35/35 + 2 new. Outstanding: [list]. Commits: <SHA1>, <SHA2>."

**If any of the above is false at end of sweep, the sweep is RED. Report RED, do not paper over it.**

---

## 9. End-of-Sweep Telegram DM Template

```
Sweep done. Build: [PASS|FAIL]. Tickets reclassified: X/35. Outstanding: [list]. Commit: <SHA>.
```

Replace the bracketed bits. Send via `send_message(target="telegram:6014795323", ...)`. No markdown that breaks MarkdownV2 (no `</3`, no unescaped `*`/`_`).

---

*End of handoff. Overseer: claim, execute Phase 1  5, verify, commit, DM. Hermes standing by for the result.*
