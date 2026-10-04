# Unify the codebase — de-conflict scripts, docs, and agent contracts

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Created: 2026-08-15
Owner: CTO session (dispatches), operator (approves deletions)
Cadence: **filler work.** Run this while blocked on CI, never instead of the
critical path. If SHIP_PLAN D1-D5 can move, move that first.

A unified codebase is not tidiness. Today an orchestrator test failed because
two delegation scripts exist and the SOP points at the deprecated one; the model
picked the documented-but-wrong path and did exactly what it was told. Ambiguity
in the repo becomes wrong behaviour in every agent that reads it.

---

## THE STANDARD FLOW (use this shape for every de-confliction round)

**Phase 1 — CALL OUT ONLY.** Produce a list of conflicts. Change nothing, delete
nothing, merge nothing. Deliverable is a table: what conflicts, where, and what
depends on each side. A phase-1 pass that edits a file has failed.

**Phase 2 — SAFE PRUNE + PLAN.** For each conflict, classify:
  - `SAFE`      — provably unreferenced, or an exact duplicate. Prune.
  - `PLAN`      — needs a migration (deprecate, redirect, merge content).
  - `CALLOUT`   — cannot be resolved without a judgement call.
  Evidence required per row: the grep/command that proves nothing depends on it.

**Phase 3 — SURFACE TO CTO.** Every `CALLOUT` comes to the CTO session with the
evidence attached. CTO decides.

**Phase 4 — ESCALATE AT <99% CONFIDENCE.** If the CTO is not ≥99% confident, it
goes to the CEO session ("SCMessenger CEO strategy") for confirmation. No
consensus → TABLE it, record it here, move on. Never block on an unanswered
question; there is always another conflict to work.

**Deletion rule:** nothing is deleted in the same round it is identified. A round
that both finds and removes has skipped the review that makes it safe.

---

## PHASE 1 FINDINGS — 2026-08-15 (first pass, nothing changed)

