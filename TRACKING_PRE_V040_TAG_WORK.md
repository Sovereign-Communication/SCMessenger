# Tracking: Pre-v0.4.0 Tag Work

**Branch:** `tracking/pre-v040-tag-work`
**Created:** 2026-08-05
**Purpose:** Track all remaining work that must land before the v0.4.0 tag, per operator directive and five-node gate plan.

---

## Status Overview (updated 2026-08-07)

PR #139 is open and mergeable at head `5b8b8e7b`, with all 31 CI checks green
(including iOS Build & Simulator Test, macOS Native Tests, all four Android
ABIs, WASM, FFI Surface Contract, and Kotlin/Swift bindings).
The Windows implementation lane has completed T1/T2/T3/T4 and passed the
targeted formatter, blocked-manager, and CLI message-request gates.
Five-node rollout and field evidence remain in progress:
- [OK] Windows Rust/CLI implementation and targeted gates
- [OK] Android<->iOS bidirectional messaging evidence from the prior rollout
- [IN PROGRESS] AWS relay rebuild/address propagation and custody evidence
- [IN PROGRESS] iOS/macOS MAC LANE update and runtime evidence
- [PENDING] full five-node G1-G6 gate, twice reproducibly

---

## Remaining Pre-v0.4.0-Tag Work (from five-node gate plan)

### 1. Identifier-Gate Follow-ups (P1/P3/P4)
**Source:** `HANDOFF/todo/IDENTIFIER_GATE_FOLLOWUPS_2026-08-04.md`
**Origin:** Phase 0b adversarial review of PR #136 block gate
**Deadline:** Before v0.4.0 tag
**Tier:** THINK/MAX via qwenpaid

| Task | Priority | Description |
|------|----------|-------------|
| **T1 (P1)** | HIGH | **DONE in `57c5d6a4`** — dual-flavor physical block rows, symmetric unblock/list handling, regression tests, and authenticated ingress resolution. Dedicated post-fix adversarial evidence remains part of the release gate. |
| **T2 (P3)** | MEDIUM | **DONE in `57c5d6a4`** — `identity_id_from_public_key_hex` requires a valid Ed25519 curve point. |
| **T3 (P4)** | HIGH | **DONE in `57c5d6a4`** — unresolved senders are suppressed from pending requests (fail closed). |
| **T4 (P5)** | MEDIUM | **DONE in `57c5d6a4`** — callers use `BlockedManager.is_blocked_resolved` as the central flavor-resolution policy. |

---

### 2. Ledger Visibility Gap (Phase 2)
**Source:** `HANDOFF/todo/LEDGER_SHARING_ANDROID_NODE_VISIBILITY_2026-08-05.md`
**Audit:** `HANDOFF/review/LEDGER_VISIBILITY_AUDIT_QWENPAID_2026-08-05.md`
**Design:** `HANDOFF/plans/TRUST_SCOPED_LAN_DISCLOSURE_DESIGN_2026-08-05.md`
**Priority:** HIGH (field-observed parity gap)
**Class:** AUDIT-GATE (core/src/{routing,transport} + app sync path)

**Observation:** iOS sees 4 nodes + 1 headless; Android sees only Christy on same network.
**Root Cause (ranked):**
1. **Most Likely:** `exchange_response_entries` filters by `success_count > 0` AND applies `is_disclosable_multiaddr` which strips RFC1918. iOS redacts LAN neighbors per NEW-2 security doctrine. Trust escalation for paired peers missing.
2. **Contributing:** Fresh Android has `success_count == 0` for all entries; ledger exchange requires proven peers.
3. **Possible:** Identity canonicalization drops entries during ingest.
4. **UI Mismatch:** Android may read from Contacts store vs Ledger store.

**Fix Direction:** RFC1918-on-RFC1918 disclosure plus contact chaining is
implemented in `exchange_response_entries()` under the resolved D1 policy.
Default-deny for strangers is preserved; mixed-version compatibility remains
sender-side filtered.

**Status:** **IMPLEMENTED in `57c5d6a4`; pending G4 field verification.**

---

### 3. Transport BLE/LAN Hiccup Verification
**Source:** `HANDOFF/todo/TRANSPORT_BLE_LAN_HICCUP_VERIFICATION_2026-08-05.md`
**Priority:** HIGH
**Class:** AUDIT-GATE (core/src/transport/)

**Observation:** Mid-session messages stopped between Android (Pixel 6a) and iOS (Christy). Restarting iOS app resumed flow. Theory: only BLE carrying traffic; LAN/WiFi never established or failed silently.

**Required Verification:**
1. Transport-level evidence: which transport carried each message
2. Confirm LAN/WiFi establishes at all (test with BLE disabled)
3. Failure mode: when one transport dies, does node detect and failover?
4. Locate keep-alive/reconnect gap (stale socket, missed heartbeat, dial-queue not retrying)

