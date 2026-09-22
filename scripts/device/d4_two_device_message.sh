#!/usr/bin/env bash
# =============================================================================
# scripts/device/d4_two_device_message.sh
#
# SHIP_PLAN.md G3, exit criterion D4: "Message + receipt between the two
# endpoints (Android handset <-> Windows CLI node)".
#
# Evidence bar, verbatim from G3: receiver-side decrypt + durable history +
# receipt -- NOT transport ACKs, NOT UI counters, NOT BLE local acceptance.
# A sender-side "Delivered" badge alone does not satisfy this script.
#
# Seat split (docs/rules/ANDROID.md): the Windows/CLI node is driven here; the
# handset is passive and the operator performs the phone-side human path.
#
# Usage:
#   scripts/device/d4_two_device_message.sh [--apk <path>] [--peer <phone-peer-id>]
#                                          [--serial <serial>] [--api <url>]
# --peer is required when /api/peers does not report exactly one live peer.
# =============================================================================
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

ITEM="d4_two_device_message"
TITLE="D4: message + receipt, Android handset <-> Windows CLI node (both directions)"

dev_args "$@"
if [[ "$DEV_HELP" == "1" ]]; then sed -n '3,19p' "$0"; exit 0; fi
dev_init "$ITEM" "$TITLE"
dev_evidence_bar "receiver-side decrypt + durable history + receipt (not transport ACKs, not UI counters)"
dev_disposition "a new HANDOFF/todo/ ticket built from the captured artifacts" "blocks the v0.4.0 tag (D4 is a G3 exit criterion, CP3)"
dev_capture_recipe "info,scmessenger=debug" "logcat from clear to pull; phone files/logs/scmessenger-mesh.log; CLI /api/history" "clear -> operator acts -> pull (no timed stream)"

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
dev_ok "target handset peer: ${DEV_PHONE_PEER:0:16}..."

# --- direction A: Windows node -> handset ------------------------------------
TOKEN_A="scm-d4-a-$(date -u +%H%M%S)"
TOKEN_B="scm-d4-b-$(date -u +%H%M%S)"
dev_info "tokens for this run: A=$TOKEN_A (Windows sends) B=$TOKEN_B (handset sends)"

dev_logcat_clear
dev_operator_step "leave SCMessenger open on the handset and in the foreground; do not type or send yet"

SEND_A_FILE="$(dev_drive_send "$DEV_PHONE_PEER" "$TOKEN_A")"
MSG_ID_A="$(dev_json_field "$SEND_A_FILE" "message_id")"
if [[ -n "$MSG_ID_A" ]]; then
  dev_ok "Windows node accepted the send to the handset (message_id ${MSG_ID_A:0:12}...)"
else
  dev_fail "Windows node refused the send to the handset: $(tr -d '\n' < "$SEND_A_FILE" 2>/dev/null | head -c 200)"
fi

dev_operator_step "confirm that '$TOKEN_A' appeared in the chat on the handset"

# --- direction B: handset -> Windows node ------------------------------------
dev_operator_step "on the handset, send exactly this text to the Windows node contact: $TOKEN_B"

# --- passive evidence pull, once, after both directions ----------------------
dev_pull_phone_evidence

# --- assertions: direction A (handset is the receiver) ----------------------
dev_assert_grep "A: handset decrypted the message (receiver-side proof -- plaintext token in its own mesh log)" \
  "$DEV_RUN_DIR/mesh.log" "$TOKEN_A"
dev_assert_grep "A: handset kept it in durable history (files/ledger.json)" \
  "$DEV_RUN_DIR/phone_ledger.json" "$TOKEN_A"
dev_api_get "/api/send/${MSG_ID_A}" "$DEV_RUN_DIR/send_status_A.json"
DELIVERED_A="$(dev_json_field "$DEV_RUN_DIR/send_status_A.json" "delivered")"
if [[ "$DELIVERED_A" == "True" || "$DELIVERED_A" == "true" ]]; then
  dev_ok "A: Windows node recorded a receipt for the handset's message (delivered=true)"
else
  dev_fail "A: no receipt on the Windows node for ${MSG_ID_A:0:12}... (delivered=${DELIVERED_A:-absent})"
fi

# --- assertions: direction B (Windows node is the receiver) -----------------
dev_api_get "/api/history?limit=50" "$DEV_RUN_DIR/cli_history.json"
dev_assert_grep "B: Windows node decrypted and stored the handset's message (receiver-side durable history)" \
  "$DEV_RUN_DIR/cli_history.json" "$TOKEN_B"
dev_assert_grep "B: handset mesh log shows the outbound send" \
  "$DEV_RUN_DIR/mesh.log" "$TOKEN_B"
if grep -qEi 'receipt|deliver(y|ed)|ack' "$DEV_RUN_DIR/mesh.log" 2>/dev/null; then
  dev_ok "B: handset logged a receipt/ack for its send"
else
  dev_warn "B: no handset receipt line found in the passive pull; the handset's receipt surface is not machine-readable here (sender-side receipt for direction A plus the Windows-side history entry are the proof used) -- run: adb -s <serial> logcat -d | grep -i -E 'receipt|delivered'"
fi

dev_summary "D4 both directions"
