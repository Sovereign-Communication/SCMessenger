# Freebuff recovery session 2026-09-13 — PHASE 1 COMPLETE, PR #282 MERGED

Written by Buffy (Freebuff lane). All claims cite commands run this session.

## Executed (in order)

1. **Model-selection overhaul, probe-verified** (operator rulings 2026-09-13):
   - OpenRouter daily rankings pulled (rankings-datasets API): deepseek-v4.1-flash
     #3, glm-5.3-flash #5, gpt-5.6-luna #1, gemini-3.8-flash = current Gemini gen.
   - Root cause of all "reasoning-only vote" failures found: harness chat.py
     `_effort_to_send("off")` OMITS the reasoning key -> provider default = ON.
     Verified fix: send `{"effort":"none"}` (probed live: v4.1-flash 63-token
     clean vote $0.000055; kimi-k3 same; glm-5.3-flash/gpt-5-mini are
     mandatory-reasoning routes -> 400, correctly retried by existing logic).
   - Learned-BYOK silent filtering found and documented
     (byok_prefixes.json had `google/`, `m/`; cleared). glm-5.3-flash and
     gpt-5-mini verified as deep-think members at >=2048 token budgets.
   - Handoff for the Harness repo: `Harness/docs/MODEL_SELECTION_HANDOFF_2026-09-13.md`
     (patch for chat.py, pool slates, allocation policy, checklist).
   - scripts/bod_governance.py: pools rebuilt on verified emitters; tests 6/6 OK.
2. **Double-blind re-review completed on corrected premises:**
   - bod-f16cfd7f (4/5 APPROVE, 5th lost to pre-patch v4.1-flash behavior)
   - **bod-bbb49423: APPROVED 5/5** (gpt-4o-mini 1.00, ling 0.95, v4-flash 0.95,
     gpt-5-mini 0.95, gpt-5.6-luna 0.96; judge v4.1-flash AGREED), $0.0038 actual.
   - Blind B verdict file updated; P1 ticket FILED for real:
     `HANDOFF/todo/P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION.md` (pre-existing
     Kotlin curve check on main; bod-dd336324 remedy; before v0.4.0 tag).
3. **PR #282 MERGED (squash) after identity guard**: head verified == reviewed
   SHA d35d3883 pre-merge; 33/33 checks SUCCESS; mergeCommit
   **b5a70bd54d14f93b63463dd39df556ae5fabbc4c** on main. Merge exit 0.
4. Local commits on `backup/cto-t2-disk-dirty-20260913`:
   bcc313e6 (heavy tier + verdicts + census), 0c4cfa1b (round-1 record),
   **0d29dcbc** (corrected verdicts + P1 ticket + verified pools).

## OPEN AT SAVE TIME — next session picks up here

- **main CI on b5a70bd5 in flight** (9 workflows; 8 pending at 14:5x UTC:
  CI, Mobile, iOS Build & Test, Docker x2, Lint, Cross, Push on main;
  Repository Hygiene already SUCCESS). Verify with:
  `gh run list --commit b5a70bd54d14f93b63463dd39df556ae5fabbc4c`
  Gate: all SUCCESS (or pre-existing-failure analysis) before Phase 2 work.
- **Phase 2 (next): T4 routing feed** — `routing_peer_seen` has zero callers;
  implement on `ConnectionEstablished` in core/src (rule-8 gated: same
  double-blind pattern; the verified panel config in bod_governance.py is
  ready to reuse). Then T1 boot seed-dial, T2 peer-store unification.
- **Phase 4:** keystore/D2 verification at build time (verify, never trust).
- Worktrees: `tmp/wt-recovery-20260913` (now stale at pre-merge main — refresh
  to b5a70bd5 before implementing). Backup branch holds the parked dirty tree.
- Harness repo handoff file is UNCOMMITTED there (checkout is shared and on
  someone else's branch `fix/reasoning-model-hints-and-bom` with unrelated
  dirty files; left untouched per shared-checkout rules).
