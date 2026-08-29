# API-Reset Execution Charter -- 2026-08-28

Status: Active
Author: CEO seat (Claude), shadow audit at operator request
Authority: Executes `SHIP_PLAN.md` D1-D7. Supersedes nothing. Amends SHIP_PLAN
in place via task L4-3 rather than becoming a second plan.
Audience: the deepseek V4 Flash orchestrator session resuming at API reset.
Retirement: delete this file when Part 2 is empty. It is a queue, not a record.

> Read Part 0 and Part 3 before touching anything. Part 0 tells you which
> documents in this repo are lying to you. Part 3 tells you what ends the run.

---

## Part 0 -- State of record

Every line below was obtained by running a command in the audit session on
2026-08-28/29. Nothing here is quoted from a plan document.

### Verified true

| Fact | Command that proved it |
|---|---|
| `main` is green at `9ed3a28d`; PR #234 merged 36/36 checks SUCCESS, nothing bypassed | `gh pr view 234 --json statusCheckRollup`; `gh run view 33228663895` |
| CI on `main` has 18 consecutive successes, 0 failures, 2 cancelled | `gh run list --workflow=ci.yml --branch main --limit 20` |
| `v0.4.0-rc.1` tag exists at `134e06d2`; **no GitHub Release object exists for it** | `git tag -l`; `gh release list` -- latest release is still v0.1.9, March 2026 |
| All four signing secrets ARE configured, since 2026-08-15 | `gh secret list -R Sovereign-Communication/SCMessenger` |
| The release pipeline fails at `:app:packageRelease` with `KeytoolException: No key with alias '***' found in keystore` | `gh run view 32817839477 --log-failed` |
| `create-release` is `skipped` because `build-android` failed. Every other release job passed: 4 CLI targets, WASM, version verify | `gh run view 32817839477 --json jobs` |
| `README.md` is 4,309 bytes, substantive, honest about the missing audit | `wc -c README.md` |
| AWS node N4 is live | `curl http://54.226.67.101:9876/health` -> `HTTP 200` |
| Only 4 checks are required on `main`: Repository Hygiene, Lint, Rust Linting, Test (ubuntu-latest) | `gh api repos/.../branches/main/protection` |
| The forgery-test CI gate (`security-regression-tests.yml`) is **NOT on main** -- it lives only in red draft PR #228 | `git ls-tree -r main --name-only` |
| PR #235 carries the Rule-8 adversarial review of the merged #234 tree, verdict APPROVE | `gh pr view 235 --json files`; `git show c1c99a8e:docs/security/...` |
| Disk: C: is 97% full, 8.0 GB free, across 30+ worktrees | `df -h /c`; `git worktree list` |

### Verified false -- do not repeat these claims

- **"Release signing secrets are unconfigured."** They are configured. The
  failure is an alias-value mismatch, not a missing secret. Two audit passes
  reported the wrong version of this; only the CI log settles it.
- **"README.md is 0 bytes"** (`SHIP_PLAN.md` S2-1). It is 4,309 bytes and done.
- **"#221/#222 are open and DRAFT, blocking the tag"** (SHIP_PLAN CP1). Both
  merged.
- **"PR #139 is open, gated on G1-G6 twice"** (`TRACKING_PRE_V040_TAG_WORK.md`,
  `_QUEUE.md`). Merged 2026-08-17. D5 is satisfied.
- **"SCMessenger is on release line v0.3.5"** (`docs/CURRENT_STATE.md`).
  `Cargo.toml` reads `0.4.0`.
- **"Docker Integration Suite is a long-standing amber lane"** (SHIP_PLAN S1-4).
  It passed on the last two pushes. PR #156 proposes marking it non-blocking and
  may now be moot.

### Open question -- assigned as L3-1

PR #235 is `main` + exactly one markdown file (parent is `9ed3a28d`; verified it
contains `cd2375b7`, `c334bd8a`, `b6c35c89`). Its `Test (ubuntu-latest)` FAILED
where the identical job on the identical code passed on `main`. The specific
failing test is **not known** -- the run was still in progress and GitHub
withholds job logs until a run completes. Do not guess it. Fetch the log.

