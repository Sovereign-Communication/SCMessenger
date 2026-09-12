#!/usr/bin/env python3
"""SCMessenger Board of Directors (BoD) Governance Engine.

Evaluates strategic, architectural, cryptographic, and philosophical proposals
against the SCMessenger Repo Philosophy and Canonical Doctrine.

Uses sovereign-harness to dispatch the proposal to a 5-member independent model
panel, requiring 100% unanimous agreement (5/5) plus concurrence from the judge
model. If any model dissents, times out, or if the judge disagrees with the
unanimous panel, the resolution fails closed (REJECTED or DEFERRED).

Cost Guarantee: Hard ceiling of $0.10 (10 cents) per run. Default routes to the
free model pool ($0.00).

Usage:
    python scripts/bod_governance.py --proposal "Proposal text..."
    python scripts/bod_governance.py --proposal-file path/to/proposal.txt
    python scripts/bod_governance.py --proposal "..." --record
    python scripts/bod_governance.py --proposal "..." --dry-run

Environment Variables:
    OPENROUTER_API_KEY      OpenRouter API key (or stored in ~/.config/scmorc/)
    HARNESS_MAX_COST        Per-run cost limit (capped at $0.10)
    HARNESS_USE_FREE        Force free models (default: true)
"""

import argparse
import datetime
import json
import os
import re
import sys
import uuid

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

# Ensure harness is importable
try:
    import harness
    from harness.config import (
        HARD_MAX_COST,
        load_settings,
        FREE_PANEL_POOL,
        FREE_JUDGE,
    )
    from harness.errors import HarnessError
    from harness.panel import panel_judge
    from harness.session import governor_for, ledger_for, router_for
    from harness._http import HttpTransport
except ImportError as err:
    print(f"[ERROR] Failed to import sovereign-harness: {err}", file=sys.stderr)
    print("[ERROR] Ensure sovereign-harness is installed or in PYTHONPATH.", file=sys.stderr)
    sys.exit(1)

MAX_BOD_COST_CEILING = 0.10
REQUIRED_PANELISTS = 5

REPO_PHILOSOPHY_RUBRIC = """
--- SCMESSENGER REPO PHILOSOPHY & CANONICAL DOCTRINE ---
1. NODES, NOT RELAYS:
   - There are NO standalone relays in SCMessenger.
   - Only NODES exist, and EVERY node relays: store-and-forward custody is a
     behavior all nodes perform, not a role.
   - The always-on cloud instance (scm-always-on-node) is a CLOUD NODE: a full
     node that also relays, exactly as every other node does.
   - No anonymous packet forwarder exists or may be introduced.
   - Full parity: CLI, Android, iOS, and cloud deployments run the same node
     with the same relay behavior.

2. SOVEREIGNTY & ZERO CENTRAL DEPENDENCIES:
   - Pure peer-to-peer sovereign mesh network.
   - Zero centralized servers, tracking services, telemetry, or external auth.
   - Discovery is LEDGER SHARING between nodes (invite/QR-seeded, gossip-propagated).
   - Eventual delivery via store-and-forward custody is non-negotiable.

3. CRYPTOGRAPHIC INTEGRITY & SOVEREIGNTY:
   - Identity Signing: Ed25519 (private keys must never leave the device).
   - Identity Hash: Blake3(ed25519_pubkey) == identity_id.
   - Key Exchange: Ephemeral X25519 ECDH per-message.
   - KDF: Blake3 derive_key.
   - Encryption: Authenticated XChaCha20-Poly1305 with a 24-byte nonce.
   - Sender Auth: AAD binding combined with Ed25519 envelope signature.
   - Single Entry Point: Storage access strictly through core/src/store/ (IronCore);
     never bypass with direct sled access.
   - Unsafe Rust: Zero unsafe blocks permitted without formal proof and justification.

4. MULTIPLATFORM PARITY & WIRE CONTRACTS:
   - Rust core is the sole cryptographic authority and state machine.
   - Platform adapters (Kotlin/Android, Swift/iOS) are dumb byte pipes.
   - No platform-specific protocol deviations.

5. HYGIENE, VERIFICATION & SAFETY:
   - No emoji anywhere in codebase, logs, comments, or handoffs.
   - No silent truncation: visibility fails open; verdict fails closed.
   - Restoring code is not restoring a feature: wire it or it is dead.
   - Describe only what you have read; verify before claiming completion.
"""

