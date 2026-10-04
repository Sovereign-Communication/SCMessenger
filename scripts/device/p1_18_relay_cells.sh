#!/usr/bin/env bash
# =============================================================================
# scripts/device/p1_18_relay_cells.sh
#
# HANDOFF/V1_0_0_EXECUTION_PLAN.md P1-18 [DEVICE]: "Relay cells: QUIC/TCP relay
# and internet relay. LAN-relay first: third node = second CLI instance on the
# Windows box in relay mode; validate relay custody end-to-end (store, forward,
# receipt convergence) with the phone offline-then-returning."
#
# Two modes:
#   (default)     full device-day run: handset offline -> custody -> handset
#                 returns -> delivery and receipt converge
#   --relay-only  device-free: starts the relay node and proves the custody path
#                 accepts and holds a message for an offline peer. This is the
#                 part that can be rehearsed while the handset is away, which is
#                 the point of T7; it does not stand in for the device verdict.
#
# The script starts and stops exactly one extra node, with its own data directory
# inside the run dir, so it is idempotent and leaves nothing running.
#
# Seat split (docs/rules/ANDROID.md): driving is on the Windows/CLI side; the
# handset is passive and the offline/online change is the operator's.
#
# Usage:
#   scripts/device/p1_18_relay_cells.sh [--relay-only] [--apk <path>]
#                                       [--peer <phone-peer-id>] [--serial <serial>]
# =============================================================================
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

ITEM="p1_18_relay_cells"
TITLE="P1-18: relay cells -- custody held while the handset is away, delivered and receipted on return"

dev_args "$@"

