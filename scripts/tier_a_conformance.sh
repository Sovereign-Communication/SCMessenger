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
#   SCM_SKIP_REMOTE_SUDO=1 skip the remote probes that need sudo (A2 SHA)
#   SCM_RESAMPLE_ATTEMPTS  reads of a transient mesh state before it is called a
#                          regression (default 2: the first sample plus one
#                          re-sample). Raise it to chase a longer blip.
#   SCM_RESAMPLE_DELAY     seconds between those reads (default 1)
#
# Row verdicts:
#   [OK]      the criterion was measured and holds
#   [FAIL]    the criterion was measured and does not hold -- exit 1
#   [WARNING] measured, but outside its own pass condition, or below a threshold,
#             or a transient state that did not persist across re-samples
#   [SKIP]    NOT measured, so no verdict is available; the reason is printed
#
# A row whose source cannot be read prints [WARNING] or [SKIP] and names the
# command that would evaluate it -- never [OK], and never a [FAIL] for a value
# that was never read. One exception is deliberate: reaching the node IS A1's
# criterion, so a node that does not answer is the finding and A1 [FAIL]s.
#
# Measured-ness has exactly ONE definition in this script (measured(), below):
# json_field removes the placeholder that a key the node never sent evaluates to,
# every row asks measured(), and state_arg -- the only thing that feeds the state
# writer -- blanks anything that is not a measurement. So an absent field can
# never be stored as a value, and the record can never claim an observation that
# did not happen.
#
# Absent is not the same fact as empty, and the two collapse the moment a body is
# reduced to text. Presence is therefore recorded at the parse boundary, where
# the key set is still visible (mark_present), and asked for by name afterwards
# (present / reported). A row decides "can I judge my criterion?" from that,
# never from an empty string -- which is what stops "sent an empty list" and
# "never sent the field" from producing the same verdict.
#
# Cross-run state (scratch/driver/tier_a_conformance.json):
#   Each run records what it measured, and CARRIES FORWARD the last value it did
#   measure for anything it could not. A run against a dead cloud node therefore
#   cannot erase the baseline, and the next healthy run still compares against
#   the last known good value -- which is the only way an identity change that
#   happens during an outage can ever be detected. Values that have never been
#   measured stay null: an unknown is not the same fact as an empty one.
#
# Exit codes:
#   0  every row was evaluated and none is [FAIL]
#   1  at least one row is [FAIL]
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
  sed -n '2,/^set -uo pipefail$/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'
  exit 0
fi

FAILS=0
WARNINGS=0
SKIPS=0
ROWS=0

row_ok()   { ROWS=$((ROWS + 1)); printf '[OK]      %s\n' "$*"; }
row_fail() { ROWS=$((ROWS + 1)); FAILS=$((FAILS + 1)); printf '[FAIL]    %s\n' "$*"; }
row_warn() { ROWS=$((ROWS + 1)); WARNINGS=$((WARNINGS + 1)); printf '[WARNING] %s\n' "$*"; }
row_skip() { ROWS=$((ROWS + 1)); SKIPS=$((SKIPS + 1)); printf '[SKIP]    %s\n' "$*"; }
info()     { printf '          %s\n' "$*"; }
section()  { printf '\n-- %s\n' "$*"; }

http_body() { curl -s -m 8 "$1" 2>/dev/null; }

# Compare JSON on content, not on whitespace: a re-serialized body must not turn
# a healthy node into a [FAIL]. One tr, so the fast path stays fast.
json_has() { printf '%s' "$1" | tr -d ' \t\r\n' | grep -q "\"$2\":\"$3\""; }

# json_field <json> <python-expression reading `d`>
#
# A key the node did not send evaluates to the literal None, and a JSON null
# evaluates to the same thing. Neither is a measurement, so neither leaves this
# function as text: mapping them to nothing here is what makes a placeholder
# impossible downstream, and is what lets measured() be the only definition of
# measured-ness in this script.
json_field() {
  local out
  out="$(printf '%s' "$1" | python3 -c "
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
")"
  case "$out" in
    None|null) out="" ;;
  esac
  printf '%s' "$out"
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

is_number() { [ -n "$1" ] && [ "$1" -eq "$1" ] 2>/dev/null; }