BOD_PANEL_PROMPT_TEMPLATE = """You are a voting member of the Sovereign Communication (SCMessenger) Board of Directors.
Your sole responsibility is to evaluate the following proposal against the project's foundational Repo Philosophy and Canonical Doctrine.

{rubric}

--- PROPOSAL FOR BOARD ADJUDICATION ---
{proposal}

--- VOTING INSTRUCTIONS ---
You must evaluate whether the proposal strictly aligns with the SCMessenger doctrine.
- If the proposal violates, contradicts, weakens, or introduces centralized or non-compliant exceptions to the doctrine, you MUST vote REJECT.
- If the proposal upholds, advances, or strictly adheres to all doctrine principles, vote APPROVE.

You MUST respond with a SINGLE valid JSON object and ABSOLUTELY NO PROSE OR BACKTICKS outside the JSON:
{{
  "vote": "APPROVE" or "REJECT",
  "alignment_score": <number between 0.0 and 1.0>,
  "doctrine_compliances": ["<key points of alignment>"],
  "doctrine_violations": ["<key points of violation, or empty if none>"],
  "rationale": "<concise 1-2 sentence explanation>"
}}
"""


def _extract_json_vote(content: str, truncated: bool = False) -> dict:
    """Extract and validate the JSON vote from a panel member's response.

    Per AGENTS.md Rule 15: NO SILENT TRUNCATION. Visibility fails open; verdict
    fails closed. If truncated or invalid JSON, fail closed as UNKNOWN/TRUNCATED
    rather than guessing.
    """
    if truncated:
        return {
            "vote": "UNKNOWN",
            "alignment_score": 0.0,
            "doctrine_compliances": [],
            "doctrine_violations": ["Truncated by max-tokens cap"],
            "rationale": "Truncated by max-tokens (Rule 15 fail-closed)",
        }
    if not content:
        return {
            "vote": "UNKNOWN",
            "alignment_score": 0.0,
            "doctrine_compliances": [],
            "doctrine_violations": ["Empty response"],
            "rationale": "Empty response",
        }
    text = content.strip()
    # Strip markdown code fencing if present
    match = re.search(r"\{.*\}", text, re.DOTALL)
    if match:
        try:
            parsed = json.loads(match.group(0))
            raw_vote = str(parsed.get("vote", "")).strip().upper()
            if raw_vote in ("APPROVE", "REJECT"):
                return {
                    "vote": raw_vote,
                    "alignment_score": float(parsed.get("alignment_score", 0.0)),
                    "doctrine_compliances": parsed.get("doctrine_compliances", []),
                    "doctrine_violations": parsed.get("doctrine_violations", []),
                    "rationale": str(parsed.get("rationale", "")).strip(),
                }
        except (ValueError, TypeError):
            pass
    # Strict fail-closed: do NOT guess APPROVE or REJECT from cut-off or non-JSON prose
    return {
        "vote": "UNKNOWN",
        "alignment_score": 0.0,
        "doctrine_compliances": [],
        "doctrine_violations": ["Malformed or incomplete JSON response"],
        "rationale": "Malformed response (Rule 15 fail-closed)",
    }


