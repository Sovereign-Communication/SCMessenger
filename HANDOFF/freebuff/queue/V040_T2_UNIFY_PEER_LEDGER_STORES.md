# V040-T2 -- Unify the two peer ledgers into one store

Status: OPEN (filed 2026-08-31, CEO audit). **Supersedes the earlier V040-T2
(ledger hygiene) and V040-T3 (address supersession)** -- both were fixes to
symptoms of this duplication and are folded in here.
Priority: P0 -- the v0.4.0 cloud-node parity gate depends on it
Lane: Freebuff / DeepSeek V4 Flash
Scope: `core/src/store/ledger_entry.rs` (target store), `cli/src/ledger.rs`
(source of the cherry-picks and of the deletions), `cli/src/main.rs`,
`cli/src/server.rs`.

Operator ruling 2026-08-31: *"we need it unified -- this is huge. Get them both
merged into one. If duplicate is necessary, then unify; if it's a messy dupe,
then cherry pick and converge."*

It is a messy dupe. Cherry-pick and converge.

---

## 1. The duplication, measured

Two peer stores exist. They have different structs, different files, different
caps, and they never converge.

| | Core store | CLI store |
|---|---|---|
| Type | `LedgerManager` / `LedgerEntry` | `ConnectionLedger` / `LedgerEntry` |
| File | `core/src/store/ledger_entry.rs` (3,733 lines) | `cli/src/ledger.rs` (2,220 lines) |
| Persists to | `storage/ledger.json` | `peers.json` |
| Entry cap | `MAX_LEDGER_ENTRIES = 1024`, plus byte caps | **none** |
| Gossiped to peers | **yes** | no |
| Seeds `ConnectToSeedPeers` | **yes** | no |
| Live count, Windows 2026-08-31 | **0** | **4,678** |
| Live count, AWS 2026-08-31 | 1 | 107 |

The store that gets shared is empty. The store that accumulates is CLI-local and
uncapped. That is the whole bug, and every symptom below follows from it.

Reproduce:

```bash
python -c "import json,io; d=json.load(io.open(r'C:\Users\SCM\AppData\Local\scmessenger\peers.json',encoding='utf-8')); print('peers.json:',len(d.get('entries',{})))"
python -c "import json,io; print('ledger.json:',len(json.load(io.open(r'C:\Users\SCM\AppData\Local\scmessenger\storage\ledger.json',encoding='utf-8'))))"
```

## 2. Which half is right about what

Neither store is wholly correct. Each holds functionality the other needs -- this
is why the answer is convergence rather than deleting one.

**The core store is right about identity and disclosure.** It canonicalizes
peer ids to the hex public key on write, understands self-certifying bindings
(`is_self_certifying_binding`, `:322`), carries `public_key` and `nickname`,
enforces record and byte caps, and owns the whole gossip surface:
`export_seed_entries`, `import_seed_entries`, `exchange_response_entries`,
`seed_addresses`. Its single production writer, `swarm.rs:5397`, is correctly
guarded by `endpoint.is_dialer()` -- and the comment there (review F11) explains
why inbound remote addresses must never be recorded: they are the peer's
ephemeral source port, and recording them would fabricate "proven" entries that
then get handed to other peers as routing advice. **That guard is correct. Do not
weaken it, and do not remove it to make anything below easier.**

**The CLI store is right about address hygiene, and already implements the two
fixes the superseded tickets asked for:**

- `record_identified_peer(peer_id, listen_addrs)` (`cli/src/ledger.rs:477`) --
  records a peer's **advertised listen addresses** from `identify`. This is
  exactly the "only store dialable listen addresses" fix.
- `reap_stale_addresses_for_peer(peer_id, confirmed_addr)` (`:509`) -- retires a
  peer's other addresses once one is confirmed. This is exactly the address
  supersession that IP churn requires.
- `is_dialable_for_this_node(multiaddr, mode, my_addrs)` (`:1035`) -- the
  self/mode/routability filter.
- `prioritize_dial_candidates` (`:1109`).
- Fields the core entry lacks: `locally_verified` (personally verified vs
  hearsay), `is_bootstrap` (never evict), `observed_peer_ids` (several ids seen
  at one address -- the misattribution signal), `first_seen`, `label`.

So the good hygiene exists; it is wired to the store nobody shares. That is why
the mesh is polluted **and** empty at the same time.

## 3. Target design

**One persisted store: the core `LedgerManager`.** It is platform-neutral --
Android, iOS and WASM bind it through `mobile_bridge`, while `ConnectionLedger`
is CLI-only. Unifying onto the CLI store would strand the mobile clients.

### 3.1 Move into the core store

Port these from `cli/src/ledger.rs`, preserving behaviour and tests:

- `record_identified_peer` -- becomes the primary way a peer's dialable
  addresses are learned. An address learned any other way is an observation, not
  a dialable entry.
- `reap_stale_addresses_for_peer` -- supersession on confirmation.
- `is_dialable_for_this_node` and `prioritize_dial_candidates` -- applied inside
  `dialable_addresses()` / `seed_addresses()` rather than at each call site.

Extend the core `LedgerEntry` with: `locally_verified: bool` (serde default
`false`, so pre-existing entries classify as hearsay until re-verified),
`is_bootstrap: bool`, `first_seen: Option<u64>`, `observed_peer_ids: Vec<String>`
(**bounded** -- apply a cap like the existing `MAX_TOPICS_PER_ENTRY` pattern),
and `label: Option<String>`. Keep every existing core field and every existing
cap.

### 3.2 The disclosure rule this unlocks -- do not skip it

