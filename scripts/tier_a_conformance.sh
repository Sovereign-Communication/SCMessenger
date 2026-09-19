#!/usr/bin/env bash
# V040-T6 -- Tier A two-node conformance harness.
#
# Ticket: HANDOFF/freebuff/queue/V040_T6_TIER_A_CONFORMANCE_HARNESS.md
#
# One command that answers "are the two always-on nodes conformant right now?",
# so a peers outage or a dead watcher is noticed by running a script instead of
# by a human re-deriving it with ad-hoc curl calls.
#
# READ-ONLY. It probes both nodes over HTTP, reads the two nodes' ledgers and
# the local watcher log. No restarts, no writes to node state, no redeploys.
#
# Usage: scripts/tier_a_conformance.sh [--help]
#
# Environment:
#   SCM_AWS_HOST           cloud-node address; skips EC2 discovery when set
#   SCM_WIN_URL            Windows node control API (default http://127.0.0.1:9876)
#   SCM_SSH_KEY            SSH key for the cloud node (default ~/.ssh/scm-node-key.pem)
#   SCMESSENGER_DATA_DIR   Windows node data dir (default %LOCALAPPDATA%/scmessenger)
#   SCM_SKIP_REMOTE_SUDO=1 skip the remote probes that need sudo (A2 SHA), which
#                          are then reported as [WARNING] with the exact command
#
# Exit codes:
#   0  every row was evaluated and none is [FAIL]
#   1  at least one row is [FAIL]
#
# A row whose source could not be read prints [WARNING] together with the exact
# command that would evaluate it. It never prints [OK]: visibility fails open,
# the verdict fails closed.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RESULTS_FILE="$REPO_ROOT/scratch/driver/tier_a_conformance.json"
EVIDENCE_DIR="$REPO_ROOT/tmp/tier_a_conformance"
WATCHER_LOG="$REPO_ROOT/scratch/driver/watcher.log"

WIN_URL="${SCM_WIN_URL:-http://127.0.0.1:9876}"
SSH_KEY="${SCM_SSH_KEY:-$HOME/.ssh/scm-node-key.pem}"
AWS_LEDGER="/opt/scm-relay-data/storage/ledger.json"
AWS_LEDGER_LOCAL="$EVIDENCE_DIR/aws_ledger.json"

NODE_DATA_DIR="${SCMESSENGER_DATA_DIR:-${SCM_DATA_DIR:-${LOCALAPPDATA:-$HOME}/scmessenger}}"
WIN_LEDGER="$NODE_DATA_DIR/storage/ledger.json"
WIN_LOG_DIR="$NODE_DATA_DIR/logs"

# A9: listeners above this are reported as noise rather than conformance. The
# cloud node carried 33 (including 80/443/8080/9090) when this ticket was filed.
LISTENER_WARN_THRESHOLD=12
# A10: the watcher is expected to write at least this often.
WATCHER_MAX_AGE_MIN=120

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  sed -n '2,32p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
  exit 0
fi

FAILS=0
WARNINGS=0
ROWS=0

row_ok()   { ROWS=$((ROWS + 1)); printf '[OK]      %s\n' "$*"; }
row_fail() { ROWS=$((ROWS + 1)); FAILS=$((FAILS + 1)); printf '[FAIL]    %s\n' "$*"; }
row_warn() { ROWS=$((ROWS + 1)); WARNINGS=$((WARNINGS + 1)); printf '[WARNING] %s\n' "$*"; }
info()     { printf '          %s\n' "$*"; }
section()  { printf '\n-- %s\n' "$*"; }

http_body() { curl -s -m 8 "$1" 2>/dev/null; }

# json_field <json> <python-expression reading `d`>
json_field() {
  printf '%s' "$1" | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
except Exception:
    print('')
    raise SystemExit(0)
if not isinstance(d, dict):
    print('')
    raise SystemExit(0)
print($2)
"
}

sha_from_provenance() {
  printf '%s' "$1" | grep -oE '[0-9a-f]{7,40}' | head -1
}