# The previous record is read once and asked many times. One parse, not one per
# field: it keeps the state schema in a single place, and a healthy run inside
# its time budget. Keys are the state file's node names (windows, aws).
PREV_STATE=""
load_previous_state() {
  [ -f "$RESULTS_FILE" ] || return 0
  PREV_STATE="$(python3 - "$RESULTS_FILE" <<'PY' 2>/dev/null
import json
import sys

try:
    with open(sys.argv[1], encoding="utf-8") as fh:
        doc = json.load(fh)
except Exception:
    raise SystemExit(0)
record = doc.get("previous") if isinstance(doc, dict) else None
if not isinstance(record, dict):
    raise SystemExit(0)
for node in ("windows", "aws"):
    entry = record.get(node)
    if not isinstance(entry, dict):
        continue
    for field in ("identity_id", "custody_audit_count"):
        value = entry.get(field)
        # null and "" both mean never measured: an unmeasured value must not be
        # comparable, which is what turns A3/A6 into [SKIP] rather than a guess.
        if value not in (None, ""):
            print("%s.%s=%s" % (node, field, value))
PY
)"
}

# previous_value <node> <field> -- last value RECORDED for that node/field,
# measured by an earlier run or carried forward from one. Empty when the field
# has never been measured. <node> is a state-file key: windows or aws.
previous_value() {
  printf '%s\n' "$PREV_STATE" | grep -m1 "^$1\.$2=" | cut -d= -f2-
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
  row_fail "A1 cloud node not located -- set SCM_AWS_HOST=<ip> or provide EC2 credentials (~/.config/scmorc/aws.env); rows marked [SKIP] below were not evaluated"
fi

# ----------------------------------------------------------------------------
# Probes. Each body is fetched once and reused by the rows that need it.
#
# The parse records TWO things per body: the values every row reads, and the set
# of keys the node actually sent. Presence is read here, in the same expression
# as the values, because this is the last place the key set is still visible --
# once a body has been reduced to text, "sent an empty list" and "never sent the
# field" are the same characters. It costs no extra process.
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

# identity: identity_id | public_key_hex | libp2p_peer_id | <keys the node sent>
IFS='|' read -r WIN_IDENTITY WIN_PUBKEY WIN_PEERID WIN_IDENT_KEYS <<EOF
$(json_field "$WIN_ID" "'|'.join([str(d.get('identity_id','')), str(d.get('public_key_hex','')), str(d.get('libp2p_peer_id','')), ' '.join(k for k in ('identity_id','public_key_hex','libp2p_peer_id') if k in d)])")
EOF
IFS='|' read -r AWS_IDENTITY AWS_PUBKEY AWS_PEERID AWS_IDENT_KEYS <<EOF
$(json_field "$AWS_ID" "'|'.join([str(d.get('identity_id','')), str(d.get('public_key_hex','')), str(d.get('libp2p_peer_id','')), ' '.join(k for k in ('identity_id','public_key_hex','libp2p_peer_id') if k in d)])")
EOF

# diagnostics: custody | path | peers | listener count | listener ports |
#              external addrs | <keys the node sent>
IFS='|' read -r WIN_CUSTODY WIN_PATH WIN_PEERS WIN_LISTENERS WIN_PORTS WIN_EXT WIN_DIAG_KEYS <<EOF
$(json_field "$WIN_DIAG" "'|'.join([str(d.get('custody_audit_count')), str(d.get('connection_path_state','')), ','.join(str(p) for p in (d.get('peers') or [])), str(len(d.get('listeners') or [])), ','.join(sorted({(l.split('/tcp/')[-1] if '/tcp/' in str(l) else str(l)) for l in (d.get('listeners') or [])})), ' '.join(str(a) for a in (d.get('external_addrs') or [])), ' '.join(k for k in ('custody_audit_count','connection_path_state','peers','listeners','external_addrs') if k in d)])")
EOF
IFS='|' read -r AWS_CUSTODY AWS_PATH AWS_PEERS AWS_LISTENERS AWS_PORTS AWS_EXT AWS_DIAG_KEYS <<EOF
$(json_field "$AWS_DIAG" "'|'.join([str(d.get('custody_audit_count')), str(d.get('connection_path_state','')), ','.join(str(p) for p in (d.get('peers') or [])), str(len(d.get('listeners') or [])), ','.join(sorted({(l.split('/tcp/')[-1] if '/tcp/' in str(l) else str(l)) for l in (d.get('listeners') or [])})), ' '.join(str(a) for a in (d.get('external_addrs') or [])), ' '.join(k for k in ('custody_audit_count','connection_path_state','peers','listeners','external_addrs') if k in d)])")
EOF

# "Empty" and "not observed" are different facts, and every row below has to say
# which one it is looking at. This is the only definition of measured-ness in
# the script: json_field has already turned the placeholder for a missing key
# into nothing, and state_arg (below) is the only path into the state writer.
measured() { [ -n "${1:-}" ] && [ "$1" != "None" ] && [ "$1" != "null" ]; }

# state_arg <node> <field> <value> -- one "<node>.<field>=<value>" argument for
# the state writer, BLANK when the value is not a measurement. This is where a
# field's measured-ness is decided for the record; the writer stores what it is
# given and has no second opinion, so an absent field is stored as not-measured
# (observed false, value null) and no placeholder string can ever be written.
state_arg() {
  if measured "${3:-}"; then
    printf '%s.%s=%s\n' "$1" "$2" "$3"
  else
    printf '%s.%s=\n' "$1" "$2"
  fi
}

# ----------------------------------------------------------------------------
# Presence: did the node send this key at all?
#
# The single owner of that question. `present` answers it for one key; `reported`
# adds "and is the value usable", which is what a row actually needs before it
# can judge a criterion. Node names here are the ones the rows use (windows,
# cloud); the state file's own names (windows, aws) are a separate concern.
declare -A PRESENT=()

mark_present() {
  local node="$1" keys=" ${2:-} "
  shift 2
  local key
  for key in "$@"; do
    case "$keys" in
      *" $key "*) PRESENT["$node.$key"]=yes ;;
      *) PRESENT["$node.$key"]=no ;;
    esac
  done
}

