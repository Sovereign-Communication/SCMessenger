# 3-node parity deploy runbook — unified candidate

Candidate SHA: `3ccf0ec2b2091b556e68caa44c3e97797094cdf2`
Branch: `unified/v040-3node-parity`

Superseded candidate for the 2026-09-17 cutover: PR #295 lane
`7becf8a0ac87bd9d7c3058723e702d4933377c22` (tree-equivalent to the synthetic
merge commit `1005da142c7e7cae94801241ac834a04824db99d` that CI actually built).
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

### Recreate recipe that actually works (verified live 2026-09-17T21:13Z)

Since PR #293 the image runs as `USER scm` (uid 10001), so any env copied
from a pre-#293 container is a trap: the legacy `SCM_CONFIG_DIR=/root/.config/scmessenger`
makes the entrypoint die with `mkdir: cannot create directory '/root': Permission denied`
in a restart loop. Two consequences, both hit in production on the 09-17 cutover:

1. **`/opt/scm-relay-data` must be writable by uid 10001.** It has been root-owned
   since the container ran as root. Fix without a privilege wrapper on the host
   (the docker group is enough):
   ```bash
   docker run --rm --user root -v /opt/scm-relay-data:/d \
     testbotz/scmessenger:<tag> chown -R 10001:10001 /d
   ```
2. **Use the image's own config dir** (`/home/scm/.config/scmessenger`), not `/root/...`.

Identity does NOT live in the config dir. It survives container replacement
because it is in the mounted sled store under `/data` — proved by the 09-17 cutover:
the node came back with the same `identity_id`, `libp2p_peer_id` and `public_key_hex`
after `docker rm -f` + fresh container on a new image. Do not treat the config
dir as the identity-bearing path, and do not skip the `/data` mount.

```bash
docker pull testbotz/scmessenger:<newtag>
docker rm -f scm-node
docker run -d --name scm-node --network host --restart unless-stopped \
  -v /opt/scm-relay-data:/data \
  -e LISTEN_PORT=9000 -e RUST_LOG=info -e SCM_LISTEN_PORT=9876 -e SCM_P2P_PORT=9002 \
  -e SCM_HTTP_HEALTH_PORT=9876 -e SCM_CONFIG_DIR=/home/scm/.config/scmessenger \
  -e SCMESSENGER_DATA_DIR=/data -e SCM_WASM_PORT=9003 -e SCM_DATA_DIR=/data \
  testbotz/scmessenger:<newtag> scm start
# verify identity is unchanged, then confirm RestartCount stays 0
docker inspect scm-node --format 'RestartCount={{.RestartCount}} Started={{.State.StartedAt}}'
```

Rollback: the previous image stays in `docker images`; re-run with the old tag.
Never `docker system prune` on this host before the checkpoint is written.

## Node 3 — Pixel 6a (operator-owned)

1. Build APK from this SHA:
   ```powershell
   .\gradlew :app:assembleDebug
   ```
2. Operator installs via adb.

   **Operator ruling, 2026-09-17:** the agent lanes are ALWAYS allowed to
   install an APK and to pull logs (`adb install`, `adb logcat`, `run-as … cat`,
   `adb pull`). No other device driving — no forcing service start/stop, no
   settings changes, no uninstall without a separate explicit approval.
3. Record `pm path` APK SHA256 and post-install service identity (E5).
4. Verify: app diagnostics git hash equals candidate; peersDiscovered matches fleet.

### PRECONDITION: the debug keystore must match, or the install cannot happen

Check this before promising a Pixel cutover (measured 2026-09-17):

| APK source | Signer #1 certificate SHA-256 |
|---|---|
| CI `android-debug-apk` artifact | `47a84596e934e98293252e3874ecae4c2e2a9c3715a8ea7ce64963357c9a6097` |
| Local `gradlew :app:assembleDebug` | `1cdef09cd3b80f9b686e5f9e7b760d360b1fd338c6720bcb59b15b233967835f` |

CI generates its own debug keystore, so a CI-built APK produces:
`INSTALL_FAILED_UPDATE_INCOMPATIBLE: Existing package com.scmessenger.android
signatures do not match newer version; ignoring!` — and the only way past it is
`adb uninstall`, which destroys the phone's on-device node identity, ledger and
contacts. That is an identity change on a node, i.e. a hard stop under
"Stop conditions" above; do not do it as a side effect of a routine cutover.

Two clean routes:
- **Identity-preserving (default):** build the APK locally at the candidate
  (`gradlew :app:assembleDebug`) so it carries the device's existing debug
  keystore, then `adb install -r` over the top. The build's embedded git hash
  must be made to match the other two nodes (check out the candidate commit
  itself, not a synthetic merge commit, or accept the provenance note).
- **CI-artifact install:** only with explicit operator approval of the identity
  reset, or after CI is given the operator's debug keystore as a secret so both
  sides sign identically.

Verify with:
```bash
"$LOCALAPPDATA/Android/Sdk/build-tools/35.0.0/apksigner.bat" verify --print-certs <apk>
```

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

- Measure before you build: `python scripts/disk_budget.py`. It reports the
  free space, every reclaimable `target/` in this checkout and in each
  registered worktree, and exits 2 when the host is below the hard floor
  (AGENTS.md rule 17). On 2026-09-17 this host hit 100% (1.2 GB free) with
  20.7 GB of build output in the main tree alone, and `du` could not even
  finish a survey of the worktrees.
- Reclaim with `python scripts/reclaim_safe.py --reclaim` (build output in
  provably-safe worktrees only). `scripts/clean_target.sh` is the blunter
  fallback and stays scoped to **this** workspace's `target/`.
- Prefer the CI artifact to a local build whenever one exists:
  `gh run download <run-id> -n <artifact-name> -D tmp/<dir>`. That is how the
  09-17 cutover avoided every large build on a nearly-full disk.
- Local runs are for a single targeted test, then wipe. The wide sweep is CI's
  job.
- Never delete uncommitted work, identity keys, rollback artifacts, `tmp/`
  evidence, or `~/.scm-purge-backup-*`. A full disk is not an exception.

## Stop conditions

Artifact hash mismatch, identity change on any node, disk <10 GB, or
entry-gate failure after one retry → write checkpoint and hand back.