RELAY_ONLY=0
while [[ ${#DEV_EXTRA[@]} -gt 0 ]]; do
  case "${DEV_EXTRA[0]}" in
    --relay-only) RELAY_ONLY=1; DEV_EXTRA=("${DEV_EXTRA[@]:1}") ;;
    *) echo "unknown argument: ${DEV_EXTRA[0]}" >&2; exit 2 ;;
  esac
done

if [[ "$DEV_HELP" == "1" ]]; then sed -n '3,28p' "$0"; exit 0; fi

RELAY_ONLY_NOTE=""
[[ "$RELAY_ONLY" == "1" ]] && RELAY_ONLY_NOTE=" (relay-only: no handset verdict is produced)"
dev_init "$ITEM" "$TITLE$RELAY_ONLY_NOTE"
dev_evidence_bar "custody held and evidenced while the peer is away, then receiver-side decrypt + durable history + receipt on return"
dev_disposition "P1-18 gap-closure ticket, or a WAN-relay waiver from the operator" "does not block the v0.4.0 tag; post-exit Phase 1 verification debt (HANDOFF/todo/_QUEUE.md:340)"
dev_capture_recipe "info,scmessenger=debug,scmessenger_core::store::relay_custody=debug" "primary /api/send polling; relay stdout captured to relay.log" "start relay -> peer offline -> send -> peer returns -> pull"

DEV_RELAY_LISTEN="${SCM_RELAY_LISTEN:-/ip4/0.0.0.0/tcp/0}"
DEV_RELAY_HTTP="${SCM_RELAY_HTTP_PORT:-9100}"
RELAY_PID=""

cleanup() {
  if [[ -n "$RELAY_PID" ]] && kill -0 "$RELAY_PID" 2>/dev/null; then
    kill "$RELAY_PID" 2>/dev/null
    sleep 2
    if kill -0 "$RELAY_PID" 2>/dev/null; then
      dev_fail "relay instance pid $RELAY_PID survived the shutdown; kill it by hand: kill $RELAY_PID"
    else
      dev_ok "relay instance stopped (pid $RELAY_PID no longer running)"
    fi
  fi
}
trap cleanup EXIT

# --- preconditions -----------------------------------------------------------
dev_require_cli_node || { dev_summary "aborted: CLI node not reachable"; exit 1; }
if [[ "$RELAY_ONLY" != "1" ]]; then
  dev_require_adb || { dev_summary "aborted: no device"; exit 1; }
  PRECONDITIONS_OK=1
  dev_require_physical || PRECONDITIONS_OK=0
  dev_require_apk_identity || PRECONDITIONS_OK=0
  dev_require_signature
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
fi

# --- step 1: start the relay node (this is what makes the cell a cell) -------
RELAY_DIR="$DEV_RUN_DIR/relay-data"
mkdir -p "$RELAY_DIR"
SCM_DATA_DIR="$RELAY_DIR" SCMESSENGER_DATA_DIR="$RELAY_DIR" \
  scmessenger-cli relay --listen "$DEV_RELAY_LISTEN" --http-port "$DEV_RELAY_HTTP" \
  --name "p118-relay" > "$DEV_RUN_DIR/relay.log" 2>&1 &
RELAY_PID=$!
dev_info "relay instance started (pid $RELAY_PID, data dir $RELAY_DIR, log $DEV_RUN_DIR/relay.log)"

relay_up() { [[ -s "$DEV_RUN_DIR/relay.log" ]]; }
if dev_poll_until 60 3 "relay instance produced its startup log" relay_up; then
  dev_ok "relay instance reported startup within 60s"
else
  dev_fail "relay instance produced no log within 60s; see $DEV_RUN_DIR/relay.log"
fi

if grep -qEi 'peer id|peerid|12D3KooW' "$DEV_RUN_DIR/relay.log" 2>/dev/null; then
  dev_ok "relay instance published its peer identity (custody is attributable to a node, not an anonymous forwarder)"
else
  dev_warn "relay log does not name its peer id, so the relaying node's identity is unrecorded; see $DEV_RUN_DIR/relay.log"
fi

# --- step 2: peer away -- the message must be accepted and NOT delivered ----
if [[ "$RELAY_ONLY" == "1" ]]; then
  dev_skip "handset leg not exercised (--relay-only); the offline-then-returning verdict requires the handset"
  dev_api_get /api/peers "$DEV_RUN_DIR/cli_peers.json"
  TARGET_PEER="$(dev_json_field "$DEV_RUN_DIR/cli_peers.json" "peers.0.peer_id")"
  [[ -z "$TARGET_PEER" ]] && TARGET_PEER="$(dev_json_field "$DEV_RUN_DIR/cli_peers.json" "peers.0")"
  [[ -z "$TARGET_PEER" ]] && TARGET_PEER="scm-p118-absent-peer"
else
  dev_operator_step "take the handset offline (airplane mode ON, or close and force-stop the app) and leave it away"
  TARGET_PEER="$DEV_PHONE_PEER"
fi

TOKEN="scm-p118-$(date -u +%H%M%S)"
SEND_FILE="$(dev_drive_send "$TARGET_PEER" "$TOKEN")"
MSG_ID="$(dev_json_field "$SEND_FILE" "message_id")"
if [[ -n "$MSG_ID" ]]; then
  dev_ok "primary node accepted the send for the away peer (message_id ${MSG_ID:0:12}...)"
else
  dev_fail "primary node refused the send for the away peer: $(tr -d '\n' < "$SEND_FILE" 2>/dev/null | head -c 200)"
fi

dev_api_get "/api/send/${MSG_ID}" "$DEV_RUN_DIR/send_status_away.json"
DELIVERED_AWAY="$(dev_json_field "$DEV_RUN_DIR/send_status_away.json" "delivered")"
if [[ "$DELIVERED_AWAY" == "True" || "$DELIVERED_AWAY" == "true" ]]; then
  dev_warn "the node already reports delivered=true while the peer is away, so 'custody held during absence' cannot be distinguished from a delivery; confirm the peer really is offline"
else
  dev_ok "message is not delivered while the peer is away (the custody path is the only candidate carrier)"
fi

if grep -qEi 'custody|store_?and_?forward|relay_custody' "$DEV_RUN_DIR/relay.log" 2>/dev/null; then
  dev_ok "relay log evidences custody activity for the away peer"
else
  dev_warn "relay log does not name custody activity, so the message may be held only by the primary node's own outbox; check $DEV_RUN_DIR/relay.log"
fi

# --- step 3 (device mode): the peer returns and the receipt converges -------
if [[ "$RELAY_ONLY" != "1" ]]; then
  dev_operator_step "bring the handset back online and to the foreground"
  delivered() {
    dev_api_get "/api/send/${MSG_ID}" "$DEV_RUN_DIR/send_status_return.json"
    local d; d="$(dev_json_field "$DEV_RUN_DIR/send_status_return.json" "delivered")"
    [[ "$d" == "True" || "$d" == "true" ]]
  }
  if dev_poll_until "${SCM_RELAY_WINDOW_SECS:-600}" 20 "receipt converged after the peer returned" delivered; then
    dev_ok "receipt converged after the peer returned (custody released and the delivery round-tripped)"
  else
    dev_fail "no receipt within the window after the peer returned; custody did not converge (see $DEV_RUN_DIR/relay.log)"
  fi

  dev_pull_phone_evidence
  dev_assert_grep "handset decrypted the custody-held message on return (receiver-side token)" \
    "$DEV_RUN_DIR/mesh.log" "$TOKEN"
  dev_assert_grep "handset kept it in durable history after the offline period" \
    "$DEV_RUN_DIR/phone_ledger.json" "$TOKEN"
fi

dev_summary "P1-18 relay cells${RELAY_ONLY_NOTE}"
