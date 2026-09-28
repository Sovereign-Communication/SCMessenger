---
worker: claude-code
severity: high
reason: review-required
esc_id: ESC-TEST0001
---

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Summary
Round-trip test of the Overseer Bridge (C). This is a synthetic escalation
to confirm the bridge forwards correctly to the overseer's Telegram DM,
writes an ACK file, and (after a reply) lands a directive in
HANDOFF/DIRECTIVES/.

## What changed
- `~/.hermes/scripts/overseer_bridge.py` — new service
- `~/Library/LaunchAgents/com.hermes.overseer-bridge.plist` — launchd entry
- `HANDOFF/ESCALATION_PROTOCOL.md` — contract for Claude Code

## Why I'm escalating
This is a test, not a real escalation. Confirming the round-trip.

## Suggested reply
REPLY WITH: APPROVE | BLOCK
