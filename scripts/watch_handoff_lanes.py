#!/usr/bin/env python3
"""Dual-lane handoff monitor for SCMessenger orchestrator.

Watches BOTH lanes that carry a GPT-MAC handoff into the Windows orchestrator:

  Lane A - SCMessenger CLI (the node's local control API + inbox bridge).
           A handoff = a new inbound message from an allow-listed peer that the
           bridge has surfaced as a HANDOFF/todo/INBOX_*.md ticket or logged to
           HANDOFF/logs/chat/<date>.jsonl.

  Lane B - PR #139 (GitHub). A handoff = a new comment or commit from the
           GPT-MAC lane (pixiegirlchristy) since the last observed marker.

It is a STATEFUL watcher: it keeps a cursor file (last-seen message id and
last-seen PR comment id) so each run reports only NEW arrivals. It is designed
to run on a schedule (cron) and to print a compact, deterministic summary.

Usage:
    python scripts/watch_handoff_lanes.py status    # one-shot, print current state
    python scripts/watch_handoff_lanes.py check      # print new arrivals since cursor
    python scripts/watch_handoff_lanes.py state      # show the cursor file path+content

Exit codes: 0 = no new arrivals, 3 = new arrivals detected (for scheduler use).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CURSOR = REPO_ROOT / "tmp" / "handoff_lane_cursor.json"
API = "http://127.0.0.1:9876"
HANDOFF_TODO = REPO_ROOT / "HANDOFF" / "todo"
CHAT_LOG = REPO_ROOT / "HANDOFF" / "logs" / "chat"
PR = "139"
OWNER_REPO = "Sovereign-Communication/SCMessenger"

# Canonical bridge config (same path scripts/inbox_bridge.py uses).
BRIDGE_CONFIG_PATH = Path(os.environ.get("APPDATA", str(Path.home() / ".config"))) / "scmessenger" / "inbox_bridge.json"

# Optional pin of the Android adb serial (wireless adb ports change on device
# reboot; leave unset to auto-detect the first attached device).
ANDROID_ADB_SERIAL = os.environ.get("SCM_ANDROID_ADB", "")

# The GPT-MAC lane identity (from the bridge allow-list / history).
GPT_MAC_IDENTITY = "3854e44295c1384854b89312e5c3925f8431b6f4c41ed66979b82b94bc93b5d7"


def _adb_bin() -> str:
    p = shutil.which("adb")
    if p:
        return p
    cand = Path(os.environ.get("LOCALAPPDATA", "")) / "Android" / "Sdk" / "platform-tools" / "adb.exe"
    return str(cand) if cand.exists() else "adb"


def android_device_identity() -> dict | None:
    """Pull the Android app's own identity via adb (run-as, no root).

    Returns None when no device is reachable (skip silently) and a dict with
    identity_id/serial/initialized when the app's identity cache is readable.
    """
    try:
        serial = ANDROID_ADB_SERIAL
        if not serial:
            out = subprocess.run([_adb_bin(), "devices"], capture_output=True, text=True, timeout=10)
            serials = [l.split()[0] for l in out.stdout.splitlines()[1:]
                       if l.strip() and "\tdevice" in l]
            if not serials:
                return None
            serial = serials[0]
        out = subprocess.run(
            [_adb_bin(), "-s", serial, "shell", "run-as", "com.scmessenger.android",
             "cat", "shared_prefs/identity_cache_prefs.xml"],
            capture_output=True, text=True, timeout=15)
        if out.returncode != 0 or "identity_id" not in out.stdout:
            return None
        m = re.search(r'name="identity_id">([^<]+)</string>', out.stdout)
        if not m:
            return None
        return {"identity_id": m.group(1), "serial": serial,
                "initialized": '<boolean name="initialized" value="true"' in out.stdout}
    except Exception:  # noqa: BLE001
        return None


def load_bridge_config() -> dict:
    if BRIDGE_CONFIG_PATH.is_file():
        return json.loads(BRIDGE_CONFIG_PATH.read_text(encoding="utf-8"))
    return {"allowed_peer_id": [], "poll_interval_secs": 3, "api": API}


def save_bridge_config(cfg: dict) -> None:
    BRIDGE_CONFIG_PATH.parent.mkdir(parents=True, exist_ok=True)
    tmp = BRIDGE_CONFIG_PATH.with_suffix(".json.tmp")
    tmp.write_text(json.dumps(cfg, indent=2), encoding="utf-8")
    tmp.replace(BRIDGE_CONFIG_PATH)


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def load_cursor() -> dict:
    if CURSOR.is_file():
        try:
            return json.loads(CURSOR.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            pass
    return {"last_message_id": None, "last_pr_comment_id": None, "last_check": None}


def save_cursor(cursor: dict) -> None:
    CURSOR.parent.mkdir(parents=True, exist_ok=True)
    CURSOR.write_text(json.dumps(cursor, indent=2), encoding="utf-8")


def api_get(path: str) -> dict:
    try:
        with urllib.request.urlopen(API + path, timeout=5) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except Exception as exc:  # noqa: BLE001
        return {"_error": str(exc)}


def node_state() -> dict:
    """Lane A liveness + inbound surface."""
    peers = api_get("/api/peers")
    result = {
        "node_reachable": "_error" not in peers,
        "peer_count": None,
        "bridge": None,
        "new_inbox_tickets": [],
        "new_chat_entries": [],
        "error": peers.get("_error"),
    }
    if isinstance(peers, dict):
        for key in ("peers", "connected_peers"):
            if isinstance(peers.get(key), list):
                result["peer_count"] = len(peers[key])
                break
    # Bridge status file (written by scripts/inbox_bridge.py).
    status_path = Path(os.environ.get("LOCALAPPDATA", "")) / "scmessenger" / "inbox_bridge.status.json"
    if status_path.is_file():
        try:
            result["bridge"] = json.loads(status_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            result["bridge"] = {"_error": "unreadable"}
    return result


def newest_inbox_ticket() -> str | None:
    files = sorted(HANDOFF_TODO.glob("INBOX_*.md"), reverse=True)
    return files[0].name if files else None


def newest_chat_entry() -> dict | None:
    files = sorted(CHAT_LOG.glob("*.jsonl"), reverse=True)
    if not files:
        return None
    try:
        lines = files[0].read_text(encoding="utf-8").splitlines()
        if lines:
            return json.loads(lines[-1])
    except (OSError, json.JSONDecodeError):
        return None
    return None


def pr_latest_comment_id() -> str | None:
    """Latest issue-comment id on PR #139 (id, not the numeric created_by)."""
    try:
        out = subprocess.run(
            ["gh", "api",
             f"repos/{OWNER_REPO}/issues/{PR}/comments",
             "--paginate", "--jq", ".[-1].id"],
            capture_output=True, text=True, timeout=60,
        )
        if out.returncode == 0 and out.stdout.strip():
            return out.stdout.strip().splitlines()[-1]
    except Exception:  # noqa: BLE001
        pass
    return None


