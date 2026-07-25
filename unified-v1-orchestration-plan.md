# SCMessenger v1.0.0 Unified Orchestration + Full Backlog Plan

Status: Active — approved for implementation
Authored: 2026-07-17
Authority: Additive to `HANDOFF/V1_0_0_EXECUTION_PLAN.md` and `HANDOFF/todo/_QUEUE.md`.

> Note (2026-07-20): the orchestration unification described in Sub-Task 9 has since completed. There is now ONE command `/orchestrate` over the master protocol `docs/ORCHESTRATION.md`; the old per-backend commands are archived under `.claude/archive/commands/`. The backlog content below stands; treat any per-command references (scmorc/scmqwen/gemini-orchestrator/swarm) as historical -- they map to backends per `docs/ORCHESTRATION.md` Section 5.

---

## Context Summary

**Hermes session (today):** Ran on a separate AWS EC2 instance with no commit
access to this repo. Changes exist only on that machine's working tree, never
committed. Must be reconstructed from `HERMES_FARM_AUDIT.MD` and applied here.

**All lanes active:** Qwen/DashScope, Groq, OpenRouter (Morph Lite), Fusion Lite,
and Gemini all have working keys. Every task below is delegated — zero native
Claude implementation.

**Key findings from Hermes farm-sim (7-node Docker topology):**
- Network plumbing PASS: 7 containers, 3 isolated networks, custody working
- Application delivery FAIL: 191 envelope decode errors — root cause traced to
  custody dispatch missing `wrap_in_drift_frame()` call
- Hermes also identified mesh message ordering bug (DriftFrame unwrap must
  precede RelayMessage parse) — already correct in THIS repo (line 2701 is after
  line 2651, which is the correct order). No action needed for that fix.
- Hermes reverted its own size-limit attempt (libp2p 0.56 does not expose
  `with_codec()` API) — no net change to behaviour.rs needed.
- A7 (NetworkError observability) and A8 (onion gating) already landed in this
  repo and verified via grep: `OnionRoutingDisabled` at `core/src/lib.rs:72`.

---

## 2026-07-17 AUDIT AMENDMENT (authoritative over sub-task status boxes)

Post-facto audit of this plan + the 07-16/17 swarm runs. Machine truth:
`scm_v1_farm_queue.jsonl`; human dispatch order: `HANDOFF/todo/_QUEUE.md`
(status-correction header); operational lessons: `docs/ORCHESTRATION.md`
Section 9. The `[ ] pending` boxes below were never maintained -- trust this
table.

| Sub-Task | True status 2026-07-17 | Evidence |
|----------|------------------------|----------|
| 1. Custody DriftFrame wrap | DONE | 82adf735; hardened + adversarial PASS-WITH-NOTES in 30b78eea |
| 2. A7/A8 commit + close | DONE | a31dcdbf, edac65ea |
| 3. F1 convergence test | DONE | 908e4b13; A-02 closed via 4274593f |
| 4. Site-3 outbox flush | DONE | 6d884f97 + ecafd504; CRITICAL_OUTBOX ticket closed to done/ |
| 5. A3 Android retry suppression | DONE | A-01 closed via 4274593f/4567ece0 |
| 6. E-01a constraints analysis | SUPERSEDED | E-00 (below) reframes the whole E-01 family; ratchet is unreachable from the production path |
| 7. Wave D batch | PARTIAL | D-01 done (908e4b13). D-02/D-04/D-05 OPEN (Qwen output reverted: wrong-dir gradle verify, vacuous success, nonexistent testRunner property). D-03 BLOCKED-PLATFORM (iOS on Windows) |
| 8. Wave T residuals | OPEN | C-06/T-04 moved back to todo/ (8da8cc90): C-06 was 212 lines of simulated/mock code behind a compile-only gate. C-05 OPEN (xcodebuild on Windows). T-02/T-03 OPEN |
| 9. Wave Z infra | PARTIAL | Z-01 done; docs/ORCHESTRATION.md + Section 9 lessons landed 07-17; Z-02/Z-03 OPEN |
| 10. C-01 root-cause + H-03 | OPEN | H-03 is a HUMAN sign-off gate |
| 11. Wave B freeze docs | OPEN | WAVE_B_FREEZE_STATUS.md still in todo/ |

Corrections to the Context Summary above:

