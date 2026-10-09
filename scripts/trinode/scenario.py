"""Scenario checks. Each step returns PASS / FAIL / NO_DATA with the evidence
lines (node, normalized event, src_line) that decided it."""
from __future__ import annotations

from typing import Dict, List, Optional, Sequence

from .correlate import _addr_peer, adjusted, ledger_learning, peer_matches

SCENARIOS = {
    "invite-aws-windows-android": [
        "Windows bootstraps from an invite naming AWS",
        "Windows <-> AWS connected (mutual identify)",
        "Android <-> Windows connected (mutual identify)",
        "Ledger shared so Android learns AWS (or AWS learns Android)",
        "Messages VERIFIED in every direction available (android<->windows)",
    ],
}


def _ev(e: dict) -> str:
    d = e["detail"]
    return f"{e['node']}:{e['event']}@{e['ts_raw']} [{d.get('marker')}] {e['src_line']}"


def _step(n, name, status, evidence=None, note=""):
    return {"step": n, "name": name, "status": status, "evidence": evidence or [], "note": note}


def run_scenario(name: str, events: Sequence[dict], ids: Dict[str, str], skew: dict,
                 messages: Sequence[dict], aws_hosts: Sequence[str] = (), strict: bool = False) -> List[dict]:
    if name not in SCENARIOS:
        raise ValueError(f"unknown scenario {name!r}; known: {sorted(SCENARIOS)}")
    names = SCENARIOS[name]
    nodes_with_events = {e["node"] for e in events}
    need = {"windows", "aws", "android"}
    missing_nodes = sorted(need - nodes_with_events)
    steps: List[dict] = []

    def ident(node, other):
        return [e for e in events if e["node"] == node and e["event"] == "peer_identified"
                and other in ids and peer_matches(e["peer"], ids[other])]

    # 1 -- invite / bootstrap naming AWS
    if "windows" not in nodes_with_events or "aws" not in ids:
        steps.append(_step(1, names[0], "NO_DATA", note="need windows events and the AWS peer id (--peer-id aws=...)"))
    else:
        named = [e for e in events if e["node"] == "windows" and e["event"] == "bootstrap_dial"
                 and (peer_matches(_addr_peer(e), ids["aws"])
                      or any(h and h in str(e["detail"].get("addr", "")) for h in aws_hosts))]
        weak = [e for e in events if e["node"] == "windows" and e["event"] == "seed_dial"
                and "connected" in str(e["detail"].get("outcome", "")) and e["detail"].get("sweep")]
        first_conn = min((adjusted(e, skew) for e in ident("windows", "aws") if adjusted(e, skew)), default=None)
        if named:
            late = first_conn and adjusted(named[0], skew) and adjusted(named[0], skew) > first_conn
            steps.append(_step(1, names[0], "PASS", [_ev(named[0])],
                               "bootstrap dial names AWS" + ("; WARNING dial logged after first AWS identify" if late else "")
                               + "; no explicit invite-import marker exists on the CLI (gap G5)"))
        elif weak and not strict:
            steps.append(_step(1, names[0], "PASS", [_ev(weak[0])],
                               "WEAK: seed-dial sweep connected; target not attributable to AWS in the log (gap G5)"))
        else:
            steps.append(_step(1, names[0], "FAIL", [],
                               "no bootstrap dial naming the AWS peer id/host on windows"
                               + ("" if not strict else " (strict-markers: seed-dial fallback disabled)")))

    # 2 / 3 -- mutual identify
    for n, (a, b) in ((2, ("windows", "aws")), (3, ("android", "windows"))):
        if a not in nodes_with_events or b not in nodes_with_events or a not in ids or b not in ids:
            steps.append(_step(n, names[n - 1], "NO_DATA",
                               note=f"need events and peer ids for both {a} and {b}"))
            continue
        ea, eb = ident(a, b), ident(b, a)
        ev = [_ev(ea[0])] if ea else []
        ev += [_ev(eb[0])] if eb else []
        if ea and eb:
            steps.append(_step(n, names[n - 1], "PASS", ev))
        else:
            side = "only " + (a if ea else b if eb else "neither side") + " logged identify"
            steps.append(_step(n, names[n - 1], "FAIL", ev, f"one-sided or absent evidence: {side}"))

    # 4 -- ledger propagation of AWS address to Android (or vice versa)
    if "android" not in nodes_with_events or "windows" not in nodes_with_events or not {"android", "aws", "windows"} <= set(ids):
        steps.append(_step(4, names[3], "NO_DATA", note="need android+windows events and all three peer ids"))
    else:
        learn = [x for x in ledger_learning(events, ids, skew)
                 if {x["learner"], x["learned"]} == {"android", "aws"} and x["via"] == "windows"]
        explicit = [x for x in learn if x["evidence"] == "explicit"]
        inferred = [x for x in learn if x["evidence"] == "inferred"]
        if explicit:
            steps.append(_step(4, names[3], "PASS", [f"explicit marker at {explicit[0]['line']}"]))
        elif inferred and not strict:
            x = inferred[0]
            steps.append(_step(4, names[3], "PASS",
                               [f"ledger exchange at {x.get('ledger_line')}", f"first identify at {x['line']}"],
                               f"INFERRED (no explicit ledger-learned-address marker, gap G1): {x['learner']} learned "
                               f"{x['learned']} via {x['via']}"))
        else:
            steps.append(_step(4, names[3], "FAIL", [],
                               "no ledger-learning evidence" + (" (strict-markers: inference disabled)" if inferred else "")))

    # 5 -- messages
    dirs = {"android->windows", "windows->android"}
    seen = {m["direction"] for m in messages}
    if not messages:
        steps.append(_step(5, names[4], "NO_DATA", note="no chat message ids observed in any node log"))
    else:
        bad = [m for m in messages if m["status"] != "VERIFIED"]
        absent = sorted(dirs - seen)
        ok_dirs = [m for m in messages if m["status"] == "VERIFIED"]
        ev = [f"{m['msg_id']} {m['direction']} {m['status']}" for m in messages[:12]]
        if bad or absent:
            note = []
            if bad:
                note.append(f"{len(bad)} message(s) not VERIFIED")
            if absent:
                note.append("no message observed for direction(s): " + ", ".join(absent))
            steps.append(_step(5, names[4], "FAIL", ev, "; ".join(note)))
        else:
            steps.append(_step(5, names[4], "PASS", ev, f"{len(ok_dirs)} message(s) VERIFIED"))
    if missing_nodes:
        for s in steps:
            s.setdefault("note", "")
        steps.append(_step(0, "scenario precondition: node data present", "NO_DATA",
                           note="no events from node(s): " + ", ".join(missing_nodes)))
    return steps
