# REGENERATE_LAKE_REGISTRY -- registry.json regeneration script

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: todo
Tier: CODER (mechanical; FLASH acceptable if scoped)
Domain: tooling
Target Files: scripts/regenerate_registry.py (new)

## Requirement

`tmp/lakes/registry.json` is gitignored, so the qwenpaid routing fix
(HANDOFF/ORCHESTRATION_TOKEN_STRATEGY.md Part 1.1, commit 9121fd3e) does not
survive a fresh checkout: `lake_route.py` silently skips any lake missing
from the registry, and the operator's primary lane would never be picked.
Write `scripts/regenerate_registry.py` that rebuilds
`tmp/lakes/registry.json` from the lake tables in
`docs/orchestration/SCM_UNIFIED_LAKE_ORCHESTRATION.md` Section 1 -- including
the qwenpaid block (CODER/THINK/MAX = qwen3.8-max-preview) -- and prints a
summary of what it wrote.

## Acceptance criteria

- `python scripts/regenerate_registry.py` rewrites `tmp/lakes/registry.json`
  with every wired lake (qwen, qwenpaid, groq, openrouter, ollama, gemini)
  and their per-tier model lists.
- The regenerated file keeps the existing `generated` / `source` metadata
  fields and the qwenpaid block for CODER/THINK/MAX.
- `python scripts/lake_route.py --help` still runs clean after regeneration
  (no schema breakage).
- Idempotent: running twice produces the same file.
- No emoji anywhere; `python scripts/rules_check.py` exit 0 on the new file.

## Gate

python scripts/rules_check.py && python scripts/regenerate_registry.py

## Notes for the orchestrator

Do NOT hand-edit registry.json again -- once this lands, regeneration is the
only path. The doc tables are the source of truth; if doc and current
registry disagree on model names, verify against
`scripts/delegate_task.py`'s provider wiring before writing.
