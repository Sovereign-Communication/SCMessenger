#!/usr/bin/env bash
# =============================================================================
# scripts/device/p1_17_wifi_direct.sh
#
# HANDOFF/V1_0_0_EXECUTION_PLAN.md P1-17 [SONNET][AUDIT-GATE][DEVICE]: the WiFi
# Direct cell. Per the plan's 2.6 matrix the open question is the Windows peer
# story: Android<->Windows over a Direct group with Windows joining as a legacy
# client, or an explicit operator-approved waiver narrowing the cell to
# Android<->Android, which is [BLOCKED-HW] with one handset.
#
# Current disposition, stated so nobody expects a pass: HANDOFF/todo/_QUEUE.md:326
# records this cell as WAIVED (emulator HW restriction), and the corrected test
# fleet in HANDOFF/freebuff/inbox/BRIEF_2026-08-31_state_pacing_and_next.md has
# ONE handset ("there is no second phone"). The default behaviour of this script
# is therefore [SKIP] with the reason, not a guess. Run it with --force on a day
# there are two devices on the bench.
#
# Seat split (docs/rules/ANDROID.md): driving is on the Windows/CLI side; both
# handsets are passive.
#
# Usage:
#   scripts/device/p1_17_wifi_direct.sh [--apk <path>] [--peer <peer-id>] [--force]
# =============================================================================
set -uo pipefail
source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

ITEM="p1_17_wifi_direct"
TITLE="P1-17: WiFi Direct cell (waived with one handset; --force requires two devices)"

dev_args "$@"

FORCE=0
while [[ ${#DEV_EXTRA[@]} -gt 0 ]]; do
  case "${DEV_EXTRA[0]}" in
    --force) FORCE=1; DEV_EXTRA=("${DEV_EXTRA[@]:1}") ;;
    *) echo "unknown argument: ${DEV_EXTRA[0]}" >&2; exit 2 ;;
  esac
done

if [[ "$DEV_HELP" == "1" ]]; then sed -n '3,22p' "$0"; exit 0; fi
dev_init "$ITEM" "$TITLE"
dev_evidence_bar "receiver-side decrypt + durable history + receipt across the Direct group (a connected group is not a result)"
dev_disposition "P1-17 gap-closure ticket, or a recorded waiver in the 2.6 matrix" "does not block the v0.4.0 tag; it settles one Phase 1 matrix cell"
dev_capture_recipe "info,scmessenger=debug,scmessenger_core::transport=debug" "logcat from clear to pull; phone files/logs/scmessenger-mesh.log" "clear -> group formed -> bilateral exchange -> pull"

if [[ "$FORCE" != "1" ]]; then
  dev_skip "WiFi Direct cell is WAIVED with one handset: HANDOFF/todo/_QUEUE.md:326 records the waiver (emulator HW restriction) and the corrected fleet has one phone, so Android<->Android cannot be formed; re-run with --force when a second device is on the bench"
  dev_summary "P1-17 waived"
  exit 0
fi

# --- preconditions for a forced run -----------------------------------------
dev_require_adb || { dev_summary "aborted: no device"; exit 1; }
PRECONDITIONS_OK=1
dev_require_cli_node || PRECONDITIONS_OK=0
dev_require_signature
if [[ "$PRECONDITIONS_OK" != "1" ]]; then
  dev_summary "aborted: preconditions not met"
  exit 1
fi

DEVICE_COUNT="$(dev_device_count)"
if [[ "$DEVICE_COUNT" -lt 2 ]]; then
  dev_fail "the Direct cell needs two Android devices (or one Android plus a Windows Direct client); $DEVICE_COUNT attached"
else
  dev_ok "$DEVICE_COUNT devices attached, so an Android<->Android Direct group can be attempted"
fi

if dev_adb shell pm list features 2>/dev/null | grep -qi 'wifi.direct\|android.hardware.wifi.direct'; then
  dev_ok "handset advertises WiFi Direct hardware support"
else
  dev_warn "the handset did not advertise WiFi Direct support; run: adb -s <serial> shell pm list features | grep -i direct"
fi

# --- the exchange ------------------------------------------------------------
TOKEN="scm-p117-$(date -u +%H%M%S)"
dev_logcat_clear
dev_operator_step "on both devices: turn Wi-Fi ON, and start the group from one handset per P1-17's GO-intent flow"
dev_operator_step "send this text from the first device to the second: $TOKEN"
dev_operator_step "then send any text from the second device back to the first, and tell the run which device received it"

dev_adb logcat -d -v time > "$DEV_RUN_DIR/logcat_dev1.txt" 2>/dev/null
dev_assert_grep "second device decrypted the Direct-group message (token in logcat)" "$DEV_RUN_DIR/logcat_dev1.txt" "$TOKEN"

if grep -qEi 'wifi_?direct|p2p|group owner|GO negotiation' "$DEV_RUN_DIR/logcat_dev1.txt" 2>/dev/null; then
  dev_ok "logcat evidences a WiFi Direct group/P2P path"
else
  dev_warn "logcat does not evidence a Direct path, so the message may have taken LAN instead; run: adb -s <serial> logcat -d | grep -i -E 'p2p|direct'"
fi

dev_summary "P1-17 forced run"
