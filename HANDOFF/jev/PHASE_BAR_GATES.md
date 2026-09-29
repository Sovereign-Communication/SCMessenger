<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
# JEV-BAR phase-completion gate (SCMessenger reference)

Reference for the external tool's `jev-phase` gate, which
`scripts/jev_phase_check.py` wraps. Definitions are taken from the tool's
`packs/phase_completion.pack.json` at the pinned commit
(`HANDOFF/jev/JEV_TOOL_PIN.md`).

## Hard gates (all must be true)

`pr_merged`, `origin_evidence`, `required_tests_present`, `local_gates_green`,
`ci_green`, `no_open_blockers`.

## Sentiment axes (six)

| Axis | Authority | Bucket |
|---|---|---|
| `merge_evidence` | code | `merge_pending` |
| `gate_tests` | code | `tests_missing` |
| `verification` | code | `gates_unverified` |
| `status_honesty` | JEV | `status_dishonest` |
| `residual_scope` | JEV | `residual_untracked` |
| `dogfood` | JEV | `dogfood_missing` |

Levels: blocking (0), at_risk (35), mixed (60), confident (85), proven (100).
An axis at level `blocking` fails the bar regardless of score. The combined
score is `0.7 * mechanical + 0.3 * semantic`; the default threshold is 85.

## Evidence JSON schema (SCMessenger side)

```json
{
  "phase": "V040-WS1",
  "pr_merged": true,
  "origin_evidence": "PR #NNN, squash <sha>",
  "tests_missing": [],
  "open_blockers": [],
  "status_row": "DONE -- <one line>",
  "local_gates_green": true,
  "ci_green": true,
  "dogfood_evidence": "receipts/cost/fallback for user-facing items"
}
```

Missing keys are a hard failure of the wrapper, not a pass.

### Which of those keys the tool's `--evidence` actually honors (verified 2026-09-25)

Exercised against the checked-out tool with `jev-phase --local-only --json
--evidence <file>`: the tool copies only `pr_merged`, `local_gates_green`,
`ci_green`, `origin_evidence`, `gate_output`, `ci_run`, `open_blockers`, and
`status_row` from the override file. `tests_missing` and `dogfood_evidence`
are ignored; `tests_missing` is recomputed from the phase contract's
`required_tests` and the files on disk (a run that claimed
`tests_missing: ["tests/does_not_exist.py"]` still reported
`tests_missing: []` and exit 0). The wrapper must therefore validate the full
schema above itself and must not assume a pass-through.

Two consequences for the wrapper's exit logic:

- With an override, a dishonest `status_row` (`**complete**` plus `open` or
  `repair` language) no longer produces a hard-gate blocker: the tool
  derives `open_blockers` from the roadmap row before applying the override.
  The bar still fails closed, but through `bar.blocking_axes` containing
  `status_honesty`, not through `blockers`.
- A missing required test puts BOTH code axes (`gate_tests`, `verification`)
  on `bar.blocking_axes`, not only `verification`.

## Exit semantics of `scripts/jev_phase_check.py`

- 0: all hard gates true, score >= `--min-score` (default 85), no blocking axis.
- 1: any hard gate false, score below threshold, blocking axis present, tool
  result fallback (prints `UNVERIFIED-JEV`), or missing evidence key.
- 2: the pinned tool could not be resolved (pin guard failure).

## Tool exit semantics observed 2026-09-25 (the wrapper's subprocess)

- `can_mark_complete` true: exit 0, no stderr.
- `can_mark_complete` false: prints the result (or JSON with `--json`), then
  `[FATAL] phase <id> completion score <score> < <min> or hard gates failed --
  do not mark complete` on stderr and exit 1. The message is the same for a
  score shortfall and for a blocking axis, so read the JSON, not the message.
- `--all` with any phase in `false_complete` (STATUS row claims complete while
  the bar fails): exit 1 with the `false_complete` list; honestly-open phases
  that fail the bar are reported in `failing` and are not an error.
- Explicit `--pack` path missing or invalid: `[FATAL]`, exit 1 (fail closed).
- Neither `--phase` nor `--all`: `[FATAL] jev-phase requires --phase or --all`,
  exit 1.
- `--out <path>` writes the same result object as `--json` prints.
