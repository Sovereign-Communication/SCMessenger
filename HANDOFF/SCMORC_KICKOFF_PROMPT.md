# /scmorc Kickoff Prompt (v1.0.0 backlog drain)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Active
Last updated: 2026-07-06

Recommended launch: **Opus 4.8, effort medium** (`/model claude-opus-4-8`,
default/medium effort). Rationale: the orchestrator never writes product code —
it triages, routes, verifies, and keeps git/HANDOFF state clean; that needs
judgment one tier above the sonnet workers but not Fable-class reasoning, and
medium effort fits a procedural loop (the deep thinking is delegated to workers
via the routing table's per-task effort). Drop to sonnet/high for the
orchestrator itself only when the usage window is under ~35%.

Paste everything below the line into a fresh session after `/scmorc <window%>`
(or as the argument text). It assumes the 2026-07-06 sprint commit `c76bd897`
is HEAD or an ancestor.

---

/scmorc — Drain the v1.0.0 backlog. Current window: <FILL IN %>.

Context you can trust without re-deriving (verified 2026-07-06):
- The Fable 5 stabilization sprint landed as commit c76bd897: async FFI surface
  (14 fns), TCP-listener-zombie fix, SubnetProbe ANR fix, gossipsub reply
  channels, Android unit tests RE-ENABLED (101 tests: 94 pass / 7 inventoried
  drift failures / 2 skipped; RoleNavigationPolicyTest 3/3 PASS).
- Verified green at that commit: cargo check --workspace, cargo fmt,
  gradle assembleDebug, docs-sync. NOT yet run: clippy,
  cargo test --workspace (--no-run and full), wasm32 target check.
- The sprint diff has NOT had its adversarial security review (the in-session
  auditor died on the token limit). Until NEXT_ITER_02's verdict is on file,
  treat the sprint as unreviewed transport/crypto work.

Your standing orders:
1. Work `HANDOFF/todo/_QUEUE.md` strictly from the top. Items 1-2
   (NEXT_ITER_01 compile gates + test triage, NEXT_ITER_02 adversarial review)
   are the verification chain for the sprint — nothing else in Phase 1
   dispatches until both are done or explicitly blocked.
2. Phase 1 (Windows/Android transport parity) drains completely before any
   Phase 2 work (PQC_*, TASK_KMP_*, WS-*) — those are frozen until the P1-19
   exit review passes. Do not relitigate the sequencing; it is operator-settled
   in HANDOFF/V1_0_0_EXECUTION_PLAN.md.
3. [DEVICE] items (NEXT_ITER_04, P1-04 retest, P1-09/14/18): prepare worker
   prompts and playbooks, then PARK them and notify the operator with exactly
   what you need (Pixel connected via adb, window of time). Work the next
   non-device queue item meanwhile.
4. Routing per the scmorc table; escalate effort before model. All diffs
   touching core/src/{crypto,transport,routing,privacy} get a read-only
   fable(high) adversarial pass before their task closes.
5. Escalate to the operator (stop, don't improvise) on: Critical/High audit
   findings, wasm32 UniFFI-async compile failures, any API-contract or
   architecture decision, anything NEEDS_REVIEW from pre-dispatch validation.
6. Keep state pristine: one commit per completed task
   (`native: completed [Task]`), queue statuses updated in _QUEUE.md, zero
   uncommitted files between dispatches, dispatch log appended in
   tmp/scmorc/dispatch_log.md.
7. Also do the cheap orchestrator-local hygiene early: triage the 9 stale
   HANDOFF/IN_PROGRESS/ files from June (queue item 5) — close superseded ones
   with notes so backlog counts stop lying.

Begin with quota check, then queue item 1.
