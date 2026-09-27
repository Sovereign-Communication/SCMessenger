<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
# V040-WS0B -- Per-item JEV gate enforcement in the burndown

Owner of this ticket: SCMessenger (main implementation lane, single writer).

Status: OPEN (filed 2026-09-25 by the coordination seat; process change)
Priority: HIGH -- without this the gates exist but nothing requires them
Lane: main implementation lane (single writer)
Scope: `.github/pull_request_template.md` (or the repo's current PR template),
`docs/rules/FREEBUFF.md`, `HANDOFF/freebuff/README.md`,
`HANDOFF/todo/_QUEUE.md`, `HANDOFF/jev/JEV_INTEGRATION_2026-09-25.md`.
Do not touch source trees.

## Rule

A work item is DONE only when, on its final head:

1. mechanical gates pass (typecheck/tests/clippy, `rules_check`, `pr_scope.sh`,
   required CI checks) and, for `core/src/{crypto,transport,routing,privacy}`,
   a recorded adversarial APPROVE from a reviewer who did not author it;
2. `python scripts/jev_canonical_check.py --wp WPn --state-file <state.json>`
   exits 0 with a keyed result (`--allow-fallback` is never DONE);
3. `python scripts/jev_phase_check.py --phase <id> --evidence <file>` exits 0
   (WS1);
4. the PR description carries the state-file path, its commit, the evidence
   path, and both exit codes.

## Work

1. Add the four fields to the PR template as a required block.
2. Add the rule to `docs/rules/FREEBUFF.md` ("Never idle" and the paste
   protocol) and to the Freebuff README's train table.
3. Add one queue row per WS ticket (WS0, WS0B, WS1, WP4-D8) in the same README
   edit that lands PR #368.
4. Add `_QUEUE.md` rows in the same status convention as the existing rows.

## Acceptance

- The PR template and both ledgers contain the rule; the README edit is part of
  the #368 landing, not a second concurrent edit.

## Stop rules

Stop and report if the #368 README edit has already landed (then apply the
rows on top of it, not in parallel).
