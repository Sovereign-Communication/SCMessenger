# CTO state — live handoff

Status: Active
Last updated: 2026-08-16 (merge train advanced; see the banner below)
Entry point: `/CTO`. This file is the whole context load.

> **2026-08-16 — READ `HANDOFF/CTO_DISPATCH_PLAN_2026-08-16.md` FIRST.**
> #167, #168, #169 and #165 are **merged to tracking**. The lane picture in §3
> below **inverted** since it was written: `Mobile`/KSP UniFFI is now GREEN and
> `Test` went RED on two transport tests. The dispatch plan carries the
> re-derived table, the verified merge mechanics, and the routing plan.
> Sections §1, §4, §5, §6, §7 and §8 of this file remain accurate.

## 0. STANDING RULE — keep this file current

**Update this file at the END of every session, and immediately on any
important change.** Operator directive, 2026-08-16, standing.

"Important" means: a merge or close, a gate result, a decision made or reversed,
a blocker found or cleared, a claim in here proven wrong. Do not batch these to
the end — a session that dies mid-run leaves the next one reading fiction.

When a section here is overtaken by events, **mark it superseded and say what
replaced it. Do not delete it.** The history of a wrong call is how the next
session avoids re-making it; every §8 lesson exists because someone deleted the
context instead of the conclusion.

## 0a. HANDOFF -- 2026-08-19. Read this first, then sections 0b/0c.

**D1 and D5 are DONE.** PR #139 merged to `main` at `6e70a3db`; `tracking` fully
absorbed (`git rev-list --left-right --count origin/main...origin/tracking` -> `1 0`).
All main lanes green. `main` is now `b4ccd30a`. Docker Publish fired: image
**`sha-6e70a3d`** exists, which unblocks the D4 node rebuild.

### Open PRs -- merge order

| PR | What | Gate |
|---|---|---|
| **#180** | DUAL_BIND fix (operator-approved: advertise only what bound) | **needs a fresh CRITICAL_VALIDATOR** -- touches `core/src/transport/`, rule 8 |
| **#179** | field evidence, AGENTS.md **rule 16**, wiring audit, `check_wiring.py` (5/5 tests, verified by CTO), zai lane | ready |
| **#177** | P0 dispositions | **NEEDS CORRECTION -- see below** |
| #178, #170, #156, #154 | Apple API limit, free lanes, docker non-blocking, APK signing verify | #154 must merge before the tag |

### TAG-BLOCKING work not yet started

**Nine Android features are wired out** -- implementation present, call sites
absent (`ebf5411b` restored files but not their callers). Full list in
`docs/fieldtest/ANDROID_WIRING_AUDIT_2026-08-18.md`. Worst three:

1. **Diagnostics/logs viewer** -- `Screen.Diagnostics` is DEFINED (`MeshApp.kt:397`)
   and NAVIGATED TO (`:287`) with **no `composable()` registration**. Lands nowhere.
2. **QR APK sharing** -- `ApkShareDialog.kt:36`, zero callers. Damages **D2**.
3. **QR join-mesh** -- `JoinMeshScreen.kt:49`, never in the NavHost.

A restoration PR is still to be written. Verify each with
`python scripts/check_wiring.py` (exit 1 = findings), **not by eye**.

### CTO error to correct in #177

`NO_MOBILE_BOOTSTRAP` was dispositioned to S4 **because** `JoinMeshScreen`
supplied a working QR join path. **It is orphaned.** That reasoning is void, so
re-open the ruling. Do not let the wrong justification stand.

### Operator rulings, 2026-08-19

- **WS deferred** to unblock Android; returns **before v1.0.0**. #180 emits TCP
  only. Cost: browser/WASM peers have no transport. Recorded in the field doc.
- **zai `glm-4.7-flash` is the primary free lane.** MANDATORY quirk: send
  `"thinking":{"type":"disabled"}` or it returns `content:""` -- a silent
  vacuous success. Qwen paid remains **off limits**.

### Field state -- rollout on real hardware

Windows node + Pixel 6a both run `b4ccd30a`. **#176 verified live**:
`pm query-activities -a VIEW -d scmessenger://invite` resolves to MainActivity.
**Messaging does NOT work between them**: 14,496 x `Failed to negotiate transport
protocol(s)`, 13 peers marked dead, 0 peers discovered. That is DUAL_BIND, and
#180 is the fix awaiting review. The in-app message to the operator is still
**queued, never delivered** -- do not claim otherwise.

### Traps that cost time this session

- `adb logcat` main buffer hides crashes. Use **`adb logcat -b crash`** first.
  Diagnosing without it produced a confident wrong answer (blamed memory).
- Do **not** set `CARGO_TARGET_DIR` for the Android gradle build. jniLibs come
  from `core/target/android-libs`; overriding it ships an APK with **no**
  `libscmessenger_core.so` and gradle still says BUILD SUCCESSFUL.
- `git show <rev>:.dotted/path` needs `MSYS_NO_PATHCONV=1` -- fails as
  plausible emptiness.
- agy has a **~90s per-tool timeout** separate from `--print-timeout`. Never ask
  it to run a cold full build.

## 0a-bis. SESSION LOG -- 2026-08-18 (CTO)

### 1. What landed and merged
- **Six PRs opened/tracked this session:**
  - **#181** `fix(orchestration): zai lane returns empty content without thinking disabled`
  - **#182** `feat(orchestration): session launch gate and end-of-session delegation audit`
  - **#183** `fix(android): restore wiring -- ALL NINE wired-out features`
  - **#184** `docs(cto): correct the NO_MOBILE_BOOTSTRAP deferral`
  - **#185** `docs(cto): session log 2026-08-18 -- all nine Android features rewired, #180 re-reviewed` (this handoff branch)
  - **#180** advanced from RED to near-green (commits `0d533dbc`, `4e67f750`)
- **#186 MERGED to main as 3bd3c947 (commit `af16cea0`):**
  - main's `Cargo.lock` now carries `h2 0.4.16`. This unblocked the entire merge train.
  - **RUSTSEC-2026-0258 fix:** Patched "h2 unbounded empty DATA frames" (h2 0.4.15 -> 0.4.16, LOW severity).
  - **The bump trap:** `cargo update -p h2 --precise 0.4.16` on the local toolchain (cargo 1.96.1, MSRV-aware resolver) CASCADED into unrelated DOWNGRADES -- `socket2 0.6.5 -> 0.5.10` and `windows-sys 0.61.2 -> 0.52.0/0.59.0/0.48.0`. That was rejected. The h2 `dependencies = [...]` block in `Cargo.lock` is BYTE-IDENTICAL between 0.4.15 and 0.4.16, so a two-line hand edit of `version` + `checksum` was provably sufficient. Final diff: +2/-2, one file. `cargo metadata --format-version 1` confirmed cargo accepts it without rewriting.
  - **Operator decision on #186 checks:** The merge was made under an EXPLICIT OPERATOR DECISION naming each pending check, because `pr_scope.sh` requires exactly that. The four pending checks were:
    1. `Android Debug APK` -- answered by `cargo tree -i h2`, which proves every path to h2 terminates at `scmessenger-cli` and it does NOT reach `scmessenger-core`, `-mobile`, `-wasm` or the Android/iOS apps.
    2. `Android JVM Unit Tests` -- answered by `cargo tree -i h2` (same proof).
    3. `iOS Build` -- answered by `cargo tree -i h2` (same proof).
    4. `Repository Hygiene Checks` -- answered by a local `git diff --check`, clean, 1 file, 2 lines.
    (Lint itself was GREEN, as were Test on ubuntu/windows/macos).

