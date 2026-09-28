# Transport -- Message Flow Halted Until iOS Restart; LAN/WiFi Path Unverified

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: OPEN
Last updated: 2026-08-05
Priority: HIGH
Class: AUDIT-GATE (core/src/transport/; security review per AGENTS.md rule 8
before any fix lands)

## Observation (operator, 2026-08-05, during the live bidirectional test)

Mid-session, messages stopped flowing between the Android node (Pixel 6a)
and the iOS node (Christy). Restarting the iOS app resumed flow perfectly.
Working theory: only the BLE transport was actually carrying traffic; the
LAN/WiFi transport never established (or failed silently with no fallback
or reconnect), leaving the session BLE-only and fragile.

## Required verification

1. Transport-level evidence: which transport (BLE vs LAN/WiFi) carried each
   message during a live session, including connection state transitions
2. Confirm LAN/WiFi transport establishes at all between the two devices on
   the same network (ideally with BLE disabled/out of range to force it)
3. Failure mode: when one transport dies, does the node detect it and fail
   over / reconnect, or does it sit silent until an app restart?
4. The iOS-restart workaround implies a keep-alive / reconnect gap -- locate
   it (stale socket, missed heartbeat, dial-queue not retrying)

## Acceptance criteria

- Captured evidence of LAN/WiFi transport delivering messages (log or e2e)
- Killing BLE mid-session -> traffic resumes over the surviving transport
  within a bounded time WITHOUT any app restart
- Same test in reverse (kill LAN path -> BLE carries)
