# CLAUDE CODE SESSION LOCKOUT -- Sonnet-over-OpenRouter spend incident

Status: LOCKOUT ACTIVE (operator directive 2026-08-04)
Severity: P0 budget protection

No agent session may launch Claude Code (`claude`, `claude -p`, native/agent
backends in docs/ORCHESTRATION.md Section 5) until the unlock procedure below
has been run and the operator lifts the lockout. Dispatch lanes
(`scripts/delegate_task.py`) are NOT affected -- they never use Claude Code.

## What happened

Paid Anthropic Sonnet traffic reached the paid OpenRouter keys from two holes:

1. `tmp/run_fusion.py` dispatched `scripts/fusion_lite.py` with
   `--judge anthropic/claude-3.5-sonnet` against the paid openrouter_fusion.env
   key. fusion_lite.py's BYOK denylist only blocked `mistralai/`.
2. Claude Code subagents used `model: inherit`; with the OpenRouter base URL
   active, an inherited/derived model resolved to a paid Sonnet slug
   (observed in-session as Sonnet 5 spend).

## Fixes already landed

- Commit 4df163a1: `anthropic/` added to BYOK_DENYLIST_PREFIXES in
  scripts/fusion_lite.py -- no Claude slug can be dispatched through the paid
  OpenRouter endpoint by that script anymore.
- Commit 81797a40: all five `.claude/agents/*.md` pinned from `model: inherit`
  to `deepseek/deepseek-v4-flash-0731`; `.claude/hooks/model_gate.sh` wired on
  SessionStart.

## Evidence the bleeding stopped (2026-08-04)

- Operator killed the last Claude Code session; no `claude`/`node` Claude Code
  processes have been observed since (tasklist / CIM process sweep).
- OpenRouter /api/v1/key usage totals probed twice ~20 minutes apart with ZERO
  delta on every key: openrouter.env (cc36) $2.561234915/$3 cap, fusion (361)
  $0.869407361/$0.75 cap (cap already tripped -- key is hard-stopped),
  direct/OR3 (8b4) $3.51852514/$5 cap, ORlocal (8818) $0.000154836/no cap.
- The /api/v1/activity endpoint returns 403 on these keys, so per-model
  timelines cannot be pulled from the API; transcript forensics under
  `~/.claude/projects/**/*.jsonl` contain the historical Sonnet references.

## Known residual holes (fix BEFORE the unlock test)

- `ANTHROPIC_DEFAULT_SONNET_MODEL` / `ANTHROPIC_DEFAULT_HAIKU_MODEL` /
  `ANTHROPIC_DEFAULT_OPUS_MODEL` are NOT set in the OR profiles, so any code
  path that requests an alias ("sonnet") would send a default Claude slug
  through the OpenRouter base URL. Set all three to
  `deepseek/deepseek-v4-flash-0731` in the profile used for the test.
- `.claude/hooks/model_gate.sh` is advisory: SessionStart hooks cannot block a
  session that already started, and `SubagentStart` is not a hook event Claude
  Code fires. The real protection is the denylist + pinned agents + the alias
  overrides above; do not rely on the hook.
- User-level `~/.claude/settings.json` carries `"model": "qwen3.8-max-preview"`
  which is not a valid OpenRouter slug; it is overridden by the profiles' env
  today but should not be trusted.

## Unlock procedure (run in this order)

1. Apply the alias overrides above to `~/.claude/settings.local.OR3.json`
   (env block), keep `ANTHROPIC_MODEL` and `ANTHROPIC_SMALL_FAST_MODEL` pinned
   to `deepseek/deepseek-v4-flash-0731`.
2. Record the current usage totals of the direct/OR3 key via GET
   https://openrouter.ai/api/v1/key (free, no Claude Code involved).
3. Run ONE controlled session:
   `claude -p "Reply with exactly: OK" --settings %USERPROFILE%\.claude\settings.local.OR3.json`
4. Re-read the key totals. The delta must match only deepseek-v4-flash pricing
   (sub-cent for one prompt). Grep the resulting session transcript under
   `~/.claude/projects/` for `claude-sonnet|claude-opus` -- must be zero.
5. Report totals + transcript grep to the operator. Only the operator lifts the
   lockout by deleting this file's ACTIVE status line.
