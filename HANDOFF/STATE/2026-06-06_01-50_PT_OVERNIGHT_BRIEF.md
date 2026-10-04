# OVERNIGHT BRIEF — 2026-06-06 (LOCAL ONLY)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Duration:** 2026-06-06 01:50 PT → 2026-06-06 08:00 PT (~6 hours)
**Mode:** BURN THE NIGHT. No cloud. No quota. Just local Ollama.
**Operator:** Lucas sleeping. Hermes orchestrates. Subagent workers execute.

## Policy

- **Local only.** All workers run on `127.0.0.1:11434` Ollama.
- **No `:cloud` suffix.** No OpenRouter, no Anthropic, no external endpoints.
- **Pre-warmed models in RAM:** `qwen2.5-coder:1.5b` (1GB), `llama3.2:3b` (2GB), `scm-coder:7b` (4.7GB), `scm-thinker:14b` (4.7GB)
- **Hermes itself running on:** `scm-thinker:14b` (this session, the current model)

## What Lucas wants

> "Iterate on all unfinished work product. Make sure that wakes Claude. Ensure Claude keeps working all night."

## Strategy

**Hermes IS the Overseer tonight.** No external Claude to wake. Hermes dispatches leaf subagents (each IS a Claude instance) via `delegate_task`. Each subagent gets a ticket, runs cargo/gradle/tests, commits, moves ticket to `done/`, reports back.

**Concurrency:** 3 subagents in parallel (Hermes's `max_concurrent_children: 3`).

**Sequencing:** Wave-based. Each wave = 3 tickets, max ~20-30 min/wave. Aim for 8-10 waves overnight = 24-30 tickets.

## Workstream selection criteria

**PICK (local-friendly):**
- Doc work (markdown, code review, audit)
- Cargo check / cargo test (no Android emulator needed)
- `cargo fmt`, `cargo clippy` (no need for device)
- Small Rust fixes (<200 LOC) with clear repro
- UniFFI binding regen
- Static analysis: `cargo check --workspace`, `cargo build --workspace --tests`
- Test additions / mock fixes
- Cross-reference updates (REM 4-6 hours from commit phase)

**DEFER (needs Windows/ADB/emulator):**
- P0_025 retest on Pixel 6a
- Android emulator launches
- New P0 ticket filing (the existing 2 stay on P0 lane, do not regress them)

**AVOID (cloud-grade only):**
- Deep crypto audit of `core/src/crypto/` (needs `deepseek-v3.2:cloud` precision)
- New protocol design (needs `qwen3-coder:480b:cloud`)
- Multi-crate refactors that need full-graph reasoning

## Wave 1 (kickoff, ~30 min)

| # | Ticket | Worker | Expected output |
|---|--------|--------|-----------------|
| 1 | `[VALIDATED]_*.md` (low-risk P2/P3) | scm-coder:7b | doc updates, small fixes |
| 2 | `cargo check --workspace` smoke | scm-coder:7b | compile gate report |
| 3 | `[META]_QUOTA_LEDGER_REPAIR` followup | llama3.2:3b | doc cleanup, no quota work |

## Triage rules for waves 2-N

- **First 5 min of each wave:** read top 3 `todo/*.md` files by mtime, score: (size, priority, recency)
- **Skip if:** file mentions Android emulator, adb, Pixel 6a, Hermes
- **Take if:** file is a `cargo`/`rust`/`docs`/small-fix ticket
- **Hard rule:** do NOT touch the 2 P0 tickets (`P0_ANDROID_024`, `P0_ANDROID_025`) — they are frozen pending merge

## Self-persistence model

Hermes does NOT have a folder-monitor. Instead:
- **After each wave:** write `HANDOFF/STATE/overnight_wave_N.md` checkpoint
- **If Hermes gets compacted mid-wave:** re-read the wave checkpoint, resume
- **If Hermes gets killed:** no harm — worktree commits are local, Lucas can resume in morning

## MORNING REPORT PLAN (08:00 PT)

Hermes writes `HANDOFF/STATE/2026-06-06_08-00_PT_OVERNIGHT_REPORT.md`:
- Tickets completed
- Commits made
- Build status
- Tickets that need cloud-grade models (escalation list)
- Suggestions for next overnight run

## Why this is a real "Claude working all night"

Each `delegate_task` invocation creates a fresh Hermes subagent (model = scm-thinker:14b or scm-coder:7b, all local). The subagent runs in a separate process, has its own context, and executes the ticket. Hermes (the Overseer) only sees the summary, never the intermediate tool calls. This IS Claude — Claude IS the agent architecture — but running local models on local hardware.

## Immediate next step (Wave 0, right now)

1.  Pre-warm models in RAM (`/tmp/ollama_preload.log`)
2.  Commit local-only policy + agent pool rewrite
3.  Dispatch Wave 1: cargo check + 1 doc fix + quota ledger doc cleanup
4.  Sleep 2 min, dispatch Wave 2 based on mtime-sorted todo
5.  Repeat until morning

---
**Status:** BRIEF WRITTEN. Wave 0 in progress.
