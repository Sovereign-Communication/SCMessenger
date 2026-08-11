# Ledger binds peer identities to addresses that are not theirs -- self, NAT-shared, loopback, Docker, VPC-internal

Status: Active
Severity: P1 systemic (best current explanation for "LAN nodes never detected")
Discovered: 2026-08-09, Windows lane, live soak at anchor `49bc3f56`
Gate: `core/src/transport/` -- MANDATORY adversarial review before any fix

## Summary

Four separately-reported symptoms are one defect: **the ledger treats
`ip:port` as a stable peer identity, and it is not.** Peer IDs are being bound
to addresses that provably cannot belong to them, including this node's own
address.

## Evidence, one 50-minute run

### 1. Identity churn on a shared NAT address -- 9 events on ONE address

`PeerID changed` fired 18 times. Distribution:

| Count | Address | What it is |
|---|---|---|
| 9 | `147.81.41.188:6891` | the household's **shared public NAT address** |
| 3 | `192.168.0.121:443` / `:8080` | **THIS NODE'S OWN LAN ADDRESS** |
| 2 | `54.226.67.101:9001` | the AWS relay |
| 2 | `192.168.0.111:443` | Android |
| 1 | `192.168.0.136:9090` | macOS |

Every node in this fleet sits behind one NAT, so they all egress from
`147.81.41.188`. The ledger sees one address whose PeerId keeps changing and
rewrites the binding each time. Behind a shared NAT, `ip:port` cannot identify
a peer -- but the ledger is treating it as if it can.

### 2. It bound another peer's identity to OUR OWN address

```
[WARNING] PeerID changed at 192.168.0.121:8080: 12D3KooWJUJ1koSW... (iOS)
[WARNING] PeerID changed at 192.168.0.121:443:  12D3KooWJUJ1koSW... (iOS)
```

`192.168.0.121` is this node's own LAN address (confirmed against
`/api/listeners`). The ledger recorded the iOS peer as living at our address.
Any dial to "iOS" using that entry targets ourselves.

### 3. Android does the same thing to the relay, 336 times

From the Pixel 6a's own log:

```
Dial error: Unexpected peer ID <Android's OWN peerid>
            at /ip6/::1/tcp/9001/p2p/<relay peerid>
```

Android holds a ledger entry claiming the AWS relay is at `::1` -- its own
loopback. It dials, reaches itself, and rejects on PeerId mismatch. 336
occurrences. **This is why Android is not connected to the relay**, which is
in turn why relayed probes to Android do not land.

### 4. Windows accepted a peer at a Docker bridge address

```
Connected to <macOS peer> via /ip4/172.17.0.1/tcp/443
             (promiscuous mode - any PeerID accepted)
```

Filed separately as `PROMISCUOUS_ACCEPT_UNROUTABLE_ADDR_2026-08-09.md`; it is
the same root disease. Plus 68 dial failures to private ranges carrying
`/p2p-circuit`, including the relay's VPC-internal `172.31.19.216`.

## Why this is likely THE explanation for the field report

The standing field complaint is "the other nodes on the same LAN were never
detected". With addresses misattributed like this:

- dials go to the wrong host (or to self) and fail,
- failures increment `failure_count` against the WRONG peer,
- entries hit `LEDGER_DEAD_FAILURE_THRESHOLD` and get marked dead,
- and `success_count` never rises -- which then hides the peer from the
  Android render path entirely
  (`ANDROID_LEDGER_VISIBILITY_ROOT_CAUSE_2026-08-09.md`).

That chain converts an address-attribution bug into a permanent
invisible-peer condition, which matches the symptom better than anything
proposed so far.

## MECHANISM CONFIRMED 2026-08-09 -- source-traced and orchestrator-verified

The five questions below were answered by a source trace; each claim was then
re-checked directly against the working tree. Results:

**1. Both ledgers are keyed by ADDRESS ALONE.** `peer_id` is a single
overwritable field on the row, not part of the key.
- `core/src/store/ledger_entry.rs:26-36` -- `LedgerEntry { multiaddr, peer_id:
  Option<String>, ... }`; lookup is a linear scan on `e.multiaddr ==
  multiaddr` (`:325`).
- `cli/src/ledger.rs:281-282` -- `entries: HashMap<String, LedgerEntry>`,
  documented "keyed by multiaddr (without /p2p/ suffix)".
