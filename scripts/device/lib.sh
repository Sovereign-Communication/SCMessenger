#!/usr/bin/env bash
# =============================================================================
# scripts/device/lib.sh -- shared machinery for the device verification scripts.
# Sourced by scripts/device/*.sh. Never executed directly.
#
# This is the single owner of the device-day contract set by
# HANDOFF/freebuff/queue/V040_T7_ANDROID_PARITY_STAGING.md:
#   1. preconditions first, fail fast with a clear message
#   2. one verdict per assertion ([OK]/[FAIL]/[WARNING]/[SKIP]), never a raw
#      log dump for a human to read
#   3. every run writes its artifacts to one named directory
#   4. idempotent and safe to re-run
#
# Seat authority, docs/rules/ANDROID.md: the HANDSET side here is passive --
# install, app start/stop, read-only pulls. The human path on the phone is
# performed by the operator, prompted by dev_operator_step. Every active drive
# (sends, node restarts, firewall changes) happens on the Windows/CLI side.
# Handset evidence follows
# HANDOFF/RUNBOOK_PIXEL_PASSIVE_VERIFICATION_2026-09-11.md.
#
# `set -e` is deliberately NOT set: a failed assertion must be reported and the
# run must continue, so every check tests its own command explicitly.
# =============================================================================

set -uo pipefail

DEV_LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEV_REPO_ROOT="$(cd "$DEV_LIB_DIR/../.." && pwd)"
cd "$DEV_REPO_ROOT"

# --- configuration (env-overridable, all optional) ---------------------------
SCM_API="${SCM_API:-http://127.0.0.1:9876}"           # Windows/CLI node HTTP API
SCM_DEVICE_SERIAL="${SCM_DEVICE_SERIAL:-}"            # pin one adb serial
SCM_APK="${SCM_APK:-android/app/build/outputs/apk/release/app-release.apk}"
SCM_EXPECTED_APK_SHA="${SCM_EXPECTED_APK_SHA:-}"      # alternative to SCM_APK
SCM_RELEASE_CERT_SHA256="${SCM_RELEASE_CERT_SHA256:-}"
SCM_CLOUD_ADDR="${SCM_CLOUD_ADDR:-}"                  # host:port of the cloud node
SCM_OPERATOR_CONFIRMED="${SCM_OPERATOR_CONFIRMED:-0}" # 1 = do not wait on stdin
SCM_ANDROID_PACKAGE="${SCM_ANDROID_PACKAGE:-com.scmessenger.android}"
DEV_PHONE_PEER="${DEV_PHONE_PEER:-}"

DEV_OKS=0
DEV_FAILS=0
DEV_WARNS=0
DEV_SKIPS=0

# --- verdict vocabulary (the only place these shapes are written) ------------
dev_record() {
  local line="$1"
  printf '%s\n' "$line"
  [[ -n "${DEV_RUN_DIR:-}" ]] && printf '%s\n' "$line" >> "$DEV_RUN_DIR/verdicts.log"
}

dev_ok()   { DEV_OKS=$((DEV_OKS + 1));     dev_record "[OK]      $*"; }
dev_fail() { DEV_FAILS=$((DEV_FAILS + 1)); dev_record "[FAIL]    $*"; }
dev_warn() { DEV_WARNS=$((DEV_WARNS + 1)); dev_record "[WARNING] $*"; }
dev_skip() { DEV_SKIPS=$((DEV_SKIPS + 1)); dev_record "[SKIP]    $*"; }
dev_info() { dev_record "[INFO]    $*"; }

dev_assert() {
  local desc="$1"; shift
  if "$@" >/dev/null 2>&1; then
    dev_ok "$desc"
  else
    dev_fail "$desc (command: $*)"
  fi
}

dev_assert_grep() {
  local desc="$1" file="$2" pattern="$3"
  if [[ ! -f "$file" ]]; then
    dev_warn "$desc: evidence file not captured ($file)"
  elif grep -qE -- "$pattern" "$file"; then
    dev_ok "$desc"
  else
    dev_fail "$desc (no match for '$pattern' in $file)"
  fi
}

