# V040-T-WATCH-POS — log-silence watchdog positive test

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: OPEN (filed 2026-09-20)
Priority: P1 (tag-adjacent correctness claim)
Lane: Freebuff
Scope: `cli/` watchdog tests and, only if required, bounded production
parameter documentation. Prefer test-only. If production logic must change to
make the positive case hold, stop and write `inbox/` — that becomes a design
decision.

## Finding

Multi-dimensional audit 2026-09-17 finding N-03: the log-silence heartbeat
watchdog **exits the process by design** when logs go quiet. That remediated
the 2026-09-15 silent wedge, but a healthy-but-quiet node can be
indistinguishable from a wedge. A **positive test** (quiet node, no wedge,
must not die) was not on file at audit time.

Anchors: `cli/src/main.rs` event-loop watchdog + log-silence heartbeat
watchdog; `logs/watchdog/watchdog.log`.

## Premise

The watchdog exists and terminates on silence. The missing piece is evidence
that silence-without-wedge does **not** kill the process.

## Work

1. Read current watchdog configuration and conditions on `origin/main`.
2. Add a test (or hermetic harness script under `scripts/` if JVM/process
   spawn is required) that:
   - constructs or runs a node path with logging quiet but event loop alive
     (or the smallest equivalent unit that exercises the watchdog decision
     function)
   - asserts the process/decision **does not** fire exit in that window
3. If the decision function is not unit-testable, extract a pure predicate
   and test that — do not leave the gate untestable.
4. Record the exact command + output in the PR body.

## Scope correction

- Do not delete the watchdog.
- Do not lengthen timeouts indefinitely to "pass" the test.
- Do not touch transport/swarm for this ticket.

## Acceptance

1. New test exists and fails if the predicate is inverted.
2. Documented command output green on Windows host (orchestrator re-runs).
3. No production behavior change, or inbox PREMISE-WRONG/BLOCKED if a change
   is required.

## Review gate

None if test-only. If production watchdog logic changes in `cli/`, none for
core dirs — but orchestrator still verifies the gate output.

## Rules

No emojis. Evidence contract. Shared checkout scope.
