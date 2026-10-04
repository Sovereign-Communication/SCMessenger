#!/usr/bin/env python3
"""Batched JEV repo insight runner for SCMessenger (orchestrator tool).

Harvests compact signals from HANDOFF / queue / PRs / git, then runs
**batched** TypeSafe JEV evaluations (harness origin/main `JevEvaluator`).
Batched calls keep input tokens and cost low; code owns mechanical facts,
JEV owns bounded semantic judgments.

Usage:
  python scripts/update_local_harness.py
  python scripts/jev_repo_insights.py --mode full --out HANDOFF/audit/JEV_REPO_INSIGHTS.md
  python scripts/jev_repo_insights.py --mode pain,unification --batch-size 4
  python scripts/jev_repo_insights.py --mode dry-run   # harvest only, no JEV

Design notes:
- Aligns with Harness WIP JEV-P3/P5: operator-declared packs only; unkeyed =
  is_fallback and must not be treated as live canonical judgment.
- Does not edit Harness P2/P3 worktrees. Uses HARNESS_REPO for harness.jev.
- Completion gate for WPs remains scripts/jev_canonical_check.py.

Evidence contract: every report section lists commands used to harvest.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

# repo root = two levels up from scripts/
ROOT = Path(__file__).resolve().parents[1]


def _load_harness():
    """SCMessenger-local harness only (vendor/sovereign-harness)."""
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from local_harness import import_harness  # type: ignore

    mod = import_harness()
    from harness.jev import jev_cost  # type: ignore

    return mod["JevEvaluator"], jev_cost, (lambda: mod["key"]), mod["root"]


def _run(cmd: List[str], cwd: Optional[Path] = None) -> str:
    try:
        out = subprocess.check_output(
            cmd,
            cwd=str(cwd or ROOT),
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        return out
    except Exception as exc:  # noqa: BLE001
        return f"[command-failed] {' '.join(cmd)}: {exc}"


def harvest_signals() -> Dict[str, Any]:
    """Mechanical harvest — code-owned facts only."""
    signals: Dict[str, Any] = {
        "harvested_at": datetime.now(timezone.utc).isoformat(),
        "root": str(ROOT),
        "commands": [
            "git log origin/main -15 --oneline",
            "gh pr list --state open",
            "scan HANDOFF/todo + freebuff/queue status lines",
        ],
    }

    signals["git_tip"] = _run(["git", "rev-parse", "origin/main"]).strip()
    signals["git_log"] = _run(["git", "log", "origin/main", "-15", "--oneline"])

    prs = _run(
        [
            "gh",
            "pr",
            "list",
            "--state",
            "open",
            "--limit",
            "40",
            "--json",
            "number,title,mergeStateStatus,isDraft",
        ]
    )
    signals["open_prs_raw"] = prs
    open_pr_count = prs.count('"number"')
    signals["open_pr_count"] = open_pr_count

    todo_items = []
    todo_dir = ROOT / "HANDOFF" / "todo"
    if todo_dir.is_dir():
        for p in sorted(todo_dir.glob("*.md")):
            text = p.read_text(encoding="utf-8", errors="replace")
            status = ""
            for line in text.splitlines()[:30]:
                if line.lower().startswith("**status") or line.lower().startswith("status:"):
                    status = line.strip()
                    break
            todo_items.append({"id": p.name, "status": status[:120], "path": str(p)})
    signals["todo_count"] = len(todo_items)
    signals["todos"] = todo_items[:40]

    queue_items = []
    qdir = ROOT / "HANDOFF" / "freebuff" / "queue"
    if qdir.is_dir():
        for p in sorted(qdir.glob("*.md")):
            text = p.read_text(encoding="utf-8", errors="replace")
            status = ""
            for line in text.splitlines()[:20]:
                if line.lower().startswith("status:"):
                    status = line.strip()
                    break
            queue_items.append({"id": p.name, "status": status[:120]})
    signals["queue_count"] = len(queue_items)
    signals["queue"] = queue_items[:40]

    impl_plan = ROOT / "HANDOFF" / "V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md"
    signals["impl_plan_exists"] = impl_plan.is_file()
    master = ROOT / "HANDOFF" / "V040_CTO_MASTER_PLAN_2026-09-20.md"
    signals["master_plan_exists"] = master.is_file()

    # keyword histograms (mechanical)
    blob = []
    for p in [
        ROOT / "HANDOFF" / "V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md",
        ROOT / "HANDOFF" / "V040_CTO_MASTER_PLAN_2026-09-20.md",
        ROOT / "HANDOFF" / "freebuff" / "README.md",
    ]:
        if p.is_file():
            blob.append(p.read_text(encoding="utf-8", errors="replace"))
    text = "\n".join(blob).lower()
    for key in (
        "routing_peer_seen",
        "public-key hex",
        "wp1",
        "wp2",
        "wp3",
        "wp4",
        "wp5",
        "rule-8",
        "jev",
        "harness",
        "connection_limits",
        "keystore",
        "wifi",
    ):
        signals.setdefault("keyword_hits", {})[key] = text.count(key)

    return signals


def _batch(items: List[Any], n: int) -> List[List[Any]]:
    n = max(1, n)
    return [items[i : i + n] for i in range(0, len(items), n)]


def build_pain_batches(signals: Dict[str, Any], batch_size: int) -> List[Dict[str, Any]]:
    """Compact pain-point units from todos + queue + PR count."""
    units = []
    for t in signals.get("todos", []):
        if re.search(r"P0|P1|BLOCK|OPEN|WiFi|identity|wedge|receipt", t.get("status", "") + t.get("id", ""), re.I):
            units.append(
                {
                    "kind": "todo",
                    "id": t["id"],
                    "signal": t.get("status", "") or t["id"],
                }
            )
    for q in signals.get("queue", []):
        if re.search(r"OPEN|DISPATCH|Rule-8|P0|P1", q.get("status", "") + q.get("id", ""), re.I):
            units.append(
                {
                    "kind": "queue",
                    "id": q["id"],
                    "signal": q.get("status", "") or q["id"],
                }
            )
    if signals.get("open_pr_count"):
        units.append(
            {
                "kind": "pr_backlog",
                "id": "open_prs",
                "signal": f"{signals['open_pr_count']} open PRs; mergeState mixed BEHIND/UNSTABLE/CLEAN",
            }
        )
    if not units:
        units.append({"kind": "empty", "id": "none", "signal": "no harvest units"})
    batches = []
    for group in _batch(units, batch_size):
        batches.append({"domain": "scmessenger_completion", "items": group})
    return batches


def build_unification_batches(signals: Dict[str, Any], batch_size: int) -> List[Dict[str, Any]]:
    units = [
        {
            "id": "identity_canon",
            "signal": "Canonical model: public-key hex addressing + derived peer id (impl plan §0)",
        },
        {
            "id": "routing_feed",
            "signal": "Swarm has routing_peer_seen; non-swarm transports still open (WP2)",
        },
        {
            "id": "docs_queue",
            "signal": f"Freebuff queue files={signals.get('queue_count')}; impl_plan={signals.get('impl_plan_exists')}",
        },
        {
            "id": "delivery_status",
            "signal": "Transport ACK vs app delivered vs receipts still split (WP4)",
        },
        {
            "id": "ci_fleet",
            "signal": "Same-SHA fleet + pinned debug signing + Mobile androidTest lane",
        },
        {
            "id": "harness_jev",
            "signal": "SCMessenger completion uses harness JEV canonical packs; Harness P2/P3 WIP separate",
        },
    ]
    return [
        {"domain": "unification", "items": group}
        for group in _batch(units, batch_size)
    ]


def build_orchestration_batches(signals: Dict[str, Any], batch_size: int) -> List[Dict[str, Any]]:
    units = []
    for q in signals.get("queue", [])[:20]:
        units.append({"id": q["id"], "signal": q.get("status", "")[:100]})
    if not units:
        units.append({"id": "queue_empty", "signal": "no queue tickets harvested"})
    return [
        {"domain": "orchestration", "items": group}
        for group in _batch(units, batch_size)
    ]


def build_historical_batches(signals: Dict[str, Any], batch_size: int) -> List[Dict[str, Any]]:
    units = [
        {
            "id": "ci_hygiene",
            "signal": "Cancel superseded Actions after merges; BUILD_AND_CI.md rule",
        },
        {
            "id": "premise_drift",
            "signal": "Audit STILL-OPEN rows vs main: TRN-04/07 closed by #305; CO-B-001 merged",
        },
        {
            "id": "freebuff_paste",
            "signal": "Stale Status lines misdispatch freebuff; check_queue_status gate",
        },
        {
            "id": "rule8",
            "signal": "Transport/core changes need non-author APPROVE before merge",
        },
        {
            "id": "fleet_same_sha",
            "signal": "Deploy artifacts under tmp/radio-<sha>; identity preserved on AWS/Windows",
        },
        {
            "id": "working_first",
            "signal": "Operator: working mesh first; tag later after WP5 + checklist",
        },
    ]
    return [
        {"domain": "historical_process", "items": group}
        for group in _batch(units, batch_size)
    ]


MODE_BUILDERS = {
    "pain_points": build_pain_batches,
    "unification": build_unification_batches,
    "orchestration": build_orchestration_batches,
    "historical_process": build_historical_batches,
}


def run_batches(
    evaluator,
    jev_cost,
    pack_name: str,
    batches: List[Dict[str, Any]],
    min_confidence: float,
) -> Dict[str, Any]:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from jev_packs import pack_for  # type: ignore

    questions = pack_for(pack_name)
    results = []
    total_tokens = 0
    total_cost = 0.0
    for i, state in enumerate(batches):
        try:
            from local_harness import evaluate_jev_with_openrouter_fallback
            res, jmeta = evaluate_jev_with_openrouter_fallback(evaluator, state, questions)
        except Exception:
            res = evaluator.evaluate(state, questions)
            jmeta = {'endpoint': 'typesafe', 'fallback_used': False}
        total_tokens += int(res.input_tokens or 0)
        total_cost += float(res.cost or 0.0)
        results.append(
            {
                "batch_index": i,
                "verdict": res.verdict,
                "supported": res.supported,
                "confidence": res.confidence,
                "is_fallback": res.is_fallback,
                "input_tokens": res.input_tokens,
                "cost": res.cost,
                "model": res.model,
                "answers": res.answers,
                "reasons": res.reasons,
                "is_passing": res.is_passing(min_confidence),
                "jev_endpoint": jmeta.get("endpoint"),
                "openrouter_fallback": jmeta.get("fallback_used"),
                "item_ids": [it.get("id") for it in state.get("items", [])],
            }
        )
    return {
        "pack": pack_name,
        "batch_count": len(batches),
        "total_input_tokens": total_tokens,
        "total_cost": round(total_cost, 6),
        "results": results,
    }


ISSUE_SORT_PACK = ROOT / "scripts" / "scmessenger_issue_sort_pack.json"


def run_issue_sort(signals: Dict[str, Any]) -> Dict[str, Any]:
    """Sort harvested tickets via Harness JevPolicy.evaluate_issue_sort.

    Operator pack is frozen in scripts/scmessenger_issue_sort_pack.json
    (JEV-P5 contract: no invented buckets).
    """
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from local_harness import make_policy  # type: ignore

    if not ISSUE_SORT_PACK.is_file():
        return {"pack": "missing", "results": [], "total_cost": 0.0}
    pack = json.loads(ISSUE_SORT_PACK.read_text(encoding="utf-8"))
    policy, mod = make_policy()
    items = []
    for trow in signals.get("todos", [])[:25]:
        items.append({"id": trow["id"], "signal": trow.get("status", "") or trow["id"]})
    for q in signals.get("queue", [])[:25]:
        items.append({"id": q["id"], "signal": q.get("status", "") or q["id"]})
    results = []
    total_cost = 0.0
    for item in items:
        issue = f"{item['id']}: {item['signal']}"
        _res, _st, combo = policy.evaluate_issue_sort({"issue": issue}, pack)
        total_cost += float(getattr(_res, "cost", 0.0) or 0.0)
        results.append(
            {
                "id": item["id"],
                "bucket": combo.get("bucket"),
                "path_id": combo.get("path_id"),
                "attention": combo.get("attention"),
                "is_fallback": combo.get("is_fallback"),
                "suggested_next_action": combo.get("suggested_next_action"),
            }
        )
    buckets = Counter(r["bucket"] or "unmatched" for r in results)
    return {
        "pack": pack.get("id"),
        "harness_tip": str(mod["root"]),
        "results": results,
        "bucket_counts": dict(buckets),
        "total_cost": round(total_cost, 6),
    }



def write_report(path: Path, signals: Dict[str, Any], runs: List[Dict[str, Any]], meta: Dict[str, Any]) -> None:
    lines = []
    lines.append("# JEV repo insight report — SCMessenger")
    lines.append("")
    lines.append(f"Generated: {signals.get('harvested_at')}")
    lines.append(f"Main tip (origin): `{signals.get('git_tip')}`")
    lines.append(f"Harness: `{meta.get('harness_path')}`")
    lines.append(f"JEV keyed: {meta.get('keyed')} model={meta.get('model')}")
    lines.append("")
    lines.append("## Harvest (code-owned)")
    lines.append("")
    lines.append(f"- Open PR count (approx from gh json keys): {signals.get('open_pr_count')}")
    lines.append(f"- HANDOFF/todo files scanned: {signals.get('todo_count')}")
    lines.append(f"- Freebuff queue files: {signals.get('queue_count')}")
    lines.append(f"- Implementation plan present: {signals.get('impl_plan_exists')}")
    lines.append(f"- Master plan present: {signals.get('master_plan_exists')}")
    hits = signals.get("keyword_hits") or {}
    if hits:
        lines.append("- Keyword hits (plans/readme):")
        for k, v in hits.items():
            lines.append(f"  - `{k}`: {v}")
    lines.append("- Commands: " + "; ".join(signals.get("commands") or []))
    lines.append("")
    lines.append("## Git tip log")
    lines.append("")
    lines.append("```text")
    lines.append((signals.get("git_log") or "").strip()[:2000])
    lines.append("```")
    lines.append("")

    grand_tokens = sum(r.get("total_input_tokens") or 0 for r in runs)
    grand_cost = round(sum(r.get("total_cost") or 0.0 for r in runs), 6)
    lines.append("## JEV batch totals")
    lines.append("")
    lines.append(f"- Packs run: {len(runs)}")
    lines.append(f"- Total input tokens: {grand_tokens}")
    lines.append(f"- Total cost (USD): {grand_cost}")
    lines.append("")

    for run in runs:
        lines.append(f"## Pack: `{run['pack']}`")
        lines.append("")
        lines.append(
            f"batches={run['batch_count']} tokens={run['total_input_tokens']} cost={run['total_cost']}"
        )
        lines.append("")
        for res in run["results"]:
            fallback = "FALLBACK" if res["is_fallback"] else "LIVE"
            lines.append(
                f"### Batch {res['batch_index']} [{fallback}] verdict={res['verdict']} "
                f"supported={res['supported']} conf={res['confidence']}"
            )
            lines.append(f"- items: {', '.join(res.get('item_ids') or [])}")
            lines.append(f"- tokens={res['input_tokens']} cost={res['cost']} model={res['model']}")
            lines.append(f"- answers: `{json.dumps(res.get('answers') or {}, ensure_ascii=False)}`")
            for reason in res.get("reasons") or []:
                lines.append(f"- reason: {reason}")
            lines.append("")

    iso = (meta or {}).get("issue_sort") or {}
    if iso:
        lines.append("## Issue-sort (Harness JevPolicy.evaluate_issue_sort)")
        lines.append("")
        lines.append(f"- pack: `{iso.get('pack')}` cost={iso.get('total_cost')}")
        lines.append(f"- harness: `{iso.get('harness_tip')}`")
        lines.append(f"- bucket_counts: `{json.dumps(iso.get('bucket_counts') or {}, ensure_ascii=False)}`")
        lines.append("")
        for row in (iso.get("results") or [])[:40]:
            lines.append(
                f"- `{row.get('id')}` -> bucket=`{row.get('bucket')}` "
                f"attention=`{row.get('attention')}` path=`{row.get('path_id')}` "
                f"fallback={row.get('is_fallback')} next=`{row.get('suggested_next_action')}`"
            )
        lines.append("")

    lines.append("## Insights for SCMessenger completion (orchestrator synthesis)")
    lines.append("")
    lines.append(
        "Mechanical harvest + JEV batches above are inputs. Final action orders stay "
        "in `HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` and "
        "`HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md`. JEV does not invent new root causes."
    )
    lines.append("")
    lines.append("### Standing synthesis themes")
    lines.append("")
    lines.append("1. Identity unification (hex) remains the first WP — dual flavor breaks send paths.")
    lines.append("2. Routing feed must be one entry for all data links — WP2 is the open hole.")
    lines.append("3. Delivery truth (ACK vs delivered vs receipts) is process + code — WP4.")
    lines.append("4. WP5 live 3-node logs are mandatory before any WiFi-fixed claim.")
    lines.append("5. Freebuff paste authority is the implementation plan DISPATCHABLE set + JEV canonical DONE.")
    lines.append("6. Harness WIP (P2 repair / jev-phase / issue-sort) stays in Harness repo; SCMessenger consumes origin/main JEV + local packs.")
    lines.append("")

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(description="Batched JEV SCMessenger repo insights")
    ap.add_argument(
        "--mode",
        default="full",
        help="comma list: pain_points,unification,orchestration,historical_process,canonical_completion,full,dry-run",
    )
    ap.add_argument("--batch-size", type=int, default=4)
    ap.add_argument("--min-confidence", type=float, default=0.70)
    ap.add_argument("--out", default="HANDOFF/audit/JEV_REPO_INSIGHTS_2026-09-21.md")
    args = ap.parse_args()

    modes = [m.strip() for m in args.mode.split(",") if m.strip()]
    if "full" in modes:
        modes = ["pain_points", "unification", "orchestration", "historical_process", "issue_sort"]

    signals = harvest_signals()
    out_path = ROOT / args.out if not Path(args.out).is_absolute() else Path(args.out)

    if "dry-run" in modes:
        out_path.write_text(
            "# JEV dry-run harvest\n\n```json\n"
            + json.dumps(signals, indent=2, ensure_ascii=False)[:12000]
            + "\n```\n",
            encoding="utf-8",
        )
        print(f"[OK] dry-run harvest written: {out_path}")
        return 0

    JevEvaluator, jev_cost, resolve_jev_key, harness_path = _load_harness()
    api_key = resolve_jev_key() or None
    evaluator = JevEvaluator(api_key=api_key)
    print(f"[INFO] harness={harness_path} keyed={bool(api_key)}")

    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from jev_packs import pack_for  # type: ignore

    runs = []
    for mode in modes:
        if mode == "canonical_completion":
            # single compact batch — WP completion evidence would be supplied via state file later
            state = {
                "domain": "canonical_completion_summary",
                "items": [
                    {
                        "id": "wp_summary",
                        "signal": "See implementation plan WP1-5; completion requires JEV pack + mechanical gates",
                    }
                ],
            }
            run = run_batches(
                evaluator, jev_cost, mode, [state], args.min_confidence
            )
        elif mode in MODE_BUILDERS:
            batches = MODE_BUILDERS[mode](signals, args.batch_size)
            run = run_batches(evaluator, jev_cost, mode, batches, args.min_confidence)
        else:
            print(f"[WARNING] unknown mode {mode}")
            continue
        runs.append(run)
        print(
            f"[OK] pack={run['pack']} batches={run['batch_count']} "
            f"tokens={run['total_input_tokens']} cost={run['total_cost']}"
        )

    issue_sort_out = None
    if "issue_sort" in modes or "full" in modes:
        try:
            issue_sort_out = run_issue_sort(signals)
            print(
                f"[OK] issue_sort pack={issue_sort_out.get('pack')} "
                f"items={len(issue_sort_out.get('results') or [])} "
                f"cost={issue_sort_out.get('total_cost')}"
            )
        except Exception as exc:  # noqa: BLE001
            print(f"[WARNING] issue_sort failed: {exc}")
            issue_sort_out = {"error": str(exc)}

    write_report(
        out_path,
        signals,
        runs,
        {
            "harness_path": str(harness_path),
            "keyed": bool(api_key),
            "model": evaluator.model,
            "issue_sort": issue_sort_out,
        },
    )
    print(f"[OK] report: {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
