# GPT ADVERSARIAL REVIEW -- Wave 1b stage 1b

Status: BLOCK -- REMEDIATION REQUIRED
Reviewed delta: `d258fd7fecf84363a286093e6f236c0d4b7fa677..068972f2d3cfe4578a7dc713a159a7d0bcee6bf5`
Authoritative tip: `068972f2d3cfe4578a7dc713a159a7d0bcee6bf5`
Remote ref: `refs/heads/wip/v040-seeding-fixes`
Verification: read-only diff and surrounding-source inspection; no Mac build
Windows signal: `cargo check -p scmessenger-core -j2` reported PASS for 1a + 1b; full `cargo test --no-run` pending

## F10 persistence -- REGRESSION

Every mutator now clones under `entries`, drops that mutex, and writes the same
`ledger.json` without a persistence mutex or revision check
(`core/src/store/ledger_entry.rs:227-236`, `:264-284`, `:330-421`,
`:623-695`). Thread A can snapshot S1, thread B can snapshot the newer S2 and
write it, then delayed A can overwrite the file with S1; memory remains at S2
but restart loses B's mutation. This is production-reachable between the swarm
task's `record_connection` (`core/src/transport/swarm.rs:4581-4607`) and the
mobile event task's wire annotations (`core/src/mobile_bridge.rs:1037-1082`),
as well as exported app calls. Concurrent direct `std::fs::write` calls also
truncate the same non-atomically replaced file, so overlap or a crash can leave
malformed JSON; startup then treats the load failure as an empty ledger
(`core/src/iron_core.rs:292-304`). Android and iOS also construct a standalone
`LedgerManager` beside IronCore using the same storage path
(`android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:910-919`,
`:1333-1345`; `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:595-608`,
`:767-781`), so independent manager mutexes cannot protect the shared file.
The old same-instance entries mutex serialized same-manager writes; stage 1b
removes even that ordering without replacing it.

## F10 batching -- NOT FIXED IN PRODUCTION

The authoritative tree already contains `annotate_identities_batch`
(`ledger_entry.rs:679-695`), and its helper extraction preserves the prior
single-entry validation and mutation behavior (`:135-207`, `:396-421`).
However, the Identify and ledger-exchange paths still call
`annotate_identity` once per entry (`mobile_bridge.rs:973-993`, `:1051-1081`),
so a remote batch still performs N locks, N full-vector clones, and N
whole-file writes. No batch, concurrent-save, stale-snapshot, malformed-file,
or crash/reload regression test was added.

## Branch/prose mismatch

Both commit `068972f2` and
`HANDOFF/gpt/GPT_SEEDING_REVIEW_RESPONSE_STAGE_1A.md` say the worker omitted
`annotate_identities_batch` and that v2b will add it, but the method is already
present at the authoritative tip (`ledger_entry.rs:679-695`). The later packet
must treat the fetched tree as authoritative so it does not duplicate the
method or mis-score the caller swap.

## Prior and later-stage findings

Stage 1a's accepted blockers are unchanged by this delta and remain queued in
v2a/v2b. F7(a), F7(b) failure wiring, F13, and NEW-6 are untouched because
stage 1b changes only `ledger_entry.rs`; they remain pending stage 2 rather
than receiving verdicts here.

## Stage decision

NO-SHIP for stage 1b as reviewed. The off-lock change reduces mutex hold time
but introduces lost-update and file-integrity failure modes. Please publish the
remediation as another commit on `wip/v040-seeding-fixes` with its exact parent
and tip; GPT will re-review only that delta plus the authoritative full tree.
