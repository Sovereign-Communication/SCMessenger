# MODEL: qwen3-coder:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 600
# token_budget: 6000

# META_ORCHESTRATOR_ROLE_PROTOCOL_v1

**Status:** VERIFIED REMAINING WORK
**Agent:** doc-author (this is a doc-only task; no code, no build)
**Budget:** 600s (LIGHT tier)
**Phase:** Orchestration hardening
**Source:** 2026-06-05 audit  Claude session re-derived Overseer role 5+ times in one conversation, burning turns on corrections
**Depends on:** none
**Blocks:** all future Claude Code sessions on this workspace

---

## Verified Gap

Claude Code does not currently install a self-contained role anchor at session start. It re-derives the Overseer role from scratch every session, which Lucas has had to correct 5+ times in the 2026-06-05 audit. The corrections are tactical, not philosophical  they fix specific bad behaviors (committing when not asked, recommending Claude/Anthropic models, spawning ad-hoc subagents, running low-signal diagnostics, asking for clarification on "orchestrate").

**Verified failure pattern (audit 2026-06-05):**

| # | Lucas's correction (excerpt) | What Claude did wrong |
|---|---|---|
| 1 | "no need to audit disk space, ensure full context" | Burned a turn on `df -h` / disk-space check instead of loading the orchestration index |
| 2 | "leverage Local optimized LLM's" | Defaulted to cloud subagents when local 7B/14B would have sufficed |
| 3 | "wait - stop - we do not have claude" | Recommended Claude/Anthropic models as the answer inside an Anthropic-hostile orchestration pipeline |
| 4 | "do not recommend any claude/anthropic models" | Repeated the same recommendation a second time after correction #3 |
| 5 | "no do not let it commit - WE have that as the gate" | Subagent ran `git commit` autonomously; commit is the Overseer's gate |
| 6 | "Shouldn't you delegate this? Write perfectly scoped handoff tasks, and run the orchestrate skill?" | Tried to author code directly instead of writing a handoff ticket; then asked what "orchestrate" meant when no such skill exists |

**Verified environment facts:**
- No `orchestrate` skill exists in `/home/scmessenger/.hermes/skills/` (only `apple`, `devops`, `github`, `mcp`, `mlops`, `ollama-slot-status`, `red-teaming`, `email`, `creative`, `data-science`, `domain`, `gaming`, `gifs`, `inference-sh`, `media`, `note-taking`, `productivity`, `diagramming`, `dogfood`, `autonomous-ai-agents`, `.archive`, etc.). The "orchestration" workflow is HANDOFF + swarm.py, not a skill.
- `HANDOFF/CLAUDE_CODE_README.md` (45 lines) currently points only to the orchestration index; it has no role-protocol section.
- `HANDOFF/STATE/2026-06-05_ORCHESTRATION_INDEX.md` (241 lines) starts with TL;DR; it has no "Role & Protocol" anchor.
- `/home/scmessenger/.claude/CLAUDE.md` does not exist; there is no user-level Claude import that would pull in the workspace protocol.

## Scope

Create / edit exactly these files:

- **NEW**: `HANDOFF/CLAUDE_CODE_PROTOCOL.md`  the actual session-startup protocol block. Self-contained, < 200 lines, 6 mandatory sections in order: `## Your role`, `## What you do NOT do`, `## OODA discipline`, `## Build commands`, `## Current state`, `## Anti-patterns observed this session`. Each section uses imperatives, not suggestions.
- **EDIT**: `HANDOFF/CLAUDE_CODE_README.md`  insert a new top section titled `## Role & Protocol` between the existing intro paragraph and the ` STATE/2026-06-05_ORCHESTRATION_INDEX.md` pointer. 3-4 lines. Content: "**Read `CLAUDE_CODE_PROTOCOL.md` after the orchestration index.** It defines your role, what you may/may not do, build commands, and the OODA discipline. This is the Overseer role anchor  do not skip."
- **EDIT**: `HANDOFF/STATE/2026-06-05_ORCHESTRATION_INDEX.md`  add a 3-line `## Role & Protocol` header ABOVE the existing `## TL;DR` section. Content: "All Claude Code sessions on this workspace must read `HANDOFF/CLAUDE_CODE_PROTOCOL.md` first. It is the Overseer role anchor. The orchestration workflow is `HANDOFF/todo/`  swarm dispatch  `HANDOFF/done/` via `git mv`. No new frameworks."
- **EDIT**: `~/.claude/CLAUDE.md` (user-level, WSL path `/home/scmessenger/.claude/CLAUDE.md`)  append a new section at the end titled `## Overseer role import` with one line: "When working in `/mnt/e/SCMessenger-Github-Repo/SCMessenger/`, read `HANDOFF/CLAUDE_CODE_PROTOCOL.md` first." If the file does not exist, create it with just that line.

