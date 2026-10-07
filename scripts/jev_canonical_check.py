#!/usr/bin/env python3
"""Bucketed JEV completion gate for SCMessenger WP tickets (orchestrator tool).

Usage:
  python scripts/jev_canonical_check.py --wp WP1 --state-file path/to/state.json
  python scripts/jev_canonical_check.py --wp WP7 --state-file s.json \
      --changed-paths-from origin/main

The default source is the pinned admission in scripts/harness_admission.json.
Set HARNESS_REPO only for an explicitly unpinned canary experiment.

State JSON should include: wp, instruction, files, acceptance, evidence
(commands/outputs), canon rows claimed.

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
  * answers must be backed by state["evidence"]; with no evidence every
    applicable `yes` is forced to `no` (uncited claims are not support).
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

_NA_TEXT = "The question does not apply to this change (excluded from the score)."


def _ynq(question: str, yes: str, no: str) -> Dict[str, Any]:
    """Typed yes/no/na question (Harness `choice` primitive)."""
    return {
        "type": "choice",
        "instructions": question
        + " Answer yes only if cited evidence (a command or named test in the state) supports it;"
        " answer na if the question does not apply to this change.",
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
        # Only selected when the diff touches shared-state / async constructs.
        "content_markers": [
            r"\bArc<", r"RwLock", r"Mutex", r"\.await\b", r"\basync\b", r"Atomic",
            r"\bsynchronized\b", r"DispatchQueue", r"\blaunch\b", r"CoroutineScope",
            r"\bactor\b", r"tokio::spawn",
        ],
    },
    "lifecycle": {
        "path_globs": ["android/**", "iOS/**", "**/Platform*"],
        "questions": {
            "resources_closed": _ynq(
                "Are resources (sockets, GATT, file handles, scopes) closed on every exit path?",
                "Every exit path releases the resources it acquired.",
                "A resource leaks on some exit path.",
            ),
        },
        "weight": 1.0,
        "protected": False,
    },
    "testplan": {
        "path_globs": ["**/tests/**", "**/*test*", "**/*Test*"],
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
    removed: Set[str] = set()
    added: List[str] = []
    for line in diff.splitlines():
        if line.startswith("---") or line.startswith("+++"):
            continue
        if line.startswith("-"):
            m = _SYMBOL_RE.match(line[1:])
            if m:
                removed.add(m.group(1))
        elif line.startswith("+"):
            added.append(line[1:])
    return {"removed_symbols": sorted(removed), "added_text": "\n".join(added)}


def select_buckets(
    paths: Iterable[str],
    *,
    deleted: Iterable[str] = (),
    removed_symbols: Iterable[str] = (),
    added_text: Optional[str] = None,
) -> List[str]:
    """Return selected bucket names (instruction always, in BUCKETS order)."""
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
        hits = [p for p in paths if path_matches(p, spec["path_globs"])]
        if name == "concurrency":
            hits = [p for p in hits if not _is_test_path(p)]
            if added_text is not None:
                markers = spec["content_markers"]
                if not any(re.search(m, added_text) for m in markers):
                    hits = []
        if hits:
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


def _score_answer(answer: Dict[str, Any], evidence_cited: bool) -> Dict[str, Any]:
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
    if not evidence_cited and out["verdict"] == "yes":
        out.update(verdict="no", score=0.0, reason="uncited: state.evidence is empty")
    return out


def score_gate(
    answers: Dict[str, Any],
    selected: List[str],
    *,
    evidence_cited: bool,
    min_confidence: float = 0.70,
    bucket_threshold: float = DEFAULT_BUCKET_THRESHOLD,
) -> Dict[str, Any]:
    """Pure scoring of a merged `bucket.qid` answer map. N/A never counts."""
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
            ans = answers.get(f"{name}.{qid}")
            q_results[qid] = (
                _score_answer(ans, evidence_cited)
                if isinstance(ans, dict)
                else {"verdict": "no", "score": 0.0, "reason": "missing answer"}
            )
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
        "buckets": buckets,
    }


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
    """Merge state `files` and git diff paths; diff content only comes from git."""
    info: Dict[str, Any] = {"paths": [], "deleted": [], "removed_symbols": [], "added_text": None}
    if args.changed_paths_from:
        info.update(git_diff_info(args.changed_paths_from, Path(args.repo_root).resolve()))
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
    if info["paths"] or info["deleted"]:
        selected = select_buckets(
            info["paths"],
            deleted=info["deleted"],
            removed_symbols=info["removed_symbols"],
            added_text=info["added_text"],
        )
    else:
        # Legacy invocation with no file list: the original canon pair + instruction.
        selected = ["identity", "routing", INSTRUCTION_BUCKET]
    questions = build_questions(selected)
    evidence_cited = bool(state.get("evidence"))
    state = dict(state, jev_gate={"buckets_selected": selected, "changed_paths": info["paths"][:200]})

    mod = import_harness()
    source = mod["source"]
    print(
        f"[INFO] harness: {source.root} sha={source.sha} "
        f"status={source.status} pinned={source.pinned} "
        f"typesafe_key={bool(mod['key'])} "
        f"openrouter_key={bool(mod.get('openrouter_key'))}"
    )
    print(f"[INFO] buckets_selected={selected} questions={len(questions)}")
    _policy, _ = make_policy()
    evaluator = _policy.evaluator
    if args.no_openrouter:
        result = evaluator.evaluate(state, questions=questions)
        meta = {"endpoint": "typesafe", "fallback_used": False}
    else:
        result, meta = evaluate_jev_with_openrouter_fallback(evaluator, state, questions)
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
        evidence_cited=evidence_cited,
        min_confidence=args.min_confidence,
        bucket_threshold=args.bucket_threshold,
    )
    for name in gate["buckets_selected"]:
        b = gate["buckets"][name]
        score = "n/a" if b["score"] is None else f"{b['score']:.2f}"
        print(f"[INFO] bucket {name}: verdict={b['verdict']} score={score}")
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
        print("[FAIL] UNVERIFIED-JEV - fallback result is not canonical DONE")
        return 0 if args.allow_fallback else 1
    if canonical_pass:
        print(f"[OK] JEV bucketed pass (min_confidence={args.min_confidence}, "
              f"bucket_threshold={args.bucket_threshold}) via {meta.get('endpoint')}")
        return 0
    print("[FAIL] JEV canonical check did not pass")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
