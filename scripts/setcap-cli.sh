#!/usr/bin/env bash
# Post-install: grant the scmessenger-cli binary CAP_NET_BIND_SERVICE on
# Linux, so it can bind privileged ports (80/443) without running as root.
#
# Why this capability and not cap_net_raw/cap_net_admin: the CLI's BLE
# support (btleplug) talks to BlueZ entirely over the D-Bus system bus, not
# raw HCI sockets, so it needs no special Linux capabilities — BlueZ's own
# D-Bus policy governs non-root access. The actual root requirement in this
# codebase is core/src/transport/multiport.rs's privileged-port listen
# addresses (used when running as a public bootstrap/relay node offering a
# cellular-friendly WebSocket fallback on :443) — binding those
# ports below 1024 requires CAP_NET_BIND_SERVICE without root.
#
# Usage:
#   scripts/setcap-cli.sh [path-to-scmessenger-cli]
#
# Defaults to target/release/scmessenger-cli relative to the repo root.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY_PATH="${1:-$ROOT_DIR/target/release/scmessenger-cli}"

if [[ "$(uname -s)" != "Linux" ]]; then
    echo "setcap-cli.sh is a no-op outside Linux (CAP_NET_BIND_SERVICE is a Linux capability)." >&2
    exit 0
fi

if [[ ! -f "$BINARY_PATH" ]]; then
    echo "ERROR: binary not found at $BINARY_PATH" >&2
    echo "Build it first: cargo build --release -p scmessenger-cli" >&2
    exit 1
fi

if ! command -v setcap >/dev/null 2>&1; then
    echo "ERROR: setcap not found. Install it via your distro's libcap package" >&2
    echo "(e.g. 'apt install libcap2-bin' or 'dnf install libcap')." >&2
    exit 1
fi

echo "Granting CAP_NET_BIND_SERVICE to $BINARY_PATH..."
if [[ "$(id -u)" -eq 0 ]]; then
    setcap cap_net_bind_service=+ep "$BINARY_PATH"
else
    sudo setcap cap_net_bind_service=+ep "$BINARY_PATH"
fi

echo "Done. Verifying:"
getcap "$BINARY_PATH"
