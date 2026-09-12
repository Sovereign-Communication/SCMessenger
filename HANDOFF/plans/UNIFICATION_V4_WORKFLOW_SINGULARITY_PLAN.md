# Unification V4 -- Workflow Singularity Plan

Status: PROPOSED -- audit complete, gate built and calibrated, no runtime code
touched. Awaiting CEO/CTO ratification of the four decisions in section 7.
Created: 2026-09-11 (audit + gate), Freebuff audit seat.
Authority: subordinate to `SHIP_PLAN.md` for sequencing and scope. This file is
the single index for the unification class; the three prior unification
documents remain as detailed findings archives (section 9).
Basis: `docs/NATURE_INSPIRED_MESH_PHILOSOPHY.md` (the philosophy cannon),
`AGENTS.md` (contract), `SHIP_PLAN.md` section 6 (current truth).

Evidence standard: every claim carries the command or `file:line` that produced
it, in this session. Two claims inherited from tickets did NOT survive contact
with the code and are corrected in section 1.10.

## How to read this

- Section 1 is the finding. It is the reason the plan exists.
- Section 2 is the fix: three primitives, not a list of duplicates.
- Section 3 is the mechanism that makes it stick: the gate, which is already
  written and already red.
- Section 4 is the boundary: what is in, what is explicitly out.
- Section 5 is the path. Section 7 is what the CEO/CTO must decide or run.

---

## 0. Why this is doctrine, not tidiness

| Cannon pillar | What it requires of the code |
|---|---|
| 1. Fractal topology | One artery (Rust core) with thin capillary platforms. Platform code re-implementing core policy is a capillary impersonating an artery |
| 2. Invariant terminal units | The edge node's state must not diverge by platform. A fact in three encodings is not one fact |
| 3. Reflection suppression | No echo, no loop, no re-entry. Self-dial storms and self-echo receipts are literal reflections |
| 4. Sublinear scaling | Each new platform must add sublinear work. Duplicated policy makes every node more expensive |
| 5. Superlinear output | Density accelerates only if nodes speak one protocol |

Two live defects are pitch-perfect reflections and were filed as unrelated bugs:
the self-dial storm (`SHIP_PLAN.md` I-06) and the receipt stampede where Windows
acked its own looped-back outbound (V3 section 1, D2). The cannon named the
class; the plan treats reflections as one invariant.

---

## 1. Audit: nine recurring classes

The operator's report -- "we fix something, it holds an iteration or two, then
it comes back" -- is measurable, and it is not nine bugs. It is nine
**duplicated authorities**, each fixed at the instance visible at the time. The
fix was always correct and always local; nothing prevented the next instance.

### 1.1 C1 -- Identity has three interchangeable encodings

`public_key_hex` (canonical), `identity_id` = `hex(blake3(pubkey))` (also 64
hex, also 32 bytes), and `libp2p_peer_id` (base58). Two are indistinguishable to
`hex::decode`.

- `#254`, `#255`, `e3b47278`, `9b5b0f2d`, `2360a3b6`, `13aabcd2`, `6e228776` --
  seven commits closing and re-closing the identity-poisoning class.
- `#244` coalesced history across pubkey/identity_id flavors; `#248` was the
  same fix for `mobile_bridge` history. Adjacent PRs, second call site.
- `c334bd8a`, `cd2375b7` -- canonicalization tests lagging the last unification.
- Measured: 613 sites mention `identity_id`/base58 conversion in core+cli;
  `core/src/crypto` has 0 (crypto is already clean).
- Android uses the loose heuristic `PeerIdValidator.isLibp2pPeerId` as a type
  test at 10 sites in `MeshRepository.kt`.

Why it returns: the two 64-hex forms type-check against each other, so no
compiler objects; every fix is a runtime compensation at a call site.

### 1.2 C2 -- Delivery truth has two definitions, and the ship gate forbids one

- Transport ACK clears the outbox at 6 CLI/1 wasm sites plus the core receipt
  path (`core/src/iron_core.rs:3630`). `mark_message_sent` itself is
  `iron_core.rs:1129`.
- `SHIP_PLAN.md` D4/D6/D7 standard: receiver-side decrypt + durable history +
  receipt, "NOT transport ACKs".
