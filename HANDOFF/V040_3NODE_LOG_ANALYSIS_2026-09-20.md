# 3-node log analysis — 2026-09-20 (CTO pass)

Mode: read-only. Not a full D4/D6/D7 tag scoring run (operator: score after
wave + phone session). Master plan: `HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md`.

## Fleet snapshot

| Node | Build | Identity | Health |
|---|---|---|---|
| Windows CLI | `0.4.0 (34b56d5:main)` via `/version` | `12D3KooWD6vZQrUq...` | healthy, running |
| AWS cloud | `0.4.0 (34b56d54...)` boot log at deploy | `12D3KooWGvCWJNo...` | healthy, mount OK |
| Pixel 6a | Pre-candidate app (not reinstalled) | `12D3KooWD776...` / Lucas | ADB connected; CI APK install **blocked** signature |

Deploy artifacts: `tmp/radio-candidates/34b56d54/`, AWS `scripts/aws_deploy.sh`
`IMAGE_TAG=testbotz/scmessenger:sha-34b56d5`.

## Windows node (API + logs)

`GET /api/diagnostics` (command this session):

- `running: true`, `connection_path_state: DirectPreferred`
- `peers`: cloud node `12D3KooWGvCWJNo...`
- circuit listener: `/ip4/18.234.62.247/tcp/9001/p2p/GvCWJNo.../p2p-circuit/p2p/D6vZQrUq...`
- `custody_audit_count: 6950`
- `history_stats`: total 10002, undelivered **261**, outbox_count **100**
- external_addrs: `192.168.0.121:8080` (LAN; no ephemeral NAT port in this snapshot)

Log hour `scm.log.2026-09-20-13` present; full SEED-DIAL grep on Windows log
file was truncated in this pass — cloud-side SEED-DIAL is authoritative below.

## AWS cloud node (`tmp/aws_log_slice.sh` → `docker logs scm-node`)

Commands: ssh + grep SEED-DIAL / custody / Gossipsub / limits (script on disk).

| Signal | Evidence |
|---|---|
| Seed dial | `[SEED-DIAL] peers=1 -- connected; re-check in 120s` repeated ~every 2 min |
| Gossip | `Gossipsub message from 12D3KooWD6vZQrUq... on topic sc-mesh` every minute |
| Custody | `Relay custody audit log count: 5811` then `5805` after retention |
| **TRN-04 live** | `Custody retention sweep: 590 record(s) scanned, none past the 604800000-ms window`; later `expired 0 of 590 ... 6 transition row(s) dropped (window 604800000ms)` |
| connection_limits in slice | **Not present** in the tailed grep window (not proof of zero fleet-wide) |

**Verdict:** cloud node on `34b56d54` is mesh-healthy with seed-dial + gossip +
**working custody retention** (audit CO-G-003 TRN-04 STILL-OPEN is disproved on
this binary).

## Pixel (passive `adb exec-out run-as ... tail mesh.log`)

| Signal | Evidence |
|---|---|
| Outbox to Windows | Flush keyed `30d0fa67...` (Windows **hex** pubkey); `outbox_flush_completed succeeded=1 failed=0` |
| Ledger gossip | `Sent peer list (2 peers) to 12D3KooWD6vZQrUq...` |
| Residual transport | `outbox_reconnect_failure ... max sub-streams reached` — consistent with stream/connection cap family (Wave-1 conn-limits ticket) |
| Maintenance | `Maintenance removed 0 expired outbox messages` on 15-min cadence |

**Not same-SHA** as Windows/AWS until APK install unblocked.

## Mesh-level conclusions (this pass)

1. **Tier A (Windows + AWS) same SHA `34b56d54` and meshed:** seed-dial PASS,
   gossip PASS, custody retention PASS live, identity stable.
2. **Pixel participates** in gossip/outbox flush to Windows but is **not**
   on the candidate APK (keystore secret).
3. **Known residuals for Wave-1:** connection/stream caps (`max sub-streams`,
   historical `limit 4` storm), AndroidTest Mobile red, AND-06 Kotlin math,
   IP-churn autonomy, beach-join P0–1.
4. **Not yet tag-ready:** checklist in master plan §4; operator phone session
   after Wave-1 + keystore.

## PR / CI train status (orchestrator)

