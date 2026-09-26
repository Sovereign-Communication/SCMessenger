NOTE (2026-07-04): The wiring workstream (350 tasks) completed as of 2026-07-03, verified via HANDOFF/done/ file count (351 task_wire_* files present, 0 remaining in HANDOFF/todo/). WIRING_TASK_INDEX.md and WIRING_PATCH_MANIFEST.{json,md} have now been regenerated (`python scripts/generate_wiring_patch_manifest.py`, run 2026-07-04) — WIRING_PATCH_MANIFEST.md confirms `Total tasks: 0`; WIRING_TASK_INDEX.md retains the original 350-row table as a historical record with a closure note prepended. This workstream is fully closed; do not re-dispatch.

# SCMessenger Wiring Master Execution Plan (All HANDOFF/todo Tasks)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## 1) Scope + Success Definition

This plan covers **every** wiring task currently in `HANDOFF/todo` and is designed to drive end-to-end runtime functionality across all app variants (Core, Android, WASM, CLI), while preserving feature parity and regression safety.

**Authoritative task inventory:** `HANDOFF/WIRING_TASK_INDEX.md` (350 tasks snapshot).

A task is only considered complete when all of the following are true:
1. Function is reachable from at least one production call path (not only tests).
2. Integration compiles for the edited target(s).
3. Behavior is exercised by automated test(s) or deterministic integration check(s).
4. Feature parity documentation is updated when user-visible/runtime behavior changes.
5. Task file is moved from `HANDOFF/todo` to `HANDOFF/done` with evidence links.

---

## 2) Current Inventory Snapshot

From `HANDOFF/todo` (350 tasks):
- **Core:** 175
- **Android:** 129
- **WASM:** 30
- **CLI:** 16

Primary hotspot targets:
- `android/.../MeshRepository.kt` (27)
- `core/src/lib.rs` (23)
- `wasm/src/lib.rs` (16)
- `core/src/store/relay_custody.rs` (14)
- `core/src/mobile_bridge.rs` (13)
- `core/src/transport/swarm.rs` (12)
- `android/.../MdnsServiceDiscovery.kt` (12)

Implication: a naive task-by-task order will thrash the same files repeatedly and create merge risk. We should execute by **target cluster + runtime domain**, not by filename alphabetically.

---

## 3) Execution Architecture (How to Finish All 350 Reliably)

## Phase A — Baseline + Guardrails (Day 0)
1. Freeze a baseline branch and record:
   - current task count
   - per-variant task count
   - current green checks per variant
2. Establish mandatory closeout template for each task:
   - `Wired call path:`
   - `Files touched:`
   - `Build/test evidence:`
   - `Parity/doc updates:`
3. Pre-create batched epic trackers (one per workstream below) in `HANDOFF/backlog` to avoid drift.

## Phase B — Batch by Hotspot (Days 1–N)
Execute in this order to minimize rebases and regressions:

1. **Core runtime entrypoints** (`core/src/lib.rs`, `core/src/mobile_bridge.rs`, `core/src/transport/swarm.rs`)
2. **Android orchestration** (`MeshRepository.kt`, transport + service classes)
3. **Relay/custody + routing optimization** (`relay_custody.rs`, routing modules)
4. **WASM bridge + daemon mode** (`wasm/src/lib.rs`, `daemon_bridge.rs`, `notification_manager.rs`)
5. **CLI transport bridge + parity completion**
6. **Cross-cutting notification/privacy/diagnostics consistency pass**

Each batch should include all related tasks for that hotspot before moving on.

## Phase C — Variant Gate Validation (after each batch)
Run only relevant checks for edited targets, then run nightly full matrix.

## Phase D — Documentation + Queue Hygiene
After each merged batch:
- move completed task markdown files to `HANDOFF/done`
- regenerate `HANDOFF/WIRING_TASK_INDEX.md`
- update parity/status docs when behavioral state changes

## Phase E — Final Hardening + Release Readiness
- full multi-variant regression
- unresolved flake triage
- confirm `HANDOFF/todo` is empty (or only explicitly deferred items with rationale)

---

## 4) Workstream Breakdown (Comprehensive)

## WS1 — Core API Entrypoint Wiring
**Targets:** `core/src/lib.rs`, `core/src/mobile_bridge.rs`.

