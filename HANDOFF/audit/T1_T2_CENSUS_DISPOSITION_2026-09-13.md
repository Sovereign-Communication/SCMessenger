# T1 + T2 CENSUS & DISPOSITION — Phase 3 (boot seed-dial, peer-store unification)

**Date:** 2026-09-13
**Written by:** Buffy (Freebuff recovery session)
**Tree verified:** origin/main `5f1cf702` (post-#282/#284), worktree
`tmp/wt-recovery-20260913`, branch `feat/t1-boot-seed-dial-20260913`.
**Method:** fresh census per the Phase 2 lesson (stale-tree premises falsify
themselves); every claim cites a command run this session.

## Headline

**Both Phase 3 premises are FALSIFIED on current main.** T1 (boot seed-dial)
and T2 (peer-store unification) are implemented, wired, and tested. No code
will be written for either. The census initially framed the open P0 rule-8
ticket as a merge breach to remediate retrospectively — **that breach
hypothesis is itself falsified; see the ADDENDUM below.** The deliverable is
this census, the ticket-status corrections, and the governance gate on this
disposition.

## T1 — CLI boot seed-dial: ALREADY LANDED (PR #266, not rule-8 gated)

The premise "headless CLI boot never dials seed peers; sole
`connect_to_seed_peers` caller is mobile_bridge.rs:862" was true on the
pre-#282 tree only. On 5f1cf702:

- `cli/src/seed_dial.rs` (whole module read this session): "Boot seed-dial
  sweep (V040-T1 HALF 2)" — bounded exponential backoff (`next_delay`:
  5s/15s/45s/120s), pure `sweep_decision` (WatchOnly/Wait/Dial),
  `sweep_once` issuing `SwarmHandle::connect_to_seed_peers()`, candidates
  from the core ledger proven+seed tiers (`candidate_count`).
- **Reachable (rule 16):** `cli/src/main.rs:2260-2271` spawns the long-lived
  sweep task at boot (`seed_dial::sweep_once` in a tokio loop).
- **Tests in-file:** backoff bounds, dial-when-seeds-nonempty-and-peers-zero
  (the T1 acceptance), wait-when-empty, watch-when-connected, and
  `candidate_count_reflects_seeded_core_ledger` (import -> non-zero).
- Landing: `git log --diff-filter=A -- cli/src/seed_dial.rs` ->
  `67d19d3c feat(cli): boot-time seed dial with bounded backoff
  (V040-T1 HALF 2) (#266)` — CLI-only surface, outside the rule-8 perimeter.
- Swarm-side `connect_to_seed_peers` implementation:
  `core/src/transport/swarm.rs:3062`.

## T2 — peer-store unification: ALREADY LANDED (PR #262, merged 2026-09-01)

The premise "split stores, 0 vs 4,678 entries, not superseded by #262/#281"
is the SHIP_PLAN 6.3b evidence from BEFORE the unification. On 5f1cf702:

- `cli/src/ledger.rs:1-17` header: "**T2 unification (2026-08-31)**: the
  CLI's own persisted `peers.json` store is GONE. The core `LedgerManager`
  is the single peer store for the whole node" — ConnectionLedger retains
  only process-lifetime dial state (backoff/claims/DialPolicyManager).
- **Migration reachable (rule 16):** `run_legacy_migration`
  (cli/src/ledger.rs:200) is invoked from **two** boot tasks —
  cli/src/main.rs:2291 (aggressive-discovery dial block) and :3684
  (initial bootstrap dial block) — importing survivors into the core store
  and archiving the file as `peers.json.migrated-<ts>` so it cannot run
  twice. "Nothing writes `peers.json` anymore."
- **Trust model (disclosure-relevant):** `import_legacy_cli_entries`
  (core/src/store/ledger_entry.rs:2241+) rejects empty/non-recordable
  multiaddrs, ephemeral source ports (the pollution source per the commit
  message), and self-entries; legacy `locally_verified` is NOT trusted —
  only operator bootstrap survives as verified, everything else re-proven by
  first live dial. Hearsay via `record_identified_peer` is "never verified"
  by design.
- Landing: `gh pr view 262` -> **MERGED 2026-09-01T09:37:54Z**, merge commit
  `68e2c275` (+2,172/-1,666 across 10 files, commit message cites the CEO
  ruling of 2026-08-31). #263 (routing feed) merged same day, `bb253eaf`.

## THE BREACH THE CENSUS SURFACED (this is the real Phase 3 work)

`HANDOFF/todo/RULE8_REVIEW_PR262_LEDGER_UNIFICATION.md` is **Status: OPEN,
P0**: "neither #262 nor #263 can merge without a recorded APPROVE", and it
bars Freebuff-lane-authored review (both PRs were Freebuff-authored; rule 8
requires a reviewer that did not author the change).

Yet `gh pr view` shows **both PRs MERGED on 2026-09-01** — no recorded
adversarial APPROVE found in `HANDOFF/review/` (grep for 262/263/ledger
returns only older visibility audits from July/August) or BOD_STATE (grep
"262|263" hits only the Phase-2 T4 entries). The merge happened past its own
gate. AGENTS.md hard rule 8 says changes are "NOT done" without the review
on file; retroactively, the review is the remedy that exists.