- `HANDOFF/todo/P0_SEND_CRYPTO_FAILS_VS_DELIVERED_2026-08-30.md` records the
  consequence: sender reads `delivered=true` while the payload never verified.
- Third instance: `IronCore::send_message_status` (`iron_core.rs:1169`) calls
  `mark_message_sent` immediately after `prepare_message`, no send at all; zero
  production callers outside generated bindings.

Six rounds, one invariant, three platforms: Aug-10 root cause -> Aug-24
"FIXED" -> Aug-25 live RCA reopens (Android never emits an app-level ack) ->
Aug-27/28 V3 D1/D2/D3 + R1/R2 -> Aug-30 BLE-truncation P0 -> `#250` then `#251`.

Why it returns: "delivered" has no owner; the clearing rule lives in eight
places that disagree about the word.

### 1.3 C3 -- One fact, two stores; one file, two writers

- Fixed and cited as the model: CLI `peers.json` unified into the core ledger by
  `#262` (`cli/src/ledger.rs:5`).
- Still live: `MeshRepository.kt:1141` and `MeshRepository.swift:637` each
  construct their own `LedgerManager(storagePath)` over the store `IronCore`
  owns, with independent `load()`/`save()` (`kt:1150`, `kt:7086`). Filed
  2026-07-26 (`CODEBASE_UNIFICATION_PLAN.md` Rank 4); still true today.
- Android decides its own retry outcome at `MeshRepository.kt:247`
  (`decidePendingOutboxFlushAction`).

### 1.4 C4 -- Transport decision authority is split three ways

Core (`core/src/transport/escalation.rs`, instantiated in
`core/src/mobile_bridge.rs`), Android (`transport/SmartTransportRouter.kt`,
`transport/TransportHealthMonitor.kt`), and Android again
(`MeshRepository.kt:1523/5766/6533/10466/10530/10610` building its own relay and
bootstrap address priority). Recorded as a defect in July; still true.

### 1.5 C5 -- Gates that cannot fail, or are not run

- `scripts/check_wiring.py` works and passes (`[OK]`, run this session). It is
  the model -- but it covers Android reachability only.
- `docs_sync_check.sh` PASSES on HEAD (run this session), after being red for
  weeks until `#260`.
- Repository-hygiene whitespace was fixed by `7f369f50` and regressed
  (`SHIP_PLAN.md` S1-2): fixed twice, gated neither time pre-push.
- There is no gate at all for identity duplication, second writers, or doc
  claims. Section 3 supplies one.

### 1.6 C6 -- The workflow layer has N sources of truth

Measured: 1,781 tracked `*.md` against 131,033 lines of Rust and 50,919 of
Kotlin; `HANDOFF/todo/` holds 20 files against a target of under 10; ~15
competing `START_HERE` / `LAUNCH_NEXT` / `NEXT_*` / `ORCHESTRATOR_TAKEOVER_*`
documents, plus `CTO_STATE.md`'s own `RESUME HERE` sections.

Hot spots confirm where these classes surface: last 200 commits touched
`MeshRepository.kt` 27 times, `DashboardViewModel.kt` 18, `PeerListScreen.kt`
16, `DashboardScreen.kt` 15.

### 1.7 C7 -- Eleven canonical documents claim dispatch authority

This is the workflow-layer version of the same disease, and it is the most
expensive one, because it makes agents do the wrong thing *on instruction*.
The gate found, at current HEAD:

| Location | Text |
|---|---|
| `AGENTS.md:74` | rule 10: "Backlog order is `HANDOFF/todo/_QUEUE.md`" |
| `AGENTS.md:280` | "`_QUEUE.md` — live dispatch order" |
| `DOCUMENTATION.md:88` | "Dispatch queue ... live pick list" |
| `HANDOFF/CTO_STATE.md:113,116` | "then `API_RESET_EXECUTION_CHARTER_2026-08-28.md` (the live queue)" |
| `HANDOFF/todo/_QUEUE.md:237` | "This file is the LIVE pick list" |
| `HANDOFF/plans/FARM_FINAL_PLAN.md:9` | "the queue (`_QUEUE.md`) remains the live pick list" |
| `docs/DOCUMENT_STATUS_INDEX.md:47` | "`_QUEUE.md` ... Live dispatch order" |
| `HANDOFF/SESSION_HANDOFF_2026-07-13_farm_v1_backlog.md:5` | "Live pick list" |
| `HANDOFF/gpt/GPT_PLANNING_040_050.md:50` | "`_QUEUE.md` -- live dispatch order" |

