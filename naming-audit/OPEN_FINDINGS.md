# Naming Audit — Open Findings Register

**Repository:** SCMessenger
**Baseline:** `origin/main` @ `e76e0593`
**Date:** 2026-10-01
**Mode:** read-only. No source file was renamed, moved, refactored, or modified in producing
this register, and none has been modified in committing it. The only file added is this one.

**What this file is.** The single surviving register of naming findings that survived
adversarial re-derivation. It supersedes three working reports that were written during the
pass and which contradict each other in places. Those reports lived in the gitignored `tmp/`
directory and are not the record; this file is.

**Sits alongside, does not replace:** `GLOSSARY.md` (the doctrine and the canonical
vocabulary), `FINDINGS.md` (the 32 original F-xx findings), `SUMMARY.md` (the original
top-10 fixes and open questions Q-1..Q-13). Those three are unchanged. Where this register
records a number that differs from theirs, this register's number is the measured one and
the correction is logged in §Corrections.

---

## How to read this register

- **`OF-nn`** are open findings. One entry each, with a corrected count, file:line evidence,
  whether the glossary settles it, whether the naming gate should catch it, and the smallest
  correct resolution.
- **`RF-nn`** in §Refuted are findings that were proposed during this pass and then
  **disproved**. They are listed because two of them were wrong in ways that would cause
  damage if acted on. **Do not act on anything in that section.**
- Most concepts in this repository carry an explicit status in `GLOSSARY.md`. **An unsettled
  concept is not a defect.** Every entry below says which it is.
- Ranking is by real impact: a name that misleads, or that crosses a serialization, wire,
  database, or persisted-format boundary, outranks a cosmetic inconsistency.

### Measurement baseline

Every count in this register was measured on:

| | |
|---|---|
| Reference | `origin/main` @ `e76e0593` |
| File set | `git ls-tree -r --name-only origin/main` — **487 code files** (`.rs .kt .kts .swift .ts .tsx .js .mjs .udl`) |
| Matching | whole-word, identifier-boundary on both sides (`relayPeerId` does not count as `relayPeer`) |
| Counts | **occurrences**, not matching lines |
| Classification | every hit labelled IDENTIFIER / STRING / COMMENT |
| Invariant | `IDENT + STRING + COMMENT == occurrence count`, 0 failures across 16 terms |

Three measurement errors were found and corrected during this pass. They are the reason an
earlier draft's numbers do not match:

1. `git grep -c` reports matching **lines**, not occurrences. Any figure taken from it is
   an undercount when a line holds two or more hits.
2. `git ls-files` lists the **index**, not `origin/main`. On this checkout the index is 25
   commits ahead with 90 staged files belonging to another session, and it carries 33 code
   files that `origin/main` does not.
3. Outputs from two different builds of the classifier were mixed into one table.

The corrected harness is `scripts/`-adjacent scratch under `tmp/audit/` and is not committed.

---

## Category 1 — Violations the policy layer does not catch

The naming gate (`naming_policy.json` + `scripts/check_naming.py`, both on the
`policy/naming-gate` branch, **not yet on `main`**) scans `.rs .kt .kts .swift .ts .tsx .js
.mjs .py .udl` and exempts `naming-audit/`, `HANDOFF/`, `docs/`, `tmp/`, `target/`,
`node_modules/`, `vendor/`, plus two named scripts. These are the gaps.

### OF-01 — Five relay tokens are grandfathered under a rationale that is false for all five

**Severity:** HIGH  **Occurrences:** 265 / 129 identifiers / 45 distinct code files

| term | occurrences | identifiers | files |
|---|---:|---:|---:|
| `relays` | 198 | 67 | 40 |
| `isBootstrapRelayPeer` | 40 | 39 | 6 |
| `isKnownRelay` | 17 | 13 | 3 |
| `isRelayHop` | 7 | 7 | 1 |
| `isInfraRelay` | 3 | 3 | 1 |

`naming_policy.json` A-2 lists all five in `grandfathered`, noted as *"Named in GLOSSARY A-2
as historical identifiers carrying the verb, and as the persisted custody field pair."*

**That rationale is false for all five.** `Relay` is a **noun** in every one — a role name,
which is precisely what A-2 denies. `relays` is a role noun too:
`core/src/transport/dial_policy.rs:376` `relays: Arc<RwLock<Vec<RelayEntry>>>` (KDoc `:375`
"List of known relay peers") and `wasm/src/transport.rs:1142`
`relays: Rc<RefCell<HashMap<String, WebSocketRelay>>>` — and `wasm/src/transport.rs` **is**
declared at `wasm/src/lib.rs:6`, so it compiles.

**Evidence that no reason is recorded:** `git grep -c -w` over `naming-audit/` returns **zero
files** for `isKnownRelay`, `isBootstrapRelayPeer`, `isRelayHop` and `isInfraRelay`. A-2's
inventory table (GLOSSARY.md:150-157) has rows for `node`, `relay`-as-noun, `relay`-as-verb
and `isRelay` — and no row for any of these.

`HANDOFF/audit/doctrine_violation_inventory.md:849` lists `isBootstrapRelayPeer()` under
"Major structural issues" and its migration guide item 15 prescribes
`isBootstrapRelayPeer()` -> `isBootstrapPeer()`. That inventory predates the glossary, so the
two are not in conflict so much as one is silent. The defect is narrower: **the policy
invented a grandfathering rationale for tokens the glossary never names.**

