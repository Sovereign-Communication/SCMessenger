"""Cross-node correlation: peer-id resolution, clock-skew estimation, per-message
classification (VERIFIED / PARTIAL / CONTRADICTED) and ledger-sharing report.

Evidence standard (mandatory): a message is VERIFIED only with
  1. receiver-side decrypt            (rx_decrypted on a receiver node)
  2. receiver-side durable history    (rx_history   on the same receiver)
  3. delivery receipt                 (receipt_received on the sender)
all causally ordered within the skew tolerance. transport_ack events, UI
counters and BLE local acceptance never count (markers with evidence=False).
"""
from __future__ import annotations

import statistics
from collections import defaultdict
from datetime import datetime, timedelta
from typing import Dict, Iterable, List, Optional, Sequence, Tuple

from . import markers
from .parse import ts as ev_ts

MIN_PREFIX = 8


# ------------------------------------------------------------------ ids ----
def peer_matches(a: Optional[str], b: Optional[str]) -> bool:
    """Prefix-tolerant peer-id comparison (Android logs truncate to 16 chars)."""
    if not a or not b:
        return False
    a, b = a.lower(), b.lower()
    n = min(len(a), len(b))
    return n >= MIN_PREFIX and a[:n] == b[:n]


def resolve_node_ids(events: Sequence[dict], explicit: Dict[str, str],
                     diag_meta: Optional[Dict[str, dict]] = None) -> Tuple[Dict[str, str], List[str]]:
    """Return ({node: peer_id}, warnings). Order: --peer-id > diagnostics
    self_peer_id > unique mutual-identify inference."""
    ids: Dict[str, str] = dict(explicit)
    warnings: List[str] = []
    for node, meta in (diag_meta or {}).items():
        sp = (meta or {}).get("self_peer_id")
        if sp and node not in ids:
            ids[node] = sp
    nodes = sorted({e["node"] for e in events})
    seen_by: Dict[str, set] = defaultdict(set)
    for e in events:
        if e["event"] == "peer_identified" and e["peer"]:
            seen_by[e["peer"]].add(e["node"])
    for node in nodes:
        if node in ids:
            continue
        cands = [p for p, by in seen_by.items()
                 if node not in by and len(p) >= 16
                 and not any(peer_matches(p, v) for v in ids.values())]
        if len(cands) == 1:
            ids[node] = cands[0]
            warnings.append(f"node id for {node} INFERRED from identify cross-reference; pass --peer-id {node}=<id>")
        else:
            warnings.append(f"node id for {node} unresolved ({len(cands)} candidates); "
                            f"pass --peer-id {node}=<id>")
    return ids, warnings


def node_of_peer(peer: Optional[str], ids: Dict[str, str]) -> Optional[str]:
    for n, i in ids.items():
        if peer_matches(peer, i):
            return n
    return None


# ----------------------------------------------------------------- skew ----
def _sorted_events(events: Iterable[dict]) -> List[dict]:
    evs = [e for e in events if e.get("ts_utc")]
    evs.sort(key=lambda e: (e["ts_utc"], e["src_line"]))
    return evs


def _identify_pair_offset(events: Sequence[dict], a: str, b: str, ids: Dict[str, str],
                          max_skew_s: float) -> Optional[dict]:
    """offset = clock_b - clock_a from mutual identify events."""
    if a not in ids or b not in ids:
        return None
    ta = [ev_ts(e) for e in events if e["node"] == a and e["event"] == "peer_identified"
          and peer_matches(e["peer"], ids[b])]
    tb = [ev_ts(e) for e in events if e["node"] == b and e["event"] == "peer_identified"
          and peer_matches(e["peer"], ids[a])]
    ta, tb = [t for t in ta if t], [t for t in tb if t]
    diffs = []
    for t in ta:
        nearest = min(tb, key=lambda x: abs((x - t).total_seconds()), default=None)
        if nearest is not None:
            d = (nearest - t).total_seconds()
            if abs(d) <= max_skew_s:
                diffs.append(d)
    if not diffs:
        return None
    return {"offset_s": statistics.median(diffs), "n": len(diffs),
            "spread_s": (max(diffs) - min(diffs)) / 2.0, "method": "identify_pairs"}


