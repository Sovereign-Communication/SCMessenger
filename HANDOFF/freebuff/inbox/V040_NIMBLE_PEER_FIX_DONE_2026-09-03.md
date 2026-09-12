Task: V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md (validation feedback loop)
Type: DONE

## Nimble-peer fix landed: PR #273 (f50ac0f9) -- 5-minute dead-mark cycle killed

CEO direction 2026-09-03 ("AWS and windows should have an established session/link, so not dialing in,
but responding... the app needs to be MUCH more nimble. Get this fixed.") -- implemented, gated, pushed,
PR opened.

### Root cause (established from logs, not theory)
Windows and AWS each dead-marked the other every ~5 min while identify confirmed liveness over the
persistent connection every 60s. Dial backoff was only cleared by ConnectionEstablished (never again on
a stable link), so secondary-address dial failures accumulated to a 3-strike dead mark that suppressed
hint-dials and relay custody pulls -- the 75-item AWS custody stall.

### Fix (3 files, +193/-19, branch freebuff/v040-nimble-peer, base 177bd840)
1. swarm.rs: IdentifyReceived (native) -> reset_peer_backoff(peer_id) -- liveness clears every addr-key
   entry for the peer, not just the connected address.
2. swarm.rs: OutgoingConnectionError + pending-dial sweep skip dead escalation while the peer has any
   live connection (address-scoped failure).
3. dial_policy.rs: dead is bounded -- DEAD_REVIVE_AFTER 60s, auto-revive via maybe_revive in
   register_dial_attempt; +2 new tests.
4. resume_prefetch.rs test fix: test_frequent_peer_decay panicked on this host (Instant boot-anchor
   younger than 2h -- "overflow when subtracting duration from instant"); strict decay asserted when
   the anchor permits, increment path otherwise. Blocks the suite on any fresh-boot host.

### Gates (Windows host, exact commands)
- cargo check -p scmessenger-core --all-targets -> clean (2 pre-existing warnings)
- cargo test -p scmessenger-core --lib -> 1402 passed / 0 failed
- cargo test -p scmessenger-cli --lib -> 82 passed / 0 failed
- cargo check -p scmessenger-wasm --target wasm32-unknown-unknown -> PASS
- cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments -> clean
- cargo fmt --check -> clean
- PR: https://github.com/Sovereign-Communication/SCMessenger/pull/273 (base cto/v040-candidate-2026-09-02)

### Gate state (updated after dispatch completed)
- **Rule-8: CLOSED -- PLAIN APPROVE on file.** R1 (REQUEST_CHANGES, A1-A4) -> dispositions verified
  against the tree (A1 partially refuted/documented, A2 refuted with grep evidence: exactly 2 guarded
  sinks, zero unguarded callers, A3/A4 accepted as doc + WARN-marker hardening) -> round-2 commit
  d82978ab -> R2 `Verdict: APPROVE`. Verdict file:
  HANDOFF/review/V040_NIMBLE_PEER_REVIEW_QWEN_2026-09-03.md (qwen3.8-2.4t-a95b, non-author, 1M context).
  Full gate set re-verified on the final head d82978ab: core 1402/0, cli 82/0, wasm32 PASS, clippy
  documented gate clean, fmt clean.
- CEO decision: merge #273 (d82978ab) into the candidate branch (PR #272's line) as part of the
  post-validation merge sequence, then the candidate -> main. Not self-merged.
- The transport-level 5-minute connection recycle that precedes each dead-mark ("Failure while closing
  connection" at :50, anchored to the AWS container boot) is NOT fixed here -- with this change it is
  harmless (no dead escalation, liveness resets within 60s). Separate investigation if desired.

## Three-node validation closeout (same-SHA fleet, 177bd840)

Nodes (all verified on 177bd840 content): Windows CLI 12D3KooWD6vZQ @ 147.81.41.188 (exe provenance
35c49318); AWS 12D3KooW9uRMQ @ 54.235.20.24:9000/9001 (container 3020c5ea, image digest
sha256:4518385a..., redeployed 07:25:50Z, data dir + identity preserved); Pixel 12D3KooWKdek3
(APK sha256 862c12da, app restarted 07:32Z).

| Gate | Mark | Evidence |
|---|---|---|
| 1. Seed dialing | PASS | aws-177bd840-startup.log (AWS boot 07:25Z), seed sweep in runbook |
| 2. External-address filtering | PASS | log evidence all 3 nodes (listen-port admission, TCP allowlist) |
| 3. Ledger/peer propagation | PASS | ledger exchange + relay circuit OK post-redeploy (AWS<->Windows) |
| 4. Inbound reachability + delivery | PASS | 3NODE-TEST-2026-09-03-WIFI-BLE-CELL.md; delivery bursts + receipts 07:44:58Z |
| 5. Coordinated restart re-mesh | PASS | AWS 07:25:50 + Pixel 07:32Z re-mesh timeline |
| 6. AWS IP churn | UNVERIFIED | genuinely unexercised (dynamic IP, zero elastic IPs -- IAM evidence); NOT run without CEO approval; available pre-tag action |

Evidence files: tmp/run-evidence/{3NODE-TEST-2026-09-03-WIFI-BLE-CELL.md,
FINDINGS-2026-09-03-0738Z.md, AWS-WIN-LINK-ANALYSIS-2026-09-03-0800Z.md,
aws-redeploy-177bd840-evidence.md, pixel-177bd840-restart-evidence.md}; AWS node logs
aws-2026-09-03-06_07.log + 07-hour on-node logs.

Post-validation findings on 177bd840: retry-storm recovery (resolved), ae277205 stale-recipient
pending message (needs terminal-failure policy), cellular-only bootstrap gap (product decision),
AWS<->Windows dead-mark cycle (NOW FIXED -- PR #273, this note).

## Single next decision needed from the CEO seat

Approve the dependency-ordered post-validation merge sequence (V040_POST_VALIDATION_MERGE_PLAN
/ V040_MERGE_APPROVAL_RECORDS_2026-09-03.md, FLAG-1/FLAG-2 calls), now including PR #273
(d82978ab, Rule-8 APPROVE on file) merged into the candidate branch cto/v040-candidate-2026-09-02
before the candidate -> main. Also decide: ae277205 terminal-failure policy, cellular-only
bootstrap path, and whether to ticket the transport-level 5-minute connection recycle (harmless
post-#273, source still unidentified).
