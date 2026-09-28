# D11 -- Ledger retains an unreachable peer address (`13.217.204.112:8080`, `failure_count: 6`)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: OPEN -- observation only, mechanism unconfirmed
Priority: MEDIUM -- a wrong address that is never evicted is a standing source of
wasted dial attempts; it is not by itself a message-delivery failure
Filed: 2026-09-26 (during the `bceacb94` rollout record, PR #389)
Node model note: applies to every node; the ledger is per-node state and the
stale entry will be present on whichever node learned it.

## What was observed

During the `bceacb94` rollout on 2026-09-26 the Windows node's peer ledger
listed `13.217.204.112:8080` for the Pixel peer with `failure_count: 6`, retained
alongside that peer's live addresses. The address is not reachable. This was
recorded in `HANDOFF/V040_3NODE_RCA_2026-09-09.md` as a new observation for that
pass, with no ticket behind it.

## Why it matters

A ledger entry that keeps a dead address is a dial target the seed sweep and the
reconnect path will keep attempting. `failure_count` is being maintained, so
something is counting failures -- but the entry is evidently not being evicted at
a threshold that reflects permanent unreachability. That is the defect shape:
**counting without converging**.

## What is NOT established

Stated explicitly so nobody reads more into this than was observed:

- **The mechanism is unconfirmed.** It is not established that this address ever
  belonged to a cloud node, that the port is wrong, or that the peer moved. A
  stale address is consistent with a former node IP, a NAT rebinding, a
  hand-entered bootstrap address, or a bad ledger merge. None of these has been
  tested.
- **It is not established that this address caused a delivery failure.** During
  the rollout window, delivery between Windows and the cloud node succeeded in
  both directions, and outbox returned to 0. The cost of this entry, if any, is
  wasted attempts -- not lost messages.
- **No eviction rule has been read.** The relevant ledger/dial-backoff code has
  not been inspected for this ticket, so the fix is not yet scoped.

## Evidence gap, stated plainly

The ledger entry was observed through the node's diagnostics during the rollout,
but the **raw capture was not preserved**. The rollout evidence directory
(`tmp/rollout-20260926-bceacb94/`) is gitignored and holds node stdout, not the
node's own `scm.log` or a ledger dump. Searching it for `13.217.204.112` and
`failure_count` returns zero matches. The only surviving record of this
observation is the RCA row itself. Treat the `failure_count: 6` figure as
reported-then-unverified.

## To confirm before acting

- Dump the Windows ledger and re-read the entry, capturing the raw output this
  time, including how many addresses the Pixel peer has and which one is used.
- Find the eviction rule for `failure_count` and determine why 6 failures did not
  remove the address.
- Check whether the same stale entry is present on the cloud node's ledger; if so
  it is a propagation question, not a local one.
- Only then decide between eviction, address refresh, and leaving it.