def _msg_rtt_pair_offset(events: Sequence[dict], a: str, b: str) -> Optional[dict]:
    """NTP-style: for a message a->b with send(a), decrypt(b), receipt_sent(b),
    receipt_received(a):  offset(b-a) = ((t_dec - t_send) - (t_rr - t_rs)) / 2."""
    by_msg: Dict[str, Dict[Tuple[str, str], datetime]] = defaultdict(dict)
    for e in events:
        if not e["msg_id"] or not e.get("ts_utc"):
            continue
        ev = e["event"]
        if ev == "delivery_state" and markers.SEND_PREPARED_DETAIL in str(e["detail"].get("detail", "")):
            ev = "send_prepared"
        by_msg[e["msg_id"]].setdefault((e["node"], ev), ev_ts(e))
    samples = []
    for m, d in by_msg.items():
        for s, r in ((a, b), (b, a)):
            try:
                fwd = (d[(r, "rx_decrypted")] - d[(s, "send_prepared")]).total_seconds()
                back = (d[(s, "receipt_received")] - d[(r, "receipt_sent")]).total_seconds()
            except KeyError:
                continue
            off_r_minus_s = (fwd - back) / 2.0
            samples.append(off_r_minus_s if (s, r) == (a, b) else -off_r_minus_s)
    if not samples:
        return None
    return {"offset_s": statistics.median(samples), "n": len(samples),
            "spread_s": (max(samples) - min(samples)) / 2.0, "method": "msg_rtt_ntp_style"}


def estimate_skew(events: Sequence[dict], ids: Dict[str, str], nodes: Sequence[str],
                  reference: Optional[str] = None, max_skew_s: float = 60.0) -> dict:
    """Per-node clock offset relative to `reference` (aws when present):
    offset_s = clock_node - clock_reference; corrected = ts - offset_s."""
    nodes = sorted(nodes)
    ref = reference or ("aws" if "aws" in nodes else (nodes[0] if nodes else None))
    pair: Dict[Tuple[str, str], dict] = {}
    for i, a in enumerate(nodes):
        for b in nodes[i + 1:]:
            r = _identify_pair_offset(events, a, b, ids, max_skew_s) \
                or _msg_rtt_pair_offset(events, a, b)
            if r:
                pair[(a, b)] = r
                pair[(b, a)] = dict(r, offset_s=-r["offset_s"])
    result = {"reference": ref, "max_skew_s": max_skew_s, "nodes": {}, "pairs": [
        dict(a=a, b=b, **v) for (a, b), v in sorted(pair.items()) if a < b]}
    if ref is None:
        return result
    result["nodes"][ref] = {"offset_s": 0.0, "spread_s": 0.0, "n": 0, "method": "reference"}
    frontier = [ref]
    while frontier:
        cur = frontier.pop(0)
        for n in nodes:
            if n in result["nodes"] or (cur, n) not in pair:
                continue
            p = pair[(cur, n)]
            base = result["nodes"][cur]
            result["nodes"][n] = {
                "offset_s": round(base["offset_s"] + p["offset_s"], 6),
                "spread_s": round(base["spread_s"] + p["spread_s"], 6),
                "n": p["n"],
                "method": p["method"] + ("" if cur == ref else f" via {cur}"),
            }
            frontier.append(n)
    for n in nodes:
        result["nodes"].setdefault(n, {"offset_s": 0.0, "spread_s": 0.0, "n": 0, "method": "unresolved"})
    return result


def adjusted(ev: dict, skew: dict) -> Optional[datetime]:
    t = ev_ts(ev)
    if t is None:
        return None
    off = skew["nodes"].get(ev["node"], {}).get("offset_s", 0.0)
    return t - timedelta(seconds=off)


def suggest_tz_offset(events: Sequence[dict], ids: Dict[str, str], a: str, b: str,
                      max_skew_s: float = 60.0) -> Optional[int]:
    """When two nodes show no pairs, a naive-timestamp node probably has the
    wrong tz. Try 15-minute multiples up to +-14h on b; return minutes to ADD
    to b's timestamps (not applied automatically)."""
    best, best_n = None, 0
    base = [e for e in events if e["node"] == b]
    for k in range(-56, 57):
        shifted = [dict(e, ts_utc=(ev_ts(e) + timedelta(minutes=15 * k)).strftime("%Y-%m-%dT%H:%M:%S.%fZ"))
                   for e in base if e.get("ts_utc")]
        others = [e for e in events if e["node"] == a]
        r = _identify_pair_offset(others + shifted, a, b, ids, max_skew_s)
        if r and r["n"] > best_n:
            best, best_n = 15 * k, r["n"]
    return best if best not in (None, 0) else None


