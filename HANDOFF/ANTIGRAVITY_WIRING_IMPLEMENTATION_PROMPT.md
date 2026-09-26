# Antigravity Prompt — Implement All SCMessenger Wiring Tasks (Gemini 3 Flash)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

You are implementing the full wiring backlog for SCMessenger. Be aggressive, systematic, and honest. Do not hand-wave. Ship real wiring, real tests, real verification.

## Mission
Complete **all wiring tasks** currently listed in `HANDOFF/todo/`, using the generated planning artifacts as the source of truth, and ensure no regressions across Core/Android/WASM/CLI.

## Mandatory Inputs (Read First)
1. `HANDOFF/WIRING_MASTER_EXECUTION_PLAN.md`
2. `HANDOFF/WIRING_PATCH_MANIFEST.json`
3. `HANDOFF/WIRING_PATCH_MANIFEST.md`
4. `HANDOFF/WIRING_TASK_INDEX.md`

Treat these files as operational requirements, not optional references.

---

## Hard Constraints
1. **Implement actual wiring code** (not just docs).
2. For each completed task:
   - Move task file from `HANDOFF/todo/` to `HANDOFF/done/`.
   - Include proof (files changed + checks run).
3. Do not claim completion unless production call paths are wired (test-only references do not count).
4. Keep PRs/batches scoped by manifest batch groups to reduce merge conflicts.
5. Run edited-target build/test gates before ending each batch.
6. If a task is blocked, document exact blocker, owner, and next action; do not silently skip.
7. **CRITICAL: LOOK BEFORE YOU LEAP** — Before you wire a function call to an external struct or trait (e.g., `BootstrapManager`, `OptimizedRoutingEngine`, `AutoAdjustEngine`), you MUST use `grep` or `read` on the target file to prove the method exists. If the method DOES NOT exist, DO NOT hallucinate it. Document the missing API as a BLOCKER in your task file, leave a `todo!()` or empty stub, and immediately move on to the next task. Never fabricate method signatures.
8. **NO STUBS** — Every wired function must have real production data flow. Do not add placeholder implementations that return empty vectors, `false`, `None`, or hardcoded values. If the underlying subsystem doesn't have the method yet, implement it there too, or document it as a BLOCKER.
9. **UniFFI Rules** — Do NOT add arbitrary lines to `api.udl`. Any new exposed IronCore methods MUST be written directly inside the `impl IronCore` block in Rust and tagged with `#[uniffi::export]`. The UDL will generate automatically from proc macros.

---

## Execution Strategy (Use This Exactly)

## Step 0 — Baseline Snapshot
- Record current totals:
  - `HANDOFF/todo` count
  - `HANDOFF/done` count
  - manifest `total_tasks`
- Confirm clean working tree.

## Step 1 — Batch Selection
Use `HANDOFF/WIRING_PATCH_MANIFEST.md` and process in order:
1. `B1-core-entrypoints`
2. `B2-core-transport-routing`
3. `B3-android-repository`
4. `B4-android-ui`
5. `B5-android-transport-service`
6. `B6-wasm`
7. `B7-cli`
8. `B8-cross-cutting`

Within a batch, work task-by-task using the exact target and anchor context from `HANDOFF/WIRING_PATCH_MANIFEST.json`.

## Step 2 — Implement Task Wiring
For each task entry:
1. Open task file in `HANDOFF/todo`.
2. Open target file from manifest.
3. Wire function into real runtime call path(s).
4. Add/adjust tests that validate behavior and prevent regression.
5. Ensure no dead code or orphaned entrypoints remain.

## Step 3 — Task Closeout (Required)
For each completed task, append evidence to the task file before moving it:
- Wired call path(s)
- Files modified
- Build/test commands + outcomes
- Notes on parity impact (Android/iOS/WASM/CLI if applicable)

Then move task file to `HANDOFF/done/`.

## Step 4 — Rebuild Manifests After Each Batch
Run generator:
- `python scripts/generate_wiring_patch_manifest.py`

This keeps remaining tasks and anchors current after refactors.

## Step 5 — Batch Gate (No Exceptions)
At end of each batch, run only relevant gates for changed areas plus at least one cross-variant sanity gate.

Minimum expected checks by area:
- Core/CLI: `cargo test` (or targeted workspace tests if runtime constrained)
- WASM: wasm-target tests/checks used by repo
- Android: Gradle compile/lint/test tasks for touched modules

If environment blocks a gate, explicitly state limitation and run nearest equivalent deterministic check.

---

## Regression Checkpoints (Critical)
At the end of every batch, verify:
1. No new unwired public functions were introduced.
2. Existing wired paths still compile and execute for touched variants.
3. Feature parity did not regress in touched surface areas.
4. No task moved to done without verification evidence.
5. `HANDOFF/WIRING_PATCH_MANIFEST.json` reflects reduced remaining count.

Every 2 batches, also run a broader integration checkpoint:
- message send/receive flow
- identity init/load flow
- transport start/stop/failover flow
- notifications/diagnostics flow (if touched)

---

## Commit/PR Cadence
- One commit sequence per batch (or smaller if risky hotspot file).
- PR body must include:
  - tasks completed (exact filenames moved)
  - runtime call paths added
  - tests/checks run
  - any blockers and follow-up task(s)

Do not mix unrelated batch groups in one PR unless necessary for compile integrity.

---

## Quality Bar (Definition of Done)
A task is done only if all are true:
- Function reachable via production execution path.
- Edited target compiles.
- Behavior verified by tests/checks.
- Task moved to `HANDOFF/done` with evidence.
- Manifests regenerated.

Project done only if:
- `HANDOFF/todo` has zero unresolved wiring tasks (or explicit blocker docs).
- Cross-variant checks are green or documented with concrete environment limitation.
- No known wiring regressions introduced.

---

## Anti-Failure Rules
- Do not mark “done” based on grep hits alone.
- Do not skip tests on hotspot files.
- Do not leave stale manifest/index after task moves.
- Do not hide uncertainty: if confidence < high, say so and add validation.

---

## Suggested Operator Output Format (Per Batch)
1. Batch ID + scope
2. Tasks completed (with moved files)
3. Wiring changes (runtime paths)
4. Tests/checks run with pass/fail
5. Regressions found/fixed
6. Remaining task count
7. Blockers (if any)

Be ruthless about correctness. Finish the wiring backlog end-to-end.
