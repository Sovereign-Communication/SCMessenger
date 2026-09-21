# JEV repo insight report — SCMessenger

Generated: 2026-09-21T02:28:03.354335+00:00
Main tip (origin): `6719f130f023f209c4616f089a1a165fcec6a9da`
Harness: `C:\Users\SCM\Documents\GitHub\Harness-jev-use`
JEV keyed: True model=jev-latest

## Harvest (code-owned)

- Open PR count (approx from gh json keys): 30
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
  - `jev`: 51
  - `harness`: 36
  - `connection_limits`: 1
  - `keystore`: 9
  - `wifi`: 29
- Commands: git log origin/main -15 --oneline; gh pr list --state open; scan HANDOFF/todo + freebuff/queue status lines

## Git tip log

```text
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
3148d38b fix(ci): composite action must not reference secrets context in action.yml (#333)
f9cb6434 docs(v040): SEC-03 storage migration safe-ahead brief (#332)
34b56d54 fix(core): canonicalize outbox queue keys at one owner (#322)
42bce3f3 fix(android,core): a same-second reply can no longer sort above its trigger (#325)
```

## JEV batch totals

- Packs run: 4
- Total input tokens: 18266
- Total cost (USD): 0.767172

## Pack: `pain_points`

batches=14 tokens=11574 cost=0.486108

### Batch 0 [LIVE] verdict=fail supported=0.41 conf=0.31
- items: ANDROID_CI_APK_SIGNATURE_BLOCKS_INPLACE_UPGRADE_2026-08-09.md, ANDROID_FFI_IN_COMPOSITION_BUILD_KILLER_2026-09-10.md, CELL_ROUTE_AWS_001_2026-09-11.md, CORE_DIAL_CANDIDATE_DOUBLE_CIRCUIT_PRUNE_2026-09-10.md
- tokens=825 cost=0.03465 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "device_release", "probabilities": {"device_release": 0.65, "identity_transport": 0.08, "process_dispatch": 0.18, "ci_queue_gates": 0.02, "docs_truth": 0.07}, "confidence": 0.56}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.41}, "severity": {"type": "score", "score": 1.54, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.02, "1": 0.42, "2": 0.56}, "confidence": 0.31}}`
- reason: dominant_pain (choice): device_release (conf: 0.56)
- reason: blocks_tag_or_working_mesh (noul): 0.41
- reason: severity (score): 1.54 (conf: 0.31)

### Batch 1 [LIVE] verdict=fail supported=0.34 conf=0.58
- items: D4_MOBILE_BRIDGE_HISTORY_FLAVOR_MATCHING_2026-08-30.md, DEPENDENCY_DEBT_TOOLCHAIN_UPGRADE_2026-08-28.md, GHOST_LEDGER_PRUNE_G1_2026-09-11.md, P0_ANDROID_FINITE_RETRY_ABANDONMENT_2026-08-10.md
- tokens=840 cost=0.03528 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "process_dispatch", "probabilities": {"process_dispatch": 0.66, "device_release": 0.14, "identity_transport": 0.01, "docs_truth": 0.18, "ci_queue_gates": 0.01}, "confidence": 0.58}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.34}, "severity": {"type": "score", "score": 1.75, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.01, "1": 0.24, "2": 0.75}, "confidence": 0.62}}`
- reason: dominant_pain (choice): process_dispatch (conf: 0.58)
- reason: blocks_tag_or_working_mesh (noul): 0.34
- reason: severity (score): 1.75 (conf: 0.62)

