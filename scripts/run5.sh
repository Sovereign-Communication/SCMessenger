#!/usr/bin/env bash
# run5.sh — 5-Node SCMessenger Mesh Test Harness (Unified)
#
# Nodes:
#   1. GCP      — headless relay (Docker on scmessenger-bootstrap)
#   2. OSX      — headless relay (local cargo binary)
#   3. Android  — full node (Pixel 6a via adb)
#   4. iOS Dev  — full node (physical device via devicectl)
#   5. iOS Sim  — full node (simulator via simctl)
#
# Usage (run from the repo root): ./scripts/run5.sh [--time=5] [--update] [--restore-on-exit]
#   --time=N          Run for N minutes then auto-exit (default: 5)
#   --update          Rebuild headless nodes; push latest to mobile if available
#   --restore-on-exit Stop any nodes this script launched before exiting
#
# Design:
#   • Full nodes (Android, iOS Device, iOS Sim): always pre-installed & running.
#     This script NEVER touches them — it only attaches log collectors passively.
#     If somehow not running, it brings them up gently (no force-stop, no reinstall).
#   • Headless nodes (GCP, OSX relay): checked and brought up to date if needed.
#     GCP: verified via docker ps; restarts container if stale.
#     OSX: started via cargo if not already running.
#
set -euo pipefail

# ── Args ──────────────────────────────────────────────────────────────────────
DURATION_MIN=5
UPDATE_APPS=0
RESTORE_ON_EXIT=0
while [ $# -gt 0 ]; do
  case "$1" in
    -t=*|--time=*)      DURATION_MIN="${1#*=}"; shift ;;
    -u|--update)        UPDATE_APPS=1; shift ;;
    --restore-on-exit)  RESTORE_ON_EXIT=1; shift ;;
    -h|--help)
      head -20 "$0" | grep -E "^#" | sed 's/^# \?//'
      exit 0
      ;;
    *) shift ;;
  esac
done

# ── Log Directory (timestamped — preserves all historical runs) ────────────────
TIMESTAMP=$(date '+%Y%m%d_%H%M%S')
LOGDIR="logs/5mesh/$TIMESTAMP"
mkdir -p "$LOGDIR"
ln -sfn "$TIMESTAMP" "logs/5mesh/latest"

# ── Constants ──────────────────────────────────────────────────────────────────
BUNDLE_ID="SovereignCommunications.SCMessenger"
SYNC_MARKER="=== TEST_START_MARKER: $(date -u +'%Y-%m-%dT%H:%M:%SZ') ==="
GCP_ZONE="us-central1-a"
GCP_HOST="scmessenger-bootstrap"
GCP_IMAGE="us-central1-docker.pkg.dev/scmessenger-bootstrapnode/scmessenger-repo/scmessenger-cli:latest"
GCP_CONTAINER_NAME="scmessenger-relay"
# NOTE: `scmessenger_core::mesh::delivery` was stale -- no `mesh` module
# exists in core/src (see core/src/lib.rs); EnvFilter silently matches zero
# spans/events for an unknown target, so that clause was a no-op. Delivery
# tracing actually lives in store::outbox / store::inbox (see
# core/src/store/outbox.rs, core/src/store/inbox.rs). Also added
# libp2p_mdns=trace and if_watch=debug: the mDNS "os error 10040"
# (WSAEMSGSIZE) failures on Windows/macOS otherwise surface as a single
# ERROR line with no socket/interface context.
OSX_RUST_LOG="info,libp2p_autonat=debug,libp2p_dcutr=debug,libp2p_relay=debug,libp2p_mdns=trace,if_watch=debug,scmessenger_core::transport=debug,scmessenger_core::store::relay_custody=debug,scmessenger_core::store::outbox=debug,scmessenger_core::store::inbox=debug"

# Prefer pre-built binary (instant start) over cargo run (30-60s compile)
if [ -f "target/debug/scmessenger-cli" ]; then
  OSX_RELAY_CMD="./target/debug/scmessenger-cli"
else
  OSX_RELAY_CMD="cargo run -p scmessenger-cli --"
fi

# ── PID tracking (always ours to kill — log collectors only) ──────────────────
GCP_LOG_PID=""
OSX_LOG_PID=""
ANDROID_LOGCAT_PID=""
IOS_DEV_LAUNCH_PID=""
IOS_DEV_STREAM_PID=""
IOS_SIM_STREAM_PID=""
TICKER_PID=""
OSX_PID=""

# Did WE start the app/service? (0=pre-existing, leave alone; 1=we started it)
STARTED_OSX=0
STARTED_ANDROID_APP=0
STARTED_IOS_DEV_APP=0
STARTED_IOS_SIM_APP=0

# ── Helper: log to stderr + logdir/harness.log ────────────────────────────────
HARNESS_LOG="$LOGDIR/harness.log"
hlog() {
  local msg="[$(date '+%H:%M:%S')] $*"
  echo "$msg" | tee -a "$HARNESS_LOG" >&2
}

# ── Helper: check if a PID is still alive ─────────────────────────────────────
pid_alive() { kill -0 "$1" 2>/dev/null; }

# ── GCP SSH helper: tries IAP tunnel (more reliable in restricted networks) ───
gcp_ssh() {
  local cmd="$1"
  # Try IAP tunnel first (avoids firewall blocks on port 22)
  if gcloud compute ssh "$GCP_HOST" --zone="$GCP_ZONE" \
      --tunnel-through-iap \
      --ssh-flag="-o ConnectTimeout=10 -o ServerAliveInterval=15 -o ServerAliveCountMax=3" \
      --command="$cmd" 2>/dev/null; then
    return 0
  fi
  # Fall back to direct SSH
  hlog "  GCP IAP tunnel failed, trying direct SSH..."
  gcloud compute ssh "$GCP_HOST" --zone="$GCP_ZONE" \
    --ssh-flag="-o ConnectTimeout=10 -o ServerAliveInterval=15 -o ServerAliveCountMax=3" \
    --command="$cmd" 2>/dev/null
}