# ledger_stats <path> <own_public_key_hex> <own_libp2p_peer_id> <own_external_addrs>
# prints: entries=<n> self_entries=<n> self_addrs=<n> offenders=<id;id;...>
ledger_stats() {
  python3 - "$1" "$2" "$3" "$4" <<'PY'
import json
import sys

path, own_key, own_peer, own_addrs = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4].split()
try:
    with open(path, encoding="utf-8") as fh:
        data = json.load(fh)
except Exception as exc:
    print("error=%s" % type(exc).__name__)
    raise SystemExit(0)
if not isinstance(data, list):
    print("error=shape")
    raise SystemExit(0)

# Own external addresses arrive as host:port; a ledger multiaddr spells the same
# endpoint as /ip4/<host>/tcp/<port>, so both halves must be matched, not the
# "host:port" string (which never appears in a multiaddr).
own_addr_pairs = []
for addr in own_addrs:
    if ":" in addr:
        host, port = addr.rsplit(":", 1)
        own_addr_pairs.append((host, port))

self_entries = 0
self_addrs = 0
offenders = []
for entry in data:
    if not isinstance(entry, dict):
        continue
    ids = [str(entry.get("peer_id") or ""), str(entry.get("public_key") or "")]
    observed = [str(x) for x in (entry.get("observed_peer_ids") or [])]
    if own_key and own_key in ids:
        self_entries += 1
        offenders.append("self-key:%s" % entry.get("peer_id"))
    elif own_peer and (own_peer in ids or own_peer in observed):
        self_entries += 1
        offenders.append("self-peer:%s" % entry.get("peer_id"))
    multiaddr = str(entry.get("multiaddr") or "")
    if multiaddr and any(h and ("/ip4/%s/" % h) in multiaddr and ("/tcp/%s" % p) in multiaddr
                         for h, p in own_addr_pairs):
        self_addrs += 1
        offenders.append("own-address:%s" % multiaddr)

print("entries=%d self_entries=%d self_addrs=%d offenders=%s"
      % (len(data), self_entries, self_addrs, ";".join(offenders[:5])))
PY
}

stat_field() { printf '%s' "$1" | tr ' ' '\n' | grep -m1 "^$2=" | cut -d= -f2-; }

remote_allowed() {
  [ -n "$AWS_HOST" ] || return 1
  [ -f "$SSH_KEY" ] || return 1
  return 0
}

fetch_remote_ledger() {
  remote_allowed || return 1
  mkdir -p "$EVIDENCE_DIR"
  timeout 45 ssh -i "$SSH_KEY" -o BatchMode=yes -o ConnectTimeout=10 \
    -o StrictHostKeyChecking=no "ec2-user@$AWS_HOST" "cat $AWS_LEDGER" \
    > "$AWS_LEDGER_LOCAL" 2> "$EVIDENCE_DIR/aws_ledger_ssh.err"
  local rc=$?
  if [ "$rc" -ne 0 ] || [ ! -s "$AWS_LEDGER_LOCAL" ]; then
    return 1
  fi
  return 0
}

# A2 needs the node's own startup provenance, which the CLI prints to its log
# (there is no HTTP surface for it: core/src/lib.rs get_build_provenance).
win_provenance() {
  local newest="" file
  # A read loop rather than xargs: xargs eats the backslashes in a Windows
  # absolute path, so the file list silently degrades to unreadable names.
  while IFS= read -r file; do
    if [ -z "$newest" ] || [ "$file" -nt "$newest" ]; then
      newest="$file"
    fi
  done < <(grep -rl -E 'Core Provenance:' "$WIN_LOG_DIR" 2>/dev/null)
  if [ -z "$newest" ]; then
    return 1
  fi
  grep -m1 -E 'Core Provenance:' "$newest" | tr -d '\r'
}

aws_provenance() {
  remote_allowed || return 1
  if [ "${SCM_SKIP_REMOTE_SUDO:-0}" = "1" ]; then
    return 1
  fi
  timeout 45 ssh -i "$SSH_KEY" -o BatchMode=yes -o ConnectTimeout=10 \
    -o StrictHostKeyChecking=no "ec2-user@$AWS_HOST" \
    "sudo -n docker logs scm-node 2>&1 | grep -m1 -oE 'Core Provenance: [^)]*\)'" 2>/dev/null | tr -d '\r'
}

