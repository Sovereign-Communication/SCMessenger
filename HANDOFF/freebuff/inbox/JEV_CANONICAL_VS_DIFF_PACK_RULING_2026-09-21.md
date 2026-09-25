# Ruling request: which JEV instrument is the WP DONE gate

Status: OPEN -- needs orchestrator confirmation (not an implementation task)
Filed: 2026-09-21 by the Freebuff lane
Evidence: `HANDOFF/freebuff/jev/` (state files + verdicts), PR #349, PR #352

## What happened

Two JEV instruments were both treated as "the JEV row" during the train, and
they disagree by a wide margin on the same work:

| Instrument | What it asks | WP1 | WP2 |
|---|---|---|---|
| harness `diff_question_pack()` | "does the changed code implement the supplied instruction?", over a state built from the raw branch diff + ticket text | fail 0.15 | fail 0.13 |
| `scripts/jev_canonical_check.py` (plan S3.3 pack) | canon_identity / canon_routing_feed / instruction_matches, over the state the implementing model supplies with acceptance rows + command evidence | **pass 0.81** | **pass 0.92** |

Both runs were keyed, live TypeSafe, `is_fallback=False`.

## Why they diverge (structural, not a defect in either)

A residual ticket in this train states its own premise: the implementation rows
are already on main, so the work is tests + greps + evidence. There is therefore
little or no *behaviour change* in the diff for `diff_question_pack()` to judge,
and it answers at the floor -- 0.13 on WP2, which did add +146 lines of
behaviour. Two unrelated packages landing on the same floor is the tell: the axis
is measuring "is this a full feature implementation", which is not what these
tickets are.

The canonical pack instead asks whether the acceptance rows are met with
evidence, and whether the two canonical invariants (one identity flavor, one
routing feed entry) still hold. That is the question the DONE contract actually
means.

## Ruling requested

Confirm that **`scripts/jev_canonical_check.py` with the plan S3.3 pack is the
WP completion gate**, and that harness `diff_question_pack()` output is advisory
signal on a diff (useful for reviewing the change itself) but is **not** a WP
DONE row. If the orchestrator instead wants the diff pack to gate, then WP1/WP3/
WP4 need a different acceptance definition, because test-and-verify tickets
cannot clear it by construction.

Until ruled, the lane's behaviour is: run the canonical script, record the state
file in `HANDOFF/freebuff/jev/`, report the verdict verbatim, and treat the diff
pack as a review aid only. No WP has been relabelled or reinterpreted to make a
row pass.

## Correction to the train record

`HANDOFF/freebuff/README.md` THE SINGLE TRAIN table carried "WP2 ... keyed JEV
**fail** (supported 0.13)". That reading is superseded by this evidence: WP2
passes the canonical pack at 0.92. PR #352's body likewise records its own
0.15 fail as superseded by a 0.81 pass on the current head.
