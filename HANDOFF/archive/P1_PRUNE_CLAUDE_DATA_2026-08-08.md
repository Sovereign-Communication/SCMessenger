# P1 -- Prune stale agent/session data (>10 GB) without losing work product

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Priority: P1
Created: 2026-08-08 (Windows orchestrator lane, documented at stand-down)
Assignee: UNASSIGNED -- next `/orchestrate` session should find this and DELEGATE it

**This ticket was deliberately written but NOT executed.** The originating
session was instructed to document it only. It doubles as a check that
`/orchestrate` discovers and delegates queued work without being told.

## Problem

Agent/session data on this machine exceeds 10 GB and keeps growing. It is
mostly transcripts, per-session scratch, stale git worktrees and conversation
databases from finished work. Disk on C: runs chronically tight -- it hit 98%
(6.5 GB free) today and blocked the build gates until ~26 GB was reclaimed from
`target/` (see `docs/fieldtest/TARGET_CLEANUP_2026-08-08.md`).

## Objective

Reclaim space from stale agent data **without destroying any work product.**
This is a careful, evidence-first task, not a sweep.

## MUST PRESERVE -- do not delete, do not "tidy"

1. **The Claude memory store**:
   `C:\Users\SCM\.claude\projects\C--Users-SCM-Documents-GitHub-SCMessenger\memory\`
   -- 48+ `.md` files plus `MEMORY.md`. This is hard-won operational knowledge
   that workers and future sessions depend on. Losing it is unrecoverable.
2. **The Antigravity conversation DB `0eae57f7-4fd1-47bb-9374-990f93590a8a.db`**
   under `C:\Users\SCM\.gemini\antigravity\conversations\`. This is the forensic
   record of the 2026-08-08 incident where a concurrent agent destroyed another
   session's work (see
   `~/.claude/.../memory/project_antigravity_destroyed_worktree_2026-08-08.md`).
   Keep it as an audit trail.
3. Anything under the repo itself: `HANDOFF/`, `docs/`, and `tmp/logs/` (field
   test captures still under analysis).
4. Any session transcript that a HANDOFF doc references by path.

## Candidate reclaim areas (verify sizes before acting -- these are leads, not facts)

| Area | Notes |
|---|---|
| `C:\Users\SCM\.gemini\antigravity\conversations\*.db` | One session DB measured 18.7 MB; there are many. Older ones are likely prunable EXCEPT the one named above. |
| `C:\Users\SCM\.gemini\antigravity\brain\*` | Many per-session dirs, some dating to mid-July. Check for work product before removing. |
| Stale **git worktrees** | `git worktree list` showed several from July that are almost certainly abandoned: two `subagent-*-Native-Engineer-self-*` under the antigravity brain dir, `AppData\Local\Temp\claude\gpt-pub9`, `gpt-pubA`, and `SCMessenger-w1`. NOTE: `.claude/worktrees/e01c-pq-mixing` is **locked** -- investigate before touching. Use `git worktree remove`, and `git worktree prune` for ones whose directory is already gone. Do NOT `rm -rf` a worktree directory without pruning git's metadata. |
| `AppData\Local\Temp\claude\<project>\<session>\` | Per-session scratchpads and `tasks/*.output` JSONL transcripts. These get large. Old sessions are prunable. |
| `C:\Users\SCM\.claude\projects\<project>\` | Session transcripts and `tool-results\` payloads. |
| `core/target/android-libs/` (1.9 GB) | Flagged during the 2026-08-08 `target/` cleanup as "role unconfirmed" and left alone. Determine whether it is regenerable before deciding. |

## Method (require this of whoever takes it)

1. **Measure first, delete second.** Produce a size-ranked inventory before
   removing anything. `du -sh <dir>/*` scoped per area -- a full recursive walk
   over 10 GB on Windows Git Bash is slow.
2. **Report the plan and get sign-off** before deleting anything outside
   obviously-regenerable build output.
3. Prefer moving to a staging directory over deleting, where space allows, so a
   mistake is reversible.
4. **Never `cargo clean`** (blocked by hook; wipes all of `target/`).
5. Recursive force-deletes outside `tmp/` and `target/` are blocked by
   `.claude/hooks/preflight_guard.py`. That block is CORRECT for this task --
   if you hit it, that is the signal to stop and get sign-off, not to override.
   Operator override exists (`SCM_ALLOW_DESTRUCTIVE=1`) but is an operator
   decision, not the worker's.
6. Verify no running process depends on anything removed. Use plain
   `tasklist | grep -i <name>` -- **`tasklist /FO CSV /NH` silently returns
   nothing under Git Bash** (it mangles `/FO` into a path), which will make you
   falsely conclude a process is dead.

## Deliverable

`docs/fieldtest/AGENT_DATA_PRUNE_<date>.md` with: inventory before, what was
removed and why it was safe, what was preserved and why, free space
before -> after, and anything deferred for an operator decision.

## Suggested delegation

Well-scoped, mechanical-with-judgment. A Sonnet worker is appropriate. Do NOT
give it blanket delete authority -- require the measure/report/sign-off loop
above.