# --- arguments (one parser, so a new script cannot invent a fifth spelling) --
# Common flags are consumed; anything else lands in DEV_EXTRA for the item
# script to interpret. --help sets DEV_HELP=1.
DEV_EXTRA=()
DEV_HELP=0
dev_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --apk)          SCM_APK="$2"; shift 2 ;;
      --expected-sha) SCM_EXPECTED_APK_SHA="$2"; shift 2 ;;
      --serial)       SCM_DEVICE_SERIAL="$2"; shift 2 ;;
      --api)          SCM_API="$2"; shift 2 ;;
      --cloud)        SCM_CLOUD_ADDR="$2"; shift 2 ;;
      --peer)         DEV_PHONE_PEER="$2"; shift 2 ;;
      --run-root)     SCM_DEVICE_RUN_ROOT="$2"; shift 2 ;;
      --confirmed)    SCM_OPERATOR_CONFIRMED=1; shift ;;
      -h|--help)      DEV_HELP=1; shift ;;
      *)              DEV_EXTRA+=("$1"); shift ;;
    esac
  done
  export SCM_API SCM_APK SCM_EXPECTED_APK_SHA SCM_DEVICE_SERIAL SCM_CLOUD_ADDR DEV_PHONE_PEER
}

# --- run directory (contract element 3) --------------------------------------
dev_init() {
  local item="$1" title="$2"
  DEV_ITEM="$item"
  DEV_RUN_DIR="${SCM_DEVICE_RUN_ROOT:-tmp/device}/${item}-$(date -u +%Y%m%dT%H%M%SZ)"
  mkdir -p "$DEV_RUN_DIR"
  export DEV_RUN_DIR DEV_ITEM

  echo "==============================================================================="
  echo "  $item -- $title"
  echo "  run dir: $DEV_RUN_DIR"
  echo "  api:     $SCM_API    (Windows/CLI node, actively driven)"
  echo "  handset: passive only -- install, app start/stop, read-only pulls"
  echo "==============================================================================="
  printf 'item=%s\nstarted_utc=%s\napi=%s\ncommit=%s\n' \
    "$item" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$SCM_API" \
    "$(git rev-parse --short HEAD 2>/dev/null || echo unknown)" > "$DEV_RUN_DIR/run.env"
}

# dev_evidence_bar "<what counts as proof>" -- recorded once, not re-argued later.
dev_evidence_bar() {
  printf 'evidence_bar=%s\n' "$*" >> "$DEV_RUN_DIR/run.env"
  dev_info "evidence bar: $*"
}

# dev_disposition "<ticket-or-none>" "<blocks|does-not-block the v0.4.0 tag>"
# Contract element 4: the failure path is decided before the run, not after.
dev_disposition() {
  printf 'disposition=%s\nblocks_tag=%s\n' "$1" "$2" >> "$DEV_RUN_DIR/run.env"
  dev_info "on failure: ticket $1; $2"
}

# dev_capture_recipe "<RUST_LOG>" "<logcat filter>" "<capture window>"
dev_capture_recipe() {
  {
    printf 'capture_recipe_rust_log=%s\n' "$1"
    printf 'capture_recipe_logcat_filter=%s\n' "$2"
    printf 'capture_recipe_window=%s\n' "$3"
    printf 'capture_recipe_artifacts=%s\n' "$DEV_RUN_DIR"
  } >> "$DEV_RUN_DIR/run.env"
  dev_info "capture: RUST_LOG='$1' logcat='$2' window='$3'"
}

# --- adb access --------------------------------------------------------------
dev_adb() {
  if [[ -n "$SCM_DEVICE_SERIAL" ]]; then adb -s "$SCM_DEVICE_SERIAL" "$@"; else adb "$@"; fi
}

dev_device_count() { adb devices 2>/dev/null | awk 'NR>1 && $2=="device"' | wc -l | tr -d ' '; }

