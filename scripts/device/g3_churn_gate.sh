#!/usr/bin/env bash
# =============================================================================
# scripts/device/g3_churn_gate.sh
#
# SHIP_PLAN.md G3 lists "the churn gate" with D4/D6/D7; its operator framing is
# the 2026-08-31 directive quoted at SHIP_PLAN.md:295-301 -- a node's public IP
# changes on every re-deploy, that is a design constraint, and a node that moved
# must rejoin with no human action and no manual re-seeding.
#
# What this proves, in order:
#   1. the cloud node's address actually changed (otherwise nothing is tested)
#   2. nobody hand-seeded the new address (the anti-cheat assertion)
#   3. the Windows node leaves Bootstrapping unaided inside the window
#   4. the new address is actually in the node's own view of its peers, so the
#      recovery is ATTRIBUTABLE to the new address rather than to a path that
#      never went away
#   5. the new address reaches the handset by ledger sharing, not by hand
#
# Assertion 4 exists because a healthy rig reports DirectPreferred whether or not
# anything churned: without it, this gate would print [OK] on a run that proved
# nothing. When the premise cannot be established the script skips or warns; it
# never reports a rejoin it cannot attribute.
#
# Usage:
#   scripts/device/g3_churn_gate.sh --cloud <new-host:port> --previous-cloud <old-ip>
#                                   [--apk <path>] [--serial <serial>] [--window <secs>]
# =============================================================================
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

ITEM="g3_churn_gate"
TITLE="Churn gate: a cloud node that changed address rejoins unaided and the new address gossips to the handset"

dev_args "$@"

PREVIOUS_CLOUD=""
WINDOW_SECS=""
while [[ ${#DEV_EXTRA[@]} -gt 0 ]]; do
  case "${DEV_EXTRA[0]}" in
    --previous-cloud) PREVIOUS_CLOUD="${DEV_EXTRA[1]:-}"; DEV_EXTRA=("${DEV_EXTRA[@]:2}") ;;
    --window)         WINDOW_SECS="${DEV_EXTRA[1]:-}";    DEV_EXTRA=("${DEV_EXTRA[@]:2}") ;;
    *) echo "unknown argument: ${DEV_EXTRA[0]}" >&2; exit 2 ;;
  esac
done

if [[ "$DEV_HELP" == "1" ]]; then sed -n '3,32p' "$0"; exit 0; fi
dev_init "$ITEM" "$TITLE"
dev_evidence_bar "Windows connection-path state leaves Bootstrapping by itself + the new address is in the node's peer view + phone ledger gains it + seed list untouched"
dev_disposition "a new HANDOFF/todo/ ticket built from the captured artifacts" "blocks the v0.4.0 tag (G3 contingency row)"
dev_capture_recipe "info,scmessenger=debug,scmessenger_core::store=debug" "CLI /api/connection-path-state polled; phone files/ledger.json re-pulled per poll" "re-deploy -> poll window (default 900s) -> pull"

WINDOW_SECS="${WINDOW_SECS:-${SCM_CHURN_WINDOW_SECS:-900}}"

# --- preconditions -----------------------------------------------------------
dev_require_cli_node || { dev_summary "aborted: CLI node not reachable"; exit 1; }
PHONE_LEG_OK=1
dev_require_adb || PHONE_LEG_OK=0
if [[ "$PHONE_LEG_OK" == "1" ]]; then dev_require_physical; fi
if [[ -z "$SCM_CLOUD_ADDR" ]]; then
  dev_skip "cloud node address not supplied (SCM_CLOUD_ADDR unset); the churn leg cannot be judged"
fi
if [[ -z "$PREVIOUS_CLOUD" ]]; then
  dev_warn "no --previous-cloud given, so 'the address actually changed' is unproven for this run"
fi

# --- step 1: the address really did change ----------------------------------
NEW_IP=""
if [[ -n "$SCM_CLOUD_ADDR" ]]; then
  NEW_IP="${SCM_CLOUD_ADDR%%:*}"
  if [[ -z "$PREVIOUS_CLOUD" ]]; then
    dev_info "new cloud address under test: $NEW_IP"
  elif [[ "${PREVIOUS_CLOUD%%:*}" == "$NEW_IP" ]]; then
    dev_fail "cloud address did not change (still $NEW_IP), so this run tests nothing about churn; re-deploy first"
  else
    dev_ok "cloud address changed: ${PREVIOUS_CLOUD%%:*} -> $NEW_IP"
  fi
fi