present() { [ "${PRESENT["$1.$2"]:-no}" = "yes" ]; }

# reported <node> <key> <value> -- the node sent the key AND the value is usable.
reported() { present "$1" "$2" && measured "${3:-}"; }

mark_present windows "$WIN_IDENT_KEYS" identity_id public_key_hex libp2p_peer_id
mark_present windows "$WIN_DIAG_KEYS" custody_audit_count connection_path_state peers listeners external_addrs
mark_present cloud "$AWS_IDENT_KEYS" identity_id public_key_hex libp2p_peer_id
mark_present cloud "$AWS_DIAG_KEYS" custody_audit_count connection_path_state peers listeners external_addrs

# ----------------------------------------------------------------------------
# Transient mesh state.
#
# connection_path_state and the peer list are outputs of a state machine, not
# counters: a node that has just (re)connected reports Bootstrapping and no
# peers for a few seconds. One sample of that is not a regression -- observed
# live on 2026-09-19, the cloud node reported Bootstrapping with zero peers and
# was back to DirectPreferred with a peer present within 15 minutes, and a
# single-sample [FAIL] there was the harness crying wolf. The failing path of A4
# and A5 is therefore re-sampled, and only a condition that persists across
# every sample is a [FAIL]; one that clears is reported as [WARNING] carrying
# both readings. A healthy run never re-samples, so it pays nothing for this.
#
# The default budget is deliberately small (the first sample plus one a second
# later) so that a rig with a persistently failing row still runs in about the
# same time as before; raise SCM_RESAMPLE_ATTEMPTS to chase a longer blip. A
# [FAIL] here prints every sample it took, so a condition that outlasted the
# budget is visible as evidence rather than asserted from one reading.
RESAMPLE_ATTEMPTS="${SCM_RESAMPLE_ATTEMPTS:-2}"
RESAMPLE_DELAY="${SCM_RESAMPLE_DELAY:-1}"

# resample_mesh <windows|cloud> -- the node's current "<path>|<peers>", fetched
# now (not the cached first sample). Empty on no response.
resample_mesh() {
  local diag=""
  case "$1" in
    windows) diag="$(http_body "$WIN_URL/api/diagnostics")" ;;
    cloud)
      [ -n "$AWS_HOST" ] || return 1
      diag="$(http_body "$AWS_URL/api/diagnostics")"
      ;;
    *) return 1 ;;
  esac
  json_field "$diag" "'|'.join([str(d.get('connection_path_state','')), ','.join(str(p) for p in (d.get('peers') or []))])"
}

# retry_until <attempts> <delay> <predicate...> -- true as soon as the predicate
# holds. The cadence lives here; what "ready" means stays with the row that owns
# the criterion.
retry_until() {
  local attempts="$1" delay="$2" attempt=0
  shift 2
  while [ "$attempt" -lt "$attempts" ]; do
    attempt=$((attempt + 1))
    sleep "$delay"
    if "$@"; then
      return 0
    fi
  done
  return 1
}

# A re-sampled row records what its re-samples saw, so a transient verdict
# prints its evidence instead of asserting itself.
RESAMPLE_TRACE=""
trace_sample() { RESAMPLE_TRACE="$RESAMPLE_TRACE; $1"; }

# A4's predicate: both nodes now list each other.
a4_mutual_now() {
  local win aws
  win="$(resample_mesh windows | cut -d'|' -f2)"
  aws="$(resample_mesh cloud | cut -d'|' -f2)"
  trace_sample "windows peers=[${win:-<none>}] cloud peers=[${aws:-<none>}]"
  case ",$win," in *",$AWS_PEERID,"*) ;; *) return 1 ;; esac
  case ",$aws," in *",$WIN_PEERID,"*) ;; *) return 1 ;; esac
  return 0
}

