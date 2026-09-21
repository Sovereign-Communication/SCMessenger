# JEV / harness integration — SCMessenger completion (2026-09-21)

Status: Active
Owner: CTO/orchestrator
Harness: use **origin/main** or worktree `C:\Users\SCM\Documents\GitHub\Harness-jev-use`
  (tip `405bbc1` — JEV-P0/P1/P2 code present). Key: `~/.config/harness/jev.env`
  via `harness.config.resolve_jev_key()`.

## WIP session audit (concurrent Harness lane)

From Harness HANDOFF + busy session `ses_ffe5f3e6872afffe3U9yhXAFuo`:

| Track | Truth |
|---|---|
| JEV-P0 / P1 | Complete on harness origin/main |
| JEV-P2 | PR #36 repair; STATUS in progress; jury deferred |
| JEV completion gate | PR #39 `harness jev-phase` (worktree Harness-jev-completion) — dogfood score ≥85 + hard gates |
| Planned JEV-P5 | issue-sort / operator-declared buckets / `jev_packs.py` — **after** P2 |
| HUL TrackB | After P2+completion merge |

**SCMessenger rule:** do not edit Harness P2/P3/P39 worktrees. Consume
`harness.jev.JevEvaluator` + local packs. Do not invent parallel Harness plans.

## What we want from JEV (full picture)

Code owns mechanics; JEV owns bounded semantic judgment. Batched calls keep
cost low (input tokens only; $42/Mtok).

| Use case | Pack | Question types | When |
|---|---|---|---|
| Pain points / historical process | `pain_points`, `historical_process` | choice + score + noul | After merge trains; audit waves |
| Unification gaps | `unification` | noul + choice | Before WP/implementation pastes |
| Orchestration / dispatch health | `orchestration` | noul + choice + score | Before freebuff paste waves |
| WP canonical completion | `canonical_completion` | 3 nouls | Before marking any WP DONE |
| Plan complexity route | harness `triage_question_pack` | choice + noul | Optional at task-file authoring |
| Diff semantic match | harness `diff_question_pack` | noul | PR review assist |

## Batching design

- Harvest compact signals locally (git, PR list, HANDOFF todo/queue, plan keywords).
- Group **N items per evaluate() call** (default batch_size=4).
- One `state` JSON: `{domain, items:[{id, signal}]}`.
- Multiple typed questions per call (pack map) — one network call answers all.
- Report aggregates tokens/cost per pack and batch.
- Unkeyed → `is_fallback=true`; report marks FALLBACK; not canonical DONE.

## Tools (SCMessenger)

| Tool | Role |
|---|---|
| `scripts/jev_packs.py` | SCMessenger insight packs (operator-declared) |
| `scripts/jev_repo_insights.py` | Harvest + batch + JEV evaluate + markdown report |
| `scripts/jev_canonical_check.py` | WP completion JEV gate (single pack) |

```text
$env:HARNESS_REPO = "C:\Users\SCM\Documents\GitHub\Harness-jev-use"
python scripts/jev_repo_insights.py --mode full --batch-size 4
python scripts/jev_canonical_check.py --wp WP2 --state-file tmp/wp2_state.json
```

## Relation to 0.4.0 completion

1. Run `jev_repo_insights.py` after major merges / before paste waves.
2. Paste only DISPATCHABLE tickets from freebuff README + implementation plan.
3. WP DONE = mechanical gates + `jev_canonical_check.py` `is_passing`.
4. WP5 live 3-node logs still mandatory for WiFi-fixed claims.
5. Tag path remains master plan checklist + operator decisions.

## Future alignment (do not block SCMessenger)

- When Harness lands `jev_packs.py` / issue-sort (JEV-P5), SCMessenger packs
  can migrate to import those buckets — until then local packs are the
  operator-declared set for this repo.
- `harness jev-phase` is a **Harness** mission STATUS gate, not SCMessenger D1–D7.
