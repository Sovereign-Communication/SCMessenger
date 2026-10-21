#!/usr/bin/env python3
"""
analyze_mesh.py — Live mesh connectivity monitor for 5-node SCMessenger test.

Nodes: GCP (headless), OSX (headless), Android (Pixel 6a), iOS Device, iOS Sim

Usage:
    # Terminal 1: scripts/run5.sh (starts nodes + log collection)
    # Terminal 2: python3 scripts/analyze_mesh.py
"""
import time
import re
import os
import sys
import datetime

# Peer ID pattern: libp2p peer IDs start with 12D3KooW
PAT = re.compile(r"(12D3KooW[1-9A-HJ-NP-Za-km-z]{44,})")

# Log files per node
logs = {
    'gcp':     'logs/5mesh/gcp.log',
    'osx':     'logs/5mesh/osx.log',
    'android': 'logs/5mesh/android.log',
    'ios_dev': 'logs/5mesh/ios-device.log',
    'ios_sim': 'logs/5mesh/ios-sim.log',
}

NODE_TYPES = {
    'gcp':     'Headless',
    'osx':     'Headless',
    'android': 'Full',
    'ios_dev': 'Full',
    'ios_sim': 'Full',
}

# Known peer IDs (update if identities rotate)
TARGET_IDS = {
    '12D3KooWMyngfNZajWRNRPdtc32uxn1sBYZE126NDD4b547BAMLj': 'gcp',
    '12D3KooWHpmuhytgzLcM4nj1hZvN5b4crB1wka3LCNfKRCd7yHj9': 'osx',
    '12D3KooWK8tm9qspf8FZ4sr2VHR48azhYuxCsiu7Ee5yQVoChamU': 'android',
    '12D3KooWHqa2jd8Ec3bbXR24Fn8Lc2rPQQwjeEiY2zUyXXMCez27': 'ios_sim',
    '12D3KooWAqrZFh84t7WbgkTcxUGesHxLUH1gTY4szfe4aEXXqvvg': 'ios_dev',
}