- **with_codec claim was wrong.** "libp2p 0.56 does not expose with_codec()"
  is false -- 0.56 puts the ceiling on the Codec via `with_codec`, not on
  the Config. Hermes's size-limit intent was therefore ported after all:
  4 MiB req / 16 MiB resp on /sc/message, /sc/relay, /sc/ledger-exchange
  (30b78eea); address-reflection and registration deliberately left at
  defaults (review F5 -- pure attack surface).
- **"All lanes active" is not true as of 07-17.** Live: groq flash, qwen
  flash, ollama gpt-oss:20b-cloud, openrouter morph (paid). Down: openrouter
  `:free` (429 pool saturation), ollama qwen3.5:397b-cloud (403), gemini
  (no key file -- agy CLI auth does not cover delegate_task.py). Router
  skips keyless/cooling lanes automatically IF every dispatch is recorded
  (`lake_route.py --record`).
- **NEW CRITICAL -- E-00:** the ratchet/PQ subsystem is dead code on the
  production path (`prepare_message_internal`/`receive_message` call bare
  legacy encrypt/decrypt; every real message today has zero forward secrecy
  and zero PQ protection). Ticket:
  `HANDOFF/todo/CRITICAL_RATCHET_SUBSYSTEM_NOT_WIRED_INTO_IRONCORE.md`.
  OPERATOR GATE: architecture decision required before dispatch; it blocks
  E-01b/c, E-02, E-03, E-04, B-01. This is the true root of the E-01 defect
  family and contradicts PQC_08's "verified" inventory.
- **Swarm quality gate (new, mandatory):** compile-only verify is not
  completion. Commits 71d02d4d/e298e9bf were reverted (23960b35/8da8cc90).
  Rules: `--mode diff` always; exit 3 = vacuous = failed; grep diffs for
  simulate/mock/placeholder; batch runners never commit or move tickets;
  one build at a time on Windows. Full list: docs/ORCHESTRATION.md Section 9.

---

## Sub-Tasks

---

### Sub-Task 1: Apply Hermes Fix-1 and commit (custody DriftFrame wrap)

**Intent:** The single root cause of 191 farm-sim envelope decode failures.
`dispatch_pending_custody_for_peer` sends raw `envelope_data` via the messaging
protocol; the receive handler expects all payloads to be DriftFrame-wrapped.
One-line change. Touches `core/src/transport/` — mandatory adversarial review.

**Expected Outcomes:**
- `core/src/transport/swarm.rs` line 1241: `custody.envelope_data.clone()`
  replaced with `wrap_in_drift_frame(&custody.envelope_data)`
- `CARGO_INCREMENTAL=0 cargo check --workspace -q --message-format=short` passes
- Adversarial review PASS on file
- Committed: `fix(transport): wrap custody relay dispatch in DriftFrame`

**Todo List:**
1. Dispatch **Morph Lite** (`morph/morph-v3-fast` via OpenRouter, `scripts/morph_lite.py`):
   single-file, 1-line change in `core/src/transport/swarm.rs` — exactly the
   Morph Lite use case. Prompt: change line 1241 from `custody.envelope_data.clone()`
   to `wrap_in_drift_frame(&custody.envelope_data)`. Include lines 1185-1260 as context.
2. Apply the returned patch
3. Run `CARGO_INCREMENTAL=0 cargo check --workspace -q --message-format=short`
4. Dispatch adversarial review to **Qwen THINK** (`qwen3-235b-a22b-thinking-2507`
   via `scripts/delegate_task.py --provider qwen --model qwen3-235b-a22b-thinking-2507`):
   feed the 4-line diff + 40 lines of context; probe for races, double-delivery,
   custody state corruption; compare with the WASM direct-send pattern at line ~4924
5. If PASS: commit and move to Sub-Task 2
6. If FAIL: fix the specific finding, re-review, then commit

**Relevant Context:**
- [`dispatch_pending_custody_for_peer`](core/src/transport/swarm.rs:1185) lines 1185-1260
- [`wrap_in_drift_frame`](core/src/transport/swarm.rs:427) already exists at line 427
- `HERMES_FARM_AUDIT.MD` lines 136-147 (exact diff from farm session)
- `.claude/rules/security.md` (transport adversarial review gate)

**Status:** [ ] pending

---

### Sub-Task 2: Commit A7/A8 if uncommitted; close tickets

**Intent:** Hermes's RESULT line confirms A7 (NetworkError observability) and A8
(onion gating) are done and cargo-check verified. Grep confirms the changes are
in this repo's files. Check commit status and seal the work.