# ══════════════════════════════════════════════════════════════════════════════
echo "╔══════════════════════════════════════════════════════════════╗"
printf  "║   SCMessenger Mesh Harness  %-32s ║\n" "$TIMESTAMP"
printf  "║   Duration: %-4sm  |  Update: %-5s  |  Restore: %-5s   ║\n" \
        "$DURATION_MIN" "$UPDATE_APPS" "$RESTORE_ON_EXIT"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
echo "Phase 1: Checking node status..."
echo ""

# ── Device Detection ───────────────────────────────────────────────────────────

# Android: pin to a specific serial to avoid "more than one device" errors
ADB_SERIAL=""
if adb devices 2>/dev/null | grep -q "device$"; then
  ADB_SERIAL=$(adb devices -l | tail -n +2 | awk '$2=="device"{print $1; exit}')
fi
ANDROID_AVAILABLE=0
[ -n "$ADB_SERIAL" ] && ANDROID_AVAILABLE=1

# iOS Physical Device: use devicectl JSON output for reliable UDID detection
IOS_DEVICE_UDID="${IOS_DEVICE_UDID:-}"
if [ -z "$IOS_DEVICE_UDID" ]; then
  IOS_DEVICE_UDID=$(xcrun devicectl list devices \
    --hide-default-columns --columns Identifier --columns State --hide-headers 2>/dev/null | \
    awk '$2 ~ /(available|connected)/ {print $1; exit}')
fi

# iOS Simulator: find booted sim; boot one if missing
IOS_SIM_UDID="${IOS_SIM_UDID:-}"
if [ -z "$IOS_SIM_UDID" ]; then
  IOS_SIM_UDID=$(xcrun simctl list devices 2>/dev/null | awk -F '[()]' '/Booted/{print $2; exit}')
fi
if [ -z "$IOS_SIM_UDID" ]; then
  SIM=$(xcrun simctl list devices available 2>/dev/null | \
    awk -F '[()]' '/iPhone 16/{print $2; exit}')
  [ -z "$SIM" ] && SIM=$(xcrun simctl list devices available 2>/dev/null | \
    awk -F '[()]' '/iPhone/{print $2; exit}')
  if [ -n "$SIM" ]; then
    xcrun simctl boot "$SIM" >/dev/null 2>&1 && IOS_SIM_UDID="$SIM" || true
  fi
fi

# ── Check if iOS device has SCMessenger running ──────────────────────────────
ios_dev_running() {
  [ -z "$IOS_DEVICE_UDID" ] && return 1
  xcrun devicectl device info processes \
    --device "$IOS_DEVICE_UDID" 2>/dev/null | \
    grep -qiE "SCMessenger\.app/SCMessenger|SCMessenger$"
}

# ── Check if iOS sim app is running ───────────────────────────────────────────
ios_sim_running() {
  [ -z "$IOS_SIM_UDID" ] && return 1
  xcrun simctl listapps "$IOS_SIM_UDID" 2>/dev/null | grep -q "$BUNDLE_ID" || return 1
  xcrun simctl spawn "$IOS_SIM_UDID" pgrep -f "SCMessenger" >/dev/null 2>&1
}

# ═══════════════════════════════════════════════════════════════════════════════
echo "Phase 1: Node Status Audit"
echo ""

# 1. GCP -----------------------------------------------------------------------
printf "  [1/5] GCP headless relay   ... "
GCP_CID=$(gcp_ssh "sudo docker ps --filter name=$GCP_CONTAINER_NAME --format '{{.ID}}' | head -n1" 2>/dev/null || true)
if [ -z "${GCP_CID:-}" ]; then
  GCP_CID=$(gcp_ssh "sudo docker ps --filter ancestor=$GCP_IMAGE --format '{{.ID}}' | head -n1" 2>/dev/null || true)
fi
GCP_RUNNING=0
if [ -n "${GCP_CID:-}" ]; then
  GCP_VERSION=$(gcp_ssh "sudo docker inspect --format='{{.Config.Image}}' $GCP_CID" 2>/dev/null | grep -oE 'cli:[^\"]+' || echo "latest")
  echo "[OK] running  (container $GCP_CID  image: $GCP_VERSION)"
  GCP_RUNNING=1
else
  echo "[FAIL] NOT RUNNING"
fi

# 2. OSX -----------------------------------------------------------------------
printf "  [2/5] OSX headless relay   ... "
OSX_RUNNING=0
if pgrep -f "scmessenger-cli.*relay" >/dev/null 2>&1; then
  OSX_PID=$(pgrep -f "scmessenger-cli.*relay" | head -1)
  echo "[OK] running  (pid $OSX_PID  binary: $OSX_RELAY_CMD)"
  OSX_RUNNING=1
else
  echo "[FAIL] NOT RUNNING  (will start: $OSX_RELAY_CMD)"
fi

# 3. Android -------------------------------------------------------------------
printf "  [3/5] Android full node    ... "
ANDROID_RUNNING=0
if [ "$ANDROID_AVAILABLE" = "1" ]; then
  if adb -s "$ADB_SERIAL" shell pidof com.scmessenger.android >/dev/null 2>&1; then
    echo "[OK] running  (serial: $ADB_SERIAL)"
    ANDROID_RUNNING=1
  else
    ANDROID_STATE=$(adb -s "$ADB_SERIAL" shell pm list packages com.scmessenger.android 2>/dev/null | grep -c "com.scmessenger.android" || echo 0)
    [[ "$ANDROID_STATE" =~ ^[0-9]+$ ]] || ANDROID_STATE=0
    if [ "$ANDROID_STATE" -gt 0 ]; then
       echo "[WARNING] installed, NOT running  (serial: $ADB_SERIAL)"
    else
       echo "[FAIL] not installed"
    fi
  fi
else
  echo "[WARNING] no device  (adb: none found)"
fi

# 4. iOS Device ----------------------------------------------------------------
printf "  [4/5] iOS Device full node ... "
IOS_DEV_RUNNING=0
if [ -n "$IOS_DEVICE_UDID" ]; then
  if ios_dev_running; then
    echo "[OK] running  (udid: ${IOS_DEVICE_UDID:0:8}…)"
    IOS_DEV_RUNNING=1
  else
    echo "[WARNING] device found, SCMessenger NOT running  (udid: ${IOS_DEVICE_UDID:0:8}…)"
  fi
