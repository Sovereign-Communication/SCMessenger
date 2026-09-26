# Gemini 3.5 Flash Orchestrator — Quick Start

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Last updated: 2026-07-10
Purpose: Drive Phase 1 backlog forward during native Claude subscription HARDLOCK window (2026-07-08 to 2026-07-10 reset)

[IMPORTANT]
For detailed configuration, model fallbacks, Windows process slot calculations, and host verification procedures, please refer to the main runbook: [GEMINI_ORCHESTRATOR_RUNBOOK.md](file:///c:/Users/SCM/Documents/GitHub/SCMessenger/docs/GEMINI_ORCHESTRATOR_RUNBOOK.md).

## What This Does

You now have a third orchestration mode: Gemini 3.5 Flash coordinates work, Qwen (DashScope) + ollama-cloud workers implement, Windows host verifies.

- **Zero native token burn** — runs entirely on foreign free tiers (Gemini 3.5 Flash, DashScope Qwen free, ollama-cloud free).
- **Full backlog coverage** — can dispatch tasks from `HANDOFF/todo/_QUEUE.md` in parallel (up to 3 concurrent).
- **Windows-verified** — all code changes verified locally before commit (build gates, no zero-diffs).
- **Staged fable audit** — tasks touching crypto/transport/routing/privacy are flagged for mandatory native Fable review post-reset (2026-07-10).

## Prerequisites

### 1. Qwen API Key (DashScope)

If you have DashScope access:

```bash
mkdir -p ~/.config/scmorc
cat > ~/.config/scmorc/dashscope.env <<'EOF'
DASHSCOPE_API_KEY=sk-... # your key here
EOF
chmod 600 ~/.config/scmorc/dashscope.env
```

Without this, mechanical tasks can still route to ollama-cloud alternatives, but Qwen dispatch will fail. If you don't have a key, Qwen tasks will be re-routed to `glm-5.1:cloud` on ollama automatically.

### 2. Verify ollama-cloud is online

```bash
curl -s https://ollama.com/api/tags | grep qwen3-coder
```

Expected: `qwen3-coder:480b:cloud` is listed. If not, the free tier may have hit capacity. In that case, agy/Gemini fallback is available for work that doesn't need the full heavy-lift models.

## How to Invoke

### Option 1: Use the `/gemini-orchestrator` skill (if registered)

```
/gemini-orchestrator
```

This launches Gemini 3.5 Flash as the orchestrator, reads the backlog, and begins dispatching.

Optional arguments:
- `/gemini-orchestrator HANDOFF/todo/P1_ANDROID_mDNS_Self_Loopback_Discovery.md` — claim a specific task
- `/gemini-orchestrator rust` — filter to Rust-domain tasks only
- `/gemini-orchestrator dry-run` — validate and prepare prompts without dispatching

### Option 2: Manual bash invocation

```bash
cd /path/to/SCMessenger
bash .claude/scripts/gemini-orchestrator-launcher.sh
```

Same optional arguments as above.

### Option 3: As a recurring loop

```
/loop 5m /gemini-orchestrator
```

Runs the orchestrator every 5 minutes, picking the next task from the queue each time. Stop with `Ctrl+C` or via `/loop-stop`.

## Workflow

### Orchestrator Launch

1. **Reads `HANDOFF/todo/_QUEUE.md`** — picks the next non-[DEVICE] task.
2. **Pre-dispatch validation** — verifies the task target exists, isn't scaffolding, and isn't already-wired.
3. **Generates worker prompt** — writes to `tmp/gemini-orchestrator/prompts/<slug>.prompt.md`.
4. **Dispatches to Qwen or ollama** — routes based on task type (see `.claude/commands/gemini-orchestrator.md` routing table).
5. **Logs dispatch** — records in `tmp/gemini-orchestrator/dispatch_log.md`.
6. **Returns** — you then manually verify the worker response (see below).

### Verification (Windows Host)

Once a worker responds:

1. **Parse response** — check `RESULT:` line (should be `DONE`, `BLOCKED`, or `FAILED`).
2. **Review files changed** — `git diff --stat` to confirm non-zero diff.
3. **Run build gate** — `cargo build --workspace` (Rust) or `./gradlew assembleDebug` (Android), etc.
4. **Commit if gate passes** — `git add -A && git commit -m "swarm: completed [Task]"`.
5. **Stage for fable audit** (if crypto/transport/routing touched) — mark with `[await-fable-audit]` in `tmp/gemini-orchestrator/fable_audit_queue.md` for post-reset review.

### Example Verification Session

```bash
# 1. Check what worker changed
git diff --stat

# 2. Run gate (example: Rust task)
cargo build --workspace

# 3. If gate passes, commit
git add -A
git commit -m "swarm: completed P1_ANDROID_mDNS_Self_Loopback_Discovery"

# 4. Move task to done/ (manual file move or via orchestrator)
mv HANDOFF/todo/P1_ANDROID_mDNS_Self_Loopback_Discovery.md HANDOFF/done/

# 5. Update dispatch log
echo "| $(date -u +%Y-%m-%dT%H:%M:%SZ) | qwen-turbo | HANDOFF/todo/... | done |" >> tmp/gemini-orchestrator/dispatch_log.md
```

## Task Routing Summary

| Task Type | Primary Model | Why |
|---|---|---|
| Mechanical (strings, docs, TODO) | qwen-plus | Cheapest, adequate |
| Standard Rust/Kotlin | qwen-turbo | Good balance |
| Hard multi-file refactor | glm-5.1:cloud | Deep reasoning |
| Test authoring | qwen-plus | Low-risk, scoped |
| Non-crypto security review | deepseek-v3.2:cloud | Good for adversarial |
| **Crypto/transport review** | **FABLE (native, post-reset)** | **Mandatory — not handled by foreign workers** |

## Quota & Limits

- **Qwen free tier**: ~1M tokens/day. Resets daily. Standard tasks = 5-20k tokens.
- **ollama-cloud free**: Model-dependent, but generally ~2-3 tasks/hour. Can queue more via prompt.
- **Gemini 3.5 Flash free**: Orchestrator coordination, very low token spend (<100 tokens/dispatch decision).

If a worker hits rate limits, it will report `BLOCKED`. Re-dispatch to a fallback model.

## Mandatory Staging: Fable Audit Queue

Tasks touching `core/src/crypto/`, `core/src/transport/`, `core/src/routing/`, or `core/src/privacy/` will be staged as "awaiting fable audit" after Windows gate passes. These are moved to `tmp/gemini-orchestrator/fable_audit_queue.md`:

```markdown
## Tasks Awaiting Fable Audit (reset 2026-07-10)

- Task: P1_CLI_BLE_Outbound_TX_Path_Missing
  Files: core/src/transport/ble.rs, core/tests/integration_ble.rs
  Gate: cargo check --workspace passed
  Status: Fable audit pending
```

After native quota resets, the native orchestrator (scmorc or human) will invoke fable to audit these before they become mergeable. Until then, they are locally committed but not shipped.

## Troubleshooting

### Qwen dispatch fails

**Symptom**: "Qwen dispatch failed" in output.

**Check**:
1. `~/.config/scmorc/dashscope.env` exists and has valid key.
2. Key is not revoked/expired in your DashScope account.
3. Fallback: Re-route to `glm-5.1:cloud` (ollama) for next dispatch.

### Worker response is empty or malformed

**Symptom**: `RESULT:` line is missing or garbled.

**Action**:
1. Check `tmp/gemini-orchestrator/responses/<slug>.response.md` for full output.
2. If model crashed/timed out, re-dispatch to same model (it may be a transient issue).
3. If still failing, escalate to next model in routing table.

### Gate fails after worker changes

**Symptom**: `cargo build --workspace` or `./gradlew assembleDebug` fails with compiler/lint errors.

**Action**:
1. Review the error lines.
2. Feed back to the worker via a follow-up prompt: "Gate failed: [error]. Fix and re-verify."
3. If worker fails to fix twice, ask the operator (may be a design issue, not a typo).

### Task marked done but was already wired / is scaffolding

**Symptom**: Pre-dispatch validation moved a task to `HANDOFF/done/` unexpectedly.

**Check**: Review the validation log in the orchestrator output. If it's correct (target is indeed test code or already has callers), no action needed — task correctly classified. If incorrect, move task back to `todo/` and re-review the target.

## Next Steps

1. **Start small**: Run `/gemini-orchestrator dry-run` to validate the first 1-2 tasks.
2. **Pick a domain**: Filter to one domain (`rust` or `android`) for the first real dispatch to build confidence.
3. **Monitor quotas**: Track worker API status (ollama.com/api/tags, Qwen dashboard) to know when free tiers hit limits.
4. **Prepare for reset**: As 2026-07-10 approaches, have the Fable audit queue ready to hand off to the native orchestrator.

## Contacts & Escalation

- **Task ambiguity**: Ask the operator; move task back to `todo/` with a note.
- **Worker failure (2+ attempts)**: Ask the operator; may need design clarification.
- **Quota exhaustion (Qwen or ollama)**: Note in dispatch log and pivot to remaining tasks; operator may need to unlock paid tier.
- **Architecture / security decision**: Escalate to operator per AGENTS.md rule 9.

## Files

- **Command spec**: `.claude/commands/gemini-orchestrator.md`
- **Launcher script**: `.claude/scripts/gemini-orchestrator-launcher.sh`
- **Dispatch log**: `tmp/gemini-orchestrator/dispatch_log.md` (auto-created)
- **Fable audit queue**: `tmp/gemini-orchestrator/fable_audit_queue.md` (created as tasks are staged)
- **Backlog**: `HANDOFF/todo/_QUEUE.md` (source of truth)

## Outcomes

By reset (2026-07-10):
- **Target**: 5–10 Phase 1 tasks completed (mechanical + standard implementation tier).
- **Expected volume**: ~50-80 committed changes, zero conflicts with native orchestrator's post-reset work.
- **Fable audit queue**: 1-2 tasks staged and ready for mandatory security review.
- **Backlog state**: Reduced todo/ count, Phase 1 progress visible in `REMAINING_WORK_TRACKING.md`.

---

Created by: Gemini 3.5 Flash Orchestrator setup (2026-07-08)
Approved by: (operator signature pending)
