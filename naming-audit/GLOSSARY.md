# Glossary — Draft Canonical Vocabulary (PROPOSED)

> **Status: proposal, not decree.** This file is for the repository owner to accept, amend, or
> reject. Every "proposed canonical" below is a recommendation with a stated reason, not a decision
> that has been made.

## Every count in this file is reproducible

    python scripts/measure_uncompiled_counts.py            # the table
    python scripts/measure_uncompiled_counts.py --json     # machine-readable

**Basis: defined once, in the `BASIS` block of `scripts/measure_uncompiled_counts.py`,
and deliberately not restated here.** That file owns it because it is what implements the
rules — a second copy in prose would be a second thing to keep in sync, and this report
already had two that had begun to disagree. The gist, offered as a pointer and not as a
second definition: first-party source, whole-word, case-sensitive, over a documented set
of exclusions, with three terms filtered because a raw count of them is meaningless. The
command above prints the corpus and uncompiled totals on every run — currently **341
files** and **7 uncompiled**, see **D-4** — so a reader never has to trust this sentence
over what the tool says.

**Why case-sensitive.** Case-insensitive whole-word matching pulls `GET` in on every HTTP
route, which would inflate the read-verb cluster with transport verbs.

**There is a second basis, and it is not this one.** FINDINGS publishes counts for names
that live mostly in tests, examples and generated bindings — files this basis excludes on
purpose. Those are measured on a wider corpus with the same counting rule:
`python scripts/measure_uncompiled_counts.py --basis wide`. Every figure in this file is
on the default basis above; the script's docstring states both.

**The counts here were restated onto this basis, and most moved.** They were previously
taken on a basis that was recorded nowhere: fifteen corpus definitions were tried against
the published figures and none reproduced them. Two things changed at once — the corpus
narrowed to first-party code, and matching became case-sensitive — so nearly every figure
falls. **This was a basis change, not a set of corrections**, and where a figure fell far
enough to change what it measures, the entry says so. Every entry whose figures moved
carries a *Count basis* note recording what it was before. `TransportType` (710), `peer`
(3461), `config` (998) and `MessageRecord` (125) are unchanged and needed no note.

## The convention, applied identically in all twelve entries

Every concept entry has the same two blocks, and **no sentence or cell belongs to both**:

| Block | Heading | Contains | Never contains |
|---|---|---|---|
| 1 | **`Inventory — what exists today`** | a table of names **found in the repository**, each with an occurrence count and representative locations | any recommendation, any status, any "canonical" column |
| 2 | **`Decision — what this entry proposes`** | `Canonical`, `Why`, `Rename plan`, `Risk`, `Effort`, `Status` | a count or a source location |

The split exists because a reader must be able to tell an existing name from a proposed one without
parsing prose. In the inventory tables, **every row is a fact about the repository as it stands** —
if a name is not in the tree, it does not appear there, not even as a suggestion. Proposals live only
in the decision block, and a proposed name that does not yet exist is marked *to be created*.

**Status vocabulary** — the `Status` field of every decision block:

| Value | Meaning |
|---|---|
| `PROPOSED` | Recommended, **not yet ratified** by the repo owner. The default for most entries. |
| `ACCEPTED-AS-IS` | Reviewed and judged already correct. A deliberate decision to change nothing — see the entry's `Rename plan` line, which says so explicitly. |
| `FROZEN` | **Settled by the public API, a serialized format, a storage schema, or a URL.** Do not rename; see §F for the compatibility approach. |
| `BLOCKED` | Cannot be decided without an answer to an open question; the `Q-xx` is named in the same line. |
| `DECIDED-BY-DOCTRINE` | `AGENTS.md` has already settled it. Listed so the code can be checked against it. |

**Five required fields**, present in all twelve entries: the concept in one plain-English sentence;
every name found, with locations and occurrence counts; a recommended canonical name and why; a
rename plan with risk and effort notes; and a status.

**Scope of counts:** the corpus the `BASIS` block of `scripts/measure_uncompiled_counts.py`
defines — pointed at, not restated. An earlier version of this line spelled the scope out as
a directory list and was wrong in two ways: it claimed `cloud/` and `ui/` were counted,
where the rule that actually runs puts **0 files** from either in the corpus, and it named
an inclusion set the rule does not have. A definition copied into a second place is a
definition that will be wrong in two places.

---

## A. Entities

### A-1. The other party in the mesh

> **Concept:** any node other than this one, identified by a public key.

**Inventory — what exists today**

| Name in the tree | Count | Representative sites |
|---|---|---|
| `Peer` | 3461 / 161 files | no single site — it is the default term and genuinely has no densest location; seen at `core/src/store/contacts.rs:31`, `core/src/transport/swarm.rs:2705` |
| `Contact` | 1378 / 63 files | `core/src/store/contacts.rs:25`, `core/src/contacts_bridge.rs:19`, `core/src/api.udl:292` |
| `Node` | 567 / 64 files | `core/src/transport/swarm.rs:2760` (`SwarmHandle`), `core/src/iron_core.rs:167` — means the **local** process in much of the codebase and *any* party in others |
| `Relay` | 1392 / 97 files | `core/src/store/relay_custody.rs:333` — historical identifier, see A-2 |

> **Count basis.** Previously published as `Peer` 20887, `Contact` 3757, `Node` 1797,
> `Relay` 5419. Now 3461, 1378, 567, 1392. The fall is the basis change, not a
> retraction: the old figures counted the token case-insensitively across a corpus that
> included docs, scripts and tests, and so swept in `PEER`, `RELAY` and similar.
>
> **These four numbers are a floor, and there is a second reason for the fall that the
> sentence above does not name.** Whole-word matching does not cross `_` or a case change,
> so 3461 counts the bare word `peer` and **excludes** `peer_id` (3155), `peerId` (2151),
> `peers` (949), `peer_ids` (10) and `peerIds` (11). The peer vocabulary is about
> **9737**, not 3461. The same applies to the rest of the row: `relay` excludes `relays`
> (176), `RelayCustodyStore` (64), `relay_custody` (12), `relay_id` (4) and `relayId` (4),
> for 1652; `contact` excludes `contacts` (513) and `contactId` (30), for 1921; `node`
> excludes `nodes` (192), `node_id` (19) and `nodeId` (18), for 796. Every one of those
> compounds is in the script's term list and regenerates with the same command.
>
> One of them is worth a reader's attention on its own: **`contact_id` appears zero
> times.** The three headwords are not spelled consistently with each other. `peer` uses
> both conventions heavily (`peer_id` 3155, `peerId` 2151); `node` uses both sparingly
> (`node_id` 19, `nodeId` 18); `contact` uses camelCase only (`contactId` 30,
> `contact_id` 0). **Which convention a headword adopts is decided per headword, not per
> project** — a fact a frequency count on one term would never have surfaced, and one that
> matters for F-09's `PeerKey` work, because a new enum has to pick a convention per
> variant rather than inherit one.
>
> **This moves the ranking in the middle, so an earlier version of this note claimed more
> than it could support.** On bare words: `Peer` 3461 > `Relay` 1392 > `Contact` 1378 >
> `Node` 567. With compounds: `Peer` 9737 > `Contact` 1921 > `Relay` 1652 > `Node` 796 —
> **`Contact` and `Relay` swap**, and they were 14 occurrences apart to begin with, so
> their order was never a real signal. The claim this entry rests on survives both, and is
> stronger with compounds: `Peer` is first by 7816 over the runner-up, and the two
> runners-up are 269 apart from each other. Read the four numbers as *the bare word
> appears N times*, not *the concept is used N times*.

