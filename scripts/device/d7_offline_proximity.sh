#!/usr/bin/env bash
# =============================================================================
# scripts/device/d7_offline_proximity.sh
#
# SHIP_PLAN.md G3, exit criterion D7: "Offline proximity messaging demonstrated
# -- the Android handset and the Windows node exchanging a message with no
# internet available."
#
# Evidence bar, verbatim from G3: receiver-side decrypt + durable history +
# receipt -- NOT transport ACKs, NOT UI counters, NOT BLE local acceptance.
# Whether the radios are "connected" is not the test; the message arriving and
# being decrypted on the far side is.
#
# "No internet" is asserted, not assumed: the script proves both sides fail to
# reach a public address, and that the CLI node still answers its own API, so a
# delivery here cannot have used a wide-area path.
#
# Seat split (docs/rules/ANDROID.md): driving is on the Windows/CLI side; the
# handset is passive and every radio/airplane-mode change is the operator's.
#
# Usage:
#   scripts/device/d7_offline_proximity.sh [--apk <path>] [--peer <phone-peer-id>]
#                                          [--serial <serial>] [--api <url>]
# =============================================================================
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

ITEM="d7_offline_proximity"
TITLE="D7: handset <-> Windows node message exchange with no internet available"

dev_args "$@"
if [[ "$DEV_HELP" == "1" ]]; then sed -n '3,22p' "$0"; exit 0; fi
dev_init "$ITEM" "$TITLE"
dev_evidence_bar "receiver-side decrypt + durable history + receipt with public reachability proven absent on both sides"
dev_disposition "a new HANDOFF/todo/ ticket built from the captured artifacts" "blocks the v0.4.0 tag (D7 is a G3 exit criterion, CP6)"
dev_capture_recipe "info,scmessenger=debug,scmessenger_core::transport=debug" "logcat from clear to pull; phone files/logs/scmessenger-mesh.log" "clear -> operator sets offline -> both directions -> pull"

internet_reachable_windows() {
  ping -n 1 -w 2000 1.1.1.1 >/dev/null 2>&1 || ping -c 1 -W 2 1.1.1.1 >/dev/null 2>&1
}
internet_reachable_phone() {
  dev_adb shell ping -c 1 -W 2 1.1.1.1 >/dev/null 2>&1
}

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

# --- step 1: the operator creates the offline state --------------------------
dev_logcat_clear
dev_operator_step "put the handset in airplane mode, then switch Bluetooth ON, leave Wi-Fi OFF, and leave the app in the foreground"
dev_operator_step "disconnect the Windows box's internet uplink (keep its local network adapter and the SCMessenger CLI node running)"

# --- step 2: assert the offline state instead of trusting it -----------------
if internet_reachable_windows; then
  dev_fail "the Windows box can still reach the public internet, so a delivery here would not prove offline proximity"
else
  dev_ok "Windows box cannot reach the public internet (ping 1.1.1.1 fails)"
fi
if internet_reachable_phone; then
  dev_fail "the handset can still reach the public internet, so a delivery here would not prove offline proximity"
else
  dev_ok "handset cannot reach the public internet (adb shell ping 1.1.1.1 fails)"
fi

WIFI_ON="$(dev_adb shell settings get global wifi_on 2>/dev/null | tr -d '\r')"
BT_ON="$(dev_adb shell settings get global bluetooth_on 2>/dev/null | tr -d '\r')"
printf 'phone_wifi_on=%s\nphone_bluetooth_on=%s\n' "${WIFI_ON:-unknown}" "${BT_ON:-unknown}" >> "$DEV_RUN_DIR/run.env"
if [[ "$BT_ON" == "1" ]]; then
  dev_ok "handset Bluetooth radio is on (the proximity path is available)"
elif [[ "$BT_ON" == "0" ]]; then
  dev_fail "handset Bluetooth is off, so no proximity transport is available; turn it on and re-run"
else
  dev_warn "handset Bluetooth state unreadable; run: adb -s <serial> shell settings get global bluetooth_on"
fi
if [[ "$WIFI_ON" == "1" ]]; then
  dev_warn "handset Wi-Fi is on; that is a LAN path, not necessarily offline proximity -- Wi-Fi without internet is still no-internet, so this is recorded rather than failed"
fi

# --- step 3: both directions, offline ---------------------------------------
TOKEN_W2P="scm-d7-w2p-$(date -u +%H%M%S)"
TOKEN_P2W="scm-d7-p2w-$(date -u +%H%M%S)"
dev_info "tokens for this run: handset-receives=$TOKEN_W2P handset-sends=$TOKEN_P2W"

SEND_FILE="$(dev_drive_send "$DEV_PHONE_PEER" "$TOKEN_W2P")"
MSG_ID="$(dev_json_field "$SEND_FILE" "message_id")"
if [[ -n "$MSG_ID" ]]; then
  dev_ok "Windows node accepted the offline send to the handset (message_id ${MSG_ID:0:12}...)"
else
  dev_fail "Windows node refused the offline send: $(tr -d '\n' < "$SEND_FILE" 2>/dev/null | head -c 200)"
fi

dev_operator_step "confirm that '$TOKEN_W2P' arrived on the handset, then send exactly this text back to the Windows node: $TOKEN_P2W"

dev_pull_phone_evidence

dev_assert_grep "handset decrypted the offline message (receiver-side plaintext token)" \
  "$DEV_RUN_DIR/mesh.log" "$TOKEN_W2P"
dev_assert_grep "handset kept the offline message in durable history (files/ledger.json)" \
  "$DEV_RUN_DIR/phone_ledger.json" "$TOKEN_W2P"

dev_api_get "/api/send/${MSG_ID}" "$DEV_RUN_DIR/send_status.json"
DELIVERED="$(dev_json_field "$DEV_RUN_DIR/send_status.json" "delivered")"
if [[ "$DELIVERED" == "True" || "$DELIVERED" == "true" ]]; then
  dev_ok "receipt recorded on the Windows node for the offline delivery (delivered=true)"
else
  dev_fail "no receipt for the offline delivery (delivered=${DELIVERED:-absent})"
fi

dev_api_get "/api/history?limit=50" "$DEV_RUN_DIR/cli_history.json"
dev_assert_grep "Windows node decrypted and stored the handset's offline message (receiver-side durable history)" \
  "$DEV_RUN_DIR/cli_history.json" "$TOKEN_P2W"

if grep -qEi 'ble|bluetooth|proximity|l2cap|gatt' "$DEV_RUN_DIR/mesh.log" 2>/dev/null; then
  dev_ok "handset log names a proximity transport for the exchange"
else
  dev_warn "handset log does not name the proximity transport, so the path taken is unproven even though delivery succeeded; run: adb -s <serial> logcat -d | grep -i -E 'ble|bluetooth|proximity'"
fi

dev_summary "D7 offline, both directions"
