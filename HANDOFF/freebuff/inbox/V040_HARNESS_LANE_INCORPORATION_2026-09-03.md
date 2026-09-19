# V040 Harness lane incorporation -- verified assessment and plan (2026-09-03)

Status: PROPOSAL, verified read-only. Source: /c/Users/SCM/Documents/GitHub/Harness
(separate git repo, branch main, MIT, pure-Python stdlib, zero runtime deps).

## What it is

Standalone evolution of FusionLite: a cost-bounded multi-model verification and
coding harness. Three faces: library, CLI, native MCP server (stdio). The
guarantees that matter for this project's review lanes:

1. Free tier by default (OpenRouter free models, rotating pool). Cost ceilings
   are pre-flight guarantees against LIVE per-token pricing (default 2c/call,
   hard max 10c); actual spend re-checked after every call, fail-closed
   mid-batch. Worst-case cost is computable before any network call.
2. No `tools` key in any payload, ever -- nothing can be force-invoked, so a
   review run cannot post to GitHub or touch the network beyond the chat call.
   Matches the standing no-posting decision.
3. Panel + structured judge verdicts with DETERMINISTIC convergence tally
   (5/5 unanimous == 100%, per-claim votes, confidence from the count, not the
   judge's self-report).
4. Self-grounding claims lint (P0, born from the SCMessenger 04b MAX_SKIP_KEYS
   lesson): every load-bearing claim must cite source_refs into a verbatim
   quoted window; absence/universal claims are rejected or auto-expanded
   against a definitions index BEFORE any model is called. Ungrounded claim =
   exit 2, zero spend.
5. Hash-chained, append-only autonomy ledger (~/.config/harness/ledger.jsonl)
   with tamper verification and per-model participation/calibration reports.
6. Capability layer (declared capability corrected by observed evidence) used
   for pool routing; BYOK-prefix leak handling learned per account; key must
   have a finite spend limit or the tool refuses to run.
7. Consent/sovereignty machinery with defer/redirect and continuation
   (`harness continue`) -- a partial task can be handed between models.

## Verification performed (this pass)

- Hermetic test suite: 140/140 OK on Python 3.14.6 in 0.483s, no network, no
  key (its own mid-batch cost check logged $0.000003 against the 2c ceiling).
- Live evidence already in production, independent of this pass:
  * 09-02 crypto audit of 9 functions (core/src/crypto, privacy, identity):
    `audits/scmessenger/audit_report.md` + `_runs/*.json` per function +
    `CTO_REVIEW.md` -- method `harness verify`, rotating 3-model free panel +
    judge, cost $0.00, read-only (hands-off confirmed in the report). Items
    marked `[confirmed by analyst]` were spot-checked against source.
  * Autonomy ledger has live entries incl. `bench/probe` today 10:12Z on
    `openrouter/free` -- the free pool is reachable from this machine.
- Key present: ~/.config/scmorc/openrouter_fusion.env (93 B) -- first-wins
  lookup order includes it. Not pip-installed; runs `python -m` from its
  checkout; `pip install -e .` installs `harness` + `harness-mcp`.

## Lane design: where it fits

Primary lane today: qwen free lane (CTO-driven, delegate_task273.py, single
continuous non-author model identity across R1 -> R2 -> FINAL -- the recorded
convention for #267/#272/#273). Harness is a SECOND free lane, panel-based, NOT
a replacement for the continuity lane. Fit by duty:

| Duty | Lane | Why |
|---|---|---|
| Rule-8 adversarial review, continuity-sensitive (in-flight rounds) | qwen | one recorded identity across rounds |
| Confirm/re-verify passes on FALSE-POSITIVE dispositions (#268/#270 shape) | harness | claims manifest against the diff window + deterministic convergence; independent of the original reviewer |
| Read-only audits / triage (crypto, transport surfaces) | harness | proven 09-02, $0, hands-off |
| Independent cross-check of a qwen APPROVE before a high-stakes merge | harness | second opinion at $0 |
| Known-answer gate checks (bench tasks) | harness | manifest-driven, verify-gate ground truth |

## Governance mapping for harness verdicts

- Verdict file convention: HANDOFF/review/V040_*_HARNESS_<topic>_2026-09-03.md,
  naming the panel participants (model slugs), the convergence tally per claim,
  the claims manifest + source window files used, and the autonomy-ledger chain
  head hash (ledger verify) so the artifact is independently checkable.
- Non-author property: the lane operator (CTO or this seat) assembles the
  claims manifest from the diff; panel models see only the quoted window and
  the claims -- no author context, same discipline as today's briefs.
- Not a substitute where "same non-author reviewer across rounds" is on file;
  if used for a Rule-8 gate, record the panel composition in the file.
- Cost: free tier only for lane use; ceilings are enforced by the tool; a paid
  escalation requires explicit CEO gate (allow_escalation stays false).

## First-live-use sequence (verify before trust)

1. Live smoke: `harness bench bench/tasks/add` (or the full 9-task manifest) on
   the free tier -- proves dispatch, panel, verify gate, ledger, $0. ~Minutes.
2. First real dispatch: convert one stalled confirm pass (#268 or #270,
   FLAG-1/FLAG-2 in V040_POST_VALIDATION_MERGE_PLAN_2026-09-03.md) into a
   claims manifest over its diff window and run `harness verify --converge`.
   Human double-check of the verdict against the disposition evidence before it
   is filed as Rule-8 evidence.
3. After two clean real runs, add the lane field to dispatch briefs
   (lane: qwen | harness) and reference this note.

## Integration points into the workflow

- Dispatch briefs: HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_*.md (add lane
  field and, for harness, the claims-manifest requirement).
- Evidence output: HANDOFF/review/ verdict files + audits/ dirs.
- Ledger of record for participation: ~/.config/harness/ledger.jsonl
  (outside the repo, hash-chained; chain head hash cited into verdict files).
- Tooling owner: this Harness git repo (outside SCMessenger). The user owns
  its development; SCMessenger consumes it by absolute path until a decision
  to vendor or submodule.

## Open items for the CEO seat

1. Authorize the live free-tier smoke (uses the OpenRouter key in
   ~/.config/scmorc/openrouter_fusion.env; free models, cost ceiling enforced).
2. Pick the first real harness dispatch: #268 confirm, #270 confirm, or the
   #272 multi-transport deferral re-review cross-check (currently stalled on
   the qwen lane).
3. Confirm the verdict-file convention and lane name (proposed: "harness free
   lane", verdict prefix V040_*_HARNESS_*).
