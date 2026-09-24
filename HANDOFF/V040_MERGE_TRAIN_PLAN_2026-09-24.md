# SCMessenger 0.4.0 merge train -- execution plan (2026-09-24)

Status: **Plan only. Nothing in this document has been executed.** No merge, no
branch deletion, no worktree removal, no tag, no node restart happened while
writing it. The next pass executes it mechanically, leg by leg.
Author lane: Freebuff (plan derived from the completed audit and the
2026-09-24 branch-cleanup evidence). Merge authority: Windows orchestrator /
operator (`AGENTS.md` rule 5, `docs/rules/FREEBUFF.md` section 4).
Plan branch: `freebuff/v040-merge-train-plan-20260924` (worktree
`tmp/merge-train-plan-20260924`, based on `origin/main` = `56d66f7190d14fdb78eebfbc005b8fbc6d507d6c`).
Authority stack (conflicts resolve in this order): `AGENTS.md` >
`docs/rules/FREEBUFF.md` > `HANDOFF/V040_FREEBUFF_TRANSITION_2026-09-21.md` >
`HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` >
`HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md` >
`HANDOFF/V040_WORKING_FIRST_PATH_2026-09-20.md` > this plan > the lane train
doc in PR #366. Where this plan contradicts #366, this plan wins on evidence
and says so at the leg.

## 0. Evidence (collected 2026-09-24, 10:59Z onward; every number has a command)

