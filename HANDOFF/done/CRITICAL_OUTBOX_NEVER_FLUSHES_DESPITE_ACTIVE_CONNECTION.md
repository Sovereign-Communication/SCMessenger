# TASK [CRITICAL]: Outbox-queued messages never get delivered even with an active direct connection

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO. Found 2026-07-12 during the first-ever successful live
Windows-CLI<->Android-emulator connection test this session. This is
arguably the most important finding of the whole session - it's the actual
core promise of a messenger app (message gets delivered) failing silently.

## What was proven working (real progress, do not re-litigate)

- Windows CLI successfully dialed and connected to the Android emulator app
  via `adb forward tcp:19001 tcp:9001` + a manually-configured
  `bootstrap_nodes` entry in `%APPDATA%\scmessenger\config.json` (NOTE:
  `SC_BOOTSTRAP_NODES` env var did NOT work for the CLI's `start` command -
  see separate note below). Log: `Connected to
  12D3KooWSLkR1yNngFGG7mheNM4wbQYRRo4D9599Rwga1gvcVfY7 via
  /ip4/127.0.0.1/tcp/19001 (promiscuous mode)`.
- Android app's own stats confirmed the SAME thing independently:
  `peersDiscovered` went from 0 (stuck the entire rest of this session,
  `Bootstrap all-failed (consecutive=106+)`) to 1, and STAYED at 1 for the
  full ~11 hour connection lifetime observed. This is the first confirmed
  successful transport-layer connection between these two nodes all session.
- Contact was added correctly via the daemon's own `/api/contacts` endpoint
  (`curl -X POST http://127.0.0.1:9876/api/contacts -d
  '{"peer_id":"12D3KooWSLkR1yNngFGG7mheNM4wbQYRRo4D9599Rwga1gvcVfY7",
  "public_key":"f583919e981275d4a9ca669987592d88fb03519630dad7068969f7b1dd651cd4",
  "name":"android-emu-v2"}'`), and `/api/send` returned `{"success":true}`.

## The bug

Despite the message API reporting success, and despite a continuously
active, stable direct connection to the exact recipient for the ENTIRE
observation window (~11 hours), the message never actually left the
sender's outbox:

```
routing_decision message_id=d596a2d9... recipient_hint=f1e95a16 priority=128
  next_hop=RouteDiscovery { hint: [241, 233, 90, 22] } decided_by=StoreAndCarry
  confidence=0.0
outbox_enqueue message_id=d596a2d9-3dfe-49b1-9864-845479c5d649
  recipient_id=f583919e981275d4a9ca669987592d88fb03519630dad7068969f7b1dd651cd4
  queued_at=1783850494996 attempts=0 payload_size=464
```

11 hours later, `grep` for that message_id in the daemon's log still shows
ONLY these two lines - no retry, no delivery attempt, `attempts` never
incremented past 0. Android's own stats confirm zero bytes ever arrived:
`ServiceStats(peersDiscovered=1, messagesRelayed=0, bytesTransferred=0,
uptimeSecs=41577)` at the same time.

**The routing engine decided `StoreAndCarry` with `confidence=0.0` for a
recipient it had an active, healthy, direct libp2p connection to the entire
time.** This strongly suggests the routing/next-hop decision logic
(`core/src/routing/optimized_engine.rs` per the log's module path) is not
correctly correlating "recipient_id" (a canonical/public-key-hex identifier)
with "the peer_id of an already-connected swarm peer" - it should have
recognized the direct connection and delivered immediately, or at minimum
retried against the known-active connection instead of leaving the message
in the outbox indefinitely with zero attempts.

## Root cause - traced 2026-07-12, precise, two distinct gap sites

Both confirmed by direct source reading (not speculation), same underlying
pattern: the "mycorrhizal" mesh-routing engine (`route_message_optimized`,
`core/src/routing/optimized_engine.rs:70`) only ever receives a 4-byte
blake3 `recipient_hint` - it has NO way to know "the caller already has a
`libp2p::PeerId` for this recipient AND the swarm may already be directly
connected to it right now." Its layers (negative cache -> prefetch cache ->
multipath -> base engine hierarchical discovery) are all designed for
*indirect*/relay routing when you genuinely don't know the path - there is
no "check `swarm.is_connected(&peer_id)` first" bypass anywhere before
falling through to these hint-only layers, even though `NextHop::Direct
{ peer_id, transport }` already exists as a variant
(`core/src/routing/engine.rs:19`) and `swarm.is_connected()` is already used
elsewhere in `swarm.rs` (e.g. lines 1190, 3125) - the pieces to fix this
exist, they're just not wired together at the send call sites.

