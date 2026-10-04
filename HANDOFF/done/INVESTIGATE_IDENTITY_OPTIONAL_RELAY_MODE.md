# INVESTIGATE: Does IronCore support an identity-optional "headless relay" node?

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: read-only investigation, no code changes.

## Context

Farm deployment (28-acre property, patchy cellular, 12 users) wants a
"headless routing backbone" node concept: a plain SCMessenger install (any
platform - Windows/Android/cloud Linux) that participates in mesh relay/
forwarding and peer discovery WITHOUT ever creating a cryptographic identity
or storing user data - just packet forwarding capacity. Before this becomes
a backlog priority, need to know whether the CURRENT codebase already
supports this or would need new work.

## What's already known

`core/src/iron_core.rs`: `IronCore::new()` (line ~267) is separate from
`initialize_identity()` (line ~556) - identity creation looks like an
explicit, distinct step rather than baked into construction. This SUGGESTS
(not confirmed) a no-identity IronCore instance might already be
constructible.

## Questions to answer

1. Can `IronCore::new()` be used, and can the transport/swarm layer
   (`core/src/transport/swarm.rs`) start, discover peers, and relay/forward
   messages for OTHER peers' traffic WITHOUT `initialize_identity()` ever
   being called? Trace what actually breaks (panics, returns Err, or
   silently no-ops) if identity-dependent code paths are hit with no
   identity present - check `core/src/identity/mod.rs` and wherever
   `IronCore` methods check for identity presence before performing
   swarm/relay operations.
2. Does the relay/custody code (`core/src/store/relay_custody.rs` or
   equivalent, and `core/src/relay/`) require an identity/signing key to
   accept, store, or forward custody messages on behalf of OTHER peers, or
   is relay/forwarding capacity identity-independent by design?
3. Is there already a CLI/config flag, UniFFI method, or Android setting to
   run in this "no identity, relay only" mode today? Check `cli/src/` for
   any existing `--relay-only`/`--headless` style flag, and
   `core/src/api.udl` for any UniFFI method that constructs IronCore without
   identity.
4. If NOT already supported, what's the smallest viable path (rough sketch,
   not implementation) to make it work: what identity-dependent assumptions
   would need to become optional?

## Output format

Plain-text findings answering each question with file:line evidence. State
clearly whether this is ALREADY WORKING, PARTIALLY WORKING (works but with
some rough edges/missing flag), or NOT SUPPORTED (would need real new work).
This determines whether "identity-optional relay backbone" goes into the
backlog as a small task or a bigger design item.
