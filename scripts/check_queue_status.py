#!/usr/bin/env python3
"""Fail when a queue ticket's status line advertises work that has already landed.

WHY THIS EXISTS
---------------
`HANDOFF/freebuff/queue/*.md` is the dispatch authority: a lane reads a ticket's
status line to decide whether the work still needs doing. On 2026-09-19 four
tickets still said "PR FILED -- #NNN open ... awaiting review" for PRs that had
merged on 2026-09-03 (T8/#271, T6/#311, T14/#270, T14/#269), and a fifth (T12)
cited the wrong PR entirely. One of them was picked up as fresh work on the
strength of its own status line. That class is mechanical, so it is checked
mechanically here rather than noticed by luck.

WHAT COUNTS AS A CLAIM (and what deliberately does not)
-------------------------------------------------------
Only an explicit openness phrase counts: "PR FILED", "#NNN open", "in review",
"awaiting (adversarial) review". A bare "PR #NNN" does NOT, because these files
are also review-dispatch RECORDS whose status line ends "REVIEW FILED, RESOLVED"
and whose `Target: PR #267 ...` line sits in the same indented block - the first
draft of this check failed all seven of those, which is how the rule below was
found rather than assumed.

Three narrowings, each paid for by an observed false positive:
  - the statement ends where the next `Key:` metadata label begins, so a `Target:`
    citation is never read as a status claim;
  - parenthesised sub-clauses are dropped, so "(NOT in PR #267 -- CEO ruling)" is
    not judged as a citation;
  - clauses containing a negation are dropped for the same reason.

WHAT IT CANNOT SEE
------------------
A ticket claiming to be OPEN while its work is already implemented on main, with
no PR number to query (T12's other half), is not detectable from a citation. This
check only knows what a status statement cites.

FAIL-OPEN ON INFRASTRUCTURE, FAIL-CLOSED ON FINDINGS
---------------------------------------------------
If `gh` is missing, unauthenticated, or the API cannot be reached, every affected
citation is reported as [WARNING] and the exit code stays 0: a network problem
must never turn into a red gate. An observed contradiction always exits 1.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

QUEUE_DIR = Path("HANDOFF/freebuff/queue")

# An explicit claim that submitted work is still open. See the docstring: a bare
# "PR #NNN" is deliberately absent.
OPEN_CLAIM_RE = re.compile(
    r"PR\s*FILED"
    r"|#\d{1,6}\s+open\b"
    r"|awaiting\s+(?:adversarial\s+|independent\s+)?review"
    r"|in\s+review|under\s+review",
    re.IGNORECASE,
)
MERGED_CLAIM_RE = re.compile(r"\bMERGED\b|\bLANDED\b|\bDONE\b|\bRESOLVED\b", re.IGNORECASE)
NEGATION_RE = re.compile(r"\bnot\b|\bnever\b|\bwithout\b|\bexcluded\b", re.IGNORECASE)
LABEL_RE = re.compile(r"^[A-Z][A-Za-z0-9 /-]{1,30}:")
PAREN_RE = re.compile(r"\([^()]*\)")
CLAUSE_SPLIT_RE = re.compile(r";|,|--|\u2014")
CITE_RES = (
    re.compile(r"#(\d{1,6})\b"),
    re.compile(r"/pull/(\d{1,6})\b"),
)

FAIL = "FAIL"
WARN = "WARNING"
OK = "OK"


def status_statement(text: str) -> str:
    """The status statement: the 'Status:' line plus continuation lines, up to the
    next metadata label or blank line."""
    lines = text.split("\n")
    try:
        start = next(i for i, ln in enumerate(lines) if ln.startswith("Status:"))
    except StopIteration:
        return ""
    out = [lines[start]]
    for line in lines[start + 1:]:
        if line.strip() == "" or LABEL_RE.match(line.strip()):
            break
        out.append(line)
    return "\n".join(out).strip()


def positive_part(statement: str) -> str:
    """The statement minus parentheticals and minus negated clauses."""
    kept = []
    for clause in CLAUSE_SPLIT_RE.split(PAREN_RE.sub(" ", statement)):
        if NEGATION_RE.search(clause):
            continue
        kept.append(clause)
    return " ".join(kept)


def cited_prs(text: str) -> list[str]:
    found: list[str] = []
    for rx in CITE_RES:
        for num in rx.findall(text):
            if num not in found:
                found.append(num)
    return found


def classify(statement: str, pr_state) -> list[tuple[str, str]]:
    """Pure classification. pr_state(number) -> dict|None|'ERROR'. Kept pure so
    --self-test can drive it with a stub instead of the network."""
    findings: list[tuple[str, str]] = []
    if not statement:
        return findings
    positive = positive_part(statement)
    claims_open = bool(OPEN_CLAIM_RE.search(positive))
    claims_merged = bool(MERGED_CLAIM_RE.search(positive))
    numbers = cited_prs(positive)

    if claims_open and not claims_merged and not numbers:
        findings.append(
            (WARN, "status claims filed/in-review work but cites no PR number or URL, "
                   "so nothing can confirm it is still open")
        )
        return findings

    for number in numbers:
        info = pr_state(number)
        if info == "ERROR":
            findings.append(
                (WARN, f"could not read PR #{number} (gh unavailable, unauthenticated, or "
                       f"the API failed) -- state UNVERIFIED, not judged")
            )
            continue
        if info is None:
            findings.append((WARN, f"cites PR #{number}, which does not exist in this repository"))
            continue
        state = info["state"]
        when = info.get("mergedAt") or info.get("closedAt") or "unknown time"
        if claims_open and not claims_merged and state in ("MERGED", "CLOSED"):
            findings.append(
                (FAIL, f"claims PR #{number} is open/awaiting review, but it is {state} ({when}) "
                       f"-- the status line is stale and will misdispatch a lane")
            )
        elif claims_merged and state == "OPEN":
            findings.append(
                (FAIL, f"claims PR #{number} has landed, but it is still OPEN -- the status line "
                       f"is wrong in the other direction")
            )
    return findings


def gh_lookup(number: str):
    """Query GitHub. Returns dict, None (does not exist), or 'ERROR' (infrastructure)."""
    try:
        proc = subprocess.run(
            ["gh", "pr", "view", number, "--json", "number,state,mergedAt,closedAt,title"],
            capture_output=True, text=True, timeout=60,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return "ERROR"
    if proc.returncode != 0:
        blob = (proc.stderr or "") + proc.stdout
        if "Could not resolve to a PullRequest" in blob or "not found" in blob.lower():
            return None
        return "ERROR"
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError:
        return "ERROR"


def self_test() -> int:
    """Prove the classifier is not vacuous, with no network involved."""
    merged = {"state": "MERGED", "mergedAt": "2026-09-03T11:26:07Z"}
    open_pr = {"state": "OPEN", "mergedAt": None, "closedAt": None}
    closed = {"state": "CLOSED", "closedAt": "2026-09-19T10:00:00Z"}

    def lookup(table):
        return lambda number: table.get(number, None)

    cases = [
        # The shapes of the four real stale tickets, verbatim.
        ("Status: PR FILED -- #271 open, CI-VERIFIED; awaiting review.",
         lookup({"271": merged}), FAIL),
        ("Status: PR FILED -- #311 open (branch `x`); all four checks passed.",
         lookup({"311": merged}), FAIL),
        ("Status: PR FILED -- #270 open (filed 2026-09-01, observed live); awaiting "
         "adversarial review.", lookup({"270": closed}), FAIL),
        ("Status: PR FILED -- #269 open (NOT in PR #267 -- CEO ruling); awaiting "
         "adversarial review.", lookup({"269": merged, "267": merged}), FAIL),
        # The wrong-number case from this thread: the citation was a closed probe PR.
        ("Status: IN REVIEW -- PR #317 (platform half). Filed 2026-09-19.",
         lookup({"317": closed}), FAIL),
        # Not stale: the cited PR really is still open.
        ("Status: PR FILED -- #270 open; awaiting adversarial review.",
         lookup({"270": open_pr}), OK),
        # A correct post-merge ticket.
        ("Status: MERGED -- this work landed as PR #271 (merged 2026-09-03).",
         lookup({"271": merged}), OK),
        # A bare ticket status is a claim about the ticket, never a PR claim.
        ("Status: OPEN (filed 2026-08-31, CEO audit)", lookup({}), OK),
        ("Status: RESOLVED ON MAIN -- verified 2026-09-01", lookup({}), OK),
        # Claims filed work with nothing to verify against.
        ("Status: PR FILED -- awaiting review.", lookup({}), WARN),
        # Reverse-direction drift.
        ("Status: MERGED -- landed as PR #270.", lookup({"270": open_pr}), FAIL),
        # URL citation form.
        ("Status: PR FILED -- https://github.com/o/r/pull/270 awaiting review",
         lookup({"270": merged}), FAIL),
        # Infrastructure failure warns, never fails.
        ("Status: PR FILED -- #270 open", lookup({"270": "ERROR"}), WARN),
        # FALSE-POSITIVE GUARDS, each paid for by a real misfire:
        # a review-dispatch RECORD, whose Target: label cites a merged PR.
        ("Status: DISPATCHED 2026-09-01 -- REVIEW FILED, RESOLVED. PR #267 body carries "
         "the triage table.\nTarget: PR **#267** `branch`\n", lookup({"267": merged}), OK),
        # the negated parenthetical citation must not be judged on its own.
        ("Status: PR FILED -- #269 open (NOT in PR #267); awaiting adversarial review.",
         lookup({"269": open_pr, "267": merged}), OK),
        # a label line must not contribute citations to the statement.
        ("Status: PR FILED -- awaiting review.\nTarget: PR #267 x\n",
         lookup({"267": merged}), WARN),
    ]
    failures = 0
    for fixture, lookup_fn, expected in cases:
        # Route fixtures through the same extraction production uses, so the
        # statement boundary (the "Target:" label rule) is exercised too.
        statement = status_statement(fixture)
        got = [level for level, _ in classify(statement, lookup_fn)]
        level = FAIL if FAIL in got else (WARN if WARN in got else OK)
        mark = "OK" if level == expected else "ERROR"
        if level != expected:
            failures += 1
        print(f"  [{mark}] expected {expected}, got {level}: {statement.splitlines()[0][:66]}")
    if failures:
        print(f"[ERROR] self-test: {failures} of {len(cases)} cases failed")
        return 1
    print(f"[OK] self-test: {len(cases)} classification cases behave as specified")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--queue-dir", default=str(QUEUE_DIR))
    ap.add_argument("--changed-only", action="store_true",
                    help="only inspect tickets changed between --base-ref and HEAD")
    ap.add_argument("--base-ref", default="",
                    help="base commit for --changed-only (PR base sha, or event.before on push)")
    ap.add_argument("--warn-only", action="store_true",
                    help="report findings but always exit 0 (used for main pushes, where a merge "
                         "is itself the event that makes a citation stale)")
    ap.add_argument("--self-test", action="store_true", help="run classifier fixtures and exit")
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    # Always exercise the classifier before trusting its verdict, so a broken check
    # cannot report success by having stopped comparing (see I-21).
    rc = self_test()
    if rc != 0:
        print("[ERROR] refusing to report a verdict from a failing self-test")
        return rc

    queue = Path(args.queue_dir)
    if not queue.is_dir():
        print(f"[WARNING] {queue} does not exist; nothing to check (failing open)")
        return 0

    tickets = sorted(queue.glob("*.md"))
    if args.changed_only:
        if not args.base_ref:
            print("[WARNING] --changed-only without --base-ref; failing open, scanning nothing")
            return 0
        proc = subprocess.run(
            ["git", "diff", "--name-only", args.base_ref, "HEAD", "--", str(queue)],
            capture_output=True, text=True,
        )
        if proc.returncode != 0:
            print(f"[WARNING] could not diff against {args.base_ref}; failing open "
                  f"({proc.stderr.strip()[:200]})")
            return 0
        changed = {Path(p).name for p in proc.stdout.split() if p.endswith(".md")}
        tickets = [t for t in tickets if t.name in changed]
        print(f"[INFO] changed-only: {len(tickets)} ticket(s) changed since {args.base_ref[:8]}")

    findings: list[tuple[str, str]] = []
    no_status: list[str] = []
    cited: dict[str, object] = {}
    for ticket in tickets:
        statement = status_statement(ticket.read_text(encoding="utf-8"))
        if not statement:
            no_status.append(ticket.name)
            continue
        for number in cited_prs(positive_part(statement)):
            if number not in cited:
                cited[number] = gh_lookup(number)
        for level, message in classify(statement, lambda n: cited[n]):
            findings.append((level, f"{ticket.name}: {message}"))

    failures = [f for f in findings if f[0] == FAIL]
    warnings = [f for f in findings if f[0] == WARN]

    print(f"[INFO] scanned {len(tickets)} file(s); resolved {len(cited)} cited PR(s): "
          f"{', '.join('#' + n for n in sorted(cited, key=int)) or 'none'}")
    if no_status:
        print(f"[INFO] {len(no_status)} file(s) carry no Status: statement and cannot be judged: "
              f"{', '.join(no_status)}")
    for level, message in failures + warnings:
        print(f"[{level}] {message}")
    if not findings:
        print("[OK] every cited PR matches the claim its ticket's status statement makes")

    if failures and not args.warn_only:
        print(f"[ERROR] {len(failures)} stale status statement(s), {len(warnings)} warning(s)")
        return 1
    if failures:
        print(f"[WARNING] {len(failures)} stale status statement(s) reported but not enforced "
              f"(--warn-only); {len(warnings)} further warning(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
