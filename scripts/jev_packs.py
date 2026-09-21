"""SCMessenger JEV question packs — bounded semantic judgments only.

Code owns harvest facts (counts, paths, PR states, greps). Jev owns typed
judgments on compact state batches. Aligns with Harness JEV-P3 note: packs
may live in one module; unkeyed callers must mark is_fallback.

These packs are SCMessenger-scoped insight packs. They do not replace
Harness `jev.py` / `jev_policy.py` ownership. Compatible with future
`jev_packs.py` pattern (operator-declared buckets only; no free invention).
"""
from __future__ import annotations

from typing import Any, Dict


def _noul(instructions: str, yes: str, no: str) -> Dict[str, Any]:
    return {
        "type": "noul",
        "instructions": instructions,
        "criteria": {"true": yes, "false": no},
    }


def _score(instructions: str, levels) -> Dict[str, Any]:
    return {
        "type": "score",
        "instructions": instructions,
        "criteria": list(levels),
    }


def _choice(instructions: str, criteria: Dict[str, str]) -> Dict[str, Any]:
    return {
        "type": "choice",
        "instructions": instructions,
        "criteria": dict(criteria),
    }


def pain_points_pack() -> Dict[str, Dict[str, Any]]:
    """Historical/process pain on a batch of extracted signals."""
    return {
        "dominant_pain": _choice(
            "Which theme best explains the pain in this batch?",
            {
                "identity_transport": "Identity format, routing feed, WiFi delivery",
                "ci_queue_gates": "CI queue, required checks, path filters, merges",
                "process_dispatch": "Orchestration, freebuff paste, stale status, wrong premises",
                "device_release": "Keystore, APK install, operator device gates",
                "docs_truth": "Contradictory docs, unindexed tickets, stale plans",
            },
        ),
        "blocks_tag_or_working_mesh": _noul(
            "Do the signals in this batch block reliable day-to-day mesh or the 0.4.0 path?",
            "They block working mesh, WP completion, or the 0.4.0 checklist.",
            "They are hygiene or post-tag work, not mesh/tag blockers.",
        ),
        "severity": _score(
            "Rate overall severity of this batch for SCMessenger completion.",
            ["low hygiene", "medium friction", "high delivery blocker"],
        ),
    }


def unification_pack() -> Dict[str, Dict[str, Any]]:
    """Unification gaps across identity, docs, queues, CI, code paths."""
    return {
        "unification_gap": _noul(
            "Does this batch show a real unification gap (two sources of truth or dual flavors)?",
            "Yes — dual flavors, dual plans, or duplicate ownership.",
            "No — single source already or not a unification issue.",
        ),
        "primary_layer": _choice(
            "Which layer needs unification first in this batch?",
            {
                "identity_keys": "Public-key hex vs peer id vs recovery fields",
                "dispatch_docs": "Plan/queue/ticket status and paste authority",
                "routing_transports": "One routing feed for all data links",
                "delivery_status": "Transport ACK vs app delivered vs receipts",
                "ci_artifacts": "Signing, APK, same-SHA fleet artifacts",
            },
        ),
        "fix_class": _choice(
            "What is the least expensive correct fix class?",
            {
                "tests_only": "Regression tests on existing main behavior",
                "small_code_fix": "Scoped code fix with acceptance grep",
                "docs_status_fix": "Status/plan/index rewrite only",
                "operator_ruling": "Needs human decision first",
                "live_evidence": "Needs 3-node logs / device proof",
            },
        ),
    }


def orchestration_pack() -> Dict[str, Dict[str, Any]]:
    """Orchestration / freebuff lane health for a slice of tickets."""
    return {
        "dispatch_ready": _noul(
            "Is this ticket ready for Freebuff paste without guesswork?",
            "Premise verified, acceptance is mechanical, files named.",
            "Premise stale, acceptance vague, or needs operator ruling first.",
        ),
        "route": _choice(
            "Best owner for the next step on this batch?",
            {
                "freebuff_impl": "Scoped Freebuff implementation PR",
                "orchestrator_merge": "Merge train / CI / docs already done",
                "operator_device": "Keystore, phone UI, secrets, tag",
                "harness_verify": "Uncertain claim — harness/JEV verify first",
                "defer_post_wave": "0.5.0 / beach-join later / not 0.4.0 path",
            },
        ),
        "process_health": _score(
            "Rate orchestration health represented by this batch.",
            ["broken misdispatch", "stale but recoverable", "clear and executable"],
        ),
    }


def historical_process_pack() -> Dict[str, Dict[str, Any]]:
    """Past merge/train/audit lessons for future process."""
    return {
        "lesson_strength": _score(
            "How strong is the process lesson in this historical slice?",
            ["weak anecdote", "repeatable lesson", "standing rule candidate"],
        ),
        "rule_candidate": _noul(
            "Should this become a standing runbook/AGENTS rule if not already?",
            "Yes — recurring failure class worth a rule.",
            "No — already covered or one-off.",
        ),
        "theme": _choice(
            "Primary theme of this historical slice?",
            {
                "ci_hygiene": "Superseded runs, path filters, strict BEHIND",
                "premise_drift": "Stale tickets vs code, false STILL-OPEN",
                "rule8_gating": "Transport/core review before merge",
                "fleet_same_sha": "Artifacts, identity, deploy consistency",
                "freebuff_paste_cost": "Wrong tickets, unindexed queue, operator time",
            },
        ),
    }


def canonical_completion_pack() -> Dict[str, Dict[str, Any]]:
    """WP/WiFi canonical completion judgment (mirrors jev_canonical_check)."""
    return {
        "canon_identity": _noul(
            "Does the change/evidence keep ONE contact identity flavor (public-key hex)?",
            "Hex remains the addressing key; peer id only derived for libp2p.",
            "A second contact-address flavor is introduced or primary.",
        ),
        "canon_routing_feed": _noul(
            "Does the change/evidence keep ONE routing feed entry for data links?",
            "All data-links use routing_peer_seen (or WP unrelated to routing).",
            "Parallel feed or connected-without-feed.",
        ),
        "instruction_matches": _noul(
            "Do implementation and evidence satisfy the WP instruction/acceptance?",
            "Acceptance met with command/test evidence.",
            "Unmet, contradicted, or assertion-only.",
        ),
    }


PACKS = {
    "pain_points": pain_points_pack,
    "unification": unification_pack,
    "orchestration": orchestration_pack,
    "historical_process": historical_process_pack,
    "canonical_completion": canonical_completion_pack,
}


def pack_for(name: str) -> Dict[str, Dict[str, Any]]:
    if name not in PACKS:
        raise KeyError(f"unknown JEV insight pack: {name}")
    return PACKS[name]()
