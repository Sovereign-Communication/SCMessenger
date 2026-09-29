# Naming Audit — Summary

**Repository:** SCMessenger — `/c/Users/SCM/Documents/GitHub/SCMessenger`
**Branch:** `glm/canonical-outlier-audit`
**Date:** 2026-09-29
**Mode:** READ-ONLY. No source file was renamed, moved, refactored, or modified. The only files
written are the three in `naming-audit/`.

**Report directory:** `./naming-audit/`
- `FINDINGS.md` — **32 active findings** (F-01 to F-33, with **F-25 retired** and its content
  absorbed into F-01), organised by **five root causes plus an explicit remainder** rather
  than by the brief's Layer 1 / Layer 2 taxonomy; see its **Contents** table
- `GLOSSARY.md` — 26 proposed canonical entries (incl. §D-2 jargon, 2 needing definitions) + a
  FROZEN-identifier compatibility table
- `SUMMARY.md` — this file

**How this report is organised:** each fact has exactly one owner. A **finding** owns its evidence
and its design fix; the **glossary** owns the canonical name, the alias list, the rename plan, the
risk and the compatibility approach; **§F** is the single register of every compatibility approach;
and **§2 below** is an index, not a second copy. Each owning section says so in place, so editing
one fact means editing one place.

---

## 1. What I covered

### First-party source, in full

| Layer | Path | Volume |
|---|---|---|
| Rust core | `core/src/**` | 180 files, ~94,300 LOC across 21 modules |
| Rust CLI | `cli/src/**` | 17 files, ~15,700 LOC |
| Rust WASM bridge | `wasm/src/**` | 8 files |
| Rust desktop bridge | `desktop_bridge/src/**` | 10 files |
| Android | `android/app/src/main/**`, `android/shared/**` | 175 `.kt` files, ~45,400 LOC |
| iOS | `iOS/SCMessenger/SCMessenger/**` (excl. `Generated/`) | 49 `.swift` files |
| Public API surface | `core/src/api.udl` (445 lines, 33 dictionaries/enums, 34 methods), `core/src/lib.rs` (the 288-line crate root) | read in full |
| HTTP/JSON-RPC surfaces | `cli/src/api.rs`, `cli/src/server.rs`, `core/src/wasm_support/rpc.rs` | read in full |

### Deep-read (individually inspected, quoted in the findings)

`core/src/transport/swarm.rs` (11,190 lines — the largest first-party file),
`core/src/transport/mod.rs`, `core/src/routing/mod.rs`, `core/src/routing/local.rs`,
`core/src/routing/smart_retry.rs`, `core/src/store/contacts.rs`, `core/src/store/history.rs`,
`core/src/store/outbox.rs`, `core/src/store/ledger_entry.rs`, `core/src/contacts_bridge.rs`,
`core/src/mobile_bridge.rs` (6,321 lines), `core/src/error.rs`, `core/src/notification.rs`,
`core/src/relay/client.rs`, `core/src/relay/server.rs`, `core/src/relay/peer_exchange.rs`,
`core/src/message/identity_envelope.rs`, `core/src/wasm_support/mod.rs`,
`cli/src/cli.rs`, `cli/src/main.rs`, `cli/src/api_axum.rs`, `cli/src/server.rs`,
`cli/src/ble_windows.rs`, `cli/src/transport_api.rs`, `wasm/src/lib.rs`,
`android/.../MeshRepository.kt` (12,622 lines — the largest first-party file of any language),
`android/.../MeshEventBus.kt`, `android/.../SmartTransportRouter.kt`,
`iOS/.../Models.swift`, `iOS/.../MeshEventBus.swift`, `iOS/.../ContactManagerFix.swift`.

### Tools used

Static text analysis only, driven by Python over `git ls-files`:

- **Extraction:** all `pub fn/struct/enum/trait/type/const` items in first-party Rust (3,118 items);
  all `struct/enum/trait/type` definitions, cross-referenced to find names defined more than once
  (54 found); all `interface/dictionary/enum/callback` blocks and method names in `api.udl`.