`locally_verified` is the field that makes gossip safe. **Only locally verified
entries may be exported** through `export_seed_entries`,
`exchange_response_entries`, or any other path that hands entries to a peer.
Hearsay is usable locally and must never be re-published as though we proved it.
Without this rule, unification would spread one node's pollution across the whole
mesh -- strictly worse than today. Add a test that asserts an unverified entry is
never exported.

### 3.3 Keep in the CLI, do not move

`cli/src/ledger.rs` also holds genuine **process-lifetime dial state**, which the
file's own comments at `:190`, `:245` and `:303` mark as never serialized:
`PeerDialState`, `AddrDialState`, `DialKey`, `try_begin_dial`, `complete_dial`,
`record_disconnect`, `dial_state`. That is dial policy, not a peer store. Leave
it in the CLI. After this change `cli/src/ledger.rs` should contain that and
essentially nothing else.

### 3.4 Delete from the CLI

The persisted duplicate: the CLI `LedgerEntry` struct, `ConnectionLedger::load`
and `save` (`:333`, `:353`, the `peers.json` reader/writer), `record_connection`,
`record_failure`, `dialable_addresses`, `merge_shared_entries`, `find_by_peer_id`,
`add_bootstrap`, `record_topic`, `all_known_topics`, `summary`. Every one has a
core equivalent, or gains one under 3.1.

Call sites are few: `grep -c "ledger\." cli/src/main.rs` reports 25, of which
most are `.clone()`/`.lock()`; the only distinct methods called are
`all_known_topics`, `summary` and `add_bootstrap`. `core/src/transport/swarm.rs`
mentions `ConnectionLedger` **only in comments** -- there is no core dependency
on it. Confirm both facts yourself before deleting.

### 3.5 Migration

One-time, on first start after the change: read an existing `peers.json`, filter
it through the new hygiene rules, import survivors into the core ledger as
**`locally_verified: false`** unless the entry's own `locally_verified` was true,
then leave `peers.json` in place but stop writing it. Do not delete a user's
file. Log the counts at INFO: `[INFO] ledger migration: 4678 in, N imported, M
rejected`.

Two real poisoned files exist to test against: the AWS node's 107-entry store at
`/opt/scm-relay-data/peers.json.poisoned-backup-1788196` on `54.235.20.24`, and
the Windows node's live 4,678-entry store at
`%LOCALAPPDATA%\scmessenger\peers.json`.

## 4. What the pollution looks like -- your migration filter must reject all of it

From the live stores, 2026-08-31:

- **Ephemeral source ports.** Thousands of `147.81.41.188:<port>` rows, one per
  outbound source port ever used. Dead the instant the connection closed.
- **Self-entries.** The AWS node records its own address `54.235.20.24:9001`
  under its own key `014b8105...` **and** under `12D3KooWD6vZQrUqpyGa`, which is
  the Windows node's peer id. Result, visible in `docker logs scm-node`:
  `Dial error: Unexpected peer ID ... at /ip4/127.0.0.1/tcp/9001/p2p/...` on a
  loop, across its own 33 listeners.
- **Identity misattribution.** `12D3KooWD6vZQrUqpyGa` appears at a residential
  IP, a cellular IP, the AWS node's own address, and two IPv6 /64s. One identity
  cannot be at five unrelated networks. `observed_peer_ids` is the signal:
  several ids at one address means the address, not the identity, is the key --
  and that is the wrong way round.
- **Private addresses learned from public peers.** 60 references to `172.17.x`
  (Docker bridge), `172.31.x` (VPC), `10.32.x`. Unreachable to us.
- **Placeholder junk.** `1.1.1.1:9000`, `1.2.3.4:9000`.

## 5. Acceptance

1. `peers.json` is no longer written. `storage/ledger.json` is the only persisted
   peer store.
2. Migration test: the real 4,678-entry file in, a small number of genuinely
   dialable entries out, **zero** self-entries, **zero** ephemeral-port entries,
   zero private addresses attributed to public peers.
3. Disclosure test: an entry with `locally_verified: false` is never returned by
   `export_seed_entries` or `exchange_response_entries`.
4. Supersession test: identity `X` known at `A`, confirmed at `B` -- `B`
   outranks `A` in `seed_addresses()`, and `A` retires without `X` itself
   becoming undialable. This must preserve the PR #256/#257 property that a
   reachable peer never sticks in the dead tier.
5. Cap test: a peer with 20 observed addresses stores at most the per-peer cap;
   the store as a whole never exceeds `MAX_LEDGER_ENTRIES`.
6. `cargo test --workspace --no-run` passes. `cargo fmt --check` clean.
   `cargo clippy` with `-D warnings` clean.
   Never read `$?` after a pipe -- `cargo fmt --check > out.txt; rc=$?; head out.txt; exit $rc`.

## 6. Review gate -- mandatory

This changes what the node discloses to other peers, so it is a privacy and
routing change, not a refactor. A fresh adversarial reviewer that did not author
the change must record an APPROVE before merge. Green CI is not sufficient.

Reviewer's focus: that the `is_dialer()` guard at `swarm.rs:5397` is intact, that
no unverified entry can reach the wire, and that the migration cannot import a
self-entry.

## 7. Sequencing

Land **V040-T1** (boot seed dial + seed-list bootstrap) first or alongside. T1's
"Half 1" promotes entries from `peers.json` into the core ledger as a bridge;
this ticket replaces that bridge with a real migration. If T2 lands first, T1's
Half 1 becomes unnecessary and T1 reduces to the boot dial alone -- say so in the
PR rather than implementing both.

## 8. Rules that apply to this task

- No emojis anywhere. Use `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- Never `unwrap()` in production paths.
- State behind `Arc<RwLock<..>>` (parking_lot). `IronCore` is the only entry
  point. No sled access outside `store/`.
- Shared checkout: touch only the files this task requires. Never revert, delete,
  stash, or commit a file you did not create. A clean `git status` is not a goal.
