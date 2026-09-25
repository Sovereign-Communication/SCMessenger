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
#   scripts/pixel_stop_acceptance.sh [--timeout 30] [--hold 20] [--manual] [--no-launch]
#
#   --manual     do not try to tap Stop; poll while you tap it yourself
#   --no-launch  do not re-launch the app first (use when you have already
#                navigated to Settings -> Mesh Service)
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
LAUNCH=1

while [ $# -gt 0 ]; do
    case "$1" in
        --timeout) TIMEOUT="$2"; shift 2 ;;
        --hold) HOLD="$2"; shift 2 ;;
        --manual) MANUAL=1; shift ;;
        --no-launch) LAUNCH=0; shift ;;
        -h|--help) sed -n '2,32p' "$0"; exit 0 ;;
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
# Resolve the Stop control's centre from a UI dump, refusing to guess.
#
# The first version of this script parsed the dump with tr/grep/awk. On the real
# device that produced (12000,1080) for a 1080x2400 screen -- an off-screen tap
# that does nothing, which the script would have reported as a successful tap.
# A verification tool that can silently do nothing is worse than none, so the
# rule here is: fail OPEN on what can be seen (print every label), fail CLOSED on
# what gets acted on (never tap an unvalidated coordinate).
resolve_stop_coords() {
    local dump="$1"
    command -v python >/dev/null 2>&1 || return 1
    python - "$dump" "$DEVICE" <<'PY'
import re, subprocess, sys

dump, device = sys.argv[1], sys.argv[2]
try:
    xml = open(dump, encoding="utf-8", errors="replace").read()
except OSError:
    sys.exit(1)

tags = re.findall(r"<node\b[^>]*?/?>", xml)
visible = sorted({m.group(1) for m in re.finditer(r'text="([^"]+)"', xml)})

hits = []
for tag in tags:
    text = re.search(r'text="([^"]*)"', tag)
    bounds = re.search(r'bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]"', tag)
    if not text or text.group(1) != "Stop" or not bounds:
        continue
    hits.append((tuple(int(v) for v in bounds.groups()), 'clickable="true"' in tag))

clickable = [h for h in hits if h[1]]
chosen = clickable[0] if len(clickable) == 1 else (hits[0] if len(hits) == 1 else None)

if chosen is None:
    print(f"[WARNING] {len(hits)} node(s) matched text='Stop' -- refusing to guess", file=sys.stderr)
    print("[INFO] labels visible in this UI tree: " + ", ".join(visible), file=sys.stderr)
    sys.exit(1)

x1, y1, x2, y2 = chosen[0]
x, y = (x1 + x2) // 2, (y1 + y2) // 2
size = subprocess.run(["adb", "-s", device, "shell", "wm", "size"],
                      capture_output=True, text=True).stdout
match = re.search(r"(\d+)x(\d+)", size)
width, height = (int(match.group(1)), int(match.group(2))) if match else (1080, 2400)
if not (0 <= x < width and 0 <= y < height):
    print(f"[WARNING] computed tap ({x},{y}) is outside {width}x{height} -- refusing",
          file=sys.stderr)
    sys.exit(1)
print(f"{x} {y}")
PY
}

if [ "$MANUAL" -eq 1 ]; then
    echo "[INFO] --manual: tap Stop in the app now; polling for $TIMEOUT s." | tee -a "$EVIDENCE"
else
    if [ "$LAUNCH" -eq 1 ]; then
        adb shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1
        sleep 3
    fi
    adb shell uiautomator dump /sdcard/pixel_stop_dump.xml >/dev/null 2>&1
    DUMP="$OUT_DIR/ui-$STAMP.xml"
    adb shell cat /sdcard/pixel_stop_dump.xml > "$DUMP" 2>/dev/null
    COORDS=$(resolve_stop_coords "$DUMP" 2>>"$EVIDENCE") || COORDS=""
    if [ -z "$COORDS" ]; then
        # Fail open on visibility: say what IS on screen, so a human can see why.
        echo "[WARNING] could not resolve a Stop control with confidence; the app must" | tee -a "$EVIDENCE"
        echo "[WARNING] be on Settings -> Mesh Service. Tap Stop yourself now (polling" | tee -a "$EVIDENCE"
        echo "[WARNING] for $TIMEOUT s). UI dump kept at $DUMP" | tee -a "$EVIDENCE"
        grep -oE 'text="[^"]+"' "$DUMP" 2>/dev/null | sort -u \
            | sed 's/^/[INFO] visible: /' | tee -a "$EVIDENCE"
    else
        echo "[INFO] tapping Stop at $COORDS" | tee -a "$EVIDENCE"
        # shellcheck disable=SC2086
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