# Precondition: adb present and exactly one handset usable. Fails with the
# literal "[FAIL] no device" so acceptance 2 of T7 is observable.
dev_require_adb() {
  if ! command -v adb >/dev/null 2>&1; then
    dev_fail "no device: adb is not on PATH; install Android platform-tools"
    return 1
  fi
  local count raw
  count="$(dev_device_count)"
  if [[ "$count" -eq 0 ]]; then
    raw="$(adb devices 2>/dev/null | awk 'NR>1 && NF' | tr '\n' ' ')"
    dev_fail "no device: adb reports no handset in 'device' state (attached: ${raw:-none}); run: adb devices"
    return 1
  fi
  if [[ "$count" -gt 1 && -z "$SCM_DEVICE_SERIAL" ]]; then
    dev_fail "no device: $count devices attached and SCM_DEVICE_SERIAL is unset, so the target is ambiguous; run: SCM_DEVICE_SERIAL=<serial> $0"
    return 1
  fi
  dev_ok "handset visible to adb ($count attached${SCM_DEVICE_SERIAL:+, pinned to $SCM_DEVICE_SERIAL})"
}

# Precondition: the installed APK is the intended build (T7 script contract).
dev_require_apk_identity() {
  local want="" source=""
  if [[ -n "$SCM_EXPECTED_APK_SHA" ]]; then
    want="$SCM_EXPECTED_APK_SHA"; source="SCM_EXPECTED_APK_SHA"
  elif [[ -f "$SCM_APK" ]]; then
    want="$(sha256sum "$SCM_APK" | awk '{print $1}')"; source="$SCM_APK"
  else
    dev_fail "installed APK identity unproven: neither SCM_APK ($SCM_APK) nor SCM_EXPECTED_APK_SHA is available; pass --apk <path> or set SCM_EXPECTED_APK_SHA"
    return 1
  fi

  local installed_path got
  installed_path="$(dev_adb shell pm path "$SCM_ANDROID_PACKAGE" 2>/dev/null | tr -d '\r' | sed -n 's/^package://p' | head -1)"
  if [[ -z "$installed_path" ]]; then
    dev_fail "$SCM_ANDROID_PACKAGE is not installed; run: adb -s <serial> install -r $SCM_APK"
    return 1
  fi
  got="$(dev_adb exec-out cat "$installed_path" 2>/dev/null | sha256sum | awk '{print $1}')"

  printf 'installed_apk_path=%s\nintended_apk_source=%s\nintended_apk_sha256=%s\ninstalled_apk_sha256=%s\n' \
    "$installed_path" "$source" "$want" "$got" >> "$DEV_RUN_DIR/run.env"

  if [[ "$want" == "$got" ]]; then
    dev_ok "installed APK is the intended build (sha256 ${got:0:16}...)"
  else
    dev_fail "installed APK sha256 ${got:0:16}... != intended ${want:0:16}... (from $source); reinstall first: adb -s <serial> install -r $SCM_APK"
  fi
}

# Precondition: debug-vs-release signing (T7 Step 4). The fleet runs a
# debug-signed build while D4/D6/D7 must be scored on the RELEASED APK, so a
# scored run needs the release signer on the device. Without a known release
# certificate digest this can only be reported, not asserted.
dev_require_signature() {
  local sig
  sig="$(dev_adb shell dumpsys package "$SCM_ANDROID_PACKAGE" 2>/dev/null \
    | tr -d '\r' | grep -A2 -i 'signatures\|SigningCertificate' | grep -oE '[0-9a-fA-F]{32,}' | head -1)"
  printf 'installed_signer_digest=%s\n' "${sig:-unknown}" >> "$DEV_RUN_DIR/run.env"

  if [[ -z "$sig" ]]; then
    dev_warn "installed APK signer could not be read; run: adb -s <serial> shell dumpsys package $SCM_ANDROID_PACKAGE | grep -i signature"
  elif [[ -z "$SCM_RELEASE_CERT_SHA256" ]]; then
    dev_warn "installed signer ${sig:0:16}... is not identified as release or debug (SCM_RELEASE_CERT_SHA256 unset), so a scored run is UNVERIFIED as release-signed; see docs/ANDROID_RELEASE_SIGNING.md"
  elif [[ "$sig" == "$SCM_RELEASE_CERT_SHA256" ]]; then
    dev_ok "installed APK is release-signed (signer ${sig:0:16}...)"
  else
    dev_fail "installed APK is NOT release-signed (signer ${sig:0:16}... != release ${SCM_RELEASE_CERT_SHA256:0:16}...); a scored run requires the released APK -- docs/ANDROID_RELEASE_SIGNING.md"
  fi
}