**Site 1 - `core/src/iron_core.rs::prepare_message_internal` (~line 693-741):**
calls `self.make_routing_decision(hint, ...)` to decide `outbox.enqueue()`
vs `drift_store.insert()` (StoreAndCarry -> drift custody, per the
`handoff_to_drift` bool at line 707-712). This function is called by
`prepare_message_with_id`, which is the FIRST thing `handle_send_message`
(`cli/src/api_axum.rs:33`) calls, before `swarm_handle.send_message`. This
queuing itself may be intentional (durability - always persist before
attempting live send), so this site alone may not be the bug; needs
confirming whether it's the `outbox_enqueue` I observed or a red herring
from a different call (`prepare_onion_message`'s relay-selection path at
line 681 also touches routing).

**Site 2 - `core/src/transport/swarm.rs`, `SwarmCommand::SendMessage` handler
(~line 4177-4266, the non-wasm variant):** calls `route_message_optimized`
with only a hint derived from the ALREADY-KNOWN `peer_id` parameter, then
converts the decision to routes via `routing_decision_to_ranked_routes`
(line 847) and dispatches via `dispatch_ranked_route` (line 792). Traced
`dispatch_ranked_route` fully: for a single-hop path (true for BOTH
`NextHop::Direct` and `NextHop::StoreAndCarry`, since
`routing_decision_to_ranked_routes` builds `path: vec![*target_peer]` for
both - see lines 887-893 and 943-952) it unconditionally calls
`swarm.behaviour_mut().messaging.send_request(&target_peer, ...)` - so a
StoreAndCarry decision does NOT skip the network attempt at this layer.
**Important contrast:** the `#[cfg(target_arch = "wasm32")]` variant of this
same handler (~line 4268) explicitly comments "WASM: Simple direct send
without complex routing" and skips the whole routing-engine detour entirely
- proving the "just send directly when you already have a peer_id" pattern
is an intentional, working design elsewhere in this codebase; it's simply
missing from the native (non-wasm) path.

**Still unresolved:** given Site 2's `send_request` call is unconditional,
the actual network attempt may have genuinely fired - the persistent
`attempts=0`/`bytesTransferred=0` result over 11 hours could mean the
`send_request` call itself failed/errored silently (not logged), OR the
reply-channel bookkeeping (`pending_messages` map, storing `reply_tx`
for later resolution - line 4254) never got resolved and the request
response never arrived/was never processed, OR Site 1's queuing happened
but Site 2 (`swarm_handle.send_message`) was never actually reached/awaited
for this specific message. This needs live instrumentation (temporary
`eprintln!`/`tracing` at each site, similar to how the PQC-07 cadence bug
was empirically diagnosed this session) rather than further static reading
to pin down definitively - static tracing has identified the architectural
gap (no connection-aware routing bypass) but not yet the EXACT reason THIS
message specifically never got attempted or never got acknowledged.

## Update 2026-07-12: Site 2 fix applied; a THIRD, separate gap confirmed

**Site 2 fix APPLIED** (`core/src/transport/swarm.rs`, native
`SwarmCommand::SendMessage` handler): added an early
`swarm.is_connected(&peer_id)` check that constructs
`RoutingDecision { primary: NextHop::Direct { peer_id: peer_id_bytes,
transport: RoutingTransportType::TCP }, decided_by: RoutingLayer::Local,
confidence: 1.0, .. }` directly when true, bypassing
`route_message_optimized` entirely (matching the wasm32 variant's existing
"simple direct send" pattern). Compile/test verification in progress; live
re-verification (repeat the CLI<->Android test, confirm bytesTransferred>0)
still required before this is DONE - do not mark closed on compile-pass alone.

**Site 3, confirmed via dedicated investigation
(`INVESTIGATE_OUTBOX_RETRY_MECHANISM.md`, done/): the outbox has NO
retry/flush mechanism at all - not broken, MISSING.** `core/src/store/outbox.rs`
has `enqueue`/`drain_for_peer`/`peek_for_peer`/`record_attempt`/`remove` as
pure data-management primitives, but nothing anywhere in the codebase calls
`record_attempt` or periodically scans the queue to attempt (re)delivery.
This is independent of the Site 2 fix and matters for the genuinely-offline
case: Site 2's fix only helps messages sent while ALREADY connected - a
message queued while the recipient is truly offline would sit forever with
`attempts=0` even after the recipient reconnects, since nothing ever comes
back to try again. **A periodic outbox-flush loop needs to be built from
scratch** (e.g. in the swarm event loop, on a timer and/or on peer-connected
events: scan `outbox` for entries whose recipient just connected, call
`drain_for_peer`, attempt delivery via the same dispatch path as Site 2,
`record_attempt`/`remove` on success or failure).

## Update 2026-07-12: adversarial review + live re-verification of Site 2