| Item | State |
|---|---|
| #337 audit lineage | MERGED `1caee28c` |
| #338 CTO master plan | MERGED `e7b42317` |
| #339 CO-B-001 dual-drain | OPEN — staged, unit test green locally |
| #336 canonical audit docs | MERGED earlier |
| Mobile APK | Artifact exists; install blocked on signature |
| Freebuff Wave-1 | Tickets DISPATCHABLE on main — operator paste |

Harness: `sovereign-harness` 0.3.3 in sync with origin/main.

## Windows rollout re-evaluation - 2026-09-24

**Disposition: KEEP `1bc78c85`; no Windows replacement or restart.**

The Windows artifact inventory and evidence are recorded in
`tmp/windows-rollout-20260924/ROLLOUT_RCA.md`. The live Windows binary is
`tmp/radio-candidates/1bc78c85/scmessenger-cli.exe`, SHA-256
`454cc346811f30a51e4a42bfb7f51f7daf458313f82ad7a5d2a8477de1fab061`.
It reported `0.4.0 (1bc78c8:HEAD:1790184283)` and was the only inventoried
Windows artifact with a complete live proof of health, identity preservation,
mutual authenticated peers, bidirectional delivery, receipt cleanup, and
sustained stability. The other complete binaries were older and had no live
superiority proof. The selection was based on proven functionality, not a
requirement to match the Pixel SHA.

### T0 baseline

Fresh evidence is under `tmp/windows-rollout-20260924/t0/`:

- Windows PID `19572`, path
  `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\radio-candidates\1bc78c85\scmessenger-cli.exe`,
  SHA-256 `454cc346811f30a51e4a42bfb7f51f7daf458313f82ad7a5d2a8477de1fab061`.
- Windows identity `985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826`,
  peer `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`.
- AWS `/version` `0.4.0 (45b0f8b8a8019f94b9c39105e45a84315ef314b7)`;
  both nodes healthy, running, `DirectPreferred`, and mutually authenticated.
- T0 diagnostics: Windows custody `4972`, history `9625/377`, undelivered
  `259`, outbox `0`; AWS custody `2676`, history `8975/185`, undelivered
  `160`, outbox `0`.

### Controlled result

The Windows-to-AWS marker `WIN2AWS-20260924T021249Z-ROLLOUT` used message ID
`1b52411e-e0d1-4890-bc04-a6e261ec9283`; it reached `delivered=true` in the
Windows status/history and in AWS received history. The AWS-to-Windows marker
`AWS2WIN-20260924T021322Z-ROLLOUT` used message ID
`3ba275b2-e8d3-4ab5-81ba-7a829820b431`; its first dispatch returned
`retrying` with `Peer not connected over BLE`, then the retry delivered it and
cleared the outbox. Both sides reported `outbox_count=0` after receipt.

Seven samples from `02:14:12Z` through `02:15:29Z` (77 seconds) recorded
healthy/running nodes, unchanged PID/path/hash/start time, unchanged versions
and identities, mutual authenticated peer triads, and zero outbox on both
nodes. No panic, backtrace, or `ERROR` appeared in the bounded post-test logs.

### RCA and residuals

Compared with the 2026-09-20 record, the old Windows snapshot had
`outbox_count=100` and `undelivered=261`; the current pass started with
`outbox_count=0` and `undelivered=259`, then proved receipt cleanup for both
controlled directions. The improvement is verified operationally, not inferred
from a version label.

The A2W first attempt selected an unavailable BLE path and recovered through
retry. A stale self-dial warning also recurred at `02:13:47Z` after the
exchange, following the same warning in the pre-test baseline at `02:02:47Z`.
The AWS log had one bounded connection-negotiation/AutoNAT warning at
`02:12:28Z`, followed by connected seed-dials and successful delivery. These
are recorded residuals for follow-up; they did not produce a crash, identity
change, delivery loss, or outbox accumulation.

Pixel and OpenClaw were not accessed in this pass. No AWS deployment, restart,
or identity change was made. No build was run because the disk guard was
BLOCKED.

## AWS rollout re-evaluation - 2026-09-24

**Disposition: KEEP `testbotz/scmessenger:sha-45b0f8b`; no AWS deployment or restart.**