**Decision — what this entry proposes**

- **Canonical:** `Peer`.
- **Why:** the most-used name by a factor of two and a half on bare words, and of five once
  compound identifiers are counted, the name the wire format and libp2p already use,
  and the only one unambiguous between "the other device" and "this device" — `Node` is currently
  doing double duty for both, which is the actual defect.
- **Rename plan:** none required for the name itself. `Contact` is a *legitimate distinct concept*
  and should be kept but scoped: a **Contact** is a Peer the local user has *deliberately saved*
  (it has `added_at`, `notes`, `local_nickname`, and lives in the contacts store); a **Peer** is
  anything the node has heard of. Today `ContactManager::add(peer_id)` and
  `ContactManager::get(peer_id)` blur this — they take a peer id and return a contact. Fix by
  naming the *key* `peer_id` (already correct) and stopping the use of `Contact` to mean "peer".
- **Risk:** MEDIUM. `Contact` is a UniFFI-boundary type consumed by both mobile clients. No wire
  keys change if only the mental model is documented.
- **Effort:** S to document the distinction; L to restructure the store around it.
- **Status:** `PROPOSED` for `Peer` and the `Contact` scoping.

---

### A-2. Store-and-forward custody (the project-specific term)

> **Concept:** the behaviour by which a node holds a message on behalf of others because it could
> not deliver it directly.

**Inventory — what exists today**

| Name in the tree | Where it appears | Note |
|---|---|---|
| `node` | prose throughout | the role name |
| `relay` *(as a noun for a role)* | legacy type and field names | the role name it replaced |
| `relay` *(as a verb)* | `relay_custody_*`, `can_forward_for_wasm` | permitted by AGENTS.md |
| `isRelay` | `android/.../data/MeshRepository.kt:809` | a **data field** named for a role that no longer exists |

**Decision — what this entry proposes**

- **Canonical:** `node` for the role; `relay` is a verb only.
- **Why:** AGENTS.md settles it — there are no standalone relays, every node relays. Historical
  code identifiers (`RelayCustodyStore`, `cmd_relay`, `relay_custody_msg_*`) are grandfathered by
  name, so no rename is proposed for them.
- **Rename plan:** delete `isRelay`. Its own comment says it no longer distinguishes anything and is
  always false, and it sits beside `isFull`, which is not. See FINDINGS F-13.
- **Risk:** LOW — the field is defaulted, so removal is source-compatible for construction sites and
  breaking only for readers.
- **Effort:** S.
- **Status:** `DECIDED-BY-DOCTRINE` for the vocabulary; the `isRelay` deletion is `PROPOSED`.

---

### A-3. The message as it is stored

> **Concept:** one row in local message history.

**Inventory — what exists today**

| Name in the tree | Count | Representative sites |
|---|---|---|
| `MessageRecord` — storage-layer struct | 125 / 24 files (shared count) | `core/src/store/history.rs:17` |
| `MessageRecord` — FFI-layer struct | 125 / 24 files (shared count) | `core/src/mobile_bridge.rs:3126` |
| `MessageRecord` — UDL dictionary | 125 / 24 files (shared count) | `core/src/api.udl:324` |
| `HistoryStats` | 15 / 9 files | `core/src/store/history.rs:81`, `core/src/mobile_bridge.rs:3151` — duplicated, see F-22 |
| `MessageDto`, `HistoryMessage` | not separately counted | `cli/src/api.rs` — the transport shape has no consistent name |
| `StoredMessage` | 19, **all in unreachable files** | `wasm/src/storage.rs:47` — defined nowhere the build reads; FINDINGS F-33. **This name is not free**: F-02 offers it as an alternative, and A-3's decision below deliberately does not. |

The one name `MessageRecord` covers three declarations, two of which have different field sets
(FINDINGS F-02).

> **Count basis.** `MessageRecord` was 125 and is still 125 — unchanged. `HistoryStats`
> was 45, now 15. The old figure swept in the `HistoryStats` fields on the FFI copy as
> well as the storage copy; the new one is the same term measured the same way as every
> other row here.

**Decision — what this entry proposes**

- **Canonical:** `MessageRecord` for the stored row; `MessageRecordDto` for any transport shape.
- **Why:** the name is already established and is frozen by the UDL. What is missing is a name for
  the DTO *role* — today the same name is used for two structs with different fields and the DTO
  role is not named at all.
- **Rename plan:** keep `MessageRecord`; introduce `MessageRecordDto` for the transport shape; merge
  the two conflicting structs (FINDINGS F-02, F-22).
- **Risk:** HIGH. `MessageRecord` is a UDL dictionary bound by both mobile clients; the `status` /
  `delivered` reconciliation is a wire change. Compatibility approach in **§F**.
- **Effort:** L — this rides on the bridge-layer consolidation (F-02, F-22), not on a standalone
  rename.
- **Status:** `FROZEN` for the name (a UDL dictionary, both mobile clients bound); the DTO split is
  `PROPOSED`.

---

### A-4. The encrypted unit on the wire

> **Concept:** an encrypted, self-describing message container.

**Inventory — what exists today**

| Name in the tree | Count | Representative sites |
|---|---|---|
| `Envelope` | 51 / 13 files | no single densest site — spread across all layers by design; `core/src/message/types.rs:147`, `core/src/transport/behaviour.rs` |
| `WireEnvelope` | (subset of the above) | `core/src/message/types.rs:147` — the serialized form |
| `DriftEnvelope` | 74 / 9 files | `core/src/drift/envelope.rs:38` — the drift protocol's own wire format |
| `payload`, `packet`, `frame`, `blob` | 689 / 32 / 92 / 13 | `core/src/transport/swarm.rs` (all four) — transport-layer terms one level down |
| `envelopeData` / `envelope_data` | 41 / 4 files and 172 / 20 files | `core/src/api.udl:63` — the *serialized* bytes, held distinct from the parsed `Envelope` |

