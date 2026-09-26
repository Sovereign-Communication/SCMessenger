# T1 Design: close the mixed-fleet block bypass (dual-flavor blocks)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: DESIGN ACCEPTED (pending implementation + adversarial review)
Source: qwenpaid / qwen3.8-max-preview design dispatch 2026-08-05 (task file
tmp/t1_block_flavor_design.prompt.md; full raw response in
tmp/t1_block_flavor_design_response.md, footer RESULT: DONE, ledger
recorded). Ticket: HANDOFF/todo/IDENTIFIER_GATE_FOLLOWUPS_2026-08-04.md (T1).

## Problem (one paragraph)

Blocks are stored under whichever identifier flavor the caller passed.
Post-canonicalization blocks are keyed by public key; wire sender_id is the
public key; ingress expands candidates ONE-WAY (pk -> derived identity_id).
A block stored under a public key is therefore never matched when the inbound
sender_id arrives as an identity_id (old pre-canonicalization client), so the
blocked peer's message gets through. The reverse direction (legacy block
under identity_id, message under public key) IS covered by derivation.

## Chosen approach: hybrid A+B (dual-write + ingress verification)

A. WRITE-TIME DUAL-FLAVOR STORAGE (primary): block/unblock/block_and_delete
   resolve both identifier flavors and persist a BlockedIdentity under EACH
   flavor (shared metadata). Unblock deletes all flavors. Migration pass on
   IronCore init scans existing entries and synthesizes missing counterparts
   (idempotent, additive, legacy-safe).
B. INGRESS VERIFICATION (defense-in-depth): receive-path candidate set
   additionally includes the AUTHENTICATED envelope public key and its
   derived identity_id, not just the plaintext sender_id; single-lock
   snapshot + fail-closed semantics retained.

Rejected: reverse-index column (sled cross-tree atomicity is weak; index
drift violates fail-closed).

## Orchestrator assessment (binding conditions for implementation)

1. Dual-write MUST be transactional within the blocked tree or fail-closed:
   a partial write must make block() return Err, never report success with
   one flavor missing. Sled single-tree transactions are available; the
   implementation must verify, not assume.
2. Unblock symmetry is a first-class requirement (test 3 below), not an
   afterthought: an unblock that leaves the counterpart flavor behind is a
   user-visible block that cannot be lifted.
3. Migration runs before the network starts; a migration failure must be
   loud (error surfaced), and ingress hardening (B) covers any residual gap
   during the window.
4. CLI pending-request listing will see duplicate entries under dual storage
   unless it dedupes by derived identity -- this folds into the T3 ticket
   (fail-closed listing fix); implement together or sequence T3 after T1.
5. Scope: core/src/store/blocked.rs, core/src/identity/keys.rs
   (resolve_identity_flavors helper), core/src/iron_core.rs receive path +
   init hook. None of these paths is in the AGENTS.md rule-8 directory list,
   but this is block-gate code: ADVERSARIAL REVIEW ON FILE BEFORE MERGE,
   same bar as the block-gate hardening (HANDOFF/review/
   CORE_BLOCK_GATE_ADVERSARIAL_REVIEW_2026-08-04.md precedent).
6. Key-rotation semantics unchanged on purpose: a block binds the identity
   flavors known at block time; a peer who rotates keys must be re-blocked.
   Documented residual risk, accepted.

## Mandatory test matrix (from the design; all must land with the fix)

1. Mixed fleet, new block + old message: block via PK; inbound sender_id is
   the IID; assert dropped (Err(Blocked)) at ingress.
2. Mixed fleet, old block + new message: block via IID; inbound sender_id is
   the PK; assert hidden/blocked.
3. Unblock symmetry: unblock via one flavor; assert BOTH entries gone.
4. Migration integrity: seed a legacy single-flavor entry; run migration;
   assert both flavors present; assert idempotent second run.
5. Fail-closed store error: force a store error mid-dual-write; assert
   block() errors and ingress drops on store-read error.
6. Envelope/claimed-id mismatch: sender_id claims IID X, envelope verifies
   PK Y with derive(Y) != X; assert the candidate set covers both and a
   block on either blocks the message.

## Dispatch plan

Implement via scripts/delegate_task.py --provider qwenpaid (MAX tier),
--mode diff, scoped files listed above, --apply --verify wrapped in
scripts/build_lock.py; gate = cargo test -p scmessenger-core --test
integration_contact_block + the six new cases. Then adversarial review
dispatch (qwenpaid MAX) against the landed diff BEFORE merge.
