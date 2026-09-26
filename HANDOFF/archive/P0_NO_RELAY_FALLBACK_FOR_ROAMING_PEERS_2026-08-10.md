# P0 -- a roaming peer is unreachable: the node never falls back to the relay

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Open -- observed LIVE during the iteration-2 roaming condition
Filed: 2026-08-10 ~04:25Z, anchor `68fcc3f1`
Severity: P0. This is the core product promise. A device that leaves WiFi
becomes unreachable, and the relay that exists to solve that is never tried.

## The condition

iOS left the home LAN and is roaming on cellular. Android stayed on home WiFi.
Windows and the AWS relay stayed up. This is the exact scenario the product must
handle: one participant goes out, the others stay put.

Windows is **connected to the AWS relay** at the time of every failure below.

## What the node actually did

Address attempts for the roaming iPhone (`12D3KooWJUJ1ko...`) in one run:

| Address attempted | Count | Why it cannot work |
|---|---|---|
| `/ip6/::1/tcp/9001` | **13** | **loopback -- this node dials ITSELF** |
| `/ip4/192.168.0.142/tcp/443` | 9 | stale LAN address; the peer left the LAN |
| `/ip4/192.168.0.142/tcp/80` | 7 | stale LAN address |
| `/ip6/2600:381:...` (carrier) | 14 | behind carrier NAT, not dialable inbound |
| `/ip6/fd74:...` (ULA) | 7 | link-local scope, not routable off-LAN |

**Relay circuit attempts: ZERO.**

The resulting error, verbatim:

```
Outgoing connection error to 12D3KooWJUJ1koSWwSEAX32z6SGaepikyqpJawpojoy6gvQ8k688:
  Dial error: Unexpected peer ID 12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw
  at /ip6/::1/tcp/9001/p2p/12D3KooWJUJ1koSWwSEAX32z6SGaepikyqpJawpojoy6gvQ8k688
```

Read that carefully: **while trying to reach the iPhone, the node connected to
itself over loopback and discovered its own PeerId at the far end.** It did this
thirteen times, more than any other candidate.

## Three distinct defects, compounding

1. **No relay fallback.** The node holds an active circuit-relay connection to
   the AWS relay and never attempts a circuit to the unreachable peer. The
   candidate ladder appears to contain only directly-learned addresses. If direct
   candidates are exhausted without a relay attempt, a roaming peer is simply
   unreachable -- the relay provides no benefit at the one moment it exists for.

2. **Loopback addresses are dialled for REMOTE peers.** `/ip6/::1/...` can never
   reach another device. It is the single most-attempted candidate here, and it
   resolves to this node. This is the self-dial defect, but worse than previously
   filed: it is not merely wasteful, it is **outcompeting** viable candidates in
   the ladder.

3. **Stale addresses are never reaped.** The iPhone's LAN addresses persisted and
   were retried long after it left. Combined with defect 1, the node exhausts
   dead candidates and stops, rather than escalating to the relay.

## Ordering matters as much as membership

Every previous connectivity failure this session was a **dial-ordering** problem,
not a missing-address problem: Android reachable at `.111` while the node retried
`.141`; macOS reachable on plain TCP while `/ws` variants were tried. Same shape
here. The relay path may well be present in the ledger and simply never reached
because loopback and stale LAN entries are ranked ahead of it.

## Acceptance criteria

1. When direct candidates for a peer are exhausted or failing, the node MUST
   attempt a circuit through a connected relay before declaring the peer
   unreachable. Prove it with a log line naming the relay circuit attempted.
2. Loopback (`::1`, `127.0.0.0/8`) and link-local/ULA addresses must never be
   dialled for a REMOTE peer. Note the directional nuance in
   `ITERATION_2_NAT_TRAVERSAL_TEST_2026-08-10.md`: this must not be implemented
   as a blanket external-address block, which would break NAT traversal.
3. Stale addresses must be demoted or reaped once a peer is known to have moved.
4. Regression test: peer known at address A, peer moves, relay connected --
   assert a relay circuit is attempted. It must fail today.
5. Instrument the ladder. Log the ordered candidate list per dial. Every failure
   this session was invisible until the ordering was dumped.

## Related

- `P0_DUAL_BIND_TCP_AND_WS_ON_SAME_PORT_2026-08-10.md` -- wrong transport form
- `P0_NO_MOBILE_BOOTSTRAP_MEANS_NO_OFF_LAN_RENDEZVOUS_2026-08-10.md` -- no rendezvous
- `P1_NESTED_CIRCUIT_ADDRESSES_STILL_FORMED_2026-08-10.md` -- address bloat
- `ITERATION_2_NAT_TRAVERSAL_TEST_2026-08-10.md` -- DCUtR is the only route to a
  direct connection since UPnP was removed, so relay fallback is not optional

## Still to capture

iOS-side logs when the device returns: whether iOS attempted to reach US via the
relay, and whether it recorded the shared external address `147.81.41.188` as
the plan anticipates. This ticket covers only the Windows half.

---

## CORRECTION 2026-08-10 ~04:30Z -- "ZERO relay attempts" WAS WRONG

**Retracting the central claim of this ticket as originally filed.** Circuit
addresses ARE attempted. My measurement was faulty, not the node.

### The measurement error

I counted candidate addresses with a regex that extracted only
`/ip[46]/<addr>/tcp/<port>[/ws]`. That pattern **stops at the port and discards
any `/p2p-circuit` suffix**, so every circuit address in the log was reduced to
its base address and became invisible as a circuit. I then reported zero.

Counting lines whose address actually contains `p2p-circuit`:

```
29  dial / candidate-ladder / connection-error lines for the roaming iPhone
    whose target address contains p2p-circuit
65  total p2p-circuit references for that peer
```

Example of a genuine circuit dial:

```
Dialing /ip6/2600:381:.../tcp/51258/p2p/12D3KooWJUJ1ko.../p2p-circuit
  (synthesizing port ladder if applicable)
```

So the node DOES construct and attempt circuit paths for the roaming peer.

### What is still true, and what changes

**Still true and still the defect:**
- Loopback `/ip6/::1/tcp/9001` was attempted 13 times for a REMOTE peer, more
  than any other candidate, and resolved to this node's own PeerId. That is real
  and remains a P0-grade ordering problem.
- Stale LAN addresses `192.168.0.142` were retried long after the peer left.
- The message did not reach the roaming peer.

**No longer claimed:** that relay fallback is absent. It exists and runs. The
defect is narrower and different in kind: the circuits being attempted are built
on **carrier IPv6 and stale base addresses** rather than on the AWS relay's
reachable address, and they inherit the same dead endpoints. A circuit through an
unreachable relay hop is no better than a direct dial to an unreachable address.

### Revised hypothesis for why the roaming peer is unreachable

Not "no relay attempted" but "**relay attempted through the wrong hop**". The
circuit candidates observed are anchored on peer-supplied carrier addresses, not
on the one relay we hold a live connection to. Whether a circuit through
`12D3KooWPJK6...` (AWS) is ever constructed needs a targeted check rather than an
inferred count -- I will not repeat the mistake of concluding absence from a
filter that could not have shown presence.

### Process note, recorded because it recurred

This is the **third** time in one session I concluded something was absent from
an extraction that could not have shown it: a truncated `/api/listeners` read
became "9001 is not advertised"; sync envelopes with empty bodies became "mobile
messaging is broken"; and now a port-terminated regex became "no relay attempts".

The repo's own rule names this exactly: *"a check that did not run looks
identical to a check that found nothing."* The countermeasure that would have
caught all three is the same -- before reporting an absence, confirm the
measurement is capable of showing the thing whose absence is claimed.
