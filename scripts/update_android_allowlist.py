#!/usr/bin/env python3
"""Pull the Android app's current identity via adb and (re)allow-list it.

The inbox bridge (scripts/inbox_bridge.py) only surfaces inbound messages from
identities in its allow-list. Android identity resets on reinstall / data
clear / identity reset, and the old identity silently stops matching. This
script re-learns the CURRENT identity straight off the device so the operator
lane is never silently lost.

Authoritative source: the app's own identity cache
(files/../shared_prefs/identity_cache_prefs.xml), read via `run-as` so no
root is required. This is the same data the app's Settings screen shows.

Usage:
    python scripts/update_android_allowlist.py --adb 192.168.0.134:36461
    python scripts/update_android_allowlist.py --adb 192.168.0.134:36461 --apply
    python scripts/update_android_allowlist.py --adb emulator-5554 --apply

By default it only READS the identity and compares it to the allow-list
(dry-run). Add --apply to write the updated config. The GPT-MAC identity is
always preserved; the Android identity is inserted/replaced.

Note: the running bridge reads config once at startup, so after --apply you
must restart the bridge (or the node soak) for the new allow-list to take
effect. The script prints the command to do that, but does not restart
anything itself.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

# The GPT-MAC lane identity -- always preserved in the allow-list.
GPT_MAC_IDENTITY = "3854e44295c1384854b89312e5c3925f8431b6f4c41ed66979b82b94bc93b5d7"

# Canonical bridge config path (matches scripts/inbox_bridge.py _appdata()).
def _canonical_config_path() -> Path:
    base = Path(_appdata()) / "scmessenger"
    return base / "inbox_bridge.json"


def _appdata() -> str:
    base = os.environ.get("APPDATA")
    if base:
        return base
    return str(Path.home() / ".config")


CONFIG_PATH = _canonical_config_path()


def adb(serial: str, *args) -> str:
    cmd = ["adb", "-s", serial, "shell", "run-as", "com.scmessenger.android", *args]
    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    if proc.returncode != 0:
        raise RuntimeError("adb failed (%d): %s %s" % (proc.returncode, proc.stdout, proc.stderr))
    return proc.stdout


def pull_identity(serial: str) -> dict:
    """Read identity_cache_prefs.xml and extract the identity fields."""
    xml = adb(serial, "cat", "shared_prefs/identity_cache_prefs.xml")
    def grab(name: str) -> str | None:
        m = re.search(r'name="%s">([^<]+)</string>' % re.escape(name), xml)
        return m.group(1) if m else None
    return {
        "identity_id": grab("identity_id"),
        "public_key": grab("public_key_hex"),
        "peer_id": grab("libp2p_peer_id"),
        "nickname": grab("nickname"),
        "initialized": "<boolean name=\"initialized\" value=\"true\"" in xml,
    }


def load_config() -> dict:
    if CONFIG_PATH.is_file():
        return json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
    return {"allowed_peer_id": [], "poll_interval_secs": 3, "api": "http://127.0.0.1:9876"}


def save_config(cfg: dict) -> None:
    CONFIG_PATH.parent.mkdir(parents=True, exist_ok=True)
    tmp = CONFIG_PATH.with_suffix(".json.tmp")
    tmp.write_text(json.dumps(cfg, indent=2), encoding="utf-8")
    tmp.replace(CONFIG_PATH)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--adb", required=True, help="adb device serial, e.g. 192.168.0.134:36461")
    ap.add_argument("--apply", action="store_true", help="write the updated allow-list")
    args = ap.parse_args()

    ident = pull_identity(args.adb)
    if not ident.get("identity_id"):
        print("[FAIL] could not read identity_id from identity_cache_prefs.xml")
        print("       Is the device connected and the app installed?  adb devices")
        return 1
    if not ident.get("initialized"):
        print("[WARNING] app reports initialized=false -- identity may be unset")

    print("[OK] Android identity (from device):")
    for k in ("identity_id", "public_key", "peer_id", "nickname"):
        print("  %-12s %s" % (k, ident.get(k)))

    cfg = load_config()
    allow = cfg.setdefault("allowed_peer_id", [])
    if isinstance(allow, str):
        allow = [allow]
        cfg["allowed_peer_id"] = allow

    and_id = ident["identity_id"]
    present = and_id in allow
    print()
    print("Allow-list current: %s" % (json.dumps(allow)))

    if present:
        print("[OK] Android identity is ALREADY allow-listed. No change needed.")
        return 0

    if not args.apply:
        print("[INFO] Android identity is MISSING from the allow-list.")
        print("       Re-run with --apply to add it (GPT-MAC identity is preserved).")
        return 2

    # Insert Android identity; keep GPT-MAC and dedupe, preserve order otherwise.
    new_allow = []
    for ident_ in allow:
        if ident_ != and_id and ident_ not in new_allow:
            new_allow.append(ident_)
    if GPT_MAC_IDENTITY not in new_allow:
        new_allow.insert(0, GPT_MAC_IDENTITY)
    new_allow.append(and_id)
    cfg["allowed_peer_id"] = new_allow
    save_config(cfg)

    print("[OK] Wrote allow-list to %s" % CONFIG_PATH)
    print("     Now: %s" % json.dumps(new_allow))
    print()
    print("[INFO] The running bridge read its config at startup. Restart it or the")
    print("       node soak to activate:")
    print("         taskkill /PID <bridge_pid> /F   (see soak/status.json bridge_pid)")
    print("       or restart the whole soak: python scripts/soak_supervisor.py run --with-bridge")
    return 0


if __name__ == "__main__":
    sys.exit(main())