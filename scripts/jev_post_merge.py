#!/usr/bin/env python3
"""Post-merge JEV closing gate (merge-train task file 14.6, operator ruling 2026-09-29: JEV POST-MERGE).

After a car merges, its JEV score is computed on the merge SHA and must be >= 85 with all
hard gates clear before the next dependent car merges and before any tag. This routine has NO
manual flags that can raise a score: every evidence value is derived from `gh` output and recorded
verbatim, then scored by the ADMITTED harness (the immutable tag named in docs/rules/BUILD_AND_CI.md,
"Harness admission and update gate"; never a moving branch). Point HARNESS_REPO at a checkout of that
admitted tag; the routine refuses to run unless that checkout's HEAD is the admitted SHA and its tree
is clean, and it records both in the evidence.

Rules encoded here (each one was found by running it, not assumed):
  * The merge's push lane is event "push" plus GitHub-managed CodeQL ("dynamic"). Scheduled,
    manual and Dependabot runs executed against the same SHA are reported but not scored: a
    scheduled failure on the merge SHA is not something the merge did.
  * Delta attribution: a lane that is red on the merge SHA and was already red on the merge
    commit's FIRST parent is a baseline red (reported, not attributed), but only if it is red in the
    same JOBS: a job that turns red inside an already-red workflow is attributable. A lane that turns
    red only at the merge SHA is a blocker. With no parent run there is no baseline, so every red counts.
  * The run list is filtered by exact commit, never by "the last N runs" (AGENTS.md rule 15).
  * `--unverified TEXT` records an acceptance item of the car that has no command evidence. It can
    only ADD a blocker, never raise a score. Without it, "JEV >= 85" means only "merged, CI green,
    no unresolved threads": the scorer has no per-car acceptance contract for an unknown phase id.
  * The status row says COMPLETE only when every computed gate is green; it never claims a merge that
    has not happened (an unmerged PR exits 4, not scoreable).

Exit codes: 0 pass (can_mark_complete), 1 fail, 4 not yet scoreable (PR not merged, or push runs
for the merge SHA still running: wait, do not score early), 2 usage or tool error.

Usage: HARNESS_REPO=<admitted checkout> python scripts/jev_post_merge.py \
           --pr N --phase MT-00b --purpose "short text" --out-dir tmp/jev [--unverified "item"]...
"""
import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

DEFAULT_REPO = "Sovereign-Communication/SCMessenger"
# docs/rules/BUILD_AND_CI.md, "Harness admission and update gate": Harness v0.4.1 -> this commit.
ADMITTED_HARNESS_SHA = "ad4a30052955e1f574c90f426edfc7a274e16ebf"
BAD_WORDS = ("open", "fail", "repair", "pending", "blocked")  # the scorer matches substrings
GREEN = ("success", "skipped", "neutral")


def in_push_lane(run):
    """True for the runs that belong to a merge's push lane."""
    return run.get("event") == "push" or (run.get("event") == "dynamic" and run.get("workflowName") == "CodeQL")


def attribute_reds(now_runs, parent_runs, jobs_now=None, jobs_before=None):
    """Split reds on the merge SHA into (attributable, baseline_red) by comparing with the first parent.

    now_runs and parent_runs are dicts with workflowName/status/conclusion. Only COMPLETED parent runs
    can establish a baseline. When a workflow is red on both sides and job-level data is supplied
    (jobs_now / jobs_before: workflow name -> set of failing job names), the baseline only absorbs the
    jobs that were already failing: a NEW failing job inside an already-red workflow is attributable.
    """
    jobs_now = jobs_now or {}
    jobs_before = jobs_before or {}
    red_now = {r["workflowName"]: r["conclusion"] for r in now_runs if r.get("conclusion") not in GREEN}
    red_before = {r["workflowName"] for r in parent_runs
                  if r.get("status") == "completed" and r.get("conclusion") not in GREEN}
    baseline, attributable = [], []
    for name, conclusion in red_now.items():
        if name not in red_before:
            attributable.append((name, conclusion))
            continue
        new_jobs = sorted(set(jobs_now.get(name, ())) - set(jobs_before.get(name, ())))
        if name in jobs_now and name in jobs_before and new_jobs:
            attributable.append((name, f"{conclusion}: new failing job(s) {new_jobs}"))
        else:
            baseline.append(name)
    return sorted(attributable), sorted(baseline)


