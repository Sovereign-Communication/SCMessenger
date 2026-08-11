# ORCHESTRATOR HEADER -- read before trusting the audit body below

Dispatched 2026-08-09, lake `qwenpaid`, model `qwen3.8-max-preview`, THINK
tier, read-only audit, 1 round. Ledger: `LEDGER_VISIBILITY_AUDIT` result ok.
Footer parsed clean (`degraded: false`, `RESULT: DONE`,
`VERIFICATION: NONE` -- correct; this lane has no execution environment).

Source packet: `HANDOFF/todo/DISPATCH_AUDIT_LEDGER_VISIBILITY_GAP_2026-08-05.md`
Ticket: `HANDOFF/todo/LEDGER_SHARING_ANDROID_NODE_VISIBILITY_2026-08-05.md`

Context supplied: `core/src/store/ledger_entry.rs` (full),
`core/src/identity/keys.rs` (full), `core/src/transport/swarm.rs:L3900-L4030`
(scoped). The audit was NOT given `addr_filter.rs` or any Kotlin source.

## Orchestrator verification of the audit's claims (spot-checked at HEAD)

CONFIRMED:
- `core/src/transport/addr_filter.rs` exists (61 KB, modified 2026-08-09).
  The model named a file it was not given and the file is real -- not a
  hallucinated path.
- `exchange_response_entries` (ledger_entry.rs:1159) does take `my_addrs`, and
  delegates to `exchange_response_entries_for_request` (:1171) passing `None`
  for the requester address (:1165).

REFUTED -- do not action the audit's stated fix direction as written:
- The audit's fix direction says to "pass requester_addr and my_addrs
  correctly in swarm exchange handlers". The production handler ALREADY does
  this. `core/src/transport/swarm.rs:4370-4382` resolves `requester_addr` from
  `connection_tracker.get_connection_by_id(...).remote_addr`, collects
  `my_listener_addrs` from `swarm.listeners()`, and calls the `_for_request`
  variant with both. The `None`-passing wrapper at ledger_entry.rs:1165 is
  used only by tests in this repo, not by the swarm handler. The audit
  reasoned from the scoped range it was given (L3900-L4030) and never saw
  line 4370.

STILL OPEN (the actual lead worth pursuing):
- WHETHER the RFC1918 / same-subnet disclosure predicate in `addr_filter.rs`
  returns false for this fleet's requester addresses even though the handler
  supplies them correctly.
- The Android rendering surface (which Kotlin/UniFFI call the node list reads,
  and whether it shows only "proven" entries) is entirely unverified -- no
  Kotlin source was in context.

## Primary field evidence this must explain

Windows node, 2026-08-08: `Ledger exchange response from <Android peer>:
they learned 1 new peers, sent 64 back`. 64 is exactly the cap, so the
SENDER was not filtered down; the loss is on the Android INGEST or RENDER
side, not in what was transmitted. Weigh that against any hypothesis that
blames the sender's disclosure filter.

## Gate status

AUDIT-GATE: any resulting fix touches `core/src/transport/` and therefore
carries the mandatory adversarial review (AGENTS.md rule 8 /
ORCHESTRATION.md Section 4) before merge. No code change has been made.

---

## READ-ONLY AUDIT: Android Ledger Visibility Gap

### 1. What is reciprocated, and what the 64/1 evidence means

`core/src/store/ledger_entry.rs:1170` `exchange_response_entries()` builds the reciprocal ledger payload from the local `LedgerManager` entries, then `core/src/store/ledger_entry.rs:1227` `ledger_entry_to_shared_routing_only()` converts each selected `LedgerEntry` into the wire form.

The disclosure filter is strict:

- Only entries with `success_count > 0` and `failure_count < LEDGER_DEAD_FAILURE_THRESHOLD` are eligible. Seed entries imported from invites or learned but never successfully dialed remain `success_count == 0` and are not shared.
- The requester’s own `peer_id` is excluded.
- The multiaddr is stripped of `/p2p/` components.
- Globally routable addresses pass `is_disclosable_multiaddr`.
- RFC1918 / CGNAT / ULA addresses pass only if `is_disclosable_on_rfc1918_network` has evidence that the requester is on the same private network, using the observed requester transport address and our own listener addresses.
- The limit — observed as 64 — is applied before conversion to `SharedPeerEntry`.
- `known_topics` is forcibly emptied.
- `public_key`, `nickname`, `success_count`, and `failure_count` are not part of `SharedPeerEntry`.

