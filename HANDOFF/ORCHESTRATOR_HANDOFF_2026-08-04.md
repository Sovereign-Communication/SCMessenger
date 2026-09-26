# ORCHESTRATOR HANDOFF — 2026-08-04 5-Node Run 2 Preparation

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date**: 2026-08-04
**Status**: ACTIVE — Next orchestrator take over from here
**Baseline**: `origin/main` at `ba362cc5` (post PR #133 merge)
**Current Branch**: `fix/5node-run2-remaining` (PR #134 open), `fix/identity-canonicalization-public-key` (PR #135 open)

---

## COMPLETION GATE

**This handoff is ready when**: The identity canonicalization PR (#135) has `iron_core.rs` validation + Android contact keying fixes merged, cloud node verified, and iOS/macOS fresh install complete. Then 5-node run 2 can be scheduled.

---

## CURRENT STATE SUMMARY

### [OK] DONE
1. **PR #133 merged** — All 29 CI checks green (including iOS Build & Simulator Test)
2. **Session audit** — `HANDOFF/audit/SESSION_AUDIT_2026-08-04_CLAUDE_LAST.md` (5 "green signal from no-op" patterns, Robolectric fabrication, fmt harness bug, 6 hardening recommendations)
3. **PR #134 created** — `HANDOFF/audit/PR134_REMAINING_TASKS.md` (complete remaining work catalog)
4. **PR #135 created** — `fix/identity-canonicalization-public-key` (step 1: prefixes + validation functions, fmt/clippy/lib-tests PASS)
4. **5-Node Run 2 Plan** — `HANDOFF/plans/FIVE_NODE_RUN_2_PLAN.md` (20 directional pairs, evidence protocol, success criteria)
5. **Task specs delegated** — Qwen (identity, cloud, iOS/macOS), GPT (iOS/macOS install)

### [IN PROGRESS] IN PROGRESS (Delegated)
| Task | Owner | Status | Deliverable |
|------|-------|--------|-------------|
| Identity canonicalization (full) | Qwen | Step 1/5 merged (PR #135); `iron_core.rs` + Android `MeshRepository.kt` next | PR with validation, index, migration |
| Cloud node verification | Qwen | SSH blocked (key auth); infra scripts at `infra/aws/` | `HANDOFF/audit/CLOUD_NODE_RUN2_VERIFICATION_2026-08-04.md` |
| iOS/macOS fresh install | GPT | Delegated; Christy's iPhone + macOS CLI | `HANDOFF/gpt/IOS_MACOS_RUN2_BUNDLE_2026-08-04.md` |

### [BLOCKED] BLOCKERS FOR RUN 2
1. **Identity canonicalization incomplete** — PR #135 is step 1 of 5:
   - `core/src/iron_core.rs`: validate `recipient_id` on send path, reject known `identity_id`, add `identity_id`→`public_key` index
   - `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`: key contacts by `public_key` from BLE beacon (not `identity_id`)
   - `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`: same (GPT lane)
   - Migration for existing contacts keyed by hash
2. **Cloud node SSH access** — Need IAM user auth for 100.56.248.69
3. **iOS parity tasks not in HEAD** — U6 receipt unification, relay de-hardcode, XCTest registration

---

## DELEGATION STATUS (Subagents Running)

| Delegation ID | Task | Status |
|---------------|------|--------|
| `deleg_07f225fe` | Identity canonicalization (Qwen) | Step 1 done; needs `iron_core.rs` + Android next |
| `deleg_cda71eb9` | iOS/macOS fresh install (GPT) | Delegated; awaiting deliverable |
| `deleg_ff58dbd5` | Cloud node verification (Qwen) | SSH blocked; infra scripts located |

**Let these complete naturally** — they are background subagents with live transcripts in `~/AppData/Local/hermes/cache/delegation/live/deleg_*/task-0.log`

---

## NEXT ORCHESTRATOR ACTIONS (Priority Order)

### 1. Complete Identity Canonicalization (CRITICAL BLOCKER)
- [ ] Fix `core/src/iron_core.rs`:
  - `prepare_message_internal`: validate `recipient_id` is public key, not hash
  - Add `identity_id` → `public_key` index in `ContactManager`
  - Blocked checks: handle both forms
- [ ] Fix Android `MeshRepository.kt` `onPeerIdentityRead`: key contacts by `public_key`
- [ ] Fix iOS `MeshRepository.swift` (GPT lane): same
- [ ] Migration: detect/repair contacts where `peer_id` != `public_key`
- [ ] Run all Windows gates (fmt, clippy, lib tests, Android build)

### 2. Resolve Cloud Node Access
- [ ] SSH to 100.56.248.69 using IAM auth (user mentioned "setup by AI via IAM user")
- [ ] Verify container image = latest CI build (post PR #133)
- [ ] Test dial from Windows, verify relay custody
- [ ] Deliver `CLOUD_NODE_RUN2_VERIFICATION_2026-08-04.md`

### 3. Complete iOS/macOS Fresh Install
- [ ] GPT delivers `IOS_MACOS_RUN2_BUNDLE_2026-08-04.md` with:
  - GATT service registered, advertising active
  - BLE markers, decrypt failures with EXACT wording
  - Answers to 5 log bundle protocol questions
  - Nickname propagation verified

### 4. Schedule 5-Node Run 2
- [ ] All 5 nodes healthy + identity canonicalization merged
- [ ] Coordinate shared UTC window
- [ ] Execute directional pair matrix (20 pairs)
- [ ] Collect log bundles per `WINDOWS_LOG_BUNDLE_PROTOCOL_2026-08-03.md`
- [ ] Produce `FIVE_NODE_RUN_2_ANALYSIS.md`

---

## KEY FILES FOR NEXT ORCHESTRATOR

| File | Purpose |
|------|---------|
| `HANDOFF/audit/PR134_REMAINING_TASKS.md` | Complete remaining work catalog |
| `HANDOFF/plans/FIVE_NODE_RUN_2_PLAN.md` | Test plan with matrix, evidence protocol, success criteria |
| `HANDOFF/audit/SESSION_AUDIT_2026-08-04_CLAUDE_LAST.md` | Process hardening lessons (5 no-op patterns, 6 recommendations) |
| `HANDOFF/todo/QWEN_IDENTITY_CANONICALIZATION_CRITICAL.md` | Qwen task spec for identity fix |
| `HANDOFF/todo/QWEN_RUN2_CLOUD_NODE_VERIFICATION.md` | Qwen task spec for cloud node |
| `HANDOFF/todo/GPT_RUN2_IOS_MACOS_FRESH_INSTALL.md` | GPT task spec for iOS/macOS |
| `HANDOFF/audit/IDENTITY_HASH_VS_PUBKEY_CONFLICT.md` | Root cause analysis (must read) |
| `HANDOFF/audit/DIRECTIONAL_PARITY_DIAGNOSTIC.md` | Diagnostic method for run 2 |

---

## VERIFICATION EVIDENCE REQUIRED (Per GPT_PLANNING_040_050_VERDICT.md)

| Claim | Minimum Evidence |
|-------|------------------|
| Connected | `ConnectionEstablished` for intended peer/address + role + provenance |
| Delivered | Receiver decrypts unique envelope + durably stores exactly one history row |
| Receipt round trip | Receiver creates receipt for message ID; sender core classifies it; platform callback updates history; pending retry removed; Delivered appears only then |
| Recovered | After forced disconnect/restart, queued/custody state drains unattended without duplicates |

**Non-evidence (does NOT count)**: dial queued, discovery count, transport ACK, relay custody, HTTP success, sender-side log only, synthetic callback, manual redial

---

## BRANCH STRATEGY (AGENTS.md Compliance)

- **Windows host (FULL class)**: Authoritative build verification; direct commits to main permitted
- **Mac lane (GPT/Codex)**: `gpt/*` branches only; iOS/xcodebuild authority
- **Qwen (paid Alibaba)**: Windows execution lane via `scripts/delegate_task.py --provider qwenpaid`
- **agy/Gemini**: Bounded audit/security packets (W1-W5)
- **Never** merge to main without operator approval; never tag without all gates passing
- **All** core/crypto/transport/routing/privacy changes require adversarial review

---

## PROCESS HARDENING (From Session Audit — Implement Before 0.4.0)

1. **Mandatory `curl -I` / `wget --spider`** for every external URL in Dockerfiles/CI/dependency pins
2. **Sibling-config checklist** for multi-file changes: `Dockerfile` ↔ `docker-compose*.yml`, `Cargo.toml` ↔ `Cargo.lock`, workflows ↔ local gates
3. **Harness negative tests**: Every gate script must fail on known-bad tree before first use and after any edit
4. **Adversarial review evidence standard**: "PASS" requires file:line for EVERY finding
5. **Note drift detection**: Quarterly audit of `.claude/rules/*.md` and memory against ground truth
6. **Dependabot automation**: Enable auto-merge for LOW/MEDIUM; require manual review for HIGH/CRITICAL with SLA
7. **Release gate**: 0.4.0 = 0 HIGH Dependabot + 0 stale MILESTONE tickets + all CI green + adversarial review on file for all core/ changes
8. **Disk hygiene cron**: Weekly `find target/debug -name "*.pdb" -delete` (frees ~19 GB, no rebuild)

---

## HANDOFF COMPLETION

**Next orchestrator starts here** — pick up identity canonicalization completion (PR #135 follow-up), then cloud node, then iOS/macOS, then schedule run 2.

**Handoff prompt is live** — when identity canonicalization PR has full validation + Android contact keying merged, and cloud node verified, and iOS/macOS bundle delivered, the 5-node run 2 gate is unblocked.
<tool_call>
<function=write_file>
<parameter=content>
# Orchestrator Handoff Prompt — 2026-08-04

## For Next Orchestrator

You are taking over the 5-node run 2 preparation. PR #133 is merged. PR #134 (remaining work catalog) and PR #135 (identity canonicalization step 1) are open.

**Read these first (in order):**
1. `HANDOFF/audit/PR134_REMAINING_TASKS.md` — complete remaining work
2. `HANDOFF/plans/FIVE_NODE_RUN_2_PLAN.md` — test plan with 20 directional pairs
3. `HANDOFF/audit/SESSION_AUDIT_2026-08-04_CLAUDE_LAST.md` — process hardening lessons
4. `HANDOFF/audit/IDENTITY_HASH_VS_PUBKEY_CONFLICT.md` — root cause (CRITICAL BLOCKER)

## Current Blockers (must resolve before run 2)

1. **Identity canonicalization incomplete** (PR #135 is step 1/5):
   - Fix `core/src/iron_core.rs`: validate recipient_id on send path, reject known identity_id, add identity_id→public_key index
   - Fix Android `MeshRepository.kt` onPeerIdentityRead: key contacts by public_key from BLE beacon
   - Fix iOS `MeshRepository.swift` (GPT lane): same
   - Migration for existing contacts keyed by hash

2. **Cloud node SSH access** — 100.56.248.69 needs IAM auth (user: "setup by AI via IAM user")

3. **iOS parity tasks not in HEAD** — U6 receipt unification, relay de-hardcode, XCTest registration

## Active Delegations (let complete naturally)

| Delegation | Task | Status |
|------------|------|--------|
| `deleg_07f225fe` | Qwen: identity canonicalization (full) | Step 1 done; needs iron_core.rs + Android next |
| `deleg_cda71eb9` | GPT: iOS/macOS fresh install | Delegated; awaiting bundle |
| `deleg_ff58dbd5` | Qwen: cloud node verification | SSH blocked; infra scripts at infra/aws/ |

Live transcripts: `~/AppData/Local/hermes/cache/delegation/live/deleg_*/task-0.log`

## Your First Actions

1. **Complete identity canonicalization** — follow `HANDOFF/todo/QWEN_IDENTITY_CANONICALIZATION_CRITICAL.md`
2. **Resolve cloud node SSH** — use IAM auth, verify container = latest CI build
3. **Collect iOS/macOS bundle** — from GPT deliverable
4. **When all 3 done** → schedule 5-node run 2 per `FIVE_NODE_RUN_2_PLAN.md`

## Verification Standard (NO EXCEPTIONS)

| Claim | Required Evidence |
|-------|------------------|
| Connected | ConnectionEstablished for intended peer/address + role + provenance |
| Delivered | Receiver decrypts unique envelope + stores exactly one history row |
| Receipt round trip | Receiver creates receipt → sender core classifies → history/outbox/UI updated |
| Recovered | After forced disconnect/restart, queued/custody drains unattended |

**Does NOT count**: dial queued, transport ACK, relay custody, HTTP success, sender-side log, synthetic callback

## Branch Rules (AGENTS.md)

- Windows host = authoritative build; direct commits to main OK
- Mac lane = gpt/* branches only; iOS/xcodebuild authority
- Qwen = Windows execution via scripts/delegate_task.py --provider qwenpaid
- Never merge to main without operator approval; never tag without all gates
- core/crypto/transport/routing/privacy = mandatory adversarial review

## Process Hardening (from session audit — implement before 0.4.0)

1. Mandatory curl -I for every external URL in Dockerfiles/CI/pins
2. Sibling-config checklist for multi-file changes
3. Harness negative tests (gate must fail on known-bad tree)
4. Adversarial review: PASS requires file:line for EVERY finding
5. Quarterly note drift audit (.claude/rules/*.md vs ground truth)
6. 0.4.0 gate = 0 HIGH Dependabot + 0 stale MILESTONE + all CI green + adversarial review on file

---

**Handoff complete when**: Identity canonicalization fully merged + cloud node verified + iOS/macOS bundle delivered. Then schedule run 2.