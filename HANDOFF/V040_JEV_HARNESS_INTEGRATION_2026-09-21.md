# JEV / harness integration — SCMessenger completion (2026-09-21)

Status: Active
Last updated: 2026-09-23 (immutable Harness admission and bounded-canary policy)
Owner: CTO/orchestrator

Production Harness source: immutable release tag `v0.4.1`, peeled commit
`ad4a30052955e1f574c90f426edfc7a274e16ebf`. The moving Harness `origin/main`
is a bounded canary source only and must be re-resolved for every run. An
external Harness checkout is diagnostic context, never production evidence.

SCMessenger implementation baseline: `origin/main`
`56d66f7190d14fdb78eebfbc005b8fbc6d507d6c`; the inspected CI workflows for
that exact SHA were green. This document is the operational runbook for
admitting Harness into SCMessenger; it does not authorize a release by itself.

JEV key: `~/.config/harness/jev.env` via
`harness.config.resolve_jev_key()`. No key is required for hermetic admission
probes.

## OpenRouter Jev fallback (operator 2026-09-21)

TypeSafe account returned **Internal Server Error**. Fallback:

| Item | Value |
|---|---|
| Model | `~typesafe/jev-latest` (OpenRouter **decisions** model) |
| Endpoint | `https://openrouter.ai/api/alpha/decisions` — **not** `chat/completions` |
| Key | OpenRouter via harness `resolve_api_key()` / `OPENROUTER_API_KEY` |
| Consumer code | `scripts/local_harness.py` `evaluate_jev_with_openrouter_fallback` |
| Env override | `SCM_JEV_OPENROUTER_MODEL` |

**Order:** TypeSafe primary → OpenRouter decisions fallback when TypeSafe is
unhealthy (ISE / no answers / transport fail). Canonical DONE still requires
a **non-fallback** typed pass (TypeSafe or OpenRouter parse that yields
official answer shapes). Structural-only fallback remains `UNVERIFIED-JEV`.

**Operator OpenRouter account requirement:** allow provider **`typesafe`**
for model `~typesafe/jev-latest`. Probe 2026-09-21 returned:
`No allowed providers are available ... Providers serving typesafe/jev-...:
typesafe, but your account's allowed-providers setting permits only: ...`
until that provider is enabled. Keys already present locally; no secret
committed.

## Local harness in SCMessenger (operator rule 2026-09-21)

**Do not edit the external Harness product tree** for SCMessenger work.

| Item | Path / command |
|---|---|
| Consumer copy | `vendor/sovereign-harness/` (gitignored) |
| Update from repo | `python scripts/update_local_harness.py --mode admit-tag`; production accepts the immutable `v0.4.1` tag only. `canary-main` is isolated and never release evidence. |
| Path resolver | `scripts/harness_source.py` (the only source/import boundary) |
| JEV adapter | `scripts/local_harness.py` (TypeSafe/OpenRouter behavior only) |
| Admission state | `scripts/harness_admission.json` |
| Admission lifecycle | `scripts/harness_admission.py` + `scripts/update_local_harness.py` |
| Issue-sort pack (operator-frozen) | `scripts/scmessenger_issue_sort_pack.json` |
| Issue-sort API used | `harness.jev_policy.JevPolicy.evaluate_issue_sort` + `harness.jev_packs` on the admitted source |

Callers (`jev_canonical_check.py`, `jev_repo_insights.py`, `harness_gate.py`,
and `bod_governance.py`) all use `harness_source.py` and therefore resolve the
same admitted source. Direct installed-package fallback is forbidden for
release evidence. Consumers have no source override; canary admission is a
separate lifecycle mode.

## Verified admission baseline (2026-09-23)

| Source | Verified state | Admission role |
|---|---|---|
| Harness `v0.4.1` | immutable release; peeled commit `ad4a3005…` | production source of truth |
| Harness `origin/main` | moving; re-resolve one SHA per run | one-SHA canary only |
| External Harness checkout | not a consumer source | diagnostic only |
| SCMessenger `origin/main` | `56d66f71…`; inspected CI green | implementation baseline for this consumer |

