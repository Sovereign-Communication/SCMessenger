# ORCHESTRATE_STRICT_HARDENING -- make orchestrate_strict.py gate-safe

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: todo
Tier: CODER
Domain: tooling
Target Files: scripts/orchestrate_strict.py

## Requirement

`scripts/orchestrate_strict.py` composes the whole loop (dial ->
delegate_task -> footer parse -> batch_handoff) but is NOT safe to run
unsupervised yet (findings in
HANDOFF/audit/ORCHESTRATION_AUDIT_QWEN_TAKEOVER_2026-08-04.md). Until this
ticket lands, the manual 9-step loop in ORCHESTRATION.md Section 2.2 remains
the safe path. Required changes:

1. Wrap the `delegate_task.py --verify` invocation in
   `scripts/build_lock.py --run` (ORCHESTRATION.md Section 2.2 step 5 and
   Section 9 lesson 5 -- one build at a time on Windows).
2. Honor the dial's `security_gate_required` / `delivery_gate_required`
   flags: stop the batch and surface the gate to the orchestrator instead of
   proceeding to commit (Section 2.2 step 7).
3. Pass the real dispatch lake as `batch_handoff.py --provider` instead of
   the literal `mixed`, so commit provenance names the actual lake.
4. Ensure `tmp/tasks/` exists and document the `<ID>.dispatch.md`
   prompt-file convention the script reads, in `tmp/tasks/README.md`
   (tmp/ is gitignored; the README is a local convenience, keep it short).

## Acceptance criteria

- `orchestrate_strict.py` dry-run mode still completes cleanly against the
  live queue (behavior unchanged for the dry path).
- A held `build_lock.py` lock makes the verify step wait or fail loudly --
  never run a second concurrent gate.
- No emoji anywhere; `python scripts/rules_check.py` exit 0.

## Gate

python scripts/rules_check.py

## Notes for the orchestrator

Do not enable unattended runs of this script until the security-gate
handling is verified against a real gated diff (a
`core/src/{crypto,transport,routing,privacy}/` touch must stop the batch).

## v2 implementation state (2026-08-10)

The v2 kernel now persists dial security and delivery gates, retains
content-addressed isolated-worker diffs for integration, requires the declared
independent review roles, and serializes standalone verification invocations
across worktrees. Deterministic coverage is in `scripts/test_orchestration_v2.py`.

This ticket remains in `todo/` deliberately: the active Mac lane may not move
HANDOFF task state, and the live dry-run could not obtain a deterministic lake
on 2026-08-10 (`[BLOCKED] ... no lake available for tier CODER`). A Windows
orchestrator must rerun the live-queue dry path with an available provider and
make the authoritative HANDOFF transition; it must not infer completion from
the deterministic unit suite alone.
