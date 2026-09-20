# Freebuff lane -- live queue

Status: Active
Last updated: 2026-09-20
Rules: `docs/rules/FREEBUFF.md` -- read it before adding a task file here.
**Order authority to the tag:** `HANDOFF/V040_TAG_PATH_UNIFIED_2026-09-20.md`
(this index must not invent work that file does not list).

## Durable CTO continuation

The tracked `/cto` entry point is `.claude/commands/CTO.md`. Its single-owner
three-node BLE workflow is `HANDOFF/V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md`,
with ownership/data flow in `HANDOFF/V040_CTO_BLE_ARCHITECTURE_2026-09-08.md`.
Fresh sessions must read those tracked files and create append-only
`HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_<UTC-BASIC>_<STAGE>.md` files. Files under
`tmp/` are historical evidence references only. Freebuff is interactive and has
no headless dispatch mode under the current lane rules, so the operator pastes
the tracked command/package contents into Freebuff; no ignored `.freebuff/`
file is authoritative.

This is the unmetered implementation lane. Models: **DeepSeek V4 Flash**, MiMo,
GLM 5.3 Flash. The `freebuff` CLI has no headless mode, so **the operator is the transport**: an agent writes the task file, the operator pastes it into Freebuff desktop. Every paste cycle costs operator attention -- a task file that sends the model down the wrong path is the expensive failure here.

```
queue/   ready to paste, run in the order below
inbox/   Freebuff writes back here -- questions, blocked reports, wrong premises
done/    completed; Status line records the PR number
```

**The return path matters.** If a task file's premise does not survive contact
with the code, Freebuff should stop and write to `inbox/` rather than implement a
fix to a problem that does not exist. A watcher on this folder wakes the
orchestrator session when a reply lands. See `inbox/README.md` for the format.

**Status-line gate:** before any paste wave, `python scripts/check_queue_status.py`
must exit 0. Stale "PR open" lines misdispatch this lane.

---

## DISPATCHABLE -- paste only these (2026-09-20)

| Order | Task file | What it fixes | Review gate |
|---|---|---|---|
| 1 | `queue/V040_QUEUE_STATUS_RECONCILE_TAGPATH.md` | Stale ticket Status lines + README false open-PR claims | none (docs) |
| 2 | `queue/V040_T_CONN_LIMITS_MULTIPORT.md` | Live `connection_limits: limit 4 reached` multi-port dial denial | **Rule-8** |
| 3 | `queue/V040_T_WATCHDOG_POSITIVE_TEST.md` | N-03 healthy-quiet node must not be watchdog-killed | none if test-only |
| 4 | `queue/V040_T13_RESIDUAL_VERIFIED.md` | **Only if** orchestrator files residual T13 items still real on main | Rule-8 as filed |

Operator decisions that are **not** freebuff pastes:
`inbox/V040_OPERATOR_DECISIONS_TAGPATH_2026-09-20.md` (keystore/D2, SEC-03,
AND-06 A/B, audit commission, PR closes).

Scoring gates D4/D6/D7, churn, fleet redeploy, and the final `v0.4.0` tag are
**operator + orchestrator**, not this lane.

---

## DO NOT PASTE -- code already on main (status rewrite only)

Verified against `origin/main` 2026-09-20 (`efd240d7`) and GitHub PR merges.
Implementation re-dispatch of these burns paste cycles.

| Ticket | Why not to paste |
|---|---|
| T1 `V040_T1_NODE_BOOT_SEED_DIAL.md` | Seed dial on main: `cli/src/seed_dial.rs`, `swarm.rs` `build_seed_dial_candidates` / `connect_to_seed_peers` |
| T2 `V040_T2_UNIFY_PEER_LEDGER_STORES.md` | Core ledger unification + legacy `peers.json` migrate/archive on main |
| T4 `V040_T4_ROUTING_FEED_ON_CONNECTION_ESTABLISHED.md` | Premise contradicted; production `routing_peer_seen` on main. D6 needs **scoring**, not another wire-up |
| T5 docs sync | `docs_sync_check.sh` PASS on main worktree 2026-09-20 |
| T6 harness | Merged #311 |
| T7 Android staging | Merged #312 -- Status was stale |
| T8 diagnostics test | Merged #271 |
| T10 FFI vacuous | RESOLVED ON MAIN |
| T11 docs reconcile | Merged #314 |
| T12 CI filters | Merged #319/#328 |
| T14 ephemeral + DHT tickets | Merged #270 / #269 |
| All `V040_REVIEW_DISPATCH_*` | Reviews already filed; do not re-paste completed qwen dispatches |
| BJ beach-join | Post-tag / post-mission unless operator moves it |
| C4 identity-aware relay admission | 0.5.0 lane |

---

## Post-tag / non-blocking

| ID | Notes |
|---|---|
| T9 PR queue burndown | Still real; orchestrator batch disposition in unified path §4 -- freebuff may help **docs/close comments**, never bulk-merge CONFLICTING PRs |
| BJ beach-join | After tag path Wave 4, unless operator pulls earlier |
| C4 | 0.5.0 |

---

## Never idle -- node availability tiers

Operator directive 2026-08-31. Full policy: `docs/rules/CONTINUOUS_EXECUTION.md`.

| Tier | Nodes | Obligation |
|---|---|---|
| **A** | AWS (Linux) + Windows CLI | Always available. Scored to the **v0.4.0 tag candidate SHA** |
| **B** | Android (Pixel 6a) | Intermittent. Code to parity now; device time is verification + logs only |
| **C** | iOS / macOS | v0.5.0 scope. Do not start |

**"Blocked on hardware" is not a terminal state.** Descend: operator decisions
-> freebuff DISPATCHABLE implementation -> orchestrator merge gates -> Tier A
scoring -> PR disposition -> tag.

## The live rig

Do not hardcode AWS public IPs. Discover via `scripts/aws_deploy.sh`. Node
binaries for live nodes live under `tmp/radio-<sha>/` (rule 17), not `target/`.

| Node | Role |
|---|---|
| AWS `scm-always-on-node` | Cloud node (full node that also relays) |
| Windows CLI `127.0.0.1:9876` | Tier A host node |
| Pixel 6a | Tier B handset |

## Adding a task

1. Verify the premise end to end **before** writing the ticket.
2. Write into `queue/` per `docs/rules/FREEBUFF.md` section 3.
3. Add its DISPATCHABLE row here **and** ensure the file is on `origin/main`.
4. Keep `HANDOFF/V040_TAG_PATH_UNIFIED_2026-09-20.md` consistent if it changes
   the tag path.
