# V040-T1 -- The CLI node never dials its known peers on boot

Status: OPEN (filed 2026-08-31, CEO audit)
Priority: P0 -- this is the v0.4.0 cloud-node parity gate
Lane: Freebuff / DeepSeek V4 Flash
Scope: `cli/src/main.rs` (startup path). Do not modify `core/src/transport/swarm.rs`
beyond adding a call site if one is genuinely required.

## The defect, proven live

The AWS node (`54.235.20.24`) was redeployed 2026-08-31T17:20Z at `main`@`69a8ba57`
with a persistent `/data` volume. Its ledger holds a real peer entry:

```json
{"multiaddr":"/ip4/98.94.45.116/tcp/9001",
 "peer_id":"aa7e73ca4b471b09cf2bf5bccde1d6500744a97c4e017a318cc1cf4798f0ebf2",
 "success_count":1,"failure_count":0}
```

45 seconds after boot, `GET /api/diagnostics` reports:

```
state: Bootstrapping
peers: []
external_addrs: []
```

It knows a peer. It never dials it.

## Root cause

`SwarmHandle::connect_to_seed_peers()` (`core/src/transport/swarm.rs:2528`)
sends `SwarmCommand::ConnectToSeedPeers`, which is handled at `swarm.rs:6646`
and builds its candidate list from `LedgerManager::seed_addresses(64)`
(`core/src/store/ledger_entry.rs:1183`). The machinery is complete and tested.

**Its only caller repo-wide is `core/src/mobile_bridge.rs:862`.** Verify yourself:

```bash
grep -rn "connect_to_seed_peers" --include=*.rs --include=*.kt --include=*.swift core/src cli/src android/app/src iOS | grep -v Generated
```

So Android calls it and the CLI does not. A headless node whose public address
changed can never rejoin the mesh: nobody can dial it at its old address, and it
never dials out.

## Why this is the gate, not a nicety

Operator ruling 2026-08-31:

> It should ledger share and re-join mesh automatically. If it moves, it should
> tell Windows and Android and then they can both ledger share the new IP with
> each other. This is how the mesh works -- automatic, not manual. Accept that
> every re-deploy is a new IP. That's how many nodes will be in the wild.

Address churn is the normal case, not an incident. Once the node dials out, the
existing ledger gossip on `ConnectionEstablished` (`swarm.rs:5486`, already
merged and platform-neutral) propagates its new address to Windows, and Windows
to Android. **The outbound dial is the only missing link in that chain.**

## The seed list is empty, so the dial alone is not enough (verified 2026-08-31)

**Read this before implementing. A naive fix is a no-op.**

There are two separate peer stores and they do not converge:

| Store | Written by | Live count | Gossiped? | Seeds `ConnectToSeedPeers`? |
|---|---|---|---|---|
| `storage/ledger.json` (core `LedgerManager`) | `swarm.rs:5397` only | Windows: **0**. AWS: **1** | Yes | **Yes** |
| `peers.json` (CLI `cli/src/ledger.rs:334`) | CLI dial policy | Windows: **4,678**. AWS: 107 | No | No |

The core ledger has exactly one production writer, `swarm.rs:5397`, and it is
correctly guarded by `endpoint.is_dialer()` -- the comment there (review F11)
explains why inbound remote addresses must never be recorded: they are the peer's
ephemeral source port and would fabricate "proven" entries that then get handed
to other peers as routing advice. That guard is right. Do not remove it.

But the consequence is a closed loop:

1. The core ledger only learns from connections **we** initiated.
2. The CLI never initiates one on boot (this ticket).
3. So the core ledger stays empty.
4. So `seed_addresses()` returns nothing.
5. So wiring up `connect_to_seed_peers()` on its own dials nothing at all.

Meanwhile `peers.json` accumulates thousands of entries from inbound connections
and is never shared with anyone. Verified on the Windows node: 4,678 entries,
2,655 with public addresses, mostly `147.81.41.188:<ephemeral port>` repeated
across five different peer ids -- while `ledger.json` sits at zero.

## Required change

This task has two halves. Both are needed or the node still cannot rejoin.

### Half 1 -- give the seed list something real

Provide a bootstrap source for the core ledger that does not depend on having
already dialled out. Pick the smallest option that works and say which you chose:

- Promote qualifying entries from `peers.json` into the core `LedgerManager` at
  startup: only entries with a plausible **listen** address (not an ephemeral
  source port), a canonical peer identity, and prior success. This is a one-way
  promotion, not a merge -- do not weaken the `is_dialer()` guard at
  `swarm.rs:5397` to achieve it.
- Or use the existing invite seed-ledger path (`import_seed_entries`,
  `ledger_entry.rs:1267`), which already exists for exactly this purpose.

Whichever you choose, the acceptance test is that a node restarted with a
populated `peers.json` and an empty `ledger.json` ends up with a non-empty
`seed_addresses()`.

### Half 2 -- dial the seeds on boot

In the CLI node startup path (`cli/src/main.rs`), after the swarm is running and
the ledger is loaded, call `connect_to_seed_peers()` and keep trying until the
node has at least one connected peer:

- Fire the first attempt once the swarm event loop is live.
- Retry with bounded exponential backoff (suggested: 5s, 15s, 45s, then every
  120s) while the peer count is zero. Stop retrying once a peer connects;
  re-arm if the peer count returns to zero.
- Log each sweep at INFO with the candidate count, so the behaviour is provable
  from `docker logs`: e.g. `[SEED-DIAL] sweep 3: 1 candidate(s), peers=0`.
- Do not spawn an unbounded task per attempt. One long-lived task.
- Respect the existing candidate caps in `swarm.rs:792,800`; do not raise them.

## Acceptance

1. A test proving a node with a populated `peers.json` and an empty
   `ledger.json` produces a non-empty `seed_addresses()` after startup (Half 1).
2. A test proving the startup path issues a seed dial when the seed list is
   non-empty and the peer count is zero (Half 2).
3. Live evidence, reproducing the exact failure this ticket documents: start the
   Windows node and the AWS node, touch nothing else, and confirm both report
   `peers` non-empty and `connection_path_state` no longer `Bootstrapping`.
   Today, both sit at `Bootstrapping` / `peers: []` indefinitely at identical
   SHAs -- that is the regression case.
3. `cargo test --workspace --no-run` passes. `cargo fmt --check` clean.
   Never read `$?` after a pipe -- capture to a file, then test the code.

## Rules that apply to this task

- No emojis anywhere. Use `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- Never `unwrap()` in production paths.
- State behind `Arc<RwLock<..>>` (parking_lot).
- This is a shared checkout: touch only the files this task requires. Never
  revert, delete, or stash a file you did not create.
- If your change ends up touching `core/src/transport/`, it is merge-blocked
  until a fresh adversarial review returns APPROVE. Prefer keeping the change
  in `cli/`.