- **Frequency:** per-identifier occurrence counts, by layer, to rank by blast radius. What
  counts as an occurrence, and over which files, is defined once in
  `scripts/measure_uncompiled_counts.py` and is not restated here.
- **Pattern sweeps:** catch-all function names (`handle*`, `process*`, `manage*`, `do*`, `misc*`,
  `util*`, `generic*`); numeric-suffix identifiers (`\w+\d+`, and specifically
  `temp*/tmp*/result*/value*/data*/foo*/bar*`); tautological suffixes (`Data|Info|Object|Value|
  Item|Thing|Stuff|Manager|Helper|Util`); `Option<bool>` fields; duplicate `fn current_timestamp`
  and similar zero-arg helpers; `#[allow(non_snake_case)]`; `#[serde(untagged)]`.
  **Tautology triage** (every candidate's call sites read individually, not pattern-matched — nine
  kept, four promoted to findings, one earlier rejection reversed); and a **jargon sweep** (§D-2)
  checking seven project terms against `docs/`, `AGENTS.md`, `README.md`, `CHANGELOG.md` and
  `core/src` doc comments.
- **Reachability:** `mod` declarations vs. files on disk, and `use`/reference greps, to separate
  live code from orphans. This is what surfaced F-06, F-12, F-14 and F-19.
- **Cross-layer divergence:** per-concept cluster counts by layer for entity names, operation
  verbs, and abbreviations.

### What I deliberately skipped, and why

- **`vendor/libp2p-swarm-0.48.0/`** — third-party, excluded by the brief. It is the source of the
  `PeerId` collision in F-07, which is why I read `libp2p::PeerId` usage without auditing that tree.
- **`AgentSwarmCline/`, `scmessenger_swarm/`** — separate agent tooling, not product source.
- **`tmp/`** — this directory holds at least **8 complete stale snapshots of the repository**
  (`tmp/fix-361/`, `tmp/harness-plan-snapshot-20260924/`, `tmp/repro-candidate{,2,3,4,5}/`,
  `tmp/repro-clean/`, `tmp/swa-20260925/`). Including it would have inflated every count by roughly
  10× and produced at least four phantom "duplicate" findings. A `RetryPolicy` search hit exactly
  this and had to be re-run with `tmp/` excluded. **Worth cleaning up separately** — it is a live
  trap for the next tool that runs over this tree.
- **`HANDOFF/` (2,067 `.md` files), `docs/`, `reference/`, `CHANGELOG`** — prose, and the brief puts
  commit/documentation hygiene out of scope. I did read `AGENTS.md` (the naming doctrine in
  §"Architecture doctrine" and Rule 16 are load-bearing for A-2 and F-06) and treated its rules as
  authoritative.
- **Generated bindings** — `iOS/.../Generated/api.swift` (11,801 lines), `SCMessengerCore.xcframework/`,
  and any UniFFI output. Out of scope per the brief, and not editable per AGENTS.md Rule 6.
- **`core/tests/` (51 files), `cli/tests`, `android/.../test/`** — read only where a test was itself
  the evidence (F-06's `cli/tests/integration.rs:3`, F-03's fixture at `store/contacts.rs:1394`).
  Test-file naming was not audited.
- **Build output, lockfiles, logs, `*.csv`, `*.pem`** — excluded by the brief and by AGENTS.md Rule 3.
- **`cloud/orchestrator/`, `scripts/`, `.claude/`** — sampled (they appeared in the catch-all and
  abbreviation sweeps) but not audited. They are infrastructure, not the product's domain, and a
  naming decision taken without them could be wrong for that layer.

### One thing I did not do

**I ran no build, no `cargo check`, no test, and no typecheck.** This audit is text-only. Every
claim about reachability (F-06, F-19) is established by reading `mod` declarations and reference
greps, which is strong evidence but is not the compiler. I flagged the two places where that
distinction matters (Q-4, Q-6) rather than asserting the orphans are dead.

