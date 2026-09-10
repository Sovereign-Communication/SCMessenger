> **RESOLVED 2026-09-10 -- moved to done/ per the 2026-08-29 closure ruling, verified in source this session.**
> Evidence (this session, main@45ab59f9): `core/src/transport/multiport.rs:75-99`
> `generate_listen_addresses()` now emits exactly one transport per port
> (comment at :79 "exactly one transport per port"), dedupes via `seen_ports`
> HashSet (:77, :84), and no longer pushes a `/ws` multiaddr for the same port.
> The CTO_STATE 2026-08-29 entry had already ruled this fixed-in-code; this move
> aligns the ticket with that ruling. Ticket's own acceptance #3 (regression test
> for duplicate (family, port) entries) is partially met via dedup logic; the
> probe-table re-run (acceptance #4/#5) remains Tier-A rig work.


# P0 -- the node binds BOTH plain TCP and WebSocket to the same port

Status: Open -- root cause identified in code and confirmed at runtime
Filed: 2026-08-10 ~01:15Z (Windows lane)
Blocks: Windows<->macOS and Windows<->Android connectivity; the five-node run

## Symptom

Inbound connections reach the node over TCP and then fail the libp2p upgrade:

```
Incoming connection negotiation aborted from /ip4/192.168.0.136/tcp/57058
  -> /ip4/192.168.0.121/tcp/8080: Listen error: Failed to negotiate transport protocol(s)
```

Reported independently by two different peers -- a macOS CLI node and an Android
node -- so it is neither peer-specific nor mobile-specific. The other lane
described the Android case in the same words: "the pinned connection did not
complete and inbound negotiation aborted."

Aborted inbound negotiations in one 8-hour run, by local port:
`80` x16, `8080` x8, `443` x8, `9001` x7, `9090` x3.

## Root cause

`core/src/transport/multiport.rs`, `generate_listen_addresses()` (~line 65):

```rust
// Helper to add IPv4 and IPv6 addresses for a port (both TCP and WS)
let mut add_port = |port: u16| {
    ...
    let tcp_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", port)...;
    let ws_addr:  Multiaddr = format!("/ip4/0.0.0.0/tcp/{}/ws", port)...;
    addresses.push((tcp_addr, port));
    addresses.push((ws_addr, port));
```

**The same port number is used for both a plain TCP listener and a WebSocket
listener.** The comment states the intent explicitly, so this is by design rather
than an oversight -- but it does not work.

### Confirmed at runtime

`netstat -ano` for the node's PID, LISTENING sockets grouped by address:

```
2  0.0.0.0:9090      2  [::]:9090
2  0.0.0.0:9001      2  [::]:9001
2  0.0.0.0:8080      2  [::]:8080
2  0.0.0.0:80        2  [::]:80
2  0.0.0.0:443       2  [::]:443
```

Exactly **two** listening sockets per port, per family. On Windows these binds
succeed (no `SO_EXCLUSIVEADDRUSE`), and an inbound connection is delivered to
only ONE of the two sockets. If the dialer speaks plain libp2p TCP and the
connection lands on the WebSocket socket -- or the reverse -- the upgrade fails
with exactly the error above.

### Independent confirmation by probe

Connect, observe whether the listener speaks first, then attempt an RFC 6455
upgrade on a fresh connection:

| Port | On connect | WS upgrade | Effective transport |
|---|---|---|---|
| 9001 | silent | rejected, closed | plain TCP |
| 443  | silent | rejected, closed | plain TCP |
| 8080 | silent | rejected, closed | plain TCP |
| 80   | silent | rejected, closed | plain TCP |
| 9090 | silent | rejected, closed | plain TCP |
| 9002 | silent | `HTTP/1.1 101 Switching Protocols` | **WebSocket** |

So one socket per port effectively "wins" and serves all traffic, while the
other is advertised but unreachable. `/api/listeners` publishes BOTH forms, so
**half of the advertised addresses cannot be served by the socket that actually
accepts**. Every peer dialing from the advertised set has a coin-flip.

## Why this was mistaken for other defects

- Read as **stale ledger addresses**, because peers appeared to dial addresses
  that did not work. But address churn recovers correctly: the Pixel moved
  `192.168.0.111` -> `.107` and Windows reconnected to the same PeerId unaided.
- Read as an **Android mDNS defect**, because Android showed the same abort.
  It shows the same abort because it is the same bug on the listening side.
- Contributed to **P0 request-response** noise: failed upgrades produce repeated
  reconnection attempts, which feed the concurrent-connection storms.

## Fix direction (needs operator decision -- merge-blocked path)

`core/src/transport/` is security-gated under rule 8. Options:

- **(a) Dedicate ports per transport.** Plain TCP on the well-known set
  (80/443/8080/9001/9090), WebSocket on its own port(s) only -- 9002 already
  behaves this way. Smallest change, matches observed reality.
- **(b) Bind one transport per port and advertise only what bound.** Requires
  the advertisement to be derived from the actual successful bind rather than
  from the attempt list.
- **(c) Use a combined transport** that can serve both protocols on one socket
  by sniffing the first bytes. Largest change; verify libp2p supports it before
  choosing.

Whichever is chosen, the invariant is: **never advertise an address that the
accepting socket cannot serve.**

## Acceptance criteria

1. For every advertised listener address, a probe of that exact address
   completes the protocol it claims. No port advertises both forms unless a
   single socket genuinely serves both.
2. `netstat` shows exactly ONE listening socket per (address, port).
3. A regression test asserts `generate_listen_addresses()` never emits two
   entries for the same `(family, port)` with different transports.
4. Re-run the probe table above and confirm every row matches its advertisement.
5. Verify on Windows specifically. The duplicate bind succeeds there; on Linux
   it may fail loudly instead, which would hide the defect from CI.

## Interim workaround (no code change)

Dial the plain TCP form for 80/443/8080/9001/9090 and `/ws` only for 9002. This
unblocks the five-node run without touching merge-blocked code.
