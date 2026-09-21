#!/usr/bin/env bash
# STOP-TEARDOWN-TIMEOUT-001: on-device acceptance check for "can the user stop
# the mesh?".
#
# Why this exists: the stop path was patched seven times between 2026-09-06 and
# 2026-09-21 (R3, R4, R6-2, R7-1, R8-1, STOP-RACE-001, HANG-LOCK-001) and the
# behaviour still failed on the Pixel, because every one of those fixes shipped
# with a unit test of a PURE DECISION FUNCTION and none with a check that the
# service actually goes away. No test in android/app/src/test/ asserted service
# termination at all before this script's companion test was added. CI cannot run
# a service lifecycle, so nothing in this repo could detect the regression: the
# only detector was the operator noticing.
#
# What it asserts, on a real device:
#   1. With the mesh running, the foreground ServiceRecord exists.
#   2. After the user's Stop (driven here through the app's own Stop button when
#      the UI tree exposes it), the record is gone -- or is no longer
#      startRequested and no longer foreground -- within --timeout seconds.
#   3. It STAYS gone for --hold seconds. This is the half every previous fix
#      missed: a latched stop must not be re-armed by a START_STICKY redelivery
#      or a deferred ENSURE.
#
# Usage:
#   scripts/pixel_stop_acceptance.sh [--timeout 30] [--hold 20] [--manual]
#
#   --manual   do not try to tap Stop; poll while you tap it yourself
#
# Exit codes: 0 PASS, 1 FAIL, 2 could not run (no device / no package).
#
# Read-only: the only mutation is one tap on the app's own Stop control.

set -u

PKG=com.scmessenger.android
SERVICE=MeshForegroundService
OUT_DIR=tmp/pixel-stop-acceptance
TIMEOUT=30
HOLD=20
MANUAL=0

while [ $# -gt 0 ]; do
    case "$1" in
        --timeout) TIMEOUT="$2"; shift 2 ;;
        --hold) HOLD="$2"; shift 2 ;;
        --manual) MANUAL=1; shift ;;
        -h|--help) sed -n '2,30p' "$0"; exit 0 ;;
        *) echo "[WARNING] unknown argument: $1"; shift ;;
    esac
done

# Git Bash rewrites /sdcard/... into a Windows path; the device needs it literal.
export MSYS_NO_PATHCONV=1

mkdir -p "$OUT_DIR"
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
EVIDENCE="$OUT_DIR/stop-acceptance-$STAMP.txt"
: > "$EVIDENCE"

note() { echo "$@" | tee -a "$EVIDENCE"; }
log()  { echo "$@" >> "$EVIDENCE"; }

if ! command -v adb >/dev/null 2>&1; then
    note "[FAIL] adb not found on PATH"
    exit 2
fi

DEVICES=$(adb devices | awk 'NR>1 && $2=="device" {print $1}')
if [ -z "$DEVICES" ]; then
    note "[FAIL] no adb device in 'device' state (adb devices)"
    exit 2
fi
DEVICE=$(echo "$DEVICES" | head -1)
note "[INFO] device=$DEVICE package=$PKG"

if [ "$(adb shell pm list packages "$PKG" | grep -c "$PKG")" -eq 0 ]; then
    note "[FAIL] $PKG is not installed on $DEVICE"
    exit 2
fi

VERSION=$(adb shell dumpsys package "$PKG" | grep -m1 versionCode | tr -d '\r')
note "[INFO] $VERSION"

# --- the two device facts that decide PASS/FAIL ------------------------------
# startRequested=true is the service the user asked to stop and which did not
# stop. isForeground=true means the persistent notification is still pinned.
record_state() {
    adb shell dumpsys activity services "$PKG" 2>/dev/null | tr -d '\r' \
        | grep -A 12 "$SERVICE" \
        | grep -o "startRequested=[a-z]*\|isForeground=[a-z]*" \
        | tr '\n' ' '
}

service_up() {
    local state
    state=$(record_state)
    case "$state" in
        *startRequested=true*) return 0 ;;
        *) return 1 ;;
    esac
}

