# SCMessenger Freebuff Readiness Verifier (paste-ready mission prompt)

**Repo:** Sovereign-Communication/SCMessenger. **Scope: SCMessenger ONLY.**

## Launch config (read before pasting)

- **Backend:** Freebuff tokens (free tier). This is a bounded verification task — checklists and environment probes, not long-horizon implementation. Do not burn GPT budget here.
- **Model:** 6.1 Sol (GPT-6 Luna if you want max thoroughness on the plan audit).
- If this tab is running on a different backend/model than above, note it in the report — the Overseer needs the true inventory.

## Mission

Before any work lane spins up: pull the handoff branch, audit the plan for issues, verify this machine is actually ready to execute, report everything to the tracking issue, get Overseer clarification on anything unclear, and only then clear the lanes for launch. You are a gate, not a worker — you verify readiness; you do not start the audit, merge, or fix anything.

## Step 1 — Pull the handoff

1. `git fetch origin freebuff/handoff-2026-10-21`. If the fetch fails, STOP and report it — nothing downstream can run.
2. Check out the `freebuff/` directory contents (do not switch branches on any working clone; use `git checkout freebuff/handoff-2026-10-21 -- freebuff/` into a scratch location or `git show` per file).
3. Verify every expected file is present and non-empty:
   - `freebuff/rollout-plan.md` (§10 smart-launch + MCP routing must be present — it's the freshness marker)
   - `freebuff/lane-0-wip-audit-burndown.md`, `lane-1-merge-train.md`, `lane-2-fix-forwards.md`, `lane-3-transport.md`, `lane-4-infra.md`
   - `freebuff/rule8-reviewer-prompt.md`, `freebuff/launch-runbook.md`
   - `freebuff/jev-gate-policy-v1.md`, `freebuff/sovereign-harness-SKILL.md`
   - Missing or empty file → report as a blocker, do not proceed past Step 3 for that item.

## Step 2 — Audit the plan

Read `freebuff/rollout-plan.md` end to end and check for issues:

1. **Scope consistency:** Lane 4 = Ubicloud-only; Lane 0 = backup/disk (not both). Sole merger = Lane 1. No lane writes another lane's files.
2. **No stale machine assumptions:** flag any `~/workspace/...` or Linux-only paths that would break on this machine (prompts carry inline fallbacks — verify they're actually there).
3. **Protocol presence:** §9 Rule-8 review protocol and §10 smart-launch/MCP routing exist and match what the lane prompts say.
4. **Contradictions:** any two files disagreeing on who owns what, merge authority, or done-criteria.
5. Post every issue found — even minor ones — in the readiness report. Do not silently fix plan files; the Overseer owns the plan.

## Step 3 — Verify the environment

1. **git + auth:** `git --version`; `gh api user --jq .login` (or the machine's existing auth). No auth → STOP, report exactly what's needed.
2. **Harness MCP:** verify the MCP server is reachable — list configured MCP servers and make ONE cheap Jev call (a Noul, e.g. "is this MCP channel working?"). If unreachable, report it as a degraded-mode flag (lanes will burn their own tokens for cheap decisions — note the cost impact, don't block on it).
3. **Jev/TypeSafe access:** confirm the lane can reach the TypeSafe API (the key lives on this lane per the skill — reference it, don't print it).
4. **Disk:** report free space on the drive holding the repos (this machine was critically low — the number matters).
5. **Worktrees:** `git worktree list` works in a test clone; no errors.
6. **Backends/models available:** list every backend and model this machine can launch lanes on (GPT, Freebuff tokens/free-tier, etc.) — the Overseer needs the true inventory to finalize lane assignments. Report the full list, not just what this tab uses.

## Step 4 — Report and request clarification

Post a readiness report as a comment on **issue #602** with:
- Handoff branch: fetched OK / failed (with error).
- File inventory: present / missing.
- Plan audit: issues found (numbered) or "no issues found".
- Environment: git/auth/MCP/Jev/disk/worktrees — each PASS / FAIL / DEGRADED with details.
- Verdict: **GO** (all green), **GO-WITH-CONDITIONS** (list them), or **BLOCKED** (list blockers).
- Explicit clarification requests to the Overseer for anything ambiguous — quote the exact plan text you're unsure about. Tag them so the Overseer can answer point by point.

## Step 5 — Retask on Overseer reply

When the Overseer answers (via #602 comments): apply the clarifications, update the verdict, and only then declare the environment **verified, tested, and ready**. If the Overseer retasks anything (e.g., degraded MCP → adjusted cost expectations), record the retasking in a follow-up #602 comment. Do not launch work lanes yourself — readiness is your whole job; Lucas launches the lanes.

## Standing rules

Jev gate v1 applies to your own assessment of readiness. Never ask the operator technical questions — ramifications and end-state only. Never modify plan or prompt files. Never merge, push, or delete anything.
