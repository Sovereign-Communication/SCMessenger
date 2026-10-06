"""SCMessenger-local JEV access: TypeSafe first, OpenRouter fallback.

Operator 2026-09-21: TypeSafe Jev account returned Internal Server Error.
Fallback uses the OpenRouter API with model ``~typesafe/jev-latest``.

This module never discovers or edits a Harness checkout. It loads the
admitted source through ``harness_source.py`` and adds only the consumer-side
JEV fallback behavior.
"""
from __future__ import annotations

import json
import os
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, Dict, Optional, Tuple

try:
    from .harness_source import (
        HarnessSourceError,
        import_harness_modules,
        resolve_source,
    )
except ImportError:
    from harness_source import (
        HarnessSourceError,
        import_harness_modules,
        resolve_source,
    )

OPENROUTER_CHAT_URL = "https://openrouter.ai/api/v1/chat/completions"
# Operator 2026-09-21: ~typesafe/jev-latest is an OpenRouter **decisions** model
# — it must use the alpha decisions API, not chat/completions.
OPENROUTER_JEV_DECISIONS_URL = "https://openrouter.ai/api/alpha/decisions"
OPENROUTER_JEV_MODEL = os.environ.get(
    "SCM_JEV_OPENROUTER_MODEL", "~typesafe/jev-latest"
)


def local_harness_root() -> Path:
    """Compatibility wrapper around the single source resolver."""
    try:
        return resolve_source().root
    except HarnessSourceError as exc:
        raise SystemExit(f"[FAIL] {exc}") from exc


def import_harness():
    """Load the declared Harness surface from the resolved checkout only."""
    try:
        mod = import_harness_modules()
    except HarnessSourceError as exc:
        raise SystemExit(f"[FAIL] {exc}") from exc
    try:
        key = mod["resolve_jev_key"]() or None
    except Exception:  # noqa: BLE001
        key = None
    try:
        or_key = mod["resolve_api_key"]() or None
    except Exception:  # noqa: BLE001
        or_key = os.environ.get("OPENROUTER_API_KEY") or None
    return {
        "root": mod["root"],
        "source": mod["source"],
        "key": key,
        "openrouter_key": or_key,
        "JevEvaluator": mod["JevEvaluator"],
        "JevPolicy": mod["JevPolicy"],
        "JevEvaluationResult": mod["JevEvaluationResult"],
        "jev_cost": mod["jev_cost"],
        "validate_operator_pack": mod["validate_operator_pack"],
        "issue_sort_question_pack": mod["issue_sort_question_pack"],
        "match_keywords": mod["match_keywords"],
        "_validate_questions": mod["_validate_questions"],
        "_parse_answer": mod["_parse_answer"],
    }


def make_policy(key=None, endpoint: Optional[str] = None, model: Optional[str] = None):
    mod = import_harness()

    class _S:
        jev_api_key = key if key is not None else mod["key"]
        jev_endpoint = endpoint or os.environ.get(
            "HARNESS_JEV_ENDPOINT", "https://api.typesafe.ai/v1/systemone"
        )
        jev_model = model or os.environ.get("HARNESS_JEV_MODEL", "jev-latest")
        min_confidence = 0.70

    return mod["JevPolicy"](_S()), mod


def _openrouter_post(
    api_key: str,
    model: str,
    state: Any,
    questions: Dict[str, Any],
    timeout: int = 120,
) -> Tuple[int, Dict[str, Any]]:
    """POST a JEV-style typed pack via OpenRouter alpha decisions API.

    Model ``~typesafe/jev-latest`` is a decisions model (OpenRouter rejects
    chat/completions for it). Account must allow the ``typesafe`` provider.
    """
    payloads = [
        {"model": model, "state": state, "questions": questions},
        {
            "model": model,
            "input": {"state": state, "questions": questions},
            "questions": questions,
        },
    ]
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
        "HTTP-Referer": "https://github.com/Sovereign-Communication/SCMessenger",
        "X-Title": "SCMessenger JEV fallback",
    }
    last: Dict[str, Any] = {}
    last_status = 0
    for payload in payloads:
        req = urllib.request.Request(
            OPENROUTER_JEV_DECISIONS_URL,
            data=json.dumps(payload).encode("utf-8"),
            headers=headers,
            method="POST",
        )
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                body = resp.read().decode("utf-8", errors="replace")
                status = int(resp.status)
        except urllib.error.HTTPError as exc:
            body = exc.read().decode("utf-8", errors="replace") if exc.fp else str(exc)
            status = int(exc.code)
        except Exception as exc:  # noqa: BLE001
            last = {"error": str(exc)}
            last_status = 0
            continue
        try:
            parsed = json.loads(body)
        except Exception:  # noqa: BLE001
            last = {"error": "non-json", "body": body[:500], "http": status}
            last_status = status
            continue
        last_status = status
        last = parsed if isinstance(parsed, dict) else {"raw": parsed}
        # Provider-not-allowed / invalid schema → try next payload shape once
        if status == 200:
            return status, last
        msg = json.dumps(last.get("error") or last)[:300]
        if "allowed-providers" in msg or "typesafe" in msg.lower():
            return status, last
    return last_status, last