**Review-vehicle legitimacy:** the ticket routes the review to a non-author
lane (Qwen/DashScope or a native seat). The BoD panel satisfies the
substantive requirement — independent, external, non-author models voting on
the disclosure/trust delta with the full diff in front of them — even though
it is not the ticket's named Qwen route. Blind B (this session) must be
labeled supplementary: Buffy is Freebuff-lane, the author-lane of #262.
Dual-approve therefore = BoD panel (substantive) + this session's Blind B
(supplementary, conflict-disclosed). If the operator prefers the named Qwen
route, this disposition stops here and the packet goes there instead.

## Disposition

1. T1: code-level acceptance criteria MET on main via #266 (boot sweep,
   backoff, real dial, ledger-sourced candidates, in-file acceptance tests).
   Nothing to implement.
2. T2: code-level acceptance MET on main via #262 (single store, migration
   wired at two boot sites, archiving, trust downgrade of hearsay).
   Nothing to implement.
3. The open P0 review ticket converts into a RETROSPECTIVE review of #262's
   actual delta (disclosure/trust changes being the gated substance), run
   through the verified BoD paid panel + supplementary Blind B this session.
   Outcome recorded in BOD_STATE + review file; ticket then closes with the
   recorded verdict — or the operator re-routes to the Qwen lane.
4. No PR is opened for Phase 3 unless the retrospective review REJECTS; a
   REJECT converts into a remediation plan for operator decision (options:
   revert of the disclosure surface, forward-fix PR, or accepted-risk
   ruling). That decision is the operator's, not this lane's.

## Commands cited (this session)

`grep -rn connect_to_seed_peers --include=*.rs core/src cli/src`;
`cat cli/src/seed_dial.rs`; `sed -n 2250,2285p cli/src/main.rs`;
`sed -n 1,30p cli/src/ledger.rs`; `grep -rn run_legacy_migration ...`
(two production call sites); `git show 68e2c275 --stat`;
`grep -n "fn import_legacy_cli_entries" -A 30 core/src/store/ledger_entry.rs`;
`gh pr view 262/263 --json state,mergedAt,mergeCommit`;
`ls HANDOFF/review/ | grep -iE "262|263|ledger"`;
`grep -n "262\|263" HANDOFF/BOD_STATE.md`.

## ADDENDUM (same session, after the first write): the rule-8 "breach" is falsified too

1. **The independent rule-8 review EXISTS and APPROVED both PRs before merge.**
   `HANDOFF/freebuff/inbox/RULE8_PR262_PR263_VERDICT_OPUS.md` (2026-08-31
   20:54): "VERDICT: PR #262 -- APPROVE" and "VERDICT: PR #263 -- APPROVE",
   reviewer self-declared independent of both PRs and of the T2 spec, trees
   pinned by ref (`2e32ffad`, `bc5bff0f`, base `b2d8d126`), four ledger-egress
   points enumerated (not just the two the charter named),
   `git merge-tree --write-tree` conflict check, per-finding file:line. Both
   PRs merged 2026-09-01 — AFTER the verdict. The charter's output filename
   (`inbox/RULE8_PR262_VERDICT.md`) differs from the delivered one
   (`RULE8_PR262_PR263_VERDICT_OPUS.md`), which is why the ticket's
   `Status: OPEN` was never cleared — a filing mismatch, not a skipped gate.
2. **Every must-fix follow-up is verifiably closed on 5f1cf702** (all read
   this session):
   - F1 (migration imports hearsay as `locally_verified`): closed by
     "V040-T13 F9: the legacy verified flag is not trusted -- only operator
     bootstrap survives" at `ledger_entry.rs:2293` and
     `locally_verified: entry.is_bootstrap` at `:2344`.
   - F2 (wire-supplied `last_seen` steers eviction/seed order): closed by
     `clamp_wire_last_seen_ms` at the legacy-merge and wire-merge paths
     (`ledger_entry.rs:2316-2318`, `:2334-2340`).
   - F7 (4-byte hint collision, widened reach): closed by PR #268 (merged
     2026-09-03, `d395e030`) — `reachable_hints: Vec<[u8; 8]>` at
     `local.rs:52`,`:72`, `neighborhood.rs:76`,`:323`.
   - F-DHT A (hearsay `kademlia.add_address` feeds): gating present at the
     ruled sites (`swarm.rs:5225`, `:5380` gated; `:5589`/`:5714` are
     explanatory comments; `:5896` is the own-address path the ruling left
     alone).
   - F6 [INFO] (doc says `peers.json` "is left in place" vs the actual
     rename at `cli/src/ledger.rs:720`): **STILL OPEN** — one doc line at
     `ledger_entry.rs:2240`. Ticketed: `P3_DOC_LEDGER_MIGRATION_F6_NOTES.md`.
3. **Live proof this session** (worktree at 5f1cf702, warm build):
   `cargo test -p scmessenger-cli --lib` -> **84 passed; 0 failed**, including
   `seed_dial::tests::{sweep_decision_dials_when_seeds_nonempty_and_peers_zero,
   next_delay_is_bounded_exponential_backoff, sweep_decision_waits_when_no_candidates,
   sweep_decision_watches_when_connected, candidate_count_reflects_seeded_core_ledger}`
   and `ledger::tests::{test_legacy_migration_strips_untrusted_verified_flag_and_archives,
   test_legacy_migration_skips_self_and_ephemeral, test_identify_is_hearsay_never_verified}`.

## Revised deliverable

No code PR for T1/T2. Remaining Phase 3 artifacts: this census (updated),
the rule-8 ticket status correction, the F6 ticket, the governance gate on
this disposition (BoD paid panel + Blind B), then one docs-only PR carrying
all of it to main.