age_minutes() {
  python3 -c "
import os, sys, time
print(int((time.time() - os.path.getmtime(sys.argv[1])) / 60))
" "$1" 2>/dev/null
}

# previous_field <node> <field> -- value recorded by the previous run, or empty.
previous_field() {
  [ -f "$RESULTS_FILE" ] || return 0
  python3 - "$RESULTS_FILE" "$1" "$2" <<'PY' 2>/dev/null
import json
import sys

try:
    with open(sys.argv[1], encoding="utf-8") as fh:
        doc = json.load(fh)
except Exception:
    raise SystemExit(0)
value = ((doc.get("previous") or {}).get(sys.argv[2]) or {}).get(sys.argv[3])
print("" if value is None else value)
PY
}

# ----------------------------------------------------------------------------
# Node discovery. Never a hardcoded address (issue I-02).
# ----------------------------------------------------------------------------
AWS_HOST="${SCM_AWS_HOST:-}"
DISCOVERY="SCM_AWS_HOST"
if [ -z "$AWS_HOST" ]; then
  DISCOVERY="EC2 API (scripts/aws_node_ip.sh)"
  AWS_HOST="$(bash "$SCRIPT_DIR/aws_node_ip.sh" | tr -d '\r\n')"
fi
AWS_URL="http://${AWS_HOST}:9876"

printf '=====================================================================\n'
printf ' Tier A conformance -- %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
printf '=====================================================================\n'
printf ' cloud node  : %s (discovery: %s)\n' "${AWS_HOST:-UNRESOLVED}" "$DISCOVERY"
printf ' windows node: %s\n' "$WIN_URL"

if [ -z "$AWS_HOST" ]; then
  row_fail "A1 cloud node not located -- set SCM_AWS_HOST=<ip> or provide EC2 credentials (~/.config/scmorc/aws.env); rows marked [WARNING] below were not evaluated"
fi

# ----------------------------------------------------------------------------
# Probes. Each body is fetched once and reused by the rows that need it.
# ----------------------------------------------------------------------------
AWS_HEALTH=""; AWS_ID=""; AWS_DIAG=""
if [ -n "$AWS_HOST" ]; then
  AWS_HEALTH="$(http_body "$AWS_URL/health")"
  AWS_ID="$(http_body "$AWS_URL/api/identity")"
  AWS_DIAG="$(http_body "$AWS_URL/api/diagnostics")"
fi
WIN_HEALTH="$(http_body "$WIN_URL/health")"
WIN_ID="$(http_body "$WIN_URL/api/identity")"
WIN_DIAG="$(http_body "$WIN_URL/api/diagnostics")"

AWS_PUBKEY="$(json_field "$AWS_ID" "d.get('public_key_hex','')")"
AWS_PEERID="$(json_field "$AWS_ID" "d.get('libp2p_peer_id','')")"
AWS_IDENTITY="$(json_field "$AWS_ID" "d.get('identity_id','')")"
WIN_PUBKEY="$(json_field "$WIN_ID" "d.get('public_key_hex','')")"
WIN_PEERID="$(json_field "$WIN_ID" "d.get('libp2p_peer_id','')")"
WIN_IDENTITY="$(json_field "$WIN_ID" "d.get('identity_id','')")"

