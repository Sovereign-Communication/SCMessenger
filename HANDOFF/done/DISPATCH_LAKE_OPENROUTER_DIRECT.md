# DISPATCH_LAKE_OPENROUTER_DIRECT -- wire the openrouter_direct lane (DeepSeek V4 Flash)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: todo
Tier: CODER
Domain: tooling + docs
Target Files: scripts/delegate_task.py, scripts/lake_route.py,
docs/ORCHESTRATION.md, docs/orchestration/SCM_UNIFIED_LAKE_ORCHESTRATION.md

## Requirement

Operator directive 2026-08-04: add a new lane `openrouter_direct` --
DeepSeek V4 Flash via OpenRouter, a super-cheap but capable model, used as
a BACKUP / as-needed lane for clearly scoped tasks (never the primary
CODER). Spend cap USD 1/day, its own API key (separate from
openrouter.env and openrouter_fusion.env). The registry block and ladder
positions already exist in tmp/lakes/registry.json (gitignored, added
2026-08-04) -- this ticket wires the code and canonical docs to match.

## Exact changes

1. scripts/delegate_task.py:
   - Add `"openrouter_direct"` to the `--provider` argparse choices.
   - `get_api_key`: new branch -- env `OPENROUTER_DIRECT_API_KEY` or env
     `OpenRouter_Paid_Key` (the operator's file uses the latter), else
     `_key_from_env_file("~/.config/scmorc/openrouter_direct.env",
     ("OPENROUTER_DIRECT_API_KEY", "OpenRouter_Paid_Key"))`. Follow the
     existing provider-branch pattern exactly (env first, then file).
   - Endpoint: reuse OPENROUTER_URL (same OpenRouter endpoint).
   - Default model when --model omitted: `deepseek/deepseek-v4-flash-0731`
     (probe-verified 2026-08-04; the `-latest` alias does NOT exist on
     OpenRouter -- HTTP 400 invalid model ID).
   - IMPORTANT: do NOT apply the openrouter provider's free-models-only
     restriction (`:free` enforcement) to openrouter_direct -- that
     restriction belongs to the `openrouter` provider only. Do NOT apply
     qwenpaid's enable_thinking / 1800s-timeout special cases either;
     plain OpenAI-compatible request path is correct.
   - Backoff on 429: reuse the standard escalating backoff the other
     OpenAI-compatible providers use (no same-model-only special case).
2. scripts/lake_route.py: in module-level TIER_LADDERS insert
   `"openrouter_direct"` immediately AFTER `"openrouter"` in both the
   FLASH and CODER ladders (matches tmp/lakes/registry.json). No other
   ladder changes.
3. docs/ORCHESTRATION.md Section 1: add an Active-lakes table row:
   `openrouter_direct | OpenRouter (dedicated key) | Backup lane for
   clearly scoped tasks; DeepSeek V4 Flash, USD 1/day cap | FLASH/CODER`.
   Keep the candidate-lakes note accurate (deepseek direct API access
   remains a candidate; this lane is OpenRouter-routed).
4. docs/orchestration/SCM_UNIFIED_LAKE_ORCHESTRATION.md: add the
   openrouter_direct registry entry in Section 1 (endpoint, key file
   ~/.config/scmorc/openrouter_direct.env, USD 1/day cap, model
   deepseek/deepseek-v4-flash-0731; note that the -latest alias is NOT a
   valid OpenRouter model ID) and one line in the routing-strategy
   section: backup/as-needed lane, never primary CODER.

## Acceptance criteria

- `python scripts/delegate_task.py --help` lists openrouter_direct as a
  provider; a dispatch with `--provider openrouter_direct` and NO key
  file present fails cleanly with the standard missing-key error (exit 1),
  not a traceback.
- `python scripts/lake_route.py --help` still clean; TIER_LADDERS edit is
  the only lake_route.py change.
- The openrouter provider's `:free`-only enforcement is untouched.
- No emoji anywhere; `python scripts/rules_check.py` exit 0 on all four
  touched files.
- Do NOT create, read, or guess at any key file contents. Do NOT run
  cargo/gradle. Do NOT touch any other script or doc.

## Gate

python scripts/rules_check.py && python scripts/delegate_task.py --help && python scripts/lake_route.py --help

## Notes for the orchestrator

After the diff lands and gates pass: (1) operator places the key file
~/.config/scmorc/openrouter_direct.env (orchestrator never sees the key),
(2) orchestrator probe-dispatches one trivial task through the lane and
records the ledger entry. Known trap: dispatch this task with FULL file
paths, never :Lstart-Lend scoped paths -- the scoped-allowlist bug
(DELEGATE_SCOPED_FILES_ALLOWLIST_FIX) rejects scoped hunks.
