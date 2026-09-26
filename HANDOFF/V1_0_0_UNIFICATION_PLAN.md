# SCMessenger v1.0.0 Unification Plan

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**SUPERSEDED (2026-07-04):** for sequencing and scope, use `HANDOFF/V1_0_0_EXECUTION_PLAN.md`
(two-phase DAG: Android<->Windows transport parity first, then everything else — all of it
ship-blocking). This file remains the audit record; its 8 findings were re-verified there
(Section 0.1): findings 1-6 closed, finding 7 (uncommitted working tree) still open,
finding 8 replaced by lane rules. The Phase 4 KMP scope question below is settled: IN
scope for v1.0.0.

**Generated:** 2026-07-03
**Author:** Claude (native Cowork session), cross-referenced against agy/Antigravity (Gemini 3.5 Flash) live session output
**Purpose:** One coherent, priority-ordered plan to close every remaining gap between the current `main` (`5363d1aa`, workspace v0.3.4) and a "perfect," fully-wired v1.0.0. Supersedes fragment-level tracking in `REMAINING_WORK_TRACKING.md` (1313 lines, substantially historical/superseded — see Finding 1 below).

---

## 0. Critical findings from the audit pass (read this first)

These change the shape of the plan versus what the input documents claim at face value.

**Finding 1 — The 350-task wiring backlog is already done, not open.**
`HANDOFF/WIRING_TASK_INDEX.md` and `HANDOFF/WIRING_MASTER_EXECUTION_PLAN.md` both describe "350 tasks" as the current inventory. Verified directly: `HANDOFF/todo/` contains **zero** `task_wire_*` files; `HANDOFF/done/` contains **351**. The index simply was never regenerated after the batches (B1-B8) completed. **Action: regenerate `WIRING_TASK_INDEX.md` and `WIRING_PATCH_MANIFEST.{json,md}` before anyone (agent or human) trusts them again** — this is a 10-minute mechanical task (`python scripts/generate_wiring_patch_manifest.py`) but skipping it risks a future session re-dispatching 350 already-done tasks.

**Finding 2 — The "39 dead_code items" audit you pasted substantially overlaps Finding 1.**
Every dead-code item in that audit (`ratchet.rs`, `relay/server.rs`, `routing/optimized_engine.rs`, `cli/api.rs`, `history.rs`, etc.) is a named target in the wiring backlog that is now in `HANDOFF/done/`. This needs verification per-item (task 2 below), not blanket trust — some `#[allow(dead_code)]` annotations may be stale leftovers even after the wiring task closed, or may be legitimate platform stubs the wiring task correctly left alone. Treat the dead-code audit as a **verification checklist against already-done work**, not a new backlog.

**Finding 3 — `REMAINING_WORK_TRACKING.md` is a 1313-line changelog, not a live backlog.** Only its top ~150 lines (the 2026-07-03 PQC section and 2026-07-02 release-readiness section) reflect current state. Below that it is dated history back to March, with entries that were themselves later reversed or superseded further down the same file (e.g., an "Android ANR Storm — CRITICAL" entry from 2026-03-19 that later sessions fixed). **Do not let any agent — including a fresh Claude or a fresh agy session — treat this file as a task queue.** Recommend archiving everything below the 2026-07-02 entry to `docs/historical/` and keeping this file as a short, current-state-only doc, per CLAUDE.md's own instruction that historical docs shouldn't be treated as execution truth.

**Finding 4 — The real, trustworthy backlog is `docs/release-readiness-2026-07-02.md`.** This is the one document in the repo that distinguishes verified-by-actually-running-the-command from claimed-but-unverified, and it is current (references the latest merge, `cbec1f4`). It is the anchor for this plan's Phase 2 and Phase 3.

**Finding 5 — Version: 0.3.4 confirmed correct** (per user, cross-checked against Android phone build, and matches `Cargo.toml` / everything agy is compiling against). CLAUDE.md's "Active release line: v0.2.1 alpha" line is stale and will misdirect any session that reads it first. Fixed as part of this plan (Phase 0).