read -r WIN_CUSTODY WIN_PATH WIN_PEERS WIN_LISTENERS <<EOF
$(json_field "$WIN_DIAG" "'%s %s %s %s' % (d.get('custody_audit_count'), d.get('connection_path_state'), ','.join(d.get('peers') or []), len(d.get('listeners') or []))")
EOF
read -r AWS_CUSTODY AWS_PATH AWS_PEERS AWS_LISTENERS <<EOF
$(json_field "$AWS_DIAG" "'%s %s %s %s' % (d.get('custody_audit_count'), d.get('connection_path_state'), ','.join(d.get('peers') or []), len(d.get('listeners') or []))")
EOF
WIN_EXT="$(json_field "$WIN_DIAG" "' '.join(d.get('external_addrs') or [])")"
AWS_EXT="$(json_field "$AWS_DIAG" "' '.join(d.get('external_addrs') or [])")"
WIN_PORTS="$(json_field "$WIN_DIAG" "','.join(sorted({(l.split('/tcp/')[-1] if '/tcp/' in l else l) for l in (d.get('listeners') or [])}))")"
AWS_PORTS="$(json_field "$AWS_DIAG" "','.join(sorted({(l.split('/tcp/')[-1] if '/tcp/' in l else l) for l in (d.get('listeners') or [])}))")"

section 'Rows'

# A1 -- both nodes reachable.
case "$WIN_HEALTH" in
  *'"status":"healthy"'*) row_ok "A1 windows /health healthy" ;;
  *) row_fail "A1 windows /health not healthy: ${WIN_HEALTH:-<no response>}" ;;
esac
if [ -z "$AWS_HOST" ]; then
  row_warn "A1 cloud /health not evaluated (cloud node unresolved)"
else
  case "$AWS_HEALTH" in
    *'"status":"healthy"'*) row_ok "A1 cloud /health healthy" ;;
    *) row_fail "A1 cloud /health not healthy: ${AWS_HEALTH:-<no response>}" ;;
  esac
fi

# A2 -- SHA parity across both nodes and origin/main.
MAIN_SHA="$(git -C "$REPO_ROOT" rev-parse origin/main 2>/dev/null | tr -d '\r')"
WIN_PROV="$(win_provenance || true)"
WIN_SHA="$(sha_from_provenance "$WIN_PROV")"
AWS_PROV="$(aws_provenance || true)"
AWS_SHA="$(sha_from_provenance "$AWS_PROV")"
if [ -z "$WIN_SHA" ]; then
  row_warn "A2 windows git hash unproven -- no 'Core Provenance:' line in any retained log under $WIN_LOG_DIR"
elif [ -z "$MAIN_SHA" ]; then
  row_warn "A2 origin/main unknown in $REPO_ROOT (run: git fetch origin main); windows=$WIN_SHA"
elif [ -z "$AWS_SHA" ]; then
  row_warn "A2 cloud git hash unproven -- run: ssh -i $SSH_KEY ec2-user@${AWS_HOST:-<ip>} \"sudo -n docker logs scm-node 2>&1 | grep -m1 'Core Provenance'\" (windows=$WIN_SHA origin/main=${MAIN_SHA:0:7})"
else
  # One node may report an abbreviated hash and the other a full 40-hex one, so
  # parity is decided on the shortest common prefix, not on string equality.
  SHORT_LEN="${#WIN_SHA}"
  if [ "${#AWS_SHA}" -lt "$SHORT_LEN" ]; then
    SHORT_LEN="${#AWS_SHA}"
  fi
  if [ "${WIN_SHA:0:$SHORT_LEN}" = "${AWS_SHA:0:$SHORT_LEN}" ] \
    && [ "${MAIN_SHA:0:$SHORT_LEN}" = "${AWS_SHA:0:$SHORT_LEN}" ]; then
    row_ok "A2 sha parity: windows=$WIN_SHA cloud=$AWS_SHA origin/main=${MAIN_SHA:0:7}"
  else
    row_fail "A2 sha mismatch: windows=$WIN_SHA cloud=$AWS_SHA origin/main=${MAIN_SHA:0:7}"
  fi
fi
info "A2 windows provenance: ${WIN_PROV:-<none>}"
info "A2 cloud provenance  : ${AWS_PROV:-<none>}"

# A3 -- identity stable against the previous run.
if [ ! -f "$RESULTS_FILE" ]; then
  row_warn "A3 identity stability unproven -- first recorded run; baseline written to $RESULTS_FILE"