**Expected Outcomes:**
- A7/A8 changes committed (or confirmed already committed)
- `HANDOFF/todo/A-07_NETWORKERROR_OBSERVABILITY.md` and A-08 tickets moved to done
- `_QUEUE.md` updated
- If A7 diff touches `mobile_bridge.rs` dial path: adversarial review dispatched

**Todo List:**
1. Run `git status --short` to check for uncommitted changes in
   `core/src/lib.rs`, `core/src/api.udl`, `core/src/mobile_bridge.rs`,
   `core/src/iron_core.rs`, `scripts/delegate_task.py`
2. If uncommitted: run `CARGO_INCREMENTAL=0 cargo check --workspace -q
   --message-format=short`; then dispatch **Qwen THINK** adversarial review on
   the mobile_bridge.rs dial-error section (it touches transport-adjacent code)
3. Commit: `fix(transport): NetworkError observability + onion routing gate (A7/A8)`
4. Create A7/A8 ticket files if absent; move to `HANDOFF/done/`
5. Mark A7/A8 DONE in `_QUEUE.md`

**Lane:** Orchestrator-local verification + Qwen THINK if review pending
**Status:** [ ] pending

---

### Sub-Task 3: Fix F1 integration_ledger_convergence.rs

**Intent:** This test has been uncommitted and failing since 2026-07-13. The
diagnosis is precise: `swarm2.dial(node1_addr)` fails with "no addresses for
peer" because the `Multiaddr` from `ListeningOn` lacks the `/p2p/<peer_id>`
suffix. Reference pattern is `integration_nat_reflection.rs`.

**Expected Outcomes:**
- Test passes with `--include-ignored`
- Committed: `fix(tests): append p2p peer_id to ListeningOn addr in ledger convergence test`
- F1 marked DONE in `_QUEUE.md`

**Todo List:**
1. Dispatch **Morph Lite** (`morph/morph-v3-fast`): single-file test fix.
   Provide `core/tests/integration_ledger_convergence.rs` (full content) +
   the relevant ~30 lines from `integration_nat_reflection.rs` showing the
   working dial pattern + the diagnosis: append
   `Protocol::P2p(keypair1.public().to_peer_id())` to the multiaddr before dialing.
2. Apply the patch; run:
   `CARGO_INCREMENTAL=0 cargo test -p scmessenger-core --test integration_ledger_convergence -- --include-ignored`
3. If pass: commit
4. If still failing: escalate to **Qwen CODER** (`qwen3-coder-plus`) with the
   full error output; one retry permitted before operator escalation

**Lane:** Morph Lite first; Qwen CODER fallback
**Status:** [ ] pending

---

### Sub-Task 4: Implement Site 3 — outbox flush-on-reconnect loop

**Intent:** The outbox has no retry/flush mechanism at all (confirmed: `enqueue`,
`drain_for_peer`, `peek_for_peer`, `record_attempt`, `remove` exist as pure data
primitives but nothing calls them on reconnect). A message queued while the
recipient is offline stays at `attempts=0` forever. Build the flush loop in the
swarm event loop triggered on `ConnectionEstablished`.

**Expected Outcomes:**
- On `SwarmEvent::ConnectionEstablished { peer_id, .. }`: scan outbox for
  entries whose recipient matches the newly-connected peer; attempt delivery
  via the same direct-send path established by the Site 2 fix; call
  `record_attempt` and `remove` on success or failure
- `cargo check --workspace` passes
- Fusion Lite 3-panel triangulation PASS (delivery logic = WS-A class, mandatory)
- Adversarial review PASS (touches `core/src/transport/`)
- Committed: `fix(core): outbox flush-on-reconnect (Site 3)`
- `CRITICAL_OUTBOX_NEVER_FLUSHES_DESPITE_ACTIVE_CONNECTION.md` moved to done

**Todo List:**
1. Read `core/src/store/outbox.rs` in full (small file; needed by the worker)
2. Read `core/src/transport/swarm.rs` around `ConnectionEstablished` handler
   (find the line number via grep: `ConnectionEstablished`) — read ~60 lines
3. Dispatch **Qwen CODER** (`qwen3-coder-plus`, 128K context) via
   `scripts/delegate_task.py --provider qwen --model qwen3-coder-plus`:
   embed `outbox.rs` full content + the `ConnectionEstablished` handler window
   + the Site 2 direct-send pattern (lines ~4380-4430 in swarm.rs) + the
   task spec from `CRITICAL_OUTBOX_NEVER_FLUSHES_DESPITE_ACTIVE_CONNECTION.md`