**Finding 6 — Two of the eight non-PQC `HANDOFF/todo/` items are stale false positives.** `P1_ANDROID_CRASH_TRIAGE.md` and `P1_ANDROID_LAN_DISCOVERY_REPAIR.md` (both dated 2026-06-04) describe bugs already fixed by commit `87d1ef61 fix(android): FAB reappear + TCP subnet probe for LAN discovery`: `SubnetProbe.kt` exists, the nested-Scaffold fix is in `MeshApp.kt` with an explanatory comment, and a global crash handler is installed in `MeshApplication.kt`. **Action: close both with a "verified already fixed" note rather than re-dispatching them** (task 1 below).

**Finding 7 — Working tree has ~183 modified, uncommitted files unrelated to any task**, spanning CI workflows, `.mimocode/` plan files, `Cargo.lock/toml`, Android sources, docs. This predates agy's PQC-01 commit and is still present after it (agy's `git status` mid-session showed only its own 5 files changed, confirming this bulk diff is separate, pre-existing state, not agy's doing). **This needs your call before heavy parallel work starts** — see Phase 0.

**Finding 8 — Two active workstreams are running on the same repo right now**: this Claude/Cowork session (sandboxed, no local Rust toolchain, cannot commit — see below) and the agy/Antigravity session on your actual Windows machine (has full toolchain, is committing). They must not both try to drive PQC or wiring work simultaneously without a lane-assignment, or they will race on the same files (`iron_core.rs` already got hand-edited by agy mid-PQC-01 to fix an unrelated WASM bug — exactly the kind of hotspot-file collision `WIRING_MASTER_EXECUTION_PLAN.md` itself warns about).

---

## 1. Constraints this plan must respect

- I (Claude, this session) run in a sandboxed Linux environment with no Rust toolchain and cannot delete/reset `.git/index.lock` — I **cannot** run `cargo build/test` or `git commit` against this repo myself. I can read, edit files, delegate to subagents, and reason/plan. Any step requiring a real build/test gate must run on your machine (via agy, via a future native Claude Code session with proper tool access, or by you directly).
- agy (Gemini 3.5 Flash via Antigravity) has the full toolchain locally and is already mid-execution on PQC. It should keep the lane it's in rather than being interrupted.
- CLAUDE.md's escalation rules stand: architecture changes, security/privacy tradeoffs, stack migrations, API-contract breaks, and release timing decisions get a stop-and-ask, not silent agent judgment. The PQC workstream and (especially) the KMP desktop-client initiative below both brush up against these.

---

## 2. Prioritized plan

### Phase 0 — Hygiene (do first, low-risk, unblocks everything else)

1. **Decide on the 183 uncommitted files** (Finding 7). Recommend: you or agy run `git status --short` fresh, eyeball whether it's leftover from a prior merge/rebase or genuine in-flight work, then either commit it under an honest message or stash it. Nothing else below should build on top of an unknown 183-file diff.
2. **Fix CLAUDE.md's version line** (Finding 5): change "Active release line: v0.2.1 alpha (v0.2.0 was the baseline)" to reflect 0.3.4. Small, mechanical, prevents every future session from anchoring on stale info.
3. **Regenerate wiring manifests** (Finding 1): run `python scripts/generate_wiring_patch_manifest.py`, confirm `WIRING_TASK_INDEX.md` shows 0 remaining, and add a one-line note at the top of `WIRING_MASTER_EXECUTION_PLAN.md` marking the wiring workstream complete as of this date so nobody re-opens it.
4. **Close the 2 stale Android todo items** (Finding 6): move `P1_ANDROID_CRASH_TRIAGE.md` and `P1_ANDROID_LAN_DISCOVERY_REPAIR.md` to `HANDOFF/done/` with a note citing `87d1ef61` as the fix commit, after a quick spot-check that the described symptoms don't still reproduce.
5. **Archive stale `REMAINING_WORK_TRACKING.md` history** (Finding 3): move everything below the 2026-07-02 entry into `docs/historical/REMAINING_WORK_TRACKING_ARCHIVE_2026.md`, keep only current-state sections in the live file.

