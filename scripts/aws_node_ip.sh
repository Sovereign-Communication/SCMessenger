#!/usr/bin/env bash
# Resolve the always-on cloud node's public IP. Single source of truth.
#
# Usage: scripts/aws_node_ip.sh
#
# Order:
#   1. $SCM_AWS_HOST, when set.
#   2. The EC2 API by instance tag (the same lookup scripts/aws_deploy.sh uses).
#
# Prints the IP on success and nothing on failure, exiting 0 either way, so
# callers fail closed with their own message. It NEVER falls back to a
# hardcoded address: the node's public IP changes on every instance
# replacement, and a stale hardcoded address is what produced issue I-02.
#
# Extract of the closed ledger (see the T6 ticket): the lookup is deliberately
# a shared helper rather than a second copy in each caller.
set -uo pipefail

if [ -n "${SCM_AWS_HOST:-}" ]; then
  printf '%s\n' "$SCM_AWS_HOST"
  exit 0
fi

# The AWS session helper is machine-local and untracked (issue I-03), so it is
# looked up relative to this script's repo first and then at its historical
# absolute path. Its absence is not an error here -- the caller reports it.
SESSION_DIR=""
for candidate in \
  "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/.codebuff_deploy/aws" \
  "$HOME/Documents/GitHub/SCMessenger/.codebuff_deploy/aws"
do
  if [ -f "$candidate/scm_session.py" ]; then
    SESSION_DIR="$candidate"
    break
  fi
done

if [ -z "$SESSION_DIR" ]; then
  exit 0
fi

python3 - "$SESSION_DIR" <<'PY'
import sys

sys.path.insert(0, sys.argv[1])
try:
    from scm_session import session
except Exception:
    raise SystemExit(0)

try:
    ec2 = session().client("ec2")
    r = ec2.describe_instances(Filters=[
        {"Name": "tag:Name", "Values": ["scm-always-on-node", "scm-node", "scmessenger"]},
        {"Name": "instance-state-name", "Values": ["running"]},
    ])
except Exception:
    raise SystemExit(0)

ips = [i.get("PublicIpAddress") for res in r.get("Reservations", [])
       for i in res.get("Instances", []) if i.get("PublicIpAddress")]
print(ips[0] if ips else "", end="")
PY
