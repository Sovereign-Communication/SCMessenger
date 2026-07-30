# GPT HANDOFF -- strategic planning: 0.4.0 completion + 0.5.0 (iOS parity)

Status: READY FOR KICKOFF
Created: 2026-07-28
Executor: GPT-5.6 Sol (xhigh) on the operator's MacBook
BUDGET GUIDANCE: this task should cost roughly 10-12% of the weekly window
(60% remains; resets in ~6 days). Depth over breadth. The remaining budget
is reserved for: adversarial review of the swarm.rs transport packet, the
pre-tag final sweep, and 0.5.0 iOS-parity design/review. Plan accordingly
-- do not pad.

## Why this task exists

You are the strongest reasoner available to this project. The Windows
orchestrator (qwen3.8-max-preview) executes well but you out-think it on
cross-cutting strategy. Deliver the authoritative, efficient plan that
drives SCMessenger to a PERFECT, FUNCTIONAL v0.4.0, then v0.5.0 with iOS
at parity with Android. You plan; implementation is delegated (qwen lane
via the Windows orchestrator; Mac session for Swift/iOS code). You may
also make Swift/iOS code changes yourself on later tasks -- this task is
planning only.

## Locked decisions -- DO NOT relitigate

- v0.4.0 = "Josh test": two-person end-to-end messaging Hawaii <->
  Pennsylvania over the live AWS relay (100.56.248.69:9001, runs
  testbotz/scmessenger CI image). Real delivery, real receipts, no
  hand-watching. Clients: Windows CLI + Android app. iOS is NOT in 0.4.0.
- ALL open ledger-seeding security findings fixed BEFORE the
  v0.4.0-alpha.1 tag (operator mandate). Verdict file:
  HANDOFF/review/LEDGER_SEEDING_ADVERSARIAL_REVIEW_2026-07-25.md.
- iOS parity is IN SCOPE for v0.5.0 -- Android and iOS must reach
  feature parity (operator directive 2026-07-28).
- Phase 1 transport parity COMPLETE (signed P1-19). WiFi Direct waived
  to v1.1 (Android<->Android only). PQC depth (09/14) parked. Farm-sim
  hostile-network infra (P1-14/P1-18 AWS rig) PAUSED by operator -- the
  relay itself is live.
- Build authority: Windows host is the only authoritative Rust/Android
  build environment; the Mac is authoritative for iOS/xcodebuild gates.
- Implementation lanes: all code dispatches run qwen3.8-max-preview via
  the paid Alibaba plan through scripts/delegate_task.py (provider
  qwenpaid) on the Windows side; GPT = hardest thinking + adversarial
  review + Swift work.
- Orchestration protocol: docs/ORCHESTRATION.md + HANDOFF/todo/_QUEUE.md
  (sequencing authority: HANDOFF/V1_0_0_EXECUTION_PLAN.md, release slicing:
  HANDOFF/plans/MILESTONE_RELEASE_PLAN.md).

## Required reading (in this order)

1. HANDOFF/todo/_QUEUE.md -- live dispatch order + 2026-07-28 takeover
   header (routing directive, verified-done list, exclusions).
2. HANDOFF/plans/MILESTONE_RELEASE_PLAN.md -- v0.4.0 blockers/effort +
   v0.5.0 section.
3. HANDOFF/V1_0_0_EXECUTION_PLAN.md -- Section 0A sequencing + rules.
4. REMAINING_WORK_TRACKING.md -- current status log.
5. tmp/v040-completion-wave.md -- the in-flight wave (what is done, in
   flight, and the review findings being remediated RIGHT NOW: 1a/1b
   NO-SHIP verdicts, v2a/v2b/v2c remediation queue, packet 2 swarm.rs
   pending).
6. HANDOFF/gpt/GPT_IOS_LANE_COMPLETION_2026-07-28.md +
   GPT_IOS_LANE_FINDINGS_2026-07-28.md -- what iOS work is ALREADY done
   (receipt unification tested 47/47, relay de-hardcode, XCTest
   registration, bindings regenerated).
7. Spot-check as needed: core/src/store/ledger_entry.rs (remediation
   target), core/src/transport/swarm.rs (packet 2 target: ConnectToSeedPeers
   ~:5537/:6026, pending-dial resolution ~:4550, RelayAbuseGuardrails
   ~:563), android/app/src/main/java/com/scmessenger/android/ (parity
   reference), iOS/SCMessenger/SCMessenger/ (parity target).