**Task themes:**
- identity/daemon mode flows (`get_identity_from_daemon`, `initialize_identity_from_daemon`, mode setters/getters)
- audit log getters/export/validation wiring
- privacy config getters/setters and policy application
- history/contact manager getter wiring

**Plan:**
1. Map each unwired function to caller candidates (bridge API, UI adapters, CLI).
2. Add production call path and safe error propagation policy.
3. Add/adjust tests per function family (unit + bridge-facing integration).
4. Run core + bridge checks before Android/WASM consumers.

---

## WS2 — Transport + Relay + Routing Core Wiring
**Targets:** `core/src/transport/swarm.rs`, `core/src/store/relay_custody.rs`, `core/src/routing/*`, `core/src/transport/health.rs`.

**Task themes:**
- relay registration/discovery bookkeeping
- path selection health scoring
- custody transition recording and dedup invariants
- reconnect/timeout budget diagnostics

**Plan:**
1. Land custody invariants first (dedup, persistence, transition audit).
2. Wire transport health metrics APIs and summary getters.
3. Integrate routing optimizer hooks and relay scoring adjustments.
4. Add deterministic simulation tests for relay/path edge conditions.

---

## WS3 — Android Repository + Service Wiring
**Targets:**
- `android/.../MeshRepository.kt`
- `android/.../service/*`
- `android/.../transport/*`
- `android/.../utils/NotificationHelper.kt`

**Task themes:**
- UI/VM to repository passthrough methods
- service lifecycle callbacks and event bus forwarding
- BLE/Wi-Fi/mDNS transport start/stop/recovery paths
- notification state management and reset/stat methods

**Plan:**
1. Close all repository passthrough wiring in one sweep (avoid repeated conflicts).
2. Wire background/foreground + permission flows.
3. Complete transport callback wiring and recovery fallback paths.
4. Validate with instrumented/log-driven flows for discovery/send/receive.

---

## WS4 — Android UI Wiring Completeness
**Targets:** Android compose/UI screens (`MeshSettingsScreen`, `PowerSettingsScreen`, `PeerListScreen`, banners, message input, etc.).

**Task themes:**
- screens/components currently defined but not invoked
- callback propagation from UI controls to view model/repository
- state rendering for error/info/warning/connection-quality components

**Plan:**
1. Build nav graph/component invocation map.
2. Wire all orphaned composables into real routes/parents.
3. Verify state sources are non-stub and repository-backed.
4. Add screenshot-based validation for user-visible changes where applicable.

---

## WS5 — WASM Daemon/Browser Mode Wiring
**Targets:** `wasm/src/lib.rs`, `wasm/src/daemon_bridge.rs`, `wasm/src/notification_manager.rs`, `wasm/src/transport.rs`.

**Task themes:**
- daemon identity fetch and JSON-RPC roundtrips
- browser option getters, socket URL setup, mode toggles
- notification behavior (foreground/background/request inference)

**Plan:**
1. Finalize mode-gated identity flow (daemon-first when configured).
2. Wire message send/receive helpers through JSON-RPC where required.
3. Complete notification decision and browser permission integration.
4. Run wasm-bindgen tests and headless/browser smoke checks.

---

## WS6 — CLI Bridge Completion
**Targets:** `cli/src/transport_bridge.rs` and related command surfaces.

**Task themes:**
- transport bridge helpers and visibility/status commands
- diagnostics surfaces to match core parity

**Plan:**
1. Ensure CLI can invoke all required core-exposed wiring endpoints.
2. Add CLI integration tests for bridge/reporting commands.

---

## WS7 — Notifications + DM Classification Cross-Variant
**Targets:** core notification module + Android/WASM adapters.

**Task themes:**
- delivery suppression/config toggles
- unknown sender vs known contact request classification
- per-platform notification stats/reset parity

**Plan:**
1. Enforce shared classifier behavior in core.
2. Confirm adapters preserve classifier decision semantics.
3. Validate parity in feature matrix docs.

---

## WS8 — Security/Crypto/Protocol Wiring Tasks
**Targets:** crypto harness + registration/deregistration verification tasks.

**Task themes:**
- payload shape rejection
- signature verification guardrails
- ratchet/session invariants

**Plan:**
1. Prioritize tasks that affect on-wire trust decisions.
2. Add property/invariant tests before/with wiring.

---

## WS9 — Performance/ANR/Health Telemetry Wiring
**Targets:** Android performance monitor + core health diagnostics.

**Task themes:**
- ANR record/query/reset APIs
- timeout budget and storage pressure summaries
- transport failure reason surfacing

