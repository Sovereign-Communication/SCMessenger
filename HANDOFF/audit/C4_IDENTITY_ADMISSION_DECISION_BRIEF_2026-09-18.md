# Decision brief: c4 — identity multiplication vs the per-peer relay share

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Rule-9 escalation (security trade-off) arising from the PR #305 rule-8 review
(c4: real, 2R/1NR, judge medium). For operator decision. Written 2026-09-18.

## The defect, precisely

TRN-07's fix caps one peer at `max(global/4, 25)` of the node's hourly relay
budget (`relay_admission`, `core/src/transport/swarm.rs` — verified this
session). The counter is keyed by the *libp2p PeerId of the connection*. A peer
holding many identities — i.e. many keypairs, which cost nothing to generate —
gets one share per identity: 4 identities on one machine reconstitute the full
200/hr budget that TRN-07 exists to protect, and deny relay to every other peer
for the remainder of the window. Same defect class as TRN-07, new dimension:
identity multiplication instead of single-peer greed.

Bounded today by: the node-global 200/hr ceiling (total work is capped), the
c3 map bound (memory cannot be exhausted), and the observation that natural
inbound relay peaks at ~15/hr on the live fleet. Unbounded today by: nothing
prevents N identities each drawing an individual share.

## Verified facts that constrain the design

1. `RelayRequest` (`core/src/transport/behaviour.rs`) carries
   `destination_peer`, `envelope_data`, `message_id`, and the optional WS13
   fields. **No source identity, no request signature.** The requester is known
   only as the noise-authenticated immediate peer.
2. PeerIds in this codebase are derivable from public keys (live-verified
   2026-09-17: all three fleet peers re-derived from their real public keys,
   3/3), so the carrier can recover the requester's public key from the
   noise-authenticated PeerId and derive its `identity_id`
   (`blake3(pubkey)`). Identity *recognition* needs **no wire change and
   nothing spoofable**: noise already proves possession of the key that hashes
   to the PeerId.
3. The RCA (TRN-04, "Not claimed") is explicit: no cryptographic sender
   authentication is possible at the custody boundary today; binding belongs
   one layer up. Any option claiming "sender auth at admission" is a wire+
   envelope change, not an admission tweak.
4. Identity-bearing signals already in the store: contacts
   (`identity_id_idx` -> public key), the device registry
   (`register(identity_id, device_id, seniority_timestamp)`; WS13.3
   registrations are canonically signed), and the blocked store's
   identity_id alias handling.

## Options

### A — Known/unknown classification with an aggregate unknown pool (recommended)

Classify each requester at admission: derive `identity_id` from the
noise-authenticated PeerId, look it up against contacts/registry. *Known*
identities keep an individual share (today's 50). *Unknown* identities share
one aggregate pool — e.g. `global/4` total per hour for all unknowns combined —
instead of individual shares. Ten Sybil identities in the unknown class now
divide one bounded pool; multiplication buys nothing.

- Closes c4's mechanism directly (the attack is creating fresh identities).
- Zero wire-format change; extends the already-extracted pure ladder, so it
  inherits the TRN-07 test pattern (`relay_per_peer_budget_tests`).
- A newcomer grace window (first contact minutes get a small individual share)
  keeps the invite-first mesh welcoming; ledger-sharing discovery means peers
  the mesh knows get classified known quickly.
- Risks: a carrier's contact set becomes security-relevant (a node with a
  poisoned contact set classifies wrong — self-inflicted, operator-visible);
  contact-flooding becomes a social vector worth watching; the unknown pool
  can be starved by unknown-but-legitimate traffic on a busy node (mitigation:
  pool scales with global budget; budget is the operator's knob).
- Complexity: LOW-MEDIUM. One classification read at admission (cacheable per
  peer per window), one pure-function parameter, native+wasm parity, tests.

### B — Signed relay requests (identity binding on the wire)

Add `sender_pubkey` + Ed25519 signature over `(message_id, destination_peer,
envelope hash)` to `RelayRequest`; admission verifies and keys the budget by
the verified identity_id.

- Makes per-identity accounting honest and lets abuse signals attach to
  durable identities. Necessary groundwork if relay abuse *attribution* ever
  becomes a requirement.
- Does NOT close c4 by itself: an attacker owns unlimited real keypairs and
  signs with them. Sybil resistance still needs a scarcity input (seniority,
  attestation) on top — more design surface.
- Wire-format change with serde-compat shims (unsigned legacy senders must
  remain admissible or the mesh forks), crypto at the admission hot path,
  wasm parity, and it contradicts the RCA's stated boundary (auth belongs one
  layer up).
- Complexity: MEDIUM-HIGH. Not the right tool for this defect.

### C — Aggregate cap only (A without the classification)

One aggregate cap on ALL relay per source node... not implementable without a
network-level grouping concept the transport does not have; grouping by
"unknown" (A) is the realizable version. Listed for completeness.

### D — Proof-of-work for unknown requesters

- Adds real cost to Sybil volume; no wire identity needed.
- Punishes mobile senders (battery), no PoW precedent in the codebase,
  parameter tuning is a research project, wasm cost profile differs.
- Complexity: HIGH, fit poor. Not recommended.

### E — Accept and bound (document, defer)

- The global ceiling already bounds total relay work; the c3 map bounds
  memory; the fleet is invite-seeded and small. Document the residual in the
  threat model and revisit at mesh scale.
- Zero risk of new defects; leaves a one-hour relay-denial window reachable
  by any peer with 4+ identities. Defensible for 0.4.0; weak for growth.

## Recommendation

A now (0.5.0 lane), E as the documented interim. A is the only option that
closes the actual mechanism without a wire change or a new trust assumption;
its risks are operational knobs, not design flaws. If/when relay abuse
attribution becomes a requirement, do B as its own reviewed change — B is
groundwork, not a c4 fix. Sequence: brief approved -> A1 (aggregate unknown
pool + grace window) on a feature branch with the pure-function tests ->
adversarial review (transport-gated) -> parity check on the wasm loop.

## Decision requested from the operator

1. Adopt A for 0.5.0 with E as interim? (yes/no/other)
2. If A: unknown-pool size — `global/4` (i.e. 50/hr shared), or a dedicated
   default independent of the global budget?
3. If A: grace window shape — fixed minutes per PeerId, or first-N-admissions?
4. Confirm B stays out of scope absent an attribution requirement.

Raw review evidence: HANDOFF/review/RULE8_PR305_VERDICT_2026-09-18.md (c4
row), Harness runs `verify_pr305_identity_regate_20260918.json`.
