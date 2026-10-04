# DISPATCH: READ-ONLY AUDIT -- Android Ledger Visibility Gap

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Mode: READ-ONLY AUDIT. Do NOT write code. Do NOT apply diffs. Deliver
analysis only, in the report format at the bottom.

## Problem (operator field observation, 2026-08-05)

Fresh Android build (APK from code 50d20011) paired bidirectionally with the
iOS node (Christy) on the same home network. But:

- iOS node listing: 4 nodes + 1 headless visible
- Android node listing: ONLY Christy visible

Doctrine: discovery is ledger sharing between nodes; every node must
converge on the same fleet view. The Android node did not converge.

Ticket: HANDOFF/todo/LEDGER_SHARING_ANDROID_NODE_VISIBILITY_2026-08-05.md

## Known architecture facts (verified at HEAD)

- `core/src/transport/swarm.rs:2223` `share_ledger(peer_id)` sends
  SwarmCommand::ShareLedger; handlers at swarm.rs:5668 and 6224.
- swarm.rs:3936-4010: on receiving a ledger exchange request, core now
  RECIPROCATES automatically from the core ledger ("Answering here means
  every node reciprocates regardless of platform") -- the old mobile bug
  (Android/iOS never calling share_ledger over UniFFI) is already fixed at
  the swarm layer.
- The reciprocal payload is built by
  `core/src/store/ledger_entry.rs:1170 exchange_response_entries()` with
  DISCLOSURE CONTROLS: 64-entry cap applied BEFORE cloning, `known_topics`
  dropped, and `ledger_entry_to_shared_routing_only()` (ledger_entry.rs:1227)
  blanks routing-irrelevant fields (review findings F6 / NEW-2). Test at
  ledger_entry.rs:2051 documents the cap/filter semantics.

## Your task

Trace the full ledger-convergence path and rank root-cause hypotheses for
the Android-only gap:

1. WHAT IS RECIPROCATED: read `exchange_response_entries` +
   `ledger_entry_to_shared_routing_only` (in the attached ledger_entry.rs).
   Which entry types / fields survive the disclosure filter? Is it possible
   that the 4 nodes + 1 headless the iOS node shows come from entry data
   that the reciprocal response strips or caps away for a freshly-paired
   peer? Does the 64-entry cap or any per-peer budget interact with a
   first-contact exchange?
2. INGESTION: after the Android node receives the reciprocal entries, where
   do they land (IronCore ledger ingest path) and could identity
   canonicalization (PR #136: public-key-based identity_id via
   blake3; `identity_id_from_public_key_hex` in core/src/identity/keys.rs)
   drop or mis-key foreign entries during ingest?
3. RENDERING: what does the Android app's node listing actually render
   (ledger entries? contacts? routing table?) -- state precisely which
   Kotlin/UniFFI surface must be checked and what it reads.
4. ASYMMETRY EXPLANATION: construct the most likely explanation for why
   iOS sees 5 nodes while Android sees 1, given both are on the same
   network and paired with each other. Note the iOS node has been in the
   fleet longer (its ledger is richer) -- does convergence require
   multiple rounds / gossip propagation that the fresh Android node never
   triggers?

## Deliverable

Ranked hypothesis list (most likely first), each with: evidence for,
evidence against, the exact file:line sites to inspect or change for the
fix, and a fix sketch. Flag anything needing adversarial review under
AGENTS.md rule 8. List any files you needed but were not provided.

## Report format (mandatory final block)

RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE (read-only audit)
FILES: <files examined>
NOTES: <max 8 lines: top hypothesis + fix direction + what the orchestrator must run next>