I also did not build, and could not have: AGENTS.md Rule 17 requires a `disk_budget.py` check
before a build, and CI is the default verifier on this lane. Naming findings do not need a build,
so this cost the audit nothing.

---## 2. The top 10 fixes, ranked by value

Value = (correctness risk retired) ÷ (effort + blast radius). Effort: **S** ≈ under a day,
**M** ≈ 1–3 days, **L** ≈ a week or more, or "needs a decision first".

> **This section is an index, not a second copy.** Each row says *what to do, why it ranks here,
> and where the detail lives*. The evidence and the design fix are in FINDINGS; the canonical
> names, alias lists, rename plans and compatibility approaches are in GLOSSARY. Nothing below
> restates either — if a finding changes, this table does not need editing.

| # | Fix | Effort | Value | Detail owned by |
|---|---|---|---|---|
| 1 | Delete the seven `.rs` files no crate root reaches — the cheapest lever in the audit, and it collapses two duplicate-name clusters at a stroke. Sizes and the per-file inventory are in RC-2, which owns them. **Q-6, Q-12 and Q-13 each gate part of it; check which before deleting.** | S | very high for the effort | inventory + cause: **RC-2**; per-file findings: **F-19**, **F-20**, **F-21**, **F-32**, **F-33** |
| 2 | Unify the timestamp helpers; put the unit in every timestamp field's name. A 1000× error in an expiry check is security-relevant, and the code's own comments show the hazard is known-live. | M | very high | evidence + design fix: **F-01**; rename plan + compat: **C-1**, **§F** |
| 3 | Rename `SwarmEvent2` → `SwarmEvent` at the definition. A `_2` scar with no v1 behind it, on the transport layer's primary event type. | S | high for the effort | F-05 |
| 4 | Collapse the duplicate `scm` command grammar — the CLI parser tests currently validate a grammar no user can invoke. | S–M | high | F-06 |
| 5 | Make the `*_bridge` modules derive their types instead of restating them. **One change retires four findings** and is the root cause of the F-02/F-03/F-16/F-22 group. Touches the UniFFI boundary; adjacent to the Rule 8 review gate. | L | high, largest design change | F-02, F-03, F-22 |
| 6 | Split `TransportType` into a mechanism and a path. A rename cannot fix this — the concept must be split first. Most likely to be contested. | L (needs Q-3) | high, contested | evidence: **F-08**; canonical names + rename plan: **C-2**; compat: **§F** |
| 7 | Merge the two `RetryPolicy` definitions — they carry a byte-identical doc comment each claiming to be the only one, which is the strongest available proof the duplication is an accident. | S | high for the effort | F-04 |
| 8 | Introduce a `PeerKey` enum for the three peer-id encodings. A storage-format change, not a rename. | L (needs Q-3) | high | evidence: **F-09**; canonical names: **A-5**; compat: **§F** |
| 9 | Delete `isRelay` and the other dead flags — including `SwarmTaskLivenessGuard`, a verbatim type duplicated inside one 11k-line file. | S | medium-high for the effort | F-13, F-15, F-18, F-10 |
| 10 | Adopt a five-line read-verb rule and retire `retrieve`. A written rule costs nothing and stops the drift; the backward rename is optional. | S (rule) / M | medium | F-23; rule: **B-1** |

**Also queued, lower rank:** the `msg`→`message` migration (F-24, M — mechanical,
identifier-only; counts and canonical names in **D**) and `cfg`→`config` (F-24, **S** — the
corrected count is in **D**, which owns it); the `/api/` route-prefix consolidation
(F-11, S–M, needs an access-log check first — compat in **§F**); the `blocked_identity_*`
constructor collapse (F-17, S/M); the `DiscoveryMode` split (F-16, S); the iOS
`ContactManagerFix` removal (F-14, S); the tautology fixes (F-28 to F-31, all S).

---

## 3. Open questions — these need the repo owner's domain judgment