### Batch 2 [LIVE] verdict=fail supported=0.58 conf=0.87
- items: P0_ANDROID_SELF_RATCHET_RESET_2026-08-10.md, P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md, P0_SEND_CRYPTO_FAILS_VS_DELIVERED_2026-08-30.md, P1_ANDROID_CHAT_ORDER_CROSS_CLOCK_2026-09-17.md
- tokens=810 cost=0.03402 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "identity_transport", "probabilities": {"process_dispatch": 0.09, "ci_queue_gates": 0.0, "docs_truth": 0.01, "device_release": 0.0, "identity_transport": 0.9}, "confidence": 0.87}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.58}, "severity": {"type": "score", "score": 1.95, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.0, "1": 0.04, "2": 0.96}, "confidence": 0.93}}`
- reason: dominant_pain (choice): identity_transport (conf: 0.87)
- reason: blocks_tag_or_working_mesh (noul): 0.58
- reason: severity (score): 1.95 (conf: 0.93)

### Batch 3 [LIVE] verdict=fail supported=0.57 conf=0.26
- items: P1_ANDROID_COLDSTART_NOTIFICATION_AND_SCAFFOLD_2026-09-16.md, P1_ANDROID_COMPOSE_CRASH_RECURRENCE_2026-09-15.md, P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION.md, P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_2026-08-10.md
- tokens=776 cost=0.032592 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "process_dispatch", "probabilities": {"ci_queue_gates": 0.03, "process_dispatch": 0.39999999999999997, "identity_transport": 0.16, "device_release": 0.16, "docs_truth": 0.25}, "confidence": 0.26}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.57}, "severity": {"type": "score", "score": 1.93, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.01, "1": 0.06, "2": 0.93}, "confidence": 0.89}}`
- reason: dominant_pain (choice): process_dispatch (conf: 0.26)
- reason: blocks_tag_or_working_mesh (noul): 0.57
- reason: severity (score): 1.93 (conf: 0.89)

### Batch 4 [LIVE] verdict=fail supported=0.64 conf=0.66
- items: P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_LIVE_RCA_2026-08-25.md, P1_CLI_OUTBOX_CANONICAL_DRAIN_AND_SLED_UNIFICATION_2026-09-16.md, P1_CLI_SEND_CANONICAL_IDENTIFIER_PARITY_2026-09-16.md, P1_CONTACT_RECOVERY_WRITES_PEERID_AS_PUBLIC_KEY_2026-08-10.md
- tokens=832 cost=0.034944 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "identity_transport", "probabilities": {"process_dispatch": 0.25, "identity_transport": 0.72, "ci_queue_gates": 0.01, "device_release": 0.01, "docs_truth": 0.01}, "confidence": 0.66}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.64}, "severity": {"type": "score", "score": 1.95, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.0, "1": 0.05, "2": 0.95}, "confidence": 0.92}}`
- reason: dominant_pain (choice): identity_transport (conf: 0.66)
- reason: blocks_tag_or_working_mesh (noul): 0.64
- reason: severity (score): 1.95 (conf: 0.92)

### Batch 5 [LIVE] verdict=fail supported=0.65 conf=0.35
- items: P1_CORE_IDENTITY_SPOOF_AND_WASM_TOPIC_PARITY_2026-09-16.md, P1_DOCKER_CONTROL_API_SECURITY_HARDENING_2026-09-16.md, P1_GHOST_GUARD_OWN_TOPIC_MESSAGE_LOSS_2026-09-16.md, P1_RELEASE_SIGNING_GATE_FAIL_CLOSED_2026-09-16.md
- tokens=765 cost=0.03213 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "identity_transport", "probabilities": {"process_dispatch": 0.09, "identity_transport": 0.48, "device_release": 0.24, "docs_truth": 0.01, "ci_queue_gates": 0.18}, "confidence": 0.35}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.65}, "severity": {"type": "score", "score": 1.99, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.0, "1": 0.0, "2": 1.0}, "confidence": 0.99}}`
- reason: dominant_pain (choice): identity_transport (conf: 0.35)
- reason: blocks_tag_or_working_mesh (noul): 0.65
- reason: severity (score): 1.99 (conf: 0.99)