# ------------------------------------------------------- message classify --
MSG_EVENTS = {"delivery_state", "rx_decrypted", "rx_history", "receipt_sent",
              "receipt_received", "history_delivered", "transport_ack"}


def _is_chat(e: dict) -> bool:
    kind = str(e["detail"].get("kind", "text")).lower()
    return not kind.startswith(markers.NON_CHAT_KINDS_PREFIXES)


def _brief(e: dict, t: Optional[datetime]) -> dict:
    return {"node": e["node"], "event": e["event"], "ts_adj": t.strftime("%Y-%m-%dT%H:%M:%S.%fZ") if t else None,
            "marker": e["detail"].get("marker"), "inferred": bool(e["detail"].get("inferred")),
            "src_line": e["src_line"]}


def classify_messages(events: Sequence[dict], skew: dict, ids: Dict[str, str],
                      tol_s: float = 2.0, relay_nodes: Sequence[str] = ("aws",)) -> List[dict]:
    all_nodes = sorted({e["node"] for e in events})
    # coverage window per node (corrected clock) -> used to decide "would have seen it"
    cover: Dict[str, Tuple[datetime, datetime]] = {}
    for n in all_nodes:
        ts_ = [adjusted(e, skew) for e in events if e["node"] == n and e.get("ts_utc")]
        ts_ = [t for t in ts_ if t]
        if ts_:
            cover[n] = (min(ts_), max(ts_))
    by_msg: Dict[str, List[dict]] = defaultdict(list)
    for e in events:
        if e["msg_id"] and e["event"] in MSG_EVENTS and e["msg_id"].lower() != "unknown":
            by_msg[e["msg_id"]].append(e)
    out = []
    for msg_id, evs in sorted(by_msg.items()):
        if not all(_is_chat(e) for e in evs if e["event"] == "rx_decrypted"):
            continue  # identity/history sync metadata, not chat
        evs = sorted(evs, key=lambda e: (adjusted(e, skew).timestamp() if adjusted(e, skew) else float("inf"),
                                         e["src_line"]))
        out.append(_classify_one(msg_id, evs, skew, ids, tol_s, relay_nodes, cover))
    drops: Dict[str, List[dict]] = defaultdict(list)
    for e in events:
        if e["event"] == "rx_drop" and e["msg_id"]:
            drops[e["msg_id"]].append(e)
    for res in out:
        for d in drops.get(res["msg_id"], []):
            res["notes"].append(
                f"rx_drop on {d['node']} stage={d['detail'].get('stage')} "
                f"reason={d['detail'].get('reason')} ({d['src_line']})")
    return out


def _first(evs, node_set, name, require_evidence=True):
    for e in evs:
        if e["event"] == name and (node_set is None or e["node"] in node_set):
            if require_evidence and not e["detail"].get("evidence", True):
                continue
            return e
    return None


