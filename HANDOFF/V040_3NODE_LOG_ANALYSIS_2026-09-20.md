# 3-node log analysis — 2026-09-20 (CTO pass)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

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

## Cross-node log analysis — rollout to `bceacb94`, 2026-09-26

Companion to the `bceacb94` section of `HANDOFF/V040_3NODE_RCA_2026-09-09.md`.
Same candidate SHA, same evidence directory `tmp/rollout-20260926-bceacb94/`.

**Scope honesty: this is a TWO-node analysis, not a three-node one.** The Pixel
leg did not run because **no device was attached** — `adb devices -l` lists no
devices after an adb server restart, and no Pixel/Android/ADB device is
enumerable over USB. No APK was installed and no handset logs were pulled, so
**the Pixel's behavioural proof awaits the hardware and the user.** The only
Pixel signal in this record is indirect and passive: it remains a live connected
peer of AWS (`12D3KooWDgLQ8jn8…` in the AWS peer list), observed from the other
two nodes without touching the device.

| Node | Version this pass | Evidence source |
|---|---|---|
| Windows CLI | `0.4.0 (bceacb9:main:1790455088)` | process/PID, binary sha256, `scm.log.2026-09-26-21` |
| AWS cloud node | `0.4.0 (bceacb9455fbf8aaaac2164059c4b7d8afcc226b:main:)` | `/version`, `/health`, `/api/identity`, `/api/diagnostics`, `docker logs scm-node` |
| Pixel 6a | not installed for this SHA | none — device absent |

### Windows node

- Alive and meshed for the whole window: PID 7140, binary sha256 `3a1ac11f…`
  (the CI artifact for this SHA), binary path and command line byte-identical to
  the pre-change node.
- Custody registration never lapsed: `[CUSTODY] Registered local identity with
  peer 12D3KooWGvCWJNo… (relay-ready)` on a 60 s cadence, count climbing 44 -> 46
  across the sampling window.
- `Relay custody audit log count: 4815` held flat; retention sweep ran clean
  (`0 of 40 record(s) expired, 0 bytes reclaimed`).
- `[OK] Relay circuit reservation ACCEPTED via 12D3KooWGvCWJNo…`, i.e. inbound
  relayed connections available despite `AutoNAT: behind NAT`.
- **0 `ERROR`, panic, or backtrace lines after the deploy** (window from
  21:47:48Z).

### AWS cloud node

- `healthy` at every sample; `RestartCount=0`, exit 0; mount
  `/opt/scm-relay-data:/data` rw=true; host uid `10001:10001` mode 755; container
  uid `10001(scm)`.
- `connection_path_state: DirectPreferred`, custody audit 3099, outbox 0, undelivered
  160 — all constant across 7 samples over 171 s.
- History survived the image change exactly (received 9703, sent 187, undelivered
  160 at T0 and identical at T1), the strongest single indicator that the
  persistent mount is still doing its job after the swap.

### Delivery, both directions

| Direction | Marker | Message id | Result |
|---|---|---|---|
| AWS -> Windows | `AWS2WIN-20260926T215328Z-BCEACB94` | `1474f2b5-7743-4246-b6f1-aacfc4632102` | `delivered: true`; Windows `inbox_receive` sender `37eb7561…`; ACK returned |
| Windows -> AWS | `WIN2AWS-20260926T215348Z-BCEACB94` | see `received_count` 9703 -> 9705 | `[OK] Message delivered … (267ms)`, `ROUTE_DECISION attempt=1 pass`; ACK returned |

Both nodes returned to `outbox_count=0` afterwards.

### Warnings, cross-referenced against tickets

| Warning | Ticket | Disposition |
|---|---|---|
| `[DIAL-BACKOFF] Peer marked as dead after 3 failed attempts` (recurring; peer `12D3KooWKrnxkGW…`) | `HANDOFF/todo/D2_SEED_DIAL_REDIAL_MISSING_SEED_PEER.md` (**not on `main`** -- filed on `fix/361-review-blockers` (PR #372, unmerged)) — quotes this exact line as its defect evidence | pre-existing, unchanged by this rollout |
| `ble_mesh: Windows GATT server / advertising error HRESULT(0x00000000)` | no dedicated ticket | **pre-existing, not a regression**: the outgoing `1bc78c8` binary emitted the byte-identical line at its own startup, with `terminal_result="no_adapter"` |
| `SEED-DIAL sweep 1: 104 candidate(s), peers=0` | D2 family | benign; custody registered regardless |
| `AutoNAT: behind NAT` | `HANDOFF/todo/D9_LIBP2P_EITHER_HANDLER_PANIC.md` (**not on `main`** -- filed on `fix/361-review-blockers` (PR #372, unmerged)); named there only in a libp2p behaviour list, not as a treatment of this warning | expected on a desktop; covered by the accepted circuit |
| `[CONN-CAP] closing redundant per-peer path` | `HANDOFF/todo/D7_CONN_CAP_REDUNDANT_PATH_CLOSE.md` (filed 2026-09-26); the Wave-1 note at lines 61 and 72 of this file covers the `max sub-streams` family but never named this line | working as designed |
| stale `13.217.204.112:8080` for the Pixel, `failure_count: 6` | `HANDOFF/todo/D11_STALE_LEDGER_PEER_ADDRESS.md` (filed 2026-09-26) | new observation this pass: an unreachable address retained alongside the Pixel's live ones |

### Why "no regression" here is operational, not diff-based

The previous Windows revision `1bc78c8` **does not resolve in this repository**:
`git cat-file -t 1bc78c8` reports "Not a valid object name" and no local ref
contains it. The outgoing and incoming trees therefore could not be diffed.
No-regression is established instead by live evidence — preserved identity on
both nodes, an authenticated mesh, bidirectional delivery with ACKs, receipt and
outbox cleanup, and constant-state sampling — not by reading a code diff.

Note also that the shared checkout on `glm/canonical-outlier-audit` @ `3f41005d`
is 148 commits behind `main` and does not contain the 2026-09-24 re-evaluation
sections that exist in its own working tree. This record was written against
`main`; that earlier unlanded content was left untouched.
