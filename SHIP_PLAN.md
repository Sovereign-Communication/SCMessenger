# SCMessenger Ship Plan -- v0.4.0 Public Alpha

Status: Active
Created: 2026-08-14
Owner: Operator (Treystu)
Supersedes for execution purposes: `HANDOFF/todo/_QUEUE.md` (see Amnesty, S0-4)

This is the **only** execution queue until v0.4.0 is tagged and downloadable.
If a task is not on this page, it is not being worked on.

---

## 0. North star

> Two people who have never met, on two phones, with no shared network,
> exchange a message and both see a delivery receipt -- using a build a
> stranger downloaded from the GitHub releases page.

**Definition of done for v0.4.0:**

| # | Exit criterion | Evidence required |
|---|---|---|
| D1 | `main` is green | All CI lanes pass on a push to `main`, run URL recorded |
| D2 | Signed APK is downloadable | `gh release view v0.4.0-alpha.1` lists an APK asset |
| D3 | README explains the product and how to install | File is non-empty, links resolve |
| D4 | Two-device message + receipt | Receiver-side decrypt + durable history + receipt, per `project_fleet_run_scoring_evidence` -- NOT transport ACKs |
| D5 | No long-lived integration branch | PR #139 merged or closed; `main` is trunk |
| D6 | Transport racing demonstrated | Message delivered when first-choice transport is unavailable, proving fallback selects a working path. Receiver-side decrypt + durable history + receipt -- NOT transport ACKs, NOT UI counters, NOT BLE local acceptance |
| D7 | Offline proximity messaging demonstrated | Two devices exchanging a message with no internet available. Receiver-side decrypt + durable history + receipt -- NOT transport ACKs, NOT UI counters, NOT BLE local acceptance |

Anything that does not move D1-D7 is deferred. No exceptions until tag.

---

## 1. Credit discipline (read this before dispatching anything)

The plan is designed so that **Claude native tokens are spent only on verdicts**.
Roughly 80% of the work below is mechanical and belongs on free lanes.

| Lane | Cost | Use for | Do NOT use for |
|---|---|---|---|
| **Qwen Claude Code CLI** (`launch_claude.ps1`) | Free | PRIMARY. Scoped diffs, CI log triage, README drafting, doc archiving | Unscoped "analyze the codebase" tasks -- it rewrites code |
| **agy** (`--add-dir`, pinned `--model`) | Free | adb/UI poking, log greps, single build commands | Multi-step reasoning; needs resume not restart on timeout |
| **Fusion Lite** | ~2c/run, 10c hard cap | Pre-commit diff review, plan sanity checks | Implementation |
| **DashScope / OpenRouter / Groq** | Free | Overflow when Qwen quota is dry; Groq micro-validation only | Large-file full-mode edits (silent truncation) |
| **Claude native (this session)** | Expensive | Go/no-go verdicts, adversarial crypto review, merge decisions | Log greps, doc moves, formatting, ticket triage |

**Rules that protect the budget:**

1. Dispatch from a scratch cwd with `--add-dir` so `CLAUDE.md` and `docs/rules/`
   are not pre-loaded into every worker; inject only the rules that task needs.
2. One task file -> one provider. Parallel dispatch of the same `--task` file
   collides on the tmp output path and one result is silently lost.
3. Use `--mode diff` for anything under ~500 lines. Full-file mode truncates
   silently on large files.
4. Verify delegated *verification* claims. A worker reporting "gate passed" is a
   claim, not evidence -- require the run URL or the command output.
5. Never re-dispatch reactively. Authorize a batch, let it run, check once.

**Expected native spend for this whole plan: 5-8 verdict checkpoints.** Each one
is a short read of evidence plus a go/no-go. That is the budget.

---

## 2. Sequenced workstreams

Sprints are strictly ordered. S1 gates S2 gates S3. Do not parallelize across
sprints -- a red `main` makes every downstream result unverifiable.

### S0 -- Clear the decks (half a day, mostly free lanes)

