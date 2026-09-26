# v0.4.0 Orchestration Plan — Josh Alpha Test

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: **Superseded** (2026-07-25 planning unity pass).
Last updated: 2026-07-19 (Bob planning session). Corrections applied 2026-07-19 after subagent verification pass.

**Use instead (in order):**

1. `HANDOFF/todo/_QUEUE.md` — **status-correction header** (authoritative v0.4.0 next actions)
2. `HANDOFF/plans/MILESTONE_RELEASE_PLAN.md` — release scope and done/open lists
3. `docs/ORCHESTRATION.md` — orchestration protocol (`/orchestrate`, not archived `/scmorc`)

This file is kept for traceability (2026-07-19 ground truth and task table). Do not
dispatch from the task table below without reconciling against the queue header.

Sequencing source (historical): HANDOFF/plans/MILESTONE_RELEASE_PLAN.md,
  HANDOFF/ALPHA_TEST_SESSION_FINDINGS_2026-07-19.md,
  HANDOFF/SESSION_HANDOFF_2026-07-20_CI_FIX.md.

Goal: Two independent identities (Lucas + Josh) successfully exchange
messages over the internet via the alpha relay
(`/ip4/100.56.248.69/tcp/9001`), with real receipts confirming delivery
both directions. Version tag **`v0.4.0-alpha.1`** published as a GitHub
Release with a downloadable Windows CLI + Android APK. (This file originally
said `v1.0.0-alpha.1`; release slicing uses v0.4.0 per MILESTONE_RELEASE_PLAN.)

---

## GROUND TRUTH (verified, do not relitigate)

### Already done since 2026-07-19

- Alpha relay: LIVE at `100.56.248.69:9001`, containerized, restart policy
  set, health API on `9876`. SG opens 22/9001/9000/9876.
- Lucas CLI <-> relay: real TCP connection ss-verified (not just "queued").
- Lucas emulator: `-gpu host` (ANR-free), APK from commit `f2831458`.
- A-09 partial mitigation: `is_dialable_multiaddr` filter applied
  (commit `36635cb0`). Loopback/link-local/site-local rejected in
  CLI ledger dial loop.
- CI fixed: `CI_RED_ON_MAIN_ALL_FEATURES.md` -- commit `bc94ffbb`
  (local, operator must push). Lint, FFI snapshots, wifi_aware test.
- GRACEFUL_AF_DIAL_POLICY items 1+2: self-dial prevention + RFC1918
  network-awareness wired, commit pending adversarial audit merge.
  Items 3+4 (per-peer backoff cap, prefer circuit-relay) still open.
- U4 receipt encoding unified (HANDOFF/done/U4_RECEIPT_ENCODING_UNIFIED.md):
  encode_receipt/decode_receipt added to core.
- BOOTSTRAP_TOPOLOGY_WIRING.md: bootstrap-topology.sh written and
  GET /api/contacts added to CLI. Static-verified only.
- PROVE_SECOND_REAL_ENDPOINT_DELIVERY.md: CONFIRMED DONE. Proof document at
  HANDOFF/PROOF_TWO_ENDPOINT_DELIVERY_2026-07-20.md. Two CLI identities
  (Alice/Bob), both directions, via alpha relay. Bug found+fixed in process:
  `handle_send_message` was parsing public key hex as libp2p PeerId (commit
  `29d01e5b`, fusion_lite verified PASS). Protocol is proven; Android E2E
  still requires device.
- D-05 unwrap/panic hardening: COMPILE GATE NOT RUN. Implementation is in
  HANDOFF/done/D-05_UNWRAP_PANIC_HARDENING_REDISPATCH.md (code complete,
  scope-clean slice). UDL itself is CLEAN (no Receipt dict in api.udl).
  Working tree has out-of-scope wrapper exports in core/src/lib.rs:81 only.
  Must revert lib.rs:81 and run compile gate before committing D-05.
  HANDOFF/IN_PROGRESS/ has 11 files: A-04, A-05, D-05, F1, farm-sim, etc.
- A-04 Android receipt unification: dispatch produced 0-byte log (silent
  failure). Not done.
- E-00 (IronCore ratchet wiring): committed 2026-07-17 (6059038c).
  Confirmed by _QUEUE.md header. The E-01b ratchet-level mixing is STILL
  open (HANDOFF/todo/E01B_FABLE_DESIGN_HANDOFF.md) but is NOT v0.4.0 scope.

