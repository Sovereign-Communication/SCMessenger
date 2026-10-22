#!/usr/bin/env python3
"""Bucketed JEV completion gate for SCMessenger WP tickets (orchestrator tool).

Usage:
  python scripts/jev_canonical_check.py --wp WP1 --state-file path/to/state.json
  python scripts/jev_canonical_check.py --wp WP7 --state-file s.json \
      --changed-paths-from origin/main

The default source is the pinned admission in scripts/harness_admission.json.
Set HARNESS_REPO only for an explicitly unpinned canary experiment.

State JSON should include: wp, instruction, files, acceptance, evidence
(commands/outputs), canon rows claimed. Optional: `diff` (unified diff text) for
content-aware bucket selection when `--changed-paths-from` is not used.

Bucket selection (content-aware): protected buckets, infra and dead_code are
path/deletion-selected. concurrency and lifecycle need an ADDED diff line matching
their markers in a matching file (path-only when no diff is known). testplan
needs a real test source path or a production code path (which then asks for a
test). Bare substring globs such as `**/*test*` are not used.

Design (schema 1.1.0). The gate is *bucketed*: each changed path is mapped to
the audit dimensions it touches (docs/jev-completion/PRESCRIPTIONS.md) and only
the questions of those buckets are asked, in ONE evaluate call with ids
``<bucket>.<question>``. Bucket questions are the Harness `choice` primitive
with the three outcomes yes / no / na. `na` means "this question does not apply
to the change" and is EXCLUDED from the score; it is never counted as
unsupported. (A `noul` question cannot express N/A, which is why the earlier
fixed noul pack failed every change unrelated to identity/routing.)

Gate rules:
  * `instruction` is always asked (noul, thresholded by --min-confidence).
  * any `no` in a protected bucket (identity, routing, security_*,
    ffi_boundary) fails the gate outright.
  * every applicable bucket must score >= --bucket-threshold (default 0.80).
  * every `yes` must cite specific evidence. Each state["evidence"] entry has
    an id (its own "id" field, else E1, E2, ... by position) and the state must
    carry state["evidence_map"]["<bucket>.<question>"] = [evidence ids]. The
    Harness choice/noul answers carry no rationale or citation field, so the
    citation lives in this structured map; a `yes` whose map entry is missing,
    empty, or names an id that does not exist is downgraded to `no`.
  * a protected bucket selected by changed paths may NOT drop out via `na` on
    its primary question (PROTECTED_PRIMARY). That `na` counts as `no` (hard
    fail) unless state["na_justifications"]["<bucket>.<question>"] holds a
    non-empty reason AND --allow-protected-na is passed; honoured overrides are
    listed in the result as `na_overrides` and printed as [WARNING].
  * an empty applicable set passes on the instruction bucket alone.

Exit 0 only if the gate passes on a live keyed answer. `--allow-fallback`
prints UNVERIFIED-JEV and is **not** canonical DONE (mechanical gates still
required; freebuff/orchestrator must not mark WP DONE).

This script does not replace mechanical gates (rules_check, tests, pr_scope).
It implements HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md section 3.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Set

SCHEMA_VERSION = "1.1.0"
DEFAULT_BUCKET_THRESHOLD = 0.80
INSTRUCTION_BUCKET = "instruction"

# Primary (canon) question of each protected bucket: `na` here is not allowed
# when the bucket was selected by changed paths (see score_gate).
PROTECTED_PRIMARY = {
    "identity": ["canon_identity"],
    "routing": ["canon_routing_feed"],
    "security_input": ["validates_input"],
    "security_crypto": ["safe_crypto"],
    "ffi_boundary": ["ffi_contract"],
}

_NA_TEXT = "The question does not apply to this change (excluded from the score)."


def _ynq(question: str, yes: str, no: str) -> Dict[str, Any]:
    """Typed yes/no/na question (Harness `choice` primitive)."""
    return {
        "type": "choice",
        "instructions": question
        + " Answer yes only if specific evidence supports it, and cite it: the state's evidence_map"
        " must list the evidence ids (E1, E2, ...) for this question. Uncited yes counts as no."
        " Answer na only if the question does not apply to this change.",
        "criteria": {"yes": yes, "no": no, "na": _NA_TEXT},
    }


# Kept verbatim from the original three-question pack (types adapted to choice).
CANON_QUESTIONS = {
    "canon_identity": {
        "type": "noul",
        "instructions": (
            "Does the change keep ONE contact identity flavor (public-key hex) "
            "as the addressing key, without introducing a second contact-address flavor?"
        ),
        "criteria": {
            "true": "Hex remains the contact/store addressing key; peer id is only derived for libp2p.",
            "false": "A second contact-address flavor is introduced or peers are addressed primarily by base58 in storage/UI.",
        },
    },
    "canon_routing_feed": {
        "type": "noul",
        "instructions": (
            "Does the change keep ONE routing feed entry point "
            "(IronCore::routing_peer_seen) for all data-link transports, "
            "without a parallel bespoke feed?"
        ),
        "criteria": {
            "true": "All data-link establishes call the same routing_peer_seen entry (or WP is unrelated to routing).",
            "false": "A second routing-presence API is added or a transport is connected without that entry.",
        },
    },
    "instruction_matches": {
        "type": "noul",
        "instructions": "Does the implementation and evidence satisfy the WP instruction and acceptance rows?",
        "criteria": {
            "true": "Acceptance rows are met with command/test evidence cited.",
            "false": "Acceptance rows are unmet, contradicted, or only asserted without evidence.",
        },
    },
}


def _canon_as_ynq(qid: str) -> Dict[str, Any]:
    q = CANON_QUESTIONS[qid]
    return _ynq(q["instructions"], q["criteria"]["true"], q["criteria"]["false"])


_SECURITY_BATTERY = {
    "validates_input": _ynq(
        "Is all untrusted input (network, FFI, BLE/proximity, files) validated before use?",
        "Inputs are validated or length/shape-checked before reaching logic.",
        "Untrusted input is used without validation.",
    ),
    "secrets_safe": _ynq(
        "Are secrets and key material kept out of logs, errors, and non-zeroized storage?",
        "No secret/key material is logged, leaked in errors, or persisted unprotected.",
        "Secret or key material is exposed.",
    ),
    "safe_crypto": _ynq(
        "Does crypto use only the approved primitives (Ed25519 sign, X25519 encrypt, XChaCha20-Poly1305 seal) correctly?",
        "Approved primitives, fresh nonces, no custom constructions.",
        "Unapproved primitive, nonce reuse, or hand-rolled crypto.",
    ),
    "failure_handled": _ynq(
        "Does failure fail closed (error returned, never silently allowed)?",
        "Errors deny/propagate; no fail-open path.",
        "A failure path allows the operation to proceed.",
    ),
}

# Content gates for path-broad buckets. Matched against ADDED diff lines of the
# bucket's own matching files only (removed lines and untouched code do not count).
CONCURRENCY_MARKERS = [
    r"RwLock", r"Mutex", r"Arc<", r"\.lock\(\)", r"\.read\(\)", r"\.write\(\)",
    r"tokio::spawn", r"\basync\s+fn\b", r"\.await\b", r"[Cc]hannel", r"\bAtomic[A-Z]\w*",
    r"\bsynchronized\b", r"\blaunch\s*[{(]", r"\bwithContext\s*\(", r"\bactor\s+\w+",
    r"DispatchQueue",
]
LIFECYCLE_MARKERS = [
    r"\b(?:open|close|connect|disconnect|register|unregister|bind|unbind|release|"
    r"dispose|shutdown|start|stop)\w*\s*\(",
    r"\bfinally\b", r"\bonDestroy\b", r"\bdeinit\b", r"\.use\s*[{(]", r"\busing\s*\(",
]
# Source files whose change means production behaviour changed (testplan asks for a test).
PRODUCTION_CODE_EXTS = (".rs", ".kt", ".swift", ".java", ".py", ".udl", ".ts", ".js")

# Buckets derive from the audit dimensions (PRESCRIPTIONS.md battery headlines:
# input not validated, fail-open, hard-to-test, concurrency, context need).
BUCKETS: Dict[str, Dict[str, Any]] = {
    "identity": {
        "path_globs": [
            "core/src/identity/**",
            "core/src/store/contacts.rs",
            "core/src/contacts_bridge.rs",
            "**/PeerKeyUtils.kt",
            "**/PeerIdValidator.kt",
        ],
        "questions": {
            "canon_identity": _canon_as_ynq("canon_identity"),
            "recipient_parse_safe": _ynq(
                "Are recipients/keys parsed from untrusted input free of unwrap/expect, with non-key recipients rejected?",
                "No unwrap/expect on recipients or keys from untrusted input; non-key recipients are rejected.",
                "A recipient/key parse can panic or accepts a non-key recipient.",
            ),
        },
        "weight": 2.0,
        "protected": True,
    },
    "routing": {
        "path_globs": [
            "core/src/routing/**",
            "core/src/transport/**",
            "core/src/mobile_bridge.rs",
            "core/src/iron_core.rs",
        ],
        "questions": {
            "canon_routing_feed": _canon_as_ynq("canon_routing_feed"),
            "callbacks_validated": _ynq(
                "Is data entering via transport callbacks (on_*_received, process_gossip) validated before reaching routing_peer_seen / routing state?",
                "Callback data is validated before it reaches routing state.",
                "Callback data reaches routing state unvalidated.",
            ),
        },
        "weight": 2.0,
        "protected": True,
    },
    "security_input": {
        "path_globs": [
            "core/src/mobile_bridge.rs",
            "core/src/transport/**",
            "core/src/routing/**",
            "cli/src/**/*.rs",
            "wasm/**",
            "**/BleGattServer.kt",
            "**/*Transport*.swift",
        ],
        "questions": {
            "validates_input": _SECURITY_BATTERY["validates_input"],
            "failure_handled": _SECURITY_BATTERY["failure_handled"],
        },
        "weight": 2.0,
        "protected": True,
    },
    "security_crypto": {
        "path_globs": ["core/src/crypto/**", "core/src/privacy/**"],
        "questions": {
            "safe_crypto": _SECURITY_BATTERY["safe_crypto"],
            "secrets_safe": _SECURITY_BATTERY["secrets_safe"],
            "failure_handled": _SECURITY_BATTERY["failure_handled"],
        },
        "weight": 2.0,
        "protected": True,
    },
    "ffi_boundary": {
        "path_globs": [
            "core/src/mobile_bridge.rs",
            "core/src/api.udl",
            "wasm/**",
            "cli/src/api*.rs",
        ],
        "questions": {
            "ffi_contract": _ynq(
                "Do FFI/UDL/wasm_bindgen boundary changes stay consistent on both sides (signatures, error mapping, no panics across the boundary)?",
                "Both sides of the boundary agree and errors map without panics.",
                "Boundary mismatch, or a panic can cross the FFI edge.",
            ),
        },
        "weight": 1.5,
        "protected": True,
    },
    "concurrency": {
        "path_globs": ["**/*.rs", "**/*.kt", "**/*.swift"],
        "questions": {
            "shared_state_safe": _ynq(
                "Is shared state behind Arc<parking_lot::RwLock>, with no lock held across await and cancellation handled?",
                "Locking discipline holds; no lock across await; cancellation handled.",
                "Lock held across await, unguarded shared state, or unhandled cancellation.",
            ),
        },
        "weight": 1.0,
        "protected": False,
        # Selected only when an added line in a matching file uses shared-state/async constructs.
        "content_markers": CONCURRENCY_MARKERS,
    },
    "lifecycle": {
        "path_globs": ["android/**", "iOS/**", "**/Platform*.kt", "**/Platform*.swift"],
        "questions": {
            "resources_closed": _ynq(
                "Are resources (sockets, GATT, file handles, scopes) closed on every exit path?",
                "Every exit path releases the resources it acquired.",
                "A resource leaks on some exit path.",
            ),
        },
        "weight": 1.0,
        "protected": False,
        # Selected only when an added line opens/closes/registers a resource or lifecycle hook.
        "content_markers": LIFECYCLE_MARKERS,
    },
    "testplan": {
        # Real test sources only (a bare "test" substring in a name does not count).
        "path_globs": [
            "**/tests/**", "**/src/test/**", "**/src/androidTest/**",
            "**/*_test.rs", "**/test_*.py", "**/*Test.kt", "**/*Tests.swift",
        ],
        "questions": {
            "named_test": _ynq(
                "Is the changed behavior covered by a named test?",
                "A named test exercises the changed behavior.",
                "Changed behavior has no covering test.",
            ),
        },
        "weight": 1.0,
        "protected": False,
    },
    "infra": {
        "path_globs": [".github/workflows/**", "docker/**", "**/Dockerfile*"],
        "questions": {
            "bounded_behaviour": _ynq(
                "Does the CI/container change keep behaviour bounded (explicit timeouts, capped retries, no unbounded loops or waits)?",
                "Timeouts and retry counts are bounded; no unbounded loop or wait is introduced.",
                "A step can run or retry without bound, or has no timeout.",
            ),
            "checks_not_weakened": _ynq(
                "Does the change leave every existing CI check, gate and test step failing on error (none skipped, disabled, continue-on-error or made advisory)?",
                "Every existing check still fails the build on error.",
                "A check is skipped, disabled, made advisory, or its failure is ignored.",
            ),
            "ci_run_evidence": _ynq(
                "Is the changed path exercised by a CI run whose result is cited as evidence?",
                "A cited CI run executed the changed workflow, image or Dockerfile.",
                "No CI run exercising the changed path is cited.",
            ),
        },
        "weight": 1.0,
        "protected": False,
    },
    "dead_code": {
        "path_globs": [],  # selected from deletions in the diff, not from paths
        "questions": {
            "no_live_callers": _ynq(
                "Do removed symbols have no remaining callers, including FFI/UDL/wasm_bindgen exports?",
                "No remaining caller or export references the removed symbols.",
                "A removed symbol still has a caller or export.",
            ),
        },
        "weight": 1.0,
        "protected": False,
    },
    INSTRUCTION_BUCKET: {
        "path_globs": [],  # always selected
        "questions": {
            "instruction_matches": CANON_QUESTIONS["instruction_matches"],
        },
        "weight": 2.0,
        "protected": False,
    },
}


# --------------------------------------------------------------------------- selection


def _glob_to_regex(glob: str) -> "re.Pattern[str]":
    """`**` crosses directories (and may match none); `*` does not cross `/`."""
    i, out = 0, ""
    while i < len(glob):
        c = glob[i]
        if glob[i : i + 3] == "**/":
            out += "(?:.*/)?"
            i += 3
        elif glob[i : i + 2] == "**":
            out += ".*"
            i += 2
        elif c == "*":
            out += "[^/]*"
            i += 1
        elif c == "?":
            out += "[^/]"
            i += 1
        else:
            out += re.escape(c)
            i += 1
    return re.compile("^" + out + "$")


def path_matches(path: str, globs: Iterable[str]) -> bool:
    norm = path.replace("\\", "/")
    while norm.startswith("./"):
        norm = norm[2:]
    return any(_glob_to_regex(g).match(norm) for g in globs)


def _is_test_path(path: str) -> bool:
    return path_matches(path, BUCKETS["testplan"]["path_globs"])


def _is_production_code(path: str) -> bool:
    return path.lower().endswith(PRODUCTION_CODE_EXTS) and not _is_test_path(path)


def git_diff_info(base_ref: str, repo_root: Path) -> Dict[str, Any]:
    """Changed paths, deleted paths, removed symbol names and added text."""

    def _git(*args: str) -> str:
        proc = subprocess.run(
            ["git", *args], cwd=repo_root, capture_output=True, text=True, encoding="utf-8", errors="replace"
        )
        if proc.returncode != 0:
            raise RuntimeError(f"git {' '.join(args)} failed: {proc.stderr.strip()[:300]}")
        return proc.stdout

    rng = base_ref if ".." in base_ref else f"{base_ref}...HEAD"
    paths = [p for p in _git("diff", "--name-only", rng).splitlines() if p.strip()]
    deleted = [p for p in _git("diff", "--name-only", "--diff-filter=D", rng).splitlines() if p.strip()]
    return {"paths": paths, "deleted": deleted, **parse_unified_diff(_git("diff", "-U0", rng))}


_SYMBOL_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?"
    r"(?:fn|def|fun|func|class|struct|enum|trait|interface)\s+([A-Za-z_][A-Za-z0-9_]*)"
)


def parse_unified_diff(diff: str) -> Dict[str, Any]:
    """Split a unified diff into removed symbols, added text (global and per file).

    File attribution comes from `+++ b/<path>` headers. A header is recognised
    only when the next line is a hunk header (`@@`), so content lines that merely
    start with `---`/`+++` are not mistaken for one. Text before any header is
    keyed by "" (applies to every path). Deleted files (`+++ /dev/null`) add nothing.
    """
    removed: Set[str] = set()
    added: List[str] = []
    by_path: Dict[str, List[str]] = {}
    diff_paths: Set[str] = set()
    current = ""
    lines = diff.splitlines()
    for i, line in enumerate(lines):
        nxt = lines[i + 1] if i + 1 < len(lines) else ""
        if line.startswith("diff --git "):
            current = ""
            continue
        if line.startswith("+++ ") and nxt.startswith("@@"):
            target = line[4:].strip()
            if target == "/dev/null":
                current = ""
            else:
                current = (target[2:] if target.startswith("b/") else target).replace("\\", "/")
                diff_paths.add(current)
            continue
        if line.startswith("--- ") and nxt.startswith("+++ "):
            continue
        if line.startswith("-"):
            m = _SYMBOL_RE.match(line[1:])
            if m:
                removed.add(m.group(1))
        elif line.startswith("+"):
            added.append(line[1:])
            by_path.setdefault(current, []).append(line[1:])
    return {
        "removed_symbols": sorted(removed),
        "added_text": "\n".join(added),
        "added_by_path": {k: "\n".join(v) for k, v in by_path.items()},
        "diff_paths": sorted(diff_paths),
    }


def _added_for(path: str, added_by_path: Optional[Dict[str, str]], added_text: Optional[str]) -> Optional[str]:
    """Added diff text that can be attributed to `path`; None when no diff is known."""
    if added_by_path is not None:
        return "\n".join(t for t in (added_by_path.get(path, ""), added_by_path.get("", "")) if t)
    return added_text


def select_buckets(
    paths: Iterable[str],
    *,
    deleted: Iterable[str] = (),
    removed_symbols: Iterable[str] = (),
    added_text: Optional[str] = None,
    added_by_path: Optional[Dict[str, str]] = None,
) -> List[str]:
    """Return selected bucket names (instruction always, in BUCKETS order).

    Path globs decide the protected buckets and infra. Content-gated buckets
    (concurrency, lifecycle) additionally need an ADDED diff line that matches
    their markers, checked per file when `added_by_path` is known (else against
    the global `added_text`). With no diff text at all, they fall back to paths.
    testplan needs a real test source path, or a production code path (which then
    asks for a test).
    """
    paths = [p.replace("\\", "/") for p in paths]
    selected: List[str] = []
    for name, spec in BUCKETS.items():
        if name == INSTRUCTION_BUCKET:
            selected.append(name)
            continue
        if name == "dead_code":
            if list(deleted) or list(removed_symbols):
                selected.append(name)
            continue
        if name == "testplan":
            hit = any(_is_test_path(p) or _is_production_code(p) for p in paths)
        else:
            hits = [p for p in paths if path_matches(p, spec["path_globs"])]
            if name == "concurrency":
                hits = [p for p in hits if not _is_test_path(p)]
            markers = spec.get("content_markers")
            if markers and hits:
                gated = []
                for p in hits:
                    text = _added_for(p, added_by_path, added_text)
                    if text is None or any(re.search(m, text) for m in markers):
                        gated.append(p)
                hits = gated
            hit = bool(hits)
        if hit:
            selected.append(name)
    return selected


def build_questions(selected: Iterable[str]) -> Dict[str, Dict[str, Any]]:
    """Merge selected buckets into one pack with ids `bucket.qid`."""
    merged: Dict[str, Dict[str, Any]] = {}
    for name in selected:
        for qid, q in BUCKETS[name]["questions"].items():
            merged[f"{name}.{qid}"] = q
    return merged


# --------------------------------------------------------------------------- scoring


def evidence_index(state: Dict[str, Any]) -> Dict[str, Any]:
    """Map evidence id -> entry. Entries may be strings or dicts with an "id"."""
    out: Dict[str, Any] = {}
    for i, entry in enumerate(state.get("evidence") or [], start=1):
        eid = str(entry["id"]) if isinstance(entry, dict) and entry.get("id") else f"E{i}"
        out[eid] = entry
    return out


def with_evidence_ids(state: Dict[str, Any]) -> Dict[str, Any]:
    """State copy whose evidence entries carry their ids, for the evaluator."""
    ev = []
    for eid, entry in evidence_index(state).items():
        ev.append(dict(entry, id=eid) if isinstance(entry, dict) else {"id": eid, "text": entry})
    return dict(state, evidence=ev)


def _citation_problem(qkey: str, evidence_ids: Set[str], evidence_map: Dict[str, Any]) -> Optional[str]:
    cited = evidence_map.get(qkey)
    if isinstance(cited, str):
        cited = [cited]
    if not isinstance(cited, list) or not cited:
        return "uncited: no evidence_map entry for this question"
    bad = [str(c) for c in cited if str(c) not in evidence_ids]
    if bad:
        return "uncited: evidence_map names unknown evidence id(s) " + ",".join(bad)
    return None


def _score_answer(answer: Dict[str, Any], citation_problem: Optional[str]) -> Dict[str, Any]:
    """Normalise one Harness answer to {verdict: yes|no|na, score, ...}."""
    if answer.get("type") == "choice":
        probs = answer.get("probabilities") or {}
        chosen = answer.get("choice")
        if chosen == "na":
            return {"verdict": "na", "score": None}
        p_yes, p_no = float(probs.get("yes", 0.0)), float(probs.get("no", 0.0))
        denom = p_yes + p_no
        score = (p_yes / denom) if denom > 0 else 0.0
        verdict = "no" if chosen == "no" else "yes"
        out = {"verdict": verdict, "score": score, "confidence": answer.get("confidence")}
    else:  # noul
        value = float(answer.get("noul", 0.0))
        out = {"verdict": "yes" if value >= 0.5 else "no", "score": value}
    if citation_problem and out["verdict"] == "yes":
        out.update(verdict="no", score=0.0, reason=citation_problem)
    return out


def score_gate(
    answers: Dict[str, Any],
    selected: List[str],
    *,
    evidence_ids: Iterable[str] = (),
    evidence_map: Optional[Dict[str, Any]] = None,
    na_justifications: Optional[Dict[str, Any]] = None,
    allow_protected_na: bool = False,
    path_selected: Optional[Iterable[str]] = None,
    min_confidence: float = 0.70,
    bucket_threshold: float = DEFAULT_BUCKET_THRESHOLD,
) -> Dict[str, Any]:
    """Pure scoring of a merged `bucket.qid` answer map.

    N/A is excluded from the score, except for the primary question of a
    protected bucket selected by path (`path_selected`; None = every selected
    bucket), where it is a hard fail unless justified AND allow_protected_na.
    """
    ids = set(evidence_ids)
    emap = evidence_map or {}
    justif = na_justifications or {}
    path_sel = set(selected) if path_selected is None else set(path_selected)
    na_overrides: List[Dict[str, str]] = []
    buckets: Dict[str, Any] = {}
    failures: List[str] = []
    weighted, weight_sum, applicable = 0.0, 0.0, 0
    for name in BUCKETS:
        spec = BUCKETS[name]
        if name not in selected:
            buckets[name] = {"score": None, "verdict": "na", "na_reason": "not_selected",
                             "protected": spec["protected"], "answers": {}}
            continue
        q_results: Dict[str, Any] = {}
        for qid in spec["questions"]:
            qkey = f"{name}.{qid}"
            ans = answers.get(qkey)
            q_results[qid] = (
                _score_answer(ans, _citation_problem(qkey, ids, emap))
                if isinstance(ans, dict)
                else {"verdict": "no", "score": 0.0, "reason": "missing answer"}
            )
            if (
                q_results[qid]["verdict"] == "na"
                and spec["protected"]
                and name in path_sel
                and qid in PROTECTED_PRIMARY.get(name, [])
            ):
                reason = justif.get(qkey)
                reason = reason.strip() if isinstance(reason, str) else ""
                if reason and allow_protected_na:
                    na_overrides.append({"question": qkey, "justification": reason})
                    q_results[qid]["na_override"] = reason
                else:
                    why = ("justification given but --allow-protected-na not set" if reason
                           else "no na_justifications entry")
                    q_results[qid] = {"verdict": "no", "score": 0.0,
                                      "reason": f"protected primary question answered na ({why}); treated as no"}
        live = {k: v for k, v in q_results.items() if v["verdict"] != "na"}
        if not live:
            buckets[name] = {"score": None, "verdict": "na", "na_reason": "model_na",
                             "protected": spec["protected"], "answers": q_results}
            continue
        applicable += len(live)
        score = min(v["score"] for v in live.values())
        threshold = min_confidence if name == INSTRUCTION_BUCKET else bucket_threshold
        has_no = any(v["verdict"] == "no" for v in live.values())
        if spec["protected"] and has_no:
            verdict = "hard_fail"
            failures.append(f"{name}: protected bucket answered no")
        elif score < threshold:
            verdict = "fail"
            failures.append(f"{name}: score {score:.2f} < {threshold:.2f}")
        else:
            verdict = "pass"
        buckets[name] = {"score": score, "verdict": verdict, "protected": spec["protected"],
                         "answers": q_results}
        weighted += score * spec["weight"]
        weight_sum += spec["weight"]
    overall = (weighted / weight_sum) if weight_sum else 0.0
    if overall < min_confidence and not failures:
        failures.append(f"overall {overall:.2f} < {min_confidence:.2f}")
    return {
        "passed": not failures,
        "failures": failures,
        "overall_score": overall,
        "applicable_count": applicable,
        "buckets_selected": list(selected),
        "buckets_na": [n for n, b in buckets.items() if b["verdict"] == "na"],
        "na_overrides": na_overrides,
        "buckets": buckets,
    }


# --------------------------------------------------------------------------- scoped evaluation
#
# PR #501 sent a 188 KB state (178,782-char diff) in ONE TypeSafe call; TypeSafe
# answered HTTP 400 max_tokens_exceeded, the pinned harness only hard-fails on
# 401/422, so the 400 silently became a local structural fallback. Large states
# are therefore split: one evaluate call per bucket, carrying only the hunks of
# the files that selected that bucket, and chunked by file when still too big.

# Per-call budget for the JSON-serialised state, in characters. Justification:
# the largest state TypeSafe is known to have judged live was 80,162 chars
# (jev482d, 24,093 input tokens); the smallest known failure was 188,197 chars.
# 48,000 sits at 60% of the largest proven-good size, so there is wide margin
# below the (unmeasured) true limit. States at or under it are sent in a single
# call exactly as before.
JEV_MAX_STATE_CHARS = 48_000
# Floor for the diff share of a call, so a very large evidence block cannot
# shrink chunks to nothing.
JEV_MIN_DIFF_CHARS = 8_000
SUMMARY_MAX_FILES = 300


class FileDiff:
    """One file's slice of a unified diff: shared header lines plus whole hunks."""

    def __init__(self, path: str, header: List[str], hunks: List[str], deleted: bool = False):
        self.path = path
        self.header = header
        self.hunks = hunks
        self.deleted = deleted

    def render(self, hunks: Optional[List[str]] = None) -> str:
        return "".join(self.header) + "".join(self.hunks if hunks is None else hunks)

    @property
    def stats(self) -> Dict[str, int]:
        add = rem = 0
        for h in self.hunks:
            for line in h.splitlines()[1:]:
                if line.startswith("+"):
                    add += 1
                elif line.startswith("-"):
                    rem += 1
        return {"added": add, "removed": rem, "hunks": len(self.hunks)}


