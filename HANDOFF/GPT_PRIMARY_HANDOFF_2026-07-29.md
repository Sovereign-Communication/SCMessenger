# GPT PRIMARY-DRIVER HANDOFF (operator directive 2026-07-29)

Status: ACTIVE
Roles per operator: GPT (Mac, GPT-5.6 Sol) = PRIMARY orchestrator brain,
judgement, and iOS implementation. Windows Claude (qwen3.8-max-preview,
weekly API quota ~80% consumed -> CONSERVE) = Windows execution arm:
builds, gates, commits, pushes, CI monitoring, emulator/device proof
execution, surgical local fixes. GPT directs; Windows Claude executes on
instruction. GPT may request Windows actions by writing a
HANDOFF/gpt/WINDOWS_REQUEST_*.md file (Windows Claude polls HANDOFF/gpt/).

## Current state (verified 2026-07-29)

### Ledger-seeding remediation (#116, wip/v040-seeding-fixes)
COMPLETE at tip a6b7abdb (range ed13500a..a6b7abdb). Every operator-
mandated finding FIXED with per-packet Fusion Tier-A reviews + per-claim
orchestrator adjudication: F10 (load cap + eviction + byte bounds at
ingest AND load + save_lock serialization + atomic durable writes +
durable shrink + corrupt-JSON recovery), F7 (threshold aligned to 3 in
ALL FOUR accessors + register gate + record_failure wiring), F13
(is_dialer gate on pending-dial resolution -- design decision: kept for
evidence integrity; collapsed-simultaneous-open self-heals via 10s
sweep), NEW-6 (global TokenBucketState burst 10 / refill 2s), 1b
lost-update/corruption regression (closed), canonical tie-breaks +
anchor stamping (desync fix), 12+ new tests. CI was red on clippy
let_unit_value/map_or + one stale test fixture; fixed in a6b7abdb;
local clippy -D warnings CLEAN; CI rerun in progress on a6b7abdb.
Per-finding evidence: HANDOFF/review/V040_FINDING_DISPOSITIONS.md.

### Merge criteria for #116 (no caveats standard)
(a) CI fully green at final SHA (in progress); (b) terminal SHIP verdict
-- request lives at HANDOFF/gpt/GPT_SEEDING_REVIEW_TERMINAL_REQUEST.md
(on the wip branch; functional range ed13500a..b1261fbf, later commits
are style/test fixes); GPT to deliver GPT_SEEDING_REVIEW_TERMINAL_VERDICT.md
on gpt/seeding-review; (c) squash-merge executed by Windows Claude on
GPT's instruction.

### PR board
- #115 (v0.4/v0.5 plan): MERGED -- the north-star plan is on main.
- #117 (iOS test truth): GPT's Mac lane -- needs actor-isolation
  extended to the four files failing under CI's Xcode 15.4:
  Views/Contacts/ContactsListView.swift, Views/Settings/DiagnosticsView.swift,
  Views/Contacts/VerifySafetyNumberSheet.swift + the batch compile group
  (ContentView, IosPlatformBridge, IdentityBackupSheets, ...). Orchestrator
  review comment on the PR has the details.
- #114 (safe device resolution): own changes CI-verified (resolution
  test passed on macOS CI); rebase after #117, merge on full green.
