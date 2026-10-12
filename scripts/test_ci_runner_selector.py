#!/usr/bin/env python3
"""Deterministic proof that the CI_RUNNER selector (PR 557) weakens no check.

The `light` scope token (blacksmith-2vcpu-ubuntu-2404) covers the small gate jobs:
ci.yml changes/handoff-scope/naming-policy/lint/orchestration, lint.yml
changes/rust, hygiene.yml hygiene-checks. They fall back to ubuntu-latest.

Evaluates the real `runs-on` expression of every affected job (ci.yml `test`,
`docs`, `ffi-surface`; security-regression-tests.yml `forgery-regression`) with
a small GitHub-Actions expression evaluator, for every combination of:

  CI_RUNNER                  unset | blacksmith | oci | hosted
  CI_RUNNER_BLACKSMITH_JOBS  unset | test | pr | test,pr | docs,ffi,forgery,pr | ...
  USE_OCI_RUNNER             unset | false | true
  event                      push | same-repo PR | fork PR | dispatch | schedule
  matrix.os                  ubuntu-latest | macos-latest  (test job only)

and asserts: runs-on is never empty/invalid; fork PRs and non-push/PR events
always get the hosted default; CI_RUNNER unset reproduces the legacy runner
exactly; hosted always means ubuntu-latest; the job keeps its name, `needs`,
`if`, `strategy.matrix`, and a non-empty step list with no continue-on-error.
With --base-ref REF the job bodies are also compared with REF: only runs-on
may differ.

Usage: python3 scripts/test_ci_runner_selector.py [--base-ref origin/main]
"""

import argparse
import itertools
import json
import os
import re
import subprocess
import sys

import yaml

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
REPO = "org/repo"

# (file, job id, expected name, expected needs, expected `if`, legacy runner kind)
JOBS = [
    (".github/workflows/ci.yml", "test", None, "changes", "${{ !cancelled() }}", "matrix"),
    (".github/workflows/ci.yml", "docs", "Docs", "changes", "needs.changes.outputs.rust_relevant == 'true'", "oci"),
    (".github/workflows/ci.yml", "ffi-surface", "FFI Surface Contract", "changes", "needs.changes.outputs.rust_relevant == 'true'", "oci"),
    (".github/workflows/security-regression-tests.yml", "forgery-regression", "Forgery/Negative Security Regression Tests", None, None, "oci"),
    # `light` token: small gate jobs, hosted by default, blacksmith-2vcpu when selected.
    (".github/workflows/ci.yml", "changes", "Detect changed areas", None, None, "light"),
    (".github/workflows/ci.yml", "handoff-scope", "Handoff ownership scope", None, None, "light"),
    (".github/workflows/ci.yml", "naming-policy", "Naming policy", None, None, "light"),
    (".github/workflows/ci.yml", "lint", "Lint", "changes", "${{ !cancelled() }}", "light"),
    (".github/workflows/ci.yml", "orchestration", "Orchestration control plane", None, None, "light"),
    (".github/workflows/lint.yml", "changes", "Detect changed areas", None, None, "light"),
    (".github/workflows/lint.yml", "rust", "Rust Linting", "changes", "${{ !cancelled() }}", "light"),
    (".github/workflows/hygiene.yml", "hygiene-checks", "Repository Hygiene Checks", None, None, "light"),
]
# Light jobs whose required check must keep failing closed when `changes` failed.
FAIL_CLOSED = {("ci.yml", "lint"), ("lint.yml", "rust")}
HOSTED = "ubuntu-latest"
BLACKSMITH = "blacksmith-4vcpu-ubuntu-2404"
BLACKSMITH_LIGHT = "blacksmith-2vcpu-ubuntu-2404"
OCI = ["self-hosted", "oci"]


# ---- minimal GitHub Actions expression evaluator --------------------------
TOKEN = re.compile(r"\s*(?:(\|\||&&|==|!=|!|\(|\)|,)|'((?:[^']|'')*)'|([A-Za-z_][\w.]*))")


def _tokens(src):
    pos, out = 0, []
    src = src.strip()
    while pos < len(src):
        m = TOKEN.match(src, pos)
        if not m:
            raise ValueError("bad expression at %r" % src[pos:pos + 20])
        if m.group(1):
            out.append(("op", m.group(1)))
        elif m.group(2) is not None:
            out.append(("str", m.group(2).replace("''", "'")))
        else:
            out.append(("id", m.group(3)))
        pos = m.end()
    return out


def _truthy(v):
    return not (v is None or v is False or v == "" or v == 0)


def _norm(v):
    return "" if v is None else (v.lower() if isinstance(v, str) else v)