Therefore, the reciprocal response is not “the fleet contact list.” It is a routing-only set of proven, disclosable addresses. If iOS’s 4 nodes plus 1 headless node are represented locally as unproven, dead-marked, private without same-subnet evidence, or beyond the 64-entry window, they can be absent from a response to a freshly paired Android peer.

The Windows evidence — “they learned 1 new peers, sent 64 back” — must be interpreted carefully. `LedgerExchangeResponse.new_peers_learned` is the number of inbound entries the Android node accepted as new from the Windows request. `peers` is the number Android sent back. These are independent counters. “Learned 1” does not necessarily mean “1 of 64 offered.” It may mean Windows offered only one usable entry, or Android deduplicated/rejected the rest. “Sent 64” means Android had at least 64 eligible entries at that moment, or the response hit the cap. If the Android install was truly fresh and had only Christy, sending 64 is inconsistent with `exchange_response_entries_for_request`. That suggests either prior persisted state, an ingestion path that incorrectly promotes learned entries to proven status, or a platform-side ledger not visible in this audit.

### 2. Ingestion and identity canonicalization

The missing swarm handler range `core/src/transport/swarm.rs:3936-4010` and the handlers at `core/src/transport/swarm.rs:5668` and `core/src/transport/swarm.rs:6224` are decisive for ingestion. The available store layer has two relevant write paths:

- `LedgerManager::record_connection` creates or updates entries with `success_count > 0`. This is the proven tier and is later eligible for disclosure.
- `LedgerManager::annotate_identity` and `annotate_identities_batch` create entries with `success_count = 0`. These are routing seeds, not proven peers.

If the ledger-exchange response is ingested through `annotate_identity` or a seed-import path, Android will learn addresses but will not consider them proven. They will not appear in `dialable_addresses()`, and they will not be re-shared until Android successfully dials them and records a connection. That is a major convergence gate.

Identity canonicalization under PR #136 is less likely to be the direct cause for ledger-exchange entries. `core/src/identity/keys.rs` `identity_id_from_public_key_hex()` derives an identity id only from a valid Ed25519 public key. It rejects inputs that are not valid curve points. However, `SharedPeerEntry` does not carry a public key at all. It carries only `multiaddr`, `last_peer_id`, `last_seen`, and now-empty `known_topics`. Therefore, ledger-exchange ingestion cannot directly mis-key via `identity_id_from_public_key_hex` unless a higher mobile bridge or contact layer invents or confuses a public key. If Android’s UI or contact store tries to canonicalize `last_peer_id` or some unrelated 64-hex identifier as a public key, `identity_id_from_public_key_hex()` may return `None`, and a careless caller could drop the entry. But from the attached Rust surface, the primary ledger key is still the stripped multiaddr, with optional `peer_id`, not the identity id.

A mis-key/drop hypothesis can explain “learned 1” only if the ingestion path is performing extra identity mapping not visible here. That requires inspection of the swarm response handler and the Android UniFFI bridge.

### 3. Rendering: what Android node listing actually reads

The Android Kotlin sources were not provided, so the exact Kotlin file names cannot be named without guessing. The audit can identify the Rust/UniFFI surfaces that must be traced:

- `LedgerManager::dialable_addresses`: returns only proven entries with `success_count > 0` and fewer than three failures.
- `LedgerManager::seed_addresses`: returns unproven seed entries, bounded by a caller-provided limit.
- `LedgerManager::summary`: count-only diagnostic.
- `LedgerManager::annotate_identity` / `annotate_identities_batch`: where wire-learned identity metadata may be attached.
- `LedgerManager::exchange_response_entries`: what Android sends, not what it renders.

The exact Kotlin/UniFFI surface to open is therefore: the Android node-list ViewModel or repository that calls any generated `LedgerManager` method, especially `dialableAddresses()` or `seedAddresses()`, and any contacts/identity store that filters by public key, nickname, pairing state, or verified status.

INSUFFICIENT: Android Kotlin repository paths, generated UniFFI binding file names, and the node-list UI source are needed to name exact Kotlin files and symbols. The likely failure is that the Android listing reads only proven or identity-annotated records, while ledger exchange has only produced unproven routing seeds.

