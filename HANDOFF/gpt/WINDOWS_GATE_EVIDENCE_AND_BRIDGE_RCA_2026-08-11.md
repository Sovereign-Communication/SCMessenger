# Windows lane -- gate evidence + inbox-bridge RCA (2026-08-11)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Last updated: 2026-08-11

Written for the GPT-MAC lane so both lanes hold the same record. Supplements
`WINDOWS_PR139_RESUME_STATE_2026-08-11B.md`; corrections to that document are
called out explicitly below.

## Terminology -- there is no relay

**This fleet has NO relay host. It has NODES only.** Operator correction, 2026-08-11.
Earlier Windows-lane messages and docs (this one included, before this revision) called
`54.226.67.101` "the AWS relay". That is wrong; it is the **AWS node**, nickname
`scm-always-on-node`. The container is named `scm-relay` for historical reasons and that
name should not be read as a role. Anywhere the older records say "relay", read "node",
and re-weigh the evidence accordingly -- see the raised-severity finding below.

## Headline corrections to the record

1. **The mesh-formation blocker is CLEARED.** `WINDOWS_PR139_RESUME_STATE_2026-08-11B.md`
   records Android at `0 peers (Core), 0 full, 0 headless`. A fresh logcat pull on
   2026-08-11T18:19:26Z shows all 20 `Mesh Stats` samples in the window reading
   **`3 peers (Core), 3 full, 0 headless`**. Android is forming p2p-circuit paths
   through both the Mac node (`12D3KooWNC5r...`) and the AWS node (`12D3KooWPJK6...`).
   The device is still dual-homed (`192.168.0.135` LAN plus `10.8.223.228`), so the
   VPN lead did **not** have to be resolved for peers to form. That lead is no longer
   the blocker and should be de-prioritised.

2. **`acked_without_receipt_protection` did NOT clear on `c93dfec5`.** Defect 3 of the
   resume doc stands. The device still holds a backlog logging
   `transport-acked message cannot be downgraded acked_count=1`, one message at
   `attempt=6`. Receiver-backed receipt evidence for Android is therefore still absent.

3. **AWS node SSH details in the record are wrong.** The live host is
   `ec2-user@54.226.67.101` and docker requires `sudo`. The documented
   `ubuntu@54.242.56.150` times out on port 22, and `ubuntu@` is rejected on the live
   host.

4. **There is no `/version` endpoint.** `/api/version`, `/api/health` and `/api/info`
   reset the connection on the CLI control API; only `/api/identity`, `/api/peers`,
   `/api/history` and `/api/send` are served. Gate requests asking for `/version` on
   the Windows node cannot be satisfied as written. GPT-MAC has dispositioned this as
   an API gap, not as evidence against the node.

## Gate evidence (2026-08-11, no PASS claimed -- gate remains CLOSED)

### Windows node

| Field | Value |
|---|---|
| Binary | `%LOCALAPPDATA%\scmessenger\soak\bin\scmessenger-cli-053fd137.exe` (PID 25268) |
| SHA-256 | `e9501c5c67fe867fb466f5f5ca7dd5c24e64c008b3d794b059686961abc72f7f` |
| Repo head | `86fa1f7be0153e196350ca56164d44168d93f2a9` (`tracking/pre-v040-tag-work`) |
| Identity | `985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826` |
| Peer id | `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw` |

The SHA-256 matches the recorded pin exactly. The binary is built from `053fd137`,
which is **not** head. Per GPT-MAC: record `053fd137` as the deliberate Windows/AWS
parity pin, not as PR-head provenance.

### Android

| Field | Value |
|---|---|
| APK SHA-256 | `7c89f7a77177bd50eb95c9fae286dbf702e8d9029175befa0fa50f570bc45392` |
| Version | `versionName 0.4.0`, `versionCode 14`, minSdk 26, targetSdk 35 |
| Installed | 2026-08-11 00:43:13 device-local (HST) |
| Peer id | `12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG` |

The hash was computed twice and agrees: device-side `toybox sha256sum`, and an
`adb pull`ed copy hashed on Windows.

**Build provenance: BOUND to `053fd137`.** Extracted from the APK itself:

```
053fd137:tracking/pre-v040-tag-work:1786442355
```

Build time `1786442355` = 2026-08-11T09:59:15Z.

