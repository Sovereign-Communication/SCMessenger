# TASK: Unify Gemini orchestration docs — eliminate all conflicts

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Context

There are 3 docs a Gemini session loads for orchestration. They currently conflict. Rewrite them so they are perfectly consistent and non-contradictory.

## Files to rewrite

### 1. `GEMINI.md` (repo root)
Currently just says "read AGENTS.md". Needs to be updated to also point to `docs/GEMINI_ORCHESTRATOR.md` as the active protocol, while keeping the AGENTS.md reference. Keep it SHORT — just a pointer doc.

New content should be:
```
# GEMINI.md
Status: Active
Last updated: 2026-07-11

Gemini-family tools (Antigravity/agy, Gemini CLI) must read, in order:
1. AGENTS.md — hard rules and capability class (you are FOREIGN WORKER)
2. docs/GEMINI_ORCHESTRATOR.md — live orchestration protocol (delegation loop, model fleet, commands)

You are the orchestrator. Do not implement. Delegate to Qwen. See docs/GEMINI_ORCHESTRATOR.md.
```

### 2. `docs/ORCHESTRATION_PLAYBOOK.md`
Currently conflicts with `GEMINI_ORCHESTRATOR.md` in these ways:
- Uses `qwen-max` (old). Correct model is `qwen3-max`
- Uses `export CARGO_INCREMENTAL=0` (Linux). On Windows PowerShell it must be `$env:CARGO_INCREMENTAL="0"; $env:PATH += ";C:\Users\SCM\.cargo\bin"`
- Step 5 says to commit (`git commit`) — Gemini CANNOT commit per AGENTS.md
- References `PQC_07` specifically — should be generic for any task

Rewrite `docs/ORCHESTRATION_PLAYBOOK.md` to:
- Remove all PQC-07-specific references, make it generic `<TASK>`
- Fix model name: `qwen3-max`
- Fix Windows PowerShell cargo commands
- Remove the git commit step (Gemini cannot commit — report only)
- Add a note at the top: "See docs/GEMINI_ORCHESTRATOR.md for the full protocol. This file is the quick command reference."
- Keep the 5-step structure but make it accurate

## Additional requirement (2026-07-11)

A new master protocol doc now exists: `docs/ORCHESTRATION.md`. Both rewritten
files must state that they are subordinate to it: GEMINI.md adds it as
reading item 3; ORCHESTRATION_PLAYBOOK.md's top note becomes: "Subordinate to
docs/ORCHESTRATION.md (master protocol) and docs/GEMINI_ORCHESTRATOR.md (role
protocol). This file is the quick command reference." Also mention the new
dispatcher flags `--tier`, `--verify "<cmd>"`, and `--max-rounds N` in the
playbook's dispatch example.

## Rules
- No emoji
- Return full content of both files
- File 1 path: `GEMINI.md`
- File 2 path: `docs/ORCHESTRATION_PLAYBOOK.md`
- Format: standard markdown code blocks with `// GEMINI.md` and `// docs/ORCHESTRATION_PLAYBOOK.md` as first lines
