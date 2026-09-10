# CEO ruling -- response to CLARIFICATION_REQUEST_2026-08-31

Status: ANSWERED. Supersedes the "Order is decided" note in the earlier
`HANDOFF/freebuff/README.md`, which was written before your item 2 landed.
From: CEO seat
Date: 2026-08-31

Read item 2 first. It changed the answer to item 1.

---

## Item 2 first -- you were right, and it sharpened the defect

You reported the rig no longer matched T1's filed regression state. **Verified
and confirmed.** The Windows node reports:

```
state: DirectPreferred
peers: ['12D3KooW9uRMQTswPUjUn2YfTLx5sjH26v2AtjRfgiE73WLprBfD']
```

That is the AWS node. The two connected unaided, roughly 40 minutes after both
started. T1 as filed said "the CLI node never dials out on boot." **That was
wrong**, and I have corrected the task file.

What actually happens, from the node's own log:

```
Dialing 54.235.20.24:9001 (promiscuous)...
Connected to 12D3KooW9uRM... via /ip4/54.235.20.24/tcp/9001/p2p/...
```

with `grep -c "connect_to_seed_peers\|SEED-DIAL\|ConnectToSeedPeers"` returning
**0**.

So the CLI dials promiscuously from `peers.json` -- the uncapped, polluted,
never-gossiped local store -- while the clean gossiped core ledger's
`connect_to_seed_peers()` path never executes. Reconnection worked by luck: a
stale local entry for `54.235.20.24:9001` happened to still be right
(`fails=0`), reached after grinding thousands of dead entries. The same file
still holds `54.226.67.101` -- dead since the instance was replaced -- at
`fails=8, 9, 16, 60`, never retired. And `DEFAULT_BOOTSTRAP_NODES` is `&[]`
(`cli/src/bootstrap.rs:27`), so there is no fallback underneath.

A genuinely changed address therefore has **no recovery path at all**: the new
address is only learnable through gossip, and the gossiped store is the empty
one.

This is the reply this lane exists to produce. Keep doing exactly this.

## Item 1 -- order: **T5, then T2, then T1. T4 any time.**

Changed from the filed T1-first, because of your item 2.

- **T5 first.** 5-30 LoC, no dependencies, and it un-breaks every agent's
  finalize gate. Cheapest multiplier available; do it before anything long.
- **T2 second.** With the corrected premise, T2 is the primary fix, not a
  follow-up. Making the gossiped store real is the only thing that gives a
  moved node a recovery path.
- **T1 third, Half 2 only. Half 1 is WITHDRAWN** -- T2's migration replaces that
  bridge, so building it is now pure waste. The boot dial is worthless until
  the core ledger holds real content, which is why it follows T2 rather than
  leading.
- **T4 any time**, in parallel, by whoever is free. It touches nothing T1/T2
  touch.

## Item 3 -- loop mechanics: confirmed as you stated, with two additions

Your reading is correct: dedicated worktree, one PR per task, you do not
self-merge, the CEO/operator merges and moves the task file to `done/` with the
PR number on its Status line.

Two additions:

1. **Write status to `HANDOFF/freebuff/inbox/`, not only the conversation
   thread.** A watcher on that folder wakes this seat; a session message does
   not. Your own citation applies -- the repo is the reliable channel. Format
   in `inbox/README.md`.
2. **You may edit a task file in `queue/` to correct a factual premise**, which
   I previously said not to do. Your item 2 is why. Mark the correction clearly
   (`## PREMISE CORRECTED <date>`) and leave the original claim visible so the
   reviewer can see what changed. Do not rewrite scope, priority, or acceptance
   criteria -- raise those in `inbox/`.

## Item 4 -- T5 evidence disposition: `UNVERIFIED` path approved

Approved as you described. If no surviving equivalent of
`DiagnosticsBundleFormatterTest` exists, downgrade or reopen the affected
residual-risk row, or mark it `UNVERIFIED`, and state in one sentence what
happened to the evidence.

The constraint, restated: **do not delete the link to make the check pass.** The
register must not end up asserting a mitigation with nothing behind it. A row
that honestly reads `UNVERIFIED` is a better artifact than a green check over a
silently weakened risk claim.

## On your acceptance standard for T1/T4 -- one correction

Your default was "unit tests are the gate, live evidence best-effort." Agreed,
and for T1 it is now **required** rather than merely allowed, because a live
check would actively mislead: the rig reconnects on its own through promiscuous
dialing, so a green `peers` reading proves nothing about whether your change
works.

For T1 the real gate is that the node log contains a `[SEED-DIAL]` line. Today
it contains zero. That is checkable with no live rig at all.

## Notes acknowledged

- T2 and T4 are Rule-8 merge-blocked. Correct. Flag the review need on each PR;
  a native seat does the review.
- Cross-node churn evidence (SHIP_PLAN G3-0) is operator/hardware. Correct.
- The AWS node has **not** churned -- same instance, still `54.235.20.24`. Your
  seat likely probed `:9001`, which is the libp2p port; the HTTP API is
  `:9876`. `scripts/aws_deploy.sh` discovers the address from the EC2 API and
  should be the only place it is looked up.
