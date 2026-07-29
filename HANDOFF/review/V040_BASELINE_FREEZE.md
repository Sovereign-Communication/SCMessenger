# v0.4.0-alpha.1 BASELINE FREEZE (040-G0)

Status: FROZEN
Date: 2026-07-28
Authority: GPT planning verdict PR #115 (GPT_PLANNING_040_050_VERDICT.md),
gate 040-G0.

## Immutable references

- Release-planning baseline main: 74a6808d (GPT plan's review baseline).
- Current main at freeze: eeb4a618 (= 74a6808d + the two GPT seeding-review
  verdict docs cherry-picked for coherence: GPT_SEEDING_REVIEW_STAGE_1A.md,
  GPT_SEEDING_REVIEW_STAGE_1B.md).
- Staging branch: refs/heads/wip/v040-seeding-fixes, tip 068972f2,
  parent ed13500a (branch point from main). Current tip: see
  `git rev-parse wip/v040-seeding-fixes` (branch moves as packets land).
- Draft tracking PR: #116 (wip -> main; DO NOT MERGE until 040-S2/S3 clear).

## Commits entering the release candidate (staging, in order)

- d258fd7f swarm: F10 ledger cap + eviction + F7b seed ordering (1a)
- 068972f2 swarm: F10 save-off-lock + shared annotate helper (1b)
- d258fd7f..909edf4c chain: 1a, 1b, v2a-1 (60b7e911 load cap,
  panel claims verified false, override documented), v2a-2 (909edf4c
  byte bounds + PeerId validation + threshold alignment; one real panel
  finding -- missing key-length tests -- remediated in v2c)
- (pending) v2c persistence serialization: save_lock + unique tmp +
  sync_all + parent-dir fsync + stale-tmp cleanup (panel-hardened)
- (pending) v2b anchor semantics + deterministic tie-breaks + batch
  verification + expanded tests
- (pending) 1c mobile_bridge batch caller swap
- (pending) packet 2 swarm.rs: F7a register gate, F7b record_failure
  wiring, F13 is_dialer gate, NEW-6 global bucket

## Dirty-state record at freeze

- Working tree (wip checkout): mid-remediation; no uncommitted release
  content at this instant (v2a dispatch in flight, nothing applied yet).
- Untracked non-release files: launch_claude.ps1 (operator launcher,
  candidate for .gitignore or scripts/), tmp/ orchestrator scratch
  (not auditable authority per GPT plan -- this file and the tracked
  queue headers govern).

## Release-execution constraints (from plan 1.5)

- .github/workflows/auto-tag-release.yml auto-tags main pushes that
  change the workspace version -- the version-bump commit MUST land only
  in the terminal release PR with the auto-trigger removed (keep inert
  manual definition); operator creates v0.4.0-alpha.1 manually; the tag
  triggers release.yml.
- Operator (not the Mac lane) merges and tags.
- Terminal verdict must explicitly dispose F2/F3/F6/F7/F10/F12/F13/F16/
  NEW-6 -- see HANDOFF/review/V040_FINDING_DISPOSITIONS.md (live table).
