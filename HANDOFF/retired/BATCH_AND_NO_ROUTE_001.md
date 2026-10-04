# BATCH: AND-NO-ROUTE-001 — Add Fallback Route Candidates + Diagnostic Logging

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Node:** implementer
**Model:** `qwen3-coder-next:cloud`
**Fallback:** `glm-5.1:cloud`
**Task Type:** Feature implementation (Android/Kotlin)
**Priority:** P1
**Scope:** ~50 LOC, single file
**Tier:** TIER 2 (Cruise Control)

## Problem

Delivery attempts fail with `reason=no_route_candidates route_fallback=null ble_only=false` during outbox retry. Messages remain stuck in pending/stored state indefinitely.

**Log Evidence:**
```
delivery_attempt msg=c5cc98c5-46fd-4e26-8258-e6187d42c9f5 medium=core phase=direct outcome=failed detail=ctx=outbox_retry reason=no_route_candidates route_fallback=null ble_only=false
```

## Root Cause

`buildRoutePeerCandidates()` at `MeshRepository.kt:5215-5237` returns an empty list when:
1. `discoverRoutePeersForPublicKey()` returns no matches
2. Contact notes contain no valid routing hints
3. `cachedRoutePeerId` is null or invalid
4. The peer ID fails validation

The empty candidates list propagates to `attemptDirectSwarmDelivery()` which logs `no_route_candidates`.

## Implementation Steps

| Step | Action | LOC | Location |
|------|--------|-----|----------|
| 1 | Add diagnostic logging to `buildRoutePeerCandidates()` showing why each source failed | ~15 | MeshRepository.kt:5215 |
| 2 | Store last-known-good `routePeerId` in contact notes on successful delivery | ~10 | MeshRepository.kt:2703 |
| 3 | Add fallback to last-known-good route when fresh discovery returns empty | ~15 | MeshRepository.kt:5241 |
| 4 | Emit user-visible "Connecting..." status when `no_route_candidates` occurs | ~10 | MeshRepository.kt:3934 |

## Verification

1. Send message to peer while device is offline
2. Verify `no_route_candidates` log includes diagnostic context
3. Go online and verify last-known-good route is attempted first
4. Verify UI shows "Connecting..." status during route discovery
5. Run `cd android && ./gradlew assembleDebug -x lint --quiet`

## Build Gate

After completing, run: `cd android && ./gradlew assembleDebug -x lint --quiet`

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.