else
  PREV_WIN_ID="$(previous_field windows identity_id)"
  PREV_AWS_ID="$(previous_field aws identity_id)"
  if [ -z "$PREV_WIN_ID" ] || { [ -n "$AWS_HOST" ] && [ -z "$PREV_AWS_ID" ]; }; then
    row_warn "A3 identity stability unproven -- the previous run in $RESULTS_FILE has no identity_id for a node being compared"
  elif [ "$PREV_WIN_ID" = "$WIN_IDENTITY" ] && [ "$PREV_AWS_ID" = "$AWS_IDENTITY" ]; then
    row_ok "A3 identity stable: windows=${WIN_IDENTITY:0:12} cloud=${AWS_IDENTITY:0:12}"
  else
    row_fail "A3 identity CHANGED: windows ${PREV_WIN_ID:0:12} -> ${WIN_IDENTITY:0:12}; cloud ${PREV_AWS_ID:0:12} -> ${AWS_IDENTITY:0:12} (persistence regression, issue I-01)"
  fi
fi

# A4 -- mesh formed: each node lists the other as a direct peer.
if [ -z "$AWS_HOST" ] || [ -z "$AWS_PEERID" ] || [ -z "$WIN_PEERID" ]; then
  row_warn "A4 mutual peer listing unproven (cloud node unresolved or an identity payload was empty)"
else
  case ",$WIN_PEERS," in
    *",$AWS_PEERID,"*) WIN_HAS_AWS=yes ;;
    *) WIN_HAS_AWS=no ;;
  esac
  case ",$AWS_PEERS," in
    *",$WIN_PEERID,"*) AWS_HAS_WIN=yes ;;
    *) AWS_HAS_WIN=no ;;
  esac
  if [ "$WIN_HAS_AWS" = "yes" ] && [ "$AWS_HAS_WIN" = "yes" ]; then
    row_ok "A4 mesh formed: each node lists the other"
  else
    row_fail "A4 mesh NOT formed: windows lists cloud=$WIN_HAS_AWS, cloud lists windows=$AWS_HAS_WIN (windows peers: ${WIN_PEERS:-none}; cloud peers: ${AWS_PEERS:-none})"
  fi
fi

# A5 -- connection path is not stuck bootstrapping.
if [ -z "$AWS_HOST" ]; then
  row_warn "A5 cloud path state not evaluated (cloud node unresolved)"
elif [ -n "$WIN_PATH" ] && [ "$WIN_PATH" != "Bootstrapping" ] && [ -n "$AWS_PATH" ] && [ "$AWS_PATH" != "Bootstrapping" ]; then
  row_ok "A5 connection path: windows=$WIN_PATH cloud=$AWS_PATH"
else
  row_fail "A5 connection path stuck: windows=${WIN_PATH:-<unknown>} cloud=${AWS_PATH:-<unknown>}"
fi

# A6 -- custody is live and non-decreasing across runs.
CUSTODY_DETAIL=""
CUSTODY_BAD=0
for pair in "windows:$WIN_CUSTODY" "cloud:$AWS_CUSTODY"; do
  NODE="${pair%%:*}"
  VALUE="${pair#*:}"
  if [ -z "$VALUE" ] || [ "$VALUE" = "None" ]; then
    CUSTODY_BAD=1
    CUSTODY_DETAIL="$CUSTODY_DETAIL ${NODE}=<absent>"
    continue
  fi
  PREV="$(previous_field "$NODE" custody_audit_count)"
  CUSTODY_DETAIL="$CUSTODY_DETAIL ${NODE}=${VALUE}"
  if [ -n "$PREV" ] && [ "$VALUE" -lt "$PREV" ] 2>/dev/null; then
    CUSTODY_BAD=1
    CUSTODY_DETAIL="$CUSTODY_DETAIL(regressed from $PREV)"
  fi
done
if [ "$CUSTODY_BAD" = "1" ]; then
  row_fail "A6 custody audit count absent or regressed:$CUSTODY_DETAIL"
elif [ -f "$RESULTS_FILE" ]; then
  row_ok "A6 custody live and non-decreasing:$CUSTODY_DETAIL"
else
  row_warn "A6 custody present but not yet comparable (first recorded run):$CUSTODY_DETAIL"
