# PQC-07 ratchet desynchronization — design response

Status: DESIGN RESPONSE — NOT AN IMPLEMENTATION
Response to: `48aec750` / `GPT_SOL_ULTRA_PQC07_RATCHET_DESYNC_2026-08-01.md`
Scope: post-0.5.0 PQC / v1.0.0 lane

## Disposition

The request is well-scoped and factually useful, but it is not a 0.5.0
acceptance task. The unified release plan explicitly excludes full PQC waves
3–5 from 0.5.0. Keep this design, its implementation, and its adversarial
review in the later v1.0.0 PQC lane. It must not displace identity, transport,
receipt, physical-device, cloud-node, or diagnostics work.

No implementation is approved by this document.

## Corrections to the request and the older tickets

The cited KDF behavior is correct: `root_key_ratchet_v2` conditionally mixes
the supplied PQ secret. The cited two advances in `handle_dh_ratchet` are also
present.

The design must nevertheless center the live path:

- `handle_dh_ratchet` is marked dead code in the audited tree.
- A changed peer DH key enters `handle_dh_ratchet_trial`.
- The trial path tries candidates, performs the two advances, and commits the
  cloned state only after authenticated decryption succeeds.
- `handle_incoming_pq_fields` currently logs failed decapsulation and returns
  `Ok(vec![])`; this is a separate soft-failure path that the final design must
  close or explicitly prove safe.

The older E01B and milestone text saying that the PQ secret is never mixed into
the root, or that `handle_dh_ratchet` unconditionally passes `None`, is stale
relative to this tree. The corrected status is: PQ mixing exists, but the
event ordering, optionality, and active trial-path contract are not yet proven.

## Proposed synchronization contract

The implementation should replace implicit local `Option` state with an
explicit, authenticated ratchet event. A suite-0x02 event is identified by:

`(session transcript, suite id, ratchet epoch, step index, direction, DH key
fingerprints, PQ status)`

The raw PQ secret is never serialized. `PQ status` is one of:

- `Present(key_id, decapsulated_secret)`, where `key_id` identifies the
  authenticated PQ ciphertext/keypair epoch; or
- `Absent(reason)`, only when absence was explicitly negotiated for that
  event.

The following invariants are mandatory:

1. Every accepted suite-0x02 event has exactly one predecessor, one epoch, and
   one step index. A peer must not infer event identity from whether a local
   pending field happens to be populated.
2. Both sides apply the same ordered event sequence. The receiving and sending
   transitions may remain separate local substeps, but they must be represented
   as distinct, domain-separated events with the same epoch and explicit step
   indices. The second substep cannot run until the first is committed.
3. `Present` and `Absent` are protocol values, not local fallbacks. A missing
   PQ field, unknown key id, failed decapsulation, or unavailable required
   predecessor is a protocol error for suite 0x02; it must not silently become
   `None` and derive a different root.
4. A peer without PQ material must negotiate the classical suite before the
   session starts, or fail the suite-0x02 session. It cannot silently downgrade
   one ratchet event.
5. State is committed atomically after the event's authenticated message is
   verified. Failed trials must leave root, chain, pending-PQ, epoch, and
   counters unchanged.

### Roles, simultaneous ratchets, and reordering

Initiator/responder role does not determine PQ availability. The authenticated
event carries the sender's event id, DH public key, PQ key id/status, and
predecessor reference. The receiver derives only after resolving that event
against its local predecessor.

For simultaneous ratchets, both events are ordered deterministically by the
protocol event id (epoch, step, and role tie-breaker), rather than by arrival
time. Each event is applied once. A duplicate is idempotently acknowledged or
discarded; an event whose predecessor is missing is bounded and queued; an
event from an already-closed epoch is rejected as stale. Arrival order must
never select a different PQ secret or a different root-advance order.

## Desynchronization detection and failure behavior

The KDF input for each event must bind the complete event descriptor, including
the suite, transcript, epoch, step, direction, both DH public-key
fingerprints, and PQ key id/status. This makes a `Present`/`Absent`, ordering,
or epoch disagreement derive a different key.

The resulting chain key must then be proven by the message authentication
operation before state adoption. A mismatch must produce an explicit ratchet
or authentication error and no state mutation. In particular:

- failed decapsulation must not return an empty successful secret;
- a missing required PQ predecessor must not call the KDF;
- the trial path may try a bounded, explicitly identified previous keypair, but
  each candidate must carry the same event id and must be committed only on
  successful authentication;
- logs should identify the session-safe event id, epoch, step, and failure
  class, never raw keys or ciphertext secrets.

## Test oracle

Before implementation is accepted, add a model-based state-machine test for
two peers. Generate a valid event trace and then perturb it with:

- initiator and responder ratchets;
- simultaneous ratchets;
- reordered, duplicated, replayed, and dropped events;
- delayed PQ material;
- `Present`/`Absent` disagreement;
- current versus previous PQ keypair candidates;
- restart/reload between the two substeps; and
- suite-0x01 control sessions.

For every trace, assert:

1. If both peers accept an event, their event descriptors, root keys, chain
   keys, epoch, and counters equal the model.
2. If descriptors or PQ status differ, acceptance fails before state commit.
3. A failed candidate leaves a byte-for-byte equivalent session state.
4. Decryption failure is observable as a typed failure, not a successful empty
   PQ value.
5. Replaying an accepted event cannot advance the root a second time.

The oracle must exercise `handle_dh_ratchet_trial` and
`handle_incoming_pq_fields`; unit tests of `root_key_ratchet_v2` alone are not
enough.

## Migration

Suite 0x01 behavior remains unchanged. If the event descriptor, KDF domain, or
wire semantics change, do not silently reuse suite 0x02. Allocate a new
authenticated suite/version, reject or re-handshake in-flight sessions that
cannot prove the new contract, and retain decryption compatibility for
previously persisted data according to the repository's migration rules.

Reusing 0x02 is acceptable only after adversarial review proves wire and state
compatibility with every already-negotiated 0x02 session. The default safe
decision is a new suite/version plus an explicit session transition.

## Required gate sequence

1. Reconcile the stale E01B/milestone statements with the live code path.
2. Produce a complete protocol design and event-state diagram.
3. Run mandatory crypto-security adversarial review.
4. Implement on a dedicated PQC branch only after the design verdict.
5. Run the state-machine/property oracle, Kani feature gates, and full Rust
   checks, including restart and migration cases.
6. Keep the result out of the 0.5.0 parity candidate unless the release owner
   explicitly re-baselines the release gates.

