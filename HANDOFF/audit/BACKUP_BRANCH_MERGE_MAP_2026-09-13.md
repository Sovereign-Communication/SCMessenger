# Backup branch merge map -- assist-lane audit, 2026-09-13

Author: Buffy (Freebuff assist lane). Read-only audit; no tree was modified
to produce this file except creating it. Every claim below cites a command
run this session (AGENTS.md rule 13).

## Subject

`backup/cto-t2-disk-dirty-20260913` (tip `4efa253e`, pushed) vs
`origin/main` (tip `5f1cf702`). The backup branch is the recovery line
parked at `b1c8957e` plus the disk-governor commit `7f39c884`. It shares
merge-base `b2d8d126` with main.

`git merge-tree --write-tree 4efa253e a5fe0724` reports 12 conflicted paths.
This memo maps each one. It does NOT merge anything.

## Headline finding

All six backup-side code campaigns are already incorporated on main. For
each, distinctive symbols introduced by the backup commit were probed with
`git log origin/main -S<symbol>` and found PRESENT, landed via the #281
(unified 3-node parity) and #282 (BoD panel + Android stability) wave:

| Backup commit | Campaign | Probed symbols | Verdict |
|---|---|---|---|
| `c459bc90` | D2/D3c/D3d/D4/D8/A4 transport defects | `recordConnectionFailure`, `startAdvertisingInternal`, `stopAdvertisingInternal` | PRESENT on main |
| `0a33c009` | bound-port advertisement gate, BLE ownership | `active_peer_selection_contract`, `address_observer_contract`, `empty_listen_port_set_fails_closed` | PRESENT on main (via 956ec371) |
| `7ff317f0` | D10 relay-reservation validation | `d10_concrete_self_base_is_rejected`, `d10_empty_local_set_does_not_block_foreign_relay`, `d10_exact_host_match_is_rejected_even_without_wildcard` | PRESENT on main (via 956ec371) |
| `9fa9f9bd` | D10b poison-listener guard | `is_poison_circuit_listener`, `poison_listener_guard_passes_direct_listeners`, `poison_listener_guard_rejects_nested_double_circuit` | PRESENT on main (via b5a70bd5/956ec371) |
| `74253491` | T14 configured-external primacy | `configured_external_address_wins_over_observations`, `set_configured_external` | PRESENT on main (via 956ec371) |
| `1749186e` | Android seed-tier sweep | `MAX_BOOTSTRAP_SEEDS`, `mergeBootstrapCandidates`, `getBootstrapCandidateAddresses`, `blankEntriesDroppedFromBothTiers`, `duplicateSeedsCollapse`, `emptyLedgerYieldsNoCandidates` | PRESENT on main |

Caveat (rule 15, stated plainly): symbol presence proves the features
landed; it does not prove every hunk of every fix landed verbatim. The
per-file resolution calls below therefore say TAKE-MAIN *with a verification
pass*, not "blindly".

## The 12 conflicted paths and recommended resolution