Owner: native Claude via subagents (docs-sync-auditor / rust-implementer for the mechanical parts), no build gate required for most of these (docs + file moves), except item 3 which needs a real `cargo`/script run — delegate that one step to agy or run it yourself.

### Phase 1 — PQC migration (in progress, agy's lane)

Let agy continue. Sequence per `PQC_00_MASTER_PLAN.md`'s wave structure:

- **Wave 0** (parallelizable): PQC-01 (**done**, `5363d1aa`), PQC-02 Envelope v2, PQC-03 Identity v2 bundle (needs PQC-01, now unblocked).
- **Wave 1**: PQC-04 suite negotiation, PQC-05 hybrid KEM module (needs adversarial review — mandatory per security.md, this is a hard gate, not optional).
- **Wave 2**: PQC-06 hybrid session init (adversarial review).
- **Wave 3**: PQC-07 PQ ratchet (**highest risk, Sonnet-tier only, auditor + gatekeeper** — do not let a smaller/faster model touch this one), PQC-09 hybrid onion, PQC-10 ML-DSA signatures.
- **Wave 4**: PQC-08 legacy path retirement, PQC-11 relay/invite dual-sig, PQC-12 TLS PQ groups.
- **Wave 5**: PQC-13 verification suite (Kani/proptest/cross-version matrix), PQC-14 docs + risk register closure.

Recommendation: after PQC-05 (first crypto-module change past the smoke-test stage), pause and get a `crypto-security-auditor` pass — either dispatched by me via the `Agent` tool if I have a way to hand it the diff, or by you asking agy to self-review against the same adversarial checklist in `.claude/rules/security.md` (race conditions, timing side channels, null checks, edge-case failures). Don't let 3+ waves of crypto work land before the first security checkpoint.

### Phase 2 — Release-readiness punch list (`docs/release-readiness-2026-07-02.md`)

This is concrete, evidence-backed, and mostly small. Ordered as the doc itself recommends (T1 → T3 → T4 → T2 → S4 → T5 → T6 → T15 → T8 → rest), grouped by blocker type:

