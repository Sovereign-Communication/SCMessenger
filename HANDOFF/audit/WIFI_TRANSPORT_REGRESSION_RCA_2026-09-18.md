# RCA: Android-to-Windows WiFi transport regression (2026-09-17 22:16Z - 2026-09-18 06:42Z)

Status: RESOLVED (service restored 06:42:29Z); root cause identified; fix NOT yet implemented.
Severity at peak: Android could not reach the Windows node by any direct WiFi path for
~8.5 hours; 278 consecutive dial failures on the Pixel; node-side denies ~50-70/hour.
Attribution: NOT caused by PR #305 or the candidate binaries (evidence below).

## Symptom

Pixel (Lucas, `12D3KooWD776DQdWh6...`, WiFi `192.168.0.108/.134`) could not form a
swarm connection to the Windows node (`12D3KooWD6vZQrUq...`, `192.168.0.121`) over
WiFi. Pixel-side: every ~81 s "Successfully dialed ... via SwarmBridge", then 33 ms
later core notified `discovery` + `disconnected`, `peersDiscovered=0` from 04:24Z,
`TransportHealthMonitor: N consecutive failures` climbing to 278. Windows-side:
`Incoming connection negotiation aborted ... Listen error: Denied: connection denied`
after noise confirmed, ~50-70/hour, and outbound delivery to the Pixel failing
(`Delivery pass failed ... continuing cyclic retries`, attempt=533).

## Evidence chain (commands and numbers from this session)

- Onset precision: last healthy exchange `ping succeeded rtt=8.1966ms` at
  `22:15:58.684Z` (`scm.log.2026-09-17-22`, connection id 61); first deny
  `22:16:03.787Z` same file line 2968. Between them, at `22:16:03.487Z`, the node
  negotiated `/sc/registration/1.0.0` with the Pixel and the stream EOF'd at .495.
- Deny shape: `multistream_select` confirms `/noise`, THEN
  `Denied: connection denied` (libp2p-swarm 0.48 `ConnectionDenied`, Display impl at
  libp2p-swarm src/lib.rs:1753). The deny is emitted by a node-side behaviour, not by
  crypto or the wire.
- Deny is Pixel-specific: source census of every `connection denied` line 09-18
  (`grep -oE "from [^ ]+ -> [^:]+" | sort | uniq -c`): 89+77+35+... all from
  `192.168.0.108/.134` (the Pixel); zero from AWS (`18.234.62.247`), which stayed
  connected the whole time.
- Node ledger arithmetic (the leak):
  `new_established_connection{...D776...}` per hour file: 758 (00h), 297 (01h),
  0,0,0,0 (02h-06h). `Connection closed` events naming D776 in ANY file today: 0.
  Only 6 sporadic `Peer left: 12D3KooWD776...` lines across six hours.
- OS truth vs swarm truth: during the outage, `netstat`/`Get-NetTCPConnection`
  showed ZERO sockets between the node and the Pixel (the only Pixel-adjacent TCP
  pair belonged to adb.exe - wireless ADB, not the mesh), while the swarm still sent
  gossipsub RPCs to `D776` and refused its dials. The node held per-peer connection
  slots for connections that no longer existed at the OS level.
- Limit config: `core/src/transport/behaviour.rs:525-532` -
  `max_established_per_peer = 4` (plus 64 incoming / 128 outgoing). Four zombie
  slots = every fresh dial denied.
- Trigger: a mobile interface handover on the Pixel (WiFi IP `.134` -> `.108`
  between eras) killed its TCP sockets without FIN/RST. The libp2p swarm on the
  desktop never observed closure and never reaped the slots.
- Restart experiment (discriminating test, 06:42Z): `taskkill /PID 2932` +
  relaunch of the SAME binary (`tmp/radio-candidates/0ca9da53/scmessenger-cli.exe`,
  same args, same data dir, identity intact). Within 40 s of start:
  `Received PeerJoined: 12D3KooWD776... with 2 addresses` at `06:42:29Z`; a live TCP
  pair formed (`192.168.0.121:443 <-> 192.168.0.108:45407`); deny count since
  restart: 0; Pixel `peersDiscovered` 0 -> 2.