def _classify_one(msg_id, evs, skew, ids, tol_s, relay_nodes, cover) -> dict:
    sends = [e for e in evs if e["event"] == "delivery_state"
             and markers.SEND_PREPARED_DETAIL in str(e["detail"].get("detail", ""))]
    sender_nodes = {e["node"] for e in sends} | {e["node"] for e in evs if e["event"] in (
        "receipt_received", "history_delivered")} | {e["node"] for e in evs if e["event"] == "delivery_state"}
    receiver_evs = [e for e in evs if e["event"] in ("rx_decrypted", "rx_history", "receipt_sent")
                    and e["node"] not in sender_nodes]
    receiver_nodes = {e["node"] for e in receiver_evs}
    contradictions: List[str] = []
    notes: List[str] = []

    dec = _first(evs, receiver_nodes, "rx_decrypted")
    hist = _first(evs, receiver_nodes, "rx_history")
    rsent = _first(evs, receiver_nodes, "receipt_sent")
    rrecv = _first(evs, sender_nodes, "receipt_received") or _first(evs, None, "receipt_received")
    send = sends[0] if sends else None
    delivered_state = next((e for e in evs if e["event"] == "delivery_state"
                            and e["detail"].get("state") == "delivered"), None)

    legs = {"decrypt": _brief(dec, adjusted(dec, skew)) if dec else None,
            "history": _brief(hist, adjusted(hist, skew)) if hist else None,
            "receipt": _brief(rrecv, adjusted(rrecv, skew)) if rrecv else None}
    if hist and hist["node"] != (dec or hist)["node"]:
        notes.append("history leg observed on a different node than decrypt leg")
        legs["history"] = None
        hist = None
    missing = [k for k, v in legs.items() if v is None]

    # --- monotonicity (same rule as verify_delivery_state_monotonicity.sh)
    per_node: Dict[str, List[str]] = defaultdict(list)
    for e in evs:
        if e["event"] == "delivery_state":
            per_node[e["node"]].append(str(e["detail"].get("state", "")).lower())
    for node, seq in per_node.items():
        for i, st in enumerate(seq):
            prior = set(seq[:i])
            for term, allowed in markers.TERMINAL_STATES.items():
                if term in prior and st not in allowed:
                    contradictions.append(f"state regression on {node}: {term} -> {st}")

    # --- receipt without decrypt
    if (rrecv or delivered_state) and not dec:
        rt = adjusted(rrecv or delivered_state, skew)
        candidates = [n for n in cover if n not in sender_nodes and n not in relay_nodes]
        covering = [n for n in candidates if rt and cover[n][0] - timedelta(seconds=tol_s) <= rt <= cover[n][1] + timedelta(seconds=tol_s)]
        if covering:
            contradictions.append(
                f"receipt/delivered recorded without receiver decrypt although {','.join(covering)} "
                f"logs cover that time")
        else:
            notes.append("no endpoint node log covers the receipt time; receiver capture missing")

    # --- causality within tolerance
    unc = sum(skew["nodes"].get(n, {}).get("spread_s", 0.0) for n in {e["node"] for e in evs})
    tol = timedelta(seconds=tol_s + unc)

    def _t(e):
        return adjusted(e, skew) if e else None
    order_checks = [("send_prepared", send, "rx_decrypted", dec),
                    ("rx_decrypted", dec, "receipt_received", rrecv),
                    ("rx_decrypted", dec, "receipt_sent", rsent),
                    ("receipt_sent", rsent, "receipt_received", rrecv),
                    ("rx_history", hist, "receipt_received", rrecv)]
    for an, a, bn, b in order_checks:
        if a and b and _t(a) and _t(b) and _t(b) < _t(a) - tol:
            contradictions.append(
                f"causality: {bn} on {b['node']} precedes {an} on {a['node']} by "
                f"{(_t(a) - _t(b)).total_seconds():.2f}s (tol {tol.total_seconds():.2f}s)")

    if contradictions:
        status = "CONTRADICTED"
    elif not missing:
        status = "VERIFIED"
    else:
        status = "PARTIAL"
    if rsent and "receipt" in missing:
        notes.append("receipt_sent observed at receiver but no receipt_received at sender (not scored)")
    if any(e["event"] == "transport_ack" for e in evs):
        notes.append("transport_ack present (ignored by standard)")
    if dec and dec["detail"].get("inferred"):
        notes.append("decrypt leg inferred from code path (marker gap G2)")
    if hist and hist["detail"].get("inferred"):
        notes.append("history leg inferred from already-stored duplicate path (marker gap G3)")
    sn = sorted(sender_nodes - set(receiver_nodes))
    rn = sorted(receiver_nodes)
    return {"msg_id": msg_id, "status": status,
            "direction": f"{','.join(sn) or '?'}->{','.join(rn) or '?'}",
            "legs": legs, "missing": missing, "contradictions": contradictions, "notes": notes,
            "timeline": [_brief(e, adjusted(e, skew)) for e in evs]}


# ---------------------------------------------------------------- ledger ---
def _entries(e: dict) -> int:
    try:
        return int(e["detail"].get("n", 0))
    except (TypeError, ValueError):
        return 0