**Glossary settles it?** No — silent on all five.
**Gate should catch it?** Yes, as a WARN. These are live identifiers in compiled Kotlin,
Swift and Rust; a new call site copying one today is copying an unreviewed name.
**Smallest correct resolution:** move all five from `grandfathered` into `denied`, each with
a `remedies` entry `{action: "rename", status: "PROPOSED"}`. Under the two-condition rule
that makes them warn-only, never blocking — correct, because the deletion was never
ratified. Add five rows to A-2's inventory table saying what each one is. **Policy and
glossary only. No code change, no behaviour change.** Effort S.

### OF-02 — Six of fourteen parsers for one log string sit in a file the gate cannot scan

**Severity:** MEDIUM  **Occurrences:** 14 across 4 files; **9 of 14 unscanned**

`Starting Swarm with PeerID: <id>` is emitted at `core/src/mobile_bridge.rs:752` and parsed
by **14** occurrences across four files. Every one of them parses that single string:

| file | occurrences | extension | scanned? |
|---|---:|---|---|
| `scripts/check_logs.py` | 3 | `.py` | yes |
| `scripts/get_peer_ids.py` | 2 | `.py` | yes |
| `scripts/run5.sh` | 8 | `.sh` | **no** |
| `scripts/run5-live-feedback.sh` | 1 | `.sh` | **no** |

`PeerID` is **0 identifiers** across all 12 code files where it occurs — it is a log-format
token, never a name. This is a persisted-format boundary: renaming the log string would
break parsing of logs already on disk, and the gate would catch 5 of the 14 sites.

**Glossary settles it?** No — recorded as `GAP-peer-id-casing`, tier `warn`, status
`AUDIT-GAP`, remedy `AUDIT-GAP`. Correctly unsettled.
**Gate should catch it?** It already warns on the two `.py` files. It cannot see `.sh`.
**Smallest correct resolution:** add `.sh` to the scan-extension list. Cost: the
STRING/COMMENT classifier degrades on shell, but the FAIL/WARN distinction still works.
**Do not rename the log string** — 14 parsers and existing operator logs depend on it.
Effort S.

### OF-03 — The policy names one enforcement point; the repository has two

**Severity:** LOW  **Occurrences:** 6 (3 per snapshot file)

`scripts/ffi-snapshots/kotlin-symbols.txt` (450 lines) and `swift-symbols.txt` (500 lines)
are a checked-in FFI surface contract. `scripts/ffi_surface.sh` extracts every
`fun|class|interface|enum|object|data class|sealed class|value class` from the generated
Kotlin and every `public func|class|protocol|enum|struct|typealias|open class|open func`
from the generated Swift, then diffs against the snapshots; `.github/workflows/ci.yml` runs
it. Both snapshots contain `envelopeData` (3 each). `.txt` is not a scan extension.

The consequence is coordination, not violation: `check_naming.py` will approve a rename that
`ffi_surface.sh` then fails. Two gates police the same names and `naming_policy.json`
mentions only one.

**Glossary settles it?** No. A-3 and A-4 record `envelopeData` 41/4 and `envelope_data`
172/20; neither names the snapshot gate.
**Gate should catch it?** Not its job — but the policy should **name** it, the way it already
carries `audit_ref`.
**Smallest correct resolution:** one line in the `naming_policy.json` header note recording
that a UDL rename additionally requires `scripts/ffi_surface.sh --update`. Effort S.

### OF-04 — A 7.1 MB tracked snapshot holds 217 governed symbol names and is six weeks stale

**Severity:** LOW-MEDIUM  **Occurrences:** 217 in 1 file

`log-visualizer/public/data/wiring_graph.json` is a generated Cytoscape call graph,
`generated_at` **2026-08-20**, built by `scripts/build_wiring_graph.py:518-569` from
`HANDOFF_AUDIT/REPO_MAP.jsonl`. It is tracked, is **not** gitignored (no `log-visualizer`
rule in `.gitignore`), and nothing in CI refreshes it.

| term | occurrences |
|---|---:|
| `PeerIdValidator` | 139 |
| `isBootstrapRelayPeer` | 46 |
| `isKnownRelay` | 25 |
| `TransportHealth` | 7 |

Renaming any token in OF-01 or OF-09 leaves 71 copies of the old spelling here that nothing
detects. It is an input to nothing — a stale dashboard — which is why this is LOW-MEDIUM and
not higher.

**Glossary settles it?** No.
**Gate should catch it?** No — `.json` is unscanned, and the file is generated.
**Smallest correct resolution:** gitignore the path and regenerate from the current
REPO_MAP. Effort S.

### OF-05 — A grandfathered entry that matches nothing, and two with no stated reason

**Severity:** LOW  **Occurrences:** `RelayCustody` = 0; `relay_id` = 9/4 files; `relayId` = 7/3

- `RelayCustody` appears in `naming_policy.json`'s `grandfathered` list and occurs **zero
  times** anywhere, in code and in `naming-audit/` alike.
- `relay_id` (9 occurrences, 4 files: `core/src/transport/swarm.rs` 4, one `HANDOFF/done/`
  file 1, `naming-audit/GLOSSARY.md` 2, `scripts/measure_uncompiled_counts.py` 1) and
  `relayId` (7, 3 files: `MeshRepository.kt` 3, GLOSSARY 2, measure script 1) are named in
  GLOSSARY.md:110 only inside a prose count list and again at `:748-749` in the abbreviation
  inventory. No rationale, no status.

