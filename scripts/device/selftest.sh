#!/usr/bin/env bash
# =============================================================================
# scripts/device/selftest.sh
#
# Proves the device-day machinery WITHOUT a handset: a stub `adb` stands in for
# the device, so the verdict vocabulary, the no-device preconditions, the JSON
# field reader, the TCP probe and the UTF-16 mesh.log decode are all exercised
# for real rather than asserted. This is what keeps the item scripts from being
# unverifiable scaffolding while the handset is away.
#
# It does NOT produce a conformance verdict for any item: a stub cannot prove an
# item is parity-ready, and T7 acceptance 3 forbids claiming otherwise. This file
# is deliberately NOT an item script -- it uses a stub device on purpose.
#
# Usage: scripts/device/selftest.sh
# =============================================================================
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

dev_args "$@"
if [[ "$DEV_HELP" == "1" ]]; then sed -n '1,20p' "$0"; exit 0; fi
dev_init "selftest" "device-day machinery self-test (stub adb, no handset)"
dev_evidence_bar "the assertion vocabulary and the precondition paths themselves; no item verdict is derived from this run"
dev_disposition "none" "does not block anything; this is a harness self-test"

SELFTEST_FAILS=0
dev_check() {  # dev_check <description> <expected-substring> <command...>
  local desc="$1" want="$2"; shift 2
  local out want_norm out_norm
  out="$("$@" 2>&1)"
  # Verdict lines are space-padded for alignment, so compare with runs of spaces
  # collapsed -- the tag and the words are what matter.
  want_norm="$(tr -s ' ' <<< "$want")"
  out_norm="$(tr -s ' ' <<< "$out")"
  if grep -qF -- "$want_norm" <<< "$out_norm"; then
    echo "[OK]      selftest: $desc"
  else
    echo "[FAIL]    selftest: $desc (expected '$want'; got: $(head -1 <<< "$out"))"
    SELFTEST_FAILS=$((SELFTEST_FAILS + 1))
  fi
}

# --- stub adb ----------------------------------------------------------------
STUB_DIR="$DEV_RUN_DIR/fake-bin"
mkdir -p "$STUB_DIR"
cat > "$STUB_DIR/adb" <<'STUB'
#!/usr/bin/env bash
args="$*"
case "$args" in
  *devices*)
    if [[ -n "${STUB_NO_DEVICE:-}" ]]; then
      printf 'List of devices attached\n'
    else
      printf 'List of devices attached\nSTUB123\tdevice\n'
    fi ;;
  *"shell getprop ro.product.model"*)
    if [[ -n "${STUB_EMULATOR:-}" ]]; then printf 'sdk_gphone64_x86_64\n'; else printf 'Pixel 6a\n'; fi ;;
  *"shell settings get global wifi_on"*) printf '0\n' ;;
  *"shell settings get global bluetooth_on"*) printf '1\n' ;;
  *"shell pm path"*) printf 'package:/data/app/~~stub/base.apk\n' ;;
  *"shell dumpsys package"*) printf 'signatures=abcdef0123456789\n' ;;
  *"logcat -d"*) printf 'stub logcat: git stamp stubhash\n' ;;
  *"cat files/logs/scmessenger-mesh.log"*)
    if command -v python3 >/dev/null 2>&1; then
      python3 -c 'import sys; sys.stdout.buffer.write("\ufeffscm-selftest-marker\n".encode("utf-16-le"))'
    else
      printf 'scm-selftest-marker\n'
    fi ;;
  *"cat files/mesh_diagnostics.log"*) printf 'stub diagnostics\n' ;;
  *"cat files/ledger.json"*) printf '{"entries":[]}\n' ;;
  *) printf '\n' ;;
esac
STUB
chmod +x "$STUB_DIR/adb"

FIXTURE="$DEV_RUN_DIR/fixture.json"
cat > "$FIXTURE" <<'JSON'
{"peers": [{"peer_id": "12D3KooWstub", "listeners": []}], "custody_audit_count": 4}
JSON
printf '{not json' > "$DEV_RUN_DIR/bad.json"

# --- 1. pure functions (present, absent, malformed) -------------------------
dev_check "dev_json_field reads a nested key" "GOT:12D3KooWstub" \
  bash -c 'source "$1"; v="$(dev_json_field "$2" "peers.0.peer_id")"; [[ -n "$v" ]] && echo "GOT:$v" || echo "EMPTY"' \
  _ "$DEV_LIB_DIR/lib.sh" "$FIXTURE"

dev_check "dev_json_field reads a top-level key" "GOT:4" \
  bash -c 'source "$1"; v="$(dev_json_field "$2" "custody_audit_count")"; [[ -n "$v" ]] && echo "GOT:$v" || echo "EMPTY"' \
  _ "$DEV_LIB_DIR/lib.sh" "$FIXTURE"

dev_check "dev_json_field reports an absent key as unmeasured" "EMPTY" \
  bash -c 'source "$1"; v="$(dev_json_field "$2" "listeners_never_sent")"; [[ -n "$v" ]] && echo "GOT:$v" || echo "EMPTY"' \
  _ "$DEV_LIB_DIR/lib.sh" "$FIXTURE"