I did not stall on any of these, and I did not guess at an answer. Each one changes the shape of a
remediation, so they need a human.

**Q-1. Is `MeshRepository.kt:6749` a live unit bug or a deliberate raw value?**
`val lastSeenRaw = obj.optLong("last_seen", 0L)` reads a **ledger** field (milliseconds, per
`core/src/store/ledger_entry.rs:15`) and assigns it to a `lastSeen` whose siblings are seconds
(`:810`, `System.currentTimeMillis().toULong() / 1000u`). There is no `/1000`. The variable is
named `...Raw`, which suggests the author knew. I could not determine whether a conversion happens
downstream. *This is a naming finding either way* — a suffix that promises a distinction the code
does not act on — but if it is a live bug it is a correctness issue that outranks everything in
this report, and it is not mine to declare from a text read.

**Q-2. Is `Contact::tombstone()` / `is_tombstone` reachable at all?**
`contacts_bridge.rs:51` constructs a tombstoned contact and the struct has an `is_tombstone` flag,
but `ContactManager::remove` (`:181`) deletes outright and never writes one. I could not find a
caller that produces a tombstone. Is retention a requirement that was implemented and not wired, or
a leftover? Determines whether F-03 is "merge two managers" or "merge two managers and implement
retention".

**Q-3. Is the 64-char hex peer id permanent or transitional?**
`core/src/store/ledger_entry.rs:486-489` calls it "UNIFICATION: live canonicalization" and rewrites
libp2p ids to hex on every write. *Permanent* ⇒ F-09 recommends the `PeerKey` enum with `Hex` as a
first-class variant. *Transitional* ⇒ a dual-read adapter that retires it. This single answer
changes fixes #6 and #8 from L/M to L/L, and I cannot tell from the code which it is.

**Q-4. Are the `cli/src/server.rs` UI stubs (`UiEvent`/`UiCommand`/`UiOutbound`, 60 lines, headed
"Stub types for BLE mesh UI integration (Phase 1B wiring)") still bound?**
`cli/tests/integration_message_requests.rs:5` imports `UiCommand`, so they are at least test-wired.
Whether a runtime path reaches them I could not establish without a build. If unbound: delete
(F-12, S). If bound: the variants need real types (M).

**Q-5. Does `MeshSettings.discovery_mode` actually reach the discovery engine?**
`settings.rs:20` persists a `settings::DiscoveryMode` (`Normal`/`Cautious`/`Paranoid`); the engine
branches on `discovery::DiscoveryMode::Open`/`LanOnly`/… (`discovery.rs:73,78,85,91`). The two
enums share no variants except `Cautious`. I did not trace the data flow. If the persisted knob is
unwired, F-16 is a naming problem; if it is wired and the enums are supposed to correspond, it is a
design bug.

**Q-6. Are `core/src/wasm_support/{mesh,storage,transport}.rs` a deliberate parked v2?**
If yes, fix #1 becomes "move them to a clearly-labelled `attic/` directory with a README" rather
than "delete", and the ghost type names stop seeding clusters. If no, delete. I found no reference
to any of the three, and `mod.rs` carries a comment saying they are "not wired into `lib.rs` until
they are updated for current libp2p" — which reads as deliberate parking. **This is why fix #1 is
gated on this question.**

**Q-7. Are `config` and `settings` two concepts or two names for one?**
472 `settings` occurrences vs 998 `config`, in near-disjoint file sets (restated —
see GLOSSARY §D for the basis and the figures this replaced). My reading is that they
*are* different — `settings` is user-facing preference state persisted in `MeshSettings`, `config` is
wiring read at startup. If so, keep both and record the distinction. If not, it is a 1800-occurrence
merge.

**Q-8. Do `text` and `body` name the same thing?**
618 vs 160 occurrences. Merging them is the single largest count reduction available in the
abbreviation table, and I am not confident enough to recommend it. A "message body" and a "text
message" are probably different types here, but I did not verify.

