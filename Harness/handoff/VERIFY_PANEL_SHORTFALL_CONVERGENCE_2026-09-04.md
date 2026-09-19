# Harness verification-engine handoff -- panel-shortfall vs convergence bug (action items)

Date: 2026-09-04
From: CEO seat (SCMessenger lane driving `harness verify` for Rule-8 delta reviews)
Status: OPEN -- action items below, evidence-backed, reproduce in one command

## TL;DR

A single defect proposition observed in the verification engine this session:
when the panel falls SHORT of its required slot count (a model 429s or
truncates and rotation exhausts), the deterministic tally converts what is a
clean, unanimous verdict among all RESPONDING models into `agreement: low /
confidence: 0.0 / defer: true` and a convergence rate of 0.0. The engine
misattributes "not enough models answered" as "the panel disagrees / cannot
be trusted." Two secondary weaknesses compound it: the judge frequently
returns no content (raw reasoning trace leaks into the verdict fields) and
the free panel pool systematically truncates on ~40% of attempts.

## Evidence (same claims manifest + evidence window, two runs)

Run 1 (`w272-deferral-verdict.json`, task `w272-deferral-e97c3f82`):
- Panelists: gemma-4-31b-it stop OK, minimax-m3 stop OK, ling-3.0-flash-fin
  429 (rate-limited), nemotron finish_reason=length (malformed), north-mini
  finish_reason=length (malformed).
- Responding models: 2/2 voted `not_real` on all 4 claims (confidence 0.9x).
- Tally: `converged: False`, `convergence_rate: 0.0`, per-claim
  `unanimous: False`, `voted_by: 2 of_panel: 3`.
- Consensus: `{"agreement": "low", "confidence": 0.0, "defer": true}`.

Run 2 (`w272-deferral-verdict-r2.json`, task `w272-deferral-e97c3f82-r2`),
same claims + window, minutes later (ling's 429 cleared):
- Panelists: gemma stop OK, minimax stop OK, ling stop OK (3/3).
- Tally: `converged: True`, `convergence_rate: 1.0`, per-claim unanimous
  True, 3/3 `not_real` on all 4 claims.
- Consensus: `{"agreement": "high", "confidence": 1.0, "defer": false}`.

The substantive answer never changed (all responding models: no defect on
all claims). The 2/3 shortfall alone flipped the run to defer.

## Root cause

`harness/core.py` `tally_convergence`:

    unanimous = (required > 0 and total == required and
                 (votes["real"] == 0 or votes["not_real"] == 0))

Unanimity requires EVERY required slot to have voted (`total == required`).
`of_panel` stays at the target size when a model is malformed/unavailable and
rotation exhausts, so `missing_votes = required - total` silently downgrades
an otherwise unanimous verdict.

Contradiction in the module's own contract:
- `core.py` ~line 417 (module docstring): "a claim CONVERGES only when every
  panelist that ANSWERED it agrees on `real`".
- `core.py` ~line 435 (`tally_convergence` docstring): "A claim is converged
  only when every REQUIRED PANEL SLOT supplied a valid boolean vote".

Implementation follows the second; the module summary promises the first.

## Action items (todo)

1. **Distinguish shortfall from disagreement in the output.** Add a
   `panel_shortfall` / `missing_votes` signal to the tally and to the
   consensus object, and keep `agreement`/`defer` derived ONLY from models
   that actually voted. A 2/2 unanimous panel must not read
   `agreement: low / defer: true`; it should read `agreement: high`
   (responder-unanimous) with an explicit `panel_shortfall: 1 of 3`.
   Decide deliberately whether merge-gate consumers treat shortfall as
   defer (fail-closed, current behavior) or as approve (responder rule) --
   and say which in the verdict file either way. Fail-closed on SHORTFALL is
   defensible; reporting it as DISAGREEMENT is not.
2. **Reconcile the two docstrings** (module summary vs `tally_convergence`)
   so the contract matches the implementation after item 1 lands.
3. **Judge reliability / content leak.** The judge (cohere/north-mini-code:
   free, the configured FREE_JUDGE) returned no parseable content in 3 of 4
   runs this session (raw reasoning trace lands in `judge_synthesis` /
   `verdict`); in the one run where it did emit JSON the prose was
   semantically INVERTED ("the referenced commits are not true ancestors")
   while every panelist why said the opposite. When the judge fails, the
   deterministic tally already produces the consensus -- that fallback is
   good and must stay. Guard the prose: do not surface a judge `verdict`
   string that contradicts the per-claim tally, and label degraded synthesis
   in the verdict file.
4. **Panel pool efficiency.** north-mini-code and nemotron-3-super
   truncated (`finish_reason=length` -> malformed JSON) on EVERY attempt
   this session (3/3 runs) -- systematically wasting ~2 of 5 panel slots
   and forcing rotation. Either raise max_tokens for the panel prompt
   (windows are 50-65 lines + 4 claims), or drop/adjust the models that
   systematically truncate from FREE_PANEL_POOL. Add one backoff retry for
   HTTP 429 before burning the slot (ling's 429 cleared within minutes; a
   transient 429 cost the run a required slot).
5. **Minor:** severity-label variance is listed under consensus
   `disagreements` (e.g. "c1 severity differs: google critical, minimax
   info, inclusion low") even when verdicts are unanimous -- pollutes the
   disagreement signal used by gate consumers. Severity is not a verdict
   disagreement.

## Reproduction

    # from Harness repo root, using any claims manifest + window + free pool
    python -m harness.cli verify --claims-file <c>.json --source-file <w>.txt \
      --panel "google/gemma-4-31b-it:free,minimax/minimax-m3:free,inclusionai/ling-3.0-flash-fin:free,nvidia/nemotron-3-super-120b-a12b:free,cohere/north-mini-code:free" \
      --judge cohere/north-mini-code:free --converge \
      --convergence-model cohere/north-mini-code:free --max-cost 2 \
      --task-id repro-shortfall --out /tmp/repro.json
    # observe the tally when <3 panelists parse; rerun and compare when 3 parse.

## Session artifacts (SCMessenger side, for the record)

- tmp/harness/w272-deferral-window.txt (62 lines), w272-deferral-claims.json
- tmp/harness/w272-deferral-verdict.json (r1: 2/3 shortfall -> defer)
- tmp/harness/w272-deferral-verdict-r2.json (r2: 3/3 -> approve)
- tmp/harness/w272-finaldelta-verdict.json (3/3 approve; judge prose inverted)