### 2. #180 DUAL_BIND state and CI Lint diagnostic
- **Root cause of the two red Test lanes:** `core/tests/test_multiport.rs` `test_custom_ports_only` asserted `addresses.len() == 6` for 3 ports, i.e. TWO addresses per port. That assertion ENCODED the dual-bind contract #180 removes. It was a stale contract, not a regression. Fixed in `0d533dbc`, which TIGHTENED the test (asserts `/tcp/` present and `/ws` absent).
- **CTO-verified gates:**
  - `cargo fmt --all --check` [OK] (exit 0)
  - `cargo test -p scmessenger-core --test test_multiport` [OK] (12 passed, 0 failed)
  - `cargo clippy -p scmessenger-core --all-features -- -D warnings` [OK] (exit 0)
- **Independent CRITICAL_VALIDATOR finding (`gemini-3.1-pro-high`):** returned [BLOCK] and FALSIFIED the CTO's own claim that #180 "emits TCP only". `core/src/transport/swarm.rs:2760-2770` unconditionally binds `/ip4/0.0.0.0/tcp/9002/ws` for the WASM bridge, and `EXCLUDED_PORTS` held only 9876 -- so configuring port 9002 would recreate dual-bind. The CTO verified the finding directly and did NOT override it. Resolved by `4e67f750` (9002 added to `EXCLUDED_PORTS` plus a unit test). A re-review was dispatched.
- **CI "Lint" cause:**
  - *[SUPERSEDED -- cause was NOT yet identified when written]:* The CI "Lint" job (~1m11s), cause NOT yet identified. `fmt` and core `clippy` both pass locally, so it is NOT those two. A workspace-wide clippy was still running when this was written. DO NOT MERGE #180 until Lint is green and the re-review verdict is recorded as a durable artifact.
  - **IDENTIFIED AND FIXED (2026-08-18):** Cause: the Lint job's fourth step, `cargo deny check`, reported `advisories FAILED` for `RUSTSEC-2026-0258` ("h2 unbounded empty DATA frames", h2 0.4.15, LOW severity, patched 0.4.16). It was red on EVERY open PR simultaneously, including PRs touching only markdown, while main showed green because main's last run predated the advisory.
  - **Diagnostic test (reusable):** The decisive cheap test was checking Lint on a PR with no Rust in it. If a markdown-only PR fails `cargo deny`, the advisory database updated upstream.

### 3. Android wiring: operator ruled ALL NINE before the tag
- `python scripts/check_wiring.py` is the gate. NEVER assess wiring by eye.
- **Baseline on `origin/main`:** 32 findings (10 C1_ZERO_CALLERS, 1 C2_UNREGISTERED_ROUTE, 1 C3_MANIFEST_MISSING, 20 C4_TRANSITIVE_DEAD).
- **After #183:** exit 0, ZERO findings [OK], verified independently by the CTO. Operator ruling, 2026-08-18: everything wired and WORKING for v0.4.0, `JoinMeshScreen` included.
- **Manifest audit discrepancy & CEO correction on record:**
  - `ANDROID_WIRING_AUDIT_2026-08-18.md` manifest section was PARTLY STALE: it listed `MeshVpnService` and `BootReceiver` as unregistered, but #176 had already restored them. Only `ShareReceiver` was actually missing. This is exactly why the gate is a script and not a document.
  - The CEO reported "three manifest registrations still missing" and a 106-line `AndroidManifest.xml`. That reading came from the SHARED CHECKOUT, which is 37 commits behind `origin/main`. `origin/main`'s manifest is 148 lines and already registers `BootReceiver` and `MeshVpnService` (PR #176). Only `ShareReceiver` was missing. `check_wiring.py` reported exactly one `C3_MANIFEST_MISSING` and was correct. On #183 the manifest is 165 lines with all seven components. Lesson to record: derive from `origin/main`, never from the shared working tree.
