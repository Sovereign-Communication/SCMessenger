#!/usr/bin/env python3
"""Generate the read-only open-PR disposition ledger for the SCMessenger burndown.

Read-only: calls the GitHub API through `gh` for every OPEN pull request and
writes a Markdown table plus a JSON sidecar under `tmp/`. It never merges,
closes, edits, or comments. It fails closed only on an OBSERVED contradiction
(a PR marked "LAND FIRST" that is conflicting, or a draft, which cannot land);
absence of data (GitHub reports mergeability as UNKNOWN while it computes) is
a [WARNING], because a verdict needs evidence. Infrastructure failures (gh
missing, rate limit, timeout) are [WARNING]; only an unusable --disposition
file or an unenumerable PR list stop the run with exit 2.

Usage (from the repository root):
  python scripts/pr_disposition_ledger.py --out tmp/pr_disposition_2026-09-25.md
  python scripts/pr_disposition_ledger.py --disposition tmp/pr_disposition.json

Exit codes: 0 ledger written (warnings allowed), 1 contradiction observed,
2 could not enumerate open PRs, or --disposition could not be read.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional

REPO = "Sovereign-Communication/SCMessenger"
# Known API ceilings. A value that lands EXACTLY on one of these is a ceiling,
# not a count, and must say so (rule 15): GitHub returns at most 100 files per
# PR in the REST/GraphQL file list, and `gh pr list --limit 100` returns at
# most 100 PRs. A silently-truncated 100 would read as a complete ledger.
PR_LIST_LIMIT = 100
PR_FILES_CAP = 100
GATED_PREFIXES = (
    "core/src/crypto/",
    "core/src/transport/",
    "core/src/routing/",
    "core/src/privacy/",
)

# gh pr checks conclusion -> ledger column. A skipped check (CodeQL on a
# Dependabot branch, for example) proves nothing about the change, so it gets
# its own column and is never counted as pass.
CHECK_CLASS = {
    "pass": "pass",
    "skipping": "skip",
    "neutral": "skip",
    "fail": "fail",
    "cancelled": "fail",
    "canceled": "fail",
    "timed_out": "fail",
    "action_required": "fail",
    "stale": "fail",
    "pending": "pending",
    "queued": "pending",
    "in_progress": "pending",
    "waiting": "pending",
    "requested": "pending",
}

# Disposition is an OPERATOR/MAIN-LANE decision table, not a mechanical result.
# Keep it next to the generator so the ledger is reproducible and reviewable;
# --disposition overrides it from JSON (partial entries are allowed: missing
# keys fall back to UNDECIDED / empty lane / default note).
DEFAULT_DISPOSITION: Dict[str, Dict[str, str]] = {
    # Workstream 0/0b
    "372": {"disposition": "LAND FIRST", "lane": "transport leg", "note": "await independent Rule-8; author of D1/D9 cannot self-sign"},
    "364": {"disposition": "SPLIT", "lane": "android lifecycle + outbox sweep", "note": "split Kotlin half (superseded by #372) from core outbox sweep + Docker vendor copy; Rule-8 on sweep half"},
    "367": {"disposition": "CLOSE or SPLIT", "lane": "android lifecycle", "note": "overlaps #364 Kotlin half"},
    "359": {"disposition": "REBASE DOCS RESIDUE", "lane": "canonical docs", "note": "drop transport commit (#372 supersedes); add scope blocks to every HANDOFF doc"},
    "368": {"disposition": "MERGE AFTER SCOPE BLOCKS", "lane": "tool plan docs", "note": "add WS queue rows in same edit"},
    "369": {"disposition": "CLOSE", "lane": "snapshot", "note": "snapshot of #368 WIP"},
    "354": {"disposition": "LAND BEFORE WP DONE", "lane": "JEV state files", "note": "WP1/WP2 state files exist only here"},
    "357": {"disposition": "REBASE DOCS", "lane": "train status docs", "note": "superseded by #368? confirm before closing"},
    "360": {"disposition": "SUPERSEDED BY WS0", "lane": "tool version floor", "note": "0.3.3 floor; WS0 replaces with pin file"},
    "361": {"disposition": "CLOSE AFTER #372", "lane": "transport", "note": "replaced by #372"},
    "349": {"disposition": "REBASE + RULE-8 + JEV", "lane": "WP2 routing feed", "note": "mobile_bridge.rs overlaps #372; rebase after #372"},
    "351": {"disposition": "REBASE AFTER #364 SPLIT", "lane": "android stop teardown", "note": "touches reserved MeshForegroundService"},
    "352": {"disposition": "REBASE + JEV", "lane": "WP1 tests", "note": "JEV row fails; needs ruling from WS0"},
    "355": {"disposition": "REBASE + RULE-8 + JEV", "lane": "WP3", "note": "core gossip; gated swarm.rs"},
    "356": {"disposition": "REBASE + RULE-8 + JEV", "lane": "WP4 tests", "note": "cli watchdog/delivery-truth; gated swarm.rs"},
    "316": {"disposition": "DOCS LAND", "lane": "outbox retry diagnosis", "note": "diagnosis doc only; fix is #364 half"},
    "329": {"disposition": "REBASE DOCS", "lane": "queue reconciliation", "note": "stale status lines"},
    "303": {"disposition": "HOLD", "lane": "merge plan doc", "note": "operator release hold"},
    "298": {"disposition": "HELD", "lane": "android workahead", "note": "operator hold until released"},
    "299": {"disposition": "HELD", "lane": "android workahead", "note": "operator hold until released"},
    "300": {"disposition": "HELD", "lane": "android workahead", "note": "operator hold until released"},
    "301": {"disposition": "HELD", "lane": "ios workahead", "note": "operator hold until released"},
    "302": {"disposition": "HELD", "lane": "android workahead", "note": "operator hold until released"},
    "211": {"disposition": "DEPENDABOT AFTER TRAIN", "lane": "ci", "note": "setup-java 5"},
    "212": {"disposition": "DEPENDABOT AFTER TRAIN", "lane": "ci", "note": "stale 11"},
    "214": {"disposition": "DEPENDABOT AFTER TRAIN", "lane": "ci", "note": "gh-aw 0.87.3"},
    "103": {"disposition": "REOPEN THEN DEPENDABOT", "lane": "ci", "note": "actions/cache 6; DIRTY"},
    "141": {"disposition": "DEPENDABOT AFTER TRAIN", "lane": "ci", "note": "upload-artifact 7; DIRTY"},
    "106": {"disposition": "ANDROID TOOLCHAIN LANE", "lane": "android deps", "note": "lifecycle-service 2.11.0"},
    "107": {"disposition": "ANDROID TOOLCHAIN LANE", "lane": "android deps", "note": "mockk-android 1.14.11"},
    "108": {"disposition": "ANDROID TOOLCHAIN LANE", "lane": "android deps", "note": "core-ktx 1.19.0"},
    "210": {"disposition": "ANDROID TOOLCHAIN LANE", "lane": "android deps", "note": "coroutines-test 1.11.0"},
    "213": {"disposition": "ANDROID TOOLCHAIN LANE", "lane": "android deps", "note": "hilt-navigation-compose 1.4.0"},
    "156": {"disposition": "REBASE DOCS/CI", "lane": "ci", "note": "docker suite non-blocking"},
    "170": {"disposition": "OPERATOR DECISION", "lane": "orchestration", "note": "stale since 2026-08-16"},
    "178": {"disposition": "CLOSE", "lane": "ios", "note": "base is its own head ref"},
    "207": {"disposition": "REBASE DOCS", "lane": "apple continuity", "note": "stale since 2026-08-21"},
    "208": {"disposition": "OPERATOR DECISION", "lane": "apple parity", "note": "CONFLICTING"},
    "209": {"disposition": "BLOCKED", "lane": "identity unification", "note": "PIN 066039; base for #216/#220"},
    "216": {"disposition": "BLOCKED", "lane": "android wiring", "note": "draft on #209 base"},
    "218": {"disposition": "OPERATOR DECISION", "lane": "transport", "note": "draft self-referential circuit guard; Rule-8"},
    "220": {"disposition": "BLOCKED", "lane": "android reachability", "note": "draft on #209 base"},
    "227": {"disposition": "OPERATOR DECISION", "lane": "android degraded storage", "note": "draft; CONFLICTING"},
    "228": {"disposition": "OPERATOR DECISION", "lane": "ci hardening", "note": "draft; 14 gated files across crypto/privacy/routing/transport; Rule-8"},
    "363": {"disposition": "OPERATOR DECISION", "lane": "external bridge", "note": "draft; foreign-material handoff"},
}


def gh(args: List[str], timeout: int = 120) -> Optional[Any]:
    try:
        proc = subprocess.run(
            ["gh", *args], capture_output=True, text=True, timeout=timeout, check=True
        )
    except FileNotFoundError:
        print("[WARNING] gh not found; cannot enumerate PRs")
        return None
    except subprocess.TimeoutExpired:
        print(f"[WARNING] gh timed out: gh {' '.join(args)}")
        return None
    except subprocess.CalledProcessError as exc:
        print(f"[WARNING] gh failed ({exc.returncode}): {exc.stderr.strip()[:300]}")
        return None
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError:
        print(f"[WARNING] non-JSON gh output: gh {' '.join(args)}")
        return None


def load_disposition(path: str) -> Optional[Dict[str, Any]]:
    """Default table, or a JSON override keyed by PR number. None = unusable."""
    if not path:
        return DEFAULT_DISPOSITION
    try:
        data = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"[FAIL] --disposition {path}: {exc}")
        return None
    if not isinstance(data, dict):
        print(f"[FAIL] --disposition {path}: expected a JSON object keyed by PR number")
        return None
    return data


def handoff_status(files: List[str]) -> str:
    handoffs = [p for p in files if p.startswith("HANDOFF/")]
    if not handoffs:
        return "n/a"
    # Scope-block compliance is read from the PR head via git show; the GitHub
    # API does not expose file contents cheaply. The ledger reports COUNT and
    # the reviewer runs validate_handoff_scope.py --changed-from on the head.
    return f"{len(handoffs)} doc(s) -- run validate_handoff_scope.py --changed-from <head>"


def check_summary(number: int) -> str:
    try:
        proc = subprocess.run(
            ["gh", "pr", "checks", str(number), "--repo", REPO],
            capture_output=True, text=True, timeout=90,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return "[WARNING] checks unavailable"
    # gh pr checks output is "name<TAB>conclusion<TAB>duration<TAB>url<TAB>".
    # The conclusion is the SECOND field, not the last (the URL is last and
    # parsing it turns every check into an unknown conclusion).
    rows = [line.split("\t") for line in proc.stdout.splitlines() if "\t" in line]
    conclusions = [fields[1].strip() for fields in rows if len(fields) >= 2]
    if not conclusions:
        return "[WARNING] no checks reported"
    counts = Counter(CHECK_CLASS.get(c, "other") for c in conclusions)
    other = sorted({c for c in conclusions if c not in CHECK_CLASS})
    suffix = f" other={other}" if other else ""
    return (
        f"pass={counts['pass']} skip={counts['skip']} fail={counts['fail']} "
        f"pending={counts['pending']} (total={len(conclusions)}){suffix}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", default="tmp/pr_disposition_ledger.md")
    parser.add_argument("--disposition", default="", help="JSON file overriding DEFAULT_DISPOSITION")
    args = parser.parse_args()

    disposition = load_disposition(args.disposition)
    if disposition is None:
        return 2

    prs = gh([
        "pr", "list", "--repo", REPO, "--state", "open", "--limit", str(PR_LIST_LIMIT),
        "--json", "number,title,headRefName,headRefOid,baseRefName,isDraft,mergeable,mergeStateStatus,updatedAt,author,url,files",
    ])
    if prs is None:
        print("[FAIL] could not enumerate open PRs")
        return 2

    generated = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    rows: List[Dict[str, Any]] = []
    contradictions: List[str] = []
    warnings: List[str] = []
    for pr in sorted(prs, key=lambda p: p["number"]):
        number = str(pr["number"])
        files = [f["path"] for f in pr.get("files") or []]
        gated = [p for p in files if p.startswith(GATED_PREFIXES)]
        policy = disposition.get(number, {})
        if not isinstance(policy, dict):
            warnings.append(f"#{number}: disposition entry is not an object; using defaults")
            policy = {}
        # Explicit assignment, never **policy: a policy file that carries a
        # computed key (number, checks, head) must not rewrite the row.
        row = {
            "number": pr["number"],
            "title": pr["title"],
            "head": f"{pr['headRefName']}@{pr['headRefOid'][:8]}",
            "draft": bool(pr.get("isDraft")),
            "mergeable": pr.get("mergeable"),
            "state": pr.get("mergeStateStatus"),
            "updated": pr["updatedAt"],
            "author": (pr.get("author") or {}).get("login", ""),
            "file_count_api": len(files),
            "gated_files": gated,
            "handoff": handoff_status(files),
            "checks": check_summary(pr["number"]),
            "disposition": str(policy.get("disposition", "UNDECIDED")),
            "lane": str(policy.get("lane", "")),
            "note": str(policy.get("note", "no disposition recorded")),
        }
        if row["disposition"] == "LAND FIRST":
            if row["draft"]:
                contradictions.append(f"#{number} is LAND FIRST but is a draft")
            elif row["mergeable"] in (None, "", "UNKNOWN"):
                warnings.append(
                    f"#{number} is LAND FIRST but mergeability is {row['mergeable'] or 'null'}"
                    " (GitHub has not computed it); not verified this run"
                )
            elif row["mergeable"] != "MERGEABLE":
                contradictions.append(f"#{number} is LAND FIRST but mergeable={row['mergeable']}")
        rows.append(row)

    # A per-PR file list at exactly the API cap is a truncated lower bound,
    # not a count. Say it on the row itself (the cell a reader quotes) and in
    # the warnings block, rather than only in the header prose.
    for r in rows:
        if r["file_count_api"] >= PR_FILES_CAP:
            r["file_count_capped"] = True
            warnings.append(
                f"#{r['number']} file list is {r['file_count_api']} = the API cap;"
                " file_count_api is a lower bound; use"
                f" `git diff --name-only origin/main...{r['head'].split('@')[-1]}`"
            )
    if len(rows) >= PR_LIST_LIMIT:
        warnings.append(
            f"open PR list is {len(rows)} = the `--limit {PR_LIST_LIMIT}` cap;"
            " this ledger may be truncated; raise the limit and re-run"
        )

    seen = {str(r["number"]) for r in rows}
    for number in sorted(
        (n for n in disposition if n not in seen), key=lambda n: (not n.isdigit(), n)
    ):
        warnings.append(
            f"disposition recorded for #{number}, which is not open; remove it from the table"
        )

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        f"# Open-PR disposition ledger (generated {generated})",
        "",
        f"Source: `gh pr list --repo {REPO} --state open --limit {PR_LIST_LIMIT}` (API file list is",
        f"capped at {PR_FILES_CAP} files per PR; `file_count_api` is therefore a lower bound -- the",
        "authoritative count is `git diff --name-only origin/main...<head>` in a worktree).",
        "A row whose `files(api)` cell reads `N+` is AT the cap: a lower bound, not a count.",
        "This ledger is READ-ONLY evidence for the main lane; it is not a merge decision.",
        "",
        f"Open PRs: {len(rows)}" + (
            f" [WARNING] at the --limit {PR_LIST_LIMIT} cap; this is a ceiling, not a count"
            if len(rows) >= PR_LIST_LIMIT else ""
        ),
        "",
        "| PR | state | head | files(api) | gated | handoff | checks | disposition | lane |",
        "|---|---|---|---|---|---|---|---|---|",
    ]
    for r in rows:
        file_cell = (
            f"{r['file_count_api']}+" if r.get("file_count_capped") else str(r["file_count_api"])
        )
        lines.append(
            f"| #{r['number']}{' (draft)' if r['draft'] else ''} | {r['mergeable']}/{r['state']} "
            f"| `{r['head']}` | {file_cell} | {len(r['gated_files'])} | {r['handoff']} "
            f"| {r['checks']} | **{r['disposition']}** | {r['lane']} |"
        )
    lines.append("")
    lines.append("## Gated files per PR (full list, no cap)")
    for r in rows:
        if r["gated_files"]:
            lines.append(f"- #{r['number']}: " + ", ".join(f"`{p}`" for p in r["gated_files"]))
    lines.append("")
    lines.append("## Notes per PR")
    for r in rows:
        lines.append(f"- #{r['number']} -- {r['disposition']} ({r['lane']}): {r['note']}")
    lines.append("")
    if warnings:
        lines.append("## Warnings")
        lines.extend(f"- {w}" for w in warnings)
        lines.append("")
    if contradictions:
        lines.append("## Contradictions")
        lines.extend(f"- {c}" for c in contradictions)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    out.with_suffix(".json").write_text(
        json.dumps({"generated": generated, "warnings": warnings, "rows": rows}, indent=1),
        encoding="utf-8",
    )
    print(f"[OK] wrote {out} and {out.with_suffix('.json')} ({len(rows)} open PRs)")
    for w in warnings:
        print(f"[WARNING] {w}")
    if contradictions:
        for c in contradictions:
            print(f"[FAIL] {c}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