Meanwhile `SHIP_PLAN.md:9` says it is the only execution queue, and
`HANDOFF/freebuff/README.md:6` implements that. So the **contract itself**
(`AGENTS.md`, rule 10) points at the superseded queue. A fresh session following
the rules arrives at the wrong page -- the same failure the delegation-script
case caused on 2026-08-15.

### 1.8 C8 -- Two delegation entry points; harness-content plan never executed

- `scripts/delegate.py` (11,483 B, 8 referencing files) and
  `scripts/delegate_task.py` (41,599 B, 73 referencing files) both exist as
  independent implementations. `_QUEUE.md` documented the wrong one once
  already (`UNIFY_CODEBASE_DECONFLICT.md` C1).
- The 2026-08-15 C4/C5 plan (move `.kiro/specs` and `.mimocode/plans` to
  content dirs, add thin adapters) was never executed: `.kiro` still holds 32
  tracked files, `.mimocode` 7, `.codex` 16, `.bob` 6, `.agents` 6.

### 1.9 C9 -- Second definitions and second stores, verified

- `pending_outbox.json` is a platform-owned outbox store on both phones:
  `MeshRepository.kt:565` (read/write at :8569-8697) and
  `MeshRepository.swift:255` (read/write at :5763-5776), alongside core
  `core/src/store/outbox.rs`. Two nodes keep two outboxes for one fact.
- `RetryPolicy` is defined **twice in core**, and the copy that claims to be
  canonical is the dead one: `core/src/lib.rs:189-198` says "This is the ONLY
  place retry policy is defined. All platforms ... use this struct", while
  `core/src/store/outbox.rs:822` defines a second one, and the crate root
  re-export (`lib.rs:100`) -- the one CLI actually uses (`main.rs:3975`) --
  points at `store::outbox`. The promised centralization exists in name only.
- Android additionally re-decides the policy (`MeshRepository.kt:247`).
- `parse_transport_type` (`core/src/iron_core.rs:110-125`) ends in
  `_ => TransportType::BLE`: any unrecognized or empty transport string is
  silently scored as a proximity-class transport. This sits on the routing feed
  that `#263`/T4 just wired live.

### 1.10 Corrections to the record (two ticket premises did not survive the code)

Recorded here rather than in a new ticket, because the point is that the plan
was tested against the artifact first.

1. **`V040_T10` premise is false.** It states `scripts/ffi_surface.sh` "passes
   when it checks nothing". It does not: it has explicit failure branches
   (`ffi_surface.sh:78-81` Kotlin, `:103-106` Swift), and running it with
   bindings absent gives `WARN: ... not generated yet` and **exit 1** (ran this
   session; real exit code captured without a pipe). The earlier reading stopped
   at line 58, before the `else` branches. T10 should be retired or rewritten
   with a true premise; the gate's D check now reports `[OK]` for this file,
   which is the correct verdict.
2. **The "WebSocket peers are mis-scored as BLE" premise is false.**
   `parse_transport_type` maps `"ws" | "wss"` to TCP explicitly
   (`core/src/iron_core.rs:116`). The real defect is the catch-all default to
   BLE (C9), which affects every unrecognized string, not WS.

Also confirmed landed, so the queue must not re-dispatch them: T1 `#266`,
T2 `#262`, T4 `#263`, T5 `#260`, T8 `#271`, T12 `#264`, T13 `#267-269`,
T14 `#269/#270`. The freebuff queue index still lists T1/T2/T4/T8/T12/T13 as
open -- the index is stale (section 7, decision D4).

---

## 2. The fix: three primitives

Every prior unification enumerated duplicates and fixed them. Enumeration does
not scale and does not persist. Fix the ability to *express* the duplicate.

### Primitive A -- ONE TYPE per identity

- `PublicKeyHex`, `IdentityId`, `PeerIdBase58` newtypes in core; derive at the
  edges; map to `String` at the UniFFI boundary so bindings stay simple.
