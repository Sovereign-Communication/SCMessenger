#!/usr/bin/env bash
# =============================================================================
# scripts/device/p1_09_lan_e2e.sh
#
# HANDOFF/V1_0_0_EXECUTION_PLAN.md P1-09 [DEVICE]: "LAN E2E validation pass
# (mDNS + TCP + WS). Scripted playbook: fresh CLI daemon + fresh app cold start,
# both directions: discovery, dial, ConnectionEstablished, E2E message + receipt,
# then kill-and-recover (restart one side, confirm re-discovery). Two full
# reproducible passes. Evidence to ledger."
#
# Evidence bar: receiver-side decrypt + durable history + receipt. A connected
# transport is not a result.
#
# Seat split (docs/rules/ANDROID.md + the Pixel runbook): the CLI side is driven
# here; on the handset this script may only start/stop the app process and pull
# logs. Every human path on the phone is the operator's, prompted below.
#
# Usage:
#   scripts/device/p1_09_lan_e2e.sh [--apk <path>] [--peer <phone-peer-id>]
#                                    [--serial <serial>] [--passes 2]
# =============================================================================
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

ITEM="p1_09_lan_e2e"
TITLE="P1-09: LAN E2E (discovery, dial, message + receipt, kill-and-recover), reproducible passes"

dev_args "$@"

