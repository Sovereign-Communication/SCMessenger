#!/usr/bin/env python3
"""Measure every occurrence count GLOSSARY.md publishes, on one stated basis.

Run from the repo root:

    python scripts/measure_uncompiled_counts.py            # the table
    python scripts/measure_uncompiled_counts.py --json     # machine-readable
    python scripts/measure_uncompiled_counts.py --basis wide

This is the single source for every occurrence count in naming-audit/. The glossary
previously carried counts taken on a basis that was recorded nowhere; 15 corpus definitions
were tried against the published figures and none reproduced them. Rather than leave numbers
no reader can re-derive, the glossary now publishes THIS basis, and this script produces it.

BASIS - THE DEFINITION, and the only statement of it in the repository.
  GLOSSARY.md, FINDINGS.md and SUMMARY.md point at this block instead of restating it,
  because a second copy is a second thing to keep in sync. Change a corpus rule here and
  only here; the reports follow the script, not the other way round.
  corpus     first-party source, extensions .rs .kt .swift .udl
  excludes   vendor/ target/ tmp/ build/ dist/ docs/ HANDOFF/ reference/ .claude/
             AgentSwarmCline/ scmessenger_swarm/ patch/
             iOS/SCMessenger/SCMessenger/Generated/  iOS/SCMessengerCore.xcframework/
             and any path containing /tests/ /test/ /androidTest/ /bin/ /examples/
  counting   whole-word, CASE-SENSITIVE, over raw file text
  compiled   decided by walking each Rust crate's module graph from its crate roots; a .rs
             file no root reaches is NOT compiled. Non-Rust source has no module graph here
             and counts as compiled, because gradle and xcodebuild own it.

WHY CASE-SENSITIVE
  Case-insensitive whole-word matching pulls in `GET` on every HTTP route, which would
  inflate the read-verb cluster with transport verbs. Case-sensitive is the smaller
  surprise. The consequence is that the A-series figures fall sharply against the old
  case-insensitive numbers - that is a BASIS change, not a correction.

SEMANTIC EXCLUSIONS (three terms, because a raw count of them is meaningless)
  cfg     Rust's `#[cfg(...)]` attribute and the `cfg!()` macro are not the abbreviation
  forget  "fire-and-forget" is a different meaning
  body    `hyper::body` / `axum::body` are framework path segments, not the concept
These three are filtered before counting. Every other term is a plain whole-word count.

THE SECOND BASIS (--basis wide) - and why it is still here
  FINDINGS.md publishes counts for type names that live only in test trees, examples and
  generated bindings - files the basis above excludes on purpose, because the glossary is
  about the shipping system. Those figures were taken on a WIDER corpus. --basis wide
  reproduces them:

      python scripts/measure_uncompiled_counts.py --basis wide

  WHY THIS EARNS ITS PLACE, rather than being machinery kept alive by inertia: it was not
  invented to fit the numbers. One line of corpus definition reproduces 32 of the 36
  figures FINDINGS publishes exactly, and 35 of 36 to within the drift of a tree being
  edited underneath the measurement. The single genuine outlier is named in FINDINGS
  rather than adjusted. The alternative to keeping it is deleting the frequency column
  from four evidence tables in an audit whose subject is naming frequency.
  CUT CONDITION: if those four tables ever stop publishing counts, delete this flag and
  WIDE_EXTRA with them. They exist for those tables and nothing else.

  The counting rule is byte-for-byte the same; only the corpus differs. wide additionally
  includes /tests/ /test/ /androidTest/ /bin/ /examples/ and the generated iOS/UniFFI
  sources. The default basis is unchanged by this flag and stays the glossary's.
"""
import json
import os
import re
import subprocess
import sys

EXTS = ('.rs', '.kt', '.swift', '.udl')
EXCL_PREFIX = ('vendor/', 'target/', 'tmp/', 'build/', 'dist/', 'docs/', 'HANDOFF/',
               'reference/', '.claude/', 'AgentSwarmCline/', 'scmessenger_swarm/',
               'patch/', 'iOS/SCMessenger/SCMessenger/Generated/',
               'iOS/SCMessengerCore.xcframework')
EXCL_PATH = re.compile(r'/(tests?|androidTest|bin|examples)/')

# --basis wide: same extensions, same doc/build exclusions, but no path filter and no
# generated-source exclusion. Written out rather than derived from EXCL_PREFIX, because
# deriving it by prefix match silently kept the xcframework and cost 28 occurrences of
# IronCore - the exact failure this file exists to prevent. See THE SECOND BASIS above.
EXCL_WIDE = ('vendor/', 'target/', 'tmp/', 'build/', 'dist/', 'docs/', 'HANDOFF/',
             'reference/', '.claude/', 'AgentSwarmCline/', 'scmessenger_swarm/',
             'patch/')