def ledger_report(events: Sequence[dict], ids: Dict[str, str], skew: dict) -> dict:
    nodes = sorted(ids)
    lev = [e for e in events if e["event"].startswith("ledger_") or e["event"] == "ledger_learned_count"]
    pairs = {}
    for i, a in enumerate(nodes):
        for b in nodes[i + 1:]:
            rec = {"pair": [a, b], "exchanged": False, "directions": {}, "explicit_address_learned": [],
                   "lines": []}
            for x, y in ((a, b), (b, a)):
                mine = [e for e in lev if e["node"] == x and peer_matches(e["peer"], ids[y])]
                d = {"started": sum(e["event"] == "ledger_exchange_started" for e in mine),
                     "offers_received": sum(e["event"] == "ledger_exchange_rx" for e in mine),
                     "entries_received_in_offers": sum(_entries(e) for e in mine if e["event"] == "ledger_exchange_rx"),
                     "replies_sent": sum(e["event"] == "ledger_exchange_reply" for e in mine),
                     "responses": sum(e["event"] == "ledger_exchange_response" for e in mine),
                     "entries_in_responses": sum(_entries(e) for e in mine if e["event"] == "ledger_exchange_response"),
                     "peers_they_learned_per_response": sum(int(e["detail"].get("learned", 0)) for e in mine
                                                            if e["event"] == "ledger_exchange_response"),
                     "learned_count_lines": sum(_entries(e) for e in mine if e["event"] == "ledger_learned_count")}
                rec["directions"][f"{x}_view_of_{y}"] = d
                rec["lines"] += [e["src_line"] for e in mine][:6]
                if any(d[k] for k in ("offers_received", "responses", "replies_sent")):
                    rec["exchanged"] = True
            pairs[f"{a}<->{b}"] = rec
    learned = ledger_learning(events, ids, skew)
    for item in learned:
        key = "explicit_address_learned" if item["evidence"] == "explicit" else "inferred_address_learned"
        for rec in pairs.values():
            if item["learner"] in rec["pair"] and item["via"] in rec["pair"]:
                rec.setdefault(key, []).append(item)
    return {"pairs": pairs, "address_learning": learned,
            "marker_note": "explicit per-address marker ledger_address_learned does not exist in code (gap G1); "
                           "inferred rows are heuristic and labelled"}


def ledger_learning(events: Sequence[dict], ids: Dict[str, str], skew: dict) -> List[dict]:
    """Which node learned which third node's address via whose ledger."""
    out: List[dict] = []
    nodes = sorted(ids)
    for e in events:
        if e["event"] == "ledger_address_learned":
            via_raw = str(e["detail"].get("via", ""))
            via = node_of_peer(via_raw.rsplit(":", 1)[-1], ids) or node_of_peer(via_raw, ids)
            learned = node_of_peer(e["peer"], ids)
            out.append({"learner": e["node"], "via": via or via_raw,
                        "learned": learned or e["peer"], "evidence": "explicit", "line": e["src_line"]})

    def tm(e):
        return adjusted(e, skew)
    for A in nodes:
        for B in nodes:
            for C in nodes:
                if len({A, B, C}) < 3:
                    continue
                exch = [e for e in events if e["node"] == A and peer_matches(e["peer"], ids[B]) and (
                    (e["event"] in ("ledger_exchange_rx", "ledger_exchange_response") and _entries(e) > 0)
                    or (e["event"] == "ledger_learned_count" and _entries(e) > 0))]
                ident = [e for e in events if e["node"] == A and e["event"] == "peer_identified"
                         and peer_matches(e["peer"], ids[C])]
                if not exch or not ident:
                    continue
                t_l, t_c = min(tm(e) for e in exch if tm(e)), min(tm(e) for e in ident if tm(e))
                if t_c <= t_l:
                    continue
                b_knew = any(e["node"] == B and e["event"] == "peer_identified" and peer_matches(e["peer"], ids[C])
                             and tm(e) and tm(e) < t_l for e in events)
                direct = [e for e in events if e["node"] == A and e["event"] in ("bootstrap_dial", "invite_imported")
                          and tm(e) and tm(e) < t_c
                          and (e["event"] == "invite_imported" or peer_matches(_addr_peer(e), ids[C]))]
                if b_knew and not direct:
                    out.append({"learner": A, "via": B, "learned": C, "evidence": "inferred",
                                "line": ident[0]["src_line"], "ledger_line": exch[0]["src_line"]})
    return out


def _addr_peer(e: dict) -> Optional[str]:
    addr = str(e["detail"].get("addr", ""))
    return addr.rsplit("/p2p/", 1)[1] if "/p2p/" in addr else None


# ------------------------------------------------- transport availability ---
TRANSPORT_KINDS = ("tcp4", "tcp6", "quic", "circuit", "dcutr", "mdns", "ble",
                   "wifi_direct", "wifi_aware", "cellular")


def _human(reason: Optional[str]) -> str:
    return (reason or "").replace("_", " ")