**Glossary settles it?** Mentions them without deciding.
**Gate should catch it?** No — both are already grandfathered.
**Smallest correct resolution:** one sentence each in A-2 saying why a persisted field is a
legitimate exception; delete the `RelayCustody` entry. Policy file only. Effort S.

---

## Category 2 — Confusing or ambiguous names within a cluster

### OF-06 — `isBootstrapRelayPeer` resolves to two different functions, one of which is dead

**Severity:** HIGH  **Occurrences:** 40 / 39 identifiers / 6 code files; **34 live call sites**

Three definitions, two behaviours:

| # | Site | Body |
|---|---|---|
| 1 | `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:10934` | `fun isBootstrapRelayPeer(peerId: String): Boolean { return false }` — one unconditional return, `peerId` unused |
| 2 | `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:5930` | `return ledgerBootstrapEntries().contains { $0.peerId == peerId }` |
| 3 | `iOS/SCMessenger/SCMessenger/ViewModels/ContactsViewModel.swift:270` | `return repo.isBootstrapRelayPeer(peerId)` — delegates to #2 |

`ledgerBootstrapEntries` (`MeshRepository.swift:3327`) unions
`getPreferredRelays(limit: 10)`, `dialableAddresses()` and `seedAddresses(limit: 10)`,
deduped by address.

**The user-visible consequence is provable, and it is not a rename problem.**
`ContactsViewModel.swift:279-280`:

```swift
// Never surface bootstrap relay/headless nodes in the Contacts nearby list.
if checkIds.contains(where: { isBootstrapRelayPeer($0) }) { return }
```

`ContactsViewModel.kt:291-294` carries the **same comment verbatim**:

```kotlin
// Never surface bootstrap relay/headless nodes in the Contacts nearby list.
val isRelay = meshRepository.isBootstrapRelayPeer(event.peerId) || ...
if (isRelay) return@collect
```

The iOS filter works. The Android filter is `if (false) return`. **iOS hides cloud-node
peers from the Contacts Nearby list; Android shows them.** Same comment, same event, same
intent, opposite result — caused entirely by one name resolving to two functions.

Live call sites that would change behaviour if unified: `MeshRepository.kt` 15 (incl.
`:10948` inside the wrapper), `ContactsViewModel.kt` 3 (`:292`, `:293`, `:387`),
`MeshRepository.swift` 13 (incl. `:5936` inside `isKnownRelay`), `ContactsViewModel.swift` 3
(`:272` wrapper, `:280`, `:343`), plus 3 in two test files.

`MeshRepository.kt:10943` `isBootstrapRelayPeerFromKey` is a fourth definition — a wrapper
that delegates to the dead function, so it is dead on arrival while its KDoc describes live
behaviour.

**Glossary settles it?** No — none of the three names appears in `naming-audit/`.
**Gate should catch it?** No — this is a semantic divergence wearing a correct-by-doctrine
name.
**Smallest correct resolution:** **decide which behaviour is intended first** — that is the
repository owner's call, not an audit's. Then name them apart: keep `isBootstrapPeer` for
the iOS ledger lookup and delete `isKnownRelay` and `isBootstrapRelayPeerFromKey` if the
decision lands on "no distinction". **A rename alone cannot fix this**, because the two names
are currently describing two different real behaviours. Effort M once decided.

### OF-07 — `isKnownRelay` has the same shape and the same disagreement

**Severity:** HIGH  **Occurrences:** 17 / 13 identifiers / 3 code files

| site | body |
|---|---|
| `android/.../data/MeshRepository.kt:10830` | `return false`, commented *"retained for API compat but always returns false"* |
| `iOS/.../Data/MeshRepository.swift:5935` | `if isBootstrapRelayPeer(peerId) { return true }` then `guard let info = discoveredPeerMap[peerId] else { return false }; return info.isRelay && !info.isFull` |

Three Android call sites (`:3823`, `:5886`, `:7063`) call a function whose own comment says
it always returns `false`; the four comment occurrences (`:6690`, `:6966`, `:10582`,
`:10829`) each document that it is dead. The five callers in
`iOS/.../Views/Dashboard/MeshDashboardView.swift` (`:139`, `:202` x2, `:315`, `:316`) get a
live classification.

**Glossary settles it?** No. **Gate should catch it?** No — see OF-01, which covers the
policy side.
**Smallest correct resolution:** same decision as OF-06, then either implement it on Android
or delete it on both. Effort S once OF-06 is decided.

### OF-08 — The cloud node has two names, and on Android they denote one value

**Severity:** MEDIUM  **Occurrences:** 12 identifiers / 2 files

`MeshRepository.kt:1878`:
`val isInfraRelay = isBootstrapRelayPeer(peerId) || isInfrastructureAgent(agentVersion)`

against `MeshRepository.kt:255-262`, verbatim:

```kotlin
        internal fun isInfrastructureAgent(agentVersion: String): Boolean {
            val agent = agentVersion.trim().lowercase()
            if (agent.isEmpty()) return false
            return agent.contains("scm-always-on-node") ||
                agent.contains("always-on") ||
                agent.contains("infra-relay")
        }
```

`isInfraRelay`: 3 occurrences, all identifiers, all in `MeshRepository.kt` (`:1878`, `:1879`,
`:1918`). `isInfrastructureAgent`: 9 occurrences, 2 files (`MeshRepository.kt` 2,
`MeshRepositoryTest.kt` 7).