`core/build.rs` stamps `SCM_BUILD_STAMP` as `hash:ref:build-time` and
`get_build_provenance()` in `core/src/lib.rs` exposes it via `option_env!`. The stamp
is compiled into the native core, so it is recoverable from
`lib/arm64-v8a/libscmessenger_core.so` inside the APK. Android also surfaces it live
through the uniffi binding on the Settings screen, which is an independent on-device
confirmation.

**CORRECTION.** An earlier revision of this document, and the Windows-lane messages
that fed it, stated the APK "cannot be bound to a commit". That was wrong. It was true
only of `logcat` and `dumpsys`, which is where the Windows lane looked; the binding was
in the artifact the whole time. `HANDOFF/todo/P1_STALE_BUILD_PROVENANCE_INVALIDATES_SHA_CLAIMS_2026-08-09.md`
does **not** apply to this APK. Do not dispatch the Mobile workflow merely to recover
provenance.

**Caveat to carry into the manifest:** the stamp is self-attested by the build host's
git at compile time, not cryptographically bound. It is the same class of evidence as
the CLI provenance line accepted for the AWS node image -- no stronger.

**GAP -- receiver-backed receipts.** Not available; see correction 2 above.

### AWS cloud node

| Field | Value |
|---|---|
| Container | `scm-relay` (name only -- this host is a NODE, not a relay), image `testbotz/scmessenger:sha-053fd13`, Up 8 hours |
| Repo digest | `testbotz/scmessenger@sha256:7e9e3d75490d83f24a8b3b4f553362b5b68fff1682fddaa3eab048ebe8d61e16` |
| Local image id | `sha256:bddc67db4d801f631ac5ec8292a45ff3e2bfa123427f309f1165b6c68a1c2214` |
| Health | `/api/health` -> `{"status":"healthy"}` |
| Peers | 3 -- Windows, Mac, ChristyLove |

**BLOCKER -- this node reports no identity.** `/api/identity` on the AWS node returns
`initialized: false` with `device_id`, `identity_id`, `libp2p_peer_id` and
`public_key_hex` all `null`, nickname `scm-always-on-node`, while the container is
healthy and actively forwarding traffic. GPT-MAC initially ruled that receipt evidence
from this host **does not count toward the gate** until the identity is resolved or
explicitly dispositioned.

**Severity raised 2026-08-11 on operator correction.** There is NO relay host in this
fleet -- there are only NODES. The `scm-relay` container name is historical and
misleading. Because this host is one of the five NODES, an identity of `null` is not a
footnote about third-party receipt evidence: it is a **node-parity blocker**, since one
of the five cannot present an identity at all.

### Root cause: the container runs the wrong subcommand

Diagnosed 2026-08-11, read-only, nothing on the host was modified.

```
CMD = ["scm","--http-bind","0.0.0.0:9876","relay",
       "--listen","/ip4/0.0.0.0/tcp/9001","--http-port","9000",
       "--name","scm-always-on-node"]
```

Per `scm --help`: `relay` = "Run headless relay node (no interactive console)",
`start` = "Start P2P messaging node", `init` = "Initialize new identity". Storage holds
`relay_network_key.pb` and **no node identity**. `/api/identity` therefore reports
`initialized: false` because this host was never initialised as a node -- it is running
as a relay, using a relay network key in place of an identity.

Nothing is corrupt. It is the wrong ROLE for a fleet the operator defines as nodes-only.

### Host inventory (before any change)

| Item | State |
|---|---|
| `scm-relay` | LIVE, `testbotz/scmessenger:sha-053fd13`, Up 8 h, created 2026-08-11T10:44:18Z |
| `scm-relay-old-6b2573fa` | STOPPED duplicate, `:latest`, Exited(137) 42 h ago, created 2026-08-06 -- removal target |
| Images | `sha-053fd13`, `sha-68fcc3f`, `sha-d48558a`, `latest` |
| Data | `/opt/scm-relay-data` bind-mounted to `/root/.local/share/scmessenger`; `logs`, `outbox`, `peers.json` (564 KB), `relay_custody`, `storage/{db,ledger.json,relay_network_key.pb,conf}` |
| Live nodes | exactly ONE |

### Approved scope for the teardown and fresh install

Recorded here because GPT-MAC gates execution on the anchor and scope being written
down, and because the Windows lane will not act on a scope held only in chat.

- **Operator authorisation:** granted directly in the operator channel 2026-08-11,
  standing and in perpetuity, for teardown and fresh install of the AWS **node**. It
  does not extend to any other destructive action.
- **Role target, decided by GPT-MAC:** option (a) -- provision an identity (`init`) and
  run `start` as a node. `initialized: false` is dispositioned as a **wrong-role
  condition**, not a fault.