_DIFF_GIT_RE = re.compile(r"^diff --git a/(.*) b/(.*)$")


def _strip_ab(target: str) -> str:
    return (target[2:] if target[:2] in ("a/", "b/") else target).replace("\\", "/")


def _file_path_from_header(header: List[str]) -> "tuple[str, bool]":
    old = new = ""
    for line in header:
        if line.startswith("--- "):
            old = line[4:].strip()
        elif line.startswith("+++ "):
            new = line[4:].strip()
    if new and new != "/dev/null":
        return _strip_ab(new), False
    if old and old != "/dev/null":
        return _strip_ab(old), new == "/dev/null"
    for line in header:
        m = _DIFF_GIT_RE.match(line.rstrip("\r\n"))
        if m:
            return m.group(2).replace("\\", "/"), False
    return "", False


def parse_file_diffs(diff: str) -> List[FileDiff]:
    """Split a unified diff into per-file headers and whole hunks.

    A hunk starts at an `@@` line and runs to the next `@@` or file boundary, so
    callers can regroup hunks without ever cutting one. Files start at
    `diff --git`; for plain `---`/`+++` diffs a file starts at a `--- ` line
    followed by `+++ ` while not inside a hunk.
    """
    lines = diff.splitlines(keepends=True)
    git_mode = any(l.startswith("diff --git ") for l in lines)
    blocks: List[List[str]] = []
    cur: Optional[List[str]] = None
    in_hunk = False
    for i, line in enumerate(lines):
        if git_mode:
            boundary = line.startswith("diff --git ")
        else:
            boundary = (not in_hunk and line.startswith("--- ")
                        and i + 1 < len(lines) and lines[i + 1].startswith("+++ "))
        if boundary:
            cur = [line]
            blocks.append(cur)
            in_hunk = False
            continue
        if cur is None:
            continue
        if line.startswith("@@"):
            in_hunk = True
        cur.append(line)
    out: List[FileDiff] = []
    for block in blocks:
        header: List[str] = []
        hunks: List[str] = []
        for line in block:
            if line.startswith("@@"):
                hunks.append(line)
            elif hunks:
                hunks[-1] += line
            else:
                header.append(line)
        path, deleted = _file_path_from_header(header)
        if path:
            out.append(FileDiff(path, header, hunks, deleted))
    return out