fi

# A7 -- ledger sanity: the gossiped ledger must carry entries on both nodes.
if [ -f "$WIN_LEDGER" ]; then
  WIN_LEDGER_OUT="$(ledger_stats "$WIN_LEDGER" "$WIN_PUBKEY" "$WIN_PEERID" "")"
  WIN_LEDGER_COUNT="$(stat_field "$WIN_LEDGER_OUT" entries)"
else
  WIN_LEDGER_OUT="error=missing"
  WIN_LEDGER_COUNT=""
fi
if [ -z "$WIN_LEDGER_COUNT" ]; then
  row_warn "A7 windows ledger unreadable at $WIN_LEDGER (${WIN_LEDGER_OUT:-error=unknown})"
elif [ "$WIN_LEDGER_COUNT" -gt 0 ] 2>/dev/null; then
  row_ok "A7 windows ledger entries=$WIN_LEDGER_COUNT ($WIN_LEDGER)"
else
  row_fail "A7 windows ledger is EMPTY (entries=0) at $WIN_LEDGER -- a node with an empty gossiped ledger has no recovery path (T2)"
fi

AWS_LEDGER_COUNT=""
if [ -z "$AWS_HOST" ]; then
  row_warn "A7 cloud ledger not evaluated (cloud node unresolved)"
elif fetch_remote_ledger; then
  AWS_LEDGER_OUT="$(ledger_stats "$AWS_LEDGER_LOCAL" "$AWS_PUBKEY" "$AWS_PEERID" "")"
  AWS_LEDGER_COUNT="$(stat_field "$AWS_LEDGER_OUT" entries)"
  if [ -z "$AWS_LEDGER_COUNT" ]; then
    row_warn "A7 cloud ledger unreadable after fetch ($AWS_LEDGER_OUT)"
  elif [ "$AWS_LEDGER_COUNT" -gt 0 ] 2>/dev/null; then
    row_ok "A7 cloud ledger entries=$AWS_LEDGER_COUNT ($AWS_LEDGER)"
  else
    row_fail "A7 cloud ledger is EMPTY (entries=0) at $AWS_LEDGER"
  fi
else
  row_warn "A7 cloud ledger unproven -- run: ssh -i $SSH_KEY ec2-user@$AWS_HOST \"cat $AWS_LEDGER\" > $AWS_LEDGER_LOCAL"
fi

# A8 -- no self-entries in either peer store (issue I-06).
check_self_entries() {
  local node="$1" stats="$2" source="$3"
  local self_entries self_addrs offenders
  self_entries="$(stat_field "$stats" self_entries)"
  self_addrs="$(stat_field "$stats" self_addrs)"
  offenders="$(stat_field "$stats" offenders)"
  if [ -z "$self_entries" ]; then
    row_warn "A8 $node peer store unreadable ($source)"
  elif [ "$self_entries" = "0" ] && [ "$self_addrs" = "0" ]; then
    row_ok "A8 $node peer store has no self-entries"
  else
    row_fail "A8 $node peer store contains itself: self_entries=$self_entries self_addrs=$self_addrs ($offenders)"
  fi
}

if [ -f "$WIN_LEDGER" ]; then
  check_self_entries "windows" \
    "$(ledger_stats "$WIN_LEDGER" "$WIN_PUBKEY" "$WIN_PEERID" "$WIN_EXT")" "$WIN_LEDGER"
else
  row_warn "A8 windows peer store unreadable at $WIN_LEDGER"
fi
if [ -n "$AWS_LEDGER_COUNT" ]; then
  check_self_entries "cloud" \
    "$(ledger_stats "$AWS_LEDGER_LOCAL" "$AWS_PUBKEY" "$AWS_PEERID" "$AWS_EXT")" "$AWS_LEDGER"
elif [ -n "$AWS_HOST" ]; then
  row_warn "A8 cloud peer store unproven -- see the A7 cloud command"
else
  row_warn "A8 cloud peer store not evaluated (cloud node unresolved)"
fi

