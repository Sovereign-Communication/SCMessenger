# P0 -- mobile nodes have no configurable bootstrap, so they cannot rendezvous off-LAN

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Open
Filed: 2026-08-10 ~03:00Z (Windows lane)
Severity: P0 for real-world use. Two phones left the LAN today and at least one
of them had no way to reach any rendezvous point.

## What happened, concretely

The operator took an iPhone and an Android device off the LAN. Before they left,
the Windows lane was asked to make sure both had the AWS relay in their ledger so
they could cross-communicate while away.

- **Android left with zero knowledge of the relay.** Grepping the device's own
  logs for the relay IP and PeerId returned **only the orchestrator's own adb
  commands**. It had never seen `54.226.67.101` or
  `12D3KooWPJK6KgKsafefLWeGs4kVbj7wBnU67yKe88ni3FHZ3Hr2`.
- **iOS probably received it** via a ledger exchange that occurred after the
  desktop learned the relay, but that was never confirmed on-device and could
  not be after it left. Treat as unverified.

Once off-LAN, a node with no rendezvous cannot be reached at all. There is no
fallback: no configured bootstrap, no DHT seed, nothing.

## Root cause: three compounding gaps

### 1. The compiled default bootstrap list is EMPTY

`cli/src/bootstrap.rs`:

```rust
pub const DEFAULT_BOOTSTRAP_NODES: &[&str] = &[];
```

Nodes ship with no rendezvous at all. They only get one if something supplies it
at runtime or build time.

### 2. The desktop's only bootstrap entry was a dead loopback address

Observed across a nine-hour run:

```
Re-dialing bootstrap: /ip4/127.0.0.1/tcp/19001
Applied backoff to bootstrap addr /ip4/127.0.0.1/tcp/19001/p2p/12D3KooWSLkR...
```

Nothing serves that port. The AWS relay's IP appeared **once** in the entire log.
This is defect #4 from the 2026-08-08 field test, still live.

### 3. There is no runtime bootstrap override on mobile

`SC_BOOTSTRAP_NODES` (read via `std::env::var` in `cli/src/bootstrap.rs`) fixes
the desktop in seconds without a rebuild. **Android and iOS have no equivalent.**
An app process does not inherit a shell environment, and there is no in-app
setting for it. So mobile rendezvous depends entirely on ledger propagation
happening opportunistically before the device leaves -- which is exactly when it
is least likely to be verified.

## Why ledger propagation is not a sufficient answer

It is the right mechanism and it demonstrably works for address churn -- the
Pixel moved `192.168.0.111` -> `.107` -> `.131` and peers followed it. But as the
*only* path for acquiring a rendezvous it is fragile:

- It requires a live connection to a peer that already knows the relay, at a
  moment nobody is watching.
- A device that is dozing is unreachable inbound. The Pixel was running and
  listening on 80/443/8080/9001/9002/9090 (confirmed via `/proc/net/tcp`) yet
  every inbound probe timed out.
- Nothing verifies it landed. There is no "do I have a rendezvous?" check before
  a device goes mobile.

## Required

1. **Ship a real default bootstrap.** `DEFAULT_BOOTSTRAP_NODES` must contain at
   least one reachable relay for release builds. An empty default means every
   fresh install is isolated until it happens to meet a peer.
2. **A mobile-settable bootstrap.** An in-app setting, a build-time constant for
   mobile artifacts, or a provisioning QR. Any of the three; the current answer
   of "none" is not viable.
3. **A readiness check before going mobile.** Something answering "does this
   device know a reachable rendezvous?" -- surfaced in the app, not only in logs.
4. **Persist a learned relay durably** so it survives app restarts and ledger
   pruning. A relay learned once and then forgotten is worse than useless because
   it looks like coverage.

## The relay's correct address, for whoever implements this

```
/ip4/54.226.67.101/tcp/9001/ws/p2p/12D3KooWPJK6KgKsafefLWeGs4kVbj7wBnU67yKe88ni3FHZ3Hr2
```

**Note the `/ws`.** The relay is WebSocket-only on 9001 -- verified by probe
(`HTTP/1.1 101 Switching Protocols`), and a plain-TCP dial to the same port times
out. This is the OPPOSITE of the LAN transport contract, where plain TCP works
and `/ws` is false for every port except 9002. Two different conventions in one
fleet is its own hazard and should be reconciled.

Only 9001 and 9876 are externally reachable; the security group does not open
8080, 9090, 80 or 443.

## Immediate mitigation applied

The Windows node now runs with:

```
SC_BOOTSTRAP_NODES=/ip4/54.226.67.101/tcp/9001/ws/p2p/12D3KooWPJK6KgKsafefLWeGs4kVbj7wBnU67yKe88ni3FHZ3Hr2
```

and is connected to the relay, which identifies as
`scmessenger/0.4.0/headless/relay`. Windows and the relay are being left up so
either phone can re-sync the moment it reaches a network where it can see them.