def chunk_file_diffs(files: List[FileDiff], budget: int) -> List[str]:
    """Pack file diffs into chunks of at most `budget` chars, never cutting a hunk.

    Whole files are packed in order. A file larger than the budget is split
    between hunks, each part repeating the file header. A single hunk larger
    than the budget becomes its own (over-budget) chunk: it is never truncated,
    because a half hunk would be judged as if it were the whole change.
    """
    chunks: List[str] = []
    current: List[str] = []
    size = 0

    def flush() -> None:
        nonlocal current, size
        if current:
            chunks.append("".join(current))
        current, size = [], 0

    for fd in files:
        whole = fd.render()
        if len(whole) <= budget:
            if size + len(whole) > budget:
                flush()
            current.append(whole)
            size += len(whole)
            continue
        flush()
        head = "".join(fd.header)
        part: List[str] = []
        psize = len(head)
        for h in fd.hunks:
            if part and psize + len(h) > budget:
                chunks.append(head + "".join(part))
                part, psize = [], len(head)
            part.append(h)
            psize += len(h)
        if part:
            chunks.append(head + "".join(part))
    flush()
    return chunks


def files_for_buckets(selected: Iterable[str], file_diffs: List[FileDiff],
                      state_deleted: Iterable[str] = ()) -> Dict[str, List[FileDiff]]:
    """Map each selected bucket to the file diffs that selected it."""
    sd = set(state_deleted)
    out: Dict[str, List[FileDiff]] = {b: [] for b in selected if b != INSTRUCTION_BUCKET}
    for fd in file_diffs:
        parsed = parse_unified_diff(fd.render())
        gone = fd.deleted or fd.path in sd
        hit = select_buckets(
            [fd.path],
            deleted=[fd.path] if gone else [],
            removed_symbols=parsed["removed_symbols"],
            added_by_path={fd.path: parsed["added_by_path"].get(fd.path, "")},
        )
        for b in hit:
            if b in out:
                out[b].append(fd)
    return out