# A5's predicate: neither node is still bootstrapping.
a5_ready_now() {
  local win aws
  win="$(resample_mesh windows | cut -d'|' -f1)"
  aws="$(resample_mesh cloud | cut -d'|' -f1)"
  trace_sample "windows=${win:-<no response>} cloud=${aws:-<no response>}"
  [ "$win" != "Bootstrapping" ] && [ "$aws" != "Bootstrapping" ] \
    && measured "$win" && measured "$aws"
}

section 'Rows'

# A1 -- both nodes reachable. The criterion IS reachability, so an unreadable
# health body is the finding, not a gap in the evidence: this row [FAIL]s on no
# response (naming the body) and cannot [OK] unless the node actually said
# healthy.
if json_has "$WIN_HEALTH" status healthy; then
  row_ok "A1 windows /health healthy"
else
  row_fail "A1 windows /health not healthy: ${WIN_HEALTH:-<no response>}"
fi
if [ -z "$AWS_HOST" ]; then
  row_skip "A1 cloud /health not evaluated (cloud node unresolved)"
elif json_has "$AWS_HEALTH" status healthy; then
  row_ok "A1 cloud /health healthy"
else
  row_fail "A1 cloud /health not healthy: ${AWS_HEALTH:-<no response>}"
fi

# A2 -- SHA parity across both nodes and origin/main. Every input is guarded: a
# provenance line without a hash, an unknown origin/main or an unreadable log
# is [WARNING] with the command, never a comparison against nothing.
MAIN_SHA="$(git -C "$REPO_ROOT" rev-parse origin/main 2>/dev/null | tr -d '\r')"
WIN_PROV="$(win_provenance || true)"
WIN_SHA="$(sha_from_provenance "$WIN_PROV")"
AWS_PROV="$(aws_provenance || true)"
AWS_SHA="$(sha_from_provenance "$AWS_PROV")"
if [ -z "$WIN_SHA" ]; then
  row_warn "A2 windows git hash unproven -- no 'Core Provenance:' line in any retained log under $WIN_LOG_DIR; run: grep -rl 'Core Provenance:' $WIN_LOG_DIR"
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

# A3 -- identity stable against the last recorded value. Both sides must have
# reported a usable identity this run and have a recorded value; anything less is
# [SKIP], because an unmeasured identity is not evidence of a changed one.
load_previous_state
PREV_WIN_ID="$(previous_value windows identity_id)"
PREV_AWS_ID="$(previous_value aws identity_id)"
if ! reported windows identity_id "$WIN_IDENTITY" && ! reported cloud identity_id "$AWS_IDENTITY"; then
  row_skip "A3 identity stability not evaluated -- no identity reported this run (windows='${WIN_IDENTITY}' cloud='${AWS_IDENTITY}')"
elif ! reported windows identity_id "$WIN_IDENTITY" || ! reported cloud identity_id "$AWS_IDENTITY"; then
  row_skip "A3 identity stability not evaluated -- one node's identity was not reported this run (windows='${WIN_IDENTITY}' cloud='${AWS_IDENTITY}')"
elif [ -z "$PREV_WIN_ID" ] || [ -z "$PREV_AWS_ID" ]; then
  row_skip "A3 identity stability not evaluated -- no recorded identity to compare against (windows='${PREV_WIN_ID}' cloud='${PREV_AWS_ID}'); this run seeds the baseline"
elif [ "$PREV_WIN_ID" = "$WIN_IDENTITY" ] && [ "$PREV_AWS_ID" = "$AWS_IDENTITY" ]; then
  row_ok "A3 identity stable: windows=${WIN_IDENTITY:0:12} cloud=${AWS_IDENTITY:0:12}"
else
  row_fail "A3 identity CHANGED: windows ${PREV_WIN_ID:0:12} -> ${WIN_IDENTITY:0:12}; cloud ${PREV_AWS_ID:0:12} -> ${AWS_IDENTITY:0:12} (persistence regression, issue I-01)"
fi

# A4 -- mesh formed: each node lists the other as a direct peer.
#
# A peer list is transient state, so the failing path is re-sampled before it is
# called a regression. And a list the node never sent is a different fact from a
# list it sent empty: an unreported list is [WARNING] naming the command, because
# a mesh cannot be judged from a field that was not there, while an empty list the
# node did send is a judgement it can and does lose.
if [ -z "$AWS_HOST" ] || [ -z "$AWS_PEERID" ] || [ -z "$WIN_PEERID" ]; then
  row_warn "A4 mutual peer listing unproven (cloud node unresolved or an identity payload was empty)"