**Q-9. Should the run-control API (`/submit-run`, `/poll-status`, `/fetch-artifact`) live on the
same port and router as the mesh Control API?**
They are the dspy/Ollama agent API, not the mesh API, and they use a different route prefix and a
different verb vocabulary from the fourteen `/api/*` routes beside them. Consolidation (F-11) is a
naming question, but splitting the surface is a product-boundary question.

**Q-10. Is the `UNIFICATION` codename still meaningful, or should the comment prefixes be retired?**
279 code-comment occurrences under six spellings (`UNIFICATION`, `UNIFICATION_V2`,
`UNIFICATION_V2_IDENTITY`, `UNIFICATION_V3`, `UNIFICATION_V2_TRANSPORT`, `UNIFICATION_DIAL`),
defined nowhere — the only `docs/` hits are an unrelated filename. It is also the *only* record of
the design decision behind F-09 (the hex-vs-multihash peer-id split), which makes it load-bearing
documentation in practice. Retire-and-describe, or keep-and-define? See GLOSSARY §D-3a.

**Q-11. Are the two things called `beacon` meant to share a word?**
An identity BLE beacon (283 hits overall) and an Apple Find My compatible beacon encoding
(`core/src/relay/findmy.rs:1-5`) share the name and are separate mechanisms. If the Find My one is
a temporary experiment, renaming it is trivial; if it is a permanent second discovery channel, the
distinction needs to be in the vocabulary from the start. See GLOSSARY §D-3b.

**Q-12. Is `cli/src/api_axum.rs` a parked rewrite, or a discarded experiment?**
The 763-line file duplicates nine type names that exist for real in `cli/src/api.rs` and
builds a second `Router` on the same `API_PORT`, but nothing compiles it. Its own header
calls it "the new Axum 0.7 server implementation"; a doc comment at `:28-32` says the
current implementation is live and this one "is not currently wired into any `mod` tree /
bin target". Those two statements disagree, and nothing in the repository says which was
meant. Delete it (effort S) or wire it in and reconcile nine names (effort L)? See
FINDINGS F-32.

**Q-13. Should the `wasm` crate be in the build at all?**
`wasm` appears in the workspace `members` list *and* in `exclude`, so
`cargo test --workspace` and `cargo clippy --workspace` skip it, and inside it three
files are unreachable even from its own crate root. Either the crate should be compiled
somewhere on a schedule, or it should not be in the tree — but that is a build-topology
decision, not a naming one, and this audit did not take it. **This is the one open
question here whose answer is not about vocabulary.** See FINDINGS F-33.

---

## 4. What I was uncertain about

Honest accounting, most-material first.

1. **Reachability is derived, not compiled.** The seven uncompiled files in RC-2 were found by
   walking each crate's module graph from its crate roots (`mod x;` resolved recursively),
   not by `use`-greps and not by a build. That derivation is reproducible —
   `python scripts/measure_uncompiled_counts.py` re-walks it and reprints the inventory — but
   I did not compile anything. *Mitigation: the files corroborate the claim themselves;
   `cli/src/api_axum.rs:32-33` says *"this module is not currently wired into any `mod` tree / bin
   target"*, which is the same conclusion the walk reached independently.* The F-06
   (`cli/src/cli.rs`) orphan claim rests on the same kind of reading: `cli/src/lib.rs:10`
   declares `pub mod cli;` while `main.rs`'s `mod` list does not.