def diff_summary(file_diffs: List[FileDiff]) -> str:
    """Compact change summary for the instruction bucket (no hunk bodies)."""
    tot = {"added": 0, "removed": 0, "hunks": 0}
    rows = []
    for fd in file_diffs:
        s = fd.stats
        for k in tot:
            tot[k] += s[k]
        rows.append(f"{fd.path} (+{s['added']} -{s['removed']}, {s['hunks']} hunks"
                    + (", deleted)" if fd.deleted else ")"))
    head = f"{len(file_diffs)} files changed, +{tot['added']} -{tot['removed']}, {tot['hunks']} hunks"
    if len(rows) > SUMMARY_MAX_FILES:
        rows = rows[:SUMMARY_MAX_FILES] + [f"... {len(file_diffs) - SUMMARY_MAX_FILES} more files"]
    return "\n".join([head, *rows])


class ScopedCall:
    def __init__(self, bucket: str, chunk: int, chunks: int, state: Dict[str, Any], questions: Dict[str, Any]):
        self.bucket, self.chunk, self.chunks = bucket, chunk, chunks
        self.state, self.questions = state, questions
        self.state_chars = len(json.dumps(state, ensure_ascii=False))


def plan_calls(state: Dict[str, Any], selected: List[str], info: Dict[str, Any],
               max_state_chars: int = JEV_MAX_STATE_CHARS) -> List[ScopedCall]:
    """One call when the state fits the budget, else one call per bucket (and chunk)."""
    whole = len(json.dumps(state, ensure_ascii=False))
    diff = state.get("diff")
    if whole <= max_state_chars or not (isinstance(diff, str) and diff.strip()):
        return [ScopedCall("*", 1, 1, state, build_questions(selected))]
    file_diffs = parse_file_diffs(diff)
    base = {k: v for k, v in state.items() if k != "diff"}
    budget = max(JEV_MIN_DIFF_CHARS, max_state_chars - len(json.dumps(base, ensure_ascii=False)))
    by_bucket = files_for_buckets(selected, file_diffs, info.get("deleted") or ())
    calls: List[ScopedCall] = []
    for name in selected:
        questions = build_questions([name])
        gate = dict(base.get("jev_gate") or {}, buckets_selected=[name], scoped_to_bucket=name)
        if name == INSTRUCTION_BUCKET:
            st = dict(base, jev_gate=gate, diff_summary=diff_summary(file_diffs))
            calls.append(ScopedCall(name, 1, 1, st, questions))
            continue
        chunks = chunk_file_diffs(by_bucket.get(name, []), budget) or [""]
        for i, text in enumerate(chunks, start=1):
            g = dict(gate, chunk=f"{i}/{len(chunks)}")
            st = dict(base, jev_gate=g, diff=text or "(no diff hunks attributable to this bucket)")
            calls.append(ScopedCall(name, i, len(chunks), st, questions))
    return calls


