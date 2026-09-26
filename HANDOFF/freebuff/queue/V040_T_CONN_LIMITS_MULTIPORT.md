# V040-T-CONN-04 — connection_limits cap refuses multi-port dials

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: OPEN (filed 2026-09-20 orchestrator; live defect)
Priority: P0 for field scoring (D4/D6 LAN paths)
Lane: Freebuff / DeepSeek V4 Flash (unmetered)
Scope: `core/src/transport/behaviour.rs` connection_limits policy,
`core/src/transport/swarm.rs` dial/admission bookkeeping if required for
per-peer port strategy. Do not redesign relay custody. Do not change identity
or crypto paths.

## The defect (live evidence)

2026-09-19 freebuff baseline inbox
`HANDOFF/freebuff/inbox/V040_3NODE_BASELINE_AND_UPDATE_PLAN_2026-09-19.md`
(read it whole; commands are named there):

- Windows node: `Inbound connection DENIED from /ip4/192.168.0.103/tcp/<ephemeral>
  -> /ip4/192.168.0.121/tcp/{80,443,8080,9001,9090,65204}: connection_limits:
  limit 4 reached` — 274 WARNs in one hour.
- Cloud node: same cap refusing the Windows node's dial to `:9001`.

Code anchor on `origin/main`:

```
core/src/transport/behaviour.rs
  connection_limits::ConnectionLimits::default()
      .with_max_established_outgoing(Some(128))
      .with_max_established_incoming(Some(64))
      .with_max_established_per_peer(Some(4))
```

The phone dials a multi-port set against each peer; only 4 concurrent
connections per peer are admitted; the rest are denied. That is the mechanism
behind delayed outbox drain reports, and it is unchanged by build bumps.

Related but distinct: zombie-connection reaping landed with the #305 train
(`RULE8_ZOMBIE_FIX_VERDICT_2026-09-18.md`). Do not revert that work. Residual
note in that verdict: `max_established_per_peer=4` still bounds sockets.

## Premise check

The cap **exists** and is **observed live**. The task is not "prove the logs"
— the task is to make multi-port dialing viable without reopening unbounded
connection storms (the reason the cap was tightened).

## Design constraints (implement this, do not re-derive)

1. Keep a **per-peer** bound (DoS control). The failure mode is "bound counts
   half-open / redundant port probes as established peers," not "any cap is
   wrong."
2. Preferred direction: distinguish **productive** connections (handshake
   completed + identify/receive traffic) from **probe/dial-in-progress**
   sockets, or collapse multi-port attempts so only the chosen path stays
   established.
3. Alternative acceptable: raise `max_established_per_peer` modestly **only if**
   accompanied by faster dead-socket close (already partially present via ping
   failure close) and a test that N simultaneous port probes do not retain N
   established slots after selection.
4. Deny logs must remain **naming the limit** (zombie-fix observability).
5. No change to custody admission or relay budget semantics from #305.

## Scope correction

- `RelayCustodyStore` / TRN-04 retention is **done** — do not "improve" it here.
- `relay_per_peer_budget` from #305 is **done** — do not conflate with
  `connection_limits`.
- Do not drive the Pixel. Evidence from logs only.
- Do not raise limits without a bound on retained sockets per peer.

## Acceptance

1. Unit/integration test: a peer dialing K ports concurrently (K>4) ends with
   at most the policy's retained established connections for that peer, and
   at least one productive path remains.
2. Live Tier A (Windows + AWS + optional phone logs): `connection_limits: limit
   4 reached` WARN rate collapses on a comparable dial burst vs the 2026-09-19
   baseline; mesh still healthy (`/health`, seed-dial cadence, delivery).
3. `cargo test -p scmessenger-core --lib` green on the change.
4. Rule-8 adversarial APPROVE on file before merge (transport directory).

## Review gate

**Rule-8 mandatory** — `core/src/transport/`.

## Rules

- No emojis. Evidence lines carry commands or UNVERIFIED.
- No `unwrap()` in production paths.
- Never read `$?` after a pipe; capture then test.
- Disk: binaries to `tmp/radio-<sha>/`, not `target/`.
- Shared checkout: only files in Scope.

## Unified path reference

`HANDOFF/V040_TAG_PATH_UNIFIED_2026-09-20.md` §3 F2 / §5 W2.
