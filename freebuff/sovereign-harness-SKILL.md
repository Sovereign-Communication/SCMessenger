# Sovereign Harness — lane usage guide

## Why
Harness calls cost a fraction of a cent vs full lane LLM sessions. Route judgment
work here; keep code writing, repo-local file ops, and git operations local.

## When — static routing table
| Task | Route to |
|---|---|
| Plan assessment before implementing | Jev score — plan-quality aspects ONLY (completeness, soundness, risk coverage, examples sufficient). Never gate on predictive aspects: "will this work" plateaus ~0.5–0.6 and DROPS with more detail. |
| Self-review / code review | Jev review, ≥90/100 gate. Extends the Rule-8 spec: every self-review, not just security sign-offs. |
| Triage (weakest link / highest impact) | Jev Choice |
| Model / provider selection | Provider router (free-tier arbitrage) |
| Deferral rationale ("should I wait on X?") | Jev Noul: "Is [X] the reason for [Y]?" |

Do NOT route: code writing, repo-local file ops, git operations.

## How
Via the installed MCP server. The TypeSafe API key exists on this lane — reference
it and trust the agent has access.

## Mechanics (verified live)
- uuid nonce per request, inside `state` (top-level `nonce` field 400s). Prevents caching.
- Order: Noul → Score → Choice, sequentially. Never batch.
- Score `criteria` = LIST of level strings, ≤5 levels (dict format 422s). Choice
  `criteria` = DICT. Noul `criteria` = {"true": ..., "false": ...}.
- Score 0–3 → percent = score/3×100. Gate bar ≥90/100 means score ≥2.7.
- For 0–100 granularity: ≤5 levels, expected value from returned probabilities.
- Ask Jev about itself via Choice (0.91–0.92 confidence on terminology).

## Fallback (Jev gate v1 — ratified 2026-10-21, replaces the old 0.95 bar)

Gate on risk, not numbers. Decision: `is_destructive` ≥ 0.5 → escalate; disposition lead < 0.20 → escalate; `advances_goal` < 0.95 → INVESTIGATE (not auto-escalate — no real risk found → proceed with note; risk found → escalate); choice confidence is telemetry, never a gate. Assessment (5 aspects): 4/5 ≥ 90 with 5th at 85–89 → auto-investigate; no risk → PASS with note; risk → BLOCK. Single aspect at 85+ never blocks alone if cleared; below 85 → fix and re-assess. Below-bar verdicts start investigation automatically — escalate only on actual risk. Full policy: `jev-gate-policy-v1.md` (ships in the freebuff handoff branches).