### Batch 6 [LIVE] verdict=fail supported=0.66 conf=0.51
- items: P1_ROUTING_ENGINE_NEVER_LEARNS_PEERS_2026-08-10.md, P1_SECURITY_RUSTLS_RUSTSEC_2026_0285_TRACKING.md, P1_SWARM_CHANNEL_BACKPRESSURE_DEADLOCK_2026-09-16.md, P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md
- tokens=745 cost=0.03129 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "process_dispatch", "probabilities": {"docs_truth": 0.06, "identity_transport": 0.33, "process_dispatch": 0.61, "device_release": 0.0, "ci_queue_gates": 0.0}, "confidence": 0.51}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.66}, "severity": {"type": "score", "score": 1.99, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.0, "1": 0.01, "2": 0.99}, "confidence": 0.98}}`
- reason: dominant_pain (choice): process_dispatch (conf: 0.51)
- reason: blocks_tag_or_working_mesh (noul): 0.66
- reason: severity (score): 1.99 (conf: 0.98)

### Batch 7 [LIVE] verdict=fail supported=0.32 conf=0.46
- items: RECEIPT_MARKER_ID_FLAVOR_MISMATCH_2026-08-09.md, V040_ARCH272_ROUTING_FEED_FINDING_2026-09-03.md, V040_BEACH_JOIN_CONTINUATION_2026-09-05.md, V040_BEACH_JOIN_PHASE0_1_2026-09-20.md
- tokens=819 cost=0.034398 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "identity_transport", "probabilities": {"process_dispatch": 0.27, "docs_truth": 0.12, "ci_queue_gates": 0.03, "identity_transport": 0.58, "device_release": 0.0}, "confidence": 0.46}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.32}, "severity": {"type": "score", "score": 1.02, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.13, "1": 0.73, "2": 0.14}, "confidence": 0.6}}`
- reason: dominant_pain (choice): identity_transport (conf: 0.46)
- reason: blocks_tag_or_working_mesh (noul): 0.32
- reason: severity (score): 1.02 (conf: 0.6)

### Batch 8 [LIVE] verdict=fail supported=0.26 conf=0.46
- items: V040_QUEUE_STATUS_RECONCILE_TAGPATH.md, V040_REVIEW_DISPATCH_267_272_FINAL_APPROVE_QWEN_2026-09-03.md, V040_REVIEW_DISPATCH_267_FDHT_QWEN_2026-09-01.md, V040_REVIEW_DISPATCH_267_RECHECK_QWEN_2026-09-02.md
- tokens=887 cost=0.037254 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "process_dispatch", "probabilities": {"ci_queue_gates": 0.27, "device_release": 0.0, "identity_transport": 0.0, "docs_truth": 0.01, "process_dispatch": 0.72}, "confidence": 0.65}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.26}, "severity": {"type": "score", "score": 0.93, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.22, "1": 0.64, "2": 0.14}, "confidence": 0.46}}`
- reason: dominant_pain (choice): process_dispatch (conf: 0.65)
- reason: blocks_tag_or_working_mesh (noul): 0.26
- reason: severity (score): 0.93 (conf: 0.46)

### Batch 9 [LIVE] verdict=fail supported=0.29 conf=0.6
- items: V040_REVIEW_DISPATCH_267_RECHECK_QWEN_2026-09-03.md, V040_REVIEW_DISPATCH_268_269_QWEN_2026-09-01.md, V040_REVIEW_DISPATCH_268_270_CONFIRM_APPROVE_QWEN_2026-09-03.md, V040_REVIEW_DISPATCH_270_EPHEMERAL_QWEN_2026-09-01.md
- tokens=951 cost=0.039942 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "process_dispatch", "probabilities": {"process_dispatch": 0.98, "identity_transport": 0.0, "device_release": 0.0, "ci_queue_gates": 0.02, "docs_truth": 0.0}, "confidence": 0.96}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.29}, "severity": {"type": "score", "score": 0.9, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.18, "1": 0.73, "2": 0.09}, "confidence": 0.6}}`
- reason: dominant_pain (choice): process_dispatch (conf: 0.96)
- reason: blocks_tag_or_working_mesh (noul): 0.29
- reason: severity (score): 0.9 (conf: 0.6)

