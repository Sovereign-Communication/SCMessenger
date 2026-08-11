# PR 139 orchestration state — continue from here

Updated: 2026-08-10 05:29 UTC
Pull request: https://github.com/Sovereign-Communication/SCMessenger/pull/139

## Resume instruction

For a fresh session, say: `continue to /orchestrate`.

Read this file, then read the latest PR 139 comments before taking action. Use
SCMessenger CLI for live intermediate coordination when transport is healthy,
but mirror every confirmed result to PR 139. A CLI `accepted` response is never
delivery evidence. A handoff is confirmed only by receiver-side
`inbox_receive` plus a delivery ACK with the exact message ID.

## Current gate state

The one-hour merge gate is CLOSED. Do not merge until all five nodes are on the
same selected head, all identities are stable for the run window, both message
directions have receiver-side inbox/ACK evidence, direct versus relay/hole-punch
paths are identified, there is no swarm panic/echo/retry loop, and the system is
continuously fully functional for one hour.

If any of those conditions fail, record the failure, fix or re-anchor, and start
the one-hour clock again. CI green, a running process, or a locally accepted
send is not physical five-node parity.

## Code and branch anchors

- PR 139 was re-anchored at `68fcc3f19124feea915de9603c5438b53e7e9c39`.
- Candidate branch: `codex/pr139-five-node-gate-fixes`.
- PR remote head is currently `c6420b3a` (`docs: record candidate dial backoff`). The candidate runtime code head is `acda09df`; commits after it are documentation-only.
- `4083e59b` is the runtime-gate candidate immediately before `acda09df`.
- The Mac candidate was built and started from the isolated candidate worktree
  with the existing persistent data. Its current process identity is:
  - PeerId: `12D3KooWNC5rEKFhuxDNDNsJ6Q58Ca75LnxfjUqspGzGRdYRUWyt`
  - identity ID: `3854e44295c1384854b89312e5c3925f8431b6f4c41ed66979b82b94bc93b5d7`
  - public key: `b7dc9198306d41952c49410f63cfd19f231536f37e886767753cbccc78616e0f`
  - provenance before candidate restart: `0.4.0 (68fcc3f1)`
  - candidate process command uses the same persistent data and the candidate binary; query `/version` before counting it as candidate provenance.

Expected test roster:

1. Windows CLI: `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`
2. Android: `12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG`
3. AWS pure relay: `12D3KooWPJK6KgKsafefLWeGs4kVbj7wBnU67yKe88ni3FHZ3Hr2`, external WebSocket `/ip4/54.226.67.101/tcp/9001/ws/...`
4. Mac CLI: current `12D3KooWNC5rEKFhuxDNDNsJ6Q58Ca75LnxfjUqspGzGRdYRUWyt`
5. iOS: pending fresh device-side provenance and live PeerId confirmation.

Do not wipe or rebuild the Mac persistent data during a run window. Earlier
rebuilds presented other Mac PeerIds (`WP1...`, `WFy...`, and now `WNC...`),
which creates stale fleet entries and breaks reply resolution. `WNC...` is the
current live-window identity only; historical parity is not claimed.

## Changes already implemented and tested

`4083e59b` contains:

- bounded CLI auto-reply: one short acknowledgement per unique user message,
  no response to `[auto-reply]`, no echo storm;
- dual-bind port probe releases each test socket;
- offline/queued send consent and identity initialization;
- receipt handling clears Delivered/legacy Read messages from the outbox/drift
  retry state;
- per-peer established-connection cap changed to 2, allowing relay plus one
  direct/hole-punch path while preventing the multi-address third connection
  that triggered the request-response panic;
- iOS persists QR/invite bootstrap seeds and promotes them after Identify;
- iOS rejects special-use IPv6 hints (loopback, unspecified, link-local, ULA,
  multicast) but retains public IPv6 candidates for roaming hole punch.

`acda09df` additionally makes API recipient resolution skip malformed legacy
contact rows whose `public_key` contains a PeerId/identity value. This prevents
a stale contact from returning a misleading 400 before the authenticated
PeerId/public-key or inbox sender-key path can resolve a reply.

Local verification already passed:

- `cargo fmt --all -- --check`
- `cargo test --package scmessenger-core --lib`: 1340 passed, 5 ignored
- `cargo test --package scmessenger-cli --bin scmessenger-cli`: 71 passed, 0 failed