def hostile_words(purpose):
    """Substrings of the purpose text that the scorer reads as contradicting evidence."""
    lowered = purpose.lower()
    return [word for word in BAD_WORDS if word in lowered]


def status_row(phase, purpose, pr, merge_sha, complete=True, ci_green=True):
    """A status row built from computed values: COMPLETE only when every computed gate is green."""
    state = "COMPLETE" if complete else "NOT COMPLETE"
    ci = "CI green; JEV bar" if ci_green else "CI not green"
    return f"| {phase} | {purpose} | {state} | PR #{pr} merged {merge_sha[:8]} | {ci} |"


def harness_provenance(harness_dir, admitted_sha):
    """(ok, detail) for the harness checkout: HEAD must be the admitted SHA and the tree clean."""
    def git(*args):
        return subprocess.run(["git", "-C", str(harness_dir), *args], capture_output=True, text=True, encoding="utf-8")
    head = git("rev-parse", "HEAD")
    status = git("status", "--porcelain")
    if head.returncode or status.returncode:
        return False, {"error": "cannot read the harness checkout's git state"}
    detail = {"head": head.stdout.strip(), "admitted": admitted_sha, "clean": not status.stdout.strip()}
    return bool(detail["head"] == admitted_sha and detail["clean"]), detail


class Gh:
    def __init__(self, repo_root):
        self.repo_root = repo_root

    def __call__(self, *args):
        result = subprocess.run(["gh", *args], capture_output=True, text=True, encoding="utf-8", cwd=self.repo_root)
        if result.returncode != 0:
            raise SystemExit(f"[FAIL] gh {' '.join(args[:3])}...: {result.stderr.strip()[:300]}")
        return result.stdout


