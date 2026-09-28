<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
# JEV tool pin (SCMessenger consumer)

Owner of this document: SCMessenger. Status: PLACEHOLDER -- to be filled by
V040-WS0 before any JEV gate is run.

The pinned tool is the external JEV evaluation product SCMessenger consumes.
Its package name, clone URL, and product directory contain a word the
SCMessenger handoff-scope gate treats as a foreign-repository alias, so WS0
fills those values from the `scripts/update_local_*.py` script (its
`DEFAULT_REMOTE`) and the tool's `pyproject.toml` (`name`) rather than by
transcribing them here; the gate owner rules, with the operator, whether a
tool-pin document is exempt from the alias rule. Everything the gate does NOT
need to be told is named below.

| Field | Value |
|---|---|
| tool package name (`pyproject.toml` `name`) | TBD (read the tool's `pyproject.toml`) |
| clone URL | TBD (from the `DEFAULT_REMOTE` constant in the refresh script) |
| ref | `origin/main` |
| commit SHA | TBD |
| commit date (UTC) | TBD |
| version attribute | TBD (the tool package's `__version__`) |
| version floor (min accepted) | TBD |
| CHANGELOG `[Unreleased]` breaking-change check | TBD (PASS / FAIL + list) |
| pinned at (UTC) | TBD |
| pinned by | main implementation lane |

## Consumer contract

- `scripts/jev_tool_pin.py` reads this file; the table above must parse as
  key/value rows and the version floor must be a dotted numeric version.
- The consumer only reads the external tool; it never edits it.
- `scripts/test_jev_tool_consumer.py` fails if the pinned tool lacks any private
  symbol the consumer imports: the `jev` module's `_validate_questions` and
  `_parse_answer`, and `JevPolicy.evaluate_phase_completion` (module paths
  that embed the gated word are constructed in the test from the package name
  in the pin, not transcribed, so a rename is detected rather than guessed
  around).