else
  echo "[WARNING] no device connected"
fi

# 5. iOS Simulator -------------------------------------------------------------
printf "  [5/5] iOS Simulator        ... "
IOS_SIM_RUNNING=0
if [ -n "$IOS_SIM_UDID" ]; then
  if ios_sim_running; then
    echo "[OK] running  (udid: ${IOS_SIM_UDID:0:8}…)"
    IOS_SIM_RUNNING=1
  else
    echo "[WARNING] booted, SCMessenger NOT running  (udid: ${IOS_SIM_UDID:0:8}…)"
  fi
else
  echo "[WARNING] no simulator booted"
fi

echo ""
echo "═══ Phase 2: Ensure All Nodes Running ═══════════════════════════"
echo ""

# 1. GCP -----------------------------------------------------------------------
echo "  [1/5] GCP:"
if [ "$GCP_RUNNING" = "0" ] || [ "$UPDATE_APPS" = "1" ]; then
  if [ "$UPDATE_APPS" = "1" ]; then
    echo "        Pulling latest image and restarting..."
    gcp_ssh "sudo docker pull $GCP_IMAGE 2>&1 | tail -3"  || hlog "        [WARNING] docker pull failed"
    gcp_ssh "sudo docker stop $GCP_CONTAINER_NAME 2>/dev/null; \
             sudo docker rm $GCP_CONTAINER_NAME 2>/dev/null; \
             sudo docker run -d --restart=unless-stopped \
               --name $GCP_CONTAINER_NAME \
               -p 9001:9001 \
               -p 9000:9000 \
               $GCP_IMAGE \
               scm relay --listen /ip4/0.0.0.0/tcp/9001 --http-port 9000" \
      || hlog "        [WARNING] GCP restart failed"
    sleep 3
    GCP_CID=$(gcp_ssh "sudo docker ps --filter name=$GCP_CONTAINER_NAME --format '{{.ID}}' | head -n1" 2>/dev/null || true)
  fi
  if [ -n "${GCP_CID:-}" ]; then
    echo "        [OK] Container $GCP_CID running"
  else
    echo "        [WARNING] GCP still not running — logs will show SSH error"
  fi
else
  echo "        [OK] Already running — passive log attach only"
fi

# 2. OSX -----------------------------------------------------------------------
echo "  [2/5] OSX:"
if [ "$OSX_RUNNING" = "0" ] || [ "$UPDATE_APPS" = "1" ]; then
  if [ "$UPDATE_APPS" = "1" ]; then
    echo "        Rebuilding binary..."
    cargo build -p scmessenger-cli 2>&1 | tail -3 || hlog "        [WARNING] cargo build failed"
    pkill -f "scmessenger-cli.*relay" 2>/dev/null || true; sleep 0.5
  fi
  echo "        Starting relay (nohup, binary: $OSX_RELAY_CMD)..."
  RUST_LOG="$OSX_RUST_LOG" \
    nohup $OSX_RELAY_CMD relay \
      --listen /ip4/0.0.0.0/tcp/9010 \
      --http-port 9011 \
    >> "$LOGDIR/osx.log" 2>&1 &
  OSX_PID=$!
  STARTED_OSX=1
  sleep 2
  if pid_alive "$OSX_PID"; then
    echo "        [OK] Started (pid $OSX_PID)"
  else
    echo "        [FAIL] Process died immediately — check $LOGDIR/osx.log"
    hlog "OSX relay failed to start"
  fi
else
  echo "        [OK] Already running (pid $OSX_PID)"
fi

# 3. Android -------------------------------------------------------------------
echo "  [3/5] Android:"
if [ "$ANDROID_AVAILABLE" = "1" ]; then
  if [ "$ANDROID_RUNNING" = "0" ]; then
    echo "        Gentle launch (am start, no force-stop)..."
    adb -s "$ADB_SERIAL" shell am start \
      -n com.scmessenger.android/.ui.MainActivity >/dev/null 2>&1 || true
    STARTED_ANDROID_APP=1
    sleep 1
    if adb -s "$ADB_SERIAL" shell pidof com.scmessenger.android >/dev/null 2>&1; then
      echo "        [OK] Launched"
    else
      echo "        [WARNING] Launch command sent but app PID not found yet"
    fi
  else
    echo "        [OK] Already running — passive logcat only"
  fi
else
  echo "        [WARNING] Skipped (no device)"
fi

# 4. iOS Device ----------------------------------------------------------------
echo "  [4/5] iOS Device:"
if [ -n "$IOS_DEVICE_UDID" ]; then
  if [ "$IOS_DEV_RUNNING" = "0" ]; then
    echo "        Launching existing install (no reinstall)..."
    xcrun devicectl device process launch \
      --device "$IOS_DEVICE_UDID" \
      --terminate-existing \
      --no-activate \
      "$BUNDLE_ID" >/dev/null 2>&1 || true
    STARTED_IOS_DEV_APP=1
    sleep 1
    if ios_dev_running; then
      echo "        [OK] Launched"
    else
      echo "        [WARNING] Launch sent — app may take a moment to appear"
    fi
  else
    echo "        [OK] Already running — passive log stream only"
  fi
else
  echo "        [WARNING] Skipped (no device)"
fi

# 5. iOS Sim -------------------------------------------------------------------
echo "  [5/5] iOS Sim:"
if [ -n "$IOS_SIM_UDID" ]; then
  if [ "$IOS_SIM_RUNNING" = "0" ]; then
    echo "        Launching existing install..."
    xcrun simctl launch "$IOS_SIM_UDID" "$BUNDLE_ID" >/dev/null 2>&1 || true
    STARTED_IOS_SIM_APP=1
    sleep 0.5
    echo "        [OK] Launched"
  else
    echo "        [OK] Already running — passive log stream only"
  fi
else
  echo "        [WARNING] Skipped"
fi

echo ""
echo "═══ Phase 3: Attach Passive Log Collectors ══════════════════════"
echo ""

