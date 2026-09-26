# Codebase unification plan -- stop the two-copies-one-fixed pattern

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO
Last updated: 2026-07-26
Scope: cross-cutting duplication audit, not a single feature. Ledger-specific
items are tracked in `HANDOFF/todo/LEDGER_CHOKE_POINT_REFACTOR.md` -- this
file cross-references rather than repeats that scope.
Priority: HIGH (three consecutive adversarial BLOCK verdicts trace to this
failure class)

## Why this exists

Every BLOCK verdict this cycle had the same shape: a concept implemented at
N sites, a fix applied at one. This file is a dispatchable, ranked list of
every other instance found by direct grep/read against current HEAD
(`6761ac4`, 2026-07-26). Each item states which copy should survive, the
mechanism that makes recreating the duplicate a compile error (not a
reminder), and the gate that proves it landed. Ranking is by "how likely is
someone to fix one and miss the other," not by tidiness.

## How to read the verification tags

- **CONFIRMED** -- read at current HEAD, file:line cited, still true.
- **ALREADY FIXED** -- was true historically (per operator brief or prior
  HANDOFF notes) but current HEAD shows it resolved; kept here as a record
  so nobody re-opens it.
- **PARTIALLY FIXED** -- a shared primitive now exists, but not all call
  sites were migrated to it, so the duplication risk persists in practice.
- **NOT REPRODUCIBLE** -- could not confirm at current HEAD; likely a stale
  AI-generated lead from an earlier audit pass.

---

## Rank 1 -- security predicates enforced at N sites, no compiler tie

### 1a. DNS/SSRF gate: `cmd_relay` has it, `cmd_start` does not
CONFIRMED. `cli/src/main.rs:2996-3000` (`cmd_relay`'s `PeerIdentified`
handler) calls `ledger::is_dialable_multiaddr(..., DnsPolicy::Reject)`
before `record_connection`. The structurally identical handler in
`cmd_start`, `cli/src/main.rs:2034-2038`, calls
`l.record_connection(&addr.to_string(), ...)` directly -- no gate, and then
immediately calls `l.to_shared_entries()` (see 1b) on the unfiltered
result. This is item 1 of `LEDGER_CHOKE_POINT_REFACTOR.md`'s "why this
exists" table; that file's fix (move the gate inside
`LedgerStore::record_connection` as a required parameter, closing the
choke point) is the correct mechanism -- do not patch `cmd_start` inline,
that only restores 2-of-2 until the next N-way handler appears.
**Survivor / mechanism:** `LEDGER_CHOKE_POINT_REFACTOR.md` section 1.
**Gate:** that file's proposed CLI test -- a `PeerIdentified` with a DNS
`listen_addr` must not reach `dialable_addresses()`, exercised through
`cmd_start`, not just `cmd_relay`.

### 1b. Disclosure filter on the ledger-exchange REQUEST path
CONFIRMED. `LedgerStore::to_shared_entries()` (`cli/src/ledger.rs:463-480`)
has no address filter, no cap, and copies `known_topics` verbatim
(line 477) -- the exact field `ledger_entry_to_shared_routing_only`
(`core/src/store/ledger_entry.rs:621-625`) deliberately blanks for the
same protocol's response direction. It is called from three sites:
`cli/src/main.rs:1866`, `:2040`, `:2894` (all `PeerIdentified` handlers
across `cmd_start`/`cmd_relay`). Already tracked as item 2 of
`LEDGER_CHOKE_POINT_REFACTOR.md`. **Do not duplicate that file's fix here**
-- cross-reference only.
**Survivor:** core's `exchange_response_entries` /
`ledger_entry_to_shared_routing_only`, per that file's section 2.

### 1c. Bootstrap-node dedup uses a third, unsound `strip_peer_id`
CONFIRMED, not previously flagged. There are now **three** independent
`/p2p/`-suffix-stripping implementations for the same conceptual
operation:
- `core/src/transport/addr_filter.rs:290` `strip_peer_id` -- the canonical
  one, handles circuit-relay addresses correctly (see its tests at
  `addr_filter.rs:677-711`).
