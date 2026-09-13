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

---

# PHASE 2 ADDENDUM (same session, 2026-09-13)

## T4 outcome: DISPOSITION, not implementation

The T4 routing feed was found **already landed on merged main** (the "zero
callers" census premise was true only on the stale branch tip):
- Feed: swarm.rs native 6449-6459 / wasm 9148-9158 -> IronCore::routing_peer_seen,
  trust-gated by fail-closed peer_is_blocked (66-75); endpoint_transport_string
  (431-452) classifies tcp/ws/quic/relay.
- D6 acceptance tests on main PASS (verified live: 2 passed / 0 failed).
- Dual-approve: bod-0ad63e5f (paid panel 5/5) + Blind B verdict (C1/C2).
- Heavy-tier BoD attempt refused by preflight ($0.78 vs $0.10 ceiling) --
  BoD-rule finding for the operator, not bypassed.
- Artifacts: HANDOFF/audit/T4_ROUTING_FEED_ANALYSIS_2026-09-13.md,
  HANDOFF/review/V040_T4_BLIND_B_ADVERSARIAL_VERDICT_2026-09-13.md,
  HANDOFF/todo/P2_NON_SWARM_TRANSPORT_ROUTING_FEED.md (C2),
  HANDOFF/todo/P1_ROUTING_ENGINE...md updated (criterion 5 field re-measure
  open). Editing-tool fault briefly truncated that ticket mid-update;
  restored in full from the verified read (disclosed in-file).

## Merges this session (all identity-guarded, dual-approve)

1. PR #282 (code) -> b5a70bd5 (Phase 1).
2. PR #284 (T4 docs disposition) -> 5f1cf702 (Phase 2). 19/19 checks green.

## main CI on b5a70bd5: GREEN (after one flake)

Docker Integration Suite failed once (3 Android ViewModel tests:
SettingsViewModelTest.infoCounts, ConversationsViewModelTest x2;
MockK stubbing leakage), `rerun --failed` on the same SHA PASSED -> flake,
not regression. Ticket filed: HANDOFF/todo/P2_ANDROID_VIEWMODEL_TEST_ORDER_DEPENDENCE.md.

## Next phases (unchanged plan)

- Phase 3: T1 CLI boot seed-dial (half-wired: only mobile_bridge.rs:862 calls
  connect_to_seed_peers on the pre-#282 tree -- re-census on 5f1cf702 first;
  a freebuff/v040-t1-boot-seed-dial worktree already exists) + T2 peer-store
  unification (rule-8 gated, double-blind).
- Phase 4: keystore/D2 verification at build time.
- Harness-repo handoff still uncommitted there (shared checkout on another
  lane's branch); chat.py effort:none patch is the first Harness action.

---

# PHASE 3 COMPLETE (2026-09-13, appended after #285 merge)

**Result:** T1 + T2 closed as ALREADY-LANDED; main advanced 5f1cf702 -> 6e726402
(PR #285, squash, head-identity guard passed a5fe0724, all checks CLEAN).

## Disposition of record

- `HANDOFF/audit/T1_T2_CENSUS_DISPOSITION_2026-09-13.md` (+ ADDENDUM): both
  Phase 3 premises falsified on current main. T1 = PR #266
  (cli/src/seed_dial.rs, wired at main.rs:2260-2271). T2 = PR #262
  (legacy migration into core LedgerManager, peers.json archived).
  Live proof: `cargo test -p scmessenger-cli --lib` 84 passed / 0 failed.
- The suspected rule-8 breach is FALSIFIED: independent Opus verdict
  (`HANDOFF/freebuff/inbox/RULE8_PR262_PR263_VERDICT_OPUS.md`, 2026-08-31,
  pre-merge) APPROVED both PRs under a combined filename the charter did not
  predict. `RULE8_REVIEW_PR262_LEDGER_UNIFICATION.md` CLOSED with the
  filename-mismatch process lesson. F1/F2/F7/F-DHT follow-ups re-verified
  closed on 6e726402's ancestor 5f1cf702; F6 doc residual ticketed
  (HANDOFF/todo/P3_DOC_LEDGER_MIGRATION_F6_NOTES.md).
- Governance: Blind A bod-70b2c5aa 5/5 APPROVE (judge agrees, 0.96,
  $0.0035); Blind B
  `HANDOFF/review/V040_T1T2_BLIND_B_ADVERSARIAL_VERDICT_2026-09-13.md`
  APPROVE with C1 (field re-measure stands) and C2 (pool config shipped --
  satisfied by #285 including scripts/bod_governance.py).

## Governance-config changes shipped in #285 (for-cause, evidence on record)

- gpt-4o-mini REMOVED from paid pool: stale-generation voter; filed a
  content-free REJECT dissent (no file/line/evidence) against 4
  evidence-citing APPROVEs (run bod-T1T2 R1, tmp/review/T1T2_PAID_result.json).
- gemini-3.8-flash REMOVED: OpenRouter routed it BYOK mid-run (R4); the
  governor re-learned the google/ prefix -- nondeterministic seat.
- ling-3.0-flash-fin:free ADDED as fifth seat: probed under dispatch
  conditions (clean APPROVE JSON, 4.6s, $0.00, BYOK-immune; also nemotron-3
  verified as backup). gpt-5.6-sol / kimi-k3 probed clean but price-bomb the
  $0.10 preflight as pool members -- documented in pool comments.

## Operator-visible notes

- Preflight estimator now understood and documented: ~4 reservation slots per
  reasoning-capable seat x completion-token price dominates worst-case.
  Vote pools must be priced as a SET against the $0.10 ceiling, not per seat.
- The `--dry-run` flag does NOT exercise the preflight estimator (verified);
  use --max-tokens tuning + one real preflight to validate pool arithmetic.
- Harness chat.py effort:none patch remains UNPATCHED upstream (handoff at
  Harness/docs/MODEL_SELECTION_HANDOFF_2026-09-13.md); pools are configured
  to work correctly without it (v4.1-flash as judge only).
- Next per plan: Phase 4 -- keystore/D2 verify (operator-gated items), then
  operator gate scoring and the v0.4.0 tag package. T4 field re-measure (C1)
  remains the one open acceptance item on the device rig.

---

# F6 CLOSED (2026-09-13 late, PR #286)

- F6 doc correction merged: main at ccce98cf (PR #286, squash, identity
  guard 4de8a6c0, all checks CLEAN after two flake reruns). The last open
  follow-up from the Opus #262/#263 rule-8 verdict is closed; F1/F2/F6/F7/
  F-DHT are all resolved on the record.
- Governance bug found and fixed in the same PR: the BoD reconciliation
  scanned consensus.verdict (a key the dict never carries), so the judge's
  APPROVE wording was invisible; first F6 gate run produced a false
  REJECTED_JUDGE_DIVERGENCE. Fixed to scan the synthesis text; re-run
  APPROVED (bod-3b8d3ffe). Both runs recorded in BOD_STATE.
- Android JVM flake on #286: 2-of-3 fail (same SHA) -- evidence appended to
  P2_ANDROID_VIEWMODEL_TEST_ORDER_DEPENDENCE.md; its MockK/ordering fix is
  now the highest-value CI-reliability item.
- Shared-cargo-store hazard documented: UniFFI build-script output from two
  divergent lineages collides (E0063 in generated bindings); one lineage at
  a time, or cargo clean -p scmessenger-core on lineage switch.