The remote CI for `acda09df` was still running at the last checkpoint. Refresh
it with `gh pr checks 139 -R Sovereign-Communication/SCMessenger`.

## Confirmed failures and evidence

The old exact `68fcc3f1` Mac runtime was kept live for observation. At
05:08:45Z it reproduced the known request-response P0:

```text
libp2p-request-response-0.29.0 assertion left=false right=true
swarm_event_loop_died: the mesh is down but the process is still up; exiting
```

The preceding logs showed address/PeerId churn, repeated negotiation failures,
and large circuit addresses being rejected by mDNS as `TxtRecordTooLong`. This
old-run failure is confirmed and is not final-gate evidence. The candidate Mac
was restarted after that failure with the preserved `WNC...` identity.

The candidate Mac restart completed at 05:10:47Z. `/version` then reported
`git_hash=acda09df`; at the 05:17Z checkpoint the process was still alive and no
candidate request-response panic had been observed. It had only AWS relay in
the authenticated peer table, with 40 listeners (38 direct, 2 circuit). Logs
showed a DCUtR hole-punch failure to Windows followed by relay fallback, plus
high negotiation-failure noise from stale/promiscuous addresses including old
`192.168.0.111` entries. This is a live observation, not a green transport
result or a five-node gate start.

The 05:27Z live poll still showed the candidate process alive with no new
request-response panic, but repeated `[DIAL-BACKOFF]` marking the Android
target dead after three failed attempts and later marking the AWS node dead as
well. Stale-address negotiation failures continued, including old local
addresses and loopback candidates. This confirms that the candidate has not
converged on a usable five-node peer set; investigate address/route selection
with Windows and Android evidence before changing further transport code.

Windows previously confirmed receiver-side evidence for Mac probe
`54501eea-95e5-4f6f-8624-642a59f98c3b`:

- `inbox_receive` at `04:46:26.335Z`;
- delivery ACK at `04:46:26.336Z`;
- live peer set then was AWS relay plus macOS; Android and iOS were absent;
- AWS external path was WebSocket-only; plain TCP to port 9001 timed out.

The Windows lane also reported that its old runtime could receive and ACK a Mac
message but could not reply to the Mac PeerId (`HTTP 400 Recipient does not
contain a valid Ed25519 public key`). That is the defect addressed by `acda09df`.

## iOS return and log capture

Physical iPhone is connected and paired:

- physical UDID: `00008130-001A48DA18EB8D3A`
- CoreDevice identifier accepted for app-container copy:
  `4731D564-2F8F-5BC6-B713-D7774AF598F9`
- app: `SovereignCommunications.SCMessenger`, version 0.4.0, build 9
- CoreDevice reports iOS 26.5.2, developer mode enabled, tunnel connected.

The first read-only capture is in the task workspace:

`/Users/christylove/Documents/Codex/2026-08-09/ch/PR139_ios_capture_20260810T0512Z/`

Files pulled:

- `mesh_diagnostics.log`
- `ledger.json`
- `contacts.db`
- `history.db`

The 223-line diagnostic snapshot contains events from 05:08:21Z through
05:08:59Z. It is a snapshot of prior activity, not yet a fresh synchronized
five-node run. Important findings:

- Android target `12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG` (public
  key prefix `c0a682ef`) repeatedly logged `multipeer ... Peer not connected`.
- iOS core dispatch repeatedly logged `skipped_local_accepted` with
  `reason=swarm_bridge_unavailable`.
- BLE retry attempts were accepted and transmitted, but the BLE central later
  disconnected. This does not prove Android received the message.
- iOS initiated history-sync consideration and dial attempts toward Android and
  the AWS relay, but no user-message receive/ACK is present in this snapshot.
- Persisted ledger counters at capture time aggregated to Android 29 successes /
  949 failures, Windows 35 / 364, and AWS relay 1 / 35. These are routing
  counters, not delivery proof.
- The iOS ledger contains the AWS relay candidates including
  `/ip4/54.226.67.101/tcp/9001/ws` and Android relay/direct candidates. Public
  IPv6 candidates exist and must be tested for hole punch rather than blanket
  blacklisted.
- The pulled `history.db` contains older identity-sync/auto-reply records and
  stale Mac identities. Some historical rows also show `delivered=true` while
  `status=Queued`; do not use that database snapshot as current delivery proof.