elif ! present windows peers || ! present cloud peers; then
  A4_UNREPORTED=""
  present windows peers || A4_UNREPORTED="windows"
  present cloud peers || A4_UNREPORTED="${A4_UNREPORTED:+$A4_UNREPORTED }cloud"
  row_warn "A4 mutual peer listing not evaluated -- $A4_UNREPORTED did not report a peers list, so an empty mesh and an unreported one cannot be told apart; run: curl -s $WIN_URL/api/diagnostics and curl -s $AWS_URL/api/diagnostics"
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
    RESAMPLE_TRACE=""
    if retry_until "$((RESAMPLE_ATTEMPTS - 1))" "$RESAMPLE_DELAY" a4_mutual_now; then
      row_warn "A4 mutual peer listing was absent on the first of $RESAMPLE_ATTEMPTS samples and present on re-sample, so this is reported not failed (windows peers: ${WIN_PEERS:-none}; cloud peers: ${AWS_PEERS:-none})$RESAMPLE_TRACE"
    else
      row_fail "A4 mesh NOT formed: windows lists cloud=$WIN_HAS_AWS, cloud lists windows=$AWS_HAS_WIN (windows peers: ${WIN_PEERS:-none}; cloud peers: ${AWS_PEERS:-none})$RESAMPLE_TRACE"
    fi
  fi
fi

# A5 -- connection path is not stuck bootstrapping. A path state the node did not
# report is not a stuck one, and neither is a single Bootstrapping sample: the
# failing path is re-sampled, and only a path that is still Bootstrapping across
# every sample is a [FAIL].
if ! reported windows connection_path_state "$WIN_PATH" || ! reported cloud connection_path_state "$AWS_PATH"; then
  row_skip "A5 connection path not evaluated -- not reported (windows='${WIN_PATH}' cloud='${AWS_PATH}')"
elif [ "$WIN_PATH" != "Bootstrapping" ] && [ "$AWS_PATH" != "Bootstrapping" ]; then
  row_ok "A5 connection path: windows=$WIN_PATH cloud=$AWS_PATH"
else
  A5_STUCK=""
  [ "$WIN_PATH" = "Bootstrapping" ] && A5_STUCK="windows"
  [ "$AWS_PATH" = "Bootstrapping" ] && A5_STUCK="${A5_STUCK:+$A5_STUCK }cloud"
  RESAMPLE_TRACE=""
  if retry_until "$((RESAMPLE_ATTEMPTS - 1))" "$RESAMPLE_DELAY" a5_ready_now; then
    row_warn "A5 connection path was Bootstrapping on the first of $RESAMPLE_ATTEMPTS samples ($A5_STUCK) and is not on re-sample, so this is reported not failed (windows=$WIN_PATH cloud=$AWS_PATH)$RESAMPLE_TRACE"
  else
    row_fail "A5 connection path stuck Bootstrapping across $RESAMPLE_ATTEMPTS samples ($A5_STUCK): windows=${WIN_PATH:-<unknown>} cloud=${AWS_PATH:-<unknown>}$RESAMPLE_TRACE"
  fi
fi

# A6 -- custody live. The ticket words this as "present and non-decreasing", but
# `custody_audit_count` is NOT a monotonic counter: it is a key count over the
# custody-audit prefix (core/src/store/relay_custody.rs:1260, count_prefix), i.e.
# a live gauge of currently-tracked records. Delivery (remove_message, :1520) and
# storage-pressure purge both lower it, so a decrease is ordinary traffic, not a
# regression. A plain decrease is therefore [WARNING] carrying its delta, and
# [FAIL] is reserved for the unambiguous case: custody at zero. Both a missing
# measurement and a missing baseline are [SKIP] -- neither is a regression.
# The pair carries the state-file key AND the label, because they are not always
# the same word (the cloud node's record is stored under "aws").
CUSTODY_DETAIL=""
CUSTODY_SKIPS=""
CUSTODY_ZERO=""
CUSTODY_DECREASED=0
for pair in "windows:windows:$WIN_CUSTODY" "aws:cloud:$AWS_CUSTODY"; do
  NODE_KEY="${pair%%:*}"
  REST="${pair#*:}"
  LABEL="${REST%%:*}"
  VALUE="${REST#*:}"
  if ! measured "$VALUE"; then
    CUSTODY_SKIPS="$CUSTODY_SKIPS ${LABEL}(not measured)"
    continue
  fi
  if ! is_number "$VALUE"; then
    CUSTODY_SKIPS="$CUSTODY_SKIPS ${LABEL}(unparsable: '${VALUE}')"
    continue
  fi
  if [ "$VALUE" -eq 0 ] 2>/dev/null; then
    CUSTODY_ZERO="$CUSTODY_ZERO ${LABEL}"
  fi
  PREV="$(previous_value "$NODE_KEY" custody_audit_count)"
  if [ -z "$PREV" ] || ! is_number "$PREV"; then
    CUSTODY_SKIPS="$CUSTODY_SKIPS ${LABEL}(no recorded value)"
    CUSTODY_DETAIL="$CUSTODY_DETAIL ${LABEL}=${VALUE}"
    continue
  fi
  if [ "$VALUE" -lt "$PREV" ]; then
    CUSTODY_DECREASED=1
    CUSTODY_DETAIL="$CUSTODY_DETAIL ${LABEL}=${VALUE}(down $((PREV - VALUE)) from recorded ${PREV})"
  else
    CUSTODY_DETAIL="$CUSTODY_DETAIL ${LABEL}=${VALUE}(recorded ${PREV})"
  fi