# ── Seed node identities for visualizer ───────────────────────────────────────
hlog "Seeding node identities for visualizer..."
# Android
if [ "$ANDROID_AVAILABLE" = "1" ]; then
  # Try to extraction from history to ensure visualizer sees it immediately
  (
    PID=$(adb -s "$ADB_SERIAL" shell pidof com.scmessenger.android 2>/dev/null | tr -d '\r\n' || true)
    if [ -n "$PID" ]; then
      OWN_ID=$(adb -s "$ADB_SERIAL" logcat -d --pid="$PID" 2>/dev/null | grep -m 1 -oE "12D3KooW[a-zA-Z0-9]{44,}" | tail -1 || true)
      if [ -n "$OWN_ID" ]; then
        echo "SC_IDENTITY_OWN p2p_id=$OWN_ID (seeded from logcat history)" >> "$LOGDIR/android.log"
      fi
    fi
  ) &
fi

# iOS Device
if [ -n "$IOS_DEVICE_UDID" ]; then
  (
    # Check if we have a recent diagnostics log we can pull ID from
    DIAG_FILE="$ROOT_DIR/ios_diagnostics_latest.log"
    if [ -f "$DIAG_FILE" ]; then
       OWN_ID=$(grep -oE "12D3KooW[a-zA-Z0-9]{44,}" "$DIAG_FILE" | tail -1 || true)
       if [ -n "$OWN_ID" ]; then
         echo "SC_IDENTITY_OWN p2p_id=$OWN_ID (seeded from diag history)" >> "$LOGDIR/ios-device.log"
       fi
    fi
  ) &
fi
# ──────────────────────────────────────────────────────────────────────────────

# ── GCP: stream docker logs with SSH keepalive ────────────────────────────────
{
  printf "\n%s\n" "$SYNC_MARKER"
  if ! gcp_ssh "CID=\$(sudo docker ps --filter name=$GCP_CONTAINER_NAME --format '{{.ID}}' | head -n1); \
                if [ -z \"\$CID\" ]; then \
                  CID=\$(sudo docker ps --filter ancestor=$GCP_IMAGE --format '{{.ID}}' | head -n1); \
                fi; \
                if [ -n \"\$CID\" ]; then \
                  sudo docker logs --tail 200 \"\$CID\" 2>&1; \
                else \
                  echo 'ERROR: No running GCP container'; \
                fi"; then
    echo "ERROR: GCP SSH unreachable via direct and IAP tunnel"
  fi
  GCP_LAST_SINCE=$(date -u +'%Y-%m-%dT%H:%M:%SZ')
  while true; do
    if ! gcp_ssh "CID=\$(sudo docker ps --filter name=$GCP_CONTAINER_NAME --format '{{.ID}}' | head -n1); \
                  if [ -z \"\$CID\" ]; then \
                    CID=\$(sudo docker ps --filter ancestor=$GCP_IMAGE --format '{{.ID}}' | head -n1); \
                  fi; \
                  if [ -n \"\$CID\" ]; then \
                    sudo docker logs --since \"$GCP_LAST_SINCE\" \"\$CID\" 2>&1; \
                  else \
                    echo 'ERROR: No running GCP container'; \
                  fi"; then
      echo "ERROR: GCP SSH unreachable via direct and IAP tunnel"
    fi
    GCP_LAST_SINCE=$(date -u +'%Y-%m-%dT%H:%M:%SZ')
    sleep 15
  done
} >> "$LOGDIR/gcp.log" 2>&1 &
GCP_LOG_PID=$!
echo "  [1/5] GCP log stream      PID=$GCP_LOG_PID  → $(basename $LOGDIR)/gcp.log"

# ── OSX: output already going to logdir if we started it; otherwise tail ──────
printf "\n%s\n" "$SYNC_MARKER" >> "$LOGDIR/osx.log"
if [ "$STARTED_OSX" = "0" ] && [ -n "$OSX_PID" ]; then
  PREV_LOG=$(find logs/5mesh -name "osx.log" \
               -not -path "*/$TIMESTAMP/*" \
               -not -path "*/latest/*" 2>/dev/null | \
             sort -r | head -1 || true)
  if [ -n "$PREV_LOG" ]; then
    tail -F "$PREV_LOG" >> "$LOGDIR/osx.log" 2>/dev/null &
    OSX_LOG_PID=$!
    echo "  [2/5] OSX relay tail      PID=$OSX_LOG_PID  → $(basename $LOGDIR)/osx.log  (from prev run)"
  else
    echo "  [2/5] OSX relay           pre-existing, no prior log — new output not capturable post-hoc"
    OSX_LOG_PID=""
  fi
else
  OSX_LOG_PID="${OSX_PID:-}"
  echo "  [2/5] OSX relay output    PID=$OSX_PID    → $(basename $LOGDIR)/osx.log"
fi

# ── Android: pinned to serial, buffer starts NOW (-T 1) ──────────────────────
if [ "$ANDROID_AVAILABLE" = "1" ]; then
  printf "\n%s\n" "$SYNC_MARKER" > "$LOGDIR/android.log"
  adb -s "$ADB_SERIAL" logcat -v threadtime -T 1 --pid="$(adb -s "$ADB_SERIAL" shell pidof com.scmessenger.android | tr -d '\r\n')" \
    >> "$LOGDIR/android.log" 2>&1 &
  ANDROID_LOGCAT_PID=$!
  echo "  [3/5] Android logcat      PID=$ANDROID_LOGCAT_PID → $(basename $LOGDIR)/android.log  (serial: $ADB_SERIAL)"
else
  echo "  [3/5] Android             skipped"
fi