A second targeted capture was pulled after launching the installed app from
the paired device:

`/Users/christylove/Documents/Codex/2026-08-09/ch/PR139_ios_capture_20260810T052145Z/`

It covers 05:21:59Z through 05:22:09Z and contains `mesh_diagnostics.log`,
`ledger.json`, and `history.db`. The Android target was still not connected;
core direct dispatch remained `swarm_bridge_unavailable`; BLE remained
accepted/transmitted but unconfirmed; and old iOS messages continued retrying
(`retry_attempt=14`, `acked_without_receipt` above 500 for one message). This
is an open iOS retry/outbox condition, not proof that Android alone is the
sender failure. The next reproduction must use fresh message IDs and include
Android logcat plus Android mesh diagnostics and receiver-side
`inbox_receive`/ACK evidence.

When iOS is part of the next run, capture before, during, and after the test:

```bash
IOS_DEVICE_UDID=4731D564-2F8F-5BC6-B713-D7774AF598F9 \
  DURATION_SEC=3600 \
  LOGDIR=/path/to/PR139_ios_capture_<utc> \
  CAPTURE_ANDROID=0 CAPTURE_IOS_SIM=0 \
  ./scripts/capture_logs.sh
```

For structured iOS polling, run `python3 scripts/ios_extractor.py` from the
repo root after confirming the device is unlocked and the app is running. The
diagnostic pull itself is more reliable than raw OSLog on this CoreDevice
setup. For each test message record UTC time, direction, exact message ID,
sender/receiver PeerIds, build SHA, route classification (direct, hole punch,
or AWS relay), receiver `inbox_receive`, delivery ACK, retry/outbox state,
restart/reconnect state, and the relevant Android/iOS diagnostic lines.

The next fresh capture must include a deliberate Android send reproduction;
do not infer Android failure solely from the iOS sender state. Capture Android
`mesh_diagnostics.log` plus logcat at the same UTC markers and require the
Android-side receiver evidence.

## Live coordination request currently in flight

The candidate Mac sent Windows this SCM CLI message:

- message ID: `8a16beb5-4f2a-4844-b913-70c4cd35a726`
- intent: fresh Windows-to-Android text with receiver inbox/ACK and Android
  route/error evidence, then fresh Windows-to-Mac send and reply validation for
  `acda09df`.

The send returned `accepted`; it is not confirmed until Windows reports the
receiver-side evidence. Mirror the exact result in PR 139 comments. If SCM CLI
transport/routing degrades again, continue exclusively through PR comments for
coordination while leaving the five-node gate closed.

At the 05:23Z PR refresh there was no new Windows response for this message.
Do not resend it or count it as delivered; use the PR thread as the fallback
request for the exact Windows/Android evidence if the SCM route remains
degraded.

## Next actions, in order

1. Read the latest PR comments and refresh CI for `e873ed4a` (runtime code is
   still `acda09df`; the latest commit is documentation-only).
2. Query the live Mac candidate `/version`, `/api/peers`, `/api/listeners`, and
   `/api/history`; confirm candidate provenance and that the swarm remains alive.
3. Get Windows receiver evidence for SCM message
   `8a16beb5-4f2a-4844-b913-70c4cd35a726`; ask Windows to report Android-side
   send/receive errors and whether the Mac reply now succeeds.
4. Pull a fresh iOS diagnostics snapshot after the app is actively connected;
   collect a synchronized Android logcat/diagnostics bundle and Windows/AWS
   logs. Use distinct message IDs for every direction.
5. If Android still fails, classify it as `swarm_bridge_unavailable`, route
   selection, address churn, BLE, or receipt/outbox behavior from both sides;
   fix only with evidence and re-run focused pair tests.
6. Re-anchor all five nodes on one verified head, preserve identities, and run
   the full matrix: Windows↔Android, Mac↔Windows, iOS↔Android, each through
   direct/LAN where possible, AWS relay fallback, and roaming iOS cellular with
   public IPv6/DCUtR hole-punch observation.
7. Start the one-hour gate only after the matrix passes. Reset the clock on any
   panic, swarm death, missing ACK, route ambiguity, message loss, identity
   change, retry storm, or loss of full functionality.

Never merge based only on CI or the fact that all nodes appear in a peer table.
