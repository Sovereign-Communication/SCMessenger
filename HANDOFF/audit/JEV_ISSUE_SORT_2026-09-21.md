# JEV repo insight report — SCMessenger

Generated: 2026-09-21T07:30:45.335209+00:00
Main tip (origin): `9d37f9e64162b5d700e74356c6c6a287af2914df`
Harness: `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\wt-local-harness-20260921\vendor\sovereign-harness`
JEV keyed: True model=jev-latest

## Harvest (code-owned)

- Open PR count (approx from gh json keys): 29
- HANDOFF/todo files scanned: 43
- Freebuff queue files: 50
- Implementation plan present: True
- Master plan present: True
- Keyword hits (plans/readme):
  - `routing_peer_seen`: 7
  - `public-key hex`: 2
  - `wp1`: 15
  - `wp2`: 13
  - `wp3`: 9
  - `wp4`: 12
  - `wp5`: 19
  - `rule-8`: 15
  - `jev`: 64
  - `harness`: 40
  - `connection_limits`: 1
  - `keystore`: 9
  - `wifi`: 29
- Commands: git log origin/main -15 --oneline; gh pr list --state open; scan HANDOFF/todo + freebuff/queue status lines

## Git tip log

```text
9d37f9e6 docs(jev): freebuff WP DONE requires keyed JEV is_passing (#345)
c8c3fdc0 docs(freebuff): land 2026-09-19 3-node baseline inbox evidence (#346)
9de879b9 fix(android): compile instrumented tests (Mobile lane) (#341)
ef5f8150 feat(jev): batched SCMessenger repo insight packs + harness integration (#344)
6719f130 docs(cto): WiFi/identity implementation plan + JEV canonical gates (#343)
95b81b5b docs(cto): fleet status on 51edac4b after Android fresh install (#342)
51edac4b fix(core): dual-drain outbox flush (CO-B-001) (#339)
dc481af3 docs(cto): 3-node log analysis 2026-09-20 (#340)
e7b42317 docs(cto): v0.4.0 master plan, audit dispositions, freebuff paste order (#338)
1caee28c chore(handoff): rebase glm audit lineage onto main (non-docs package) (#337)
d7f169c1 docs(audit): canonical outlier inventory package (docs-first split)
224aaa8f docs(audit): set docs landing PR URL in DOCS_SPLIT note
72c9644d docs(audit): record docs landing PR 336 in INDEX and tracking notes
ce14579c docs(audit): land canonical outlier package on main-based branch
882e95db ci(mobile): allow workflow_dispatch so APK jobs can run without a path-touching push (#334)
```

## JEV batch totals

- Packs run: 0
- Total input tokens: 0
- Total cost (USD): 0

## Issue-sort (Harness JevPolicy.evaluate_issue_sort)

- pack: `scmessenger_v040_completion` cost=0.0
- harness: `C:\Users\SCM\Documents\GitHub\SCMessenger\tmp\wt-local-harness-20260921\vendor\sovereign-harness`
- bucket_counts: `{"process_dispatch": 14, "device_release": 2, "unmatched": 21, "unification_docs": 3, "identity_transport": 5, "delivery_truth": 5}`

