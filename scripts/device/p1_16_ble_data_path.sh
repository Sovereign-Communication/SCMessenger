#!/usr/bin/env bash
# =============================================================================
# scripts/device/p1_16_ble_data_path.sh
#
# HANDOFF/V1_0_0_EXECUTION_PLAN.md P1-16 [SONNET][AUDIT-GATE][DEVICE]: the BLE
# Android<->Windows data path, worst-case cell. The plan states the exit test
# literally: "WiFi off, no internet, both radios on -- message composed on phone
# arrives on Windows via BLE, and vice versa."
#
# Also carries P2_ANDROID_BLE_MAC_Rotation_Breaks_Session_Continuity: the Pixel
# rotates its Bluetooth MAC roughly every 15 minutes, so peripheral identity must
# key off the SCM service UUID / identity handshake rather than the MAC. That
# check needs a ~15-minute window, so it is opt-in via --mac-rotation.
#
# Pre-registered outcome: P1-15 (the transport-matrix audit) asks whether BLE is
# a real data path today or discovery-only. If delivery fails here, the failure
# IS the finding -- this script does not paper over it.
#
# Seat split (docs/rules/ANDROID.md): every radio change and all driving are the
# operator's on the Windows/CLI side; the handset is passive.
#
# Usage:
#   scripts/device/p1_16_ble_data_path.sh [--apk <path>] [--peer <phone-peer-id>]
#                                         [--serial <serial>] [--mac-rotation]
# =============================================================================
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

ITEM="p1_16_ble_data_path"
TITLE="P1-16: BLE Android<->Windows data path with WiFi off and no internet"

dev_args "$@"