4. Apply returned patch; run `CARGO_INCREMENTAL=0 cargo check --workspace -q`
5. Dispatch **Fusion Lite** 3-panel triangulation (delivery logic rule):
   `python scripts/fusion_lite.py --prompt-file tmp/site3-review.md --panel 3 --max-cost 0.01`
   Panel prompt: review the diff for double-delivery if peer reconnects rapidly,
   outbox starvation if multiple peers reconnect simultaneously, race between
   flush and new enqueue
6. Dispatch **Qwen THINK** adversarial review (transport/ gate)
7. If both PASS: commit
8. Gate fail or review FAIL: feed short error lines back to Qwen CODER for fix;
   one retry before escalating to Qwen THINK for implementation

**Lane:** Qwen CODER + Fusion Lite 3-panel + Qwen THINK adversarial
**Status:** [ ] pending

---

### Sub-Task 5: A3 — Android Kotlin retry suppression

**Intent:** Step 3 of `CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK`.
Kotlin must stop escalating transport-success to failed/corrupted when there is
no receipt ACK, widen the receipt window, and add a regression test.

**Expected Outcomes:**
- `data/MeshRepository.kt`: retry loop suppressed when core reports InCustody/queued
- `utils/BackoffStrategy.kt` or equivalent: receipt window widened
- Kotlin regression test added
- `cd android && ./gradlew assembleDebug -x lint --quiet` passes
- Fusion Lite 3-panel triangulation PASS (WS-A delivery logic rule)
- Committed: `fix(android): suppress retry on InCustody, widen receipt window (A3)`

**Todo List:**
1. Dispatch **Qwen CODER** (`qwen3-coder-plus`) with:
   - The relevant Kotlin sections from `MeshRepository.kt`, `BackoffStrategy.kt`,
     `TransportManager.kt` (grep first, embed only relevant functions)
   - The A-01 spec from `SCM_V1_FARM_BUILD_MASTER_BACKLOG.md` §3
   - The delivery state context from `SESSION_HANDOFF_2026-07-13` lines 111-117
2. Apply returned patch; run Android gate
3. Dispatch **Fusion Lite** 3-panel: `python scripts/fusion_lite.py --panel 3 --max-cost 0.01`
   Review: does the Kotlin change correctly map all core receipt states to UI
   delivery states without false positives?
4. If PASS: commit; move `U5_ANDROID_RECEIPT_UNIFICATION.md` dependency updated
5. If Fusion Lite finds issues: micro-remediate the exact finding via Morph Lite
   (single-file Kotlin fix) then re-run panel

**Lane:** Qwen CODER + Fusion Lite 3-panel
**Status:** [ ] pending

---

### Sub-Task 6: E-01a — PQC-07 attempt 3 constraints analysis (parallel with 4-5)

**Intent:** Read-only THINK dispatch. Extracts the constraints that attempt 3 must
satisfy from the two failed attempt diffs. Can run in parallel with sub-tasks 4-5
since it touches nothing in the working tree.

**Expected Outcomes:**
- `HANDOFF/review/E01a_attempt_constraints.md` written with:
  - Attempt 1 failure mode: asymmetric mixing at DH crossing — reorder desync
  - Attempt 2 failure mode: per-message symmetric mixing — packet loss desync
  - Attempt 3 requirements: ratchet-step-tied mixing, self-synchronizing via
    the DH public envelope header, survives both reorder and loss
- Operator reviews before E-01b is dispatched (escalation gate)

**Todo List:**
1. Dispatch **Gemini 2.5 Pro** (`gemini-2.5-pro` via `scripts/delegate_task.py
   --provider gemini --model gemini-2.5-pro`) — large context window needed to
   read both attempt diffs at once:
   Feed: both `HANDOFF/review/PQC_07_ATTEMPT*` patch files + full content of
   `HANDOFF/todo/PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md` + relevant
   sections of `core/src/crypto/ratchet.rs`
2. Write returned analysis to `HANDOFF/review/E01a_attempt_constraints.md`
3. Log dispatch to `tmp/lakes/ledger.jsonl` (or `tmp/scmorc/dispatch_log.md`)
4. STOP — do NOT dispatch E-01b until operator explicitly approves the constraints

**Lane:** Gemini 2.5 Pro (large context); Qwen THINK fallback if Gemini unavailable
**Status:** [ ] pending

---

### Sub-Task 7: Wave D infrastructure batch (parallel dispatch)

**Intent:** Five independent Wave D items can all run concurrently on separate
lanes. No crypto code touched; adversarial review only needed for D-05 changes
in `mobile_bridge.rs` FFI boundary.

