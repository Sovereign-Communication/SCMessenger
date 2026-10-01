# Naming Audit — Findings

**Repository:** SCMessenger (`/c/Users/SCM/Documents/GitHub/SCMessenger`)
**Branch at audit time:** `glm/canonical-outlier-audit` (working tree has uncommitted changes from other sessions; this audit is read-only and touched nothing)
**Date:** 2026-09-29
**Method:** static analysis only. No build, no test run, no code change. See `SUMMARY.md` §Method.

Findings are ranked by **severity**, then by **blast radius** (occurrence count / dependents).

Severity key:
- **HIGH** — public API, wire format, or core domain type; a wrong read of the name produces a wrong program.
- **MEDIUM** — internal but widely used; drift is likely and already observable.
- **LOW** — local, cosmetic, or in a layer not on the hot path.

---

## Contents — findings grouped by root cause

The findings are ordered by the **cause** that produces them, not by the brief's
Layer 1 / Layer 2 taxonomy. A maintainer deciding what to fix first wants to know
which findings a single change clears, and that is not a question the taxonomy can
answer. **Finding IDs are stable identifiers, not ranks** — F-28 is not "28th", and
the order below is the ranking.

**The ranking rule, and what grouping does to it.** Severity is the primary sort.
**Within a section, findings run HIGH, then MEDIUM, then LOW, with no exception.**
**Sections are ordered by their highest-ranked finding**, so a section whose best
finding is MEDIUM — RC-5 — comes after every section that contains a HIGH. One
consequence is visible in the table: a section ends on a MEDIUM and the next begins on
a HIGH. That is not a sorting error, it is the boundary between two root causes, and a
single flat list sorted globally by severity would interleave five unrelated groups in
one column. The table below is the index that tells you where each boundary falls.

Where a finding's blast radius names a surface rather than a count ("the entire
transport event surface", "the UDL public API"), it is read as larger than any bare
occurrence count. Five sections tie at HIGH, and their blast-radius figures are in
incommensurable units — occurrences of a token, uncompiled LOC, "the CLI's
user-facing surface" — so those five are ordered by **fix leverage**: how many
findings one change clears. RC-1 needs one change for four findings and RC-2 one
deletion for three; RC-3 is one change for two; RC-4's five findings need five
separate decisions, so it follows them.

| Section | Root cause | Findings, in rank order |
|---|---|---|
| **RC-1** | The FFI/bridge layer restates core types instead of deriving them | F-02, F-03, F-22, F-16 |
| **RC-2** | Files that are not compiled keep every duplicate-name count inflated | F-19, F-20, F-21, F-32, F-33 |
| **RC-3** | A surface has a second, already-diverged copy of itself | F-06, F-15 |
| **RC-4** | A type name has no owning layer, so each layer defines its own | F-07, F-08, F-04, F-18, F-29 |
| **REMAINDER** | No shared cause — the defect is local to the name | F-09, F-05, F-01 (+ F-25 retired), F-17, F-14 (+ F-27), F-12, F-13, F-10, F-28, F-31 |
| **RC-5** | A concept has several names and no recorded canonical choice, so each new call site picks one | F-26, F-23, F-30, F-11, F-24 |

**How the two tests were applied.** Layer 1 asked: *can one sentence say what this is
and what it holds/does, without using "or" and without "sometimes"?* A failure of that
test is a type or field whose shape is a union of meanings. Layer 2 asked: *does this
concept have one name?* A failure is a cluster of names with no canonical choice. Both
kinds appear below; each section header says which one its members show. The fix for a
Layer 1 finding is a design change, the fix for a Layer 2 finding is a decision, and the
decisions are owned by `GLOSSARY.md`.

---

## Part 0 — The shape of the problem (read this first)

Three of the top ten findings are not "bad names". They are **one name with N
implementations**, and the implementations have already diverged. In this repository that is the
dominant naming failure mode, and it is structurally different from both classic layers:

- It is a Layer 1 problem (the concept is not one thing) *and* a Layer 2 problem (the vocabulary
  is not canonical) *simultaneously*, which is why neither a rename nor a convention fixes it.
- It is self-concealing: the duplicate types compile, the duplicate tests pass, and the duplicate
  CLI grammar is parsed by a test that reads as if it tests the shipped binary.

54 type names are defined in more than one first-party Rust file. The ones with real
blast radius are grouped under the five root causes below; see **Contents** for which
cause owns which finding. The duplicates that no finding covers are listed, once, at the
end of the **REMAINDER** section.

---

## RC-1 — The FFI/bridge layer restates core types instead of deriving them

**Members, in rank order:** F-02, F-03, F-22, F-16

**The cause, stated once here.** `mobile_bridge.rs` and the `*_bridge` modules hand-copy
the storage layer's types instead of generating them from it, and the copies have
already diverged. Nothing in the build fails when they drift: both copies compile, both
are exported, and the FFI surface is served by whichever copy the caller reached. The
evidence is spread across the four findings below; each of them states what is wrong
with its own types, not the cause.

**This is the highest-leverage group in the audit: one structural change clears all four
findings**, because they are the same defect in four type families. No finding's
remediation has been rewritten for this grouping. Where a fix depends on a vocabulary
decision, that decision is owned by `GLOSSARY.md` and the finding points at it.

---

## F-02 — `MessageRecord` exists twice with different fields, and one of them contradicts itself

**Severity: HIGH** · **Blast radius: 125 occurrences, 3 declarations (2 Rust + 1 UDL), FFI-bound**

**Locations:**
- `core/src/store/history.rs:17` — storage-layer record
- `core/src/mobile_bridge.rs:3126` — FFI-layer record
- `core/src/api.udl:324` — the UDL dictionary both feed

**Evidence — the two Rust structs side by side:**

```rust
// core/src/store/history.rs:17
pub struct MessageRecord {
    pub id: String,
    pub direction: MessageDirection,
    pub peer_id: String,
    pub content: String,
    pub timestamp: u64,
    pub sender_timestamp: u64,
    pub delivered: bool,
    pub hidden: bool,                 // <-- no `status` field
}
```

```rust
// core/src/mobile_bridge.rs:3126
pub struct MessageRecord {
    ...
    pub delivered: bool,
    #[serde(default)]
    pub status: MessageStatus,        // <-- Queued | InCustody | Sent | Delivered
    #[serde(default)]
    pub hidden: bool,
}
```

Three distinct problems inside one name:

1. **Same name, different shape.** The store cannot represent `InCustody`; the bridge can. The
   storage record has no way to express the state the UI is told to render.
2. **Redundant, contradictory siblings.** `MessageRecord` carries `delivered: bool` *and*
   `status: MessageStatus`, where `status == Delivered` already implies `delivered == true`.
   Nothing in either struct keeps them in agreement.
3. **`#[serde(default)]` hides the disagreement.** Every legacy row deserialises `status` as
   `Queued` regardless of its `delivered` flag. The name `MessageRecord` implies "the message as
   it is"; the field set means "the message, plus a UI progress projection that may be fiction."