- **Accepted consequence:** the PeerId WILL change. `12D3KooWPJK6...` is currently the
  bootstrap address other nodes rely on, including Android's circuit paths, so
  bootstrap/circuit configuration must be updated everywhere. This is a topology
  change, not a redeploy.
- **Preserve:** `/opt/scm-relay-data` identity/history and all unrelated host
  resources.
- **Remove:** only the identified stopped duplicate `scm-relay-old-6b2573fa`.
- **Prove:** exactly one live node, before and after.
- **Hold:** G1-G6 stays stopped until the new identity, bootstrap updates,
  health/version and all artifact provenance are re-verified.

### EXECUTED 2026-08-11T19:20-19:26Z -- role transition complete

Run under the operator's standing authorisation and GPT-MAC's recorded scope. Route 2
is selected for the artifact anchor, but this role change is independent of it: the
image was NOT rebuilt.

| | Before | After |
|---|---|---|
| Invocation | `scm --http-bind 0.0.0.0:9876 relay --listen /ip4/0.0.0.0/tcp/9001 --http-port 9000 --name scm-always-on-node` | `scm --http-bind 0.0.0.0:9876 start --port 9000` |
| Identity | `initialized: false`, all fields null | `initialized: true` |
| PeerId | `12D3KooWPJK6KgKsafefLWeGs4kVbj7wBnU67yKe88ni3FHZ3Hr2` | `12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy` |
| Health route | `/api/health` | `/health` |

- `identity_id` `0b33200936f41deb55e674e1d798b5c2aac7494a8a95ea34cd59c3b013c226ad`
- `public_key` `8db1612aa6330be410f7f181a43ee4743b23045bb1d3c69594d864c37b28f92c`
- `device_id` `e7a76bf1-2742-43d1-9a97-bf12f90a4b61`, seniority `1786476044`
- **New bootstrap multiaddr:**
  `/ip4/54.226.67.101/tcp/9001/p2p/12D3KooWKMUXfjvWeodBUJbSwBuRXBU3d6XSbP1AJXL9WhaS3yKy`
  IP and port are UNCHANGED; only the `/p2p/` suffix moves.
- Listeners: p2p `0.0.0.0:9001` v4+v6 (same port as before), control API `0.0.0.0:9876`,
  WS `127.0.0.1:9000`.
- Image UNCHANGED: `sha-053fd13`, digest
  `sha256:7e9e3d75490d83f24a8b3b4f553362b5b68fff1682fddaa3eab048ebe8d61e16`.
- Data preserved: `/opt/scm-relay-data` intact (`relay_network_key.pb`, `ledger.json`,
  `peers.json`, `outbox`, `relay_custody`). Pre-change backup
  `/opt/scm-node-data-backup-20260811T1920Z.tar.gz`, 6593947 bytes, sha256
  `aaa2f9fd7ef82715037a2439983157f2298403d32e170e07f007f93cfb680a2e`.
- Exactly ONE live node: `scm-node`. Duplicate `scm-relay-old-6b2573fa` REMOVED as
  approved. Former container retained STOPPED as `scm-relay-preroleswitch-20260811`;
  GPT-MAC has directed that it be kept for rollback until parity is proven.

**Health probe change -- action required for any monitoring.** Under `start` the health
route is `/health`. `/api/health` now returns EMPTY. An empty `/api/health` must NOT be
read as node failure; probes hardcoding it will silently report nothing.

**Two errors made and corrected during execution**, recorded so they are not repeated:
`--port` sets the WS port and p2p binds to port+1, so an initial `--port 9001` silently
moved p2p to 9002; corrected to `--port 9000` to restore p2p on 9001. Separately, an
initial claim that the node was "isolated" was wrong -- it came from grepping the wrong
ports; p2p was bound to `0.0.0.0` throughout.

### STOP CONDITION -- bootstrap mismatch, unresolved

Halted here per GPT-MAC's instruction to stop on any bootstrap mismatch.

Every node's bootstrap still names the OLD PeerId. The Windows-owned config has been
updated (backup at `%APPDATA%\scmessenger\config.json.bak.preroleswitch-20260811`) but
**does not take effect until the Windows node restarts, which has NOT been done** --
that restarts the soak generation, and cascading restarts across the fleet is not the
Windows lane's decision. Mac, Android and iOS configs are not Windows-owned and were
not touched. GPT-MAC owns propagation to those three; Windows config activation will be
coordinated afterwards as one bounded change.

