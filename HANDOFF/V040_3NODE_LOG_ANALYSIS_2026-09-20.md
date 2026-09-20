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