### Batch 10 [LIVE] verdict=fail supported=0.28 conf=0.69
- items: V040_REVIEW_DISPATCH_272_ARCH_QWEN_2026-09-03.md, V040_REVIEW_DISPATCH_272_CANDIDATE_QWEN_2026-09-02.md, V040_REVIEW_DISPATCH_272_RECHECK_QWEN_2026-09-02.md, V040_REVIEW_DISPATCH_273_NIMBLE_PEER_QWEN_2026-09-03.md
- tokens=938 cost=0.039396 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "process_dispatch", "probabilities": {"device_release": 0.0, "docs_truth": 0.01, "process_dispatch": 0.85, "ci_queue_gates": 0.14, "identity_transport": 0.0}, "confidence": 0.81}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.28}, "severity": {"type": "score", "score": 0.87, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.17, "1": 0.79, "2": 0.04}, "confidence": 0.69}}`
- reason: dominant_pain (choice): process_dispatch (conf: 0.81)
- reason: blocks_tag_or_working_mesh (noul): 0.28
- reason: severity (score): 0.87 (conf: 0.69)

### Batch 11 [LIVE] verdict=fail supported=0.21 conf=0.39
- items: V040_REVIEW_DISPATCH_276_OUTBOX_FIX_QWEN_2026-09-04.md, V040_T13_RULE8_FOLLOWUPS_262_263.md, V040_T1_NODE_BOOT_SEED_DIAL.md, V040_T2_UNIFY_PEER_LEDGER_STORES.md
- tokens=862 cost=0.036204 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "process_dispatch", "probabilities": {"process_dispatch": 0.74, "docs_truth": 0.04, "identity_transport": 0.0, "device_release": 0.0, "ci_queue_gates": 0.22}, "confidence": 0.67}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.21}, "severity": {"type": "score", "score": 0.4, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.63, "1": 0.33, "2": 0.04}, "confidence": 0.39}}`
- reason: dominant_pain (choice): process_dispatch (conf: 0.67)
- reason: blocks_tag_or_working_mesh (noul): 0.21
- reason: severity (score): 0.4 (conf: 0.39)