### Still OPEN (required for v0.4.0)

See Section 2 task table below for the authoritative list and delegation plan.

### The single most important finding

`ALPHA_TEST_SESSION_FINDINGS_2026-07-19.md` confirms: the dial returns
"Bootstrap connected" but swarm.dial() replies Ok(()) on enqueue, not on
ConnectionEstablished. Live evidence: ss -tn on relay showed ZERO established
connections throughout despite the app claiming success. This is the v0.4.0
primary blocker -- if not already fixed by f2831458, all other work is moot.
Verify first: `ss -tn state established` on relay must show Lucas's real IP.
The CI handoff confirms Lucas CLI -> relay IS verified as a real connection.
The dial fix (f2831458) is confirmed as the image deployed to the relay.
Whether Android emulator -> relay is a real connection now (not just queued)
is the remaining verification needed.

---

## TASK TABLE (v0.4.0 only)

Priority order: top = dispatch first. Items marked [HUMAN] are operator-only.
Model routing follows scmorc routing table (Qwen primary, fusion_lite/groq
secondary, openrouter/ollama backup).

| # | Task | Ticket | Status | Delegation | Gate | Audit? |
|---|------|--------|--------|------------|------|--------|
| P0a | Push CI fix to origin | CI_RED_ON_MAIN_ALL_FEATURES | [HUMAN] push bc94ffbb | Lucas only | CI green all 4 jobs | N |
| P0b | Move PROVE_SECOND_REAL_ENDPOINT_DELIVERY to done/ | PROVE_SECOND_REAL_ENDPOINT_DELIVERY | CONFIRMED DONE (PROOF_TWO_ENDPOINT_DELIVERY_2026-07-20.md, two CLIs, both dirs) | Orchestrator moves ticket to done/ | file in done/ | N |
| P1 | Graceful dial items 3+4 (per-peer backoff + prefer relay-circuit) | GRACEFUL_AF_DIAL_POLICY | Items 1+2 done, 3+4 open | Qwen THINK -> CODER | cargo check/clippy clean | Yes (transport/) |
| P2 | Outbox Site-1 flush on reconnect | OUTBOX_FLUSH_ON_CONNECT_RETRY | TODO (95% patch exists) | Qwen CODER (reuse patch) | cargo test -p scmessenger-core --lib store::outbox | No (but fusion_lite triangulation required) |
| P3 | Android retry suppression (A3 step 3) | CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK (done/), step 3 | Open per _QUEUE.md | Qwen CODER, Kotlin | Android build clean, manual receipt test | No |
| P4 | A-04 Android receipt unification re-dispatch | IN_PROGRESS/DISPATCH_WAVE_CRYPTO_CODER | 0-byte silent failure | Qwen CODER (re-dispatch, strong model) | Kotlin compiles, receipt round-trip | No |
| P5 | D-05 unwrap/panic hardening: revert UDL scope-creep, run compile gate | HANDOFF/done/D-05_UNWRAP_PANIC_HARDENING_REDISPATCH | Compile gate not run; UDL changes need revert | Orchestrator: revert UDL, then cargo check --workspace | cargo check clean, grep verification | No |
| P6 | CI - FFI surface drift fix (if D-05 UDL revert changes snapshot) | CI_RED_ON_MAIN_ALL_FEATURES (secondary) | Depends on P5 | Qwen HAIKU (mechanical) | FFI snapshot matches | No |
| P7 | V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS | V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS | Ready -- needs [HUMAN] go-ahead | Lucas: `git tag v1.0.0-alpha.1 && git push origin v1.0.0-alpha.1` | GitHub Release page live with APK + CLI | N |
| HUMAN | H-04 AWS relay activate decision (already done -- relay IS live) | WAVE_H_HUMAN_GATES | RESOLVED | N/A | Relay running | N/A |

### Tasks NOT in v0.4.0 scope (do not dispatch)

- E-01b/E-01c PQ ratchet root-key mixing (v1.0.0, FROZEN)
- PQC-09 through PQC-14 (FROZEN on E-01c)
- iOS lane (H-01 billing fix not yet resolved)
- Meeting Mode / KMP desktop
- Farm simulation 12-node Docker soak (v0.5.0)
- A-09 full closure (auth, connection_limits, dedup -- v0.5.0)
- WiFi Direct Android<->Android (v1.1)

---

## DELEGATION DETAILS (per task)

### P1: Graceful Dial Items 3+4