- `core/src/crypto` accepts only `PublicKeyHex` (already true in practice: 0
  `identity_id` references), codifying the boundary rather than changing it.
- `PeerIdValidator` becomes strict base58 and stops being used as a type test.
- Scope guard: wrap the **boundary** (delivery, ledger, contacts, outbox) and
  let the compiler find the rest. Do not rewrite 613 sites in one pass.

### Primitive B -- ONE WRITER per fact

- Remove platform `LedgerManager` construction; expose one accessor on
  `IronCore` (pattern exists: `relay_bootstrap_manager_handle`).
- Retire `pending_outbox.json` on both phones; drive the core outbox through one
  FFI surface.
- Delete the duplicate `RetryPolicy`; keep exactly one definition and point the
  crate root at it.
- Delete the dead `send_message_status` FFI export.

### Primitive C -- ONE TRUTH per message

```
queued --(transport wrote)--> sent --(signed app receipt verified)--> delivered --> read
```

- `sent` is the honest name for what a transport ACK proves; it stops the redial
  storm (which is what R1/R2 actually needed) without claiming delivery.
- Deletion happens only at `delivered`. Transport ACK must not delete the entry
  and must not set `delivered`. This is what makes D4 scoreable.
- One ack format: the signed encrypted receipt envelope. Platforms render; core
  decides. `parse_transport_type` gets an explicit `Unknown` variant instead of
  defaulting to BLE.

---

## 3. The mechanism: `scripts/singularity_check.py` (built, calibrated, red)

Five executable invariants. Validated against HEAD; false positives were removed
by narrowing rules, never by widening an allowlist.

| Check | Invariant | A new duplicate fails when |
|---|---|---|
| A one-identity | one encoder per form; the type boundary exists | a second non-delegating Rust encoder for one identity form appears |
| B one-writer | core owns stores | a platform constructs `LedgerManager`, keeps `pending_outbox.json`, or a second retry policy definition appears |
| C one-truth | core decides transitions | `mark_message_sent` is called outside `core/src/iron_core.rs` |
| D gates-can-fail | a gate with empty-able input can still fail | a `[[ -n "$VAR" ]]` guard over `find`/`ls` output has no `else`/failure branch |
| E doc-claims | canonical docs match code | more than one doc claims queue authority; `AGENTS.md` names `_QUEUE.md` without `SHIP_PLAN.md`; a version claim disagrees with `Cargo.toml`; markdown count exceeds the ceiling |

Run: `python3 scripts/singularity_check.py` (report mode, exit 0) or
`--mode strict` (exit 1 on any FAIL). `--json` for CI.

**RED baseline at HEAD 2026-09-11: 23 failures -- A 0, B 5, C 7, D 0, E 11.**
D is genuinely green: `ffi_surface.sh` and the other guarded scripts can fail
(section 1.10).

**The rule that makes it stick:** a ticket closing any class in section 1 may
not close without its invariant ID and a passing gate. That converts "fixed for
an iteration" into "fixed", and it lands the gate with the fix rather than after
it.

---

## 4. Boundary -- what is in, and what is out

### IN (this plan owns)

| # | Item | Why |
|---|---|---|
| B1 | One delivery transition (Primitive C) | Required by D4/D6/D7; the only contradiction with the ship gate's own evidence standard |
| B2 | One retry policy + delete dead FFI method | Two definitions where one is documented as the only one |
| B3 | One outbox per fact; remove platform `pending_outbox.json` | Second store on 2 of 3 nodes |
| B4 | Remove platform `LedgerManager` (accessor on `IronCore`) | Second writer on 2 of 3 nodes |
| B5 | `parse_transport_type` explicit unknown | Silent proximity classification on the live routing feed |
| B6 | Doc/queue authority: one entry chain, `todo/` under 10 | C7: the contract currently points at the superseded queue |
| B7 | Wire the gate into CI + finalize path | C5: nothing enforces the invariants |
| B8 | Identity boundary types for delivery/ledger/contacts | C1 root cause; staged, boundary only |

### OUT (explicitly, with the reason)

