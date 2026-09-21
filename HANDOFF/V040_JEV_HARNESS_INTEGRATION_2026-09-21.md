# JEV / harness integration — SCMessenger completion (2026-09-21)

Status: Active
Owner: CTO/orchestrator
Harness: use **origin/main** or worktree `C:\Users\SCM\Documents\GitHub\Harness-jev-use`
  (tip `405bbc1` — JEV-P0/P1/P2 code present). Key: `~/.config/harness/jev.env`
  via `harness.config.resolve_jev_key()`.

## Local harness in SCMessenger (operator rule 2026-09-21)

**Do not edit the external Harness product tree** for SCMessenger work.

| Item | Path / command |
|---|---|
| Consumer copy | `vendor/sovereign-harness/` (gitignored) |
| Update from repo | `python scripts/update_local_harness.py` (clone/pull `Sovereign-Communication/harness` origin/main) |
| Path resolver | `scripts/local_harness.py` |
| Issue-sort pack (operator-frozen) | `scripts/scmessenger_issue_sort_pack.json` |
| Issue-sort API used | `harness.jev_policy.JevPolicy.evaluate_issue_sort` + `harness.jev_packs` on **vendor** origin/main |

Callers (`jev_canonical_check.py`, `jev_repo_insights.py`, `harness_gate.py`)
default to `vendor/sovereign-harness`. `HARNESS_REPO` env can override.

`harness_gate.py` outputs now default under `tmp/harness-runs/seat-gates/`
inside SCMessenger (not external Harness audits/).

## WIP session audit (concurrent Harness lane) — expanded

From Harness HANDOFF, busy session `ses_ffe5f3e6872afffe3U9yhXAFuo` task
journals T2/T6/T7, and origin/main docs (2026-09-21):

| Track | Truth |
|---|---|
| JEV-P0 / P1 | Complete on harness origin/main (PR #34/#35) |
| JEV-P2 | PR #36 repair; STATUS **in progress**; `JEV-P2-jury` deferred |
| JEV completion gate | PR #39 `harness jev-phase` — score ≥85 + hard gates; worktree `Harness-jev-completion` |
| Planned JEV-P5 | issue-sort / operator-declared buckets — **after** P2 merge |
| HUL TrackB | Mission packs `missions/<id>/`; **blocked** until P2 green; no product code yet |
| SCMessenger burndown in Harness | Sep 13 P0/P1/P3 harness bugs fixed (judge rotation, gpt-5 reasoning hints) |

### JEV-P5 design (from WIP T6 — do not implement in Harness now)

- IDs: `JEV-P5-buckets`, `issue-sort`, `envelope`, `orchestration`, `cli`
- **0-hallucination:** choice criteria only from **operator-declared** buckets
- Extend `jev_policy` + new `jev_packs.py` — not a second client
- Template: `triage_question_pack` + `evaluate_triage`
- Unmatched → `bucket=None`; unkeyed → `is_fallback=true`
- `session.jev_for` is orphan — do not extend
- Hermetic tests later: `tests/test_jev_issue_sort.py`

### SCMessenger alignment (this repo)

Our `scripts/jev_packs.py` uses **operator-declared** criteria maps only —
same 0-hallucination rule. Packs may later migrate to Harness `jev_packs.py`
after JEV-P5 lands; until then SCMessenger packs are local and explicit.

**Do not touch:** `Harness-jev-p2`, `Harness-jev-completion`, P1 worktrees,
Harness PR #36/#39 branches.

**SCMessenger rule:** consume `harness.jev.JevEvaluator` + local packs. WP
DONE = mechanical gates + `jev_canonical_check.py`. Harness `jev-phase` is a
Harness mission STATUS gate, not SCMessenger D1–D7.

**WIP consult:** message queued to busy session `ses_ffe5f3e6872afffe3U9yhXAFuo`;
reply will land async — pack design already matches their T6 constraints.

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
