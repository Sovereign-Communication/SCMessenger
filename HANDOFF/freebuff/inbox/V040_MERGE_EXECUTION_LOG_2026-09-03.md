# V040 Merge execution log (2026-09-03)

Status: IN PROGRESS. Each entry records the executed merge, its per-merge gate
result, and the Rule-8 artifact citation. Sequence per
V040_POST_VALIDATION_MERGE_PLAN_2026-09-03.md section 2. Execution seat:
Windows host (operator-directed, orchestrator authority per AGENTS.md 5b).

## 1. PR #264 (T12 CI pacing) -- MERGED 2026-09-03 10:44:32Z

- Branch: freebuff/v040-t12-ci-pacing. Merged head: 23264902
  (updated from ec177fd9 to satisfy strict branch protection; main merged in).
  Scope: CI workflow files only. Rule-8 required: NO (verified scope).
- Gates: tip verified live; not-in-main confirmed (git merge-base
  --is-ancestor => exit 1); required checks on merged head 23264902 all
  SUCCESS (Repository Hygiene, Lint, Rust Linting, Test ubuntu-latest).
- Method: squash merge (repo convention). Main: 67d19d3c -> a6e9ece1.
- Confirm: gh pr view 264 => state=MERGED, mergeCommit=a6e9ece1.
- Note: strict protection required the update-branch head move; CI-only scope
  so no Rule-8 head-fidelity concern.

## 2. PR #271 (T8 restore test) -- MERGED 2026-09-03 11:26:07Z

- Branch: freebuff/v040-t8-restore-test. Merged head: 62a75afa (updated from
  8fc58817; main merged in). Scope: Android test + doc only. Rule-8 required: NO.
- Gates: tip verified live; not-in-main confirmed; required checks on 62a75afa
  all SUCCESS (Hygiene, Lint, Rust Linting, Test ubuntu-latest -- the latter
  finished after a heavy runner-backlog wait, started 11:02:31Z).
- Method: squash merge. Main: a6e9ece1 -> 36fc1faa.
- Confirm: gh pr view 271 => state=MERGED, mergeCommit=36fc1faa.

## 3. PR #269 (T14 preexisting, transport) -- MERGED 2026-09-03 15:10Z

- Branch: freebuff/v040-t14-preexisting-fixes, updated head b7a9d9a3 (main
  merged in; reviewed content = single b2a7b345 commit, preserved exactly).
  APPROVE on file: HANDOFF/review/V040_T13_F7_REVIEW_QWEN_2026-09-01.md
  (qwen-max, non-author; #269 section "Verdict: APPROVE", comment-only Low
  findings -- no head-SHA pin, verdict valid at the updated head).
- Gates: a) tip b7a9d9a3 verified live MATCH; b) diff present (mergeable);
  c) cherry + b2a7b345 single commit; d) merge-tree --write-tree exit=0, no
  conflicts; e) Rule-8 artifact cited above. Full CI green at b7a9d9a3 (all
  checks COMPLETED SUCCESS incl. Test ubuntu/windows/macos, Android, iOS,
  WASM, FFI Surface).
- Method: squash merge. Main: 36fc1faa -> dc90269e.
- Confirm: gh pr view 269 => state=MERGED; main log shows
  "fix(core): close two unverified Kademlia add_address feeds (V040-T14) (#269)".

## 7. PR #273 (nimble-peer fix) -- MERGED INTO CANDIDATE 2026-09-03 15:12Z

- Branch: freebuff/v040-nimble-peer @ d82978ab, base cto/v040-candidate-2026-09-02.
  Rule-8 PLAIN APPROVE on file (V040_NIMBLE_PEER_REVIEW_QWEN_2026-09-03.md,
  non-author continuous reviewer, verdict @ d82978ab).
- CI fully green at d82978ab: the Android Debug APK job that failed on the
  transient Maven Central HTTP 429 was re-run after the Mobile workflow
  completed -- run 33741858743 concluded SUCCESS. 0 failed, 0 pending checks.
