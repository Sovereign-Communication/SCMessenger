import os, re, sys

LOGDIR = sys.argv[1]
logs = {
    'gcp':     os.path.join(LOGDIR, 'gcp.log'),
    'osx':     os.path.join(LOGDIR, 'osx.log'),
    'android': os.path.join(LOGDIR, 'android.log'),
    'ios_dev': os.path.join(LOGDIR, 'ios-device.log'),
    'ios_sim': os.path.join(LOGDIR, 'ios-sim.log'),
}
NODE_TYPES = {'gcp':'Headless','osx':'Headless','android':'Full','ios_dev':'Full','ios_sim':'Full'}
PAT = re.compile(r"(12D3KooW[1-9A-HJ-NP-Za-km-z]{44,})")
OWN_ID_PATTERNS = [
    re.compile(r'===\s*OWN_IDENTITY:\s*(12D3KooW[a-zA-Z0-9]{44,})\s*==='),
    re.compile(r'local_peer_id\s*=\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'SwarmBridge with peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    # relay agent string pattern: relay/<peerid> — only valid for headless nodes
    re.compile(r'agent: scmessenger/[^/]+/headless/relay/(12D3KooW[a-zA-Z0-9]{44,})'),
    # Android logcat: identity info from MeshRepository/IronCore
    re.compile(r'Mesh service started.*?libp2pPeerId=\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'"libp2p_peer_id"\s*:\s*"(12D3KooW[a-zA-Z0-9]{44,})"'),
    # Android: own identity emission
    re.compile(r'Emitted IdentityDiscovered.*?peerId=(12D3KooW[a-zA-Z0-9]{44,})'),
    # Android logcat with tag prefix: "D/Rust  ( 1234): Starting Swarm with PeerID: ..."
    re.compile(r'Rust\s*:\s*Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'SCMessengerCore\s*:\s*.*?peer.?id[:\s]+(12D3KooW[a-zA-Z0-9]{44,})', re.I),
]
CONNECT_PAT = re.compile(r'(connected|PeerConnected|peer.*connect)', re.I)
ERROR_PAT   = re.compile(r'(Failed to negotiate|connection error|ERR)', re.I)
RELAY_PAT   = re.compile(r'Relay circuit reservation')
NAT_PAT     = re.compile(r'AutoNAT.*?(Public|Private|Unknown)', re.I)

def read(path):
    try:
        with open(path, 'r', errors='ignore') as f: return f.read()
    except: return ""

def strip_ansi(s): return re.sub(r'\x1b\[[^m]*m', '', s)

contents = {n: strip_ansi(read(p)) for n, p in logs.items()}

# For each node, try to find its OWN peer ID (not a peer it's talking to)
# Strategy: the OWN ID should NOT appear as a relay agent string in ANOTHER node
file_to_id = {}
all_candidates = {}
for name, content in contents.items():
    if not content: continue
    for pat in OWN_ID_PATTERNS:
        m = pat.search(content)
        if m and len(m.group(1)) >= 52:
            cand = m.group(1)
            # Validate: a real "own" ID generally appears in lines about local config
            # (not in "Peer identified:" lines which describe remote peers)
            all_candidates[name] = cand
            file_to_id[name] = cand
            break

# Cross-check: if a candidate ID appears as a relay AGENT in another log, it's a
# remote peer being described, not the local node's own ID. De-conflict.
relay_agent_ids = set()
for name, content in contents.items():
    for m in re.finditer(r'agent: scmessenger/[^/]+/[^/]+/relay/(12D3KooW[a-zA-Z0-9]{44,})', content):
        relay_agent_ids.add(m.group(1))
    for m in re.finditer(r'agent: scmessenger/[^/]+/[^/]+/identity/(12D3KooW[a-zA-Z0-9]{44,})', content):
        relay_agent_ids.add(m.group(1))

# Un-assign any full node that incorrectly grabbed a headless node's ID
for name in list(file_to_id.keys()):
    if NODE_TYPES.get(name) == 'Full' and file_to_id[name] in relay_agent_ids:
        del file_to_id[name]

# Fallback for full nodes (like Android/iOS) whose startup logs might be truncated.
# The most frequently appearing peer ID that isn't a known relay node is usually their own.
taken_ids = set(file_to_id.values()) | relay_agent_ids
for name, content in contents.items():
    if name not in file_to_id:
        all_ids = PAT.findall(content)
        freq = {}
        for pid in all_ids:
            if pid not in taken_ids:
                freq[pid] = freq.get(pid, 0) + 1
        if freq:
            best_id = sorted(freq.items(), key=lambda x: x[1], reverse=True)[0][0]
            file_to_id[name] = best_id
            taken_ids.add(best_id)

matrix = {name: set(PAT.findall(c)) for name, c in contents.items()}

# Header
print(f"  {'Node':<10} {'Own ID':<26} {'Lines':>6} {'Relays':>6} {'NAT':<9} {'Connects':>9} {'Errors':>7}")
print("  " + "─" * 82)
for name in logs:
    c   = contents[name]
    pid = file_to_id.get(name, 'unknown')
    pid_d = (pid[:22] + '..') if len(pid) > 22 else pid
    lines  = c.count('\n')
    relays = len(RELAY_PAT.findall(c))
    nat_m  = NAT_PAT.findall(c)
    nat    = nat_m[-1].lower() if nat_m else '?'
    conns  = len(CONNECT_PAT.findall(c))
    errs   = len(ERROR_PAT.findall(c))
    has_content = lines > 2
    icon = '' if (pid != 'unknown' and has_content) else ('⏳' if has_content else '')
    print(f"  {icon} {name:<8} {pid_d:<26} {lines:>6} {relays:>6} {nat:<9} {conns:>9} {errs:>7}")

print()
print("  Visibility Matrix (did node X see node Y's peer ID?):")
print(f"  {'Node':<10} {'Peers Seen':<12} Missing")
print("  " + "─" * 62)
all_ok = True
for name in logs:
    seen = matrix[name]
    missing = []
    for other in logs:
        if other == name: continue
        oid = file_to_id.get(other)
        if not oid or oid not in seen:
            missing.append(other)
            all_ok = False
    seen_count = len(logs) - 1 - len(missing)
    icon = '' if not missing else ('️ ' if len(missing) <= 2 else '')
    print(f"  {icon} {name:<8} {seen_count}/{len(logs)-1:<10} {', '.join(missing) or 'none'}")

print()
if all_ok:
    print("   FULL MESH — All nodes visible to all peers!")
else:
    gaps = sum(1 for n in logs for o in logs if o != n and
               (not file_to_id.get(o) or file_to_id.get(o) not in matrix[n]))
    total = len(logs) * (len(logs)-1)
    pct = int(100 * (total - gaps) / total)
    print(f"  ️  Partial mesh — {gaps}/{total} gap(s)  ({pct}% connected)")
    print(f"     Tip: run longer (--time=10) for peer IDs to propagate fully")

# === Severity Classification ===
print()
print("  Issue Severity Classification:")
print("  " + "─" * 60)

CRASH_PAT = re.compile(r'(panic|fatal|stack.?overflow|SIGSEGV|SIGABRT)', re.I)
CRITICAL_PAT = re.compile(r'(NoClassDefFound|OutOfMemory|FATAL|core dumped)', re.I)
ERROR_PAT_CLASS = re.compile(r'(error|exception|failed|fail)', re.I)
WARN_PAT_CLASS = re.compile(r'(warning|warn|deprecated)', re.I)

for name, content in contents.items():
    if not content:
        continue
    crashes = len(CRASH_PAT.findall(content))
    critical = len(CRITICAL_PAT.findall(content))
    errors = len(ERROR_PAT_CLASS.findall(content))
    warnings = len(WARN_PAT_CLASS.findall(content))
    
    icon = '' if crashes > 0 else ('' if critical > 0 else ('' if errors > 5 else ''))
    print(f"  {icon} {name:<10} crashes={crashes} critical={critical} errors={errors} warnings={warnings}")

# Component breakdown
print()
print("  Component Error Breakdown:")
print("  " + "─" * 60)
COMPONENTS = {
    'BLE': re.compile(r'BLE|Gatt|L2CAP|bluetooth', re.I),
    'Transport': re.compile(r'transport|dial|connect|relay|swarm', re.I),
    'Delivery': re.compile(r'deliver|receipt|send_msg|message', re.I),
    'Identity': re.compile(r'identity|peer.?id|12D3KooW', re.I),
    'Storage': re.compile(r'sled|database|store|persist', re.I),
    'UI': re.compile(r'ViewModel|Fragment|Activity|SwiftUI|View', re.I),
}

for name, content in contents.items():
    if not content:
        continue
    has_events = False
    for comp_name, comp_pat in COMPONENTS.items():
        comp_errors = len(comp_pat.findall(content))
        if comp_errors > 0:
            if not has_events:
                print(f"  {name}:")
                has_events = True
            print(f"    {comp_name}: {comp_errors} events")

print()
# Log file health summary
print("  Log file health:")
for name, path in logs.items():
    if os.path.exists(path):
        sz = os.path.getsize(path)
        lines = contents[name].count('\n')
        icon = '' if lines > 5 else ('️' if lines > 0 else '')
        print(f"    {icon} {name:<10} {lines:>6} lines  {sz:>8} bytes  {path}")
    else:
        print(f"     {name:<10} (no log file)")