Ticket: `HANDOFF/todo/GRACEFUL_AF_DIAL_POLICY.md`
Current state: Items 1+2 committed (self-dial prevention, RFC1918 awareness).
Items 3+4 need dial-loop restructuring:
  - Per-peer backoff state (dedup by peer ID, not address)
  - Max 3 concurrent outbound dials to unknown peers
  - Exponential backoff: 5s, 30s, 120s, 5min, 30min
  - After relay circuit established: prefer circuit-relay over direct dials

Files: `cli/src/ledger.rs`, possibly `cli/src/main.rs`
Dispatch: Qwen THINK (design note first, then CODER impl)
  Step 1: `python scripts/delegate_task.py --task HANDOFF/todo/GRACEFUL_AF_DIAL_POLICY.md --provider qwen --tier thinking --files cli/src/ledger.rs cli/src/main.rs --mode analyze`
  Step 2: After design note verified: CODER dispatch for items 3+4 only
Gate: `cargo check -p scmessenger-cli`, `cargo clippy -p scmessenger-cli -- -D warnings`
Audit: Mandatory crypto-security-auditor (transport/routing concerns)
  -- fable read-only worker after CODER lands
Notes from ticket: IPv6 ULA (`fc00::/7`) is a pre-existing gap (not a
  regression). `get_bound_addresses()` empty-on-bind-fail edge case flagged
  but not fixed in items 1+2. Do not expand scope to include these.

### P2: Outbox Site-1 Flush on Reconnect

Ticket: `HANDOFF/done/OUTBOX_FLUSH_ON_CONNECT_RETRY.md`
NOTE: This file is in done/ but its Status header says "TODO" -- a premature
  move. The task is NOT done. The reference patch is at:
  `HANDOFF/review/OUTBOX_FLUSH_ATTEMPT_296LINES.patch`

The exact bug that blocked the prior attempt:
  `enqueue` initializes `next_retry_at` to a FUTURE time, so flush filter
  excludes fresh messages. Fix: fresh enqueue sets `next_retry_at = None`
  (due immediately).

Files: `core/src/store/outbox.rs`, `core/src/iron_core.rs`, `cli/src/main.rs`
Dispatch:
  `python scripts/delegate_task.py --task HANDOFF/done/OUTBOX_FLUSH_ON_CONNECT_RETRY.md --provider qwen --tier coder --files core/src/store/outbox.rs core/src/iron_core.rs cli/src/main.rs --apply --verify "cargo test -p scmessenger-core --lib store::outbox" --max-rounds 3`
Gate: `cargo test -p scmessenger-core --lib store::outbox` green,
  specifically `test_flush_peer_messages` passing.
Triangulation: 3 distinct Qwen verifier passes OR one fusion_lite panel
  (capped $0.01) before commit. Not crypto/transport -- but delivery logic,
  so review semantics carefully.
Notes: The reference patch is ~90% reusable. The bug is a single-line fix
  (`next_retry_at = None` for fresh enqueue). Worker should apply the patch,
  fix that specific line, and verify.

### P3: Android Retry Suppression (A3, step 3)

Ticket: `HANDOFF/done/CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md`
  steps 3-4 (file is in done/ but step 3 explicitly marked STILL OPEN)
Current state: Steps 1-2 (core receipt classification + CLI serde_json fix)
  landed 2026-07-13. Step 3 is Kotlin-side only.

What step 3 requires:
  - Transport-success must never escalate to failed/corrupted
  - Widen receipt window (30s is too narrow for relay paths)
  - Kotlin regression test: mock transport-success, no receipt -> state stays
    "sent, unconfirmed", never "failed"

Files: `android/app/src/main/java/com/scmessenger/android/` MeshRepository.kt,
  SmartTransportRouter.kt (whichever owns the retry escalation logic)
Dispatch: Qwen CODER, Kotlin tier
  `python scripts/delegate_task.py --task HANDOFF/done/CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md --provider qwen --tier coder --files android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt --apply --verify "cd android && ./gradlew :app:compileDebugKotlin -x lint --quiet" --max-rounds 2`
Gate: `cd android && ./gradlew :app:compileDebugKotlin -x lint --quiet`
Notes: The line in MeshRepository.kt that drops after 12 retries is at ~L605.
  The "corrupted" marker is at ~L700-723. Do not touch PQC or transport code.

### P4: A-04 Android Receipt Unification Re-dispatch

