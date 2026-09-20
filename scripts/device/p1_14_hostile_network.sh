#!/usr/bin/env bash
# =============================================================================
# scripts/device/p1_14_hostile_network.sh
#
# HANDOFF/V1_0_0_EXECUTION_PLAN.md P1-14 [DEVICE]: "Firewall profiles on the
# Windows box (block 9001/9002 inbound; then allow only 443; then only 80),
# phone on hotspot/cellular vs LAN; assert delivery still lands via the ladder
# each time, and that second contact is fast (memory works)."
#
# This is the literal form of the plan's "if 443 gets through, use 443"
# acceptance test. Each rung asserts the ACTUAL delivery (receiver-side decrypt,
# durable history, receipt) and the latency of the first vs second contact.
#
# Dependency, stated up front so the operator is not surprised: the ladder
# itself is P1-11/P1-12 work. If only one listen port is reported this script
# says so and the rungs are expected to fail -- that is the pre-registered
# outcome, not a surprise on device day.
#
# Seat split (docs/rules/ANDROID.md): firewall changes and all driving are on the
# Windows/CLI side, by the operator; the handset is passive.
#
# Usage:
#   scripts/device/p1_14_hostile_network.sh [--apk <path>] [--peer <phone-peer-id>]
#                                           [--serial <serial>] [--api <url>]
# =============================================================================
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

ITEM="p1_14_hostile_network"
TITLE="P1-14: hostile-network port ladder -- delivery and remembered path at each firewall rung"

dev_args "$@"
if [[ "$DEV_HELP" == "1" ]]; then sed -n '3,22p' "$0"; exit 0; fi
dev_init "$ITEM" "$TITLE"
dev_evidence_bar "receiver-side decrypt + durable history + receipt at every rung, plus first-vs-second contact latency"
dev_disposition "a new HANDOFF/todo/ ticket built from the captured artifacts" "does not block the v0.4.0 tag; post-exit Phase 1 verification debt (HANDOFF/todo/_QUEUE.md:333)"
dev_capture_recipe "info,scmessenger=debug,scmessenger_core::transport=debug" "CLI /api/listeners + /api/send polling; phone logcat and mesh.log per rung" "per rung: apply -> first contact -> second contact -> pull"

CAP_SECS="${SCM_RUNG_CAP_SECS:-90}"

# send_and_time <recipient> <token> -> seconds until the node reports the receipt
send_and_time() {
  local recipient="$1" token="$2" file id waited=0
  file="$(dev_drive_send "$recipient" "$token")"
  id="$(dev_json_field "$file" "message_id")"
  if [[ -z "$id" ]]; then echo ""; return 1; fi
  while [[ "$waited" -lt "$CAP_SECS" ]]; do
    dev_api_get "/api/send/$id" "$DEV_RUN_DIR/latency_status.json"
    local delivered
    delivered="$(dev_json_field "$DEV_RUN_DIR/latency_status.json" "delivered")"
    if [[ "$delivered" == "True" || "$delivered" == "true" ]]; then echo "$waited"; return 0; fi
    sleep 1
    waited=$((waited + 1))
  done
  echo ""; return 1
}

# --- preconditions -----------------------------------------------------------
dev_require_adb || { dev_summary "aborted: no device"; exit 1; }
PRECONDITIONS_OK=1
dev_require_apk_identity || PRECONDITIONS_OK=0
dev_require_signature
dev_require_cli_node || PRECONDITIONS_OK=0
if [[ "$PRECONDITIONS_OK" != "1" ]]; then
  dev_summary "aborted: preconditions not met"
  exit 1
fi

if [[ -z "$DEV_PHONE_PEER" ]]; then
  dev_api_get /api/peers "$DEV_RUN_DIR/cli_peers.json"
  DEV_PHONE_PEER="$(dev_json_field "$DEV_RUN_DIR/cli_peers.json" "peers.0.peer_id")"
  [[ -z "$DEV_PHONE_PEER" ]] && DEV_PHONE_PEER="$(dev_json_field "$DEV_RUN_DIR/cli_peers.json" "peers.0")"
fi
if [[ -z "$DEV_PHONE_PEER" ]]; then
  dev_fail "handset peer id unknown: /api/peers reported no single peer; re-run with --peer <phone-peer-id>"
  dev_summary "aborted: target peer unknown"
  exit 1