Observed mesh state at the stop point: Windows sees only the Mac node (`NC5r`); the AWS
node's `/api/peers` reads `[]`.

**There are NO embedded bootstraps -- this changes the Route 2 cost model.**
`cli/src/bootstrap.rs` declares `pub const DEFAULT_BOOTSTRAP_NODES: &[&str] = &[];` --
empty. The Docker `entrypoint.sh` comment claiming "Bootstrap nodes are now embedded in
the binary at build time" is FALSE. Every node depends entirely on its own
`config.json` or `SC_BOOTSTRAP_NODES`. **Rebuilding artifacts at the new integration
anchor will NOT propagate the new PeerId.** It must be written into each node's config
explicitly, on every node, whichever anchor is built.

**Open discrepancy, flagged not diagnosed.** The AWS node logged
`Connected to 12D3KooWD6vZ... via /ip4/147.81.41.188/tcp/9001 (promiscuous mode - any
PeerID accepted)` at 19:26:26Z while `/api/peers` returned `[]`. An established
connection absent from the peer list is either a peers-endpoint difference between
relay and start modes, or a real registration gap. If `/api/peers` is the matrix's
evidence source it may under-report, so this must be reconciled before G1-G6.

### Open decision -- which anchor the five-node test runs on

All three artifacts are **already aligned on `053fd137`**: Windows binary, AWS node
image, and the Android APK (see the corrected Android section above). So there are two
routes and they differ materially in cost:

1. **Run on `053fd137`.** No rebuild, no CI dispatch. Parity already holds. Only the
   AWS role switch is needed.
2. **Move to a combined integration anchor** carrying the Mac connection-admission fix
   (`860f5ed5`) plus the Windows bridge fix. This requires rebuilding and redeploying
   **every** artifact on all five nodes.

Worth noting before that cost is accepted: the Windows bridge fix is
`scripts/inbox_bridge.py`, orchestration tooling that is **not compiled into any
artifact** -- `cli/` and `core/` do not reference it. It therefore has no bearing on
node parity and is not a reason to move the anchor. The only substantive question is
whether the five-node test requires the Mac connection-admission fix. That commit is
not present in this checkout (`git cat-file -t 860f5ed5` -> not a valid object), so the
Windows lane cannot assess it; GPT-MAC and the operator own that call.

**RESOLVED 2026-08-11: Route 2 selected by GPT-MAC.** Their justification: `860f5ed5`
addresses a libp2p crash/assertion risk in connection admission and must be inside the
tested artifact set; additionally the Mac CLI is already at `6e50963d` and the signed
physical-iPhone v2 was built from `860f5ed5`, so Route 1 at `053fd137` would force a
rollback and rebuild of the Mac and iPhone plus fresh Android attestation. A clean
isolated integration commit carrying the Mac admission fix will be staged and its exact
SHA declared before any build or deployment. `053fd137` is NOT the final five-node test
anchor. The bridge fix stays in the PR record as orchestration evidence but does not
drive artifact provenance.

## RCA -- the inbox bridge silently dropped every GPT-MAC message

**Symptom.** GPT-MAC sent a gate re-request that produced no HANDOFF ticket, no ACK,
and no wake. It was found only by reading `/api/history` by hand.

**Root cause.** `%APPDATA%\scmessenger\inbox_bridge.json` carried a single
`allowed_peer_id`, the Android identity `a43772fe...`. GPT-MAC's identity
`3854e442...` was absent. Design rule 4 of `scripts/inbox_bridge.py` gives
non-allow-listed senders no ticket and no ACK by intent, so the drop was silent and
correct-by-design. The status file recorded `ignored_inbound_in_window: 50` against
`allowlisted_inbound_in_window: 1`.

**Contributing cause.** The running bridge (pid 10340, started 09:48:58Z) was an
orphan, not the supervisor's child: `soak/status.json` showed `bridge_pid: null`.
`soak_supervisor.py` launches a bridge under `--with-bridge`, but the single-instance
lock was already held by the orphan, so its child exited repeatedly and the supervisor
gave up after 3 attempts (`bridge_failures > 3`). A config fix would therefore never
have been picked up by a supervisor respawn -- there was none to respawn.

**Fix.** `allowed_peer_id` is now a list carrying both identities. `cmd_run` reads
config once at startup, so a restart was required.

**Restart, performed in a controlled window 2026-08-11T18:50:03Z.** The node soak was
deliberately NOT touched (generation 8, started 17:26:14Z, healthy throughout); only
the orphaned bridge was replaced. New bridge pid 25740.

