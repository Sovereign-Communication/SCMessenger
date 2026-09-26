# Live verification of the merged-SHA deploy: 0fd69fb2 traffic evidence

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Date: 2026-09-18 (UTC), window 22:28:01Z -> 22:37:59Z. Scope: post-deploy traffic
evidence on the merged binaries (PR #305 + #306, zombie-connection fix included),
following the drive pattern of `LIVE_VERIFICATION_305_2026-09-18.md`. Evidence
pass only -- nothing was fixed, restarted, or reconfigured during the window.

Binaries under test (provenance, read from the nodes themselves):

- Windows node: `0.4.0 (0fd69fb 2026-09-18T21:30:47+00:00)`, Core Provenance
  `0fd69fb:main:1789767009` (CLI artifact from CI run 35396440769; identity
  `985a25f9...`, PID 22380, started 22:18Z).
- AWS node: `0.4.0 (0fd69fb291590e215737735ed484dcf973b2f05b:main:)` (image
  `testbotz/scmessenger:sha-0fd69fb`, container `scm-node`; identity
  `37eb7561...`).
- Deploy record + pre-state: `tmp/deploy-55ef300b/DEPLOYED_0fd69fb2_20260918.md`.

## 1. Pixel: not exercised, gap named

The Pixel was not visible over wireless debugging at any point in this window
(`adb devices` -> empty list). Per the operator's order this pass records
2-node evidence and names the gap rather than forcing it. One indirect signal
that the Pixel's internet path to AWS still exists: AWS re-registered relay
custody for the Pixel peer (`12D3KooWD776DQdWh6iHV8Qcnpj9jvTXhpJAgSsFPbMCaRtQpFmn`)
repeatedly during the window, and AWS logged
`Relay circuit already active for 12D3KooWD776...` at 22:31:02Z. The on-device
Android behaviour was not observed.

## 2. Direct delivery both directions (12 sends)

### Windows -> AWS (6 sends, 5 s apart, to contact `AWS-CloudNode`, pk `69805e17...`)

Sender side (`GET /api/send/<id>`, all 6, verbatim statuses): **6/6
`{"status":"delivered","delivered":true}`** to peer
`37eb7561...`, timestamps 1789770517-1789770545 (tmp capture:
`tmp/deploy-55ef300b/traffic-0fd69fb2/w2a-status.txt`).

Receiver side (AWS `docker logs --since T0`, verbatim sample):

```
22:28:37.909Z DEBUG Received DriftFrame type: Data from 12D3KooWD6vZ... (windows)
22:28:37.911Z INFO  store::inbox: event="inbox_receive" message_id=a654cc84-ad3a-407c-a298-56b56da2a71d
                    sender_id=985a25f9... received_at=1789770517911
22:28:37.912Z INFO  Sending delivery ACK for a654cc84-... to 12D3KooWD6vZ...
22:28:38.168Z INFO  [OK] Message delivered successfully to 12D3KooWD6vZ... (255ms)
```

**6/6 `inbox_receive`** on AWS with the exact sender message_ids, **6/6 delivery
ACKs**, latency 252-255 ms each.

### AWS -> Windows (6 sends, 5 s apart, to contact `Claude-Windows-Driver`, pk `30d0fa67...`)

Sender side: 6/6 `{"success":true,...,"status":"accepted"}` (message_ids
`d7390f14`, `a4eb683e`, `ff9ec3a0`, `53c5f642`, `6a141223`, `610a0b00`).

Receiver side (Windows node stdout, verbatim):

```
22:29:12.766Z INFO auto_reply_ack_queued in_reply_to=d7390f14-2ecf-4baa-b34e-8e0eab23ad91
                   to=12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31
22:29:19.551Z INFO auto_reply_suppressed_rate_limit in_reply_to=a4eb683e-... min_interval_secs=60
   (suppressions for the remaining 4, same reason)
```

All 6 A2W messages were **received and processed** by the Windows node (the
first triggered the auto-reply, the rest hit the 60 s auto-reply rate limit --
by design). The auto-reply itself traversed back to AWS: AWS logged
`inbox_receive message_id=81a629b9-278f-4d1e-bd30-3d7716793995
sender_id=985a25f9...` at 22:29:12Z. **Full round trip A2W->auto-reply->AWS
completed**, and the W2A leg plus ack legs make 7 received records on AWS for
6+1 sends.

## 3. Relay function and custody (4 phantom drives)

Four sends to the still-present phantom contact (pk `82277046...`, destination
`12D3KooWJaS3om...`, offline by construction). Windows requester, verbatim:

```
22:35:44.672Z INFO  ROUTE_DECISION message_id=12D3KooWJaS3om...-1789770944672 attempt=1 pass=0
                    candidate=1/1 route=direct relay=- destination=12D3KooWJaS3om...
                    reason=INITIAL_SEND policy_reason=STORE_AND_CARRY
22:35:44.673Z WARN  no direct route; committing to drift store via
                    relay carrier=12D3KooWD776DQdWh6iHV8Qcnpj9jvTXhpJAgSsFPbMCaRtQpFmn
```

AWS carrier, verbatim (all 4 drives, +5 s cadence):

```
22:35:45.954Z INFO  Relay request from 12D3KooWD6vZ... for message 12D3KooWJaS3om...-1789770944672
22:35:45.954Z DEBUG relay request accepted in Phase A compat mode (no device enforcement)
                    identity_id="0c53c5b9bd70a68748b2a974d50e7694e96151f3c0aa76ae5c82bf5149dcad46"
22:35:45.958Z INFO  Accepted custody 12D3KooWJaS3om...-1789770944672-1789770945955-13
                    for offline destination 12D3KooWJaS3om...
```

**4/4 RelayRequests accepted, 4/4 custody records written** (sequence numbers
-13/-15 visible), requester-derived `identity_id=0c53c5b9...` (noise-derived
from the requester's key) recorded on each acceptance. Phantom send statuses
remain `pending/delivered:false` -- correct for an offline destination under
store-and-forward custody. Custody re-registration heartbeat both directions
every ~60 s (`[CUSTODY] Registered local identity with peer ... (relay-ready)`).

## 4. Discovery / identification / ping

- Bidirectional identify every ~60 s on both nodes: Windows
  `Identified peer 12D3KooWGvCW... protocols: 16, discoverable_addrs: 23`; AWS
  `Identified peer 12D3KooWD6vZ... protocols: 15, discoverable_addrs: 12`.
- Ping cadence: this build emits ping **failures** to the log, not successes.
  Zero `Ping failed` lines since the new binary started (22:18Z). The single
  ping-failure line in the 22:00 hour file (`22:12:49Z, peer=12D3KooWD776...
  connection_id=ConnectionId(2161), "ping protocol negotiation timed out"`)
  belongs to the PRE-swap process and is the designed dead-connection close
  firing on the Pixel's stale socket. Healthy-cadence evidence is therefore
  indirect: unbroken 60 s identify/custody cycles plus continuous delivery
  across the window.

## 5. The zombie-fix field check: per-peer connection boundedness

The check the zombie fix exists for -- do ghost slots accumulate? -- over the
16-send window plus heartbeat traffic:

| vantage | reading |
|---|---|
| Windows netstat T0 (22:28:01Z) | 1 established pair: `192.168.0.121:443 <-> 18.234.62.247:9001` (PID 22380) |
| Windows netstat T1 (22:34:03Z) | still exactly 1, same pair |
| Windows netstat T_end (22:37:59Z) | still exactly 1, same pair |
| AWS `ss` (22:33Z) | exactly 1 mesh pair: `172.31.18.74:9001 <-> 147.81.41.188:12314` |

13 delivered messages, 4 relay drives, 2x60 s identify/custody cycles -- and the
per-peer socket count never left 1. No second connection to the same peer was
created at any point, so no per-peer limit saturation and nothing for a reaper
to clean. **Bounded.**

## 6. Denies and the new classifier

Zero node-side denies occurred on either node in the window (greps for
`denied|negotiation|zombie|reap` over node stdout and the current LOGD hour
file: 0 relevant lines). The only deny-adjacent line on either node:

```
AWS 22:26:02.040Z DEBUG Incoming connection negotiation aborted from
  /ip4/166.196.8.193/tcp/6401 -> /ip4/172.31.18.74/tcp/9001:
  Listen error: Failed to negotiate transport protocol(s)
```

-- a pre-window probe from a non-mesh public IP failing protocol negotiation
before noise, correctly NOT a node-side deny and not labeled one. Because no
connection-limits deny occurred, the classifier's node-side branch
(`ConnectionLimits(n)` cause naming) was not exercised live; its behaviour is
covered by the unit test `deny_classifier_names_the_cause_instead_of_a_generic_string`
(swarm.rs) and the field evidence that motivated it.

## 7. Startup panic watch (libp2p-swarm 0.48 either.rs)

`tmp/win-node-0fd69fb2.err` line count: **4 at T0 (22:28:01Z), 4 at T1
(22:34:03Z), 4 at T_end (22:37:59Z)** -- static across the whole traffic
window. The one-off startup panic did not recur under load. Verdict: noted,
not a defect this pass; watch on subsequent passes.

## 8. End-state identities (byte-identical, read at T_end)

| node | T0 | T_end | verdict |
|---|---|---|---|
| Windows `identity_id` | `985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826` | same | UNCHANGED |
| AWS `identity_id` | `37eb75612c179d57a83801ba71f3545ffdf3d009adbf73c652dae37536b1d006` | same | UNCHANGED |

(device_id, peer_id, public_key, seniority likewise identical on both; full
triples in `tmp/deploy-55ef300b/aws-prestate-20260918.md` and the deploy
record.)

## 9. Per-node verdict

- **Windows node: CLEAN.** 6/6 delivered W2A with receipts, 6/6 A2W received
  and auto-reply round-tripped, 4/4 relay drives committed via the AWS carrier,
  identify/custody heartbeats nominal, connections bounded at 1, zero denies,
  zero ping failures, panic static, identity unchanged.
- **AWS node: CLEAN.** 7/7 expected inbox records (6 direct + 1 auto-reply),
  6/6 delivery ACKs at 252-255 ms, 4/4 relay acceptances with custody writes
  and the requester's derived identity recorded, identify/heartbeat nominal,
  exactly 1 mesh socket pair, identity unchanged.
- **Pixel: NOT EXERCISED** (not visible over wireless debugging; gap named in
  section 1). AWS-side relay circuit for the Pixel peer remained active.

## 10. What could NOT be exercised (stated plainly)

1. **Per-peer budget refusals live** -- deliberately not driven this pass
   (evidence pass, light volume); the ladder's refusal path is proven in
   `LIVE_VERIFICATION_305_2026-09-18.md` sections 3/6a/8 on the same code line.
2. **The classifier's node-side deny branch** -- no deny occurred (section 6);
   unit-tested only.
3. **Custody expiry** -- the 4 records written this window are minutes old
   against a 7-day retention; expiry remains proven in-process only
   (`test_trn04_custody_lifecycle.rs`), as in the prior run.
4. **Zombie reap under a real handover** -- no handover occurred; the fix's
   tracker/reap path ran idle (no candidates), which is the correct no-op, and
   its regression tests cover the firing case.
5. **Pixel-side Android behaviour** -- not visible (section 1).

## 11. State changes this pass made

None beyond traffic: 16 sends (12 direct, 4 relay-driven), their histories,
7 inbox records on AWS, 4 custody records addressed to the phantom (subject to
retention), and auto-reply/rate-limit events. No restarts, no config changes,
no contacts added or removed.
