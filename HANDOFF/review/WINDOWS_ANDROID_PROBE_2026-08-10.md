# Windows <-> Android Live Delivery Probe

Status: Complete
Probe window (UTC): 2026-08-11T04:30:33Z - 2026-08-11T04:39:52Z
Operator/agent: Claude (Cowork sandbox), read-only against the running node except
for the two `POST /api/send` calls described below.
Filename says 2026-08-10 per instruction; all timestamps recorded below are the
true UTC values observed during the run (2026-08-11 UTC).

## 0. Node under test

- HTTP control API: `http://127.0.0.1:9876`
- `GET /version` (queried 2026-08-11T04:30:39Z and re-checked 2026-08-11T04:36:35Z,
  unchanged both times):
  ```json
  {"build_time":"2026-08-10T15:31:07.573633900+00:00","core_provenance":"0.4.0 (1023d7ae:tracking/pre-v040-tag-work:1786342886)","git_hash":"e5284b7b","version":"0.4.0"}
  ```
- Local identity (`GET /api/identity`): `libp2p_peer_id = 12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`,
  `identity_id = 985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826`,
  `nickname = Claude-Windows-Driver`.
- OS process: `scmessenger-cli-e5284b7b.exe`, PID 15520, started
  2026-08-11T04:23:28Z (per `Get-Process.StartTime`, converted to UTC), path
  `C:\Users\SCM\AppData\Local\scmessenger\soak\bin\scmessenger-cli-e5284b7b.exe`.
  Same PID confirmed still running at 2026-08-11T04:39:52Z (end of probe).
- Soak supervisor state (`%LOCALAPPDATA%\scmessenger\soak\status.json`):
  `generation: 1`, `run_started_at: 2026-08-11T04:23:28Z`, `healthy: true`,
  `peer_count: 2` at every 15s probe from 04:36:17Z through 04:38:32Z (last
  sample before this file was written). Generation never incremented during
  the probe, i.e. the supervisor never respawned the node.
- Node was NOT killed, restarted, or reconfigured by this probe. Only
  `GET`s and two `POST /api/send` calls were made against it.

## 1. API contract used (from `cli/src/api.rs`, read before use)

- `POST /api/send` body `{"recipient": <peer_id|pubkey|identity_id|nickname>, "message": <string>}`.
  Response: `{"success": bool, "error": Option<string>, "message_id": Option<string>, "status": Option<string>}`.
  `status` is `"accepted"` (dispatched to BLE or swarm successfully) or
  `"retrying"` (initial dispatch failed but the swarm event loop is still
  alive). **`accepted` only means the local node handed the envelope to the
  transport layer — it is not receiver-side confirmation.**
- `GET /api/send/:message_id` — reads the same history record directly:
  `{"message_id","status" ("delivered"|"pending"),"delivered": bool,"peer_id","timestamp"}`.
  `delivered` flips to `true` only when `history_store_manager().mark_delivered()`
  is called, which in `cli/src/main.rs` (~line 2516) happens **only** when this
  node receives a `MessageType::Receipt` envelope from the peer whose
  `receipt.message_id` matches. This is a genuine protocol-level delivery ACK,
  not a heuristic.
- `POST /api/history` (note: POST, not GET) body `{"peer_id": Option<string>, "limit": Option<usize>}`
  (peer_id here is the **identity_id**, not the libp2p PeerId). Response:
  `{"messages":[{"id","peer_id","content","direction" ("sent"|"received"),"timestamp","delivered"}]}`.
  Important nuance found in `core/src/iron_core.rs` (~line 3484-3491):
  for **received** messages, `delivered` is hardcoded `true` at insert time —
  it means "this node stored the inbound message locally," NOT "the sender
  received an ACK." For **sent** messages, `delivered` starts `false`
  (`cli/src/api.rs` `handle_send_message`) and only becomes `true` via the
  same Receipt-driven `mark_delivered` path above. Do not read
  `delivered:true` on a `received` row as evidence about the other direction.
- `GET /api/peers`, `GET /api/listeners`, `GET /api/swarm/stats`,
  `GET /api/connection-path-state`, `GET /api/discovery/peers`,
  `GET /api/external-address` were also used; see Section 5 for what they
  do and do not prove about routing.