The complete 25-image inventory and non-secret image metadata are under
`tmp/aws-rollout-20260924/image-inventory.txt` and
`tmp/aws-rollout-20260924/image-metadata.txt`. The live image ID is
`sha256:5403c7f98e13f44a8337c9bb29b2dd6885dfc0d519aa6b9c2603cfd28b00dff2`,
revision `45b0f8b8a8019f94b9c39105e45a84315ef314b7`, created
`2026-09-23T17:35:14Z`. It is the newest available image by recorded image
creation time and the only image with a complete live proof of authenticated
bidirectional delivery, receipt cleanup, zero outbox, and sustained stability.
The older `56d66f7`, `6490bed`, `51edac4`, `34b56d5`, and remaining images had
no verified functional improvement. Selection was based on proven behavior,
not a requirement to match the Pixel or Windows SHA.

### T0 container and node baseline

Evidence is under `tmp/aws-rollout-20260924/t0/`, captured at
`2026-09-24T02:18:50Z`:

- Container `scm-node` was running with image `sha-45b0f8b`, image ID
  `sha256:5403c7f...`, user `scm` (`uid=10001`), exit code `0`, and
  `RestartCount=0`.
- Persistent mount was `/opt/scm-relay-data -> /data`, writable. Host ownership
  was `10001:10001`, mode `755`; container identity was `uid=10001(scm)`.
  The captured environment included `SCM_DATA_DIR=/data` and
  `SCM_CONFIG_DIR=/home/scm/.config/scmessenger`.
- AWS `/version` was `0.4.0 (45b0f8b8a8019f94b9c39105e45a84315ef314b7)`;
  `/health` was healthy; diagnostics showed custody `2676`, history
  `8979/186`, undelivered `160`, and `outbox_count=0`.
- AWS identity remained device `8a6b20bb-5f9f-46c4-8a6c-6aece03ce427`,
  identity `37eb75612c179d57a83801ba71f3545ffdf3d009adbf73c652dae37536b1d006`,
  peer `12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31`, and public
  key `69805e175cdc59b244f36001303e0b2e735f729557d2069a02688c0e69764a7c`.
- AWS and Windows exposed each other's authenticated self-certifying peer
  triad. The AWS node also had one other pre-existing peer; it was not used as
  a target.

No backup or replacement was initiated because the current image was retained.
The existing image ID/tag and persistent mount provide the rollback reference;
any future replacement must repeat the identity, ownership, and `/data` mount
gates before cutover.

### Controlled result

AWS to Windows used marker `AWS2WIN-20260924T021915Z-AWS-ROLLOUT` and message
`9bc99932-ca54-49b8-9ae7-51eab65c911d`. It reached `delivered=true` in AWS
sent history and Windows received history. Windows to AWS used marker
`WIN2AWS-20260924T022000Z-AWS-ROLLOUT` and message
`dbc8f568-d830-4ee8-8939-dff4bd95bc68`; it reached `delivered=true` in
Windows sent history and AWS received history.

The AWS and Windows logs contain successful delivery lines, delivery ACKs, and
`receipt_outbox_cleared` events for the controlled messages. Both nodes
returned to `outbox_count=0`; undelivered counts remained AWS `160` and
Windows `259`.

Seven samples from `02:20:51Z` through `02:22:22Z` (91 seconds) recorded
healthy/running nodes, stable AWS container state, unchanged image/version,
unchanged identities, mutual authenticated peer triads, unchanged Windows PID
and binary hash, and zero outbox on both nodes. The final recheck at
`02:22:50Z` still showed the same image ID, `RestartCount=0`, persistent mount,
and healthy APIs. No panic, backtrace, or `ERROR` appeared in the bounded AWS
final log. One non-fatal connection-negotiation warning at `02:22:28Z` was
followed by `SEED-DIAL peers=2 -- connected` at `02:22:31Z`.

### RCA and residuals

The 2026-09-20 record had AWS `34b56d54` health and custody evidence but no
controlled bidirectional receipt proof. The current result adds explicit
message IDs, delivered status on both sides, delivery ACK/receipt logs, and
outbox cleanup while preserving the AWS identity and data mount.

The bounded connection warning is a transport fallback residual, not a
sustained failure. The current image's outbox-sweep behavior was directly
exercised and returned both nodes to zero outbox. No image change is justified
without a separately verified functional improvement.

Pixel and OpenClaw were not accessed. No AWS deployment, restart, image
removal, or identity change was made. No build was run; this pass used the
existing image inventory and live API/container evidence.