**Plan:**
1. Wire capture first, then reporting getters, then reset flows.
2. Validate telemetry appears in user- or dev-facing diagnostics exports.

---

## WS10 — Build/Supply-Chain Pipeline Wiring Tasks
**Targets:** tasks named around optimization/security audit pipelines.

**Plan:**
1. Ensure pipelines are callable in CI and local scripts.
2. Attach artifacts and fail-fast thresholds.

---

## WS11 — Cross-Variant Parity Gate
For each completed batch, validate parity across:
- Core API surface availability
- Android adapter wiring
- WASM adapter wiring
- CLI access (if applicable)

Use `docs/FEATURE_PARITY.md` as canonical state table update point.

---

## WS12 — Final Burn-Down Protocol
1. Re-run unwired-function detector.
2. Compare against `HANDOFF/WIRING_TASK_INDEX.md`.
3. Any surviving task must be either:
   - completed and moved to `done`, or
   - deferred with explicit blocker + owner + date.

---

## 5) Batch Sizing + Throughput Targets

Given 350 tasks, recommended batch model:
- **Batch size:** 20–35 related tasks
- **Expected batches:** 8 (B1..B8)
- **Cadence:** each batch includes code + checks + docs + task moves

Suggested order:
1. Core entrypoints (`B1`)
2. Core transport/routing (`B2`)
3. Android repository (`B3`)
4. Android UI (`B4`)
5. Android transport/service (`B5`)
6. WASM (`B6`)
7. CLI (`B7`)
8. Cross-cutting closure (`B8`)

---

## 6) Quality Gates (Must Pass)

For every wiring batch:
1. **Compile gates** for edited platforms.
2. **Behavior gates** (tests/integration scripts/log evidence).
3. **Parity gate** (no accidental variant regression).
4. **Observability gate** (diagnostics reflect new path).
5. **Documentation gate** (task file + index + parity/state docs updated).

If any gate fails, batch cannot be marked complete.

---

## 7) Risk Register + Mitigation

1. **Hotspot merge conflicts** (`core/src/lib.rs`, `android/.../MeshRepository.kt`, `wasm/src/lib.rs`)
   - Mitigation: strict hotspot-batching and short-lived branches.
2. **False “wired” claims from test-only call paths**
   - Mitigation: require production call-path evidence in task closeout.
3. **Cross-variant drift**
   - Mitigation: parity gate after each batch, not just at end.
4. **Transport regressions hidden by flaky environment**
   - Mitigation: deterministic unit/integration checks + log signatures.
5. **Queue hygiene failure**
   - Mitigation: mandatory task move + index regeneration in same PR.

---

## 8) Operational Checklist (Per Task)

1. Open task file in `HANDOFF/todo`.
2. Confirm target + function + expected caller chain.
3. Implement production wiring.
4. Add/update tests.
5. Run edited-target checks.
6. Update task evidence section.
7. Move task to `HANDOFF/done`.
8. Regenerate index file.
9. Recount by variant and confirm downward trend.

---

## 9) “Fully Functional Across Variants” Exit Criteria

All criteria required:
- `HANDOFF/todo` has no unresolved wiring tasks.
- Core/Android/WASM/CLI compile gates pass on release configs.
- Critical runtime flows validated end-to-end (identity, send/receive, transport failover, notifications, diagnostics).
- No known unwired production functions in scanned targets.
- Feature parity docs reflect true implemented state.

---


## 10) Exact Patch Preparation Artifacts

To eliminate ambiguity before implementation, use these generated artifacts:
- `HANDOFF/WIRING_PATCH_MANIFEST.json` — machine-readable patch queue with exact file + anchor line for each task.
- `HANDOFF/WIRING_PATCH_MANIFEST.md` — human-readable patch queue grouped by execution batch.
- `scripts/generate_wiring_patch_manifest.py` — regeneration script (run after task moves or refactors).
- Generator behavior: fails fast if any task has unresolved anchor coordinates; known task→symbol aliases are recorded in manifest as `resolved_symbol`.

**Required workflow before touching implementation code:**
1. Regenerate manifest from current `HANDOFF/todo`.
2. Pick the next batch from manifest (`B1`..`B8`).
3. For each task, stage only the listed target file at the listed anchor line plus the required caller/test files.
4. Keep each PR scoped to one batch to avoid hotspot merge churn.

