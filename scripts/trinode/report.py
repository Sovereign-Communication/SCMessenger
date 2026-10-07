"""Evidence directory + verdict output.

Layout: <out_root>/<UTC date>/<run id>/
  raw/<node>/...        raw captures (collectors / --from-dir copies)
  events.jsonl          all normalized events, sorted by corrected time
  events_<node>.jsonl   per-node normalized events
  manifest.json         schema_version, nodes, versions, collection times, skew, sha256 of every raw file
  verdict.json / verdict.md
Exit codes: 0 PASS, 1 FAIL, 2 INSUFFICIENT DATA.
"""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
from collections import Counter
from datetime import datetime, timezone
from typing import Dict, List, Sequence

SCHEMA_VERSION = 1
TOOL_VERSION = "1.0.0"
EXIT_PASS, EXIT_FAIL, EXIT_INSUFFICIENT = 0, 1, 2


def sha256_file(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def _git_commit(repo_root: str) -> str:
    try:
        r = subprocess.run(["git", "-C", repo_root, "rev-parse", "HEAD"], capture_output=True, timeout=10)
        return r.stdout.decode().strip() if r.returncode == 0 else ""
    except Exception:
        return ""


def decide(messages: Sequence[dict], steps: Sequence[dict], nodes_with_events: Sequence[str],
           required_nodes: Sequence[str], required_msg_ids: Sequence[str] = (),
           have_scenario: bool = False) -> dict:
    """Overall verdict. PASS only if every required message is VERIFIED and every
    scenario step PASSes."""
    reasons: List[str] = []
    insufficient: List[str] = []
    failed: List[str] = []
    missing_nodes = [n for n in required_nodes if n not in nodes_with_events]
    if missing_nodes:
        insufficient.append("no events from required node(s): " + ", ".join(missing_nodes))
    by_id = {m["msg_id"]: m for m in messages}
    required = list(required_msg_ids) or [m["msg_id"] for m in messages]
    if not required:
        insufficient.append("no chat message ids observed")
    for mid in required:
        m = by_id.get(mid)
        if m is None:
            insufficient.append(f"required message {mid} not present in any log")
        elif m["status"] != "VERIFIED":
            failed.append(f"message {mid} is {m['status']}"
                          + (f" (missing: {', '.join(m['missing'])})" if m["missing"] else "")
                          + (f" ({'; '.join(m['contradictions'])})" if m["contradictions"] else ""))
    for s in steps:
        if s["status"] == "FAIL":
            failed.append(f"scenario step {s['step']} FAIL: {s['name']}")
        elif s["status"] == "NO_DATA":
            insufficient.append(f"scenario step {s['step']} NO_DATA: {s['name']}")
    contradicted = any(m["status"] == "CONTRADICTED" for m in messages)
    if contradicted:
        code, label = EXIT_FAIL, "FAIL"
    elif insufficient and not failed:
        code, label = EXIT_INSUFFICIENT, "INSUFFICIENT_DATA"
    elif failed or insufficient:
        code, label = EXIT_FAIL, "FAIL"
    else:
        code, label = EXIT_PASS, "PASS"
    return {"overall": label, "exit_code": code, "failed": failed, "insufficient": insufficient}


def write_run(run_dir: str, *, run_id: str, repo_root: str, params: dict, collected: Sequence[object],
              events: Sequence[dict], ids: Dict[str, str], id_warnings: Sequence[str], skew: dict,
              messages: Sequence[dict], steps: Sequence[dict], ledger: dict, verdict: dict,
              corroboration: dict) -> None:
    os.makedirs(run_dir, exist_ok=True)
    nodes: Dict[str, dict] = {}
    for c in collected:
        n = nodes.setdefault(c.node, {"files": [], "collected_at": c.collected_at, "meta": {}, "errors": []})
        for rel in c.files:
            p = os.path.join(run_dir, rel)
            n["files"].append({"path": rel, "sha256": sha256_file(p), "bytes": os.path.getsize(p)})
        n["meta"].update(c.meta)
        n["errors"] += c.errors
        n["collected_at"] = min(n["collected_at"], c.collected_at) if n["collected_at"] else c.collected_at
    agents = Counter()
    for e in events:
        a = e["detail"].get("agent")
        if a:
            agents[(e["node"], a.rstrip(","))] += 1
    for name, n in nodes.items():
        evs = [e for e in events if e["node"] == name and e.get("ts_utc")]
        n["event_count"] = sum(1 for e in events if e["node"] == name)
        n["first_event_utc"] = min((e["ts_utc"] for e in evs), default=None)
        n["last_event_utc"] = max((e["ts_utc"] for e in evs), default=None)
        n["peer_id"] = ids.get(name)
    manifest = {
        "schema_version": SCHEMA_VERSION, "run_id": run_id, "tool": "tri_node_verify",
        "tool_version": TOOL_VERSION, "tool_repo_commit": _git_commit(repo_root),
        "created_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "params": params, "nodes": nodes, "peer_id_warnings": list(id_warnings),
        "agents_seen": [{"observer": o, "agent": a, "count": c} for (o, a), c in sorted(agents.items())],
        "clock_skew": skew, "diagnostics_corroboration": corroboration,
        "outputs": {"events": "events.jsonl", "verdict": ["verdict.json", "verdict.md"]},
    }
    ordered = sorted(events, key=lambda e: (e.get("ts_utc") or "", e["node"], e["src_line"]))
    with open(os.path.join(run_dir, "events.jsonl"), "w", encoding="utf-8", newline="\n") as fh:
        for e in ordered:
            fh.write(json.dumps(e, sort_keys=True) + "\n")
    for name in nodes:
        with open(os.path.join(run_dir, f"events_{name}.jsonl"), "w", encoding="utf-8", newline="\n") as fh:
            for e in ordered:
                if e["node"] == name:
                    fh.write(json.dumps(e, sort_keys=True) + "\n")
    # manifest last so it can also hash the normalized outputs
    for rel in ["events.jsonl"] + [f"events_{n}.jsonl" for n in nodes]:
        manifest.setdefault("derived_files", []).append(
            {"path": rel, "sha256": sha256_file(os.path.join(run_dir, rel))})
    vj = {"schema_version": SCHEMA_VERSION, "run_id": run_id, **verdict,
          "messages": list(messages), "scenario_steps": list(steps), "ledger": ledger,
          "clock_skew": skew}
    with open(os.path.join(run_dir, "verdict.json"), "w", encoding="utf-8", newline="\n") as fh:
        json.dump(vj, fh, indent=2, sort_keys=True)
    with open(os.path.join(run_dir, "manifest.json"), "w", encoding="utf-8", newline="\n") as fh:
        json.dump(manifest, fh, indent=2, sort_keys=True)
    with open(os.path.join(run_dir, "verdict.md"), "w", encoding="utf-8", newline="\n") as fh:
        fh.write(render_md(vj, manifest))


def render_md(v: dict, manifest: dict) -> str:
    L: List[str] = []
    L.append(f"# Tri-node verdict: {v['overall']} (exit {v['exit_code']})")
    L.append("")
    L.append(f"Run `{v['run_id']}`. Evidence standard: VERIFIED requires receiver-side decrypt + durable "
             "history write + delivery receipt (transport ACKs, UI counters and BLE local acceptance never count).")
    L.append("")
    if v["failed"]:
        L.append("## Failures")
        L += [f"- [FAIL] {x}" for x in v["failed"]]
        L.append("")
    if v["insufficient"]:
        L.append("## Insufficient data")
        L += [f"- [INFO] {x}" for x in v["insufficient"]]
        L.append("")
    sk = v["clock_skew"]
    L.append("## Clock skew (offset = node clock - reference clock)")
    L.append(f"Reference: `{sk['reference']}`")
    L.append("")
    L.append("| node | offset s | spread s | samples | method |")
    L.append("|---|---|---|---|---|")
    for n, s in sorted(sk["nodes"].items()):
        L.append(f"| {n} | {s['offset_s']:.3f} | {s['spread_s']:.3f} | {s['n']} | {s['method']} |")
    L.append("")
    if v["scenario_steps"]:
        L.append("## Scenario steps")
        for s in v["scenario_steps"]:
            L.append(f"- **{s['status']}** step {s['step']}: {s['name']}")
            for e in s["evidence"]:
                L.append(f"  - evidence: `{e}`")
            if s.get("note"):
                L.append(f"  - note: {s['note']}")
        L.append("")
    L.append("## Messages")
    if not v["messages"]:
        L.append("No chat message ids were observed in any node log.")
    for m in v["messages"]:
        L.append(f"### {m['status']}: `{m['msg_id']}` ({m['direction']})")
        for leg, d in m["legs"].items():
            if d:
                L.append(f"- {leg}: {d['node']} {d['event']} at {d['ts_adj']} `{d['src_line']}`"
                         + (" (inferred)" if d["inferred"] else ""))
            else:
                L.append(f"- {leg}: MISSING")
        for c in m["contradictions"]:
            L.append(f"- [FAIL] {c}")
        for n in m["notes"]:
            L.append(f"- [INFO] {n}")
        L.append("")
    L.append("## Ledger sharing per node pair")
    for key, rec in v["ledger"]["pairs"].items():
        L.append(f"- `{key}`: exchanged={rec['exchanged']}")
        for dk, d in rec["directions"].items():
            if any(d.values()):
                L.append(f"  - {dk}: " + ", ".join(f"{k}={x}" for k, x in d.items() if x))
        for x in rec.get("explicit_address_learned", []) + rec.get("inferred_address_learned", []):
            L.append(f"  - address learning ({x['evidence']}): {x['learner']} learned {x['learned']} via {x['via']}")
    L.append(f"- note: {v['ledger']['marker_note']}")
    L.append("")
    L.append("## Manifest")
    L.append(f"See `manifest.json` ({len(manifest['nodes'])} node(s), sha256 of every raw file).")
    return "\n".join(L) + "\n"