- `core/src/store/ledger_entry.rs:79-81` `strip_peer_id_component` --
  correctly delegates to the canonical one (this is the "good" pattern;
  no action needed, listed only so it isn't miscounted as a duplicate).
- `cli/src/config.rs:279-285` `Config::strip_peer_id` -- an **independent
  reimplementation**: `multiaddr.find("/p2p/")` with no circuit-relay
  awareness, used by `add_bootstrap_node` (`config.rs:288-298`) to dedup
  saved bootstrap entries by IP:Port. This is the exact naive-strip shape
  `LEDGER_CHOKE_POINT_REFACTOR.md` flags as "NEW-7 residual" (F8) in
  `relay/invite.rs:134-164` -- same bug pattern, third location.
**Risk:** lower than 1a/1b (config-file dedup, not a wire-trust boundary),
but it means F8's fix, if applied only where NEW-7 points, will still miss
this site.
**Survivor / mechanism:** delete `Config::strip_peer_id`; import
`scmessenger_core::transport::addr_filter::strip_peer_id` the same way
`cli/src/ledger.rs` already does (the model case cited in the operator
brief). No `pub(crate)` re-export needed -- `addr_filter` functions are
already `pub`.
**Gate:** a test asserting `Config::add_bootstrap_node` correctly dedups a
circuit-relay address that differs only in the relay hop (the case the
naive strip gets wrong); `cargo test -p scmessenger-cli`.

---

## Rank 2 -- one shared constant now exists and is still not used

### 2. Gossipsub topic strings: three definitions, ~12 hardcoded call sites
CONFIRMED, and worse than the 2026-07-12 audit (`UNIFICATION_AUDIT_FINDINGS.md`
item 2) recorded, which said "no single shared constant exists anywhere."
One now exists -- but nothing uses it:
- `core/src/lib.rs:262-263` defines `TOPIC_LOBBY`/`TOPIC_MESH` (added since
  the last audit). **Zero references** to either constant anywhere in
  `core/src` outside their own definition -- `core/src/transport/swarm.rs`
  still hardcodes the literal strings at lines 2425, 2426, 2539, 2540,
  3007, 5652, 5653, 5752, 5753 (9 sites in one file).
- `cli/src/bootstrap.rs:30,33` defines a **second**, differently-named pair
  (`LOBBY_TOPIC`/`MESH_TOPIC`), used only within that file
  (`bootstrap.rs:131`) -- does not import or reference `core::TOPIC_LOBBY`.
- `cli/src/main.rs:1626` and `:2691` still hardcode the raw array literal
  `["sc-lobby", "sc-mesh"]` directly, using neither constant.
Three names, one string pair, and the "fix" (defining a constant) landed
without the follow-through of pointing any call site at it -- this is the
clearest example in the codebase of the failure mode this whole plan
exists to close: partial application looks identical to no application
from the perspective of the next person who changes the topic name.
**Survivor:** `core::TOPIC_LOBBY` / `core::TOPIC_MESH` (core is the crate
every platform depends on; CLI and swarm.rs should import from there, not
define their own).
**Mechanism:** delete `cli/src/bootstrap.rs`'s `LOBBY_TOPIC`/`MESH_TOPIC`
and re-point `bootstrap.rs:131` at `scmessenger_core::TOPIC_LOBBY` /
`TOPIC_MESH`; replace all 9 `swarm.rs` literals and both `main.rs` array
literals with the same import. Once nothing else defines the string, a
`rg '"sc-lobby"|"sc-mesh"'` outside `core/src/lib.rs` and test modules
returns empty -- make that the gate, not a promise to grep before merging.
**Gate:** `rg '"sc-(lobby|mesh)"' --glob '!*test*'` returns only
`core/src/lib.rs`; `cargo build --workspace`.

### 3. Three env-var names for one bootstrap-nodes concept
CONFIRMED, exactly as described. `cli/src/bootstrap.rs:38` is the **only**
site that actually reads an env var: `std::env::var("SC_BOOTSTRAP_NODES")`.
- `docker/docker-compose.yml:33,49` sets `BOOTSTRAP_NODES=...` -- read by
  nothing; silent no-op for anyone deploying via that compose file.
- `docker/Dockerfile:7-8` declares `ARG SCMESSENGER_BOOTSTRAP_NODES` and
  re-exports it as `ENV SCMESSENGER_BOOTSTRAP_NODES` -- also read by
  nothing at runtime (`core/src/iron_core.rs:3369`'s doc comment names a
  fourth concept, the hardcoded `CORE_BOOTSTRAP_NODES` Rust const at
  `core/src/transport/bootstrap.rs:28`, which is `&[]` -- empty by design,
  not related to any env var).
**Survivor:** `SC_BOOTSTRAP_NODES` (the one live reader).
**Mechanism:** this is a config-surface problem, not a type-system one --
the compiler cannot catch an unused Docker `ARG`. Fix by deletion: remove
`BOOTSTRAP_NODES` from both `docker-compose.yml` occurrences and
`SCMESSENGER_BOOTSTRAP_NODES` from `Dockerfile`, replace with
`SC_BOOTSTRAP_NODES` in both, and grep the rest of `docker/*.yml` (7 files
matched `BOOTSTRAP_NODES` in this sweep; not all individually re-verified
here -- re-grep before editing) for the same stale name.
**Gate:** `docker compose config` (or equivalent) shows `SC_BOOTSTRAP_NODES`
reaching the container env; manual smoke test that a compose-launched node
picks up a configured bootstrap peer.

---

## Rank 3 -- dead modules shadowing live ones by name

### 4. Two `Commands` enums; the binary only sees one
CONFIRMED. `cli/src/main.rs:191` defines `enum Commands` and is the one the
binary target parses CLI args with (`main.rs` has no `mod cli;` -- the
binary crate never compiles `cli.rs` at all). `cli/src/cli.rs:160` defines
a **second, divergent** `pub enum Commands`, reachable only via `pub mod
cli;` in `cli/src/lib.rs:11` -- the library target. No production code
anywhere calls `cli::Commands` (`rg 'cli::Commands|use crate::cli::'`
returns nothing outside `cli.rs` itself). The operator's claim that
`cli.rs`'s enum is "missing `audit`" was not individually re-diffed
variant-by-variant in this pass, but the structural finding -- two enums,
one dead -- is confirmed.
**Survivor:** `main.rs`'s `Commands` (the one actually parsing argv).
**Mechanism:** either delete `cli/src/cli.rs` and its `pub mod cli;`
declaration if nothing external depends on the library target's copy, or
-- if some downstream (Android JNI shim, integration test) does construct
`cli::Commands` -- make `main.rs` `use crate::cli::Commands;` instead of
redefining it, so there is exactly one definition regardless of which
target compiles it. Check `android/` and `core/tests/` for
`scmessenger_cli::cli::` references before deleting.
**Gate:** `cargo build --workspace` plus `rg 'enum Commands'` returns one
result.

### 5. Two relay bootstrap modules, one genuinely dead, one partially fed
CONFIRMED with a correction to the operator's framing. There are actually
**three** files named `bootstrap.rs`: `core/src/transport/bootstrap.rs`,
`core/src/relay/bootstrap.rs`, and `cli/src/bootstrap.rs` (the last is
live -- it owns `SC_BOOTSTRAP_NODES` reading and the topic-list helper from
item 2 -- not part of this finding).
- `core/src/relay/bootstrap.rs` -- **fully dead**. Its `InvitePayload`
  (line 61), `BootstrapMethod`, `SeedPeer` etc. have zero references
  anywhere outside the file itself (`rg 'relay::bootstrap|bootstrap::InvitePayload|bootstrap::SeedPeer'`
  in `core/src` returns nothing). It also contains, per its own comment at
  line 126, the residue of a since-removed fake
  `pq_sig == b"TAMPERED"` check (ALREADY FIXED -- the comment is
  documenting the removal, not describing live code; confirm no
  regression before closing this line item).
- `core/src/transport/bootstrap.rs` -- **not dead, but under-fed**.
  `BootstrapManager` is constructed via `with_defaults()` in
  `core/src/iron_core.rs:381,472,563` and wired to public `IronCore`
  methods (`get_fallback_relays`, `get_healthy_relays`,
  `get_all_relay_stats` at `iron_core.rs:3358-3400`). But its data source,
  `CORE_BOOTSTRAP_NODES` (`transport/bootstrap.rs:28`), is a hardcoded
  `&[]`, and `iron_core.rs:3355`'s own doc comment says "no live swarm
  wiring feeds this yet" for the stats half. So: live plumbing, no live
  data -- different failure mode than `relay/bootstrap.rs`, but the same
  symptom (a module that looks load-bearing and mostly is not).
**Survivor:** `core::relay::invite::InviteToken` for invites (already the
operator-confirmed live path); `core::transport::bootstrap::BootstrapManager`
for the relay-health/fallback-address role, once fed real data.
**Mechanism:** delete `core/src/relay/bootstrap.rs` and its `pub mod
bootstrap;` in `relay/mod.rs:7` outright -- nothing depends on it. For
`transport/bootstrap.rs`, either wire real swarm events into it (tracked
separately -- out of scope for a unification pass) or rename/doc-comment
it unambiguously as "address-list only, health tracking not live" so
nobody assumes `get_all_relay_stats()` reflects reality.
**Gate:** `cargo build --workspace` after deleting `relay/bootstrap.rs`;
`rg 'relay::bootstrap'` returns nothing.

### 6. BLE GATT UUIDs duplicated verbatim across the Windows-specific and cross-platform implementations
CONFIRMED, and flagged per the operator's "call out legitimate cfg-gated
separation" instruction -- this one is **partially** legitimate and
partially not. `cli/src/ble_windows.rs:21-23` (`#![cfg(target_os =
"windows")]`, a real platform-specific WinRT BLE implementation) and
`cli/src/ble_mesh.rs:43` (compiled on all platforms, presumably
btleplug-backed) both define `GATT_SERVICE_UUID = 0x0000_DF01_...` (and
`IDENTITY_CHAR_UUID`, `MESSAGE_CHAR_UUID`) as identical literals. The two
*implementations* are legitimately separate (different BLE stacks per
platform -- do not merge the code). But the UUID **values** are not
platform-specific -- they are the wire protocol's BLE service identity,
and if a future change updates one file's literal without the other, BLE
devices running the two backends stop finding each other over mesh
discovery. This is a value that must be shared even though the code
around it must not be.
**Survivor:** the three UUID constants only, hoisted to one place.
**Mechanism:** move `GATT_SERVICE_UUID`/`IDENTITY_CHAR_UUID`/
`MESSAGE_CHAR_UUID` into a small shared module (e.g. `cli/src/ble_ids.rs`,
no `cfg` gate) and have both `ble_windows.rs` and `ble_mesh.rs` import
from it. This makes the platform split explicit (implementations differ,
identity constants do not) rather than implicit (two files that happen to
agree today).
**Gate:** `cargo build --workspace` on a config that compiles both
(`--target x86_64-pc-windows-msvc` covers `ble_windows.rs`; any target
covers `ble_mesh.rs`); a `const_eq` style compile-time assertion is
unnecessary once there is only one definition.

---

## Rank 4 -- two `LedgerManager` instances over the same file

CONFIRMED. `core/src/iron_core.rs:169` holds `IronCore.ledger_manager:
crate::store::LedgerManager`, hydrated from `storage_path` at
`iron_core.rs:458` (`hydrated_ledger_manager(p)`, same `p` used to build
the rest of `IronCore`'s storage). Independently,
`android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:911`
constructs `uniffi.api.LedgerManager(storagePath)` at the same
`storagePath`, then calls `.load()` at `MeshRepository.kt:919`. Both
wrap the same on-disk file with whole-file `save()`/`load()` semantics
(`core/src/store/ledger_entry.rs:143-172` `load`/`save_with_entries`/
`save`) -- last writer wins if both are mutated in the same process
lifetime. iOS has the mirrored construction at
`iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:623` per the
operator brief (not independently re-read in this pass; same shape
expected, verify before closing).
**Survivor:** `IronCore.ledger_manager` -- the client should never own a
second handle to the same file.
**Mechanism:** this needs a UniFFI-exported accessor
(`IronCore::ledger_manager_handle()` or equivalent, mirroring the pattern
already used for `relay_bootstrap_manager_handle()` at
`iron_core.rs:3346-3350`) so `MeshRepository.kt`/`.swift` read/write
through the core's single instance instead of instantiating their own.
This is the mobile-client half of the same choke-point principle
`LEDGER_CHOKE_POINT_REFACTOR.md` applies to the CLI/wire side -- cross-
reference, do not re-plan the wire-side portion here.
**Gate:** Android/iOS integration test writing a ledger entry through one
handle and reading it back through `IronCore`'s own state (not a second
file read) without a `.load()` round-trip in between.

---

## Rank 5 -- serialization: checked, mostly clean, one loose end

### 7. `encode_receipt`/`decode_receipt` -- ALREADY FIXED, one bypass remains
ALREADY FIXED at the core layer: `core/src/iron_core.rs:2821-2847`
(UniFFI-exported, `#[cfg(not(target_arch = "wasm32"))]`) is a thin wrapper
delegating to the true implementation,
`core/src/message/types.rs:231-241`, which is `serde_json`-based (not
bincode) and explicitly documented as "the ONLY way receipts should be
serialized anywhere in the codebase." `Message::receipt()`
(`types.rs:175-192`) also calls this same `encode_receipt` -- the
"separate core function that bincode-serializes a Receipt" lead from
`UNIFICATION_AUDIT_FINDINGS.md`'s Qwen pass is **NOT REPRODUCIBLE** at
current HEAD; `rg bincode` against `cli/src` returns zero matches
entirely, so the "CLI bincode-decodes JSON" claim is also **NOT
REPRODUCIBLE**.
One loose end: `cli/src/main.rs:2136` decodes an incoming receipt with
`serde_json::from_slice::<scmessenger_core::Receipt>(&msg.payload)`
directly, bypassing the exported `decode_receipt`. Behaviorally identical
today (both are `serde_json::from_slice` under the hood) but it means a
future change to `decode_receipt` (e.g. adding a version tag or migrating
off JSON) silently does not apply to this call site.
**Survivor:** `scmessenger_core::decode_receipt`.
**Mechanism:** swap `main.rs:2136`'s direct `serde_json::from_slice` call
for `scmessenger_core::decode_receipt(msg.payload.clone())`. Trivial diff,
low priority -- listed for completeness, not urgency.
**Gate:** `cargo build -p scmessenger-cli`; existing receipt-delivery
integration tests still pass.

---

## Checked and found NOT to be duplicates (do not "unify" these)

- **`transport::reputation::AbuseReputationManager` vs
  `abuse::reputation::EnhancedAbuseReputationManager`**
  (`core/src/transport/reputation.rs:181`,
  `core/src/abuse/reputation.rs:14`) -- legitimate composition, not a
  parallel hierarchy: `EnhancedAbuseReputationManager` holds a
  `base_manager: AbuseReputationManager` field
  (`abuse/reputation.rs:15,22,35`) and delegates to it. `IronCore` uses
  only the `Enhanced` wrapper. No action.
- **`ble_windows.rs` vs `ble_mesh.rs` BLE implementations themselves**
  (not the UUID constants, which are item 6 above) -- legitimately
  separate: one is `cfg(target_os = "windows")`-gated WinRT, the other is
  the cross-platform btleplug path. Do not merge the connection-handling
  code.
- **`core/src/store/ledger_entry.rs::strip_peer_id_component`** -- already
  delegates to `addr_filter::strip_peer_id`; this is the target pattern
  for item 1c, not a violation of it.
- **`.gitattributes` LF scope (`*.rs`/`Dockerfile*`/`*.swift`/`*.sh`
  only, no `*.md`/`*.yml`/`*.toml`)** -- confirmed as stated in the
  operator brief (`.gitattributes` has exactly those four rules). This is
  a policy gap, not a code duplication, and is lower priority than
  everything above -- CI already surfaces it as a `git diff --check`
  failure when it bites, which is a working (if annoying) gate. Consider
  adding `*.md text eol=lf` / `*.yml text eol=lf` / `*.toml text eol=lf`
  as a cheap follow-up, not part of this plan's ranked work.

---

## Suggested dispatch order

1c, 1a/1b (cross-ref only, do not re-plan) -> 2 -> 3 -> 4 -> 5 -> 6 -> 7.
Rank 1 and the ledger-choke-point items share a root cause (security
predicates with no single enforcement point) and should land together or
back-to-back so the fourth adversarial review (required per
`LEDGER_CHOKE_POINT_REFACTOR.md`'s closing note) can cover both in one
pass instead of two.
