# Canonical Outlier Audit -- iteration 5 (DIM-E + DIM-F)
Task: HANDOFF/freebuff/queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md
Date: 2026-09-19
Branch: glm/canonical-outlier-audit
HEAD: 1acb63531aaa1e7c22071bb7430c72b5a9753210
Model: Buffy (Freebuff)
Mode: REPORT-ONLY

## Evidence commands

- `head -30 HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` (sanctioned pointer read;
  policy header + current address + 2026-09-14 policy-violation note).
- `grep -rn -iE "bootstrap (nodes|peers|relays)|hardcoded" docs HANDOFF core/src
  cli/src` (active-scope filter applied; output in transcript).
- `grep -rn -E "\b[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\b" core/src cli/src`
  (filtered; all hits are test literals or RFC-5737/documentary addresses).
- `grep -rln "18\.234\.62\.247" --include=*.md docs HANDOFF README.md` (full list
  captured -- 29 files outside the sanctioned pointer; complete list in this
  report, no elision).
- `grep -n "STALE|WARNING" docs/FEATURE_PARITY.md`; `head -25
  HANDOFF/freebuff/queue/V040_T10_FFI_SURFACE_GATE_PASSES_VACUOUSLY.md`.
- DIM-F: `grep -n "Uniffi\|fn " core/src/api.udl | wc -l` = 110 (surface
  inventory anchor); docs cited by path below.

## Counts

- DIM-E new findings: 3 (CO-E-001..003)
- DIM-F new findings: 3 (CO-F-001..003)

## DIM-E findings