# --- step 2: nobody hand-seeded the new address ------------------------------
if ! command -v scmessenger-cli >/dev/null 2>&1; then
  dev_warn "seed-list check unproven: scmessenger-cli is not on PATH, so bootstrap_nodes cannot be read; run: scmessenger-cli config get bootstrap_nodes -- a manual seed would invalidate this gate"
else
  SEED_LIST="$(scmessenger-cli config get bootstrap_nodes 2>/dev/null)"
  printf 'bootstrap_nodes=%s\n' "${SEED_LIST:-unreadable}" >> "$DEV_RUN_DIR/run.env"
  if [[ -z "$SEED_LIST" ]]; then
    dev_warn "seed list unreadable; run: scmessenger-cli config get bootstrap_nodes"
  elif [[ -n "$NEW_IP" ]] && printf '%s' "$SEED_LIST" | grep -qF "$NEW_IP"; then
    dev_fail "the new cloud address IS in bootstrap_nodes, so recovery was hand-seeded and the gate is not satisfied"
  else
    dev_ok "new cloud address is absent from bootstrap_nodes (recovery is not hand-seeded)"
  fi
fi

# --- step 3: the Windows node rejoins, attributably -------------------------
path_recovered() {
  dev_api_get /api/connection-path-state "$DEV_RUN_DIR/path_poll.json"
  local state; state="$(dev_json_field "$DEV_RUN_DIR/path_poll.json" "state")"
  [[ -n "$state" && "$state" != "Bootstrapping" ]]
}

if [[ -z "$SCM_CLOUD_ADDR" ]]; then
  dev_skip "rejoin not judged: with no cloud address supplied there is no moved address to rejoin"
else
  dev_api_get /api/connection-path-state "$DEV_RUN_DIR/path_at_start.json"
  STATE_AT_START="$(dev_json_field "$DEV_RUN_DIR/path_at_start.json" "state")"
  printf 'path_state_at_start=%s\nnew_cloud_address=%s\n' "${STATE_AT_START:-unreadable}" "$NEW_IP" >> "$DEV_RUN_DIR/run.env"

  if [[ "$STATE_AT_START" == "Bootstrapping" || -z "$STATE_AT_START" ]]; then
    if dev_poll_until "$WINDOW_SECS" 20 "Windows node leaves Bootstrapping after the address change" path_recovered; then
      dev_api_get /api/peers "$DEV_RUN_DIR/cli_peers.json"
      if grep -qF "$NEW_IP" "$DEV_RUN_DIR/cli_peers.json" 2>/dev/null; then
        dev_ok "Windows node rejoined unaided within ${WINDOW_SECS}s and its peer view lists $NEW_IP (recovery attributed to the new address)"
      else
        dev_fail "the path recovered but $NEW_IP is not in the Windows node's peer view, so the recovery is not attributable to the new address; see $DEV_RUN_DIR/cli_peers.json"
      fi
    else
      dev_fail "Windows node did not leave Bootstrapping within ${WINDOW_SECS}s of the address change (state=$(dev_json_field "$DEV_RUN_DIR/path_poll.json" state)); the rejoin path does not work without intervention"
    fi
  else
    dev_warn "the Windows node was already $STATE_AT_START before this run, so a rejoin cannot be attributed to the address change; re-run immediately after the deploy (baseline recorded in run.env)"
  fi
fi

# --- step 4: the new address reaches the handset by ledger sharing ----------
phone_has_new_addr() {
  dev_adb exec-out run-as "$SCM_ANDROID_PACKAGE" cat files/ledger.json > "$DEV_RUN_DIR/phone_ledger.json" 2>/dev/null
  grep -qF "$NEW_IP" "$DEV_RUN_DIR/phone_ledger.json" 2>/dev/null
}
if [[ "$PHONE_LEG_OK" != "1" ]]; then
  dev_skip "handset leg not exercised: no handset attached, so address propagation to the phone is unverified"
elif [[ -z "$NEW_IP" ]]; then
  dev_skip "handset address-propagation not judged: no cloud address supplied to look for"
elif dev_poll_until "$WINDOW_SECS" 30 "handset ledger learned the new cloud address" phone_has_new_addr; then
  dev_ok "handset ledger carries the new cloud address $NEW_IP (propagated by ledger sharing, not by hand)"
else
  dev_fail "handset ledger never learned $NEW_IP within ${WINDOW_SECS}s; the phone would need a manual re-seed after an address change"
fi

dev_summary "churn gate, window ${WINDOW_SECS}s"