done
if [ -n "$CUSTODY_ZERO" ]; then
  row_fail "A6 custody EMPTY on:$CUSTODY_ZERO -- a node tracking zero custody records holds no store-and-forward custody at all:$CUSTODY_DETAIL"
elif [ "$CUSTODY_DECREASED" = "1" ]; then
  row_warn "A6 custody decreased (live gauge: delivery and purge lower it, so this is reported not failed):$CUSTODY_DETAIL"
elif [ -n "$CUSTODY_SKIPS" ]; then
  row_skip "A6 custody comparison not evaluated for:$CUSTODY_SKIPS; measured:$CUSTODY_DETAIL"
else
  row_ok "A6 custody live and not below the last recorded value:$CUSTODY_DETAIL"
fi

# A7 -- ledger sanity: the gossiped ledger must carry entries on both nodes. A
# missing file or an unreadable one (bad JSON, wrong shape, permissions) leaves
# the count empty, which is [WARNING] naming the command; entries=0 is the only
# count that is a finding, and it is [FAIL]. ledger_stats prints entries=<int> or
# error=<reason>, so the count is numeric whenever it is present -- there is no
# third case to guard.
if [ -f "$WIN_LEDGER" ]; then
  WIN_LEDGER_OUT="$(ledger_stats "$WIN_LEDGER" "$WIN_PUBKEY" "$WIN_PEERID" "")"
  WIN_LEDGER_COUNT="$(stat_field "$WIN_LEDGER_OUT" entries)"
else
  WIN_LEDGER_OUT="error=missing"
  WIN_LEDGER_COUNT=""
fi
if [ -z "$WIN_LEDGER_COUNT" ]; then
  row_warn "A7 windows ledger unreadable at $WIN_LEDGER (${WIN_LEDGER_OUT:-error=unknown}) -- run: python3 -c \"import json;print(len(json.load(open(r'$WIN_LEDGER'))))\""
elif [ "$WIN_LEDGER_COUNT" -gt 0 ]; then
  row_ok "A7 windows ledger entries=$WIN_LEDGER_COUNT ($WIN_LEDGER)"
else
  row_fail "A7 windows ledger is EMPTY (entries=0) at $WIN_LEDGER -- a node with an empty gossiped ledger has no recovery path (T2)"
fi

AWS_LEDGER_COUNT=""
if [ -z "$AWS_HOST" ]; then
  row_skip "A7 cloud ledger not evaluated (cloud node unresolved)"
elif fetch_remote_ledger; then
  AWS_LEDGER_OUT="$(ledger_stats "$AWS_LEDGER_LOCAL" "$AWS_PUBKEY" "$AWS_PEERID" "")"
  AWS_LEDGER_COUNT="$(stat_field "$AWS_LEDGER_OUT" entries)"
  if [ -z "$AWS_LEDGER_COUNT" ]; then
    row_warn "A7 cloud ledger unreadable after fetch ($AWS_LEDGER_OUT) -- run: ssh -i $SSH_KEY ec2-user@$AWS_HOST \"cat $AWS_LEDGER\" > $AWS_LEDGER_LOCAL"
  elif [ "$AWS_LEDGER_COUNT" -gt 0 ]; then
    row_ok "A7 cloud ledger entries=$AWS_LEDGER_COUNT ($AWS_LEDGER)"
  else
    row_fail "A7 cloud ledger is EMPTY (entries=0) at $AWS_LEDGER"
  fi
else
  row_warn "A7 cloud ledger unproven -- run: ssh -i $SSH_KEY ec2-user@$AWS_HOST \"cat $AWS_LEDGER\" > $AWS_LEDGER_LOCAL"
fi

