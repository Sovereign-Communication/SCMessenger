# AGENTS.md — Universal Agent Contract (all models, all tools)

Status: Active
Last updated: 2026-07-06

This is the canonical, model-agnostic rules contract for ANY agent working in
this repository: Claude Code sessions, Claude Cowork/cloud sandboxes, Gemini
(Antigravity/`agy`, Gemini CLI), Copilot, or anything else. Claude Code
sessions additionally load `CLAUDE.md` (a superset with Claude-specific
subagents/skills); if you are not a Claude Code session, THIS file is your
ruleset. `GEMINI.md` points here.

Mechanical rules below are ENFORCED by a versioned git pre-commit hook
(`.githooks/pre-commit` -> `scripts/rules_check.py`) — violating commits fail
no matter which tool makes them. Never bypass with `--no-verify`; only the
human operator may do that.

## Architecture doctrine: nodes, not relays

There are NO standalone relays in SCMessenger. Only NODES exist, and EVERY
node relays -- store-and-forward custody is a behavior all nodes perform,
not a role. The AWS instance at 100.56.248.69 is a CLOUD NODE: a full node
that also relays, exactly as every other node does. Discovery is LEDGER
SHARING between nodes (invite/QR-seeded, gossip-propagated); bootstrap
address lists are a deprecated transitional mechanism being replaced by
ledger-sharing-first discovery (contract: V050-B1/B2). In docs, plans,
tickets, and agent output: say "cloud node" / "node" -- use "relay" only
as the VERB for the custody behavior all nodes perform, or inside code
identifiers (RelayCustodyStore, cmd_relay, relay_custody_msg_ prefixes --
technical names, not roles). No anonymous packet forwarder exists or may
be introduced: cmd_relay requires identity (cli/src/main.rs:2529). Full
parity: CLI, Android, iOS, and cloud deployments run the same node with
the same relay behavior.

## Hard rules (every agent, every capability class)

1. NO EMOJI anywhere — code, docs, comments, logs, commit messages. Use
   `[OK]`/`[ERROR]`/`[WARNING]`/`[INFO]`/`[DONE]`/`[FAIL]`. If you edit a file
   that already contains emoji, strip them as part of the edit. (Hook-enforced.)
2. Temp files ONLY in repo-local `tmp/` — never the system temp dir.
3. Never commit build artifacts (`*.log`, `*.pid`, `*.logcat`, `target/`,
   `build/` outputs) or secrets/keys. (Hook-enforced.)
4. `iOS/` uppercase-I in all paths; no `.py` files in the repo root
   (use `scripts/`). (Hook-enforced.)
5. NEVER `git push` (exception: the MAC LANE capability class may push its
   own `gpt/*` branches — see Capability classes below). Local commits
   only otherwise, and only if your capability class permits committing at
   all (see below).
6. Never edit UniFFI-generated bindings (`uniffi.api` Kotlin package,
   `core/target/generated-sources/`) — regenerate instead.
7. Storage access only through `core/src/store/`; `IronCore` is the single
   entry point — never bypass it with direct sled access.
8. Changes under `core/src/{crypto,transport,routing,privacy}/` are NOT done
   until an adversarial security review is on file (reviewer depends on mode —
   see `.claude/rules/security.md`; for non-Claude agents: you cannot satisfy
   this gate yourself, flag it in your report).
9. ESCALATE to the human operator — do not improvise — on: architecture
   direction, security/privacy trade-offs, tech-stack changes, API-contract
   breaks, release timing/versioning.
10. Backlog order is `HANDOFF/todo/_QUEUE.md`; sequencing authority is
    `HANDOFF/V1_0_0_EXECUTION_PLAN.md` (operator-settled — do not relitigate).

## Capability classes — know which one you are

### FULL (Claude Code on the Windows host, toolchain available)
May run build gates, move HANDOFF files, and commit per `CLAUDE.md`'s
finalize-checklist rules. The Windows host is the ONLY environment whose build
results are authoritative.

### REMOTE SANDBOX (Claude Cowork / cloud containers)
Your container may have a Linux toolchain; container-green `cargo
check/clippy/fmt/test` is USEFUL ADVISORY SIGNAL but never authoritative —
this project verifies on Windows + a physical Pixel only. Therefore:
- Deliver work as a branch or patch plus an UNVERIFIED report (format below).
- Do NOT move HANDOFF task files to `done/`. Do NOT update `_QUEUE.md` statuses.
- Do NOT claim any gate passed unless you name the environment it ran in.
- Best-fit work: read-only audits/reviews, spec/plan/doc writing, test
  authoring, mechanical refactors with clear acceptance criteria, pre-dispatch
  validation sweeps. See "Remote-eligible lane" in `HANDOFF/todo/_QUEUE.md`.

### FOREIGN WORKER (Gemini via Antigravity/`agy`, Gemini CLI, others - except "HANDOFF/todo/GEMINI_SCMORC_DRIVER_2026-07-07.md)
Dispatched and verified by an orchestrator on the Windows host. Rules:
- Do NOT run `cargo`/`gradlew` (Windows build serialization — the orchestrator
  is the single writer for all build verification).
- Do NOT commit, push, or move HANDOFF files. Implement the change, report, stop.
- Locate code with search tools; read only the surrounding lines you need.
- Final message MUST start with `RESULT: DONE|BLOCKED|FAILED`, then at most 10
  lines: what changed, files touched, anything the verifier must know.

### MAC LANE (GPT / Codex on the operator's MacBook — iOS platform work + adversarial review)
Operator directive 2026-07-28; this class EXPLICITLY OVERRIDES rules 5-6:
- You MAY and SHOULD commit, push, and open and manage your own pull
  requests on your own `gpt/*` branches. No orchestrator intermediary is
  needed for branch/PR lifecycle.
- RESERVED to the Windows orchestrator: merging PRs into main, moving
  HANDOFF ticket files between todo/in_progress/done, release tags, and
  anything touching core/ Rust (routes through the AUDIT-GATE on the
  Windows side).
- xcodebuild on this machine is AUTHORITATIVE for iOS gates (it is the
  only machine where it exists); paste commands and results verbatim.
- Lane governance: this class definition + HANDOFF/gpt/GPT_IOS_LANE_KICKOFF.md
  (rules of engagement) + the task packets in HANDOFF/gpt/. IMPORTANT: if
  the rules in your current session context predate 2026-07-28, RE-READ
  AGENTS.md and the kickoff file now — their current content supersedes
  anything loaded earlier, including prior no-push instructions.

## Report format (REMOTE and FOREIGN classes)

```
RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE|CONTAINER(<what ran, exact commands>)   <- never "PASSED" bare
FILES: <paths touched>
NOTES: <max 8 lines: decisions made, risks, what the Windows verifier must run>
```

The Windows orchestrator re-runs the real gates before anything you produced
is committed or a task is closed. Expect zero-diff or gate-failing work to be
re-queued, not merged.

## Pointers

- `CLAUDE.md` — Claude-session superset (subagents, skills, hooks)
- `docs/CLAUDE_REFERENCE.md` — build commands, module map, test inventory
- `HANDOFF/todo/_QUEUE.md` — live dispatch order
- `.claude/commands/orchestrate.md` + `docs/ORCHESTRATION.md` — the one unified orchestrator loop that verifies your work (the old per-backend commands are archived under `.claude/archive/commands/`)