**Acceptance Criteria:**
- Captured evidence of LAN/WiFi transport delivering messages
- Killing BLE mid-session → traffic resumes over surviving transport (no app restart)
- Same test in reverse (kill LAN path → BLE carries)

**Note:** PR #137 (transport liveness: zombie transport reconciliation + auto-reconnect) should address this. Need field verification.

---

### 4. Five-Node Gate Execution (Twice Reproducible)
**Source:** `HANDOFF/plans/V040_V050_FIVE_NODE_GATE_PLAN_2026-08-05.md` (Section "THE FIVE-NODE GATE")

| Gate | Criterion | Status |
|------|-----------|--------|
| **G1** | Pairwise bidirectional: every reachable pair exchanges messages both directions with receipts | Pending |
| **G2** | Transport coverage: LAN, BLE, internet relay (with custody proof) all deliver | Pending |
| **G3** | Delivery truth: statuses reflect receipts E2E, no false failures, outbox flushes on reconnect | Pending |
| **G4** | Fleet convergence: every node lists full fleet within bounded window; restart re-converges without re-pair | **D1 implemented; pending field verification** |
| **G5** | Liveness: disrupt network → peers auto-reconnect without app restart (fleet proof of PR #137) | Pending |
| **G6** | Provenance: all five nodes report same git stamp | Pending |

**Must run twice, reproducibly, before tag.**

---

### 5. Dependabot Vulnerabilities (Queued Cleanup)
**Status:** 7 vulnerabilities on main (3 high)
**Action:** Queued cleanup wave post-v0.4.0 tag

---

## Open Operator Decisions

| # | Decision | Blocks |
|---|----------|--------|
| **D1** | ~~Ledger Phase 2 disclosure policy~~ — **RESOLVED 2026-08-06; implemented in `57c5d6a4`** | G4 field verification |
| **D2** | v0.4.0 tag flavor (alpha.N vs stable) after gate passes twice | release artifacts |
| **D3** | Accept AWS IP drift (EIP denied by IAM) or widen policy / DDNS | unattended reconnect to cloud node |

---

## Files to Watch / Modify

### Core Changes Required
- `core/src/store/ledger_entry.rs` — `exchange_response_entries()` trust-gated disclosure
- `core/src/transport/addr_filter.rs` — `is_disclosable_multiaddr` (already strict, correct)
- `core/src/identity/keys.rs` — `identity_id_from_public_key_hex` validation (T2)
- `core/src/transport/swarm.rs` — ledger exchange handler trust context extraction
- `cli/src/server.rs` — `GetPendingMessageRequests` filter (T3)
- `core/src/store/blocked.rs` — `BlockedManager.is_blocked_resolved` (T1, T4)

### Design / Audit Docs
- `HANDOFF/review/LEDGER_VISIBILITY_AUDIT_QWENPAID_2026-08-05.md`
- `HANDOFF/plans/TRUST_SCOPED_LAN_DISCLOSURE_DESIGN_2026-08-05.md`
- `HANDOFF/todo/IDENTIFIER_GATE_FOLLOWUPS_2026-08-04.md`
- `HANDOFF/todo/LEDGER_SHARING_ANDROID_NODE_VISIBILITY_2026-08-05.md`
- `HANDOFF/todo/TRANSPORT_BLE_LAN_HICCUP_VERIFICATION_2026-08-05.md`

### Evidence / Rollout Docs
- `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` — current AWS IP: `54.226.67.101`
- `HANDOFF/gpt/GPT_GO_IOS_MACOS_POST_PR138_2026-08-05.md` — iOS/macOS in-place update packet
- `HANDOFF/plans/V040_V050_FIVE_NODE_GATE_PLAN_2026-08-05.md` — gate criteria
- `HANDOFF/gpt/GPT_MAC_PR139_TAKEOVER_2026-08-07.md` — current MAC LANE packet
- `HANDOFF/gpt/IOS_MACOS_PR139_STATUS_2026-08-07.md` — GPT-MAC result (when filed)

---

## Dispatch Order (per _QUEUE.md)

1. **Identifier-gate T1-T4** — **IMPLEMENTED in `57c5d6a4`; Windows targeted gates pass**
2. **Ledger Phase 2** — **D1 IMPLEMENTED in `57c5d6a4`; G4 field verification pending**
3. **MAC LANE update** — build/install and runtime evidence from GPT-MAC
4. **Transport BLE/LAN verification** — field test with transport evidence capture
5. **Five-node gate execution** — twice, with recorded evidence

---

## Notes for Next Session

- No unresolved GitHub review threads are present on PR #139.
- T1-T4 and the PR’s ledger implementation are landed at `57c5d6a4`.
- Windows targeted gates pass; full workspace/CI status and field evidence must
  still be recorded against the exact release candidate.
- GPT-MAC has a self-contained packet; its result file is still absent.
- G1-G6 must be executed twice before any operator-controlled tag decision.