Both detect the same thing — the AWS always-on cloud node — and neither uses `node`, which
is A-2's canonical role word. Because Android's `isBootstrapRelayPeer` is `return false`,
**`isInfraRelay` on Android reduces exactly to `isInfrastructureAgent`**: two names, one
value.

**Glossary settles it?** No — neither name is in `naming-audit/`.
**Gate should catch it?** `isInfraRelay` is grandfathered; `isInfrastructureAgent` is on no
list.
**Smallest correct resolution:** rename both to `isCloudNode` — 12 identifiers, 2 files, one
expression. Pairs naturally with the `isRelay` -> `isInfraNode` rename already prepared.
Effort S.

### OF-09 — `TransportHealth` is defined twice on Android, and the collision reaches a public signature

**Severity:** MEDIUM  **Occurrences:** 24, **all identifiers** / 4 code files

| site | shape |
|---|---|
| `android/.../transport/SmartTransportRouter.kt:54` | `data class TransportHealth` — `lastSuccessAt`, `lastFailureAt`, `successCount: Long`, `failureCount: Long`, `averageLatencyMs`, `lastLatencyMs`, plus computed `successRate` and `isHealthy` |
| `android/.../transport/TransportHealthMonitor.kt:15` | `data class TransportHealth` — `successCount: Int`, `failureCount: Int`, `totalLatencyMs`, `lastUpdated`, `consecutiveFailures` |

The two fields they share, `successCount` and `failureCount`, have **different types**
(`Long` vs `Int`). Both are live: the router's at `:100`, `:157`, `:159`, `:165`; the
monitor's at `:13`, `:24`, `:32`, `:42`, `:43`, `:66`.

**The collision has already reached a public API.** `MeshRepository.kt:12372`:

```kotlin
fun getTransportHealthSummary(): Map<String, com.scmessenger.android.transport.TransportHealthMonitor.TransportHealth> {
```

That fully-qualified return type exists only because the unqualified name is ambiguous.
`TransportHealthMonitor` itself is instantiated twice — `MeshRepository.kt:66` and
`TransportManager.kt:42`.

The iOS `struct TransportHealth` in `SmartTransportRouter.swift` (11 occurrences) is the
deliberate per-platform mirror of the **router's** copy, not the monitor's.

**Glossary settles it?** No — the name does not appear in `naming-audit/`.
**Gate should catch it?** No — not on any list.
**Smallest correct resolution:** rename the **monitor's** copy to `TransportHealthRecord` —
7 occurrences, 1 file, entirely private to `TransportHealthMonitor`. Renaming the router's
copy would collide with the Swift one that mirrors it. Effort S.

### OF-10 — `PeerIdValidator`'s iOS implementation is filed inside a 7,292-line file

**Severity:** LOW  **Occurrences:** 274 / 267 identifiers / 19 code files

- `android/app/src/main/java/com/scmessenger/android/utils/PeerIdValidator.kt:18` —
  `object PeerIdValidator`, in a file named for it, with three test files named for it.
- `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:7253` — `struct PeerIdValidator`,
  at the bottom of the repository file, with no file of its own.

There is **no Rust `PeerIdValidator`** — confirmed by direct grep over `*.rs`; the earlier
suggestion that one existed was wrong.

Densest files: `MeshRepository.kt` 99, `ContactsViewModel.kt` 45, `PeerIdValidatorTest.kt`
42, `PeerIdValidatorCurveVectorTest.kt` 18, `ContactsViewModel.swift` 16,
`MeshRepository.swift` 13, `PeerIdValidatorCanonicalTest.kt` 12, and 12 files with 1-4 each.

**Two runtimes require two implementations**, so the duplication is deliberate. What is
wrong is the file organisation. **I cannot tell whether the two implementations agree** —
there is no shared fixture and no cross-platform vector file, and I did not read both bodies
in this pass. Do not assume they agree.

**Glossary settles it?** Partially — `FINDINGS.md:1648` records the `X2` outlier at
`PeerIdValidator.kt:135`. The cross-platform split is not recorded.
**Gate should catch it?** No.
**Smallest correct resolution:** move the Swift `struct` to
`iOS/SCMessenger/SCMessenger/Utils/PeerIdValidator.swift` to match the Kotlin layout. A file
move with **zero identifier changes.** Effort S.

### OF-11 — Two test classes in each of two pairs share a simple name in the same source set

**Severity:** LOW  **Occurrences:** 4 files

| file | bytes | tests | package |
|---|---:|---:|---|
| `android/app/src/test/.../test/ContactsViewModelTest.kt` | 7,358 | 5 | `...android.test` |
| `android/app/src/test/.../ui/viewmodels/ContactsViewModelTest.kt` | 10,812 | 7 | `...android.ui.viewmodels` |
| `android/app/src/test/.../data/ReceiptUnificationTest.kt` | 11,484 | 2 | `...android.data` |
| `android/app/src/test/.../test/ReceiptUnificationTest.kt` | 24,452 | 7 | `...android.test` |

**These are not duplicated logic.** The `ContactsViewModelTest` pair covers
`loadContacts` / `addContact` / `removeContact` / `search` / online status against
`promoteNearbyPeerToContact` / `PeerEvent` handling / `refreshDiscovery`. Both compile, both
run, and a stack trace naming `ContactsViewModelTest` does not say which.