# Multiple patterns for detecting a node's own peer ID from its log
OWN_ID_PATTERNS = [
    # Rust headless/CLI
    re.compile(r'local_peer_id\s*=\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'Local Peer ID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'SwarmBridge with peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'Peer ID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    # Mobile (Android Timber tags, iOS log)
    re.compile(r'SwarmBridge.*peer[_ ]id[=:]\s*(12D3KooW[a-zA-Z0-9]{44,})', re.I),
    re.compile(r'local.*peer.*id[=:]\s*(12D3KooW[a-zA-Z0-9]{44,})', re.I),
    re.compile(r'MeshService.*started.*?(12D3KooW[a-zA-Z0-9]{44,})', re.I),
    re.compile(r'our peer id[:\s]+(12D3KooW[a-zA-Z0-9]{44,})', re.I),
    re.compile(r'identity.*?(12D3KooW[a-zA-Z0-9]{44,})', re.I),
    # agent_version relay pattern: "relay/<peerid>"
    re.compile(r'relay/(12D3KooW[a-zA-Z0-9]{44,})'),
]

# Message send/receive pattern detection
MSG_SENT_PAT = re.compile(r'(send|sent|delivering|dispatch)', re.I)
MSG_RECV_PAT = re.compile(r'(receiv|inbound|incoming|message received)', re.I)
CONNECT_PAT  = re.compile(r'(connected|PeerConnected|peer.*connect)', re.I)
ERROR_PAT    = re.compile(r'(Failed to negotiate|connection error|ERR)', re.I)

matrix    = {k: set() for k in logs}
file_to_id = {}
id_to_label = TARGET_IDS.copy()

def resolve_own_id(name, content):
    """Try all patterns to find this node's own peer ID."""
    clean = re.sub(r'\x1b\[.*?m', '', content)  # strip ANSI
    for pat in OWN_ID_PATTERNS:
        m = pat.search(clean)
        if m:
            candidate = m.group(1)
            # Sanity: must be a valid-length peer ID not already claimed by another node
            if len(candidate) >= 52:
                return candidate
    # Fallback: use the known TARGET_IDS mapping
    for pid, label in TARGET_IDS.items():
        if label == name:
            peers_in_log = set(PAT.findall(clean))
            if pid in peers_in_log:
                return pid
    return None

def log_age(path):
    try:
        mtime = os.path.getmtime(path)
        delta = datetime.datetime.now() - datetime.datetime.fromtimestamp(mtime)
        secs = int(delta.total_seconds())
        if secs < 60: return f"{secs}s"
        elif secs < 3600: return f"{secs//60}m{secs%60:02d}s"
        else: return f"{secs//3600}h{(secs%3600)//60:02d}m"
    except OSError:
        return "?"

def count_events(content, pat):
    return len(pat.findall(content))

print(" SCMessenger 5-Node Mesh Monitor — waiting for logs...")
print("   Run scripts/run5.sh if nodes are not yet started.\n")

try:
    while True:
        contents = {}
        for name, path in logs.items():
            if not os.path.exists(path):
                contents[name] = ""
                continue
            try:
                with open(path, 'r', errors='ignore') as f:
                    contents[name] = f.read()
            except OSError:
                contents[name] = ""

        # Update peer ID discovery and visibility matrix
        for name, content in contents.items():
            if not content:
                continue
            # Find all peer IDs mentioned in this log
            clean = re.sub(r'\x1b\[.*?m', '', content)
            peers_in_log = set(PAT.findall(clean))
            matrix[name].update(peers_in_log)

            # Resolve this node's own ID
            if name not in file_to_id:
                own = resolve_own_id(name, content)
                if own:
                    file_to_id[name] = own
                    id_to_label[own] = name

        # === RENDER ===
        os.system('clear' if os.name == 'posix' else 'cls')
        now = datetime.datetime.now().strftime('%H:%M:%S')
        print(f"╔════════════════════════════════════════════════════════════════╗")
        print(f"║  SCMessenger 5-Node Mesh Status                     {now}  ║")
        print(f"╚════════════════════════════════════════════════════════════════╝")

        # Node identity table
        print(f"\n{'Node':<10} {'Type':<9} {'Own Peer ID':<24} {'Log Age':<8} {'Log Lines'}")
        print("─" * 72)
        for name in logs:
            path = logs[name]
            pid = file_to_id.get(name, "???")
            pid_display = (pid[:20] + "…") if pid != "???" else "not detected"
            ntype = NODE_TYPES.get(name, "???")
            age = log_age(path) if os.path.exists(path) else "no log"
            lines = contents[name].count('\n') if contents.get(name) else 0
            status = "" if pid != "???" else "⏳"
            print(f"{status} {name:<8} {ntype:<9} {pid_display:<24} {age:<8} {lines}")

        # Visibility matrix
        total = len(logs) - 1  # max peers each node can see
        print(f"\n  Visibility Matrix (target: {total} peers each):")
        print(f"  {'Node':<10} {'Sees Full':<22} {'Sees Headless':<20} {'Missing'}")
        print("  " + "─" * 75)

        all_ok = True
        for name in logs:
            own_id = file_to_id.get(name)
            seen_ids = matrix.get(name, set())

            full_seen, headless_seen, missing = [], [], []

            for other_name, other_type in NODE_TYPES.items():
                if other_name == name:
                    continue
                other_id = file_to_id.get(other_name)
                if other_id and other_id in seen_ids:
                    (full_seen if other_type == 'Full' else headless_seen).append(other_name)
                else:
                    label = other_name if other_id else f"{other_name}?"
                    missing.append(label)
                    all_ok = False

            full_str = f"{len(full_seen)}/{sum(1 for t in NODE_TYPES.values() if t=='Full')-1} " \
                       f"({', '.join(full_seen) or '-'})"
            head_str = f"{len(headless_seen)}/{sum(1 for t in NODE_TYPES.values() if t=='Headless')} " \
                       f"({', '.join(headless_seen) or '-'})"
            miss_str = ", ".join(missing) if missing else " None"
            icon = "" if not missing else ("️ " if len(missing) <= 2 else "")
            print(f"  {icon} {name:<8} {full_str:<22} {head_str:<20} {miss_str}")

        # Event summary
        print(f"\n  Event Counts (since log start):")
        print(f"  {'Node':<10} {'Connects':>9} {'Errors':>8} {'Notable'}")
        print("  " + "─" * 55)
        for name in logs:
            c = contents.get(name, "")
            connects = count_events(c, CONNECT_PAT)
            errors   = count_events(c, ERROR_PAT)
            # Check for recent protocol negotiation errors (last 2000 chars)
            recent_errors = count_events(c[-2000:], ERROR_PAT) if c else 0
            notable = f"️  {recent_errors} recent errors" if recent_errors > 5 else ""
            print(f"  {'':2}{name:<8} {connects:>9} {errors:>8}  {notable}")

        # External (unknown) peers
        all_known = set(TARGET_IDS.keys()) | set(file_to_id.values())
        stray_total = 0
        stray_lines = []
        for name in logs:
            strays = [p for p in matrix.get(name, set()) if p not in all_known and p != file_to_id.get(name)]
            if strays:
                stray_total += len(strays)
                stray_lines.append(f"  {name}: {len(strays)} external peers")
        if stray_lines:
            print(f"\n   External peers detected (network health proxy):")
            for line in stray_lines:
                print(line)

        # Overall health
        print()
        ids_found = sum(1 for n in logs if n in file_to_id)
        if all_ok and ids_found == len(logs):
            print("   FULL MESH — All nodes visible to all peers!")
        elif ids_found < len(logs):
            print(f"  ⏳ Waiting for {len(logs) - ids_found} node(s) to identify themselves...")
        else:
            missing_count = sum(1 for n in logs
                                for o in logs if o != n
                                and (not file_to_id.get(o) or file_to_id.get(o) not in matrix.get(n, set())))
            print(f"  ️  Partial mesh: {missing_count} visibility gaps remaining")

        print("\n  (refreshes every 3s — Ctrl+C to stop)")
        time.sleep(3)

except KeyboardInterrupt:
    print("\n\nStopped.")