def _extract_or_answers(or_resp: Dict[str, Any]) -> Dict[str, Any]:
    """Pull a TypeSafe-shaped answers map out of an OpenRouter chat body."""
    try:
        content = or_resp["choices"][0]["message"]["content"]
    except Exception:  # noqa: BLE001
        return {}
    if isinstance(content, dict):
        return content.get("answers") or content
    if not isinstance(content, str):
        return {}
    text = content.strip()
    if text.startswith("```"):
        text = text.split("```", 2)[1]
        if text.startswith("json"):
            text = text[4:]
    try:
        obj = json.loads(text)
    except Exception:  # noqa: BLE001
        # last resort: first {...} block
        i, j = text.find("{"), text.rfind("}")
        if i >= 0 and j > i:
            try:
                obj = json.loads(text[i : j + 1])
            except Exception:  # noqa: BLE001
                return {}
        else:
            return {}
    if isinstance(obj, dict):
        return obj.get("answers") or obj
    return {}


def evaluate_jev_with_openrouter_fallback(
    evaluator,
    state: Any,
    questions: Dict[str, Any],
    *,
    openrouter_key: Optional[str] = None,
    openrouter_model: str = OPENROUTER_JEV_MODEL,
) -> Tuple[Any, Dict[str, Any]]:
    """TypeSafe first; OpenRouter `~typesafe/jev-latest` on ISE/failure.

    Returns (JevEvaluationResult-like, meta) where meta records which endpoint
    produced the result. Never invents buckets; parse failures stay honest.
    """
    mod = import_harness()
    meta: Dict[str, Any] = {
        "primary": "typesafe",
        "fallback_used": False,
        "openrouter_model": openrouter_model,
    }
    primary = evaluator.evaluate(state, questions)
    primary_failed = bool(primary.is_fallback) or primary.verdict == "fail"
    # Only fall back when TypeSafe itself is unhealthy, not on a clean fail
    # that answered the pack (supported>0 answers present).
    typesafe_unhealthy = bool(primary.is_fallback) and not primary.answers
    if not typesafe_unhealthy:
        # Also fall back when reasons mention server/ise and no answers
        reasons = " ".join(primary.reasons or []).lower()
        if primary.is_fallback or (
            not primary.answers
            and any(x in reasons for x in ("http", "500", "error", "invalid", "reject"))
        ):
            typesafe_unhealthy = True
    if not (typesafe_unhealthy or (primary_failed and not primary.answers)):
        meta["endpoint"] = "typesafe"
        meta["cost"] = primary.cost
        meta["input_tokens"] = primary.input_tokens
        return primary, meta

    key = openrouter_key or mod.get("openrouter_key")
    if not key:
        meta["endpoint"] = "typesafe"
        meta["fallback_skipped"] = "no OpenRouter key"
        return primary, meta

    status, or_resp = _openrouter_post(key, openrouter_model, state, questions)
    meta["openrouter_http"] = status
    answers_raw = _extract_or_answers(or_resp)
    if not answers_raw:
        meta["endpoint"] = "typesafe"
        meta["fallback_used"] = False
        meta["openrouter_error"] = or_resp.get("error") if isinstance(or_resp, dict) else "parse"
        return primary, meta

    # Validate/normalize through the same admitted Harness symbols.
    try:
        expected = mod["_validate_questions"](questions)
        answers = {}
        for key_q, question in expected.items():
            if key_q not in answers_raw:
                raise ValueError(f"OpenRouter response missing {key_q}")
            answers[key_q] = mod["_parse_answer"](
                answers_raw[key_q], question["type"], key_q, question
            )
        nouls = [a["noul"] for a in answers.values() if a.get("type") == "noul"]
        action_conf = [
            a["confidence"]
            for a in answers.values()
            if a.get("type") in ("choice", "score")
        ]
        supported = min(nouls) if nouls else 1.0
        confidence = min(action_conf) if action_conf else 0.0
        min_conf = getattr(evaluator, "min_confidence", 0.70)
        verdict = (
            "pass"
            if supported >= min_conf
            and (not action_conf or confidence >= min_conf)
            else "fail"
        )
        reasons = []
        for key_q, answer in answers.items():
            if answer.get("type") == "noul":
                reasons.append(f"{key_q} (noul via openrouter): {answer['noul']}")
            elif answer.get("type") == "choice":
                reasons.append(
                    f"{key_q} (choice via openrouter): {answer['choice']} "
                    f"(conf: {answer.get('confidence')})"
                )
            else:
                reasons.append(
                    f"{key_q} (score via openrouter): {answer.get('score')} "
                    f"(conf: {answer.get('confidence')})"
                )
        usage = {}
        if isinstance(or_resp, dict):
            usage = or_resp.get("usage") or {}
        input_tokens = int(usage.get("prompt_tokens") or usage.get("input_tokens") or 0)
        output_tokens = int(usage.get("completion_tokens") or usage.get("output_tokens") or 0)
        cost = mod["jev_cost"](input_tokens) if input_tokens else 0.0
        result = mod["JevEvaluationResult"](
            verdict,
            confidence,
            supported,
            answers,
            reasons,
            cost=cost,
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            is_fallback=False,
            model=openrouter_model,
        )
        meta["endpoint"] = "openrouter"
        meta["fallback_used"] = True
        meta["cost"] = cost
        meta["input_tokens"] = input_tokens
        return result, meta
    except Exception as exc:  # noqa: BLE001
        meta["endpoint"] = "typesafe"
        meta["fallback_used"] = False
        meta["openrouter_parse_error"] = str(exc)
        return primary, meta
