import os, re, sys

LOGDIR = "logs/5mesh/latest"
logs = {
    'gcp':     os.path.join(LOGDIR, 'gcp.log'),
    'osx':     os.path.join(LOGDIR, 'osx.log'),
    'android': os.path.join(LOGDIR, 'android.log'),
    'ios_dev': os.path.join(LOGDIR, 'ios-device.log'),
    'ios_sim': os.path.join(LOGDIR, 'ios-sim.log'),
}
OWN_ID_PATTERNS = [
    re.compile(r'===\s*OWN_IDENTITY:\s*(12D3KooW[a-zA-Z0-9]{44,})\s*==='),
    re.compile(r'local_peer_id\s*=\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'SwarmBridge with peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'Initialized core for peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'agent: scmessenger/[^/]+/headless/relay/(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'Mesh service started.*?libp2pPeerId=\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'"libp2p_peer_id"\s*:\s*"(12D3KooW[a-zA-Z0-9]{44,})"'),
    re.compile(r'Emitted IdentityDiscovered.*?peerId=(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'Rust\s*:\s*Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'SCMessengerCore\s*:\s*.*?peer.?id[:\s]+(12D3KooW[a-zA-Z0-9]{44,})', re.I),
]

def strip_ansi(s): return re.sub(r'\x1b\[[^m]*m', '', s)

file_to_id = {}
for name, path in logs.items():
    if not os.path.exists(path): continue
    with open(path, 'r', errors='ignore') as f:
        content = strip_ansi(f.read())
        for pat in OWN_ID_PATTERNS:
            m = pat.search(content)
            if m:
                file_to_id[name] = m.group(1)
                break

for name, pid in file_to_id.items():
    print(f"{name}: {pid}")