**Expected Outcomes per item:**

| ID | Outcome | Gate |
|---|---|---|
| D-06 | 44 TODOs classified; v1-blockers fixed | `cargo check --workspace` |
| D-01 | Farm test runner has REST API (submit-run, poll-status, fetch-artifact) | Python smoke test |
| D-02 | `testDebugUnitTest` is runnable in Gradle | `./gradlew testDebugUnitTest --dry-run` |
| D-03 | `SCMessengerTests/` registered in xcodeproj; `xcodebuild test` runs | xcode build |
| D-05 | ~60 unwrap/panic sites at FFI+startup+crypto boundary hardened | `cargo check --workspace` |

**Todo List:**
1. **D-06 first** (FLASH, fastest; findings may reduce D-05 scope):
   Dispatch **Groq** (`qwen/qwen3-32b` or `llama-3.3-70b-versatile` via
   `scripts/delegate_task.py --provider groq`): read all `TODO` occurrences via
   grep output (embed the list), classify each as v1-blocker / file-ticket /
   wontfix, fix the blockers in-place

2. **D-01** (infra lane, Python):
   Dispatch **Qwen CODER** with the farm test runner spec from
   `SCM_V1_FARM_BUILD_MASTER_BACKLOG.md` D-01 entry; ~300 Python LoC

3. **D-02** (android lane):
   Dispatch **Qwen CODER** with `android/app/build.gradle`, the D-02 spec,
   and the discovery from `SESSION_HANDOFF_2026-07-06_scmorc.md` lines 54-64
   (force-disabled tests context)

4. **D-03** (ios lane):
   Dispatch **Morph Lite** (`morph/morph-v3-fast`): xcodeproj PBXNativeTarget
   addition is a single-file mechanical change <500 lines

5. **D-05** (core lane, split into 3 parallel sub-workers):
   - Sub-worker A: `mobile_bridge.rs` FFI exports — **Qwen CODER** + Qwen THINK adversarial
   - Sub-worker B: startup path (`cli/src/main.rs`, `core/src/iron_core.rs` init) — **Qwen CODER**
   - Sub-worker C: crypto boundary (`core/src/crypto/`) — **Qwen CODER** + Qwen THINK adversarial
   Each sub-worker gets its own file list; no overlaps

6. After all D items pass their gates: commit each separately with
   `fix(infra): <item name> (D-0N)`

7. After D-02 lands: queue T-01 and D-04

**Lane:** Groq FLASH (D-06) / Qwen CODER (D-01, D-02, D-05) / Morph Lite (D-03)
**Status:** [ ] pending

---

### Sub-Task 8: Wave T residuals and quick closes

**Intent:** T-05 and T-06 are FLASH-tier closes requiring no code. T-02 and T-03
can dispatch in parallel once no file overlap exists with Sub-Task 7's D-05 work.

**Expected Outcomes:**
- T-05: T4.5 ticket confirmed done and moved to `HANDOFF/done/`
- T-06: v1.0.0 known-limitation entry for desktop BLE peripheral written in release docs
- T-02: `AWARE_PORT` per-peer port negotiation via service-info TLV implemented
- T-03: WiFi Direct Rust transport for Android implemented (2 parallel workers)

**Todo List:**
1. **T-05**: Orchestrator-local — grep to confirm `tasks/T4.5/progress.md` gates
   all green; if yes, move ticket to done (no worker needed)
2. **T-06**: Dispatch **Groq FLASH** (`llama-3.1-8b-instant`):
   write one known-limitation entry for `CHANGELOG.md` or `docs/RELEASE_NOTES.md`
   about desktop BLE peripheral not being a BLE peripheral in v1.0.0
3. **T-02**: Dispatch **Qwen CODER** with `core/src/transport/wifi_aware.rs` and
   `android/` WiFi Aware service-info TLV sections (~150 Rust + ~100 Kotlin)
4. **T-03**: Dispatch 2 concurrent **Qwen CODER** workers:
   - Worker A (Rust): `transport/wifi_direct.rs` mirroring `wifi_aware.rs` pattern
   - Worker B (Kotlin): Android platform bridge for WiFi Direct
   Files are fully disjoint; safe to run concurrently

**Lane:** Groq FLASH (T-05 verify, T-06) / Qwen CODER (T-02, T-03)
**Status:** [ ] pending

---

### Sub-Task 9: Unified orchestrator contract and infrastructure files (Wave Z)

