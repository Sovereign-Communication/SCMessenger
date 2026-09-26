# BATCH: AND-DELIVERY-001 — Fix Delivery State Tracking (msg=unknown)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Node:** triage-router
**Model:** `gemini-3-flash-preview:cloud`
**Fallback:** `deepseek-v4-flash:cloud`
**Task Type:** Bug fix (Android/Kotlin)
**Priority:** P1
**Scope:** ~40 LOC, single file
**Tier:** TIER 2 (Cruise Control — budget-conscious model selection)

## Problem

Multiple log entries show `msg=unknown` instead of actual message IDs. Messages failing to send with "Network error". Delivery attempt count at 169 for one message.

## Root Cause

Message ID is not properly propagated through the delivery pipeline. When delivery attempts are logged, the message ID field is null/blank, falling back to the string "unknown".

## Implementation Steps

| Step | Action | LOC | Location |
|------|--------|-----|----------|
| 1 | Find where `msg=unknown` originates — trace the delivery attempt logging path in `MeshRepository.kt` | — | MeshRepository.kt:5612-5632 |
| 2 | Ensure `messageId` is passed through `logDeliveryAttempt()` from all callers | ~15 | MeshRepository.kt |
| 3 | Add null/blank guard on messageId before logging, with Timber.w warning | ~10 | MeshRepository.kt |
| 4 | Implement max retry limit (e.g., 12 attempts max before marking as failed) | ~15 | MeshRepository.kt |

## Verification

1. Run `cd android && ./gradlew assembleDebug -x lint --quiet`
2. Check that `logDeliveryAttempt` always receives a valid messageId
3. Verify messages stop retrying after 12 attempts
4. Verify Timber.w is logged when messageId is null/blank

## Build Gate

After completing, run: `cd android && ./gradlew assembleDebug -x lint --quiet`

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.