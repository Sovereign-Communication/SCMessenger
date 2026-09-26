<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: implementer
# BUDGET: 1800
# token_budget: 18000

# VALIDATED_P0_ANDROID_024_DISPATCH

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer (Android/Platform/Implementation)
**Budget:** 1800s (EXECUTE tier)
**Phase:** P0 (blocker for new-user onboarding)
**Source:** `HANDOFF/todo/P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md` (82 lines, full spec)
**Depends on:** `HANDOFF/todo/[META]_ORCHESTRATOR_COLD_START_RECOVERY.md` (cold pool must be warm first)
**Blocks:** v0.2.3 hotfix release; user-blocking onboarding bug
**AGENT_MODEL:** qwen3-coder-next:cloud (per `docs/orchestration/ORCHESTRATOR_DIRECTIVE.md` table for `implementer`)

---

## Verified Gap

The P0 spec (`P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md`) is complete and validated. It has file:line targets, hypotheses, repro steps, acceptance criteria, and cross-OS impact notes. **Nothing in this ticket rewrites that work.**

What's missing is a **dispatch ticket**  a contract that tells the orchestrator pool exactly which agent gets the work, in what order, with what gates. The P0 spec is the WHAT; this dispatch ticket is the WHO/HOW.

Without a dispatch ticket, the P0 will sit in `HANDOFF/todo/` indefinitely. The pool is cold; the Overseer has not invoked `orchestrate` for it; no agent is bound.

**Verified environment facts:**
- P0 spec location: `HANDOFF/todo/P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md`
- Sibling validated ticket: `HANDOFF/todo/[VALIDATED]_P0_ANDROID_024_Identity_Generation_Reentrant_Guard.md` (already in `todo/`, do NOT touch)
- Quota: tier must be EXECUTE (50% 7d window) for P0 work. HARDLOCK aborts.
- Agent model: `qwen3-coder-next:cloud` (the `implementer` slot's default per the directive table).

## Scope

4 sub-tasks. This ticket is the dispatch contract; the subagent that picks it up must execute steps 14 in order.

1. **Pre-flight quota check.** Read `.claude/quota_state.json`. Confirm:
   - `timestamp` is within 5 minutes of "now" (re-scrape if stale).
   - `sevenDay`  99.5% (anything above is HARDLOCK  abort this ticket, do not launch, document in `HANDOFF/ORCHESTRATOR_LOG.md` with tag `[HARDLOCK_ABORT]`).
   - `tier` in {HEAVY-LIFT, EXECUTE, MIXED} is preferred for P0 work; LIGHT/MICRO also acceptable.

2. **Pre-flight pool check.** Run `bash .claude/orchestrator_manager.sh pool status`. Confirm:
   - `orchestrator_active: true`
   - Free slots  1. If zero, wait for `pool patrol` to surface a completion, or `pool stop` a stale agent.

3. **Dispatch the P0.** Run:
   ```bash
   bash .claude/orchestrator_manager.sh pool launch implementer HANDOFF/todo/P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md
   ```
   The launch returns an `agent_id`. Note it. Tail `.claude/agents/<agent_id>/agent.log` and wait for one of:
   - `BUILD FAIL`  surface to Overseer, do NOT move the P0 to `done/`.
   - `COMPLETE` marker  proceed to step 4.
   - No movement for 30 min  run `pool patrol`, then `pool stop` if hung.

4. **Verify and close out.** On `COMPLETE`:
   ```bash
   cargo check --workspace
   ./gradlew :app:assembleDebug -x lint --quiet
   ```
   Both must pass. On success:
   ```bash
   git mv HANDOFF/todo/P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md HANDOFF/done/
   git mv HANDOFF/todo/[VALIDATED]_P0_ANDROID_024_DISPATCH.md HANDOFF/done/   # this ticket
   ```
   Write a brief post-mortem to `HANDOFF/STATE/2026-06-05_P0_ANDROID_024_RESOLVED.md` with: root cause, files changed, build hashes, `agent_id` from step 3.

## File Targets

- `HANDOFF/todo/P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md` [REFERENCE, NOT MODIFIED]  the detailed spec
- `HANDOFF/todo/[VALIDATED]_P0_ANDROID_024_Identity_Generation_Reentrant_Guard.md` [DO NOT TOUCH]  sibling ticket, separate work
- `.claude/quota_state.json` [READ]  pre-flight gate
- `.claude/orchestrator_state.json` [READ]  pool gate
- `.claude/agents/<agent_id>/agent.log` [TAIL]  live progress
- `HANDOFF/STATE/2026-06-05_P0_ANDROID_024_RESOLVED.md` [CREATE on success]  post-mortem

## Build Verification Commands

Run on completion (step 4):

```bash
export CARGO_INCREMENTAL=0
cargo check --workspace
cd /home/scmessenger/scmessenger-build/android
./gradlew :app:assembleDebug -x lint --quiet
```

Both must exit 0. APK target ~291MB. If either fails, surface to Overseer  do NOT move the P0 to `done/`.

## Acceptance Gates

1. Pre-flight quota check passed (tier  HARDLOCK equivalent; not HARDLOCK).
2. Pre-flight pool check passed (free slot available).
3. `pool launch implementer` returned a valid `agent_id` and the agent started (log line within 60s).
4. Agent reached `COMPLETE` (or surfaced `BUILD FAIL` for Overseer triage).
5. `cargo check --workspace` passes.
6. `./gradlew :app:assembleDebug -x lint --quiet` passes.
7. P0 spec file moved to `HANDOFF/done/` via `git mv`.
8. This dispatch ticket moved to `HANDOFF/done/` via `git mv`.
9. Post-mortem written to `HANDOFF/STATE/2026-06-05_P0_ANDROID_024_RESOLVED.md`.

## CRITICAL

You are forbidden from considering this task 'complete' until you:
1. Run `git mv` on BOTH the P0 spec file AND this dispatch ticket.
2. Write the post-mortem to `HANDOFF/STATE/`.

Do NOT rewrite the P0 spec. Do NOT touch the sibling `Identity_Generation_Reentrant_Guard` ticket. Do NOT bypass the pre-flight checks.

## Routing

`[REQUIRES: IMPLEMENTER] [PHASE: P0] [TIER: 2-3] [DEPENDS_ON: [META]_ORCHESTRATOR_COLD_START_RECOVERY] [AGENT_MODEL: qwen3-coder-next:cloud]`