## Verified state snapshot (2026-07-28, trust but spot-check)

DONE at HEAD: outbox Site-1 flush (f521f142); receipt round-trip +
classification (8f866bfc, core iron_core.rs:3064); Android retry
suppression (P3, 2026-07-23); ledger choke-point refactor (22b921ca);
graceful dial policy; queued-vs-connected command reply (held until
ConnectionEstablished); iOS receipt unification + de-hardcode + XCTest
registration (Mac-verified 47/47); farm-sim bootstrap repaired (/dns4 +
SC_BOOTSTRAP_NODES) + relay infra pulls CI image; CI green on main
(2026-07-24, iOS lane cancelled-tolerated).
IN FLIGHT (wip/v040-seeding-fixes): ledger cap/eviction/ordering (1a),
save-off-lock (1b -- NO-SHIP: lost-update regression), remediation queue
v2a (load cap + byte bounds + thresholds), v2c (save_lock + atomic
writes), v2b (anchor semantics + determinism + tests), then 1c
(mobile_bridge batch caller) and packet 2 (swarm.rs: F7a dial-policy
register gate, F7b record_failure wiring, F13 is_dialer gate on pending
dial resolution, NEW-6 global anti-Sybil bucket).
KNOWN OPEN beyond seeding: fresh CLI<->Android-emulator E2E delivery
proof at current HEAD (0A.8 mandate; evidence must be
ConnectionEstablished + delivered receipts, NOT dial-queue logs); P6-style
FFI snapshot hygiene; ios-build-test.yml path fix; version bump
0.3.5->0.4.0 + release artifacts + tag (operator pushes/tags); documented
residuals (sustained-burst anchor aging; cross-instance mobile
LedgerManager on shared storage path).

## Your deliverable -- one file, five sections

### Section 1: v0.4.0 completion plan
Ordered task list from NOW to a defensible v0.4.0-alpha.1 tag. Per task:
id, owner (GPT-think / Mac-Swift / qwen-impl-via-Windows /
Windows-orchestrator-gate / operator), inputs/deps, exact
evidence/gate required for DONE, rough size (S/M/L), and which tasks run
in parallel tracks vs the critical path. Include the tag checklist
(artifacts, CI, operator actions, Lucas port forwards/DDNS). Define the
minimum set that makes the Josh test REAL -- no scope creep, no gold
plating; flag anything in the current backlog that is NOT needed for
0.4.0 and should be explicitly deferred.

### Section 2: v0.5.0 plan with iOS-Android parity
The parity gap list (what Android has that iOS lacks -- transport
behaviors, receipt UI truth, retry suppression semantics, ledger/seed
consumption, settings, background delivery, bindings drift discipline)
with file-level targets under iOS/SCMessenger/, ordered, sized, and gated
(xcodebuild evidence per task, Mac-authoritative). Plus whatever
MILESTONE_RELEASE_PLAN assigns to 0.5.0. Include a bindings-drift
prevention discipline (regen/check gate so the checksum-mismatch class
never recurs).

### Section 3: verification and evidence standards
The exact evidence each version gate demands so "done" is never a log
line again: delivery proof protocol (what counts as a real delivery, what
counts as a real receipt round trip, how to capture ConnectionEstablished
evidence on CLI/Android/iOS), and the per-PR gate matrix (Windows gates,
Mac gates, adversarial-review triggers).

### Section 4: top-5 "looks done but isn't" risks
Concrete failure modes that would let 0.4.0 ship broken while every
checklist box is ticked (think delivery-truth disease classes this repo
has already suffered: queued-vs-connected, silent false success,
receipt misclassification). Each with the specific verification that
kills it.

### Section 5: GPT budget allocation
Given ~60% weekly budget remaining and a 6-day reset: which future tasks
deserve xhigh GPT thinking (name them), which deserve only a GPT review
pass, and which must NOT touch GPT at all. Allocate percentages summing
to <= 50% (reserve 10% contingency).

## Delivery

Commit the plan as HANDOFF/gpt/GPT_PLANNING_040_050_VERDICT.md on branch
gpt/planning-040-050, push, open a PR (full branch/PR autonomy per
AGENTS.md MAC LANE -- do NOT merge). Also print a one-page executive
summary in your session output. No emojis anywhere. Where you disagree
with a locked decision's EXECUTION (not its existence), say so explicitly
with the alternative -- the operator reads those notes.