> **Count basis — this entry changed what it measures, so read it before using the
> numbers.** `Envelope` was published as 2184 across 93 files and is now 51 across 13.
> The old figure was a case-insensitive count of the *word* `envelope`; the new one
> counts the *type* `Envelope`. Those are different questions, and for an entry about a
> type the second is the right one — but a reader who took 2184 as "how often the type
> is named" will find the number has fallen by 40x, and the reason is the question
> changed, not the code. `payload`/`packet`/`frame`/`blob` were 1582/163/269/51 and are
> now 689/32/92/13 for the same reason. **The structural claim this entry makes — that
> four transport-layer words sit one level below `Envelope` and mean different things —
> rests on the field sets, not on the counts, and is unaffected.**
>
> The `envelopeData` row is new to this entry. The name was first reported in FINDINGS'
> tautology scan at 57 across 10 files on the old unrecorded basis; here the two spellings
> are counted separately — `envelopeData` 41 across 4 files, `envelope_data` 172 across 20 —
> and the fall is the same basis change as the rest of the entry. **Use the 20-file figure
> when you want the field; the 4-file one is the camelCase spelling alone.**

**Decision — what this entry proposes**

- **Canonical:** `Envelope` for the domain object, `WireEnvelope` for its serialized form,
  `DriftEnvelope` for the drift protocol's variant.
- **Why:** the pairing is already consistent and correct. `DriftEnvelope` is defensible in its own
  right because drift is a distinct versioned wire protocol. `payload`/`frame`/`packet` are
  transport-layer terms a level down and are not competing for the same slot.
- **Rename plan:** **none. This is a deliberate decision to change nothing**, not an unfinished
  entry — the cluster was audited, found coherent, and is recorded here so a future reviewer does not
  re-audit it. The one thing that is *not* settled is `DriftEnvelope`, which is accepted on the
  reasoning above rather than on evidence.
- **Risk:** LOW — no change proposed, so no risk is incurred.
- **Effort:** S — verification only, nothing to implement.
- **Status:** `ACCEPTED-AS-IS`, deliberately. Not `PROPOSED`: there is nothing awaiting ratification.

---

### A-5. The identifier of a peer

> **Concept:** a stable string naming a peer, in one of several encodings.

**Inventory — what exists today**

| Name in the tree | What it holds | Count | Representative sites |
|---|---|---|---|
| `PeerId` | the libp2p multihash string, `12D3KooW…` | the default form | `libp2p` crate, all Rust, all wire formats; e.g. `core/src/identity/keys.rs:776` |
| `canonicalPeerId` / canonical hex | 64-char hex of the Ed25519 key | 279 / 7 files | `android/.../MeshRepository.kt:736`, `core/src/store/ledger_entry.rs:489` |
| `routePeerId` | the peer to send via, on a chosen path | 309 / 4 files | `android/.../MeshRepository.kt:765` |
| `blePeerId` | BLE-specific encoding | 184 / 13 files | `android/.../MeshRepository.kt:730` |
| `libp2pPeerId` | libp2p-specific encoding | 368 / 21 files | `android/.../MeshRepository.kt:841` |
| `identity_id` | a Blake3 hash of the public key | 605 / 32 files | `core/src/lib.rs:116`, defined with its siblings at `core/src/identity/keys.rs:87-95` |

> **Count basis.** `canonicalPeerId` was 279 (Android) and is now 279 / 7 files — the
> figure itself did not move. The four rows that read "—" now carry counts, because the
> measurement script can measure them: `routePeerId` 309 / 4, `blePeerId` 184 / 13,
> `libp2pPeerId` 368 / 21, `identity_id` 605 / 32. **These were never absent from the
> tree; the earlier entry recorded a location and declined to count them.** A reader who
> has read "no count recorded" should now use these figures.

**Decision — what this entry proposes**

- **Canonical:** `PeerId` for the libp2p form. For the others: **to be created** — `PeerKeyHex`, or
  better a `PeerKey` enum per FINDINGS F-09.
- **Why:** `canonical` names a *policy*, not a kind, and is doing the job of a type. `PeerId` itself
  is libp2p's name and a wire-format identifier, so it is not the glossary's to rename.
- **Rename plan:** rename the non-libp2p encodings to say what they are rather than to claim
  canonicity. **Whether the hex form is permanent is unresolved** (Q-3): permanent ⇒ the `PeerKey`
  enum with `Hex` as a first-class variant; transitional ⇒ a dual-read adapter that retires it.
- **Risk:** HIGH — the hex form is what the ledger persists. Compatibility approach in **§F**.
- **Effort:** L.
- **Status:** `FROZEN` for `PeerId` and `identity_id` (both serialized / libp2p-owned);
  `BLOCKED` on **Q-3** for the hex form, `routePeerId` and the transport-specific encodings.

---

## B. Operations

### B-1. Read verbs

> **Concept:** obtaining a value the system did not just compute.

**Inventory — what exists today**

| Verb in the tree | Count | Representative sites |
|---|---|---|
| `get` | 663 / 113 files | no single densest site — `get` is the default verb; `core/src/iron_core.rs:1266` (`get_registration_state`), `android/.../MeshRepository.kt:437` |
| `read` | 493 / 76 files | `core/src/iron_core.rs` |
| `list` | 363 / 79 files | `core/src/store/contacts.rs:782` (`list`), `android/.../DashboardViewModel.kt` |
| `find` | 133 / 40 files | `core/src/relay/findmy.rs`, `core/src/store/ledger_entry.rs` |
| `load` | 203 / 36 files | `core/src/store/ledger_entry.rs:845` (`pub fn load`) |
| `resolve` | 95 / 26 files | `core/src/iron_core.rs`, `android/.../MeshRepository.kt` |
| `poll` | 25 / 11 files | `cli/src/api.rs:1547` (`handle_poll_status`), `cli/src/bin/heartbeat-probe.rs` |
| `fetch` | 20 / 10 files | `cli/src/api.rs:1569` (`handle_fetch_artifact`) — the only `fetch_` in *compiled* code; the uncompiled `cli/src/api_axum.rs:746` declares the same route again (**D-4**, F-32); also `iOS/.../NotificationBackgroundProcessor.swift` |
| `query` | 136 / 26 files | `cli/src/api.rs` |
| `retrieve` | 8 / 5 files | genuinely thin and spread; densest is `cli/src/server.rs` (5 hits) |

> **Count basis.** Previously published as `get` 5470, `read` 1961, `list` 1689,
> `load` 731, `resolve` 579, `find` 352, `poll` 164, `fetch` 121, `query` 294,
> `retrieve` 22. Now 663, 493, 363, 203, 95, 133, 25, 20, 136, 8. The basis change
> accounts for it. **The conclusion this entry argues is unchanged**: `get` still
> outnumbers every other read verb by at least 2x, and `retrieve` is still thin enough
> at 8 occurrences that retiring it is a single afternoon. What the old numbers implied
> — that this is a 5,000-call-site problem — was never true of the compiled code.

**Decision — what this entry proposes**

