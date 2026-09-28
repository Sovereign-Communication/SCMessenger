<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
# V040-WS0 -- Upgrade the JEV tool consumer to the newest version

Owner of this ticket: SCMessenger (main implementation lane, single writer).

Status: OPEN (filed 2026-09-25 by the coordination seat; not yet implemented)
Priority: HIGH -- every JEV completeness gate depends on the pinned tool
Lane: main implementation lane (single writer)
Scope: the three tool-consumer scripts under `scripts/` (the mandatory-gate
wrapper, the local-import helper, and the refresh script; their names are the
ones already used by `scripts/jev_canonical_check.py` and
`scripts/jev_repo_insights.py`), `scripts/jev_canonical_check.py`, new
`scripts/jev_tool_pin.py`, new `scripts/test_jev_tool_consumer.py`,
`HANDOFF/jev/JEV_TOOL_PIN.md`, `HANDOFF/jev/JEV_INTEGRATION_2026-09-25.md`,
the 0.4.0 JEV integration note in `HANDOFF/` dated 2026-09-21, and the tool
setup runbook under `docs/runbooks/`. Do not edit the external tool's tree. Do
not touch `core/`, `cli/`, `android/`, `Cargo.*`, `vendor/libp2p-swarm-0.48.0/`,
`.github/workflows/ci.yml`, or `AGENTS.md`.

The "JEV tool" is the external JEV evaluation product SCMessenger consumes.
Its package name, clone URL, product directory, environment-variable name, and
consumer-copy directory all contain a word the SCMessenger handoff-scope gate
treats as a foreign-repository alias -- and the gate flags that word inside
file names and identifiers too. This ticket therefore refers to the tool as
"the tool", names its scripts by role, and reads the exact names from the
`scripts/` modules the implementer is already editing. Private symbols the
consumer imports are named by module path only (`jev._validate_questions`,
`jev._parse_answer`).

## Premise (verified 2026-09-25 against main and the checked-out tool)

- SCMessenger's consumer layer assumes tool version 0.3.3 (the 2026-09-21
  integration note; PR #360 never landed).
- The checked-out tool tree is at 0.4.1 plus an unreleased JEV-BAR layer whose
  CHANGELOG `[Unreleased]` section adds: per-axis sentiment buckets,
  `packs/phase_completion.pack.json` (6 axes, 5 levels, default threshold 85),
  a `jev-phase` subcommand with `--all --min-score`, `can_mark_complete`,
  `improvements:`, `--evidence`, and four CLI/correctness fixes.
- The tool's CLI subcommands now include more than the six `--kind` values the
  wrapper accepts (`verify spend ledger trust lint-claims smoke`).
- The tool exposes no `--version` flag; version is the package's `__version__`
  with a `pyproject.toml` fallback.
- The local-import helper imports two PRIVATE symbols from the tool's `jev`
  module -- `_validate_questions` and `_parse_answer` -- to validate the
  OpenRouter fallback answers. A 0.x release may move them; the consumer must
  fail loudly, not silently fall back to unvalidated answers.

## Work

1. Pin. Create `HANDOFF/jev/JEV_TOOL_PIN.md` recording: tool package name,
   clone URL, the resolved `origin/main` commit SHA, the version, the commit
   date, the CHANGELOG `[Unreleased]` breaking-change check result, and the UTC
   timestamp. The pin is the ONLY place the version floor lives.
2. Refresh. Run the refresh script (gitignored consumer copy under `vendor/`,
   branch `consumer` at `origin/main`). Print the resolved SHA and version in
   `[OK]` lines; fail if the tool's `jev.py` is missing (already does) and fail
   if the version is below the pin floor.
3. Version guard. New `scripts/jev_tool_pin.py`: read the pin file, resolve the
   tool root from the environment variable the local-import helper already
   reads or from its `vendor/` consumer copy, read the package `__version__`
   (fall back to `pyproject.toml`), compare against the pin floor, and exit
   non-zero with `[FAIL]` on mismatch. `--print` emits the resolved root/SHA/
   version for run logs. The local-import helper's root-resolution function
   calls it so every JEV script inherits the guard.
4. Wrapper. Extend the wrapper's `--kind` choices to the tool's current
   subcommand set; add a `--version-check` that runs the pin guard before any
   kind; print the resolved version in every `[RESULT]` line. Keep exit codes
   0/1/2/3/4.
5. Compat test. New `scripts/test_jev_tool_consumer.py` (unittest, hermetic):
   - the two private symbols the consumer imports are present with the expected
     signatures, else `[FAIL]` with the exact symbol;
   - `JevPolicy` has `evaluate_phase_completion`;
   - the pin file parses, names a commit SHA and a version, and the version is
     >= the floor recorded in the integration contract;
   - the local-import helper returns every key the JEV scripts use.
   Add a CI step that runs it (the gate owner's CI edit, not this task).
6. Docs. Replace every 0.3.3 reference in SCMessenger docs with the pinned
   version and the pin-file path. `HANDOFF/jev/JEV_INTEGRATION_2026-09-25.md`
   becomes the canonical contract (WP state-file schema, pin, commands).

## Acceptance

1. `python scripts/jev_tool_pin.py --print` exits 0 and prints root, SHA, version.
2. `python scripts/test_jev_tool_consumer.py` exits 0.
3. `python scripts/jev_canonical_check.py --wp WP1 --state-file <state.json>`
   still runs against the pinned tool (use `--allow-fallback` only to prove the
   path executes; that is not DONE).
4. Diff touches only the files in Scope plus CI (gate owner).

## Stop rules

Stop and report if: the tool's newest `origin/main` cannot be resolved on this
host; a private symbol the consumer imports is gone; the CHANGELOG
`[Unreleased]` section lists a breaking change to a symbol the consumer uses; or
any path in Scope overlaps the reserved list.
