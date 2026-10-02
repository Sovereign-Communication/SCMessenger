# V040/V050 merge train + unification: land everything, prove the mesh on three nodes

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** IN PROGRESS -- Phase 0 and T-1..T-4 are done; the merge train executes under section 14; live state is the tracker PR #403; amended 2026-09-29 (see 14.6)
**Priority:** P0 (the v0.4.0 tag path)
**Lane:** Freebuff. Any lane may execute within its own limits (0.2).
**Created:** 2026-09-27
**Owner:** operator

**Scope -- may touch:**
- new branches named `freebuff/train-*`, and the PRs it opens from them
- reports in `HANDOFF/freebuff/inbox/`
- state outside the repo in `C:/Users/SCM/Documents/GitHub/scm-train-state/` (called STATE_DIR below)
- evidence staging in the train worktree's `tmp/evidence/`

**Scope -- must NOT touch:**
- other lanes' branches or PR heads (supersede them with a new PR instead)
- any worktree it did not create
- the primary checkout (`C:/Users/SCM/Documents/GitHub/SCMessenger`, branch `glm/canonical-outlier-audit` = PR #359 head, carries other sessions' uncommitted work)
- `vendor/`, except by taking one whole reviewed tree (MT-05)
- the Pixel's UI

**Supersedes for sequencing:** `HANDOFF/freebuff/V040_0_LANE_UNIFY_MERGE_TRAIN_2026-09-23.md` and the 2026-09-01..09-20 V040_* status/train/dispatch files in this `queue/`. T-4 carries forward any task they hold that this file lacks.

**Sources:**
- the Harness hourglass context pull of 2026-09-27 (sections 1-4, P1-P10)
- direct verification in the Opus planning session with `gh`/`git`: the open-PR list; that `origin/main` has no `vendor/` tree and no vendor lines in `docker/Dockerfile`

**Estimates:** lines of code (LoC) only.
- "measured" = GitHub PR additions/deletions, or `git merge-tree` LAND_LOC against origin/main, on 2026-09-27.
- "est" = judgment. Replace it with the measured number when the car runs.

---

## 0. Rules of the road

**0.1 Read order.**
1. CLAUDE.md
2. AGENTS.md (FREEBUFF LANE and hard rules)
3. docs/rules/FREEBUFF.md
4. HANDOFF/freebuff/inbox/README.md
5. sections 0-3 of this file
6. docs/rules/BUILD_AND_CI.md
7. docs/rules/SECURITY_PROTOCOL.md
8. docs/rules/ANDROID.md, plus docs/runbooks/CI_APK_TO_PHONE.md (Android and TRI cars only)
9. the Freebuff-desktop setup runbook for the JEV tools, `docs/runbooks/HARNESS_*_SETUP.md`

Read the later sections of this file one at a time, when you reach them. It is long; never load it all at once.

Precedence: those documents win on HOW (format, reporting, tooling, safety). This file wins on WHAT and ORDER.

**0.2 Lane limits** (AGENTS.md FREEBUFF LANE, FREEBUFF.md; verbatim rules summarized).
- Freebuff MAY:
  - run cargo/gradlew, after checking that no other build is live;
  - commit its own task's files to a scoped branch and push it (fast-forward only);
  - open PRs for its own work;
  - run `adb install -r` and passively pull logs (`adb logcat`; its own app files through `run-as`);
  - call the JEV tools (jev_phase, log_judgment, issue_sort) over MCP.
- CI is the primary build verifier for this lane (operator directive 2026-09-22).
- Freebuff may NOT:
  - give a Rule-8 sign-off on its own work (a fresh Claude reviewer signs, section 14 A2);
  - drive the Pixel (no input, tap, `am start`, force-stop, or UI reading);
  - revert, stash, delete or commit a file it did not create, except through BK-01's tool.
- **Operator grants 2026-09-28** (section 14):
  - merging PRs under A1
  - AWS ssh under A6
  - tagging and publishing under A10
  - closing superseded PRs under A4
- Superseding PRs carry the original authors' commits unchanged: merge, never rewrite. Each links the PR it supersedes. After the new one merges, the lane closes the old PR with evidence (A4).

**0.3 Gates.** Section 14 lists what is pre-approved. `[OPERATOR GATE]` now applies only to anything outside those approvals, to the hard stops in 14.5, and to physical Pixel actions. For those:
1. Write an inbox report (0.6).
2. Print the GATE block.
3. Wait for the operator to type `GO <id>`, `MERGED <pr>`, `SKIP <id>` or `STOP`.

One `GO <group>` (for example `GO MT-02`) approves every gate inside that group only.

**0.4 Never:**
- two build tools at once. Check `tasklist | findstr /I "cargo.exe rustc.exe java.exe gradle"` first.
- a local build while `python scripts/disk_budget.py --fast` exits 2
- `cargo clean --target`
- `git stash`, `reset --hard`, `checkout -- <path>` or `clean` in a worktree you did not create
- `git commit -a` (stage explicit paths)
- force-push, `--admin`, or cancelling required checks on the SHA about to merge
- emojis (use [OK] [FAIL] [WARNING] [INFO])

**0.5 Shell hygiene:**
- Run `git add`, `git commit` and `git push` as separate commands. Chained, they hang.
- Never read `$?` after a pipe.
- In Git Bash, `git show <rev>:<path>` needs `MSYS_NO_PATHCONV=1`.
- CI queue hygiene: after a merge, cancel superseded runs, and batch pushes.

**0.6 Report formats.**

Inbox report, per HANDOFF/freebuff/inbox/README.md. File name `<CAR-ID>_<what>_<YYYY-MM-DD>.md`. Body:
```
Task: V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md / <car id>
Type: QUESTION | BLOCKED | PREMISE-WRONG | DONE
<evidence: the exact command and its output, a run URL, or the word UNVERIFIED>
```

REPORT block, printed after every car:
```
[CAR] <id> <title> | [OK] DONE / [WARNING] PARTIAL / [FAIL] BLOCKED
[PR] #<n> <state> merge <sha or ->   supersedes: #<n>, ...
[LOC] measured +<a>/-<d> (<files> files); vendored excluded: +<n>
[GATES] CI <run url> <conclusion> | JEV <score>/100 hard gates <ok|list> | Rule-8 <reviewer + SHA | n/a>
[NEXT] <car> (deps met: yes/no)
```

GATE block:
```
[OPERATOR GATE] <id>: <exact action needed from the operator>
[WHY] <one line> | [BLAST RADIUS] <what changes; how to undo>
Reply: GO <id> | MERGED <pr> | SKIP <id> | STOP
```

**0.7 Evidence.** Stage evidence in the train worktree under `tmp/evidence/<date>/<car>/`. Record it into a committed HANDOFF file before you delete anything. tmp/ is not a store (CI_APK_TO_PHONE.md).

---

## 1. Ground truth (2026-09-27)

**main:**
- HEAD 1dc70f0b (merge of #391: wasm workspace membership; this restores the Dependabot cargo updater).
- The last push was green on CI, Lint, CodeQL, Docker Publish, Docker Integration Suite and Repository Hygiene.
- The scheduled **Security Scan failed**: run 36292343654, 2026-09-27T03:45Z.
- `docs/jev-roadmap.md` does not exist. SCM work goes through the documented `--evidence` override surface (JEV_DOGFOOD_RUN_2026-09-22.md).

**Open PRs: 57** (#103-#402). The 204 unmerged remote branches triage as:

| Class | Count |
|---|---|
| CLEAN | 71 |
| CONFLICT | 116 |
| CONTAINED | 17 |

**Duplicate fixes in open PRs:**
- #361 and #359 both carry the two-tier per-peer connection cap (`core/src/transport/per_peer_cap.rs`).
- #364 and #367 both serialize Android stop/start. They are independent rewrites of `MeshForegroundService.kt`, and both add `StopStartFloodTest.kt`.
- #359, #361, #364 and #372 are one lineage, sharing 29-35 files per pair.

**D9 (vendored libp2p-swarm 0.48.0 degrade):**
- Deployed live on all three CLI nodes since 2026-09-22/23, with zero panics since. Merged nowhere.
- Vendor trees differ between PRs:
  - #361 and #364 share tree b6f5962c.
  - #372 has c0b1fba5, which adds the `warn!` level and a policy comment block.
- Only #364 changes `docker/Dockerfile` (DOCKER-VENDOR-001). #361 and #372 would break the Docker build on their own.

**Identity:** main's `derive_public_key_from_peer_id` (`core/src/store/contacts.rs:491-529`) binds a public key without any Ed25519 curve check on two of its three paths. One of those is the "last 32 bytes of any base58 string" fallback. PR #383 (WP1) closes it.

**Nodes** (2026-09-24 KEEP dispositions; these are the rollback targets):

| Node | Build | Identity / peer | Access |
|---|---|---|---|
| Windows CLI | LIVE `bceacb94` (2026-09-26 rollout), `C:/Users/SCM/.local/bin/scmessenger-cli.exe`, sha256 3a1ac11f2ee1c163ac6486e2e9f24d8b5d352aeb93dc78645ec74a5e32af7fe9. Rollback: `1bc78c85`, `tmp/radio-candidates/1bc78c85/scmessenger-cli.exe`, sha256 454cc346811f30a51e4a42bfb7f51f7daf458313f82ad7a5d2a8477de1fab061 | 985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826 / 12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw | local; control API 127.0.0.1:9876; supervised by `scripts/run_node_supervised.ps1` (restart on crash; supervisor log and node logs in `C:/Users/SCM/AppData/Local/scmessenger/logs/`); relaunch method in 7.2 |
| AWS (always-on) | LIVE `bceacb94` (per `/version`, 2026-09-28); image tag by the aws_deploy convention `testbotz/scmessenger:sha-bceacb9` (confirm with `docker inspect` over ssh). Rollback: `sha-45b0f8b` (id sha256:5403c7f98e13...). Container `scm-node`, mount `/opt/scm-relay-data:/data` | 37eb75612c179d57a83801ba71f3545ffdf3d009adbf73c652dae37536b1d006 / 12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31 | ssh `ec2-user`, key `~/.ssh/scm-node-key.pem`; host by discovery (scripts/aws_deploy.sh; last known 18.234.62.247); API :9876 |
| Pixel 6a | the CI APK from 2026-09-23 | **identity WIPED**; the old triad 779e9ea3... is still a contact on Windows | wireless adb; package `com.scmessenger.android` |
| OpenClaw cloud node | v040d9degrade-672dffcb (OC lane owns it) | 821f161c... / 12D3KooWDgLQ8jn... | not driven by this train |

**0.4.0 tag checklist** (V040_CTO_MASTER_PLAN_2026-09-20.md section 4): 10 items, none checked.

**SHIP_PLAN release gates** (scoreboard dated 2026-08-31):

| Gate | Criterion | State |
|---|---|---|
| D2 | Signed APK downloadable | BLOCKED (keystore alias) |
| D4 | Two-device message + receipt | PARTIAL |
| D6 | Transport racing | BLOCKED (no routing-feed call site) |
| D7 | Offline proximity | NOT STARTED |

**Worktrees (22)** and their train disposition:

| Worktree | Branch | PR | Behind/ahead | Uncommitted lines | Disposition |
|---|---|---|---|---|---|
| SCMessenger (primary) | glm/canonical-outlier-audit | #359 | 150/24 | 77 | P0-2 rescue; the WIP maps to #402 + #364 sweep + WP1 helper + pre-commit disk warning + staged vendor |
| tmp/android-artifact-main-wt | freebuff/android-third-node-artifact-main-20260924 | #367 | 15/1 | 0 | MT-06 |
| tmp/android-artifact-wt | freebuff/android-third-node-artifact-20260924 | - | 150/25 | 0 | CONFLICT 48 docs; archive-proposed after MT-08 |
| tmp/fix-361 | fix/361-review-blockers | #372 | 2/48 | 0 | MT-05 |
| tmp/harness-plan-snapshot-20260924 | recovery/harness-plan-snapshot-20260924 | #369 draft | 15/2 | 1 | T-4 reconcile |
| tmp/jev-completion-gate-20260924 | orchestrate/jev-completion-gate | #396 | 15/3 | 0 | MT-00a |
| tmp/repro-candidate, -2, -3, -4, -5 (+ -clean each), tmp/repro-clean | detached | - | - | 1 / 35 / 37 / 37 / 37 / 34 (clean: 0) | P0-2 / P0-3; retire in U-1 |
| tmp/swa-20260925 | safe/work-ahead-20260925 (not on origin) | - | ? | 13 | P0-2, P0-3 |
| SCMessenger-v040-harness-plan | freebuff/v040-v050-harness-plan (not on origin) | - | ? | 18 | P0-2, P0-3; T-4 reconcile |
| wt-cargo-wasm | fix/cargo-wasm-workspace-members | - | CONTAINED (#391) | 0 | retire in U-1 |
| wt-d9-degrade | d9-libp2p-degrade | #361 | 17/30 | 0 | MT-05 (superseded) |
| wt-ss-outbox-040 | freebuff/and-stopflood-outbox-sweep-040 | #364 | 17/33 | 0 | MT-05 Dockerfile hunk; MT-07 |
| wt-wp1 | freebuff/wp1-identity-unification | #383 | 7/8 | 0 | MT-03 |
| wt-train-plan | docs/merge-train-v040-v050-20260927 | this file | 0/1 | 0 | MT-00b lands this file |

---

## 2. Gates every car must pass

**2.1 Build (CI-primary; BUILD_AND_CI.md CI-Primary Build Doctrine).**
- The required order is commit -> push -> CI -> reclaim.
- A local build is a failover only. Before one, check that no other build is live, and that `python scripts/disk_budget.py --fast` does not exit 2.
- Compile gate before calling code complete: `cargo test --workspace --no-run -j12` (locally if allowed, else the CI job that runs it).
- Android cars also run the ANDROID.md pre-merge checklist.
- Docs cars also run `scripts/docs_sync_check.sh`.

**2.2 CI.**
- Every required check is green on the PR head.
- After merge, the push run on main is green.
- Read failures with `gh run view <id> --log-failed`. Never guess.

**2.3 JEV completion gate** (all three parts, every car):
- a. **Keyed JEV evidence gate from #396,** after MT-00a lands: the keyed JEV completion evidence and exact-SHA control-plane gate that #396 adds must pass for the car's merge SHA.
- b. **Harness bar through the documented `--evidence` override surface:**
  - Use the admitted SCM-local harness (BUILD_AND_CI.md "Harness admission and update gate": production is the immutable tag v0.4.1, installed by `scripts/update_local_harness.py`; the canary is never used).
  - MCP: `jev_phase {repo_root, phase: "<car>", min_score: 85}`.
  - CLI: `harness jev-phase --local-only --repo-root <train worktree> --phase <car> --evidence STATE_DIR/jev/<car>.evidence.json --min-score 85 --out STATE_DIR/jev/<car>.json`.
  - The evidence JSON (Appendix B) holds only values captured from commands.
  - If the admitted harness has no `jev-phase`, record `UNVERIFIED-JEV` and rely on parts a and c. Report it as a QUESTION.
- c. **WP cars (MT-03):** `python scripts/jev_canonical_check.py --wp WPn` must return is_passing at the 0.70 minimum confidence. An unkeyed fallback is `UNVERIFIED-JEV`, which is not done.
- d. **Runtime evidence (TRI):** `log_judgment` with the frozen pack `scm-ops-log-v1` (Appendix C), after the operator freezes it.

**2.4 Rule-8 adversarial review** (SECURITY_PROTOCOL.md).
- **Mandatory** for any diff in `core/src/crypto/`, `core/src/transport/` (BLE, relay, QUIC paths), `core/src/routing/` or `core/src/privacy/`, and for the vendored libp2p-swarm patch.
- The reviewer:
  - must not have authored, proposed or specified the fix;
  - cannot be replaced by a multi-model panel;
  - signs a verdict that covers only the head SHA reviewed. Any later commit touching the gated modules voids it.
- Freebuff never signs. It writes a QUESTION inbox report naming the PR and head SHA, and the operator dispatches an uninvolved reviewer.
- Review is also **recommended** (identity binding) for #383 and #400.

**2.5 Delivery evidence** (any runtime claim). A message is delivered only with all five of these:

| # | Evidence |
|---|---|
| 1 | the send API's final status |
| 2 | sender history `direction=sent, delivered=true` |
| 3 | receiver history `direction=received` |
| 4 | the log ACK / `receipt_outbox_cleared` lines |
| 5 | `outbox_count` back to 0 on both sides |

This is the 2026-09-24 ROLLOUT_RCA method. Transport ACKs alone never count.

---

## 3. Operator prerequisites and rulings (human-only)

**Superseded where they differ by section 14:** the operator interview of 2026-09-28 answered H-1..H-11, D-01, R-1 and R-2, and granted standing approvals. Read section 14 first.

The executor checks these, records their state in TRAIN_STATE, and raises a QUESTION when a car depends on one that is not done.

| Id | What | Unblocks |
|---|---|---|
| H-1 | Regenerate the release keystore and fix SCMESSENGER_KEY_ALIAS (SHIP_PLAN G1; no APK was ever published, so there is no lineage to preserve) | D2; the release artifact |
| H-2 | Pin the debug keystore secret in CI (S1-1) so a CI APK `install -r` does not demand a wipe-install | TRI Pixel install without identity loss |
| H-3 | Pixel identity: restore it from backup or onboard fresh, then never wipe it again | TRI-040 |
| H-4 | D10 ruling: is the Windows node LAN-first or internet-reachable? | MT-09e |
| H-5 | Dispatch an uninvolved Rule-8 reviewer for each gated car | MT-02 (#399), MT-04, MT-05, MT-07, MT-09 items, MT-10 |
| H-6 | Grant Freebuff ssh to the AWS node for deploys, log pulls and TRI sends. Record the grant in FREEBUFF.md through MT-00b. | D5/D6 part 1, TRI |
| H-7 | Commission the external crypto audit, or record a waiver (SHIP_PLAN G4-2) | TAG-040 |
| H-8 | SEC-03: dated accept, or a migration branch started with an owner | TAG-040 |
| H-9 | AND-06 A1+A2, or a dated waiver (MT-09j is the code half) | TAG-040 |
| H-10 | Branch protection: strict:true serializes merges and multiplies CI across 57 PRs (issue I-31). Rule: keep strict, or use a merge queue. | train throughput |
| H-11 | All merges, the tag, and publishing the release | everything |

| Ruling | Question | Recommendation |
|---|---|---|
| D-01 | Land WP1-WP4 (#383, #352, #349, #355, #356) in 0.4.0? | YES. The P0 umbrella says messages are not delivered even on WiFi; that blocks gate D4; and #383 closes the unvalidated key binding above. |
| R-1 | #352 (the WP1 regression battery): its JEV row fails and it needs a ruling | operator |
| R-2 | Include the churn gate (SHIP_PLAN G3-0, AWS with a new IP) in TRI-040? It needs an EC2 stop/start. | operator |

---

## 4. Phase 0 -- Protect (no code changes)

**P0-1 Inventory.**
- Create STATE_DIR.
- Record `git worktree list`, the uncommitted-line count per worktree, the open PRs (`gh pr list --state open --limit 100 --json number,headRefName,mergeable,isDraft`), and the last 8 main runs.
- Write `TRAIN_STATE.md` listing every car in this file as TODO.

**P0-2 Rescue uncommitted work** (`[OPERATOR GATE] GO P0-2` before step c). Worktrees in scope: primary (77 lines), repro-candidate (1), -2 (35), -3 (37), -4 (37), -5 (37), repro-clean (34), swa-20260925 (13), SCMessenger-v040-harness-plan (18), harness-plan-snapshot (1).

a. **Capture, read-only**, into `STATE_DIR/rescue/<wt>/`:
   - `git -C <wt> rev-parse HEAD > base.txt`
   - `git -C <wt> diff --cached --binary > staged.patch`
   - `git -C <wt> diff --binary > unstaged.patch`
   - `git -C <wt> ls-files --others --exclude-standard > untracked.txt`, then copy those files, preserving paths.

b. **Prove the capture is complete:**
   - `git worktree add STATE_DIR/scratch/<wt> <base>`
   - apply both patches; copy the untracked files in
   - `git -C <scratch> status --short | wc -l` must equal the original count.

c. **Commit the copy** in the scratch worktree:
   - branch: `rescue/<wt>-20260927`
   - message: "rescue: uncommitted state of <wt> as of 2026-09-27 (owner unknown; not reviewed)"
   - push.

d. **Never modify the original worktree.** Its owner may still be working in it.

LoC landed: 0. Rescue branches are archives.

**P0-3 Unreachable commits.**
- For the detached HEADs cd459127, 7c78077c, 6447608f, 9e0c11c8 and a015ee5a, run `git branch -a --contains <sha>`.
- If the output is empty: `[OPERATOR GATE]`, then `git branch archive/repro/<wt> <sha>` and push it.
- Also push `safe/work-ahead-20260925` and `freebuff/v040-v050-harness-plan` if `git ls-remote --heads origin <name>` shows they are missing.

**P0-4 Security Scan triage.**
- `gh run view 36292343654 --log-failed` -> record the cause.
- `gh workflow view "Security Scan"` -> does it run on pull_request? If it does and it is red, MT-01 goes first.

**P0-5 Train worktree.**
- `git worktree add C:/Users/SCM/Documents/GitHub/wt-train -b freebuff/train-base-20260927 origin/main`
- Cut every car branch `freebuff/train-<car>` from a freshly fetched `origin/main` inside it.

**P0-6 Harness check.**
- The admitted harness (v0.4.1) resolves, and its JEV tools are reachable over MCP (setup runbook: `docs/runbooks/HARNESS_*_SETUP.md`).
- Check whether `jev_phase` exists in the admitted version. Record the answer.

**P0-7 Access check.**
- `adb devices -l` (H-3 status)
- whether the H-6 ssh grant is recorded
- CI APK signing state (H-2)
- Record all three. Nothing is provoked on the device.

**P0-8 BK-01 -- back up everything to GitHub, then reclaim only what is safe** (operator directive 2026-09-27/28; runs now, while MT-00a waits on CI).
- **Rule:** uncommitted work (WIP) is never removed, reset, cleaned, stashed or switched. It is backed up to GitHub and left in place.
- **Tool:** `scripts/backup_purge.sh` on this branch (v2, SAFE; read its header first). Set `BP=C:/Users/SCM/Documents/GitHub/wt-train-plan/scripts/backup_purge.sh`.
  - `STATE_DIR/backup_purge.sh` is only a stub that forwards to it.
  - Never perform any of its steps by hand.
  - Edit it only to fix a crash. Commit the fix on this branch, and put the diff in the REPORT.
- **Steps** (any cwd; outputs go to `STATE_DIR/backup_purge/`):
  1. `bash "$BP" inventory`
  2. `bash "$BP" backup`
     - It pushes every working state to `refs/heads/backup/20260927/...`.
     - It uploads ignored non-build files to the DRAFT release `backup-local-20260927` (maintainers only).
     - Any `[FAIL]`: stop and report.
  3. `bash "$BP" verify` -- must end with `0 missing`.
  4. `bash "$BP" purge --dry-run` -- read `STATE_DIR/backup_purge/decisions.tsv`.
  5. `bash "$BP" purge`
  6. `bash "$BP" report`
- **Purge may remove only:**
  - clean worktrees that are fully on GitHub and hold no local-only data
  - the lane's scratch copies whose full working state is byte-identical to a `rescue/*` commit on GitHub
  - local branches whose commits are all on GitHub
  - build output, via `scripts/reclaim_safe.py`
- **Purge never touches:**
  - any worktree with WIP (KEEP-WIP)
  - secrets and node/app data (KEEP-LOCAL-DATA)
  - `tmp/` and every other non-build file
  - the kept checkouts: primary, `wt-train`, `wt-train-plan`
  - the running Windows node (it lives outside every checkout)
- **The backup is permanent:** refs under `backup/20260927/` and the draft release `backup-local-20260927` must never be archived or deleted (T-2/T-3).
- **Acceptance:**
  - verify prints `0 missing`
  - `decisions.tsv` has no REMOVE for a worktree with WIP
  - REPORT on #403 with counts only: no secret or vault paths

---

## 5. Phase 1 -- Triage (the Harness pull already ran it on 2026-09-27; re-run before any deletion)

**T-1 Re-run the triage.** Run Appendix A. main moves: #361 and #372 flipped from CLEAN to CONFLICT after #391.

**T-2 Decide each branch.**
- **CONTAINED (17): archive them.** `ci/fix-cargo-deny-pin-2026-09-04`, `cto/android-degraded-storage-wiring-2026-08-22` (its PR #227 is still open: close it as landed), `docs/pr234-security-verdict`, `docs/rule8-review-v040`, `docs/v040-final-status`, `fix/cargo-wasm-workspace-members`, `fix/logging-observability-defaults`, `fix/release-signing-preflight`, `freebuff/v040-t1-half2`, `freebuff/v040-t14-preexisting-fixes`, `freebuff/v040-t8-restore-test`, `gpt/codeql-regex-remediation`, `gpt/ios-test-truth`, `gpt/pr111-safe-device-resolution`, `gpt/security-dom-hardening`, `worktree-agent-1ecfaf2`, `worktree-agent-b4e91f2`.
- **Named in a car** (sections 6 and 10): land it through that car.
- **iOS/macOS/Swift** (`gpt/*ios*`, `gpt/v050-*`, `gpt/apple-*`, `codex/ios-*`, `workahead/ios-*`, PRs #178, #207, #208, #301): MAC lane only (V5-14).
- **Large landing/recovery branches from 2026-09-09..09-14, archive-proposed:**

  | Branch | Conflicts |
  |---|---|
  | unified/v040-3node-parity | 198 |
  | purge-backup/mimo-checkpoint-20260914 | 200 |
  | recovery/scmessenger-dirty-20260911 | 195 |
  | recovery/setup-gates-20260911 | 180 |
  | fix/mimocode-requests-and-v040-land | 177 |
  | cto/t2-disk-ruling-2026-08-31 | 177 |

- **CLEAN or CONFLICT, last commit on or after 2026-09-01, not in a car:** REVIEW. Add one row to `STATE_DIR/REVIEW_QUEUE.md`: ref, class, LAND_LOC, files, commit subjects.
- **Last commit before 2026-09-01, not in a car:** archive-proposed.
- **Refs on remotes other than origin:** list only. Never delete.

**T-3 Archive** (approved, section 14 A3: tags only):
- Re-verify each approved ref's SHA.
- Tag it: `git tag archive/<branch> <sha>`.
- Push each tag by name.
- Delete NO remote branches; the operator ruled keep-branches.
- Never tag or touch `backup/20260927/*` or `rescue/*`.

**T-4 Carry-forward.**
- Read `HANDOFF/freebuff/V040_0_LANE_UNIFY_MERGE_TRAIN_2026-09-23.md`, the queue files it supersedes, `recovery/harness-plan-snapshot-20260924` (#369) and `freebuff/v040-v050-harness-plan`.
- Every task they list that this file lacks goes into ONE inbox QUESTION, as a table.
- Do not merge those branches.

---

## 6. Phase 2 -- v0.4.0 train

**Order:** BK-01 (P0-8, runs while MT-00a waits on CI) -> MT-01 (if the scan gates PRs) -> MT-00a -> MT-00b and MT-02 in parallel -> MT-03 -> MT-06 -> MT-04 -> MT-05 -> MT-07 -> MT-08 -> MT-09 -> MT-10 -> MT-11 -> MT-12 -> AUD-040 (section 14, H-7) -> TRI-040 (section 7) -> TAG-040 (section 8).

**Car pattern** (every car unless it says otherwise):
1. Run the car's "verify" step. If its acceptance already holds on origin/main, write a DONE inbox report with the command output, mark the car DONE-VERIFIED, and move on.
2. A clean existing PR needs no push. Check its gates, then raise a merge GATE.
3. A conflicting PR, or one that must be combined: `git switch -c freebuff/train-<car> origin/main`, then `git merge --no-ff --no-edit origin/<source>`. Resolve only as the car's notes direct; anything else is BLOCKED. Push the branch and open a PR titled "<car>: ... (supersedes #n)".
4. Rule-8 cars: QUESTION for the review (H-5), pinned to the PR head SHA.
5. CI green on required checks -> the lane merges under section 14 A1.
6. JEV gate (2.3) -> REPORT.

| Car | Content | Source | LoC (measured unless est) | Deps | Rule-8 |
|---|---|---|---|---|---|
| MT-00a | CI and JEV gates | #396, #397, #402 | +695/-8 plus #402 (measure it) | P0 | no |
| MT-00b | Doctrine and rules unification (docs only) | doctrine files from origin/fix/361-review-blockers; #386, #388, #357, #376; this file | +1,002/-8, plus est +300-1,500 extraction | MT-00a | no |
| MT-01 | Security Scan fix | P0-4 | est +5-40 | P0-4 | no |
| MT-02 | Seven fresh fixes | #401, #400, #399, #398, #382, #381, #380 | +485/-70 | MT-00a (MT-01 if gating) | #399 YES |
| MT-03 | P0 umbrella WP1-WP4 (D-01) | #383, #352, #349, #355, #356 | +3,291/-179 | MT-02 | recommended (#383) |
| MT-06 | Android lifecycle | #351, #367, then a port from #364 | +1,728/-187, plus est +40-150 | MT-00a | no (android-qa) |
| MT-04 | D6 routing feed on ConnectionEstablished | verify first; origin/freebuff/v040-t4-routing-feed | est +60-120 (branch: +232/-6) | MT-03 | YES |
| MT-05 | D9 + D1 transport, supersedes #361 and #372 | #372 + #364's Dockerfile hunk | own code est +1,500-2,700 (PR: <= +5,074/-36 non-vendor); vendor +13,403 upstream copy, a +204/-45 patch | MT-00b, MT-06 | YES |
| MT-07 | Periodic outbox sweep, the #364 remainder | #364 | <= +3,228/-81; est unique +150-600 | MT-05, MT-06 | YES (swarm.rs) |
| MT-08 | T-CONN-04 dedupe + docs, the #359 remainder | #359 | <= +8,131/-293; est unique +500-3,000 (docs) | MT-05 | if transport code remains |
| MT-09 | 0.4.0 defect tickets and release blockers | tickets | est +590-1,980 (a-k) | MT-05 | per item |
| MT-10 | Triangulation tooling + missing lifecycle log markers | new | est +340-570 | MT-09 | markers in transport/ |
| MT-11 | Housekeeping (SHIP_PLAN G5) | - | ~0 code | any time | no |
| MT-12 | Rules-drift guard | hook | est +30-60 | MT-00b | no |

### MT-00a -- CI and JEV gates

- **Source:**
  - #396 `orchestrate/jev-completion-gate` (CLEAN, +665/-8, 5 files): keyed JEV completion evidence + exact-SHA control-plane gate.
  - #397 `freebuff/mobile-android-gate` (CLEAN, +30).
  - #402 `freebuff/handoff-gate-lane-separation-20260927` (MERGEABLE): the handoff-scope gate. Its CI job calls `scripts/validate_handoff_scope.py`.
- **Status 2026-09-28:**
  - The operator merged #402 (merge commit faf22a57db).
  - #396 and #397 were updated onto main with update-branch; CI is re-running.
  - Before the update, #396's only failure was CodeQL, which is not a required check. Required checks: Repository Hygiene Checks, Lint, Rust Linting, Test (ubuntu-latest), Handoff ownership scope.
  - When both are green, merge them under section 14 A1.
  - After the update: #397 is CLEAN (28/28).
  - #396 fails CodeQL with two high `actions/cache-poisoning/poisonable-step` alerts at `.github/workflows/ci.yml:142/144` (head `2b68b7d6`). A1 blocks the merge until they are fixed or proven false positive, with evidence on the PR. The fix may supersede #396 via a `freebuff/train-mt00a-*` branch.
- **Scope correction:**
  - After these land, every later PR must pass the new jobs.
  - #402 enforces single-owner handoff documents. When a docs PR mixes owners, split it per owner; do not weaken the gate.
  - The primary checkout's uncommitted `ci.yml`/`rules_check.py` edits are a local copy of #402. Ignore them.
- **Acceptance:**
  - All three merged.
  - The first main push after them is green, including the new jobs.
  - #396's gate yields a keyed record for its own merge SHA.

### MT-00b -- Doctrine and rules unification (docs only)

- **Evidence:** BUILD_AND_CI.md existed in 7 variants across worktrees on 2026-09-27. The operator's CI-Primary doctrine and the harness admission gate are on branches, not on main. The #359/#361/#372 lineage carries about 30 doctrine files that main lacks.
- **Source:**
  1. From `origin/fix/361-review-blockers` (the newest of the lineage; its BUILD_AND_CI.md is the section superset of all 7 variants), for each doctrine path: `git checkout origin/fix/361-review-blockers -- <path>`. The paths:
     - AGENTS.md
     - docs/rules/BUILD_AND_CI.md, docs/rules/FREEBUFF.md
     - NODE_MODEL.md, and the docs/runbooks/* files in the lineage
     - SHIP_PLAN.md, HANDOFF/todo/_QUEUE.md
     - the D2/D3/D5_D6/D8/D9 tickets
     - V040_READINESS_3NODE_CONFIRMATION_2026-09-22.md, V050_PHILOSOPHY_AND_BORROW_PLAN.md
     - OC_LANE_AUDIT_2026-09-22.md, OC_DOGFOOD_INCORPORATION_PLAN.md, RETICULUM_AUDIT_2026-09-21.md
     - the JEV dogfood run record
     - HANDOFF/harness/packs/scm-ops-issues-v1.json
  2. Where main has a newer commit on a path (`git log -1 --format=%cs origin/main -- <path>`), merge by hand and keep main-only content.
  3. Also land #386 (+95), #388 (+362), #357 (+277/-8) and #376 (+268).
  4. Add this file. Record the section 14 grants (A1 merges, A6 AWS ssh, A10 tag and publish) in `docs/rules/FREEBUFF.md` and in AGENTS.md's FREEBUFF LANE, as dated operator rulings of 2026-09-28.
- **Why before MT-05:** the transport PRs then carry only code. Rules stop depending on which worktree an agent sits in.
- **Acceptance:**
  - `scripts/docs_sync_check.sh` passes.
  - #402's gate is green.
  - After merge, `git diff --stat origin/main origin/fix/361-review-blockers -- <doctrine paths>` is empty, or lists only main-newer content kept on purpose.

### MT-01 -- Security Scan fix (conditional first)

- `gh run view 36292343654 --log-failed`.
- If it is a cargo-deny advisory, run `cargo update -p <crate> --precise <fixed>`. Never add an ignore; the waived rustls RUSTSEC-2026-0285 is tracked separately in P1_SECURITY_RUSTLS_RUSTSEC_2026_0285_TRACKING.md. If the resolver cascades, make the smallest possible `Cargo.lock` edit.
- **Acceptance:** `gh workflow run "Security Scan" --ref freebuff/train-mt01-security` goes green.

### MT-02 -- Fresh fixes (all CLEAN, 2-9 commits behind main)

| PR | Change | Paths | LoC | Gate |
|---|---|---|---|---|
| #401 | Drain all peer-identity spellings in one pass (the CLI-03 half of the P1 outbox blocker) | core/src/store/outbox.rs | +111/-25 | - |
| #400 | Verify the contact bundle signature before persisting | core/src/store/contacts.rs | +35 | review recommended |
| #399 | Drop a stale ratchet session on unrecoverable decrypt failure | core/src/crypto/encrypt.rs | +145/-14 | **Rule-8** |
| #398 | POST /api/send option-B contract | cli/src/api.rs, cli/src/api_axum.rs, docs/API_CONTRACT.md | +177/-15 | - |
| #382 | /api/send reports success:false when only queued | cli/src/api.rs | +5/-1 | - |
| #381 | Reuse the live IronCore in the send-queue fallback (message loss) | cli/src/main.rs | +7/-15 | - |
| #380 | Grant consent in cmd_test | cli/src/main.rs | +5 | - |

- **Scope correction:** #382 and #398 both redefine /api/send success semantics. Compare `gh pr diff 382` with `gh pr diff 398`. If #398 covers #382, QUESTION the operator to close #382 as superseded.
- Land #401 first.
- **Acceptance:** each merged; CI green; JEV per PR; #399 has an uninvolved Rule-8 APPROVE on its head SHA.

### MT-03 -- P0 umbrella WP1-WP4 (ruling D-01)

| PR | Content | LoC |
|---|---|---|
| #383 | WP1: close contact-key fabrication and read/write asymmetry | +1,978/-97, 8 files |
| #352 | WP1 regression battery (ruling R-1) | +469/-6 |
| #349 | WP2: routing_peer_seen from all data-link transports (mobile_bridge) | +146/-1 |
| #355 | WP3: one auto-subscribe decision for both gossip loops | +178/-21 |
| #356 | WP4: watchdog decision and delivery-truth refusal | +520/-54 |

- **#383 review points** (the 2026-09-27 Harness panel, advisory only):
  - the ~583-line `iron_core.rs` integration is the riskiest surface
  - the `stored_at_millis: 0` sentinel must not break timestamp semantics downstream
  - WP1 documents a canonicalization fork: contacts accept any valid key, while the ledger/outbox require re-derivation. V5-01 closes that fork.
- **Acceptance:**
  - `python scripts/jev_canonical_check.py --wp WP1` .. `--wp WP4`, each is_passing >= 0.70
  - `core/tests/integration_wp1_identity_unification.rs` green in CI
  - P0 umbrella acceptance items 1-8 are re-proved in TRI-040. The umbrella says: do not claim WiFi fixed without WP5 evidence.

### MT-06 -- Android lifecycle (AND-SS-001, STOP-TEARDOWN-TIMEOUT-001)

- **Order:**
  1. **#351** (+1,423/-94, 16 files, CLEAN).
  2. **#367** (+305/-93, 2 files). If it conflicts with #351 in `MeshForegroundService.kt`, supersede it from `freebuff/train-mt06-fgs`.
  3. **MT-06c:** diff #364's `MeshForegroundService.kt` and `StopStartFloodTest.kt` against main. #364 has 76 lines #367 lacks. Port any behavior main lacks (for example in-flight teardown coalescing) and merge the test cases into the single `StopStartFloodTest.kt`. est +40-150.
- **Acceptance:**
  - ANDROID.md checklist; CI Android jobs green.
  - Device check: the operator taps; the executor reads logcat and `mesh.log`.
    - 6 STOP taps within 2 s produce exactly 1 teardown sequence.
    - START afterwards brings the mesh up.
    - No startForeground crash.

### MT-04 -- D6 routing feed (SHIP_PLAN T4)

- **Verify first:**
  ```
  MSYS_NO_PATHCONV=1 git grep -n -E "routing_peer_seen|peer_seen\(" origin/main -- core/src/transport/swarm.rs core/src/iron_core.rs
  ```
  If the SwarmEvent::ConnectionEstablished handler already feeds the routing engine, mark DONE-VERIFIED with that output. The Rule-8 review of PR #263 (routing feed) is recorded as resolved; check whether #263 merged.
- **Else:**
  - SHIP_PLAN 6.2 anchors: the handler at `swarm.rs:5277`; the ledger-convergence block at `swarm.rs:5486` (2026-08-31 numbering, re-anchor).
  - Add `peer_seen` with the transport type parsed from the endpoint multiaddr, using the `parse_transport_type`/`parse_peer_id_32` helpers from #239.
  - Reuse `origin/freebuff/v040-t4-routing-feed` (+232/-6, conflicts in iron_core.rs and swarm.rs) where it applies.
- **Acceptance:**
  - A test shows the routing engine learning a peer on ConnectionEstablished.
  - TRI-040 case C7 shows `routing_decision` events with nonzero confidence.

### MT-05 -- D9 + D1 transport (supersedes #361 and #372)

1. **Check containment.** `git merge-base --is-ancestor origin/d9-libp2p-degrade origin/fix/361-review-blockers`. If false, list the #361-only commits with `git cherry -v origin/fix/361-review-blockers origin/d9-libp2p-degrade` and include them.
2. **Merge #372.** Cut `freebuff/train-mt05-d9-d1` and merge `origin/fix/361-review-blockers`. Resolve the single `Cargo.toml` conflict: keep #391's wasm members/exclude layout AND add the vendor member.
3. **Doctrine files** must already match main (MT-00b). Where they don't, take main's.
4. **Dockerfile.** Bring over only the DOCKER-VENDOR-001 hunk from `origin/freebuff/and-stopflood-outbox-sweep-040:docker/Dockerfile` (lines 42-53: the comment plus `COPY vendor/libp2p-swarm-0.48.0/Cargo.toml` and `COPY vendor/libp2p-swarm-0.48.0/src`). Without it the image cannot build: main's Dockerfile has no vendor lines.
5. **Vendor tree.** Take #372's tree (c0b1fba5). Confirm that fmt/clippy/`rules_check.py` exclude `vendor/` without loosening first-party code.
6. **Fidelity.** Diff `vendor/libp2p-swarm-0.48.0` against `~/.cargo/registry/src/index.crates.io-*/libp2p-swarm-0.48.0`. The expected result is 12 files: `handler/either.rs` (the `unreachable!()` -> "D9-DEGRADE" drop) plus the #372 deltas. Attach the patch to the PR.
7. **PR body:** "Supersedes #361 and #372". After the merge, raise a QUESTION asking the operator to close both.

- **Scope correction:** #359 also implements the two-tier cap in `core/src/transport/per_peer_cap.rs`. This car's reviewed version wins; MT-08 drops #359's copy. Never land both.
- **Acceptance:**
  - CI green, including Docker Integration Suite.
  - D9 acceptance 1: an induced connect burst (`cli/src/bin/conn-fanout.rs`) does not panic.
  - D9 acceptance 4: a regression test pins the ordering.
  - Rule-8 APPROVE from an uninvolved reviewer on the head SHA. It must cover the vendor patch and the open question the patch itself records: can a remote peer induce the desync?
  - Docker Publish green on main after the merge.

### MT-07 -- Periodic outbox sweep (OUTBOX-SWEEP-001; the #364 remainder)

- Cut `freebuff/train-mt07-outbox-sweep` and merge `origin/freebuff/and-stopflood-outbox-sweep-040`.
- Resolve these in favour of main:
  - `vendor/`
  - `docker/Dockerfile`
  - `Cargo.toml`
  - `MeshForegroundService.kt` and `StopStartFloodTest.kt`
- For the two tickets (AND_STOP_START_FLOOD_AND_CANCEL, OUTBOX_NO_PERIODIC_RETRY_SWEEP), take the newest content.
- What should remain: the 120 s `outbox_sweep_interval` in `core/src/transport/swarm.rs` (the same +41 lines as the primary checkout's WIP), plus tests.
- **Acceptance:** a test with a connected peer and a stranded outbox entry shows the entry re-flushed within one sweep interval without a reconnect. TRI-040 R9 holds. Rule-8 (swarm.rs). The PR supersedes #364.

### MT-08 -- #359 remainder (T-CONN-04 dedupe + canonical docs)

- Work only from `origin/glm/canonical-outlier-audit`. Never touch the primary checkout.
- Cut `freebuff/train-mt08-conn04-docs` and merge it.
- **Docs:** the 48 conflicts are all HANDOFF docs. Take the newer side per path; where both sides add sections, keep both.
- **`per_peer_cap.rs`:** take main's (MT-05).
- **Any transport code diff left over:** Rule-8.
- The PR supersedes #359.

### MT-09 -- 0.4.0 defect tickets and release blockers

- **Step a, for each item:** verify it on origin/main. Does the ticket's acceptance already hold? Record the command and output.
  - If it holds: move the ticket to `done/` with that evidence.
  - If not: implement it on `freebuff/train-mt09-<x>`, one defect per PR.

| Car | Ticket (HANDOFF/todo) | Anchor | LoC est | Rule-8 |
|---|---|---|---|---|
| MT-09a | D2: seed sweep never re-dials a missing seed-tier peer | `cli/src/seed_dial.rs::sweep_decision()` returns WatchOnly when peer_count > 0 | +60-150 | default yes (transport-adjacent) |
| MT-09b | D3: self-recipient guard (enqueue guard, drain guard, `/api/outbox/cancel`) | `handle_send_message`, the outbox drain | +120-250 | if core/src/drift |
| MT-09c | D5/D6: persist bootstrap_nodes. Part 1 is config only on both cloud nodes (needs H-6); parts 2-3 are code. | config.json; `core/src/transport/bootstrap.rs` | +60-230 | if bootstrap.rs |
| MT-09d | D8: custody-only delivery receipt convergence. Reproduce first (A -> custody on B -> C offline, then attach). | core/src/drift, `core/src/store/relay_custody.rs` | +120-300 | yes (per ticket) |
| MT-09e | D10: public listener binding (after the H-4 ruling) | the multi-port adaptive listener | +20-150 | if core/src/transport |
| MT-09f | P1 CORE-02: IronCore in-memory outbox split-brain | `iron_core.rs:476` `Outbox::new()` | +30-120 | no |
| MT-09g | P1 Docker control API security hardening. Evidence: AWS runs `scm --http-bind 0.0.0.0:9876 start`. | control API bind + auth | +40-200 | review recommended |
| MT-09h | P1 release signing gate fail-closed | release workflow | +20-80 | no |
| MT-09i | P1 swarm bounded event channel backpressure deadlock (TRN-01) | core/src/transport | +50-200 | yes |
| MT-09j | P1 Android UniFFI curve-check relocation (AND-06; must land before the tag) | Kotlin -> `is_valid_public_key` over UniFFI | +20-100 | no |
| MT-09k | P1 Compose crash recurrence + FFI-in-composition build killer | android/ | +50-200 | no |
| MT-09l | P1 Windows node silent wedge (investigate) | set at step a | set at step a | per path |
| MT-09m | P1 async delivery receipts never converge (blocks D4; overlaps WP4 and D8) | verify after MT-03 + MT-09d | - | per path |
| MT-09n | P1 core identity spoof + WASM topic parity (overlaps WP1/WP3) | verify after MT-03 | - | per path |

Order: D3 before D2. D3's self-dial loop poisons the backoff table that D2 reads.

### MT-10 -- Triangulation tooling + missing lifecycle log markers

- **Add `message_id` and the canonical peer id** (never message content) where they are missing today:
  - `event = "outbox_egress_dispatched"` (`iron_core.rs:3294`)
  - the relay custody accept lines (`relay_custody.rs:679/695/702`), which carry identity_id and device_id only
  - custody delivery-on-attach (no per-message line exists)
  - the core receipt send (no line exists)
  - `event = "receipt_outbox_cleared"` (`iron_core.rs:3803`)
- **Tooling:**
  - `scripts/tri3.py` per Appendix D
  - `scripts/tri3_markers.json`, built from the real strings in Appendix D and confirmed against real logs
  - three test fixtures: pass, missing receipt, clock skew
- **Rule-8:** only for lines added under core/src/transport/.

### MT-11 -- Housekeeping (SHIP_PLAN G5, updated 2026-09-27)

- Close #227 (CONTAINED).
- Dependabot:
  - G5 said merge #214/#212/#211/#141. #141 now conflicts: rebase or close it. #103 conflicts too.
  - #213, #210, #108, #107 and #106 go to DEPENDENCY_DEBT (0.5.0).
- #156 (Docker suite non-blocking): G5 says close it. Operator.
- Archive the `HANDOFF/todo/INBOX_2026-08-11*` files (10 remain).
- Move verified-done tickets to `done/` with their PR numbers.
- Mark the superseded queue files.
- Correct the dead AWS IP (54.226.67.101) wherever docs still cite it.

### MT-12 -- Rules-drift guard (a hook change; lands as a normal PR under section 14 A1)

Extend the SessionStart orientation hook to print `[WARNING]` in either case:
- `git diff --quiet origin/main -- AGENTS.md CLAUDE.md docs/rules SHIP_PLAN.md` fails
- HEAD is more than 20 commits behind origin/main

est +30-60.

---

## 7. TRI-040 -- 3-node triangulated mesh proof

This covers SHIP_PLAN G3 (gates D4, D6, D7) and P0 umbrella WP5.

**7.1 Preconditions.**
- Every MT car is DONE, DONE-VERIFIED or operator-SKIPped. main is green.
- One commit C (the origin/main tip) is used everywhere.
- H-2 and H-3 are done. H-6 is granted.
- The JEV-LOG pack is frozen. Freezing is delegated by the 2026-09-28 interview ("triangulate fully with jev"):
  - the lane builds `scm-ops-log-v1` per Appendix C
  - verifies every keyword occurs in real logs
  - commits it under `HANDOFF/harness/packs/`
  - posts it on #403 (the operator may veto)

**7.2 Deploy C as a candidate** (approved: section 14 A5, A6 and A7; enable logging per A8 first). KEEP doctrine: no image or binary change without verified functional improvement, and this run is that verification. On FAIL, roll back.

- **AWS:**
  - Deploy: `IMAGE_TAG=testbotz/scmessenger:sha-<C7> scripts/aws_deploy.sh`. This is the only supported path: container `scm-node`, `--network host`, `-v /opt/scm-relay-data:/data`, `scm --http-bind 0.0.0.0:9876 start`, then a health poll.
  - Drive it only via ssh, calling `curl http://127.0.0.1:9876/...` on the host. That way the drive keeps working after MT-09g hardens the public bind.
  - Rollback: `IMAGE_TAG=testbotz/scmessenger:sha-45b0f8b scripts/aws_deploy.sh`.
- **Windows:**
  - Stage C's CI Windows artifact at `tmp/radio-candidates/<C>/scmessenger-cli.exe` and record its sha256.
  - The current live build is `bceacb94` at `C:/Users/SCM/.local/bin/scmessenger-cli.exe`; keep it as the rollback.
  - Stop the supervisor first, then the node. Otherwise the supervisor restarts the node:
    - stop the `powershell.exe` whose command line contains `run_node_supervised.ps1`
    - then `Stop-Process -Name scmessenger-cli`
  - Relaunch supervised and detached from the session (WMI):
    - Put the auto-reply text in `SCM_AUTO_REPLY`: the supervisor splits `-NodeArgs` on spaces.
    - Use the exact form from 2026-09-28:
      `Invoke-CimMethod -ClassName Win32_Process -MethodName Create -Arguments @{ CommandLine = 'cmd.exe /c set "SCM_AUTO_REPLY=<text>" && powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "C:\Users\SCM\Documents\GitHub\SCMessenger\scripts\run_node_supervised.ps1" -ExePath "<exe>" -NodeArgs "start -p 9001" -LogFile "C:\Users\SCM\AppData\Local\scmessenger\logs\node-supervisor.log"'; CurrentDirectory = 'C:\Users\SCM\AppData\Local\scmessenger' }`
  - Pass when all of these hold:
    - `/health` is healthy
    - `/version` names C
    - the newest `scm.log.*` shows `Identity initialized: Some("985a25f9...")`
    - `/api/diagnostics` peers include the AWS peer
  - The identity and data dir (`C:/Users/SCM/AppData/Local/scmessenger`) are never moved or reset.
- **Pixel:**
  - Get the CI APK for C, following CI_APK_TO_PHONE.md steps 1-3. Record `output-metadata.json` applicationId and versionCode, plus the run headSha.
  - Install with `adb install -r`.
  - If it fails with `INSTALL_FAILED_UPDATE_INCOMPATIBLE`: STOP. That is H-2; uninstalling would wipe the identity.
  - The operator opens the app.
- **Contacts:** re-seed the Pixel's triad (new or restored) on Windows and AWS through the CLI/API. Record all three triads in `nodes.json`.
- **Baselines, T0 and T-end:**
  - Windows and AWS: `/version`, `/health`, `/api/diagnostics` (custody count, history total/undelivered, outbox_count).
  - Windows clock: `w32tm /query /status`. AWS clock: `chronyc tracking` (via ssh).
  - Pixel clock: do NOT run any other `adb shell` command. Bound its offset from paired events (7.5 R2).

**7.3 Passive log capture.**
- **Pixel:**
  - `adb logcat -v threadtime,UTC,year > android.log`, running from before C1 until after the last case. The device clock is set to HST, so keep UTC stamps.
  - Then `adb exec-out run-as com.scmessenger.android cat files/logs/scmessenger-mesh.log > mesh.log`. The file is UTF-16 (FF FE): decode it before grepping.
- **AWS:** `ssh ... "sudo docker logs -t --since <T0> scm-node" > aws.log`.
- **Windows:** the hourly node logs `C:/Users/SCM/AppData/Local/scmessenger/logs/scm.log.<YYYY-MM-DD-HH>` covering the window.
- **All:** sha256 of every log into `manifest.json`.

**7.4 Message matrix** (75 driven messages).
- Send through the control API exactly as `docs/API_CONTRACT.md` documents it at commit C (MT-02 changed /api/send).
- Save each request/response JSON; the response carries the message id.
- Body marker: `TRI040-<case>-<n>-<UTC>`.

| Case | Gate | Path | Driven by | Messages |
|---|---|---|---|---|
| C1 | D4 | Windows -> Pixel, LAN | executor | 5 |
| C2 | D4 | Pixel -> Windows | operator taps | 3 |
| C3 | D4 | Windows -> AWS | executor | 5 |
| C4 | D4 | AWS -> Windows | executor (ssh) | 5 |
| C5 | D4 | AWS -> Pixel | executor (ssh) | 5 |
| C6 | D4 | Pixel -> AWS | operator | 2 |
| C7 | D6 | Windows -> Pixel with the first-choice transport down (Pixel WiFi off, cellular on): fallback through the AWS relay; `routing_decision` confidence > 0 | executor + operator toggle | 5 |
| C8 | custody (D8) | Pixel airplane mode on; Windows sends 3; wait 60 s; airplane mode off; delivered from custody and the sender status converges | executor + operator | 3 |
| C9 | soak | 60 min: Windows <-> AWS alternating every 2 min (30), plus Windows -> Pixel every 10 min (6) | executor | 36 |
| C10 | AND-SS-001 | 6 STOP taps in 2 s, START, then Windows -> Pixel | operator + executor | 2 |
| C14 | D7 | offline proximity: WAN blocked at the router (or an outbound firewall rule on Windows), Pixel mobile data off; Windows <-> Pixel over LAN/BLE only, 2 each way | operator + executor | 4 |
| C15 (R-2) | G3-0 | churn: AWS comes back on a new public IP; the nodes re-mesh unaided (D2/T1 evidence) | executor: `aws ec2 stop-instances` / `start-instances` (section 14, A6) | 0 (connectivity only) |

**7.5 Triangulation rules** (`scripts/tri3.py`, Appendix D). PASS requires every rule for 100% of the 75 messages.

| Rule | Requirement |
|---|---|
| R1 completeness | Direct path: sender E1, E3, E9 plus receiver E6, E7, E8. Relayed or custody path: additionally relay E4, E5. Direct messages must show NO custody event for that id on AWS. |
| R2 order | After clock correction (tolerance 2 s): E1 <= E3 <= (E4 <= E5) <= E6 <= E7 <= E8 <= E9. The Pixel offset is bounded by the minimum over pairs of (pixel E6 - windows E3). |
| R3 identity | Each libp2p PeerId maps to exactly one public_key_hex in all three logs. The sender in E6 equals the sender's canonical id. |
| R4 uniqueness | Exactly one durable receive per message_id. Duplicates are allowed only when a dedup line is logged. |
| R5 provenance | All three nodes report commit C (Windows `/version`, AWS `/version`, the Pixel APK headSha plus an app log line). |
| R6 delivery | The 2.5 five-way evidence holds for every message. |
| R7 hygiene | 0 panics or backtraces. `D9-DEGRADE` counts are reported per node. 0 `identity_registration_missing` for matrix recipients. |
| R8 mesh | Every expected pair shows connection establishment in both logs. Disconnects are mirrored, or explained by the C7/C8/C10/C14/C15 operator actions. |
| R9 state | At T-end, outbox_count is 0 on Windows and AWS, and undelivered has not increased. Custody deltas are explained by C8. |

**7.6 Judgment and JEV.**
- `log_judgment` on each log with `scm-ops-log-v1`, `--max-cost 0.02` each. PASS: no item at the `blocking` level. Unmatched items are reported as they are, never smoothed.
- The JEV phase `TRI-040` evidence cites: C, the three log sha256s, the `tri_report.json` path, the judgment paths, and the 75 message ids.

**7.7 FAIL.**
1. Write a BLOCKED inbox report with the failing ids and the matching excerpt from each of the three logs.
2. Roll back to KEEP (7.2).
3. Fix in a new car MT-1x.
4. Re-run the whole matrix on the new C. Partial re-runs never count.

---

## 8. Tag v0.4.0 (operator)

The master plan section 4 checklist, all 10 items:
1. Wave-1 code landed, or waived in writing
2. Audit HIGHs CO-B-001/002 closed, and CO-G-001/002 disposed (CO-B-001 merged as #339)
3. AND-06 A1+A2, or a dated waiver (H-9; MT-09j)
4. SEC-03 (H-8)
5. AndroidTest compile green on Mobile
6. Pinned debug keystore plus a successful Pixel `install -r` (H-2)
7. 3-node same-SHA plus the log pack (TRI-040)
8. D1: required CI green on the tag SHA
9. External audit commissioned, or waived (H-7)
10. The tag and the release (H-11)

SHIP_PLAN G4 adds:
- The tag is final v0.4.0, not an rc.
- Fill the ledger.
- Delete API_RESET_EXECUTION_CHARTER.
- Retire V040_COMPLETION_PLAN.

**TAG-040** (approved, section 14 A10):
1. Check which workflows fire on `v*` tags.
2. `git tag -a v0.4.0 <C> -m "v0.4.0"`
3. `git push origin v0.4.0`
4. Publish the release with notes: D2 skipped, the H-7 ruling, known issues, the TRI-040 evidence.

Checklist items 6, 8 and 9 are closed by the rulings in 14.2: H-2 done; D1 green; H-7 replaced by AUD-040.

---

## 9. Unify the workspace (after TAG-040)

**U-1 Retire worktrees** (approved, section 14 A12).
- Use BK-01's tool only: run `bash "$BP" backup`, then `verify`, then `purge` (`$BP` as in P0-8).
- It removes only clean, fully pushed worktrees and byte-identical scratch copies.
- A worktree holding WIP is never removed: it stays, backed up.

**U-2 Branches.** None deleted: the operator ruled tags only (section 14, A3).

**U-3 Exit criteria:**
- every remaining worktree's rules files are identical to origin/main
- every remaining worktree is a kept checkout, or holds WIP backed up on GitHub
- 0 unclassified remote branches
- 0 duplicate PRs for one fix

---

## 10. Phase 6 -- v0.5.0 train (nothing starts before TAG-040)

Scope sources:
- HANDOFF/plans/V050_PHILOSOPHY_AND_BORROW_PLAN.md: adopt Reticulum/LXMF designs and philosophy only, zero RNS/LXMF code, no crypto downgrade.
- CODEBASE_UNIFICATION_PLAN.md
- DEPENDENCY_DEBT_TOOLCHAIN_UPGRADE_2026-08-28.md
- P0_ANDROID_FINITE_RETRY_ABANDONMENT (v0.5.0-blocking)

The philosophy doc's explicitly rejected items stay rejected.

**V5-00 Scope lock.**
- Diff this section against `git grep -n -i -E "0\.5\.0|v0\.5|V050" origin/main -- HANDOFF docs "*.md"`. Pay particular attention to HANDOFF/plans/MILESTONE_RELEASE_PLAN.md.
- Send one QUESTION with the delta table.
- Post the scope table on #403, then continue (section 14 A12). Operator objections apply from the moment they are seen.

| Car | Content | Anchors / sources | LoC est | Rule-8 |
|---|---|---|---|---|
| V5-01 | A1 canonical PeerIdentity (see below) | see below | +500-800/-300-600 | YES |
| V5-02 | A2 IronCore sole store owner (see below) | iron_core.rs:476; MeshRepository.kt:911 | +100-250/-80-200 | no |
| V5-03 | A3 protocol registry (ticket U2 in_progress; see below) | CODEBASE_UNIFICATION_PLAN items 2, 3, 6, 8 | +50-120/-35-75 | YES for the swarm.rs literal swaps |
| V5-04 | A4 core state machines (see below) | core; Kotlin FGS shrinks | +300-500/-100-250 | YES |
| V5-05 | Nicknames (see below) | Kotlin sites listed below | +30-60/-25-100 | no |
| V5-06 | Relay identity registration on Identify moves into core; delete the CLI copy and the mobile_bridge copy | cli/src/main.rs; mobile_bridge.rs | +80-150/-100-200 | YES |
| V5-07 | CODEBASE_UNIFICATION_PLAN rank 1 + the ledger choke point, as ONE Rule-8 review (see below) | see below | +55-150/-35-85 | YES |
| V5-08 | Dead code (see below) | see below | +10-40/-200-600 | no |
| V5-09 | Philosophy wave (see below) | V050_PHILOSOPHY_AND_BORROW_PLAN.md | +420-950 code/docs; +200-500 docs | P3 AUDIT-GATE transport |
| V5-10 | P0 Android finite retry abandonment (PF-1/PF-12) | android/ | +100-300 | no |
| V5-11 | Dependency debt: the coordinated Android toolchain upgrade, absorbing dependabot #213, #210, #108, #107, #106 | android/ | +50-300 | no |
| V5-12 | Android work-ahead (see below) | measured | +629/-38 (plus #303, +18/-7) | no |
| V5-13 | Remaining P1/P2 tickets not closed in 0.4.0 (see below) | tickets | set at step a | per path |
| V5-14 | iOS parity, MAC lane only (see below) | measured upper bound; overlapping, 387-865 behind | <= +7,700/-580, plus #301 +32 | MAC lane |
| V5-15 | TRI-050 tooling deltas for the new state machines | tri3 markers | +50-150 | as needed |

**V5-01 A1 canonical PeerIdentity**
- One parser that never fabricates a key.
- Every store key, route, outbox queue, notification id and CLI argument goes through it.
- Exported over UniFFI and wasm.
- It closes WP1's documented contacts-vs-ledger fork.
- Delete or redirect these per-site canonicalizers:
  - `derive_public_key_from_peer_id` (contacts.rs:491-529)
  - `Config::strip_peer_id` (cli/src/config.rs:279-285; import `addr_filter::strip_peer_id` instead)
  - `migrate_libp2p_peer_ids_to_canonical_hex`
  - `history_peer_matches`
  - outbox `canonical_peer_key`
  - `parse_peer_id_32`
  - CLI `peer_id_from_contact_identifier`
  - Android `canonicalContactIdPublic` (NOTIF-UNIFY-001)

**V5-02 A2 IronCore as sole store owner**
- CORE-02 remainder.
- CLI store construction goes through IronCore.
- One LedgerManager: a UniFFI accessor, following the `relay_bootstrap_manager_handle()` pattern at iron_core.rs:3346. Android `MeshRepository.kt:911` stops constructing its own; iOS via the MAC lane.

**V5-03 A3 protocol registry** (ticket U2, in_progress)
- Topics: 9 literals in swarm.rs plus 2 in cli/src/bootstrap.rs plus 2 in cli/src/main.rs, all pointing at `core::TOPIC_LOBBY`/`TOPIC_MESH` (lib.rs:262-263).
- Bootstrap env vars: only `SC_BOOTSTRAP_NODES` survives. Remove `BOOTSTRAP_NODES` (docker-compose.yml:33,49) and `SCMESSENGER_BOOTSTRAP_NODES`.
- BLE UUIDs: `cli/src/ble_windows.rs:21-23` and `cli/src/ble_mesh.rs:43` share one module.
- The receipt decode bypass at `cli/src/main.rs:2136`.
- A CI grep gate: `rg '"sc-(lobby|mesh)"' --glob '!*test*'` must match only `core/src/lib.rs`.

**V5-04 A4 core state machines**
- Node start/stop is idempotent and serialized, with a generation token, so no platform shell can race `stop()`.
- One DeliveryState: queued / sent / custody / delivered / failed.

**V5-05 Nicknames**
- Export normalize/select from Rust over UniFFI.
- Delete the 3 private Kotlin copies: MeshRepository.kt:9549, ContactsViewModel.kt:117, DashboardViewModel.kt:862.
- Point ContactDisplayName.kt (74 LoC) at the export.
- Swift copies go through the MAC lane.

**V5-07 CODEBASE_UNIFICATION_PLAN rank 1 + ledger choke point** (one Rule-8 review)
- 1a: the DNS/SSRF gate becomes a required parameter of `LedgerStore::record_connection` (cmd_start at main.rs:2034 currently skips it).
- 1b: `to_shared_entries` uses core's `ledger_entry_to_shared_routing_only` (3 call sites).
- 1c: `strip_peer_id` (the V5-01 part).

**V5-08 Dead code**
- The second `Commands` enum at cli/src/cli.rs:160. First check android/ and core/tests/ for references.
- `core/src/relay/bootstrap.rs`, fully dead.

**V5-09 Philosophy wave**
- P1: README threat-model prose (+40-100).
- P2: `scm status` / `scm probe <peer>` (+150-300).
- P3: gossip governance for constrained transports (+150-350) [AUDIT-GATE transport].
- P4: session-establishment byte-budget metrics (+80-200).
- Design notes only: C1 custody-store peering and Q1 QR paper messages (+200-500 docs) [AUDIT-GATE + operator escalation before any code].

**V5-12 Android work-ahead**
- #300 +149/-11, #302 +267/-4, #298 +64/-23, #299 +149.
- workahead/notif-cold-start-gate (+18/-7, 1 conflict).
- #303 (merge plan) is superseded by this file: archive it.

**V5-13 Remaining P1/P2 tickets not closed in 0.4.0**
- P1 chat order across clocks
- P2 wire envelope truncation
- P2 Kotlin warning triage (about 15 sites)
- CORE_BLOCK_GATE_IDENTIFIER_FIX (in_progress)
- CORE_DIAL_CANDIDATE_DOUBLE_CIRCUIT_PRUNE
- GHOST_LEDGER_PRUNE_G1
- CELL_ROUTE_AWS_001

**V5-14 iOS parity** (MAC lane only)
- A-05 iOS receipt unification
- #208 gpt/v050-parity-burndown (+2,008/-164, 1 conflict: Info.plist)
- gpt/v050-parity-burndown-v2 (+4,350/-48, CLEAN)
- gpt/v050-ios-release-ready (+1,015/-320, 7 conflicts)
- gpt/ios-v050-delivery-retry-wip (+327/-48, CLEAN)
- #207, #178, #301

**Order:**
1. V5-00
2. V5-01
3. V5-02, V5-03, V5-07 (in parallel after V5-01)
4. V5-04
5. V5-05, V5-06
6. V5-08
7. V5-10, V5-11, V5-12, V5-13
8. V5-09
9. V5-15
10. TRI-050

V5-14 runs in parallel on the MAC lane and joins before TRI-050.

Out of v0.5.0: PQC_FULL_INTEGRATION_REVIEW (post-0.5.0), PQC-09 (parked), and the V050-R1..R3 research lane.

---

## 11. TRI-050 and tag v0.5.0

Same procedure as section 7, on commit C5, with three more cases:

| Case | What it proves |
|---|---|
| C11 | Identity spelling mix: sending to a contact stored under its libp2p PeerId form delivers, and threads under one canonical id |
| C12 | Windows rapid `scm stop` / `scm start` race: V5-04 holds |
| C13 | The sender's delivery state equals the receipt evidence for every message (V5-04 DeliveryState) |

If the MAC lane has an iOS build of C5, iOS joins as a 4th node, driven by the MAC lane. Its log becomes a 4th triangulation source.

Tag and publish v0.5.0 under section 14 A10, the same way as TAG-040.

---

## 12. LoC ledger

"To land" = already written, in PRs or branches. "To write" = new code.

| Release | Bucket | LoC |
|---|---|---|
| v0.4.0 | To land, upper bound (sum of PR diffs, non-vendor) | +23,634 / -862 |
| v0.4.0 | To land, after dedupe (est) | +9,800 - +15,500 (MT-00 1.7-4.1K, MT-02 0.5K, MT-03 3.3K, MT-05 1.5-2.7K, MT-06 1.7K, MT-07 0.15-0.6K, MT-08 0.5-3.0K) |
| v0.4.0 | To land, vendored upstream copy | +13,403 (reviewed as a +204/-45 patch plus #372 deltas) |
| v0.4.0 | To write (est) | +1,065 - +2,920 (MT-01, MT-04, MT-06c, MT-09 a-k, MT-10, MT-12; MT-09 l-n set at step a) |
| v0.4.0 | For reference: SHIP_PLAN 6.5 (2026-08-31) | ~1,300-2,530 to the tag. Items since landed are CONTAINED (T1 half 2, T8, T14). |
| v0.5.0 | To land (measured) | Android +629/-38 (plus +18/-7); iOS on the MAC lane <= +7,700/-580 plus +32 |
| v0.5.0 | To write (est) | +1,895 - +4,120 / -875 - -2,110. Unification is roughly net-negative code. |

---

## 13. Live status and cross-session updates

- **Car status, source of truth:** `STATE_DIR/TRAIN_STATE.md`, the executor's own ledger.
- **Shared tracker:** PR #403. On every resume, run `gh pr view 403 --comments` before continuing.
  - The executor posts one comment per completed car (the REPORT block from 0.6).
  - The operator and other sessions post cross-session updates there: pushes, rulings, gate answers.
  - A comment from the operator that answers a GATE counts as that GO.
- **Retired:** the v1 draft `HANDOFF/freebuff/TRAIN_V040_V050_UNIFY_2026-09-27.md` (never committed; deleted). Any state carried over from it must be re-verified with commands.
- **2026-09-28 update,** from the operator via the Opus session (details in the #403 comment of that date):
  - #402 merged; #396 and #397 updated onto main.
  - The Windows node is running `bceacb94` under the supervisor.
  - BK-01 is inserted as P0-8.

---

## 14. Operator standing approvals and rulings (interview, 2026-09-28)

These supersede section 3 and any conflicting line in FREEBUFF.md or AGENTS.md for this train. MT-00b records A1, A6 and A10 in FREEBUFF.md and AGENTS.md (FREEBUFF LANE).

### 14.1 Standing approvals: the lane acts without asking, inside these conditions

**A1 Merge PRs** with `gh pr merge <n> --merge` (merge commit), only when ALL of these hold:
- The branch is up to date with main. Use `gh pr update-branch <n>` and wait for the re-run.
- Every REQUIRED check is green on the head SHA. Re-read the list before each merge:
  `gh api repos/Sovereign-Communication/SCMessenger/branches/main/protection/required_status_checks --jq '.contexts[]'`
- POST-MERGE closing gate (ruling 14.6, 2026-09-29): the car's JEV gate (2.3) is scored on the merge SHA immediately after the merge and is >= 85 with hard gates clear before the next dependent car merges and before any tag. Only the MT-00a anchor PR (#414) is exempt (it adds the gate; it is still scored after its merge). Every other condition in this list is a pre-merge hard gate.
- Gated code (`core/src/{crypto,transport,routing,privacy}/` or `vendor/`) has an A2 APPROVE on that exact head SHA.
- There are no unresolved review threads or requested changes.
- Any failing non-required check has been investigated and shown not to be a real defect.

Never `--admin`. Never force-push.

**A2 Rule-8 review** of gated diffs. Both steps run, and both are recorded on the PR.
1. **Harness first pass,** hard cap $0.10 per gated PR:
   `<HARNESS> verify --prompt-file <packet.md> --source-file <diff.patch> --converge --max-cost 0.10`, where `<HARNESS>` is the admitted-harness invocation recorded in TRAIN_STATE.md at P0-6
   The packet holds: the PR, the head SHA, the diff, the SECURITY_PROTOCOL.md checklist (race conditions, null checks, timing side channels, edge cases), the ticket acceptance and any prior dossier. For D9 that dossier is `HANDOFF/review/D1_D9_HARNESS_ADVERSARIAL_FINDINGS_2026-09-22.md`.
2. **Claude final sign-off,** always. Pass every artifact (packet, diff, Harness verdict) to a fresh session:
   `/isolated-request --model sonnet --budget 1.0 <prompt>`
   - Fallback 1: `python C:/Users/SCM/Documents/GitHub/Harness/.claude/skills/isolated-request/run.py --model sonnet --budget 1.0 --prompt-file <file>`, run from the worktree root.
   - Fallback 2, clean context: from an empty scratch dir, `claude -p --model sonnet --permission-mode plan --add-dir <worktree> < <file>`.

Rules for the review:
- The reviewer authored nothing, and its verdict names the head SHA.
- A later commit that touches gated code voids the verdict.
- REJECT: fix, then run both steps again.
- The lane never signs its own work.

**A3 T-3 archive** means pushing `archive/<branch>` tags only.
- Never delete a remote branch.
- Never touch `backup/*` or `rescue/*`.

**A4 Close superseded or obsolete PRs** only after the superseding PR has merged, or the content is proven on main (merge-tree CONTAINED).
- Comment with the evidence when closing.
- Branches stay.

**A5 Windows node: full drive.**
- Allowed: redeploy and restart per 7.2, API sends and reads, logs, debug logging.
- The node runs at all times except during a deliberate redeploy. If it is found down, relaunch the live build and report.
- No Task Scheduler entry (operator: no).

**A6 AWS node: full drive.** ssh as `ec2-user` with `~/.ssh/scm-node-key.pem`, for these uses only:
- `scripts/aws_deploy.sh` deploys and rollbacks
- `sudo docker logs`
- API calls to `127.0.0.1:9876` on the host
- the churn test's `aws ec2 stop-instances` / `start-instances`, using the local AWS CLI. First confirm `aws sts get-caller-identity`, then discover the instance exactly as `aws_deploy.sh` does.

No other system changes, installs or firewall edits.

**A7 Pixel: install and passive logs only.**
- Allowed: `adb install -r` of CI APKs; `adb logcat`; `run-as` log reads.
- Never drive the UI. Every tap, toggle or onboarding step is the operator's, requested through a GATE.

**A8 Comprehensive logging, before TRI runs:**
- **Windows:** add `set "RUST_LOG=info,scmessenger=debug"` to the 7.2 launch chain.
- **AWS:** confirm `RUST_LOG=info,scmessenger=debug` in the running container (`docker inspect`).
- **Pixel:** debug APK, logcat plus `mesh.log`.
- MT-10's per-message markers are live.

**A9 Paid spend caps:**

| Use | Cap |
|---|---|
| Harness first pass (A2) | $0.10 per gated PR |
| AUD-040 | $0.10 total |
| TRI log_judgment | $0.10 per TRI run |
| Claude review | $1.00 each |

Stop and ask before exceeding any cap.

**A10 Tag and publish v0.4.0 and v0.5.0** when every tag gate passes:
1. Read which workflows fire on `v*` tags, and whether they succeed without D2 signing.
2. Push the annotated tag on the proven commit C.
3. Publish the release, with notes covering: D2 skipped, the H-7 ruling, known issues, and the TRI evidence summary.

**A11 BK-01** backup and safe reclaim, per P0-8. It never removes WIP.

**A12 Continue into v0.5.0** after TAG-040:
1. U-1..U-3, using BK-01's rules.
2. V5-00: post the scope table on #403, then continue.
3. The V5 cars, TRI-050, TAG-050.

Operator objections on #403 apply from the moment the lane sees them.

### 14.2 Rulings

| Id | Ruling |
|---|---|
| D-01 | WP1-WP4 (#383, #352, #349, #355, #356) land in v0.4.0 (MT-03). |
| R-1 | #352 lands on its own evidence. Confirm with `python scripts/jev_canonical_check.py --wp WP1` (is_passing >= 0.70). |
| R-2 | The churn test C15 is in TRI-040; the lane runs the EC2 step (A6). |
| H-1 / D2 | Skipped for v0.4.0: no signed release APK. The release notes say so. |
| H-2 | Done: the debug keystore pin works (P0-7: Mobile 5/5 green). |
| H-3 | Onboard fresh (see below). |
| H-4 / D10 | Stay internet-reachable. MT-09e shrinks to rate-limiting or downgrading the `Parse(Method)` scanner-noise lines; Rule-8 if the change lands in `core/src/transport`. |
| H-5 | Per A2. |
| H-6 | Granted, per A6. |
| H-7 | External crypto audit replaced by AUD-040 for v0.4.0 (see below). |
| H-8 / SEC-03 | Open the sled-migration branch with an owner and a plan. Nothing merges before the tag. |
| H-9 / AND-06 | Implement A1+A2 before the tag (MT-09j). |
| H-10 | Keep strict branch protection; update branches before merging. |
| H-11 | Merges, the tag and publishing are delegated per A1 and A10. |

**H-3 detail:**
- The operator opens the app and onboards a new identity.
- The lane re-seeds it as a contact on Windows and AWS.
- The old triad (779e9ea3...) stays in place but is excluded from TRI matrices.
- `tmp/pixel-data-backup-20260921` is never touched.

**H-7 detail:** the external crypto audit is NOT commissioned for v0.4.0. AUD-040 replaces it:
- Scope: a Harness audit (<= $0.10 total) of `core/src/{crypto,transport,routing,privacy}`, the D9 vendor patch and the PQC status, with JEV completion gates.
- High-severity findings get an A2 step 2 Claude confirmation and a ticket.
- REJECT-level findings block the tag.
- Record it as a dated operator ruling in SHIP_PLAN G4-2 and in the release notes.

### 14.3 T-4 default dispositions (applied unless the operator overrides on #403)

- **Already merged** (#339, #353, #354, #358, #362, #402): mark DONE after verifying each state.
- **Open PRs:**
  - #357: MT-00b.
  - #360: MT-00b if still relevant after rebase; otherwise close with evidence (A4).
  - #316: MT-08.
  - #329: MT-11; close it if its three status lines are superseded.
  - #363 (draft OpenClaw bridge ops): REVIEW queue, not v0.4.0.
  - #366 and #368: verify state.
  - #369: read in T-4, never merged.
- **`MeshServiceViewModelTest`** (the CI blocker on #361/#364): fix its real cause inside MT-05/MT-06. Never weaken or skip it.
- **#364 split:** FGS behaviour -> MT-06c; Dockerfile hunk -> MT-05; outbox sweep -> MT-07.
- **`train/A3-deny-waiver`** (removes the rustls RUSTSEC-2026-0285 waiver): land it only when `cargo deny check` passes without the waiver. Otherwise keep the waiver and the P1 tracking ticket.
- **T-COB001:** verify against main (CO-B-001 merged as #339); if still open, add it to MT-09.

### 14.4 Still operator-only

- Physical Pixel actions: open and onboard; the toggles for C2, C6, C7, C8, C10 and C14; waking, unlocking and pairing for adb.
- Pausing other agent sessions before BK-01's purge.
- Anything outside 14.1.

### 14.5 Hard stops: write BLOCKED or QUESTION, then wait

- main goes red after a merge. Stop merging; the fix or revert goes through the same gates.
- A Rule-8 REJECT.
- A secret found in a diff.
- Any identity change on the Windows or AWS node.
- A spend cap would be exceeded.
- A destructive action not covered by 14.1.
- Any TRI rule failure. Roll back per 7.7.

### 14.6 Rulings of 2026-09-29 (operator, answered in chat; recorded on #403)

Three [OPERATOR GATE] items were answered with the recommended option each.

| Gate | Ruling | Effect |
|---|---|---|
| JEV-ORDER | JEV POST-MERGE | A1's JEV bullet is a post-merge closing gate (see A1). Every other A1 condition stays a pre-merge hard gate. A miss after a merge is a 14.5 stop: fix-forward or revert through the same gates, and it blocks the next dependent car and the tag. Only the MT-00a anchor PR (#414) is exempt. |
| #404 Rule-8 REJECT | FIX+REVIEW | The dispatch identity is validated BEFORE any checkout. The fix and the anchor land as one PR, #414, which supersedes #407 and #404 (close them under A4 only after #414 merges). |
| A2 step 2 | Supplied by the orchestrator session | Clean-context, read-only Claude reviewers (sonnet by default, opus for the D9 vendored-libp2p and transport PRs), dispatched from the orchestrator session, sign only the exact head SHA reviewed and never author the change. The Harness first pass stays <= $0.10 per gated PR. Multi-model panels still cannot clear Rule-8. |

Consequences recorded here so no file contradicts them: the evidence for the
post-merge JEV gate is captured from commands after the merge (Appendix B), a
status row never claims a merge that has not happened, and `pr_merged` is read
from `gh pr view <n> --json state,mergeCommit`, not from prose. Two limits found by
an adversarial review of the ruling: a car with merged dependents is fixed forward,
never reverted (a revert followed by an update of a dependent silently drops the
car's content from the dependent; only a tip car may be reverted), and an
acceptance item of the car that has no command evidence is recorded as an open
blocker, because the scorer has no per-car contract for an unknown phase id and
would otherwise score a car on CI colour alone.

---

## Appendix A -- merge-tree triage (Git Bash, git >= 2.38, read-only apart from loose objects)

```bash
#!/usr/bin/env bash
set -u
MAIN=origin/main
OUT="${1:-C:/Users/SCM/Documents/GitHub/scm-train-state/TRIAGE.tsv}"
T=$(git rev-parse "$MAIN^{tree}")
printf 'ref\tclass\tland_loc\tconflicts\tdate\tbehind\tahead\n' > "$OUT"
git for-each-ref --format='%(refname:short)' refs/remotes/origin |
  grep -v -e '^origin/HEAD$' -e '^origin/main$' -e '^origin$' |
  while read -r ref; do
    if git merge-base --is-ancestor "$ref" "$MAIN"; then continue; fi
    counts=$(git rev-list --left-right --count "$MAIN...$ref")
    behind=${counts%%[[:space:]]*}; ahead=${counts##*[[:space:]]}
    date=$(git log -1 --format=%cs "$ref")
    out=$(git merge-tree --write-tree --name-only --no-messages "$MAIN" "$ref" 2>/dev/null)
    rc=$?
    tree=$(printf '%s\n' "$out" | head -n1)
    conf=""
    if [ "$rc" -eq 0 ] && [ "$tree" = "$T" ]; then
      class=CONTAINED; loc="0"
    elif [ "$rc" -eq 0 ]; then
      class=CLEAN; loc=$(git diff --shortstat "$MAIN" "$tree")
    elif [ "$rc" -eq 1 ]; then
      class=CONFLICT; loc=$(git diff --shortstat "$MAIN...$ref")
      conf=$(printf '%s\n' "$out" | tail -n +2 | grep -v '^$' | head -5 | paste -sd, -)
    else
      class=ERROR; loc=""
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$ref" "$class" "$loc" "$conf" "$date" "$behind" "$ahead" >> "$OUT"
  done
```

## Appendix B -- JEV `--evidence` template (every value captured from a command)

Follow the override surface as documented in the JEV dogfood run record (`git ls-files | grep -i JEV_DOGFOOD_RUN`).

```json
{
  "phase": "MT-02-401",
  "status_row": "| MT-02-401 | outbox drain spellings | COMPLETE | PR #401 merged <merge sha> | CI green; JEV bar |",
  "pr_merged": true,
  "ci_green": true,
  "local_gates_green": true,
  "origin_evidence": "<verbatim output of: gh pr view 401 --json state,mergeCommit,statusCheckRollup>",
  "open_blockers": []
}
```

- A COMPLETE `status_row` must not contain the words open, fail, repair, pending or blocked. The scorer reads them as contradicting evidence. Watch for substrings too: "failover", "OpenClaw".
- `local_gates_green` is true only with a captured exit code 0, or the CI job URL under the CI-primary doctrine.

## Appendix C -- JEV-LOG pack `scm-ops-log-v1` (the operator must freeze it before TRI)

**Base:** the 10 buckets of the frozen issue-sort pack `HANDOFF/harness/packs/scm-ops-issues-v1.json`, unchanged:

| Bucket | Bucket | Bucket | Bucket | Bucket |
|---|---|---|---|---|
| capacity | backoff | poison_queue | crypto_noise | ble_lane |
| storage | identity_device | orchestration | android_build | supply_chain |

**Add two buckets.** Both keywords were observed in live evidence: the D9 patch's log text and ticket D10's scanner line.

```json
"d9_degrade": {"label": "D9 Either-router mismatch dropped (degrade, not panic)", "kind": "trouble_area",
  "path_id": "vendor/libp2p-swarm-0.48.0", "keywords": ["D9-DEGRADE"],
  "suggested_next_action": "Count per hour per node; compare with the pre-D9 rate (2-3 panics per hour on the bridge node); a rising count goes to the D9 ticket's remote-induced-desync question.",
  "attention": "medium"},
"api_scanner_noise": {"label": "HTTP listener parse noise from internet scanners", "kind": "trouble_area",
  "path_id": "core/src/transport", "keywords": ["Parse(Method)"],
  "suggested_next_action": "See D10; count per hour; expected to fall to zero after the D10 binding ruling (H-4).",
  "attention": "low"}
```

**Add the score block** that `log_judgment` requires:

```json
"score": {"id": "scm-log-severity-v1",
  "instructions": "Score each log item by its effect on end-to-end delivery of a TRI matrix message.",
  "levels": ["benign", "noise", "degraded", "blocking"]}
```

## Appendix D -- `scripts/tri3.py` specification and real markers

**Inputs:**
- `nodes.json`: per node, public_key_hex, identity_id, libp2p PeerId, commit, and the clock offset.
- `matrix.json`: case, body marker, from, to, expected path (direct | relay | custody | any), and the message id from the send response. Pixel-originated sends are linked by case window plus the receiver's `inbox_receive` id.
- `tri3_markers.json`.
- The three logs. `mesh.log` is decoded from UTF-16 first.

**Output:** `tri_report.md` and `tri_report.json`: one row per message with E1-E9 per node, verdict and reason. Exit 0 only on 100% PASS. Never print message bodies.

**Markers (2026-09-27 source):**

| Event | Rust core marker | Where | Android marker (MeshRepository.kt) |
|---|---|---|---|
| E1 enqueue | `event = "outbox_enqueue"` (message_id, recipient_id) | core/src/store/outbox.rs:236 (span `packet_lifecycle`) | via core |
| E2 route | `event = "routing_decision"` | core/src/routing/optimized_engine.rs:168 | - |
| E3 egress | `event = "outbox_egress_dispatched"` (no message_id: MT-10 adds it) | core/src/iron_core.rs:3294 | - |
| E4 custody accept | "node custody accepted in cooperative mesh mode ..." / "relay custody accepted in Phase A/B compat mode ..." (identity_id only: MT-10 adds message_id) | core/src/store/relay_custody.rs:679 / 695 / 702 | - |
| E5 custody forward | none today (MT-10 adds it); "[CUSTODY-REARM]" is the retry reset only | core/src/store/relay_custody.rs:852 | - |
| E6 receive | `event = "inbox_receive"` (message_id, sender_id) | core/src/store/inbox.rs:238 | "UNIFICATION onMessageReceived pairing: ... messageId=..." :2236 |
| E7 durable | the `inbox_receive` persist, plus receiver history via the API | inbox.rs:238 | - |
| E8 receipt send | none today (MT-10 adds it) | iron_core.rs | "Targeted delivery receipt sent for <id> to <sender>" :2980 |
| E9 receipt receive | `event = "outbox_dequeue"` reason=delivery_confirmed (message_id); `event = "receipt_outbox_cleared"`; history "Successfully marked message <id> as delivered" | outbox.rs:389; iron_core.rs:3803; core/src/store/history.rs:342 | "[RECEIPT-RX] History updated: msg=<id>" :2666 |
| failure | "Failed to decrypt ratchet message from peer ..." | iron_core.rs:3661 | "[ERROR] Receipt send FAILED ..." :2871 / :2903 |

## Appendix E -- prompts

**LAUNCH** (paste once into Freebuff; operator-approved 2026-09-28):
```
SCMessenger merge train -- FULL LAUNCH (Freebuff lane), operator-approved 2026-09-28.
You are the executor. Goal: land all v0.4.0 work, prove the mesh on three nodes, tag and publish
v0.4.0, then do the same for v0.5.0 -- using only the approvals in section 14 of the task file.

Task file: C:/Users/SCM/Documents/GitHub/wt-train-plan/HANDOFF/freebuff/queue/V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md
(branch docs/merge-train-v040-v050-20260927 = PR #403, the shared tracker. If the worktree copy is
missing: MSYS_NO_PATHCONV=1 git show origin/docs/merge-train-v040-v050-20260927:HANDOFF/freebuff/queue/V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md)
State: C:/Users/SCM/Documents/GitHub/scm-train-state/TRAIN_STATE.md -- re-read before every step,
update after every step.

READ FIRST, in full: CLAUDE.md, AGENTS.md (FREEBUFF LANE), docs/rules/FREEBUFF.md,
HANDOFF/freebuff/inbox/README.md, then task-file sections 0-3 and 14. Section 14 holds the operator's
standing approvals and rulings and overrides anything older. Then run `gh pr view 403 --comments` and
reconcile TRAIN_STATE.md, verifying each claim with the command it names. Read the other sections one
at a time, as you reach them.

ORDER
 1. Commit your two untracked inbox reports in wt-train-plan (HANDOFF-SCOPE block,
    python scripts/validate_handoff_scope.py --repo-root . --staged, push to the #403 branch).
 2. P0-8 BK-01, with BP=C:/Users/SCM/Documents/GitHub/wt-train-plan/scripts/backup_purge.sh:
    bash "$BP" inventory -> backup -> verify (0 missing) -> purge --dry-run -> purge -> report.
    Uncommitted work is never removed.
 3. MT-00a: merge #397 now (A1). #396 has two high CodeQL cache-poisoning alerts (ci.yml:142/144):
    fix or prove false positive before merging it.
 4. T-3: push archive tags only (A3).
 5. T-4: apply 14.3.
 6. Section 6 cars in order: MT-01, MT-00b, MT-02, MT-03, MT-06, MT-04, MT-05, MT-07, MT-08,
    MT-09, MT-10, MT-11, MT-12.
 7. AUD-040 (14.2 H-7).
 8. TRI-040 (section 7).
 9. TAG-040 (A10).
 10. U-1..U-3, then v0.5.0: V5-00 (post the scope table, then continue), the V5 cars, TRI-050,
     TAG-050 (A12).
 While a car waits on CI or review, prepare the next car whose deps are met.

YOU MAY (14.1)
 - merge PRs when every A1 condition holds
 - run Rule-8 reviews per A2: Harness first pass <= $0.10, then a Claude final sign-off via
   /isolated-request or its fallbacks. You never sign your own work.
 - push archive tags; close superseded PRs with evidence
 - fully drive the Windows node and the AWS node: ssh, aws_deploy.sh, docker logs, the API, and
   EC2 stop/start for the churn test
 - install APKs on the Pixel and pull its logs passively
 - enable debug logging on all three nodes (A8)
 - tag and publish releases (A10)
 Spend caps are in A9.

YOU MAY NOT
 - drive the Pixel UI: ask the operator for every tap, toggle and onboarding step
 - delete remote branches, force-push, use --admin, or bypass hooks (--no-verify)
 - weaken or skip a test or gate
 - touch a file or worktree you did not create, except through BK-01's tool
 - change a node identity
 - exceed a spend cap

EVERY CAR
 verify step first (DONE-VERIFIED if it already holds on main) -> steps -> acceptance commands ->
 JEV gate (2.3: >= 85, hard gates clear) -> merge under A1 -> main CI green on the merge SHA ->
 REPORT block on #403 and in TRAIN_STATE.md.
 - Delivered means receiver decrypt + durable history + receipt back; never transport ACKs.
 - CI is the primary build verifier. No local build while `python scripts/disk_budget.py` exits 2;
   never two builds at once.
 - Harness: invoke it exactly as recorded in TRAIN_STATE.md at P0-6 (the admitted SCM-local harness).

STOP AND ASK (14.5)
 Triggers: main red after a merge; a Rule-8 REJECT; a secret in a diff; any Windows/AWS identity
 change; a spend cap would be exceeded; a destructive action not in 14.1; any TRI rule failure
 (roll back per 7.7).
 Then: write the inbox report (Type BLOCKED or QUESTION), post it on #403, and continue with any
 other car whose deps are met.
If the repo contradicts the task file: write Type PREMISE-WRONG with the command output and stop
that car.
Begin with READ FIRST.
```

**RESUME** (each new Freebuff session):
```
Resume the SCMessenger merge train (Freebuff lane).
Read C:/Users/SCM/Documents/GitHub/scm-train-state/TRAIN_STATE.md, then sections 0-3 and 14 of
HANDOFF/freebuff/queue/V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md, and the section for the next car.
Read the cross-session updates: gh pr view 403 --comments.
Re-verify the last recorded state with commands before continuing:
gh pr view <n> --json state,mergeCommit; git log -1 origin/main; the last JEV output.
Same approvals (section 14) and the same hard stops (14.5).
```

**TRI-040** (each run or re-run):
```
Run TRI-040 per section 7 of HANDOFF/freebuff/queue/V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md on
commit <C>. Windows and AWS: you drive (AWS only if the H-6 grant is recorded in FREEBUFF.md; drive
the AWS API through ssh to 127.0.0.1:9876). Pixel: install -r and passive log pull only; print each
operator action from the 7.4 matrix and wait for "DONE <case>". Finish with tri3.py (R1-R9 on all
75 messages), log_judgment on each log with the frozen scm-ops-log-v1 pack, and the TRI-040 JEV
evidence. Any rule failure: BLOCKED inbox report, roll the nodes back to the KEEP builds, stop.
Partial re-runs never count.
```