- Method: squash merge into the candidate. Candidate head: 177bd840 ->
  44fee3c4 (squash commit 44fee3c4 = #273's nimble-peer fix onto the candidate).
- Confirm: gh pr view 273 => state=MERGED, mergedAt 2026-09-03T15:12:27Z.

## Pending queue (updated after #269 + #273 landed)

4. PR #268 (T13-F7) -- FLAG-1: awaiting qwen confirm-APPROVE verdict
   (dispatch V040_REVIEW_DISPATCH_268_270_CONFIRM_APPROVE_QWEN_2026-09-03.md)
   or harness-lane confirm pass per V040_HARNESS_LANE_INCORPORATION_2026-09-03.md.
5. PR #270 (T14 ephemeral) -- FLAG-2: same as #268.
6. PR #267 (T13-FDHT gate) -- FINAL APPROVE on file; merges after #268/#270
   (same-file churn ordering, swarm.rs/ledger_entry.rs). Content unchanged by
   #269's landing; re-run gates (incl. merge-tree vs the advanced main) at
   merge time.
8. PR #272 (candidate) -- MERGES LAST. NOTE: candidate head has MOVED to
   44fee3c4 (post-#273). 177bd840 evidence gates the pre-#273 tree; the
   post-#273 focused three-node validation must run at 44fee3c4 with fresh
   CI artifacts (wincli + APK at the new SHA). #272's FINAL APPROVE will need
   a delta addendum for 44fee3c4. #272 currently BEHIND (main advanced via
   #264/#271/#269); 30 CI checks re-running at 44fee3c4, 0 failed.
   FLAG-4 (validation at final head) + FLAG-5 (multi-transport re-review
   verdict on file) both still gate the merge.

## 2026-09-03 15:20Z -- final-head prep at 44fee3c4 (no legs run, nothing dispatched)

- #272 CI at 44fee3c4: 7 workflow runs triggered 15:11Z (30 checks, 0 failed).
  Hygiene/Auto Label/Lint completed SUCCESS; iOS Build & Test + Cross
  in_progress; CI + Mobile queued (runner backlog). CI and Mobile carry the
  Windows CLI Artifact and Android Debug APK jobs needed for the final-head
  test artifacts.
- Gate documents staged for the moved head (both ASCII-clean, inbox/):
  - V040_REVIEW_DISPATCH_272_DEFERRAL_REREVIEW_44fee3c4_2026-09-03.md
    (deferral re-review brief re-pinned to 44fee3c4; records that #273 does
    NOT touch the UDP/QUIC admission question -- 177bd840..44fee3c4 shows
    zero lines touching listen_port_from_bound_addr / sync_external_address
    / Udp / Quic / tcp-string; swarm.rs hunks only at 3437/5166/5851;
    observation.rs + reflection untouched).
  - V040_CANDIDATE_272_DELTA_ADDENDUM_REQUEST_44fee3c4_2026-09-03.md
    (FINAL APPROVE re-approval request: gate (e) reviewed-SHA == TIP no
    longer holds at the moved head; evidence: git diff --quiet d82978ab
    44fee3c4 exits 0 -- trees byte-identical, so the #273 content carried
    into the candidate is exactly the independently APPROVED d82978ab tree).
- Neither document dispatched -- qwen lane is CTO-driven; harness go remains
  the CEO's pending decision.

## 2026-09-03 16:30Z -- final-head artifacts staged at 44fee3c4 ([WARNING] tree finding)

- #272 CI at 44fee3c4: ALL 7 workflow runs COMPLETED SUCCESS 16:27Z
  (Hygiene, Auto Label, Lint, iOS Build & Test, Cross, CI, Mobile). PR
  rollup: 0 failed, 0 not-completed -- fully green (30 checks).
- Staged (hashes verified post-copy):
  - .codebuff_deploy/wincli-44fee3c4/ (scmessenger-cli.exe 21,979,648 B,
    sha256 27d3afb7...; README-PROVENANCE.txt; cli-provenance.txt)
  - .codebuff_deploy/pixel-apk-44fee3c4/ (app-debug.apk 65,518,111 B,
    sha256 881b6308...; README-PROVENANCE.txt; output-metadata.json --
    com.scmessenger.android debug 0.4.0, versionCode 14)
  - Smoke: exe --help exits 0, banner 0.4.0 (1a208f7).
  - Raw downloads kept in .codebuff_deploy/artifacts-44fee3c4/.
- [WARNING] PROVENANCE FINDING -- the CI artifacts are NOT the pure 44fee3c4
  tree: the pull_request-event checkout builds refs/pull/272/merge =
  1a208f72 ("Merge 44fee3c4 into dc90269e"). Main advanced past this PR's
  base (67d19d3c) via #264/#271/#269, so merge-ref tree (a9926999...) !=
  44fee3c4 tree (1532c0a2...). Delta present in the binaries: #269's
  core/src/transport/swarm.rs change (+43) plus #264 CI files, #271
  test-only file, docs (12 files +260/-33). At 177bd840 the trees were
  identical; they are NOT now.
- Consequence: ci.yml and mobile.yml run ONLY on push-to-main + PR (no
  workflow_dispatch, no candidate-branch push), so GitHub CI CANNOT produce
  an exact-44fee3c4 wincli/APK -- local cargo/gradle builds are the only
  exact-source path. docker-publish.yml DOES have workflow_dispatch, so the
  AWS image can be exact-44fee3c4 via the CTO's dispatch path.
- Deeper implication recorded: 44fee3c4 is the candidate head TODAY, but it
  is NOT the final test tree under the "land everything first" doctrine --
  #268/#270/#267 still merge to main before #272. After they land,
  update-branch #272, and PR CI at the updated head produces EXACT artifacts
  of the true shipped tree (branch head == merge ref once the branch
  contains main). Pending CEO decision: test at 44fee3c4 (as prepared) vs
  wait for the full landing so the test runs once on the true final tree.

## 2026-09-03 19:20Z -- CEO DECISIONS LOCKED + HARNESS LANE GREEN-LIT

1. HARNESS LANE: GO (explicit user confirmation). First-live-use sequence per
   V040_HARNESS_LANE_INCORPORATION_2026-09-03.md executes now: #268/#270
   confirm passes convert to claims manifests, run harness verify --converge
   on the free tier. Verdicts filed to HANDOFF/review/ per the governance
   mapping (V040_*_HARNESS_* convention, panel composition + ledger chain
   cited). Human double-check before filing as Rule-8 evidence.
2. #267 ORDERING: decision delegated to this seat. Decision recorded: KEEP
   PLAN ORDER (after #268/#270). Rationale: the plan orders #267 after
   #268/#270 for swarm.rs/ledger_entry.rs churn ordering; the harness lane
   is now live to produce the #268/#270 verdicts quickly, so the churn
   rationale still holds -- no reason to invert. Revisit only if harness
   verdicts stall materially.
3. TEST TREE: WAIT FOR TRUE FINAL TREE. No 3-node test at 44fee3c4. The
   final tree = after #268/#270/#267 land on main + #272 update-branch +
   delta-addendum verdict at the new head + CI artifacts of the shipped
   tree + AWS image/redeploy at the same head. Test runs once on that tree.

## 2026-09-03 20:54Z -- #268 MERGED; #270/#267 updated; #274 arrived from CTO

- #268 MERGED to main: squash d395e030, 20:53:58Z. Head at merge = 930ff373
  (update-branch of reviewed 7bafe83d; PR-own content preserved byte-intact,
  verified 7bafe83d ancestor). Harness confirm-APPROVE on file
  (V040_T13_F7_CONFIRM_APPROVE_HARNESS_2026-09-03.md). Required 4 checks
  green (Hygiene/Lint/Rust Linting/Test ubuntu); merge-tree exit 0; cherry 2.
- #270 update-branch: 5e0d47eb -> 3c4c0714 (main + #268 merged in; strict
  protection). Reviewed head 6fd0230b preserved as ancestor; PR diff intact
  (15 files, observation.rs +152 / swarm.rs +105). CI re-running.
- #267 update-branch: 80197ef5 -> 17fa959f. Reviewed head preserved as
  ancestor; PR diff intact (16 files, swarm.rs 165, ledger gating). NOTE:
  FINAL APPROVE file pins "@ 80197ef5" explicitly -> delta-addendum required
  before merge (stop-and-report rule). CI re-running in parallel.
- CTO delivered PR #274 (kill 5-min recycle, freebuff/v040-nimble-peer):
  Rule-8 APPROVE (R4 @ 6764e2b0). AUDIT: merge-base w/ candidate = 177bd840;
  d82978ab (pre-squash #273 head) ancestor; 44fee3c4 tree == d82978ab tree
  (byte-identical); merge-tree #274->candidate exit 0. The "44fee3c4 NOT
  ancestor" is squash topology only -- merging applies exactly the 5-file
  delta (+504/-10: swarm.rs 444, cli ledger.rs/main.rs, Cargo.toml/lock
  if-addrs). Content lineage SAFE. CI at 6764e2b0: iOS Build & Simulator
  Test / macOS Native Tests IN_PROGRESS, iOS Build QUEUED (not fully green).
  Merge authority = CEO seat (CTO's own note). NOT merged.
- AWS leg: still NOT done -- latest docker-publish at candidate = 177bd840
  (06:47Z); no run at 44fee3c4; no new redeploy evidence in tmp/run-evidence.
  Image must be built AFTER #274 lands (head will move).

## 2026-09-03 21:21Z -- CI status snapshot (all still in queue-drain)

- #270 @ 3c4c0714: Auto Label + Repository Hygiene SUCCESS; Lint/Cross/iOS
  Build & Test/CI/Mobile queued. Required 4 not yet green.
- #267 @ 17fa959f: Auto Label + Repository Hygiene SUCCESS; Lint/CI/Cross/
  iOS queued. Required 4 not yet green. Merge HELD for delta addendum.
- #274 @ 6764e2b0: iOS Build & Test SUCCESS; Mobile queued (holds macOS
  Native Tests / iOS Build / APK). 0 failed. CEO merge decision on file
  (V040_PR274_MERGE_DECISION_CEO_2026-09-03.md).
- Main tip: dc90269e (after #268 squash d395e030). Nothing else merged.

## 2026-09-03 21:30Z -- #267 addendum CLOSED via harness; AWS note for CTO

- Harness delta-addendum pass for #267 @ 17fa959f: claims manifest +
  verbatim git window (66 lines, sections A-F: ancestry, 2-parent merge
  topology, only-4-main-commits delta, PR diff D==E byte-identical, merge
  tree exit 0). Panel 3/3 x 4/4 claims all real:false, judge convergence
  0.97, cost $0.00. Verdict: FINAL APPROVE at 80197ef5 CARRIES to
  17fa959f. Filed HANDOFF/review/V040_T13_FDHT_DELTA_ADDENDUM_17fa959f_2026-09-03.md
  (chain head 3673ac46 seq 177, ledger verify clean). #267 merge now only
  waits on required CI green at 17fa959f.
- PROVENANCE NOTE: the earlier 16-file vs 3-file PR-diff anomaly was a
  STALE LOCAL origin/main ref (dc90269e vs d395e030) -- after fetch,
  PR diff at moved head == PR diff at reviewed head exactly. No content
  issue. Lesson: fetch origin/main before three-dot diffs.
- AWS/artifact coordination note written for CTO:
  V040_CTO_AWS_LEG_AND_FINAL_TREE_ARTIFACTS_2026-09-03.md -- image must
  be built AFTER #274 lands; wincli/APK exact via post-update-branch PR CI
  (merge ref == branch head once branch contains main).
- CI: #270/#267 Lint/CI/Cross/iOS queued ~35 min; #274 Mobile queued.
  Nothing merged this section.

## 2026-09-03 22:00Z -- #270 MERGED; #267 re-updated + addendum-2 on file

- #270 MERGED to main: squash c824fe9a, 21:55:55Z. Head at merge =
  3c4c0714 (update-branch of reviewed 6fd0230b; PR content preserved,
  ancestor-verified). Harness confirm-APPROVE on file
  (V040_T14_EPHEMERAL_CONFIRM_APPROVE_HARNESS_2026-09-03.md). Required 4
  checks green; merge-tree exit 0; cherry 3. Gates: tip MATCH, diff
  present, verdict YES.
- #267 second update-branch (strict protection post-#270): 17fa959f ->
  90f37d8f (2-parent merge: 17fa959f + c824fe9a). Verified 80197ef5 AND
  17fa959f ancestors; PR diff at 90f37d8f IDENTICAL to reviewed head
  (3 files 507/108); merge-tree exit 0. Harness addendum-2 filed
  (HANDOFF/review/V040_T13_FDHT_DELTA_ADDENDUM_2_90f37d8f_2026-09-03.md):
  2/2 parseable panelists false x4 claims, agreement high conf 1.0; judge
  returned no content (noted); ling truncated (excluded). Chain head
  25eaf9a7 seq 182. #267 merge now waits ONLY on required CI green at
  90f37d8f (re-running; Lint/CI queued).
- STALE-REF LESSON REAPPLIED: addendum-2 window initially showed a bogus
  16-file D vs 3-file E mismatch -- local origin/main was d395e030 while
  real main had advanced to c824fe9a (#270 merged between fetch and
  regen). Fetch origin/main before every three-dot diff. No content
  issue; regenerated with fresh refs.
- Main tip: c824fe9a (contains #264/#268/#269/#270/#271).
- #274 @ 6764e2b0: still UNSTABLE-not-green -- Mobile leg queued (only
  leg left). CEO go still pending; NOT merged.

## 2026-09-03 22:25Z -- STOP-AND-REPORT: #267 Lint FAILED (infra, not content)

- #267 @ 90f37d8f CI run 33810562446 (CI workflow): Lint JOB 100831131684
  FAILED 22:06->22:12Z, exit 101. Required check Lint now FAILURE on #267.
- ROOT CAUSE (job log): `cargo install cargo-deny` (ci.yml:28, UNPINNED,
  no --version/--locked) compiled tinyvec 1.13.0 which errors
  "cannot find macro `vec` in this scope" (tinyvec.rs:710) on the
  runner's dtolnay/rust-toolchain@stable -- toolchain/dependency
  incompatibility in the TOOL-INSTALL step, BEFORE any repo code linted.
- NOT content-related: the same Lint job PASSED at 17fa959f ~21:53Z
  (25 min earlier); #267's content is byte-identical at 90f37d8f
  (addendum-2, 3 files 507/108 == reviewed). Likely the runner pool
  rolled to a newer ubuntu image/toolchain; repo-wide + transient until
  the pool settles or ci.yml pins the toolchain/cargo-deny (a CI-fix
  change belongs to a separate PR, CTO/CEO lane -- NOT this pass).
- Rerun attempted: `gh run rerun 33810562446 --failed` REFUSED --
  workflow still running (Test ubuntu queued). Rerun possible once the
  run completes.
- MERGE HELD for #267 until Lint is green at 90f37d8f (required-check
  gate). All other gates + verdicts GREEN (addendum-2 on file).
- #274 @ 6764e2b0 Mobile leg still queued (~2h). CEO go still pending.

## 2026-09-03 23:22Z -- #274 FULLY GREEN + pre-flight pack; #267 Lint rerun queued

- #274 @ 6764e2b0: iOS Build SUCCESS 23:20Z -> CI fully green (0 failed /
  0 pending). Pre-flight gates ALL PASS (tip MATCH, diff vs candidate
  present, cherry 6, merge-tree exit 0, state CLEAN). Ready-to-execute
  pack appended to V040_PR274_MERGE_DECISION_CEO_2026-09-03.md. NOT
  merged -- CEO go still required.
- #267: CI run 33810562446 COMPLETED (macos Test passed); failed Lint job
  RERUN triggered 23:21Z (gh run rerun --failed). Watching to conclusion.
- Main tip: c824fe9a. Candidate: 44fee3c4. No CTO files this pass.

## STOP-AND-REPORT 2026-09-04 ~05:25Z -- #267 Lint failed AGAIN (same tinyvec infra error)
- Lint rerun (run 33810562446, job 23:21:07Z->23:27:16Z) conclusion=failure.
- Log evidence: job first compiled tinyvec 1.12.0 OK (23:23:48Z), then cargo
  re-resolved tinyvec 1.13.0 (downloaded 23:26:48Z) and failed 23:27:02Z:
  "error: cannot find macro `vec` in this scope" at
  /home/runner/.cargo/registry/.../tinyvec-1.13.0/src/tinyvec.rs:710:21.
- Root cause unchanged: ci.yml:28 is a bare `cargo install cargo-deny`
  (no --version/--locked), so a fresh tinyvec patch release breaks the
  tool-install step BEFORE any repo code is linted. Nondeterministic:
  1.12.0 passes, 1.13.0 fails; a third rerun could flip back -- not a fix.
- Content-proof: identical job PASSED at 17fa959f ~25 min before the first
  failure; the tinyvec error is in the runner toolchain, not PR code.
- #267 NOT merged. Held at 90f37d8f. Addendum-2 verdict on file
  (V040_T13_FDHT_DELTA_ADDENDUM_2_90f37d8f_2026-09-03.md) covers content.
- Two resolution paths, NOT decided here (CEO ruling required):
  (a) small CI PR pinning the toolchain: ci.yml Lint job ->
      `cargo install cargo-deny --version <pinned> --locked` (or pin the
      rust toolchain for the job) -- CTO lane, unblocks all future Lint
      gates repo-wide; (b) CEO infra-exception ruling to merge #267 with
      the Lint failure waived, evidence chain above on file.
- Note: docker-publish automation fired at main tips again -- c824fe9a
  (21:55:58Z, success) and d395e030 (20:54:03Z, success). These are MAIN
  tree images, not the candidate tree; final-tree image still needs a
  dispatch after #272 lands. No new redeploy evidence in tmp/run-evidence
  (newest remains the 177bd840 round, Sep 2 21:27Z).
- #272 mss=DIRTY at 44fee3c4 (main advanced past its base) -- expected;
  update-branch + re-pin verdicts + re-CI at merge time.

## 2026-09-04 05:35Z -- #274 MERGED into candidate (CEO standing go), #275 opened
- #274 (nimble-peer recycle fix) squash-merged into cto/v040-candidate-2026-09-02
  as 48672b18 (05:34:51Z). Gates: tip MATCH 6764e2b0 (pre-flight pack in
  V040_PR274_MERGE_DECISION_CEO_2026-09-03.md), CI 0 failed / 0 pending,
  Rule-8 APPROVE R4 @ 6764e2b0 on file (V040_NIMBLE_RECYCLE_REVIEW_QWEN_2026-09-03.md),
  lineage audited (branch on d82978ab, tree byte-identical to 44fee3c4).
- CONSEQUENCE: candidate head moved 44fee3c4 -> 48672b18. #272's head branch
  moved with it -> #272 verdicts pinned at 44fee3c4 (delta addendum request +
  deferral re-review brief) MUST be re-pinned at 48672b18 before #272 merges.
- #275 opened: ci/fix-cargo-deny-pin-2026-09-04 @ e3e329cd (off main c824fe9a),
  pin cargo-deny --version 0.20.2 --locked (tinyvec 1.13.0 fix; crate lockfile
  verified to pin tinyvec 1.11.0). CI-file-only -> no Rule-8 required.
  Owner: merge-execution seat. Fixes the Lint infra failure holding #267.

---
## #267 MERGED to main -- 2026-09-04 07:21:53Z
- Squash commit: 45ab59f9 (main tip now 45ab59f9)
- Head merged: 15228404 (third post-review move: 80197ef5 -> 17fa959f -> 90f37d8f -> 15228404)
- Verdict coverage: FINAL APPROVE @ 80197ef5 + addendum (17fa959f) + addendum-2 (90f37d8f) + addendum-3 (15228404), all on file in HANDOFF/review/
- Lint path: tinyvec/cargo-deny infra failure at 90f37d8f (bare `cargo install cargo-deny` at ci.yml:28); fixed by #275 (pin `--version 0.20.2 --locked`, squash b79702ba); #275 Lint SUCCESS proved fix; #267 Lint SUCCESS at 15228404
- Gates at merge: tip MATCH 15228404; diff present (10 ahead); merge-tree exit 0; both reviewed heads (80197ef5, 90f37d8f) ancestor-verified; all 4 required checks green (Lint, Test ubuntu/windows/macos)
- Addendum-3 verdict: HANDOFF/review/V040_T13_FDHT_DELTA_ADDENDUM_3_15228404_2026-09-03.md (2/2 parseable panelists x 4 claims real:false, judge APPROVE, ledger seq 186)

---
## #272 RESOLUTION COMMIT + Rule-8 delta APPROVE -- 2026-09-04 ~08:35Z
- Blocked state: #272 update-branch refused (conflicts: swarm.rs, observation.rs, local.rs, optimized_engine.rs, iron_core.rs -- the architecture candidate vs main's T13/T14 line).
- Resolution: merge commit e97c3f82 (parents 48672b18 candidate + 45ab59f9 main) authored in isolated worktree tmp/cand-merge; per-hunk decisions D1/D2/D3 documented in V040_CANDIDATE_MERGE_CONFLICT_BRIEF_2026-09-04.md and the commit message: D1 = #270 accept-all-empty-for-wasm record semantics win (candidate's 3 reversed fail-closed tests deleted; #270 suite kept); swarm.rs = #270 publication guard folded INTO sync_external_address helper (all 4 call sites pass bound_addresses); D2 = [u8;8] hint width carried forward, no [u8;4] remnants; D3 = parity fields dropped, no dup tests (main's reliability test verbatim, once; onion/D6 tests intact once).
- Local verification (resolution tree): cargo check workspace clean; clippy --workspace --all-features -- -D warnings clean; cargo fmt --check clean; lib suite 1411 passed 0 failed; integration 9/14/6 passed; focused suites for every resolved hunk green.
- Pushed: 48672b18..e97c3f82 FF to origin/cto/v040-candidate-2026-09-02 (~08:28Z). PR #272 head = e97c3f82, CI triggered (Mobile in_progress, Cross/CI/iOS queued).
- Rule-8 delta APPROVE on file: HANDOFF/review/V040_PR272_RESOLUTION_DELTA_RULE8_APPROVE_e97c3f82_2026-09-04.md (harness free-lane panel, 5/5 claims not_real unanimous, confidence 0.965-0.975, judge APPROVE, ledger seq 222-226).
- Remaining to #272 merge: CI green at e97c3f82; exact final-tree artifacts (wincli/APK from PR CI at e97c3f82; docker-publish dispatch at e97c3f82 -- CTO AWS leg); focused 3-node test on true final tree.

---
## 2026-09-04 09:45Z — 3-NODE TEST PREP: artifacts staged, both local legs LIVE at e97c3f82

- #272 CI at e97c3f8247b29dd344467e05137b24f0f110a10a: FULLY GREEN (all ~30 checks COMPLETED, 0 failed, mss=CLEAN).
- Provenance: PR artifacts built at merge-ref c5d6a4edced7370459e1c9748b53c6fe072cceb2; merge-ref tree == e97c3f82 tree (both 94d9d7c01b8bfd2bf4e68f9f41eae1bd637901ef). Verified.
- Staged: .codebuff_deploy/wincli-e97c3f82/scmessenger-cli.exe (sha256 ac7fb51a..., exe --help OK, CLI 0.4.0 c5d6a4e) + cli-provenance.txt (verified line appended); .codebuff_deploy/pixel-apk-e97c3f82/app-debug.apk (sha256 acb7bd88..., 4 ABIs, output-metadata.json).
- Windows leg LIVE: PID 20688, launched 09:41:53Z, log .codebuff_deploy/windows/wincli-e97c3f82-20260903T234153.log. [SEED-DIAL] sweep 1: 32 candidate(s), peers=0 (IPv6-unreachable on one candidate; sweep continues). Connected to AWS node 12D3KooW9uRMQT... via 54.235.20.24:9001 09:41:56Z; connected to fresh Pixel 12D3KooWBPdNE... via 192.168.0.135 09:44:20Z/09:44:42Z. Control API on 127.0.0.1:9876, Warp ws 9000, listeners 9002/ws etc.
- Android leg LIVE: clean reinstall (uninstalled old signature-mismatched build, installed CI debug APK, Success, versionCode 14 / 0.4.0). App started 09:44:06Z, PID 2452, identity b9fe29c5.../12D3KooWBPdNE1NB4uwMq41jL2YQxBZcMQBSwfnnEFBxpRVLkEGW (nickname Lucas). Swarm up, listeners on 192.168.0.135; joined Windows node via LAN discovery (no QR needed).
- AWS leg: NOT YET at final head — running node i-0b735c4f26aea42ed (54.235.20.24) still old image (177bd840-era); CTO dispatch on file V040_CTO_FINAL_HEAD_e97c3f82_AWS_LEG_2026-09-04.md (docker-publish at e97c3f82 + redeploy). No CTO movement since 09:13Z, no live cargo/docker.
- Driver (freebuff-CLI watcher) DISABLED 2026-09-04 by CEO request: PID 21892 killed, Startup lnk removed, no scheduled task existed. Replacement: on-demand /drive slash command at .claude/commands/drive.md.

---
## #276 MERGED to candidate -- 2026-09-06 ~01:1xZ (session clock)
- PR #276 squash-merged via gh pr merge 276 --squash: merge commit b0f7ac4eb2551dd3d6b7d28a308c1765c824007e on cto/v040-candidate-2026-09-02 (candidate tip now b0f7ac4e; prior tip e97c3f82).
- Rule-8 closure: qwen R14 plain APPROVE on file (tmp/rev276_r14_response.md, model qwen3.8-max-0902, usage_src=api in=15024 out=853, ledger line ts 2026-09-06T00:22:51Z). Chain R11-R13 dispositions committed on branch (95fe0368, 1fc8c60d, 6359f661).
- Gates at merge (branch head 6359f661, Windows host): core check --all-targets PASS; core lib 1416 passed / 0 failed / 5 ignored; cli lib 83/0; wasm32 check PASS; clippy --workspace -D warnings PASS; fmt PASS. Zero new dependencies (no manifest/lockfile touched - verified).
- PR CI at 6359f661: all checks green (Android JVM, macOS Native, iOS Build, iOS Simulator, Android APK, Wiring Gate, label) - mergeStateStatus went CLEAN after iOS Simulator completed.
- Merge executed by the merge-authority seat per CEO approval V040_CEO_APPROVAL_PUSH_276_TO_FULL_3NODE_GREEN_2026-09-04.md step 4. No tag, no release.
- Next per approval file: re-pin #272 verdicts at b0f7ac4e; rebuild wincli+APK at b0f7ac4e; docker-publish + AWS redeploy at b0f7ac4e (preserve relay data dir; old instance i-0b735c4f26aea42ed stays STOPPED as rollback); full 3-node validation with ON-DEVICE Windows->Pixel Text gate + AWS-relayed hop.