- Auto-ACK mechanism confirmed by reading both sides of the protocol: on
  receiving `MessageType::Text`, the Windows CLI (`cli/src/main.rs` ~2431-2439)
  automatically calls `core.prepare_receipt()` and sends a `Receipt` envelope
  back to the sender. The Android app implements the equivalent
  (`MeshRepository.kt`, `sendDeliveryReceiptAsync()`, called after processing
  an inbound text at ~line 2140). So both ends are expected to auto-ACK a
  received Text message with a protocol Receipt, independent of any
  human-typed reply.

## 2. Baseline (2026-08-11T04:30:33Z - 04:30:39Z)

`GET /api/peers`:
```json
{"peers":[{"peer_id":"12D3KooWPJK6KgKsafefLWeGs4kVbj7wBnU67yKe88ni3FHZ3Hr2","reputation":50.0},{"peer_id":"12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG","reputation":50.0}]}
```
Both peers present throughout the probe (re-checked at 04:36:35Z, identical).

`GET /api/listeners` (2026-08-11T04:30:39Z, abridged to the relevant entries;
full raw output captured in this run's tool transcript):
- `/ip4/54.226.67.101/tcp/9001/p2p/12D3KooWPJK6KgKsafefLWeGs4kVbj7wBnU67yKe88ni3FHZ3Hr2/p2p-circuit/p2p/12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`
  (relay reservation for THIS node's own external reachability, via the AWS
  peer)
- `/ip4/[REDACTED-PUBLIC-IP]/tcp/14432/p2p/12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG/p2p-circuit/p2p/12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`
  (relay reservation for THIS node's own external reachability, via the
  Android peer)
- `/ip4/192.168.0.135/tcp/9090/p2p/12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG/p2p-circuit/p2p/12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`
  (same, over a private LAN address)
- Remainder are local TCP/WS listeners on 127.0.0.1 / 192.168.0.121 / ::1.

These are **this node's own advertised listen/relay-reservation addresses**
(how others can dial in to Windows) — they describe Windows' inbound
reachability, not the path an outbound send to Android actually took.

`GET /api/connection-path-state` (2026-08-11T04:30:39Z): `{"state":"DirectPreferred"}`.
Per `cli/src/api.rs` `get_connection_path_state()` this is a **global**
heuristic (`peers non-empty` + `listeners non-empty` => `DirectPreferred`),
computed from aggregate counts, not per-peer. It is not evidence about the
Android connection specifically.

`GET /api/swarm/stats` (2026-08-11T04:30:39Z and re-checked 04:36:35Z):
`{"stats":[]}` — empty both times, despite two peers being connected. No
per-connection `current_address`/state data was available at any point in
this probe.

`POST /api/history` (limit 20, unfiltered) baseline tail: dominated by
recurring `identity_sync`/`history_sync` protocol traffic with peer
`a43772fe4343079a56d05b7816d38d0db0144dcbb906b4572d98a784ce4a279a` (this is
Android's `identity_id` — confirmed below) roughly every 60s, plus older
`sent` rows from prior manual probes, all showing `delivered:false`.

## 3. Direction A: Windows -> Android

Target: `recipient = "12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG"`
(Android's libp2p PeerId, per contact `Lucaso` in `GET /api/contacts`,
public_key `c0a682eff9128f4e9d1511c39b1e35526d9ceb4d93429a630c0649cacf16b9a5`).

Probe UUID: `bfa86fbf-82ba-474c-9331-c0c123d6607a`
Message body: `SCM-PROBE bfa86fbf-82ba-474c-9331-c0c123d6607a Windows-to-Android delivery probe 2026-08-11T04:32Z`

**Correction / disclosure:** the first `Invoke-WebRequest` call to `POST /api/send`
threw a client-side PowerShell error ("NonInteractive mode... Read and Prompt
functionality is not available") while trying to read an error-stream body.
That error was cosmetic to the PowerShell client only — the HTTP request had
already been accepted and processed by the node server-side. This produced an
**unintended duplicate send** of the identical probe text, ten seconds apart.
Both are reported below for honesty; `300ea972-...` is the one whose
`message_id` was actually captured from a successful client response.

| # | message_id | request sent (UTC) | HTTP result | `status` field |
|---|---|---|---|---|
| 1 (accidental duplicate) | `a12f62eb-1177-4d45-bb70-5f0b4f6b53a6` | 2026-08-11T04:32:15.525Z (request start) | request succeeded server-side; client display errored | `accepted` (inferred from server-side history record; response body was never displayed client-side) |
| 2 (confirmed/reported) | `300ea972-4622-4dcb-9fca-b264ec9e8cd0` | 2026-08-11T04:32:25.341Z - 04:32:25.526Z | HTTP 200 | `{"success":true,"error":null,"message_id":"300ea972-4622-4dcb-9fca-b264ec9e8cd0","status":"accepted"}` |

Both history rows for these sends carry `peer_id: a43772fe4343079a56d05b7816d38d0db0144dcbb906b4572d98a784ce4a279a`
(Android's identity_id), confirming both were addressed correctly.

**Polling** (`GET /api/send/300ea972-4622-4dcb-9fca-b264ec9e8cd0`, every 15s,
2026-08-11T04:32:54Z - 04:35:39Z, 12 polls / 3 minutes as specified):

```
04:32:54.502Z poll=1  status=pending delivered=False
04:33:09.663Z poll=2  status=pending delivered=False
04:33:24.667Z poll=3  status=pending delivered=False
04:33:39.683Z poll=4  status=pending delivered=False
04:33:54.699Z poll=5  status=pending delivered=False
04:34:09.715Z poll=6  status=pending delivered=False
04:34:24.720Z poll=7  status=pending delivered=False
04:34:39.727Z poll=8  status=pending delivered=False
04:34:54.731Z poll=9  status=pending delivered=False
04:35:09.738Z poll=10 status=pending delivered=False
04:35:24.754Z poll=11 status=pending delivered=False
04:35:39.762Z poll=12 status=pending delivered=False
```

**Extended check beyond the required window:** re-polled both message IDs at
2026-08-11T04:36:35Z and again at 2026-08-11T04:39:52Z (roughly 4 and 7.5
minutes after the confirmed send). Both remained:
```json
{"message_id":"300ea972-4622-4dcb-9fca-b264ec9e8cd0","status":"pending","delivered":false,...}
{"message_id":"a12f62eb-1177-4d45-bb70-5f0b4f6b53a6","status":"pending","delivered":false,...}
```

**Terminal state (API's own vocabulary): `pending`, `delivered: false`, for
both message IDs, for the entire ~7.5-minute observation window.** No
transition to `delivered`/`retrying`/`failed` was observed. The API exposes
no `queued` or `failed` state distinct from `pending` in this build — see
Section 1 for the full status vocabulary.

## 4. Direction B: Android -> Windows

We cannot make the phone send on command (no adb; it is on cellular). Per
instructions, this section reports the most recent genuine (non-protocol-housekeeping)
Android-originated inbound message.

Filtered `POST /api/history` on `peer_id = a43772fe...4a279a` (Android's
identity_id), scanning the last 60 rows: the traffic is dominated by
`kind:"identity_sync"`/`kind:"history_sync"` protocol housekeeping messages
recurring roughly every 60-100 seconds (these are automatic, not
human-originated).

**Most recent genuine (`kind:"text"`) Android-originated inbound message:**

- `message_id: e7a8b366-ff27-451d-9cea-e8ba97d9d6f3`
- `direction: received`, `delivered: true` (per Section 1's caveat, this only
  means "Windows stored it," not that a receipt was sent for it)
- `timestamp (UTC): 2026-08-11T04:33:33Z`
- Full decrypted envelope:
  ```json
  {"schema":"scm.message.identity.v1","kind":"text","text":"confirmed delivery probe.","sender":{"identity_id":"a43772fe4343079a56d05b7816d38d0db0144dcbb906b4572d98a784ce4a279a","public_key":"c0a682eff9128f4e9d1511c39b1e35526d9ceb4d93429a630c0649cacf16b9a5","device_id":"[REDACTED-DEVICE-ID]","nickname":"Lucaso","libp2p_peer_id":"12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG","listeners":["/ip6/[REDACTED-IPV6]/tcp/9090/ws","/ip6/[REDACTED-IPV6]/tcp/9090/ws","/ip4/192.168.0.135/tcp/443"],"external_addresses":["/ip4/192.168.0.135/tcp/443","/ip4/192.168.0.121/tcp/9090"],"connection_hints":["/ip6/[REDACTED-IPV6]/tcp/9090/ws","/ip6/[REDACTED-IPV6]/tcp/9090/ws","/ip4/192.168.0.135/tcp/443","/ip4/192.168.0.121/tcp/9090"]}}
  ```
- The sender envelope's `public_key` and `libp2p_peer_id` match the `Lucaso`
  contact and the Android peer named in the orchestrator's LIVE STATE
  exactly, so this is authenticated as genuinely originating from the
  Android device's key, not spoofable.
- Timing: this arrived 68-78 seconds after our two Direction-A sends
  (04:32:15Z / 04:32:25Z), and its text ("confirmed delivery probe.") strongly
  suggests it is a human reply on the Android device to seeing our probe
  text arrive. **This is circumstantial: the text does not quote our UUID or
  message ID, so it cannot be cryptographically or programmatically tied to
  our specific probe message versus some other concurrent activity.** It is
  the strongest available Android->Windows evidence given the constraints,
  but it is a human-typed acknowledgement, not a protocol Receipt.
- **Bridge corroboration:** `%LOCALAPPDATA%\scmessenger\inbox_bridge.status.json`
  and `.state.json` (read-only, not modified) confirm the bridge (PID 24684,
  started 2026-08-11T04:23:28Z) saw and filed this exact message: `last_received_message_at: 2026-08-11T04:37:34Z` (a later message; the
  e7a8b366 message was filed earlier in the same session), `node_reachable: true`,
  `node_health_ok: true`, `consecutive_node_failures: 0`. The corresponding
  ticket file `HANDOFF/todo/INBOX_2026-08-11T043333Z_e7a8b366ff27.md` exists
  with matching sender/message-id/timestamp and states the sender "matched
  the configured allow-list exactly."
- Eleven total `INBOX_*.md` tickets exist under `HANDOFF/todo/`, all with
  2026-08-11 04:26Z-04:37Z timestamps, all filed during this soak
  generation — none of these were created or modified by this probe (this
  probe only read them).

No protocol-level `Receipt` from Android for our Direction-A messages was
observed (see Section 3) — the auto-ACK mechanism described in Section 1
did not visibly fire for either `a12f62eb-...` or `300ea972-...` within the
observation window, despite Android apparently being reachable enough to
send a human-typed reply and ongoing `identity_sync`/`history_sync` traffic
in both directions throughout the same window.

## 5. Route classification

**Not determinable for either direction from this API, and we are not
inferring it from peer presence.** Specifically:

- `GET /api/swarm/stats` returned `{"stats":[]}` at both baseline and
  re-check — the per-connection `current_address`/state table
  (`ApiConnectionStats`) had no entries for either peer despite both being
  connected. This is the field that would have given a real per-peer route
  (direct address vs relay circuit address).
- `GET /api/connection-path-state` returns a single global string
  (`DirectPreferred`) computed from aggregate peer/listener counts
  (`cli/src/api.rs::get_connection_path_state`), not a per-peer route. It
  cannot be attributed to the Android connection specifically.
- `GET /api/discovery/peers` reports `"transport":"tcp/lan"` for **both**
  peers, including the AWS relay peer at a public IP over the open internet.
  Reading `cli/src/api.rs::handle_get_discovery_peers` shows this field is a
  **hardcoded literal string** (`transport: "tcp/lan".to_string()`), not a
  real transport read. It is not usable as route evidence and is called out
  here specifically so it isn't mistaken for one.
- `GET /api/listeners` and `GET /api/external-address` describe this node's
  own inbound reachability (including relay-circuit reservations through
  both the AWS peer and, oddly, through the Android peer itself), not the
  path taken by an outbound send. See Section 2.
- Android's self-reported envelope (Section 4) lists both IPv6 addresses
  (`[REDACTED-IPV6]...`, consistent with a cellular carrier) and a
  private LAN address (`192.168.0.135`) plus, unexpectedly, Windows' own LAN
  address (`192.168.0.121`) in its `external_addresses`/`connection_hints`.
  This looks like stale/cached data from an earlier same-LAN session rather
  than the current cellular state, and is not reliable enough to classify
  the live route either.

**Conclusion: route (direct / LAN / hole-punch / relayed-via-AWS) for
neither direction could be determined from the exposed API in this probe.**

## 6. Node health across the probe window

- Process `scmessenger-cli-e5284b7b.exe` PID 15520 was running continuously
  from before the baseline (04:30:33Z) through the final check (04:39:52Z);
  same PID at both ends.
- `soak/status.json`: `generation: 1` throughout, `healthy: true` at every
  15-second supervisor probe sampled (04:36:17Z - 04:38:32Z), `peer_count: 2`
  at every sample, `zero_peers_for_secs: 0`, `consecutive_unreachable: 0`.
  Generation staying at 1 means the supervisor never respawned the node
  during this run, which began at 04:23:28Z (before this probe started) and
  continued past the end of this probe.
- Grepped `%LOCALAPPDATA%\scmessenger\logs\scm.log.2026-08-11-04` (tracing
  output for the current hour) and
  `%LOCALAPPDATA%\scmessenger\soak\runlogs\run_20260811T042328Z_gen001.log`
  (full stdout/stderr for the current generation) for
  `panic|PANIC|swarm_event_loop_died|STATUS_STACK_BUFFER_OVERRUN|thread .* panicked|restart|respawn`
  (case-insensitive): **no matches in either file.**
- **No panic, no `swarm_event_loop_died`, and no restart occurred during
  this probe window.**

## 7. What this probe does and does not prove

**Proven:**
- The Windows node, at build `e5284b7b` / `0.4.0`, accepted a `POST /api/send`
  targeting Android's libp2p PeerId and returned HTTP 200 with
  `status: "accepted"` and a `message_id`, for two separate sends carrying the
  same probe UUID.
- Neither sent message reached the `delivered: true` state (the API's
  Receipt-backed proof of receiver-side ACK) within an observation window of
  at least 7.5 minutes.
- A message cryptographically authenticated as originating from Android's
  key (`c0a682eff9...`, matching contact `Lucaso` /
  `12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG`) arrived at the
  Windows node 68-78 seconds after the probe sends, was decrypted, stored,
  and auto-filed as an inbox-bridge ticket. Its text ("confirmed delivery
  probe.") is strongly suggestive of a human reaction to our probe but
  cannot be tied to the specific message ID.
- Bidirectional `identity_sync`/`history_sync` protocol traffic with Android
  continued to flow in both directions throughout the probe window,
  indicating the peer connection itself was live and exchanging authenticated
  application-layer traffic, not merely present in a stale peer list.
- The node process did not crash, panic, or get respawned by its supervisor
  at any point during the probe.

**NOT proven:**
- That Android's messaging application actually received, decrypted, and
  displayed either `a12f62eb-...` or `300ea972-...` to a user. No
  `inbox_receive` + matching-message-ID delivery ACK was observed for either
  message, which is the standard this probe was required to hold to. The
  `accepted` status and the unrelated human reply are not substitutes for
  that.
- Why the expected auto-ACK Receipt (Section 1) did not fire back for our
  specific messages, given that Android was clearly reachable enough to
  reply with a human-typed text message and to keep exchanging
  identity_sync/history_sync traffic in the same window. This probe only
  establishes the absence of the Receipt within the window; it does not
  diagnose the cause (app foregrounded vs backgrounded, receipt path
  specifically broken, cellular NAT/relay asymmetry, timing beyond the
  window, or something else).
- Which physical/logical transport path (direct, hole-punched, or relayed
  through the AWS node) carried any of this traffic in either direction —
  the API does not expose per-peer route data in this build (Section 5).
- Anything about Android-side logs, since adb was unavailable per the task's
  constraints; all Android-side evidence here is inferred solely from what
  the Windows node received and stored.