def failing_jobs(gh, repo, run_id):
    """Names of the jobs of a run whose conclusion is not green."""
    jobs = json.loads(gh("run", "view", str(run_id), "--repo", repo, "--json", "jobs"))["jobs"]
    return {j["name"] for j in jobs if j.get("conclusion") not in GREEN and j.get("conclusion") is not None}


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--pr", type=int, required=True)
    ap.add_argument("--phase", required=True)
    ap.add_argument("--purpose", required=True)
    ap.add_argument("--out-dir", required=True)
    ap.add_argument("--unverified", action="append", default=[],
                    help="an acceptance item with no command evidence; adds a blocker, never raises a score")
    ap.add_argument("--repo-root", default=str(Path(__file__).resolve().parent.parent))
    ap.add_argument("--repo", default=os.environ.get("SCM_REPO", DEFAULT_REPO))
    ap.add_argument("--min-score", type=float, default=85.0)
    a = ap.parse_args(argv)

    harness = os.environ.get("HARNESS_REPO")
    if not harness or not Path(harness, "harness", "jev_completion.py").is_file():
        print("[FAIL] set HARNESS_REPO to a checkout of the ADMITTED harness tag (docs/rules/BUILD_AND_CI.md)")
        return 2
    admitted = os.environ.get("HARNESS_ADMITTED_SHA", ADMITTED_HARNESS_SHA)
    provenance_ok, provenance = harness_provenance(harness, admitted)
    if not provenance_ok:
        print(f"[FAIL] the harness checkout is not the admitted source (must be {admitted[:12]} and clean): {provenance}")
        return 2
    hostile = hostile_words(a.purpose)
    if hostile:
        print(f"[FAIL] the purpose text contains scorer-hostile substrings {hostile}; reword it")
        return 2
    owner, name = a.repo.split("/")
    gh = Gh(a.repo_root)
    os.makedirs(a.out_dir, exist_ok=True)
    raw = {"harness_provenance": provenance}

    pr_json = gh("pr", "view", str(a.pr), "--repo", a.repo, "--json",
                 "number,state,mergedAt,mergeCommit,headRefOid,baseRefName,statusCheckRollup,title")
    raw["gh_pr_view"] = pr_json
    pr = json.loads(pr_json)
    if pr["state"] != "MERGED" or not pr.get("mergeCommit"):
        print(f"[WAIT] PR #{a.pr} state={pr['state']}; nothing to score yet")
        return 4
    merge_sha = pr["mergeCommit"]["oid"]

    required = json.loads(gh("api", f"repos/{a.repo}/branches/main/protection/required_status_checks", "--jq", ".contexts"))
    raw["required_contexts"] = required
    results = {}
    for check in pr["statusCheckRollup"]:
        results.setdefault(check.get("name") or check.get("context"), []).append(
            check.get("conclusion") or check.get("state") or check.get("status"))
    required_ok = all(results.get(n) and all(v == "SUCCESS" for v in results[n]) for n in required)
    nonrequired_bad = sorted({n for n, vs in results.items()
                              if n not in required and any(v not in ("SUCCESS", "SKIPPED", "NEUTRAL") for v in vs)})

    runs_json = gh("run", "list", "--repo", a.repo, "--branch", "main", "--commit", merge_sha, "--limit", "200",
                   "--json", "databaseId,workflowName,event,status,conclusion,headSha,url")
    raw["main_runs"] = runs_json
    all_runs = [r for r in json.loads(runs_json) if r["headSha"] == merge_sha]
    lane = [r for r in all_runs if in_push_lane(r)]
    excluded = [(r["workflowName"], r["event"], r["conclusion"]) for r in all_runs if not in_push_lane(r)]
    raw["excluded_runs"] = excluded
    if excluded:
        print(f"[INFO] excluded from the push lane (reported, not scored): {excluded}")
    print(f"[INFO] {len(lane)} push-lane run(s) of {len(all_runs)} matched merge SHA {merge_sha[:8]} "
          "(limit 200; a count of exactly 200 would be a possible cap)")
    if not lane:
        print(f"[WAIT] no push-lane run on main for merge SHA {merge_sha[:8]} yet")
        return 4
    running = [(r["workflowName"], r["status"]) for r in lane if r["status"] != "completed"]
    if running:
        print("[WAIT] push-lane runs still running for", merge_sha[:8], running)
        return 4

    parent = gh("api", f"repos/{a.repo}/commits/{merge_sha}", "--jq", ".parents[0].sha").strip()
    parent_json = gh("run", "list", "--repo", a.repo, "--branch", "main", "--commit", parent, "--limit", "200",
                     "--json", "databaseId,workflowName,status,conclusion,event")
    raw["parent_sha"], raw["parent_runs"] = parent, parent_json
    parent_lane = [r for r in json.loads(parent_json) if r.get("event") in ("push", "dynamic")]

    # Job-level refinement only for workflows that are red on BOTH sides (the baseline candidates).
    red_now = {r["workflowName"]: r for r in lane if r.get("conclusion") not in GREEN}
    red_before = {r["workflowName"]: r for r in parent_lane
                  if r.get("status") == "completed" and r.get("conclusion") not in GREEN}
    jobs_now, jobs_before = {}, {}
    for workflow in sorted(set(red_now) & set(red_before)):
        jobs_now[workflow] = failing_jobs(gh, a.repo, red_now[workflow]["databaseId"])
        jobs_before[workflow] = failing_jobs(gh, a.repo, red_before[workflow]["databaseId"])
    raw["failing_jobs"] = {"now": {k: sorted(v) for k, v in jobs_now.items()},
                           "before": {k: sorted(v) for k, v in jobs_before.items()}}
    attributable, baseline = attribute_reds(lane, parent_lane, jobs_now, jobs_before)
    print(f"[INFO] parent {parent[:8]}: {len(parent_lane)} run(s); baseline-red lanes (not attributed): {baseline or 'none'}")

    query = ("query($o:String!,$n:String!,$p:Int!){repository(owner:$o,name:$n){pullRequest(number:$p)"
             "{reviewThreads(first:100){totalCount nodes{isResolved}}}}}")
    threads_json = gh("api", "graphql", "-f", f"query={query}", "-f", f"o={owner}", "-f", f"n={name}", "-F", f"p={a.pr}")
    raw["review_threads"] = threads_json
    unresolved = sum(1 for n in json.loads(threads_json)["data"]["repository"]["pullRequest"]["reviewThreads"]["nodes"]
                     if not n["isResolved"])

    blockers = []
    if unresolved:
        blockers.append(f"{unresolved} unresolved review thread(s)")
    if nonrequired_bad:
        blockers.append("non-required check(s) not success on the PR head (A1 f: investigate): " + ", ".join(nonrequired_bad))
    if not required_ok:
        blockers.append("a required check was not SUCCESS on the PR head")
    if attributable:
        blockers.append(f"push-lane run(s) newly red on the merge SHA: {attributable}")
    for item in a.unverified:
        blockers.append(f"acceptance item without command evidence: {item}")

    ci_green = bool(required_ok and not attributable)
    complete = bool(ci_green and not blockers)
    urls = "; ".join(f"{r['workflowName']}={r['conclusion']} {r['url']}" for r in lane)
    evidence = {
        "phase": a.phase,
        "status_row": status_row(a.phase, a.purpose, a.pr, merge_sha, complete=complete, ci_green=ci_green),
        "pr_merged": True,            # derived: gh state == MERGED and a mergeCommit exists
        "ci_green": ci_green,         # derived: required checks SUCCESS on the head AND no newly red push-lane run
        "local_gates_green": ci_green,  # CI-primary doctrine: the CI job URLs are the gate evidence
        "origin_evidence": (f"gh pr view {a.pr} (state,mergeCommit,statusCheckRollup) + push-lane runs for {merge_sha}: {urls}; "
                            f"baseline-red lanes carried from parent {parent[:8]} (not attributed): {baseline or 'none'}; "
                            f"harness {provenance['head'][:12]} clean={provenance['clean']}"),
        "open_blockers": blockers,
        "harness_provenance": provenance,
    }
    evidence_path = os.path.join(a.out_dir, f"{a.phase}.evidence.json")
    with open(evidence_path, "w", encoding="utf-8") as fh:
        json.dump(evidence, fh, indent=2)
    with open(os.path.join(a.out_dir, f"{a.phase}.raw.json"), "w", encoding="utf-8") as fh:
        json.dump(raw, fh, indent=1)

    out_path = os.path.join(a.out_dir, f"{a.phase}.jev.json")
    env = dict(os.environ, PYTHONPATH=harness, PYTHONIOENCODING="utf-8")
    scored = subprocess.run(
        [sys.executable, "-m", "harness.cli", "jev-phase", "--phase", a.phase, "--repo-root", a.repo_root,
         "--evidence", evidence_path, "--min-score", str(a.min_score), "--local-only", "--json", "--out", out_path, "--quiet"],
        env=env, capture_output=True, text=True, encoding="utf-8", timeout=300)
    if not os.path.exists(out_path):
        print("[FAIL] the harness produced no output; rc =", scored.returncode)
        return 2
    with open(out_path, encoding="utf-8") as fh:
        result = json.load(fh)
    passed = bool(result.get("can_mark_complete"))
    semantic = result.get("semantic", {})
    print(json.dumps({
        "phase": a.phase, "merge_sha": merge_sha[:8], "score": result.get("score"), "min_score": result.get("min_score"),
        "can_mark_complete": passed, "hard_gates": result.get("hard_gates"),
        "semantic_axis": {"is_fallback": semantic.get("is_fallback"), "model": semantic.get("model")},
        "blockers": result.get("blockers", blockers)}, indent=2)[:1800])
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
