# TASK: Add --verify auto-fix loop to scripts/delegate_task.py

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Add an automatic verify-and-fix loop to `scripts/delegate_task.py` so that a
single dispatch can apply code, run a local verification command, and if it
fails, automatically re-prompt the model with the errors -- up to N rounds --
with ZERO orchestrator involvement between rounds.

## New CLI arguments

1. `--verify "<shell command>"` (optional string). Example:
   `--verify "cargo check -p scmessenger-core"`. Only meaningful with
   `--apply`; if given without `--apply`, print a warning and ignore it.
2. `--max-rounds N` (optional int, default 3). Total model calls allowed
   including the first one.

## Behavior (only when --apply and --verify are both set)

After applying file blocks (existing logic), enter a loop:

1. Run the verify command with `subprocess.run(cmd, shell=True,
   capture_output=True, text=True, env=verify_env)` where `verify_env` is
   `os.environ` copied with `CARGO_INCREMENTAL` set to `"0"` (Windows rlib
   safety rule).
2. If exit code 0: print `[OK] verify passed on round {n}` and exit 0.
3. If nonzero and rounds remain: build a follow-up prompt containing:
   - A header: "Your previous attempt was applied but verification failed."
   - The verify command and the LAST 6000 characters of combined
     stdout + stderr.
   - The CURRENT (post-apply) contents of every file in `--files`, re-read
     fresh from disk, in the same fenced format used for the initial prompt.
   - The same output-format instruction as the initial prompt (FULL file
     contents, filename as first line inside the code block, no partial
     files, no diffs).
   Send it to the SAME provider/model as the original dispatch, using the
   same request code path. Save each round's raw response to
   `tmp/{taskname}_response_round{n}.md`. Apply the returned blocks
   (existing `extract_file_blocks` + apply logic). Print
   `[ROUND {n}] verify failed (exit {code}); re-dispatching fix...` before
   each re-dispatch.
4. If the response contains no applicable file blocks, count it as a failed
   round and re-dispatch with an added line: "REMINDER: you MUST return full
   file contents in fenced code blocks with the filename as the first line."
5. After `--max-rounds` total model calls with verify still failing: print
   `[FAIL] verify still failing after {N} rounds` plus the last 2000 chars
   of verify output, and `sys.exit(2)`.

## Refactoring constraints

- Extract the existing request/response code (payload build, urllib request,
  response parse, save-to-tmp, apply) into reusable function(s) so the loop
  calls the same code as the first dispatch. Do NOT duplicate the HTTP code.
- ALL existing behavior must remain byte-for-byte identical when --verify is
  not passed: same arguments, same prompts, same output messages, same file
  handling, same key loading (env first, then ~/.config/scmorc/*.env files).
- Standard library only (urllib, subprocess, argparse, os, re, sys, json).
  No new dependencies.
- No emoji anywhere. Plain-text tags like [OK] / [FAIL] / [ROUND n] only.
- Keep the QWEN_TIER_MAP, VALID_EXTENSIONS, and parser exactly as they are.

## Output format (MANDATORY)

Return the FULL updated contents of exactly one file in a single fenced code
block, with `// scripts/delegate_task.py` as the first line inside the block.
No partial files, no diffs, no commentary inside the block.
