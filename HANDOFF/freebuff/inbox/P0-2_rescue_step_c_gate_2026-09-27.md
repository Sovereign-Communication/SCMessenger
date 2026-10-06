Task: V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md / P0-2
Type: QUESTION

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

P0-2 steps a and b are complete and proven; step c (commit + push of rescue
branches) sits behind its [OPERATOR GATE]. A branch-name collision with the
earlier train's already-pushed rescue branches needs an operator choice.

## Evidence (commands and outputs, 2026-09-27)

Step a, read-only capture into STATE_DIR/rescue/<wt>/ (script:
STATE_DIR/p02_rescue.sh capture). Untracked-file counts per worktree:
SCMessenger 33, repro-candidate 0, repro-candidate2 0, repro-candidate3 0,
repro-candidate4 0, repro-candidate5 0, repro-clean 0, swa-20260925 15,
SCMessenger-v040-harness-plan 6, harness-plan-snapshot-20260924 0.

Step b, completeness proof (STATE_DIR/p02_rescue.sh prove). `git -C <scratch>
status --porcelain | wc -l` vs the source worktree, 10/10 exact MATCH:

```
[OK] SCMessenger status count MATCH src=78 scratch=78
[OK] repro-candidate status count MATCH src=1 scratch=1
[OK] repro-candidate2 status count MATCH src=35 scratch=35
[OK] repro-candidate3 status count MATCH src=37 scratch=37
[OK] repro-candidate4 status count MATCH src=37 scratch=37
[OK] repro-candidate5 status count MATCH src=37 scratch=37
[OK] repro-clean status count MATCH src=34 scratch=34
[OK] swa-20260925 status count MATCH src=13 scratch=13
[OK] SCMessenger-v040-harness-plan status count MATCH src=18 scratch=18
[OK] harness-plan-snapshot-20260924 status count MATCH src=1 scratch=1
```

Delta vs the earlier train's already-pushed `rescue/<wt>-20260927` branch trees
(STATE_DIR/p02_delta.sh + `git ls-tree` re-classification; details in
STATE_DIR/p02_delta_detail.txt):

- 9 of 10 worktrees: the rebuilt current state is BYTE-IDENTICAL to the pushed
  rescue tree (283/300 paths exact blob match; the only non-matches are all in
  the primary checkout). Nothing to push for these 9.
- Primary checkout (SCMessenger), 84 paths:
  - 67 paths byte-identical to rescue/SCMessenger-20260927.
  - 8 root tmp_*.py byte-identical at scripts/tmp_*.py in that tree (the
    earlier capture renamed them to pass the handoff gate; content preserved).
  - 1 deletion (core/examples/nat_reflection_demo.rs) consistent: absent in
    both current state and the rescue tree.
  - 5 HANDOFF/*.md drifted since the earlier capture (V040_3NODE_LOG_ANALYSIS,
    todo/D9_LIBP2P_EITHER_HANDLER_PANIC, review/D1_D9_HARNESS_ADVERSARIAL_
    FINDINGS, todo/AND_STOP_START_FLOOD_AND_CANCEL, todo/OUTBOX_NO_PERIODIC_
    RETRY_SWEEP). Their CURRENT bytes are reachable from origin refs per this
    session's blob test (STATE_DIR/bs_uncommitted_blob.tsv), but the earlier
    capture holds different revisions.
  - scripts/validate_handoff_scope.py and tests/test_handoff_scope.py: current
    bytes are NOT on GitHub anywhere. The rescue tree holds older revisions
    (351ac8b8, 555a6607); PR #402 holds a third revision of the script
    (3c5dcd13).
  - .claude/alibaba_cloud_config.env.bak-20260927T111450Z: SECRETS (the
    paid-usage-stop key backup). Captured in STATE_DIR/rescue/SCMessenger/
    untracked/ only. Must never be pushed to GitHub (AGENTS.md rule 3: never
    commit secrets/keys). Excluded from every push option below.
  - tests/__pycache__/test_handoff_scope.cpython-314.pyc: gitignored
    (.gitignore:340 __pycache__/), compiled derivative; not captured, not
    pushed.

Collision: the task file names the step-c branch `rescue/<wt>-20260927`, but
those branches already exist on origin at the earlier capture's SHAs
(rescue/SCMessenger-20260927 @ ba9f67db, verified live via `git ls-remote
origin refs/heads/rescue/*` this session).

## The question

How should step c push the primary checkout's delta? Options:

- A (recommended): append ONE new commit on top of rescue/SCMessenger-20260927
  (tree = current primary state minus the secrets .bak and the .pyc), push
  fast-forward. History keeps both captures. No other branch changes.
- B: push the same tree as a new branch rescue/SCMessenger-20260927b; the
  existing branch stays untouched.
- C (minimal): push only the two unbacked work-product files
  (scripts/validate_handoff_scope.py, tests/test_handoff_scope.py) as
  rescue/SCMessenger-delta-20260927; the 5 HANDOFF drifts rely on the existing
  blob-level reachability evidence.

In every option: the secrets .bak and the .pyc are excluded from GitHub; the
.bak remains captured in STATE_DIR. The other 9 worktrees are not pushed (their
current state is already byte-identical to the pushed rescue branches).

Reply: GO P0-2 (A) | GO P0-2 (B) | GO P0-2 (C) | SKIP P0-2 | STOP