def evaluate_board_proposal(
    proposal: str,
    max_cost: float = MAX_BOD_COST_CEILING,
    max_tokens: int = 8192,
    reasoning_effort: str = "auto",
    dry_run: bool = False,
    task_id: str = None,
    use_paid: bool = False,
) -> dict:
    """Run the 5-judge panel and judge synthesis over a proposal."""
    cost_ceiling = min(float(max_cost), MAX_BOD_COST_CEILING)
    task_id = task_id or f"bod-{uuid.uuid4().hex[:8]}"

    if dry_run:
        now_utc = datetime.datetime.now(datetime.timezone.utc).isoformat()
        return {
            "resolution_id": task_id,
            "timestamp": now_utc,
            "status": "DRY_RUN",
            "proposal": proposal,
            "verdict": "DRY_RUN_OK",
            "summary": "Dry-run simulation mode. No external API calls made.",
            "panel_votes": {},
            "all_models_agreed": True,
            "judge_agreed": True,
            "panel_count": REQUIRED_PANELISTS,
            "required_panelists": REQUIRED_PANELISTS,
            "approvals": REQUIRED_PANELISTS,
            "rejections": 0,
            "judge_model": "dry_run_judge",
            "cost": 0.0,
            "cost_ceiling": cost_ceiling,
            "notes": "Dry-run mode requested. No external API calls made.",
        }

    os.environ["HARNESS_MAX_PANELISTS"] = str(REQUIRED_PANELISTS)
    overrides = {
        "max_panelists": REQUIRED_PANELISTS,
        "max_cost": cost_ceiling,
        "use_free": not use_paid,
    }
    if use_paid:
        overrides["panel_pool"] = (
            "inclusionai/ling-3.0-flash,"
            "meta-llama/llama-3.1-8b-instruct,"
            "deepseek/deepseek-chat,"
            "openai/gpt-4o-mini,"
            "ibm-granite/granite-4.0-h-micro"
        )
        overrides["judge"] = "openai/gpt-4o-mini"
    settings = load_settings(overrides=overrides)

    api_key, gov = governor_for(settings, cost_ceiling)
    ledger = ledger_for(settings, caller="bod-governance")
    router = router_for(settings)

    prompt = BOD_PANEL_PROMPT_TEMPLATE.format(
        rubric=REPO_PHILOSOPHY_RUBRIC.strip(), proposal=proposal.strip()
    )

    panel_pool = settings.panel_pool
    judge_model = settings.judge or (FREE_JUDGE if not use_paid else "inclusionai/ling-3.0-flash")

    result = panel_judge(
        transport=HttpTransport(),
        api_key=api_key,
        governor=gov,
        prompt=prompt,
        panel=panel_pool,
        judge=judge_model,
        max_tokens=max_tokens,
        reasoning_effort=reasoning_effort,
        reasoning_token_budget=0.4,
        task_id=task_id,
        ledger=ledger,
        max_panelists=REQUIRED_PANELISTS,
        run_convergence=False,
        free_tier=settings.use_free,
    )

    actual_cost = result.get("actual_cost", getattr(gov, "spent", 0.0))
    panel_results = result.get("panel_results", [])
    judge_synthesis = result.get("judge_synthesis", {})
    consensus = result.get("consensus", {})

    # Tally the 5 panel votes
    votes_by_model = {}
    approvals = 0
    rejections = 0
    unknowns = 0

    for res in panel_results:
        model = res.get("model", "unknown_model")
        content = res.get("content", "")
        truncated = bool(res.get("truncated")) or res.get("finish_reason") == "length"
        vote_data = _extract_json_vote(content, truncated=truncated)
        votes_by_model[model] = vote_data
        if vote_data["vote"] == "APPROVE":
            approvals += 1
        elif vote_data["vote"] == "REJECT":
            rejections += 1
        else:
            unknowns += 1

    panel_count = len(panel_results)
    shortfall = panel_count < REQUIRED_PANELISTS or unknowns > 0

    # Strict unanimity check: ALL 5 models must agree on the vote
    unanimous_approved = panel_count == REQUIRED_PANELISTS and approvals == REQUIRED_PANELISTS
    unanimous_rejected = panel_count == REQUIRED_PANELISTS and rejections == REQUIRED_PANELISTS
    all_models_agreed = (unanimous_approved or unanimous_rejected) and not shortfall

    # Check judge consensus
    judge_content = judge_synthesis.get("content", "") if isinstance(judge_synthesis, dict) else ""
    judge_verdict_raw = consensus.get("verdict", "")
    judge_agreed = False

    if all_models_agreed:
        target_vote = "APPROVE" if unanimous_approved else "REJECT"
        judge_verdict_str = str(consensus.get("verdict", "")).lower()
        judge_content_str = str(judge_content).lower()
        judge_deferred = bool(consensus.get("defer", False))
        judge_agreement = str(consensus.get("agreement", "")).lower()
        disagreements = consensus.get("disagreements", []) or []

        if target_vote == "APPROVE":
            # Judge agrees with unanimous approval if:
            # 1. Judge did not defer
            # 2. Judge recognized high/medium consensus without unresolvable disagreements
            # 3. Verdict indicates approval/alignment and does not recommend rejection
            indicates_reject = any(
                phrase in judge_verdict_str
                for phrase in [
                    "recommend reject",
                    "should reject",
                    "vote: reject",
                    "verdict: reject",
                    "must reject",
                    "reject the proposal",
                    "does not align",
                    "violates",
                ]
            )
            indicates_approve = (
                any(
                    token in judge_verdict_str
                    for token in ["approv", "align", "compliant", "adhere", "accept", "passed"]
                )
                or (judge_agreement == "high" and not disagreements)
            )

            if not judge_deferred and judge_agreement in ("high", "medium") and not indicates_reject and indicates_approve:
                judge_agreed = True
        elif target_vote == "REJECT":
            # Judge agrees with unanimous rejection if judge deferred, saw low agreement, or flagged rejection
            indicates_reject = any(
                token in judge_verdict_str
                for token in ["reject", "violate", "non-compliant", "conflict", "fail"]
            )
            if judge_deferred or judge_agreement in ("low", "none") or indicates_reject:
                judge_agreed = True

    # Derive the final Board of Directors decision
    if shortfall:
        final_verdict = "DEFERRED_PANEL_SHORTFALL"
        status = "DEFERRED"
        summary = f"Only {panel_count}/{REQUIRED_PANELISTS} models submitted valid votes. Fails closed."
    elif not all_models_agreed:
        final_verdict = "REJECTED_DISSENT"
        status = "REJECTED"
        summary = f"Dissent detected among panel: {approvals} APPROVE, {rejections} REJECT. 100% unanimity required."
    elif not judge_agreed:
        final_verdict = "REJECTED_JUDGE_DIVERGENCE"
        status = "REJECTED"
        summary = "Panel achieved unanimity, but the judge model failed to confirm concurrence."
    elif unanimous_approved:
        final_verdict = "APPROVED"
        status = "APPROVED"
        summary = "Unanimous 5/5 panel approval with judge concurrence. Proposal aligns with repo philosophy."
    else:
        final_verdict = "REJECTED"
        status = "REJECTED"
        summary = "Unanimous 5/5 panel rejection with judge concurrence. Proposal violates repo philosophy."

    resolution = {
        "resolution_id": task_id,
        "timestamp": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "status": status,
        "proposal": proposal,
        "verdict": final_verdict,
        "summary": summary,
        "all_models_agreed": all_models_agreed,
        "judge_agreed": judge_agreed,
        "panel_count": panel_count,
        "required_panelists": REQUIRED_PANELISTS,
        "approvals": approvals,
        "rejections": rejections,
        "panel_votes": votes_by_model,
        "judge_model": result.get("judge_model", judge_model),
        "judge_synthesis": judge_synthesis,
        "consensus": consensus,
        "cost": actual_cost,
        "cost_ceiling": cost_ceiling,
    }

    return resolution


