# ROOT CAUSE -- Android renders 1 peer after receiving 64 ledger entries

Status: Root cause identified, orchestrator-verified. Fix NOT written.
Closes the investigation opened by
`HANDOFF/todo/LEDGER_SHARING_ANDROID_NODE_VISIBILITY_2026-08-05.md`
Gate: any fix touches `core/src/store/` and the Android UI -- adversarial
review required (AGENTS.md rule 8) before merge.

## The answer

**Wire-learned ledger entries are invisible to the Android peer list because
the render path filters on `success_count > 0`, and freshly ingested foreign
entries are created with `success_count: 0`.**

An entry only earns `success_count > 0` when THIS node itself successfully
dials that address. So all 64 entries received from a peer sit at 0 and are
filtered out of the list. This is not a disclosure-filter problem, not an
identity-canonicalization problem, and not a transport problem. The 64 entries
arrive, are accepted, and are persisted correctly -- they are then hidden at
render time.

## Verified evidence (every claim re-checked by the orchestrator against source)

Render-path filter, `core/src/store/ledger_entry.rs:786-793`:

```rust
pub fn dialable_addresses(&self) -> Vec<LedgerEntry> {
    let entries = self.entries.lock();
    entries
        .iter()
        .filter(|e| e.success_count > 0 && e.failure_count < LEDGER_DEAD_FAILURE_THRESHOLD)
        .cloned()
        .collect()
}
```

New-entry write, `core/src/store/ledger_entry.rs:335-350` (`annotate_identity_locked`):
`entries.push(LedgerEntry { ... success_count: 0, failure_count: 0, ... })`.

`success_count` is raised only by `record_connection()`
(`ledger_entry.rs:678`), i.e. only on a successful outbound dial by this node.

Chain from the UI to that filter, each hop verified:
`PeerListScreen.kt:40-44` -> `DashboardViewModel.peers` (`:37-38`, populated by
`loadPeers()` `:117-203`) -> `DashboardViewModel.kt:121`
`meshRepository.getDialableAddresses()` -> `MeshRepository.kt:5545-5546`
`ledgerManager?.dialableAddresses()` -> UniFFI
`core/target/generated-sources/uniffi/kotlin/uniffi/api/api.kt:8347` ->
`ledger_entry.rs:786`.

## A second, independent surface with the same symptom

The Dashboard stat cards ("N Node / N Headless", `strings.xml:256-257` -- an
exact match for the operator's "4 nodes plus 1 headless" phrasing) read
`meshRepository.discoveredPeers` ONLY (`DashboardViewModel.kt:53,59,66` --
verified: all three counters map over `discoveredPeers`, none touch the
ledger). `discoveredPeers` is fed exclusively by direct swarm
discovery/identify callbacks (`mobile_bridge.rs:921-937` ->
`iron_core.rs:1142-1168`).

So gossip-learned peers are **structurally incapable** of moving that counter,
by design. If the operator's "listing" was the dashboard cards, the ledger is
irrelevant to it and the real question becomes why Android's own direct
discovery is not finding the other LAN devices.

**Both surfaces must be checked before declaring a fix**, because they fail
for two different reasons.

## Why the condition is durable rather than transient

`connect_to_seed_peers()` (`swarm.rs:2390-2409`) runs exactly once at service
startup (`mobile_bridge.rs:809-822`) and pulls at most
`SEED_DIAL_LEDGER_CANDIDATES = 8` (`swarm.rs:749`, verified) unproven entries.
If the ledger exchange lands after that sweep, none of the 64 new addresses is
ever dialed, so none can promote to `success_count > 0`, so none can ever
become visible -- until an app restart. That makes this a permanent-looking
gap rather than a slow-convergence one, matching the field report.

## Ingest is NOT the problem -- ruled out

Ledger-exchange responses are ingested entirely in shared Rust, identical on
every platform: `swarm.rs:4411-4462` emits `LedgerReceived`;
`mobile_bridge.rs:1091-1148` filters via `addr_filter::is_dialable_multiaddr`
and calls `annotate_identities_batch`. There is no Kotlin-side gate on this
path -- Kotlin never sees the entries in transit.

Also ruled out: a split-brain between Kotlin's `LedgerManager(storagePath)`
handle (`MeshRepository.kt:915`) and the swarm's `core.ledger_manager`.
`LedgerManager::new()` resolves a process-local registry keyed by normalized
path (`ledger_entry.rs:141-163`) and clones the same
`Arc<Mutex<Vec<LedgerEntry>>>`.

## Secondary silent-discard worth a log line

`DashboardViewModel.kt:163`: `val rawPeerId = entry.peerId ?: return@forEach`
drops any entry with a null `peerId`. Not the active cause for this batch
(ledger-exchange entries carry a peer_id -- `mobile_bridge.rs:1134-1141` only
accepts when `last_peer_id` is `Some`), but it silently discards
seed/invite-imported entries, which per `ledger_entry.rs:877` intentionally
carry no peer_id.

## Falsification step to run BEFORE any fix

On Android, log at the same instant: `ledgerManager?.getLedgerSummary()` total
(`MeshRepository.kt:5709-5710`), `getDialableAddresses().size`
(`:5545-5546`), and `discoveredPeers.value.size`. If total is ~64+ while
dialable is ~1, cause 1 is confirmed. If `discoveredPeers` stays at 1
regardless, cause 2 is confirmed and the investigation moves to Android's
direct LAN discovery.

## Fix direction (NOT implemented; needs the adversarial gate)

The filter conflates "proven dialable by me" with "known to exist". Those are
different questions and the UI is asking the second one while calling the
first. Options: surface unproven entries as a distinct, visibly-unverified
tier rather than hiding them; or give the peer list its own accessor that does
not gate on `success_count`. Do NOT simply drop the `success_count` filter
from `dialable_addresses()` -- that function is also used for relay selection
(`get_preferred_relays()`, `ledger_entry.rs:903-914`, called from
`MeshRepository.kt:1285,5559,8932,8996,9076,9372`), where "proven" is exactly
the right semantics. Changing it in place would degrade relay choice.

## Provenance

Traced by a Sonnet subagent (read-only, no writes, no builds). Every
file:line, and all four load-bearing claims -- the filter, the zero-init, the
dashboard counters, and the seed cap -- were re-verified directly by the
orchestrator against source before this document was written.