### Detection gate

Before any update, record all of the following:

```text
git ls-remote https://github.com/Sovereign-Communication/harness.git 'refs/tags/v0.4.1^{}'
git ls-remote https://github.com/Sovereign-Communication/harness.git refs/heads/main
```

A release candidate must resolve the expected tag to the expected peeled
commit. A branch result without an exact SHA is not an admitted source. A
local dirty checkout is not a candidate. A missing tag, missing commit,
unexpected remote, unreadable version, or dirty source is `[BLOCKED]`.

### Admission and compatibility gate

Stage candidates under `tmp/harness-admission/<tag>-<sha>`; never use a
system temp directory. The staged checkout must be clean and the local
admission probe must pass:

1. package/version check from `pyproject.toml` before importing package code;
2. import probe for every SCMessenger-used symbol, including
   `JevEvaluator`, `JevPolicy`, and the private `_validate_questions` /
   `_parse_answer` functions;
3. CLI probe for `verify`, `ledger verify`, `spend`, `trust`, and
   `lint-claims`;
4. report-schema probe for the fields consumed by `harness_gate.py`;
5. no-network/no-key smoke for import, version, and fallback classification.

Release evidence additionally requires green upstream CI for the exact Harness
SHA and green SCMessenger consumer CI for the update PR.

Any missing symbol, schema mismatch, version mismatch, dirty source, or
unexpected fallback is a refusal. Structural JEV fallback is never a
canonical completion result.

### Admission lifecycle semantics

The updater implements three modes. Production promotion is accepted only
after the exact tag and peeled commit match the pinned manifest baseline:

- `admit-tag`: resolve and verify `v0.4.1`, run admission, then promote the
  candidate;
- `canary-main`: resolve one exact `origin/main` SHA, run the same probe in a
  separate staging path, return a `CANARY` result, then remove the candidate
  without changing the production manifest;
- `rollback`: restore the retained previous source and update the manifest.

The lifecycle and no-key probe are covered by
`scripts/test_harness_admission.py`. Upstream and SCMessenger CI remain release
gates; this local implementation does not record CI evidence.
Promotion is fail-closed and recoverable for handled filesystem or manifest
errors. The candidate is fully validated before the active path changes; a
manifest failure returns the candidate to staging and restores the previous
active source. A failed rollback restores both the current active source and
the retained previous source at the path recorded in the unchanged manifest.
The updater must not use `checkout -B` on a dirty vendor copy, and it must
never edit an external Harness worktree.

### Ownership boundaries

- Harness maintainer: upstream tags, API/CLI compatibility, Harness CI, and
  release publication.
- SCMessenger consumer owner: `harness_source.py`, `harness_admission.py`,
  the admission manifest, `local_harness.py`, the updater, wrappers, tests, and
  this runbook.
- SCMessenger orchestrator: 0.4.0 freeze, PR ordering, CI, and merge.
- Platform owners: Android, Windows CLI, cloud node, and native behavior.
- Security reviewer: independent review for gated core changes.

No owner may silently replace the production tag with a moving branch.

### Overlapping PR disposition

| PR | Disposition | Reason |
|---|---|---|
| #360 `freebuff/harness-version-floor` | Superseded | Its intent is implemented on the current baseline; do not merge the old implementation. |
| #362 `claude/harness-lane-state-2026-09-22` | Separate | Append-only lane evidence; review independently. |
| #347 / #344 and later merged consumer work | Baseline | This branch adds the source and admission lifecycle. |

## Historical WIP session audit (superseded; retained for provenance)

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

**Historical note:** The session status below is a 2026-09-21 snapshot and is
not the current admission authority. The verified admission baseline and gates
above supersede it.

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
python scripts/update_local_harness.py --mode admit-tag
python scripts/jev_repo_insights.py --mode full --batch-size 4
python scripts/jev_canonical_check.py --wp WP2 --state-file tmp/wp2_state.json
```

Canary validation is explicit:
`python scripts/update_local_harness.py --mode canary-main`; it never changes
production.

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