### D1-D7 scoreboard

| # | Criterion | State | The one thing standing in the way |
|---|---|---|---|
| D1 | main green | **[OK]** | Nothing. Keep it that way. |
| D2 | Signed APK downloadable | **[BLOCKED]** | `SCMESSENGER_KEY_ALIAS` mismatch. Operator-only fix (Part 1.1). |
| D3 | README explains product + install | **[OK]** | Nothing. SHIP_PLAN's claim otherwise is stale. |
| D4 | Two-device message + receipt | **[PARTIAL]** | Convergence proven on a 3-node dev-build soak (UNIFICATION_V3, merged). Not yet scored on a **released APK**, cross-network. |
| D5 | No long-lived integration branch | **[OK]** | #139 merged. Branch sprawl (212 refs) is hygiene, not D5. |
| D6 | Transport racing | **[BLOCKED]** | `routing_peer_seen` (`iron_core.rs:2571`) has **zero callers**. Routing confidence is stuck at 0.0 fleet-wide, so "fallback selected a working path" is unprovable by construction. Fix is draft PR #215. |
| D7 | Offline proximity | **[NOT STARTED]** | Sequenced after D4/D6. |

**Two blockers above are not named anywhere in SHIP_PLAN**: `routing_peer_seen`
(D6) and `P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT` (~half of advertised listen
addresses cannot bind, so they are silently unreachable -- degrades D4 and
invalidates D6 fallback claims). That is a gap in the canonical plan, not
backlog noise. Task L4-3 closes it.

### OPERATOR RULING 2026-08-29 -- read this before you prioritise anything

> **v0.4.0 gate:** the AWS/Ubuntu node performs **full-parity store-and-forward
> / relay capability + direct connection assistance for peers.**
> **v0.5.0:** the same, plus iOS/macOS, for full five-node parity.

The capability is **already implemented and ungated**: `relay_server` and
`dcutr` are constructed unconditionally
(`core/src/transport/behaviour.rs:521,348`); the `headless` flag changes only
the identify agent_version string (`:508`). Custody lives in
`core/src/store/relay_custody.rs`. So this gate is **prove it and fix what the
proof breaks**, not build it.

Consequence: the three defects below stop being backlog and become the gate.
Lane 0 supersedes Lane 1 for priority.

---

## Part 1 -- Work only a human can do

Do not attempt these. Do not work around them. If they are not done, say so
plainly in your status and proceed with Part 2.

**1.1 -- Operator, ~2 minutes. This unblocks D2, D3-install, D4, D6, D7.**

```bash
keytool -list -v -keystore scmessenger-release.jks | grep -i "alias name"
```

Compare to the `SCMESSENGER_KEY_ALIAS` secret **including case**.
`docs/ANDROID_RELEASE_SIGNING.md` documents `-storetype JKS` and alias
`scmessenger`, but modern `keytool` writes PKCS12, and PKCS12 alias lookup is
case-sensitive where JKS was not. That is the most probable mismatch. Then:

```bash
gh secret set SCMESSENGER_KEY_ALIAS -R Sovereign-Communication/SCMessenger
```

**1.2 -- Board decision, money.** Publishing the release (not tagging it) gates
on the external crypto audit being **COMMISSIONED** -- firm named, scope written,
price agreed, dates set -- per the standing ruling in `HANDOFF/CTO_STATE.md`
section 0-2026-08-23d. Commissioning is the gate; completion is not. Nothing an
agent does moves this.

**1.3 -- Operator ruling.** `HANDOFF/todo/P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT_2026-08-10.md`
offers options a/b/c and needs an architecture decision. Task L5-1 prepares the
brief; the operator rules.

**1.4 -- Operator + hardware.** The D4/D6/D7 demos: second Android handset,
cross-network cellular+WiFi, released APK. Not a software task.