# ----------------------------------------------------------------- failure diagnostics


def describe_http_error(status: Any, resp: Any) -> Dict[str, Any]:
    """HTTP status plus error type/message from a TypeSafe/OpenRouter error body."""
    err = resp.get("error", resp) if isinstance(resp, dict) else resp
    etype = message = None
    if isinstance(err, dict):
        etype = err.get("type") or err.get("code") or err.get("error_type")
        message = err.get("message") or err.get("detail")
    elif isinstance(err, str):
        message = err
    if message is None and err is not None:
        message = json.dumps(err, ensure_ascii=False, default=str)
    return {"http": status, "error_type": etype, "message": (str(message)[:300] if message else None)}


class RecordingTransport:
    """Wraps the harness transport (without modifying it) to keep each reply.

    The pinned evaluator swallows every non-401/422 failure into a local
    fallback; this is the only place the real HTTP status survives.
    """

    def __init__(self, inner: Any):
        self.inner = inner
        self.records: List[Dict[str, Any]] = []

    def post(self, *args: Any, **kwargs: Any):
        try:
            status, resp = self.inner.post(*args, **kwargs)
        except Exception as exc:  # noqa: BLE001
            self.records.append({"http": None, "error_type": type(exc).__name__, "message": str(exc)[:300]})
            raise
        self.records.append(
            {"http": status, "error_type": None, "message": None} if status == 200
            else describe_http_error(status, resp)
        )
        return status, resp

    def __getattr__(self, name: str) -> Any:
        return getattr(self.inner, name)