**Human-only, blocking everything downstream of CI:**
- **H1 — GitHub Actions runners are dead** (billing/quota, not code — every job completes in 1-2s with no runner assigned). This has been true since at least 2026-06-15. Until fixed, no CI result on any branch means anything. **This is your highest-leverage single action** — nothing here can get real CI-verified without it.
- **H2 — Physical-device tests** (WiFi Aware/Direct, BLE, DTN mule) — needs hardware, cannot be delegated.
- H3 (PR #1 merge decision) — already done, merged as `cbec1f4`.

**Model-executable, small, no dependencies — good first batch for a subagent:**
- S2 (add `libdbus-1-dev` to CI Linux jobs — one line in `ci.yml`)
- S3 (make `ffi_surface.sh` fail closed instead of vacuously passing)
- S6 (fence `identity_signing_key_for_test` behind `#[doc(hidden)]` + rename)
- S7 (normalize remaining CRLF Rust files + `.gitattributes` pin)

**Model-executable, correctness bugs found by PR review (T1-T7, Rust/CLI, verifiable via `cargo test` — no hardware needed):**
- T1 (identity backup exports wrong contact store for mobile — real data-loss bug on restore)
- T2 (CLI message-request flow reads/writes wrong contact store + first-match-only key lookup bug)
- T3 (backup import "validates" while silently dropping corrupted ratchet sessions)
- T4 (import swallows contact-persist failures — breaks all-or-nothing guarantee)
- T5 (export masks contact storage-read failures)
- T6 (potential bincode compat break for old inbox records — needs exposure check first)
- T7 (flaky timing-based Argon2id test — replace with known-answer test)
- S4 (contact key-prefix migration, needed for T1/T2 to be safe)
- S5 (stop rendering all-zero safety numbers as real — a genuine trust/security UX bug: two clients with malformed keys currently render the *same* fake "verified" number)

**Mobile, PLAUSIBLE-verified by reading only — need CI (post-H1) or local SDK to prove, but safe to implement now:**
- T8-T13 (Android: QR composable perf/cache bugs, safety-number memoization, import dialog state leak, UTF-16 vs codepoint length bug, WiFi Aware loopback proxy — 3 real defects in one file, test package name mismatch)
- T14-T17 (iOS: import sheet dismisses before showing result, verification errors swallowed silently, synchronous KDF freezes UI thread, background-service tests race)
- T18 (housekeeping: resolve the 26 open PR review threads as each fix lands)

**S1, S8, S9** are already done or gated on S1 (S1 says "Done — executed on this branch after the merge").

### Phase 3 — Dead-code triage (cross-referenced against Finding 1/2)

For each of the 39 items in your pasted audit: grep for real callers now that the wiring backlog has landed, then classify as (a) confirmed wired now, safe to remove the `#[allow(dead_code)]` annotation, (b) legitimate platform stub — add a doc comment explaining the gate instead of leaving a bare annotation, or (c) genuinely dead — remove. Given Finding 1/2, I expect the large majority to resolve to (a) or (b) already, making this mostly a documentation pass rather than new implementation work. The one unambiguous action already identified: remove the 4 unused test-only imports in `core/src/transport/swarm.rs:5352-5353` — trivial, zero risk.

### Phase 4 — KMP desktop client (separate initiative, explicit scope call needed)

`TASK_KMP_COMPOSE_ARCHITECT.md` / `TASK_KMP_DEVOPS_PACKAGING.md` / `TASK_KMP_QA_INTEROP.md` / `TASK_KMP_RUST_UNIFFI_LINUX.md` describe a **new Linux desktop Compose Multiplatform client** — this is net-new architecture, not a bug fix or wiring gap, and it's the kind of "technology stack addition" CLAUDE.md's escalation rules flag for explicit sign-off. It doesn't block v1.0.0 of the existing Android/iOS/WASM/CLI product unless you consider desktop-Linux a v1.0.0 requirement.

**This needs your decision, not mine:** is a Linux desktop client in scope for v1.0.0, or is it a post-1.0 initiative? I'd rather ask than assume given it's a multi-week architectural addition (new Gradle/KMP module structure, DI framework change from Hilt to Koin, a full desktop UI layer) sitting in the same `todo/` folder as one-line bug fixes.

---

## 3. Suggested sequencing summary

1. Phase 0 hygiene (fast, unblocks trust in all tracking docs)
2. Phase 1 continues in parallel on agy's machine (already moving, don't interrupt)
3. Phase 2's small/no-dependency items (S2, S3, S6, S7) and Rust-verifiable correctness bugs (T1-T7, S4, S5) — dispatchable to subagents now, no hardware/CI needed
4. Phase 3 dead-code triage — mostly documentation, low risk, can run anytime, good filler between PQC waves
5. Phase 2's mobile items (T8-T18) — implement now, verification waits on H1 (CI) or your local Android Studio/Xcode
6. H1/H2 — yours to action outside any agent's reach
7. Phase 4 — pending your scope decision

---

## 4. What I need from you to keep moving

- Confirm the Phase 0 items are fine to execute (especially archiving `REMAINING_WORK_TRACKING.md` history and closing the 2 stale Android tasks).
- Say whether I should start dispatching Phase 2/3 subagent work now (I can plan and delegate via the `Agent` tool, but recall I can't commit from this sandbox — output would need to land via a subagent that has real git access, or you'd pull my proposed diffs manually).
- Answer the Phase 4 scope question (KMP desktop: in or out of v1.0.0).
- Let me know when agy finishes its current PQC task so I can sanity-check the next wave before it starts, per the security-review gate above.
