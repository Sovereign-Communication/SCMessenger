# D1/D9 transport fixes -- harness adversarial findings dossier (2026-09-22)

**Reviewed artifacts.**
- D1: commit `7810845b` -- two-tier per-peer connection policy
  (`core/src/transport/behaviour.rs`, `per_peer_cap.rs`, `swarm.rs`, plus the
  test-only `cli/src/bin/conn-fanout.rs` harness).
- D9: uncommitted vendored patch -- `vendor/libp2p-swarm-0.48.0` with the
  event-routing `unreachable!()` sites converted to logged graceful drops,
  wired via `[patch.crates-io]` (diff taken against the pristine crates.io
  0.48.0 source in the cargo registry).

**Reviewer, and what this document is.** The reviewer of record is the
harness panel on the OpenClaw node (governed `harness-mcp` `panel_verify`,
free lane, `task_max_cost: 0.0`, driver `~/review/harness_sec_review.py`),
run by Buffy (Freebuff lane) at the operator's direction. Per
`docs/rules/SECURITY_PROTOCOL.md`, panels surface candidate findings and are
**not** the rule-8 gate, and an agent that authored a fix cannot sign it off:
Buffy authored the D9 patch and the D1 implementation, so this dossier is a
findings record plus author-side verification triage. **The rule-8 gate for
both changes remains OPEN** until an uninvolved reviewer signs (see
"Gate status").

**Provenance.** Ledger `~/.config/harness/ledger.jsonl` seq 236-261 (round A:
task_ids `sec-review-d1-7810845b-0922a`, `sec-review-d9-vendor-0922a`) and
seq 263-288 (round B: `...-0922b`), plus `sec-review-sort-0922a/-b`. Actual
cost $0.00; raw outputs on the node at `~/review/*-raw*.txt`.

**Panel health disclosure.** Both rounds ran during OpenRouter free-tier
saturation. Round A produced two substantive D1 panelists
(`nvidia/nemotron-3-super-120b-a12b:free`, 18.8 KB of analysis, hit the
output-length cap mid-reasoning; a second free-lane panelist returned a
structured 5-finding list) and one substantive D9 panelist
(`cohere/north-mini-code:free`: OVERALL pass, 2 informational). Round B was
degraded further (429s; the one panelist burned its budget on reasoning and
returned no visible content). No judge synthesis was possible either round;
the harness honestly recorded `defer: true` and kept every attempt. The
`issue_sort` severity pass fell back to unkeyed keyword matching and is
weightless; the severities below come from the panelist text plus author
triage, clearly separated.

## Findings -- D1 (two-tier per-peer connection policy)

| # | Claimed | Triage | Verdict |
|---|---|---|---|
| F1 | CRITICAL: trim logic passes `&mut Vec<ConnectionId>` where `&[ConnectionId]` is expected (swarm.rs ~6720) | A borrow conflict is a compile-time error; the crate compiles clean (`cargo clippy -p scmessenger-core` 0 warnings, 1457 lib tests pass) and the artifact is deployed and running on both nodes since 10:42Z. The panelist hallucinated an API mismatch it could not compile against. | **Refuted** |
| F2 | HIGH: `path_last_activity` may retain dead connection entries | Confirmed, narrower than claimed and LOW, not HIGH. Verified in source: the partial-close arm (`num_established > 0`, swarm.rs ~6705) prunes both the path and its activity entry, and the trim path removes the extras it closes; only the last-close arm (`num_established == 0`, ~6768) removes `peer_established_paths` without draining the final connection's activity entry. Impact is bounded memory growth -- one stale `Instant` per peer churn cycle. No correctness or security impact: libp2p `ConnectionId`s are monotonic and never reused, so a stale entry is never consulted again. | **Confirmed, LOW** (remedy: in the `== 0` arm, drain the ids out of `path_last_activity` when removing the peer entry). Deliberately NOT fixed in this pass so the diff under review stays stable; file as follow-up. |
| F3 | MEDIUM: admission ceiling 16 lets 4 peers transiently fill the node-wide incoming cap of 64 | Accepted as designed, with the trade-off already disclosed in SCM_NODES_AUDIT.md section 8. The retained bound of 4 is enforced within one event-loop pass of establishment (observed live: 8 trims per burst run, `retained=4 retained_bound=4`), so the 16/peer occupancy is transient; node-wide caps (64 in / 128 out / 32 pending) are unchanged from upstream and per-peer admission still binds. This is exactly the V040-T-CONN-04 behavior the operator approved to fix the 169-denial lockout. | **Accepted, disclosed** |
| F4 | LOW: oldest-first tie-break may close the wrong path on equal activity | Intended behavior, not a defect: on equal activity the oldest-established path closes first, which is what lets a fresh dial survive a peer holding stale paths -- the anti-lockout property this fix exists to provide. Covered by the regression test `fresh_dial_survives_a_peer_holding_stale_paths`. | **Disputed / working as intended** |
| F5 | (author-side, found during triage) MEDIUM-LOW: the retained-bound trim exists only in the NATIVE event loop; the WASM loop (swarm.rs ~9418/~9461) has no CONN-CAP bookkeeping | True asymmetry. WASM targets still cannot exceed the admission ceiling (16), so the cap binds; they just do not trim down to the retained bound of 4. Not exploitable; parity follow-up for the WASM loop. | **Confirmed, parity follow-up** |