def fallback_reason(result: Any, meta: Dict[str, Any], typesafe: Optional[Dict[str, Any]]) -> Optional[str]:
    """Human-readable cause of a fallback result; None when the result is live."""
    if not getattr(result, "is_fallback", False):
        return None
    parts = []
    if typesafe is None:
        parts.append("TypeSafe: no reply recorded (no key or transport failure)")
    else:
        parts.append(f"TypeSafe HTTP {typesafe.get('http')} error_type={typesafe.get('error_type')}"
                     + (f": {typesafe['message']}" if typesafe.get("message") else ""))
    if meta.get("openrouter_http") is not None:
        parts.append(f"OpenRouter HTTP {meta.get('openrouter_http')}")
    if meta.get("openrouter_error"):
        parts.append("OpenRouter error: " + json.dumps(meta["openrouter_error"], ensure_ascii=False, default=str)[:300])
    if meta.get("openrouter_parse_error"):
        parts.append("OpenRouter parse error: " + str(meta["openrouter_parse_error"])[:200])
    if meta.get("fallback_skipped"):
        parts.append("OpenRouter skipped: " + str(meta["fallback_skipped"]))
    parts.append("result is the local structural fallback")
    return "; ".join(parts)


# ----------------------------------------------------------------- aggregation


def _yes_ratio(ans: Dict[str, Any]) -> float:
    if ans.get("type") == "choice":
        probs = ans.get("probabilities") or {}
        y, n = float(probs.get("yes", 0.0)), float(probs.get("no", 0.0))
        return (y / (y + n)) if (y + n) > 0 else 0.0
    return float(ans.get("noul", 0.0))


def merge_chunk_answers(per_chunk: List[Dict[str, Any]]) -> Optional[Dict[str, Any]]:
    """Combine one question's answers over chunks: the worst applicable one wins.

    `na` chunks (question does not apply to those files) are ignored unless
    every chunk is `na`. A chunk answering `no` therefore always dominates.
    """
    live = [a for a in per_chunk if not (a.get("type") == "choice" and a.get("choice") == "na")]
    if not live:
        return per_chunk[0] if per_chunk else None
    return min(live, key=lambda a: (a.get("choice") != "no", _yes_ratio(a)))


class AggregateResult:
    """Harness-result-shaped union of per-call results (all calls must pass)."""

    def __init__(self, results: List[Any], answers: Dict[str, Any]):
        live = [r for r in results if not r.is_fallback]
        self.answers = answers
        self.is_fallback = any(r.is_fallback for r in results)
        self.verdict = "pass" if results and all(r.verdict == "pass" for r in results) else "fail"
        self.supported = min((r.supported for r in results), default=0.0)
        self.confidence = min((r.confidence for r in live), default=0.0)
        self.cost = sum(float(r.cost or 0.0) for r in results)
        self.input_tokens = sum(int(r.input_tokens or 0) for r in results)
        self.output_tokens = sum(int(getattr(r, "output_tokens", 0) or 0) for r in results)
        models = sorted({str(r.model) for r in results})
        self.model = models[0] if len(models) == 1 else ",".join(models)
        self.reasons = [x for r in results for x in (r.reasons or [])]


