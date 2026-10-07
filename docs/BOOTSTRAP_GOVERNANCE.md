# Peer Seed Address Governance

> **Status:** Current
> **Last updated:** 2026-10-06

> Terminology note: "bootstrap node" survives only as a config-key name. It does
> not denote a node role. There are no dedicated relays and no bootstrap tier --
> there are only nodes, and every node is a full relay. See
> `docs/BOOTSTRAP.md` for the joining model and `docs/TRANSPORT_ARCHITECTURE.md`
> for the architecture.

> [SUPERSEDED IN PART 2026-10-06] Operator decision for issue #469
> (`docs/BOOTSTRAP_RENDEZVOUS_DECISION_469.md`): the ONLY seed source is an
> invite (QR or invite info). Static or operator-supplied seed lists -- the
> `SC_BOOTSTRAP_NODES` variable (runtime and build-time), `config set
> bootstrap_node_add`, and "supply one address manually" cold start -- are
> superseded and scheduled for removal (tasks T4, T10). Sections below that
> describe them stay for traceability until the removal lands. The invite
> sections and the "no dedicated relays / every node is a full relay" model
> remain current.

## Decision

Peer discovery is governed by **ledger exchange plus local discovery**. Static
seed address lists exist only as an optional, user-supplied cold-start input, and
ship empty.

Concretely:

- `CORE_BOOTSTRAP_NODES` (`core/src/transport/bootstrap.rs`) is `&[]`.
- `DEFAULT_BOOTSTRAP_NODES` (`cli/src/bootstrap.rs`) is `&[]`.
- The `bootstrap_nodes` config field (`cli/src/config.rs`) defaults to empty.

Nothing routable is compiled into a public build. There is no project-operated
entry infrastructure to govern, and no enrollment process for community
addresses.

## Resolution Order for Seed Addresses

An optional startup dial list is still resolved. This governs *which seed
addresses a cold node dials first*, nothing more -- none of it is required for
ongoing operation.

CLI (`default_bootstrap_nodes()` in `cli/src/bootstrap.rs`), first non-empty wins
and is used exclusively:

1. **Runtime environment variable** -- `SC_BOOTSTRAP_NODES`, comma-separated
   multiaddrs. Intended for operators, Docker, and CI. This is the **only**
   spelling the code reads; an unprefixed `BOOTSTRAP_NODES` is silently ignored.
2. **Build-time value** of the same variable, captured via `option_env!`. For
   private/closed deployments.
3. **`DEFAULT_BOOTSTRAP_NODES`** -- compiled-in list, **empty in all shipped
   builds**.

Core (`BootstrapManager::new()` in `core/src/transport/bootstrap.rs`) takes the
**union** rather than an exclusive override: environment-derived addresses first,
then `CORE_BOOTSTRAP_NODES` (also empty). Because the compiled list is empty in
shipped builds, union and override are indistinguishable in practice, but the
two code paths do differ.

Separately, the CLI persists whatever the user added into `config.json`
(`bootstrap_nodes`, `cli/src/config.rs`). That list is loaded as-is; there is no
merge of new compiled defaults into an existing config on upgrade.

> [WARNING] Earlier revisions of this document described a three-tier chain with
> a **remote URL fetch** step (`remote_url` in `BootstrapConfig`, HTTP GET of a
> JSON multiaddr array, 5-second timeout) and a `static_nodes` config field.
> **Neither exists in the codebase.** `BootstrapConfig` carries only backoff,
> retry, timeout, discovery-toggle, and circuit-breaker settings; there is no
> `remote_url`, no `static_nodes`, and no HTTP bootstrap fetch anywhere in
> `core/`, `cli/`, or `wasm/`. Remote-URL seeding is an unimplemented idea, not a
> supported option. The manager type is `BootstrapManager`, not
> `BootstrapResolver`.

Once any connection exists -- from a seed address, from mDNS/BLE/Wi-Fi on the
local network, or from an inbound dial -- peer records arrive over the
`/sc/ledger-exchange/1.0.0` protocol, are persisted, and become the durable
source of remote peers. The resolution chain above is not consulted again for
discovery.

## Trust Model

- **Seed addresses carry no privilege.** Supplying an address to dial is not a
  grant of trust: the peer at that address gets exactly the same treatment as a
  peer found by mDNS or learned from the ledger. It cannot read message contents
  (end-to-end encryption) and cannot impersonate any peer (cryptographic
  identities). The most a bad seed can do is refuse service or gossip junk peer
  records, which is bounded by reputation tracking and by the fact that no node
  is load-bearing for entry.

- **The user chooses the entry point.** Because nothing is shipped, the trust
  decision at cold start is explicit and local: the user or operator decides
  whose address to use. No default delegates that decision to the project.

- **Ledger entries are unsigned hints, not attestations.** A record learned via
  ledger exchange asserts only "this address was reachable for the peer that
  told us". Connections are still authenticated by libp2p Noise against the
  PeerId, so a wrong or malicious address fails closed rather than yielding a
  wrong peer.

- **Identity flexibility.** A node may rotate its libp2p PeerId without breaking
  clients that dial it by IP:port; the client accepts whichever valid Noise
  identity the remote presents. This supports key rotation and multi-node
  deployments behind one address. The cost is that IP:port-only addresses do not
  pin identity -- include `/p2p/<PEER_ID>` when identity pinning matters.

- **No PKI or certificate pinning.** Trust rests on the user-supplied seed, the
  authenticated transport, and the persisted ledger.

## Operator Guidance

An operator running a reachable node for their own users configures those
clients with that node's address. This is ordinary configuration of a private
deployment, not participation in a shipped list.

```bash
# Runtime, per client
export SC_BOOTSTRAP_NODES="/ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>,/ip4/<NODE_IP_2>/tcp/9001/p2p/<PEER_ID_2>"

# Persisted in the CLI's config.json
scmessenger-cli config set bootstrap_node_add /ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>
scmessenger-cli config get bootstrap_nodes
```

Substitute your own values -- there are no addresses to copy from this document.
Per-node operational setup is in `docs/RELAY_OPERATOR_GUIDE.md`; the joining
model and the full command reference are in `docs/BOOTSTRAP.md`.

## Open Enhancements

- **Reputation-weighted dial order:** prefer seed and ledger addresses with
  better observed connect success.
- **Ledger entry expiry and pruning policy:** bound stale-record accumulation.

Already implemented and no longer pending: gossip-based peer discovery via
ledger exchange, and removal of all shipped default addresses.

## References

- Ledger exchange protocol registration: `core/src/transport/behaviour.rs`
- Ledger storage (core/mobile, `LedgerManager`): `core/src/store/ledger_entry.rs`
- Ledger storage (CLI, `peers.json`): `cli/src/ledger.rs`
- Seed address resolution: `core/src/transport/bootstrap.rs` (`BootstrapManager`,
  `BootstrapConfig`, `CORE_BOOTSTRAP_NODES`)
- CLI seed handling: `cli/src/bootstrap.rs`, `cli/src/config.rs`
- Joining model and command reference: `docs/BOOTSTRAP.md`
- Node operator guide: `docs/RELAY_OPERATOR_GUIDE.md`
