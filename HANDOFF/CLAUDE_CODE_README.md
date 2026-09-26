# Claude Code — Start Here

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**You are a subagent or Claude Code session picking up SCMessenger work.**

## Role & Protocol

**Read `CLAUDE_CODE_PROTOCOL.md` after the orchestration index.** It defines your role, what you may/may not do, build commands, and the OODA discipline. This is the Overseer role anchor — do not skip.

The full state of this project, the work that's been done today, the work that
remains, and the exact build commands to use are all in:

## → `STATE/2026-06-05_ORCHESTRATION_INDEX.md`

**Read that file first.** It contains:
- What's DONE (do not redo)
- What's NEXT (the delegation queue — pick from this)
- Build environment setup (CRITICAL — without this nothing builds)
- Quota state
- Constraints (no fake PASS, build before commit, move ticket on commit)
- Open questions for the user

The delegation queue is in **`todo/`** (45 tickets). When you complete a ticket,
move it to `done/` with `git mv`.

The research that informs the next phase of work is in **`research/`** —
specifically the 4-phase dynamic-port migration plan.

The full state of the parallel subagent work (A/B/C) from this morning is in
**`STATE/2026-06-05_NEARBY_DISCOVERY_PRODUCTION_PUSH.md`**.

---

## Quick orientation

- **Android app is currently installed and running** on Lucas's Pixel 6a
  (PID 7643, v0.2.2). Do not reinstall unless asked.
- **7 Android tickets** are the next batch (Subagent C ran out of tool budget).
- **2 P0 CLI tickets** (Drift, key collision) block cross-OS triangulation.
- **Build chain is now reproducible from WSL.** See the index for the full env.

---

## When in doubt

- If a build fails, **read the error carefully** before changing code.
- If a ticket file is missing context, **read the related HANDOFF STATE files** for background.
- If you make a change that doesn't verify, **document it honestly** in the commit message and the HANDOFF state file. Do not fabricate PASS reports.
- If you're uncertain about scope, **stay inside the files your ticket owns** and document what you couldn't reach.