class Evaluator:
    def __init__(self, src, ctx):
        self.t, self.i, self.ctx = _tokens(src), 0, ctx

    def peek(self):
        return self.t[self.i] if self.i < len(self.t) else (None, None)

    def eat(self, val=None):
        tok = self.peek()
        if val is not None and tok[1] != val:
            raise ValueError("expected %s got %s" % (val, tok))
        self.i += 1
        return tok

    def parse(self):
        v = self.or_()
        if self.i != len(self.t):
            raise ValueError("trailing tokens")
        return v

    def or_(self):
        v = self.and_()
        while self.peek() == ("op", "||"):
            self.eat()
            r = self.and_()
            v = v if _truthy(v) else r
        return v

    def and_(self):
        v = self.eq()
        while self.peek() == ("op", "&&"):
            self.eat()
            r = self.eq()
            v = r if _truthy(v) else v
        return v

    def eq(self):
        v = self.unary()
        while self.peek()[1] in ("==", "!="):
            op = self.eat()[1]
            r = self.unary()
            same = _norm(v) == _norm(r)
            v = same if op == "==" else not same
        return v

    def unary(self):
        if self.peek() == ("op", "!"):
            self.eat()
            return not _truthy(self.unary())
        return self.atom()

    def atom(self):
        kind, val = self.eat()
        if kind == "str":
            return val
        if kind == "op" and val == "(":
            v = self.or_()
            self.eat(")")
            return v
        if kind == "id":
            if self.peek() == ("op", "("):
                self.eat()
                args = []
                if self.peek() != ("op", ")"):
                    args.append(self.or_())
                    while self.peek() == ("op", ","):
                        self.eat()
                        args.append(self.or_())
                self.eat(")")
                return self.call(val.lower(), args)
            if val in ("true", "false"):
                return val == "true"
            if val == "null":
                return None
            return self.ctx.get(val)
        raise ValueError("unexpected %s" % ((kind, val),))

    def call(self, name, args):
        if name == "contains":
            return str(_norm(args[1])) in str(_norm(args[0]))
        if name == "format":
            s = args[0]
            for n, a in enumerate(args[1:]):
                s = s.replace("{%d}" % n, "" if a is None else str(a))
            return s
        if name == "fromjson":
            return json.loads(args[0])
        raise ValueError("unsupported function " + name)


def runs_on_value(expr, ctx):
    m = re.fullmatch(r"\$\{\{(.*)\}\}", expr.strip(), re.S)
    return Evaluator(m.group(1), ctx).parse() if m else expr.strip()


def legacy(kind, ctx):
    """Runner each job used before the selector existed."""
    if kind == "matrix":
        return ctx["matrix.os"]
    if kind == "light":
        return HOSTED  # light jobs were always plain ubuntu-latest
    trusted = ctx["github.event_name"] == "push" or (
        ctx["github.event_name"] == "pull_request"
        and ctx["github.event.pull_request.head.repo.full_name"] == ctx["github.repository"]
    )
    if trusted and ctx.get("vars.USE_OCI_RUNNER") != "false":
        return OCI
    return HOSTED


def contexts():
    runners = [None, "blacksmith", "oci", "hosted"]
    scopes = [None, "test", "pr", "test,pr", "docs,ffi,forgery,pr", "test,docs,ffi,forgery,pr", "docs", "garbage",
              "light", "light,pr", "test,light,pr", "light,docs,ffi,forgery,pr",
              "test,docs,ffi,forgery,light,pr", "lightweight,pr", "Light,pr", " light,pr"]
    use_oci = [None, "false", "true"]
    events = {
        "push": ("push", REPO),
        "same_repo_pr": ("pull_request", REPO),
        "fork_pr": ("pull_request", "fork/repo"),
        "dispatch": ("workflow_dispatch", None),
        "schedule": ("schedule", None),
    }
    for r, s, u, (ev, (ev_name, head)), os_ in itertools.product(
        runners, scopes, use_oci, events.items(), ["ubuntu-latest", "macos-latest"]
    ):
        ctx = {
            "github.event_name": ev_name,
            "github.repository": REPO,
            "github.event.pull_request.head.repo.full_name": head,
            "matrix.os": os_,
        }
        if r is not None:
            ctx["vars.CI_RUNNER"] = r
        if s is not None:
            ctx["vars.CI_RUNNER_BLACKSMITH_JOBS"] = s
        if u is not None:
            ctx["vars.USE_OCI_RUNNER"] = u
        yield ev, ctx


def load(rel, ref=None):
    if ref:
        text = subprocess.run(
            ["git", "show", "%s:%s" % (ref, rel)], cwd=ROOT, capture_output=True, text=True, check=True
        ).stdout
    else:
        with open(os.path.join(ROOT, rel), encoding="utf-8") as fh:
            text = fh.read()
    return yaml.safe_load(text)["jobs"]