- **Canonical:** a five-verb house rule — `get_*` returns a value already held in memory (pure,
  cheap); `list_*` / `query_*` returns a collection, possibly filtered or paginated; `fetch_*`
  crosses a process or network boundary to retrieve bytes; `load_*` reads persisted or cold state
  into memory; `poll_*` asks a running operation for progress without waiting.
- **Why:** the inventory has ten verbs and no rule distinguishing them. A written rule costs
  nothing, is enforceable, and touches no wire format.
- **Rename plan:** adopt the rule prospectively and **retire `retrieve`** (8 occurrences on
  this basis, no distinct meaning; published as 22 before the restatement). Backward renaming is optional and should be done only where a name is actively
  misleading. Two known violations to fix under the rule: `wasm/src/lib.rs:1184`
  `get_history_manager` returns a *manager*; `handle_fetch_artifact` is already correctly named.
- **Risk:** LOW.
- **Effort:** S for the rule; M for the mechanical rename.
- **Status:** `PROPOSED` for the rule and the retirement.

---

### B-2. Delete verbs

> **Concept:** an operation that makes a stored thing stop existing, or makes a collection empty.

**Inventory — what exists today**

| Verb in the tree | Count | Representative sites | Note |
|---|---|---|---|
| `remove` | 423 / 98 files | `core/src/store/contacts.rs:733`, `core/src/contacts_bridge.rs:181`, `android/.../MeshRepository.kt:4855` | |
| `clear` | 203 / 71 files | `android/.../MeshRepository.kt`, `cli/src/main.rs` | |
| `delete` | 47 / 22 files | `android/.../ContactsScreen.kt:251` (`onDelete`), `iOS/.../ContactsViewModel.swift:238` | UI-layer only |
| `drop` | 137 / 48 files | — | mostly `Iterator::drop` / lifetime mechanics, not a domain verb |
| `purge` | 11 / 9 files | `core/src/store/relay_custody.rs:62` (`purged_records`) | the one genuine domain use |
| `forget` | 25 / 9 files | `wasm/src/transport.rs` (12 hits), `android/.../MeshRepository.kt` | 11 of the 61 raw hits are "fire-and-forget", a different meaning, excluded |
| `destroy` | 19 / 12 files | — | |
| `expire` | 12 / 9 files | — | |
| `unlink` | 0 / 0 files | — | |

> **Count basis — one figure was wrong, not merely on another basis.** `unlink` was
> published as 21 occurrences in 7 files. There are **none**: it does not appear in
> first-party code at all, and the 21 came from prose outside the corpus. The other
> figures move as the rest do: `remove` 1074 → 423, `clear` 669 → 203, `delete` 339 →
> 47, `drop` 260 → 137, `purge` 57 → 11, `forget` 50 → 25, `destroy` 31 → 19,
> `expire` 24 → 12. **`remove` still dominates, so the conclusion of this entry stands**, but
> `delete` at 47 is far smaller than 339 suggested — it is mostly a UI-layer word, and
> this is code-only counting, so treat the entry as a `remove` problem.

**Decision — what this entry proposes**

- **Canonical:** `remove_*` for the domain verb; `forget_*` reserved for ledger-evidence semantics
  (dropping a peer without deleting local evidence — a genuinely different operation, already used
  consistently); `purge_*` reserved for retention/GC.
- **Why:** the domain layer is already consistent — `ContactManager::remove` in both copies. The
  drift is UI-only: Android Compose passes `onDelete = { viewModel.removeContact(...) }`
  (`ContactsScreen.kt:251`) and iOS has both `removeContact` (`ContactsViewModel.swift:200`) and
  `deleteContacts(at:)` (`:238`).
- **Rename plan:** retire `delete_` outside UI callbacks. **No domain-layer rename.** `delete_*`
  appears in persisted audit event names (`AuditEventType::ContactRemoved`,
  `core/src/observability.rs:38`) — treat those as FROZEN, see **§F**.
- **Risk:** LOW-MEDIUM, entirely from the persisted audit event names.
- **Effort:** S — the only divergence is UI callbacks, and it is mechanical.
- **Status:** `PROPOSED` for retiring `delete_` outside the UI; the persisted audit names are
  `FROZEN`.

---

### B-3. Create verbs

> **Concept:** an operation that brings a new thing into existence, or admits an existing thing into
> a set that did not previously hold it.

**Inventory — what exists today**

| Verb in the tree | Count | Representative sites | Note |
|---|---|---|---|
| `new` | 3657 / 161 files | `core/src/iron_core.rs:167`, `core/src/mobile_bridge.rs:3615` | constructors |
| `create` | 146 / 46 files | `core/src/mobile_bridge.rs` | |
| `add` | 361 / 62 files | `core/src/store/contacts.rs:607`, `android/.../AddContactScreen.kt` | |
| `insert` | 512 / 69 files | `core/src/transport/swarm.rs`, `cli/src/server.rs` | storage layer |
| `make` | 34 / 23 files | `core/src/routing/global.rs` | |
| `register` | 54 / 17 files | `core/src/store/blocked.rs`, `core/src/transport/dial_policy.rs` | |
| `save` | 54 / 15 files | `core/src/store/ledger_entry.rs:845` | |
| `put` | 178 / 24 files | `android/.../MeshRepository.kt` | |
| `upsert` | 15 / 5 files | — | |

> **Count basis.** `new` was 5168 and is now 3657 — the least-changed figure here, and
> it is unchanged in rank. `create` was 1095 → 146, `add` 1612 → 361, `insert` 651 →
> 512, `make` 1087 → 34, `register` 492 → 54, `save` 300 → 54, `put` 287 → 178,
> `upsert` 55 → 15. The basis change accounts for it. **The conclusion is unchanged and
> in one place stronger**: `insert` at 512 is now within reach of `add` at 361, so the
> storage-layer/UI-layer split this entry describes is the live one.

**Decision — what this entry proposes**

- **Canonical:** `create_*` for domain objects, `add_*` for collection membership, `register_*` for
  identity onboarding.
- **Why:** the code already reads this way — `add` is correctly used for collection membership
  (`ContactManager::add`) and `register` for registry joins. The rule makes it explicit rather than
  incidental.
- **Rename plan:** no rename of `add_` or `register_` — both are already correct. One genuine
  ambiguity remains: `store` is a **noun** throughout (`core/src/store/`, `StorageBackend`, 2163
  occurrences) while `save_*` is a verb. Keep them apart; do not merge.
- **Risk:** LOW.
- **Effort:** S — mostly codifying existing practice.
- **Status:** `PROPOSED` for the rule; `ACCEPTED-AS-IS` for `add_` and `register_` specifically.

---

## C. Cross-layer fields

### C-1. `last_seen` — the repository's worst field name

> **Concept:** when we last had evidence this peer was reachable.
>
> **Ownership:** this entry owns the canonical names, the rename plan, the risk and the
> compatibility approach for this field. The evidence — which types declare it, in what unit, and
> where — is owned by **FINDINGS F-01** and is not repeated here. The wire-key compatibility row is
> owned by **§F** and is not repeated here either.

