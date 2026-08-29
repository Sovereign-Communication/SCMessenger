#!/usr/bin/env bash
# Fireworks backup opinion helper - quick sanity check / second opinion.
# Usage: scripts/fireworks_opinion.sh "prompt text"
# Reads key from ~/.config/scmorc/fireworks.env (never prints it).
set -euo pipefail
if [ $# -lt 1 ]; then
  echo "usage: $0 \"prompt\"" >&2
  exit 2
fi
ENV_FILE="$HOME/.config/scmorc/fireworks.env"
if [ ! -f "$ENV_FILE" ]; then
  echo "missing $ENV_FILE" >&2
  exit 2
fi
set -a; source "$ENV_FILE"; set +a
: "${FIREWORKS_API_KEY:?FIREWORKS_API_KEY unset}"
MODEL="${FIREWORKS_MODEL:-accounts/fireworks/models/nemotron-lightning-3p5-30b-a3b}"
python3 - "$1" "$MODEL" <<'PY'
import json, os, sys, urllib.request
prompt, model = sys.argv[1], sys.argv[2]
key = os.environ["FIREWORKS_API_KEY"]
body = json.dumps({
    "model": model,
    "max_tokens": 4096,
    "top_k": 40,
    "presence_penalty": 0,
    "frequency_penalty": 0,
    "prompt": prompt,
}).encode()
req = urllib.request.Request(
    "https://api.fireworks.ai/inference/v1/completions",
    data=body,
    headers={"Accept": "application/json", "Content-Type": "application/json",
             "Authorization": "Bearer " + key},
)
try:
    with urllib.request.urlopen(req, timeout=120) as r:
        data = json.load(r)
    print(data["choices"][0]["text"].strip())
except urllib.error.HTTPError as e:
    print("FIREWORKS_HTTP_ERROR", e.code, e.read().decode()[:500], file=sys.stderr)
    sys.exit(1)
except Exception as e:
    print("FIREWORKS_ERROR", e, file=sys.stderr)
    sys.exit(1)
PY
