# Coordination fallback protocol -- do not lose the ability to talk

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Written: 2026-08-10 ~00:45Z
Owner: Windows lane (primary orchestrator while GPT-MAC is intermittent)

## The honest position

**SCMessenger CLI is NOT yet usable for cross-lane coordination.** No message
has ever been delivered between the Windows and macOS nodes. Do not plan around
it working.

What works today, and what does not:

| Capability | State | Evidence |
|---|---|---|
| Node stays alive | **Working** on `d48558a8` | 8h33m, zero panics (was 62s on `ebd723ab`) |
| Nodes discover each other | **Partly** | macOS peer held ~70 min, 16:05-17:18Z; currently 0 peers |
| TCP reachability between lanes | **Working** | macOS dials land on our `:8080` |
| libp2p protocol upgrade | **BROKEN** | `Failed to negotiate transport protocol(s)` |
| Message delivery | **Never achieved** | macOS message stuck in its outbox |

So the ordering is: transport reachability is solved, the protocol upgrade is
not, and delivery has never been demonstrated.

## Channel hierarchy -- use in this order

1. **GitHub PR #139 comments. PRIMARY.** This is the only channel that has ever
   worked between the lanes. It is durable, timestamped, readable by every
   agent and by the operator, and it survives every node crash. Every
   substantive decision, candidate SHA, and piece of evidence goes here
   regardless of what else is working.
2. **Files committed to `main`.** For anything longer than a comment: plans,
   tickets, evidence dumps. Survives sessions and model handoffs. This is what
   makes orchestration resumable by any model.
3. **SCMessenger CLI messaging.** ASPIRATIONAL. When it works it becomes the
   fast path, but it is never the only path. **Never** move a coordination
   decision onto the CLI until a receipt-backed round trip has been
   demonstrated between the two lanes on the same anchor.

Rule: if a decision exists only in a CLI message, it does not exist.

## Known-good fallback assets

Kept under `tmp/known_good/` (gitignored, local to the Windows host):

- `scmessenger-cli-d48558a8.exe` -- the binary with 8h33m of proven uptime.
  27,326,464 bytes. If a newer build regresses, **stop it and run this one**
  rather than debugging a broken node while blind.
- `identity_backup_<timestamp>/` -- `peers.json`, `relay_custody`, and the
  non-database parts of the data directory.

### Gap in the backup, and how to close it

`storage/db` and `outbox/db` are sled databases and are **locked while the node
runs**, so the current backup does NOT contain the identity keypair. A cold copy
is required.

**Take it at the next planned restart**, before starting the new binary:

```bash
# with the node STOPPED
cp -r "$LOCALAPPDATA/scmessenger/storage" "$LOCALAPPDATA/scmessenger/outbox" \
      "$LOCALAPPDATA/scmessenger/relay_custody" "$LOCALAPPDATA/scmessenger/peers.json" \
      <backup dir>/
```

Why this matters: the Windows PeerId
`12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw` is recorded in every
other node's ledger. Losing it does not just cost us our identity -- it adds a
permanent ghost entry to every peer on the fleet, which is the same class of
defect we spent today reducing. **Never wipe the data directory to "get a clean
start".**

## Restart / rollback runbook

1. Note the current uptime and copy the stderr log aside -- an uptime figure is
   evidence and is lost on restart.
2. Stop the node (`taskkill //PID <pid> //F`).
3. Take the cold backup above.
4. Start the new binary with stdout and stderr to **separate** files. The
   request-response panic never appears in the rolling `scm.log`; if stderr is
   not captured separately the root cause is lost.
5. Verify within 2 minutes: `/api/identity` PeerId is unchanged,
   `/api/listeners` is populated, `/api/peers` starts filling.
6. If the PeerId changed, or the node dies within 5 minutes: **roll back to
   `scmessenger-cli-d48558a8.exe`** and report. Do not iterate forward on a
   node you cannot keep alive.

## Batching discipline

Fixes are batched per anchor, not applied one at a time to a live node:

1. Land fixes on a branch, gate them in CI (shared runners, not this host).
2. Merge to the PR branch so both lanes anchor on one SHA.
3. Both lanes rebuild, restart, and post provenance of the RUNNING process.
4. Only then gather runtime evidence.

The failure mode this avoids was demonstrated today: the anchor moved eight
times in three hours, CI runs cancelled each other, and neither lane could
finish a build-plus-verify cycle before the target moved. A build here takes
~5 minutes and the full matrix considerably longer; the cadence has to leave
room for that.

## Current open transport defects, in priority order

1. **Inbound negotiation aborts.** `Failed to negotiate transport protocol(s)`
   on advertised ports. Blocks Windows<->macOS AND Windows<->Android, so it is
   not mobile-specific. Suspected websocket vs plain-TCP mismatch: the listener
   set mixes `/tcp/N` and `/tcp/N/ws` across ports. **Top blocker.**
2. **Port 9001 bound but not advertised.** `netstat` shows the process
   `LISTENING` on `0.0.0.0:9001`; `/api/listeners` contains no `:9001` entry.
   Peers holding older ledger entries dial a port the swarm does not advertise.
3. **Self-dialing persists.** `Local peer ID at /ip6/::1/tcp/9001` -- the node
   connects to itself over loopback. Reduced from earlier sweeps, not gone.
4. **P0 request-response assertion.** Did not reproduce in 8h33m on
   `d48558a8`. Not declared fixed -- one sample, and a prior build also survived
   40+ minutes before dying. The dial-dedup fix in PR #144 removes the trigger;
   the upstream drift remains.
