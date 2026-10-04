---
name: dreaming
description: "Run a dreaming lessons-learned distillation pass."
version: 1.0.0
author: Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [memory, lessons-learned, distillation, consolidation, reflection, dreaming]
    related_skills: []
---

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

# Dreaming — Distilled Lessons Learned from the Historical Record

A background-style reflection pass that reads the FULL Hermes session history
(the "elongated dream"), distills durable lessons, and writes them back to
memory + skills in compact form. The deliverable is a digest written to the
repo's `HANDOFF_AUDIT/` folder.

## Trigger conditions

Use this when the operator asks to:
- "dream" / "run a dreaming pass" / "distill my lessons learned"
- consolidate what we've learned across many sessions
- do a periodic lessons-learned review
- check what durable facts/skills Hermes should retain

Intended to eventually run unattended via cron (daily), but cron is OFF until
a manual run is proven robust. Manual runs are the primary path today.

## Workflow summary

1. **Gather** the historical record + current memory + current skills.
2. **Distill** into: new durable facts, lessons/pitfalls, consolidations,
   stale items to deprecate.
3. **Write back** atomically: memory batch `operations` + skill_manage.
4. **Report**: write a dated digest to `HANDOFF_AUDIT/` and summarize to the
   operator (who decides whether to keep each change).

## Step 1 — Gather

Run these to assemble the raw material. Do NOT skip; the dream is only as good
as its inputs.

```
# Browse recent session history (most recent first)
session_search()                                   # no args -> recent sessions
session_search(query="<lesson-relevant topics>", limit=5)

# Read current durable memory
read_file(path to MEMORY.md / USER.md)             # or use memory tool + current state
```

Use `session_search` with targeted queries for the domains the operator cares
about (e.g. "build test compile", "deployment gate", "security review"). Pull
the last several sessions' bookends (goal -> resolution) — those are where
conclusions live. Browse 5-10 sessions minimum for a good dream.

Also list current skills with `skills_list()` to know what already exists
(avoid duplicating and to spot consolidation candidates).

## Step 2 — Distill

Given all gathered material, produce FOUR lists:

1. **NEW_DURABLE_FACTS** — stable preferences, environment quirks, conventions.
   One-line each.
2. **LESSONS_AND_PITFALLS** — "when X, do Y" procedures. These belong in
   SKILLS, not memory.
3. **CONSOLIDATIONS** — overlapping memories/skills that should merge.
4. **STALE** — items no longer true that should be removed/replaced.

Rules:
- Prefer EDIT/consolidate over blind ADD. A memory store that grows without
  bound is a liability.
- Keep each memory entry compact and high-signal (the store is size-capped).
- Do NOT write work logs / task-progress / PR numbers to memory — those live
  in session_search.
- Lessons that are procedural belong in a skill, not memory.

## Step 3 — Write back

Apply the distillation with atomic, reversible operations:

```
# memory: use ONE batch operations call for all changes
target="memory" (or "user" for facts about who the user is)
operations=[ {action:add|replace|remove, content, old_text}, ... ]

# skills: create/patch only with explicit operator confirmation
skill_manage(action="patch", name=..., old_string=..., new_string=...)
```

NEVER delete a skill without operator confirmation — curation is
non-destructive (curator archives, never deletes). If a skill is genuinely
stale, propose archiving it to the operator rather than deleting.

## Step 4 — Report (the digest)

Write a dated digest to `HANDOFF_AUDIT/`:

```
HANDOFF_AUDIT/DREAM_<YYYY-MM-DD>.md
```

Containing:
- Date + model used for distillation
- What was gathered (how many sessions scanned)
- NEW_FACTS added (exact text)
- SKILLS patched/created
- CONSOLIDATIONS done
- STALE items removed
- Anything the operator should review/reject

Then deliver to the operator: a short summary in chat (or via the orchestrator
check-in / on demand, per operator preference) pointing at the digest path.
The operator decides whether each change sticks — always offer the option to
revert.

## Pitfalls

- **Don't trust per-turn exit codes piped through `tail`** — that's tail's
  exit, not the command's. Capture real exit with `; echo EXIT=$?` or grep a
  log for failure markers.
- **Windows workspace builds**: default-parallel `cargo test --workspace`
  OOMs the paging file (os error 1455). Use `-j 12` when nothing else heavy
  runs.
- **Memory is size-capped and injected every turn** — be ruthless about
  compactness and staleness. Batch ALL memory changes in one call.
- **Don't duplicate an existing skill** — run `skills_list()` first.
- **Cron sessions pass `skip_memory=True`**, so a cron-driven dream MUST do
  its memory writes via the memory tool (it does), not rely on injected memory
  state.

## Verification

- Digest file exists at `HANDOFF_AUDIT/DREAM_<date>.md` and is non-empty.
- Memory batch op reported success (char budget respected).
- No skill was deleted without operator OK.
- Operator reviewed the digest and either confirmed or reverted.

## Notes

Default distillation model: `deepseek/deepseek-v4-flash-0731` (cost-effective;
matches operator's FusionLite budget preference). Cron cadence target: DAILY,
but stays OFF until at least one manual dream run is proven robust end-to-end.
