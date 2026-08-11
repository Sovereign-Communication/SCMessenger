#!/usr/bin/env bash
# SCMessenger CLI — install locally built binary to ~/.local/bin and register a user daemon.
#
# Prerequisites: build first with `cargo build --release -p scmessenger-cli`
# Optional: SCMESSENGER_BIN=/path/to/scmessenger-cli to copy a specific artifact.
#
# Linux: writes ~/.config/systemd/user/scmessenger.service (enable manually).
# macOS: writes and loads ~/Library/LaunchAgents/io.scmessenger.cli.plist.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULT_BIN="${ROOT}/target/release/scmessenger-cli"
BIN="${SCMESSENGER_BIN:-$DEFAULT_BIN}"
DEST_DIR="${HOME}/.local/bin"
DEST="${DEST_DIR}/scmessenger-cli"

if [[ ! -f "$BIN" ]]; then
  echo "error: CLI binary not found at: $BIN" >&2
  echo "  Build with: (cd repo && cargo build --release -p scmessenger-cli)" >&2
  exit 1
fi

mkdir -p "$DEST_DIR"
cp -f "$BIN" "$DEST"
chmod +x "$DEST"
echo "Installed: $DEST"
echo "Ensure PATH includes: $DEST_DIR"

OS="$(uname -s)"
if [[ "$OS" == "Linux" ]]; then
  UNIT_DIR="${HOME}/.config/systemd/user"
  mkdir -p "$UNIT_DIR"
  cat >"${UNIT_DIR}/scmessenger.service" <<EOF
[Unit]
Description=SCMessenger CLI daemon (local mesh + Web UI on 127.0.0.1)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=${DEST} start
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF
  echo "Wrote systemd user unit: ${UNIT_DIR}/scmessenger.service"
  echo "  systemctl --user daemon-reload"
  echo "  systemctl --user enable --now scmessenger.service"
elif [[ "$OS" == "Darwin" ]]; then
  PLIST="${HOME}/Library/LaunchAgents/io.scmessenger.cli.plist"
  DATA_DIR="${HOME}/Library/Application Support/scmessenger"
  LOG_DIR="${DATA_DIR}/logs"
  LAUNCHD_LABEL="io.scmessenger.cli"
  LAUNCHD_DOMAIN="gui/$(id -u)"

  mkdir -p "$(dirname "$PLIST")"
  # Keep launchd's own stdout/stderr in the same durable log area as the CLI's
  # hourly scm.log files. mkdir -p is intentionally non-destructive.
  mkdir -p "$LOG_DIR"
  cat >"$PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>${LAUNCHD_LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>${DEST}</string>
    <string>start</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>RUST_LOG</key>
    <string>debug</string>
  </dict>
  <key>WorkingDirectory</key>
  <string>${DATA_DIR}</string>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <!-- Bound crash/restart loops while retaining launch-on-login behavior. -->
  <key>ThrottleInterval</key>
  <integer>30</integer>
  <key>StandardOutPath</key>
  <string>${LOG_DIR}/launchd.stdout.log</string>
  <key>StandardErrorPath</key>
  <string>${LOG_DIR}/launchd.stderr.log</string>
</dict>
</plist>
EOF
  echo "Wrote launchd plist: $PLIST"

  if ! plutil -lint "$PLIST" >/dev/null; then
    echo "error: generated launchd plist is invalid: $PLIST" >&2
    exit 1
  fi

  if command -v launchctl >/dev/null 2>&1; then
    # Refresh only this user's SCMessenger agent so an updated plist takes
    # effect. This does not touch SCMessenger data, identity, or pairings.
    if launchctl print "${LAUNCHD_DOMAIN}/${LAUNCHD_LABEL}" >/dev/null 2>&1; then
      launchctl bootout "${LAUNCHD_DOMAIN}/${LAUNCHD_LABEL}" >/dev/null
    fi

    if launchctl bootstrap "$LAUNCHD_DOMAIN" "$PLIST" \
      && launchctl print "${LAUNCHD_DOMAIN}/${LAUNCHD_LABEL}" >/dev/null 2>&1; then
      echo "Loaded and verified launchd agent: ${LAUNCHD_LABEL}"
    else
      echo "warning: launchd agent was written but could not be loaded and verified" >&2
      echo "  launchctl bootstrap ${LAUNCHD_DOMAIN} \"${PLIST}\"" >&2
      exit 1
    fi
  else
    echo "warning: launchctl unavailable; agent was written but not loaded" >&2
  fi
else
  echo "OS $OS: no automatic service file generated (use Task Scheduler on Windows; see scripts/install.ps1)."
fi
