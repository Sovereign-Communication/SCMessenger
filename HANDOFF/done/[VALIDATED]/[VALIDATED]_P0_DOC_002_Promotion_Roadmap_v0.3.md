<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: qwen3-coder:480b:cloud
# BUDGET: 1200
# token_budget: 12000

# P0_DOC_002_Promotion_Roadmap_v0.3

**Status:** VERIFIED REMAINING WORK
**Agent:** architect-planner
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 release
**Source:** planfromclaudeforhermes 2 Phase G.4
**Depends on:** ALL other v0.2.1 tasks must be DONE

---

## Verified Gap

`docs/historical/plans/PRODUCTION_ROADMAP.md` (53KB) describes v1.0 plan and lists v0.2.1 items as P0/P1. After v0.2.1-complete ships, those items must be moved to "done" and v0.3 alpha items promoted to active P0.

## Scope (~45 LoC of edits to docs/historical/plans/PRODUCTION_ROADMAP.md)

## File Targets

- `docs/historical/plans/PRODUCTION_ROADMAP.md` [EDIT  close v0.2.1 items, promote v0.3 items]

## Edits Required

### Section 1: Update Module Status Matrix

Change all ` Dormant`  ` Wired` for: Drift, Routing, Privacy.

Change all ` No wired`  ` Active` for: compression, outbox flush.

### Section 2: Mark v0.2.1 P0 items as COMPLETED

In PHASE 1 (Stability & Baseline Hardening), mark complete:
- 1.1 Build & Test Fixtures: all checkboxes 
- 1.2 Android Stability: all checkboxes 
- 1.5 Security Quick Wins: all checkboxes 
- 1.6 Verification Gate: all checkboxes 

In PHASE 2 (Core Wiring), mark complete:
- 7. Drift Protocol not wired:  RESOLVED
- 8. Mycorrhizal Routing not wired:  RESOLVED
- 9. Privacy modules dormant:  RESOLVED
- 10. Outbox flush on PeerDiscovered incomplete:  RESOLVED

### Section 3: Add PHASE 0 (v0.3 alpha) at the top

```markdown
## PHASE 0: v0.3 Alpha (Active)

**Goal:** Global-scale mesh infrastructure. Build on v0.2.1-complete foundation.

### 0.1 STUN/TURN Integration
- Add STUN client for NAT type detection
- TURN fallback for symmetric NAT
- Auto-fallback when direct + relay fail

### 0.2 Mesh Health Monitoring
- Metrics collection: peer counts, latency, message success rate
- Per-transport success rate (BLE, WiFi, QUIC, TCP)
- Connection quality scoring
- Health dashboard

### 0.3 Persistent Peer Reputation
- Move reputation from in-memory to sled-backed
- Propagate reputation via signed attestations to trusted peers
- Reputation decay over time

### 0.4 Cross-Device Message Deduplication
- Track message_id across devices in audit log
- Cross-device sync via DriftProtocol SyncSession

### 0.5 Group Messaging
- Channels (multi-party)
- Broadcast encryption
- Group membership management
- Group state sync

### 0.6 Message Search
- Inverted index over message content
- Search across devices
- Encrypted search (per-peer key)

### 0.7 Property-Based Testing
- quickcheck/proptest for crypto primitives
- Network simulation harness
- Chaos testing (random peer drops, latency injection)

### 0.8 CI Pipeline Hardening
- Required checks on `main` branch
- Branch protection rules
- Cargo audit + cargo deny automated

### 0.9 Fuzzing Harness
- cargo-fuzz for parser fuzzing
- AFL++ integration for protocol fuzzing

### 0.10 Graceful Shutdown
- Drain pending messages before exit
- Flush sled databases cleanly
- Notify connected peers of impending shutdown
```

### Section 4: Add verification link

Add a link at the top:
```markdown
**v0.2.1-complete shipped:** YYYY-MM-DD  see RELEASE_NOTES_v0.2.1.md
```

## Build Verification Commands

```bash
# Confirm file is updated
head -30 docs/historical/plans/PRODUCTION_ROADMAP.md
grep "v0.2.1-complete shipped" docs/historical/plans/PRODUCTION_ROADMAP.md
grep "PHASE 0: v0.3 Alpha" docs/historical/plans/PRODUCTION_ROADMAP.md
wc -l docs/historical/plans/PRODUCTION_ROADMAP.md
```

## Acceptance Gates

1. `docs/historical/plans/PRODUCTION_ROADMAP.md` updated
2. All 17 success criteria from `planfromclaudeforhermes` 7 marked  in the doc
3. New PHASE 0 (v0.3 alpha) added with 10 sub-items
4. Link to `RELEASE_NOTES_v0.2.1.md` present
5. Commit: `docs: v0.2.1-complete  close roadmap items, promote v0.3 alpha`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: QWEN_CODER_480B] [REQUIRES: ARCHITECT_ROLE] [DEPENDS_ON: ALL_OTHER_TASKS_DONE]
