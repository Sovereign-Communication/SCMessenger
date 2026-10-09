"""Synthetic three-node log builder used by the tests.

Every line shape comes from markers.py (itself derived from origin/main
source). The REAL captured logs (fixtures/real) contain no message-level
markers at all, so message scenarios must be synthesized; this module is the
only place that does so, and it never claims a real capture.
"""
from __future__ import annotations

import os
import tempfile
from datetime import datetime, timedelta, timezone
from typing import Dict, List

AWS_ID = "12D3KooWFixtureAwsRelay00000000000000000000000"
WIN_ID = "12D3KooWFixtureWindowsNode000000000000000000000"
AND_ID = "12D3KooWFixtureAndroidPhone0000000000000000000"
IDS = {"aws": AWS_ID, "windows": WIN_ID, "android": AND_ID}
AWS_ADDR = f"/ip4/203.0.113.10/tcp/9001/p2p/{AWS_ID}"
BASE = datetime(2026, 10, 1, 21, 0, 0, tzinfo=timezone.utc)
ANDROID_SKEW = 1.5   # android clock runs 1.5 s ahead of true time
WINDOWS_SKEW = 0.3
AWS_SKEW = 0.0


def T(sec: float, skew: float = 0.0) -> datetime:
    return BASE + timedelta(seconds=sec + skew)


def iso(t: datetime) -> str:
    return t.strftime("%Y-%m-%dT%H:%M:%S.%fZ")


def aws(sec, text, lvl=" INFO", target="scmessenger_core::transport::swarm"):
    t = T(sec, AWS_SKEW)
    return f"{iso(t)[:-1]}123Z {iso(t)} {lvl} {target}: {text}"


def win(sec, text, lvl=" INFO", target="scmessenger_cli"):
    return f"{iso(T(sec, WINDOWS_SKEW))} {lvl} {target}: {text}"


def andj(sec, text, lvl="INFO"):
    import json
    return json.dumps({"timestamp": iso(T(sec, ANDROID_SKEW)), "level": lvl, "fields": {"message": text}})


def andlog(sec, text, lvl="I", tag="MeshRepository"):
    t = T(sec, ANDROID_SKEW)
    return f"{t.strftime('%Y-%m-%d %H:%M:%S.%f')[:-3]} +0000  1415  1590 {lvl} {tag}: {text}"


def connect_lines() -> Dict[str, List[str]]:
    """Invite -> windows bootstraps to AWS -> android joins via windows ->
    ledger exchange -> android learns AWS."""
    ident = "Identified peer {p} - agent: scmessenger/0.4.0/full/relay/{p}, protocols: 16, discoverable_addrs: 9"
    L = {"aws": [], "windows": [], "android": []}
    L["windows"].append(win(0.0, f"  [OK] Dialing bootstrap: {AWS_ADDR}", target="scmessenger_core::transport::swarm"))
    L["windows"].append(win(1.2, ident.format(p=AWS_ID), target="scmessenger_core::transport::swarm"))
    L["aws"].append(aws(1.0, ident.format(p=WIN_ID)))
    L["windows"].append(win(30.0, ident.format(p=AND_ID), target="scmessenger_core::transport::swarm"))
    L["android"].append(andj(30.1, ident.format(p=WIN_ID)))
    L["windows"].append(win(30.3, f"Ledger exchange from {AND_ID}: offered 0 peer entries (v1, allowed=true)",
                            target="scmessenger_core::transport::swarm"))
    L["android"].append(andj(30.9, f"Ledger exchange response from {WIN_ID}: they learned 0 new peers, sent 2 back"))
    L["android"].append(andj(40.0, ident.format(p=AWS_ID)))
    L["aws"].append(aws(40.1, ident.format(p=AND_ID)))
    return L


def msg_lines() -> Dict[str, List[str]]:
    L = {"aws": [], "windows": [], "android": []}
    # m-ok-aw: android -> windows, fully evidenced
    L["android"].append(andlog(60.0, "delivery_state msg=m-ok-aw state=pending detail=message_prepared_local_history_written"))
    L["windows"].append(win(60.4, f"Sending delivery ACK for m-ok-aw to {AND_ID}"))
    L["windows"].append(win(60.4, f"msg_rx_processed peer={AND_ID} msg=m-ok-aw decrypt=ok history=written"))
    L["android"].append(andlog(60.9, "[RECEIPT-RX] Received from core: msg=m-ok-aw status=delivered"))
    L["android"].append(andlog(60.9, "delivery_state msg=m-ok-aw state=delivered detail=delivery_receipt_status=delivered first_receipt=true"))
    # m-ok-wa: windows -> android; history via the already-stored duplicate path
    L["android"].append(andlog(70.0, f"UNIFICATION onMessageReceived pairing: sender={WIN_ID} canonical=abc routePeerId=null messageId=m-ok-wa kind=text transport=INTERNET isKnownContact=true isChatEvent=true publicKey=ab12cd34"))
    L["android"].append(andlog(70.2, "Duplicate inbound m-ok-wa already stored; re-acknowledging (time_variance=10ms first_transport=INTERNET)"))
    L["android"].append(andlog(70.3, f"Targeted delivery receipt sent for m-ok-wa to {WIN_ID}", lvl="D"))
    L["windows"].append(win(70.6, f"Delivery ACK received from {AND_ID}: msg_id=m-ok-wa"))
    # m-partial: decrypt only, no history, no receipt
    L["android"].append(andlog(80.0, "delivery_state msg=m-partial state=pending detail=message_prepared_local_history_written"))
    L["windows"].append(win(80.4, f"Sending delivery ACK for m-partial to {AND_ID}"))
    # m-contra: delivered recorded, receiver (windows, which covers that time) has nothing
    L["android"].append(andlog(90.0, "[RECEIPT-RX] Received from core: msg=m-contra status=delivered"))
    L["android"].append(andlog(90.0, "delivery_state msg=m-contra state=delivered detail=delivery_receipt_status=delivered first_receipt=true"))
    # m-regress: delivered then pending
    L["android"].append(andlog(100.0, "delivery_state msg=m-regress state=delivered detail=x"))
    L["android"].append(andlog(100.5, "delivery_state msg=m-regress state=pending detail=y"))
    # m-transport: only transport-level acks
    L["android"].append(andlog(110.0, "delivery_attempt msg=m-transport medium=internet phase=aggregate outcome=acked detail=ctx=send"))
    L["windows"].append(win(110.1, "[OK] Message delivered successfully to 12D3KooWFixtureAndroidPhone0000000000000000000 (40ms)",
                            target="scmessenger_core::transport::swarm"))
    # filler so every node's coverage spans the whole run
    L["aws"].append(aws(120.0, "Relay custody audit log count: 5"))
    L["windows"].append(win(120.0, "Relay custody audit log count: 5", target="scmessenger_core::transport::swarm"))
    L["android"].append(andj(120.0, "tick"))
    return L


def build(dirpath: str, lines: Dict[str, List[str]]) -> str:
    names = {"aws": "docker.log", "windows": "scm.log.2026-10-01-21", "android": "logcat.txt"}
    for node, ls in lines.items():
        d = os.path.join(dirpath, node)
        os.makedirs(d, exist_ok=True)
        with open(os.path.join(d, names[node]), "w", encoding="utf-8", newline="\n") as fh:
            fh.write("\n".join(ls) + "\n")
    return dirpath


def merge(*parts: Dict[str, List[str]]) -> Dict[str, List[str]]:
    out = {"aws": [], "windows": [], "android": []}
    for p in parts:
        for k, v in p.items():
            out[k] += v
    return out