CRATE_ROOTS = {
    'core': ['core/src/lib.rs'],
    'cli': ['cli/src/lib.rs', 'cli/src/main.rs'],
    'wasm': ['wasm/src/lib.rs'],
    'desktop_bridge': ['desktop_bridge/src/lib.rs'],
    'mobile': ['mobile/src/lib.rs'],
}
MOD_RE = re.compile(r'^\s*(?:#\[[^\]]*\]\s*)?(?:pub(?:\([^)]*\))?\s+)?mod\s+([a-z_0-9]+)\s*;', re.M)

# term -> a line filter, or None for a plain whole-word count. `cfg` is stripped at the
# token level rather than dropping whole lines, so a line carrying both an attribute and a
# real identifier still contributes the identifier.
def _cfg_strip(line):
    if not re.search(r'\bcfg\b', line):
        return line
    out = re.sub(r'#\s*\[\s*cfg[^\]]*\]', '', line)
    out = re.sub(r'cfg!\s*\([^)]*\)', '', out)
    return out


EXCLUSIONS = {
    'cfg':    _cfg_strip,
    'forget': lambda line: re.sub(r'fire-and-forget', '', line, flags=re.I),
    'body':   lambda line: re.sub(r'\w+::\s*body\b', '', line),
}

# Every term GLOSSARY.md publishes a count for, grouped by the entry that owns it.
TERMS = {
    # The compound spellings are listed because whole-word matching does not cross `_` or
    # a case change, so without them A-1's headword figures are a floor and a reader cannot
    # see by how much. See the A-1 Count basis note.
    'A-1': ['peer', 'peer_id', 'peerId', 'peers', 'peer_ids', 'peerIds',
            'contact', 'contact_id', 'contactId', 'contacts',
            'node', 'node_id', 'nodeId', 'nodes',
            'relay', 'relay_id', 'relayId', 'relays', 'relay_custody',
            'RelayCustodyStore'],
    'A-3': ['MessageRecord', 'HistoryStats', 'StoredMessage'],
    'A-4': ['Envelope', 'DriftEnvelope', 'payload', 'packet', 'frame', 'blob',
            'envelopeData', 'envelope_data'],
    'A-5': ['canonicalPeerId', 'routePeerId', 'blePeerId', 'libp2pPeerId', 'identity_id'],
    'B-1': ['get', 'read', 'list', 'load', 'resolve', 'find', 'poll', 'fetch', 'retrieve', 'query'],
    'B-2': ['remove', 'clear', 'delete', 'drop', 'purge', 'forget', 'destroy', 'expire', 'unlink'],
    'B-3': ['new', 'create', 'add', 'insert', 'make', 'register', 'save', 'put', 'upsert'],
    'C-2': ['TransportType'],
    'F-33': ['WasmMeshNode'],
    'D-2': ['drift', 'custody', 'beacon', 'UNIFICATION', 'mycorrhizal', 'mycelium',
            'rhizomorph', 'triad', 'mule', 'hopscotch', 'dialplan', 'salting'],
    'D':  ['config', 'settings', 'cfg', 'prefs', 'params', 'options',
           'message', 'text', 'msg', 'body',
           'addr', 'address', 'multiaddr', 'url', 'host', 'endpoint',
           'session', 'token', 'authorization', 'authentication', 'auth'],
}

# Counts FINDINGS.md publishes in its own tables, measured only under --basis wide.
# They are absent from TERMS deliberately: the default basis would report them against a
# corpus those files are not in, and a figure that needs a different corpus to reproduce
# should not sit in the glossary's term list.
WIDE_EXTRA = {
    'F-residue': ['IronCore', 'BlockedIdentity', 'MeshSettings', 'Output',
                  'TransportState', 'RatchetKey', 'PeerInfo', 'BlockedManager',
                  'MultiPathDelivery', 'MeshSettingsManager', 'StoredEnvelope',
                  'DeviceState', 'BootstrapManager', 'DropReason', 'DiscoveredPeer',
                  'RelayStats', 'Input', 'ScanResult', 'BleAdapterInfo', 'Args',
                  'LegacyReceivedMessage'],
    'F-tautology': ['completeData', 'userInfo', 'rawValue', 'newValue', 'removeValue',
                    'initialValue', 'PeerInfo', 'RelayPeerInfo', 'BleAdapterInfo',
                    'PeerDiscoveryInfo', 'JSONObject', 'withJSONObject',
                    'publishIdentityInfo', 'getIdentityInfo'],
    'F-evidence': ['NsdServiceInfo', 'WifiP2pDnsSdServiceInfo', 'ByteArray'],
    'F-22': ['HistoryManager', 'HistoryStats', 'MessageDirection'],
}


def die(msg):
    """Refuse to produce a measurement. A table of zeros is the one output a reader of
    this report cannot distinguish from a real result, so it is never returned."""
    print(f'[ERROR] {msg}', file=sys.stderr)
    print('  This script must run from the repository root, so that `git ls-files`',
          file=sys.stderr)
    print('  enumerates the tree it is measuring.', file=sys.stderr)
    sys.exit(2)