| Item | Why it is out |
|---|---|
| iOS parity work | Tier C until v0.5.0 (`SHIP_PLAN.md` section 4) |
| Farm drills, KMP, meeting mode, PQC-14 | Post-tag by standing decision |
| Full 613-site identity rewrite | Boundary first; the compiler finds the rest |
| New orchestration tooling, dashboards, visualizers | `SHIP_PLAN.md` section 4 |
| Rewriting the three prior unification plan docs | Archive and cross-reference; they are evidence |
| Transport decision authority (C4) | Real, but wave 2 -- after the tag; it touches `core/src/transport/` |
| Changing CI required checks or branch protection | Owned by the CI lane; the gate is additive |
| Editing `.codebuff_deploy/aws/launch.py` (I-03) | Untracked, owned by another session |

---

## 5. Path: waves, packets, acceptance

One theme = one branch = one revert. Each packet below is paste-ready for the
freebuff queue once the CEO/CTO approves it.

### Wave 0 -- zero runtime risk (docs, CI, gate; free lanes)

| Packet | Work | Anchors | Gate | LoC |
|---|---|---|---|---|
| W0-1 | Queue-authority fix: make `SHIP_PLAN.md` the named queue everywhere | `AGENTS.md:74,280`; `DOCUMENTATION.md:11,88`; `CTO_STATE.md:113,116`; `_QUEUE.md:237`; `FARM_FINAL_PLAN.md:9`; `DOCUMENT_STATUS_INDEX.md:47`; `SESSION_HANDOFF_2026-07-13:5`; `GPT_PLANNING_040_050.md:50` | E | 20-40 |
| W0-2 | Queue-index reconciliation: move landed T1/T2/T4/T5/T8/T12/T13/T14 to `done/` with PR numbers; retire or rewrite T10 with the corrected premise | `HANDOFF/freebuff/README.md` table | E | 20-30 |
| W0-3 | `HANDOFF/todo/` 20 -> under 10 with evidence headers | SHIP_PLAN G5 | E ceiling + docs_sync | doc moves |
| W0-4 | Wire `singularity_check.py --mode strict` into CI and the finalize path | `.github/workflows/`, `docs/rules/` | D | 10-20 |
| W0-5 | Markdown ratchet: ceiling 1,784, may only decrease | `singularity_check.py:MD_FILE_CEILING` | E | 1 |

### Wave 1 -- required by D4/D6/D7 (Rule-8 review mandatory)

| Packet | Work | Anchors | Gate |
|---|---|---|---|
| W1-1 | One truth: core owns the transition. Introduce one core entry point for "transport wrote"; move the 8 call sites into it; `delivered` only on verified receipt | `core/src/iron_core.rs:1129,3630`; `cli/src/main.rs:2925,3121,3989,4282`; `cli/src/api.rs:889`; `cli/src/api_axum.rs:299`; `wasm/src/lib.rs:1173` | C |
| W1-2 | Delete the dead `send_message_status` export (no production callers; would clear before delivery) | `core/src/iron_core.rs:1139-1177` | C |
| W1-3 | One retry policy: keep `store/outbox.rs`, delete the `retry_policy` module, keep the crate-root re-export pointing at the survivor | `core/src/lib.rs:100,189-198`; `core/src/store/outbox.rs:822`; used at `cli/src/main.rs:3975` | B |
| W1-4 | One outbox per fact: retire `pending_outbox.json` on both phones | `MeshRepository.kt:565,8569-8697`; `MeshRepository.swift:255,5763-5776` | B |
| W1-5 | One writer: remove platform `LedgerManager`; add an `IronCore` accessor | `MeshRepository.kt:1141,1150,7086`; `MeshRepository.swift:637` | B |
| W1-6 | Drop Android's duplicate retry decision | `MeshRepository.kt:247-280` | B |
| W1-7 | `parse_transport_type`: explicit `Unknown`, never default to BLE | `core/src/iron_core.rs:110-125` | C |
| W1-8 | Identity boundary types at the delivery/ledger/contacts surface | `core/src/identity/{keys.rs,mod.rs}`; UniFFI layer | A |

### Wave 2 -- after the tag

Primitive A full rollout; transport decision authority (C4); harness-content
consolidation (C8); `delegate.py`/`delegate_task.py` collapse.

### Acceptance: one 3-node soak, seven named checks

