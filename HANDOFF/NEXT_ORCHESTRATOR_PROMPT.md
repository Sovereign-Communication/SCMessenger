# ORCHESTRATOR HANDOFF PROMPT — Next Session Start Here

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## CONTEXT
PR #133 merged (all 29 CI green). PR #134 (remaining work) and PR #135 (identity canonicalization step 1) open. 5-node run 2 blocked on identity canonicalization completion.

## IMMEDIATE BLOCKERS (resolve in order)

### 1. Identity Canonicalization — CRITICAL (PR #135 follow-up)
**Files to fix:**
- `core/src/iron_core.rs` — `prepare_message_internal`: validate `recipient_id` is public key, reject known `identity_id`; add `identity_id`→`public_key` index in ContactManager
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` — `onPeerIdentityRead`: key contacts by `public_key` from BLE beacon (not `identity_id`)
- `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift` — same (GPT lane)
- Migration: detect/repair contacts where `peer_id` != `public_key`

**Spec:** `HANDOFF/todo/QWEN_IDENTITY_CANONICALIZATION_CRITICAL.md`
**Root cause:** `HANDOFF/audit/IDENTITY_HASH_VS_PUBKEY_CONFLICT.md`

### 2. Cloud Node (dynamic IP: see HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md)
SSH blocked (key auth). User: "setup by AI via IAM user — no passwords."
**Infra scripts:** `infra/aws/farm-sim-manage.sh`, `infra/aws/provision-relay.sh`
**Deliverable:** `HANDOFF/audit/CLOUD_NODE_RUN2_VERIFICATION_2026-08-04.md`

### 3. iOS/macOS Fresh Install (GPT lane)
Christy's iPhone + macOS CLI. Needs GATT service, advertising, BLE markers, nickname propagation.
**Spec:** `HANDOFF/todo/GPT_RUN2_IOS_MACOS_FRESH_INSTALL.md`
**Deliverable:** `HANDOFF/gpt/IOS_MACOS_RUN2_BUNDLE_2026-08-04.md`

## ACTIVE DELEGATIONS (let complete)
| ID | Task | Owner |
|----|------|-------|
| `deleg_07f225fe` | Identity canonicalization (full) | Qwen |
| `deleg_cda71eb9` | iOS/macOS fresh install | GPT |
| `deleg_ff58dbd5` | Cloud node verification | Qwen |

Transcripts: `~/AppData/Local/hermes/cache/delegation/live/deleg_*/task-0.log`

## VERIFICATION STANDARD (no exceptions)
| Claim | Required Evidence |
|-------|------------------|
| Connected | `ConnectionEstablished` for intended peer/address + role + provenance |
| Delivered | Receiver decrypts unique envelope + stores exactly one history row |
| Receipt round trip | Receiver creates receipt → sender core classifies → history/outbox/UI |
| Recovered | After forced disconnect/restart, queued/custody drains unattended |

**Does NOT count:** dial queued, transport ACK, relay custody, HTTP success, sender-side log, synthetic callback

## BRANCH RULES
- Windows host = authoritative build; direct commits to main OK (FULL class)
- Mac lane = `gpt/*` branches only; iOS/xcodebuild authority
- Qwen = Windows execution via `scripts/delegate_task.py --provider qwenpaid`
- Never merge to main without operator approval; never tag without all gates
- core/crypto/transport/routing/privacy = mandatory adversarial review

## WHEN TO SCHEDULE RUN 2
All 3 blockers resolved → coordinate shared UTC window → execute 20 directional pairs per `HANDOFF/plans/FIVE_NODE_RUN_2_PLAN.md` → produce `FIVE_NODE_RUN_2_ANALYSIS.md`

---

**Start by reading:** `HANDOFF/ORCHESTRATOR_HANDOFF_2026-08-04.md` (full context) and `HANDOFF/audit/PR134_REMAINING_TASKS.md` (work catalog)