**D1 net:** no REJECT-level finding. One real (minor, bounded) leak (F2), one
accepted disclosed trade-off (F3), one parity follow-up (F5). The live A/B
evidence stands: 9 denials on the old relay vs 0 denials / deterministic
trims on the fixed relay, and zero organic denials since the 05:49Z swap.

## Findings -- D9 (vendored libp2p-swarm graceful-degrade patch)

| # | Claimed | Triage | Verdict |
|---|---|---|---|
| F6 | INFORMATIONAL: panic-on-desync replaced with graceful degradation | That is the fix, working as intended. Regression tests prove the upgrade: with upstream's `unreachable!()` restored at the two live-panic sites, `on_behaviour_event_side_desync_must_not_panic` reproduces the panic; with the patch, all 3 tests pass and matched-side routing is byte-identical to upstream (`transpose_matched_sides_still_route`). | **Intended** |
| F7 | INFORMATIONAL: vendored `Cargo.toml` gains dev-deps + non-default `upstream-tests` feature; `Cargo.lock` gains 24 dev-only packages (criterion, quickcheck, trybuild, plotters, and friends) | Disclosed and correct: those entries exist only so the vendored crate's own test modules can run; none of them build into the release binaries (dev-dependencies do not affect `--release` artifacts). Supply-chain note recorded; audit of `Cargo.lock` per SECURITY_PROTOCOL.md is satisfied by this dossier's package list. | **Disclosed, benign** |

**D9 net:** the only substantive panelist returned `OVERALL: pass` ("hardens
against crashes from side-desync without weakening DoS posture"); no
candidate finding survived as a defect. Deployed on both nodes since
10:42-10:43Z with zero `panicked` events since.

## Gate status (rule 8) -- NOT CLEARED

What this dossier is: candidate findings from a governed multi-model panel,
plus author-side verification triage, on file.

What it is not, by explicit protocol: the sign-off. Two blockers remain,
both named by `docs/rules/SECURITY_PROTOCOL.md`:

1. Multi-model panels are not a substitute for the rule-8 gate on
   transport code, regardless of how the panel answers.
2. The author cannot self-sign (Buffy authored D1's implementation and the
   D9 patch).

Path to clear: an uninvolved reviewer -- per repo convention the
`crypto-security-auditor` subagent (native sessions), a read-only
fable(high) worker (`/scmorc`), or `deepseek-v3.2/v4-pro:cloud` (ollama
swarm) -- reviews both diffs against this dossier and produces the verdict
file in this directory. The reviewer should spend its attention on F2 (the
activity-map leak and its remedy), F3 (the accepted trade-off), and F5 (WASM
parity), and should re-verify the D9 byte-identical matched-side claim
directly from `vendor/libp2p-swarm-0.48.0/src/handler/either.rs`.

**Scope note:** both fixes are deployed to the relay and bridge nodes
(artifact `v040d9degrade-672dffcb-worktree`, sha256 `844eae88...`). A
verdict is scoped to what it reviewed: any further commit touching
`core/src/transport/` or the vendor tree voids the clean bills above for
those areas and requires re-review.