def record_resolution_to_handoff(resolution: dict, repo_root: str):
    """Record the resolution in HANDOFF/BOD_STATE.md."""
    state_file = os.path.join(repo_root, "HANDOFF", "BOD_STATE.md")
    if not os.path.exists(state_file):
        print(f"[WARNING] State file {state_file} does not exist. Skipping record.", file=sys.stderr)
        return

    entry = [
        f"\n### Resolution {resolution['resolution_id']} [{resolution['status']}]",
        f"- **Timestamp**: {resolution['timestamp']}",
        f"- **Verdict**: `{resolution['verdict']}`",
        f"- **Cost**: ${resolution['cost']:.6f} (Ceiling: ${resolution['cost_ceiling']:.2f})",
        f"- **Summary**: {resolution['summary']}",
        f"- **Panel Voting** ({resolution['approvals']}/{resolution['required_panelists']} APPROVE):",
    ]

    for model, vote_info in resolution.get("panel_votes", {}).items():
        vote = vote_info.get("vote", "UNKNOWN")
        score = vote_info.get("alignment_score", 0.0)
        rationale = vote_info.get("rationale", "")
        entry.append(f"  - `{model}`: **{vote}** (Score: {score:.2f}) - _{rationale}_")

    entry.append(f"- **Judge Model**: `{resolution.get('judge_model', 'unknown')}` (Agreed: {resolution.get('judge_agreed')})")
    entry.append("- **Proposal Text**:")
    entry.append(f"  > {resolution['proposal'].replace(chr(10), chr(10) + '  > ')}")
    entry.append("")

    with open(state_file, "a", encoding="utf-8") as f:
        f.write("\n".join(entry))

    print(f"[OK] Resolution {resolution['resolution_id']} appended to HANDOFF/BOD_STATE.md")


