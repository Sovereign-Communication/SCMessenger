# Delegation & Worker Dispatch SOP

Status: Active
Last updated: 2026-08-14 (dynamic local resource admission added; previously this knowledge
existed only in a single agent's private memory store, which delegated workers
cannot read -- which is why these failures kept recurring)

Loaded on demand. Every rule below traces to a measured failure, not a
precaution.

## Lane priority (economics, measured)

1. **Qwen via Claude Code CLI** (`launch_claude.ps1`, Alibaba MaaS) -- PRIMARY.
   Free and shell-capable, so it can run its own verification.
2. **OpenRouter** (`claude --settings ~/.claude/settings.local.OR.json`, or
   `scripts/delegate_task.py`) -- secondary.
3. **DashScope / Qwen direct** -- OpenAI-compatible, ~1M tokens per Qwen model.
   Key at `~/.config/scmorc/dashscope.env`; helper `tmp/scmorc/qwen.sh`.
4. **Groq** -- LPU, fast. `delegate_task.py --provider groq`, key at
   `~/.config/scmorc/groq.env`. Needs a curl User-Agent or Cloudflare returns
   error 1010. Tight TPM, so micro/validation tasks only.
5. **ollama / agy** -- MICRO tasks only. A single trivial ollama-claude call
   once consumed 5.7% of a 5-hour Anthropic window.

Quota ledger: `docs/QWEN_QUOTA_LEDGER.md` is canonical -- update it there.

## When to delegate at all

Delegate scoped, parallel, or specialized units of work. Do NOT delegate
one-liners: dispatch overhead exceeds the task. A bounded 1-5 tool-call
diagnostic is cheaper done inline. Audits and micro-tasks route to free lanes;
reserve Claude-native capacity for verdicts and judgment calls.

Do not run Fable-tier subagent or workflow fan-outs. Haiku/Sonnet workers only,
free lanes first.

## Dynamic local resource admission

Local direct workers and build lanes use task-sized host-memory reservations, not
a fixed per-worker RSS ceiling. Before launch, estimate the worker plus all
descendants, add a 10% default margin, and reserve through
`scripts/resource_admission.py` in the shared `tmp/lakes/active_workers.json`
registry. Admit only when fresh host telemetry shows that the reservation fits
alongside every active reservation, the 2 GiB headroom floor, and the global
worker budget. Bind and sample the full process tree, then release only after
cleanup. An explicit authenticated human or terminal-operator directive may
approve an exception worker for any stated purpose, but it cannot bypass
availability, headroom, monitoring, serialized builds, or safety gates. Remote
API-lake calls remain quota-tracked separately unless they launch local
processes. Unknown telemetry is a blocked admission, never a guess.
`scripts/resource_manager.sh --admission` exposes the same snapshot; its legacy
CPU/percentage checks are advisory and cannot bypass this gate.

## agy

- **Always pass `--add-dir`.** Without it, agy re-discovers the repo path on
  every dispatch and frequently bails before finishing. This is the root cause
  of what looks like random timeouts.
- **Always pin `--model` explicitly, with the exact quoted name.** Shorthand
  silently substitutes a different model with no error, and agy can route to
  claude-sonnet/opus and quietly spend Anthropic quota. `agy models` lists both
  pools -- Claude and Gemini are separate quotas.
- Binary lives at `AppData\Local\agy\bin\agy.exe`. The User-PATH entry vanishes
  recurrently; invoke by full path. A stale process environment is not the same
  as real PATH drift -- check before "fixing" it.
- The 5-minute default print-timeout is too short for real debugging. On
  timeout use `--continue`; do NOT re-dispatch fresh, which discards the work
  already paid for.
- agy needs `--dangerously-skip-permissions` for unattended runs.

## scripts/delegate_task.py

- **Full-file mode is unsafe past roughly 300-500 lines** -- models silently
  truncate. Compare `wc -l` against the response length; use `--mode diff`
  beyond that.
- **Flash-tier Qwen prefers diff format** for small edits. Pass `--mode diff`
  explicitly for scoped fixes or you get a vacuous success (exit 3).
- **Parallel output collision:** sending the same `--task` file to two providers
  writes the same `tmp/` output path, and one silently clobbers the other. Use
  distinct task files or distinct output paths.
- `max_tokens` resolves to the model maximum. State the wanted output length in
  the prompt itself -- the ceiling is not a brevity constraint.

## Qwen (cloud, direct API)

Needs tight scoping plus full file context. Loosely-scoped analysis tasks make
it rewrite code instead of analyzing it. Use `fusion_lite` for plans and Qwen
for scoped diff edits.

## Fusion Lite

- Model slugs must be vendor-prefixed. Tier-B panel names copied out of handoff
  docs are not valid OpenRouter slugs.
- Spend policy: 2 cents normal, 10 cents maximum hard cap. No per-run approval
  needed inside those caps. A full 3-model panel runs about 2 cents.
- **A truncated Gemini response is not a real defect.** Gemini burns budget on
  hidden reasoning tokens. Fix with `reasoning.effort=low`, not just a larger
  `max_tokens`. A cheap "continue" call beats a full re-run.

## Dispatch context hygiene

Run `claude -p` from a scratch cwd with `--add-dir` pointed at the repo, so
`CLAUDE.md` and project rules are not preloaded into the worker. Then inject
only the rules that task actually needs. Every worker spawn otherwise re-pays
the full project instruction set.

## Verification discipline

**Verify delegated verification claims.** A worker once produced a complete,
plausible AWS health report for a node that was actually down. A delegated
"I verified it" is a claim, not evidence -- require the command output.

Ask for a bounded output length in the dispatch prompt. A worker asked for "a
verdict in under 100 words plus the raw command output" has far less room to
fabricate than one asked to "verify and report".

## Background task hygiene

A stuck child process produces no notification. Periodically audit background
tasks by real process age -- the existence of an `.output` file is not evidence
of progress.

## Orchestration

Read `docs/ORCHESTRATION.md` in full before an orchestration run; grepped
fragments previously led to using the wrong primary implementation lane. Dispatch
from `HANDOFF/todo/` tickets directly.