| Fact | Value | Command run in this session |
|---|---|---|
| `origin/main` tip (unchanged since 2026-09-22 push) | `56d66f7190d14fdb78eebfbc005b8fbc6d507d6c` | `git rev-parse origin/main` after `git fetch origin --prune` |
| Open PRs | 47 | `gh pr list --state open --limit 200 --json ...` (saved `tmp/merge-train-evidence-20260924/open-prs-full.json`) |
| Branch protection on `main` | required contexts `Repository Hygiene Checks`, `Lint`, `Rust Linting`, `Test (ubuntu-latest)`; `strict: true`; 0 required approvals; enforce_admins on; force-pushes disabled | `gh api repos/.../branches/main/protection` |
| Repo merge settings | squash + merge + rebase allowed; `delete_branch_on_merge: true` | `gh api repos/...` |
| Last green push on `main` | CI 35674589356, Mobile 35674589429, iOS 35674589467, CodeQL 35674589194, Hygiene 35674589537, Lint 35674589346, Docker Publish 35674589352, Cross 35674589414, all `success` at `56d66f71` on 2026-09-22 | `gh run list --branch main --limit 12` |
| Registered worktrees | 20 (10 on branches, 10 detached repro trees under `tmp/`) | `git worktree list --porcelain` |
| Shared checkout | `glm/canonical-outlier-audit` at `3f41005d`, 52 dirty entries (other sessions' WIP) | `git status --porcelain=v1 \| wc -l` |
| Disk | 3.0 GB free, `disk_budget.py` verdict `BLOCKED`; 11.9 GB `target/` held because the shared checkout is dirty | `python scripts/disk_budget.py`, `python scripts/reclaim_safe.py --durable-ref origin/main` |
| Rule-8 verdicts on file for D1/D9/outbox-sweep | **none**; the dossier `HANDOFF/review/D1_D9_HARNESS_ADVERSARIAL_FINDINGS_2026-09-22.md` (on the D9 branch) states "Gate status (rule 8) -- NOT CLEARED" | `git ls-tree -r --name-only <branch> -- HANDOFF/review \| grep -Ei 'RULE8\|VERDICT\|APPROV'` on `origin/main`, `d9-libp2p-degrade`, `glm/canonical-outlier-audit`, `45b0f8b8` |
| Branch cleanup already done (local only) | 38 old merged branches deleted, 140 old branches renamed to `preservation/2026-09-24/*`, 33 kept; 94.8 MB verified bundle at `tmp/branch-cleanup-20260924/scmessenger-all-refs-20260924.bundle`; nothing pushed | `tmp/branch-cleanup-20260924/cleanup-execute-LIVE.log` |

Rule-8 protocol text that binds this train (from `origin/main`):
`docs/rules/SECURITY_PROTOCOL.md` sections "Adversarial Review Protocol" and
"Reviewer independence" (author cannot sign; panels are not the gate; a verdict
is scoped to the commit reviewed; later commits touching the reviewed modules
void it), `docs/rules/FREEBUFF.md` lines 148-149 ("fresh adversarial review
returning APPROVE from a reviewer that did not author the change ... no
exceptions"), `AGENTS.md` rule 8.

## 1. Merge mechanics (identical for every leg)

Branch protection has `strict: true`, so GitHub will refuse a merge whose head
does not contain the current `main`. That forces the update-branch dance below
for every leg, including the ones GitHub currently reports as
`MERGEABLE/CLEAN` (those are clean only because nothing has merged since they
were opened).

For each leg, in this exact order, from an isolated worktree (never the
shared checkout):

```text
git fetch origin --prune
# 0. rollback point: snapshot the pre-merge main tip on the remote
git push origin "refs/remotes/origin/main:refs/heads/archive/pre-leg-<N>-<YYYYMMDD>"
# 1. refresh the branch ref
git fetch origin "+refs/heads/<HEAD_REF>:refs/remotes/origin/<HEAD_REF>"
# 2. worktree
git worktree add tmp/train-leg-<N> -b train/<N>-<slug> origin/<HEAD_REF>
# 3. blockers (exit 1 = stop; read every line)
scripts/pr_scope.sh <PR>
# 4. bring main in (never rebase a published branch: force-push is forbidden)
git -C tmp/train-leg-<N> merge --no-edit origin/main   # resolve per the leg's conflict rule
git -C tmp/train-leg-<N> push origin HEAD:<HEAD_REF>   # fast-forward only
# 5. wait for the required contexts on the NEW head SHA
gh pr checks <PR> --watch
gh pr view <PR> --json mergeable,mergeStateStatus,headRefOid
# 6. merge (squash, the repo's precedent: #278, #322, #325, #339, #353)
gh pr merge <PR> --squash --delete-branch=false
# 7. record
git rev-parse origin/main            # new tip = the squash commit
# 8. cancel superseded runs on older SHAs (docs/rules/BUILD_AND_CI.md rule 1)
gh run list --branch <HEAD_REF> --json databaseId,headSha,status | cancel all but the merged head
# 9. backup
git bundle create tmp/branch-cleanup-20260924/scmessenger-all-refs-<leg>-<newtip>.bundle --stdin < refs.txt
git bundle verify <bundle>
# 10. local mechanical gates on the clean worktree (cheap, no build)
python scripts/rules_check.py --staged ; python scripts/check_wiring.py ; python scripts/check_queue_status.py
# 11. next leg only after the CI on the new main tip is green for the required contexts
```

Never: `git rebase` on a published branch, `--force`, `git push --delete` on a
shared branch, `gh pr merge` on a PR whose head moved after the review, or any
of this inside `C:\Users\SCM\Documents\GitHub\SCMessenger`.

Squash-merge consequences the executor must accept: the PR head is not an
ancestor of `main` (the merge-train proof is `git show <squash>^{tree} ==
git show <head>^{tree}`), GitHub does not auto-close stacked PRs, and
`pr_scope.sh <PR>` for any later leg must use the new base.

## 2. Dependency order (why this sequence)

```text
A0 docs (no code) ---+
A1 harness docs ----+
A2 android copy -----+--> A3 #361 D9+D1 (rule-8) --> A4 #364 stop/start+sweep (rule-8) --> A5 #351 (lane rework)
A6 #368/#369 -------+
                           |
                           +--> A7 WPs #352 #349 #355 #356 (rule-8 on #355/#356)
                           +--> A8 #359 docs residue (close)
                           +--> rollout (operator) --> WP5 3-node --> 0.4.0 tag
```

Hard dependencies proven by commit ancestry and file overlap in this session:

- `#364` contains `cc51e41a` (`git merge-base --is-ancestor cc51e41a 45b0f8b8` = 0) and is stacked on `#361` (`63f4a7d7` is the third commit of `#364`). `#361` must land first.
- `#364` also carries `f9770378` (D9 vendored patch) and the docker fix `45b0f8b8`; merging `#361` alone leaves Docker Publish broken for every post-D9 SHA (`HANDOFF/freebuff/V040_0_LANE_UNIFY_MERGE_TRAIN_2026-09-23.md` addendum). Merge either `#361` with the tree of `#364` or `#361` then `#364` immediately; both are in this plan, in that order.
- `#367` is the same AND-SS-001 change as `cc51e41a` on a clean base (single commit `63f761ab` on `56d66f71`; identical `StopStartFloodTest.kt`; `MeshForegroundService.kt` differs by the sweep-free 185/123 delta). After `#364` lands, `#367` is superseded, not merged.
- `#359` contains `7810845b` (D1) which is also inside `#361`; `#359` is CONFLICTING/DIRTY and its 24 commits overlap `#361`'s docs. After `#361` lands, `#359` reduces to a docs residue; the D1 transport commit is dropped, not re-merged.
- `#351`, `#349`, `#355`, `#356` share `MeshServiceViewModel.kt`, `mobile_bridge.rs` and `swarm.rs` with `#364`; they are re-based onto post-`#364` main with the lane owner resolving, and their Rule-8 reviews (for `swarm.rs`) are against the post-merge head.
- `#357` edits `HANDOFF/freebuff/README.md`, which `#366` also edits: `#366` (the train doc) wins the table; `#357`'s four JEV state cells and its `inbox/AND06_...` ruling are re-applied.
- `#368` and `#369` are the same lane with `#369` a superset (WIP snapshot, one failing hygiene check): `#368` lands, `#369` is closed.

## 3. Legs

Phase labels used below: **P1** = the merge legs A0-A9 (section 1
mechanics apply to each); **P2** = worktree and local-branch removal after
the owning leg merges (section 6); **P3** = PR closure and remote-branch
cleanup after the train closes (section 8).

Each leg lists: source ref (remote), pinned head SHA (re-verify before
executing), target, contents, gates, blockers, and the "unless there is a
reason not to" answer.

### A0 -- #366 lane-unification train doc (docs)

- Source `origin/freebuff/lane-unify-040-train-20260923` = `55b073fa79dcfea9361b05d48bb10d2268e4cf69` -> target `origin/main`.
- Contents: adds `HANDOFF/freebuff/V040_0_LANE_UNIFY_MERGE_TRAIN_2026-09-23.md` only. 19/19 checks SUCCESS, MERGEABLE/CLEAN, 2 commits, 1 file, no gated file, no overlap with any other train PR.
- Gates: `pr_scope.sh 366` exit 0; required contexts on the updated head; `rules_check.py`.
- Blocker: none known.
- Reason not to: none. This is the lane's own train doc; it is superseded by this plan but still useful history.

### A1 -- #362 Harness lane state (docs)

- Source `origin/claude/harness-lane-state-2026-09-22` = `d33d18f0267164732cb0dc74790ccca3b8f30179` -> `origin/main`.
- Contents: append-only notes in `HANDOFF/CTO_STATE.md` and `HANDOFF/CEO_STATE.md`. 19/19 SUCCESS, CLEAN, 1 commit, 2 files.
- Overlap: `CTO_STATE.md` is also touched by `#359`; land before `#359` (or accept `#359`'s version on conflict).
- Worktree: `C:/Users/SCM/Documents/GitHub/SCMessenger-harness-state` is checked out on this branch and is clean; it is removed only in phase P2 after the merge.
- Reason not to: none.

### A2 -- #358 doctrine rows and Android copy

- Source `origin/freebuff/canonical-doctrine-rows-20260921` = `24e6d93d24ce67c7534deadbcfb0999232fbbc7c` -> `origin/main`.
- Contents: 2 commits, 5 files (`DiagnosticsReporter.kt`, `network_security_config.xml` XML comment fix that unblocked aapt2, three docs). 31/31 SUCCESS, CLEAN. No overlap with train PRs.
- Gates: `pr_scope.sh 358`; Android Debug APK must be SUCCESS on the updated head (it is the aapt2 fix).
- Reason not to: none.

### A3 -- #361 D9 vendored libp2p-swarm graceful degrade + D1 two-tier per-peer cap (Rule-8)

- Source `origin/d9-libp2p-degrade` = `63f4a7d73ef05ace437b54d11c14a256c71eb923` -> `origin/main`. Worktree `C:/Users/SCM/Documents/GitHub/wt-d9-degrade` (clean).
- Contents (30 commits): `f9770378` vendored `libp2p-swarm-0.48.0` with the two `unreachable!()` sites converted to logged drops, `[patch.crates-io]` in Cargo.toml/lock, `9bc7a4c0`/`febdac8f` vendor test/doc-test trimming, `7810845b` D1 per-peer cap (`behaviour.rs`, `per_peer_cap.rs`, `swarm.rs`), FFI snapshot regeneration `63f4a7d7`, plus the OC-lane docs, node-model rules, runbooks, audit docs and tickets (D2, D3, D5/D6, D8, D9, JEV dogfood run). Gated files: `core/src/transport/{behaviour,mod,per_peer_cap,swarm}.rs` + the vendor tree.
- Gated checks at head: 32 SUCCESS, 1 FAILURE (`Android JVM Unit Tests`, run 35787291582): `MeshServiceViewModelTest > toggle during STOPPING sends explicit start instead of becoming a no-op` and `... during STARTING sends stop instead of becoming a no-op`. These are pre-existing on the branch (the same two fail on `#364`), introduced with the `MeshServiceViewModel.kt` change in `770ded22`, and are the one CI blocker.
- Blockers, in order:
  1. **Rule-8 APPROVE on file for the D1 + D9 diffs at head `63f4a7d7`**, signed by a reviewer who did not author them, routed as the dossier's "Path to clear" describes (crypto-security-auditor subagent, read-only fable(high) via `/scmorc`, or deepseek-v3.2/v4-pro:cloud). The dossier at `HANDOFF/review/D1_D9_HARNESS_ADVERSARIAL_FINDINGS_2026-09-22.md` (on this branch) is the reviewer's input; its F2 (activity-map leak, LOW), F3 (accepted ceiling trade-off) and F5 (WASM parity) are the named focus points, and the reviewer must re-verify the byte-identical matched-side claim in `vendor/libp2p-swarm-0.48.0/src/handler/either.rs`. The verdict file lands in `HANDOFF/review/` on the branch and is named with the head SHA. The Freebuff author cannot sign (SECURITY_PROTOCOL "Reviewer independence").
  2. **Android JVM Unit Tests green**: the lane owner decides whether the two toggle expectations or the ViewModel code is wrong, fixes it on the branch, and the Rule-8 verdict is re-pinned to the new head if `core/src/transport/` or the vendor tree changed. A ViewModel-only fix keeps the D1/D9 verdict valid; record the post-hoc delta and its author in the merge record either way.
  3. `pr_scope.sh 361` exit 0 (it will flag >20 commits and the gated files; that is expected, the Rule-8 verdict is the unblock).
- Reason not to: three reasons exist today (no Rule-8 verdict, red Android lane, >20 commits). After 1 and 2, the only remaining reason is commit count, which is inherent to a 30-commit lane branch and is answered by the squash-merge tree proof.

### A4 -- #364 mesh stop/start serialization + periodic outbox sweep (Rule-8)

- Source `origin/freebuff/and-stopflood-outbox-sweep-040` = `45b0f8b8a8019f94b9c39105e45a84315ef314b7` -> `origin/main`. Worktree `C:/Users/SCM/Documents/GitHub/wt-ss-outbox-040` (clean).
- Contents (3 commits on top of `#361`): `cc51e41a` AND-SS-001 (lifecycle mutex, stop coalescing, `StopStartFloodTest.kt`, `MeshServiceViewModel.kt` debounce) + OUTBOX-SWEEP-001 (periodic re-flush of due outbox entries for connected peers in the native swarm loop, `integration_outbox_flush_reconnect.rs` regression), `bdb5b256` fmt-only, `45b0f8b8` docker/DOCKER-VENDOR-001. Gated files: `core/src/transport/swarm.rs` (sweep) and `core/src/mobile_bridge.rs` (not gated). Android/Kotlin: no rule-8; `check_wiring.py` required.
- After A3, update the branch: `git merge origin/main` into the branch (fast-forward push). The `swarm.rs` conflict with A3's D1 trim code is expected: keep both, sweep interval 60-120 s per the ticket, due-check + receipt idempotency.
- Gated checks at pinned head: 32 SUCCESS, 1 FAILURE (same two `MeshServiceViewModelTest` toggle tests, run 35894514034). The same fix as A3 blocker 2 applies.
- Blockers: (1) A3 merged; (2) Rule-8 APPROVE for the post-A3 head covering the sweep (`core/src/transport/swarm.rs`) and the vendor/docker delta; the D1/D9 verdict from A3 is scoped to `63f4a7d7` and is void for the sweep; (3) `Android JVM Unit Tests` green; (4) `pr_scope.sh 364` exit 0.
- Reason not to: today the branch is red and its Rule-8 scope overlaps `#361`'s. After A3 and a fresh sweep verdict, none.

### A5 -- #351 bounded stop teardown (lane rework, not a blind merge)

- Source `origin/freebuff/android-stop-teardown-timeout` = `db475b23d1239ac3aed326a19a9f6c15285a3607` -> `origin/main`, re-applied onto post-A4 main by the lane owner. `#351` has no worktree; it is fetched into a fresh leg worktree.
- Contents: `ea89f9be` bounded teardown (timeout wrapper), `db475b23` release the core past stop so stop->Start reopens the store (MESSAGE-STORE-LOCK-001), `32ca5818` + `d104572b` queue ticket and a hardened acceptance script, `core/src/{contacts_bridge,mobile_bridge,store/backend}.rs` and four Android files, `scripts/check_unbounded_runblocking.py` + tests, `.github/workflows/mobile.yml` acceptance step.
- Expected conflicts after A4: `MeshForegroundService.kt` (timeout goes around the coalesced teardown), `MeshServiceViewModel.kt`, `mobile_bridge.rs`, `MeshRepository.kt`, `HANDOFF/freebuff/README.md`. Resolution rule: A4's serialization semantics win; `#351` contributes only the bounded wait and the store release.
- Gates: 33 checks (all SUCCESS at pin, BEHIND); `pr_scope.sh 351`; `check_wiring.py`; no rule-8 (no gated file) unless the resolution pulls `core/src/transport/` in, in which case a fresh verdict is required.
- Reason not to: `#364` supersedes its stop/start half; merging the old head would reintroduce the flood. Rework first.

### A6 -- #368 harness admission and staged rollout (docs + scripts)

- Source `origin/recovery/harness-plan-wip-20260924` = `bac5d7537dbfc7ab5eae69376c72a01b88e8a619` -> `origin/main`. 19/19 SUCCESS, CLEAN, 1 commit, 4 docs files.
- `#369` (`recovery/harness-plan-snapshot-20260924` = `d56a6222b9992d52d50bb664701d67317583c20b`, draft, 1 check FAILURE `Repository Hygiene` = trailing whitespace in the added `scripts/harness_admission*.py`, run 35958427849) is the WIP snapshot on top and is **closed, not merged**; the uncommitted remainder stays in `C:/Users/SCM/Documents/GitHub/SCMessenger-v040-harness-plan` (13 dirty entries) and in `tmp/branch-cleanup-20260924/wip/SCMessenger-v040-harness-plan/`.
- Worktree `tmp/harness-plan-snapshot-20260924` is on `#369`'s branch and has one dirty file (`scripts/harness_admission.json`); it is not touched by this train and is removed only after the owner decides.
- Overlap after A0/A1: `HANDOFF/freebuff/README.md` and `docs/rules/BUILD_AND_CI.md` also change in `#351`/`#364`/`#361`. Resolution: `#361`'s CI-primary doctrine text wins for `BUILD_AND_CI.md`; `#366`'s train table wins for the README; `#368`'s admission sections are re-applied around them.
- Gates: `pr_scope.sh 368`; `rules_check.py`; the hygiene check on the updated head.
- Reason not to: none after conflict resolution.

### A7 -- V050 work packages, in WP order (post-A4 main)

All four are re-based onto post-A4 `main`; `swarm.rs` and `mobile_bridge.rs` conflicts with A4 are resolved by the lane owner. WP3 (`#355`) and WP4 (`#356`) are gated (`swarm.rs`) and each needs its own Rule-8 verdict at its post-merge head.

| Order | PR | Source ref = head SHA | Contents | Overlap with A4 | Extra gate |
|---|---|---|---|---|---|
| 1 | #352 | `origin/freebuff/v050-wp1-identity` = `6b5b08f37a267c5929bb93dce5770ae6d01db049` | WP1 identity regression battery + hash-confusion guard (`core/src/iron_core.rs`, 2 commits) | none | the PR's own "JEV row fails, needs a ruling" (AND-06 cutover prerequisite) must be dispositioned by the operator before merge; not gated |
| 2 | #349 | `origin/freebuff/v050-wp2-routing-feed-all-transports` = `6a13352031352ff356eec22d839d765559145bf3` | WP2 routing feed from every data-link transport (`core/src/mobile_bridge.rs`, 4 commits incl. a main merge) | `mobile_bridge.rs` | not gated; `check_wiring.py`; 33/33 green |
| 3 | #355 | `origin/freebuff/v050-wp3-inbound-completeness` = `dd146c5df1f0eab587df3c1151e28b7ba8d2cc4a` | WP3 one auto-subscribe decision for both gossip loops (`core/src/transport/swarm.rs`, 1 commit) | `swarm.rs` | **Rule-8** |
| 4 | #356 | `origin/freebuff/v050-wp4-delivery-truth` = `4f87d52fe447f55fe6e37e0ed11b3da7e2113f1b` | WP4 watchdog predicate + delivery-truth refusal (`cli/src/{lib,main}.rs`, `cli/src/watchdog.rs`, `swarm.rs`, roundtrip test, 2 commits) | `swarm.rs` | **Rule-8**; 33/33 green |

### A8 -- #359 canonical docs residue (close, do not merge)

- `origin/glm/canonical-outlier-audit` = `3f41005d1a6a2b9b608dd3960640a642c92f792e`; CONFLICTING/DIRTY; 24 commits; contains `7810845b` (D1, already inside `#361`), the canonical audit package, master/working-first/implementation-plan docs, `HANDOFF/harness/`, and the disk-governance commits.
- After A1 and A3 land: the executor runs `git merge-base --is-ancestor 7810845b <squash-of-#361>` and, if true, updates the branch with main, drops the D1 transport commit (rule 16: restore the docs, not a duplicate code change), and opens a docs-only PR for the residue (audit index, master-plan refresh, `HANDOFF/harness/*`, queue/ticket files). If false, stop and escalate: D1 would not be on main.
- If the residue cannot be made conflict-free within the operator's patience, the alternative is `git diff origin/main...3f41005d -- '*.md' 'HANDOFF/**' 'SHIP_PLAN.md'` applied by hand onto a docs-only branch, and the branch is closed. Either way, the `Android`/`core` WIP sitting uncommitted in the shared checkout on top of `3f41005d` is **not** part of this plan; it stays with its owner (backup: `tmp/branch-cleanup-20260924/wip/SCMessenger/`).
- Gates: `pr_scope.sh 359`; Rule-8 only if any transport file survives the residue (it must not).

### A9 -- post-train hygiene PRs (optional, after A8)

- `#357` (`089a7fc30fe8f996e89c4e9372e8c71f692dc930`, 3 commits, 19/19): re-apply its four README cells and `inbox/AND06_...` ruling onto the post-A0/A6 README; then merge.
- `#354` (`cb617149303a20895791e00c24618d8273069713`, 1 commit, adds `HANDOFF/freebuff/jev/{WP1,WP2}_state_2026-09-21.json` + inbox ruling, 19/19): merge, no conflicts.
- `#316` (`7b6f15bf2925eed91db3ed79517ca7ba29757692`, docs diagnosis, 19/19): close with "delivered by #364 (`cc51e41a`) and ticket `OUTBOX_NO_PERIODIC_RETRY_SWEEP_2026-09-23`" (that ticket is already inside `#364`).
- `#329` (`e1b9263db43ab9e91d6d58bc02f16ac7e222dfb4`, queue status reconcile, 19/19, DIRTY): update with main, re-check, merge if the three status lines still match main.
- `#360` (`e0dea50147ac77defd0dab5354b24c645ed86f7f`, DIRTY): the handoff doc path `Harness/handoff/...` no longer exists on main (deleted by the audit branch); the lane owner relocates it under `HANDOFF/harness/` and re-lands `scripts/harness_gate.py` (version floor). Not urgent; conflicts with A6 on `harness_gate.py`.
- `#363` (draft, `ops/openclaw/*`, 27 green): leave draft; review deployment paths before rollout.
- `#362`'s worktree and `#351`'s absence are handled in P2.

## 4. Rule-8 dispatch (who signs what, at which SHA)

| PR | Modules to review | Verdict SHA | Reviewer must be |
|---|---|---|---|
| #361 | `core/src/transport/{behaviour,mod,per_peer_cap,swarm}.rs`, `vendor/libp2p-swarm-0.48.0/**`, `Cargo.toml`/`Cargo.lock` (patch + 24 dev-deps) | head after the Android-test fix | not the D1/D9 author (Freebuff lane), not a panel-only pass |
| #364 | `core/src/transport/swarm.rs` (sweep), `core/tests/integration_outbox_flush_reconnect.rs`, `docker/Dockerfile` (vendor copy) | post-A3 head | same uninvolved reviewer is acceptable for a second, separately scoped verdict |
| #355, #356 | `core/src/transport/swarm.rs` | post-A4 head | uninvolved |
| #351 | none gated (verify the rework did not pull `core/src/transport/` in) | n/a | n/a |
| #228, #218, #216, #220 (if ever revived) | their gated files | n/a in this train | n/a |

A verdict is void for any later commit touching the reviewed modules
(SECURITY_PROTOCOL "Reviewer independence"). The merge record for each gated
leg names: head SHA reviewed, squash SHA merged, tree equality, verdict file,
and any post-verdict delta with its author.

## 5. Rollback points

- Before every leg: `archive/pre-leg-<N>-<date>` on origin at the pre-merge `main` tip (step 0 above). These are snapshots, not resets; nothing rewrites `main`.
- After every leg: the squash commit SHA is recorded in the execution log; rollback is a `gh pr create` revert PR ("Revert #<PR>"), reviewed and merged like any other change.
- After every leg: a full `git bundle` of all refs under `tmp/branch-cleanup-20260924/` (verified) so the pre-train state is reconstructible even if remote branches are later deleted.
- Node rollouts are out of scope here; when they happen, binaries are staged under `tmp/radio-<sha>/` and the previous binary is kept as the rollback (AGENTS.md rule 17).

## 6. Branch and worktree protection (20 registered worktrees at plan time)

| Worktree | Branch | Disposition |
|---|---|---|
| `SCMessenger` (shared) | `glm/canonical-outlier-audit` @ `3f41005d`, 52 dirty | **protected**: never used for merges, rebases or checkouts; WIP belongs to other sessions |
| `SCMessenger-harness-state` | `claude/harness-lane-state-2026-09-22` (clean) | after A1: `git worktree remove` then local branch `-d` |
| `wt-d9-degrade` | `d9-libp2p-degrade` (clean) | after A3 and A4: remove, then `-d` |
| `wt-ss-outbox-040` | `freebuff/and-stopflood-outbox-sweep-040` (clean) | after A4: remove, then `-d` |
| `wt-ss-docs` | `freebuff/lane-unify-040-train-20260923` (clean) | after A0: remove, then `-d` |
| `tmp/android-artifact-wt`, `tmp/android-artifact-main-wt` | `freebuff/android-third-node-artifact-20260924` = `#367` head, `...-main-20260924` = `#367` | after A4 closes `#367`: remove, then `-d` |
| `SCMessenger-v040-harness-plan` | `freebuff/v040-v050-harness-plan` @ `bac5d753`, 13 dirty | **protected**: owner decides; A6 lands the committed part |
| `tmp/harness-plan-snapshot-20260924` | `recovery/harness-plan-snapshot-20260924` @ `d56a6222`, 1 dirty | **protected**: `#369` closed as snapshot; owner decides |
| `tmp/jev-completion-gate-20260924` | `orchestrate/jev-completion-gate` @ `f546bb22` (clean, green CI 35967990903) | not part of this train; orchestrator decides |
| `tmp/repro-*` (10 detached: repro-candidate, repro-candidate2/3/4/5, their -clean twins, repro-clean) | detached; 6 have staged changes, 2 have unpushed commits | **protected**: never prune, never remove |

Rules: a branch checked out in a worktree is never deleted (git refuses it);
removal order is always worktree first, then `git branch -d` (never `-D`); no
remote branch is deleted except through the P3 procedure.

## 7. Main synchronization

- `main` has not moved since 2026-09-22 (`56d66f71`, 8 green workflows). Every leg's update-branch merge is what keeps it that way under `strict: true`.
- The active checkout is not a branch of `main`; do not "fast-forward the shared checkout" to main as part of this train. Its 52 dirty entries are other sessions' work.
- After the last leg, `main` is the new tip; the Windows node, the cloud nodes and the Pixel are rolled to CI artifacts of that tip per the working-first path section 2 (operator-gated, not in this plan).

## 8. Remote cleanup (P3, after every leg is merged and the plan is closed)

1. `gh pr close <PR> --comment "<reason>"` for: #369 (superseded by #368), #367 (superseded by #364), #316 (delivered by #364), #359 (after A8 residue), #360 (if relocated), #303/#302/#301/#300/#299/#298 (hold, working-bar ruling), #227/#220/#216 (reachability line), #218 (dead code, do not merge), #209 (superseded), #103 (action/cache v3 -> v4, dependabot), #141 (upload-artifact v7 DIRTY; rebase or close), #170 (free API lanes; Lint/Rust red since 2026-08-16), #156 (docker suite non-blocking; stale, Lint/Rust red), #207 (docs placeholder), #178/#208 (Mac lane owns).
2. `git fetch origin --prune` on every clone; GitHub's `delete_branch_on_merge` has already removed merged heads.
3. Remote branches that remain and are >24 h old are pruned with the same procedure as the 2026-09-24 local cleanup: bundle first, then `gh api -X DELETE repos/.../git/refs/heads/<branch>` one at a time with a printed list and operator approval. The Freebuff lane never deletes a branch without a human in the loop.
4. Local branches follow the worktree table in section 6.
5. Re-run the full bundle and manifest (`tmp/branch-cleanup-20260924/`) so the post-train state is itself backed up.

## 9. Target reclamation (nondestructive, only after clean/merged proof)

- Gate: `python scripts/reclaim_safe.py --durable-ref origin/main` must print the worktree under "SAFE to reclaim target/". SAFE means clean, no unpushed commits, HEAD is an ancestor of `origin/main`, and no process runs from the tree. Today it prints `(none)` for all 20 worktrees because the shared checkout is dirty and the detached repro trees have unpushed commits.
- Action: `python scripts/reclaim_safe.py --reclaim --durable-ref origin/main` (plus `--reclaim-shared-target` / `--reclaim-emulator-state` if the survey shows bytes). Nothing else deletes `target/`. `rm -rf target` is forbidden; the live node binary must never live in `target/` (the Windows node runs from `C:/Users/SCM/.local/bin/scmessenger-cli.exe`).
- Order: commit and push the WIP -> merge -> prove -> reclaim. Never reclaim to make room for the merges this train needs; 3.0 GB free is enough for a worktree checkout and a branch update because objects are shared.
- The two legacy scripts `scripts/delete_merged_branches.sh` and `scripts/verify_branch_merges.sh` hard-code 2026-02 branch names; do not run them.

## 10. Operator decisions this plan cannot make

1. Who signs Rule-8 for #361/#364/#355/#356 (an uninvolved reviewer must be named and dispatched; the author cannot sign).
2. Whether the two failing `MeshServiceViewModelTest` toggle expectations or the ViewModel code is correct (lane owner decides, then the fix).
3. The `#352` JEV-row ruling (AND-06 cutover prerequisite) and the WP1-WP4 JEV evidence sufficiency.
4. Whether #369's WIP snapshot scripts (`harness_admission*.py`) are wanted on main at all.
5. Pixel install is a fresh install (identity loss accepted by the operator 2026-09-23); the keystore secret remains unset.
6. Disposition of the shared checkout's uncommitted WIP (not this plan's to touch).

## 11. Execution record template (fill one per leg)

```text
LEG <id> PR <n>
  head reviewed : <sha>      verdict file: HANDOFF/review/<...>  reviewer: <name>
  update merge  : <merge-sha on branch>   pushed: fast-forward
  required ctx  : Hygiene/Lint/Rust Linting/Test(ubuntu) = SUCCESS on <new head>
  squash commit : <sha>      tree proof: show <squash>^{tree} == show <head>^{tree}
  rollback ref  : archive/pre-leg-<n>-<date> = <pre-merge main>
  bundle        : tmp/branch-cleanup-20260924/scmessenger-all-refs-<leg>-<sha>.bundle (verified)
  superseded runs cancelled: <run ids>
  new main tip  : <sha>      next leg unblocked when its CI is green
```

## 12. Disposition of all 47 open PRs (mechanical, from `open-prs-full.json`)

`H` = head SHA (pinned 2026-09-24), `mb` = merge-base with `origin/main`, `C` = commits ahead, `S/F` = check successes/failures at head, `gated` = files under `core/src/{crypto,transport,routing,privacy}/`.

| PR | head ref = H | mb | C | S/F | gated | disposition |
|---|---|---|---|---|---|---|
| 369 | `recovery/harness-plan-snapshot-20260924` = `d56a6222b9992d52d50bb664701d67317583c20b` | `56d66f71` | 2 | 26/1 | no | close (WIP snapshot; trailing whitespace) |
| 368 | `recovery/harness-plan-wip-20260924` = `bac5d7537dbfc7ab5eae69376c72a01b88e8a619` | `56d66f71` | 1 | 19/0 | no | A6 merge |
| 367 | `freebuff/android-third-node-artifact-main-20260924` = `63f761abb6927ab04cbd86cdb99f047a7b6ae5a4` | `56d66f71` | 1 | 31/0 | no | close (superseded by #364) |
| 366 | `freebuff/lane-unify-040-train-20260923` = `55b073fa79dcfea9361b05d48bb10d2268e4cf69` | `56d66f71` | 2 | 19/0 | no | A0 merge |
| 364 | `freebuff/and-stopflood-outbox-sweep-040` = `45b0f8b8a8019f94b9c39105e45a84315ef314b7` | `56d66f71` | 33 | 32/1 | `transport/swarm.rs` | A4 merge (Rule-8) |
| 363 | `codex/openclaw-bridge-ops` = `43b0d371884d406182d07ebd6c278ae55a92ce9d` | `56d66f71` | 1 | 27/0 | no | hold (draft) |
| 362 | `claude/harness-lane-state-2026-09-22` = `d33d18f0267164732cb0dc74790ccca3b8f30179` | `56d66f71` | 1 | 19/0 | no | A1 merge |
| 361 | `d9-libp2p-degrade` = `63f4a7d73ef05ace437b54d11c14a256c71eb923` | `56d66f71` | 30 | 32/1 | 4 transport files + vendor | A3 merge (Rule-8) |
| 360 | `freebuff/harness-version-floor` = `e0dea50147ac77defd0dab5354b24c645ed86f7f` | `1b28c010` | 2 | 6/0 | no | rework (A9) |
| 359 | `glm/canonical-outlier-audit` = `3f41005d1a6a2b9b608dd3960640a642c92f792e` | `629a3eef` | 24 | 6/0 | 4 transport files | A8 docs residue, then close |
| 358 | `freebuff/canonical-doctrine-rows-20260921` = `24e6d93d24ce67c7534deadbcfb0999232fbbc7c` | `56d66f71` | 2 | 31/0 | no | A2 merge |
| 357 | `freebuff/train-status-20260921` = `089a7fc30fe8f996e89c4e9372e8c71f692dc930` | `56d66f71` | 3 | 19/0 | no | A9 merge after README resolution |
| 356 | `freebuff/v050-wp4-delivery-truth` = `4f87d52fe447f55fe6e37e0ed11b3da7e2113f1b` | `56d66f71` | 2 | 33/0 | `transport/swarm.rs` | A7 (Rule-8) |
| 355 | `freebuff/v050-wp3-inbound-completeness` = `dd146c5df1f0eab587df3c1151e28b7ba8d2cc4a` | `56d66f71` | 1 | 33/0 | `transport/swarm.rs` | A7 (Rule-8) |
| 354 | `freebuff/train-20260921` = `cb617149303a20895791e00c24618d8273069713` | `56d66f71` | 1 | 19/0 | no | A9 merge |
| 352 | `freebuff/v050-wp1-identity` = `6b5b08f37a267c5929bb93dce5770ae6d01db049` | `6490bed3` | 2 | 33/0 | no | A7 after JEV ruling |
| 351 | `freebuff/android-stop-teardown-timeout` = `db475b23d1239ac3aed326a19a9f6c15285a3607` | `6490bed3` | 4 | 33/0 | no | A5 rework then merge |
| 349 | `freebuff/v050-wp2-routing-feed-all-transports` = `6a13352031352ff356eec22d839d765559145bf3` | `23bebebe` | 4 | 33/0 | no | A7 merge |
| 329 | `freebuff/status-reconcile-20260919` = `e1b9263db43ab9e91d6d58bc02f16ac7e222dfb4` | `b3018764` | 1 | 19/0 | no | A9 merge if still true |
| 316 | `freebuff/outbox-retry-fix` = `7b6f15bf2925eed91db3ed79517ca7ba29757692` | `b529011b` | 1 | 19/0 | no | close (delivered by #364) |
| 303 | `workahead/merge-plan-handoff-20260916` = `da3438ea7351d3da8ae1170c8b1af46e17b0d2b1` | `20cfb91d` | 4 | 19/0 | no | hold (draft) |
| 302 | `workahead/android-install-qr-payload` = `b107084befc3081e4e8b2fe6af9ace72949becd7` | `fb3ce1ae` | 5 | 33/0 | no | hold (draft) |
| 301 | `workahead/ios-apk-link-share` = `64e66b73fc9ab3ab22bdb2acf757a9a807013d45` | `fb3ce1ae` | 5 | 33/0 | no | hold (draft) |
| 300 | `workahead/android-apk-host-hardening` = `73730e2f6d4ff2f5803f784bf63af193245de781` | `fb3ce1ae` | 5 | 33/0 | no | hold (draft) |
| 299 | `workahead/android-persist-join-seeds` = `1291e91872b67e4760d570c140a89db9192f0189` | `fb3ce1ae` | 5 | 32/1 | no | hold (draft) |
| 298 | `workahead/android-joinmesh-gms-gate` = `8af8c607d9f653601114a820493f76a1f0aa5725` | `fb3ce1ae` | 4 | 33/0 | no | hold (draft) |
| 228 | `cto/ci-hardening-2026-08-23` = `ffa10281638ea3eec499c5165bdbcbe28b8dd68f` | `fb3ce1ae` | 3 | 12/20 | 14 files | close/reopen (drifted) |
| 227 | `cto/android-degraded-storage-wiring-2026-08-22` = `2e9a9d9671e30955f942483fc678b77dace62044` | `ef58aa66` | 1 | 33/0 | no | hold (reachability line) |
| 220 | `cto/android-reachability-only-2026-08-22` = `92b8ec12378c42e2a40244bf17b37f0a7e540bbd` (base `feat/identity-id-unification`) | `63c99bcd` | 35 | 5/2 | no | close (superseded) |
| 218 | `cto/panic-self-circuit-guard-2026-08-22` = `c4108d99a072b0b4288352bab5c5b95c6ce3522e` | `63c99bcd` | 1 | 31/2 | `transport/addr_filter.rs` | hold (dead code by its own admission) |
| 216 | `cto/android-wiring-restore-2026-08-22` = `b2020a20fbbd333c7f59211997d11e6c97513187` (base `feat/identity-id-unification`) | `63c99bcd` | 34 | 4/3 | no | close (superseded by #220/#227) |
| 214 | `dependabot/github_actions/github/gh-aw-0.86.2` = `293c9c69632d86838c0d86d6104ce98b669ce43a` | `20cfb91d` | 5 | 27/0 | no | merge (post-train) |
| 213 | `dependabot/gradle/android/androidx.hilt-hilt-navigation-compose-1.4.0` = `37d29f1ab3f4f4af540f4a101658baa1fe597488` | `9de879b9` | 1 | 23/2 | no | open (build.gradle conflict with 210/108/107/106) |
| 212 | `dependabot/github_actions/actions/stale-11` = `9ad902c924cd693891709655dc59e1fd4b46315d` | `20cfb91d` | 5 | 27/0 | no | merge (post-train) |
| 211 | `dependabot/github_actions/actions/setup-java-5` = `1f4e5a709d239f04a4bbb5336bcffe58ffc797f2` | `20cfb91d` | 5 | 27/0 | no | merge (post-train) |
| 210 | `dependabot/gradle/android/org.jetbrains.kotlinx-kotlinx-coroutines-test-1.11.0` = `51f41d87b265e2e62a228a445e4f654e2cc788ae` | `9de879b9` | 1 | 24/1 | no | open (build.gradle conflict) |
| 209 | `feat/identity-id-unification` = `594bae1860822f108512a6450010a210a3b02d9b` | `63c99bcd` | 29 | 6/0 | no | close (superseded; DIRTY) |
| 208 | `gpt/v050-parity-burndown` = `11f3e5e93f2a8bc3ddab5e61f573b6dbdb5e1d4c` | `63c99bcd` | 25 | 21/6 | no | Mac lane (DIRTY) |
| 207 | `gpt/apple-v1-cao-continuity-2026-08-21` = `b02ec7d9d33eb85c135182c30ee2f797f4f5bd8c` | `48303050` | 1 | 20/1 | no | close (placeholder) |
| 178 | `gpt/ios-macos-launch-debug-20260810` = `c1b457bbb1cecf0aeb63fb4965a5620a0e8415ac` (base itself) | `fa835584` | 10 | 3/3 | no | Mac lane |
| 170 | `gpt/free-api-lanes-20260815` = `d85eb6b970c9d6a64923a051b77a2144da721961` | `ef431acc` | 1 | 24/2 | no | close or owner rebase |
| 156 | `ci/docker-suite-nonblocking` = `a7d7cc490d24d4fdcb2d1f9e8f528557878b96ee` | `ebf5411b` | 1 | 24/2 | no | close (stale; Lint/Rust red) |
| 141 | `dependabot/github_actions/actions/upload-artifact-7` = `23cf7b61f0d3d828a5bd2f37fcfa071e65179a5e` | `589479c3` | 2 | 11/0 (16 cancelled) | no | close/reopen (DIRTY) |
| 108 | `dependabot/gradle/android/androidx.core-core-ktx-1.19.0` = `b1e14a0d542e049bef64c74ed5d6744965b12f85` | `6edeea09` | 1 | 11/8 | no | open (build.gradle conflict) |
| 107 | `dependabot/gradle/android/io.mockk-mockk-android-1.14.11` = `d51154a843d154ad977e8d4ef6b21a1255a8976f` | `a78fa218` | 1 | 10/9 | no | open (build.gradle conflict) |
| 106 | `dependabot/gradle/android/androidx.lifecycle-lifecycle-service-2.11.0` = `3eee8707795084a80b51cde601e75d1569f33b11` | `6edeea09` | 1 | 11/8 | no | open (build.gradle conflict) |
| 103 | `dependabot/github_actions/actions/cache-6` = `bfba815477152bac9bac66e98c24e8a6f8110805` | `0117c1ec` | 1 | 21/2 | no | close (cache v3 -> v4) |

Count check: the table has 47 rows, matching the 47 open PRs reported by `gh pr list --state open --limit 200`.

## 13. What "done" for this plan means

Every leg A0-A9 has a recorded squash SHA, tree proof, rollback ref, verified
bundle and green required contexts on the resulting main; Rule-8 verdicts are
on file for A3, A4, #355, #356; P1-P3 are executed with the operator present;
`main` is at the final tip with a green push run; and the post-train bundle
exists. The tag decision is the operator's and is not part of this plan.
