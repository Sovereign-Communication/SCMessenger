# DISPATCH: DESIGN DOC -- Trust-Scoped LAN Disclosure for Ledger Exchange

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Mode: DESIGN/WRITING. Produce a design document in markdown. No code
changes. No diffs.

## Background (verified facts at HEAD, 2026-08-05)

- Ledger exchange (`/sc/ledger-exchange/1.0.0`) auto-reciprocates on every
  completed Noise handshake (core/src/transport/swarm.rs:3936+).
- The reciprocal payload is built by
  core/src/store/ledger_entry.rs:1170 `exchange_response_entries()`:
  filters `success_count > 0`, drops the requester's own entry, applies
  `is_disclosable_multiaddr` (which NEVER discloses RFC1918 ranges -- test
  `exchange_response_never_discloses_private_ranges` at line ~2051), caps
  at `limit` entries, blanks `known_topics` (review finding F6/NEW-2).
- Doctrine: discovery is ledger sharing between nodes; every node should
  converge on the same fleet view.
- Field consequence (2026-08-05): on a home LAN (192.168.0.0/24) no node
  can share its LAN neighbors via ledger exchange; fleet convergence on
  LAN depends entirely on local discovery (mDNS/BLE), breaking the
  ledger-sharing doctrine and producing asymmetric fleet views.
- Operator decision 2026-08-05: "both, sequenced" -- Phase 1 fixes
  Android local discovery (separate dispatch); Phase 2 (THIS design) adds
  trust-scoped LAN disclosure.

## Your task: write the design document

Deliver a complete design doc (markdown, no emojis) covering:

1. THREAT MODEL: what NEW-2 / F6 disclosure controls protect against
   (topology leakage to any Noise-handshaked stranger, SSRF-adjacent
   abuse of private-range addresses, group-membership leakage via topics)
   and what exactly changes when RFC1918 disclosure becomes possible.
2. GATING MECHANISM: RFC1918 entries may only flow to peers with
   CRYPTOGRAPHIC pairing proof -- not mere Noise handshake completion.
   Design the trust predicate: what existing pairing/identity evidence in
   the codebase qualifies (identity signatures, block/allow list state,
   ledger annotate_identity entries, contact status)? Prefer reusing
   existing primitives over new protocol concepts. Name exact files and
   functions to consult (e.g. core/src/identity/, core/src/store/).
3. WIRE COMPATIBILITY: how the change rides the existing
   `/sc/ledger-exchange/1.0.0` protocol (field additions? version bump to
   1.0.1? capability advertisement?) without breaking older nodes; state
   the mixed-version fleet behavior explicitly.
4. API SURFACE: exact function-level changes in
   core/src/store/ledger_entry.rs and the swarm.rs exchange handler
   (swarm.rs:5668 / 6224 paths), including how the caller proves the peer
   qualifies at call time.
5. DOWNGRADE/ABUSE ANALYSIS: can an unpaired peer trick the predicate
   (replay, impersonation via relayed handshakes, MITM on the pairing
   channel)? Enumerate attacks and the design's answer to each.
6. TEST PLAN: unit tests for the predicate, the filter behavior paired vs
   unpaired, mixed-version exchange, and the disclosure-regression tests
   that must KEEP passing (private ranges still never leak to unpaired
   peers).
7. ROLLOUT: feature-flag vs direct, default-off considerations, and the
   adversarial-review gate (AGENTS.md rule 8) checklist for the reviewer.
8. OPEN QUESTIONS for the operator, if any.

## Constraints

- Security doctrine first: the default for unpaired peers must remain
  exactly today's behavior. Any relaxation is opt-in by trust evidence.
- No emojis anywhere. Plain markdown. Cite file paths precisely.

## Report format (mandatory final block)

RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE (design doc)
FILES: <none; document text is the deliverable>
NOTES: <max 8 lines: design headline + key open questions>