### 4. Why iOS sees five nodes while Android sees one

The most coherent asymmetry is: iOS has been in the fleet longer and has accumulated proven, successfully connected ledger entries. Android is fresh and has only one directly proven relationship: Christy. Ledger sharing is not sufficient for convergence if learned entries remain seed-tier. Convergence requires:

1. Receive ledger entries.
2. Dial the learned addresses.
3. Succeed and call `record_connection`.
4. Learn identity via Identify and annotate.
5. Re-export those entries in later ledger exchanges.

If Android does not actively sweep `seed_addresses`, or if its UI only displays proven/contact-list entries, it will remain stuck at one visible node even after receiving more addresses.

RFC1918 disclosure rules can compound this. On a home network, most fleet entries are probably private addresses. If the swarm call does not provide the observed requester address or the node’s own LAN listener addresses, `exchange_response_entries_for_request` fails closed and withholds private entries. That can make a LAN fleet invisible to a newly paired peer.

---

## Ranked hypotheses

### H1 — RFC1918 disclosure fail-closed for LAN fleet entries

Most likely.

Evidence for:
- The fleet is on the same home network, so the missing nodes are likely RFC1918 entries.
- `exchange_response_entries_for_request` discloses private entries only when `is_disclosable_on_rfc1918_network` has matching requester/listener evidence.
- If `requester_addr` is `None`, private entries fail closed.
- iOS may see nodes because it already has direct proven connections, while Android cannot learn them through ledger exchange.

Evidence against:
- Android “sent 64 back” implies it had many eligible entries, unless those were public, old, or incorrectly promoted.
- If the missing nodes are globally routable, this hypothesis weakens.

Sites to inspect:
- `core/src/transport/swarm.rs:3936-4010`
- `core/src/transport/swarm.rs:5668`
- `core/src/transport/swarm.rs:6224`
- `core/src/store/ledger_entry.rs:1170`
- `core/src/transport/addr_filter.rs` — needed but not provided.

Fix sketch:
Ensure the swarm handler passes both the observed requester transport address and all relevant local listener addresses into `exchange_response_entries_for_request`. Add structured logs showing why private entries were included or excluded. Do not weaken the privacy gate; improve evidence collection.

Adversarial gate:
Yes. Any change under `core/src/transport` or privacy disclosure logic requires AGENTS.md rule 8 review.

### H2 — Android ingests learned peers as unproven seeds and UI hides them

Very likely, possibly co-primary with H1.

Evidence for:
- `annotate_identity` creates entries with `success_count = 0`.
- `dialable_addresses` requires `success_count > 0`.
- `seed_addresses` exists separately because seeds are not proven.
- Android showing only Christy matches a UI that displays only proven or paired contacts.
- “Learned 1” may mean only Christy became proven; other learned addresses remained invisible seeds.

Evidence against:
- “Sent 64” suggests Android already had many eligible proven entries, unless persisted state existed or promotion is happening.
- If Android UI reads all ledger entries, this hypothesis fails.

Sites to inspect:
- `core/src/transport/swarm.rs:5668`
- `core/src/transport/swarm.rs:6224`
- `LedgerManager::annotate_identity`
- `LedgerManager::annotate_identities_batch`
- `LedgerManager::dialable_addresses`
- `LedgerManager::seed_addresses`
- Android Kotlin node-list ViewModel — INSUFFICIENT context.

Fix sketch:
If doctrine requires fleet visibility from ledger seeds, Android must either render headless seed entries or actively dial them via a seed sweep. After successful connection, `record_connection` promotes them. Do not mark wire-learned entries as proven without an actual successful dial.

Adversarial gate:
Yes if the fix changes `core/src/transport/swarm.rs`, dial policy, or routing behavior. UI-only rendering changes are not in the mandatory crypto/transport/routing/privacy set, but should still be reviewed for information exposure.

### H3 — 64-entry cap and lack of prioritization hide relevant entries

Possible.

Evidence for:
- The response was exactly 64, the cap.
- `exchange_response_entries_for_request` applies `.take(limit)` without explicit ranking for recency, same-subnet relevance, or contact priority.
- If iOS or Android has a large older ledger, the first 64 eligible entries may omit the local fleet.

