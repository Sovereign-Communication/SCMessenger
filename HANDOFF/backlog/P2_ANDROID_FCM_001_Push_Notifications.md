# P2_ANDROID_FCM_001: Firebase Cloud Messaging Push Notifications

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** BACKLOG
**Priority:** P2 — Nice-to-have, not blocking Play Store submission

## Problem
FCM push notifications are not integrated. The app currently relies on local notifications via `NotificationHelper.kt` which works when the app is in the background but not when the app is fully killed.

## Why Backlog
- Play Store does NOT require FCM for approval
- Local notifications already work for backgrounded app
- FCM integration is complex: requires Firebase project, `google-services.json`, `FirebaseMessagingService`, and server-side relay infrastructure
- SCMessenger's mesh architecture (peer-to-peer) makes FCM less useful — messages arrive via BLE/WiFi/relay without needing push

## When to Implement
- After Play Store launch
- If users report missing notifications when app is killed