def check(base_ref=None):
    fails, n = [], 0
    for rel, jid, name, needs, cond, kind in JOBS:
        job = load(rel)[jid]
        expr = job["runs-on"]
        where = "%s:%s" % (rel, jid)
        if name is not None and job.get("name") != name:
            fails.append("%s: name changed to %r" % (where, job.get("name")))
        if job.get("needs") != needs:
            fails.append("%s: needs changed to %r" % (where, job.get("needs")))
        if job.get("if") != cond:
            fails.append("%s: if changed to %r" % (where, job.get("if")))
        if not job.get("steps"):
            fails.append("%s: no steps" % where)
        for st in job.get("steps", []):
            if st.get("continue-on-error") not in (None, False):
                fails.append("%s: continue-on-error step %r" % (where, st.get("name")))
        if kind == "light" and (os.path.basename(rel), jid) in FAIL_CLOSED:
            if not any("Fail closed" in (st.get("name") or "") and "needs.changes.result != 'success'" in str(st.get("if")) for st in job.get("steps", [])):
                fails.append("%s: fail-closed guard on `changes` result missing" % where)
        if kind == "matrix":
            m = job["strategy"]["matrix"]
            if "ubuntu-latest" not in m.get("os", []) or job["strategy"].get("fail-fast") is not False:
                fails.append("%s: matrix/fail-fast changed" % where)
        if base_ref:
            old = load(rel, base_ref)[jid]
            a = {k: v for k, v in job.items() if k != "runs-on"}
            b = {k: v for k, v in old.items() if k != "runs-on"}
            if a != b:
                diff = sorted(k for k in set(a) | set(b) if a.get(k) != b.get(k))
                fails.append("%s: differs from %s beyond runs-on: %s" % (where, base_ref, diff))
        for ev, ctx in contexts():
            if kind != "matrix" and ctx["matrix.os"] != "ubuntu-latest":
                continue  # matrix.os only exists for the test job
            n += 1
            got = runs_on_value(expr, ctx)
            shown = {k: v for k, v in ctx.items() if k.startswith(("vars", "matrix"))}
            tag = "%s ev=%s %s" % (where, ev, shown)
            if got in (None, "", [], False) or (isinstance(got, list) and not all(got)):
                fails.append("EMPTY runs-on: " + tag)
                continue
            if not (isinstance(got, str) or (isinstance(got, list) and all(isinstance(x, str) for x in got))):
                fails.append("INVALID runs-on %r: %s" % (got, tag))
                continue
            if got not in [HOSTED, BLACKSMITH, BLACKSMITH_LIGHT, OCI, "macos-latest", "windows-latest"]:
                fails.append("UNKNOWN runner %r: %s" % (got, tag))
            trusted = ev in ("push", "same_repo_pr")
            default = ctx["matrix.os"] if kind == "matrix" else HOSTED
            scope = ctx.get("vars.CI_RUNNER_BLACKSMITH_JOBS") or "test"
            toks = scope.lower().split(",")  # exact tokens (GitHub contains() ignores case); no substring/space matching
            if kind == "light":
                want = (
                    BLACKSMITH_LIGHT
                    if ctx.get("vars.CI_RUNNER") == "blacksmith"
                    and "light" in toks
                    and (ev == "push" or (ev == "same_repo_pr" and "pr" in toks))
                    else HOSTED
                )
                if got != want:
                    fails.append("light job got %r, expected %r: %s" % (got, want, tag))
            elif got == BLACKSMITH_LIGHT:
                fails.append("non-light job moved to the 2vcpu runner: " + tag)
            if not trusted and got != default:
                fails.append("untrusted event got %r instead of %r: %s" % (got, default, tag))
            if kind == "matrix" and ctx["matrix.os"] != "ubuntu-latest" and got != ctx["matrix.os"]:
                fails.append("non-ubuntu matrix leg moved to %r: %s" % (got, tag))
            r = ctx.get("vars.CI_RUNNER")
            if r is None and got != legacy(kind, ctx):
                fails.append("CI_RUNNER unset differs from legacy (%r vs %r): %s" % (got, legacy(kind, ctx), tag))
            if r == "hosted" and got != default:
                fails.append("CI_RUNNER=hosted not default: " + tag)
            if kind == "light" and got not in (HOSTED, BLACKSMITH_LIGHT):
                fails.append("light job on unexpected runner %r: %s" % (got, tag))
            if got in (BLACKSMITH, BLACKSMITH_LIGHT) and r != "blacksmith":
                fails.append("blacksmith without CI_RUNNER=blacksmith: " + tag)
            if got == OCI and r not in ("oci", None):
                fails.append("oci runner selected under CI_RUNNER=%s: %s" % (r, tag))
    return fails, n


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--base-ref", help="also assert only runs-on differs from this git ref")
    args = ap.parse_args()
    fails, n = check(args.base_ref)
    if fails:
        for f in fails[:40]:
            print("[FAIL] " + f)
        print("[FAIL] %d violation(s) over %d evaluated combinations" % (len(fails), n))
        return 1
    print(
        "[OK] CI_RUNNER selector: %d combinations over %d jobs; no runs-on is empty, "
        "untrusted events stay hosted, unset == legacy, job bodies intact" % (n, len(JOBS))
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
