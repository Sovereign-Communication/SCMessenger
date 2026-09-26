Status: Audit findings
Last updated: 2026-08-03

# Feature Parity Evidence Audit

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Summary

Source: `docs/FEATURE_PARITY.md` (dated 2026-07-24, 237 lines)

**Inventory:**
- Total [OK] rows in matrix: 94
- Evidence distribution (all 94 [OK] rows):
  - WIRED (code exists, callable, no independent test/device verification): 91 rows
  - TESTED (automated test referenced): 0 rows
  - DEVICE (real-device run with log/date evidence): 0 rows
  - UNKNOWN (no basis stated at all): 3 rows
- Additional [WARNING] rows: 2
- Additional [FAIL] rows: 0

**Critical finding:** Zero rows contain device-verification evidence. All [OK] claims are WIRED (function presence only) or UNKNOWN. Meanwhile, messaging does not work end-to-end between paired Android/iOS devices as of 2026-08-03.

---

## Downgrades Required

| Feature | Current | Proposed | Reason |
|---------|---------|----------|--------|
| BLE (L2CAP/GATT) - Android | [OK] | [FAIL] | GATT server unregistered 17h; 264 `mesh_ble_forward` entries with 0 `mesh_ble_forward_return`. No BLE inbound connectivity established. |
| BLE (L2CAP/GATT) - iOS | [OK] | [FAIL] | 0 `ble_central_connected`, 0 services discovered, 0 subscriptions, 0 writes. Central failed to establish any BLE connection. |
| sendMessage (all platforms) | [OK] | [FAIL] | Message delivery fails in both directions (Android→iOS and iOS→Android). Confirmed via real-device testing 2026-08-03. |
| receiveMessage (Core/CLI/WASM) | [OK] | [UNKNOWN] | Cannot receive end-to-end if sendMessage fails. No independent evidence messages traverse the stack in production. |
| resolveIdentity | [OK] | [WARN] | Conflicting identity_id and public_key keying schemes break contact resolution reliability. |
| resolveToIdentityId | [OK] | [WARN] | Same conflict as resolveIdentity. |
| markDelivered (all platforms) | [OK] | [UNKNOWN] | Delivery receipts cannot function if message transmission fails. No independent evidence. |
| Multipeer Connectivity - iOS | [OK] | [UNKNOWN] | No device evidence separating Multipeer from BLE. If BLE is primary and broken, fallback is unverified. |
| dial - Android | [OK] | [WARN] | Reports success on Dial queue, not on ConnectionEstablished. Real P2P connection not verified. |

---

## Proposed Evidence Legend

[OK-DEVICE] = Function wired + real-device run verified with log/date evidence  
[OK-TEST] = Function wired + automated test suite passes  
[OK-WIRED] = Function exists and is callable; no test or device verification  
[WARN] = Function wired but known issue blocks reliable use  
[FAIL] = Function not wired or non-functional  
[UNKNOWN] = No implementation evidence stated

---

## Rows Not Assessed

- Web UI parity section (lines 184–193): All marked [OK] with no test/device evidence. Claim is UI visual parity, not message-delivery or transport parity. Cannot downgrade without running the web client against real devices. Deferred.
- AutoAdjustEngine power-management features (mobile-only): No device evidence that battery tuning works. Deferred pending device battery profiling.
- Bootstrap and ledger managers: Assumed working based on app startup success, but no independent evidence presented. Deferred.

---

## Remediation Path

1. **Immediate (blockers):**
   - Fix Android GATT server unregistration (BLE inbound)
   - Fix iOS BLE central connection logic (services/subscriptions/writes)
   - Re-enable message send/receive with real-device verification

2. **Short-term (before next release):**
   - Migrate from identity_id/public_key conflict to single keying scheme
   - Add device-verified integration tests for each marked [OK-DEVICE]

3. **Long-term:**
   - Convert all [OK-WIRED] to [OK-TEST] or [OK-DEVICE] with evidence trail
   - Sunset the matrix if it cannot be verified via automation or scheduled real-device runs

---

**End audit. File size follows.**
