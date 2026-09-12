# V040 CTO RESTART COORDINATION -- 2026-09-03 (paste-ready)

Supersedes nothing; complements the standing sync note
(V040_CEO_CTO_SYNC_2026-09-03.md) and the merge execution log
(V040_MERGE_EXECUTION_LOG_2026-09-03.md), which remain the evidence record.
This note is the delta briefing for a CTO session restart: what has landed
since your nimble-peer round (10:01Z), what is staged, what you own next,
and the decisions that are NOT yours to make.

## 1. Verified state at write time (2026-09-03 18:25Z)

- main tip  = dc90269e (squashes a6e9ece1 #264, 36fc1faa #271, dc90269e #269)
- candidate = 44fee3c4 (177bd840 + squash 44fee3c4 #273, nimble-peer fix)
- #268 head 7bafe83d, #270 head 6fd0230b, #267 head 80197ef5 -- all
  unchanged since your last write, all BEHIND main (update-branch at
  merge time, per the approval records)
- #272 head 44fee3c4, OPEN, BEHIND; PR CI fully green at 44fee3c4
  (all 7 workflow runs SUCCESS 16:27Z, 0 failed / 0 pending)
- No cargo/rustc/gradle/docker/scmessenger-cli processes running -- clean
  build slate, no serialization conflict
- Pixel 6a reconnected via adb mDNS (192.168.0.111:37805, transport 5,
  bluejay). App process NOT running -- correct under land-everything-first;
  install/start happens at leg run time.

## 2. What landed since your last write (all Rule-8-governed, log kept)

- #264 CI-pacing -> main (a6e9ece1), #271 t8-restore-test -> main
  (36fc1faa), #269 t14-preexisting -> main (dc90269e). No Rule-8 required
  for #264/#271 (CI/test/doc scope); #269 APPROVE on file
  (V040_T13_F7_REVIEW_QWEN_2026-09-01.md). All gates run per PR: tip
  check, git diff --quiet origin/main...branch, git cherry, merge-tree
  exit=0 on swarm.rs/observation.rs/ledger_entry.rs.
- #273 nimble-peer -> candidate (44fee3c4). CI went green after the
  Maven-429 APK rerun. Squash tree byte-identical to the independently
  APPROVED d82978ab; parent = 177bd840. #273 touches resume_prefetch.rs,
  dial_policy.rs, swarm.rs (dial-policy/dead-mark region only) -- ZERO
  lines on the admission/promotion surface (listen_port_from_bound_addr,
  sync_external_address), observation.rs untouched, so it does NOT touch
  the UDP/QUIC admission question.

## 3. Staged for the final-head test (NOT installed, NOT launched)

- .codebuff_deploy/wincli-44fee3c4/ -- exe sha256 27d3afb7..., smoke
  OK (--help exit 0, banner 0.4.0 (1a208f7)), README-PROVENANCE.txt
- .codebuff_deploy/pixel-apk-44fee3c4/ -- app-debug.apk sha256
  881b6308..., com.scmessenger.android debug 0.4.0 versionCode 14
- [WARNING] Provenance finding, read the two README-PROVENANCE.txt:
  CI artifacts are the refs/pull/272/merge tree (1a208f72 = 44fee3c4
  merged into dc90269e), NOT the pure 44fee3c4 tree -- main advanced
  past the PR base, so the binaries carry #269's swarm.rs delta. At
  177bd840 the merge-ref tree was identical to the branch head; it is
  NOT now. ci.yml/mobile.yml have no workflow_dispatch and no
  candidate-branch push, so GitHub CI cannot produce an exact-44fee3c4
  exe/APK. docker-publish.yml DOES dispatch -- exact image is possible.

## 4. What YOU own next (CTO)

1. AWS leg at the new head (the only real CTO task left before testing):
   - dispatch docker-publish.yml at the candidate branch for an
     exact-44fee3c4 image -- no Docker Publish run exists at 44fee3c4,
     the image is still 177bd840-only (CI run 33724848436)
   - elevation-redeploy i-0b735c4f26aea42ed (54.235.20.24) per the
     runbook's deploy commands: docker pull
     testbotz/scmessenger:cto-v040-candidate-2026-09-02 + rm -f + run,
     same data volume, identity preserved. NEVER terminate -- IAM
     run_instances DENIED for scmessenger-relay-orchestrator, reuse only.
   - evidence: log file with image sha + container id + startup sweep.
2. qwen lane dispatch (your driver; briefs are staged, not yet sent):
   - #268/#270 confirm-APPROVE -- dispatch has been pending since 20:41Z
     (V040_REVIEW_DISPATCH_268_270_CONFIRM_APPROVE_QWEN_2026-09-03.md)
   - #272 delta addendum + deferral re-review at 44fee3c4 -- briefs
     staged at V040_CANDIDATE_272_DELTA_ADDENDUM_REQUEST_44fee3c4_2026-09-03.md
     and V040_REVIEW_DISPATCH_272_DEFERRAL_REREVIEW_44fee3c4_2026-09-03.md.
     CEO-sanctioned outcome (option (c)): APPROVE at 44fee3c4 recording
     the UDP/QUIC deferral -- deferred for the v0.4.0 gate, NOT excluded
     forever, nothing structurally forecloses it, QUIC already live in the
     routing ladder. Option (d) -- CTO-only disposition with no lane
     verdict -- remains the ONLY unacceptable outcome.
3. Iterate any lane verdict to a plain APPROVE before it closes a gate
   (directive: reviews must end in APPROVE).

## 5. Decisions that are NOT yours (CEO-owned, pending)

- Harness lane go: the stalled #268/#270 confirm passes could run through
  the Harness free lane (140/140 tests pass, already in live production
  use at $0.00 -- V040_HARNESS_LANE_INCORPORATION_2026-09-03.md). Waiting
  on explicit user go; do not dispatch harness live without it.
- #267 ordering under the lane stall: plan order says after #268/#270
  (swarm.rs/ledger_entry.rs churn); CEO may elect to merge ahead.
- Test now at 44fee3c4 (artifacts staged, but merge-preview tree for
  win/android) vs wait for the true final tree: after #268/#270/#267 land,
  update-branch #272, and PR CI at the updated head produces EXACT
  artifacts of the shipped tree -- one SHA, everything included.

## 6. Governance guards (unchanged, all standing)

- No self-merge, no tag, no release; every merge individually
  CEO-approved; stop-and-report on any tip move or gate failure.
- Build serialization: only one cargo/gradle at a time on this host.
- No disk cleanup while cargo/rustc builds are live (scripts/clean_target.sh).
- Further worktree removal requires CEO buyoff on file (historical note).
- Return contract: raw command output, PR numbers, approval artifacts;
  report as RESULT/VERIFICATION/FILES/NOTES.

-- End of coordination note --