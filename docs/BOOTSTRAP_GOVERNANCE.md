# Peer Seed Address Governance

> **Status:** Current
> **Last updated:** 2026-10-07

> Terminology note: there are no dedicated relays, no bootstrap tier and no
> node classes -- only nodes, and every node is a full relay. The only
> distinction between nodes is whether an identity is loaded. Use "always-on
> node", never "bootstrap node". See `docs/BOOTSTRAP.md` for the joining model,
> `docs/rules/NODE_MODEL.md` and `docs/TRANSPORT_ARCHITECTURE.md`.

## Decision (operator, 2026-10-06; issue #469)

Authoritative record: `docs/BOOTSTRAP_RENDEZVOUS_DECISION_469.md`.

1. **Invite-only bootstrap.** The only seed source is an invite (QR or invite
   info). Peer discovery otherwise is local (LAN/BLE/Wi-Fi Aware/Multipeer) and
   ledger gossip over `/sc/ledger-exchange/1.0.0`.
2. **Nothing is static.** No shipped addresses, no seed list, no environment
   variable, no config key, no DNS/URL seed. Task T4 (#485) removes the legacy
   `SC_BOOTSTRAP_NODES` variable, the `bootstrap_nodes` config field and
   `bootstrap_node_add/remove` pseudo-keys, and the empty compiled constants.
3. **All nodes are equal.** Predominance is earned by reputation (uptime,
   recency, failure count, relay health), never by a flag. An always-on AWS node
   becomes predominant for store-and-forward only because it behaves well.
   (The `headless_bonus` term in `relay_health.rs` is a node-class input and is
   removed by T11.)
4. **Retry never gives up, and the UI never claims absence.** Discovery backoff
   is event-driven (network, BLE, foreground, invite, ledger events reset it to
   aggressive, then it decays with jitter to a density/power-derived ceiling);
   the UI never shows a "no network peers" message -- node indicators are the
   only connectivity presentation.

Nothing routable is compiled into a public build and there is no project-operated
entry infrastructure to govern, no enrollment process for community addresses,
and no PR path for contributing addresses.

## Trust Model

- **An invite is a signed hint, not a grant of trust.** The inviter's signature
  covers the seed ledger (`get_signable_data()`), so tampering invalidates the
  invite. Seed entries are bare multiaddrs imported as unproven
  (`success_count = 0`); they are promoted only by a successful live dial, never
  directly from wire data. The inviter chooses what to disclose; treat an invite
  as though its address list were public.
- **Seed or ledger peers carry no privilege.** The peer at a seed address gets
  the same treatment as one found by mDNS or learned from the ledger. It cannot
  read message contents (end-to-end encryption) and cannot impersonate a peer
  (cryptographic identities). The most a bad node can do is refuse service or
  gossip junk records, bounded by reputation and by no node being load-bearing.
- **Ledger entries are unsigned hints, not attestations.** An entry asserts only
  "this address was reachable for the peer that told us". Connections are
  authenticated by libp2p Noise; a wrong address fails closed. Exchanged pairs
  reach the DHT only when `ledger_verified_pair` holds.
- **Reputation, not roles.** Preference among known nodes comes from observed
  behaviour: `LedgerManager::get_preferred_relays`
  (`core/src/store/ledger_entry.rs`), `core/src/transport/reputation.rs`,
  `core/src/transport/relay_health.rs`.
- **Identity flexibility.** A node may rotate its libp2p PeerId without breaking
  clients that dial by IP:port; include `/p2p/<PEER_ID>` when pinning matters.
- **No PKI or certificate pinning.** Trust rests on the invite signature, the
  authenticated transport, and the persisted ledger.

## Operator Guidance

An operator running an always-on node mints invites for the people they want to
bring in (`scm invite create`, T2). That node is an ordinary node; nothing about
it is compiled into anything. Per-node setup is in
`docs/RELAY_OPERATOR_GUIDE.md`; the joining model is in `docs/BOOTSTRAP.md`.
Test and simulation rigs follow the same flow: one node mints an invite and the
others redeem it (T10); topologies are fixtures, not shipped defaults.

## Open Enhancements

- Ledger entry expiry and pruning policy to bound stale-record accumulation.
- Reputation-weighted dial order across ledger candidates (partly present in
  `get_preferred_relays`).

## References

- Decision and implementation spec: `docs/BOOTSTRAP_RENDEZVOUS_DECISION_469.md`
- Invite token and seed ledger: `core/src/relay/invite.rs`
- Ledger exchange protocol registration: `core/src/transport/behaviour.rs`
- Ledger storage (core/mobile, `LedgerManager`): `core/src/store/ledger_entry.rs`
- Ledger storage (CLI, `peers.json`): `cli/src/ledger.rs`
- Joining model: `docs/BOOTSTRAP.md`
- Node model: `docs/rules/NODE_MODEL.md`
- Node operator guide: `docs/RELAY_OPERATOR_GUIDE.md`

---

# Superseded Content (historical record)

[SUPERSEDED 2026-10-06] The sections below record the earlier static-seed
governance model, including the former "Resolution Order for Seed Addresses".
They are retained as history only and are not guidance. Superseded by
`docs/BOOTSTRAP_RENDEZVOUS_DECISION_469.md`; the mechanisms they describe
(`SC_BOOTSTRAP_NODES`, build-time seeding, `bootstrap_nodes` config,
`config set bootstrap_node_add`, `CORE_BOOTSTRAP_NODES`,
`DEFAULT_BOOTSTRAP_NODES`) are removed by T4 (#485).

## [Superseded] Decision

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

## [Superseded] Resolution Order for Seed Addresses

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

## [Superseded] Operator Guidance

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