**Adversarial review verdict (crypto-security-auditor): NEEDS FIXES, but the
diff itself is safe to keep.** No race condition (single-threaded swarm event
loop, no `.await` between the `is_connected` check and dispatch), no
reputation/rate-limit/abuse-guardrail bypass (those gates only exist on the
inbound relay path, never touched by outbound `SendMessage`), hardcoded
`RoutingTransportType::TCP` and `confidence: 1.0` are both cosmetic (proven by
tracing `routing_decision_to_ranked_routes` - the embedded `peer_id`/transport
fields are discarded in favor of the real `target_peer` parameter). Two real
gaps flagged: (B) no regression test exercises the `is_connected==true` bypass
branch - needs one before this is considered done; (C) this diff alone does
**not** close this ticket - Site 1 (`iron_core.rs`) and Site 3 (outbox
flush/retry) remain untouched, confirmed by the reviewer independently
re-deriving the same three-site breakdown this doc already had.

**Live re-verification: Site 2 fix CONFIRMED WORKING.** Rebuilt the CLI with
the fix, restarted the daemon against the same live Android emulator
connection used for the original 11-hour-stuck repro. Result:
```
ROUTE_DECISION ... route=direct ... policy_reason=DIRECT_FROM_ROUTING_ENGINE ... relay_score=1.000
[OK] Message delivered successfully to 12D3KooWSLkR1yNngFGG7mheNM4wbQYRRo4D9599Rwga1gvcVfY7 (5-11ms)
```
Multiple messages delivered in single-digit milliseconds immediately upon
reconnection, versus the old behavior (`StoreAndCarry`/`confidence=0.0`,
never delivered for 11 hours). This confirms Site 2 does what it was meant
to do for the direction it covers (CLI-as-sender to an already-connected
peer). See the newly filed
`CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md` for a separate,
Android-side bug found via the same live session (Android-as-sender message
delivered successfully but self-reported as failed due to a missing
receipt/ack path) - unrelated to Site 2/3 but discovered in the same
investigation.

## What's left to do

1. Finish compile/test verification of the Site 2 fix (in progress).
2. Mandatory adversarial review of the Site 2 diff (`core/src/transport/`).
3. Live re-verification: repeat the Windows-CLI<->Android-emulator test,
   confirm a message sent to an ALREADY-connected peer now actually arrives
   (bytesTransferred>0, visible in recipient history) - this validates Site
   2 specifically, not Site 3.
4. Separately, design and implement the Site 3 outbox-flush loop (new work,
   not a wiring fix - track as its own follow-up once Site 2 is confirmed
   live, to keep this ticket's scope reviewable in one pass). Consider
   whether it belongs in `core/src/transport/swarm.rs` (has peer-connected
   events already) or `core/src/store/outbox.rs` itself with an external
   trigger.
5. Still open: whether Site 1's (`iron_core.rs::prepare_message_internal`)
   outbox/drift-store branching needs its own connection-aware check, or
   whether it's fine as pure durability-queuing now that Site 2 delivers
   immediately when connected and Site 3 (once built) handles the
   reconnect-later case.

## Separate, smaller finding worth folding into cleanup

`SC_BOOTSTRAP_NODES` environment variable (read by
`core/src/transport/bootstrap.rs::resolve_env_bootstrap_nodes`) had NO
effect on the CLI's `start` command - only editing `bootstrap_nodes` in
`~/AppData/Roaming/scmessenger/config.json` directly worked. Worth checking
whether `cmd_start` in `cli/src/main.rs` actually wires `SC_BOOTSTRAP_NODES`
through to whatever consumes `BootstrapManager`, or whether the CLI's own
config-file bootstrap list is a completely separate code path from
`BootstrapManager`/`resolve_env_bootstrap_nodes` (the "promiscuous ledger
dialing" behavior observed in CLI logs looks like a third, distinct
mechanism from both).

## Gate

This is the actual core value proposition of the app (message delivery) -
treat as release-blocking for v1.0.0. Mandatory adversarial review once
fixed (`core/src/routing/`). Verification: repeat this exact live-device
test (Windows CLI <-> Android emulator via adb forward + bootstrap config)
and confirm a sent message is actually received (bytesTransferred > 0 on the
recipient, message visible in the recipient's history) within a reasonable
time, not just accepted by the API.

## CLOSURE 2026-07-17 (native audit session)

Site 2 fixed+live-verified via commit ecafd504 (route-bypass for already-
connected peers). Site 3 fixed via commit 6d884f97: swarm.rs PeerIdentified
handler calls handle_peer_connection_event -> Outbox::flush_peer_messages with
exponential backoff re-enqueue (iron_core.rs:2553-2591). Site 1 judged pure
durability queuing once 2/3 landed. Counterpart evidence:
HANDOFF/done/OUTBOX_FLUSH_ON_CONNECT_RETRY.md and
done/[VALIDATED]_P1_PLATFORM_001_Outbox_Flush_PeerDiscovered.md.
[x] File moved to done/