fi

# --- does the ladder even exist on this build? ------------------------------
dev_api_get /api/listeners "$DEV_RUN_DIR/cli_listeners.json"
LISTEN_COUNT="$(python3 -c '
import json,sys
try:
    d=json.load(open(sys.argv[1]))
except Exception:
    print(0); raise SystemExit
if isinstance(d,dict):
    for k in ("listeners","addresses","listener_addresses"):
        if isinstance(d.get(k),list):
            print(len(d[k])); raise SystemExit
    print(0)
elif isinstance(d,list):
    print(len(d))
else:
    print(0)
' "$DEV_RUN_DIR/cli_listeners.json" 2>/dev/null)"
printf 'cli_listen_port_count=%s\n' "${LISTEN_COUNT:-unknown}" >> "$DEV_RUN_DIR/run.env"
if [[ "${LISTEN_COUNT:-0}" -gt 1 ]]; then
  dev_ok "CLI reports $LISTEN_COUNT listen ports, so a port ladder is present"
else
  dev_warn "CLI reports ${LISTEN_COUNT:-0} listen port(s); the ladder (P1-11) does not appear to be landed, so the rungs below are expected to fail -- this script fails closed rather than reporting a false pass"
fi

# --- rungs -------------------------------------------------------------------
rungs=(
  "block 9001 and 9002 inbound|9001|9002"
  "allow only 443 inbound|443|"
  "allow only 80 inbound|80|"
)

for rung in "${rungs[@]}"; do
  IFS='|' read -r label expect_port other_port <<< "$rung"
  echo
  dev_info "===== rung: $label ====="
  dev_operator_step "apply the Windows firewall profile for this rung: $label (leave the phone on its current network)"

  T1="scm-p114-$(date -u +%H%M%S)-a"
  T2="scm-p114-$(date -u +%H%M%S)-b"
  LAT1="$(send_and_time "$DEV_PHONE_PEER" "$T1")"
  if [[ -z "$LAT1" ]]; then
    dev_fail "rung '$label': first contact did not reach a receipt within ${CAP_SECS}s (ladder did not carry it)"
  else
    dev_ok "rung '$label': first contact delivered and receipted in ${LAT1}s"
  fi
  LAT2="$(send_and_time "$DEV_PHONE_PEER" "$T2")"
  if [[ -z "$LAT2" ]]; then
    dev_fail "rung '$label': second contact did not reach a receipt within ${CAP_SECS}s"
  else
    dev_ok "rung '$label': second contact delivered and receipted in ${LAT2}s"
  fi

  if [[ -n "$LAT1" && -n "$LAT2" ]]; then
    if (( LAT2 <= LAT1 )); then
      dev_ok "rung '$label': remembered path is not slower (${LAT2}s <= ${LAT1}s)"
    elif (( LAT1 <= 5 )); then
      dev_warn "rung '$label': both contacts were fast (${LAT1}s, ${LAT2}s), so path memory is not measurable at this scale"
    else
      dev_fail "rung '$label': second contact was slower (${LAT2}s > ${LAT1}s) after a ${LAT1}s first contact -- the remembered path was not used"
    fi
  fi

  dev_pull_phone_evidence
  dev_assert_grep "rung '$label': handset decrypted the message (receiver-side token)" "$DEV_RUN_DIR/mesh.log" "$T1"
  dev_assert_grep "rung '$label': handset kept it in durable history" "$DEV_RUN_DIR/phone_ledger.json" "$T1"
  if grep -qF "$expect_port" "$DEV_RUN_DIR/mesh.log" 2>/dev/null || grep -qF "$expect_port" "$DEV_RUN_DIR/logcat.txt" 2>/dev/null; then
    dev_ok "rung '$label': handset log names port $expect_port, so the rung's intended path is evidenced"
  else
    dev_warn "rung '$label': port $expect_port is not named in the captured logs, so delivery is proven but the path used is not; run: adb -s <serial> logcat -d | grep -F $expect_port"
  fi
  if [[ -n "${other_port:-}" ]] && grep -qE "$other_port" "$DEV_RUN_DIR/mesh.log" 2>/dev/null; then
    dev_warn "rung '$label': log still mentions blocked port $other_port -- confirm the block is actually in force"
  fi
done

dev_operator_step "restore the original Windows firewall profile"
dev_summary "P1-14 ladder complete"
