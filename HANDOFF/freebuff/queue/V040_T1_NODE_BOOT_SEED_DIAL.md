# V040-T1 -- The CLI node never dials its known peers on boot

Status: OPEN (filed 2026-08-31, CEO audit)
Priority: P0 -- this is the v0.4.0 cloud-node parity gate
Lane: Freebuff / DeepSeek V4 Flash
Scope: `cli/src/main.rs` (startup path). Do not modify `core/src/transport/swarm.rs`
beyond adding a call site if one is genuinely required.

## PREMISE CORRECTED 2026-08-31 -- read this, the original filing was too strong

The Freebuff lane reported that the rig no longer matched this ticket's filed
regression state. That report was correct, and re-investigation sharpened the
defect rather than dissolving it. **Credit where due: this is exactly the reply
this lane exists to produce.**

**What the original filing said:** "the CLI node never dials out on boot."
**That is wrong.** It does dial. Windows connected to the AWS node roughly 40
minutes after both started, unaided:

```
Dialing 54.235.20.24:9001 (promiscuous)...
Connected to 12D3KooW9uRMQTswPUjUn2YfTLx5sjH26v2AtjRfgiE73WLprBfD
  via /ip4/54.235.20.24/tcp/9001/p2p/12D3KooW9uRM...
```

`/api/diagnostics` on the Windows node now reports
`connection_path_state: DirectPreferred` with that peer.

**What is actually broken, and it is worse than the original framing:**

The CLI dials **promiscuously from `peers.json`** -- the uncapped, polluted,
never-gossiped local store -- while `connect_to_seed_peers()`, which reads the
clean, canonical, *gossiped* core ledger, never runs at all. Confirmed: the node
log contains **zero** occurrences of `connect_to_seed_peers`, `SEED-DIAL`, or
`ConnectToSeedPeers`.

```bash
grep -c "connect_to_seed_peers\|SEED-DIAL\|ConnectToSeedPeers" <node log>   # 0
```

Three consequences, all load-bearing for the v0.4.0 churn gate:

1. **Reconnection is luck, not design.** It worked here only because a stale
   local entry for `54.235.20.24:9001` happened to still be correct
   (`fails=0`). It took ~40 minutes of grinding through thousands of entries.
2. **A genuinely changed address has no recovery path at all.** A new address is
   only learnable through ledger gossip, and the gossiped store is the empty
   one. The same `peers.json` still holds `54.226.67.101` -- the address that
   died when the instance was replaced -- at `fails=8`, `9`, `16`, `60`, never
   retired.
3. `DEFAULT_BOOTSTRAP_NODES` is `&[]` (`cli/src/bootstrap.rs:27`), so there is no
   hardcoded fallback underneath this. Local store or nothing.

**Effect on this task:** the boot dial (Half 2) is still required, but it is
worthless until the core ledger has real content -- which is
`V040_T2_UNIFY_PEER_LEDGER_STORES.md`'s job, done properly. **Half 1 of this
ticket is therefore withdrawn**, and T2 now runs first. See the order ruling in
`HANDOFF/freebuff/README.md`.

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

### Half 1 -- WITHDRAWN 2026-08-31, do not implement

Superseded by `V040_T2_UNIFY_PEER_LEDGER_STORES.md`, which now runs first and
performs a real migration instead of this bridge. Kept below only so the PR
reviewer can see what was dropped and why.

### Half 1 (withdrawn) -- give the seed list something real

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

### Half 2 -- dial the seeds on boot (THIS IS THE WHOLE TASK NOW)

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

1. ~~A test for Half 1's promotion~~ -- withdrawn, T2 covers this.
2. A test proving the startup path issues a seed dial when the seed list is
   non-empty and the peer count is zero.
3. Proof the seed dial actually runs: the node log contains a `[SEED-DIAL]`
   line. Today it contains **zero** occurrences of `connect_to_seed_peers`,
   `SEED-DIAL`, or `ConnectToSeedPeers` -- that absence is the regression this
   task closes, and it is checkable without any live rig.
4. Live evidence is **best-effort, not the gate**. The rig reconnects on its own
   via promiscuous dialing from `peers.json`, so a green live check does NOT
   prove this task worked -- only the `[SEED-DIAL]` log line does. Mark live
   observations `UNVERIFIED` where the node is not reachable from your seat.
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