# A8 -- no self-entries in either peer store (issue I-06).
#
# This check needs the node's own identity to know what "itself" is. With no
# public key and no peer id, nothing can match, so a store that was never
# identified and a clean store look identical -- and the row would print [OK]
# from a source it never read. Those two inputs must therefore be REPORTED and
# usable. The external-address list is different: an empty list is a legitimate
# answer (a node behind NAT reports no external address) and makes the
# own-address half vacuous, so only the key being absent is unproven.
own_identity_note() {
  local node="$1" url="$2" pubkey="$3" peer="$4" missing=""
  reported "$node" public_key_hex "$pubkey" || missing="$missing public_key_hex"
  reported "$node" libp2p_peer_id "$peer" || missing="$missing libp2p_peer_id"
  present "$node" external_addrs || missing="$missing external_addrs"
  if [ -n "$missing" ]; then
    printf 'the node did not report%s, so its own entries cannot be identified; run: curl -s %s/api/identity and curl -s %s/api/diagnostics' \
      "$missing" "$url" "$url"
  fi
}

check_self_entries() {
  local node="$1" stats="$2" source="$3" identity_note="$4"
  local self_entries self_addrs offenders
  self_entries="$(stat_field "$stats" self_entries)"
  self_addrs="$(stat_field "$stats" self_addrs)"
  offenders="$(stat_field "$stats" offenders)"
  if [ -n "$identity_note" ]; then
    row_warn "A8 $node self-entry check unproven -- $identity_note"
  elif [ -z "$self_entries" ] || [ -z "$self_addrs" ]; then
    row_warn "A8 $node peer store unreadable ($source) -- run: python3 -c \"import json;print(len(json.load(open(r'$source'))))\""
  elif [ "$self_entries" = "0" ] && [ "$self_addrs" = "0" ]; then
    row_ok "A8 $node peer store has no self-entries"
  else
    row_fail "A8 $node peer store contains itself: self_entries=$self_entries self_addrs=$self_addrs ($offenders)"
  fi
}

if [ -f "$WIN_LEDGER" ]; then
  check_self_entries "windows" \
    "$(ledger_stats "$WIN_LEDGER" "$WIN_PUBKEY" "$WIN_PEERID" "$WIN_EXT")" "$WIN_LEDGER" \
    "$(own_identity_note windows "$WIN_URL" "$WIN_PUBKEY" "$WIN_PEERID")"
else
  row_warn "A8 windows peer store unreadable at $WIN_LEDGER -- run: ls -l $WIN_LEDGER"
fi
if [ -n "$AWS_LEDGER_COUNT" ]; then
  check_self_entries "cloud" \
    "$(ledger_stats "$AWS_LEDGER_LOCAL" "$AWS_PUBKEY" "$AWS_PEERID" "$AWS_EXT")" "$AWS_LEDGER" \
    "$(own_identity_note cloud "$AWS_URL" "$AWS_PUBKEY" "$AWS_PEERID")"
elif [ -n "$AWS_HOST" ]; then
  row_warn "A8 cloud peer store unproven -- see the A7 cloud command"
else
  row_skip "A8 cloud peer store not evaluated (cloud node unresolved)"
fi

# A9 -- listener surface sane. The parse reduces the listener list to its length,
# so absent and empty both arrive as 0 -- which is why presence decides first.
# A node that reported no list is not judged; a node that sent an empty list has
# told us it accepts no connection, and neither is conformance. Above the
# threshold the count is reported as noise rather than conformance.
check_listeners() {
  local node="$1" count="$2" ports="$3" diag_url="$4"
  if ! present "$node" listeners; then
    row_warn "A9 $node listener count not evaluated -- the node did not report a listener list; run: curl -s $diag_url/api/diagnostics"
  elif [ "$count" -eq 0 ]; then
    row_warn "A9 $node reported an empty listener list, so it can accept no connection; run: curl -s $diag_url/api/diagnostics"
  elif [ "$count" -gt "$LISTENER_WARN_THRESHOLD" ]; then
    row_warn "A9 $node binds $count listeners (> $LISTENER_WARN_THRESHOLD): $ports (issue I-12)"
  else
    row_ok "A9 $node binds $count listeners"
  fi
}

check_listeners "windows" "$WIN_LISTENERS" "$WIN_PORTS" "$WIN_URL"
if [ -z "$AWS_HOST" ]; then
  row_skip "A9 cloud listener count not evaluated (cloud node unresolved)"
else
  check_listeners "cloud" "$AWS_LISTENERS" "$AWS_PORTS" "$AWS_URL"
fi

# A10 -- the local watcher is alive. An age that cannot be read, or a negative
# one because the mtime is in the future, is not liveness evidence: [WARNING]
# with the command, never [OK].
if [ ! -f "$WATCHER_LOG" ]; then
  row_warn "A10 watcher log not present at $WATCHER_LOG -- no liveness evidence either way; run: ls -l $WATCHER_LOG"