### Batch 12 [LIVE] verdict=fail supported=0.44 conf=0.18
- items: V040_T4_ROUTING_FEED_ON_CONNECTION_ESTABLISHED.md, V040_T9_PR_QUEUE_BURNDOWN.md, V040_T_AND06_KOTLIN_COLLAPSE.md, V040_T_AND06_UNIFI_CUTOVER.md
- tokens=794 cost=0.033348 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "docs_truth", "probabilities": {"process_dispatch": 0.31, "device_release": 0.01, "identity_transport": 0.06, "docs_truth": 0.34, "ci_queue_gates": 0.28}, "confidence": 0.18}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.44}, "severity": {"type": "score", "score": 1.52, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.04, "1": 0.4, "2": 0.56}, "confidence": 0.28}}`
- reason: dominant_pain (choice): docs_truth (conf: 0.18)
- reason: blocks_tag_or_working_mesh (noul): 0.44
- reason: severity (score): 1.52 (conf: 0.28)

### Batch 13 [LIVE] verdict=fail supported=0.51 conf=0.43
- items: V040_T_ANDROIDTEST_COMPILE_FIX.md, V040_T_COB001_WASM_OUTBOX_DUAL_DRAIN.md, open_prs
- tokens=730 cost=0.03066 model=jev-1.13.0
- answers: `{"dominant_pain": {"type": "choice", "choice": "ci_queue_gates", "probabilities": {"device_release": 0.01, "docs_truth": 0.01, "process_dispatch": 0.09, "ci_queue_gates": 0.87, "identity_transport": 0.02}, "confidence": 0.84}, "blocks_tag_or_working_mesh": {"type": "noul", "noul": 0.51}, "severity": {"type": "score", "score": 1.62, "legend": {"0": "low hygiene", "1": "medium friction", "2": "high delivery blocker"}, "probabilities": {"0": 0.01, "1": 0.35, "2": 0.64}, "confidence": 0.43}}`
- reason: dominant_pain (choice): ci_queue_gates (conf: 0.84)
- reason: blocks_tag_or_working_mesh (noul): 0.51
- reason: severity (score): 1.62 (conf: 0.43)

## Pack: `unification`

batches=2 tokens=1394 cost=0.058548

### Batch 0 [LIVE] verdict=fail supported=0.75 conf=0.41
- items: identity_canon, routing_feed, docs_queue, delivery_status
- tokens=726 cost=0.030492 model=jev-1.13.0
- answers: `{"unification_gap": {"type": "noul", "noul": 0.75}, "primary_layer": {"type": "choice", "choice": "identity_keys", "probabilities": {"ci_artifacts": 0.0, "identity_keys": 0.69, "routing_transports": 0.16, "delivery_status": 0.14, "dispatch_docs": 0.01}, "confidence": 0.61}, "fix_class": {"type": "choice", "choice": "docs_status_fix", "probabilities": {"small_code_fix": 0.37, "live_evidence": 0.04, "tests_only": 0.04, "docs_status_fix": 0.52, "operator_ruling": 0.03}, "confidence": 0.41}}`
- reason: unification_gap (noul): 0.75
- reason: primary_layer (choice): identity_keys (conf: 0.61)
- reason: fix_class (choice): docs_status_fix (conf: 0.41)

### Batch 1 [LIVE] verdict=fail supported=0.58 conf=0.36
- items: ci_fleet, harness_jev
- tokens=668 cost=0.028056 model=jev-1.13.0
- answers: `{"unification_gap": {"type": "noul", "noul": 0.58}, "primary_layer": {"type": "choice", "choice": "ci_artifacts", "probabilities": {"dispatch_docs": 0.0, "ci_artifacts": 1.0, "delivery_status": 0.0, "routing_transports": 0.0, "identity_keys": 0.0}, "confidence": 0.99}, "fix_class": {"type": "choice", "choice": "docs_status_fix", "probabilities": {"small_code_fix": 0.15, "live_evidence": 0.02, "operator_ruling": 0.06, "tests_only": 0.29, "docs_status_fix": 0.48}, "confidence": 0.36}}`
- reason: unification_gap (noul): 0.58
- reason: primary_layer (choice): ci_artifacts (conf: 0.99)
- reason: fix_class (choice): docs_status_fix (conf: 0.36)

## Pack: `orchestration`

batches=5 tokens=4040 cost=0.16968

### Batch 0 [LIVE] verdict=fail supported=0.2 conf=0.41
- items: AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md, V040_3NODE_VALIDATION_RUNBOOK_2026-09-03.md, V040_ARCH272_ROUTING_FEED_FINDING_2026-09-03.md, V040_BEACH_JOIN_CONTINUATION_2026-09-05.md
- tokens=801 cost=0.033642 model=jev-1.13.0
- answers: `{"dispatch_ready": {"type": "noul", "noul": 0.2}, "route": {"type": "choice", "choice": "orchestrator_merge", "probabilities": {"operator_device": 0.08, "defer_post_wave": 0.18, "harness_verify": 0.17, "freebuff_impl": 0.04, "orchestrator_merge": 0.53}, "confidence": 0.41}, "process_health": {"type": "score", "score": 0.89, "legend": {"0": "broken misdispatch", "1": "stale but recoverable", "2": "clear and executable"}, "probabilities": {"0": 0.18, "1": 0.75, "2": 0.07}, "confidence": 0.63}}`
- reason: dispatch_ready (noul): 0.2
- reason: route (choice): orchestrator_merge (conf: 0.41)
- reason: process_health (score): 0.89 (conf: 0.63)

### Batch 1 [LIVE] verdict=fail supported=0.16 conf=0.25
- items: V040_BEACH_JOIN_PHASE0_1_2026-09-20.md, V040_CEO_CLEANUP_MERGE_DIRECTIVE_2026-09-03.md, V040_CTO_ARCHITECTURE_AND_3NODE_HANDOFF_2026-09-02.md, V040_CTO_HANDOFF_2026-09-01.md
- tokens=759 cost=0.031878 model=jev-1.13.0
- answers: `{"dispatch_ready": {"type": "noul", "noul": 0.16}, "route": {"type": "choice", "choice": "orchestrator_merge", "probabilities": {"orchestrator_merge": 0.4, "operator_device": 0.34, "defer_post_wave": 0.07, "freebuff_impl": 0.03, "harness_verify": 0.16}, "confidence": 0.25}, "process_health": {"type": "score", "score": 0.96, "legend": {"0": "broken misdispatch", "1": "stale but recoverable", "2": "clear and executable"}, "probabilities": {"0": 0.19, "1": 0.67, "2": 0.14}, "confidence": 0.51}}`
- reason: dispatch_ready (noul): 0.16
- reason: route (choice): orchestrator_merge (conf: 0.25)
- reason: process_health (score): 0.96 (conf: 0.51)

### Batch 2 [LIVE] verdict=fail supported=0.26 conf=0.52
- items: V040_CTO_PROMPT_2026-09-02.md, V040_CTO_TRACKING_PROTOCOL.md, V040_QUEUE_STATUS_RECONCILE_TAGPATH.md, V040_REVIEW_DISPATCH_267_272_FINAL_APPROVE_QWEN_2026-09-03.md
- tokens=728 cost=0.030576 model=jev-1.13.0
- answers: `{"dispatch_ready": {"type": "noul", "noul": 0.26}, "route": {"type": "choice", "choice": "orchestrator_merge", "probabilities": {"freebuff_impl": 0.03, "harness_verify": 0.18, "defer_post_wave": 0.01, "operator_device": 0.07, "orchestrator_merge": 0.71}, "confidence": 0.64}, "process_health": {"type": "score", "score": 1.1, "legend": {"0": "broken misdispatch", "1": "stale but recoverable", "2": "clear and executable"}, "probabilities": {"0": 0.11, "1": 0.68, "2": 0.21}, "confidence": 0.52}}`
- reason: dispatch_ready (noul): 0.26
- reason: route (choice): orchestrator_merge (conf: 0.64)
- reason: process_health (score): 1.1 (conf: 0.52)

### Batch 3 [LIVE] verdict=fail supported=0.21 conf=0.41
- items: V040_REVIEW_DISPATCH_267_FDHT_QWEN_2026-09-01.md, V040_REVIEW_DISPATCH_267_RECHECK_QWEN_2026-09-02.md, V040_REVIEW_DISPATCH_267_RECHECK_QWEN_2026-09-03.md, V040_REVIEW_DISPATCH_268_269_QWEN_2026-09-01.md
- tokens=890 cost=0.03738 model=jev-1.13.0
- answers: `{"dispatch_ready": {"type": "noul", "noul": 0.21}, "route": {"type": "choice", "choice": "harness_verify", "probabilities": {"operator_device": 0.03, "harness_verify": 0.53, "defer_post_wave": 0.02, "orchestrator_merge": 0.36, "freebuff_impl": 0.06}, "confidence": 0.41}, "process_health": {"type": "score", "score": 0.99, "legend": {"0": "broken misdispatch", "1": "stale but recoverable", "2": "clear and executable"}, "probabilities": {"0": 0.19, "1": 0.63, "2": 0.18}, "confidence": 0.46}}`
- reason: dispatch_ready (noul): 0.21
- reason: route (choice): harness_verify (conf: 0.41)
- reason: process_health (score): 0.99 (conf: 0.46)

### Batch 4 [LIVE] verdict=fail supported=0.27 conf=0.01
- items: V040_REVIEW_DISPATCH_268_270_CONFIRM_APPROVE_QWEN_2026-09-03.md, V040_REVIEW_DISPATCH_270_EPHEMERAL_QWEN_2026-09-01.md, V040_REVIEW_DISPATCH_272_ARCH_QWEN_2026-09-03.md, V040_REVIEW_DISPATCH_272_CANDIDATE_QWEN_2026-09-02.md
- tokens=862 cost=0.036204 model=jev-1.13.0
- answers: `{"dispatch_ready": {"type": "noul", "noul": 0.27}, "route": {"type": "choice", "choice": "orchestrator_merge", "probabilities": {"harness_verify": 0.32, "freebuff_impl": 0.18, "orchestrator_merge": 0.45, "operator_device": 0.03, "defer_post_wave": 0.02}, "confidence": 0.33}, "process_health": {"type": "score", "score": 1.34, "legend": {"0": "broken misdispatch", "1": "stale but recoverable", "2": "clear and executable"}, "probabilities": {"0": 0.11, "1": 0.44, "2": 0.45}, "confidence": 0.01}}`
- reason: dispatch_ready (noul): 0.27
- reason: route (choice): orchestrator_merge (conf: 0.33)
- reason: process_health (score): 1.34 (conf: 0.01)

## Pack: `historical_process`

batches=2 tokens=1258 cost=0.052836

### Batch 0 [LIVE] verdict=fail supported=0.66 conf=0.26
- items: ci_hygiene, premise_drift, freebuff_paste, rule8
- tokens=670 cost=0.02814 model=jev-1.13.0
- answers: `{"lesson_strength": {"type": "score", "score": 1.93, "legend": {"0": "weak anecdote", "1": "repeatable lesson", "2": "standing rule candidate"}, "probabilities": {"0": 0.0, "1": 0.06, "2": 0.9400000000000001}, "confidence": 0.9}, "rule_candidate": {"type": "noul", "noul": 0.66}, "theme": {"type": "choice", "choice": "premise_drift", "probabilities": {"rule8_gating": 0.14, "fleet_same_sha": 0.02, "ci_hygiene": 0.38, "freebuff_paste_cost": 0.05, "premise_drift": 0.41}, "confidence": 0.26}}`
- reason: lesson_strength (score): 1.93 (conf: 0.9)
- reason: rule_candidate (noul): 0.66
- reason: theme (choice): premise_drift (conf: 0.26)

### Batch 1 [LIVE] verdict=fail supported=0.59 conf=0.26
- items: fleet_same_sha, working_first
- tokens=588 cost=0.024696 model=jev-1.13.0
- answers: `{"lesson_strength": {"type": "score", "score": 1.4, "legend": {"0": "weak anecdote", "1": "repeatable lesson", "2": "standing rule candidate"}, "probabilities": {"0": 0.05, "1": 0.5, "2": 0.45}, "confidence": 0.26}, "rule_candidate": {"type": "noul", "noul": 0.59}, "theme": {"type": "choice", "choice": "fleet_same_sha", "probabilities": {"rule8_gating": 0.01, "premise_drift": 0.0, "freebuff_paste_cost": 0.0, "fleet_same_sha": 0.99, "ci_hygiene": 0.0}, "confidence": 0.98}}`
- reason: lesson_strength (score): 1.4 (conf: 0.26)
- reason: rule_candidate (noul): 0.59
- reason: theme (choice): fleet_same_sha (conf: 0.98)

## Insights for SCMessenger completion (orchestrator synthesis)

Mechanical harvest + JEV batches above are inputs. Final action orders stay in `HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` and `HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md`. JEV does not invent new root causes.

### Standing synthesis themes

1. Identity unification (hex) remains the first WP — dual flavor breaks send paths.
2. Routing feed must be one entry for all data links — WP2 is the open hole.
3. Delivery truth (ACK vs delivered vs receipts) is process + code — WP4.
4. WP5 live 3-node logs are mandatory before any WiFi-fixed claim.
5. Freebuff paste authority is the implementation plan DISPATCHABLE set + JEV canonical DONE.
6. Harness WIP (P2 repair / jev-phase / issue-sort) stays in Harness repo; SCMessenger consumes origin/main JEV + local packs.
