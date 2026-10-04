# TASK: Add --mode diff to scripts/delegate_task.py

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Full-file round-trips waste most of the token budget when the change is
small. Add an opt-in unified-diff mode to `scripts/delegate_task.py`.

## New CLI argument

`--mode {full,diff}`, default `full`. `full` keeps current behavior exactly.

## Behavior when --mode diff

1. PROMPT CHANGE: instead of asking for FULL file contents, the prompt
   instructs: "Return your changes as unified diffs, one fenced ```diff
   block per file, using standard `--- a/<path>` and `+++ b/<path>` headers
   with 3 lines of context. Do NOT return full files. For a NEW file, use
   `--- /dev/null` and `+++ b/<path>`."
2. NEW PARSER: add `extract_diff_blocks(content)` that finds every fenced
   ```diff block and returns the raw diff text of each. Do not modify
   `extract_file_blocks`.
3. APPLICATION: concatenate the diff blocks, write to
   `tmp/{taskname}_patch_round{n}.diff`, then run
   `git apply --whitespace=nowarn <patchfile>` via subprocess. Exit code 0
   means applied; print the list of files touched (parse the `+++ b/...`
   lines).
4. FALLBACK ON APPLY FAILURE: if `git apply` fails, print
   `[WARN] diff apply failed; falling back to full-file mode for this task`
   and re-dispatch the SAME round with the full-file prompt and the current
   file contents (the existing code path). From then on the task continues
   in full mode (including any --verify fix rounds).
5. INTERACTION WITH --verify: fix rounds use the current effective mode
   (diff until a fallback happened, then full). The follow-up prompt in
   diff mode includes the verify errors and the CURRENT file contents
   (models cannot write correct diffs against files they cannot see), and
   asks for a corrective unified diff.

## Constraints

- All existing behavior byte-for-byte identical when `--mode` is omitted.
- Standard library + `git` CLI only. No new Python dependencies.
- No emoji; plain-text tags [OK]/[WARN]/[FAIL]/[ROUND n] only.
- Keep QWEN_TIER_MAP, VALID_EXTENSIONS, key loading, and the verify loop
  from the previous task intact.

## Output format (MANDATORY)

Return the FULL updated contents of exactly one file in a single fenced
code block with `// scripts/delegate_task.py` as the first line inside the
block. (Yes, full file for THIS task -- the tool you are editing is the one
that gains diff support.)
