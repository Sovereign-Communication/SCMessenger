# V040 3-node delivery trace -- open items (2026-09-19)

Date: 2026-09-19
Lane: Freebuff / DeepSeek V4 Flash
Trigger: operator report -- messages sent from the cell (Android) to the Windows
node at 07:30 and 07:33 local "haven't gone through".
Status: diagnosis only. No code changed by this note; every item below is an
observed log fact plus the question it raises.

## Evidence pulled, and what could not be pulled

- **Windows node: pulled.** `%LOCALAPPDATA%\scmessenger\logs\scm.log.2026-09-19-<HH>`,
  hourly files in **UTC**. Retained coverage today is hours **10 through 17** only.
  Local time is UTC-10, so 07:30/07:33 local = **17:30/17:33 UTC**.
- **Pixel: NOT pulled.** `adb devices` was empty at the time of the trace and the
  operator-supplied endpoint `192.168.0.134:41735` refused the connection
  (the wireless-debugging port rotates whenever Wi-Fi/toggling changes), with
  `adb mdns services` discovering nothing. So the phone half of this trace is
  missing and no item below is attributed beyond what the Windows node logged.
  A pull helper now exists at `tmp/pixel_pull.sh` (uncommitted, deliberately not
  under `scripts/device/` which belongs to staged PR #312).

## Node identities (established from this node's own config and log)

- This node's LAN address: `192.168.0.222`.
- Phone: libp2p `12D3KooWD776DQdWh6iHV8Qcnpj9jvTXhpJAgSsFPbMCaRtQpFmn`,
  nickname `Lucas`, app-level `sender_id=f83ab16319ca5b801f1c088935f2215c6aae9aa246b992f01c0f27f06b03cbe5`,
  LAN `192.168.0.113`.
- Cloud node: `12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31`, which is
  `%APPDATA%\SCMessenger\config.json`'s sole `bootstrap_nodes` entry
  (`/ip4/18.234.62.247/tcp/9001/p2p/12D3KooWGvCWJNo...`).

## What actually happened to the 07:30/07:33 messages

They were **not lost**. They arrived about 5-8 minutes late, in one burst:

```
17:30:04  last normal peer activity before the gap
17:31-17:37  ZERO inbound dial attempts from the phone (per-minute
             "Inbound connection DENIED" count goes to 0 for seven minutes)
17:38:08.906Z  phone reconnects; inbound conns on /ip4/192.168.0.113
17:38:08.952Z  [OK] Relay server: accepted reservation from 12D3KooWD776... — acting as relay for this peer
17:38:08.997Z  inbox_receive f2ddf411-e911-4320-8ff9-54fe1d420c88
17:38:09.020Z  inbox_receive b96826fe-a42f-4420-9f56-6ee0bd80c00d
17:38:09.402Z  inbox_receive faa231f6-69b6-4d12-81ea-c630bd7ba781
```

302 `inbox_receive` events from the phone arrived today across the retained hours.
So the mechanism is "phone stopped dialing, then flushed its outbox on reconnect".
**Why** the phone stopped dialing for seven minutes is not determinable from the
Windows log -- that needs the Pixel log.

## Open item 1 -- seven-minute dial blackout while the per-peer cap held 4 slots

Every denial in the hour carries the same reason, and it is a **deliberate bound**,
not a defect in itself:

```
WARN scmessenger_core::transport::swarm: Inbound connection DENIED from
  /ip4/192.168.0.113/tcp/<port> -> /ip4/192.168.0.222/tcp/<port>:
  connection_limits: limit 4 reached
```

`core/src/transport/behaviour.rs:524-535` sets `with_max_established_per_peer(Some(4))`,
already annotated "Keep direct path, relay path, and headroom for mobile interface
handover (Wi-Fi to Cellular transition) before dead sockets time out".

Scale: **220** distinct source/target port pairs were refused in the 17:00 hour,
against **3** `Connected to 12D3KooWD776...` and **2** accepted relay reservations.
The phone dials a 5-port ladder repeatedly, so once it holds 4 slots every further
dial is refused.

**The open question**: if those 4 slots are ever held by sockets that are already
dead (the exact case the code comment anticipates), the phone cannot re-establish
until they time out -- which would produce precisely this seven-minute blackout.
Confirming it needs the phone-side dial/backoff log. Any change here is
`core/src/transport` (**rule 8** gated) and needs adversarial review; it was
deliberately NOT changed.

## Open item 2 -- StoreAndCarry routing decision at confidence 0.0 on the same LAN

```
17:38:08.999895Z INFO scmessenger_core::routing::optimized_engine:
  event="routing_decision" message_id=309caed2b5c04679a3db999d6c29e2a3
  recipient_hint=9ff96f0aea838ebe priority=128
  next_hop=RouteDiscovery { hint: [159, 249, 111, 10, 234, 131, 142, 190] }
  decided_by=StoreAndCarry confidence=0.0
```

The routing engine had no route and fell back to store-and-carry with confidence
`0.0` -- at a moment when the phone was connected to this node over the same LAN
(`192.168.0.113` <-> `192.168.0.222`) and had just exchanged ledger state. Worth a
dedicated look for 3-node testing, because "no route" while the peer is directly
connected is the path that turns a direct LAN send into a store-and-forward wait.

## Open item 3 -- one burst, one auto-reply, a double suppression, and a silent third

Three messages arrived within 405 ms; the auto-reply accounting is asymmetric:

```
17:38:08.997  inbox_receive f2ddf411
17:38:09.019  auto_reply_ack_queued           in_reply_to=f2ddf411
17:38:09.020  inbox_receive b96826fe
17:38:09.022  auto_reply_suppressed_rate_limit in_reply_to=b96826fe
17:38:09.040  auto_reply_suppressed_rate_limit in_reply_to=b96826fe   <- logged twice
17:38:09.402  inbox_receive faa231f6
              (no auto_reply_ack_queued and no auto_reply_suppressed_rate_limit at all)
```

Today totals: `auto_reply_ack_queued` 3, `auto_reply_suppressed_rate_limit` 2.
The 1-per-minute cap is intentional (`AUTO-REPLY-RATE-001`, operator directive), so
suppression is correct behaviour. Two things are not explained by the cap: the
**duplicate** suppression row for the same `in_reply_to`, and `faa231f6` producing
**no event of either kind**. A message that is neither answered nor recorded as
suppressed is indistinguishable from one that was dropped.

Consequence for the operator experience: a burst of three messages that arrives
late yields exactly one reply, which reads as "my messages were not actioned" --
the same misreading the missing phone log would otherwise have caused.

## Open item 4 -- 167 custody accepts addressed to the cloud node, with no logged direct connection to it

```
17:03:49.147Z Accepted custody 12D3KooWGvCWJNo...-1789837429859-...-1420 for
              offline destination 12D3KooWGvCWJNo... (relay message ...)
```

Today: **167** custody records accepted whose destination is the cloud node and
whose log line says `offline destination`. For the same peer this node logged
**3** `Peer left` events (16:32:59, 17:38:38, 17:46:19) and **0** `Connected to`
and **0** `Identified peer` lines.

Fact stated precisely: the retained logs contain no direct-connection line for the
cloud node, while they do record it leaving three times and record 167 messages
taken into store-and-forward custody for it. Whether the cloud link is genuinely
down, or up but not logged the same way, is **unresolved** -- and it matters,
because every one of those 167 messages waits on that link.

## What would close these

1. One pull with the phone attached (`bash tmp/pixel_pull.sh`), giving the dial/
   backoff timeline for 17:31-17:37 and the app-side view of the three sends.
2. The cloud node's own log for the same window, to settle item 4 from the other end.
3. Item 1 needs adversarial review before any transport change (rule 8).