# Precondition: a physical handset, not the emulator. _QUEUE.md lets [DEVICE]
# tasks fall back to the AVD, but radio paths (BLE, WiFi Direct, offline
# proximity) cannot be proven there.
dev_require_physical() {
  local model
  model="$(dev_adb shell getprop ro.product.model 2>/dev/null | tr -d '\r')"
  printf 'device_model=%s\n' "${model:-unknown}" >> "$DEV_RUN_DIR/run.env"
  if [[ "$model" == *sdk* || "$model" == *mulator* ]]; then
    dev_fail "radio path needs a physical handset; attached device is the emulator ('${model:-unknown}')"
    return 1
  fi
  dev_ok "physical handset attached (${model:-unknown})"
}

# Precondition: the Windows/CLI node is up and answers its own API.
dev_require_cli_node() {
  local health version
  health="$(curl -fsS --max-time 5 "$SCM_API/health" 2>/dev/null)"
  if [[ -z "$health" ]]; then
    dev_fail "CLI node not answering at $SCM_API/health; start it: scmessenger-cli start (SCM_API points at the :9876 API)"
    return 1
  fi
  version="$(curl -fsS --max-time 5 "$SCM_API/version" 2>/dev/null)"
  printf '%s\n' "$version" > "$DEV_RUN_DIR/cli_version.json"
  DEV_CLI_GIT_HASH="$(printf '%s' "$version" | sed -n 's/.*"git_hash":"\([^"]*\)".*/\1/p')"
  dev_ok "CLI node healthy at $SCM_API (git_hash ${DEV_CLI_GIT_HASH:-unknown})"
}

# Precondition: both sides run the same build. P1-04 downstream rule: every
# cross-device session starts by comparing stamps.
dev_require_stamp_parity() {
  if [[ -z "${DEV_CLI_GIT_HASH:-}" || "$DEV_CLI_GIT_HASH" == "unknown" ]]; then
    dev_warn "build stamps not compared: the CLI did not report git_hash; run: curl -s $SCM_API/version"
    return 0
  fi
  dev_adb logcat -d > "$DEV_RUN_DIR/stamp_logcat.txt" 2>/dev/null
  if grep -qF "$DEV_CLI_GIT_HASH" "$DEV_RUN_DIR/stamp_logcat.txt" 2>/dev/null; then
    dev_ok "build stamps match: handset logcat carries ${DEV_CLI_GIT_HASH:0:12}"
  else
    dev_warn "handset build stamp not found in logcat (CLI is ${DEV_CLI_GIT_HASH:0:12}), so a cross-device result cannot be attributed to one build; run: adb -s <serial> logcat -d | grep -i git"
  fi
}

# Precondition: the cloud node address, when this item's scored path needs it.
dev_require_cloud_node() {
  if [[ -z "$SCM_CLOUD_ADDR" ]]; then
    dev_skip "cloud node address not supplied (SCM_CLOUD_ADDR unset); cloud leg not exercised"
    return 1
  fi
  local ip="${SCM_CLOUD_ADDR%%:*}"
  if ping -n 1 -w 2000 "$ip" >/dev/null 2>&1 || ping -c 1 -W 2 "$ip" >/dev/null 2>&1; then
    dev_ok "cloud node $SCM_CLOUD_ADDR reachable"
  else
    dev_warn "cloud node $SCM_CLOUD_ADDR did not answer ping (it may still answer on its node port); continuing"
  fi
}

# --- passive handset evidence (read-only, per the Pixel runbook) -------------
dev_logcat_clear() {
  dev_adb logcat -c >/dev/null 2>&1 && dev_info "logcat buffer cleared; capture window opens now" \
    || dev_warn "logcat buffer not cleared; evidence may predate this run"
}

dev_pull_phone_evidence() {
  local out="$DEV_RUN_DIR"
  dev_adb logcat -d -v time > "$out/logcat.txt" 2>/dev/null \
    && dev_ok "logcat captured ($(wc -l < "$out/logcat.txt" | tr -d ' ') lines)" \
    || dev_warn "logcat capture failed; run: adb -s <serial> logcat -d -v time"

  local mesh_raw="$out/mesh.log.raw"
  if dev_adb exec-out run-as "$SCM_ANDROID_PACKAGE" cat files/logs/scmessenger-mesh.log > "$mesh_raw" 2>/dev/null \
     && [[ -s "$mesh_raw" ]]; then
    # mesh.log is often UTF-16 (FF FE): decode before grepping, always.
    if head -c2 "$mesh_raw" | od -An -tx1 | grep -qi 'ff fe'; then
      if command -v iconv >/dev/null 2>&1; then
        iconv -f UTF-16LE -t UTF-8 "$mesh_raw" > "$out/mesh.log" 2>/dev/null || tr -d '\0' < "$mesh_raw" > "$out/mesh.log"
      else
        tr -d '\0' < "$mesh_raw" > "$out/mesh.log"
      fi
    else
      cp "$mesh_raw" "$out/mesh.log"
    fi
    dev_ok "phone mesh log captured ($(wc -l < "$out/mesh.log" | tr -d ' ') lines; UTF-16 decoded when needed)"
  else
    dev_warn "phone mesh log not captured; run: adb -s <serial> exec-out run-as $SCM_ANDROID_PACKAGE cat files/logs/scmessenger-mesh.log"
  fi

  dev_adb exec-out run-as "$SCM_ANDROID_PACKAGE" cat files/mesh_diagnostics.log > "$out/diag.log" 2>/dev/null \
    && dev_ok "phone diagnostics log captured" \
    || dev_warn "phone diagnostics log not captured; run: adb -s <serial> exec-out run-as $SCM_ANDROID_PACKAGE cat files/mesh_diagnostics.log"

  dev_adb exec-out run-as "$SCM_ANDROID_PACKAGE" cat files/ledger.json > "$out/phone_ledger.json" 2>/dev/null \
    && dev_ok "phone ledger captured (durable peer/address store)" \
    || dev_warn "phone ledger not captured; run: adb -s <serial> exec-out run-as $SCM_ANDROID_PACKAGE cat files/ledger.json"
}

# --- operator gate (the human path is the operator's, never the seat's) ------
dev_operator_step() {
  printf 'operator_step=%s\n' "$1" >> "$DEV_RUN_DIR/run.env"
  echo
  echo "-------------------------------------------------------------------------------"
  echo "  OPERATOR: $1"
  echo "-------------------------------------------------------------------------------"
  if [[ "$SCM_OPERATOR_CONFIRMED" == "1" ]]; then
    dev_info "operator step not awaited (SCM_OPERATOR_CONFIRMED=1)"
    return 0
  fi
  if [[ -t 0 ]]; then
    read -r -p "  press Enter once done (Ctrl-C to abort): " _ || true
  else
    dev_warn "stdin is not a terminal, so the operator step could not be confirmed; re-run interactively or set SCM_OPERATOR_CONFIRMED=1 after doing it"
  fi
}

# --- Windows/CLI-side drive (the authorized active side) ---------------------
dev_api_get() {  # dev_api_get <path> <outfile>
  curl -fsS --max-time 30 "$SCM_API$1" > "$2" 2>/dev/null
}

DEV_LAST_SEND=""
dev_drive_send() {  # dev_drive_send <recipient> <text> -> writes one JSON file
  local recipient="$1" text="$2"
  local out="$DEV_RUN_DIR/send_$(date -u +%H%M%S%N).json"
  curl -fsS --max-time 30 -X POST "$SCM_API/api/send" \
    -H 'Content-Type: application/json' \
    -d "$(printf '{"recipient":"%s","message":"%s"}' "$recipient" "$text")" > "$out" 2>/dev/null
  DEV_LAST_SEND="$out"
  printf '%s' "$out"
}

# dev_json_field <file> <dotted-key> -- emits the value, or nothing when the key
# is absent. An absent key is unmeasured, never a placeholder.
dev_json_field() {
  local file="$1" key="$2"
  [[ -s "$file" ]] || return 0
  command -v python3 >/dev/null 2>&1 || return 0
  python3 - "$file" "$key" <<'PY' 2>/dev/null
import json, sys
try:
    with open(sys.argv[1], encoding="utf-8", errors="replace") as fh:
        data = json.load(fh)
except Exception:
    sys.exit(0)
cur = data
for part in sys.argv[2].split("."):
    if isinstance(cur, list):
        try:
            cur = cur[int(part)]
        except Exception:
            sys.exit(0)
    elif isinstance(cur, dict):
        if part not in cur:
            sys.exit(0)
        cur = cur[part]
    else:
        sys.exit(0)
if cur is None:
    sys.exit(0)
print(cur if not isinstance(cur, (dict, list)) else json.dumps(cur))
PY
}

# --- polling / reachability --------------------------------------------------
# dev_poll_until <timeout_s> <interval_s> <label> <cmd...>
dev_poll_until() {
  local timeout="$1" interval="$2" label="$3"; shift 3
  local waited=0
  while [[ "$waited" -lt "$timeout" ]]; do
    if "$@" >/dev/null 2>&1; then
      log_poll_ok "$label" "$waited"
      return 0
    fi
    sleep "$interval"
    waited=$((waited + interval))
  done
  return 1
}
log_poll_ok() { dev_info "$1 satisfied after ${2}s"; }

# dev_tcp_probe <host> <port> <timeout_s> -- 0 when a TCP connection is accepted.
dev_tcp_probe() {
  local host="$1" port="$2" tmo="${3:-4}"
  if command -v timeout >/dev/null 2>&1; then
    timeout "$tmo" bash -c "exec 3<>/dev/tcp/$host/$port" >/dev/null 2>&1
  else
    (exec 3<>"/dev/tcp/$host/$port") >/dev/null 2>&1
  fi
}

# --- summary -----------------------------------------------------------------
dev_summary() {
  local window="${1:-}"
  echo
  echo "==============================================================================="
  printf '  %s: %d ok / %d fail / %d warning / %d skip%s\n' \
    "$DEV_ITEM" "$DEV_OKS" "$DEV_FAILS" "$DEV_WARNS" "$DEV_SKIPS" "${window:+  ($window)}"
  echo "  artifacts: $DEV_RUN_DIR"
  echo "==============================================================================="
  printf 'summary ok=%d fail=%d warning=%d skip=%d\n' \
    "$DEV_OKS" "$DEV_FAILS" "$DEV_WARNS" "$DEV_SKIPS" >> "$DEV_RUN_DIR/verdicts.log"
  if [[ "$DEV_FAILS" -gt 0 ]]; then
    echo "  verdict: FAIL -- this item is not verified"
    return 1
  fi
  if [[ "$DEV_WARNS" -gt 0 || "$DEV_SKIPS" -gt 0 ]]; then
    echo "  verdict: PARTIAL -- nothing failed, but not every assertion could be evaluated"
    return 0
  fi
  echo "  verdict: PASS"
  return 0
}