**Intent:** Create the missing infrastructure files so any model (Claude, Gemini,
Codex) can pick up the queue cold. `docs/ORCHESTRATION.md` is referenced but
missing; `scm_v1_farm_queue.jsonl` and `tmp/lakes/` files do not exist yet.

**Expected Outcomes:**
- `docs/ORCHESTRATION.md` created; resolves broken references in scmorc/scmqwen
- `scm_v1_farm_queue.jsonl` has one JSON line per open task
- `tmp/lakes/registry.json` has the lake registry from `SCM_UNIFIED_LAKE_ORCHESTRATION.md` §1
- `tmp/lakes/ledger.jsonl` exists (empty + format header)
- `tmp/lakes/round_robin_state.json` has seed counters (all 0)
- `scripts/lake_route.py` (~150 LoC) reads the above and prints `provider model`
- `scripts/delegate_task.py` accepts `--provider gemini`
- Each of five command files has a "Lake contract" header section

**Todo List:**
1. Dispatch **Groq** (`llama-3.3-70b-versatile`) for `docs/ORCHESTRATION.md`:
   thin document pointing to `SCM_UNIFIED_LAKE_ORCHESTRATION.md`, plus a
   Section 2.1 that captures the cross-lane dispatch ladder currently scattered
   across the command files
2. Dispatch **Qwen FLASH** (`qwen3-coder-flash`) for `scm_v1_farm_queue.jsonl`:
   input is the wave tables from `SCM_V1_FARM_BUILD_MASTER_BACKLOG.md` §3;
   output is a JSONL file with fields: `id`, `wave`, `tier`, `status`, `depends`,
   `blocks`, `review`, `loc_est`, `description`
3. Dispatch **Morph Lite** for the Gemini provider addition to `delegate_task.py`
   (per `SCM_UNIFIED_LAKE_ORCHESTRATION.md` §6.2 spec, ~15 LoC, single file)
4. Dispatch **Qwen CODER** for `scripts/lake_route.py` (per §6.3 spec, ~150 LoC):
   reads registry + ledger + round_robin_state; prints `provider model`; appends
   ledger record on exit; sets cooldowns from 429 responses
5. Orchestrator creates `tmp/lakes/` directory and writes seed JSON files directly
   (trivial; no worker needed)
6. For each command file: dispatch **Groq FLASH** to prepend a 5-line
   "Lake contract" header block naming the unified doc and the lane this command
   specializes; one dispatch per file (5 total, can run concurrently)

**Lane:** Groq (docs) / Qwen FLASH (JSONL) / Morph Lite (delegate_task.py) / Qwen CODER (lake_route.py) / Groq FLASH (command file headers)
**Status:** [ ] pending

---

### Sub-Task 10: C-01 transport root-cause + H-03 gate documentation

**Intent:** C-01 is THINK-tier and the gate for the entire parity completion wave.
C-02..C-06 block on three operator sign-offs (H-03). Document the human gates
clearly so they can be acted on immediately.

**Expected Outcomes:**
- C-01: root-cause analysis complete; if a fix is needed, implemented and
  adversarial-reviewed
- `HANDOFF/todo/WAVE_H_HUMAN_GATES.md` written listing all 5 H items with
  exact operator actions and what each unblocks
- C-02..C-06 ticket files created (gated)

**Todo List:**
1. Dispatch **Gemini 2.5 Pro** or **Qwen THINK** for C-01: provide
   `core/src/transport/` source context (grep for the negotiation failure
   path; embed relevant 60-80 lines) + the P1-04 history from `_QUEUE.md`
2. C-01 fix (if needed) must get adversarial review — **Qwen THINK** or
   **Fusion Lite** panel
3. Dispatch **Groq FLASH** to write `HANDOFF/todo/WAVE_H_HUMAN_GATES.md`:
   input is the H-series entries from `SCM_V1_FARM_BUILD_MASTER_BACKLOG.md` §3
4. Create C-02..C-06 ticket files via **Qwen FLASH** (gated, doc-only packets)

**Lane:** Gemini/Qwen THINK (C-01) / Groq FLASH (docs) / Qwen FLASH (tickets)
**Status:** [ ] pending

---

### Sub-Task 11: Wave B freeze documentation and pre-staging

**Intent:** Wave B is frozen until E-01c lands. All tickets exist. Add freeze
markers and pre-stage packets so dispatch is instant on unfreeze.

**Expected Outcomes:**
- Each B-series ticket in `HANDOFF/todo/` has a `FROZEN: waiting for E-01c`
  header line appended
