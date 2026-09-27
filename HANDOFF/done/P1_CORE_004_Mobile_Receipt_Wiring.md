# P1_CORE_004: Mobile Delivery Receipt Wiring

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P1 (Core Functionality)
**Platform:** Android, iOS
**Status:** ALREADY IMPLEMENTED
**Verified:** 2026-04-15

## Findings

Delivery receipts are **already fully wired** on Android:

1. **Receipt generation on receive**: `onMessageReceived()` in `MeshRepository.kt` calls `sendDeliveryReceiptAsync()` for every inbound message type (text, identity_sync, history_sync, and duplicates).

2. **Receipt transmission**: `sendDeliveryReceiptAsync()` (lines 1659-1765) calls `ironCore.prepareReceipt(senderPublicKeyHex, messageId)` to create an encrypted receipt envelope, then transmits it via `attemptDirectSwarmDelivery()` with up to 6 retry attempts and exponential backoff.

3. **Receipt consumption**: When a receipt arrives at the sender's device, `onReceiptReceived()` (lines 1544-1612) handles it  marks the message as delivered via `historyManager.markDelivered()`, removes it from the pending outbox, and emits `MessageEvent.Delivered()`.

4. **Core Rust processing**: `receive_message()` (line 1602-1688) detects `MessageType::Receipt`, validates the sender/recipient, and updates history state.

5. **Deduplication**: `deliveredReceiptCache` (ConcurrentHashMap with 2-hour TTL) prevents duplicate receipt processing. `pendingReceiptSendJobs` prevents duplicate concurrent receipt sends.

6. **Delivery convergence**: The swarm layer publishes `DeliveryConvergenceMarker` via gossipsub topic `sc-receipt-convergence` for relay-forwarded messages, enabling custody cleanup.

No additional wiring is needed. The task description's claim that "receipts are generated but not properly integrated" is incorrect  the integration is complete.

## No Changes Required