**Inventory — what exists today**

| Name in the tree | Where it is declared |
|---|---|
| wire key `last_seen` | `cli/src/api.rs`, `core/src/wasm_support/rpc.rs` |
| `last_seen` (seconds) | `core/src/store/contacts.rs:31` |
| `last_seen` (milliseconds) | `core/src/store/ledger_entry.rs:167` |
| `last_seen_ts` | `core/src/notification.rs:254` |
| `lastSeen` | `android/.../MeshRepository.kt:810` |
| `lastSeenMs` | `android/.../MeshRepository.kt:847` |
| `lastSeenRaw` | `android/.../MeshRepository.kt:6749` |

**Decision — what this entry proposes**

- **Canonical:** every binding field carries its unit. `lastSeenMs` is the only name in the
  inventory that already does, and is the model to follow.
- **Why:** the same name holds two units (seconds and milliseconds) across the store layer, so the
  unit has to move into the name. The wire key cannot.
- **Rename plan:** `last_seen` → `last_seen_secs` (contacts) and → `last_seen_ms` (ledger);
  `last_seen_ts` → `last_seen_ms`; `lastSeen` → `lastSeenSecs`; `lastSeenRaw` → **delete**. The
  wire key `last_seen` is **not** renamed — it is a serialized format, see **§F**.
- **Risk:** MEDIUM — the ambiguity *is* the bug.
- **Effort:** M.
- **Status:** `FROZEN` for the wire key; `PROPOSED` for the three binding renames;
  `ACCEPTED-AS-IS` for `lastSeenMs`; `BLOCKED` on **Q-1** for `lastSeenRaw`, which may be a live bug.
  **Evidence and design fix:** FINDINGS F-01.

---

### C-2. `TransportType` — seven definitions, no canonical form

> **Concept:** how bytes move between two nodes.

**Inventory — what exists today**

| Name in the tree | Count | Where it is defined |
|---|---|---|
| `TransportType` | 7 definitions, 710 occurrences | the seven-definition inventory is evidence and is owned by **FINDINGS F-08**, not repeated here — the definitions are `core/src/transport/abstraction.rs:11`, `core/src/routing/local.rs:19`, `core/src/relay/client.rs:24`, `android/.../MeshEventBus.kt:163`, `android/.../SmartTransportRouter.kt:27`, `iOS/.../Models.swift:70`, `iOS/.../MeshEventBus.swift:70` |

**Decision — what this entry proposes**

- **Canonical:** two names, both **to be created** — `TransportMechanism` (BLE, TCP, QUIC,
  WebSocket: *how bytes move*) and `MessagePath` (direct-LAN, via-mesh, local-loopback: *which
  route was taken*).
- **Why:** one name cannot cover both axes. The transport layer wants a mechanism; the routing
  layer and both mobile UIs want a path. Only `BLE` appears in all seven current definitions, which
  is the concrete measure of how little they have in common. A rename alone cannot fix this — the
  concept has to be split first.
- **Rename plan:** split the concept, *then* generate the per-language enums from one definition (a
  UDL enum plus a codegen step, or a checked-in generated file with a CI drift check). Retire
  `SmartTransportRouter.TransportType.CORE` or give it a real name — `CORE` is not a transport, it is
  "whatever the core node did".
- **Risk:** HIGH — the largest item in the audit. If the split reaches a UDL dictionary it is a
  breaking change for both mobile clients; compatibility approach and sequencing in **§F**
  (sequence after F-22 removes the duplicate DTOs).
- **Effort:** L.
- **Status:** `BLOCKED` on **Q-3** — do not rename before the split is decided. The two new names
  are `PROPOSED` and depend on that decision.

---

## D. Abbreviations

**There is no glossary, terminology, or abbreviation document anywhere in this repository.**
`ls docs/ | grep -i "glossar\|terminolog\|naming\|abbrev"` returns nothing. The "Defined in repo?"
column records where — if anywhere — each term is pinned down, and was established by searching
`docs/`, `AGENTS.md`, `README.md`, `CHANGELOG.md`, and Rust doc comments in `core/src`.

**Inventory — what exists today**

| Concept | Abbreviation in the tree | Count | Representative sites | Defined in repo? |
|---|---|---|---|---|
| configuration | `cfg` | **42 / 14 files** (see note) | `cli/src/main.rs:3600`, `cli/src/api.rs:1386` | **No** — the only `cfg` in any doc comment is Rust's `#[cfg]` attribute (`core/src/crypto/kani_proofs.rs:10`), a different thing |
| configuration | `config` | 998 / 55 files | `cli/src/main.rs`, `core/src/privacy/cover.rs` | self-defining |
| configuration | `settings` | 472 / 48 files | `android/.../SettingsScreen.kt`, `SettingsViewModel.kt` | self-defining |
| configuration | `prefs` | 74 / 5 files | `android/.../MeshRepository.kt`, `PreferencesRepository.kt` | self-defining |
| configuration | `params` | 98 / 6 files | `core/src/wasm_support/rpc.rs:17` | self-defining |
| configuration | `options` | 47 / 14 files | `wasm/src/notification_manager.rs` | self-defining |
| message | `msg` | 734 / 54 files | `core/src/store/outbox.rs:226` (`enqueue(&mut self, msg: QueuedMessage)`) | **No** — `grep -rn "/// .*msg"` over `core/src` returns zero |
| message | `text` | 618 / 70 files | `android/.../SettingsScreen.kt` | self-defining |
| message | `body` | 160 / 37 files | `cli/src/server.rs` | self-defining |
| network address | `addr` | 1019 / 39 files | `core/src/transport/swarm.rs:141` (`is_discoverable_multiaddr(addr: &Multiaddr)`) | **No** — ~210 of these are stdlib/libp2p type names (`SocketAddr`, `IpAddr`, `Ipv4Addr`, `Ipv6Addr`) which are not the project's to rename |
| network address | `address` | 824 / 60 files | `android/.../BleGattClient.kt` | self-defining |
| network address | `multiaddr` | 809 / 34 files | `core/src/transport/swarm.rs` | **Yes** — a libp2p term of art, defined in `docs/` |
| network address | `url`, `host`, `endpoint` | 121 / 212 / 86 — **`url` is 39 hits (32%) uncompiled** | `iOS/.../MeshRepository.swift`, `android/.../SubnetProbe.kt`, `core/src/notification.rs:79` | self-defining, and distinct terms |
| authentication | `session` | 301 / 35 files | `core/src/crypto/encrypt.rs`, `core/src/crypto/session_manager.rs` | self-defining |
| authentication | `token` | 162 / 7 files | `core/src/relay/invite.rs` | self-defining |
| authentication | `authorization`, `authentication` | 1 / 26 | `iOS/.../NotificationManager.swift`, `core/src/crypto/ratchet.rs` | self-defining |
| authentication | `auth` | 6 / 3 files | `core/src/store/contacts.rs`, `android/.../MeshRepository.kt` | **No** — the one near-hit is a different sense: "auth tag" at `core/src/crypto/backup.rs:221` is the AEAD authentication tag, a standard term |

