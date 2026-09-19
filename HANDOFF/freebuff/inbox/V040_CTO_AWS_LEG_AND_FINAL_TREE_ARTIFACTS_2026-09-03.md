# CTO -- AWS LEG + FINAL-TREE ARTIFACT PLAN (paste-ready, dated 2026-09-03)

Status: coordination note. No merge authority changes: #274 merge is CEO-
seat; every merge remains individually CEO-approved; no self-merge/tag/
release; no cleanup while cargo/docker builds are live
(scripts/clean_target.sh).

## Where things stand (verified 21:30Z)

- main = d395e030 (after #268 squash d395e030; #264/#271/#269 earlier).
- candidate (cto/v040-candidate-2026-09-02) = 44fee3c4 (contains #273).
- #270 @ 3c4c0714, #267 @ 17fa959f: CI queued (runner backlog); #267
  review-lane hold CLOSED -- harness delta addendum on file
  (HANDOFF/review/V040_T13_FDHT_DELTA_ADDENDUM_17fa959f_2026-09-03.md)
  confirms the FINAL APPROVE carries to the moved head byte-intact.
- #274 @ 6764e2b0: Rule-8 APPROVE on file; CI not fully green (Mobile leg
  queued). CEO merge decision requested
  (HANDOFF/freebuff/inbox/V040_PR274_MERGE_DECISION_CEO_2026-09-03.md).

## Your AWS leg (the one real task still open on your lane)

Latest docker-publish at the candidate branch = 177bd840 image (06:47Z).
NO run exists at 44fee3c4; no redeploy evidence newer than the 177bd840
round. The exact image for the final tree MUST be built AFTER #274 lands
(the candidate head will move), so it matches the shipped tree:

1. Await CEO go + #274 merge into cto/v040-candidate-2026-09-02 (new head
   H = 44fee3c4 + #274 delta).
2. Dispatch docker-publish.yml via workflow_dispatch at the candidate
   branch -> exact image at H (ci.yml/mobile.yml have no dispatch; the
   image is the ONLY artifact that can be exact-SHA without a PR).
3. Elevation-redeploy i-0b735c4f26aea42ed (54.235.20.24) with the image
   at H -- REUSE the instance, NEVER terminate (IAM guard: run_instances
   dry-run DENIED for scmessenger-relay-orchestrator). Record container
   ID + same data volume + identity continuity evidence per the 177bd840
   precedent (tmp/run-evidence/aws-redeploy-177bd840-evidence.md format).
4. Return evidence: run URL + head SHA, container ID, redeploy log with
   timestamps.

## wincli/APK exact final-tree artifacts (documented path, no local builds)

Finding (provenance audit, staged artifacts): PR-event CI builds
refs/pull/N/merge; at 44fee3c4 that merge ref differed from the branch
head because main had advanced past the PR base (#264/#271/#269). Once
the candidate branch is update-branched with main AND #274 is in, the
merge ref converges to the branch head (branch contains main; no
divergence) -- so PR CI at that head produces EXACT artifacts of the true
shipped tree: Windows CLI Artifact + Android Debug APK from the #272 PR
runs at the updated head, plus the staged per-SHA convention
(.codebuff_deploy/wincli-<sha>/, .codebuff_deploy/pixel-apk-<sha>/ with
README-PROVENANCE.txt + sha256).

Sequence for the final test (after all merges): #272 update-branch ->
delta-addendum verdict at the new head -> #272 PR CI green -> download +
verify + stage wincli/APK at the exact head -> focused 3-node live window
(recycle gone: no :50 closes, no :15 dead-marks, 75-item custody drain,
ae277205 resolution, delivery burst) -> #272 to main.

## Return contract

Raw command output, run URLs, PR numbers, container ID, approval
artifacts. RESULT: DONE|BLOCKED|FAILED + VERIFICATION lines.