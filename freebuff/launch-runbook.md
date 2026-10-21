# Launch Runbook — Freebuff Lanes (2026-10-21 ~12:50 HST)

## Tab order and models

Open mission tabs in this order. Paste the prompt file's full contents into each tab.

| # | Tab | Model | Prompt file |
|---|-----|-------|-------------|
| 1 | SCM Lane 0 — WIP Audit + Burn-Down | GPT-6 Luna | `~/workspace/scmessenger-wip-audit-burndown-prompt-2026-10-21.md` |
| 2 | SCM Lane 1 — Merge Train | GPT-6 Luna | `~/workspace/scmessenger-lane-1-merge-train.md` |
| 3 | SCM Lane 2 — Fix-Forwards | GPT-6 Luna | `~/workspace/scmessenger-lane-2-fix-forwards.md` |
| 4 | SCM Lane 3 — Transport | GPT-6 Luna | `~/workspace/scmessenger-lane-3-transport.md` |
| 5 | SCM Lane 4 — Infra (Ubicloud) | 6.1 Sol (Luna fallback) | `~/workspace/scmessenger-lane-4-infra.md` |
| 6 | BigEnergyCo lane | GPT-6 Luna | `~/workspace/lane-prompts/bigenergyco-lane-2026-10-21.md` |
| 7 | CivicScope lane | 6.1 Sol (Luna fallback) | `~/workspace/lane-prompts/civicscope-lane-2026-10-21.md` |
| — | Harness burndown | (already running — your existing tab) | `~/workspace/harness-burndown-lane-prompt-2026-10-20.md` |

**Why this mapping:** Luna on every implementation/judgment lane (0–3, BigEnergyCo) — it's the proven implementer and these lanes make merge/ship decisions. Sol on Lane 4 and CivicScope — both are well-scoped, lower-stakes work (ops checklist; pre-build prep), and diversifying models is free robustness. If Sol ever stalls, fall back to Luna, no ceremony.

**Rule-8 reviewers:** separate thread, **6.1 Sol or Astra** — a different model from the implementer on purpose (different blind spots). Two ways to run it:
- (a) The reviewing lane spawns the reviewer as a subagent inline (fast; fresh context, different model).
- (b) You open a dedicated review tab and paste the reviewer prompt below + the PR number (cleanest independence; use for the big ones: #521, #599).
- Reviewer prompt file: `~/workspace/scmessenger-rule8-reviewer-prompt-2026-10-21.md`

## Sequence

1. **Lane 0 first, alone.** Paste its prompt, let it run Phase 0–1 (read-only sweep). Wait for its inventory on issue #602. Nothing else starts until the inventory is posted — it's the map.
2. **Lanes 1–4.** Paste prompts. Each registers its claimed worktree paths + branches as a comment on #602 (off-limits to Lane 0's purge from that moment).
3. **BigEnergyCo + CivicScope tabs.** Independent — launch anytime.
4. **Lane 0 continues** Phases 2–4 (backup → verified purge → burn-down) once you've eyeballed the inventory, or per its prompt's auto-continue — your call at launch.

## Only-you items (nobody else can do these)

1. **Post the #595 + #602 coordination comments** — drafts are ready, waiting on your word. (Say "post them" and I'll do it from here.)
2. **Ubicloud PAT** — Lane 4 will surface a Secure Vault card; approve it on your phone/laptop.
3. **#624 SSH access** — if Lane 3 reports it's blocked, that's your key/server to unlock.
4. **Trash-empty confirms** — Lane 0 stages, you confirm.
5. **The 3 Linux-box orphan commits** (`6d75b5b9`, `b686ebd5`, `876f5700`) — still only-local on my machine, unowned by any lane. Say "back them up" and I'll push them to `backup/` branches from here.
6. **#306 Fireworks key** (harness) — still unacknowledged; needs your machine.

## Files to have on the laptop

This runbook + the 7 prompt files + `~/workspace/jev-gate-policy-v1.md` (prompts carry an inline summary fallback if the file isn't there). Unified plan for reference: `~/workspace/scmessenger-rollout-2026-10-21.md`.