### CO-E-001 -- The AWS address single-source policy is violated by 29 tracked files, including active todo tickets
- Dimension: DIM-E
- Severity: MED
- Target: 0.4.0
- Location: sanctioned pointer `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md`
  (policy: "IPs in this repo are ephemeral. This file is the ONE place the
  orchestrator updates immediately after every AWS node rebuild. Read it fresh
  at use time; never copy an IP from any other doc, ticket, or config.")
  vs 29 other files containing `18.234.62.247`.
- Authority contradicted: the pointer's own operator-directive policy header
  (2026-08-04); AGENTS.md DIM-E intent (hardcoded AWS addresses in active docs).
- Evidence: full grep -rln list, reproduced in full (29 files, no elision):
  1. HANDOFF/audit/CELLULAR_PATH_TRIANGULATION_2026-09-15.md
  2. HANDOFF/audit/IDENTIFIER_PARITY_AUDIT_2026-09-15.md
  3. HANDOFF/audit/MULTIDIMENSIONAL_AUDIT_2026-09-17.md
  4. HANDOFF/audit/RCA_STOP_RACE_AND_CELL_STORED_2026-09-11.md
  5. HANDOFF/CEO_STATE.md
  6. HANDOFF/freebuff/inbox/V040_3NODE_BASELINE_AND_UPDATE_PLAN_2026-09-19.md
  7. HANDOFF/freebuff/inbox/V040_REVALIDATION_GATES_DONE_2026-09-07.md
  8. HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md (the sanctioned file itself)
  9. HANDOFF/PEER_AUDIT_AND_33DA1982_CUTOVER_2026-09-11.md
  10. HANDOFF/todo/CELL_ROUTE_AWS_001_2026-09-11.md
  11. HANDOFF/todo/P1_CLI_SEND_CANONICAL_IDENTIFIER_PARITY_2026-09-16.md
  12. HANDOFF/todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md
  13. HANDOFF/V040_3NODE_DEPLOY_EVIDENCE_238a8c53_2026-09-10.md
  14. HANDOFF/V040_3NODE_REDEPLOY_E8C8F52B_2026-09-10.md
  15. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T120957Z_PREFLIGHT.md
  16. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T171520Z_NODE_READY.md
  17. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T174811Z_NODE_READY.md
  18. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260908T224800Z_BLE01_SCANNER_FIX_LOCAL.md
  19. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T001500Z_T14_GOLIVE.md
  20. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T033500Z_PARITY_PREP_E3E4.md
  21. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T053500Z_TRANSPORT_VERIFY.md
  22. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T080000Z_D1_LIVE_CUTOVER.md
  23. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T093500Z_DROPPHASE_RCA.md
  24. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T172000Z_UNIFICATION_GATES_LIVE.md
  25. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T190000Z_PR279_FULLGREEN.md
  26. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260909T225500Z_ANR_MAIN_FFI_FIX.md
  27. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260917T211913Z_NODE_READY.md
  28. HANDOFF/V040_CTO_3NODE_BLE_FINAL_HANDOFF_20260908T120957Z.md
  29. HANDOFF/V040_CTO_3NODE_BLE_CHECKPOINT_20260910T051500Z_OPERATOR_TEST_RCA_FIX.md
  (list order as returned by grep -rln; item 8 is the sanctioned file itself,
  so 28 stray copies.) Active-ticket copies: items 10, 11, 12. The pointer
  itself records (2026-09-14) that the policy was violated once already and
  the address went dead unnoticed for a day.
- Why it is an outlier: 28 stale copies guarantee the next rebuild silently
  invalidates every checkpoint doc that dials the old IP; the policy exists
  precisely to prevent this and is not enforced mechanically.
- Suggested remediation class: inventory-ticket (sweep references to pointer
  form; the dated checkpoint docs may keep their historical values if labeled)
- Status: OPEN

### CO-E-002 -- Discovery-era docs in the Active index still describe bootstrap node tiers as current design
- Dimension: DIM-E
- Severity: MED
- Target: 0.4.0
- Location: `docs/NAT_TRAVERSAL_PLAN.md:12` ("**Bootstrap Nodes** -- Well-known
  relay nodes that help peers discover each other") and `:63` ("Mobile apps
  configure bootstrap nodes from settings"); `docs/GLOBAL_ROLLOUT_PLAN.md:21`
  ("third-party relays/bootstrap nodes are both valid"); both listed as
  `Mixed` in DOCUMENT_STATUS_INDEX section 5 (usage rule: verify before use).
- Authority contradicted: AGENTS.md doctrine (bootstrap address lists are a
  deprecated transitional mechanism; discovery is ledger sharing);
  docs/BOOTSTRAP.md:16 and docs/BOOTSTRAP_GOVERNANCE.md:8 (correct doctrine).
- Evidence: grep output quoted; DOCUMENT_STATUS_INDEX section 5 rows read this
  session (lines 130-142 of that file).
- Why it is an outlier: Mixed status is the correct classification, but the
  NAT plan's opening premise predates the ledger-sharing pivot without any
  dated in-file correction; the two doctrine-correct docs and these two
  coexist without cross-reference.
- Suggested remediation class: docs-correct (dated supersession note at the
  top of each Mixed doc, T11 style)
- Status: OPEN

### CO-E-003 -- Claim "The Rust core and the CLI contain no hardcoded routable IP addresses" verified TRUE; no finding
- Dimension: DIM-E
- Severity: none (verified-consistent, recorded for the ledger)
- Location: `docs/BOOTSTRAP.md:46`
- Evidence: IP-literal grep over core/src + cli/src (filtered): all remaining
  hits are test-literal blocks (`identity_envelope.rs:224,227,254,300,301`,
  `mobile_bridge.rs:4858-4864`) or RFC-5737/documentary addresses
  (`relay/invite.rs:937` 198.51.100.7, `:1016` 6.6.6.6, `mobile_bridge.rs:1106`
  link-local 169.254.169.254 in a comment about metadata endpoints). No
  routable production IP literal in either crate.

## DIM-F findings

### CO-F-001 -- FFI surface: api.udl is the declared contract; T10 vacuous-gate defect marked RESOLVED ON MAIN but gate robustness itself not re-proven this session
- Dimension: DIM-F
- Severity: LOW
- Target: 0.5.0
- Location: `core/src/api.udl` (110 `fn`/`Uniffi`-matching lines, count from
  grep -c); `HANDOFF/freebuff/queue/V040_T10_FFI_SURFACE_GATE_PASSES_VACUOUSLY.md`
  (Status: RESOLVED ON MAIN, verified 2026-09-01, resolution note at bottom).
- Authority contradicted: none currently; recorded as parity-debt context.
- Evidence: counts and ticket header quoted; the RESOLVED claim is the
  ticket's own (2026-09-01) -- I did not re-run scripts/ffi_surface.sh against
  a generated-sources tree this session; marked UNVERIFIED rather than
  re-asserted.
- Why it is filed: iOS/macOS (0.5.0) inherits this gate; its vacuous-pass
  history belongs in the 0.5.0 ledger until the robust path is re-proven.
- Suggested remediation class: inventory-ticket (re-prove ffi_surface.sh
  fail-loud behavior when generated sources are absent)
- Status: OPEN (claim-level)

### CO-F-002 -- FEATURE_PARITY.md matrix is self-labeled stale and unre-audited: any 0.5.0 iOS work planned from it starts from false data
- Dimension: DIM-F
- Severity: MED
- Target: 0.5.0
- Location: `docs/FEATURE_PARITY.md:1` ("Status: Active -- MATRIX STALE,
  re-audit required before v0.4.0 sign-off") and `:13` ("[WARNING] Do not read
  the matrix below as current (2026-08-09)")
- Authority contradicted: none (self-labeling is the T11-endorsed pattern);
  filed as parity DEBT, not a violation.
- Evidence: grep output quoted.
- Why it is filed: the label demands a re-audit "before v0.4.0 sign-off"; no
  re-audit ticket is visible in the Freebuff queue listing (queue ls output in
  iter4 evidence) -- the demand is currently unscheduled.
- Suggested remediation class: inventory-ticket (schedule the matrix re-audit)
- Status: OPEN

### CO-F-003 -- Apple-lane PR state not queried: open iOS PRs from prior audits remain UNVERIFIED
- Dimension: DIM-F
- Severity: PROCESS
- Target: 0.5.0
- Location: task DIM-F bullet "Open Apple-lane PRs/docs referenced in prior
  audits"
- Authority contradicted: none.
- Evidence: no `gh` query was run this session for PR state (out of audit
  scope to guess; Rule 12 -- status lines carry evidence or UNVERIFIED).
  Prior-audit references available: SHADOW audit section "Active Tracking PRs
  & Implementation Status" (line 120 area, read in M1 prep); MAC LANE packet
  dir `HANDOFF/gpt/` exists (ls in DIM-E evidence).
- Why it is filed: the 0.5.0 consumer needs this list and it is not produced
  here.
- Suggested remediation class: needs-operator (orchestrator lane query)
- Status: UNVERIFIED

## Not done / UNVERIFIED

- ffi_surface.sh fail-loud re-proof (CO-F-001, above).
- Open Apple-lane PR enumeration (CO-F-003, above).
- Android-vs-CLI feature-universality sweep (docs claiming universal features):
  spot-checked via FEATURE_PARITY label only; full matrix re-audit is its own
  task and explicitly out of scope here (T11-style).

## Next iteration aim

DIM-G (process/queue/audit-history outliers) + final triage.