def pr_state() -> dict:
    """Lane B: PR head + latest comment/metadata.

    NOTE: comments MUST be fetched with --paginate: gh api defaults to the
    first page (30 items), so .[-1] without pagination returns the 30th-oldest
    comment, not the newest -- new handoffs would never be detected once the
    PR exceeds 30 comments.
    """
    state = {"head": None, "updated_at": None, "state": None,
             "latest_comment_id": None, "latest_comment_author": None, "error": None}
    try:
        out = subprocess.run(
            ["gh", "pr", "view", PR, "--json", "headRefOid,updatedAt,state"],
            capture_output=True, text=True, timeout=20)
        if out.returncode == 0:
            d = json.loads(out.stdout)
            state.update(head=d.get("headRefOid"), updated_at=d.get("updatedAt"),
                         state=d.get("state"))
    except Exception as exc:  # noqa: BLE001
        state["error"] = str(exc)
    try:
        out = subprocess.run(
            ["gh", "api", f"repos/{OWNER_REPO}/issues/{PR}/comments", "--paginate",
             "--jq", ".[-1] | {id, user:.user.login, created_at}"],
            capture_output=True, text=True, timeout=60)
        # --jq runs PER PAGE under --paginate, so stdout has one JSON line per
        # page; the overall newest comment is the LAST line.
        if out.returncode == 0 and out.stdout.strip():
            last_line = [l for l in out.stdout.splitlines() if l.strip()][-1]
            d = json.loads(last_line)
            state["latest_comment_id"] = str(d.get("id"))
            state["latest_comment_author"] = d.get("user")
            state["latest_comment_at"] = d.get("created_at")
    except Exception as exc:  # noqa: BLE001
        state["error"] = (state.get("error") or "") + " | " + str(exc)
    return state