def transport_availability(events: Sequence[dict], nodes: Sequence[str]) -> dict:
    """Per node, per transport kind: last logged state/detail, number of state
    changes, max connected-peer count seen, and the timestamp of the last
    marker. A kind with no `[TRANSPORT]` marker at all is reported as
    state="no-marker": the node's log does not say, which is itself a finding
    (the build predates the marker or the transport never initialised)."""
    table: Dict[str, Dict[str, dict]] = {n: {} for n in nodes}
    tev = [e for e in events if e["event"] == "transport_status"]
    tev.sort(key=lambda e: (e.get("ts_utc") or "", e["src_line"]))
    for e in tev:
        node, kind = e["node"], e["detail"].get("tkind")
        if kind not in TRANSPORT_KINDS:
            continue
        row = table.setdefault(node, {}).setdefault(
            kind, {"state": None, "detail": "", "changes": 0, "max_peers": None,
                   "last_ts": None, "src_line": None, "markers": 0})
        peers = e["detail"].get("peers")
        row["markers"] += 1
        if peers is not None:
            n = int(peers)
            row["max_peers"] = n if row["max_peers"] is None else max(row["max_peers"], n)
            if row["state"] is None:  # periodic count before any state line
                row["state"] = e["detail"].get("state")
            continue  # a periodic count is not a state change
        state, detail = e["detail"].get("state"), _human(e["detail"].get("reason"))
        if (state, detail) != (row["state"], row["detail"]):
            row["changes"] += 1
        row["state"], row["detail"] = state, detail
        row["last_ts"], row["src_line"] = e.get("ts_utc"), e["src_line"]
    for node in table:
        for kind in TRANSPORT_KINDS:
            table[node].setdefault(kind, {"state": "no-marker", "detail": "", "changes": 0,
                                          "max_peers": None, "last_ts": None, "src_line": None,
                                          "markers": 0})
    return table


def routing_summary(events: Sequence[dict], nodes: Sequence[str]) -> dict:
    """[ROUTING] peer_seen counts per node and per source transport."""
    out: Dict[str, dict] = {n: {"events": 0, "peers": set(), "sources": defaultdict(int)} for n in nodes}
    for e in events:
        if e["event"] != "routing_peer_seen":
            continue
        o = out.setdefault(e["node"], {"events": 0, "peers": set(), "sources": defaultdict(int)})
        o["events"] += 1
        if e["peer"]:
            o["peers"].add(e["peer"])
        o["sources"][str(e["detail"].get("source"))] += 1
    return {n: {"events": o["events"], "distinct_peers": len(o["peers"]),
                "sources": dict(sorted(o["sources"].items()))} for n, o in out.items()}


def drop_and_stop_summary(events: Sequence[dict], nodes: Sequence[str]) -> dict:
    """[RX-DROP] counts by stage/reason and [MESH-STOP] sequences, per node."""
    drops: Dict[str, Dict[str, int]] = {n: defaultdict(int) for n in nodes}
    stops: Dict[str, List[dict]] = {n: [] for n in nodes}
    for e in sorted(events, key=lambda x: (x.get("ts_utc") or "", x["src_line"])):
        if e["event"] == "rx_drop":
            key = f"{e['detail'].get('stage')}/{e['detail'].get('reason') or e['detail'].get('kind')}"
            drops.setdefault(e["node"], defaultdict(int))[key] += 1
        elif e["event"] == "rx_drop_suppressed":
            key = f"{e['detail'].get('stage')}/suppressed"
            drops.setdefault(e["node"], defaultdict(int))[key] += int(e["detail"].get("count") or 0)
        elif e["event"] == "rx_stall":
            drops.setdefault(e["node"], defaultdict(int))["rx/stall"] += 1
        elif e["event"] == "mesh_stop":
            stops.setdefault(e["node"], []).append(
                {"phase": e["detail"].get("phase"), "result": e["detail"].get("result"),
                 "ms": e["detail"].get("ms"), "ts": e.get("ts_utc"), "src_line": e["src_line"]})
    stop_summary = {}
    for n, seq in stops.items():
        if not seq:
            stop_summary[n] = {"sequence": [], "clean": None}
            continue
        phases = [x["phase"] for x in seq]
        timed_out = any(x["result"] == "timeout" for x in seq)
        stop_summary[n] = {"sequence": seq, "clean": ("complete" in phases) and not timed_out}
    return {"rx_drops": {n: dict(d) for n, d in drops.items()}, "mesh_stop": stop_summary}