- End-to-end functionality after recovery: `POST /api/send` message
  `d609c1a5-29cc-4f14-a4cb-064b2a2d35bc` to the Pixel ->
  `[OK][OK] Delivered` on the node (06:44:27Z), receipt outbox cleared, and on the
  Pixel `Message from 30d0fa67...`, `delivery_attempt ... outcome=received
  transport=INTERNET`, notification raised (20:44:28 HST). Pixel's delivery receipt
  path back to the node also fired (`UNIFICATION message_relay ctx=receipt_send`).

## Why this is not PR #305 / the candidate binaries

1. Onset `2026-09-17T22:16:03Z` precedes the candidate swap (node restarted on the
   candidate at `00:21:47Z` and `00:37:44Z`) by ~2 hours.
2. The node build running at onset was `c2ce2f6` (main, PR #295 merge, started
   22:07:41Z). The previous build `1005da1` has an IDENTICAL source tree to
   `c2ce2f6` (GitHub compare: `files: []`, diverged, ahead 1) - so there is no
   source-level regression between last-good and first-bad at all.
3. The deny path is the swarm's own `connection_limits` behaviour, which PR #305
   does not touch (its diff is custody retention, relay admission accounting, key
   validation, and a diagnostics route).

## Open question for the fix (not yet answered)

Why did the swarm never reap the dead connections? Ping was active on healthy
connections earlier (`ping succeeded rtt=8ms`), and HANDOFF/CEO_STATE.md records a
prior policy of "immediate disconnect on ping failure". Either ping-failure
handling does not disconnect (changed or never wired on this path), or ping was
never negotiated on the zombie connections (noise established, ping stream never
opened, so no failure event can fire) and nothing else watches liveness. The fix
ticket must pin this down first.

## Fix directions (candidate, for a reviewed ticket)

1. Disconnect on `ping::Failure` for the affected peer (re-instate/verify the
   recorded policy) - targeted, cheap.
2. Periodic liveness sweep: if `swarm.is_connected(peer)` but no successful ping/
   traffic in N minutes, drop the peer's connections.
3. Make `max_established_per_peer` deny behavior observable: log WHICH limit and
   the current per-peer count at WARN when denying (today the operator sees only
   "connection denied" with no cause; this RCA cost an hour because of that).
4. Raising the cap is explicitly NOT a fix: it only widens the zombie window.

## Current fleet state after recovery

- Windows node: candidate `0ca9da53`, identity `985a25f9...`/`12D3KooWD6vZQrUq...`
  intact, PID 13828, restarted 06:42Z (this restart was the recovery mechanism;
  stdout capture `tmp/v040-live-verify-20260917/win-node-run2-deny-rca.out`).
- Pixel: app untouched, version 0.4.0 (lastUpdateTime 2026-09-16), discovery
  restored; received real traffic post-recovery.
- AWS: unaffected throughout; still on candidate `aab7e2a` image.

## Fix addendum (2026-09-18, post-RCA)

The fix is implemented and reviewed; the open question above is answered by the
field logs pulled during implementation:

- Ping-failure disconnect IS wired (`close_connection` on `ping::Failure`), but
  it is per-connection: on 09-17 `Ping failed` fired once for D776 (23:33Z) with
  FOUR connections established -- one ghost slot was freed, three persisted, and
  the per-peer cap stayed saturated. Per-connection liveness cannot see a peer's
  other dead sockets; that is the reap's reason to exist.
- Fix commits: `8cc356b8` (ZombieTracker: liveness stamps + deny-cause
  classification + periodic reap, native and wasm loops, 9 regression tests) and
  `44ed8071` (exact-IP join for peerless deny attribution, replacing a substring
  join that could misattribute a LAN neighbor's denied dial).
- Rule-8 gate: CLEAR, full 3-of-3 seat panel, one dissent (z2) overruled on code
  evidence -- verdict at `HANDOFF/review/RULE8_ZOMBIE_FIX_VERDICT_2026-09-18.md`.
- Deny observability is now real: a connection_limits deny logs WHICH limit and
  count (direction 3 above); unknown causes print verbatim, never bare "denied".
- Direction 4 stands: the cap was not raised.