- `HANDOFF/todo/WAVE_B_FREEZE_STATUS.md` created as a summary
- B-02 (`PQC_09_HYBRID_ONION.md`) marked DOUBLE FROZEN (E-01c + AD-8 lift)

**Todo List:**
1. Dispatch **Groq FLASH** to prepend freeze headers to each existing B-series
   ticket file (6 files: PQC_09_HYBRID_ONION, PQC_09_ONION_COMPILE_FIX,
   PQC_09_SECURITY_REVIEW_FIXES, PQC_10_MLDSA_IDENTITY_SIGNATURES,
   PQC_10_MLDSA_MODULE_MISSING, PQC_11_RELAY_INVITE_HYBRID_AUTH,
   PQC_12_TRANSPORT_TLS_PQ, PQC_13_VERIFICATION_SUITE, PQC_14_DOCS_AND_RISK_REGISTER)
2. Write `HANDOFF/todo/WAVE_B_FREEZE_STATUS.md` summarising dispatch order on unfreeze

**Lane:** Groq FLASH
**Status:** [ ] pending

---

## Execution Order and Parallelism

```
IMMEDIATE (can all start in parallel):
  Sub-Task 1   — Morph Lite (Fix-1, 1 line) + Qwen THINK adversarial
  Sub-Task 2   — Orchestrator-local + Qwen THINK if needed
  Sub-Task 3   — Morph Lite (F1 test fix)
  Sub-Task 6   — Gemini 2.5 Pro (E-01a, read-only, no repo change)
  Sub-Task 7   — Groq/Qwen/Morph batch (D-06, D-01, D-02, D-03, D-05 all parallel)
  Sub-Task 8   — Groq FLASH + Qwen CODER (T-05, T-06, T-02, T-03)
  Sub-Task 9   — Groq/Qwen/Morph batch (ORCHESTRATION.md, JSONL, lake_route.py)
  Sub-Task 11  — Groq FLASH (B-wave freeze headers)

AFTER Sub-Task 1 passes gate:
  Sub-Task 4   — Qwen CODER + Fusion Lite + Qwen THINK (Site 3 outbox loop)

AFTER Sub-Task 4 and Sub-Task 5 (A3) both pass:
  Sub-Tasks A-03, A-04 (U5), A-05 (U6), A-06 (U7) — Qwen CODER

AFTER operator reviews E-01a (Sub-Task 6):
  E-01b — Qwen MAX + adversarial; OPERATOR ESCALATION GATE

AFTER E-01b adversarial PASS:
  E-01c — Qwen CODER (signed-off spec)
  THEN: E-01d, E-02, E-03, E-04, Wave B unfreeze

AFTER D-02 lands (Sub-Task 7):
  T-01, D-04

AFTER C-01 + H-03 sign-offs (Sub-Task 10):
  C-02, C-03, C-04, C-05, C-06 in order
```

**Build serialization rule:** Only one `cargo` or `gradlew` process at a time on
Windows. Sub-tasks that require builds are serialized through the orchestrator.
Workers implement code only; the orchestrator runs all gates.

---

## Complete Open Backlog Reference

### P0 — Farm delivery truth (dispatch now)
| ID | Task | Lane | AUDIT-GATE |
|---|---|---|---|
| Fix-1 | Custody DriftFrame wrap | Morph Lite + Qwen THINK | Yes (transport/) |
| Site-3 | Outbox flush-on-reconnect | Qwen CODER + Fusion Lite + Qwen THINK | Yes (transport/) |
| A3 (A-01) | Android retry suppression | Qwen CODER + Fusion Lite | No |
| F1-fix | Ledger convergence test | Morph Lite / Qwen CODER | No |

### Wave A — Delivery truth
| ID | Task | Lane |
|---|---|---|
| A-02 | F1 confirm run | after F1-fix |
| A-03 | F2 MeshStore persistence | Qwen THINK audit + CODER fix |
| A-04 | U5 Android receipt unification | Qwen CODER + Fusion Lite |
| A-05 | U6 iOS receipt unification | Qwen CODER + Fusion Lite |
| A-06 | U7 schema drift audit | Qwen THINK + CODER |
| A-07 | NetworkError observability | DONE (Hermes today) |
| A-08 | Onion gating | DONE (Hermes today) |