- `_QUEUE.md` -> bucket=`process_dispatch` attention=`P1` path=`HANDOFF/freebuff/README.md` fallback=True next=`check_queue_status then paste DISPATCHABLE only`
- `ANDROID_CI_APK_SIGNATURE_BLOCKS_INPLACE_UPGRADE_2026-08-09.md` -> bucket=`device_release` attention=`P1` path=`HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md` fallback=True next=`operator secrets or androidTest PR train`
- `ANDROID_FFI_IN_COMPOSITION_BUILD_KILLER_2026-09-10.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `ANDROID_INBOUND_CRYPTOERROR_2026-08-09.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `CELL_ROUTE_AWS_001_2026-09-11.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `CO_A005_DOCTRINE_COPY_PLAN_2026-09-20.md` -> bucket=`unification_docs` attention=`P2` path=`HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` fallback=True next=`update implementation plan + README index`
- `CODEBASE_UNIFICATION_PLAN.md` -> bucket=`unification_docs` attention=`P2` path=`HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` fallback=True next=`update implementation plan + README index`
- `CORE_DIAL_CANDIDATE_DOUBLE_CIRCUIT_PRUNE_2026-09-10.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `D4_MOBILE_BRIDGE_HISTORY_FLAVOR_MATCHING_2026-08-30.md` -> bucket=`device_release` attention=`P1` path=`HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md` fallback=True next=`operator secrets or androidTest PR train`
- `DEPENDENCY_DEBT_TOOLCHAIN_UPGRADE_2026-08-28.md` -> bucket=`unification_docs` attention=`P2` path=`HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` fallback=True next=`update implementation plan + README index`
- `GHOST_LEDGER_PRUNE_G1_2026-09-11.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `P0_ANDROID_FINITE_RETRY_ABANDONMENT_2026-08-10.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `P0_ANDROID_SELF_RATCHET_RESET_2026-08-10.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md` -> bucket=`identity_transport` attention=`P0` path=`HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md` fallback=True next=`paste V050_WP1 then WP2 (Rule-8)`
- `P0_SEND_CRYPTO_FAILS_VS_DELIVERED_2026-08-30.md` -> bucket=`delivery_truth` attention=`P0` path=`HANDOFF/freebuff/queue/V050_WP4_DELIVERY_TRUTH_2026-09-21.md` fallback=True next=`paste V050_WP4 + watchdog test`
- `P1_ANDROID_CHAT_ORDER_CROSS_CLOCK_2026-09-17.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `P1_ANDROID_COLDSTART_NOTIFICATION_AND_SCAFFOLD_2026-09-16.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `P1_ANDROID_COMPOSE_CRASH_RECURRENCE_2026-09-15.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_2026-08-10.md` -> bucket=`delivery_truth` attention=`P0` path=`HANDOFF/freebuff/queue/V050_WP4_DELIVERY_TRUTH_2026-09-21.md` fallback=True next=`paste V050_WP4 + watchdog test`
- `P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_LIVE_RCA_2026-08-25.md` -> bucket=`delivery_truth` attention=`P0` path=`HANDOFF/freebuff/queue/V050_WP4_DELIVERY_TRUTH_2026-09-21.md` fallback=True next=`paste V050_WP4 + watchdog test`
- `P1_CLI_OUTBOX_CANONICAL_DRAIN_AND_SLED_UNIFICATION_2026-09-16.md` -> bucket=`delivery_truth` attention=`P0` path=`HANDOFF/freebuff/queue/V050_WP4_DELIVERY_TRUTH_2026-09-21.md` fallback=True next=`paste V050_WP4 + watchdog test`
- `P1_CLI_SEND_CANONICAL_IDENTIFIER_PARITY_2026-09-16.md` -> bucket=`identity_transport` attention=`P0` path=`HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md` fallback=True next=`paste V050_WP1 then WP2 (Rule-8)`
- `P1_CONTACT_RECOVERY_WRITES_PEERID_AS_PUBLIC_KEY_2026-08-10.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `P1_CORE_IDENTITY_SPOOF_AND_WASM_TOPIC_PARITY_2026-09-16.md` -> bucket=`identity_transport` attention=`P0` path=`HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md` fallback=True next=`paste V050_WP1 then WP2 (Rule-8)`
- `AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md` -> bucket=`identity_transport` attention=`P0` path=`HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md` fallback=True next=`paste V050_WP1 then WP2 (Rule-8)`
- `V040_3NODE_VALIDATION_RUNBOOK_2026-09-03.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `V040_ARCH272_ROUTING_FEED_FINDING_2026-09-03.md` -> bucket=`process_dispatch` attention=`P1` path=`HANDOFF/freebuff/README.md` fallback=True next=`check_queue_status then paste DISPATCHABLE only`
- `V040_BEACH_JOIN_CONTINUATION_2026-09-05.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `V040_BEACH_JOIN_PHASE0_1_2026-09-20.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `V040_CEO_CLEANUP_MERGE_DIRECTIVE_2026-09-03.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `V040_CTO_HANDOFF_2026-09-01.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `V040_CTO_PROMPT_2026-09-02.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `V040_CTO_TRACKING_PROTOCOL.md` -> bucket=`None` attention=`None` path=`None` fallback=True next=`None`
- `V040_QUEUE_STATUS_RECONCILE_TAGPATH.md` -> bucket=`process_dispatch` attention=`P1` path=`HANDOFF/freebuff/README.md` fallback=True next=`check_queue_status then paste DISPATCHABLE only`
- `V040_REVIEW_DISPATCH_267_272_FINAL_APPROVE_QWEN_2026-09-03.md` -> bucket=`process_dispatch` attention=`P1` path=`HANDOFF/freebuff/README.md` fallback=True next=`check_queue_status then paste DISPATCHABLE only`
- `V040_REVIEW_DISPATCH_267_FDHT_QWEN_2026-09-01.md` -> bucket=`process_dispatch` attention=`P1` path=`HANDOFF/freebuff/README.md` fallback=True next=`check_queue_status then paste DISPATCHABLE only`
- `V040_REVIEW_DISPATCH_267_RECHECK_QWEN_2026-09-02.md` -> bucket=`process_dispatch` attention=`P1` path=`HANDOFF/freebuff/README.md` fallback=True next=`check_queue_status then paste DISPATCHABLE only`
- `V040_REVIEW_DISPATCH_267_RECHECK_QWEN_2026-09-03.md` -> bucket=`process_dispatch` attention=`P1` path=`HANDOFF/freebuff/README.md` fallback=True next=`check_queue_status then paste DISPATCHABLE only`

## Insights for SCMessenger completion (orchestrator synthesis)

Mechanical harvest + JEV batches above are inputs. Final action orders stay in `HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` and `HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md`. JEV does not invent new root causes.

### Standing synthesis themes

1. Identity unification (hex) remains the first WP — dual flavor breaks send paths.
2. Routing feed must be one entry for all data links — WP2 is the open hole.
3. Delivery truth (ACK vs delivered vs receipts) is process + code — WP4.
4. WP5 live 3-node logs are mandatory before any WiFi-fixed claim.
5. Freebuff paste authority is the implementation plan DISPATCHABLE set + JEV canonical DONE.
6. Harness WIP (P2 repair / jev-phase / issue-sort) stays in Harness repo; SCMessenger consumes origin/main JEV + local packs.