---

## Part 2 -- The deepseek queue

Lanes are independent and may run concurrently **except** where they need a
build; builds serialize (Part 3). Within a lane, order is strict.

### Lane 0 -- Cloud-node parity (THE v0.4.0 GATE, per the 2026-08-29 ruling)

Run this lane first. Everything here is agent-doable except L0-4.

- **L0-1.** **Redeploy the AWS node at the current `main` SHA.** Docker Publish
  succeeded on the #234 merge (run `33228663826`), so a fresh
  `testbotz/scmessenger:latest` carrying V2+V3 exists now. The live node was
  last recorded on `9f54b107` (`gpt-pr139-receipt-filter-20260811`), which
  **predates every V2/V3 convergence fix**. Use `scripts/aws_deploy.sh`.
  Identity persists at `/opt/scm-relay-data`; docker needs sudo (passwordless).
  **Any custody measurement taken before this is measuring the wrong build.**
  Evidence: the node reporting the deployed SHA, read on the box -- `/health`
  returns 200 but exposes no version, so HTTP cannot confirm it.

- **L0-2.** **Prove store-and-forward custody, receiver-side.** Recipient
  offline -> sender sends -> cloud node takes custody -> recipient comes back
  -> message arrives. Score on receiver decrypt + durable history + receipt.
  **Transport ACKs do not count.** This is the exact scenario
  `HANDOFF/in_progress/ANDROID_RELAY_INBOUND_EVIDENCE_2026-08-10_CELLULAR.md`
  caught failing: the node reported success in 264ms while no circuit to the
  destination had existed 20 seconds earlier. Treat that file as the
  regression case, and re-run it first.

- **L0-3.** **Prove direct connection assistance.** Two peers that cannot
  reach each other directly connect with the cloud node's help
  (`relay_client` circuit, then `dcutr` upgrade to direct). Evidence: a
  dcutr upgrade observed in logs on both sides, plus a delivered message.

- **L0-4.** **Operator ruling required** on
  `P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT` (options a/b/c). Same port advertised
  for TCP and WS, only one binds, so roughly half the cloud node's advertised
  addresses are unreachable -- peers cannot reach the assistor. Prepare the
  brief (task L5-1); do not decide it.

- **L0-5.** PR #215 (`routing_peer_seen`) is on this gate, not merely on D6:
  confidence pinned at 0.0 degrades assisted path selection. Execute via
  Lane 2, which carries the mandatory Rule-8 review.

### Lane 1 -- Release pipeline (was highest value; now second to Lane 0)

- **L1-1.** Add a fail-fast alias preflight to `.github/workflows/release.yml`,
  immediately after `Decode release keystore` and before the Gradle step. Today
  the alias mismatch surfaces 24 minutes into `packageRelease`; this makes it
  fail in seconds. Must not print the alias or password.

  ```bash
  keytool -list -keystore android/app/release.keystore \
    -storepass "$SCMESSENGER_KEYSTORE_PASSWORD" \
    -alias "$SCMESSENGER_KEY_ALIAS" >/dev/null 2>&1 \
    || { echo "[FAIL] SCMESSENGER_KEY_ALIAS is not present in the decoded keystore"; exit 1; }
  ```

- **L1-2.** After the operator confirms 1.1, rehearse the entire signed build
  **without burning a tag**:

  ```bash
  gh workflow run release.yml -R Sovereign-Communication/SCMessenger -f artifacts_only=true
  ```

  Verified: `build-android` has no tag requirement and no `artifacts_only` gate,
  so it performs the full signed AAB + APK build and uploads them as workflow
  artifacts; only `create-release` is skipped. This is a zero-risk dress
  rehearsal. **Do this before any re-tag.** Evidence: the run URL plus the
  artifact listing showing a non-zero `.apk`.

