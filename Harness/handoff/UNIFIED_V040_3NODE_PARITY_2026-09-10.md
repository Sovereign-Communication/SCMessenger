# Harness handoff — unified v0.4.0 3-node parity candidate

Status: Active
Created: 2026-09-10
Workspace: `C:\Users\SCM\Documents\GitHub\MiMoSCMessengerFresh`
Branch: `unified/v040-3node-parity`
Tip: `9fec7b77`
Gates: cargo check PASS; observation unit tests 11/11 PASS
Deploy runbook: `HANDOFF/UNIFIED_V040_3NODE_DEPLOY_RUNBOOK_2026-09-10.md`

## What this is

Operator ordered a cherry-pick/unify of the complete app across all diverged
lines so all 3 nodes can run one candidate tree.

Plan file: `HANDOFF/UNIFIED_V040_3NODE_PARITY_PLAN_2026-09-10.md`

## Lines unified

| Line | Tip | Role |
|---|---|---|
| CTO 3-node campaign | `66a09cbc` | Baseline (last live operator PASS) |
| `origin/main` | `c5b7c530` | T1-T13 trunk (#261-#277) |
| `cto/v040-candidate` (#272) | `5fe7d664` | outbox, mobile_bridge, iron_core, FFI, dead-mark, never-dial |

Merge commits:
1. `b22dcaff` — main into CTO baseline
2. `3c2140d3` — v040-candidate into the above

## Conflict decisions (authoritative for re-apply)

- `local.rs`: main hint `[u8;8]` + shared `sort_by_reliability` + v040 `total_cmp`
- `ledger_entry.rs`: demote-not-exclude (CTO D3c) + `is_bootstrap` (main)
- `observation.rs`: configured-external primacy (CTO) + empty-listen-set wasm semantics (main/T14)
- `swarm.rs`: keep T14 `set_configured_external_address` + D10b; use `listen_ports_from_multiaddrs`
- `AndroidPlatformBridge.kt`: no lifecycle pause (CTO) + no FFI echo (v040 R10-F3)
- Removed orphaned `SwarmHandle::add_kad_address` (no callers; enum variant dropped by #272 merge)

## Gates already run (this session)

- `cargo check -p scmessenger-core -p scmessenger-cli` — **PASS** (after AddKadAddress fix)
- Pending: targeted tests (`test_address_observation`, `test_configured_external_address`)
- Pending: Android `:app:compileDebugKotlin` / assembleDebug
- Pending: full core suite if disk allows (E9 wants >=20 GB; currently ~17 GB free)

## Disk discipline (operator ruling)

- Do **not** `rm -rf` anything outside confirmed SCMessenger build caches.
- Prefer `scripts/reclaim_safe.py` / `scripts/clean_target.sh --all` only, and only
  on trees that are committed/pushed.
- Keep rollback binaries **outside** `target/` (RCA W4).
- Current free: ~17 GB — enough for check + targeted tests; full Android + full
  suite may need reclaim first.

## 3-node parity deploy (after remaining gates)

One SHA for all three nodes:

1. **Windows CLI** — build `scmessenger-cli` at unified SHA; stage rollback copy
   outside `target/`; relaunch with recorded bootstrap env/config.
2. **AWS** — docker image at same SHA via `.codebuff_deploy/aws/`; capture
   `sha256sum` + `/version` (E3/E4).
3. **Pixel 6a** — operator installs APK from same SHA; record `pm path` hash (E5).

Success: all three report the same git hash, then re-run 3-node matrix
(baseline / cell-only / BLE after radio reboot).

## Known open items NOT closed by this unify

- Rule-8 D10 review still PENDING (merge-to-main gate)
- mDNS advertisement `TxtRecordTooLong` filter follow-up
- Windows BT radio hardware wedge (operator reboot)
- Final `v0.4.0` tag (CEO/operator)

## Resume commands

```powershell
cd C:\Users\SCM\Documents\GitHub\MiMoSCMessengerFresh
git status
git log -3 --oneline
cargo check -p scmessenger-core -p scmessenger-cli
```
