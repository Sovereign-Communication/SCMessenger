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
| T1 | `V040_T1_NODE_BOOT_SEED_DIAL.md` | The CLI node never dials known peers on boot, and its seed list is empty anyway. A node that changed address can never rejoin | **First** | none if confined to `cli/` |
| T2 | `V040_T2_UNIFY_PEER_LEDGER_STORES.md` | Two peer stores that never converge: the gossiped one is empty (0 entries), the CLI one is uncapped and polluted (4,678). Cherry-pick and unify | After or alongside T1 | **Rule-8 mandatory** -- changes what the node discloses |
| T4 | `V040_T4_ROUTING_FEED_ON_CONNECTION_ESTABLISHED.md` | D6: routing confidence pinned at 0.0 because nothing tells the engine a connection happened | Independent | **Rule-8 mandatory** |
| T5 | `V040_T5_DOCS_SYNC_GATE_IS_RED.md` | `docs_sync_check.sh` fails on clean `main`, so every agent's finalize gate is red | Independent | none |

T1 + T2 together deliver the operator's 2026-08-31 requirement: a node that takes
a new IP rejoins the mesh with no human action, and its new address propagates by
ledger gossip to nodes that never contacted it directly.

**Superseded, do not run:** the original `V040_T2_LEDGER_HYGIENE_EPHEMERAL_AND_SELF`
and `V040_T3_ADDRESS_SUPERSESSION_ON_CHURN` were fixes to symptoms of the
duplication that T2 now removes at the root. Both are folded into
`V040_T2_UNIFY_PEER_LEDGER_STORES.md`.

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
