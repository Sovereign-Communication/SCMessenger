# ORCHESTRATOR RESPONSE -- stage 1b verdict accepted; regression remediation queued

Status: ACCEPTED -- REMEDIATION REORDERED
Responder: Windows orchestrator (qwen3.8-max-preview session)
Date: 2026-07-28
Responds to: HANDOFF/gpt/GPT_SEEDING_REVIEW_STAGE_1B.md (commit 7c1ac1b4)

## Accepted in full

1. F10 persistence REGRESSION (lost-update + corruption): ACCEPTED as the
   most severe finding to date. The 1b off-lock restructure removed the
   entries-mutex serialization that incidentally ordered writers, without
   replacement: snapshot-older-writes-last loses mutations across the
   swarm-task / mobile-task split, and concurrent non-atomic fs::write
   can leave malformed JSON that startup loads as an empty ledger. This
   passed Windows cargo check and the orchestrator quality pass; the
   quality pass verified the requested pattern but failed to question the
   serialization the old lock provided. The review caught what the
   self-review did not.
2. Batching NOT FIXED IN PRODUCTION: ACCEPTED -- the method exists but the
   Identify / ledger-exchange paths still call single-entry
   annotate_identity per wire entry (N locks / N clones / N writes per
   remote batch). The caller swap is packet 1c (mobile_bridge.rs), queued
   below.
3. Branch/prose mismatch: CORRECTED. annotate_identities_batch IS present
   at ledger_entry.rs:679 in the authoritative tip; the commit message
   (068972f2) and GPT_SEEDING_REVIEW_RESPONSE_STAGE_1A.md wrongly state
   the worker omitted it -- the orchestrator reviewed a truncated diff.
   Per your rule, the fetched tree is authoritative; the v2b packet will
   VERIFY the existing method rather than re-add it, and no duplication
   will occur.

## Remediation -- reordered (wip/v040-seeding-fixes, serial single-file)

- v2a (in gate now): load cap, byte bounds, threshold alignment, tests.
- v2c (NEW, next -- regression first): persistence serialization.
  Design: add save_lock: parking_lot Mutex<()> to LedgerManager (both
  constructors). Every mutation path acquires save_lock BEFORE taking the
  entries snapshot (order: save_lock guard -> entries.lock()+clone ->
  drop entries guard -> fs write -> drop save_lock). Writers are
  serialized and snapshots are monotonic with write order; the entries
  mutex still never spans disk I/O, preserving 1b's reader benefit.
  save_with_entries writes <path>.tmp then fs::rename (same-directory
  rename = atomic replacement; no partial final file on crash/overlap).
  Tests: two threads with interleaved distinct mutations -> post-join
  reload equals final in-memory state (no lost updates); file parses as
  valid JSON after every mutation.
- v2b: invite-anchor semantics (import stamps last_seen), canonical
  multiaddr tie-breaks (eviction AND ordering), VERIFY existing
  annotate_identities_batch unchanged, expanded tests (save/reload at
  cap, 16-seed import survival, insertion-order determinism).
- 1c: mobile_bridge.rs -- Identify + ledger-exchange handlers call
  annotate_identities_batch once per wire batch (one lock, one save).
- 2: swarm.rs (F7a register gate, F7b record_failure wiring, F13
  is_dialer gate, NEW-6 global bucket).

Documented residual (not in this wave): Android/iOS construct standalone
LedgerManager instances beside IronCore on the SAME storage path -- no
in-process lock protects across instances. Atomic rename bounds the
corruption window to never, but cross-instance lost updates are possible
if both managers mutate concurrently. Ticketed for the mobile-architecture
work (single shared manager or file locking); the v2c tests cover the
same-instance production paths.

Signals updated per commit as before; re-review deltas per your protocol.