**And the vocabulary around it forks again** — three enums for one question ("what is the state of
this message?"):

- `DeliveryStatus { Sent, Delivered, Read, Failed }` — `core/src/api.udl:22`, receipt codec
- `MessageStatus { Queued, InCustody, Sent, Delivered }` — `core/src/api.udl:317`, "Canonical message
  status for UI rendering"
- `MessageDirection { Sent, Received }` — `core/src/api.udl:310`

`DeliveryStatus` and `MessageStatus` both carry `Sent` and `Delivered` with no stated difference.
`MessageDirection` carries a *third* `Sent`.

**Remediation (design, then vocabulary):**
1. Pick one state enum. `MessageStatus` is the better carrier (it subsumes the UI's needs) — see
   GLOSSARY. Delete `DeliveryStatus` or rename it `ReceiptStatus` to name the *artefact* it
   describes rather than the message.
2. Delete `delivered: bool`; derive it (`status == Delivered`) at the binding boundary only. This is
   a wire-format change on `MessageRecord` — sequence the compatibility note in GLOSSARY.
3. Merge the two Rust structs. The bridge type should be a DTO generated from the store type, not a
   hand-maintained copy. If UniFFI cannot express that, rename them honestly
   (`StoredMessage` / `MessageDto`) so the duplication is visible in review.

---

## F-03 — `ContactManager` exists twice, writes to the same sled tree under two different key schemes

**Severity: HIGH** · **Blast radius: 78 occurrences, 2 Rust definitions, 1 iOS typealias, FFI-bound**

**Locations:**
- `core/src/store/contacts.rs:131` — domain manager, key scheme A
- `core/src/contacts_bridge.rs:91` — FFI manager, key scheme B
- `core/src/lib.rs:110` re-exports the **bridge** one as `scmessenger_core::ContactManager`
- `iOS/SCMessenger/SCMessenger/ContactManagerFix.swift:188` — `typealias ContactManager = ContactManagerFixed`

**Evidence — two key derivations for one entity:**

```rust
// core/src/store/contacts.rs:81
fn contact_key(peer_id: &str) -> Vec<u8> {
    [CONTACT_KEY_PREFIX, peer_id.as_bytes()].concat()   // namespaced
}
```

```rust
// core/src/contacts_bridge.rs:144
let key = contact.peer_id.as_bytes();                  // raw, un-namespaced
db.insert(key, value)
```

The two managers have identical method names — `add`, `get`, `remove`, `list`, `search`,
`set_nickname`, `set_local_nickname`, `update_last_seen`, `reconcile_from_history`, `count`,
`flush`, `verify_integrity` — over *different* key spaces in the *same* tree. A record written by
one is invisible to the other.

The same duplication exists on `Contact` itself (`core/src/store/contacts.rs:25` vs
`core/src/contacts_bridge.rs:19`): same name, same first eight fields, the bridge copy adding
`verified_at` and `is_tombstone`.

**Two names inside the FFI copy that lie:**

- `ContactManager::remove` (`contacts_bridge.rs:181`) does a bare `db.remove(peer_id.as_bytes())`.
  The store copy of the same method (`store/contacts.rs:733`) additionally cascades to
  `contact_bundle_key(&contact.public_key)`. Same verb, same name, different effects — the FFI
  path leaves orphaned key bundles.
- `Contact::tombstone(peer_id)` (`contacts_bridge.rs:51`) constructs a record with
  `is_tombstone: true`. But `ContactManager::remove` never writes one — it deletes. **The
  `is_tombstone` field and the `tombstone()` constructor name a retention behaviour that the only
  caller-facing removal path does not implement.** I could not determine whether tombstoning is
  reachable through some other path; see open question Q-2.

**Remediation (design):** one `ContactManager`, one key derivation, one `Contact`. The bridge
becomes a thin FFI shim that calls `store::contacts` (which `store/contacts.rs:15-20` already says
is the intent: *"This module is compiled for every target, so the recovery path in `contacts_bridge`
imports it rather than keeping a second copy of the string that could drift"* — the string was
unified; the type was not). Delete `Contact::tombstone` and `is_tombstone` unless a caller is found.

---

## F-22 — `HistoryManager` / `HistoryStats` / `MessageDirection` are each defined twice

**Severity: MEDIUM** · **Blast radius: `HistoryManager` 111, `HistoryStats` 45, `MessageDirection` 131**

**Basis for that line:** `--basis wide`, the wider corpus defined in
`scripts/measure_uncompiled_counts.py`; this table quotes names that live mostly in tests
and generated bindings, which the glossary's corpus excludes. Regenerate with
`python scripts/measure_uncompiled_counts.py --basis wide`. All three reproduce exactly. An
earlier version of this note recorded `HistoryManager` and `MessageDirection` as off by one
and blamed drift; that was a property of the tree the audit was written in, not of the
measurement, and both are exact at this commit.

| Concept | `store/history.rs` | `mobile_bridge.rs` |
|---|---|---|
| manager | `:89` | `:3185` |
| stats | `:81` | `:3151` |
| direction | `:11` | `:3111` |

`MessageDirection` is byte-identical in both. `HistoryManager` and `HistoryStats` are near-identical.
This is the same shadow-copy shape as F-03 and F-02, and it means the FFI layer owns a hand-kept
duplicate of the storage layer's types. The cause is uniform across F-02, F-03, F-16 and F-22, and is
stated once at **RC-1** above.

**Remediation:** one fix covers all four. Make the bridge modules hold only FFI-shaped *functions*,
and let the UDL/UniFFI layer generate the DTOs from the store types. Effort L, but it is the
highest-leverage structural change in this audit.

---

## F-16 — `DiscoveryMode` means "privacy gradient" in one place and "transport mechanism" in another

**Severity: MEDIUM** · **Blast radius: 74 occurrences; both are re-exported near the crate root**

| Location | Variants | What it actually selects |
|---|---|---|
| `core/src/settings.rs:4` | `Normal, Cautious, Paranoid` | a **privacy** dial (persisted user setting) |
| `core/src/transport/discovery.rs:29` | `Open, Manual, DarkBLE{group_key}, Silent, LanOnly` | a **discovery mechanism** |
| `wasm/src/lib.rs:20` | `Normal, Cautious, Paranoid` | a third copy of the settings one |

`core/src/lib.rs:98` re-exports the settings one; `core/src/transport/mod.rs:65` re-exports the
transport one. Both are reachable as `scmessenger_core::DiscoveryMode` and
`scmessenger_core::transport::DiscoveryMode`, and the variant sets do not overlap except
`Cautious`.

There is also a likely **use**-vs-**name** gap: `MeshSettings.discovery_mode: settings::DiscoveryMode`
(`settings.rs:20`, default `Normal`) is the persisted knob, while the discovery engine actually
branches on `discovery::DiscoveryMode::Open` (`discovery.rs:73,78,85,91`). I did not trace whether
`settings.discovery_mode` reaches the engine. **See Q-5.**

**Remediation:** rename the settings one `PrivacyLevel` (or `DiscoveryPrivacyLevel`) and the
transport one stays `DiscoveryMode`. The variant names then match what they select. Delete the
duplicate in `wasm/src/lib.rs` and import from `core`. Effort S (rename) + M (if Q-5 confirms the
knob is unwired).

---

## RC-2 — Files that are not compiled keep every duplicate-name count inflated

**Members, in rank order:** F-19, F-20, F-21, F-32, F-33

**The cause, stated once here.** A `.rs` file that no crate root declares is invisible to
the compiler and visible to a grep, so a human searching for "the transport error type"
finds candidates the build never checks. Every duplicate-name count in this report is
therefore an upper bound. This is the cheapest group to clear: deletion removes a
finding outright.

**The inventory, verified by walking each crate's module graph** (`mod x;` resolved from
every crate root, recursively, excluding `bin/`, `tests/` and `examples/`, which are
separate compilation units):

| Crate | Unreachable file | Lines | Written up as |
|---|---|---:|---|
| `core` | `core/src/wasm_support/mesh.rs` | 442 | **F-19** (with the two below) |
| `core` | `core/src/wasm_support/storage.rs` | 492 | **F-19** |
| `core` | `core/src/wasm_support/transport.rs` | 468 | **F-19** |
| `cli` | `cli/src/api_axum.rs` | 763 | **F-32** |
| `wasm` | `wasm/src/mesh.rs` | 550 | **F-33** |
| `wasm` | `wasm/src/storage.rs` | 497 | **F-33** |
| `wasm` | `wasm/src/worker.rs` | 447 | **F-33** |
| | **7 files, 3 crates** | **3,659** | all seven are now findings |

`cli/src/api_axum.rs` is declared in neither `cli/src/lib.rs` nor `cli/src/main.rs`, and
nothing in `cli/`, `core/` or `wasm/` mentions it. It restates nine type names that exist
for real in `cli/src/api.rs` — a second copy of a surface, in a file nobody can see.

The `wasm` crate is a third case: its `Cargo.toml` is listed in the workspace `members`
list *and* in `exclude`, so `--workspace` skips it, and CI's `cargo test --workspace`,
`cargo clippy --workspace` and `cargo doc --workspace` never build it. Inside it,
`lib.rs` declares only `connection_state`, `daemon_bridge`, `notification_manager` and
`transport` — `mesh.rs`, `storage.rs` and `worker.rs` are unreachable even when the
crate is built by hand. **Treat every `wasm/src/` citation in this report as a
"verified by hand, not by CI" citation.**

**This inventory is the reason several counts in `GLOSSARY.md` are marked.** Every
occurrence count in this report is a text count over first-party source, and a text count
cannot tell a compiled file from an unreachable one. The marked cells are the ones where
the difference changes a decision; the measurement is GLOSSARY's, under §D.

**Caveat on the counts.** Because the cause is invisible to the compiler, a "defined
twice" claim in this group may be "defined once and parked". Each finding below states
its own evidence; none of them claims a runtime defect.

---

## F-19 — `core/src/wasm_support/{mesh,storage,transport}.rs` are not compiled, and they seed the duplicate-name clusters

**Severity: HIGH (as a Layer 2 amplifier)** · **1,402 lines / 41,188 bytes across 3 files** (measured; the wider 7-file uncompiled inventory is **RC-2**'s)

`core/src/wasm_support/mod.rs` in full:

```rust
pub mod rpc;
```

`mesh.rs`, `storage.rs`, and `transport.rs` sit beside it in the same directory (dated Aug 14 and
Jul 2) and are **not declared**. A grep for `wasm_support::` across `core/src`, `cli/src`, `wasm/src`,
`desktop_bridge/src` returns only `wasm_support::rpc` — plus one comment at
`core/src/relay/client.rs:367` that references `wasm_support::transport::WasmTransportManager`.

They are not compiled, so nothing in them can be wrong at runtime. But they contain `MeshError`
(`core/src/wasm_support/mesh.rs:45`), `TransportError` (`core/src/wasm_support/transport.rs:66`),
`ConnectionState` (`core/src/wasm_support/transport.rs:32`), `DiscoveryMode`, `WasmMeshNode`
(`core/src/wasm_support/mesh.rs:57`), `TransportState`, `WebSocketRelay`, `StorageError`
(`core/src/wasm_support/storage.rs:45`), `RelayStats`, `PeerInfo`. **Every one of those is also
defined somewhere real.**

This is why a reader searching for "the transport error type" finds four candidates (F-20), and why
`MeshError` looks doubly-defined (F-21). It is a naming problem whose source is dead code, which
means the cheapest fix is deletion.

**Remediation:** delete the three files (or add `#[path]`-declared modules if they are a deliberate
reference copy — in which case say so in a README, because nothing says so now). Effort S.
**Uncertainty:** they may be a deliberately parked v2 implementation. I found no reference. **Q-6.**

---

## F-20 — `TransportError` is defined four times

**Severity: HIGH** · **Blast radius: 42 occurrences, 4 definitions, 2 of them re-exported at the crate root**

| Location | Variants | Note |
|---|---|---|
| `core/src/error.rs:154` | `NoiseHandshake, ConnectionReset{peer_id}, …` | the crate-root one (`core/src/lib.rs:73-75`) |
| `core/src/transport/abstraction.rs:233` | `PeerNotFound, TransportNotAvailable, SendFailed, ConnectionFailed, InvalidPayload, …` | |
| `core/src/wasm_support/transport.rs:66` | `ConnectionFailed, NotConnected, InvalidUrl, MessageTooLarge, AlreadyConnected, SendFailed` | uncompiled (F-19) |
| `cli/src/transport_api.rs:16` | `InvalidPeerId, InvalidCapabilities` | `InvalidCapabilities` is `#[allow(dead_code)]` and, per its own comment, *"not yet constructed anywhere"*| `core/src/transport/abstraction.rs` and `core/src/wasm_support/transport.rs` share two variant
*names* (`ConnectionFailed`, `SendFailed`) with different arities. The CLI's two-variant enum is a
fourth dialect.

**Remediation:** delete the wasm one with F-19. Merge the CLI's two variants into the abstraction
enum (it is a 35-line file; the `#[allow(dead_code)]` variant is a reserved slot, not a concept).
Keep `error.rs` as canonical for protocol/handshake failures and `abstraction` for send-path
failures, and **rename them apart**: `TransportError` and `SendError` say which one you have.
Effort S.

---

## F-21 — `MeshError` is defined twice, and the crate root exports one of them

**Severity: MEDIUM** · **Blast radius: 102 occurrences**

- `core/src/error.rs:20` — `VersionMismatch{received, expected}`, transport failures, …
- `core/src/wasm_support/mesh.rs:45` — `StoreError, InvalidEnvelope, NotActive, RelayDisabled` (uncompiled)

`core/src/lib.rs:30` re-exports the first as `scmessenger_core::MeshError`. The second is
invisible to the compiler but greppable to humans. F-19 resolves this one.

---

## F-32 — `cli/src/api_axum.rs` is not compiled, and it restates nine types that exist for real

**Severity: MEDIUM** · **Blast radius: 763 lines, 9 duplicated type names, 89 occurrences of those names; no runtime risk, because nothing compiles it**

**Location:** `cli/src/api_axum.rs` (763 lines). Not declared in `cli/src/lib.rs` (which lists
twelve `pub mod`s) nor in `cli/src/main.rs` (which lists eleven `mod`s). Nothing in `cli/`,
`core/` or `wasm/` references the module.

**Evidence — the file is written to be compiled, and says it is not:**

```rust
// cli/src/api_axum.rs:1-2
// Axum-based server implementation for SCMessenger Control API
// This file contains the new Axum 0.7 server implementation
```

```rust
// cli/src/api_axum.rs:20
use super::api::{
    AddContactRequest, AddContactResponse, ConnectionPathStateResponse, DiscoveredPeer,
    ...
```

`super::api` resolves only if this file is a direct child module of the crate root, so it
was authored to sit beside `mod api;` in `main.rs` and never got a `mod` line. It imports
nineteen names from the module it duplicates.

```rust
// cli/src/api_axum.rs:28-32
/// Default number of messages `/api/history` returns when `limit` is
/// omitted. Kept in sync with `api::DEFAULT_HISTORY_LIMIT`; see that
/// constant's doc comment for rationale. NOTE: this module is not currently
/// wired into any `mod` tree / bin target (see `start_api_server` below) --
/// `cli/src/api.rs` is the live implementation bound to `API_PORT`.
```

**The file documents its own exclusion, and still carries a manual sync obligation**
(`DEFAULT_HISTORY_LIMIT` at `:33`, "Kept in sync with `api::DEFAULT_HISTORY_LIMIT`") that
no compiler checks, because no compiler reads it.

**Nine of its ten type definitions are restatements of live ones:**

| Name | `cli/src/api_axum.rs` (uncompiled) | `cli/src/api.rs` (live) |
|---|---|---|
| `TestConfig` | `:38` | `:217` |
| `SubmitRunRequest` | `:47` | `:226` |
| `SubmitRunResponse` | `:53` | `:232` |
| `PollStatusResponse` | `:72` | `:238` |
| `FetchArtifactResponse` | `:79` | `:245` |
| `RunStatus` | `:85` (enum) | `:251` (enum) |
| `RunState` | `:93` | `:259` |
| `RunRegistry` | `:100` | `:266` |
| `ApiContext` | `:107` | `:612` |
| `IdentityResponse` | `:59` | *no counterpart — unique to this file* |

`start_api_server` (`:711`) builds a second `Router` on `API_PORT` — the same port
`api.rs` binds. Two servers, one port, one live.

**What this costs a reader:** the run-control surface F-11 describes appears to be
implemented twice. `cli/src/api_axum.rs:746` re-declares
`/fetch-artifact/:run_id/:name` — the route F-11 cites as the only `fetch_` in the
repository. That claim is true of *compiled* code and false of the tree, which is the
whole hazard.

**Remediation — two options, and the report does not choose between them.** The decision
belongs to whoever owns the CLI's HTTP surface:
1. **Delete the file.** Effort S. Nothing imports it, nothing compiles it, and its own
   comment says `api.rs` is the live implementation. This also removes 4 of the 31
   `cfg` occurrences and 39 of the 121 `url` occurrences counted elsewhere in this
   report (see GLOSSARY §D).
2. **Wire it in and reconcile.** Effort L. That means deciding whether axum 0.7 replaces
   the hand-rolled router in `api.rs` or coexists with it, then resolving nine names,
   two `RunRegistry` definitions, and the `DEFAULT_HISTORY_LIMIT` sync obligation — which
   is a *different* fix from the constant's current status quo.

**What cannot be determined from the repository.** The file contradicts itself about its
own purpose: its header calls it "the new Axum 0.7 server implementation", and a doc
comment thirty lines down says the current implementation is live and this one is not
wired. I cannot tell whether the intent was to replace `api.rs` or to discard this. I
also cannot tell whether the missing `mod` line is an accident or a deliberate
reversion — no commit message, ticket, or comment records it, and this audit does not
read prior audits. **Q-12.**

---

## F-33 — three `wasm/src/` files are unreachable from their own crate root, and two names exist nowhere else in the build

**Severity: MEDIUM** · **Blast radius: 1,494 lines across 3 files, 16 type declarations, 2 names with zero compiled definitions; no runtime risk, because nothing compiles them**

**Locations:**
- `wasm/src/mesh.rs` (550 lines), `wasm/src/storage.rs` (497), `wasm/src/worker.rs` (447)
- `wasm/src/lib.rs:3-6` declares the only four modules in the crate: `connection_state`,
  `daemon_bridge`, `notification_manager`, `transport`
- `wasm/src/mesh.rs:12` — `use crate::storage::{StoredMessage, WasmStorage, StorageConfig};`

The three files reference each other and nothing else. No reachable module in the crate
names `crate::mesh`, `crate::storage` or `crate::worker`, and no other crate in the
workspace depends on the `wasm` crate.

**Evidence — two type names have no compiled definition anywhere in the repository:**

```rust
// wasm/src/mesh.rs:68
pub struct WasmMeshNode {
```

```rust
// core/src/wasm_support/mesh.rs:57
pub struct WasmMeshNode {
```

`WasmMeshNode` (35 occurrences, 2 definitions) is defined in exactly two places and
**both are files the build never reads**. The same is true of `StoredMessage` (19
occurrences), which is defined once, at `wasm/src/storage.rs:47`, and nowhere else.

**`StoredMessage` is not a neutral name — it is a collision with a remediation already
proposed in this report.** F-02 offers `StoredMessage` as one of the honest names for the
storage-layer record:

> rename them honestly (`StoredMessage` / `MessageDto`) so the duplication is visible in
> review.

Adopting it would bind the name to a type that exists only in an unreachable file.
GLOSSARY A-3, which owns the canonical-name decision, chose `MessageRecord` /
`MessageRecordDto` instead and is unaffected — but **F-02's alternative is not**, and a
reader taking that branch without checking would collide.

The other fourteen declarations (`MeshConfig`, `MeshNodeState`, `PeerInfo`, `RelayStats`,
`EvictionPolicy`, `StorageConfig`, `WasmStorage`, `ServiceWorkerBridge`,
`ServiceWorkerStatus`, `BackgroundSyncConfig`, `PushNotificationPayload`,
`PushNotificationHandler`, `DefaultSyncHandler`, `DefaultNotificationHandler`) duplicate
names that do exist elsewhere, so each one is a false positive in any duplicate-type scan.

**Remediation — two options, and the report does not choose between them.**
1. **Delete the three files.** Effort S. Nothing references them; the crate builds and
   tests without them. This removes the `StoredMessage` and `WasmMeshNode` collisions and
   the fourteen false duplicate names in one step.
2. **Declare them and put the crate back in the build.** Effort L, and not a rename at
   all: `wasm` is listed in the workspace `members` *and* in `exclude`, so
   `cargo test --workspace` and `cargo clippy --workspace` skip it entirely. Making these
   files live means deciding what builds the WASM target and when. **This audit does not
   take that position** — it is a build-topology decision outside a naming audit, and it
   is recorded as an open question rather than answered here. **Q-13.**

**What cannot be determined from the repository.** Nothing in these three files, in
`wasm/src/lib.rs`, or in the crate's `Cargo.toml` says why they left the module tree. The
comments describe an in-memory storage layer for browser environments and a service-worker
bridge — coherent work, not scratch — but there is no note, ticket, or reference marking
them as parked. I cannot tell whether they are a paused feature or a superseded draft.

---

## RC-3 — A surface has a second, already-diverged copy of itself

**Members, in rank order:** F-06, F-15

**The cause, stated once here.** A copy of a surface was made instead of a reference to
it, the copy was then modified independently, and the tests kept passing because they
exercise the copy. The shipped surface and the tested surface are different surfaces.
F-06 is the large instance — an entire command grammar, with eight duplicated type
names (`Cli`, `Commands`, `IdentityAction`, `ContactAction`, `BlockAction`,
`ConfigAction`, `DiscoveryAction`, `SwarmAction`), each cited in F-06 itself. F-15 is
the same shape nine lines long, inside a single file. The severity gap between them is
blast radius, not kind.

---

## F-06 — `cli/src/cli.rs` is a second, already-diverged copy of the `scm` command grammar

**Severity: HIGH** · **Blast radius: the CLI's user-facing surface; 8 duplicated type names**

**Locations:**
- `cli/src/cli.rs:154` `pub struct Cli`, `cli/src/cli.rs:160` `pub enum Commands` (+ `IdentityAction`,
  `ContactAction`, `BlockAction`, `ConfigAction`, `DiscoveryAction`, `SwarmAction`)
- `cli/src/main.rs:194` `struct Cli`, `cli/src/main.rs:232` `enum Commands` (+ the same six actions)
- `cli/src/lib.rs:10` declares `pub mod cli;`; `cli/src/main.rs`'s `mod` list does **not**.

**Evidence — the two grammars have already drifted.** Diffing the `Commands` bodies:

```
< /// Run headless relay/bootstrap node (no interactive console)      [cli/src/cli.rs]
---
> /// Run headless relay node (no interactive console)                 [cli/src/main.rs]
```

and `main.rs`'s `Cli` carries a global `--http-bind` flag and `Commands::Run` carries
`--auto-reply`, neither of which exists in `cli/src/cli.rs`.

`cli/src/cli.rs` is compiled only into the library target and is referenced only by
`cli/tests/integration.rs:3`:

```rust
use scmessenger_cli::cli::{Cli, Commands, ContactAction};
```

**So the CLI parser tests parse a grammar that is not the grammar that ships.** The test file reads
as if it validates `scm contact add`; it validates a sibling that no user can invoke. This is
precisely the failure mode AGENTS.md Rule 16 describes — a restored/declared thing with no reachable
call site — expressed in type names rather than in wiring.

**Remediation:** move the single grammar into `cli/src/cli.rs` as the one definition, have
`main.rs` import it, and delete the copy. Alternatively delete `cli/src/cli.rs` and point the tests
at `main.rs`'s (harder — `main.rs` types are private). Either way the *names* stop meaning "one of
two grammars". Effort S–M.

---

## F-15 — `SwarmTaskLivenessGuard` is copy-pasted verbatim into one 11,190-line file

**Severity: LOW** · **Blast radius: `core/src/transport/swarm.rs:3851` and `:8309`**

```rust
// swarm.rs:3851 and swarm.rs:8309 — identical, ~9 lines each
struct SwarmTaskLivenessGuard(Arc<AtomicBool>);
impl Drop for SwarmTaskLivenessGuard { ... }
let _liveness_guard = SwarmTaskLivenessGuard(event_loop_alive);
```

Two identical private type definitions in the same file, in two different task bodies. This is the
copy-paste-divergence pattern the brief names: nothing distinguishes them today, and nothing will
when one of them needs a fix.

**Remediation:** hoist to one `mod`-level definition near the top of `swarm.rs`. Effort S.

---

## RC-4 — A type name has no owning layer, so each layer defines its own

**Members, in rank order:** F-07, F-08, F-04, F-18, F-29

**The cause, stated once here.** Five type names are used in more than one layer with no
layer owning the definition, so "what is the `TransportType`" has a different answer in
Rust than in Kotlin than in Swift. The result is not a duplication of a known type — it
is N types that agree on a name and nothing else, with variant sets that do not even
overlap. These are collision failures: the name is load-bearing, and it resolves to
whichever definition the importing file happened to reach.

**This group is a decision queue, not a work queue.** Unlike RC-1 to RC-3 there is no
single change here: each member needs its own canonical choice plus a compatibility
approach, and those are owned by `GLOSSARY.md` — F-07 and F-08 point at their entries.
F-18's fifth definition is the uncompiled one and belongs to **RC-2**; clearing it is
F-19's deletion, not a decision made here.

---

## F-07 — `PeerId` names two different things in one crate, and the crate says so in a comment

**Severity: HIGH** · **Blast radius: 947 occurrences of the token; `error.rs`, `global.rs`, `resume_prefetch.rs`, `iron_core.rs` all in scope**

**Locations:**
- `core/src/routing/local.rs:15` — `pub type PeerId = [u8; 32];` (an Ed25519 public key)
- `libp2p::PeerId` — a multihash identity string (`12D3KooW…`), used throughout
- Re-exported at `core/src/routing/mod.rs:34` **and** `core/src/transport/mod.rs:93`, so
  `scmessenger_core::transport::PeerId` is the `[u8; 32]` alias

**Evidence — the hazard is already documented, which means it is already known:**

```rust
// core/src/routing/resume_prefetch.rs:433
// routing::PeerId is [u8; 32], not libp2p::PeerId
```

```rust
// core/src/iron_core.rs:2767
let peer_id: crate::routing::PeerId = peer_id_bytes.try_into().unwrap_or([0u8; 32]);
```
in the same file as, at `iron_core.rs:4017` and `:4189`:
```rust
... -> std::collections::HashMap<libp2p::PeerId, ...>
dyn Fn(libp2p::PeerId, crate::transport::health::ConnectionState) + Send + Sync,
```

`core/src/error.rs:13` does `use crate::routing::local::PeerId;` — the crate's central error module
imports the byte-array alias under the libp2p name. Any error variant carrying a `PeerId` is
therefore carrying 32 raw bytes, while every other `PeerId` in the system is a string.

**Remediation (vocabulary, mechanical, high payoff):** rename the alias to what it is —
`Ed25519PublicKey` (or `RawPublicKey`). It is a `pub type`, so this touches only the `routing`
subtree. Do **not** touch `libp2p::PeerId`. The canonical-name decision and the alias list
are owned by **GLOSSARY A-5**; if A-5 changes, only that entry needs editing. Effort S,
blast radius contained to `core/src/routing/` plus `error.rs`.

---

## F-08 — `TransportType` is defined seven times across three languages, with disjoint variant sets

**Severity: HIGH** · **Blast radius: 710 occurrences of the name**

| # | Location | Variants |
|---|---|---|
| 1 | `core/src/transport/abstraction.rs:11` | `BLE, WiFiAware, WiFiDirect, Internet, Local` |
| 2 | `core/src/routing/local.rs:19` | `BLE, WiFiAware, WiFiDirect, TCP, QUIC, Circuit` |
| 3 | `core/src/relay/client.rs:24` | `Tcp, Quic, WebSocket` |
| 4 | `android/.../service/MeshEventBus.kt:163` | `BLE, WIFI_AWARE, WIFI_DIRECT, INTERNET, TCP_MDNS` |
| 5 | `android/.../transport/SmartTransportRouter.kt:27` | `WIFI_DIRECT, BLE, CORE, TCP_MDNS` |
| 6 | `iOS/.../Models/Models.swift:70` | `multipeer, ble, internet, tcpMdns` |
| 7 | `iOS/.../Services/MeshEventBus.swift:70` | `ble, multipeer, internet, tcpMdns` |

**`BLE` is the only variant present in all seven.** The two Android definitions disagree with each
other (#4 has `WIFI_AWARE`/`INTERNET`, #5 has `CORE`; neither has the other's). `CORE` — "the
libp2p/internet relay", per the comment at `SmartTransportRouter.kt:30` — exists nowhere else.
`LOCAL` (a test transport) is declared in production code at `abstraction.rs:17`.

Naming note: the three Rust copies disagree on case (`Tcp` vs `TCP`), and the two iOS copies agree
exactly — suggesting the iOS pair was copied and the Rust pair was not.

**The sentence test:** "the transport a message went over" cannot be written without "or" for
definitions 1–3 (which describe *mechanisms*: BLE, TCP, QUIC) versus 4–7 (which describe *paths*:
internet, LAN, Multipeer). These are two different concepts wearing one name.

**Remediation:** this is **both** layers and must be done in that order.
1. **Design first.** Decide whether the concept is a *mechanism* (how bytes move) or a *path*
   (direct LAN vs. via the mesh). Do not attempt this as a rename. **The canonical names for the
   two resulting concepts are owned by GLOSSARY C-2**, which recommends `TransportMechanism` and
   `MessagePath`; that entry also owns the rename plan and the compatibility approach. If C-2
   changes, this finding needs no edit.
2. Then generate the per-language enums from one definition (a UDL enum plus a codegen step, or a
   checked-in generated file with a CI drift check).
3. Retire `SmartTransportRouter.TransportType.CORE` or give it a real name; `CORE` is not a
   transport, it is "whatever the core node did".

Effort L. This is the largest single item in the audit and the one most likely to be contested.

---

## F-04 — `RetryPolicy` is defined twice and both copies claim to be the only one

**Severity: HIGH** · **Blast radius: 12 occurrences, 2 definitions, both at the crate root**

**Locations:**
- `core/src/lib.rs:189-198` — `pub mod retry_policy { pub struct RetryPolicy { ... } }`
- `core/src/store/outbox.rs:904` — `pub struct RetryPolicy { ... }`
- `core/src/lib.rs:100` — `pub use store::outbox::RetryPolicy;`

So `scmessenger_core::RetryPolicy` (the outbox one) and
`scmessenger_core::retry_policy::RetryPolicy` (the module one) are **two distinct,
mutually incompatible types with the same name**, both reachable from the crate root.

**Evidence — the same doc comment, verbatim, in both files:**

```rust
// core/src/lib.rs:193-196
/// This is the ONLY place retry policy is defined. All platforms
/// (CLI, Android, iOS, WASM) use this struct. Changes to backoff
/// strategy apply everywhere automatically.
```

```rust
// core/src/store/outbox.rs:899-902
/// This is the ONLY place retry policy is defined. All platforms
/// (CLI, Android, iOS, WASM) use this struct. Changes to backoff
/// strategy apply everywhere automatically.
```

An identical claim of uniqueness, duplicated, is the strongest available proof that the duplication
is not deliberate. The field sets differ: `store::outbox::RetryPolicy` has
`suppress_on_custody: bool`; `lib.rs::retry_policy::RetryPolicy` does not.

Add a third sibling, `BackoffStrategy` (`core/src/routing/smart_retry.rs:10`,
`{ base_ms, max_ms, multiplier }`) and a fourth, `RetryStrategy`
(`core/src/transport/mesh_routing.rs:299`, re-exported at `core/src/transport/mod.rs:65`), and the
concept "how long to wait before retrying" has three types and two of them compute the same thing.

**Remediation (vocabulary + deletion):** keep `store::outbox::RetryPolicy` — it is the one with
`suppress_on_custody`, it is the one re-exported at the crate root, and it is the one the doc
comment describes. Delete `lib.rs::retry_policy` entirely (it is 45 lines including its own tests).
Rename `BackoffStrategy` → `BackoffPolicy` or fold it into `RetryPolicy`; rename `RetryStrategy`
in `mesh_routing` after establishing what it actually does — it may be a *routing* policy wearing a
retry name. This is a public-API rename; the outbox one is already `Serialize`, so see the
compatibility note in GLOSSARY.

---

## F-18 — `ConnectionState` means four different things

**Severity: MEDIUM** · **Blast radius: 102 occurrences, 5 definitions**

| Location | What it is |
|---|---|
| `core/src/transport/health.rs:15` | TCP connection lifecycle: `Connecting, Connected, Disconnecting, Disconnected, …` |
| `core/src/relay/client.rs:72` | relay-circuit lifecycle: `Connecting, Handshaking, Connected, Disconnected` |
| `core/src/relay/server.rs:49` | same as above, again, private |
| `core/src/wasm_support/transport.rs:32` | a third, in an **uncompiled** file (see F-19) |
| `cli/src/ble_windows.rs:21` | **`fragments: HashMap<u16, Vec<u8>>`, `total_fragments: Option<u16>`** |

The last row is the clearest instance in the audit: a BLE L2CAP **fragment reassembly buffer** named
`ConnectionState`, in a file whose sibling is also called `ConnectionState`, meaning a TCP connection
lifecycle. No single sentence describes both.

**Remediation:** `cli/src/ble_windows.rs:21` → `FragmentReassembly` (or `L2capReassembly`).
The two relay copies merge into one. Effort S.

---

## F-29 — `serviceInfo` names three different types, one of them a raw `ByteArray`

**Severity: MEDIUM** · **Blast radius: 98 occurrences across 8 Android files, 3 distinct types**

**Locations and the three things the one name refers to:**

| Where | What `serviceInfo` actually is | Count |
|---|---|---|
| `android/.../transport/MdnsServiceDiscovery.kt:56,183` | `NsdServiceInfo` — an Android framework mDNS record | 37 |
| `android/.../transport/WifiDirectTransport.kt:226` | `WifiP2pDnsSdServiceInfo` — a different Android framework record | 2 |
| `android/.../transport/WifiAwareTransport.kt:35,51`; `TransportManager.kt:35,384`; `AndroidPlatformBridge.kt:463`; `MeshRepository.kt:1319` | **`ByteArray`** — raw, hand-decoded TLV bytes | 15 (see note) |

**On the Count column:** the first two are whole-corpus counts and reproduce exactly on
`--basis wide`. The third, `ByteArray` 15, is **not** a corpus count at all: it is the number
of `ByteArray`-typed `serviceInfo` sites this table enumerates, where the corpus figure for
that name is 129. The two must not be compared.

**Evidence — the third case is the tautology, because the suffix promises structure the type does
not carry:**

```kotlin
// android/.../transport/WifiAwareTransport.kt:51
fun decodePortTlv(serviceInfo: ByteArray): Int? {
    while (i + 1 < serviceInfo.size) {
        val tlvType = serviceInfo[i]
        val tlvLen = serviceInfo[i + 1].toInt() and 0xff
        ...
```

A `ByteArray` called `serviceInfo` is a value named for information it does not contain. The caller
must know the TLV encoding, which is why `decodePortTlv` exists to reconstruct the one field the
code actually wants, by hand, at every use site. The identical name on a structured
`NsdServiceInfo` two files away means a reader cannot tell from the name whether they are holding
a parsed record or an unparsed buffer.

**Remediation:** the two framework types keep their names — they mirror the framework class and are
not the project's to rename. The `ByteArray` case is the defect: rename it `serviceTlvBytes`
(WiFi Aware) or `serviceBlob`, and introduce a `ServiceTlv` decoder type so `decodePortTlv` is a
method on it rather than a free function every caller must remember to invoke. The name is
currently a *lie about structure*; make it a lie about bytes. Effort S.

**Why this was previously listed as "considered and rejected":** the original pass saw 99
occurrences and assumed the surrounding Android types disambiguated. They do at the type level —
but the third row above is genuinely overloaded, so the earlier rejection was wrong. Corrected.

---

## REMAINDER — findings with no shared cause

Eleven of the thirty active findings share no cause with anything else in this audit,
and are not forced into a group to make the table of contents tidier. Each is local to
its own name: a union of meanings in one field, a name that contradicts its own doc
comment, a workaround that outlived its cause, a suffix that adds nothing.

Three of them are HIGH, which is why this section sits with the HIGH sections rather
than at the end of the file. Two retired or duplicate records travel here rather than in
a group: **F-25** is retired and points at F-01, its owner; **F-27** records F-14 again
from the vocabulary angle and sits directly beneath it.

The tautology scan that produced F-28 and F-31 — including every candidate that was
examined and deliberately kept — is the closing table of this section.

---

## F-09 — `Contact.peer_id` is a union of three identifier kinds encoded in a bare `String`

**Severity: HIGH** · **Blast radius: 2144 `peerId` occurrences (Android), 279 `canonicalPeerId`**

**Evidence — three shapes for one field, discriminated by runtime string-sniffing:**

1. **libp2p identity multihash** (base58, `12D3KooW…`) — the historic form
2. **canonical 64-char hex** of the Ed25519 public key — introduced by "UNIFICATION". The ledger
   transparently rewrites between the two on every write
   (`core/src/store/ledger_entry.rs:486-489`: *"UNIFICATION: live canonicalization helper …
   Converts libp2p 12D3 peer_id to canonical 30d0fa public_key_hex on every write"*)
3. **synthetic placeholder** — `"peer-…"`. `core/src/store/contacts.rs:104`:
   ```rust
   normalized.to_lowercase().starts_with("peer-")
   ```
   and again at `core/src/message/identity_envelope.rs:176`. The constant
   `PLACEHOLDER_KEY_NOTE` (`store/contacts.rs:21`) documents the third state: *"public_key
   unavailable: not self-certifying from peer id; awaiting verified key"*.

The type is `String`. The discriminator is `len() == 64` (`ledger_entry.rs:566`) *or* a
`starts_with("peer-")` prefix test. `store/contacts.rs:1394` even has a fixture with
`"peer_id":"peer-old"`.

**The name `canonicalPeerId` is the sharpest symptom.** It appears 279 times in the Android layer
alone (`MeshRepository.kt:736, 829, 838, 1894, 2187…`) and means "the hex form, chosen over the
libp2p form". "Canonical" names a *policy*, not a *kind*. The same class
(`PeerDiscoveryInfo`, `MeshRepository.kt:801`) then carries five peer-id fields — `peerId`,
`canonicalPeerId`, `libp2pPeerId`, `routePeerId`, `blePeerId` — plus `transportToCanonicalMap`
(`MeshRepository.kt:672`, commented `// libp2pPeerId -> canonicalPeerId`).

**Remediation (design):** an enum.

```rust
enum PeerKey {
    Libp2p(String),          // validated by .parse::<libp2p::PeerId>()
    Hex(String),             // exactly 64 hex chars
    Placeholder,             // carries no key; see PLACEHOLDER_KEY_NOTE
}
```
Make the field private and expose accessors. The string-sniffing sites collapse into pattern
matches, and a fourth form becomes a compile error rather than a runtime surprise. This is a
storage-format change — `store/contacts.rs` already has to handle the legacy forms on read, so the
migration is a read-time widening with a write-time narrowing. Effort L, and it needs the owner's
decision on whether the hex form is meant to be permanent (see Q-3).

---

## F-05 — `SwarmEvent2` is the only `SwarmEvent`, and the `2` is hidden at the re-export

**Severity: HIGH** · **Blast radius: the entire transport event surface (CLI, mobile bridge, WASM, desktop bridge)**

**Locations:**
- `core/src/transport/swarm.rs:2703` — `pub enum SwarmEvent2` (the only definition)
- `core/src/transport/mod.rs:96` — `SwarmEvent2 as SwarmEvent`
- `core/src/lib.rs:101` — `pub use transport::{start_swarm, start_swarm_with_config, SwarmCommand, SwarmEvent, SwarmHandle};`

**Evidence:**

```rust
// core/src/transport/mod.rs:95-97
pub use swarm::{
    default_routing_engine_handle, extract_ed25519_public_key_from_peer_id, start_swarm,
    start_swarm_with_config, SwarmCommand, SwarmEvent2 as SwarmEvent, SwarmHandle,
};
```

`grep -n "^pub enum SwarmEvent" core/src/transport/swarm.rs` returns nothing — there is no v1. The
`_2` is a scar from a rename that was completed at the import site and never completed at the
definition site. `swarm.rs:245` is a comment that still names the type honestly
(`SwarmEvent2::PeerDiscovered`), while every consumer sees `SwarmEvent`.

**Why this is worse than a cosmetic suffix:** inside `swarm.rs` the type is called `SwarmEvent2`
and outside it is called `SwarmEvent`. A reader comparing a doc comment in `swarm.rs` against a
`use` statement in `main.rs` sees two different types and must know the alias exists to connect
them. The `2` was supposed to mark "deprecated v1, do not use" and now marks the *only* one.

**Remediation (vocabulary, mechanical, high payoff):** rename `SwarmEvent2` → `SwarmEvent` at the
definition and all ~40 use sites in `swarm.rs`; delete the `as` alias. Purely internal — the
external name does not change, so this is a no-op for FFI consumers. Effort S.

---

## F-01 — `current_timestamp()` means "seconds" in 7 places and "milliseconds" in 2

**Severity: HIGH** · **Blast radius: 61 occurrences, 9 definitions, cross-crate**

**Locations (all 9 definitions, all identical name):**

| Location | Returns |
|---|---|
| `core/src/contacts_bridge.rs:444` | `as_secs()` |
| `core/src/mobile_bridge.rs:4308` | `as_millis()` |
| `core/src/routing/local.rs:367` | `as_secs()` |
| `core/src/routing/neighborhood.rs:402` | `as_secs()` |
| `core/src/store/blocked.rs:672` | `as_secs()` |
| `core/src/store/contacts.rs:1012` | `as_secs()` |
| `core/src/store/history.rs:73` | `as_secs()` |
| `core/src/store/ledger_entry.rs:15` | **`as_millis()`** |
| `core/src/store/storage.rs:178` | `as_secs()` |

**Evidence (quoted):**

```rust
// core/src/store/ledger_entry.rs:15
fn current_timestamp() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64          // <-- milliseconds
}
```

```rust
// core/src/store/contacts.rs:1012
fn current_timestamp() -> u64 {
    ...
        .as_secs()                  // <-- seconds
}
```

The name promises a timestamp and says nothing about the unit. The codebase *knows* this, and
documents the hazard rather than fixing it — `core/src/store/ledger_entry.rs:2498`:

> "UNIT CONVERSION, do not "simplify" this away: `LedgerEntry::last_seen` is stored in
> **milliseconds** (see `current_timestamp` at the top of this file), while `SharedPeerEntry::last_seen`
> is a Unix timestamp in **seconds**"

Downstream, the hazard is carried by the field name `last_seen`. **This is the single owner of the
observed unit inventory for that name** — the rename plan, canonical names and compatibility
approach for it live in **GLOSSARY C-1**, not here:

| Type / binding | Spelled | Actual unit | Site |
|---|---|---|---|
| `Contact.last_seen: Option<u64>` | `last_seen` | seconds | `core/src/store/contacts.rs:31`, `core/src/contacts_bridge.rs:25` |
| `LedgerEntry.last_seen: Option<u64>` | `last_seen` | **milliseconds** | `core/src/store/ledger_entry.rs:167` |
| `SharedPeerEntry.last_seen: u64` | `last_seen` | seconds | `core/src/store/ledger_entry.rs:2509` (divided by 1000) |
| `PeerStatus::Stale` / `Dormant { last_seen }` | `last_seen` | seconds | `core/src/routing/local.rs:39,43` |
| `relay::protocol::PeerRecord.last_seen: u64` | `last_seen` | seconds | `core/src/relay/peer_exchange.rs:173` (vs `peer_ttl_secs`) |
| `NotificationEndpoint.last_seen_ts: u64` | `last_seen_ts` | **milliseconds** | `core/src/notification.rs:254,360` (from `now_ms()`) |
| Kotlin peer state | `lastSeen` | seconds | `android/.../MeshRepository.kt:810` |
| Kotlin BLE route observation | `lastSeenMs` | milliseconds | `android/.../MeshRepository.kt:847` |
| Kotlin ledger read | `lastSeenRaw` | milliseconds, **unconverted** | `android/.../MeshRepository.kt:6749` |
| HTTP/JSON wire key | `last_seen` | as-written | CLI HTTP API + `core/src/wasm_support/rpc.rs` |

**Cross-layer spelling.** The table above is also a Layer 2 finding in its own right: one wire field
is spelled **four different ways** across the stack — `last_seen` (Rust), `last_seen_ts`
(notification), `lastSeen` / `lastSeenMs` / `lastSeenRaw` (Kotlin). The Kotlin trio in particular is
three names for one concept, one of which (`lastSeenMs`) is the only one that names its unit
correctly and should be the model. *(This paragraph absorbed the retired finding F-25.)*

And once more, on the client, where the *name* admits the problem and the *code* does not fix it —
`android/.../MeshRepository.kt:6749`:

```kotlin
val lastSeenRaw = obj.optLong("last_seen", 0L)
val lastSeen  = if (lastSeenRaw > 0) lastSeenRaw.toULong() else null
```

`lastSeenRaw` is a ledger field (milliseconds, per `ledger_entry.rs:15`) assigned straight into a
field whose siblings are seconds (`MeshRepository.kt:810`: `System.currentTimeMillis().toULong() / 1000u`).
The `Raw` suffix promises a conversion that never happens. **See open question Q-1** — I am confident
about the name problem, less confident that this specific line is a live bug.

**Why no name fixes it:** the sentence "a Unix timestamp" is satisfiable by all nine. The problem is
that the *unit* is an implicit, unstated part of the type. This is a type problem wearing a
naming costume.

**Remediation (design, not vocabulary):**
1. Introduce one crate-level `Time` module exposing `now_secs() -> u64` and `now_millis() -> u64`
   with the unit in the name. Delete all nine private copies. (`crates.io` `web-time` already
   depends on the right primitive; this is a wrapper, not a dependency change.) *This finding owns
   this step; it is a design change, not a vocabulary decision.*
2. **The rename plan for the fields is owned by GLOSSARY C-1** — which binding keeps which unit,
   which wire key is frozen, and the compatibility approach. Do not restate it here; if C-1 changes,
   this finding needs no edit.
3. For the store layer, prefer a newtype (`Millis(u64)`) over a rename once the rename has landed;
   the rename is the cheap first step and the one that makes the newtype reviewable.

---

## F-25 — ~~Cross-layer name divergence for one field: `last_seen`~~ — RETIRED, absorbed into F-01

> **This finding ID is retired, not deleted.** Its entire content was a third statement of facts
> already owned elsewhere: the unit inventory is **F-01**'s evidence, and the canonical-name
> decision and rename plan are **GLOSSARY C-1**'s. Keeping a third copy is the defect this report
> was restructured to remove. The one thing F-25 uniquely added — that the field is spelled *four
> different ways across four layers*, which makes it a Layer 2 vocabulary problem and not only a
> units problem — is now stated in **F-01** under "Cross-layer spelling". The finding count in
> SUMMARY.md is 30 active findings plus this retired ID.

---

## F-17 — `blocked_identity_*` — four constructors named after their arguments, not their result

**Severity: MEDIUM** · **Blast radius: the UDL public API (`core/src/api.udl`), bound by both mobile clients**

```
blocked_identity_new
blocked_identity_with_device_id
blocked_identity_with_reason
blocked_identity_with_notes
```

Four entry points to the same type, distinguished only by which field the caller happened to supply.
A caller must know, before calling, that "the one with a reason" silently does not set the device
id. The sentence "constructs a BlockedIdentity" fits all four equally well; the sentence
"constructs a BlockedIdentity from a reason" fits exactly one, and that is the naming information
that is actually being conveyed.

**Remediation:** one constructor `BlockedIdentity::new(peer_id, reason?, device_id?, notes?)` with
`Option` fields, or a builder. The UDL change is source-breaking for Android and iOS; see the
GLOSSARY compatibility note. Effort S (Rust) / M (with client migration).

---

## F-14 — `ContactManagerFix.swift` — a `typealias` that makes `ContactManager` mean two things on iOS

**Severity: MEDIUM** · **Blast radius: the whole iOS contact surface**

**Location:** `iOS/SCMessenger/SCMessenger/ContactManagerFix.swift`

```swift
// line 1-2
// Fixed ContactManager implementation to work around UniFFI generation issues
// This provides a proper class structure that conforms to ContactManagerProtocol

// line 6
open class ContactManagerFixed: ContactManagerProtocol { ... }

// line 67
extension ContactManager: ContactManagerProtocol { }

// line 188
typealias ContactManager = ContactManagerFixed
```

On iOS, `ContactManager` resolves to (a) the UniFFI-generated Rust-backed class in
`Generated/api.swift` and `SCMessengerCore.xcframework`, or (b) `ContactManagerFixed`, a hand-written
wrapper — depending on which file you are reading and whether the typealias is in scope. The file
name itself (`ContactManagerFix`) names a *workaround*, not a concept, and a workaround that has
outlived its cause is a permanent tax on every reader.

**Remediation:** if the UniFFI issue is fixed, delete the file and the typealias. If it is not, rename
to something that names the concept — e.g. `ContactStore` — and keep a `// TODO(uni-ffi)` note. Do
not leave a typealias that shadows a generated class name. Effort S (delete) / M (rename).
**Compatibility:** iOS-only, no wire format involved; the UniFFI surface is unchanged either way.

---

## F-27 — iOS `typealias ContactManager = ContactManagerFixed` shadows a generated class

Recorded as F-14 immediately above; kept as its own ID because it is a *vocabulary* collision (one
name, two meanings in the same language) rather than a nameability problem. 78 occurrences
repo-wide.

---

## F-12 — `UiOutbound::Legacy(UiEvent) | JsonRpc(Value)` — a typed enum that erases its own types

**Severity: MEDIUM** · **Blast radius: `cli/src/server.rs`, the WebSocket console**

**Location:** `cli/src/server.rs:11-70`

```rust
// Stub types for BLE mesh UI integration (Phase 1B wiring)
pub enum UiEvent {
    ...
    ContactList { contacts: Vec<serde_json::Value> },
    HistoryList { peer_id: String, messages: Vec<serde_json::Value> },
    ConfigData  { config: serde_json::Value },
}

pub enum UiOutbound {
    Legacy(UiEvent),
    JsonRpc(serde_json::Value),
}
```

Four problems in 60 lines:

1. **The variants claim shapes they do not carry.** `ContactList` promises a contact list and
   delivers arbitrary JSON. The enum is a naming layer over untyped data — the "type" adds
   documentation, not safety.
2. **`UiOutbound` is two encodings of one concept.** "A message to the UI" is either a `UiEvent`
   or a raw `serde_json::Value`. Which one a given consumer receives is a runtime property.
3. **`UiEvent::IdentityInfo`** (line 40) shares its name with the `IdentityInfo` struct in
   `core/src/lib.rs:115`, and is a third distinct identity shape (the file's own header says
   "Stub types … Phase 1B wiring").
4. **Same concept, four field names, in one enum** — `UiCommand` (`cli/src/server.rs:72`):

```rust
ContactRemove { contact: String },   // <-- `contact`
ContactAdd    { peer_id: String, ... },
HistoryList   { peer_id: String, limit: Option<u32> },
Send          { recipient: String, message: String, id: Option<String> },
```

The thing being addressed is `peer_id` in two variants, `contact` in a third, and `recipient` in a
fourth — all within fifteen lines.

**Remediation:** if these stubs are still bound, promote `ContactList`/`HistoryList`/`ConfigData`
to real types or delete the variants. Rename the three address fields to one. If they are not bound,
delete the whole block — the header comment suggests they are Phase-1B scaffolding. **See Q-4**:
I could not determine reachability within this audit's read-only budget.

---

## F-13 — `isRelay: Boolean = false` — a field whose concept was deleted and whose name survived

**Severity: MEDIUM** · **Blast radius: 75 occurrences; sits on the most-imported Android type**

**Location:** `android/.../data/MeshRepository.kt:799-809`

```kotlin
// UNIFICATION_V2: All nodes are relays — isRelay retained for backward compat but no longer distinguishes.
// Every node is a full relay; the field is now always false in UI terms (no "infrastructure" category).
data class PeerDiscoveryInfo(
    ...
    val isFull: Boolean,         // True if peer identity is authenticated
    val isRelay: Boolean = false,
    ...
)
```

The comment is unambiguous: the field no longer distinguishes anything and is always false, and it
is kept "for backward compat". A boolean that is always false is a name for nothing. It is also
*adjacent* to `isFull`, a boolean that is very much not always false, and the two are read
together in the same data class — which is the worst possible place for a dead flag.

AGENTS.md's architecture doctrine is explicit that there are no standalone relays, so this is a
concrete instance of a settled decision not having propagated to the data model.

**Remediation:** delete `isRelay`. If any consumer still reads it, `git grep -n "isRelay"` on the
Android module will show them; the field is defaulted, so removal is source-compatible for
construction sites but breaking for readers. Effort S.

**Outcome, 2026-10-01 — partially applied, and this entry's premise was wrong.** The operator
confirmed that *all nodes relay* is core philosophy and that nothing may question it at the code
level. The deletion was NOT taken, because the field is not always false as this entry claims:
`isKnownRelay()` and `isBootstrapRelayPeer()` do return `false` unconditionally, but `isRelayHop`
and `isInfraRelay` still populate it true, and three consumers read it —
`collectKnownRelayPeerIds` filters on it (Kotlin), `dynamicRelays` filters on it (iOS), and
`isFull = !isRelay && ...` depends on it. Deleting it would therefore have been a behaviour
change smuggled in as a rename.

What was applied instead is the naming half: `isRelay` -> `isInfraNode`, 75 occurrences across
`MeshRepository.kt`, `ContactsViewModel.kt`, `DashboardViewModel.kt`, `MeshRepository.swift` and
`MeshDashboardView.swift`, satisfying rule A-2 (canonical role noun `node`). The field's real
meaning -- infrastructure peers -- is now what it says. **Deleting it remains open** and needs a
decision about those three consumers.

---

## F-10 — `calculate_next_attempt` returns a timestamp, and `isAtMaxDelay` is camelCase in a snake_case crate

**Severity: MEDIUM** · **Blast radius: re-exported at `core/src/routing/mod.rs:38` and `core/src/transport/mod.rs:90`**

**Evidence:**

```rust
// core/src/routing/smart_retry.rs:22-29
/// Calculate the next attempt time based on exponential backoff strategy.
/// # Returns
/// Unix timestamp (in milliseconds) when the next attempt should occur.
pub fn calculate_next_attempt(attempt_count: u32, strategy: &BackoffStrategy) -> u64 {
```

The doc comment is the contract and it contradicts the name twice: the function returns a *time*,
not an *attempt*; and it returns **milliseconds** — a third unit in a repository where the F-01
question is already live. The parameter `attempt_count` is also a lie in the other direction: the
body treats `0` as "first attempt, no delay" and `n` as "n previous attempts", so it is an attempt
*index*, not a count.

The result is also non-deterministic and untestable without a wall clock — every call reads
`SystemTime::now()`, and the tests (lines 142–154) work around it with `result1`, `result2`,
`result3`, `result10` and ±100 ms tolerance windows:

```rust
let result1 = calculate_next_attempt(1, &strategy);
assert!(result1 >= now + 1000 && result1 <= now + 1100);
```

Those four `resultN` names are the numeric-suffix-by-copy-paste pattern named in the brief, and
`result10` in particular names nothing but "the tenth call in this test".

**Second name in the same file:**

```rust
// core/src/routing/smart_retry.rs:49-54
#[allow(non_snake_case)]
pub fn isAtMaxDelay(delay_ms: u64, max_ms: u64) -> bool {
    delay_ms >= max_ms
}
```

A one-line comparison with a `#[allow(non_snake_case)]` escape hatch, re-exported through two
module boundaries. The `#[allow]` tells the reader "this is wrong and we know it".

**Remediation:**
1. Rename `calculate_next_attempt` → `next_attempt_at_ms` and take `now_ms: u64` as a parameter
   rather than reading the clock. The function becomes pure, the tolerance windows disappear, and
   `result1..result10` become `attempt_1_at`, etc. Effort S.
2. `isAtMaxDelay` → `is_at_max_delay`, drop the `#[allow]`, and either inline it (it is one `>=`) or
   keep it as a named predicate. Effort S.

---

## F-28 — `RegistrationStateInfo` — the suffix restates the field it contains, and that field is a `String`

**Severity: MEDIUM** · **Blast radius: 46 occurrences, 2 definitions, 1 FFI-bound method**

**Locations (byte-identical field sets):**
- `core/src/lib.rs:148`
- `core/src/store/relay_custody.rs:102`

**Evidence:**

```rust
// core/src/lib.rs:148  (and identically core/src/store/relay_custody.rs:102)
pub struct RegistrationStateInfo {
    pub state: String,
    pub device_id: Option<String>,
    pub seniority_timestamp: Option<u64>,
}
```

Three defects in four lines:

1. **The suffix is pure restatement.** "Registration State Info" contains a field called `state`.
   The name says the same thing twice, which is the `userInfoData` pattern the brief names.
2. **The field is stringly-typed.** The permitted values are `"active"`, `"handover"`,
   `"abandoned"`, `"none"` — visible as string literals at `core/src/store/relay_custody.rs:1547,
   1552, 1557, 1562`. Nothing in the type system connects the three to that set, and a fourth
   state is a runtime string, not a compile error.
3. **The accessor triples the redundancy:** `pub fn get_registration_state_info(&self, ...)` at
   `core/src/store/relay_custody.rs:538`, returning `RegistrationStateInfo`.

**Remediation (design first, then vocabulary):** replace `state: String` with an enum —
`RegistrationState { Active, Handover, Abandoned, None }` — which deletes defect 2 and makes every
downstream `match` exhaustive. Then rename the struct to `RegistrationState` and the getter to
`registration_state()`, which deletes defects 1 and 3 in one step. Delete the duplicate at
`core/src/store/relay_custody.rs:102` and have `core/src/lib.rs` re-export the store type (the
same bridge-shadowing pattern as F-22).

**Note on blast radius:** `RegistrationStateInfo` is *not* a UDL dictionary — I checked
`core/src/api.udl` and it is absent — so this rename does not cross the FFI boundary. Effort S.

---

### The tautology scan (F-28, F-29, F-30, F-31)

The suffix scan (`Data|Info|Object|Value|Item|Thing|Stuff`) produced eleven candidates.
Four survived review as genuine tautologies and are written up individually: **F-28** and
**F-31** are in this remainder, **F-29** is under **RC-4** and **F-30** under **RC-5**,
because a name that means several unrelated things is a collision, not padding. The rest
are recorded in the closing table of this section, with the reason each one stays — the
scan was reasoned about, not pattern-matched.

---

## F-31 — `GroupInfo` — the suffix is padding on a struct of five plain fields

**Severity: LOW** · **Blast radius: 12 uses, `core/src/transport/wifi_direct.rs:49`**

**Evidence:**

```rust
// core/src/transport/wifi_direct.rs:49
pub struct GroupInfo {
    pub group_owner: bool,
    pub group_owner_ip: Option<String>,
    pub client_ips: Vec<String>,
    pub interface_name: String,
    pub port: Option<u16>,
}
```

Every field is a group attribute. "Group Info" contains no information beyond "Group", which is the
brief's `dataObject` pattern exactly. It is also ambiguous: `Group` alone is too generic in a
codebase that also has gossipsub topics, neighbourhood tables and BLE groups.

**Remediation:** `WifiDirectGroup`. 12 uses, no FFI, no serialization. Effort S.

**Note on the `*Info` family generally:** `PeerInfo`, `RelayPeerInfo` and `BleAdapterInfo` were
examined and **kept** — see below. `Info` on a DTO describing a thing is conventional and, in this
codebase, understood. It is padding only when the base noun already fully describes the fields, as
in `GroupInfo`.

---

### Tautology scan: checked and deliberately kept

Recorded so the scan is auditable. Each of these ends in a tautological suffix and **survived**
review.

**Basis for the Count column:** `--basis wide`, the wider corpus defined in
`scripts/measure_uncompiled_counts.py`. This table needs it because several of these names
are used mostly in tests, which the glossary's corpus excludes. Regenerate the whole
column with

    python scripts/measure_uncompiled_counts.py --basis wide

**All 14 figures reproduce exactly on that command**, `envelopeData` included once A-4
counts it. An earlier version of this note listed two that did not — `removeValue` and
`getIdentityInfo`, each off by one — and explained the gap as drift from a tree being edited
underneath the measurement. That was true of the tree this audit was written in and is not
true here: these figures are measured at the commit named in GLOSSARY's header, so there is
nothing to drift. `RelayPeerInfo` 15 was the one figure in this table no command could check;
it is measured now and reproduces.

| Name | Count | Why it stays |
|---|---:|---|
| `envelopeData` / `envelope_data` | counted in **A-4** | Distinguishes the *serialized bytes* (`Vec<u8>`, from `PreparedMessage`) from the parsed `Envelope` struct. It also is a UDL dictionary field (`core/src/api.udl:63`), i.e. a public API name. |
| `completeData` | 35 / 4 files | `BleGattClient.kt:737` — the reassembled buffer, built by concatenating sorted chunks. It is load-bearing precisely because it contrasts with the `chunk`s it is assembled from. |
| `userInfo` | 35 / 3 files | `NotificationManager.swift:100` — `notificationContent.userInfo` is the **UNUserNotificationCenter framework key**. Framework-mandated, not first-party. |
| `rawValue`, `newValue`, `removeValue`, `initialValue` | 51, 19, 48, 21 | Android Compose / SwiftUI framework API (`TextFieldValue.rawValue`, `StateFlow` conventions). Not project vocabulary. |
| `PeerInfo`, `RelayPeerInfo`, `BleAdapterInfo` | 55, 15, 14 | Conventional DTO suffixes naming a *thing*; the base noun is the concept. Contrast `GroupInfo` (F-31), where the base noun fully subsumes the fields. |
| `PeerDiscoveryInfo` | 33 | Names a distinct, richer type (11 fields incl. `transports: Set<String>`) than any other peer DTO. A local variable `discoveryInfo` at `MeshRepository.kt:1736` drops the "Peer" qualifier — a minor inconsistency, not worth a finding. |
| `JSONObject`, `withJSONObject` | 17, 10 | `org.json.JSONObject`; framework type. |
| `publishIdentityInfo`, `getIdentityInfo` | 29, 75 | Verb + noun, not a suffix tautology. |

Candidates that did **not** survive review are written up individually, not listed here: F-28 and
F-31 are in the REMAINDER above, F-29 is under **RC-4** and F-30 under **RC-5**.

### The duplicate type names that no finding covers

**These are not findings.** They are the residue of a repo-wide scan for type names declared
in more than one first-party Rust file, after removing every name a finding already owns.
The scan found 54; 24 are the subject of a finding above, 9 exist only because
`cli/src/api_axum.rs` is uncompiled (now recorded in **RC-2**), and the 21 below are what
neither explains.

**This table replaces `SUMMARY.md` §Appendix A**, which held the same 54 rows including the
33 that findings already owned. It lived in the summary, was a second index of facts
FINDINGS owns, and went stale twice. Its home is here because this is the file that owns
type-duplication evidence, and a name can appear in this table only if no finding owns it —
a structural rule, not a promise to keep it up to date.

Every citation below was re-verified against the repository when this table was written.
Paths are full, not basenames: the old table's bare basenames are what let a citation
resolve to the wrong file in the first place.

**Basis for the Occurrences column:** `--basis wide`, the wider corpus defined in
`scripts/measure_uncompiled_counts.py`. This table needs it because duplicate declarations
turn up in test fixtures and generated bindings as readily as in `src/`. Regenerate with

    python scripts/measure_uncompiled_counts.py --basis wide

**19 of the 20 rows reproduce exactly on that command.** The exception is named rather
than quietly adjusted: `Output` reads **83** in the original scan and **20** when re-measured,
and no corpus or matching rule tested recovers 83 — it is a common English word, and the
original appears to have counted something other than this word.

**One row was removed rather than renumbered.** `Args` was listed here as two definitions
in `cli/src/bin/conn-fanout.rs` and `cli/src/bin/stress-test.rs`. `conn-fanout.rs` does not
exist at this commit, so the name is defined once and is not a duplicate-type row at all. It
is dropped rather than kept with a stale citation — a table of names that are declared more
than once cannot carry a row whose second declaration is gone. The row's evidence is its four
definitions, not its frequency, and it is here because the definitions are duplicated.

| Name | Occurrences | Defs | Definitions | Note |
|---|---:|---:|---|---|
| `IronCore` | 556 | 2 | `core/src/iron_core.rs:167`; `wasm/src/lib.rs:200` | |
| `BlockedIdentity` | 158 | 2 | `core/src/blocked_bridge.rs:16`; `core/src/store/blocked.rs:32` | bridge vs store, the RC-1 shape |
| `MeshSettings` | 153 | 2 | `core/src/settings.rs:12`; `wasm/src/lib.rs:28` | `wasm` crate, not built by CI |
| `Output` | 83 | 4 | `core/src/dspy/modules.rs:17,106,165,218` | per-module, intentional — also in the rejected list |
| `TransportState` | 66 | 2 | `core/src/transport/manager.rs:26`; `wasm/src/transport.rs:20` | struct vs enum, same name; `wasm` crate |
| `RatchetKey` | 60 | 2 | `core/src/crypto/pq/hybrid.rs:30`; `core/src/crypto/ratchet.rs:48` | in `crypto/` — inside the AGENTS.md Rule 8 review gate, not investigated |
| `PeerInfo` | 55 | 4 | `core/src/privacy/circuit.rs:39`; `core/src/routing/local.rs:48`; `core/src/transport/peer_broadcast.rs:25`; `wasm/src/mesh.rs:51` | `wasm` crate |
| `BlockedManager` | 51 | 2 | `core/src/blocked_bridge.rs:52`; `core/src/store/blocked.rs:108` | bridge vs store, the RC-1 shape |
| `MultiPathDelivery` | 49 | 2 | `core/src/routing/multipath.rs:67`; `core/src/transport/mesh_routing.rs:401` | |
| `MeshSettingsManager` | 47 | 2 | `core/src/mobile_bridge.rs:3034`; `wasm/src/lib.rs:107` | `wasm` crate |
| `StoredEnvelope` | 44 | 2 | `core/src/drift/store.rs:17`; `core/src/relay/server.rs:37` | |
| `DeviceState` | 33 | 2 | `core/src/drift/policy.rs:8`; `core/src/mobile_bridge.rs:109` | |
| `BootstrapManager` | 30 | 2 | `core/src/relay/bootstrap.rs:185`; `core/src/transport/bootstrap.rs:101` | two files with the same basename — a likely layering overlap, flagged for the owner |
| `DropReason` | 27 | 2 | `core/src/drift/relay.rs:73`; `core/src/transport/ble/l2cap.rs:354` | |
| `DiscoveredPeer` | 26 | 2 | `cli/src/api.rs:203`; `core/src/transport/wifi_aware.rs:143` | |
| `RelayStats` | 22 | 3 | `core/src/transport/internet.rs:121`; `core/src/transport/mesh_routing.rs:114`; `wasm/src/mesh.rs:60` | `wasm` crate |
| `Input` | 17 | 4 | `core/src/dspy/modules.rs:16,105,164,217` | per-module, intentional — also in the rejected list |
| `ScanResult` | 15 | 2 | `core/src/store/backend.rs:6`; `core/src/transport/ble/scanner.rs:183` | a `type` alias against a `struct` — not the same kind of thing under one name |
| `BleAdapterInfo` | 14 | 2 | `cli/src/ble_daemon.rs:211`; `desktop_bridge/src/types.rs:123` | |
| `LegacyReceivedMessage` | 4 | 2 | `core/src/store/inbox.rs:14`; `core/src/store/inbox.rs:551` | same file, two shapes, worth a look |

**The two that most likely deserve a finding, and why neither is written up here:**
`BlockedIdentity` and `BlockedManager` are the RC-1 shape again (a `*_bridge` type beside
its `store/` original) in a subsystem this audit did not otherwise open, so a claim about
their divergence needs evidence I did not gather. `ScanResult` is a `type` alias in one file
and a `struct` in the other — a stronger defect than a plain duplicate, and equally
unexamined. Both are offered as the next place to look, not as findings.

---

## RC-5 — A concept has several names and no recorded canonical choice, so each new call site picks one

**Members, in rank order:** F-26, F-23, F-30, F-11, F-24

**The cause, stated once here.** A concept needed a name, the choice was never recorded,
and the next call site made its own. The counts below are not a list of synonyms to
delete — they are the shape of a decision that was deferred: `get`/`fetch`/`load`,
`config`/`settings`/`cfg`, `peer`/`contact`/`node` are each one concept with a dominant
form and a long tail, and the long tail grows with every new file.

**Nothing here is a code change.** Each member's fix is one decision, and each decision
already has an owner in `GLOSSARY.md` (A-1, B-1, §D and the rest), which holds the
canonical name, the alias list, the rename plan, the risk and the status. These findings
supply the evidence; the glossary does not repeat it, and they do not repeat the
decision.

**This section is last** because its highest-ranked finding is MEDIUM. Severity is the
primary sort, so nothing above it may be lower than MEDIUM; the REMAINDER that precedes
it contains three HIGH findings and therefore outranks everything here.

---

## F-26 — "the other party" is called peer, contact, node, or relay, sometimes in one expression

**Severity: MEDIUM** · **Blast radius: 3605 `peer`, 1409 `relay`, 1347 `contact`, 632 `node`**

```
 3605  161 files  peer
 1409   96 files  relay
 1347   64 files  contact
  632   65 files  node
```

Restated onto the basis GLOSSARY states; regenerate with
`python scripts/measure_uncompiled_counts.py`. What these were and why they moved is in
GLOSSARY A-1. **The finding is unchanged, but a ranking claim here did not survive the
restatement:** `peer` still leads by a factor of two and a half, and `relay` (1409) and
`contact` (1347) are 62 occurrences apart — a tie whose order flips once compound
identifiers are counted. All four figures are bare words; GLOSSARY A-1 carries the compound
counts.

The clearest evidence is inside a single Rust type: `core/src/store/contacts.rs:25` defines
`struct Contact { pub peer_id: String, ... }` and every method on `ContactManager` takes
`peer_id: String` (`add`, `get`, `remove`, `set_nickname`, `update_last_seen`, …). The type is a
contact; the key is a peer. These are the same entity — the same test fixture
(`contacts.rs:1394`) uses `peer-old` as a contact id.

`node` and `relay` are a second, project-specific axis. AGENTS.md settles it: there are no
standalone relays, `relay` is a *verb* for store-and-forward custody, and code identifiers may keep
historical names. I found the identifiers (`RelayCustodyStore`, `cmd_relay`, `relay_custody_msg_`)
and did not flag them, per the exception AGENTS.md grants. What I did find is `isRelay` (F-13) —
a *data* name for a *role* that no longer exists, which is outside that exception.

**Remediation:** `Contact` is the entity; `PeerId` is its key. That is already the dominant pattern
and should be stated as the rule, with `Node` reserved for the local process. Effort S (document);
M (any rename), and no rename is actually required for correctness.

---

## F-23 — Read verbs: `get` / `fetch` / `load` / `retrieve` / `poll` with no rule

**Severity: MEDIUM** · **Blast radius: 655 `get`, 207 `load`, 20 `fetch`, 8 `retrieve`**

```
   655  112 files  get
   496   76 files  read
   374   81 files  list
   207   37 files  load
    82   26 files  resolve
   133   40 files  find
    25   11 files  poll
    20   10 files  fetch
     8    5 files  retrieve
```

Counted by `python scripts/measure_uncompiled_counts.py` on the basis GLOSSARY states.
These are far smaller than the figures this finding originally carried (5470 / 1961 /
1689 / 731 / 579 / 352 / 164 / 121 / 22) because the basis narrowed to first-party code
and became case-sensitive; see GLOSSARY B-1 for the full comparison. **The finding is
unaffected — `get` still outnumbers every other read verb, and `retrieve` at 8 is still
small enough that retiring it is an afternoon.** What the old numbers implied, that this
was a 5,000-call-site problem, was never true of the compiled code.

There is no rule. `wasm/src/storage.rs:147 get_messages_for_hint` and
`core/src/wasm_support/mesh.rs:164 get_messages_for_hint` share a name and not a signature
(`hint: &str` against `hint: [u8; 4]`) — and, verified against the module graph, both sit
in files the build never compiles (**RC-2**), so the collision is invisible to the
compiler in both directions. `cli/src/api.rs:400 get_history_via_api` is the only
`get_history_via_*`. `wasm/src/lib.rs:1184 get_history_manager` returns a *manager*, not a history.
`handle_fetch_artifact` (F-11) is the only `fetch_` in compiled code; the uncompiled
`cli/src/api_axum.rs:746` declares a second copy of the same route (F-32).

**Remediation — a small, enforceable rule, not a mass rename:**
- `get_*` — returns a value the process already holds (pure, in-memory)
- `query_*` / `list_*` — returns a collection, possibly filtered
- `fetch_*` — crosses a process or network boundary to retrieve bytes
- `load_*` — reads persisted state into memory (cold path)
- delete `retrieve` entirely (8 occurrences on this basis; it was published as 22) in
  favour of `get` or `fetch`

Enforce with a `//! naming:` line in each module or a clippy lint if one is available. Effort S
for the rule, M for the rename.

---

## F-30 — one concept, three names: `IdentityInfo` / `identityInfo` / `identityData`

**Severity: MEDIUM** · **Blast radius: ~94 occurrences (73 + 79 + 12 combined with overlap), 22 + 18 + 3 files**

**Locations:**
- `core/src/api.udl:44` and `core/src/lib.rs:115` — `IdentityInfo`, the UDL dictionary (structured:
  `identity_id`, `public_key_hex`, `device_id`, `seniority_timestamp`, `initialized`, `nickname`,
  `libp2p_peer_id`)
- `android/.../data/MeshRepository.kt:598-599` — `identityInfo: StateFlow<uniffi.api.IdentityInfo?>`,
  the Kotlin property wrapping it
- `android/.../data/MeshRepository.kt:3341` — `identityData`, a **`ByteArray` of JSON**

**Evidence:**

```kotlin
// android/.../data/MeshRepository.kt:598
private val _identityInfo = MutableStateFlow<uniffi.api.IdentityInfo?>(readCachedIdentityFields())
// android/.../data/MeshRepository.kt:3341
private var identityData: ByteArray = "{}".toByteArray()
// android/.../data/MeshRepository.kt:3440
identityData = beaconJson // Store for immediate use by GATT server
```

`identityInfo` is a typed snapshot; `identityData` is an untyped JSON string served to a GATT
client. Both are "this node's identity as broadcast to peers". Three names, two shapes, and the
third name is the one that lost its type.

**Remediation:** the property-wrapping-a-UDL-type pattern (`_identityInfo` / `identityInfo`) is
idiomatic and should stay. `identityData` is the defect — it is a `ByteArray` defaulting to
`"{}"`, so an unparseable or empty payload is indistinguishable from a valid one at the type level.
Rename to `identityBeaconJson` (naming what it is) and, better, replace it with the typed
`DriftEnvelope`/identity-envelope type that `core/src/message/identity_envelope.rs` already provides.
Effort S for the rename, M for the typing.

---

## F-11 — The `scm` Control API carries two route families with two different verb vocabularies

**Severity: MEDIUM** · **Blast radius: 17 routes, one router, one port**

**Locations:** `cli/src/api.rs:1641-1670`

```rust
.route("/api/identity",            get(handle_get_identity))
.route("/api/peer-resolve",        get(handle_peer_resolve))
.route("/api/send",                post(handle_send_message))
.route("/api/peers",               get(handle_get_peers))
.route("/api/listeners",           get(handle_get_listeners))
.route("/api/external-address",    get(handle_get_external_address))
.route("/api/diagnostics",         get(handle_export_diagnostics))
.route("/api/drift-status",        get(handle_get_drift_status))
.route("/api/discovery/scan",      post(handle_trigger_discovery_scan))
.route("/submit-run",              post(handle_submit_run))     // <-- no /api prefix
.route("/poll-status/:run_id",     get(handle_poll_status))     // <-- no /api prefix
.route("/fetch-artifact/:run_id/:name", get(handle_fetch_artifact))  // <-- no /api prefix
```

Two problems:

1. **Two prefixes.** Fourteen routes under `/api/`, three at the root. A client cannot construct a
   base URL from the namespace.
2. **Two verb vocabularies.** The mesh family uses `get_` / `trigger_` / `export_`. The run-control
   family uses `submit_` / `poll_` / `fetch_` — and `fetch_` is the only `fetch_` in the
   *compiled* tree (the uncompiled `cli/src/api_axum.rs:746` declares the same route again; F-32)
   repository (121 total occurrences of the atom, 29 files, and this handler is the outlier).

The second family is also a *different product surface*: `/submit-run`, `/poll-status`,
`/fetch-artifact` are the dspy/Ollama agent-run API (`core/src/dspy/`,
`core/src/wasm_support/rpc.rs`), mounted on the same router and port as the mesh control API. Two
unrelated APIs sharing one HTTP surface with one undifferentiated naming scheme.

**Remediation:** move the run-control routes under `/api/runs/*` (`/api/runs/submit`,
`/api/runs/:run_id`, `/api/runs/:run_id/artifacts/:name`) and settle the handler-name convention
(`get_`/`post_` matching the verb, or drop the verb entirely and let the path carry it).
**Compatibility:** the three root routes have no `/api/` prefix, so they are externally reachable
and may be in use — see the URL-rename compatibility row in GLOSSARY. Effort S (path only) to
M (if clients need aliases).

---

## F-24 — Abbreviations: `cfg`/`config`/`settings`, `msg`/`message`, `addr`/`address`

**Severity: LOW–MEDIUM** · **Blast radius: broad but low-harm individually**

| Cluster | Counts (restated — GLOSSARY §D owns the basis and the previous figures) | Assessment |
|---|---|---|
| config | `config` 998/55 files, `settings` 472/48, `cfg` **42/14 files** (27/9 comment-excluded) (corrected 2026-09-29 — the earlier figure of 667/170 counted Rust's `#[cfg]` attribute, and a further 4 of the 31 real hits are in the uncompiled `cli/src/api_axum.rs`; see GLOSSARY §D and F-32), `prefs` 74/5, `params` 98/6, `options` 47/14 | **Real problem.** `cfg` has *more files* than `config` despite a third of the occurrences — it is the local-idiom form and the split is structural. `cfg` vs `config` should be decided; `settings` vs `config` is a genuine semantic distinction worth keeping (see Q-7) |
| message | `message` 1742/177, `text` 619/71, `msg` 756/54, `body` 158/37 | **Real problem** at the FFI/wire boundary. `msg` appears in 128 files; `text` and `body` mean different things (see Q-8) |
| address | `addr` 1022/39, `address` 829/59, `multiaddr` 823/34, `url` 121, `host` 214, `endpoint` 87 | `multiaddr` is a libp2p term of art and correctly distinct. `addr` vs `address` is a coin-flip per author |
| auth | `session` 298, `token` 168, `authorization` 1, `authentication` 26, `auth` **6** | `auth` is used only 6 times and is a UniFFI/ODR keyword echo — leave it |

**Remediation:** pick `config` and `message`; migrate `cfg` → `config` and `msg` → `message` in
identifiers only, never in serialized keys (see GLOSSARY §D for the canonical names and §F for
compatibility). Counts: GLOSSARY §D owns them. Effort: `msg` → `message` is **M**
(756 hits, 54 files, scriptable); `cfg` → `config` is **S** (42 hits, 14 files — 27 in 9
once comments are excluded).

---

# Findings I considered and rejected

Kept out of the findings above to preserve signal; listed so the owner knows they were checked:

- **`result1/result2/result3/result10`** in `core/src/routing/smart_retry.rs:142-154` — real numeric
  suffixes by copy-paste, but they are test-local and are dissolved by F-10's fix. Not ranked separately.
- **`doInsert`** (`iOS/.../Generated/api.swift:373`) and **`performMaintenance`**
  (`api.swift:1986`) — generated bindings. Out of scope.
- **`Output` / `Input` defined 4× each** in `core/src/dspy/modules.rs` — one per DSPy module
  (`lines 17, 105, 164, 217`). Structurally correct for that pattern; the names are generic but the
  modules disambiguate. Low value.
- **`handle_*` in `cli/src/api.rs` (23 handlers)** — this is the axum handler convention, applied
  consistently. Not a catch-all-naming problem.
- **`serviceInfo` in Android (98 occurrences, 8 files)** — **WITHDRAWN, now F-29.** The original
  rejection claimed "the surrounding types disambiguate at every call site I sampled". Reading the
  call sites showed the third use (`WifiAwareTransport.kt:51`) is a raw `ByteArray` hand-decoded as
  TLV, which the other two are not. The rejection was wrong.
- **`X2` in `android/.../utils/PeerIdValidator.kt:135`** (5 occurrences) — appears to be a
  deliberate test-fixture naming pair. Not a divergence risk.
- **9× `current_timestamp`** is one finding (F-01), not nine.