dev_check "dev_json_field reports a malformed file as unmeasured" "EMPTY" \
  bash -c 'source "$1"; v="$(dev_json_field "$2" "peers")"; [[ -n "$v" ]] && echo "GOT:$v" || echo "EMPTY"' \
  _ "$DEV_LIB_DIR/lib.sh" "$DEV_RUN_DIR/bad.json"

# --- 2. preconditions (T7 acceptance 2) -------------------------------------
# adb lives in the Android SDK, not in /usr/bin, so a coreutils-only PATH hides
# exactly adb while leaving the library's own tools available.
dev_check "adb missing from PATH fails closed" "[FAIL] no device" \
  bash -c 'PATH=/usr/bin:/bin; source "$1"; dev_require_adb' _ "$DEV_LIB_DIR/lib.sh"

dev_check "empty device list fails closed" "[FAIL] no device" \
  bash -c 'STUB_NO_DEVICE=1; export STUB_NO_DEVICE; PATH="$1:$PATH"; source "$2"; dev_require_adb' \
  _ "$STUB_DIR" "$DEV_LIB_DIR/lib.sh"

dev_check "one usable device passes the precondition" "[OK] handset visible" \
  bash -c 'PATH="$1:$PATH"; source "$2"; dev_require_adb' _ "$STUB_DIR" "$DEV_LIB_DIR/lib.sh"

dev_check "an emulator is rejected where a radio path is required" "[FAIL] radio path needs a physical handset" \
  bash -c 'STUB_EMULATOR=1; export STUB_EMULATOR; PATH="$1:$PATH"; source "$2"; dev_require_physical' \
  _ "$STUB_DIR" "$DEV_LIB_DIR/lib.sh"

dev_check "a physical handset passes the radio precondition" "[OK] physical handset attached" \
  bash -c 'PATH="$1:$PATH"; source "$2"; dev_require_physical' _ "$STUB_DIR" "$DEV_LIB_DIR/lib.sh"

# --- 3. passive evidence pull, including the UTF-16 decode ------------------
STUB_PULL_DIR="$DEV_RUN_DIR/pull-check"
mkdir -p "$STUB_PULL_DIR"
bash -c 'PATH="$1:$PATH"; source "$2"; DEV_RUN_DIR="$3"; export DEV_RUN_DIR; dev_pull_phone_evidence' \
  _ "$STUB_DIR" "$DEV_LIB_DIR/lib.sh" "$STUB_PULL_DIR" > "$DEV_RUN_DIR/pull_check_output.txt" 2>&1

dev_check_pull_file() {
  if [[ -s "$STUB_PULL_DIR/$1" ]]; then
    echo "[OK]      selftest: evidence pull produced $1"
  else
    echo "[FAIL]    selftest: evidence pull did not produce $1"
    SELFTEST_FAILS=$((SELFTEST_FAILS + 1))
  fi
}
dev_check_pull_file logcat.txt
dev_check_pull_file diag.log
dev_check_pull_file phone_ledger.json

if [[ -f "$STUB_PULL_DIR/mesh.log" ]] && grep -qF 'scm-selftest-marker' "$STUB_PULL_DIR/mesh.log"; then
  echo "[OK]      selftest: UTF-16 mesh.log decoded into a greppable file"
else
  echo "[FAIL]    selftest: UTF-16 mesh.log decode did not yield the marker"
  SELFTEST_FAILS=$((SELFTEST_FAILS + 1))
fi

if [[ -f "$STUB_PULL_DIR/mesh.log" ]]; then
  bytes_all="$(wc -c < "$STUB_PULL_DIR/mesh.log" | tr -d ' ')"
  bytes_no_nul="$(tr -d '\0' < "$STUB_PULL_DIR/mesh.log" | wc -c | tr -d ' ')"
  if [[ "$bytes_all" == "$bytes_no_nul" ]]; then
    echo "[OK]      selftest: decoded mesh.log carries no NUL bytes"
  else
    echo "[FAIL]    selftest: decoded mesh.log still contains NUL bytes ($bytes_all vs $bytes_no_nul)"
    SELFTEST_FAILS=$((SELFTEST_FAILS + 1))
  fi
fi

# --- 4. the TCP probe used to prove a transport is really gone --------------
dev_check "closed port is reported unreachable" "closed-ok" \
  bash -c 'source "$1"; if dev_tcp_probe 127.0.0.1 1 2; then echo "unexpectedly-open"; else echo "closed-ok"; fi' \
  _ "$DEV_LIB_DIR/lib.sh"

if command -v python3 >/dev/null 2>&1; then
  python3 -m http.server 18777 --bind 127.0.0.1 >/dev/null 2>&1 &
  SERVER_PID=$!
  sleep 2
  dev_check "open port is reported reachable" "open-ok" \
    bash -c 'source "$1"; if dev_tcp_probe 127.0.0.1 18777 3; then echo "open-ok"; else echo "unexpectedly-closed"; fi' \
    _ "$DEV_LIB_DIR/lib.sh"
  kill "$SERVER_PID" 2>/dev/null
fi

# --- 5. summary --------------------------------------------------------------
echo
if [[ "$SELFTEST_FAILS" -gt 0 ]]; then
  echo "[FAIL] selftest: $SELFTEST_FAILS check(s) failed; the machinery is not trustworthy yet"
  exit 1
fi
echo "[OK] selftest: every machinery check passed (no item verdict is implied)"
exit 0