def run_calls(calls: List[ScopedCall], call_fn: Any) -> "tuple[Any, Dict[str, Any], List[Dict[str, Any]]]":
    """Run every planned call; return (result, aggregate meta, call records).

    `call_fn(state, questions) -> (result, meta, typesafe_record_or_None)`.
    With one call the harness result is returned untouched (verdict semantics
    unchanged). With several, answers of a fallback call are dropped (their keys
    are not question ids), so those questions score as missing: fail closed.
    """
    results: List[Any] = []
    records: List[Dict[str, Any]] = []
    metas: List[Dict[str, Any]] = []
    per_q: Dict[str, List[Dict[str, Any]]] = {}
    for call in calls:
        result, meta, typesafe = call_fn(call.state, call.questions)
        results.append(result)
        metas.append(meta)
        records.append({
            "bucket": call.bucket, "chunk": f"{call.chunk}/{call.chunks}",
            "state_chars": call.state_chars, "endpoint": meta.get("endpoint"),
            "is_fallback": bool(result.is_fallback),
            "fallback_reason": fallback_reason(result, meta, typesafe),
            "typesafe_http": (typesafe or {}).get("http"),
            "typesafe_error_type": (typesafe or {}).get("error_type"),
            "input_tokens": result.input_tokens,
        })
        if not result.is_fallback:
            for qid, ans in (result.answers or {}).items():
                if isinstance(ans, dict):
                    per_q.setdefault(qid, []).append(ans)
    endpoints = sorted({str(m.get("endpoint")) for m in metas})
    meta_out = {
        "endpoint": "+".join(endpoints),
        "fallback_used": any(m.get("fallback_used") for m in metas),
        "openrouter_model": next((m["openrouter_model"] for m in metas if m.get("openrouter_model")), None),
    }
    if len(results) == 1:
        return results[0], meta_out, records
    merged = {q: m for q, lst in per_q.items() if (m := merge_chunk_answers(lst)) is not None}
    return AggregateResult(results, merged), meta_out, records


# --------------------------------------------------------------------------- CLI


def load_harness():
    """Import JEV from SCMessenger-local harness (vendor/sovereign-harness)."""
    try:
        from local_harness import import_harness  # type: ignore
    except ImportError:
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        from local_harness import import_harness  # type: ignore
    mod = import_harness()
    return (
        mod["JevEvaluator"],
        None,
        mod["root"],
        mod["key"],
    )


def resolve_paths(args: argparse.Namespace, state: Dict[str, Any]) -> Dict[str, Any]:
    """Merge state `files`, git diff and an optional state `diff` (unified diff text).

    `added_by_path` stays None when no diff is known at all; select_buckets then
    falls back to path-only selection for the content-gated buckets.
    """
    info: Dict[str, Any] = {"paths": [], "deleted": [], "removed_symbols": [], "added_text": None,
                            "added_by_path": None}
    if args.changed_paths_from:
        info.update(git_diff_info(args.changed_paths_from, Path(args.repo_root).resolve()))
    state_diff = state.get("diff")
    if isinstance(state_diff, str) and state_diff.strip():
        parsed = parse_unified_diff(state_diff)
        by_path = dict(info["added_by_path"] or {})
        for p, text in parsed["added_by_path"].items():
            by_path[p] = "\n".join(t for t in (by_path.get(p, ""), text) if t)
        info["added_by_path"] = by_path
        info["added_text"] = "\n".join(t for t in (info["added_text"], parsed["added_text"]) if t)
        info["removed_symbols"] = sorted(set(info["removed_symbols"]) | set(parsed["removed_symbols"]))
        info["paths"] = sorted(set(info["paths"]) | set(parsed["diff_paths"]))
    state_files = [f if isinstance(f, str) else f.get("path", "") for f in state.get("files", []) or []]
    info["paths"] = sorted(set(info["paths"]) | {f for f in state_files if f})
    info["deleted"] = sorted(set(info["deleted"]) | set(state.get("deleted_files", []) or []))
    info["removed_symbols"] = sorted(set(info["removed_symbols"]) | set(state.get("removed_symbols", []) or []))
    return info