- **L1-3.** Only after L1-2 is green, prepare (do not execute) the tag proposal:
  which SHA, `v0.4.0-rc.2` vs promoting to final `v0.4.0`. Note for the
  proposal: `verify_versions.sh` passes for a final `v0.4.0` tag (Cargo is
  `0.4.0`), and `release.yml` marks any tag containing `rc`/`alpha`/`beta` as a
  **draft** release. A draft is not a public download. The operator cuts the tag.

### Lane 2 -- D6 unblock (the highest-value code change left)

- **L2-1.** PR #215 (`routing_peer_seen`). Rebase onto green `main`, resolve the
  canonicalization test assertions that fail on its stale base.
- **L2-2.** It touches `core/src/routing/`. **Mandatory Rule-8 adversarial
  review before integration** -- a fresh reviewer that did not author the change.
  No merge without a recorded APPROVE.
- **L2-3.** Merge only when every check is green.

### Lane 3 -- PR queue burn-down (volume work; ideal use of an unlimited lane)

- **L3-1.** Fetch the completed log for run `33228698078` and identify the actual
  failing test on PR #235. Root-cause it. #235 carries the Rule-8 APPROVE that is
  publish-gate item 2 -- it is not a docs chore. Then merge it green.
- **L3-2.** One workflow fix unblocks two PRs: the Android Wiring Gate step fails
  with `ModuleNotFoundError: No module named 'scripts'` on both #220 and #216.
  Fix `PYTHONPATH` in the workflow.
- **L3-3.** Merge the four green, workflow-only dependabot PRs after branch
  update: **#214, #212, #211, #141**. Zero build surface.
- **L3-4.** Close the five superseded checkpoint/dispatch docs PRs: **#223, #224,
  #225, #205, #206**. State the supersession reason in each close comment.
- **L3-5.** Rebase **#227** and **#209** -- both already pass checks, both merely
  `DIRTY`.
- **L3-6.** Re-run **#108, #107, #106, #103** now that #234's fmt fix is on main,
  then re-judge. Leave **#213, #210** deferred -- genuine Kotlin/AGP toolchain
  floor, tracked in `HANDOFF/todo/DEPENDENCY_DEBT_TOOLCHAIN_UPGRADE_2026-08-28.md`.
- **L3-7.** Decide **#156** (Docker suite non-blocking). The lane now passes; the
  premise may be dead. Recommend close-as-moot unless evidence says otherwise.

### Lane 4 -- Make the backlog stop lying

The audit had to re-derive from source what these files already knew. The next
reader should not have to.

- **L4-1.** Move to `HANDOFF/done/` **with an evidence citation on each**:
  `P0_ANDROID_SELF_RATCHET_RESET`, both `P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE*`,
  `RECEIPT_MARKER_ID_FLAVOR_MISMATCH`, `RCA_DELIVERY_ACK_IMPLEMENTATION_PLAN`,
  `P0_DEEPLINK_PARSES_BUT_NEVER_DIALS` (verified resolved in source:
  `MainViewModel.kt:374` -> `connectToPeer`, route registered at `MeshApp.kt:412`).
- **L4-2.** Correct the six false claims listed in Part 0 at their source files.
  Smallest possible diffs.
- **L4-3.** Amend `SHIP_PLAN.md`: add `routing_peer_seen` and dual-bind as named
  S-items, add the 2026-08-29 cloud-node-parity ruling as the v0.4.0 gate, and
  fix the CP1/S2-1 stale rows. This is the amendment that lets this charter be
  deleted.

- **L4-4.** **`scripts/docs_sync_check.sh` FAILS on `main` today**, for a cause
  unrelated to any current change: a broken markdown link in
  `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` pointing at
  `android/.../DiagnosticsBundleFormatterTest.kt`, a file **deleted in
  `149d3725`** with no surviving equivalent (`git log --diff-filter=D`).
  This matters beyond tidiness: `docs-sync` is a mandatory step in the
  `finalize-checklist`, so **every agent's finalize gate currently fails**,
  which trains agents to ignore or bypass it.
  Not a mechanical fix -- the link was evidence backing a row in a *residual
  risk register*, and deleting the reference could silently weaken a risk
  claim. Read the row, decide whether the mitigation still holds, and fix the
  register accordingly. Escalate if the risk row no longer has evidence.

