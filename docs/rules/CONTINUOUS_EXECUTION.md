# Continuous execution -- never idle on an absent node

Status: Active
Created: 2026-08-31
Authority: Operator directive, 2026-08-31. Supersedes `SHIP_PLAN.md` section 4's
blanket deferral of v1.0.0 scope **for Tier A work only** -- the tag still sets
priority order, but the queue may no longer run dry while waiting on hardware.
Tier: 1 (loaded on demand)

> "Ensure that if the Android node is unavailable, we continue working... that
> any connected nodes are always iterated on fully to v1.0.0, and that any
> non-connected nodes are still being coded for up to parity level. That any
> discovered previous existing issues/errors are owned, accounted for, and added
> to planning accordingly." -- Operator, 2026-08-31

## 1. Why this exists

Two failures on 2026-08-31 made the case:

- The v0.4.0 plan sequenced D4/D6/D7 behind "operator + hardware," so every
  agent-doable item finished and the queue stalled waiting on a phone.
- The driver watcher died at 01:31 with `ERROR: bash not found` and stayed dead
  for **seven hours**. Nothing noticed, because nothing was watching the
  watcher. Continuous execution that fails silently is worse than none: it
  produces confident "no anomalies" reports from a process that is not running.

`HANDOFF/V1_0_0_EXECUTION_PLAN.md` compounds this. Its Phase 1 is titled
"Android <-> Windows full transport cooperation," so the plan is organised by
*feature* and is therefore device-gated by construction. This document
reorganises execution by **availability** instead.

## 2. Node availability tiers

| Tier | Nodes | Availability | Obligation |
|---|---|---|---|
| **A** | AWS (Amazon Linux, Docker) + Windows CLI | Continuous. Both are ours; if one is down, restoring it is the top task | Driven to **full v1.0.0 conformance**, continuously, with evidence |
| **B** | Android (Pixel 6a) | Intermittent -- operator-carried | **Coded to parity now, verified later.** Device time is for verification and log capture, never for writing code |
| **C** | iOS / macOS | Absent; v0.5.0 scope per the 2026-08-29 ruling | Not coded against. Do not start |

Tier A is not "the nodes we test on." It is the standing obligation: those two
must be at v1.0.0 quality for every capability that does not require a handset.

## 3. The never-idle ladder

When you finish a task, or are blocked, descend this ladder and take the first
item that is actionable **right now**. Never report "blocked on hardware" as a
terminal state -- it is a signal to move down the ladder, not to stop.

1. **Restore Tier A.** If either always-on node is down, degraded, or running a
   SHA older than `main`, fix that first. A Tier A outage invalidates every
   result gathered during it.
2. **v0.4.0 gate items** that do not need a handset (the `HANDOFF/freebuff/`
   queue).
3. **Tier A v1.0.0 conformance** -- the next unmet row of the two-node
   conformance matrix (section 4).
4. **Tier B parity coding** -- implement the Android side of a v1.0.0 item and
   stage its verification (section 5). The code lands and CI-tests without the
   device.
5. **Owned-issue burn-down** -- the ledger in `SHIP_PLAN.md` section 7.
6. **PR queue and backlog truth** -- `SHIP_PLAN.md` G5.

If all six are genuinely empty, say so explicitly with evidence. That is a real
state, and it has never yet occurred in this project.

## 4. The Tier A bar: what "100% perfection" means

A capability is Tier A conformant when **all** of these hold:

1. It works between the AWS node and the Windows node, on current `main`, with
   both reporting the same git SHA.
2. Scored receiver-side: decrypt, durable history, and receipt. Never transport
   ACKs, never UI counters, never local acceptance.
3. It survives a **restart of either node** with identity and history intact.
4. It survives an **address change** on the AWS node -- redeploy takes a new
   public IP, nothing is reconfigured by hand, and the mesh re-forms. This is
   the operator's 2026-08-31 churn requirement and it is a conformance row, not
   a stretch goal.
5. It has an automated check that fails loudly, wired into the continuous rig
   (section 6).

A row that passes 1-3 but not 4-5 is **not** conformant. Record it as partial
with the specific gap named.

## 5. Tier B parity protocol -- code now, verify later

Device time is the scarcest resource in this project. It must never be spent
writing code. Every Android item is "parity ready" only when all four exist
before the handset appears:

1. **The code is merged**, behind CI (unit tests, the wiring gate, lint).
2. **A verification script** that runs on the device with no authoring -- exact
   `adb` commands, expected output, pass/fail stated in advance.
3. **A log-capture recipe**: the exact `RUST_LOG` line, the logcat filter, the
   capture window, and where the artifact is written. If the test fails, the
   debugging evidence must already be in hand -- a failed device session that
   produces no logs has wasted the window entirely.
4. **A pre-registered failure disposition**: what happens if it fails. Which
   ticket it becomes, and whether it blocks the tag.

Android agents are authorised for app updates and passive log collection only;
active device/mesh driving belongs to the Windows, aidws, and Ubuntu agents
(`docs/rules/ANDROID.md`). Write the scripts accordingly.

## 6. The continuous rig, and watching the watcher

`scratch/driver/` holds an event-driven watcher: a `FileSystemWatcher` over the
node log directory, zero CPU while idle, which wakes and runs `driver.sh` on any
node activity.

**It must be verified alive, not assumed alive.** Its 2026-08-31 outage lasted
seven hours precisely because liveness was assumed. Two rules:

- The watcher logs `using bash: <path>` and `starting; watching <dir>` on every
  start. Absence of a recent line in `watcher.log` means it is dead, regardless
  of what any earlier report claimed.
- Any session taking the orchestrator seat checks `watcher.log`'s last
  timestamp as part of orientation. A stale watcher is a Tier A outage under
  ladder step 1.

Silence from a monitor is not evidence of health. It is evidence of silence.

## 7. Owning discovered issues

When you find a defect that is not on the plan -- including one that predates
your change and is not your fault -- you own getting it recorded. It goes in the
ledger at `SHIP_PLAN.md` section 7 with:

- what it is, and the command whose output proves it,
- whether it is fixed, ticketed, or accepted, and
- if ticketed, the task file; if accepted, why that is safe.

"Noted in passing" is not a disposition. An issue mentioned in a report and
absent from the ledger is an issue that will be rediscovered by the next session
at full cost -- this has already happened repeatedly here, which is why
`SHIP_PLAN.md` section 6.3 exists to list claims the repo makes that are false.

Not every issue needs fixing now. Accepting one deliberately, in writing, with a
reason, is a valid and often correct disposition. Leaving it unrecorded is not.