### C1. Two delegation scripts, SOP points at the deprecated one — CONFIRMED HARM
- `scripts/delegate_task.py` (mainline, 41.6 KB) vs `scripts/delegate.py`
  (on `chore/delegation-lane-routing`, PR #150).
- `docs/rules/DELEGATION.md` on mainline has a full `## scripts/delegate_task.py`
  section and **zero** mentions of `delegate.py`.
- Caused a real failure today: an orchestrator read the SOP, used
  `delegate_task.py`, and dispatched to lanes that are dead (qwen/dashscope
  return HTTP 401).
- Disposition: **PLAN.** Merge PR #150, then make `delegate_task.py` a thin
  deprecation pointer. Do not delete it in the same change.

### C2. `orchestrat*` scripts — RESOLVED 2026-08-15, and the first pass was wrong

Phase 1 called this "eight scripts with unclear division of labour" and guessed
that `test_orchestration.sh` / `test_orchestration_v2.py` were a versioned pair
with one dead. Reading them says otherwise on both counts.

Six of the eight are already the cohesive layered system this ticket was going
to propose building — a kernel plus adapters, the same shape as C4/C5:

| Script | Role (from its own docstring) |
|---|---|
| `orchestrate_strict.py` (725) | the KERNEL — "a composition layer, not a frontend-specific controller" |
| `orchestration_contract.py` (123) | contract accessor — "adapters and the kernel use this module" |
| `orchestration_worktree.py` (95) | writer-worktree lifecycle |
| `orchestrator_guard.py` (75) | capability guard derived from the manifest |
| `parse_orchestration_footer.py` (220) | structured worker-report parser |
| `test_orchestration_v2.py` (398) | v2 contract and negative evals |

Evidence it is the live entry point: `orchestrate_strict.py` is referenced by
**18** files. The per-tool adapters already point at it
(`.claude/commands/orchestrate.md`, `.qwen/commands/orchestrate.md`,
`GEMINI.md`), so the C4/C5 pattern is already implemented here.

The two remaining scripts are NOT dead and NOT duplicates:
- `test_orchestration.sh` invokes `scripts/advanced_monitor.sh` and
  `scripts/resource_manager.sh`, both of which exist. It tests the MONITORING
  subsystem, not orchestration.
- `orchestrator_activate.sh` reads `.claude/orchestrator_state.json` (exists)
  and cites `HANDOFF/AGENT_HANDOFF_GUIDANCE.md` (exists). It reports status; it
  activates nothing.

So the conflict was a NAMING COLLISION, not duplication — two unrelated things
sharing a prefix, which is exactly what made the first pass misread them.

- Disposition: **DONE.** Renamed for honest names, nothing deleted:
  `test_orchestration.sh` -> `test_monitoring_stack.sh`
  `orchestrator_activate.sh` -> `orchestrator_status.sh`
  Stale reference in `docs/historical/ORCHESTRATOR_QUICKREF.md` updated.

### C3. Seventeen `verify_*` scripts — INDEXED 2026-08-15, one prune candidate

The concern was duplication. Measured, there is almost none: they cover
different subsystems and only ONE has no reference anywhere in the repo.

Wired into CI (these are load-bearing, do not touch):
  verify_ios_bindings.sh   Swift bindings in sync with the UDL
  verify_versions.sh       read-only release metadata verifier

Referenced but manual (log-analysis and field-test helpers, mostly invoked by
hand during a run; the reference counts are largely HANDOFF/docs mentions):
  verify_receipt_convergence.sh (20)   verify_ble_only_pairing.sh (16)
  verify_simulation.sh (15)            verify_relay_flap_regression.sh (14)
  verify_integration.sh (11)           verify_ws12_matrix.sh (10)
  verify_task_completion.sh (9)        verify_delivery_state_monotonicity.sh (8)
  verify_branch_merges.sh (5)          verify_platform_security.sh (5)
  verify_cross_pair_local.sh (4)       verify_all_builds.sh (3)
  verify_task_systematic.sh (3)        verify_all.ps1 (1)

Zero references anywhere:
  verify_swift_violations.py (0)  -- "compiler-free verification for specific
  SwiftLint rules". Kotlin/Swift linting runs in CI via a different path.

- Disposition: **PLAN, mostly no-op.** The family is not duplicated and needs no
  consolidation. `verify_all.ps1` (1 ref) and `verify_all_builds.sh` (3) overlap
  by NAME only -- the .ps1 is an omni-diagnostic scanner, the .sh is a
  cross-platform build check. Different jobs, misleading names; rename if it
  ever bites.
  One prune candidate: `verify_swift_violations.py`. Per the deletion rule it is
  NOT removed in the round that found it. Next round: confirm no CI job or
  developer invokes it, then remove.

### C4 + C5. Agent contracts and harness config — RESOLVED, operator ruling 2026-08-15

Operator: "unify and rip/replace — we need a cohesive strategy that will work
for any tool."

**Correction to the first pass:** GEMINI.md's relationship was recorded as
"undeclared". That was wrong — it was not read before being classified. It is
505 bytes and already says "Read `AGENTS.md` ... Gemini is an adapter to
`scripts/orchestrate_strict.py`, not an alternate controller." It is the
reference implementation of the pattern below, not a conflict.

Sizes: `AGENTS.md` 9,904 B · `CLAUDE.md` 3,629 B · `GEMINI.md` 505 B.

#### The strategy: one contract, thin adapters, executable-only config dirs

1. **`AGENTS.md` is THE contract.** Model-agnostic, harness-agnostic, one copy.
   Every rule that is true regardless of which tool is running lives here or in
   `docs/rules/`. This already matches the emerging cross-vendor convention, so
   a new tool arrives already knowing where to look.

2. **Each harness gets exactly ONE thin adapter file**, at whatever path that
   tool loads by convention (`CLAUDE.md`, `GEMINI.md`, `.codex/AGENTS.md`, ...).
   Its only job: point at `AGENTS.md`, then state the handful of mechanics
   unique to that harness. `GEMINI.md` at 505 B is the size target. `CLAUDE.md`
   at 3.6 KB is at its cap and must not grow — it is re-paid uncached on every
   subagent spawn.

3. **A harness config directory holds ONLY what that tool's runtime loads
   automatically.** Hooks, commands, skills, settings. Nothing else.

   The test, and it is the whole rule: **"does this tool's runtime read this
   file by itself, without a human naming it?"** If no, it is content, and
   content does not live in a harness directory.

4. **Shared prose moves to `docs/rules/`.** One copy, referenced by every
   adapter. This is already the instinct behind `.claude/rules/*.md` being
   stubs — that directory auto-loads into every spawn, so detail there is paid
   for repeatedly. Generalise that to all harnesses.

5. **Specs and plans move to `docs/` or `HANDOFF/`.** They are project content
   that happens to sit under a vendor directory.

#### What that means per directory (evidence: `git ls-files` by subdir)

| Dir | Contents | Verdict |
|---|---|---|
| `.claude` | scripts 23, skills 11, archive 7, prompts 6, rules 5, hooks 5 | MIXED — keep hooks/skills/commands; `archive/` and `prompts/` are content, `rules/` are already stubs |
| `.kiro` | specs 32 | **CONTENT** — no executable config at all. Move to `docs/specs/`, then the dir goes |
| `.codex` | agents 13, hooks 2, hooks.json | executable — keep, add adapter pointer |
| `.mimocode` | plans 5, config 1, doc 1 | **CONTENT** — plans move to `HANDOFF/`; keep only the config |
| `.bob` | skills 6 | executable — keep, add adapter pointer |
| `.agents` | skills 4, rules 1 | executable — keep; the `rules` file folds into `docs/rules/` |
| `.qwen` | commands 1 | executable — keep |

Nothing here requires knowing which harnesses are "live", which is why this
stopped being a CALLOUT: the rule is about what a runtime loads, not about who
is using it. A dormant harness with a valid adapter costs one small file.

- Disposition: **PLAN.** Sequence: (a) move `.kiro/specs` and `.mimocode/plans`
  out; (b) add adapter pointers for `.codex`/`.bob`/`.agents`/`.qwen`;
  (c) fold `.agents/rules` into `docs/rules/`; (d) only then remove emptied
  directories. Per the deletion rule, no directory is removed in the round that
  empties it.

### C6. Six documents claiming planning authority — RESOLVED 2026-08-15

`SHIP_PLAN.md` declares itself "the **only** execution queue until v0.4.0 is
tagged". Five others also read as authoritative and none deferred to it:
`REMAINING_WORK_TRACKING.md`, `docs/CURRENT_STATE.md`, `HANDOFF/todo/_QUEUE.md`,
`HANDOFF/V1_0_0_EXECUTION_PLAN.md`, `DOCUMENTATION.md`.

- Disposition: **DONE.** A supersession banner added under the H1 of each of
  the five, pointing at SHIP_PLAN.md and stating that authority returns once
  v0.4.0 is tagged. Nothing deleted, nothing rewritten, fully reversible — and
  it removes the ambiguity most likely to misdirect a freshly woken orchestrator
  that opens whichever of the six it happens to find first.

### C7. Duplicate-by-platform pairs — RESOLVED 2026-08-15, not duplicates

- `OllamaQuotaScraper.ps1` (14 refs) / `.sh` (5 refs): a genuine
  Windows/POSIX pair of the same tool. Both referenced, neither in CI. This is
  the correct shape for a cross-platform helper on a Windows host with Git Bash
  — keep both.
- `verify_all.ps1` (1 ref) / `verify_all_builds.sh` (3 refs): NOT a pair. The
  .ps1 is "SCMessenger Omni-Diagnostic Scanner (V4)"; the .sh is a
  cross-platform build verification. Different jobs that collide on the
  `verify_all` prefix — the same naming-collision pattern as C2.

- Disposition: **DONE, no change.** Nothing here is duplicated. Both flagged
  cases were names colliding, not code. Worth noting the pattern: three of the
  seven conflicts in this ticket (C2, C3's verify_all, C7) turned out to be
  prefix collisions rather than duplication. Filename similarity is a very weak
  signal in this repo, and Phase 1 over-weighted it every time.

---

## NEXT ROUND

Status after the 2026-08-15 pass: C2, C3, C4+C5, C6, C7 resolved or planned.
C1 is the only one still blocked, on PR #150 merging.

Remaining, in order:
1. C1 — once PR #150 lands, make `scripts/delegate_task.py` a deprecation
   pointer at `scripts/delegate.py`. Do not delete it in that change.
2. C4+C5 execution — move `.kiro/specs` and `.mimocode/plans` out, add adapter
   pointers for `.codex`/`.bob`/`.agents`/`.qwen`, fold `.agents/rules` into
   `docs/rules/`, and only then remove emptied directories.
3. C3 tail — confirm nothing invokes `verify_swift_violations.py`, then remove.

**Lesson from this pass, for whoever runs the next one:** Phase 1 classified by
filename and was wrong on four of seven items. `GEMINI.md` was already correct,
the orchestration system was already the architecture we planned to build, and
two "duplicate pairs" were unrelated tools sharing a prefix. Open the file before
you classify it — AGENTS.md rule 13. The repo is consistently more coherent than
a directory listing makes it look.