- **Build verification and validation:**
  - *[SUPERSEDED -- Build status: #183 has NOT been compiled yet. The Android gradle build gate is still owed.]*
  - **Compiled clean:** `./gradlew :app:compileDebugKotlin` returned `BUILD SUCCESSFUL` in 51m 53s, exit 0 [OK].
  - **Two independent `CRITICAL_VALIDATOR` passes (`gemini-3.1-pro-high`):** verdicts committed to `docs/security/PR183_VALIDATION_2026-08-18.md`.
    - **Pass 1:** [BLOCK] with three HIGH findings (passphrase data loss on an ignored `commit()` return; unconsented dial of attacker-supplied addresses, reachable from any web page via the `BROWSABLE` intent filter; hardcoded strings).
    - **Pass 2:** [APPROVE_WITH_FINDINGS], prior block cleared, and it caught a FOURTH bug both Pass 1 and the CTO missed: `Toast.makeText` called from `Dispatchers.IO` in `ShareReceiver` -- compiles clean, crashes at runtime with `Looper.prepare()`. Pre-existing, but #183 made it reachable by registering the receiver.
- **CI wiring gate:** `check_wiring.py` is now wired into CI as an "Android Wiring Gate" job in `.github/workflows/mobile.yml` (on PR #183, so the gate and the fix land together -- adding it anywhere else turns main red, since main still has 32 findings). It runs the gate's own unit tests FIRST, then the gate, with no shell pipeline masking the exit code and no `continue-on-error`. This satisfies the CEO's tag-blocking requirement and makes AGENTS.md rule 16 executable.
- **Rule 16 citation:** Rule 16 DOES exist; PR #179 adds it ("RESTORING CODE IS NOT RESTORING A FEATURE. WIRE IT, OR IT IS DEAD."). The CEO believed the citation in `check_wiring.py` was wrong because #179 is unmerged. No fix needed.

### 4. Security finding in #183 [OPEN] -- needs an operator ruling before the tag
#183 routes `MeshRepository.getPlatformSecuredPassphrase()` from plaintext `context.getSharedPreferences("platform_secure_keys", MODE_PRIVATE)` to `SecurityUtils.getEncryptedSharedPreferences(context)`. That is a genuine fix -- a backup passphrase was being stored in the clear. But there are TWO hazards:
1. **MIGRATION:** `SecurityUtils` uses a DIFFERENT file, `"scmessenger_secure_prefs"` (`SecurityUtils.kt:18`). On an existing install the lookup returns null and the code GENERATES A NEW passphrase, orphaning any existing backup. No migration step exists, and the old plaintext secret is left on disk.
2. **RECOVERY PATH DESTROYS SECRETS:** `SecurityUtils.kt:26` calls `context.deleteSharedPreferences(...)` and retries whenever `EncryptedSharedPreferences` fails to initialise. Android `KeyStore` invalidation on a lock-screen or biometric change is a common, expected event, so this can silently destroy the stored passphrase. Pre-existing in `SecurityUtils`, but #183 makes it load-bearing for user data for the first time.
- **Status [OPEN] -- NEEDS AN OPERATOR RULING BEFORE THE TAG:** Record this as [OPEN]. CTO recommendation: add a migration that reads the old file, writes it into the encrypted store, then deletes the plaintext -- and do not merge that hunk of #183 until it exists.

### 5. Tooling findings, CI runner pathology, and branch protection
- **zai glm-4.7-flash:** Returns HTTP 200 with `content:""` unless the request carries `"thinking":{"type":"disabled"}`; the answer goes to `reasoning_content` instead. The CTO reproduced both halves live against the API. #181 fixes `scripts/delegate.py`. `scripts/lane_probe.py` has the SAME bug and is NOT yet fixed. The zai free tier also rate-limits fast -- a third call within a few minutes returned HTTP 429, so it cannot carry unlimited bulk work.
- **`session_orchestration_audit.py` (#182):** STATUS column is unreliable: it reported 5 of 7 dispatches as ERROR/TIMEOUT when they had completed successfully with valid reports. Its token and step accounting looked correct. Fix before trusting the STATUS column.
- **Preflight hook false positives:** Produced THREE false positives this session: it matches the literal string `"agy"` and the characters `"|"` plus `"$?"` anywhere in a command, including inside unrelated Python source and in correctly-written non-piped commands. Same class of defect as #167.
- **Disk reclamation:** C: fell to 3.9 GB free (99%). The operator approved reclaiming `.scm-zai-target` (7.37 GB, pure cargo artifacts, no git), two merged worktrees' `target/` dirs, and the `SCMessenger-ZaiComplete` checkout (clean, 0 uncommitted, 0 unpushed, fully pushed to `Treystu/soc-em.git`). Recovered to 16.6 GB. `.scm-shared-target` (26 GB) was deliberately PRESERVED as the warm `scmessenger-core` cache.
- **CI runner pathology (2026-08-18):** Two jobs hung rather than failed: the Lint job ran 2h26m while EVERY sibling job in the same run completed successfully, and Android Debug APK was auto-cancelled at 1h15m. The fix that worked: cancel the run, then `gh run rerun <id> --failed` once the queue had drained; the rerun finished in ~13 minutes. Also: pushing repeatedly to one PR spawns a full run-set per push (7 workflows), which starves every other PR. Cancelling SUPERSEDED run-sets on your own branch is safe and took the queue from 8-done/20-queued to 15-done/12-queued.
- **Branch protection status [OPEN]:** `main` is NOT branch-protected. `gh api repos/Sovereign-Communication/SCMessenger/branches/main/protection` returns HTTP 404 "Branch not protected". The handoff records `apply_branch_protection.sh --apply` as operator-approved, but it has never been run. This is step 3 of the documented path to the tag and is still open.

### 6. Orchestration (operator directive, 2026-08-18)
Delegate through Antigravity. `"agy"` IS the Antigravity CLI. Tiering:
- **CTO:** Drives high-level strategy and decisions.
- **`gemini-3.7-flash-high`:** Orchestrates and implements.
- **`gemini-3.1-pro-high`:** Runs adversarial `CRITICAL_VALIDATOR` passes -- using a different, stronger model than the implementer is what makes the review independent, and it is what caught the 9002 finding.
- **zai `glm-4.7-flash`:** Carries bulk simple work once #181 lands.
- **Session lifecycle scripts:** Session start runs `scripts/session_launch_audit.sh`; session end runs `scripts/session_orchestration_audit.py` (both from #182).
- **Session stats:** 7 dispatches, 590 worker steps, roughly 1.63M worker tokens.

### 7. Still true, do not soften
Two nodes on one LAN STILL cannot message each other until #180 merges. #180 is the fix and it is NOT merged. No v0.4.0 tag exists. The in-app message to the operator is still queued, never delivered.

## 0b. OPERATOR APPROVAL GATE — standing, 2026-08-16

### Who may merge — role-bound, not negotiable

| Role | May merge? |
|---|---|
| **CTO** (this seat) | **YES** — under the confidence test below. Merging is a CTO decision |
| ORCHESTRATOR / CONTROLLER | **NO.** Coordinates, dispatches, integrates verified output. Never merges |
| Worker lanes — SCANNER, IMPLEMENTER, VALIDATOR, agy, any HTTP lane | **NEVER.** They open PRs and report. A worker holding merge rights defeats every gate above it |
| OPERATOR | Always, and overrides this table |

A worker asking the CTO to merge is a dispatch event, not permission. A green
gate is not approval either: `pr_scope.sh` exiting 0 means no reason was
*found*.

### The confidence test — deterministic, run it in order

Before any **irrevocable or potentially destructive** action:

1. **Am I at 100%?** All five must hold, or the answer is no:
   - every gate green, or every blocker named out loud with evidence answering it
   - every required review exists as a **durable artifact**, not a recollection
   - **zero UNKNOWNs.** Undetermined is never treated as safe
   - the blast radius is bounded and I can state it
   - I verified the load-bearing claim myself, with a command, this session
2. **At 100%** -> execute. Do not ask. Sequencing is the CTO's to own.
3. **Below 100%** -> confer with the CEO session. Reach 100% or consensus, then
   execute.
4. **CEO and CTO cannot both reach 100%** -> escalate to the operator with both
   positions stated. Do not split the difference.

"I think it's fine" is below 100%. "The worker said so" is below 100%. "It
passed CI" alone is below 100% when a review artifact is also required.

### Blast radius — only as big as it needs to be

**Keep the blast radius only as big as it needs to be, within the constraints
currently available.**

Both halves bind. Minimise scope, sequence so a failure is small and
attributable, prefer several small merges to one large one, and never inflate a
change already in flight with unrelated work. But minimise **within what is
actually achievable now** — #139 is 204 commits because `tracking` is the
long-lived integration branch and collapsing that is not available today. The
rule asks for the smallest radius reachable, not an impossible one.

Worked example, 2026-08-16: #174 (required for D1) merged alone, while #171 and
#173 — tooling with no bearing on D1 — were held until after the trunk merge.
Batching them would have saved two ~50 minute CI cycles and inflated the largest
PR in the repo. Wall clock was the cheaper thing to spend.

---

**The test for gating is DESTRUCTIVENESS, not whether it writes.** Operator
directive, standing, refined 2026-08-16.

The operator's reasoning, which is the rule: *opening a PR "isn't destructive,
and really only helps to safely preserve data, as it offers a place to track the
changes."* Gating work-preservation strands work in worktrees — which is how
this repo has lost things. We are moving to **small, frequent PRs**; a
200-commit PR is what made per-merge buyoff necessary, and that is going away.

| Operator approval FIRST (irreversible, or outside CTO authority) | CTO executes at 100% confidence | Proceed freely (preserves work, or read-only) |
|---|---|---|
| Tags, releases, branch protection | **Merging** (see the confidence test) | Reading anything; read-only git (`log`/`diff`/`merge-tree`/`rev-list`) |
| Force-push, history rewrite | Closing/reopening a PR | **Opening a PR. PR comments. PR body/title updates.** |
| Deleting a branch or worktree registration | Reclaiming `target/` in a SAFE worktree | Committing and pushing to **your own** branch |
| Anything touching the shared checkout's working tree | Pushing to a shared branch you own | Writing files in **your own** worktree |
| Deleting files or worktrees outside a SAFE `target/` | Dispatching an IMPLEMENTER into an isolated worktree | `pr_scope.sh`, `gh pr checks`, CI logs |
| | | Compile/test verification (deconflict builds first) |
| | | Dispatching read-only SCANNER / VALIDATOR |
| | | `tmp/` scratch; reporting findings and recommendations |

Investigation is not a change. Verification is not a change. Preserving work in
a tracked place is not a change. **Destroying, discarding, or releasing is.**

Two calls the CTO made by inference rather than instruction — correct them if
wrong: **closing** a PR is treated as gated (it discards rather than preserves,
even though it is reopenable), and **IMPLEMENTER dispatch is free when
isolated**, because an isolated writer produces a branch and a PR, which is
preservation. An implementer that would touch the shared checkout or a shared
branch is gated.

A green gate is still not approval: `pr_scope.sh` exiting 0 means no reason was
*found*, not that the operator said yes.

Present the evidence, state the recommendation, then wait. A green gate is not
approval; `pr_scope.sh` exiting 0 means no reason was *found*, not that the
operator said yes.

## 0c. The verification loop — keep this shape

This is the loop that caught a CRITICAL-adjacent defect on 2026-08-16 after the
CTO had already talked himself into "it looks fixed". Do not shorten it.

1. **The controller never self-certifies.** Reading the code and concluding it
   is fine is a *claim*, not a review. `docs/ORCHESTRATION.md` forbids the
   controller from making that call and AGENTS.md rule 8 requires an
   independent sign-off. The CTO read `swarm.rs`, saw the guardrail call, and
   declared W1 fixed. An independent validator found the cooldown was erased by
   `forget_peer` on full disconnect. **The gate exists for the person running
   it, not just for other people.**
2. **Frame the packet adversarially.** Hand the worker your reading as *a claim
   to falsify*, in those words: "If you merely agree with it, this review has no
   value." A packet that asks for confirmation gets confirmation.
3. **Spot-check what comes back.** A delegated verification is still a claim.
   Verify the load-bearing assertion with your own command — not the whole
   report, just the one thing the verdict rests on.
4. **Expect corrections in both directions.** On 2026-08-16 workers corrected
   the CTO twice (the #164/#169 renormalization claim; W1), and the CTO
   corrected workers twice (a `git diff -w` cited as empty when blank lines
   survive it; a "270 occurrences" census that counted argv unpacking). Neither
   side is the authority. The command output is.
5. **Prefer UNCERTAIN to a clean answer.** Tell workers so explicitly. This gate
   already produced one false "[OK] clear" while six gated files were invisible.
6. **Artifacts, not chat.** Verdicts go to `docs/security/` or the PR. A review
   that exists only in a session transcript did not happen — and untracked work
   in this shared checkout has been destroyed before, so commit it.

7. **A REPEAT MISTAKE IS A PROCESS DEFECT, NOT A MEMORY LAPSE.** Operator
   directive, standing 2026-08-16. The second occurrence of any mistake stops
   being about the mistake and becomes about the process that failed to catch
   it. When one happens: run an RCA, then **change the mechanism** — do not
   write a better reminder.

   **The governing finding: a lesson stored as prose gets re-learned; a lesson
   stored as a gate does not.**

   Evidence, from this repo. Rule 14 has an executable form (`pr_scope.sh`) and
   has held. Rule 13 has none. The trailing-whitespace lesson had none — it sat
   in §8 as prose, the CTO **quoted it earlier the same day**, and then committed
   a worker-produced artifact verbatim and turned `Repository Hygiene` red on the
   trunk merge for the second time, in the same lane, on the same PR (#174).
   Re-reading was never going to be the fix.

   So the RCA question is never "why did I forget?" It is **"what gate was
   missing, and can I build it?"** If a gate genuinely cannot exist, say so
   explicitly and accept the residual risk in writing — that is a decision, not
   an oversight.

   Corollary for delegation: **worker-produced files are not exempt from the
   repo's gates.** Worker *code* gets compiled and reviewed; worker *artifacts*
   — markdown, reports, docs — were being committed on sight. Run
   `git diff --cached --check` (and the emoji check) against anything a worker
   generated, before committing it.

Mechanics that keep dispatch healthy: isolated worktree per writer, never the
shared checkout; deconflict builds (`tasklist` for cargo/gradle/java) before any
dispatch that builds; a distinct log-dir per concurrent `agy_run.sh` or two runs
on the same model and SHA silently overwrite each other; `--add-dir` and an
exact `--model` always; 30m+ timeouts, and a transient `error_message` mid-run
is not a failure — check whether it recovered before re-dispatching.

### Seat status

**2026-08-16: this is the ONLY live CTO session.** The other sessions listed by
`mcp__ccd_session_mgmt__list_sessions` as `isRunning: true` — "Cto resume v040",
"Scm cto 1000 hst" — are **stale processes, not active seats** (operator
confirmed). The §8 "one CTO seat" caution stands for the future, but it is
resolved for now: no need to re-establish the seat before merging.

### Session log — 2026-08-16

| Change | Evidence |
|---|---|
| #167, #168, #169, #165 merged to tracking | `manager.rs:470` carries `saturating_sub`; `.gitattributes` carries `*.kt`/`*.kts`/`*.md eol=lf` |
| `Repository Hygiene` and `Rust Linting` went GREEN on #139 | 11s / 4m32s, confirmed from the check list |
| **#152 CLOSED** | Audited (`CTO-152-AUDIT`), then verified independently: whitespace + blank-line movement only; conflicts on `MeshApplication.kt`, which tracking superseded via `17216e1a`/`149d3725`. Nothing lost. Evidence on the PR |
| **#171 opened, HELD** | `pr_scope.sh` no-truncation + AGENTS.md rule 15. Independently validated (`CTO-171-VALIDATE`): APPROVE, R3 fails-closed verified. Held until #139 lands so it does not restart the trunk merge's CI |
| AGENTS.md **rule 15** added | No renumbering: all existing citations (rules 1,2,5,8,9,11,12,13,14) still resolve. Coherence audit dispatched as `CTO-AGENTS15-COHERENCE` |
| CEO escalation sent | README honest-first framing; dependency-deferral trigger |
| **The #139 crypto gate was found NOT satisfied** | Last recorded verdict was BLOCK. See the correction banner on §4 |
| **W1 found still live**, refuting the CTO's own reading | `CTO-139-CRYPTO-REVERIFY` -> `docs/security/PR139_REVERIFY_2026-08-16.md`. F1 FIXED, everything else CLEAN, W1 NOT FIXED |
| **W1 fixed** -> PR #172 | `forget_peer` removed with both call sites (native `:5397`, WASM `:7736`). Independently validated: `docs/security/W1_FIX_VALIDATION_2026-08-16.md` -- APPROVE_WITH_FINDINGS, W1 CLOSED, REGRESSION RISK NONE |
| W1 fix gates pass | target test 1 passed/0 failed; `cargo test --workspace --no-run` **CARGO_EXIT=0**, zero errors |
| **19 GB reclaimed** | 76 PDB files in `.scm-shared-target/debug/deps` held 19 GB of 27 GB. Disk 100% -> 92% |
| Operator approved the merge sequence | #172 -> tracking, then #139 -> main. Stop before branch protection and the tag |

### Open findings — not yet fixed, safe to dispatch

1. **Preflight guard false positive.** It blocks read-only commands whose *file
   path* merely contains `agy` — no dispatch involved. #167 fixed this class for
   git commands; still open for others. Do NOT reach for
   `SCM_SKIP_DISPATCH_CHECK=1` to read a log; use Read/Grep instead.
2. **`agy_run.sh` log collision.** `RAW="$LOGDIR/agy_${MODEL}_${STAMP}.jsonl"` is
   model + HEAD SHA, so two concurrent dispatches on the same model write the
   same file. Pass a distinct 4th arg (log-dir) per dispatch. Same class as the
   known `delegate_task.py` collision.
3. **145 `.md` files pending renormalization** — see §9 of the dispatch plan.
   Held; collides with #139, which touches 91 `.md` files.

4. **Rule 15 propagation backlog — the REAL list.** `CTO-AGENTS15-COHERENCE`
   returned "270 occurrences across 74 files". **That number is wrong; do not
   dispatch against it.** It pattern-matched `head`/`tail`/`[:N]`
   indiscriminately and counted argv unpacking (`sys.argv[1:4]`), display
   formatting (`pid[:22]`), deliberate single-value extraction (`adb version |
   head -1`), and even the comment lines in `pr_scope.sh` that *describe* the
   fix. It also under-reported, because the packet scoped the search to
   `scripts/` and `.claude/hooks/` and therefore missed `.codex/`.

   The genuine defect is narrow: **a list of violations, errors, or findings
   shown to a decision-maker, silently cut short.** Verified sites:

   | File:line | Truncation | Hides |
   |---|---|---|
   | `scripts/verify_all_builds.sh:24,31,38` | `tail -5` | clippy / gradle / iOS **build failure output** |
   | `scripts/verify_incremental_gate.py` (x5) | `stderr[:1000]` | compiler errors |
   | `scripts/verify_delivery_state_monotonicity.sh:64` | `regressions[:10]` | delivery-state regressions |
   | `scripts/verify_swift_violations.py:40` | `bad[:15]` | Swift violations |
   | `.claude/hooks/preflight_guard.py:571` | `risky[:4]` | risky ops shown before a destructive command |
   | `.claude/hooks/check_no_emoji.py:45` | `matches[:10]` | emoji violations |
   | `.codex/hooks/check_no_emoji.py:45` | `matches[:10]` | same file, second copy |
   | `scripts/rules_check.py:78` | `hits[:8]` | rule violations |
   | `scripts/repo_audit.sh:27` | `head -n 200` | audit hits |
   | `scripts/triage_lane.sh:72` | `tail -25` | `git diff --stat` between pass and fail |
   | `scripts/apply_branch_protection.sh:77,80` | `head -40` / `head -20` | branch-protection API state |

   `verify_all_builds.sh` is the worst of these: a script whose entire job is to
   prove the build is good shows only the last 5 lines when it is not.

   **Lesson for future dispatches:** a grep-shaped question returns
   grep-shaped answers. Ask for the *semantic* defect and require the worker to
   justify each hit, or budget for the CTO to sort the census by hand.

5. **W1 regression protection is thin** (from `W1_FIX_VALIDATION_2026-08-16.md`
   V5, non-blocking). With `forget_peer` gone the unit test cannot exercise the
   disconnect path, so it simulates the scenario in a comment. A future refactor
   could reintroduce a map-clearing call in `start_swarm_with_config` and the
   test would still pass. Closing it needs an integration test driving
   `SwarmEvent::ConnectionEstablished`/`ConnectionClosed` and asserting no
   redundant `LedgerExchangeRequest`. Does not affect current correctness.

6. ~~**`git merge-base --is-ancestor` disagreed with reality.**~~
   **DIAGNOSED AND FIXED 2026-08-16 (PR #173).**

   Root cause: `--is-ancestor` returns **0 = merged, 1 = NOT merged, 128 = REF
   ERROR**. The check collapsed non-zero to "not merged". PR #165 merged at
   16:53Z, GitHub deleted the branch, `git fetch --prune` dropped the local ref,
   and the check then returned **128** -- reported as "not merged".

   That is a **permanent** false negative: the ref never returns, so a
   merged-and-pruned worktree could never be reclaimed. It failed safe, but a
   gate that can never open is still broken.

   Fixed in #173 via `scripts/reclaim_safe.py`: 128 yields **UNKNOWN**, never
   "no" and never SAFE, with a PR-state fallback when the ref is gone. Verdict
   requires all three of clean + zero unpushed + merged. `reap_worktrees.sh`
   carried the same bug and is updated.

   **Verified reclaim survey (24 worktrees): 14 SAFE, 8 HOLD, 1 PATH-GONE,
   0 unpushed anywhere.** PR #165 confirmed MERGED (`81a4bbd2`). 5 GB reclaimed
   from `scm-android-gate` and `scm-fix-transport-defects`; source trees intact.

   Still open from this: **`e01c-pq-mixing` is registered in `git worktree list`
   but absent from disk**, so that list is not a trustworthy inventory. Needs
   `git worktree prune` -- not run, it deletes.

7. **`LNK1318: Unexpected PDB error; LIMIT` is disk exhaustion**, not
   corruption. Observed at 963 MB free / 100% full mid-link. Add it to the
   CLAUDE.md list beside `STATUS_STACK_BUFFER_OVERRUN` and "can't find crate".
   The reclaim that fixes it: delete `*.pdb` in
   `$CARGO_TARGET_DIR/debug/deps` -- 76 files held 19 GB. Debug symbols only,
   regenerated on link. **Never** touch `core/target/generated-sources/`.

8. **`scripts/clean_target.sh --dry-run` is not a real flag.** CLAUDE.md's
   routing table instructs running it before deleting artifacts; the script does
   not implement it and just prints usage. The documented safety step does not
   do what the doc says. Real modes are `--triples` and `--deps`.

9. **agy has a ~90s PER-TOOL timeout, separate from `--print-timeout`.** A
   worker with `--print-timeout 45m` still died polling a cold `cargo` build in
   90-second slices for 20 minutes -- and the build had actually SUCCEEDED. Do
   not ask an agy worker to run a cold full build: have it make the change and
   commit, then run the compile gate separately.

Everything below has a command next to it. **Re-derive before acting** — this
file ages, the repo does not.

---

## 1. The goal

Ship **v0.4.0 as an Android beta** the operator can hand to friends and family.
Then v0.5.0 iOS. `SHIP_PLAN.md` D1-D5 is the definition of done and the only
execution queue until the tag. **Nothing in v0.5.0/v1.0.0 scope starts before the
0.4.0 tag.**

Latest thing a stranger can download is **v0.1.9, from 2026-03-19.** That number
is the whole problem.

### Exit criteria status

| | Criterion | State |
|---|---|---|
| **D1** | `main` is green | **Blocked on #139.** Every red lane is explained; see §3 |
| **D2** | Signed APK downloadable | **Unblocked** — all four signing secrets set 2026-08-15 17:08Z. Needs the tag |
| **D3** | README explains the product | **DONE** |
| **D4** | Two-device message + receipt | Blocked on D2 and a node rebuild; see §5 |
| **D5** | No long-lived integration branch | **Blocked on #139** (merging it satisfies this) |

---

## 2. Where the merge train stands

**Merged to `tracking` this sprint (11):** #149 UniFFI build fix + 7 restored
Android sources + orchestration tooling; #150 lane routing; #153 backlog amnesty
87→8; #157 circuit-relay transport prefix; #158 pr_scope truncation fix; #159 D4
runbook; #160 release notes; #161 dead-IP retirement; #162 five-layer integration
suites; #163 shared cargo cache docs; #164 hygiene; #146 Android durable delivery.

**Closed:** #147, with proof — `git log --oneline 7538e4e9 --not origin/tracking`
returned zero commits. It was a branch cut from `tracking` but aimed at `main`,
so GitHub rendered the whole tracking-vs-main delta as its own 102-file diff.
**Read the ancestry, not the diff stat.**

### Open right now

```
gh pr list --limit 20
gh pr checks 139 ; gh pr checks 165
bash scripts/pr_scope.sh 139        # the REPAIRED gate -- see §6
```

| PR | Base | What it is |
|---|---|---|
| **#139** | main ← tracking | **THE TRUNK MERGE. D1 + D5 together.** Checks re-triggered 2026-08-16 after the four merges below |
| #152 | main | Hygiene whitespace. **CONFLICTS with tracking** and is probably obsolete after #164/#169. Verify after #139; do not close blind |
| #154 | main | `apksigner verify` guard. **Merge this before tagging** — see §5 |
| #156 | main | Docker Integration Suite non-blocking + issue #155 |
| #170 | main | Free-lane orchestration tooling. Its red `Lint` is `core/src/lib.rs:159` **inherited from main** — it self-clears when #139 lands |
| 13 dependabot | main | **DEFER all, close none.** They are the post-tag S4 queue. GitHub reports 7 vulnerabilities on the default branch, 3 high — real, but not before the tag |

**Merged to tracking 2026-08-16 (4):** #167 dispatch-guard false positives; #168
stale-checkout gate + dispatch timeout floor; #169 `.gitattributes` eol=lf +
whitespace/rustfmt (clears `Lint`, `Rust Linting`, `Repository Hygiene`); #165
transport saturating latency score + zero-duration bandwidth bypass (clears
`Test` ×3 and `macOS Native Tests`). #165 carried a full adversarial APPROVE,
zero findings, `CRYPTO_TOUCHED: NO`.

---

## 3. Critical path to the tag

1. ~~**#165 green → merge to tracking.**~~ **DONE 2026-08-16**, together with
   #167, #168 and #169. All four fixes verified present on `tracking`:
   `manager.rs:470` now reads `100u64.saturating_sub(...)`, and `.gitattributes`
   declares `*.kt`/`*.kts`/`*.md` as `eol=lf`. #139's checks re-triggered.
2. **#139 → main.** This is D1 + D5. The repaired `pr_scope.sh` will raise five
   blockers; four are resolved and must be named explicitly rather than silently
   overridden:
   - *"100 commits, is this based on the branch you are merging into?"* —
     intentional here. `tracking` IS the long-lived integration branch, and
     merging it is precisely what D5 asks for.
   - *"touches merge-blocked directories"* — true, six files. **The
     crypto-security-auditor verdict exists** (§4). Its one HIGH finding is fixed
     by #157, which is merged into tracking, so #139 now carries the fix.
   - *"checks still running"* — must actually be green. Do not merge on a
     pending check.
   - *"no conflicts"* — clean.
3. `bash scripts/apply_branch_protection.sh --apply` (operator approved;
   `enforce_admins` true, **0** required approvals — raising it to 1 locks a
   single-operator repo out, GitHub forbids self-approval). **Do NOT list
   `Docker Integration Suite` as a required check** — see §6.
4. **Merge #154**, then tag `v0.4.0-alpha.1`.
5. Verify the published APK is genuinely release-signed (§5). **D2 + D3.**
6. Rebuild the AWS node to the tagged SHA, then run D4 (§5).

### Why every red lane on `main` is red

> **Superseded 2026-08-16.** This table described the state on 2026-08-15 and has
> since inverted — `Mobile`/KSP is GREEN and `Test` went RED. The current table,
> re-derived from the logs, is §3 of
> `HANDOFF/CTO_DISPATCH_PLAN_2026-08-16.md`. Kept here for history.

Verified from the literal CI logs, not inferred:

| Lane | Real cause | Fix |
|---|---|---|
| `CI` | its only failing job was `Lint` → `cargo fmt` on `core/src/lib.rs:159` | #139 |
| `Lint` | the same single fmt diff | #139 |
| `Mobile` | KSP `error.NonExistentClass` — the UniFFI bug | #139 (carries #149) |
| `Repository Hygiene` | trailing whitespace | #164 (merged) |
| `Docker Integration Suite` | UniFFI metadata stripping in the container | #156, non-blocking |

`CI`'s other five jobs (Test on windows/ubuntu/macos, FFI Surface Contract, Docs)
all PASSED on `main`. The lane was red on one formatting diff.

---

## 4. Security review of #139 — verdict on record

> **CORRECTED 2026-08-16. THIS SECTION WAS WRONG ABOUT THE GATE.**
>
> It records the verdict as "NEEDS FIXES. No CRITICAL hole" and reads as though
> the crypto gate is satisfied. **It is not.** The actual artifacts in
> `docs/security/` are a three-link chain that ends unresolved:
>
> | Artifact | Commit | Verdict |
> |---|---|---|
> | `PR139_ADVERSARIAL_REVIEW_2026-08-08.md` | `6cb7033a` | **BLOCK** — F1 CRITICAL (RFC1918 ledger disclosure gate never checked the requester; internal subnet map + peer-id-to-private-address binding to any unauthenticated remote) plus F2–F5 HIGH |
> | `PR139_REMEDIATION_2026-08-08.md` | — | remediation claimed |
> | `PR139_REVIEW_15dbcde0_2026-08-09.md` | `15dbcde0` | **BLOCK** — supersedes the first for that range; everything else clean; new **W1**: failover re-exchange is an unrated outbound amplifier |
>
> **The last recorded verdict on this PR is BLOCK, and no artifact clears W1.**
> The section below never mentions F1 or W1 at all. Whoever wrote it was
> describing a different, later review of a narrower diff — the §8 lesson
> ("your own past statements are claims") applied to this file itself.
>
> CTO code reading on 2026-08-16 indicates both are fixed at the current head —
> W1 gated behind `allow_failover_reexchange` on native (`swarm.rs:5343`) and
> WASM (`:7708`) with tests at `:8764-8768`; F1 now requires the requester's
> observed address, fails closed on `None`, rejects `P2pCircuit` on both sides,
> excludes CGNAT, and narrows to /24 or /64 (`addr_filter.rs:470`).
>
> **That reading is NOT a review.** AGENTS.md rule 8 requires an adversarial
> sign-off and `docs/ORCHESTRATION.md` forbids the controller from making that
> call. `CTO-139-CRYPTO-REVERIFY` is dispatched to produce the missing artifact
> at `docs/security/PR139_REVERIFY_2026-08-16.md`.
>
> **Do not merge #139 until that verdict exists and says APPROVE.**

A `crypto-security-auditor` pass ran against the six merge-blocked files #139
touches (`core/src/crypto/backup.rs`, and `addr_filter/behaviour/dial_policy/
observation/swarm` under `core/src/transport/`; +1,645/-154).

**Verdict: NEEDS FIXES.** No CRITICAL hole; X25519 and XChaCha20-Poly1305
untouched. Nothing was found that admits, dials, or discloses to a peer that
should be rejected — the diff actually *tightens* several gaps (stale-connection
disclosure, fail-open block checks, nested relay circuits).

- **HIGH — `dial_policy.rs` `build_relay_addresses`. FIXED by #157 (merged).**
  The loop set `has_ip`/`has_port` for the Ip4/Ip6 and Tcp/Udp match arms but
  never pushed those components, so circuit-relay addresses lost their transport
  prefix entirely. libp2p's relay transport requires a concrete address and fails
  with `MissingRelayAddr` before any I/O, at debug log level only. Relay NAT
  traversal was broken mesh-wide, in the same change set that removed UPnP.
- **LOW** — blocked-peer status is a distinguishable oracle: Registration and
  Relay answer with an explicit `"blocked"` error while AddressReflection and
  LedgerExchange stay silent. Post-tag.
- **LOW/INFO** — `mdns_dial_attempted` is unbounded under LAN mDNS spoofing.
  Requires L2 adjacency. Post-tag.
- **INFO** — backup salt moved `OsRng` → `rand::random()`. Cryptographically
  equivalent; no action.

**Two further defects, found by the integration suites in #162, fixed in #165:**

- `transport/manager.rs` — `std::cmp::max(0, 100 - latency_ms as u64)`. The
  subtraction evaluates first and u64 has nothing below zero, so any link over
  100 ms panicked in debug and **wrapped to near `u64::MAX` in release, inverting
  transport selection so the worst path scored highest.** Every cellular/WAN link
  is over 100 ms, and D4 is a cross-network test.
- `transport/internet.rs` — timestamps are whole seconds, so a relay in the same
  second as registration gave `conn_duration == 0` and the `if conn_duration > 0`
  guard skipped the bandwidth limit entirely.

Both were **pre-existing** (present on `main`, untouched by #139).

---

## 5. D2 and D4 — what is actually required

### D2 — signing is unblocked, but not proven

All four secrets are set (verified by name only; no agent has ever seen a value):
`SCMESSENGER_KEYSTORE_BASE64`, `_KEYSTORE_PASSWORD`, `_KEY_ALIAS`, `_KEY_PASSWORD`.

**Secrets existing is not proof signing works.** The base64 went through a
PowerShell pipe, and `release.yml`'s signed steps are conditional on
`HAS_KEYSTORE` — a malformed secret still yields a green job and a **debug-signed
APK**. Merge **#154** before tagging; it adds `apksigner verify --print-certs` and
fails the job on an unsigned or debug-signed artifact. If tagging first, check by
hand — `CN=Android Debug` means it did not sign.

The keystore lives at `%USERPROFILE%\kiee\` on the operator's machine. Never in
the repo, never read/copied/printed by any agent, permanently operator-only.

### D4 — Pixel 6a ↔ the AWS node

Operator decision: D4 runs **Android ↔ the AWS node**. There are no "relays" in
this architecture — every node relays — so this is node-to-node and
**cross-platform**, which is *stronger* evidence than two Android handsets.

Verified live 2026-08-15 via the EC2 API as `user/scmessenger-relay-orchestrator`:

```
i-006b14491d421bd0d  running  t3.micro  us-east-1  tag scm-always-on-node
curl http://54.226.67.101:9876/health   -> {"status":"healthy"}  (256 ms)
```

- **The node is Amazon Linux 2023, NOT Ubuntu.** `ssh ubuntu@` gives
  `Permission denied`; **`ssh ec2-user@` works**. At least 8 repo docs still say
  `ubuntu@` and every one of them fails.
- **`HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` is the canonical address pointer
  and it is correct.** The IP is dynamic — the account holds zero Elastic IPs and
  `ec2:AllocateAddress` is an **explicit deny** in `SCMessengerRelayFreeTierOnly`.
  Do not try to route around that; it is a deliberate cost guardrail, and the
  product does not need a stable address (v0.4.0 removed hardcoded bootstrap
  addresses; discovery is invite/QR ledger seeding).
- **The node runs code from a closed branch** — `/version` reports `9f54b107` on
  `gpt/pr139-receipt-filter-20260811` (PR #147, closed). It must be rebuilt.
- **NEVER build on the t3.micro.** A previous attempt ran 16 hours and OOMed.
  Pull the CI-prebuilt image.
- **Ordering is forced:** `docker-publish.yml` only fires on push to `main`,
  publishing `sha-<7char>`. So **#139 → main → CI publishes the image → rebuild
  node → run D4.** There is no way to prove D4 before the trunk merge.
- Runbook: `HANDOFF/D4_NODE_REBUILD_RUNBOOK.md` (merged, #159). Identity baseline
  to prove a rebuild did not orphan the ledger:
  `libp2p_peer_id 12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy`.

Scoring is unchanged: **receiver-side decrypt + durable history + receipt.** Not
transport ACKs, not UI counters, not BLE local acceptance.

---

## 6. Tooling and infrastructure

| Script | Purpose |
|---|---|
| `scripts/pr_scope.sh` | executable "unless there's a reason not to?"; **fails closed** |
| `scripts/triage_lane.sh` | first moves on a red lane — history before hypothesis |
| `scripts/agy_run.sh` | dispatch with per-step progress + stall detection |
| `scripts/reap_worktrees.sh` | reap abandoned worktrees; refuses DIRTY ones |
| `scripts/clean_target.sh` | scoped artifact reclamation; never calls `cargo clean` |
| `scripts/apply_branch_protection.sh` | branch protection, dry-run verified |

**`pr_scope.sh` failed open on 2026-08-15 and has been repaired (#158).** It read
its file list from `gh pr view --json files`, which caps at **100 files**. #139
changes 253. The first 100 held none of the merge-blocked paths, so it printed
`[OK] clear of core/src/{crypto,transport,routing,privacy}` while six gated files
were invisible — on the largest PR in the repo, on the exact check it exists for.
It now derives from `git diff --name-only origin/<base>...origin/<head>`,
announces loudly if it ever falls back to the API, and fails closed when the API
returns exactly 100 files. **Any PR reporting exactly 100 changed files should be
assumed truncated.**

**Docker Integration Suite is non-blocking (#156)** via `continue-on-error: true`
on the single failing step — narrowly scoped, so other Docker breakage still
fails the job. It therefore reports green while that step is broken. **D1 is to be
evaluated with this lane explicitly excluded**, and it must NOT be listed as a
required check in branch protection. Issue #155 tracks the real fix.

**Shared cargo cache (#163).** Every dispatched worker used to build its own
`target/`; one reached 16 GB and filled a 237 GB disk to 99%, at which point rustc
failed with `no space on device` and the compile gate could not run at all. Use:

```
export CARGO_TARGET_DIR=C:/Users/SCM/Documents/GitHub/.scm-shared-target
export CARGO_INCREMENTAL=0
```

Concurrent builds then block on the cargo lock — which enforces the existing
"never two build tools at once" rule rather than fighting it. Documented in
`docs/rules/BUILD_AND_CI.md`. Disk was 4.2 GB free at worst, 34 GB after cleanup.

---

## 7. OPEN — do not guess

1. **Was `ebf5411b`'s deletion of 7 Android sources intentional?** Restored on
   #149 on the CTO's read that APK sharing is active work. If it was a deliberate
   strip-down, revert the restore. Note the Josh fork independently deleted the
   *tests* for two of them, preserved on `josh-fork/local-worktree-state-2026-08-15`.
2. **Josh single-transport build** — operator ruled it is NOT the v0.4.0 default;
   ships as **v0.3.9** if at all. The transport quarantine described in an earlier
   session summary is **not implemented**; `d0e3258a` is 4 files, +23/-5.
3. **README framing** — the CEO was asked to bless the honest-first tone before
   the tag. No reply as of handoff.
4. **Dependency debt** — 7 vulnerabilities on the default branch, 3 high. Deferred
   to post-tag S4, which is right for shipping but should not stay deferred long
   on a security product.

---

## 8. Standing lessons

**Open the file.** Repeatedly this project has classified an artifact without
reading it and been wrong: `GEMINI.md` was already correct; two "duplicate pairs"
were prefix collisions; #147 looked like 102 files of unique work and had zero
unique commits; the repo already had a maintained canonical node-address file
that ~99 documents ignored. **The repo is consistently more coherent than its
directory listing suggests.** `AGENTS.md` rules 13 and 14 exist because of this.

**Verify the mechanism before quantifying.** "AWS node is down" came from a
5-second curl cap against an address that had simply changed. The node was
healthy in 256 ms the whole time.

**Commit before you clear.** A disk cleanup nearly discarded four untracked
integration-test files. Committing them to a branch first is what surfaced two
shipping transport defects on their very first compile. Two other worktrees held
work that existed on **no** remote: `wip-w1-ledger` (20 commits + 73 uncommitted
lines, unpushed entirely) and the JoshFork clone's 5 working-tree changes. Both
are now on origin. **Survey for unpushed work before reclaiming anything.**

**Committing a file as-is is not the same as it passing CI.** Preserving four
untracked files faithfully introduced 9 lines of trailing whitespace and turned
`Repository Hygiene` red on the trunk merge. Run the repo's own checks against
anything you commit, even when you are only preserving someone else's content.

**Dispatch budgets: 90 minutes, not 45.** Three "capability failures" on this
project were too-short timeouts. A task ending in `cargo test --workspace` needs
90m; the relay-ladder fix died at 36 minutes mid-compile with the work complete
but unpushed. Its worktree survived, so nothing was lost — check the worktree
before re-dispatching.

**One CTO seat.** Two sessions ran concurrently on 2026-08-15 and both were
editing this file and capable of merging. They did not collide, but only by luck.
If you find evidence of another active session, establish who holds the seat
before merging anything.

**Destructive history:** `git checkout <ref> -- .` destroyed four files of another
session's uncommitted work. `cargo clean --target <triple>` wiped 44.7 GB. The
preflight hook now blocks both and prints the working form — if it fires, read
it; it is there because someone already paid for that lesson.