def main(argv: Optional[List[str]] = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--wp", required=True)
    ap.add_argument("--state-file", required=True)
    ap.add_argument("--min-confidence", type=float, default=0.70)
    ap.add_argument("--bucket-threshold", type=float, default=DEFAULT_BUCKET_THRESHOLD,
                    help="Minimum score for every applicable non-instruction bucket (default 0.80)")
    ap.add_argument("--changed-paths-from", metavar="BASE_REF",
                    help="Select buckets from `git diff` against BASE_REF (merged with state files)")
    ap.add_argument("--repo-root", default=".", help="Repo for --changed-paths-from (default: cwd)")
    ap.add_argument(
        "--allow-protected-na",
        action="store_true",
        help="Honour state['na_justifications'] for `na` on a path-selected protected bucket's "
             "primary question (default: such na is treated as no)",
    )
    ap.add_argument(
        "--allow-fallback",
        action="store_true",
        help="Exit 0 on structural fallback only if explicitly allowed (still prints UNVERIFIED-JEV).",
    )
    ap.add_argument(
        "--no-openrouter",
        action="store_true",
        help="Disable OpenRouter ~typesafe/jev-latest fallback",
    )
    ap.add_argument(
        "--max-state-chars", type=int, default=JEV_MAX_STATE_CHARS,
        help="Largest JSON state sent in one evaluate call; above it the diff is scoped per bucket "
             f"and chunked by file (default {JEV_MAX_STATE_CHARS})",
    )
    ap.add_argument(
        "--result-file",
        help="Write structured JEV result JSON for a controller-owned evidence gate",
    )
    args = ap.parse_args(argv)

    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from local_harness import (  # type: ignore
        evaluate_jev_with_openrouter_fallback,
        import_harness,
        make_policy,
    )

    state = json.loads(Path(args.state_file).read_text(encoding="utf-8"))
    state.setdefault("wp", args.wp)

    info = resolve_paths(args, state)
    path_selected: Optional[Set[str]] = None
    if info["paths"] or info["deleted"]:
        selected = select_buckets(
            info["paths"],
            deleted=info["deleted"],
            removed_symbols=info["removed_symbols"],
            added_text=info["added_text"],
            added_by_path=info["added_by_path"],
        )
        diff_basis = "content-aware" if info["added_by_path"] is not None or info["added_text"] is not None else "path-only"
        print(f"[INFO] bucket selection basis={diff_basis}")
    else:
        # Legacy invocation with no file list: the original canon pair + instruction.
        selected = ["identity", "routing", INSTRUCTION_BUCKET]
        path_selected = set()  # not selected by path: legitimate na stays excluded
    if path_selected is None:
        path_selected = set(selected)
    questions = build_questions(selected)
    ev_idx = evidence_index(state)
    evidence_map = state.get("evidence_map") or {}
    na_justifications = state.get("na_justifications") or {}
    state = dict(with_evidence_ids(state), jev_gate={"buckets_selected": selected, "changed_paths": info["paths"][:200]})

    mod = import_harness()
    source = mod["source"]
    print(
        f"[INFO] harness: {source.root} sha={source.sha} "
        f"status={source.status} pinned={source.pinned} "
        f"typesafe_key={bool(mod['key'])} "
        f"openrouter_key={bool(mod.get('openrouter_key'))}"
    )
    _policy, _ = make_policy()
    evaluator = _policy.evaluator
    recorder: Optional[RecordingTransport] = None
    if hasattr(evaluator, "transport"):
        recorder = RecordingTransport(evaluator.transport)
        evaluator.transport = recorder

    def call_fn(call_state: Dict[str, Any], call_questions: Dict[str, Any]):
        if recorder is not None:
            recorder.records.clear()
        if args.no_openrouter:
            res = evaluator.evaluate(call_state, questions=call_questions)
            m = {"endpoint": "typesafe", "fallback_used": False}
        else:
            res, m = evaluate_jev_with_openrouter_fallback(evaluator, call_state, call_questions)
        return res, m, (recorder.records[-1] if recorder is not None and recorder.records else None)

    calls = plan_calls(state, selected, info, args.max_state_chars)
    scoped = not (len(calls) == 1 and calls[0].bucket == "*")
    print(f"[INFO] buckets_selected={selected} questions={len(questions)} calls={len(calls)} scoped={scoped}")
    if scoped:
        for c in calls:
            print(f"[INFO] call bucket={c.bucket} chunk={c.chunk}/{c.chunks} state_chars={c.state_chars}")
    result, meta, call_records = run_calls(calls, call_fn)
    for rec in call_records:
        if rec["fallback_reason"]:
            print(f"[WARNING] fallback bucket={rec['bucket']} chunk={rec['chunk']}: {rec['fallback_reason']}")
    fb_reasons = sorted({r["fallback_reason"] for r in call_records if r["fallback_reason"]})
    print(f"[INFO] endpoint={meta.get('endpoint')} fallback_used={meta.get('fallback_used')}")
    if meta.get("openrouter_model"):
        print(f"[INFO] openrouter_model={meta['openrouter_model']}")
    print(f"[INFO] verdict={result.verdict} supported={result.supported} confidence={result.confidence}")
    print(f"[INFO] is_fallback={result.is_fallback} cost={result.cost} tokens_in={result.input_tokens} model={result.model}")
    print(f"[INFO] answers={json.dumps(result.answers, ensure_ascii=False)}")
    for reason in result.reasons:
        print(f"[INFO] reason: {reason}")

    # The Harness verdict/is_passing would treat an `na` choice like any other
    # low-confidence answer, so the pass decision is computed here, with N/A
    # excluded; the raw Harness fields are still reported for audit.
    gate = score_gate(
        result.answers or {},
        selected,
        evidence_ids=ev_idx.keys(),
        evidence_map=evidence_map,
        na_justifications=na_justifications,
        allow_protected_na=args.allow_protected_na,
        path_selected=path_selected,
        min_confidence=args.min_confidence,
        bucket_threshold=args.bucket_threshold,
    )
    for name in gate["buckets_selected"]:
        b = gate["buckets"][name]
        score = "n/a" if b["score"] is None else f"{b['score']:.2f}"
        print(f"[INFO] bucket {name}: verdict={b['verdict']} score={score}")
    for ov in gate["na_overrides"]:
        print(f"[WARNING] PROTECTED-NA OVERRIDE {ov['question']}: {ov['justification']}")
    for failure in gate["failures"]:
        print(f"[INFO] gate failure: {failure}")
    canonical_pass = bool(gate["passed"]) and not result.is_fallback

    result_payload = {
        "schema_version": SCHEMA_VERSION,
        "wp": args.wp,
        "is_passing": bool(canonical_pass),
        "is_fallback": bool(result.is_fallback),
        "fallback_used": bool(meta.get("fallback_used")),
        "keyed": bool(mod["key"]),
        "scoped": scoped,
        "max_state_chars": args.max_state_chars,
        "fallback_reason": "; ".join(fb_reasons) if fb_reasons else None,
        "calls": call_records,
        "endpoint": meta.get("endpoint"),
        "model": result.model,
        "confidence": result.confidence,
        "supported": result.supported,
        "answers": result.answers,
        "reasons": result.reasons,
        "cost": result.cost,
        "input_tokens": result.input_tokens,
        "output_tokens": getattr(result, "output_tokens", 0),
        "buckets_selected": gate["buckets_selected"],
        "buckets_na": gate["buckets_na"],
        "na_overrides": gate["na_overrides"],
        "buckets": {
            n: {"score": b["score"], "verdict": b["verdict"], "answers": b["answers"]}
            for n, b in gate["buckets"].items()
        },
        "applicable_count": gate["applicable_count"],
        "overall_score": gate["overall_score"],
        "gate_failures": gate["failures"],
        "min_confidence": args.min_confidence,
        "bucket_threshold": args.bucket_threshold,
    }
    if args.result_file:
        result_path = Path(args.result_file)
        result_path.parent.mkdir(parents=True, exist_ok=True)
        result_path.write_text(
            json.dumps(result_payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    # Canonical DONE requires a live keyed answer (TypeSafe or OpenRouter),
    # never pure structural fallback / UNVERIFIED-JEV.
    if result.is_fallback:
        print("[FAIL] UNVERIFIED-JEV - fallback result is not canonical DONE"
              + (f" (reason: {'; '.join(fb_reasons)})" if fb_reasons else ""))
        return 0 if args.allow_fallback else 1
    if canonical_pass:
        print(f"[OK] JEV bucketed pass (min_confidence={args.min_confidence}, "
              f"bucket_threshold={args.bucket_threshold}) via {meta.get('endpoint')}")
        return 0
    print("[FAIL] JEV canonical check did not pass")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
