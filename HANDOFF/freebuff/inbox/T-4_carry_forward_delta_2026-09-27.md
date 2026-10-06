# T-4 carry-forward delta -- tasks the sources list that the merge-train task file lacks

Task: V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md / T-4
Type: QUESTION

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Sources read in full this session (via `git show <ref>:<path>`, nothing merged):
- origin/freebuff/lane-unify-040-train-20260923:HANDOFF/freebuff/V040_0_LANE_UNIFY_MERGE_TRAIN_2026-09-23.md
- origin/freebuff/v040-merge-train-plan-20260924:HANDOFF/V040_MERGE_TRAIN_PLAN_2026-09-24.md (+375)
- origin/recovery/harness-plan-snapshot-20260924 and origin/freebuff/v040-v050-harness-plan
  (delta vs main: HANDOFF/V040_JEV_HARNESS_INTEGRATION_2026-09-21.md,
   HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md, freebuff/README.md,
   docs/rules/BUILD_AND_CI.md + harness_admission*.py on the snapshot)
- origin/train/A3-deny-waiver (P1_SECURITY_RUSTLS_RUSTSEC_2026_0285_TRACKING.md + deny.toml delta)
- freebuff/queue/ tickets named below (status lines read on origin/main)

Interpretation note: the lane-unify doc enumerates no superseded queue files beyond
"supersedes earlier install notes", so the freebuff/queue/ set was checked against the
train's 69 cars instead. Everything the sources list that ALREADY has a car (MT-05 for
#361/#372, MT-06 for #351/#367, MT-07 for #364, MT-08 for #359, MT-03 for #352/#349/#355/#356,
R-1 for the #352 JEV row, H-5 for Rule-8 dispatch, H-1/H-3 for the Pixel/keystore items,
MT-11 for #227/#156/dependabot) is excluded from this table.