# A9 -- listener surface sane.
check_listeners() {
  local node="$1" count="$2" ports="$3"
  if [ -z "$count" ] || [ "$count" = "None" ]; then
    row_warn "A9 $node listener count unproven"
  elif [ "$count" -gt "$LISTENER_WARN_THRESHOLD" ] 2>/dev/null; then
    row_warn "A9 $node binds $count listeners (> $LISTENER_WARN_THRESHOLD): $ports (issue I-12)"
  else
    row_ok "A9 $node binds $count listeners"
  fi
}

check_listeners "windows" "$WIN_LISTENERS" "$WIN_PORTS"
if [ -z "$AWS_HOST" ]; then
  row_warn "A9 cloud listener count not evaluated (cloud node unresolved)"
else
  check_listeners "cloud" "$AWS_LISTENERS" "$AWS_PORTS"
fi

# A10 -- the local watcher is alive.
if [ ! -f "$WATCHER_LOG" ]; then
  row_warn "A10 watcher log not present at $WATCHER_LOG -- no liveness evidence either way"
else
  WATCHER_AGE="$(age_minutes "$WATCHER_LOG")"
  if [ -z "$WATCHER_AGE" ]; then
    row_warn "A10 watcher log age unreadable at $WATCHER_LOG"
  elif [ "$WATCHER_AGE" -le "$WATCHER_MAX_AGE_MIN" ] 2>/dev/null; then
    row_ok "A10 watcher log written $WATCHER_AGE min ago"
  else
    row_fail "A10 watcher DEAD: last write $WATCHER_AGE min ago (> $WATCHER_MAX_AGE_MIN), $WATCHER_LOG (issue I-13)"
  fi
fi

# The churn row needs a redeploy, so it cannot be read-only and stays operator-run.
section 'Skipped by design'
printf '[SKIP]    Tier A churn re-measure (SHIP_PLAN G3-0): needs a redeploy, not read-only. Run IMAGE_TAG=<candidate> scripts/aws_deploy.sh, then this harness twice.\n'
ROWS=$((ROWS + 1))

# ----------------------------------------------------------------------------
# Record this run, so the next run's A3/A6 have something to compare against.
# ----------------------------------------------------------------------------
mkdir -p "$(dirname "$RESULTS_FILE")"
python3 - "$RESULTS_FILE" "$WIN_IDENTITY" "$WIN_PEERID" "$WIN_SHA" "$WIN_CUSTODY" "$WIN_LEDGER_COUNT" "$AWS_IDENTITY" "$AWS_PEERID" "$AWS_SHA" "$AWS_CUSTODY" "$AWS_LEDGER_COUNT" <<'PY'
import json
import sys
import time

path = sys.argv[1]
try:
    with open(path, encoding="utf-8") as fh:
        previous = json.load(fh).get("current")
except Exception:
    previous = None

current = {
    "recorded_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    "windows": {
        "identity_id": sys.argv[2],
        "libp2p_peer_id": sys.argv[3],
        "git_sha": sys.argv[4],
        "custody_audit_count": sys.argv[5],
        "ledger_entries": sys.argv[6],
    },
    "aws": {
        "identity_id": sys.argv[7],
        "libp2p_peer_id": sys.argv[8],
        "git_sha": sys.argv[9],
        "custody_audit_count": sys.argv[10],
        "ledger_entries": sys.argv[11],
    },
}

with open(path, "w", encoding="utf-8") as fh:
    json.dump({"current": current, "previous": previous}, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

section 'Summary'
printf '          rows=%s fail=%s warning=%s\n' "$ROWS" "$FAILS" "$WARNINGS"
printf '          results: %s\n' "$RESULTS_FILE"
if [ -s "$AWS_LEDGER_LOCAL" ]; then
  printf '          cloud ledger snapshot: %s\n' "$AWS_LEDGER_LOCAL"
fi

if [ "$FAILS" -gt 0 ]; then
  printf '[FAIL]    %s row(s) not conformant\n' "$FAILS"
  exit 1
fi
printf '[DONE]    every evaluated row is conformant\n'
exit 0