### Lane 5 -- Prepare operator decisions (do not decide)

- **L5-1.** Write the dual-bind decision brief: options a/b/c, blast radius, cost
  of each, your recommendation, and the one command that would confirm the fix.
  One page. Operator rules.

### Lane 6 -- Operational risk

- **L6-1.** C: is at 97% with 8.0 GB free. A build that dies on disk looks
  exactly like a code failure and has already cost this project time. Inventory
  the 30+ worktrees, identify dead ones, and propose a prune list.
  **Propose only** -- deletion needs operator approval, and several worktrees
  hold other sessions' uncommitted work.
- **L6-2.** Branch inventory: 212 remote refs, 18 provably merged. Note that
  `--merged` undercounts squash-merged branches. Propose; do not delete.

---

## Part 3 -- Hard stops

Any one of these ends the action, not the run. Report and continue elsewhere.

1. **Never run two build tools at once.** Check first; another session's cargo or
   Gradle build may be live. As of this charter's writing, a
   `cargo build --release -p scmessenger-cli` was running in `tmp/cli-build`.
2. **No merge unless every check is green.** No thresholds, no "just the required
   four", no exceptions.
3. **No merge touching `core/src/{crypto,transport,routing,privacy}` without a
   fresh Rule-8 adversarial review returning APPROVE**, from a reviewer that did
   not author the change. This applies to Lane 2.
4. **No tag, no release publish, no `gh secret set`, no branch or worktree
   deletion, no force-push.** Propose; the operator executes.
5. **This checkout is shared.** Never revert, stash, delete, or commit a file you
   did not create. A clean `git status` is not a goal. Use your own worktree.
6. **No new plan documents.** The repo holds ~1,695 markdown files against
   ~120k lines of Rust. Amend in place. Agents are measured in commits merged to
   a green main, not documents produced.
7. **Nothing from SHIP_PLAN section 4** -- no v0.5.0, PQC-14, farm drills, KMP,
   iOS parity, or new orchestration tooling. Prep tickets already exist for
   several of these; they are not permission to start.

---

## Part 4 -- Evidence contract

A worker reporting "gate passed" is a claim, not evidence. This project has been
burned by a fabricated health report for a node that was down, and by a "verified"
claim that came from a grep.

Every status line you produce carries one of:

- the exact command and its output, or
- a GitHub Actions run URL, or
- `UNVERIFIED` -- which is an acceptable and useful answer.

Specifically: do not describe a file, commit, PR, or run from memory or from its
filename. Run the command that shows it. Your own earlier statements are claims,
not facts.

For anything about a branch, git is authoritative and the GitHub API is a
fallback that must announce itself: `git rev-list --count`, `git diff --name-only`.
An API page size of 100 is a ceiling, not a count.

Print all of it. No `head -N` on an evidence list. Express lower confidence by
printing more, never less.

---

## Part 5 -- What success looks like in this window

Not a tag. The tag needs Part 1, and Part 1 needs humans.

Success, in priority order under the 2026-08-29 ruling:

1. **The AWS/Ubuntu node redeployed at the current `main` SHA**, and custody +
   connection assistance measured against it -- with the cellular-evidence
   regression case re-run. Whatever that measurement breaks becomes the ticket.
2. `main` green and carrying the routing fix (PR #215, Rule-8 reviewed).
3. A **proven** signed-APK build from a dress-rehearsal run -- no tag burned.
4. PR queue down from 28 to under 10; backlog that no longer contradicts the
   source; SHIP_PLAN amended so the next session does not re-derive any of this.

That leaves exactly three things between SCMessenger and a public v0.4.0: one
secret value, one procurement decision, and one afternoon with two phones and a
cloud node that has been proven to carry the mail.