### Wave E — Crypto soundness
| ID | Task | Lane | Gate |
|---|---|---|---|
| E-01a | Constraints analysis | Gemini 2.5 Pro / Qwen THINK | OPERATOR REVIEW |
| E-01b | Design spec | Qwen MAX + adversarial | OPERATOR ESCALATION |
| E-01c | Implement spec | Qwen CODER | after E-01b PASS |
| E-01d | Proptest matrix | Qwen CODER | after E-01c |
| E-02 | Force-ratchet check | Qwen CODER | after E-01c |
| E-03 | PQ refresh trigger | Qwen THINK + CODER | after E-01c |
| E-04 | Wire ratchet step | Qwen CODER | after E-01c |

### Wave D — Infrastructure
| ID | Task | Lane |
|---|---|---|
| D-01 | Farm REST API | Qwen CODER |
| D-02 | Android Robolectric | Qwen CODER |
| D-03 | iOS XCTest target | Morph Lite |
| D-04 | Emulator test job | Qwen CODER (after D-02) |
| D-05 | unwrap/panic hardening | Qwen CODER x3 + Qwen THINK (FFI+crypto) |
| D-06 | TODO triage | Groq FLASH |
| D-07 | GitHub Actions billing | HUMAN H-01 |

### Wave C — Parity
| ID | Task | Lane |
|---|---|---|
| C-01 | P1-04 transport root-cause | Gemini/Qwen THINK + adversarial |
| C-02 | Listen-side adaptive port | Qwen CODER (after H-03 sign-offs) |
| C-03 | Advertise/dial adaptive ports | Qwen CODER |
| C-04 | Hardcoded-port sweep | Qwen FLASH |
| C-05 | Hostile-network test | Qwen CODER |
| C-06 | Node task P1-18 | Qwen CODER |
| C-07 | Parity exit review | HUMAN + Qwen THINK |

### Wave B — PQC depth (FROZEN until E-01c)
All tickets exist in `HANDOFF/todo/`. Dispatch order on unfreeze:
B-01 -> B-03 -> B-04 -> B-05 -> B-06 -> B-07.
B-02 double-frozen (also AD-8 onion seam freeze).

### Wave T — DAG residuals
| ID | Task | Lane |
|---|---|---|
| T-01 | T1.2 Robolectric WiFi-Aware test | Qwen CODER (after D-02) |
| T-02 | T1.3 AWARE_PORT per-peer | Qwen CODER |
| T-03 | T1.4 WiFi Direct Rust (Android) | Qwen CODER x2 |
| T-04 | T2.4 verify-close | after D-02/D-03 |
| T-05 | T4.5 verify-close | Orchestrator-local FLASH |
| T-06 | T1.8 BLE desktop doc | Groq FLASH |

### Wave H — Human gates (no dispatch)
| Item | Action | Unblocks |
|---|---|---|
| H-01 | GitHub Actions billing/org fix | D-07, D-03 run, iOS lane |
| H-02 | Physical two-device WiFi Aware/BLE field trials | T-02 final |
| H-03 | Three P1-10 sign-offs (peer_exchange, GroupInfo.port, transport_memory) | C-02..C-04 |
| H-04 | AWS cloud node resume decision | B4 infra (committed, paused) |
| H-05 | Final release sign-off | All waves complete |

---

## LoC Totals (from `SCM_V1_FARM_BUILD_MASTER_BACKLOG.md` §4)

| Wave | Shipping LoC | Test LoC | State |
|---|---|---|---|
| A | ~1,360 | ~790 | A7/A8 done; A3/Site-3 in flight |
| E | ~1,000 | ~300 | E-01a in flight |
| D | ~1,350 | ~400 | In flight (parallel) |
| C | ~950-1,250 | ~200 | C-01 in flight |
| B | ~2,250 | ~800 | FROZEN |
| T | ~1,000 | ~380 | T-05/T-06 closing; T-02/T-03 in flight |
| Z/H | 0 | 0 | Z in flight; H blocked human |
| **TOTAL** | **~7,900-8,200** | **~2,870** | |

---

## Standing Rules

- E-01c blocked until E-01b adversarial PASS
- PQC-11/13 frozen until E-01 lands
- PQC-09 double-frozen (E-01c + AD-8 onion seam freeze lift)
- All `crypto/`, `privacy/`, `transport/` diffs: adversarial review mandatory
- All WS-A delivery logic diffs: Fusion Lite 3-panel or 3 Qwen verifier dispatches
- Windows build serialization: one cargo/gradle at a time; orchestrator is the single build writer
- Workers never run builds; workers never commit; workers never move HANDOFF files
- No emojis (hook-enforced)
- Storage only through `core/src/store/IronCore`
- UniFFI bindings never edited manually
