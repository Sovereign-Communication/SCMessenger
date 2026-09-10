# Freebuff inbox -- write replies here

This is the return path. Anything the Freebuff lane needs from the orchestrator
seat goes in this folder as a new markdown file. A watcher notices new and
changed files here and wakes the orchestrator session.

Use it for:

- **A clarification question.** Order of tasks, an ambiguous acceptance
  criterion, two defensible readings of a scope line.
- **A premise that did not survive contact with the code.** If a task file says
  a function has zero callers and it has three, STOP and write that here rather
  than implementing a fix to a problem that does not exist. This is the single
  most valuable thing this folder receives.
- **A blocked report.** What you tried, the exact command and its output, and
  what you need unblocked.
- **A completion note.** The PR number, what changed, and any acceptance
  criterion you could NOT satisfy.

## Format

One file per message, named `<TASK-ID>_<what>_<YYYY-MM-DD>.md`, for example
`V040_T1_question_order_2026-08-31.md`. Start with:

```
Task: V040_T1_NODE_BOOT_SEED_DIAL.md
Type: QUESTION | BLOCKED | PREMISE-WRONG | DONE
```

Then the body. Carry evidence, not impressions: the exact command and its
output, or a run URL, or the word `UNVERIFIED`. A claim that a gate passed is
not the same as the gate's output.

Do not edit files in `../queue/` to ask a question -- write here instead, so the
task file stays a clean brief and the question is separately answerable.