All 12 are BOTH-CHANGED at the blob level (neither side's content exists in
the other's history), so none resolve automatically.

### Group A -- TAKE MAIN (backup content verified present via symbols)

| Path | Main-side shaped by | Backup-side shaped by |
|---|---|---|
| `core/src/routing/local.rs` | #263 bb253eaf, #268 d395e030, #281 | 0a33c009 |
| `core/src/routing/optimized_engine.rs` | #268, #281 | 0a33c009 |
| `core/src/store/ledger_entry.rs` | #262 68e2c275, #267 45ab59f9, #281 | c459bc90, 2b84879f (fmt) |
| `core/src/transport/swarm.rs` | #267, #268, #270 c824fe9a, #281 | 7ff317f0, 9fa9f9bd, 2b84879f |
| `core/src/transport/observation.rs` | #270, #281 | 74253491, 0a33c009 |
| `android/.../data/MeshRepository.kt` | #262, #281, #282 | 1749186e, c459bc90, b1c8957e |
| `android/.../service/AnrWatchdog.kt` | #281 | b1c8957e |
| `android/.../transport/TransportManager.kt` | #281 | 0a33c009 |

Resolution: take main's version, then one diff pass
(`git diff 4efa253e:<path> 5f1cf702:<path>`) to eyeball for any backup
nuance the symbol probe could not see. The backup blobs remain reachable
from the backup branch forever, so nothing is lost by deferring.

### Group B -- MANUAL MERGE (documentation unique to each side)

| Path | Notes |
|---|---|
| `HANDOFF/CTO_STATE.md` | main-side: #280 ticket moves; backup-side: D10 reviewer packet, resume-state refresh. Both narratives are real history; merge both. |
| `HANDOFF/CEO_STATE.md` | main-side: #281; backup-side: receipt-return RCA addendum + parked-tree backup. Merge both. |
| `HANDOFF/review/V040_T14_EPHEMERAL_REVIEW_QWEN_2026-09-01.md` | both sides extended the same review doc; union-merge. |
| `.codebuff_deploy/rev/pr251.diff` | deploy artifact; lowest stakes -- take either, note the choice. |

## Merge-path options (operator decision -- rules 9/12)

- **Option A (merge in place):** merge `backup/cto-t2-disk-dirty-20260913`
  into a fresh branch off current main, resolve per the tables above, run
  full gates. Cost: 8 code files need a careful verification pass.
- **Option B (recommended, cheaper):** the backup branch's only surviving
  unique value is the disk-governor commit `7f39c884` (5 files: two cargo
  configs, rules_check.py, reap_worktrees.sh, AGENTS.md hygiene) plus the
  Group-B docs. Cherry-pick `7f39c884` onto a fresh branch off main, port
  the three HANDOFF doc narratives, and retire the remaining 85 commits
  (their content is verifiably on main via #281/#282). Zero code conflicts.
  Requires operator sign-off because it abandons branch lineage (rule 12).

## F6 residual (paste-ready for the Phase-3 session's lane)

Independent verification this session: `cli/src/ledger.rs` (main) documents
and implements rename-on-success (`peers.json` -> `peers.json.migrated-<ts>`,
lines 5-16, 188-197); "leaving in place" is only the error path (lines 214,
224). The stale half is the core-side doc comment:

```diff
--- a/core/src/store/ledger_entry.rs
+++ b/core/src/store/ledger_entry.rs
@@ -2238,7 +2238,7 @@
     /// (operator-configured by fiat) survives as verified; everything else
     /// imports as hearsay and is re-proven by the first live dial.
-    /// `peers.json` is left in place; the caller simply stops writing it.
+    /// `peers.json` is archived by the caller on successful migration
+    /// (renamed to `peers.json.migrated-<ts>`, cli/src/ledger.rs); it is
+    /// left in place only when unreadable or invalid legacy JSON.
     pub fn import_legacy_cli_entries(
```

The Phase-3 session owns the F6 ticket (P3_DOC_LEDGER_MIGRATION_F6_NOTES.md);
this diff is an offer, not an edit.

## Status snapshot at writing time

- PR #285 (feat/t1t2-governance-20260913 -> main): MERGEABLE, zero failed
  checks, `Test (windows-latest)` + `Bindings (Swift)` in flight.
- main CI at 5f1cf702: fully green (CI, CodeQL, Docker).
- The Phase-3 session's T1/T2 census conclusions were independently
  corroborated by this lane before their thread was read: #266/#262/#263
  merge commits verified in history; `bod_governance.py` byte-identical
  between their branch and the backup branch.

## Update (same day, later) -- assist lane

- PR #285 MERGED at 2026-09-13T20:57:57Z; main advanced to `6e726402`.
  Verified: all 6 disposition files landed (+666/-16), RULE8 ticket now
  CLOSED on main with closure record, BoD per-seat APPROVEs (0.95) present
  in BOD_STATE.md.
- Recoverability audit of the 8 reaped worktrees: every local branch ref
  persists and every tip is contained on the remote (fresh fetch; two tips
  are already ancestors of main). The reap destroyed nothing.
- `scripts/backup_merge_resolve.sh` added as the gated executor for the
  options below: dry-run default, `--apply` requires
  `SCM_BACKUP_MERGE_APPROVED=1`, never commits, stops before Group B.
