#!/usr/bin/env bash
# =============================================================================
# scripts/device/d6_transport_racing.sh
#
# SHIP_PLAN.md G3, exit criterion D6: "Transport racing demonstrated -- message
# delivered when first-choice transport is unavailable, proving fallback selects
# a working path."
#
# Evidence bar, verbatim from G3: receiver-side decrypt + durable history +
# receipt -- NOT transport ACKs, NOT UI counters, NOT BLE local acceptance.
#
# The first-choice transport is removed the way a hostile network removes it:
# the operator turns the handset's Wi-Fi off and leaves cellular + Bluetooth on,
# so the LAN/direct path dies while a wide-area path stays available. The script
# proves the removal actually happened before it accepts any delivery as
# fallback evidence.
#
# Seat split (docs/rules/ANDROID.md): driving is on the Windows/CLI side; the
# handset is passive and the radio toggle is the operator's.
#
# Usage:
#   scripts/device/d6_transport_racing.sh [--apk <path>] [--peer <phone-peer-id>]
#                                         [--serial <serial>] [--api <url>]
# =============================================================================
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

ITEM="d6_transport_racing"
TITLE="D6: delivery when the first-choice transport is unavailable (fallback proven)"

dev_args "$@"
if [[ "$DEV_HELP" == "1" ]]; then sed -n '3,21p' "$0"; exit 0; fi
dev_init "$ITEM" "$TITLE"
dev_evidence_bar "receiver-side decrypt + durable history + receipt, with the first-choice path proven gone first"
dev_disposition "a new HANDOFF/todo/ ticket built from the captured artifacts" "blocks the v0.4.0 tag (D6 is a G3 exit criterion, CP5)"
dev_capture_recipe "info,scmessenger=debug,scmessenger_core::transport=debug" "logcat from clear to pull; CLI /api/connection-path-state polled" "clear -> operator toggles Wi-Fi -> send -> pull"

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

# --- step 1: the first-choice path must exist BEFORE it is removed -----------
dev_api_get /api/connection-path-state "$DEV_RUN_DIR/path_before.json"
PATH_BEFORE="$(dev_json_field "$DEV_RUN_DIR/path_before.json" "state")"
if [[ -z "$PATH_BEFORE" ]]; then
  dev_fail "connection path state unreadable before the run; run: curl -s $SCM_API/api/connection-path-state"
elif [[ "$PATH_BEFORE" == "Bootstrapping" ]]; then
  dev_fail "no first-choice path exists before the test (state=$PATH_BEFORE), so fallback cannot be demonstrated; bring both nodes up and re-run"
else
  dev_ok "first-choice path established before the test (state=$PATH_BEFORE)"
fi

# --- step 2: the operator removes the first-choice transport -----------------
dev_logcat_clear
dev_operator_step "turn the handset's Wi-Fi OFF (keep mobile data and Bluetooth ON), then leave SCMessenger in the foreground"

# --- step 3: prove the removal actually happened ----------------------------
path_gone() {
  dev_api_get /api/connection-path-state "$DEV_RUN_DIR/path_after.json"
  [[ "$(dev_json_field "$DEV_RUN_DIR/path_after.json" "state")" != "DirectPreferred" ]]
}
if dev_poll_until "${SCM_D6_WINDOW_SECS:-180}" 15 "first-choice path left DirectPreferred" path_gone; then
  dev_ok "first-choice (direct/LAN) path is gone after the radio change (state=$(dev_json_field "$DEV_RUN_DIR/path_after.json" state))"
else
  dev_fail "the first-choice path never changed, so this run did NOT test fallback (state stayed $(dev_json_field "$DEV_RUN_DIR/path_after.json" state)); a delivery here would prove nothing"
fi

# --- step 4: the message must still land, receiver-side ----------------------
TOKEN="scm-d6-$(date -u +%H%M%S)"
dev_info "token for this run: $TOKEN"

SEND_FILE="$(dev_drive_send "$DEV_PHONE_PEER" "$TOKEN")"
MSG_ID="$(dev_json_field "$SEND_FILE" "message_id")"
if [[ -n "$MSG_ID" ]]; then
  dev_ok "Windows node accepted the send with the direct path down (message_id ${MSG_ID:0:12}...)"
else
  dev_fail "Windows node refused the send with the direct path down: $(tr -d '\n' < "$SEND_FILE" 2>/dev/null | head -c 200)"
fi

dev_operator_step "confirm that '$TOKEN' appeared in the chat on the handset"

dev_pull_phone_evidence

dev_assert_grep "handset decrypted the message over the fallback path (receiver-side plaintext token)" \
  "$DEV_RUN_DIR/mesh.log" "$TOKEN"
dev_assert_grep "handset kept it in durable history (files/ledger.json)" \
  "$DEV_RUN_DIR/phone_ledger.json" "$TOKEN"

dev_api_get "/api/send/${MSG_ID}" "$DEV_RUN_DIR/send_status.json"
DELIVERED="$(dev_json_field "$DEV_RUN_DIR/send_status.json" "delivered")"
if [[ "$DELIVERED" == "True" || "$DELIVERED" == "true" ]]; then
  dev_ok "receipt recorded on the Windows node for the fallback delivery (delivered=true)"
else
  dev_fail "no receipt for the fallback delivery (delivered=${DELIVERED:-absent})"
fi

# --- step 5: name the transport that actually carried it ---------------------
if grep -qEi 'relay|quic|wss|/tcp/|websocket' "$DEV_RUN_DIR/mesh.log" 2>/dev/null; then
  dev_ok "handset log names a wide-area/relay transport for the delivery (fallback path identified)"
else
  dev_warn "handset log does not name the transport that carried the message, so 'fallback selected a working path' rests on the delivery evidence above only; run: adb -s <serial> logcat -d | grep -i -E 'relay|quic|tcp'"
fi

# --- step 6: record the routing-confidence evidence gap ---------------------
dev_api_get /api/diagnostics "$DEV_RUN_DIR/cli_diagnostics.json"
if [[ -n "$(dev_json_field "$DEV_RUN_DIR/cli_diagnostics.json" "routing.confidence")" ]]; then
  dev_ok "adaptive-routing confidence is exposed and non-empty"
else
  dev_warn "no adaptive-routing confidence surface in /api/diagnostics, so SHIP_PLAN 6.2's 'confidence non-zero' claim cannot be asserted from the API; the fallback claim rests on the receiver-side evidence above (pre-registered gap, not a silent pass)"
fi

dev_operator_step "restore the handset's Wi-Fi"
dev_summary "D6 with the first-choice path proven gone"