def main():
    parser = argparse.ArgumentParser(
        description="SCMessenger Board of Directors (BoD) 5-Judge Governance Adjudicator"
    )
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--proposal", type=str, help="Inline proposal text to evaluate")
    group.add_argument("--proposal-file", type=str, help="Path to file containing proposal")

    parser.add_argument(
        "--max-cost",
        type=float,
        default=MAX_BOD_COST_CEILING,
        help=f"Maximum cost allowed in USD (default: ${MAX_BOD_COST_CEILING:.2f}, capped at $0.10)",
    )
    parser.add_argument(
        "--max-tokens",
        type=int,
        default=8192,
        help="Maximum completion tokens per call (default: 8192)",
    )
    parser.add_argument(
        "--reasoning-effort",
        type=str,
        default="auto",
        choices=["auto", "low", "medium", "high", "off"],
        help="Reasoning effort requested from reasoning models (default: auto)",
    )
    parser.add_argument(
        "--paid",
        action="store_true",
        help="Dispatch through paid model pool (bounded by $0.10 ceiling)",
    )
    parser.add_argument(
        "--record",
        action="store_true",
        help="Append resolution to HANDOFF/BOD_STATE.md",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Simulate execution without external network calls",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output raw JSON resolution only",
    )

    args = parser.parse_args()

    proposal_text = args.proposal
    if args.proposal_file:
        try:
            with open(args.proposal_file, "r", encoding="utf-8") as f:
                proposal_text = f.read().strip()
        except OSError as e:
            print(f"[ERROR] Failed to read proposal file: {e}", file=sys.stderr)
            sys.exit(1)

    if not proposal_text:
        print("[ERROR] Proposal text is empty.", file=sys.stderr)
        sys.exit(1)

    repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))

    try:
        resolution = evaluate_board_proposal(
            proposal=proposal_text,
            max_cost=args.max_cost,
            max_tokens=args.max_tokens,
            reasoning_effort=args.reasoning_effort,
            dry_run=args.dry_run,
            use_paid=args.paid,
        )
    except HarnessError as e:
        print(f"[FAIL] Sovereign-harness error: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"[FAIL] Unexpected governance error: {e}", file=sys.stderr)
        sys.exit(1)

    if args.record:
        record_resolution_to_handoff(resolution, repo_root)

    if args.json:
        print(json.dumps(resolution, indent=2))
        sys.exit(0 if resolution["status"] == "APPROVED" else 2)

    # Human-readable format
    print("=" * 70)
    print(f"BOARD OF DIRECTORS RESOLUTION: {resolution['resolution_id']}")
    print(f"Status: [{resolution['status']}] | Verdict: {resolution['verdict']}")
    print(f"Cost: ${resolution['cost']:.6f} (Ceiling: ${resolution['cost_ceiling']:.2f})")
    print("=" * 70)
    print(f"Proposal: {resolution['proposal']}")
    print("-" * 70)
    print(f"Summary: {resolution['summary']}")
    print(f"Panel Voting ({resolution['approvals']}/{resolution['required_panelists']} APPROVE):")
    for model, vote_info in resolution.get("panel_votes", {}).items():
        v = vote_info.get("vote", "UNKNOWN")
        s = vote_info.get("alignment_score", 0.0)
        r = vote_info.get("rationale", "")
        print(f"  [{v}] {model} (score: {s:.2f}) - {r}")
    print(f"Judge Model: {resolution.get('judge_model')} (Agreed: {resolution.get('judge_agreed')})")
    print("=" * 70)

    # Return exit code: 0 if APPROVED or DRY_RUN, 2 if REJECTED/DEFERRED
    sys.exit(0 if resolution["status"] in ("APPROVED", "DRY_RUN") else 2)


if __name__ == "__main__":
    main()