Verification, from the bridge status file immediately after restart:

| Counter | Before | After |
|---|---|---|
| `ignored_inbound_in_window` | 50 | **0** |
| `allowlisted_inbound_in_window` | 1 | **50** |

The traffic the bridge was silently discarding is now accepted.

**Standing risk this exposes.** A single-identity allow-list plus silent-by-design
drops means any new lane joining the fleet is invisible to the bridge until someone
notices the absence of replies. Consider surfacing `ignored_inbound_in_window > 0` as
a health signal rather than a passive counter.

## RCA 2 -- the allow-list fix triggered an unbounded ACK storm

Fixing the allow-list immediately exposed a second, latent defect. Recorded in full
because roughly 1400 junk messages were sent to the GPT-MAC lane as a result.

**Symptom.** Within about three minutes of the restart, `acks_sent_total` went from 15
to over 1400 and was still climbing at roughly 30 messages/second. Every one was a
`[SEEN] <id> received (logged, not queued as a task)` ACK addressed to GPT-MAC.

**Root cause.** `classify_content()` recognised only the `scm.message.*` envelope that
the mobile apps wrap all outbound traffic in. The CLI and desktop lanes emit **bare**
delivery receipts -- `{"message_id":..,"status":"Delivered","timestamp":..}` -- which
carry neither `kind` nor a `schema`+`text` pair. That shape fell through to `"content"`,
routed as `chat`, and was ACKed. The far node then emitted a delivery receipt *for that
ACK*, which arrived as fresh allow-listed inbound and was ACKed in turn. Unbounded
feedback loop.

**Why it was latent.** Android was the only allow-listed peer, and it wraps every
message -- receipts included -- in the envelope. The bare-receipt path had never been
exercised. Allow-listing any non-mobile peer would have triggered it.

**Fix (commit `11710cf3`).** `_is_delivery_receipt()` classifies a bare receipt as
housekeeping, so no ACK is emitted. Strict by construction: exact key-set subset of
`{message_id,status,timestamp}`, both required keys present, non-empty `status`, and any
text-bearing field disqualifies the match. A human message therefore cannot be swallowed
by this rule, preserving the conservative-by-construction invariant the classifier is
built around.

**Verification.**

| Check | Result |
|---|---|
| Routing suite | 58/58 pass (6 new regressions) |
| `acks_sent_total`, idle, 3 samples 40s apart | flat at 1466 |
| `acks_sent_total`, per genuine inbound message | +1 exactly (1466 -> 1467 -> 1468) |

Final bridge pid 2012, started 2026-08-11T18:54:20Z. The node soak was never touched
across either restart (generation 8, started 17:26:14Z).

**Note on ticket routing.** GPT-MAC's prose messages route as `chat`, not `handoff`, so
they are logged and ACKed but do **not** produce `HANDOFF/todo/INBOX_*.md` tickets --
only `/handoff`-prefixed messages do. Watching the ticket directory is therefore not a
valid wake mechanism for that lane; poll `/api/history` instead.

## Evidence artifacts

Held under `tmp/` on the Windows host and deliberately NOT committed -- `tmp/` is
gitignored, the raw captures are unredacted, and this repository is public.

| Artifact | SHA-256 |
|---|---|
| `tmp/gate_evidence_20260811T1848Z/bridge_status_prerestart.json` | `00fb3529421d036bb9a4f5480ec444692f910595aa33abd9a478eddedd04f5f6` |
| `tmp/gate_evidence_20260811T1848Z/bridge_config_prefix.json` | `6c03d31e1f970d31a2ce5b1bc9162b3e4dfc9deaf3930d90a41439a162e5ba84` |
| `tmp/gate_evidence_20260811T1848Z/bridge_config_current.json` | `5a4bbff4c08ba07b4197b3ac618a3279a6bf73639f6324373a2c88ea480a1b70` |
| `tmp/gate_evidence_20260811T1848Z/history_snapshot.json` | `0378d4f27ea972639e7519e3419670b3361cdf6c68a45243cdc4cfd9a1c40c73` |
| `tmp/androidlogs_20260811T181926Z/logcat_app.txt` | 304083 bytes, 1359 lines, span 18:03-18:18Z |

The full app-scoped logcat was transferred to GPT-MAC over the mesh in 87 chunks and
they confirmed receiver-side verification of all 87 plus the completion marker. The
2.8 MB full-system logcat is held on the Windows host and has not been sent.
