# DELEGATE_SCOPED_FILES_ALLOWLIST_FIX -- scoped --files paths break diff apply

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: todo
Tier: CODER
Domain: tooling
Target Files: scripts/delegate_task.py

## Symptom (observed 2026-08-04, Qwen orchestration-setup e2e test)

Dispatch with a scoped target:

    python scripts/delegate_task.py --task <ticket> --provider groq \
      --model llama-3.1-8b-instant \
      --files HANDOFF/ORCHESTRATION_TOKEN_STRATEGY.md:L80-L86 \
      --apply --mode diff

The worker returned a well-formed diff whose headers named
`a/HANDOFF/ORCHESTRATION_TOKEN_STRATEGY.md` -- the exact target -- yet the
apply path rejected every hunk:

    [REJECTED] diff targets file(s) outside --files/--allow-new-file,
    dropping those hunks: ['HANDOFF/ORCHESTRATION_TOKEN_STRATEGY.md']
    [WARN] diff apply failed; falling back to full-file mode for this task

Full-file fallback then found no file blocks, nothing was applied, and the
script exited 0 (should surface as vacuous per Section 9 lesson 2). The
scoped-targeting feature (commit 9121fd3e) is therefore unusable end to
end: every scoped dispatch is rejected.

## Root-cause hypothesis (verify, don't assume)

The apply-side allowlist compares diff header paths against the RAW
`--files` entries, which still carry the `:Lstart-Lend` suffix.
`parse_scoped_files()` already separates bare paths from the scope map
(delegate_task.py ~line 573); the allowlist check must compare against the
parsed bare paths (or strip the suffix) instead of the raw strings.

## Acceptance criteria

- A scoped dispatch (`--files path:L80-L86`) whose diff targets `path`
  applies cleanly -- the REJECTED message no longer fires for the scoped
  target itself.
- Unscoped dispatches behave byte-identically to today (existing
  unit-test-style checks from TOKEN_STRATEGY Part 5 still hold).
- When apply yields zero hunks AND no --verify command is given, the exit
  code matches the vacuous contract (exit 3), not 0.
- No emoji; scripts/rules_check.py exit 0.

## Gate

python scripts/rules_check.py && python scripts/delegate_task.py --help

## Notes for the orchestrator

Evidence: tmp/ORCH_E2E_GITIGNORE_LINE_REFS_2026-08-04_response.md (worker
diff with correct headers, rejected). Related but separate router gap
(record in commit message, do not fix here): lake_route.py cooldowns are
per-model, so a lake-level dead API key (observed 401 on every qwen model,
2026-08-04) only cools one model at a time and the dial keeps returning
the dead lake.