- Dependabot (13 open): thiserror 1->2 (#69) is BREAKING -- operator-
  approved POST-TAG; the rest batch-mergeable after rebase (gradle bumps
  need assembleDebug verification).

## Economy and lanes
- QWEN WEEKLY ~80% CONSUMED: no qwenpaid dispatches unless critical.
- GPT (you) primary for orchestration brain, implementation judgement,
  and iOS; Mac lane is the only xcodebuild authority.
- AGY = PRIMARY DELEGATION POINT for executable work (operator directive
  2026-07-29). Do not do work yourself that agy can do. Quotas are
  REMAINING percentages (fresh): Gemini pool weekly 86% / 5-hour 99% --
  use the Gemini pool FREELY; Claude pool (via agy) SELECTIVELY.
  Models: gemini-3.6-flash-high|medium|low (workhorses),
  gemini-3.5-flash-*, gemini-3.1-pro-high|low (deeper),
  claude-sonnet-4-6 / claude-opus-4-6-thinking (Claude pool, selective),
  gpt-oss-120b-medium.
  WORKING INVOCATION FORM (operator-verified 2026-07-29): the prompt
  flag (--print or -p) MUST be adjacent to the prompt string (prompt
  last); separating them makes agy drop the prompt and reply with a
  self-introduction:
    agy --add-dir <repo> --model "gemini-3.6-flash-high" --print-timeout 30m --print "<task>"
  Add --dangerously-skip-permissions for unattended runs. Piped form
  also works: echo "<task>" | agy --model "gemini-3.6-flash-high"
  --dangerously-skip-permissions. Windows binary:
  %LOCALAPPDATA%\agy\bin\agy.exe (PATH drifts; use full path). agy
  edits files in place and runs commands -- instruct it to NOT
  commit/push; the orchestrator gates and commits.
- Fusion Lite (scripts/fusion_lite.py): SEPARATE OpenRouter quota
  (openrouter_fusion.env, ~$0.75/day); hard ceiling raised to $0.10
  (operator-approved 2026-07-29) for Tier-B premium panels
  (deepseek-v4-pro + kimi-k2-thinking + qwen3-235b-thinking). Vendor-
  diverse second opinions on demand; your judgement is primary.
- CI = authoritative gate for Rust/Android/iOS/WASM (Windows box is
  RAM-bound; local builds are slow -- prefer CI, use local only for
  fast targeted checks).
- Windows Claude = execution arm ONLY (builds, gates, commits, pushes,
  CI monitoring, device proofs, surgical gate-unblocking fixes); does
  no implementation work unless necessary.

## Operator-pending blockers
1. S4/S5 cloud node route: 100.56.248.69 is a Tailscale CGNAT address;
   the Windows host has no Tailscale -> unreachable. Operator to provide
   a public endpoint (public IP or DDNS + port forwards per H-04) or
   run Tailscale on the Windows host. S4 runbook:
   HANDOFF/review/V040_S4_DELIVERY_PROOF_RUNBOOK.md (AVD is scm_pixel_35).
2. Release signing: SCMESSENGER_KEYSTORE_* repo secrets ABSENT -> release
   ships debug-signed APK unless operator adds a keystore (sovereign
   model may prefer debug-signed/per-user signing -- operator call).
3. Tag procedure: auto-tag trap DEFUSED (auto-tag-release.yml is now
   workflow_dispatch-only); operator creates v0.4.0-alpha.1 manually
   after the terminal release PR (version bump 0.3.5->0.4.0 in
   Cargo.toml:9 + android/build.gradle:34-35 by hand -- NEVER run
   scripts/sync_version.sh, it corrupts versionCode).

## Key state files (tracked; tmp/ is Windows-local/untracked -- not visible to you)
- Queue: HANDOFF/todo/_QUEUE.md (2026-07-28 takeover header).
- Plan: HANDOFF/gpt/GPT_PLANNING_040_050_VERDICT.md (on main).
- Finding dispositions + evidence: HANDOFF/review/V040_FINDING_DISPOSITIONS.md.
- Release mechanics (verified file:line): HANDOFF/review/V040_S3_RELEASE_MECHANICS.md.
- S4 runbook: HANDOFF/review/V040_S4_DELIVERY_PROOF_RUNBOOK.md.
- S5 Josh runbook draft: HANDOFF/review/V040_S5_JOSH_WAN_RUNBOOK.md.
- All review artifacts: HANDOFF/review/ and HANDOFF/gpt/.

## Conventions (binding, hook-enforced)
No emojis anywhere. Worker contract (RESULT: DONE|BLOCKED|FAILED first
line). Delegation discipline: orchestrators do not write application
code (1-3 line surgical gate-unblocking fixes excepted). Commit
provenance prefixes (native:/swarm:/fix:/docs:). Never push unless the
operator asks (Windows Claude executes merges/tags on instruction).
CI-first gating. One build at a time on Windows.