> **Two corrections, both now superseded by the basis above.** `cfg` was first reported as
> "667 hits / 170 files" — ~95% wrong, because it matched Rust's `#[cfg(...)]` attribute and
> the `cfg!()` macro. A hand re-count with those excluded, and with comments excluded too,
> gave 31 in 10 files, then 27 in 9 once 4 were found to sit in `cli/src/api_axum.rs`, which
> the build never compiles. **The table now carries 42 / 14 files**, which is the script's
> figure on the stated basis. It is higher than the hand-count for one reason worth knowing:
> the script filters the *attribute and the macro* but does not strip comments, so a `cfg`
> inside a comment is counted. **For planning the rename, treat 42 as an upper bound and the
> earlier 27 as the comment-excluded figure**; the effort grade is S either way, and F-24
> carries the same caveat. The other two filtered terms are `forget` (11 of 61 raw hits are
> "fire-and-forget", B-2) and `body` (`hyper::body` / `axum::body` path segments).

**Decision — what this entry proposes**

- **Canonical:** `config` and `message`; `address` for the address cluster.
- **Why:** `cfg` and `msg` are undefined anywhere in the repo and cost nothing to expand;
> `config` and `message` are self-defining and already dominant. `multiaddr`, `host`, `endpoint`
> and `url` are distinct terms, not abbreviations of `address`, and are kept.
- **Rename plan:** `cfg` → `config`; `msg` → `message` — **in identifiers only**. Never in JSON
> keys, sled key prefixes, `#[serde(rename)]` targets or UDL field names; mixing those makes the diff
> unreviewable and the migration unreversible. Keep `settings` **only** for user-facing preferences,
> distinct from `config` wiring (Q-7). **Do not merge `text` and `body`** — they are different
> things (Q-8). Do not attempt to rename stdlib `SocketAddr`/`IpAddr`/`Ipv6Addr`.
- **Risk:** LOW for `cfg`; MEDIUM for `msg`, because `msg_id`/`message_id` appear as JSON keys.
- **Effort:** S for `cfg` → `config` (42 occurrences, 14 files — 27 in 9 once comments are
  excluded; see the note above); M for `msg` → `message` (734 occurrences, 54 files,
  scriptable).
- **Status:** `PROPOSED`. The authentication cluster is `ACCEPTED-AS-IS` — the counts already show a
  consistent preference and no rename is proposed.

---

## D-2. Project jargon — is it defined anywhere?

> Sweep method: take the distinctive project terms, count code occurrences, then search `docs/`,
> `AGENTS.md`, `README.md`, `CHANGELOG.md` and `core/src` doc comments for a definition. A term
> counts as **defined** only if something states what it *is*, not merely that it exists.

**Inventory — what exists today**

| Term in the tree | Code hits | Defined where |
|---|---:|---|
| `drift` | 101 | `docs/ARCHITECTURE.md`, `docs/ARCHITECTURE_MODULE_MAP.md`; module doc at `core/src/drift/mod.rs` |
| `custody` | 144 | `docs/ARCHITECTURE_MODULE_MAP.md`; `AGENTS.md` ("store-and-forward custody is a behavior all nodes perform"); `core/src/drift/relay.rs:123` |
| `beacon` | 151 | **nowhere.** Two mechanisms share the word: the identity BLE beacon (`android/.../MeshRepository.kt:3341,3440`) and the Apple Find My compatible encoding (`core/src/relay/findmy.rs:1-5`) |
| `UNIFICATION` (+ 5 variants) | 220 | **nowhere.** The only `docs/` hits are an unrelated *filename*, `ANDROID_ID_UNIFICATION_BUG_2026-03-14.md` (`docs/ARCHIVE_WORK_TRACKING.md:287,304,319`) |
| `mycorrhizal` / `mycelium` / `rhizomorph` | 8 / 0 / 0 | `core/src/routing/engine.rs:4`; `core/src/routing/mod.rs:2-6` defines the three-layer model |
| `triad` | 27 | `core/src/identity/keys.rs:87-95` — a full doc comment enumerating all three members and their relationships. Not in `docs/`. |
| `mule`, `hopscotch`, `dialplan`, `salting` | 0 code hits | n/a — archived `docs/` planning material only, not code vocabulary |

> **Count basis.** Previously published as `drift` 547, `custody` 468, `beacon` 283,
> `UNIFICATION` 279, `mycorrhizal`/`mycelium`/`rhizomorph` 21/3/3, `triad` 37. Now 101, 144,
> 151, 220, 8/**0**/**0**, 27. Two of those are worth more than the basis change. **`mycelium`
> and `rhizomorph` have no code occurrences at all** — the 3 each were counting prose in
> `docs/historical/SOVEREIGN_MESH_PLAN.md`, which this column ("Code hits") excludes by its
> own header. So the three-layer model this row describes is a *documented* concept with a
> single code reference (`mycorrhizal`, 8 hits in 5 files), not three live terms. **That is
> a stronger version of what the entry argues** — the model is thinner in code than the old
> numbers implied.

**Decision — what this entry proposes**

- **Canonical:** no new names proposed. The decision here is which terms need a written definition
  and which are already fine.
- **Why:** an undefined term that appears 279 times in production comments is a documentation
  defect, not a naming one. The two that fail are `UNIFICATION` and `beacon`.
- **Rename plan:** none for `drift`, `custody`, `mycorrhizal`/`mycelium`/`rhizomorph` (all defined;
  `rhizomorph` is defined by contrast in the `mod.rs` layer list and should be said once in prose).
  None for `triad` — `PeerIdTriad` is well-named and well-documented. For the two that fail, see
  §D-3.
- **Risk:** LOW — documentation only.
- **Effort:** S for each definition paragraph.
- **Status:** `ACCEPTED-AS-IS` (deliberately — no rename needed) for `drift`, `custody`,
  `mycorrhizal`/`mycelium`/`rhizomorph`, `triad`. `BLOCKED` on **Q-10** for `UNIFICATION` and
  **Q-11** for `beacon`, both of which need a decision the owner must make.

---

### D-3. Two entries that need a definition written

**D-3a. `UNIFICATION` — a codename with six spellings and no definition (highest priority)**

Inventory — the six spellings in the tree and their counts:

| Spelling | Count |
|---|---:|
| `UNIFICATION` | 220 |
| `UNIFICATION_V2` | 21 |
| `UNIFICATION_V2_IDENTITY` | 8 |
| `UNIFICATION_V3` | 7 |
| `UNIFICATION_V2_TRANSPORT` | 6 |
| `UNIFICATION_DIAL` | 2 |