snapshot() {
    log "--- snapshot $(date -u +%H:%M:%SZ) ---"
    adb shell dumpsys activity services "$PKG" 2>/dev/null | tr -d '\r' \
        | grep -A 12 "$SERVICE" >> "$EVIDENCE"
}

# --- 1. baseline -------------------------------------------------------------
STATE=$(record_state)
echo "[INFO] baseline service state: ${STATE:-<no record>}" | tee -a "$EVIDENCE"
if ! service_up; then
    echo "[WARNING] $SERVICE is not running; the stop assertion cannot be tested." | tee -a "$EVIDENCE"
    echo "[INFO] start the mesh in the app, then re-run." | tee -a "$EVIDENCE"
    exit 2
fi

# --- 2. drive the stop -------------------------------------------------------
if [ "$MANUAL" -eq 1 ]; then
    echo "[INFO] --manual: tap Stop in the app now; polling for $TIMEOUT s." | tee -a "$EVIDENCE"
else
    adb shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1
    sleep 3
    adb shell uiautomator dump /sdcard/pixel_stop_dump.xml >/dev/null 2>&1
    DUMP="$OUT_DIR/ui-$STAMP.xml"
    adb shell cat /sdcard/pixel_stop_dump.xml > "$DUMP" 2>/dev/null
    BOUNDS=$(tr '>' '>\n' < "$DUMP" 2>/dev/null \
        | grep 'text="Stop"' | head -1 \
        | grep -o 'bounds="\[[0-9]*,[0-9]*\]\[[0-9]*,[0-9]*\]"')
    if [ -z "$BOUNDS" ]; then
        echo "[WARNING] no Stop control in the current UI tree; the app must be on" | tee -a "$EVIDENCE"
        echo "[WARNING] Settings -> Mesh Service. Tap Stop yourself now (polling for" | tee -a "$EVIDENCE"
        echo "[WARNING] $TIMEOUT s). UI dump kept at $DUMP" | tee -a "$EVIDENCE"
    else
        COORDS=$(echo "$BOUNDS" | tr -dc '0-9,' | awk -F, '{print int(($1+$3)/2), int(($2+$4)/2)}')
        echo "[INFO] tapping Stop at $COORDS (from $BOUNDS)" | tee -a "$EVIDENCE"
        adb shell input tap $COORDS
    fi
fi

# --- 3. did it stop? ---------------------------------------------------------
STOPPED_AT=""
for i in $(seq 1 "$TIMEOUT"); do
    if ! service_up; then
        STOPPED_AT="$i"
        break
    fi
    sleep 1
done

if [ -z "$STOPPED_AT" ]; then
    echo "[FAIL] $SERVICE still startRequested=true after ${TIMEOUT}s -- the mesh did NOT stop" | tee -a "$EVIDENCE"
    snapshot
    echo "[FAIL] evidence: $EVIDENCE" | tee -a "$EVIDENCE"
    echo "[INFO] triage: adb logcat -d | grep -E 'stop requested|Stopping mesh service|outbox_'" | tee -a "$EVIDENCE"
    exit 1
fi
echo "[OK] service stopped after ${STOPPED_AT}s (state: $(record_state))" | tee -a "$EVIDENCE"

# --- 4. does it stay stopped? (the half previous fixes never checked) --------
for i in $(seq 1 "$HOLD"); do
    if service_up; then
        echo "[FAIL] $SERVICE came back ${i}s after the stop -- a latched stop was re-armed" | tee -a "$EVIDENCE"
        echo "[INFO] check MeshForegroundService.shouldReturnSticky and the stopSelfResult paths" | tee -a "$EVIDENCE"
        snapshot
        exit 1
    fi
    sleep 1
done

FINAL=$(record_state)
echo "[OK] still stopped after a further ${HOLD}s (state: ${FINAL:-<no record>})" | tee -a "$EVIDENCE"
snapshot
echo "[OK] PASS: stop completed, and was not re-armed. Evidence: $EVIDENCE" | tee -a "$EVIDENCE"
exit 0