2. **The glossary's counts were on a basis recorded nowhere, and I could not reproduce it —
   so they have been restated, and the basis is now a choice I have to defend rather than a
   gap.** I tried fifteen corpus definitions (extension sets, test and `bin/` inclusion, five
   crates) against fifteen published figures; the closest candidate was 37% low on `config`
   and three times high on `new`. Every count in the glossary now comes from
   `python scripts/measure_uncompiled_counts.py` on the basis that script's `BASIS` block
   defines and that GLOSSARY points to rather than restates, and every entry whose figure
   moved carries a *Count basis* note saying what it was.
   **Read that as a basis change, not a set of corrections:** the corpus narrowed to
   first-party code and matching became case-sensitive, so nearly every figure falls, and
   `Envelope` alone falls 2184 → 51 because the question changed from "the word" to "the
   type". What is still uncertain is the choice itself — a first-party-only corpus is right
   for *what ships* and wrong for *how much a rename touches*, and `tests/` and `examples/`
   hold real call sites, which is why FINDINGS' two evidence tables use `--basis wide`
   instead. *The restatement also surfaced a defect the old numbers hid: whole-word matching
   excludes compound identifiers, so `peer` 3461 is a floor — the peer vocabulary is 9737
   once `peer_id` and `peerId` are counted, and the middle of the A-1 ranking inverts.
   GLOSSARY A-1 carries both.*

3. **Occurrence counts are identifier-atom matches, not semantic matches.** `peer` 3461 counts
   `peerId`, `peer_id`, `PeerDiscoveryInfo`, `peers`. That is right for ranking blast radius and
   wrong for reading as "the concept Peer appears 3461 times". Every count in the report is
   now measured by that script counts the bare word only. The compounds are counted
   separately and reported in GLOSSARY A-1: `peer` the word is 3461, `peer` the vocabulary is
   9737.

4. **I sampled `core/src/transport/swarm.rs` (11,190 lines) and `MeshRepository.kt` (12,622 lines)
   rather than reading them end to end.** They are 2 of the 4 largest first-party files. The
   findings drawn from them (F-05, F-13, F-15) came from targeted structural searches, so a
   long-tail of name issues in those two files is likely uncounted. Both files are also where
   "one type, N copies" problems are most likely to hide.

5. **I did not audit the mobile UI layer's own vocabulary** (Kotlin/Swift ViewModels and screens,
   ~20,000 LOC) beyond what surfaced in the sweeps. `ContactsViewModel`, `SettingsViewModel`,
   `ChatViewModel` and their screens are a plausible home for a Layer 2 vocabulary cluster and I
   have no evidence either way.

6. **Two readings are plausible for `core/src/transport/mesh_routing.rs`'s `RetryStrategy`**
   (re-exported at `core/src/transport/mod.rs:65`). I flagged it in F-04 as possibly a *routing* policy
   wearing a retry name, and I did not read it. That specific characterisation is a guess.

7. **`serviceInfo` was initially recorded as "considered and rejected", and that was wrong.**
   The rejection rested on "the surrounding types disambiguate at every call site I sampled".
   Reading all three use sites showed the third is a raw `ByteArray` hand-decoded as TLV
   (`WifiAwareTransport.kt:51`), which the other two are not. It is **F-29**, and the original
   rejection is marked withdrawn in FINDINGS rather than quietly deleted. *The lesson generalises:
   "I sampled it and it looked fine" is not the same as "I read every site", and I had presented
   the first as the second.*
8. **The tautology triage read call sites for the eleven highest-count candidates, not all 32 the
   scan returned.** The nine "kept" entries were each judged on a read of their defining site;
   entries below 10 occurrences were judged on the name alone. If a low-count tautology matters, it
   is probably not in that tail anyway — but the triage is proportional, not exhaustive.
9. **The `.xcframework` headers I read are a build artifact.** I used them to confirm that
   `ContactManagerProtocol` exists in the generated bindings (F-14), which is sound, but I did not
   audit them and they may lag the current UDL.
10. **I could not compare against the project's own prior audits.** `HANDOFF/review/` contains
   `SCOPE_INVENTORY_2026-09-24.md` and `D1_D9_HARNESS_ADVERSARIAL_FINDINGS_2026-09-22.md`, and
   `HANDOFF/todo/` has open tickets. Some of what I found may already be known and tracked. I
   deliberately did not read them — the brief asks for an independent audit, and anchoring on
   prior findings would have narrowed the search. **Worth a cross-check before acting.**
