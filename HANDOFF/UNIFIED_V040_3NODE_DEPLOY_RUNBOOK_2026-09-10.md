# 3-node parity deploy runbook — unified candidate

Candidate SHA: `3ccf0ec2b2091b556e68caa44c3e97797094cdf2`
Branch: `unified/v040-3node-parity`
Workspace: `C:\Users\SCM\Documents\GitHub\MiMoSCMessengerFresh`
Gates so far: `cargo check -p scmessenger-core -p scmessenger-cli` PASS; observation unit tests 11/11 PASS.

Do not deploy until Android compile gate and (if disk allows) a broader core test pass are recorded.

## One-candidate rule

All three nodes MUST report this exact git hash in `/version` or app diagnostics
before the 3-node matrix is scored. Mixed generations are a known failure mode
(RCA A1/P1/W3).

## Node 1 — Windows CLI

1. Record current live identity + config path (do not clobber keys).
2. Stage rollback **outside** any `target/` (RCA W4):
   `tmp/radio-<sha>/rollback/scmessenger-cli.exe` + SHA256SUMS.
3. Build:
   ```powershell
   cargo build -p scmessenger-cli --release
   ```
4. Stop old node; copy new exe; relaunch with the recorded bootstrap config
   (E6 pattern: AWS multiaddr persisted in `config.json bootstrap_nodes`).
5. Verify: `/version` git hash, peers, `external_addrs` == configured T14 endpoint.

## Node 2 — AWS cloud node

1. Build/publish docker image at the same SHA via `.codebuff_deploy/aws/`.
2. Capture E4 evidence: SSH `sha256sum` of container binary + `docker image` digest.
3. Redeploy preserving `/data` identity bind-mount.
4. Verify: `curl /version` git hash equals candidate; health 200; Windows + Pixel peers.

## Node 3 — Pixel 6a (operator-owned)

1. Build APK from this SHA:
   ```powershell
   .\gradlew :app:assembleDebug
   ```
2. Operator installs via adb (no shell-forced service actions from CTO lanes).
3. Record `pm path` APK SHA256 and post-install service identity (E5).
4. Verify: app diagnostics git hash equals candidate; peersDiscovered matches fleet.

## Score matrix (after all three match SHA)

| Gate | Pass condition |
|---|---|
| Baseline | message + receiver decrypt + durable history + receipt |
| Cell-only / AWS custody | same, with first-choice LAN path unavailable |
| BLE (after radio reboot) | phone<->Windows BLE-specific ingress, no TCP/mDNS fallback |

Not transport ACKs, not UI counters, not BLE local acceptance alone.

## LAN discovery reality (measured 2026-09-16, Windows node on this host)

Recorded here because phone-initiated LAN arrival depends on it, and because it
is an environment fact, not a code fact. Establish it by test, not by reading:
from the phone, `nc` to 192.168.0.121 succeeds on 9002, 443, 80, 8080 and 9090.

- The node's libp2p TCP listener is NOT on 9001 on this host. 9001 is already
  bound by the node's own local HTTP control server
  (`Warp HTTP+WS server listening on ws://127.0.0.1:9001`), so the swarm's
  listeners land on `/ip4/192.168.0.121/tcp/{443,80,8080,9090}` plus the
  WebSocket listener `/ip4/192.168.0.121/tcp/9002/ws`.
- `SubnetProbe` probes 9001 and 9002, so on this node it can only ever hit 9002.
  That port answers a TCP connect and completes an HTTP websocket upgrade
  (host-side handshake returns 101), but the Android client cannot complete a
  libp2p dial against it: the node's logs show 0
  `direction=inbound transport=ws` against 35 `transport=tcp` inbounds, and the
  phone's dial of `/ip4/192.168.0.121/tcp/9002/ws` fails client-side in about
  80ms with `IronCoreException$NetworkException` and no connection attempt
  visible on the peer. SubnetProbe no longer reports a hit on the WebSocket port
  as a dial candidate (see `SubnetProbe.WEBSOCKET_PORT`).
- Therefore Android LAN arrival for this node is via mDNS - the node advertises
  direct `/ip4` addresses only, because `build_mdns_advertised_addrs` excludes
  `/ws/` and `/p2p-circuit/` - or via node-initiated connections.
- The host firewall is NOT the blocker: no inbound allow rule is listed for
  9001/9002 (`Get-NetFirewallPortFilter` shows one for 443 only), yet the phone
  connects to every one of those ports. Do not chase a firewall rule for this.

## Disk notes

- ~17 GB free. If Android/full suite needs room: run `scripts/reclaim_safe.py`
  or `scripts/clean_target.sh --all` only on **this** workspace's `target/`.
- Never delete uncommitted work, identity keys, or rollback artifacts.

## Stop conditions

Artifact hash mismatch, identity change on any node, disk <10 GB, or
entry-gate failure after one retry → write checkpoint and hand back.