def _collect_arrivals() -> list:
    """Compute new arrivals and advance the cursor. Returns list of arrival lines."""
    cursor = load_cursor()
    arrivals = []
    ni = newest_inbox_ticket()
    nc = newest_chat_entry()
    ps = pr_state()

    # --- Lane A: new INBOX ticket ---
    if ni and ni != cursor.get("last_inbox_ticket"):
        arrivals.append(f"[CLI] new INBOX ticket: {ni}")
        cursor["last_inbox_ticket"] = ni

    # --- Lane A: new chat entry with GPT-MAC as peer ---
    if nc:
        mid = str(nc.get("message_id") or "")
        peer = str(nc.get("peer_id") or "")
        if peer == GPT_MAC_IDENTITY and mid != cursor.get("last_chat_message_id"):
            arrivals.append(
                f"[CLI] new GPT-MAC chat message {mid[:8]} "
                f"ts={nc.get('timestamp')}: {(str(nc.get('content'))[:120])}")
            cursor["last_chat_message_id"] = mid

    # --- Lane B: new PR comment ---
    if ps.get("latest_comment_id") and ps["latest_comment_id"] != cursor.get("last_pr_comment_id"):
        author = ps.get("latest_comment_author") or "?"
        arrivals.append(
            f"[PR] new comment id={ps['latest_comment_id']} by {author} "
            f"at {ps.get('latest_comment_at')}")
        cursor["last_pr_comment_id"] = ps["latest_comment_id"]

    # --- Lane C: Android identity drift (re-owning the Android lane) ---
    # If the device's own identity_id no longer matches the bridge allow-list,
    # the bridge would silently drop all Android inbound. Detect via adb,
    # auto-allow-list the new identity, and surface it ONCE per drifted identity.
    ident = android_device_identity()
    if ident:
        dev_id = ident["identity_id"]
        prev = cursor.get("last_android_identity")
        if prev and prev != dev_id and dev_id != cursor.get("last_android_alert_identity"):
            # Identity reset detected. Auto-apply to the allow-list.
            cfg = load_bridge_config()
            allow = cfg.setdefault("allowed_peer_id", [])
            if isinstance(allow, str):
                allow = [allow]
                cfg["allowed_peer_id"] = allow
            if dev_id not in allow:
                allow.append(dev_id)
                save_bridge_config(cfg)
                action = "allow-list updated"
            else:
                action = "already allow-listed"
            arrivals.append(
                f"[ANDROID] identity drift: {prev[:12]}... -> {dev_id[:12]}... "
                f"({action}; restart inbox_bridge to activate)")
            cursor["last_android_alert_identity"] = dev_id
        cursor["last_android_identity"] = dev_id

    cursor["last_check"] = now_iso()
    save_cursor(cursor)
    return arrivals


def check(verbose: bool) -> int:
    ns = node_state()
    ni = newest_inbox_ticket()
    nc = newest_chat_entry()
    ps = pr_state()
    arrivals = _collect_arrivals()

    if verbose:
        print("== LANE A: SCM CLI ==")
        print(f"  node_reachable={ns.get('node_reachable')} peer_count={ns.get('peer_count')}")
        br = ns.get("bridge") or {}
        if br:
            print(f"  bridge alive, allowlisted_inbound={br.get('allowlisted_inbound_in_window')} "
                  f"ignored={br.get('ignored_inbound_in_window')} "
                  f"tickets_on_disk={br.get('tickets_on_disk')}")
        print(f"  newest INBOX ticket: {ni}")
        print(f"  newest chat entry:   {nc.get('message_id','-') if nc else '-'} "
              f"peer={str(nc.get('peer_id',''))[:8] if nc else '-'}")
        print("== LANE B: PR %s ==" % PR)
        print(f"  head={ps.get('head')} state={ps.get('state')} updated={ps.get('updated_at')}")
        print(f"  latest comment: id={ps.get('latest_comment_id')} author={ps.get('latest_comment_author')} "
              f"at={ps.get('latest_comment_at')}")
        print("== NEW ARRIVALS ==")
        if arrivals:
            print("\n".join("  " + a for a in arrivals))
        else:
            print("  (none since last check)")

    return 3 if arrivals else 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["status", "check", "state"])
    ap.add_argument("--quiet", action="store_true",
                    help="check, but print NOTHING when there are no new arrivals "
                         "(watchdog mode for cron); print only the arrivals when there are.")
    args = ap.parse_args()
    if args.cmd == "status":
        ns = node_state()
        br = ns.get("bridge") or {}
        print("node_reachable=%s peer_count=%s" % (ns.get("node_reachable"), ns.get("peer_count")))
        if br:
            print("bridge alive pid=%s allowlisted=%s ignored=%s tickets=%s" % (
                br.get("bridge_pid"), br.get("allowlisted_inbound_in_window"),
                br.get("ignored_inbound_in_window"), br.get("tickets_on_disk")))
        if ns.get("error"):
            print("node_error=" + ns["error"])
        ps = pr_state()
        print("PR%s head=%s state=%s" % (PR, ps.get("head"), ps.get("state")))
        print("PR latest_comment=%s author=%s" % (ps.get("latest_comment_id"), ps.get("latest_comment_author")))
        return 0
    if args.cmd == "state":
        print("cursor_file=" + str(CURSOR))
        print(json.dumps(load_cursor(), indent=2))
        return 0
    if args.quiet:
        # Watchdog mode: only print when there are new arrivals.
        arrivals = _collect_arrivals()
        if arrivals:
            print("\n".join(arrivals))
            return 3
        return 0
    return check(verbose=True)


if __name__ == "__main__":
    sys.exit(main())