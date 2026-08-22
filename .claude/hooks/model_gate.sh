#!/usr/bin/env bash
# Model gate hook (SessionStart + SubagentStart).
#
# P0 enforcement: every session and every subagent in this repo MUST run the
# exact model deepseek/deepseek-v4-flash-0731. Any other model -- especially a
# costly Claude Sonnet/Opus slug routed through the OpenRouter Anthropic
# gateway -- is blocked at session/subagent start.
#
# The current model is taken from ANTHROPIC_MODEL (set in the shell env; for
# subagent starts the hook input JSON may carry a model too). An exact string
# equality check -- no prefix/suffix tolerance -- decides allow vs block.
set -uo pipefail

EXPECTED="deepseek/deepseek-v4-flash-0731"

# The current model: prefer the running process env, then the hook stdin JSON
# if it carries an ANTHROPIC_MODEL equivalent.
CURRENT="${ANTHROPIC_MODEL:-}"
if [ -z "$CURRENT" ] && [ -p /dev/stdin ]; then
  # stdin may hold hook input JSON; extract a model field if present
  IN="$(cat 2>/dev/null || true)"
  CAND="$(printf '%s' "$IN" | python -c 'import sys,json
try:
  d=json.load(sys.stdin)
  print(d.get("model") or d.get("tool_input",{}).get("model") or "")
except Exception:
  print("")' 2>/dev/null || true)"
  if [ -n "$CAND" ]; then
    CURRENT="$CAND"
  fi
fi

if [ "$CURRENT" = "$EXPECTED" ]; then
  # Allow: emit SessionStart/SubagentStart success JSON.
  printf '{"systemMessage":"Model confirmed: %s","continue":true}' "$EXPECTED"
  exit 0
fi

# Block: emit a stop JSON explaining the exact-match requirement.
printf '{"systemMessage":"MODEL GATE BLOCKED: active model is not the required exact deepseek flash", "stopReason":"This SCMessenger session/subagent must run exactly deepseek/deepseek-v4-flash-0731 (got %s). Re-launch with ANTHROPIC_MODEL=deepseek/deepseek-v4-flash-0731 and no other model override; do not route a costly Sonnet/Opus slug through the OpenRouter gateway.", "continue":false}' "${CURRENT:-<unset>}"
echo
exit 0
