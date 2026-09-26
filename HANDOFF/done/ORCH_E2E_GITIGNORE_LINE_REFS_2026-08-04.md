# ORCH_E2E_GITIGNORE_LINE_REFS_2026-08-04 -- fix stale .gitignore line numbers

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: todo
Tier: FLASH
Domain: docs
Target Files: HANDOFF/ORCHESTRATION_TOKEN_STRATEGY.md:L80-L86

## Requirement

Part 1.1 of HANDOFF/ORCHESTRATION_TOKEN_STRATEGY.md says: "tmp/ is
gitignored (.gitignore lines 5 and 93; confirmed with git check-ignore)".
The line numbers are stale: `git check-ignore -v tmp/` on the current tree
reports `.gitignore:6:tmp/*` and `.gitignore:94:tmp/`. Update the citation
so it reads lines 6 and 94. Change ONLY the two line numbers inside that
parenthetical; do not reword, reflow, or touch anything else in the file.

## Acceptance criteria

- The parenthetical reads "`.gitignore` lines 6 and 94".
- Zero other content changes in the diff.
- No emoji introduced; the file stays clean for scripts/rules_check.py.

## Gate

python scripts/rules_check.py

## Provenance

End-to-end tooling test of the Qwen Code orchestration setup
(orchestrate -> dial -> delegate -> footer parse -> gate -> handoff),
2026-08-04. Free lane only.
