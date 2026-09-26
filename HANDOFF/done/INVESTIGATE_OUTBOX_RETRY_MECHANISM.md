# INVESTIGATE: Does core/src/store/outbox.rs have a working periodic retry/flush?

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: read-only investigation, no code changes in this pass.

## Context

`CRITICAL_OUTBOX_NEVER_FLUSHES_DESPITE_ACTIVE_CONNECTION.md` (todo/) found
that a message queued via `core/src/iron_core.rs::prepare_message_internal`'s
`self.outbox.write().enqueue(...)` sat with `attempts=0` for 11+ hours even
with an active connection to the recipient. A separate fix
(core/src/transport/swarm.rs) now makes already-connected sends bypass the
outbox-adjacent routing decision and dispatch immediately - but that only
helps messages sent WHILE already connected. For the general case (genuinely
offline recipient, connects later), something needs to periodically retry
outbox entries.

## Questions to answer

1. Does `core/src/store/outbox.rs` (608 lines) have ANY function that scans
   queued messages and attempts re-delivery (a `flush`/`retry`/`process`-style
   method)? Quote the function signature and its body's core logic if found.
2. If such a function exists, is it actually CALLED anywhere - on a timer/
   interval, on a peer-connected event, or only manually/on-demand? Search
   for its call sites across `core/src/`.
3. Does anything update the `attempts` field on a `QueuedMessage` after
   enqueue? If `attempts` only ever gets set at creation (0) and never
   incremented, that's strong independent evidence no retry loop actually
   runs, matching the observed 11-hour `attempts=0` symptom.
4. Is there an analogous mechanism in `core/src/drift/` (the "drift custody"
   store that `prepare_message_internal` hands StoreAndCarry-decided messages
   to instead of the outbox) - does IT have a working retry/gossip mechanism,
   in contrast?

## Output format

Plain-text findings with file:line evidence for each question. State
plainly: WORKING (retry loop exists and is wired up), BROKEN (exists but
never called/never increments attempts), or MISSING (no such mechanism
exists at all). This determines whether the outbox needs a new retry loop
built from scratch or just needs its existing one connected properly.
