<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
# JEV integration contract (SCMessenger, 2026-09-25)

Owner of this document: SCMessenger. This is the canonical contract for
SCMessenger's JEV completeness gates. It supersedes the older JEV integration
note in `HANDOFF/` dated 2026-09-21 (its file name contains the word the
handoff-scope gate treats as a foreign alias; the gate owner must either exempt
that path or rename it -- see the alias policy item in the coordination plan).
The external JEV tool's tree is read-only to SCMessenger; every command below
goes through the local-import helper in `scripts/`.

## Version pin

- Pin record: `HANDOFF/jev/JEV_TOOL_PIN.md` (filled by WS0).
- Guard: `python scripts/jev_tool_pin.py --print` exits 0 only when the resolved
  tool version is >= the pin floor; every JEV script calls it through the
  local-import helper's root-resolution function.
- Refresh: the `scripts/update_local_*.py` script (gitignored consumer copy
  under `vendor/`, branch `consumer` at `origin/main`).

## Gate 1 -- keyed canonical (per WP)

`python scripts/jev_canonical_check.py --wp WPn --state-file <state.json>`

State JSON keys (see the template `scripts/wp_state_template.json`):

| Key | Meaning |
|---|---|
| `wp` | WP id (WP1..WP5) |
| `instruction` | the WP instruction text from the implementation plan |
| `files` | changed files |
| `acceptance` | acceptance rows |
| `evidence` | command outputs / CI run / logs |
| `canon_identity` | one contact identity flavor kept |
| `canon_routing_feed` | one routing feed entry point |
| `instruction_matches` | acceptance rows met with cited evidence |

The template lives under `scripts/`, not `HANDOFF/`, because the handoff-scope
gate treats every file under `HANDOFF/` as a handoff and a valid JSON document
cannot carry the gate's comment-style scope block.

Exit 0 requires a keyed result with `is_passing(0.70)` from the evaluator
(TypeSafe, or the OpenRouter `~typesafe/jev-latest` decisions fallback).
`--allow-fallback` prints `UNVERIFIED-JEV` and is never DONE.
WP1/WP2 state files exist only in PR #354; that PR lands before any WP DONE
claim.

## Gate 2 -- phase bar (per item, JEV-BAR layer)

`python scripts/jev_phase_check.py --phase <id> --evidence <evidence.json>`

Exit 0 requires: all six hard gates true (`pr_merged`, `origin_evidence`,
`required_tests_present`, `local_gates_green`, `ci_green`, `no_open_blockers`),
combined score >= 85, no blocking sentiment axis, `can_mark_complete` true. See
`PHASE_BAR_GATES.md` for the evidence schema and axis definitions. The tool
command is the tool's `jev-phase` subcommand, invoked by module
(`python -m <tool package>.cli jev-phase`).

## Read-only insight

`python scripts/jev_repo_insights.py --mode full [--batch-size 40]` -- read
triage only, never a DONE signal.

## What JEV is not

JEV is a completeness judgement, not a security verdict. It never satisfies
Rule-8, never replaces `rules_check`, `pr_scope.sh`, or CI, and a JEV failure
means iterate the implementation until the evidence supports completion.
