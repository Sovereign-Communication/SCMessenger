# Promiscuous mode accepted a peer identity at a non-routable Docker address

Status: Active
Severity: P2 security (needs adversarial review; not yet shown exploitable)
Discovered: 2026-08-09, Windows lane, live soak at anchor `49bc3f56`
Gate: `core/src/transport/` -- MANDATORY adversarial review before any fix
merges (AGENTS.md rule 8, ORCHESTRATION.md Section 4)

## Observation

```
2026-08-10T06:33:01.968891Z INFO scmessenger_core::transport::swarm:
  Connected to 12D3KooWNC5rEKFhuxDNDNsJ6Q58Ca75LnxfjUqspGzGRdYRUWyt
  via /ip4/172.17.0.1/tcp/443 (promiscuous mode - any PeerID accepted)
```

`172.17.0.1` is the default **Docker bridge gateway**. It is not routable
from this LAN and cannot be the macOS node's address. The node nevertheless
recorded a connection to that peer identity at that address, because
promiscuous mode accepts any PeerID that answers.

All four promiscuous accepts in the run, for contrast:

| Address | Assessment |
|---|---|
| `/ip4/54.226.67.101/tcp/9001` | AWS relay, legitimate |
| `/ip4/192.168.0.136/tcp/9002` | macOS on LAN, legitimate |
| `/ip4/192.168.0.136/tcp/443` | macOS on LAN, legitimate |
| `/ip4/172.17.0.1/tcp/443` | **not legitimate** -- Docker bridge |

## Related: the ledger is carrying addresses that should never be dialed here

Same run, 68 distinct dial failures to **private** ranges carrying a
`/p2p-circuit` suffix, including:

- `/ip4/172.17.0.1/tcp/9001/ws/p2p/<relay>/p2p-circuit` -- Docker bridge
- `/ip4/172.31.19.216/tcp/9002/ws/p2p/<relay>/p2p-circuit` -- the relay's own
  AWS VPC-internal address
- `/ip4/10.39.118.49/tcp/9090/ws/p2p/<peer>/p2p-circuit`

These are other hosts' internal addresses that propagated through ledger
exchange. They are unreachable from here by construction, so each is pure
dial churn -- and they are the raw material a promiscuous accept operates on.

Also observed, same class: this node advertised **itself** over mDNS at a
doubly-nested circuit address (`macOS -> relay -> us`), rejected as
`TxtRecordTooLong`. Nesting depth 2.

## Why this is filed rather than fixed

The combination worth reasoning about is: *addresses that peers can influence*
(ledger-propagated internal addresses) plus *an accept path that does not bind
identity to address* ("any PeerID accepted"). That is the shape of an identity
or routing confusion issue. It is NOT demonstrated exploitable here -- libp2p's
transport-level handshake still authenticates the peer's key, so a wrong host
answering at `172.17.0.1` should fail the handshake rather than impersonate.

What is confirmed is that the node **logged and recorded** a peer as connected
at an address that cannot be theirs, which at minimum corrupts the ledger's
address quality and feeds the dial churn above.

Do not let the "probably fine because libp2p authenticates" reasoning close
this without a proper look -- that is exactly the kind of assumption the
adversarial gate exists to test.

## Prior art in the backlog

`P0_UPNP_PANIC_KILLS_DESKTOP_NODE_2026-08-08.md` item 2 already flagged
"Promiscuous dialing accepts any PeerID -- security review warranted" as a
theoretical objection. This ticket supplies the concrete instance it lacked.

## Asks for whoever picks this up

1. Determine what "promiscuous mode" actually relaxes -- is it only the
   *dial* candidate filter, or does it also relax any *accept*-side identity
   binding? Cite the code, do not reason from the log wording.
2. Decide whether private/link-local/VPC-internal addresses learned from a
   REMOTE peer should ever enter the dial set. A same-subnet check exists in
   `core/src/transport/addr_filter.rs` -- establish whether it runs on this
   path.
3. Consider not persisting an address into the ledger when the connection
   that produced it came from a promiscuous accept at an unroutable address.

## Evidence location

`tmp/logs/win_node_soak_49bc3f56.log` (soak from 2026-08-10T06:32:58Z,
anchor `49bc3f56`).

**That path is gitignored and so is any `.log` copy** -- `.gitignore:10` is a
global `*.log`, so copying it into `docs/fieldtest/` does NOT preserve it
either (I tried; the copy was ignored too). The repo convention in
`docs/fieldtest/` is `.md` writeups, not raw logs.

Every log line this ticket depends on is therefore quoted inline above, which
is the durable record. If you need the full log and this machine has been
cleaned, it is gone -- re-run the soak rather than trusting a summary.
