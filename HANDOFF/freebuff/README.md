# Freebuff lane -- live queue

Status: Active
Last updated: 2026-08-31
Rules: `docs/rules/FREEBUFF.md` -- read it before adding a task file here.
Plan this queue executes: `SHIP_PLAN.md` section 6.

This is the unmetered implementation lane. Models: **DeepSeek V4 Flash**, MiMo,
GLM 5.3 Flash. The `freebuff` CLI has no headless mode, so **the operator is the
transport**: an agent writes the task file, the operator pastes it into Freebuff
desktop. Every paste cycle costs operator attention -- a task file that sends the
model down the wrong path is the expensive failure here.

```
queue/   ready to paste, run in the order below
inbox/   Freebuff writes back here -- questions, blocked reports, wrong premises
done/    completed; Status line records the PR number
```

**The return path matters.** If a task file's premise does not survive contact
with the code, Freebuff should stop and write to `inbox/` rather than implement a
fix to a problem that does not exist. A watcher on this folder wakes the
orchestrator session when a reply lands. See `inbox/README.md` for the format.

---

## Queue -- v0.4.0

| # | Task file | What it fixes | Order | Review gate |
|---|---|---|---|---|
| T1 | `V040_T1_NODE_BOOT_SEED_DIAL.md` | The CLI node never dials known peers on boot, and its seed list is empty anyway. A node that changed address can never rejoin | **3rd** (Half 2 only) | none if confined to `cli/` |
| T2 | `V040_T2_UNIFY_PEER_LEDGER_STORES.md` | Two peer stores that never converge: the gossiped one is empty (0 entries), the CLI one is uncapped and polluted (4,678). Cherry-pick and unify | **2nd** | **Rule-8 mandatory** -- changes what the node discloses |
| T4 | `V040_T4_ROUTING_FEED_ON_CONNECTION_ESTABLISHED.md` | D6: routing confidence pinned at 0.0 because nothing tells the engine a connection happened | Any time -- touches nothing T1/T2 touch | **Rule-8 mandatory** |
| T6 | `V040_T6_TIER_A_CONFORMANCE_HARNESS.md` | No single command answers "are the two always-on nodes conformant right now?" -- which is how a 13-hour peer outage and a 7-hour dead watcher both went unnoticed | Any time -- read-only, no source changes | none |
| T5 | `V040_T5_DOCS_SYNC_GATE_IS_RED.md` | `docs_sync_check.sh` fails on clean `main`, so every agent's finalize gate is red | **1st** -- cheapest multiplier | none |
| T8 | `V040_T8_RESTORE_DIAGNOSTICS_FORMATTER_TEST.md` | A WS11 test was deleted as "orphaned" but its class still exists and is used. `format()` has had no coverage since 2026-08-14 | Any time -- CI only, no handset | none |
| T9 | `V040_T9_PR_QUEUE_BURNDOWN.md` | 29 open PRs, not one mergeable: all `BEHIND` the #234-#258 run, so their green checks were computed against a base that no longer exists | Any time -- CI only | escalate anything touching `core/src/{crypto,transport,routing,privacy}` |
| T10 | `V040_T10_FFI_SURFACE_GATE_PASSES_VACUOUSLY.md` | The FFI Surface Contract check runs on every PR and exits 0 when the bindings are missing, verifying nothing | Any time -- CI only | none |
| T7 | `V040_T7_ANDROID_PARITY_STAGING.md` | Device time is spent authoring tests instead of gathering evidence. Stage the Android work so the handset session is verification only | Whenever the handset is away | none |

T1 + T2 together deliver the operator's 2026-08-31 requirement: a node that takes
a new IP rejoins the mesh with no human action, and its new address propagates by
ledger gossip to nodes that never contacted it directly.

### Order -- RULED 2026-08-31, revised after the lane's clarification

**T5, then T2, then T1. T4 any time, in parallel.**

