# Delegation & Worker Dispatch SOP

Status: Active
Last updated: 2026-08-15 (lane ranking replaced with a selection function after
the two lanes previously named PRIMARY both went to HTTP 401)

Loaded on demand. Every rule below traces to a measured failure, not a
precaution.

## Choosing a lane

**There is no primary lane.** A ranked list is the wrong shape for this problem:
between 2026-08-04 and 2026-08-15 the two lanes this document called PRIMARY
(Qwen CLI, DashScope) both went to 401, and OpenRouter silently retired four
`:free` tiers. Any list of favourites is wrong within days of being written.

Route by properties instead, re-derived from the live roster each time:

    python scripts/delegate.py --task <file> --tier <tier>
    python scripts/delegate.py --list-lanes     # current capacity + expiry
    python scripts/lane_probe.py                # re-measure

`scripts/lanes.json` holds measured latency, context, quota and per-lane quirks
for every lane, plus a `dead` list with the observed error, so a lane that fails
is recorded rather than rediscovered. It carries an explicit expiry date.

The selection function, in order of precedence:

1. **Capability.** Can the lane do this at all? A task that must run `gh`,
   `cargo`, `gradlew` or `adb` cannot go to an HTTP lane at any price.
   `delegate.py` blocks this before spending a call.
2. **Context.** Will the prompt fit? Lanes whose window cannot hold it are
   dropped automatically.
3. **Cost class.** `free` is auto-selectable. `metered` (agy-claude) and
   `expensive` (native) are never entered automatically -- escalation to them is
   always a deliberate act after a free lane has actually failed.
4. **Measured latency**, last. It only breaks ties among lanes that already
   qualify. Optimising for it first is how you end up sending a 500-line refactor
   to a 0.7s micro lane.

Quota headroom beats raw speed for bulk work: a 0.7s lane with a tight TPM
ceiling is worse than a 2.5s lane with 1M tokens/day if you are dispatching
fifty tasks.

**Escalate deliberately, and only after diagnosing why the cheap lane failed.**
Most blocks are defects in the task file, not the lane -- rewrite the spec before
you buy a bigger model. Order: better task file -> longer-context lane ->
agy-gemini (free, shell-capable) -> agy-claude (spends Anthropic quota) ->
native (verdicts only).

Full usage, task-file authoring rules and the escalation protocol live in the
`delegate` skill (`.claude/skills/delegate/SKILL.md`).

Quota ledger: `docs/QWEN_QUOTA_LEDGER.md` is canonical -- update it there.

## The failure mode that wastes the most time

Free reasoning models spend their entire `max_tokens` budget on hidden reasoning
and return `content: ""`. Measured: nemotron-nano-9b returned 0 content chars
against 5,946 reasoning chars. **This is not a refusal and not a dead lane.**
Send `reasoning: {effort: "low"}` on OpenRouter, or `{exclude: true}` for
nemotron-ultra. Never send a `reasoning` field to Google, NVIDIA NIM, Cerebras or
Groq -- they reject it. `delegate.py` handles this per-provider; if you call an
API directly, you own it.

Related: the same model can differ 4x in latency by route. `nemotron-3-ultra-550b`
is 12.5s on NVIDIA NIM direct and 48.2s through OpenRouter. When a provider's own
endpoint exists, prefer it.

## When to delegate at all

Delegate scoped, parallel, or specialized units of work. Do NOT delegate
one-liners: dispatch overhead exceeds the task. A bounded 1-5 tool-call
diagnostic is cheaper done inline. Audits and micro-tasks route to free lanes;
reserve Claude-native capacity for verdicts and judgment calls.

Do not run Fable-tier subagent or workflow fan-outs. Haiku/Sonnet workers only,
free lanes first.

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

## scripts/delegate_task.py (deprecated)

**DEPRECATED (2026-08-15).** Use `scripts/delegate.py` instead. The measured
failure modes below still apply to existing callers of `delegate_task.py`:

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