| ID | Task | Lane | Done when |
|---|---|---|---|
| S0-1 | Commit or stash the current working-tree changes on `tracking/pre-v040-tag-work`. Shared checkout -- do not touch files you did not create. | Operator | `git status` shows only intentional work |
| S0-2 | Triage the 16 open PRs into MERGE / CLOSE / DEFER. The 13 dependabot PRs are one batch decision, not 13. | Qwen | A 16-line table with a verdict per PR |
| S0-3 | Merge or close PR #139. This is a decision, not a task -- if it cannot merge this week, close it and cherry-pick what matters. | **Native verdict** | D5 satisfied |
| S0-4 | Backlog amnesty: `git mv HANDOFF/todo/* HANDOFF/archive/` except items that map to D1-D7. Keep `_QUEUE.md`. | agy | `HANDOFF/todo` holds <= 10 files |
| S0-5 | Untrack root junk: `screen.png`, `window_dump.xml`, `local.properties`, stray `adb_logcat*.txt`. `local.properties` holds local SDK paths and should never have been tracked. | Qwen | `git ls-files` root listing is clean |

> S0-5 note: `.gitignore` already covers `*.pem` and `*apiKey*.csv`, so the
> untracked key and CSV in the working tree are ignored, not leaked. Confirmed
> 2026-08-14. Do not commit them.

### S1 -- Green main (the gate everything else depends on)

Four lanes are failing as of run `31659699771` (2026-08-13). Fix in this order;
each is independently mergeable.

| ID | Lane failing | What we know | Assigned lane |
|---|---|---|---|
| S1-1 | **Mobile** | Root cause identified: `Release signing is not configured; release tasks must fail.` Needs a signing config wired from `android/keystore.properties.template` + repo secrets. This is also a hard blocker for D2. | Qwen (config) + Operator (secrets) |
| S1-2 | **Repository Hygiene** | Previously fixed once by `7f369f50` (trailing whitespace) and regressed. Fix the check to be enforceable pre-push, not just in CI. | agy |
| S1-3 | **Lint** (Rust clippy/fmt) | Exact error not yet isolated -- the `--log-failed` output is dominated by toolchain-setup noise. First task is to extract the real error lines. | Qwen |
| S1-4 | **Docker Integration Suite** | Long-standing amber lane. If it cannot be fixed in one pass, mark it non-blocking and say so explicitly in the workflow -- do not leave a permanently red required check. | Qwen, then **native verdict** on blocking status |

**Local pre-push guard (do this once, saves CI cycles):**

```bash
cargo fmt --check; if [ $? -ne 0 ]; then echo "[FAIL] fmt"; fi
```

Never read `$?` after a pipe -- a piped gate can never fail.

**S1 exit:** one push to `main` where every lane is green. Record the run URL.
This is **native verdict checkpoint 1**, and it satisfies D1.

### S2 -- Make it downloadable

| ID | Task | Lane | Done when |
|---|---|---|---|
| S2-1 | Write `README.md`. It is currently 0 bytes. Use the existing repo description as the opening line; sections: what it is, threat model in three sentences, install (Android APK, CLI), build from source, project status honesty note. | Qwen drafts, **native edits** | File is non-empty and accurate |
| S2-2 | Wire release signing (depends on S1-1) and produce a signed APK from a tagged commit with `SCM_GIT_HASH` embedded -- `816422fc` already exports it. | Operator + agy | APK installs on the Pixel 6a |
| S2-3 | Tag `v0.4.0-alpha.1` and publish a release with the APK attached and real release notes drawn from `CHANGELOG.md`. Latest public release is v0.1.9 from March -- close that five-month gap. | Operator | D2 + D3 satisfied |
| S2-4 | Set the repo homepage URL to the install guide. Enable Discussions as the inbound channel. | Operator | Repo metadata updated |

**S2 exit: native verdict checkpoint 2** -- read the README as a stranger would
and confirm the download path works end to end.

### S3 -- Prove the north star

