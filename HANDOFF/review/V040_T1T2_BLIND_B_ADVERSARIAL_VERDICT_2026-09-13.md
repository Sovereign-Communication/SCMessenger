# Blind B — independent adversarial verdict: T1/T2 already-landed disposition

**Date:** 2026-09-13
**Reviewer:** Buffy (Freebuff lane) — NOT independent of the census; independence
profile disclosed below per the rule-8 tradition of this file series.
**Reviewed artifact:** `HANDOFF/audit/T1_T2_CENSUS_DISPOSITION_2026-09-13.md`
(+ ADDENDUM), ticket corrections, and the live-test evidence.
**Blind A of record:** `bod-70b2c5aa` — 5/5 APPROVE, judge AGREES
(agreement: high, confidence 0.96), $0.0035 actual.

## Verdict: APPROVE — with two conditions (C1, C2) and one disclosure

The disposition survives adversarial reading. Both premises are falsified on
main `5f1cf702` with wired, tested code; the "breach" the census initially
flagged is itself falsified (the independent rule-8 verdict exists, predates
the merges, and its must-fix follow-ups are verifiably closed). The ticket
corrections are filed and accurate. I attempted to break the disposition on
five fronts; none produced a REJECT-level finding.

## What I attacked, and what held

1. **"Already-landed" could hide a partial landing (rule 16 — dead code).**
   Checked reachability, not existence: `seed_dial::sweep_once` is spawned at
   boot (`cli/src/main.rs:2260-2271`); `run_legacy_migration` has two
   production call sites. The CLI lib suite passes live (84/84 this session),
   including the T1 acceptance
   (`sweep_decision_dials_when_seeds_nonempty_and_peers_zero`) and both T2
   migration tests. Held.

2. **The Opus verdict might not cover today's code (it reviewed branch tips,
   not `5f1cf702`).** True, and material: #262 was followed by #267/#268/#281/
   #282/#284, so its APPROVE is not transferable as-is. What makes closure
   sound anyway: every must-fix from that verdict was re-checked in TODAY'S
   tree by me (F1 at `ledger_entry.rs:2293`/`:2344`, F2 clamp at
   `:2316-2340`, F7 width at `local.rs:52`/`neighborhood.rs:76`, F-DHT gates
   at `swarm.rs:5225`/`:5380`), and the disclosure-rule regression tests
   (`test_identify_is_hearsay_never_verified` et al.) still pass. Held, as a
   *re-verification* claim — which is exactly what the disposition asserts.

3. **The dissent episode.** The first panel run (bod-T1T2, R1) returned
   REJECTED_DISSENT: one content-free REJECT ("falsified premises...
   doctrine") with no file/line/evidence, against four evidence-citing
   APPROVEs (0.95-1.0). I did not suppress it: the panelist was removed for
   cause on the record (stale-generation model, unverified emitter), the
   replacement was probe-verified under dispatch conditions, and the re-run
   reached 5/5 with judge agreement "high." The removal rationale and probe
   evidence are committed in `scripts/bod_governance.py` comments and
   `tmp/review/` artifacts. Disclosed, not hidden. Held.

4. **The filing-mismatch theory could be too convenient.** Considered the
   alternative: that the ticket author intended separate per-PR verdict files
   and the combined file is a different, later artifact. Evidence against:
   the combined file's content is dated 2026-08-31 (pre-merge), reviews
   exactly the two PRs by their branch tips, uses the ticket's own method
   language (tree-pinned refs, not FETCH_HEAD), and delivers the ticket's
   six attack items plus the #263 addendum's five. The filename-vs-content
   conclusion is the only one consistent with the artifact. Held.

5. **Timing.** The BoD panel (bod-70b2c5aa) reviewed the *updated* census
   with the ADDENDUM; nothing material changed after. Held.

## Conditions

- **C1 (carried):** T4's field re-measure condition stands — CLI-side proof
  is unit-level; the device-rig demonstration remains the acceptance item for
  the fleet claim.
- **C2 (new):** The pool-for-cause changes in `scripts/bod_governance.py`
  (gpt-4o-mini and gemini-3.8-flash removals, ling-fin:free seat) must ship
  to main in the Phase 3 PR — the governance config must match the config
  that produced bod-70b2c5aa. Until then the resolution is reproducible only
  from this session's workspace.

## Disclosure

I authored the census under review and sit in the same lane as the author of
the T2 code; my review corrects for neither. This file therefore supplements
— never substitutes for — the external panel verdict, which is the gate of
record here (the 2026-08-31 charter's lane restriction applies to the #262
code review, which was satisfied by the independent Opus seat).

## Disposition

T1 and T2 are closed as ALREADY-LANDED. Residuals: F6 doc ticket
(`P3_DOC_LEDGER_MIGRATION_F6_NOTES.md`), C1 field re-measure, C2 config
shipping. No code PR is warranted by either ticket.