## File Targets

- `HANDOFF/CLAUDE_CODE_PROTOCOL.md` [NEW]  protocol block, < 200 lines, 6 sections
- `HANDOFF/CLAUDE_CODE_README.md` [EDIT]  add `## Role & Protocol` section
- `HANDOFF/STATE/2026-06-05_ORCHESTRATION_INDEX.md` [EDIT]  add `## Role & Protocol` header above TL;DR
- `/home/scmessenger/.claude/CLAUDE.md` [EDIT or CREATE]  append Overseer import

## Build Verification Commands

This is a doc-only task. No `cargo` or `gradle` invocations. Verification is by file inspection:

```bash
cat /mnt/e/SCMessenger-Github-Repo/SCMessenger/HANDOFF/CLAUDE_CODE_PROTOCOL.md
cat /mnt/e/SCMessenger-Github-Repo/SCMessenger/HANDOFF/CLAUDE_CODE_README.md
cat /home/scmessenger/.claude/CLAUDE.md
cd /mnt/e/SCMessenger-Github-Repo/SCMessenger && git status
cd /mnt/e/SCMessenger-Github-Repo/SCMessenger && git diff --stat
```

Expected: protocol file visible, README has new section, `~/.claude/CLAUDE.md` has the import line, `git status` shows 4 files (1 new + 3 modified), `git diff --stat` shows 3 modified + 1 new.

## Acceptance Gates

- [ ] `HANDOFF/CLAUDE_CODE_PROTOCOL.md` exists, is < 200 lines, and contains all 6 mandatory sections (`## Your role`, `## What you do NOT do`, `## OODA discipline`, `## Build commands`, `## Current state`, `## Anti-patterns observed this session`) in that order.
- [ ] Each mandatory section uses imperatives (no "you might want to", no "consider", no "it is recommended"); uses "Do not" / "You MAY" / "You MUST" phrasing.
- [ ] `HANDOFF/CLAUDE_CODE_README.md` has a new `## Role & Protocol` section inserted ABOVE the ` STATE/2026-06-05_ORCHESTRATION_INDEX.md` pointer, pointing to `CLAUDE_CODE_PROTOCOL.md` as the first thing to read AFTER the orchestration index.
- [ ] `HANDOFF/STATE/2026-06-05_ORCHESTRATION_INDEX.md` has a `## Role & Protocol` header inserted ABOVE the existing `## TL;DR` section, with the 3-line reminder that the workflow is HANDOFF + swarm.py and no new frameworks.
- [ ] `/home/scmessenger/.claude/CLAUDE.md` has the `## Overseer role import` section (or the file was newly created with it), and any pre-existing content was preserved on append.
- [ ] `cd /mnt/e/SCMessenger-Github-Repo/SCMessenger && git add -A && git commit -m "meta: install Overseer role protocol block  prevents session re-derivation"` succeeds. Do NOT push.
- [ ] A post-mortem is written to `HANDOFF/STATE/2026-06-05_ORCHESTRATOR_ROLE_PROTOCOL_INSTALLED.md` (use `git add` + commit; do NOT include the post-mortem in this ticket's `git mv`).

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` command to move this ticket markdown file from `HANDOFF/todo/` to `HANDOFF/done/`. If you do not move the file, the Orchestrator assumes you failed.

[REQUIRES: QWEN3-CODER] [PHASE: META] [BLOCKS: ALL_FUTURE_CLAUDE_SESSIONS]