Fleet: Windows CLI node, Android handset, AWS cloud node. Scored from all three
logs, never from one.

| # | Invariant |
|---|---|
| W1 | No self-dial, no self-receipt echo |
| W2 | One mesh row per real peer; self excluded; no phantoms |
| W3 | `sent` on transport write; `delivered` only after a verified signed receipt |
| W4 | No outbox retry growth while a receipt is outstanding |
| W5 | One writer per store (write through one handle, read through the other) |
| W6 | Node rejoins unaided after an address change |
| W7 | `singularity_check.py --mode strict` exits 0 |

---

## 6. What "unified" means, concretely

`singularity_check.py --mode strict` exit 0 plus W1-W7 is the definition. It is
binary, it is runnable by any agent, and it cannot be satisfied by writing a
document. That is the difference between this plan and the three before it.

---

## 7. Handoff to the CEO/CTO seat

### Decisions requested (do not improvise; `AGENTS.md` rule 9)

| # | Decision | Why it blocks |
|---|---|---|
| D1 | Approve the `sent` / `delivered` split (Primitive C) | It preserves R1/R2's intent while restoring D4's standard. Without it, W1-1 conflicts with merged `0c75bf1a` |
| D2 | Confirm platform `LedgerManager` + `pending_outbox.json` removal is in v0.4.0 scope | Smaller than T2 was; it is the last second writer on the receipt path |
| D3 | Confirm the pre-tag boundary vs `SHIP_PLAN.md` section 4 | Wave 0 is doc/CI only; wave 1 is gated core work |
| D4 | Retire or rewrite `V040_T10` (premise false, section 1.10) and correct the freebuff queue index | Prevents a wasted operator paste cycle |
| D5 | Authorize wiring `singularity_check.py --mode strict` into CI | It is red by design until wave 1; wire after wave 1, or wire now as non-required |

### Commands to reproduce this audit (no state changed)

```bash
python3 scripts/singularity_check.py            # 23 failures, full list
python3 scripts/singularity_check.py --json      # machine-readable
bash scripts/ffi_surface.sh > tmp/o.txt 2>&1; rc=$?; cat tmp/o.txt; echo $rc   # exit 1
bash scripts/docs_sync_check.sh                  # PASS
python3 scripts/check_wiring.py                  # PASS
git log -200 --name-only --format= | grep -E '\.(kt|rs)$' | sort | uniq -c | sort -rn | head
```

### Do not

- Do not re-dispatch T1/T2/T4/T5/T8/T12/T13/T14 -- landed (section 1.10).
- Do not dispatch T10 as written -- its premise is false.
- Do not "fix" `parse_transport_type` for WS -- WS already maps to TCP; fix the
  catch-all.
- Do not merge anything touching `core/src/{crypto,transport,routing,privacy}`
  without a fresh adversarial APPROVE (Rule 8) -- wave 1 items W1-1, W1-7 and
  W1-8 are in scope for that gate.
- Do not raise `MD_FILE_CEILING`.
- Do not treat a green CI as evidence the invariants hold; CI does not run this
  gate until D5.

---

## 8. Non-goals

No new orchestration tooling, dashboard, or visualizer pre-tag. No iOS work. No
farm drills. No big-bang rewrite. No re-litigation of `SHIP_PLAN.md` sequencing
or of settled operator decisions.

## 9. Lineage (findings archives, not superseded facts)

- `HANDOFF/todo/CODEBASE_UNIFICATION_PLAN.md` (2026-07-26) -- ranked duplication
  inventory; its Rank 4 is C3/B4 here.
- `HANDOFF/todo/UNIFY_CODEBASE_DECONFLICT.md` (2026-08-15) -- phase discipline
  (find, then prune, then escalate) adopted in sections 4-5.
- `HANDOFF/plans/UNIFICATION_V2_RESULTS_PLAN.md` (2026-08-26) -- pillars P0-P5;
  P0 -> Primitive A, P1 -> Primitive C.
- `HANDOFF/plans/UNIFICATION_V3_DELIVERY_CONVERGENCE_PLAN.md` (2026-08-28) --
  D1-D3/R1-R2 are correct and landed; R1/R2's clearing rule is what Primitive C
  reconciles.
- `scripts/singularity_check.py` -- the executable form of this document.