MAC_ROTATION=0
while [[ ${#DEV_EXTRA[@]} -gt 0 ]]; do
  case "${DEV_EXTRA[0]}" in
    --mac-rotation) MAC_ROTATION=1; DEV_EXTRA=("${DEV_EXTRA[@]:1}") ;;
    *) echo "unknown argument: ${DEV_EXTRA[0]}" >&2; exit 2 ;;
  esac
done

if [[ "$DEV_HELP" == "1" ]]; then sed -n '3,24p' "$0"; exit 0; fi
dev_init "$ITEM" "$TITLE"
dev_evidence_bar "receiver-side decrypt + durable history + receipt over BLE, with WiFi off and no internet proven"
dev_disposition "P1-16 gap-closure ticket if delivery fails, else nothing" "does not block the v0.4.0 tag; it closes the BLE cell of the Phase 1 transport matrix (2.6)"
dev_capture_recipe "info,scmessenger=debug,scmessenger_core::transport=debug" "logcat from clear to pull; phone files/logs/scmessenger-mesh.log" "clear -> radios set -> both directions -> pull"

internet_reachable_phone() { dev_adb shell ping -c 1 -W 2 1.1.1.1 >/dev/null 2>&1; }

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

# --- step 1: the worst-case radio state, asserted not assumed ---------------
dev_logcat_clear
dev_operator_step "handset: Wi-Fi OFF, mobile data OFF, Bluetooth ON, app in the foreground; Windows: Bluetooth radio ON"

WIFI_ON="$(dev_adb shell settings get global wifi_on 2>/dev/null | tr -d '\r')"
BT_ON="$(dev_adb shell settings get global bluetooth_on 2>/dev/null | tr -d '\r')"
printf 'phone_wifi_on=%s\nphone_bluetooth_on=%s\n' "${WIFI_ON:-unknown}" "${BT_ON:-unknown}" >> "$DEV_RUN_DIR/run.env"

if [[ "$WIFI_ON" == "0" ]]; then
  dev_ok "handset Wi-Fi is off, so no LAN path exists"
elif [[ "$WIFI_ON" == "1" ]]; then
  dev_fail "handset Wi-Fi is on, so a delivery here could have taken the LAN path; the BLE cell would be unproven"
else
  dev_warn "handset Wi-Fi state unreadable; run: adb -s <serial> shell settings get global wifi_on"
fi
if [[ "$BT_ON" == "1" ]]; then
  dev_ok "handset Bluetooth is on"
elif [[ "$BT_ON" == "0" ]]; then
  dev_fail "handset Bluetooth is off, so the BLE path cannot exist"
else
  dev_warn "handset Bluetooth state unreadable; run: adb -s <serial> shell settings get global bluetooth_on"
fi
if internet_reachable_phone; then
  dev_fail "handset can still reach the public internet, so this is not the worst-case cell"
else
  dev_ok "handset cannot reach the public internet"
fi

# --- step 2: both directions over BLE ---------------------------------------
TOKEN_W2P="scm-p116-w2p-$(date -u +%H%M%S)"
TOKEN_P2W="scm-p116-p2w-$(date -u +%H%M%S)"

SEND_FILE="$(dev_drive_send "$DEV_PHONE_PEER" "$TOKEN_W2P")"
MSG_ID="$(dev_json_field "$SEND_FILE" "message_id")"
if [[ -n "$MSG_ID" ]]; then
  dev_ok "Windows node accepted the BLE-path send (message_id ${MSG_ID:0:12}...)"
else
  dev_fail "Windows node refused the send with WiFi off: $(tr -d '\n' < "$SEND_FILE" 2>/dev/null | head -c 200)"
fi

dev_operator_step "confirm '$TOKEN_W2P' arrived on the handset, then send this text back to the Windows node: $TOKEN_P2W"

dev_pull_phone_evidence

dev_assert_grep "handset decrypted the BLE message (receiver-side plaintext token)" "$DEV_RUN_DIR/mesh.log" "$TOKEN_W2P"
dev_assert_grep "handset kept it in durable history (ledger.json)" "$DEV_RUN_DIR/phone_ledger.json" "$TOKEN_W2P"

dev_api_get "/api/send/${MSG_ID}" "$DEV_RUN_DIR/send_status.json"
DELIVERED="$(dev_json_field "$DEV_RUN_DIR/send_status.json" "delivered")"
if [[ "$DELIVERED" == "True" || "$DELIVERED" == "true" ]]; then
  dev_ok "receipt recorded on the Windows node for the BLE-path delivery"
else
  dev_fail "no receipt for the BLE-path delivery (delivered=${DELIVERED:-absent}) -- per P1-15, record whether BLE is a data path or discovery-only"
fi

dev_api_get "/api/history?limit=50" "$DEV_RUN_DIR/cli_history.json"
dev_assert_grep "Windows node decrypted and stored the handset's BLE message (receiver-side history)" \
  "$DEV_RUN_DIR/cli_history.json" "$TOKEN_P2W"

if grep -qEi 'ble|bluetooth|gatt|l2cap' "$DEV_RUN_DIR/mesh.log" 2>/dev/null; then
  dev_ok "handset log names a BLE transport for the exchange"
else
  dev_warn "handset log does not name BLE, so the path is unproven even if delivery succeeded; run: adb -s <serial> logcat -d | grep -i -E 'ble|bluetooth|gatt'"
fi

# --- step 3 (opt-in): MAC-rotation continuity -------------------------------
if [[ "$MAC_ROTATION" != "1" ]]; then
  dev_skip "MAC-rotation continuity not exercised (needs roughly a 15-minute window); re-run with --mac-rotation"
else
  dev_operator_step "toggle the handset's Bluetooth OFF and back ON (forces the peripheral identity to be re-established)"
  dev_info "waiting out the rotation window before the continuity exchange"
  sleep "${SCM_MAC_ROTATION_SECS:-900}"

  TOKEN_AFTER="scm-p116-after-$(date -u +%H%M%S)"
  SEND_FILE="$(dev_drive_send "$DEV_PHONE_PEER" "$TOKEN_AFTER")"
  dev_operator_step "confirm '$TOKEN_AFTER' arrived after the Bluetooth toggle"
  dev_pull_phone_evidence
  dev_assert_grep "session survived the MAC rotation (message decrypted on the handset afterwards)" \
    "$DEV_RUN_DIR/mesh.log" "$TOKEN_AFTER"
  dev_assert_grep "post-rotation message also reached durable history" \
    "$DEV_RUN_DIR/phone_ledger.json" "$TOKEN_AFTER"
fi

dev_summary "P1-16 BLE worst-case cell"
