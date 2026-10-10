"""Command line driver: collect -> parse -> correlate -> scenario -> evidence dir."""
from __future__ import annotations

import argparse
import hashlib
import os
import sys
from datetime import datetime, timezone
from typing import Dict, List, Optional

from . import collectors, correlate, parse, report, scenario


def _kv(items: Optional[List[str]], what: str) -> Dict[str, str]:
    out: Dict[str, str] = {}
    for it in items or []:
        if "=" not in it:
            raise SystemExit(f"{what} expects node=value, got {it!r}")
        k, v = it.split("=", 1)
        out[k.strip()] = v.strip()
    return out


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="tri_node_verify.py",
        description="Passive 3-node log triangulation verifier (Android / Windows CLI / AWS relay). "
                    "Scores delivery ONLY on receiver decrypt + durable history + delivery receipt.")
    src = p.add_argument_group("sources (combine freely)")
    src.add_argument("--from-dir", help="ingest captured raw logs: <dir>/<node>/<files>")
    src.add_argument("--map", action="append", metavar="NODE=GLOB",
                     help="with --from-dir on a flat dir: map files to a node (repeatable)")
    src.add_argument("--android", action="store_true", help="collect over adb (read-only)")
    src.add_argument("--adb-serial", help="adb serial, ip:port for wireless ADB (default: only attached device)")
    p.add_argument("--passive", action="store_true",
                   help="never send any command to the Pixel: refuses --android and --adb-serial, so the "
                        "Android node is only ever read from --from-dir captures")
    src.add_argument("--windows", action="store_true", help="collect local Windows CLI node logs")
    src.add_argument("--win-log-dir", help=r"default %%LOCALAPPDATA%%\scmessenger\logs")
    src.add_argument("--win-diag-url", default="http://localhost:9001/api/diagnostics")
    src.add_argument("--aws", action="store_true", help="collect AWS relay over ssh (read-only)")
    src.add_argument("--aws-host", help="host/IP (or env SCM_AWS_HOST); never stored, IP is ephemeral")
    src.add_argument("--aws-user", help="ssh user (or env SCM_AWS_USER, default ubuntu)")
    src.add_argument("--aws-key", help="ssh key path (or env SCM_AWS_KEY)")
    src.add_argument("--aws-container", default="scm-node")
    src.add_argument("--aws-diag-url", default="http://127.0.0.1:9876/api/diagnostics",
                     help="URL evaluated ON the AWS host (default from the 2026-09-29 evidence probe)")
    src.add_argument("--since", default="2h", help="docker logs --since window for AWS (default 2h)")
    p.add_argument("--peer-id", action="append", metavar="NODE=ID",
                   help="libp2p peer id of a node (repeatable). Strongly recommended: aws, windows, android")
    p.add_argument("--scenario", choices=sorted(scenario.SCENARIOS), help="evaluate an expected sequence")
    p.add_argument("--require-msg", action="append", default=[], metavar="MSG_ID",
                   help="message id that MUST be VERIFIED (default: every observed chat message)")
    p.add_argument("--tol-s", type=float, default=2.0, help="causal ordering tolerance, seconds (default 2)")
    p.add_argument("--max-skew-s", type=float, default=60.0, help="largest clock skew considered when pairing")
    p.add_argument("--android-tz-offset-min", type=int, default=0,
                   help="UTC offset (minutes) of zone-less Android timestamps (mesh_diagnostics.log / "
                        "threadtime); logcat from this tool is already UTC")
    p.add_argument("--year", type=int, default=datetime.now(timezone.utc).year,
                   help="year for year-less logcat threadtime lines")
    p.add_argument("--strict-markers", action="store_true",
                   help="reject inferred/weak scenario evidence (require explicit markers)")
    p.add_argument("--out-root", default=os.path.join("tmp", "evidence"))
    p.add_argument("--run-id")
    p.add_argument("--repo-root", default=".")
    return p


def _node_files(run_dir: str, c: collectors.CollectResult) -> List[str]:
    skip = {"diagnostics.json"}
    return [os.path.join(run_dir, f) for f in c.files if os.path.basename(f) not in skip
            and not os.path.basename(f).startswith("diagnostics")]