- Therefore ONE address cannot hold two peer_ids. A second peer answering at
  the same address overwrites the first. This is the NAT-collision mechanism.

**2. `PeerID changed` warns, then overwrites unconditionally.**
`cli/src/ledger.rs:102-111` emits the warning; `:114` then does
`self.last_peer_id = Some(peer_id.to_string());` regardless.
`observed_peer_ids` (`:117-119`) keeps append-only history, but every consumer
reads `last_peer_id` -- so history is recorded and ignored.
**Worse: the core ledger does the same overwrite with NO warning at all**
(`core/src/store/ledger_entry.rs:326`, `entry.peer_id = Some(peer_id);`, and
again at `:715`). The core-side version of this bug is strictly more silent
than the CLI one, which is why it went unnoticed.

**3. `is_self_address` EXISTS but is wired to the wrong side of the pipeline.**
Defined at `core/src/transport/addr_filter.rs:721`. It gates what gets
**dialed**, never what gets **written**:
- Ingest paths call only `is_dialable_multiaddr`, never `is_self_address` --
  `cli/src/ledger.rs:419` and `:589`, `core/src/mobile_bridge.rs:1022-1026`
  and `:1123-1127`.
- Dial paths do call it -- `cli/src/ledger.rs:935-964`,
  `core/src/transport/swarm.rs:252-280`.

Consequence, and this is the important part: **a corrupted row is still
persisted and still served to other peers in ledger-exchange replies.** The
dial-time check only stops THIS node from acting on it; it does not stop the
bad binding from propagating across the fleet.

**4. Docker/VPC ranges are not special-cased.** `172.17.x` and `172.31.x` sit
inside `172.16.0.0/12`, so `Ipv4Addr::is_private()` treats them exactly like a
home LAN address. Under `NetworkMode::Local` (the default nearly everywhere)
they are dialable, recordable and discoverable. Loopback IS rejected
unconditionally (`addr_filter.rs:130-149` IPv4, `:297-323` IPv6, including
embedded-IPv4 forms) -- so Android's `::1` relay entry was NOT created through
`is_dialable_multiaddr`; it entered by another route, which is worth pinning
down.

**5. Promiscuous acceptance decides what is PERSISTED, not just what is
dialed.** `core/src/transport/swarm.rs:5151-5160`: on `ConnectionEstablished`,
if `endpoint.is_dialer()`, the handler calls `record_connection(ledger_addr,
peer_id)` using the live socket's remote address, with **no `is_self_address`
and no routability check at the call site**. The only gate is
`is_recordable_multiaddr`, which by documented design does not reject loopback
or RFC1918 ("an address a socket just came off demonstrably works for us").

That is the complete path by which `172.17.0.1` became a persisted address for
a remote peer.

**Non-finding worth recording:** the hypothesis that desktop ingest is weaker
than mobile ingest is FALSE. Both call the identical
`is_dialable_multiaddr(..., NetworkMode::Local, DnsPolicy::Reject)`. They
accumulate the same junk equally.

## What must be established (do NOT jump to a fix)

1. Where does the ledger key an entry -- by address, by `(peer_id, address)`,
   or by peer with an address list? Cite the code.
2. What does `PeerID changed` do on fire: rewrite the binding, or add? If it
   rewrites, two nodes behind one NAT will fight over one row indefinitely.
3. Should an address ever be recorded from a promiscuous accept?
4. Should this node's OWN listen addresses be excluded from every peer's
   candidate set? That check appears to be missing or not reached.
5. Does `addr_filter.rs` have a self-address / RFC1918-from-remote rule, and
   does it run on the ledger-ingest path?

## Falsification

If this is right, a node whose ledger is pruned of (a) its own addresses,
(b) any address learned from a remote peer that is in a private range not on
the local subnet, and (c) shared-NAT external addresses used as identity keys,
should show a sharp drop in failed dials and should start seeing LAN peers.
Test on one node before proposing a core change.

## Related tickets

- `PROMISCUOUS_ACCEPT_UNROUTABLE_ADDR_2026-08-09.md` (accept side)
- `ANDROID_LEDGER_VISIBILITY_ROOT_CAUSE_2026-08-09.md` (render consequence)
- `ANDROID_INBOUND_CRYPTOERROR_2026-08-09.md` (Android/relay disconnection)
