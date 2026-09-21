# Three-node live baseline + update plan (2026-09-19, read-only pass)

Type: EVIDENCE + PLAN. Nothing was updated, restarted, or reconfigured for this
note -- every line below is a read of a running node, and every command is named.
Filenames on the nodes carry the build they are running; identities were read
from the nodes' own logs, not from the 2026-09-03 runbook, whose table is now
stale (see "Runbook delta").

## Live node table (read this pass)

| Node | Live identity | Address | Build running | How read |
|---|---|---|---|---|
| Windows CLI | 12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw | LAN 192.168.0.121, WAN 147.81.41.188 | `git-sha 0fd69fb2...`, CLI 0.4.0, built 2026-09-18T21:30Z | `tmp/radio-candidates/0fd69fb2.../cli-provenance.txt`; process started 2026-09-18 12:18 local |
| AWS cloud (`scm-always-on-node`) | 12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31 | 18.234.62.247 (dynamic) | image `testbotz/scmessenger:sha-0fd69fb`, container up 28 h | `describe_instances` via `.codebuff_deploy/aws/scm_session.py`; `docker ps` over SSH |
| Pixel 6a (bluejay) | 12D3KooWD776DQdWh6iHV8Qcnpj9jvTXhpJAgSsFPbMCaRtQpFmn | LAN 192.168.0.103, app updated 2026-09-19 10:24 | versionName 0.4.0, versionCode 15 | `adb connect 192.168.0.103:34291`; `dumpsys package` |

Windows and cloud are on the **same** build (`0fd69fb`, 2026-09-18); the phone is
on today's 10:24 install, which the 2026-09-19 device check verified contains
#309 (b529011b). So the two Rust nodes are one main-tip behind.

## Health: all three are running and working

- **Windows node** (`%LOCALAPPDATA%\scmessenger\logs\scm.log.2026-09-20-02`,
  written within the minute): `Relay custody audit log count: 5981`; accepted
  custody for an offline destination three times in ten seconds; ledger exchange
  with a peer; `contacts_canonical_hex_live` then `contacts_add` for nickname
  "Lucas" learned from an identity envelope.
- **Cloud node** (`docker logs scm-node`): `Relay custody audit log count: 5961`;
  `DriftFrame type: Data` arriving repeatedly from the phone; `Gossipsub message
  ... on topic sc-mesh (345 bytes)`; routing optimization tick healthy.
- **Phone** (`adb logcat`): `Mesh Stats: 2 peers (Core), 2 full, 0 headless`,
  uptime 11769 s; address snapshots show the mesh formed through **both** circuit
  relays -- `/ip4/192.168.0.121/tcp/443/p2p/D6vZQrUq.../p2p-circuit/p2p/D776DQdW`
  (Windows) and `/ip4/18.234.62.247/tcp/9001/p2p/GvCWJNo.../p2p-circuit/p2p/D776DQdW`
  (cloud). `pending_outbox.json` is 2 bytes, i.e. `{}` -- the messages that were
  stuck this morning have drained.

## Live defect, confirmed from both sides at once

The per-peer connection cap is still refusing the phone's dials, and this pass
captured the *same* event in two independent logs:

- Windows node: `Inbound connection DENIED from /ip4/192.168.0.103/tcp/<ephemeral>
  -> /ip4/192.168.0.121/tcp/{80,443,8080,9001,9090,65204}: connection_limits:
  limit 4 reached` -- 274 WARNs in one hour, the hour's dominant message class.
- Cloud node: `Inbound connection DENIED from /ip4/147.81.41.188/tcp/18981 ->
  /ip4/172.31.18.74/tcp/9001: connection_limits: limit 4 reached` -- the same
  cap refusing the *Windows* node's dial to the cloud node.

192.168.0.103 is the phone (mdns located its wireless-debug endpoint there), so
the phone dials a multi-port set against each peer and only 4 concurrent
connections per peer are admitted; the rest are denied. That is the mechanism
behind the morning's "messages sat 4.5 and 8 minutes" report, and it is
unchanged by the build bump.

The hour's 32 ERRORs are all one shape and are not a node fault:
`ERROR warp::server::run: server connection error: hyper::Error(Parse(Method))`
-- malformed HTTP arriving at the node's web port, from the same dialing burst.

## Update procedure per node (NOT executed)

Precondition: the drain lands first (see below). Updating before it would need a
second update within the hour.

1. **Windows**: download the CI artifact for the target main tip --
   `gh run download <run-id> -n windows-cli-<sha> -D tmp/radio-candidates/<sha>`
   (ci.yml publishes `windows-cli-<github.sha>` with the exe + `cli-provenance.txt`,
   30-day retention, only when `rust_relevant` -- which fails open). Stop the live
   `scmessenger-cli.exe` (PID 22380, running from
   `tmp/radio-candidates/0fd69fb2.../`), start the new exe, then read
   `cli-provenance.txt` back to confirm the running commit.
2. **Cloud**: the image is built only from main (`docker-publish.yml` publishes
   `latest`). Then `scripts/aws_deploy.sh` -- it discovers the host from the EC2
   tag, pulls, and recreates `scm-node` with `/opt/scm-relay-data` preserved.
   Runbook governance applies: never terminate `i-0b735c4f26aea42ed`, no IP churn
   test, no version bump or tag.
3. **Phone**: `adb install -r` of the CI APK artifact for a post-#309 commit. The
   signing identity was pinned in #324/#326, so a `-r` install over the current
   build is expected to work now; the verified app-data backup at
   `tmp/device-backup-20260919/app-data.tar.gz` remains the safety net.

## Runbook delta (V040_3NODE_VALIDATION_RUNBOOK_2026-09-03.md is stale)

- Cloud identity is now `GvCWJNo...`, not `9uRMQTs...`; its IP is
  18.234.62.247, not 54.235.20.24 (dynamic IP, as Gate 6 predicted).
- Phone identity is now `D776DQdW...`, not `Kdek3ZZ...` -- consistent with the
  reinstall the operator authorised on 2026-09-19, which regenerates node
  identity. The runbook's addresses (192.168.0.129/121) have also moved
  (192.168.0.103/121).
- Windows identity `D6vZQrUq...` and its WAN 147.81.41.188 still match.
- The runbook's dated execution records were left untouched; this note is the
  correction rather than an edit to a historical record.

## Drain state at the time of writing (why "everything has landed" is not yet true)

Main tip `1b28c010`. #322 (`1827d160`) has all four required contexts green but
`macOS Native Tests` and `Bindings (Swift)` -- both of which compile the Rust core
its diff changes -- still `IN_PROGRESS`, so it is not mergeable under the
criterion. #325 (`d2dd7548`) and #329 (`e1b9263d`) are `MERGEABLE` but `BEHIND`,
one update-branch each; #329's prepared local commit (`ea2e657e`, its 3 ticket
files) is unpushed and ready to be the last landing.
