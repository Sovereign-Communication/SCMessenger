# NEXT ORCHESTRATOR — LAUNCH PROMPT

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## [START] START HERE (30 seconds)

```bash
cd /c/Users/SCM/Documents/GitHub/SCMessenger
cat HANDOFF/NEXT_ORCHESTRATOR_PROMPT.md
```

## [BLOCKER] YOUR 3 BLOCKERS (resolve in order)

### 1. Identity Canonicalization — CRITICAL
```bash
# Read the spec
cat HANDOFF/todo/QWEN_IDENTITY_CANONICALIZATION_CRITICAL.md

# Fix these files (step 1 of 5 already in HEAD via PR #135):
# - core/src/iron_core.rs          → validate recipient_id, add identity_id→public_key index
# - android/.../MeshRepository.kt  → key contacts by public_key from BLE beacon
# - iOS/.../MeshRepository.swift   → same (GPT lane)
# - Migration for existing contacts
```

### 2. Cloud Node (dynamic IP: see HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md)
```bash
# SSH via IAM (user: "no passwords")
cat infra/aws/farm-sim-manage.sh
cat infra/aws/provision-relay.sh
# Verify container = latest CI build, test dial, verify relay custody
```

### 3. iOS/macOS Fresh Install (GPT lane)
```bash
# Christy's iPhone + macOS CLI
cat HANDOFF/todo/GPT_RUN2_IOS_MACOS_FRESH_INSTALL.md
# Collect: GATT service, advertising, BLE markers, nickname propagation, 5 questions
```

## [OK] VERIFICATION STANDARD (no exceptions)
| Claim | Evidence Required |
|-------|------------------|
| Connected | `ConnectionEstablished` for intended peer/address + role |
| Delivered | Receiver decrypts unique envelope + stores exactly one history row |
| Receipt round trip | Receipt created → sender core classifies → history/outbox/UI updated |
| Recovered | After forced disconnect/restart, queued/custody drains unattended |

## [BLOCKED] DOES NOT COUNT
dial queued, transport ACK, relay custody, HTTP success, sender-side log, synthetic callback, manual redial

## [RULES] BRANCH RULES
- Windows host = authoritative build; direct commits to main OK
- Mac lane = `gpt/*` branches only; iOS/xcodebuild authority
- Qwen = Windows execution via `scripts/delegate_task.py --provider qwenpaid`
- core/crypto/transport/routing/privacy = mandatory adversarial review

## [FILES] KEY FILES
- `HANDOFF/audit/PR134_REMAINING_TASKS.md` — complete work catalog
- `HANDOFF/plans/FIVE_NODE_RUN_2_PLAN.md` — 20 directional pairs test plan
- `HANDOFF/audit/IDENTITY_HASH_VS_PUBKEY_CONFLICT.md` — root cause

---

**When all 3 blockers done** → coordinate shared UTC window → execute 20 directional pairs per `FIVE_NODE_RUN_2_PLAN.md` → produce `FIVE_NODE_RUN_2_ANALYSIS.md`