**Glossary settles it?** No. **Gate should catch it?** No.
**Smallest correct resolution:** give each a distinguishing name or move both into one
package. Cosmetic; do it when either file is next edited. Effort S.

---

## Category 3 — Non-unified code

### OF-12 — `PeerEvent` and `StatusEvent` are declared twice inside iOS

**Severity:** MEDIUM  **Occurrences:** 2 names, 3 declaration sites each

| name | Android | iOS site A | iOS site B |
|---|---|---|---|
| `PeerEvent` | `MeshEventBus.kt:108` `sealed class` | `MeshRepository.swift:471` `enum` (**not** `Equatable`) | `MeshEventBus.swift:43` `enum … : Equatable` |
| `StatusEvent` | `MeshEventBus.kt:136` `sealed class` | `MeshRepository.swift:477` `enum` (**not** `Equatable`) | `MeshEventBus.swift:58` `enum … : Equatable` |

**Abstraction they should share: one event hierarchy per name, published by the bus and
consumed by the repository.** The bus is the publisher, so the repository should consume the
bus's type rather than declare a parallel one that lacks `Equatable`.

**Why this is not deliberate:** the two iOS copies are not equivalent — one conforms to
`Equatable`, one does not. A deliberate mirror would not differ on a protocol conformance in
the one place it matters for a `Flow` or `AnyPublisher` chain.

**I cannot tell** which of the two iOS declarations is meant to be authoritative without
reading the subscription sites, which this pass did not do.

**Glossary settles it?** No — neither name is in `naming-audit/`. **Gate should catch it?** No.
**Smallest correct resolution:** delete the repository's copies and re-point its subscribers
at the bus's types. Scope is two declaration sites plus their consumers. Effort M.

---

## Category 4 — Duplicates

Every entry in this section is a **genuine duplicate**: two implementations of one thing,
where per-platform duplication would have been **similar on purpose**. The split is stated
per entry so the two are not confused.

### OF-13 — `android/shared/` is an 18-file Gradle module that nothing builds

**Severity:** MEDIUM-HIGH  **Occurrences:** 18 files, 40,359 bytes

**This is `FINDINGS.md` RC-2's blind spot.** RC-2's uncompiled-file inventory walked `.rs`
crate roots. It never walked a Gradle module graph, so an entire Kotlin module escaped it.

Evidence:

- `android/settings.gradle:19` — `include ':app'`. That is the only include.
- `settings.gradle:21-22` — `include ':shared'` with
  `project(':shared').projectDir = new File('shared')`, which binds the **root** `shared/`
  (5 files), **not** `android/shared/`.
- A grep for `android/shared` and `:shared` across `*.gradle`, `*.gradle.kts`, `*.yml`,
  `*.sh`, `*.py`, `*.toml` returns 5 hits, **all** for the root `:shared`
  (`.github/workflows/desktop.yml:50`, `docs/workflows/desktop.yml:67`,
  `scripts/build_desktop.sh:20`, `settings.gradle:21-22`). **Zero** reference
  `android/shared/`.

Contents include `commonMain/.../viewmodel/ContactsViewModel.kt`,
`commonMain/.../viewmodel/ChatViewModel.kt`, `commonMain/.../model/Contact.kt`,
`di/SharedModule.kt`, `AppViewModel.kt`, `androidMain/` and `desktopMain/` platform
actuals, and two 8-9 KB desktop UI files.

**Why it matters for naming:** it seeds the duplicate cluster with a phantom
`ContactsViewModel` and `ChatViewModel` — which is why a sweep sees each at three files
rather than two — and it contributes **2 `lastSeen` occurrences**
(`commonMain/.../model/Contact.kt`, `commonMain/.../viewmodel/ContactsViewModel.kt`) to every
`.kt` sweep. The 225-occurrence `lastSeen` figure is therefore an upper bound.

**Why not deliberate:** an unbuilt module is not a design choice. **I cannot tell** whether
this is an abandoned experiment or a wiring mistake from the tree alone; `SUMMARY.md`'s
Q-6, Q-12 and Q-13 ask exactly this question of the Rust orphans and do not ask it of this
one.

**Glossary settles it?** No. **Gate should catch it?** No — the gate judges added lines, and
these files are frozen.
**Smallest correct resolution:** decide first (abandon, or wire it). If abandoned, delete
the 18 files; if intended, add `include ':shared'` pointing at `android/shared`. Either way
the duplicate cluster shrinks by two names. Effort S once decided.

### OF-14 — Two Android sealed hierarchies share six variant names one-for-one

**Severity:** MEDIUM  **Occurrences:** 2 files, 6 shared variant names