else
  WATCHER_AGE="$(age_minutes "$WATCHER_LOG")"
  if ! is_number "$WATCHER_AGE"; then
    row_warn "A10 watcher log age unreadable at $WATCHER_LOG -- run: ls -l $WATCHER_LOG"
  elif [ "$WATCHER_AGE" -lt 0 ]; then
    row_warn "A10 watcher log mtime is $((0 - WATCHER_AGE)) min in the future, so liveness cannot be judged (clock skew or a bad mtime) -- run: ls -l $WATCHER_LOG"
  elif [ "$WATCHER_AGE" -le "$WATCHER_MAX_AGE_MIN" ] 2>/dev/null; then
    row_ok "A10 watcher log written $WATCHER_AGE min ago"
  else
    row_fail "A10 watcher DEAD: last write $WATCHER_AGE min ago (> $WATCHER_MAX_AGE_MIN), $WATCHER_LOG (issue I-13)"
  fi
fi

# The churn row needs a redeploy, so it cannot be read-only and stays operator-run.
section 'Skipped by design'
row_skip "Tier A churn re-measure (SHIP_PLAN G3-0): needs a redeploy, not read-only. Run IMAGE_TAG=<candidate> scripts/aws_deploy.sh, then this harness twice."

# ----------------------------------------------------------------------------
# Record this run: measured values replace the record, unmeasured ones carry the
# last known good value forward so a degraded run cannot erase the baseline.
# Every argument below goes through state_arg, which decided measured-ness --
# so a blank value here means "not measured", never "the value was empty".
# ----------------------------------------------------------------------------
mkdir -p "$(dirname "$RESULTS_FILE")"
STATE_NOTE="$(python3 - "$RESULTS_FILE" \
  "$(state_arg windows identity_id "$WIN_IDENTITY")" \
  "$(state_arg windows libp2p_peer_id "$WIN_PEERID")" \
  "$(state_arg windows git_sha "$WIN_SHA")" \
  "$(state_arg windows custody_audit_count "$WIN_CUSTODY")" \
  "$(state_arg windows ledger_entries "$WIN_LEDGER_COUNT")" \
  "$(state_arg aws identity_id "$AWS_IDENTITY")" \
  "$(state_arg aws libp2p_peer_id "$AWS_PEERID")" \
  "$(state_arg aws git_sha "$AWS_SHA")" \
  "$(state_arg aws custody_audit_count "$AWS_CUSTODY")" \
  "$(state_arg aws ledger_entries "$AWS_LEDGER_COUNT")" <<'PY'
import json
import sys
import time

path = sys.argv[1]
NODES = ("windows", "aws")
FIELDS = ("identity_id", "libp2p_peer_id", "git_sha", "custody_audit_count", "ledger_entries")
# A key that was never sent evaluates to these in a shell; state_arg blanks them
# before they reach us. This guard is the invariant that no placeholder can be
# stored, not a second definition of measured-ness (state_arg owns that).
PLACEHOLDERS = ("None", "null")

measured = {}
for arg in sys.argv[2:]:
    key, _, value = arg.partition("=")
    if value in PLACEHOLDERS:
        value = ""
    measured[key] = value

now = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
try:
    with open(path, encoding="utf-8") as fh:
        doc = json.load(fh)
except Exception:
    doc = {}
if not isinstance(doc, dict):
    doc = {}

previous_record = doc.get("current") if isinstance(doc.get("current"), dict) else {}

current = {"recorded_at": now}
carried = []
for node in NODES:
    known = previous_record.get(node)
    if not isinstance(known, dict):
        known = {}
    entry = {}
    fresh_here = False
    carried_here = []
    for field in FIELDS:
        value = measured.get("%s.%s" % (node, field), "")
        if value != "":
            # Measured this run, by the one definition of measured (state_arg).
            entry[field] = value
            fresh_here = True
        elif known.get(field) not in (None, ""):
            # Last known good value: an outage must not blank the baseline.
            entry[field] = known[field]
            carried_here.append(field)
        else:
            # Never measured: an unknown, not an empty fact, and not a value.
            entry[field] = None
    entry["observed"] = {field: (measured.get("%s.%s" % (node, field), "") != "") for field in FIELDS}
    entry["observed_at"] = now if fresh_here else known.get("observed_at")
    current[node] = entry
    if carried_here:
        carried.append("%s(%s)" % (node, ",".join(carried_here)))

with open(path, "w", encoding="utf-8") as fh:
    json.dump({"current": current, "previous": previous_record}, fh, indent=2, sort_keys=True)
    fh.write("\n")

if carried:
    print("state: carried forward the last recorded value for " + "; ".join(carried))
PY
)"
[ -n "$STATE_NOTE" ] && info "$STATE_NOTE"

section 'Summary'
printf '          rows=%s ok=%s fail=%s warning=%s skip=%s\n' \
  "$ROWS" "$((ROWS - FAILS - WARNINGS - SKIPS))" "$FAILS" "$WARNINGS" "$SKIPS"
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
