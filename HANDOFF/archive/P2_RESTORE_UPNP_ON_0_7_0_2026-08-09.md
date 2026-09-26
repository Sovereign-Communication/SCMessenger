# P2 -- Restore UPnP port mapping once libp2p-upnp 0.7.0 is published

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Blocked on upstream release
Severity: P2 (optimization; NOT a v0.4.0 blocker)
Filed: 2026-08-09 (Windows/Claude lane, operator-requested)
Blocks: nothing. Deferred deliberately past v0.4.0.
Related: `HANDOFF/todo/P0_UPNP_PANIC_KILLS_DESKTOP_NODE_2026-08-08.md` (the removal)

## Why this ticket exists

PR #139 removed UPnP outright to stop a P0 that was killing every desktop node
(`libp2p-upnp-0.5.0/src/behaviour.rs:497:38: mapping should exist`). The removal
was the correct release action. The operator's position is that UPnP is still
wanted -- this ticket ensures the deletion is a deferral, not a silent drop.

## Do NOT restore on 0.6.0

The panic is fixed in **0.7.0**, not 0.6.0. Upstream changelog
(`protocols/upnp/CHANGELOG.md`):

> **0.7.0** -- "Fix panic with `mapping should exist` caused by conflating
> port-level mapping state with per-request in-flight tracking under a single
> `listener_id` key."

That is our exact crash, root cause and all. 0.6.0 contains only:

> - Change `Event::NewExternalAddr` / `Event::ExpiredExternalAddr` from tuple
>   variants to struct variants that include both local and external addresses.
> - Skip port mapping when an active port mapping is present.
> - Fix excessive retry attempts for failed port mappings by implementing
>   exponential backoff.

Restoring on 0.6.0 would reintroduce the same node-killing panic. Verified
2026-08-09.

## Upstream status

`crates.io` tops out at **0.6.0** (published 2025-10-27). **0.7.0 is not
published** -- it exists only in the `rust-libp2p` master changelog. This ticket
is blocked until it lands.

## Compatibility -- good news

0.5.0 and 0.6.0 declare identical requirements, and the workspace already
satisfies them, so 0.7.0 is expected to be a drop-in with **no libp2p bump**:

| Dependency | 0.5.0 / 0.6.0 requires | Workspace resolves |
|---|---|---|
| `libp2p-core` | `^0.43.1` | 0.43.2 |
| `libp2p-swarm` | `^0.47.0` | 0.47.1 |
| `igd-next` | `^0.16.1` | -- |

Re-verify 0.7.0's own requirements when it publishes; do not assume.

## Is it actually needed?

**It is a worthwhile optimization, not a requirement.** State this plainly so
nobody treats the restore as urgent:

- AutoNAT + DCUtR + circuit relay already cover NAT traversal, and the mesh runs
  correctly without UPnP.
- Many routers ship with UPnP disabled by default, for good security reasons.
- It is **useless under CGNAT** -- mapping a port on the local router does not
  help when the carrier NAT is upstream. CGNAT is known to be in play for this
  fleet; PR #139 contains dedicated CGNAT handling.

What it does buy, and why the operator wants it: a desktop node on a
UPnP-enabled non-CGNAT router becomes directly inbound-reachable, which makes it
a materially better relay for other peers, cuts latency, and reduces the mesh's
dependence on the AWS always-on node. That fits the sovereignty goal.

## Required shape when it comes back

**Behind a config flag, default off.** This is the durable lesson and it is not
about one bad version. `libp2p-upnp` has fixed a panic in 0.1.1, 0.2.1, 0.2.2,
0.5.0, and now 0.7.0. The structural fault is that a dependency's `expect()`
runs on the same task that owns message delivery, so an upstream panic takes the
whole mesh down. Opt-in confines the next occurrence to users who chose it
instead of every desktop node in a fleet run.

Note 0.2.1's entry -- "Fix a panic caused when dropping `upnp::Behaviour` such as
when used together with `Toggle`" -- so `Toggle` is a known-exercised pattern
with this crate and is the natural mechanism for the flag.

## Acceptance criteria

1. `libp2p-upnp` >= 0.7.0 published on crates.io and its dependency
   requirements re-verified against the workspace.
2. UPnP re-added behind a config flag that defaults to **off**.
3. A desktop soak with the flag **on** survives **more than 17 minutes** -- the
   longer of the two observed 0.5.0 failure uptimes (5m42s and 16m42s). See
   `docs/fieldtest/PR139_WINDOWS_LANE_CORRELATION_2026-08-09.md`.
4. A desktop soak with the flag **off** is unaffected.
5. Confirm an external address is actually mapped and added via
   `swarm.add_external_address()` on a UPnP-capable router -- otherwise the
   feature is cost without benefit.

## Current state of the removal (verified at `bfc7cac9`)

- No `upnp` references in `Cargo.toml`, `core/src/`, or `cli/src/`; the libp2p
  `"upnp"` feature is gone, not just the behaviour.
- `libp2p-upnp` does not appear in the build graph -- it is not compiled.
- A stale unused `libp2p-upnp 0.5.0` entry remains in `Cargo.lock`. Harmless
  (nothing references it) but worth pruning on the next unpinned resolve.