- `android/app/src/main/java/com/scmessenger/android/data/IdentityCreationEvent.kt:28`
  `sealed class IdentityCreationEvent` — `PreparingStorage` `:30`, `GeneratingSalt` `:33`,
  `GeneratingKeypair` `:36`, `ComputingFingerprint` `:39`, `PersistingToStorage` `:42`,
  `VerifyingIdentity` `:45`. 46 lines total.
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityProgressStage.kt:24`
  `sealed class IdentityProgressStage(...)` — `Idle` `:38`, then **the same six names** at
  `:46`, `:60`, `:68`, `:76`, `:84`, `:92`. 118 lines total.

Both express the same identity-creation lifecycle, and the variant names match one-for-one.
The second adds `Idle` and takes constructor parameters; the first takes none.

**Why not deliberate:** two `data object`s per name means a caller cannot pass one where the
other is expected. Two hierarchies for one lifecycle is one hierarchy too many.

**I cannot tell** whether one is meant to replace the other or whether they feed different
consumers — an event bus versus a progress UI is plausible — without reading both call-site
sets, which this pass did not do.

**Glossary settles it?** No. **Gate should catch it?** No.
**Smallest correct resolution:** determine which is consumed, then make the other a
projection of it (a mapping function, not a second declaration). Effort M.

### OF-15 — `NetworkDiagnostics` names both a probe service and its result value

**Severity:** MEDIUM  **Occurrences:** 2 definitions, 2 files

- `android/app/src/main/java/com/scmessenger/android/network/NetworkDiagnostics.kt:26`
  `class NetworkDiagnostics @Inject constructor(…)` — 171 lines;
  `testNetworkConnectivity()`, `testInternetConnectivity()`, `testDnsResolution()`,
  `testCommonPorts()`, `testRelaySpecificConnectivity()`, `detectNetworkType()`,
  `detectNetworkRestrictions()`. Hilt-injected at `di/AppModule.kt:59-62`.
- `android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt:460`
  `data class NetworkDiagnostics(networkType, blockedPorts, hasInternet, hasValidated,
  isMetered, upstreamBandwidth, downstreamBandwidth, recommendedTransports)` with
  `toLogString()`.

Both are live. **Abstraction they should share: none — these are two different things and
should have two names.** The probe is a service; the result is a value. One name for both
forces every call site to disambiguate, and two already do:
`MeshRepository.kt:550` writes
`com.scmessenger.android.network.NetworkDiagnostics(context)` fully qualified, and
`MeshRepository.kt:11691` fully-qualifies the return type `...transport.NetworkDiagnostics`.

**This is the same symptom as OF-09** — a name collision forcing a fully-qualified reference
at a public boundary — which makes it a pattern on Android's public surface rather than a
one-off.

**Glossary settles it?** No. **Gate should catch it?** No.
**Smallest correct resolution:** rename the data class to `NetworkDiagnosticsSnapshot` (or
the service to `NetworkProbe`), matching the existing `getNetworkDiagnosticsSnapshot()`
already at `MeshRepository.kt:11691`. Effort S.

### OF-16 — `NetworkStatus` is declared twice inside iOS, and one copy is a stub

**Severity:** LOW  **Occurrences:** 2 definitions

- `iOS/SCMessenger/SCMessenger/Background/NotificationBackgroundProcessor.swift:175`
  `struct NetworkStatus`, used `:145`, constructed `:165`.
- `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:433` `struct NetworkStatus`, used
  `:417 networkStatus = NetworkStatus()`.

`NotificationBackgroundProcessor.swift:165` returns
`NetworkStatus(isUsable: true, description: "Simulated - reachable")` — the Background copy
is hard-coded, so it is a stub rather than a second implementation.

**Why not deliberate:** a hard-coded stub carrying the same name as a live value type is a
collision, not a mirror. **I cannot tell** whether the stub is load-bearing or leftover.

**Glossary settles it?** No. **Gate should catch it?** No.
**Smallest correct resolution:** read the Background call site; if the stub is reachable,
replace it with the real type or give it a distinct name. Effort S.

### OF-17 — `HANDOFF_AUDIT/.context_cache/` is a mirror of 15 documents, 11 of them already forked

**Severity:** MEDIUM  **Occurrences:** 15 files

Measured exhaustively against `origin/main`:

| | count |
|---|---:|
| **DIVERGED** from a `HANDOFF/` counterpart | **11** |
| **No counterpart anywhere in the tree** | **4** |
| Identical to a counterpart | **0** |

The 11 diverged pairs are all against `HANDOFF/done/`, including both `[VALIDATED]` files.
The 4 orphans are `BATCH_P1_CORE_MYCO_ROUTING.md`, `BATCH_RUST_GROUPB_DSPY_MODULES.md`,
`task_p0_android_play_readiness.md`, `task_p1_android_hardening.md`.

These are not inert: `.context_cache/P0_JSONRPC_PARITY_EXPANSION_001.md` carries 3
`StoredMessage` and 8 `isKnownRelay` occurrences, and
`.context_cache/P1_CORE_002_Mycorrhizal_Routing_Production_Wire.md` carries 1 `relay_id`.
Both sit in scan-exempt paths, so neither is judged by the gate.

**Why not deliberate:** a cache with **zero** identical entries and no refresh path is not a
cache. 11 of 15 have already forked, so a reader who finds one cannot tell which is current.

**Glossary settles it?** No — the whole directory is outside `naming-audit/`'s scope.
**Gate should catch it?** No — `HANDOFF_AUDIT/` is not in the exempt-prefix list but `.md`
is not a scan extension either.
**Smallest correct resolution:** `git rm -r HANDOFF_AUDIT/.context_cache` and gitignore it.
**This is a deletion of tracked files and needs the operator's explicit approval.**
Effort S.

### OF-18 — Four tracked build outputs under `dist/`

**Severity:** LOW  **Occurrences:** 4 files, 84,737 bytes

| file | bytes |
|---|---:|
| `dist/wasm/scmessenger_wasm.js` | 75,406 |
| `dist/index.html` | 3,648 |
| `dist/main.js` | 3,305 |
| `dist/josh_install/install-apk.bat` | 2,378 |

This is the complete set: a sweep of `origin/main` for
`(^|/)(target|build|dist|\.build)/` outside `vendor/` returns exactly these four. None is
gitignored.

**Not a naming defect.** The minified wasm bundle contains **zero** denied or grandfathered
terms. It is recorded here because it is a tracked build output, and because AGENTS.md rule 3
prohibits committing build artifacts.

**Glossary settles it?** No. **Gate should catch it?** No — AGENTS.md rule 3 does, via
`rules_check.py`, which currently checks `*.log`, `*.pid`, `*.logcat`, `target/` and
`android/**/build/` but **not** `dist/`.
**Smallest correct resolution:** `git rm --cached` the four and gitignore `dist/`. Effort S.

---

## Refuted — do not act

Proposals made during this pass and then disproved. Recorded because **RF-01 would have
broken CI** if acted on, and because two more would have wasted a reviewer's time.

### RF-01 — Delete `iOS/SCMessengerCore.xcframework/`: **WRONG, and it would break CI**

**Do not delete it.** An earlier draft recommended `git rm -r` on the grounds that "nothing
references it". That was inferred from a **single** negative grep of one `.pbxproj` file, and
it was wrong. Exhaustive sweep across `*.pbxproj`, `*.xcconfig`, `Package.swift`, `Podfile`,
all workflow files, all scripts, `.gitignore`, `.gitattributes`, `.swiftlint.yml` and every
`.md`:

- `.gitignore:81-82` ignores the **root** `SCMessengerCore.xcframework/` as a duplicate;
  `.gitignore:347-348` **explicitly un-ignores**
  `!iOS/SCMessengerCore.xcframework/` and `!iOS/SCMessengerCore.xcframework/**`. The
  tracking is a deliberate negation.
- `.github/workflows/ios-build-test.yml:178-180`, verbatim: *"Required on CI: only the
  headers of `iOS/SCMessengerCore.xcframework` are committed. The static libraries are
  excluded by the `*.a` rule in .gitignore, so a fresh clone cannot link without this
  step."* The job builds it at `:181-184` and asserts it exists at `:186-194`.
- Generated by `scripts/build_xcframework.sh`; also touched by
  `scripts/rebuild_ios_core.sh`, `scripts/verify_ios_bindings.sh`, `scripts/preflight.sh`,
  `iOS/copy-bindings.sh`.
- Excluded from linting (`iOS/.swiftlint.yml:120`) and from hygiene checks
  (`hygiene.yml:95`, `:261`).
- **A documented rule**: `docs/rules/BUILD_AND_CI.md:284-285` — *"XCFramework at
  `iOS/SCMessengerCore.xcframework/`"* — enforced by the Repository Hygiene workflow.
- **Already recorded**: `SUMMARY.md:94` lists it among generated bindings; `SUMMARY.md:310`
  states *"The `.xcframework` headers I read are a build artifact"*; `FINDINGS.md:1087`
  references it.

The framework is simultaneously **wrong in the earlier draft, deliberate, and already
recorded**. The only residual observation is that the committed copy is stale relative to
the live generated binding (151 public functions present only in `Generated/api.swift`;
7 present only in the committed header, including three real API functions). Since CI
regenerates it, that is build-artifact freshness, not a naming defect. **Not carried
forward.**

### RF-02 — `log-visualizer/public/data/unwired_functions.json` is a stale snapshot: **WRONG**

Proposed as a companion to OF-04. It contains **zero** occurrences of `isKnownRelay`,
`isBootstrapRelayPeer` or `PeerIdValidator`. An earlier draft's "238" was a bare `relay`
substring count over a different term set. **Not carried forward.**

### RF-03 — "The xcframework holds ~559 relay occurrences": **WRONG**

`HANDOFF/audit/doctrine_violation_inventory.md` §1C records ~559 and exempts the directory
with the remedy *"fix the Rust side, regenerate, done"*. Measured: **zero** occurrences of
the word `relay` or `relays` in any of the 7 files in that directory. The summary table in
the same document is also internally inconsistent — iOS total 509, exceptions ~559.

The number is wrong and the prescribed regeneration did not happen, but that is a **counts
defect in a `HANDOFF/` document**, not a naming defect. Worth a one-line correction by
whoever owns that file. **Not carried forward as a finding.**

### RF-04 — The 45 Rust duplicate type names are new: **WRONG**

A sweep of 239 compiled `.rs` files finds 40 type names declared in more than one file.
All 40 are already recorded in `FINDINGS.md` (F-02, F-03, F-04, F-06, F-08, F-16, F-18,
F-19, F-20, F-21, F-22, F-28 and the REMAINDER section). **Zero new.** Already recorded.

### RF-05 — `TransportType` defined seven times is a new finding: **WRONG**

Re-measured: 3 Rust (`core/src/transport/abstraction.rs:11`, `core/src/routing/local.rs:19`,
`core/src/relay/client.rs:24`), 2 Kotlin (`MeshEventBus.kt:163`,
`SmartTransportRouter.kt:27`), 2 Swift (`Models.swift:70`, `MeshEventBus.swift:70`), none in
`core/src/api.udl`. F-08's count is correct and GLOSSARY C-2 already owns the resolution
(`BLOCKED` on Q-3). **Already recorded.**

### RF-06 — The 30 cross-platform duplicate type names are defects: **WRONG for 27 of them**

Of the 30 names a sweep finds declared on both `android/` and `iOS/`, **27 are
per-runtime code that is deliberate**: two languages, two runtimes, two implementations, one
mirrored name. `MeshRepository`, `SmartTransportRouter`, `TopicManager`, `SettingsViewModel`,
`NearbyPeer`, `PeerDiscoveryInfo`, `DecodedMessagePayload`, `DeliveryAttemptResult`,
`MessageDedupEntry`, `RoutingHints`, `TransportIdentityResolution` and the rest are
**similar on purpose** and are recorded as such, so a later sweep does not "correct" them.

Three are not per-runtime and **are** carried forward: the `PeerEvent`/`StatusEvent` pair
(OF-12), the `TransportHealth` Android pair (OF-09), and the `android/shared/` phantoms
(OF-13).

### RF-07 — The duplicate test classes are duplicated logic: **WRONG**

Proposed as a category-4 duplicate, then measured. The two `ContactsViewModelTest` files
cover disjoint scopes (5 tests vs 7), as do the two `ReceiptUnificationTest` files (2 vs 7).
**Reclassified, not refuted** — carried forward as OF-11, a naming collision at LOW.

### RF-08 — The 85 index-only files change any verdict: **WRONG**

The 33-code-file gap between the first pass's `git ls-files` set and `origin/main` is
**37 `vendor/libp2p-swarm-0.48.0/` files** plus **4 wip-branch project files**
(`cli/src/bin/conn-fanout.rs`, `core/src/transport/per_peer_cap.rs`,
`core/tests/integration_wp1_identity_unification.rs`,
`android/app/src/test/.../service/StopStartFloodTest.kt`). Excluding them moves the Rust
duplicate count by **zero** — no vendor type name collides with a project one.

The Rust count *did* move, 45 -> 40, and **not** because of the file set: the earlier sweep
counted definitions, this one counts distinct files, and the five that dropped (`IfWatcher`,
`Input`, `LegacyReceivedMessage`, `Output`, `SwarmTaskLivenessGuard`) are each declared
several times **inside one file**. `core/src/dspy/modules.rs:16-17,105-106,164-165,217-218`
declares `type Input` and `type Output` four times apiece as associated types. Real
duplication — F-15 covers `SwarmTaskLivenessGuard` — but not cross-file, and correctly
excluded from that count.

**Noted for the operator:** `core/src/transport/per_peer_cap.rs` exists on the wip branch and
is in a rule-8 gated directory. It will need an uninvolved adversarial reviewer when it
lands. Flagging, not auditing.

---

## Corrections

Where a number changed between the working reports and this register. The final value is the
one stated in the entry.

| item | working draft | final | cause |
|---|---:|---:|---|
| `PeerID` | 33 occ / 14 files | **27 / 12** | classifier counted identifier prefixes inside strings and comments |
| `relays` | 215 / 42 | **198 / 40** | as above, plus `git grep -c` line counts |
| `TransportHealth` | 31 / 8 | **24 / 4** | as above; 4 "files" were neighbour identifiers |
| `PeerIdValidator` | 257 / 18 | **274 / 19** | index file set skipped `PeerIdValidatorCurveVectorTest.kt` (18 occurrences) |
| `isInfrastructureAgent` body | `scm-always-on-node` | **recorded verbatim at OF-08** | — |
| Rust duplicates | 45 | **40** | definitions counted instead of distinct files |
| cross-platform names | 23 | **30** | index file set plus a platform-only filter |
| within-platform names | not reported | **14** | never swept |
| `.context_cache` | "15 files, 6/6 differ" | **15: 11 diverged, 4 orphan, 0 identical** | the draft sampled 6 of 15 and generalised |
| `dist/` files | "7" | **4** | heading never matched its own list |
| `PeerID` log parsers | "11 sites, 3 files" | **14 across 4 files** | undercounted the `.sh` scripts |
| xcframework symbol diff | "44 missing / 4 extra" | **151 / 7** | the earlier regex missed indented declarations |
| `MessageDto` | 0 | **0 in code, 3 in `naming-audit/` prose** | corrected in the working draft |

Counts in `FINDINGS.md`, `GLOSSARY.md` and `SUMMARY.md` are **not** restated by this
register. Where they disagree, this register's number is the measured one, and the
correction belongs in whichever file owns that concept — not here.

---

## Suggested order, if the repository owner picks this up

1. **OF-13** — `android/shared/`. The same disease `FINDINGS.md` RC-2 already documents for
   Rust, in a language RC-2 never walked. Deciding it shrinks two duplicate names.
2. **OF-06 / OF-07** — the predicate split. A behaviour decision first, then naming.
3. **OF-01** — policy and glossary only. No code, no behaviour, warns instead of blocking.
4. **OF-09 / OF-15** — two collisions that have already forced fully-qualified public
   signatures. Small, self-contained renames.
5. **OF-08** — pairs with the `isRelay` -> `isInfraNode` rename already prepared.
6. **OF-02 / OF-03** — two one-line configuration changes.
7. **OF-10, OF-11, OF-16** — file moves and renames, do when next touched.

**No entry in items 1-7 touches `core/src/{crypto,transport,routing,privacy}/`** except
OF-06's behaviour decision if it lands there. OF-01, OF-02, OF-03, OF-10, OF-11 are
records-only or file moves and need no rule-8 adversarial review.