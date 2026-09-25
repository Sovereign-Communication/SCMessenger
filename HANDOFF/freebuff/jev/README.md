# Canonical JEV evidence -- 0.4.0 train

Status: Active
Owner: Freebuff lane (evidence), orchestrator (verdict)
Authority: `HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` section 3

These are the **state files actually fed** to the canonical completion gate, one
per work package, so a WP verdict is reproducible rather than trusted. The gate
is `scripts/jev_canonical_check.py`, which evaluates the section 3.3 canonical
question pack:

- `canon_identity` -- one contact identity flavor (public-key hex) as the
  addressing key
- `canon_routing_feed` -- one routing feed entry point
  (`IronCore::routing_peer_seen`) for all data-link transports
- `instruction_matches` -- the WP instruction and acceptance rows are met, with
  command/test evidence cited

## Re-run

```text
python scripts/update_local_harness.py
python scripts/jev_canonical_check.py --wp WP1 --state-file HANDOFF/freebuff/jev/WP1_state_2026-09-21.json
python scripts/jev_canonical_check.py --wp WP2 --state-file HANDOFF/freebuff/jev/WP2_state_2026-09-21.json
```

Exit 0 requires a **keyed, non-fallback** `result.is_passing(0.70)`. An
`is_fallback=True` result prints `UNVERIFIED-JEV` and is not DONE.

## Recorded verdicts (2026-09-21, this checkout, keyed TypeSafe)

| WP | State file | Verdict | supported | is_fallback | cost | model |
|---|---|---|---|---|---|---|
| WP1 | `WP1_state_2026-09-21.json` | **pass** | 0.81 | False | 0.085218 | jev-1.13.0 |
| WP2 | `WP2_state_2026-09-21.json` | **pass** | 0.92 | False | 0.066654 | jev-1.13.0 |

Axes (noul): WP1 canon_identity 0.96, canon_routing_feed 0.95,
instruction_matches 0.81. WP2 canon_identity 0.95, canon_routing_feed 0.97,
instruction_matches 0.92.

## Why this directory exists: the two packs are not the same gate

Two very different instruments were both being called "the JEV row" in the train:

1. **The harness default `diff_question_pack()`** -- asks one question, "does the
   changed code implement the supplied instruction?", over a state built from the
   raw branch diff plus the ticket text. This is the harness product's own
   reviewer; it is not this repo's DONE gate.
2. **The canonical pack above** -- asks the three canonical questions over the
   state the implementing model supplies, with the acceptance rows and their
   command evidence.

Reading (1) as the DONE gate produced two stale FAILs recorded in the train: WP2
at 0.13 (README table) and WP1 at 0.15. Both had a structural cause, not a
defect: for a residual ticket whose own premise says the implementation is
already on main, pack (1) is asked to find behaviour change in a diff that
contains little or none, so it answers at the floor regardless of whether the
work is real.

Under pack (2), with the fix commit included and each acceptance row carrying its
command evidence, both packages pass. The lesson is not "the row was too
strict": it is that the train must name which instrument it means. The DONE
contract in the README section 7 names the script, so the script is the gate.

Recorded as a ruling request in
`HANDOFF/freebuff/inbox/JEV_CANONICAL_VS_DIFF_PACK_RULING_2026-09-21.md`.
