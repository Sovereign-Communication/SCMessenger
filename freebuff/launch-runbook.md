# Launch Runbook — Freebuff Lanes (2026-10-21 ~12:50 HST)

## Tab order, backends, and models

Open mission tabs in this order. Paste the prompt file's full contents into each tab. **Backend rule:** free tier for high-volume mechanical work, GPT for judgment-heavy and long-horizon work. The harness provider router does free-tier arbitrage automatically — these assignments are the explicit policy on top of it.

| # | Tab | Backend | Model | Prompt file |
|---|-----|---------|-------|-------------|
| 0 | SCM Readiness Verifier (gate) | Freebuff tokens | 6.1 Sol (Luna if thorough) | `freebuff/readiness-verifier.md` — runs FIRST, alone. Pulls the handoff, audits the plan, verifies the environment, posts the readiness report to #602, requests Overseer clarification. Nothing else launches until it declares GO. |
| 1 | SCM Lane 0 — WIP Audit + Burn-Down | GPT | GPT-6 Luna | `freebuff/lane-0-wip-audit-burndown.md` — long phased mission, the foundation everything builds on. Don't cheap out here. |
| 2 | SCM Lane 1 — Merge Train | Freebuff tokens | GPT-6 Luna | `freebuff/lane-1-merge-train.md` — high-volume mechanical (rebases, CI checks, merges); rules are explicit, Overseer verifies every merge. |
| 3 | SCM Lane 2 — Fix-Forwards | GPT | GPT-6 Luna | `freebuff/lane-2-fix-forwards.md` — diagnostic work, judgment-heavy. |
| 4 | SCM Lane 3 — Transport | GPT | GPT-6 Luna | `freebuff/lane-3-transport.md` — hardest implementation, Rule-8 heavy. |
| 5 | SCM Lane 4 — Infra (Ubicloud) | Freebuff tokens | 6.1 Sol (Luna fallback) | `freebuff/lane-4-infra.md` — scoped ops checklist. |
| 6 | BigEnergyCo lane | GPT | GPT-6 Luna | `freebuff/bigenergyco-lane.md` (bigenergyco branch) |
| 7 | CivicScope lane | Freebuff tokens | 6.1 Sol (Luna fallback) | `freebuff/civicscope-lane.md` (civicscope branch) |
| — | Harness burndown | (already running — your existing tab) | — | `~/workspace/harness-burndown-lane-prompt-2026-10-20.md` |

**Reviewers:** separate thread, **6.1 Sol or Astra**, backend per availability — a different model from the implementer on purpose. Reviewer prompt: `freebuff/rule8-reviewer-prompt.md`.

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

Everything is on GitHub — no workspace access needed. On the laptop, in each repo clone:
`git fetch origin freebuff/handoff-2026-10-21 && git checkout freebuff/handoff-2026-10-21 -- freebuff/`
(or `git checkout freebuff/handoff-2026-10-21` to see the whole branch). SCMessenger's branch carries: this runbook, the unified rollout plan (§10 = smart launch + MCP routing), all 5 lane prompts, the Rule-8 reviewer prompt, the v1-corrected sovereign-harness skill, and `jev-gate-policy-v1.md`. BigEnergyCo's and civicscope's branches carry their lane prompt + the Jev policy.