# ── iOS Device: console stream (captures stdout from SCMessenger) ─────────────
if [ -n "$IOS_DEVICE_UDID" ]; then
  printf "\n%s\n" "$SYNC_MARKER" > "$LOGDIR/ios-device.log"
  printf "\n%s\n" "$SYNC_MARKER" > "$LOGDIR/ios-device-system.log"

  if [ "$IOS_DEV_RUNNING" = "1" ] && [ "$STARTED_IOS_DEV_APP" = "0" ]; then
    IOS_DEV_LAUNCH_PID=""
    {
      echo "NOTE: iOS app was already running before run5.sh started."
      echo "NOTE: passive physical-device app-console capture is unavailable without relaunch."
      echo "NOTE: ios-device-system.log remains valid for radio/Bluetooth/Multipeer context."
    } >> "$LOGDIR/ios-device.log"
  else
    xcrun devicectl device process launch \
      --device "$IOS_DEVICE_UDID" \
      --console \
      --no-activate \
      "$BUNDLE_ID" \
      >> "$LOGDIR/ios-device.log" 2>&1 &
    IOS_DEV_LAUNCH_PID=$!
  fi

  # System log stream: BLE + MPC + App subsystems
  log stream \
    --style compact \
    --level info \
    --predicate 'process == "SCMessenger" OR subsystem == "com.apple.bluetooth" OR subsystem == "com.apple.MultipeerConnectivity" OR subsystem == "com.scmessenger"' \
    >> "$LOGDIR/ios-device-system.log" 2>&1 &
  IOS_DEV_STREAM_PID=$!

  if [ -n "${IOS_DEV_LAUNCH_PID:-}" ]; then
    echo "  [4/5] iOS Device          Launch=$IOS_DEV_LAUNCH_PID  App=$(basename $LOGDIR)/ios-device.log  System=$(basename $LOGDIR)/ios-device-system.log"
  else
    echo "  [4/5] iOS Device          App=passive-unavailable  App=$(basename $LOGDIR)/ios-device.log  System=$(basename $LOGDIR)/ios-device-system.log"
  fi
else
  echo "  [4/5] iOS Device          skipped (no device)"
fi

# ── iOS Simulator: log stream (reliable, subsystem-filtered) ─────────────────
if [ -n "$IOS_SIM_UDID" ]; then
  printf "\n%s\n" "$SYNC_MARKER" > "$LOGDIR/ios-sim.log"
  xcrun simctl spawn "$IOS_SIM_UDID" log stream \
    --level info \
    --style compact \
    --predicate 'process == "SCMessenger" OR subsystem == "com.scmessenger"' \
    >> "$LOGDIR/ios-sim.log" 2>&1 &
  IOS_SIM_STREAM_PID=$!
  echo "  [5/5] iOS Sim             PID=$IOS_SIM_STREAM_PID → $(basename $LOGDIR)/ios-sim.log"
else
  echo "  [5/5] iOS Sim             skipped"
fi