PASSES=2
while [[ ${#DEV_EXTRA[@]} -gt 0 ]]; do
  case "${DEV_EXTRA[0]}" in
    --passes) PASSES="${DEV_EXTRA[1]:-2}"; DEV_EXTRA=("${DEV_EXTRA[@]:2}") ;;
    *) echo "unknown argument: ${DEV_EXTRA[0]}" >&2; exit 2 ;;
  esac
done

if [[ "$DEV_HELP" == "1" ]]; then sed -n '3,20p' "$0"; exit 0; fi
dev_init "$ITEM" "$TITLE"
dev_evidence_bar "receiver-side decrypt + durable history + receipt, per direction, per pass, including after a restart"
dev_disposition "a new HANDOFF/todo/ ticket built from the captured artifacts" "does not block the v0.4.0 tag; it is post-exit Phase 1 verification debt (HANDOFF/todo/_QUEUE.md:333)"
dev_capture_recipe "info,scmessenger=debug,scmessenger_core::transport=debug,libp2p_mdns=debug" "logcat from clear to pull; CLI /api/connection-path-state" "clear -> per-pass actions -> pull; repeated per pass"

# --- preconditions -----------------------------------------------------------
dev_require_adb || { dev_summary "aborted: no device"; exit 1; }
PRECONDITIONS_OK=1
dev_require_physical || PRECONDITIONS_OK=0
dev_require_apk_identity || PRECONDITIONS_OK=0
dev_require_signature
dev_require_cli_node || PRECONDITIONS_OK=0
dev_require_stamp_parity
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
dev_info "running $PASSES pass(es); the pass count is the plan's two-full-passes requirement"

path_connected() {
  dev_api_get /api/connection-path-state "$DEV_RUN_DIR/path_state.json"
  [[ "$(dev_json_field "$DEV_RUN_DIR/path_state.json" "state")" == "DirectPreferred" ]]
}

for ((pass = 1; pass <= PASSES; pass++)); do
  echo
  dev_info "===== pass $pass of $PASSES ====="
  dev_logcat_clear
  dev_operator_step "pass $pass: bring the handset app to the foreground (cold start if it was not running)"

  # discovery + dial + ConnectionEstablished
  if dev_poll_until "${SCM_LAN_WINDOW_SECS:-180}" 15 "pass $pass: LAN path established (discovery -> dial -> connected)" path_connected; then
    dev_ok "pass $pass: LAN path established without manual seeding"
  else
    dev_fail "pass $pass: no LAN path within the window (state=$(dev_json_field "$DEV_RUN_DIR/path_state.json" state))"
  fi

  TOKEN_W2P="scm-p109-${pass}-w2p-$(date -u +%H%M%S)"
  TOKEN_P2W="scm-p109-${pass}-p2w-$(date -u +%H%M%S)"

  SEND_FILE="$(dev_drive_send "$DEV_PHONE_PEER" "$TOKEN_W2P")"
  MSG_ID="$(dev_json_field "$SEND_FILE" "message_id")"
  if [[ -n "$MSG_ID" ]]; then
    dev_ok "pass $pass: Windows node accepted the LAN send (message_id ${MSG_ID:0:12}...)"
  else
    dev_fail "pass $pass: Windows node refused the LAN send: $(tr -d '\n' < "$SEND_FILE" 2>/dev/null | head -c 200)"
  fi

  dev_operator_step "pass $pass: confirm '$TOKEN_W2P' arrived, then send this text back to the Windows node: $TOKEN_P2W"

  dev_pull_phone_evidence
  dev_assert_grep "pass $pass: handset decrypted the LAN message (receiver-side token)" "$DEV_RUN_DIR/mesh.log" "$TOKEN_W2P"
  dev_assert_grep "pass $pass: handset kept it in durable history (ledger.json)" "$DEV_RUN_DIR/phone_ledger.json" "$TOKEN_W2P"

  dev_api_get "/api/send/${MSG_ID}" "$DEV_RUN_DIR/send_status.json"
  DELIVERED="$(dev_json_field "$DEV_RUN_DIR/send_status.json" "delivered")"
  if [[ "$DELIVERED" == "True" || "$DELIVERED" == "true" ]]; then
    dev_ok "pass $pass: receipt recorded on the Windows node"
  else
    dev_fail "pass $pass: no receipt (delivered=${DELIVERED:-absent})"
  fi

  dev_api_get "/api/history?limit=50" "$DEV_RUN_DIR/cli_history.json"
  dev_assert_grep "pass $pass: Windows node decrypted and stored the handset's message (receiver-side history)" \
    "$DEV_RUN_DIR/cli_history.json" "$TOKEN_P2W"

  # kill-and-recover: force-stop + start is seat-authorized; the human path is not touched
  dev_info "pass $pass: kill-and-recover -- force-stopping and relaunching the handset app process"
  dev_adb shell am force-stop "$SCM_ANDROID_PACKAGE" >/dev/null 2>&1 \
    && dev_ok "pass $pass: app force-stopped" \
    || dev_warn "pass $pass: force-stop failed; run: adb -s <serial> shell am force-stop $SCM_ANDROID_PACKAGE"
  dev_adb shell am start -n "$SCM_ANDROID_PACKAGE/.ui.MainActivity" >/dev/null 2>&1 \
    && dev_ok "pass $pass: app relaunched" \
    || dev_warn "pass $pass: relaunch failed; run: adb -s <serial> shell am start -n $SCM_ANDROID_PACKAGE/.ui.MainActivity"

  if dev_poll_until "${SCM_LAN_WINDOW_SECS:-180}" 15 "pass $pass: re-discovery after the restart" path_connected; then
    dev_ok "pass $pass: re-discovery after restart"
  else
    dev_fail "pass $pass: path did not recover after the restart"
  fi

  TOKEN_AFTER="scm-p109-${pass}-after-$(date -u +%H%M%S)"
  SEND_FILE="$(dev_drive_send "$DEV_PHONE_PEER" "$TOKEN_AFTER")"
  MSG_ID="$(dev_json_field "$SEND_FILE" "message_id")"
  dev_operator_step "pass $pass: confirm '$TOKEN_AFTER' arrived after the restart"
  dev_pull_phone_evidence
  dev_assert_grep "pass $pass: message delivered after the restart (receiver-side token)" "$DEV_RUN_DIR/mesh.log" "$TOKEN_AFTER"
  if [[ -n "$MSG_ID" ]]; then
    dev_ok "pass $pass: post-restart send was accepted (message_id ${MSG_ID:0:12}...)"
  else
    dev_fail "pass $pass: post-restart send was refused"
  fi
done

dev_summary "P1-09, $PASSES pass(es)"