def git_ls_files():
    """Tracked paths, or exit non-zero.

    An earlier version shelled out with os.popen and returned whatever came back. Run from
    any other directory that is an empty list, and the script then printed a full table of
    zeros and exited 0 - indistinguishable from a real measurement, in the one place a
    wrong answer is most likely to be believed."""
    r = subprocess.run(['git', 'ls-files'], capture_output=True, text=True,
                       encoding='utf-8', errors='replace')
    if r.returncode != 0:
        die(f'not inside a git repository (git ls-files exited {r.returncode}).')
    files = [f for f in r.stdout.split('\n') if f]
    if not files:
        die('`git ls-files` returned no paths.')
    return files


def resolve_modules(files):
    """Walk each crate's module graph. Returns (reachable, uncompiled)."""
    def decls(path):
        try:
            txt = open(path, encoding='utf-8', errors='ignore').read()
        except OSError:
            return {}
        out = {}
        for m in re.finditer(r'#\[path\s*=\s*"([^"]+)"\s*\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+'
                             r'([a-z_0-9]+)\s*;', txt):
            out[m.group(2)] = os.path.normpath(os.path.join(os.path.dirname(path), m.group(1)))
        for m in MOD_RE.finditer(txt):
            if m.group(1) in out:
                continue
            d = os.path.dirname(path)
            for cand in (f'{d}/{m.group(1)}.rs', f'{d}/{m.group(1)}/mod.rs'):
                if cand in files:
                    out[m.group(1)] = cand
                    break
        return out

    reachable, queue = set(), [r for rs in CRATE_ROOTS.values() for r in rs if r in files]
    while queue:
        cur = queue.pop()
        if cur in reachable or cur not in files:
            continue
        reachable.add(cur)
        queue.extend(decls(cur).values())

    rust_src = {f for f in files if f.endswith('.rs') and '/src/' in f
                and not f.startswith(EXCL_PREFIX) and not EXCL_PATH.search(f)}
    return reachable, sorted(rust_src - reachable)


def build_corpus(files, basis):
    """Returns (blob, missing).

    A path `git ls-files` lists but the worktree does not hold is SKIPPED and reported, not
    fatal. This checkout is shared and routinely carries deletions other sessions made, so
    a tracked-file-missing must not be the reason the report cannot be re-derived.
    """
    excl, pathf = (EXCL_WIDE, None) if basis == 'wide' else (EXCL_PREFIX, EXCL_PATH)
    blob, missing = {}, []
    for f in files:
        if not f.endswith(EXTS) or f.startswith(excl):
            continue
        if pathf and pathf.search(f):
            continue
        try:
            blob[f] = open(f, encoding='utf-8', errors='ignore').read()
        except OSError:
            missing.append(f)
    return blob, missing


def main():
    files = git_ls_files()
    basis = ('wide' if '--basis' in sys.argv
             and sys.argv[sys.argv.index('--basis') + 1:sys.argv.index('--basis') + 2] == ['wide']
             else 'default')
    blob, missing = build_corpus(files, basis)
    if not blob:
        die(f'the {basis} corpus is empty: no first-party source matched. The basis is '
            f'defined in the BASIS block of this script; run from the repository root.')
    _, uncompiled = resolve_modules(files)
    unc = set(uncompiled)
    groups = TERMS if basis == 'default' else {**TERMS, **WIDE_EXTRA}

    out = {}
    for entry, terms in groups.items():
        for t in terms:
            pat = re.compile(r'\b' + re.escape(t) + r'\b')
            skip = EXCLUSIONS.get(t)
            tot = nf = uh = uf = 0
            for f, txt in blob.items():
                if skip:
                    n = sum(len(pat.findall(skip(line))) for line in txt.split('\n'))
                else:
                    n = len(pat.findall(txt))
                if not n:
                    continue
                tot += n
                nf += 1
                if f in unc:
                    uh += n
                    uf += 1
            out[t] = {'entry': entry, 'total': tot, 'files': nf,
                      'uncompiled': uh, 'uncompiled_files': uf,
                      'excluded': bool(skip)}

    if '--json' in sys.argv:
        print(json.dumps(out, indent=1, sort_keys=True))
        return

    print(f'basis: {basis} | corpus: {len(blob)} files | uncompiled: {len(uncompiled)} files',
          file=sys.stderr)
    for f in missing:
        print(f'[WARNING] tracked but absent from the worktree, not counted: {f}',
              file=sys.stderr)
    print()
    print('| Entry | Term | Total | Files | In uncompiled | Share | Excluded |')
    print('|---|---|---:|---:|---:|---:|---|')
    for entry, terms in groups.items():
        for t in terms:
            r = out[t]
            share = f"{r['uncompiled'] / r['total'] * 100:.0f}%" if r['total'] else '-'
            mark = ' yes' if r['excluded'] else ''
            print(f"| {entry} | `{t}` | {r['total']} | {r['files']} | "
                  f"{r['uncompiled']} ({r['uncompiled_files']}f) | {share} |{mark} |")


if __name__ == '__main__':
    main()