This revises the earlier T1-first ruling. The lane reported that the rig no
longer matched T1's filed regression state; that report was correct and it
changed the answer. Full reasoning:
`inbox/RULING_2026-08-31_clarification_response.md`.

| Order | Task | Why here |
|---|---|---|
| 1 | **T5** | 5-30 LoC, no dependencies, un-breaks every agent's finalize gate. Cheapest multiplier available -- do it before anything long |
| 2 | **T2** | Now the primary fix, not a follow-up. Making the gossiped store real is the only thing that gives a moved node a recovery path |
| 3 | **T1, Half 2 only** | **Half 1 is WITHDRAWN** -- T2's migration replaces that bridge. The boot dial is worthless until the core ledger holds real content |
| any | **T4** | Touches nothing T1/T2 touch |

**Why the correction matters.** T1 as filed claimed the CLI never dials on boot.
It does -- promiscuously, from the polluted `peers.json`, while
`connect_to_seed_peers()` (which reads the clean *gossiped* ledger) never runs
at all: zero occurrences in the node log. The two nodes did reconnect unaided,
but only because a stale local entry happened to still be correct, after ~40
minutes of grinding dead addresses. A genuinely changed address still has no
recovery path, because a new address is only learnable via gossip and the
gossiped store is the empty one.

**Consequence for acceptance:** a live "we have peers" check would actively
mislead on T1 -- the rig reconnects on its own regardless. The real gate is a
`[SEED-DIAL]` line in the node log, which is checkable with no live rig.

**Superseded, do not run:** the original `V040_T2_LEDGER_HYGIENE_EPHEMERAL_AND_SELF`
and `V040_T3_ADDRESS_SUPERSESSION_ON_CHURN` were fixes to symptoms of the
duplication that T2 now removes at the root. Both are folded into
`V040_T2_UNIFY_PEER_LEDGER_STORES.md`.

## Never idle -- node availability tiers

Operator directive 2026-08-31. Full policy: `docs/rules/CONTINUOUS_EXECUTION.md`.

| Tier | Nodes | Obligation |
|---|---|---|
| **A** | AWS (Linux) + Windows CLI | Always available. Driven to **full v1.0.0 conformance**, continuously |
| **B** | Android (Pixel 6a) | Intermittent. **Coded to parity now, verified later** -- device time is for verification and log capture, never for writing code |
| **C** | iOS / macOS | v0.5.0 scope. Do not start |

**"Blocked on hardware" is not a terminal state.** It means descend the ladder:
restore Tier A -> v0.4.0 gate items -> Tier A v1.0.0 conformance -> Tier B parity
coding -> owned-issue burn-down (`SHIP_PLAN.md` section 7) -> PR queue. Take the
first item actionable right now.

## The live rig these tasks were written against

Both nodes run `main`@`69a8ba57` and reproduce the T1 failure on demand -- start
them, touch nothing, and both sit at `connection_path_state: Bootstrapping` with
`peers: []` indefinitely.

| Node | Address | Identity |
|---|---|---|
| AWS (Amazon Linux 2023, Docker) | discovered via `scripts/aws_deploy.sh`; was `54.235.20.24` on 2026-08-31 | `640a5dc8...` / `12D3KooW9uRM...` / pubkey `014b8105...` |
| Windows CLI | `127.0.0.1:9876` local API | `985a25f9...` / `12D3KooWD6vZQrUqpyGa` / pubkey `30d0fa67...` |

The node's public IP **changes on every instance replacement** -- that is the
design constraint these tasks exist to satisfy, not an incident. Never hardcode
it; `scripts/aws_deploy.sh` discovers it from the EC2 API.

## Adding a task

1. Write the file into `queue/` following the contract in
   `docs/rules/FREEBUFF.md` section 3.
2. Add its row to the table above. An unindexed task file is invisible.
3. Verify the premise end to end **before** dispatching. T1 was first written
   with an incomplete premise -- the fix as specified would have been a no-op --
   and was corrected before it cost a cycle.