# ── Sanity check all collectors are alive after 5s ───────────────────────────
sleep 5
echo ""
echo "  Collector health check (5s after start):"
_chk() {
  local name=$1 pid=$2
  if [ -z "$pid" ]; then
    printf "    %-20s skipped\n" "$name"
  elif pid_alive "$pid"; then
    local sz
    sz=$(wc -c < "$LOGDIR/${3:-/dev/null}" 2>/dev/null | tr -d ' ') || sz=0
    printf "    %-20s [OK] alive  (pid %-6s %s bytes)\n" "$name" "$pid" "$sz"
  else
    printf "    %-20s [FAIL] died!\n" "$name"
    hlog "WARN: $name log collector (pid $pid) died early"
  fi
}
_chk "GCP log stream"    "$GCP_LOG_PID"         "gcp.log"
_chk "OSX relay"         "$OSX_LOG_PID"         "osx.log"
_chk "Android logcat"    "$ANDROID_LOGCAT_PID"  "android.log"
_chk "iOS Dev launch"    "$IOS_DEV_LAUNCH_PID"  "ios-device.log"
_chk "iOS Dev system"    "$IOS_DEV_STREAM_PID"  "ios-device-system.log"
_chk "iOS Sim stream"    "$IOS_SIM_STREAM_PID"  "ios-sim.log"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
printf "║  All nodes up. Auto-stopping in %-28s ║\n" "${DURATION_MIN}m."
printf "║  Logs → %-52s ║\n" "$LOGDIR/"
echo "║  Ctrl+C to stop early.                                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ── Live status ticker ─────────────────────────────────────────────────────────
status_ticker() {
  local iter=0
  sleep 10
  while true; do
    iter=$((iter+1))
    echo ""
    echo "── $(date '+%H:%M:%S') Status (tick $iter) ──────────────────────────────"

    # GCP
    GCP_LINES=$(wc -l < "$LOGDIR/gcp.log" 2>/dev/null | tr -d ' \n')
    GCP_CIRCUITS=$(grep -c "circuit\|Circuit" "$LOGDIR/gcp.log" 2>/dev/null || echo 0)
    printf "  GCP:     %-5s lines  %-5s circuit events" "$GCP_LINES" "$GCP_CIRCUITS"
    pid_alive "${GCP_LOG_PID:-0}" 2>/dev/null && echo " [OK]" || echo " [FAIL] stream dead"

    # OSX
    OSX_RELAYS=$(grep -c "Relay circuit reservation" "$LOGDIR/osx.log" 2>/dev/null || echo 0)
    OSX_PEERS=$(grep -oE "12D3KooW[A-Za-z0-9]+" "$LOGDIR/osx.log" 2>/dev/null | sort -u | wc -l | tr -d ' \n')
    printf "  OSX:     %-5s peers  %-5s relay reservations" "$OSX_PEERS" "$OSX_RELAYS"
    pid_alive "${OSX_LOG_PID:-0}" 2>/dev/null && echo " [OK]" || echo " (external)"

    # Android
    if [ -f "$LOGDIR/android.log" ]; then
      ANDROID_PEERS=$(grep -Ec "IdentityDiscovered|PeerEvent emitted: (Connected|Disconnected)|Peer.*identif|peer_identified" "$LOGDIR/android.log" 2>/dev/null || echo 0)
      ANDROID_BLE=$(grep -Ec "Identity beacon from|Characteristic write successful|delivery_attempt .*medium=ble|Delivery via BLE|BleGatt" "$LOGDIR/android.log" 2>/dev/null || echo 0)
      ANDROID_LINES=$(wc -l < "$LOGDIR/android.log" 2>/dev/null | tr -d ' \n')
      ANDROID_NAT=$(grep "NAT status" "$LOGDIR/android.log" 2>/dev/null | tail -1 | grep -oE 'Public|Private|Unknown' || echo "?")
      printf "  Android: %-5s lines  peer=%-5s ble=%-5s NAT=%s" "$ANDROID_LINES" "$ANDROID_PEERS" "$ANDROID_BLE" "$ANDROID_NAT"
      pid_alive "${ANDROID_LOGCAT_PID:-0}" 2>/dev/null && echo " [OK]" || echo " [FAIL] logcat dead"
    else
      echo "  Android: no log"
    fi

    # iOS Dev
    if [ -f "$LOGDIR/ios-device.log" ]; then
      IOS_DEV_APP_LINES=$(wc -l < "$LOGDIR/ios-device.log" 2>/dev/null | tr -d ' \n')
      IOS_DEV_APP_PEERS=$(grep -Ec "Peer.*identif|peer_identified|IdentityDiscovered|Connected\\(peerId|Starting Swarm with PeerID|Initialized core for peer id|local_peer_id" "$LOGDIR/ios-device.log" 2>/dev/null || echo 0)
      IOS_DEV_SYS_LINES=0
      IOS_DEV_RADIO=0
      if [ -f "$LOGDIR/ios-device-system.log" ]; then
        IOS_DEV_SYS_LINES=$(wc -l < "$LOGDIR/ios-device-system.log" 2>/dev/null | tr -d ' \n')
        IOS_DEV_RADIO=$(grep -Ec "bluetoothd|com\\.apple\\.bluetooth|com\\.apple\\.MultipeerConnectivity|CB[A-Z]" "$LOGDIR/ios-device-system.log" 2>/dev/null || echo 0)
      fi
      printf "  iOS Dev: app=%-5s peer=%-5s system=%-5s radio=%-5s" "$IOS_DEV_APP_LINES" "$IOS_DEV_APP_PEERS" "$IOS_DEV_SYS_LINES" "$IOS_DEV_RADIO"
      pid_alive "${IOS_DEV_STREAM_PID:-0}" 2>/dev/null && echo " [OK]" || echo " [FAIL] stream dead"
    else
      echo "  iOS Dev: no log"
    fi

    # iOS Sim
    if [ -f "$LOGDIR/ios-sim.log" ]; then
      IOS_SIM_LINES=$(wc -l < "$LOGDIR/ios-sim.log" 2>/dev/null | tr -d ' \n')
      IOS_SIM_PEERS=$(grep -Ec "Peer.*identif|peer_identified|IdentityDiscovered|Connected\\(peerId|Starting Swarm with PeerID|Initialized core for peer id|local_peer_id" "$LOGDIR/ios-sim.log" 2>/dev/null || echo 0)
      printf "  iOS Sim: %-5s lines  %-5s peer events" "$IOS_SIM_LINES" "$IOS_SIM_PEERS"
      pid_alive "${IOS_SIM_STREAM_PID:-0}" 2>/dev/null && echo " [OK]" || echo " [FAIL] stream dead"
    else
      echo "  iOS Sim: no log"
    fi

    # Notable recent events
    RECENT=$(grep -hE "[OK] Relay| NAT| DCUtR|Peer.*identif|delivery_attempt .*medium=(ble|core|relay-circuit)|isFull=true" \
      "$LOGDIR"/*.log 2>/dev/null | tail -4)
    if [ -n "$RECENT" ]; then
      echo "  Recent:"
      echo "$RECENT" | sed 's/^/    /'
    fi
    echo "────────────────────────────────────────────────"
    sleep 15
  done
}
status_ticker &
TICKER_PID=$!

# ── Shutdown: release only what we own ────────────────────────────────────────
shutdown() {
  echo ""
  echo "Shutting down..."

  kill "$GCP_LOG_PID"     2>/dev/null || true
  [ -n "${OSX_LOG_PID:-}" ]          && kill "$OSX_LOG_PID"        2>/dev/null || true
  [ -n "${ANDROID_LOGCAT_PID:-}" ]   && kill "$ANDROID_LOGCAT_PID" 2>/dev/null || true
  [ -n "${IOS_DEV_LAUNCH_PID:-}" ]   && kill "$IOS_DEV_LAUNCH_PID" 2>/dev/null || true
  [ -n "${IOS_DEV_STREAM_PID:-}" ]   && kill "$IOS_DEV_STREAM_PID" 2>/dev/null || true
  [ -n "${IOS_SIM_STREAM_PID:-}" ]   && kill "$IOS_SIM_STREAM_PID" 2>/dev/null || true
  [ -n "${TICKER_PID:-}" ]           && kill "$TICKER_PID"         2>/dev/null || true

  # Stop nodes we started (only if --restore-on-exit or default behavior)
  if [ "$RESTORE_ON_EXIT" = "1" ]; then
    if [ "$STARTED_OSX" = "1" ] && [ -n "${OSX_PID:-}" ]; then
      echo "  Stopping OSX relay (restore-on-exit)..."
      kill "$OSX_PID" 2>/dev/null || true
    else
      echo "  OSX relay: pre-existing — left running [OK]"
    fi

    if [ "$STARTED_ANDROID_APP" = "1" ] && [ "$ANDROID_AVAILABLE" = "1" ]; then
      echo "  Stopping Android app (restore-on-exit)..."
      adb -s "$ADB_SERIAL" shell am force-stop com.scmessenger.android >/dev/null 2>&1 || true
    else
      echo "  Android: pre-existing — left running [OK]"
    fi

    if [ "$STARTED_IOS_DEV_APP" = "1" ] && [ -n "${IOS_DEVICE_UDID:-}" ]; then
      echo "  Stopping iOS Device app (restore-on-exit)..."
      xcrun devicectl device process terminate \
        --device "$IOS_DEVICE_UDID" \
        --bundle-id "$BUNDLE_ID" 2>/dev/null || true
    else
      [ -n "${IOS_DEVICE_UDID:-}" ] && echo "  iOS Device: pre-existing — left running [OK]"
    fi

    if [ "$STARTED_IOS_SIM_APP" = "1" ] && [ -n "${IOS_SIM_UDID:-}" ]; then
      echo "  Stopping iOS Sim app (restore-on-exit)..."
      xcrun simctl terminate "$IOS_SIM_UDID" "$BUNDLE_ID" 2>/dev/null || true
    else
      [ -n "${IOS_SIM_UDID:-}" ] && echo "  iOS Sim: pre-existing — left running [OK]"
    fi
  else
    echo "  restore-on-exit disabled: leaving all started nodes running [OK]"
  fi

  echo ""

  # ── Post-run analysis ──────────────────────────────────────────────────────
  echo "╔══════════════════════════════════════════════════════════════╗"
  echo "║   Post-Run Mesh Analysis                                    ║"
  echo "╚══════════════════════════════════════════════════════════════╝"
  echo ""

  LOGDIR_SNAP="$LOGDIR"
  if command -v python3 >/dev/null 2>&1; then
    python3 - "$LOGDIR_SNAP" <<'PY'
import os, re, sys

LOGDIR = sys.argv[1]
all_logs = {
    'gcp':     os.path.join(LOGDIR, 'gcp.log'),
    'osx':     os.path.join(LOGDIR, 'osx.log'),
    'android': os.path.join(LOGDIR, 'android.log'),
    'ios_dev': os.path.join(LOGDIR, 'ios-device.log'),
    'ios_dev_system': os.path.join(LOGDIR, 'ios-device-system.log'),
    'ios_sim': os.path.join(LOGDIR, 'ios-sim.log'),
}
visible_nodes = ['gcp', 'osx', 'android', 'ios_dev', 'ios_sim']
NODE_TYPES = {'gcp':'Headless','osx':'Headless','android':'Full','ios_dev':'Full','ios_sim':'Full'}
PAT = re.compile(r"(12D3KooW[1-9A-HJ-NP-Za-km-z]{44,})")
OWN_ID_PATTERNS = {
    'gcp': [
        re.compile(r'===\s*OWN_IDENTITY:\s*(12D3KooW[a-zA-Z0-9]{44,})\s*==='),
        re.compile(r'Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    ],
    'osx': [
        re.compile(r'===\s*OWN_IDENTITY:\s*(12D3KooW[a-zA-Z0-9]{44,})\s*==='),
        re.compile(r'Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    ],
    'android': [
        re.compile(r'Mesh service started.*?libp2pPeerId=\s*(12D3KooW[a-zA-Z0-9]{44,})'),
        re.compile(r'Rust\s*:\s*Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
        re.compile(r'Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
        re.compile(r'Initialized core for peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    ],
    'ios_dev': [
        re.compile(r'local_peer_id\s*=\s*(12D3KooW[a-zA-Z0-9]{44,})'),
        re.compile(r'Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
        re.compile(r'SwarmBridge with peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})'),
        re.compile(r'Initialized core for peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    ],
    'ios_sim': [
        re.compile(r'local_peer_id\s*=\s*(12D3KooW[a-zA-Z0-9]{44,})'),
        re.compile(r'Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
        re.compile(r'SwarmBridge with peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})'),
        re.compile(r'Initialized core for peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    ],
}
APP_PEER_PAT = re.compile(r'(Peer.*identif|peer_identified|IdentityDiscovered|Connected\(peerId|Peer discovered:|peer.*connect)', re.I)
BLE_PAT = re.compile(r'(Identity beacon from|BLE.*identity|BleGatt|Characteristic write successful|Delivery via BLE|delivery_attempt .*medium=ble|CBPeripheral|CBCentralManager)', re.I)
DIRECT_PAT = re.compile(r'(delivery_attempt .*medium=core phase=direct|[OK] Direct delivery ACK|direct delivery)', re.I)
RELAY_PAT = re.compile(r'(Relay circuit reservation|reservation registered|Relaying message|delivery_attempt .*medium=relay-circuit|relay-circuit route=|Lost relay peer|p2p-circuit)', re.I)
WIFI_PAT = re.compile(r'(WiFi Direct|wifi.?direct|NearbyMediums|transport=WiFi|transport=WIFI|multipeer)', re.I)
RADIO_PAT = re.compile(r'(bluetoothd|com\.apple\.bluetooth|com\.apple\.MultipeerConnectivity|MultipeerConnectivity)', re.I)
CONNECT_PAT = re.compile(r'(connected|PeerConnected|peer.*connect)', re.I)
ERROR_PAT   = re.compile(r'(Failed to negotiate|connection error|ERR|all_transports_failed)', re.I)
SENT_PAT    = re.compile(r'([OK] Direct delivery ACK|outcome=success|sent message)', re.I)
RECV_PAT    = re.compile(r'([OK] Received message|msg_rx_processed|receive_message)', re.I)
NAT_PAT     = re.compile(r'AutoNAT.*?(Public|Private|Unknown)', re.I)

def read(path):
    try:
        with open(path, 'r', errors='ignore') as f: return f.read()
    except: return ""

def strip_ansi(s): return re.sub(r'\x1b\[[^m]*m', '', s)

contents = {n: strip_ansi(read(p)) for n, p in all_logs.items()}

def find_own_id(name, content):
    for pat in OWN_ID_PATTERNS.get(name, []):
        m = pat.search(content)
        if m and len(m.group(1)) >= 52:
            return m.group(1)
    return None

file_to_id = {name: find_own_id(name, contents.get(name, "")) for name in visible_nodes}
duplicates = {}
for name, pid in file_to_id.items():
    if not pid:
        continue
    duplicates.setdefault(pid, []).append(name)

ambiguous_ids = {pid: names for pid, names in duplicates.items() if len(names) > 1}
for pid, names in ambiguous_ids.items():
    for name in names:
        file_to_id[name] = None

matrix = {name: set(PAT.findall(contents.get(name, ""))) for name in visible_nodes}

def count_matches(pattern, text):
    if not text:
        return 0
    return len(pattern.findall(text))

metrics = {}
for name in visible_nodes:
    content = contents.get(name, "")
    nat_m = NAT_PAT.findall(content)
    metrics[name] = {
        'peer_ids_seen': len(set(PAT.findall(content))),
        'app_peers': count_matches(APP_PEER_PAT, content),
        'ble': count_matches(BLE_PAT, content),
        'direct': count_matches(DIRECT_PAT, content),
        'relay': count_matches(RELAY_PAT, content),
        'wifi': count_matches(WIFI_PAT, content),
        'sent': count_matches(SENT_PAT, content),
        'recv': count_matches(RECV_PAT, content),
        'conns': count_matches(CONNECT_PAT, content),
        'errs': count_matches(ERROR_PAT, content),
        'nat': nat_m[-1].lower() if nat_m else '?',
        'lines': content.count('\n'),
    }

metrics['ios_dev']['radio'] = count_matches(RADIO_PAT, contents.get('ios_dev_system', ''))
metrics['ios_dev']['system_lines'] = contents.get('ios_dev_system', '').count('\n')

# Header
print(f"  {'Node':<10} {'Own ID':<26} {'Sent':>5} {'Recv':>5} {'Relay':>6} {'Conns':>6} {'NAT':<9} {'Errors':>7}")
print("  " + "─" * 88)
for name in visible_nodes:
    c = contents.get(name, "")
    pid = file_to_id.get(name) or 'unknown'
    pid_d = (pid[:22] + '..') if len(pid) > 22 else pid
    lines = metrics[name]['lines']
    sent = metrics[name]['sent']
    recv = metrics[name]['recv']
    relays = metrics[name]['relay']
    nat = metrics[name]['nat']
    conns = metrics[name]['conns']
    errs = metrics[name]['errs']
    has_content = lines > 2
    icon = '[OK]' if (pid != 'unknown' or has_content) else '[FAIL]'
    print(f"  {icon} {name:<8} {pid_d:<26} {sent:>5} {recv:>5} {relays:>6} {conns:>6} {nat:<9} {errs:>7}")

print()
print("  Transport Evidence (app-level evidence, not inferred visibility):")
print(f"  {'Node':<10} {'PeerIDs':>7} {'App':>5} {'BLE':>5} {'Direct':>7} {'Relay':>6} {'WiFi':>5} {'Notes'}")
print("  " + "─" * 96)
for name in visible_nodes:
    note_parts = []
    if name == 'ios_dev':
        radio = metrics[name].get('radio', 0)
        if radio and metrics[name]['app_peers'] == 0:
            note_parts.append('system-radio-only')
        elif radio:
            note_parts.append(f"system-radio={radio}")
    if file_to_id.get(name) is None and metrics[name]['lines'] > 0:
        note_parts.append('own-id-not-captured')
    if metrics[name]['ble'] > 0 and metrics[name]['app_peers'] == 0 and name != 'ios_dev':
        note_parts.append('ble-without-app-peer-id')
    note = ', '.join(note_parts) or '-'
    print(
        f"  {'[OK]' if metrics[name]['lines'] > 2 else '[WARNING] ':<2} {name:<8} "
        f"{metrics[name]['peer_ids_seen']:>7} {metrics[name]['app_peers']:>5} "
        f"{metrics[name]['ble']:>5} {metrics[name]['direct']:>7} {metrics[name]['relay']:>6} "
        f"{metrics[name]['wifi']:>5} {note}"
    )

print()
print("  Visibility Matrix (known own IDs only; unknown collector gaps are not counted as failures):")
print(f"  {'Node':<10} {'Known Seen':<12} {'Unknown IDs':<20} Missing Known")
print("  " + "─" * 92)
all_ok = True
unknown_visibility = False
for name in visible_nodes:
    seen = matrix[name]
    missing = []
    unknown = []
    expected_known = 0
    seen_known = 0
    for other in visible_nodes:
        if other == name:
            continue
        oid = file_to_id.get(other)
        if not oid:
            unknown.append(other)
            unknown_visibility = True
            continue
        expected_known += 1
        if oid in seen:
            seen_known += 1
        else:
            missing.append(other)
            all_ok = False
    if missing:
        icon = '[FAIL]'
    elif unknown:
        icon = '[WARNING] '
    else:
        icon = '[OK]'
    print(f"  {icon} {name:<8} {seen_known}/{expected_known:<10} {', '.join(unknown) or 'none':<20} {', '.join(missing) or 'none'}")

print()
if ambiguous_ids:
    print("  [WARNING] Suppressed ambiguous own IDs:")
    for pid, names in sorted(ambiguous_ids.items()):
        short_pid = pid[:22] + '..'
        print(f"     {short_pid}: {', '.join(names)}")
    print()

if all_ok and not unknown_visibility:
    print("   FULL MESH — All nodes visible to all peers!")
else:
    if unknown_visibility:
        unknown_nodes = [name for name in visible_nodes if not file_to_id.get(name)]
        print("  [WARNING] Visibility is partially indeterminate.")
        print(f"     Own IDs not captured in current log window: {', '.join(unknown_nodes)}")
        print("     Treat those as collector gaps unless transport evidence above is also empty.")
    else:
        gaps = sum(
            1
            for n in visible_nodes
            for o in visible_nodes
            if o != n and file_to_id.get(o) and file_to_id.get(o) not in matrix[n]
        )
        total = len(visible_nodes) * (len(visible_nodes) - 1)
        pct = int(100 * (total - gaps) / total) if total else 0
        print(f"  [WARNING] Partial mesh — {gaps}/{total} known-ID gap(s)  ({pct}% connected)")
    print("     Tip: longer runs improve peer-ID propagation, but app/system log split now makes collector blind spots explicit.")

print()
# Log file health summary
print("  Log file health:")
for name, path in all_logs.items():
    if os.path.exists(path):
        sz = os.path.getsize(path)
        lines = contents[name].count('\n')
        icon = '[OK]' if lines > 5 else ('[WARNING]' if lines > 0 else '[FAIL]')
        print(f"    {icon} {name:<10} {lines:>6} lines  {sz:>8} bytes  {path}")
    else:
        print(f"    [FAIL] {name:<10} (no log file)")
PY
  else
    echo "  python3 not found — log file sizes:"
    for log in "$LOGDIR"/*.log; do
      [ -f "$log" ] && printf "    %-20s %s lines\n" "$(basename "$log")" "$(wc -l < "$log" | tr -d ' ')"
    done
  fi

  echo ""
  echo "Logs: $LOGDIR/"
  echo "Done."
  exit 0
}
trap "shutdown" INT TERM

# ── Auto-exit after duration ───────────────────────────────────────────────────
sleep $((DURATION_MIN * 60))
echo ""
echo "⏰ Time limit (${DURATION_MIN}m) reached."
kill -TERM $$