Evidence against:
- iOS currently lists only 5 nodes, so the active fleet may be small.
- If the ledger contains fewer than 64 eligible entries, the cap is not the cause.

Sites to inspect:
- `core/src/store/ledger_entry.rs:1170`
- `core/src/store/ledger_entry.rs:1227`
- Swarm call sites that choose the limit value: `core/src/transport/swarm.rs:3936-4010`, `:5668`, `:6224`.

Fix sketch:
Before applying the cap, prioritize entries that are same-subnet, recently seen, contact-associated, or already connected. Preserve the hard cap. Add tests that a small LAN fleet is not evicted by old public entries.

Adversarial gate:
Recommended. This is store logic, but it changes disclosure ordering and may affect privacy leakage.

### H4 — Identity canonicalization drops or mis-keys foreign entries

Less likely for ledger exchange.

Evidence for:
- PR #136 introduces public-key-based identity ids.
- `identity_id_from_public_key_hex` returns `None` for invalid public keys.
- If Android’s higher layer treats a 64-hex identifier as a public key incorrectly, entries could be dropped.

Evidence against:
- `SharedPeerEntry` has no public key field.
- Ledger entries are keyed by stripped multiaddr and optional `peer_id`.
- No attached Rust path shows ledger-exchange ingestion calling `identity_id_from_public_key_hex`.

Sites to inspect:
- `core/src/identity/keys.rs` `identity_id_from_public_key_hex`
- Swarm ingestion handlers: `core/src/transport/swarm.rs:5668`, `:6224`
- Android UniFFI/contact bridge — INSUFFICIENT context.

Fix sketch:
Do not derive identity ids from absent or ambiguous data. Ledger-exchange entries should remain routing seeds until a live connection identifies the peer. If a caller receives `None` from canonicalization, it should preserve the routing entry rather than discard it.

Adversarial gate:
Yes if changes touch `core/src/identity` or crypto identity handling.

### H5 — Android storage is not actually fresh or uses a different ledger scope

Possible but lower priority.

Evidence for:
- `LedgerManager::new` uses path normalization and a process-local shared registry.
- `LedgerManager::ephemeral` never persists.
- If Android uses an ephemeral manager or a different storage path, convergence state can differ from expectations.
- “Sent 64” from a supposedly fresh node suggests prior state or a different ledger source.

Evidence against:
- No attached Android bootstrap code shows the storage path.
- The bug is specific to listing visibility, not necessarily persistence.

Sites to inspect:
- `LedgerManager::new`
- `LedgerManager::ephemeral`
- Android core initialization / UniFFI bridge — INSUFFICIENT context.

Fix sketch:
Log the normalized ledger storage path at startup. Confirm Android and the platform bridge use the same durable `LedgerManager` instance. Ensure an app reinstall or “fresh build” actually clears or retains ledger state as intended.

Adversarial gate:
No, unless it touches transport or privacy disclosure.

---

## Missing context required

- `core/src/transport/swarm.rs` full relevant ranges around 3936-4010, 5668, and 6224.
- `core/src/transport/addr_filter.rs` for `is_disclosable_multiaddr`, `is_disclosable_on_rfc1918_network`, `is_recordable_multiaddr`, and `strip_peer_id`.
- Android Kotlin sources or generated UniFFI bindings showing the node-list data source.
- Mobile bridge code that calls `LedgerManager` from Android.
- iOS node-list implementation to confirm whether it renders ledger entries, contacts, or routing table.
- Windows CLI log source to confirm exact semantics of “learned” and “sent”.

---ORCHESTRATION_METADATA---
RESULT: DONE
VERIFICATION: NONE
FILES: ["core/src/store/ledger_entry.rs", "core/src/identity/keys.rs", "core/src/transport/swarm.rs", "core/src/transport/addr_filter.rs"]
NOTES: ["top hypothesis: RFC1918 same-subnet disclosure fail-closed plus Android rendering only proven/seed-promoted entries", "fix direction: pass requester_addr and my_addrs correctly in swarm exchange handlers, then make Android promote or display seed entries without marking them proven", "orchestrator must run next: read swarm.rs:3936-4010, swarm.rs:5668, swarm.rs:6224, addr_filter.rs, and Android Kotlin/UniFFI node-list surface"]
---END---