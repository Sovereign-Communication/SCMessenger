<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
# V040-WS1 -- JEV-BAR phase-completion gate for every work item

Owner of this ticket: SCMessenger (main implementation lane, single writer).

Status: OPEN (filed 2026-09-25 by the coordination seat; depends on WS0 pin)
Priority: HIGH -- this is the "iterate until fully implemented" gate
Lane: main implementation lane (single writer)
Scope: new `scripts/jev_phase_check.py`, new `scripts/test_jev_phase_check.py`,
`HANDOFF/jev/PHASE_BAR_GATES.md`, `HANDOFF/jev/JEV_INTEGRATION_2026-09-25.md`,
and the shared state template `scripts/wp_state_template.json`.
Do not touch `core/`, `cli/`, `android/`, `Cargo.*`, `vendor/`, or the external
tool's tree.

## Premise (verified 2026-09-25)

- The checked-out external JEV tool tree's `jev-phase` command scores a phase
  against `packs/phase_completion.pack.json`: six axes
  (`merge_evidence`, `gate_tests`, `verification` code-owned;
  `status_honesty`, `residual_scope`, `dogfood` JEV-owned), five sentiment
  levels, six hard gates (`pr_merged`, `origin_evidence`,
  `required_tests_present`, `local_gates_green`, `ci_green`, `no_open_blockers`),
  combined score `0.7 * mechanical + 0.3 * semantic`, `can_mark_complete` =
  all hard gates AND score >= `--min-score` (default 85) AND no blocking axis.
  The layer is fail-closed (JEV may only lower code authority).
- The tool exposes this through its CLI (`--phase`, `--all`, `--repo-root`,
  `--evidence`, `--min-score`, `--local-only`, `--json`) and through
  `JevPolicy.evaluate_phase_completion`. SCMessenger has no wrapper for it.

## Work

1. New `scripts/jev_phase_check.py`:
   - inputs: `--phase <id>` (required unless `--all`), `--repo-root` (default
     repo root), `--evidence <json>` (optional overrides for a single phase),
     `--min-score` (default 85), `--local-only`, `--json`;
   - resolve the tool through the local-import helper in `scripts/` (WS0 pin
     guard applies);
   - invoke the tool's `jev-phase` CLI as a subprocess with `PYTHONPATH` set to
     the pinned root, capture the JSON result; do NOT import private tool
     modules from here;
   - exit 0 only when `can_mark_complete` is true, every hard gate is true,
     the score >= `--min-score`, and `bar.blocking_axes` is empty; print every
     blocker, blocking axis, and `improvements` entry on failure;
   - `[FAIL]` if the tool result is fallback/heuristic unless `--local-only` was
     passed explicitly, in which case print `UNVERIFIED-JEV` and exit 1;
   - `[FAIL]` if the evidence JSON is missing required keys.
2. Evidence schema (`HANDOFF/jev/PHASE_BAR_GATES.md`): the JSON every item must
   supply -- `phase`, `pr_merged`, `origin_evidence` (PR/SHA), `tests_missing`,
   `open_blockers`, `status_row` text, local gate log paths, CI run URL, and
   dogfood evidence (receipts/cost/fallback) for user-facing items. The shared
   state template `scripts/wp_state_template.json` covers gate 1.
3. Hermetic tests `scripts/test_jev_phase_check.py`: fake tool result JSON for
   (a) all gates pass, score >= threshold, no blocking axis -> 0; (b) any hard
   gate false -> 1; (c) blocking axis -> 1; (d) score below threshold -> 1;
   (e) fallback result -> 1 with `UNVERIFIED-JEV`; (f) missing evidence key ->
   1. No network.

## Acceptance

1. `python scripts/jev_phase_check.py --phase V040-WS1 --evidence <file> --json`
   exits 0 for a passing fixture and 1 for each failing fixture.
2. `python scripts/test_jev_phase_check.py` exits 0.
3. The evidence schema and the pin are referenced from
   `HANDOFF/jev/JEV_INTEGRATION_2026-09-25.md`.

## Stop rules

Stop and report if the pinned tool's `jev-phase` output shape differs from the
above (schema change is an upstream break; do not patch the consumer to guess),
or if any path in Scope overlaps the reserved list.