It functions as an informal version-history scheme embedded in production code:

```rust
// core/src/store/contacts.rs:660
// UNIFICATION_V2_IDENTITY: Use single source of truth for identity_id derivation.
// core/src/store/ledger_entry.rs:388
// UNIFICATION_V2_TRANSPORT: nature-inspired eviction — only when at capacity.
```

A reader who does not know the codename cannot tell what `UNIFICATION_V2_IDENTITY` is meant to
distinguish from `UNIFICATION_V3`, or from a bare `UNIFICATION`. Worse, the codename is the *only*
record of the design decision behind F-09's hex-vs-multihash split — the reason a field is
sometimes one and sometimes the other is documented solely as "UNIFICATION".

**Decision — what this entry proposes**

- **Canonical:** the term needs a *definition*, not a rename. Write one paragraph in
  `docs/ARCHITECTURE.md` stating what unification was and which decisions it produced (the canonical
  hex peer id being the main one, per `core/src/store/ledger_entry.rs:486-489`), then decide
  whether the comment prefixes stay.
- **Why:** if the codename is retired, the six spellings need replacing with descriptions of *what
  changed* — `UNIFICATION_V2_IDENTITY` is already nearly self-describing (`identity_id` derivation)
  and `UNIFICATION_V2_TRANSPORT` likewise.
- **Rename plan:** retire-and-describe, or keep-and-define. Retire-and-describe touches ~60 comment
  lines and changes no code.
- **Risk:** LOW — comments only.
- **Effort:** S either way.
- **Status:** `BLOCKED` on **Q-10**.

**D-3b. `beacon` — one word, two mechanisms**

283 code hits split across an identity BLE beacon (a GATT/JSON payload the node publishes so peers
can learn it, `android/.../MeshRepository.kt:3341,3440`) and an Apple Find My compatible beacon
encoding (`core/src/relay/findmy.rs:1-5`, a separate BLE advertisement scheme with its own XOR-based
encryption). Nothing in the repo says these are different things sharing a word.

**Decision — what this entry proposes**

- **Canonical:** reserve `beacon` for the identity broadcast; **to be created** —
  `findMyAdvertisement` for the other.
- **Why:** they are separate mechanisms with separate encodings; sharing a word invites the reader
  to assume shared semantics.
- **Rename plan:** rename the Find My one to `findMyAdvertisement`, or to whatever vocabulary
  `relay/findmy.rs` adopts. If the module is a temporary experiment, deleting it is better than
  naming it.
- **Risk:** LOW — one module.
- **Effort:** S.
- **Status:** `BLOCKED` on **Q-11**.

---

## D-4. Counts that include code the build never compiles

**Every occurrence count in this glossary is a text count, and a text count cannot tell a
compiled file from one no crate root reaches.** Seven Rust files are in that second
category; the verified inventory is FINDINGS **RC-2**, which covers all seven across F-19,
F-32 and F-33. A name that appears only there is not vocabulary the project uses — it is
vocabulary a file nobody builds proposes, and renaming production code to match it would
be the wrong move.

**This section owns one column: how much of a term sits in those seven files.** It does
not restate the counts the entries publish — the entry owns its own figure, and this
section cites it. The tables below are generated:

    python scripts/measure_uncompiled_counts.py

**Basis: the same one the header points to** — `scripts/measure_uncompiled_counts.py`, the
`BASIS` block. Not restated, on purpose. The tables below come out of the command above
running over the same term list the entries use, so this section cannot disagree with them
about what was measured.

**The Total column here and the figure the owning entry publishes are the same
measurement**, taken by the same command on the same basis. This section adds the one
dimension an entry does not carry: how much of the term sits in the seven files no crate
root reaches. The entries used to carry figures from a different, unrecorded basis; those
were restated onto this one, and each entry's *Count basis* note records what its figure
was before. An earlier version of this section still said those figures "could not be
recovered" and told the reader to compare like with like only inside these tables. That
was true when written and stopped being true when the restatement landed; the sentence
survived it, which is the failure this section is supposed to prevent.

### Counts materially inflated by uncompiled code

Only terms at or above 5% are listed; everything below that cannot change a decision and is
covered by the note underneath.

| Term | Entry | Total | In the 7 uncompiled files | Share |
|---|---|---:|---|---:|
| `StoredMessage` | A-3 | 19 | 19 (2 files) | **100%** |
| `WasmMeshNode` | F-33 | 35 | 35 (2 files) | **100%** |
| `node_id` | A-1 | 19 | 12 (1 file) | 63% |
| `url` | §D | 121 | 39 (1 file) | 32% |
| `triad` | D-2 | 27 | 7 (1 file) | 26% |
| `node` | A-1 | 567 | 125 (2 files) | 22% |
| `register` | B-3 | 54 | 9 (1 file) | 17% |
| `config` | §D | 998 | 133 (7 files) | 13% |
| `drop` | B-2 | 137 | 18 (5 files) | 13% |
| `fetch` | B-1 | 20 | 2 (2 files) | 10% |
| `cfg` | §D | 42 | 4 (1 file) | 10% |
| `relays` | A-1 | 176 | 14 (4 files) | 8% |
| `insert` | B-3 | 512 | 40 (6 files) | 8% |
| `msg` | §D | 734 | 54 (2 files) | 7% |
| `read` | B-1 | 493 | 30 (5 files) | 6% |

### Terms with no uncompiled occurrences at all

`DriftEnvelope`, `Envelope`, `HistoryStats`, `MessageRecord`, `RelayCustodyStore`,
`TransportType`, `UNIFICATION`, `auth`, `authentication`, `authorization`, `beacon`,
`blePeerId`, `blob`, `canonicalPeerId`, `contactId`, `create`, `custody`, `delete`,
`destroy`, `endpoint`, `envelopeData`, `expire`, `forget`, `frame`, `host`,
`libp2pPeerId`, `multiaddr`, `mycorrhizal`, `nodeId`, `options`, `packet`, `params`,
`peerId`, `peerIds`, `peer_ids`, `prefs`, `purge`, `put`, `query`, `relayId`,
`relay_custody`, `relay_id`, `retrieve`, `routePeerId`, `save`, `session`, `settings`,
`token`, `upsert` — 49 of the 99 terms measured. Every occurrence is in compiled
code, so these entries need no qualification.

### Terms below the line, and why the line is there

27 terms sit between 0% and 5%, the highest being `new` at 4.6%. A share that small cannot change a
decision about a name: it means a handful of occurrences in files nobody builds, not a
competing vocabulary. The 5% line is a reading aid, not a finding, and it is stated here so
that a term's absence from the table above reads as a measurement rather than an omission.