Ticket: From `HANDOFF/IN_PROGRESS/DISPATCH_WAVE_CRYPTO_CODER.md` task 2.
Prior attempt: `tmp/a04_dispatch.log` was 0 bytes. Silent failure.
Prior dispatch model: qwen3-coder-plus (insufficient).

What it does: call core's unified `encode_receipt()`/`decode_receipt()`
  (already exist, added by U4) from Kotlin MeshRepository. Remove any
  duplicate/legacy receipt encoding on the Kotlin side.

Files: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`,
  `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt`
Dispatch: Qwen CODER (upgrade to qwen3-235b or qwen3-max if available)
  `python scripts/delegate_task.py --task HANDOFF/IN_PROGRESS/DISPATCH_WAVE_CRYPTO_CODER.md --provider qwen --tier max --files android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt --apply --verify "cd android && ./gradlew :app:compileDebugKotlin -x lint --quiet" --max-rounds 2`
Gate: `cd android && ./gradlew :app:compileDebugKotlin -x lint --quiet`
Notes: U4 functions are exported from `core/src/lib.rs` as
  `encode_receipt`/`decode_receipt`. UniFFI generates the Kotlin bridge
  already (if UDL is clean -- verify P5 first). Do NOT touch api.udl here.

### P5: D-05 Compile Gate + UDL Scope-Creep Revert

Ticket: `HANDOFF/done/D-05_UNWRAP_PANIC_HARDENING_REDISPATCH.md`
This is an ORCHESTRATOR action, not a worker dispatch:

Step 1 (orchestrator): Check working tree for out-of-scope UDL changes:
  `git diff core/src/api.udl`
  If `Receipt` dictionary or `encode_receipt_from_components` appear: revert.
  `git checkout HEAD -- core/src/api.udl core/src/lib.rs`
  (only if lib.rs has the out-of-scope wrappers that reference non-existent types)

Step 2 (orchestrator): Run compile gate:
  `CARGO_INCREMENTAL=0 cargo check --workspace`
  `CARGO_INCREMENTAL=0 cargo test --workspace --no-run`

Step 3: If compile gate fails with errors from the D-05 changes, dispatch
  a targeted Qwen HAIKU fix for the exact errors only.
  If compile gate passes: commit D-05 slice.

Step 4: Update FFI snapshots if UDL revert changes them:
  Read CI diff output, manually apply additions to:
  `scripts/ffi-snapshots/kotlin-symbols.txt`
  `scripts/ffi-snapshots/swift-symbols.txt`
  (cannot run ffi_surface.sh --update on Windows, per session handoff note)

Gate: `cargo check --workspace` exit 0, `cargo clippy --workspace
  --all-features -- -D warnings` exit 0 (matching CI standard).
Notes: The D-05 implementation notes say changes to `crypto/encrypt.rs` and
  `identity/keys.rs` are also in the working tree and considered in-scope/
  benign. Keep those, only revert the UDL/lib.rs scope-creep.

### P6: CI Snapshot Drift (conditional on P5)

Only needed if P5's UDL revert changes the FFI surface.
Dispatch: Qwen HAIKU (mechanical, 1-file edit)
  Manually compare `git diff scripts/ffi-snapshots/` after P5 and apply.

### P7: Install Artifact for Alpha Testers

Ticket: `HANDOFF/todo/V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS.md`
This is a HUMAN gate (Lucas only -- standing rule: never push tags without
  operator go-ahead). No agent action needed.

When Lucas is ready:
  1. Confirm CI is green on latest commit (post P0a push).
  2. `git tag v1.0.0-alpha.1 && git push origin v1.0.0-alpha.1`
  3. release.yml runs: Windows/Linux/macOS CLI + Android debug APK + WASM
  4. GitHub Release page created at
     https://github.com/Sovereign-Communication/SCMessenger/releases
  5. Send Josh the release URL.

Notes:
  - Debug APK is fine for Josh (trusted alpha tester, install unknown sources).
  - Relay bootstrap address is hardcoded at MeshRepository.kt:8492
    (`/ip4/100.56.248.69/tcp/9001`). This is correct for the Josh test.
  - No Android signing key configured -- debug APK only. That is acceptable.
  - DO NOT push the tag before all code changes (P1-P5) are committed and CI
    is green. A release from a red build is not trustworthy.

---

## EXECUTION ORDER

```
[HUMAN] P0a — Lucas pushes bc94ffbb to origin, waits for CI green
              (blocks P7; does not block P1-P6)