def run(argv: Optional[List[str]] = None, runner: collectors.Runner = collectors.default_runner,
        out=sys.stdout) -> int:
    a = build_parser().parse_args(argv)
    if a.passive and (a.android or a.adb_serial):
        print("[FAIL] --passive forbids --android/--adb-serial: no command may reach the Pixel. "
              "Feed its logs with --from-dir --map android=<glob> instead.", file=sys.stderr)
        return report.EXIT_INSUFFICIENT
    now = datetime.now(timezone.utc)
    run_id = a.run_id or f"tri-{now.strftime('%H%M%S')}-{hashlib.sha1(os.urandom(8)).hexdigest()[:6]}"
    run_dir = os.path.join(a.repo_root, a.out_root, now.strftime("%Y-%m-%d"), run_id)
    os.makedirs(run_dir, exist_ok=True)

    collected: List[collectors.CollectResult] = []
    if a.from_dir:
        nm = {k: [v] for k, v in _kv(a.map, "--map").items()}
        collected += collectors.FromDirCollector(a.from_dir, nm).collect(run_dir)
    if a.android:
        collected.append(collectors.AndroidCollector(a.adb_serial, runner=runner).collect(run_dir))
    if a.windows:
        collected.append(collectors.WindowsCollector(a.win_log_dir, a.win_diag_url).collect(run_dir))
    if a.aws:
        collected.append(collectors.AwsCollector(a.aws_host, a.aws_user, a.aws_key, a.aws_container,
                                                 a.since, runner=runner, diag_url=a.aws_diag_url).collect(run_dir))
    if not collected:
        print("[FAIL] no sources: pass --from-dir and/or --android/--windows/--aws", file=sys.stderr)
        return report.EXIT_INSUFFICIENT

    events: List[dict] = []
    for c in collected:
        events += parse.parse_files(c.node, _node_files(run_dir, c), rel_to=run_dir, year=a.year,
                                    naive_offset_min=a.android_tz_offset_min if c.node == "android" else 0)
    diag_meta = {c.node: c.meta.get("diagnostics") for c in collected if c.meta.get("diagnostics")}
    nodes = sorted({c.node for c in collected})
    ids, id_warn = correlate.resolve_node_ids(events, _kv(a.peer_id, "--peer-id"), diag_meta)
    skew = correlate.estimate_skew(events, ids, nodes, max_skew_s=a.max_skew_s)
    for n, s in skew["nodes"].items():
        if s["method"] == "unresolved" and n != skew["reference"]:
            id_warn.append(f"clock skew for {n} unresolved (no identify/message pairs); ordering checks "
                           "run on raw clocks")
    messages = correlate.classify_messages(events, skew, ids, tol_s=a.tol_s)
    ledger = correlate.ledger_report(events, ids, skew)
    steps: List[dict] = []
    if a.scenario:
        hosts = [h for h in (a.aws_host or os.environ.get("SCM_AWS_HOST"),) if h]
        steps = scenario.run_scenario(a.scenario, events, ids, skew, messages, hosts, a.strict_markers)
    required_nodes = ["android", "windows", "aws"] if a.scenario else sorted(nodes)
    present = sorted({e["node"] for e in events})
    verdict = report.decide(messages, steps, present, required_nodes, a.require_msg, bool(a.scenario))
    corroboration = {n: {k: m[k] for k in ("custody_audit_count", "history_stats") if k in m}
                     for n, m in diag_meta.items()}
    params = {"argv": [x for x in (argv if argv is not None else sys.argv[1:])
                       if not x.startswith(("--aws-host", "--aws-key", "--aws-user"))],
              "scenario": a.scenario, "tol_s": a.tol_s, "max_skew_s": a.max_skew_s,
              "android_tz_offset_min": a.android_tz_offset_min, "strict_markers": a.strict_markers}
    report.write_run(run_dir, run_id=run_id, repo_root=a.repo_root, params=params, collected=collected,
                     events=events, ids=ids, id_warnings=id_warn, skew=skew, messages=messages, steps=steps,
                     ledger=ledger, verdict=verdict, corroboration=corroboration)
    print(f"[{'OK' if verdict['exit_code'] == 0 else 'INFO'}] verdict={verdict['overall']} exit={verdict['exit_code']}", file=out)
    print(f"[INFO] evidence: {run_dir}", file=out)
    for w in id_warn:
        print(f"[WARNING] {w}", file=out)
    for f in verdict["failed"] + verdict["insufficient"]:
        print(f"[INFO] {f}", file=out)
    return verdict["exit_code"]


def main() -> None:
    sys.exit(run())