**One entry records no occurrence count at all, and none is supplied here.** C-1 is a
spellings table: it records where each spelling of `last_seen` is declared and carries no
count to qualify. A-5 used to be the second such entry — `routePeerId`, `blePeerId` and
`libp2pPeerId` were shown as "—", a location rather than a frequency — but the measurement
script can measure them, so A-5 now carries real figures (309, 184, 368) and none of the
three is inflated by uncompiled code.

**`cfg` is filtered, and it qualifies on the filtered count.** It is stripped three ways —
`#[cfg(...)]` attribute, `cfg!()` macro, and nothing else — because a raw whole-word count
returns 572 where the real identifier count is far smaller. On that treatment the script
gives **42 in 14 files, of which 4 are in `cli/src/api_axum.rs`**, a file the build never
compiles, which is the 10% row above. It appeared in neither an earlier version of this
table nor the sentence that used to sit here, which said it "does not appear in the
tables" — both were wrong in the same direction, and the table was the thing to fix. The
entry and F-24 carry the finer comment-excluded figure; this section does not restate it.

### What a reader should act on

1. **`StoredMessage` does not exist in the build.** All 19 occurrences are in unreachable
   files; the type is defined once, at `wasm/src/storage.rs:47`. F-02 offers `StoredMessage`
   as an alternative name for the storage-layer record, and A-3 — which owns that decision —
   chose `MessageRecord` / `MessageRecordDto` instead. **Adopting F-02's branch without
   checking would bind a production name to a phantom type.** See FINDINGS F-33.
2. **`url` and `node` are the counts most distorted**, at 32% and 22%. If either entry's
   recommendation is acted on, re-read it against the compiled files first. `WasmMeshNode`
   joins them at 100%.
3. **Everything else stands.** The peer-id spellings, the address and authentication
   clusters, the create and delete verb clusters and `last_seen` are unaffected — their
   evidence is in code the build reads.

---

## E. Names that already carry a policy claim and should be demoted

| Name in the tree | Problem | Decision |
|---|---|---|
| `SwarmEvent2` | suffix implies a v1 that does not exist | `PROPOSED` → `SwarmEvent` (definition site only). Evidence: FINDINGS F-05 |
| `RegistrationStateInfo` | "State Info" containing a field called `state`; also `String`-typed | `PROPOSED` → `RegistrationState`. Evidence: FINDINGS F-28 |
| `canonicalPeerId` | "canonical" is a policy claim, not a kind | see A-5, `BLOCKED` on Q-3 |
| `ContactManagerFix`, `ContactManagerFixed` | names a workaround, not a concept | `PROPOSED` — delete if the UniFFI issue is resolved. Evidence: FINDINGS F-14 |
| `handle_*` in `cli/src/api.rs` | actually a consistent axum convention | `ACCEPTED-AS-IS` — **not** a defect |
| `Output` / `Input` in `core/src/dspy/modules.rs` ×4 | generic, but disambiguated per module | `ACCEPTED-AS-IS` |

---

## F. FROZEN — do not rename, here is what to do instead

> **This table is the single owner of every compatibility approach in the report.** No other
> section restates one; concept entries in §A–§E point here rather than repeating the approach.

These are identifiers that cross a boundary. A rename here is a **migration**, not a refactor.

| Identifier | Where it is frozen | Compatibility approach |
|---|---|---|
| `last_seen` (JSON key) | CLI HTTP API, JSON-RPC daemon bridge, ledger records | **Alias.** Keep the key. Add a new `last_seen_ms` alongside for one release, have readers prefer it, remove the old in the next major. Bindings (`lastSeen`, `lastSeenMs`) are **source-only renames, no compat work** — see C-1. |
| `TOPIC_LOBBY` / `TOPIC_MESH` / `TOPIC_RECEIPT_CONVERGENCE` | `core/src/lib.rs:282-284`, values `sc-lobby` / `sc-mesh` / `sc-receipt-convergence` | **Frozen absolutely.** The source comment is right: *"mismatched values silently partition the mesh."* Never rename. |
| `/api/*` HTTP routes | `cli/src/api.rs:1641-1666` | **Alias.** Where a path moves (F-11), serve both paths for one release and log the old one's use to measure it. The three un-prefixed routes (`/submit-run`, `/poll-status/:run_id`, `/fetch-artifact/...`) have the widest unknown client set — check access logs first. |
| JSON-RPC method names | `core/src/wasm_support/rpc.rs`, consumed by `wasm/src/daemon_bridge.rs` and `cli/src/server.rs` | **Alias.** The daemon bridge is versioned by protocol; add the new name, keep the old as a deprecated alias that emits a warning. |
| `MessageRecord` UDL dictionary | `core/src/api.udl:324` | **Deprecation period.** If `delivered: bool` is removed in favour of `status`, keep the bool populated for one release as a derived value; the Android/iOS bindings read it. |
| `DeliveryStatus` / `MessageStatus` enums | `core/src/api.udl:22,317`, receipt codec | **Deprecation period.** They are serialized in receipts. Do not rename the wire variants; add the missing semantics in the doc comments, and add a `ReceiptStatus` alias rather than moving variants. |
| sled key prefixes (`CONTACT_KEY_PREFIX`, `CONTACT_BUNDLE_KEY_PREFIX`) | `core/src/store/contacts.rs:81-87` | **Migration, not a rename.** If F-03 unifies the key scheme, existing rows must be read under both schemes and rewritten on read. This is a data migration and needs its own plan. |
| `Contact.peer_id` placeholder format (`"peer-…"`) | `core/src/store/contacts.rs:104`, `message/identity_envelope.rs:176` | **Migration.** If F-09 introduces `PeerKey::Placeholder`, keep the string detection on the read path for existing rows. |
| `MeshServiceConfig` / discovery UDL dictionaries | `core/src/api.udl` | **Deprecation period.** Only affected if the `TransportType` split (C-2) reaches the UDL. Sequence after F-22 removes the duplicate DTOs. |
| `AuditEventType::ContactRemoved` | `core/src/observability.rs:38`, persisted | **Frozen.** Do not rename a persisted audit event; see B-2. |

---

## G. Open items this glossary cannot decide

These are recorded in full in `SUMMARY.md` §3. In brief:

- **Q-3** — is the 64-char hex peer id a permanent canonical form or a transitional one? The
  answer determines whether A-5 recommends a `PeerKey` enum (permanent) or a dual-read adapter
  (transitional), and whether C-2's split can proceed.
- **Q-7** — are `config` and `settings` two different concepts, or two names for one? The counts
  suggest they are genuinely different, but that was not verified from the code.
- **Q-8** — do `text` and `body` name the same thing in this codebase? Merging them is the single
  largest count reduction available in §D and I am not confident enough to recommend it.
- **Q-10** — is the `UNIFICATION` codename still meaningful, or should the comment prefixes be
  retired and replaced with descriptions? See D-3a.
- **Q-11** — are the two things called `beacon` meant to share a word? See D-3b.