[ORCHESTRATOR] P5 — D-05 UDL revert + compile gate (cheap, orchestrator-local)
                    Prerequisite for P4 (Kotlin receipt dispatch needs clean UDL)

[Qwen THINK] P1 design — Graceful dial items 3+4 design note
[Qwen CODER] P2 — Outbox Site-1 flush (reuse 296-line patch, fix enqueue bug)
[fable audit] P1 security review — after P1 CODER lands

[Qwen CODER] P3 — Android retry suppression (Kotlin, after P2 direction is clear)
[Qwen CODER] P4 — A-04 Android receipt unification (after P5 cleans UDL)

[VERIFY] P0b — Confirm PROVE_SECOND_REAL_ENDPOINT_DELIVERY is actually done
               (ss -tn on relay; two-CLI proof as described in session handoff)

[HUMAN] P7 — Lucas tags v1.0.0-alpha.1 after all above are committed + CI green
```

Parallelism notes:
- P1 (design only) + P2 + P5 can run concurrently (different file domains).
- P3 and P4 are both Kotlin-side; run sequentially (same file: MeshRepository.kt).
- P4 depends on P5 being clean (UDL must not reference non-existent types).
- fable audit for P1 must complete before P1 is committed.
- P7 is terminal; blocks on CI green + all other tasks committed.

---

## FREE LANES BEFORE CLAUDE WORKERS

Per scmorc routing table: Claude workers (subscription burn) are LAST RESORT.
For v0.4.0 specifically:

| Task | Free lane | Claude-only reason |
|------|-----------|-------------------|
| P1 design note | Qwen THINK (free) | None -- free lane first |
| P1 CODER impl | Qwen CODER (free) | None |
| P1 fable audit | [AUDIT-GATE] fable | Transport code, mandatory adversarial |
| P2 | Qwen CODER (free) | None |
| P3 | Qwen CODER (free) | None |
| P4 | Qwen CODER (free) | None |
| P5 | Orchestrator-local | No dispatch needed |
| P6 | Qwen HAIKU or orchestrator-local | None |

The ONLY Claude worker spend needed for v0.4.0 is the P1 fable adversarial
review. Everything else runs on free-tier Qwen or groq or is orchestrator-local.
Estimated Claude quota burn: 1 fable/high worker (~5-10% window).

---

## RISK REGISTER

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Dial still not real (f2831458 fix incomplete) | Medium | Blocks everything | Verify ss -tn on relay before dispatching other tasks |
| P2 patch fails again (enqueue timing) | Low | 1-2 extra rounds | Fix is precisely identified; worker has exact bug description |
| A-04 silent failure again | Medium | P4 delay | Upgrade to qwen3-235b or qwen3-max; split into smaller prompt |
| D-05 compile errors from UDL leftovers | Medium | P5 blocker | Check git diff api.udl first; revert before dispatching |
| CI stays red after P0a push | Low | Blocks P7 | CI fix commit bc94ffbb was verified locally; likely green |
| Josh EC2 emulator still slow (no KVM) | High | Delays real test | Use second CLI as "Josh" substitute first (proven faster path) |
| fable audit for P1 finds another circuit-relay bug | Medium | Delay | Expected; items 3+4 are the riskier changes in this task |

---

## WHAT "DONE FOR v0.4.0" MEANS

v0.4.0 is done when ALL of these are true:
1. CI green on main (all 4 jobs: Lint, FFI Surface, Test x3).
2. P1 (graceful dial 3+4) committed + adversarial audit on file.
3. P2 (outbox Site-1 flush) committed + `test_flush_peer_messages` green.
4. P3 (Android retry suppression) committed.
5. P4 (A-04 Android receipt unification) committed.
6. P5 (D-05 compile gate) committed.
7. PROVE_SECOND_REAL_ENDPOINT_DELIVERY verified (not just claimed done).
8. GitHub Release `v1.0.0-alpha.1` published (Lucas's call, operator action).

Items NOT required:
- Full PQC (E-01b through PQC-14 are v1.0.0 scope)
- iOS build
- Farm simulation
- GRACEFUL_AF_DIAL full A-09 closure (items 3+4 of graceful-dial ARE required
  to prevent the promiscuous-dial noise from poisoning the Josh test;
  full A-09 auth/connection_limits is v0.5.0)
- KMP desktop