| ID | Task | Lane | Evidence |
|---|---|---|---|
| S3-1 | Rebuild all nodes to the tagged SHA. Per `HANDOFF/PR139_FIVE_NODE_GATE_STATUS_2026-08-13.md`, Windows CLI and AWS were on stale SHAs and macOS/iOS were offline -- that gate has never actually run clean. | agy + Operator | Every node reports the tag's git hash |
| S3-2 | Run the two-device test on the **released APK**, not a dev build. Cross-network: one on cellular, one on WiFi. | Operator + agy | Receiver decrypt + durable history + receipt |
| S3-3 | If it fails, the failure becomes the only ticket. Do not open a workstream -- fix and re-run. | Qwen impl | Re-run passes |
| S3-4 | Transport racing gate: message delivered when first-choice transport is unavailable, proving fallback selects a working path. | Operator + agy | Receiver-side decrypt + durable history + receipt (NOT transport ACKs, UI counters, or BLE local acceptance) |
| S3-5 | Offline proximity gate: two devices exchange a message with no internet available. | Operator + agy | Receiver-side decrypt + durable history + receipt (NOT transport ACKs, UI counters, or BLE local acceptance) |

**S3 exit: native verdict checkpoint 3** -- score the run on receiver-side
evidence only. Transport ACKs, UI counters, and BLE local acceptance do not
count. This satisfies D4, D6, and D7.

### S4 -- After the tag (do not start before it)

- **External crypto audit.** Hybrid X25519 + ML-KEM-768 is the differentiator and
  the liability. Self-review by the fleet that wrote it is not a credential.
  Budget real money here, not tokens.
- **Android last mile.** 162 unwired functions, 84 in `MeshRepository.kt`. Burn
  down only what D4 exercises; the rest is speculative surface.
- **Dependency debt.** Six months of unpatched deps on a security product.

---

## 3. Governance changes (permanent, start now)

1. **Red main is a stop-the-line event.** No feature work while a required lane
   is failing. A red main makes every other result unverifiable.
2. **Trunk-based.** Branches live under 48 hours. No more long-lived tracking
   branches -- that is how #139 became a second main.
3. **Agents are measured in commits merged to green main**, not handoff documents
   produced. The repo currently holds 1,695 markdown files / ~223k words against
   ~120k lines of Rust. Stop writing to each other.
4. **One doc per fact.** `docs/CURRENT_STATE.md` is the state; this file is the
   plan. New handoff docs require a reason that is not "context transfer".
5. **Concurrent-lane cap.** Give each agent its own `git worktree`. The shared
   checkout has already destroyed uncommitted work once.

---

## 4. Explicitly not doing (until after tag)

- v0.5.0 / v1.0.0 planning, PQC-14 close-out, farm drills, KMP/meeting mode
- iOS parity work (`iOS_V040_PARITY_IMPLEMENTATION_PLAN.md`) -- Android ships first
- The remaining 78 unwired non-Android functions
- Any new orchestration tooling, dashboard, or visualizer

Each of these is defensible on its own. Together they are why nothing has
reached a user since March.

---

## 5. Checkpoint ledger

Fill this in as the plan executes. Empty cells are the honest status.

| Checkpoint | Criterion | Date | Evidence |
|---|---|---|---|
| CP1 | D1 -- main green | 2026-08-23 | `main`@`b538f3ba`: every push-triggered workflow (CI, Lint, Repository Hygiene, Docker Publish, Docker Integration Suite, Cross, iOS Build & Test, Mobile) reports `conclusion: success`, verified via the GitHub Actions API. Two P0 fixes (#221, #222) remain open and DRAFT, blocking the tag, not `main`'s own greenness -- see `HANDOFF/CTO_STATE.md` 2026-08-23 checkpoint sections. |
| CP2 | D2 + D3 -- release published | | |
| CP3 | D4 -- two-device proof | | |
| CP4 | D5 -- #139 resolved | | |
| CP5 | D6 -- transport racing proof | | |
| CP6 | D7 -- offline proximity proof | | |