| # | Task | Source (evidence) | Why it lacks a car | Suggested disposition |
|---|---|---|---|---|
| A1 | PR #366 lane-unify train doc: merge | plan doc A0 (19/19 green, CLEAN) | no train car names #366 | merge early (docs-only) |
| A2 | PR #358 doctrine rows + Android copy (the aapt2 XML fix): merge | plan doc A2 (31/31 green); branch freebuff/canonical-doctrine-rows-20260921 (one of the 9 pushed today) | MT-00b names only #386/#388/#357/#376, not #358 | fold into MT-00b or merge standalone; its Android fix gates the Debug APK job |
| A3 | PR #368 harness admission + staged rollout docs: merge; PR #369 close as WIP snapshot | plan doc A6; JEV_HARNESS_INTEGRATION admission sections | no car lands the admission policy; P0-6 confirmed BUILD_AND_CI.md "Harness admission" section is absent on main | land via MT-00a/MT-00b (which claim to create it) or merge #368; close #369 |
| A4 | PR #354 JEV WP1/WP2 state json + ruling: merge | plan doc A9 (19/19) | not named anywhere in the train | merge (docs-only, no conflicts) |
| A5 | PR #329 status-reconcile: update with main, re-check, merge if the 3 status lines still match | plan doc A9; tracker comment 1 says "leave it for MT-11" | MT-11's task list has no #329 row | add to MT-11 explicitly or close |
| A6 | PR #360 harness version-floor: port the intent onto fresh main; relocate the Harness/handoff/... doc under HANDOFF/harness/; re-land scripts/harness_gate.py | plan doc A9 + JEV_HARNESS_INTEGRATION "Superseded for implementation; port its intent" | no car; conflicts with A3 on harness_gate.py | recreate the guard after A3 lands; do not merge the old PR |
| A7 | PR #362 harness lane state (CTO/CEO_STATE appends): merge or close | plan doc A1 + JEV doc "keep separate" | not named in the train | decide: merge as docs evidence or close |
| A8 | PR #363 openclaw-bridge-ops (draft): keep draft; review deployment paths before rollout | plan doc A9/12 | not named in the train | hold; revisit at rollout |
| A9 | PR #316 outbox-retry-fix: close "delivered by #364 (cc51e41a)" | plan doc A9; lane-unify merge order 5 | not named in the train | close with that reason once MT-07 lands |
| A10 | PR closures with no MT-11 slot: #220, #216 (superseded), #218 (dead code by its own admission), #209 (superseded, DIRTY), #170 (Lint/Rust red since 08-16), #207 (placeholder), #367 (close-superseded-by-#364 after MT-06) | plan doc sec 8 + 12 (47-row disposition table) | MT-11 lists only #227/#156/dependabot | extend MT-11's closure list or issue one batch QUESTION |
| B1 | MeshServiceViewModelTest toggle expectations vs the ViewModel code: decide which is wrong, fix on the branch, re-pin the Rule-8 verdict if gated files moved | plan doc sec 10.2 + A3/A4 blocker 2 (the one CI blocker on both #361 and #364) | no ruling slot in the train (H-1..H-11/D-01/R-1/R-2 do not cover it) | operator/lane-owner ruling before MT-05 |
| B2 | #364 shape: split A4a (sweep + docker) / A4b (Android stop-start) or merge whole with one verdict -- "the branch owner produces the split" | plan doc A4 preferred-shape table | MT-07 assumes a whole-branch merge | ruling before MT-07; the split frees the Android half from the transport Rule-8 wait |
| B3 | Whether #369's scripts/harness_admission*.py are wanted on main at all | plan doc sec 10.4 | no ruling slot | decide with A3 |
| C1 | Per-leg rollback refs (archive/pre-leg-<N>-<date> at the pre-merge main tip) + verified per-leg all-refs bundles | plan doc sec 1 steps 0/9 + sec 5 | the train's car pattern (section 6) has neither | adopt into the car pattern; cheap and already scripted in the plan |
| C2 | Reclamation: reclaim_safe.py --durable-ref origin/main must print SAFE before any target/ delete; never run scripts/delete_merged_branches.sh or verify_branch_merges.sh (hard-coded 2026-02 names) | plan doc sec 9 | train MT-11 does not mention either | adopt; AGENTS.md rule 17 already implies it |
| D1 | OpenClaw node (13.217.204.112) rollout to the same CI-built provenance as AWS (readiness gate 3) | lane-unify doc | TRI-040 is a 3-node proof; OpenClaw is a 4th rollout target | confirm in/out of TRI-040 scope |
| D2 | Pixel readiness gate 2: the CI APK's provenance must include cc51e41a; install is a FRESH install (old identity 779e9ea3... leaves stale entries in peers) | lane-unify doc gates + addendum | MT-06 device checks lack the provenance requirement | add to MT-06 acceptance or TRI-040 |
| E1 | Land the rustls RUSTSEC-2026-0285 waiver removal (deny.toml guard re-arm): the branch drops the ignore with NO Cargo.lock change -- verify resolved rustls >= 0.23.45 first, else `cargo deny check advisories` re-arms red | train/A3-deny-waiver (tracking doc "Status: Closed -- waiver removed, guard re-armed 2026-09-25"; deny.toml delta removes the ignore) | MT-01 says "tracked separately" but no car lands the removal | new small car after MT-01; its own scope note: not a v0.4.0 blocker, FIRST PUBLIC RELEASE blocker |
| F1 | V040-T-COB001 dual-drain IronCore outbox flush (wasm + all callers): OPEN, P1-HIGH, 0.4.0-target, message-strand class | queue/V040_T_COB001_WASM_OUTBOX_DUAL_DRAIN.md Status line; MT-02 #401 is explicitly "the CLI-03 half of the P1 outbox blocker" | no car carries the wasm/callers half | new defect car or fold into MT-09f |
| F2 | V040-T-ANDROIDTEST-COMPILE: ticket says OPEN P0 "Mobile lane red; APK job fails after artifact upload" -- but branch fix-androidtest-compile-20260920 is CONTAINED (content already on main) and Mobile is 5/5 green incl. the latest main run 36311537543 | queue/V040_T_ANDROIDTEST_COMPILE_FIX.md + TRIAGE.tsv + p07_runs_mobile.txt | contradicts itself | verify on main and close the ticket (likely stale) |
| F3 | Checked and NOT missing (listed so the operator sees they were verified): V040-T6 MERGED (PR #311), V040-T8 MERGED (PR #271), V040-T12 MERGED (#319/#328; residual = ops cost), V040-T1 + V040-T2 "CODE ON MAIN -- do not re-dispatch" | queue status lines on origin/main | n/a | no action |

Reply format (tracker PR #403 or here): one line per row -- LAND / CLOSE / SKIP / MERGE / RULE <text>.
